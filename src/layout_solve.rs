use crate::*;
use crate::Axis::{X, Y};
use crate::layout_tree::*;

use bumpalo::collections::Vec as BumpVec;

const MAX_FIT_STACK_FRAC: f32 = 0.999;
const MAX_FIT_STACK_STEPS: usize = 64;
const FIT_STACK_EPSILON: f32 = 1e-7;

#[inline(always)]
fn clamp(size: f32, min: Option<f32>, max: Option<f32>) -> f32 {
    let size = max.map_or(size, |max| size.min(max));
    min.map_or(size, |min| size.max(min))
}

#[inline(always)]
fn inner(size: f32, padding: f32) -> f32 {
    (size - 2.0 * padding).max(0.0)
}

/// Measures a node's text in window fractions, at a given inner width or unwrapped.
pub(crate) type MeasureText<'a> = dyn FnMut(&InnerNode, Option<f32>) -> Xy<f32> + 'a;

impl Nodes {
    pub(crate) fn solve_layout(&mut self, window_size: Xy<f32>, measure_text: &mut MeasureText) {
        self.layout.window_size = window_size;
        self.build_dependency_graph();
        self.solve(measure_text);
    }

    fn solve(&mut self, measure_text: &mut MeasureText) {
        let mut next = 0;
        while next < self.layout.solve_queue.len() {
            let slot = self.layout.solve_queue[next];
            next += 1;
            let size = self.solve_slot(slot, measure_text);
            let k = slot_index(slot.axis, slot.kind);
            self.layout.solver_data[slot.node].solved[k] = size;
            self.layout.solver_data[slot.node].solved_mask |= 1 << k;

            let range = self.layout.solver_data[slot.node].dependents;
            for d in range[k]..range[k + 1] {
                let dependent = self.layout.dependents[d as usize];
                let count = &mut self.layout.solver_data[dependent.node].n_unsolved[slot_index(dependent.axis, dependent.kind)];
                *count -= 1;
                if *count == 0 {
                    self.layout.solve_queue.push(dependent);
                }
            }
        }
    }

    /// The solved value, if the slot exists and is already solved. A bound that isn't solved yet depends on the slot that reads it.
    #[inline(always)]
    fn solved(&self, i: NodeI, axis: Axis, kind: Kind) -> Option<f32> {
        let k = slot_index(axis, kind);
        let slots = &self.layout.solver_data[i];
        (slots.solved_mask & (1 << k) != 0).then_some(slots.solved[k])
    }

    #[inline(always)]
    fn value(&self, i: NodeI, axis: Axis, kind: Kind) -> f32 {
        self.layout.solver_data[i].solved[slot_index(axis, kind)]
    }

    #[inline(always)]
    fn pixels_to_frac(&self, pixels: f32, axis: Axis) -> f32 {
        pixels / self.layout.window_size[axis]
    }

    fn solve_slot(&mut self, slot: Slot, measure_text: &mut MeasureText) -> f32 {
        let i = slot.node;
        let axis = slot.axis;
        match slot.kind {
            // The root always covers the whole window. It isn't its own child, so this slot has no dependencies.
            Kind::Regular if i == ROOT_I => 1.0,
            Kind::Final => {
                let min = self.solved(i, axis, Kind::Min);
                let max = self.solved(i, axis, Kind::Max);
                clamp(self.value(i, axis, Kind::Regular), min, max)
            }
            Kind::EvenShare => self.solve_even_share(i, axis),
            Kind::GridCells => self.solve_grid_cells(i),
            declared => match self.declared(i, axis, declared).unwrap() {
                Size::Pixels(pixels) => self.pixels_to_frac(pixels, axis),
                Size::AspectRatio(aspect) => {
                    let other = self.value(i, axis.other(), Kind::Final);
                    let window_aspect = self.layout.window_size.x / self.layout.window_size.y;
                    match axis {
                        X => other * aspect / window_aspect,
                        Y => other * window_aspect / aspect,
                    }
                }
                Size::FitContent => self.solve_fitcontent(i, axis, measure_text),
                Size::Frac(frac) => self.available_from_parent(i, axis) * frac,
                Size::Fill if self.is_stacked(i, axis) => {
                    let parent = self.links[i].parent;
                    match self.has_even_share(parent, axis) {
                        true => self.value(parent, axis, Kind::EvenShare),
                        // A `FitContent` stack has no space of its own to give, so the child collapses and its bounds decide its size.
                        false => 0.0,
                    }
                }
                Size::Fill => self.available_from_parent(i, axis),
            },
        }
    }

    fn available_from_parent(&self, i: NodeI, axis: Axis) -> f32 {
        let parent = self.links[i].parent;
        let inner = inner(self.value(parent, axis, Kind::Final), self.padding(parent, axis));
        if self.layout.params[i].free_placement {
            return inner;
        }
        match self.layout.params[parent].children_layout {
            ChildrenLayout::Grid { spacing_x, spacing_y, .. } => {
                let spacing = self.pixels_to_frac(Xy::new(spacing_x, spacing_y)[axis], axis);
                let n = self.layout.solver_data[parent].grid_lines[axis] as f32;
                let cell = ((inner - spacing * (n - 1.0)) / n.max(1.0)).max(0.0);
                let span = self.layout.params[i].grid_span[axis].max(1) as f32;
                span * cell + (span - 1.0) * spacing
            }
            ChildrenLayout::Stack { axis: stack_axis, .. } if stack_axis == axis => {
                (inner - self.stack_gaps(parent, axis)).max(0.0)
            }
            _ => inner,
        }
    }

    fn solve_fitcontent(&mut self, i: NodeI, axis: Axis, measure_text: &mut MeasureText) -> f32 {
        let (children, gaps) = match self.layout.params[i].children_layout {
            ChildrenLayout::Grid { columns, spacing_x, spacing_y, flow } => {
                let spacing = self.pixels_to_frac(Xy::new(spacing_x, spacing_y)[axis], axis);
                let n = self.layout.solver_data[i].grid_lines[axis] as f32;
                let cell = match columns {
                    MainAxisCellSize::Width(w) if axis == flow.main_axis => self.pixels_to_frac(w, axis),
                    // Each cell is as big as the biggest child in it.
                    _ => {
                        let mut cell = 0.0f32;
                        for_each_layout_child!(self, i, child, {
                            if ! self.layout.params[child].free_placement {
                                let span = self.layout.params[child].grid_span[axis].max(1) as f32;
                                cell = cell.max((self.child_fit_size(child, axis) - (span - 1.0) * spacing) / span);
                            }
                        });
                        cell
                    }
                };
                ((n * cell + spacing * (n - 1.0)).max(0.0), 0.0)
            }
            ChildrenLayout::Stack { axis: stack_axis, .. } if stack_axis == axis => {
                (self.fit_stack_main(i, axis), self.stack_gaps(i, axis))
            }
            // The children overlap, so the node is as big as the biggest one.
            ChildrenLayout::Stack { .. } | ChildrenLayout::Free => {
                let mut biggest = 0.0f32;
                for_each_layout_child!(self, i, child, {
                    if ! self.layout.params[child].free_placement {
                        biggest = biggest.max(self.child_fit_size(child, axis));
                    }
                });
                (biggest, 0.0)
            }
        };

        let mut own_content = match self.layout.params[i].text {
            TextContent::None => 0.0,
            _ if self.text_wraps(i, axis) => {
                let width = inner(self.value(i, X, Kind::Final), self.padding(i, X));
                measure_text(&self[i], Some(width))[Y]
            }
            _ => measure_text(&self[i], None)[axis],
        };
        if let Some(image) = self.layout.params[i].image {
            own_content = own_content.max(self.pixels_to_frac(image[axis], axis));
        }

        children.max(own_content) + gaps + 2.0 * self.padding(i, axis)
    }

    /// The size of the children of a `FitContent` stack along the stack's axis, without the gaps. `Frac` children make the size depend on itself, so this iterates to a fixed point.
    fn fit_stack_main(&self, i: NodeI, axis: Axis) -> f32 {
        let mut estimate = 0.0f32;
        for _ in 0..MAX_FIT_STACK_STEPS {
            let mut fixed = 0.0f32;
            let mut free_frac = 0.0f32;
            let mut has_frac = false;
            for_each_layout_child!(self, i, child, {
                if self.layout.params[child].free_placement {
                    // Free children aren't part of the stack.
                } else if let Size::Frac(frac) = self.layout.params[child].size[axis] {
                    has_frac = true;
                    let wanted = estimate * frac;
                    let allowed = clamp(wanted, self.solved(child, axis, Kind::Min), self.solved(child, axis, Kind::Max));
                    match allowed == wanted {
                        true => free_frac += frac,
                        false => fixed += allowed,
                    }
                } else {
                    fixed += self.child_fit_size(child, axis);
                }
            });

            // Without `Frac` children, nothing depends on the estimate, so the first result is exact.
            if ! has_frac {
                return fixed;
            }

            let next = match free_frac <= MAX_FIT_STACK_FRAC {
                true => fixed / (1.0 - free_frac),
                false => fixed,
            };
            let settled = (next - estimate).abs() <= FIT_STACK_EPSILON;
            estimate = next;
            if settled {
                break;
            }
        }
        estimate
    }

    /// The size of a child that a `FitContent` parent can fit to. A flex child depends on the parent, so the parent can only use its other bounds.
    #[inline(always)]
    fn child_fit_size(&self, child: NodeI, axis: Axis) -> f32 {
        match not_flex(self.layout.params[child].size[axis]) {
            true => self.value(child, axis, Kind::Final),
            false => clamp(0.0, self.solved(child, axis, Kind::Min), self.solved(child, axis, Kind::Max)),
        }
    }

    /// The share of each `Fill` child in a stack. Children that their bounds stop at a different size are frozen at that size, and the others share what is left.
    fn solve_even_share(&self, i: NodeI, axis: Axis) -> f32 {
        /// What the even share of a stack needs to know about one of its `Fill` children.
        #[derive(Debug)]
        struct FillChild {
            /// The child's regular size, if it isn't `Fill`.
            content: Option<f32>,
            min: Option<f32>,
            max: Option<f32>,
            /// True if the child's `Fill` is its `max_size`. Then the share can only shrink the content. Else it can only grow it.
            capped_by_content: bool,
            frozen: bool,
        }

        impl FillChild {
            #[inline(always)]
            fn size(&self, share: f32) -> f32 {
                let raw = match self.content {
                    Some(content) if self.capped_by_content => content.min(share),
                    Some(content) => content.max(share),
                    None => share,
                };
                clamp(raw, self.min, self.max)
            }
        }

        let own = inner(self.value(i, axis, Kind::Final), self.padding(i, axis));
        let mut budget = (own - self.stack_gaps(i, axis)).max(0.0);

        with_arena(|arena| {
            let mut fills = BumpVec::new_in(arena);
            for_each_layout_child!(self, i, child, {
                let node = &self.layout.params[child];
                if node.free_placement {
                    // Free children don't take part in the share.
                } else if self.any_size_is_fill(child, axis) {
                    let content = not_fill(node.size[axis]).then(|| self.value(child, axis, Kind::Regular));
                    fills.push(FillChild {
                        content,
                        min: self.solved(child, axis, Kind::Min),
                        max: self.solved(child, axis, Kind::Max),
                        capped_by_content: content.is_some() && node.max_size[axis] == Some(Size::Fill),
                        frozen: false,
                    });
                } else {
                    budget -= self.value(child, axis, Kind::Final);
                }
            });

            let mut n_frozen = 0;
            let mut frozen_total = 0.0;
            let mut share = budget / fills.len().max(1) as f32;

            while n_frozen < fills.len() {
                share = (budget - frozen_total) / (fills.len() - n_frozen) as f32;

                let mut total_violation = 0.0;
                for child in fills.iter() {
                    if ! child.frozen {
                        total_violation += child.size(share) - share;
                    }
                }
                if total_violation.abs() < 1e-9 {
                    break;
                }

                let mut froze_any = false;
                for child in fills.iter_mut() {
                    let size = child.size(share);
                    let stuck = match total_violation > 0.0 {
                        true => size - share > 1e-9,
                        false => size - share < -1e-9,
                    };
                    if ! child.frozen && stuck {
                        child.frozen = true;
                        n_frozen += 1;
                        frozen_total += size;
                        froze_any = true;
                    }
                }
                if ! froze_any {
                    break;
                }
            }


            share
        })
    }

    /// Assigns the grid's children to cells. Returns the number of cells along the main axis.
    fn solve_grid_cells(&mut self, i: NodeI) -> f32 {
        let ChildrenLayout::Grid { columns, spacing_x, spacing_y, flow } = self.layout.params[i].children_layout else {
            unreachable!();
        };
        let main = flow.main_axis;
        let n_main = match columns {
            MainAxisCellSize::Count(n) => (n as usize).max(1),
            MainAxisCellSize::Width(w) => {
                let spacing = self.pixels_to_frac(Xy::new(spacing_x, spacing_y)[main], main);
                let width = self.pixels_to_frac(w, main);
                let inner_main = inner(self.value(i, main, Kind::Final), self.padding(i, main));
                ((inner_main + spacing) / (width + spacing)).floor().max(1.0) as usize
            }
        };
        self.assign_grid_cells(i, n_main, flow);
        n_main as f32
    }

    fn assign_grid_cells(&mut self, i: NodeI, n_main: usize, flow: GridFlow) {
        // Each cell is `occupied[line * n_main + pos]`, where a line goes along the main axis.
        let n_lines = with_arena(|arena| {
            let mut occupied = BumpVec::new_in(arena);
            let mut cursor_line = 0;

            for_each_layout_child!(self, i, child, {
                if ! self.layout.params[child].free_placement {
                    let span = self.layout.params[child].grid_span;
                    let span_pos = (span[flow.main_axis].max(1) as usize).min(n_main);
                    let span_line = span[flow.main_axis.other()].max(1) as usize;

                    let mut line = if flow.backfill { 0 } else { cursor_line };
                    let pos = 'search: loop {
                        for pos in 0..=n_main - span_pos {
                            if grid_area_is_free(&occupied, n_main, line, pos, span_line, span_pos) {
                                break 'search pos;
                            }
                        }
                        line += 1;
                    };

                    let needed = (line + span_line) * n_main;
                    if occupied.len() < needed {
                        occupied.resize(needed, false);
                    }
                    for l in line..line + span_line {
                        occupied[l * n_main + pos..l * n_main + pos + span_pos].fill(true);
                    }
                    if ! flow.backfill {
                        cursor_line = line;
                    }

                    let mut cell = Xy::new(0, 0);
                    cell[flow.main_axis] = pos as u16;
                    cell[flow.main_axis.other()] = line as u16;
                    self.layout.solver_data[child].grid_cell = cell;
                }
            });

            occupied.len() / n_main
        });

        let mut lines = Xy::new(0, 0);
        lines[flow.main_axis] = n_main as u16;
        lines[flow.main_axis.other()] = n_lines as u16;
        self.layout.solver_data[i].grid_lines = lines;

        // Reversed directions mirror the cells after they are placed.
        for axis in [X, Y] {
            let reversed = match axis {
                X => flow.x_fill_direction == Direction::RightToLeft,
                Y => flow.y_fill_direction == Direction::RightToLeft,
            };
            if reversed {
                for_each_layout_child!(self, i, child, {
                    if ! self.layout.params[child].free_placement {
                        let span = self.layout.params[child].grid_span[axis].max(1);
                        let cell = &mut self.layout.solver_data[child].grid_cell[axis];
                        *cell = lines[axis] - span - *cell;
                    }
                });
            }
        }
    }

    #[inline(always)]
    fn padding(&self, i: NodeI, axis: Axis) -> f32 {
        self.pixels_to_frac(self.layout.params[i].padding[axis], axis)
    }

    #[inline(always)]
    fn stack_gaps(&self, i: NodeI, axis: Axis) -> f32 {
        let ChildrenLayout::Stack { axis: stack_axis, spacing, .. } = self.layout.params[i].children_layout else {
            return 0.0;
        };
        if stack_axis != axis {
            return 0.0;
        }
        let n = self.layout.solver_data[i].n_placed_children as f32;
        self.pixels_to_frac(spacing, axis) * (n - 1.0).max(0.0)
    }
}

fn grid_area_is_free(occupied: &[bool], n_main: usize, line: usize, pos: usize, span_line: usize, span_pos: usize) -> bool {
    for l in line..line + span_line {
        for p in pos..pos + span_pos {
            if occupied.get(l * n_main + p) == Some(&true) {
                return false;
            }
        }
    }
    true
}

impl Ui {
    pub(crate) fn layout_calculate_sizes(&mut self) {
        let window_size = self.sys.logical_size();
        let System { nodes, renderer, size, .. } = &mut self.sys;
        nodes.solve_layout(window_size, &mut |node, width| {
            measure_text(&mut renderer.text, node.text_i.as_ref().unwrap(), width, *size)
        });
        self.write_sizes_recursive(ROOT_I);
    }

    fn write_sizes_recursive(&mut self, i: NodeI) {
        let layout = &self.sys.nodes.layout;
        let size = layout.size(i);
        let cell = layout.grid_cell(i);
        let lines = layout.grid_lines(i);
        let node = &mut self.sys.nodes[i];
        node.l2_solved[X][SizeType::Final as usize] = Some(size.x);
        node.l2_solved[Y][SizeType::Final as usize] = Some(size.y);
        node.grid_element_column_i = cell.x;
        node.grid_element_row_i = cell.y;
        node.grid_n_columns = lines.x;
        node.grid_n_rows = lines.y;

        for_each_child!(self, i, child, {
            self.write_sizes_recursive(child);
        });
    }
}

const TEXT_WIDTH_TOLERANCE: f32 = 0.05;

/// Measures a text in window fractions, at a given width or unwrapped.
fn measure_text(text: &mut keru_draw::Text, text_i: &TextI, width: Option<f32>, window: Xy<f32>) -> Xy<f32> {
    match text_i {
        TextI::TextBox(handle) => {
            let text_box = text.get_text_box_mut(handle);

            let widths = text_box.content_widths();
            let unwrapped = widths.max.max(widths.min);
            let width = match width {
                Some(width) => (width * window[X]).clamp(widths.min, unwrapped),
                None => unwrapped,
            };

            text_box.set_width(width);
            let height = text_box.layout().height();

            Xy::new((width + TEXT_WIDTH_TOLERANCE) / window[X], height / window[Y])
        }
        TextI::TextEdit(handle) => {
            let text_edit = text.get_text_edit_mut(handle);
            let height = if text_edit.single_line() {
                match text_edit.layout().lines().next() {
                    Some(first_line) => first_line.metrics().line_height,
                    // In this empty case, we could probably get the height by looking at the metrics default font, but it's kind of complicated.
                    None => 0.0, 
                }
            } else {
                0.0
            };
            // A text edit can be scrolled, so it never asks for any width.
            Xy::new(0.0, height / window[Y])
        }
    }
}
