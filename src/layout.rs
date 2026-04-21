use glam::vec2;

use crate::*;
use crate::inner_node::*;

use bumpalo::collections::Vec as BumpVec;

pub(crate) const BIG_FLOAT: f32 = 100000.0;

struct GridOccupancy<'a> {
    cells: BumpVec<'a, bool>,
    n_per_line: usize,
    n_lines: usize,
    cursor_line: usize,
}

impl<'a> GridOccupancy<'a> {
    fn new(n_per_line: usize, arena: &'a bumpalo::Bump) -> Self {
        Self { cells: BumpVec::new_in(arena), n_per_line, n_lines: 0, cursor_line: 0 }
    }

    fn is_free(&self, line: usize, pos: usize, span_line: usize, span_pos: usize) -> bool {
        for l in line..line + span_line {
            if l >= self.n_lines { continue; } // unallocated lines are free
            for p in pos..pos + span_pos {
                if self.cells[l * self.n_per_line + p] { return false; }
            }
        }
        true
    }

    fn occupy(&mut self, line: usize, pos: usize, span_line: usize, span_pos: usize) {
        let needed = line + span_line;
        if self.n_lines < needed {
            self.cells.resize(needed * self.n_per_line, false);
            self.n_lines = needed;
        }
        for l in line..line + span_line {
            for p in pos..pos + span_pos {
                self.cells[l * self.n_per_line + p] = true;
            }
        }
    }

    /// Find the first free rectangle of size (span_line x span_pos), occupy it, and return its (line, pos).
    /// If `backfill` is true, search from the beginning (dense, fills gaps). Otherwise search from the cursor.
    fn place_next(&mut self, span_line: usize, span_pos: usize, backfill: bool) -> (usize, usize) {
        let span_pos = span_pos.min(self.n_per_line).max(1);
        let span_line = span_line.max(1);
        let mut line = if backfill { 0 } else { self.cursor_line };
        loop {
            for pos in 0..=self.n_per_line - span_pos {
                if self.is_free(line, pos, span_line, span_pos) {
                    self.occupy(line, pos, span_line, span_pos);
                    if !backfill {
                        self.cursor_line = line;
                    }
                    return (line, pos);
                }
            }
            line += 1;
        }
    }
}

/// Convert (col_span, row_span) to occupancy (span_line, span_pos) based on main_axis.
/// X-major: line=row, pos=col.  Y-major: line=col, pos=row.
fn to_occ_spans(col_span: usize, row_span: usize, flow: GridFlow) -> (usize, usize) {
    match flow.main_axis {
        Axis::X => (row_span, col_span),
        Axis::Y => (col_span, row_span),
    }
}

/// Convert occupancy (line, pos) to (logical_col, logical_row).
fn from_occ(line: usize, pos: usize, flow: GridFlow) -> (usize, usize) {
    match flow.main_axis {
        Axis::X => (pos, line),
        Axis::Y => (line, pos),
    }
}

/// Apply flow reversal: convert logical (col, row) to actual (col, row) for placement.
fn apply_reversal(logical_col: usize, logical_row: usize, col_span: usize, row_span: usize, n_cols: usize, n_rows: usize, flow: GridFlow) -> (usize, usize) {
    let col = if flow.x_fill_direction == Direction::RightToLeft { n_cols - col_span - logical_col } else { logical_col };
    let row = if flow.y_fill_direction == Direction::RightToLeft { n_rows - row_span - logical_row } else { logical_row };
    (col, row)
}

/// Iterate on the children linked list.
#[macro_export]
#[doc(hidden)] // Ideally these wouldn't even be public
macro_rules! for_each_child {
    ($ui:expr, $start:expr, $child:ident, $body:block) => {
        {
            let mut current_child = $start.first_child;
            while let Some($child) = current_child {
                if ! $ui.sys.nodes[$child].exiting {
                    $body
                }
                current_child = $ui.sys.nodes[$child].next_sibling;
            }
        }
    };
}

/// Iterate on the children linked list.
#[macro_export]
#[doc(hidden)] // Ideally these wouldn't even be public
macro_rules! for_each_child_including_lingering {
    ($ui:expr, $start:expr, $child:ident, $body:block) => {
        {
            let mut current_child = $start.first_child;
            while let Some($child) = current_child {
                $body
                current_child = $ui.sys.nodes[$child].next_sibling;
            }
        }
    };
}

/// Iterate on the children linked list.
#[macro_export]
#[doc(hidden)] // Ideally these wouldn't even be public
macro_rules! for_each_child_including_lingering_reverse {
    ($ui:expr, $start:expr, $child:ident, $body:block) => {
        {
            let mut current_child = $start.last_child;
            while let Some($child) = current_child {
                $body
                current_child = $ui.sys.nodes[$child].prev_sibling;
            }
        }
    };
}

/// Iterate on the linked list of hidden children
#[macro_export]
#[doc(hidden)]
macro_rules! for_each_hidden_child {
    ($ui:expr, $start:expr, $child:ident, $body:block) => {
        {
            let mut current_child = $start.first_hidden_child;
            while let Some($child) = current_child {
                $body
                current_child = $ui.sys.nodes[$child].next_hidden_sibling;
            }
        }
    };
}

impl Ui {
    pub(crate) fn relayout(&mut self) {
        let partial_relayouts = ! self.sys.changes.partial_relayouts.is_empty();
        let full_relayout = self.sys.changes.full_relayout;
        let text_changed = self.sys.changes.text_changed;
        let nothing_to_do = !partial_relayouts && !full_relayout && !text_changed && !self.sys.changes.unfinished_animations;
        if nothing_to_do {
            return;
        }

        // if anything happened at all, we'll need to rerender.
        self.sys.changes.need_gpu_rect_update = true;
        self.sys.changes.need_rerender = true;

        // todo: bring back partial relayouts
        self.relayout_from_root();
        // if full_relayout {
        //     self.relayout_from_root();
        // } else {
        //     self.do_partial_relayouts();
        // }

        self.rebuild_render_data();

        self.sys.changes.reset_layout_changes();

        // after doing a relayout, we might be moving the hovered node away from the cursor.
        // So we run resolve_hover again, possibly causing another relayout next frame
        self.resolve_hover();
    }

    // this gets called even when zero relayouts are needed. in that case it just does nothing. I guess it's to make the layout() logic more readable
    pub(crate) fn _do_partial_relayouts(&mut self) {
        // sort by depth
        // todo: there was something about it being close to already sorted, except in reverse
        // the plan was to sort it in reverse and then use it in reverse
        self.sys.changes.partial_relayouts.sort();
        self.sys.partial_relayout_count = 0;

        for i in 0..self.sys.changes.partial_relayouts.len() {
            // in partial_relayout(), we will check for overlaps.
            // todo: if that works as expected, maybe we can skip the limit/full relayout thing, or at least raise the limit by a lot.
            let relayout = self.sys.changes.partial_relayouts[i];
            
            self._partial_relayout(relayout.i);
        }

        if self.sys.partial_relayout_count != 0 {
            let nodes = if self.sys.partial_relayout_count == 1 { "node" } else { "nodes" };
            log::info!("Partial relayout ({:?} {nodes})", self.sys.partial_relayout_count);
        }

        self.sys.partial_relayout_count = 0;
        self.sys.changes.partial_relayouts.clear();
    }

    pub(crate) fn relayout_from_root(&mut self) {
        log::info!("Full relayout");

        // 1st recursive tree traversal: start from the root and recursively determine the size of all nodes
        let starting_proposed_size = Xy::new(1.0, 1.0);

        self.recursive_determine_size_and_hidden(ROOT_I, ProposedSizes::container(starting_proposed_size), false);
        
        // 2nd recursive tree traversal: now that all nodes have a calculated size, place them.
        // we don't do update_rects here because the first frame you can't update... but maybe just special-case the first frame, then should be faster
        self.recursive_place_children(ROOT_I);
        
        self.sys.nodes[ROOT_I].last_layout_frame = self.sys.current_frame;

    }

    pub(crate) fn _partial_relayout(&mut self, i: NodeI) {
        // if the node has already been layouted on the current frame, stop immediately, and don't even recurse.
        // when doing partial layouts, this avoids overlap, but it means that we have to sort the partial relayouts cleanly from least depth to highest depth in order to get it right. This is done in `relayout()`.
        let current_frame = self.sys.current_frame;
        if self.sys.nodes[i].last_layout_frame >= current_frame {
            return;
        }

        // 1st recursive tree traversal: start from the root and recursively determine the size of all nodes
        // For the first node, use the proposed size that we got from the parent last frame.
        let starting_proposed_size = self.sys.nodes[i].last_proposed_sizes;
        let hidden_branch = if i == ROOT_I {
            false
        } else {
            match self.sys.nodes[self.sys.nodes[i].parent].params.children_can_hide {
                ChildrenCanHide::Yes => true,
                ChildrenCanHide::No => false,
                ChildrenCanHide::Inherit => false, // This should be determined by traversing up, but for partial relayout we simplify
            }
        };
        self.recursive_determine_size_and_hidden(i, starting_proposed_size, hidden_branch);
        
        // 2nd recursive tree traversal: now that all nodes have a calculated size, place them.

        self.recursive_place_children(i);

        self.sys.nodes[i].last_layout_frame = self.sys.current_frame;
    }


    fn get_size(
        &mut self,
        i: NodeI,    
        child_proposed_size: Xy<f32>, // the size that was proposed to us specifically after dividing between children
        whole_parent_proposed_size: Xy<f32>, // the whole size that the parent proposed to ALL its children collectively
    ) -> Xy<f32> {
        let mut size = child_proposed_size; // this default value is mostly useless

        for axis in [X, Y] {
            match self.sys.nodes[i].params.layout.size[axis] {
                Size::FitContent => {
                    size[axis] = child_proposed_size[axis];
                }, // propose the whole available size. We will shrink our final size later if they end up using less or more 
                Size::Fill => {
                    size[axis] = child_proposed_size[axis]; // use the whole available size
                },
                Size::Pixels(pixels) => {
                    size[axis] = self.pixels_to_frac(pixels, axis); // ignore the proposed size and force our pixel size
                },
                Size::Frac(frac) => {
                    size[axis] = whole_parent_proposed_size[axis] * frac;
                }
                Size::AspectRatio(_aspect) => {} // do nothing
            }
        }

        // apply AspectRatio
        for axis in [X, Y] {
            if let Size::AspectRatio(aspect) = self.sys.nodes[i].params.layout.size[axis] {
                match self.sys.nodes[i].params.layout.size[axis.other()] {
                    Size::AspectRatio(_second_aspect) => {
                        log::warn!("A Size shouldn't be AspectRatio in both dimensions. (node: {})", self.node_debug_name(i));
                    }
                    _ => {
                        let window_aspect = self.sys.size.x / self.sys.size.y;
                        let mult = match axis {
                            X => 1.0 / (window_aspect * aspect),
                            Y => window_aspect * aspect,
                        };
                        size[axis] = size[axis.other()] * mult;
                    }
                }
            }
        }

        return size;
    }

    fn get_inner_size(&mut self, i: NodeI, size: Xy<f32>) -> Xy<f32> {
        let mut inner_size = size;

        // remove padding
        let padding = self.pixels_to_frac2(self.sys.nodes[i].params.layout.padding);
        for axis in [X, Y] {
            inner_size[axis] -= 2.0 * padding[axis];
        }

        // remove stack spacing
        if let ChildrenLayout::Stack { axis, spacing, .. } = self.sys.nodes[i].params.children_layout {
            let n_children = self.sys.nodes[i].n_children as f32;
            let spacing = self.pixels_to_frac(spacing, axis);

            if n_children > 1.5 {
                inner_size[axis] -= spacing * (n_children - 1.0);
            }
        }

        return inner_size;
    }

    fn recursive_determine_size_and_hidden(
        &mut self,
        i: NodeI,
        proposed_sizes: ProposedSizes,
        hideable_branch: bool,
    ) -> Xy<f32> {
        self.sys.nodes[i].last_proposed_sizes = proposed_sizes;
        
        // Set can_hide flag based on parent's children_can_hide setting
        self.sys.nodes[i].can_hide = hideable_branch;
        
        // Determine this node's children_can_hide setting for its children
        let children_can_hide = match self.sys.nodes[i].params.children_can_hide {
            ChildrenCanHide::Yes => true,
            ChildrenCanHide::No => false,
            ChildrenCanHide::Inherit => hideable_branch,
        };

        let size = self.get_size(i, proposed_sizes.to_this_child, proposed_sizes.to_all_children);
        let size_to_propose = self.get_inner_size(i, size);

        let children_layout = self.sys.nodes[i].params.children_layout;
        let padding = self.pixels_to_frac2(self.sys.nodes[i].params.layout.padding);
        let mut content_size = Xy::new(0.0, 0.0);

        match children_layout {
            ChildrenLayout::Free => {
                for_each_child!(self, self.sys.nodes[i], child, {
                    let child_size = self.recursive_determine_size_and_hidden(child, ProposedSizes::container(size_to_propose), children_can_hide);
                    content_size.update_for_child(child_size, None);
                });
    
                // Propose the whole size_to_propose to the contents, and let them decide.
                if self.sys.nodes[i].text_i.is_some() {
                    let text_size = self.determine_text_size(i, size_to_propose);
                    content_size.update_for_content(text_size);
                }
                if self.sys.nodes[i].imageref.is_some() {
                    let image_size = self.determine_image_size(i, size_to_propose);
                    content_size.update_for_content(image_size);
                }
            },
            ChildrenLayout::Stack { axis, spacing, arrange: _ } => {
                let spacing = self.pixels_to_frac(spacing, axis);

                let mut available_size_left = size_to_propose;
                let mut n_added_children = 0;
                let mut n_fill_children = 0;
                // First, do all non-Fill children (free_placement children are recursed but excluded from the stack flow)
                for_each_child!(self, self.sys.nodes[i], child, {
                    if self.sys.nodes[child].params.free_placement {
                        self.recursive_determine_size_and_hidden(child, ProposedSizes::container(size_to_propose), children_can_hide);
                    } else if self.sys.nodes[child].params.layout.size[axis] != Size::Fill {
                        let child_size = self.recursive_determine_size_and_hidden(child, ProposedSizes::stack(available_size_left, size_to_propose), children_can_hide);
                        content_size.update_for_child(child_size, Some(axis));
                        if n_added_children != 0 {
                            content_size[axis] += spacing;
                        }
                        available_size_left[axis] -= child_size[axis];
                        n_added_children += 1;
                    } else {
                        n_fill_children += 1;
                    }
                });

                if n_fill_children > 0 {
                    // then, divide the remaining space between the Fill children
                    let mut size_per_child = available_size_left;
                    if n_fill_children > 1 {
                        available_size_left[axis] -= ((n_fill_children - 1) as f32) * spacing;
                    }

                    size_per_child[axis] /= n_fill_children as f32;
                    for_each_child!(self, self.sys.nodes[i], child, {
                        if !self.sys.nodes[child].params.free_placement && self.sys.nodes[child].params.layout.size[axis] == Size::Fill {
                            let child_size = self.recursive_determine_size_and_hidden(child, ProposedSizes::stack(size_per_child, size_to_propose), children_can_hide);
                            content_size.update_for_child(child_size, Some(axis));
                            if n_added_children != 0 {
                                content_size[axis] += spacing;
                            }
                            available_size_left[axis] -= child_size[axis];
                            n_added_children += 1;
                        }
                    });
                }
            },
            ChildrenLayout::Grid { columns, spacing_x, spacing_y, flow } => {
                let n = self.sys.nodes[i].n_children as usize;
                if n > 0 {
                    content_size = with_arena(|arena| {
                        let spacing_x_frac_pre = self.pixels_to_frac(spacing_x, X);
                        let spacing_y_frac_pre = self.pixels_to_frac(spacing_y, Y);
                        let n_columns = match columns {
                            MainAxisCellSize::Count(n) => (n as usize).max(1),
                            MainAxisCellSize::Width(w) => match flow.main_axis {
                                Axis::X => {
                                    let w_frac = self.pixels_to_frac(w, X);
                                    ((size_to_propose.x + spacing_x_frac_pre) / (w_frac + spacing_x_frac_pre)).floor().max(1.0) as usize
                                }
                                Axis::Y => {
                                    let h_frac = self.pixels_to_frac(w, Y);
                                    ((size_to_propose.y + spacing_y_frac_pre) / (h_frac + spacing_y_frac_pre)).floor().max(1.0) as usize
                                }
                            },
                        };

                        let mut occ = GridOccupancy::new(n_columns, arena);
                        for_each_child!(self, self.sys.nodes[i], child, {
                            if !self.sys.nodes[child].params.free_placement {
                                let col_span = (self.sys.nodes[child].params.grid_element.column_span as usize).max(1);
                                let row_span = (self.sys.nodes[child].params.grid_element.row_span as usize).max(1);
                                let (span_line, span_pos) = to_occ_spans(col_span, row_span, flow);
                                occ.place_next(span_line, span_pos, flow.backfill);
                            }
                        });
                        let n_cross = occ.n_lines;

                        let (n_cols, n_rows) = match flow.main_axis {
                            Axis::X => (n_columns, n_cross),
                            Axis::Y => (n_cross, n_columns),
                        };

                        self.sys.nodes[i].grid_n_columns = n_cols as u16;
                        self.sys.nodes[i].grid_n_rows = n_rows as u16;

                        let spacing_x_frac = spacing_x_frac_pre;
                        let spacing_y_frac = spacing_y_frac_pre;

                        let cell_w = ((size_to_propose.x - spacing_x_frac * (n_cols as f32 - 1.0)) / n_cols as f32).max(0.0);
                        let cell_h = ((size_to_propose.y - spacing_y_frac * (n_rows as f32 - 1.0)) / n_rows as f32).max(0.0);

                        let mut row_heights = BumpVec::new_in(arena);
                        row_heights.resize(n_rows, 0.0f32);
                        let mut occ = GridOccupancy::new(n_columns, arena);
                        for_each_child!(self, self.sys.nodes[i], child, {
                            if self.sys.nodes[child].params.free_placement {
                                self.recursive_determine_size_and_hidden(child, ProposedSizes::container(size_to_propose), children_can_hide);
                            } else {
                                let col_span = (self.sys.nodes[child].params.grid_element.column_span as usize).max(1);
                                let row_span = (self.sys.nodes[child].params.grid_element.row_span as usize).max(1);
                                let (span_line, span_pos) = to_occ_spans(col_span, row_span, flow);
                                let (occ_line, occ_pos) = occ.place_next(span_line, span_pos, flow.backfill);
                                let (logical_col, logical_row) = from_occ(occ_line, occ_pos, flow);
                                let (actual_col, actual_row) = apply_reversal(logical_col, logical_row, col_span, row_span, n_cols, n_rows, flow);

                                self.sys.nodes[child].grid_element_column_i = actual_col as u16;
                                self.sys.nodes[child].grid_element_row_i = actual_row as u16;

                                let child_cell_size = Xy::new(
                                    col_span as f32 * cell_w + (col_span - 1) as f32 * spacing_x_frac,
                                    row_span as f32 * cell_h + (row_span - 1) as f32 * spacing_y_frac,
                                );
                                let child_actual = self.recursive_determine_size_and_hidden(child, ProposedSizes::container(child_cell_size), children_can_hide);

                                let h_per_row = (child_actual.y - (row_span - 1) as f32 * spacing_y_frac) / row_span as f32;
                                for r in 0..row_span {
                                    let row = actual_row + r;
                                    if row < row_heights.len() {
                                        row_heights[row] = row_heights[row].max(h_per_row);
                                    }
                                }
                            }
                        });

                        let total_h = row_heights.iter().sum::<f32>() + spacing_y_frac * (n_rows as f32 - 1.0).max(0.0);
                        Xy::new(size_to_propose.x, total_h)
                    });
                }
            },
        }
       

        // Decide our own size. 
        //   We either use the size that we decided before, or we change our mind to based on children.
        // todo: is we're not fitcontenting, we can skip the update_for_* calls instead, and then remove this, I guess.
        let mut final_size = size;

        for axis in [X, Y] {
            match self.sys.nodes[i].params.layout.size[axis] { // todo if let
                Size::FitContent => {
                    // if we use content_size instead of the size above, then content_size doesn't have padding in
                    let mut content_size_with_padding = content_size;
                    content_size_with_padding[axis] += 2.0 * padding[axis];
                    final_size[axis] = content_size_with_padding[axis];
                }
                _ => {},
            }
        }

        self.sys.nodes[i].size = final_size;
        return final_size;
    }

    fn determine_image_size(&mut self, i: NodeI, proposed_size: Xy<f32>) -> Xy<f32> {
        if let Some(imageref) = &self.sys.nodes[i].imageref {
            match imageref {
                crate::render::ImageRef::Raster(loaded) => {
                    // use intrinsic size
                    let size_pixels = Xy::new(loaded.width as f32, loaded.height as f32);
                    return self.pixels_to_frac2(size_pixels);
                }
                crate::render::ImageRef::Svg(_loaded) => {
                    // no intrinsic size
                    return proposed_size;
                }
            }
        }
        // Fallback if no image is loaded
        let fallback_pixels = Xy::new(100.0, 100.0);
        return self.pixels_to_frac2(fallback_pixels);
    }

    fn determine_text_size(&mut self, i: NodeI, proposed_size: Xy<f32>) -> Xy<f32> {
        let text_i = self.sys.nodes[i].text_i.as_ref().unwrap();

        match text_i {
            TextI::TextEdit(handle) => {
                let text_edit = self.sys.renderer.text.get_text_edit_mut(&handle);

                if text_edit.single_line() {
                    let layout = text_edit.layout();
                    let text_height = if let Some(first_line) = layout.lines().next() {
                        first_line.metrics().line_height
                    } else {
                        0.0 // todo rethink
                    };

                    let text_width = proposed_size.x * self.sys.size[X];

                    text_edit.set_size((text_width, text_height));

                    let text_size_pixels = Xy::new(text_width, text_height);
                    return self.pixels_to_frac2(text_size_pixels);

                } else {
                    let w = proposed_size.x * self.sys.size[X];
                    let h = proposed_size.y * self.sys.size[Y];

                    text_edit.set_size((w, h));
                    return proposed_size;
                }

            }
            TextI::TextBox(handle) => {

                let fit_content_y = self.sys.nodes[i].params.layout.size[Y] == Size::FitContent;
                let fit_content_x = self.sys.nodes[i].params.layout.size[X] == Size::FitContent;

                let h = if fit_content_y {
                    BIG_FLOAT
                } else {
                    proposed_size.y * self.sys.size[Y]
                };

                let w = if fit_content_x {
                    if fit_content_y {
                        proposed_size.x * self.sys.size[X]
                    } else {
                        BIG_FLOAT
                    }
                } else {
                    proposed_size.x * self.sys.size[X]
                };

                let text_box = self.sys.renderer.text.get_text_box_mut(&handle);
                text_box.set_size((w, h));
                
                let layout = text_box.layout();
                let size_pixels = Xy::new(layout.width(), layout.height());
                let size = self.pixels_to_frac2(size_pixels);        

                return size;
            }
        }
    }

    pub(crate) fn recursive_place_children(&mut self, i: NodeI) {
        self.sys.nodes[i].content_bounds = XyRect::new_symm([f32::MAX, f32::MIN]);

        self.sys.partial_relayout_count += 1;

        match self.sys.nodes[i].params.children_layout {
            ChildrenLayout::Free => self.place_children_free(i),
            ChildrenLayout::Stack { arrange, axis, spacing } => self.place_children_stack(i, axis, arrange, spacing),
            ChildrenLayout::Grid { columns, spacing_x, spacing_y, flow } => self.place_children_grid(i, columns, spacing_x, spacing_y, flow),
        }

        for_each_child!(self, self.sys.nodes[i], child, {
            self.recursive_place_children(child);
        });
    }

    fn place_children_stack(&mut self, i: NodeI, axis: Axis, arrange: Arrange, spacing: f32) {
        let (main, cross) = (axis, axis.other());
        let stack_rect = self.sys.nodes[i].layout_rect;

        let padding = self.pixels_to_frac2(self.sys.nodes[i].params.layout.padding);
        let spacing = self.pixels_to_frac(spacing, axis);
        
        // On the main axis, totally ignore the children's chosen Position's and place them according to our own Stack::Arrange value.
        // free_placement children are excluded from the stack flow and placed freely instead.

        let mut n: u16 = 0;
        let mut total_size = 0.0;
        for_each_child!(self, self.sys.nodes[i], child, {
            if !self.sys.nodes[child].params.free_placement {
                total_size += self.sys.nodes[child].size[main];
                n += 1;
            }
        });

        if n > 0 {
            total_size += spacing * (n - 1) as f32;
        }

        let mut walking_position = match arrange {
            Arrange::Start => stack_rect[main][0] + padding[main],
            Arrange::End => stack_rect[main][1] - padding[main] - total_size,
            Arrange::Center => {
                let center = (stack_rect[main][1] + stack_rect[main][0]) / 2.0;
                center - total_size / 2.0
            },
            _ => todo!(),
        };

        for_each_child!(self, self.sys.nodes[i], child, {
            if self.sys.nodes[child].params.free_placement {
                self.place_child_free(child, i);
            } else {
                let child_size = self.sys.nodes[child].size;

                match self.sys.nodes[child].params.layout.position[cross] {
                    Pos::Center => {
                        let origin = (stack_rect[cross][1] + stack_rect[cross][0]) / 2.0;
                        self.sys.nodes[child].layout_rect[cross] = [
                            origin - child_size[cross] / 2.0 ,
                            origin + child_size[cross] / 2.0 ,
                        ];
                    },
                    Pos::Start => {
                        let origin = stack_rect[cross][0] + padding[cross];
                        self.sys.nodes[child].layout_rect[cross] = [origin, origin + child_size[cross]];
                    },
                    Pos::Pixels(pixels) => {
                        let static_pos = self.pixels_to_frac(pixels, cross);
                        let anchor_offset = match self.sys.nodes[child].params.layout.anchor[cross] {
                            Anchor::Start => 0.0,
                            Anchor::Center => -child_size[cross] / 2.0,
                            Anchor::End => -child_size[cross],
                            Anchor::Frac(f) => -child_size[cross] * f,
                        };
                        let origin = stack_rect[cross][0] + padding[cross] + static_pos + anchor_offset;
                        self.sys.nodes[child].layout_rect[cross] = [origin, origin + child_size[cross]];
                    },
                    Pos::Frac(frac) => {
                        let inner_size = stack_rect.size()[cross] - 2.0 * padding[cross];
                        let static_pos = frac * inner_size;
                        let anchor_offset = match self.sys.nodes[child].params.layout.anchor[cross] {
                            Anchor::Start => 0.0,
                            Anchor::Center => -child_size[cross] / 2.0,
                            Anchor::End => -child_size[cross],
                            Anchor::Frac(f) => -child_size[cross] * f,
                        };
                        let origin = stack_rect[cross][0] + padding[cross] + static_pos + anchor_offset;
                        self.sys.nodes[child].layout_rect[cross] = [origin, origin + child_size[cross]];
                    },
                    Pos::End => {
                        let origin = stack_rect[cross][1] - padding[cross];
                        self.sys.nodes[child].layout_rect[cross] = [origin - child_size[cross], origin];
                    },
                }

                self.sys.nodes[child].layout_rect[main] = [walking_position, walking_position + child_size[main]];

                self.set_local_layout_rect(child, i);
                self.init_enter_animations(child);

                walking_position += self.sys.nodes[child].size[main] + spacing;

                self.update_content_bounds(i, self.sys.nodes[child].layout_rect);
            }
        });

        // self.set_children_scroll(i);
    }

    fn place_children_grid(&mut self, i: NodeI, _columns: MainAxisCellSize, spacing_x: f32, spacing_y: f32, _flow: GridFlow) {
        let n = self.sys.nodes[i].n_children as usize;
        if n == 0 { return; }

        let n_cols = self.sys.nodes[i].grid_n_columns as usize;
        let n_rows = self.sys.nodes[i].grid_n_rows as usize;
        if n_cols == 0 { return; }

        with_arena(|arena| {
            let parent_rect = self.sys.nodes[i].layout_rect;
            let padding = self.pixels_to_frac2(self.sys.nodes[i].params.layout.padding);
            let spacing_x_frac = self.pixels_to_frac(spacing_x, X);
            let spacing_y_frac = self.pixels_to_frac(spacing_y, Y);

            let inner_w = parent_rect.size().x - 2.0 * padding.x;
            let cell_w = ((inner_w - spacing_x_frac * (n_cols as f32 - 1.0)) / n_cols as f32).max(0.0);

            // Compute row heights from stored positions and child sizes
            let mut row_heights: BumpVec<f32> = BumpVec::new_in(arena);
            row_heights.resize(n_rows, 0.0f32);
            for_each_child!(self, self.sys.nodes[i], child, {
                if !self.sys.nodes[child].params.free_placement {
                    let row_span = (self.sys.nodes[child].params.grid_element.row_span as usize).max(1);
                    let actual_row = self.sys.nodes[child].grid_element_row_i as usize;
                    let h_per_row = (self.sys.nodes[child].size.y - (row_span - 1) as f32 * spacing_y_frac) / row_span as f32;
                    for r in 0..row_span {
                        let row = actual_row + r;
                        if row < row_heights.len() {
                            row_heights[row] = row_heights[row].max(h_per_row);
                        }
                    }
                }
            });

            // Compute cumulative y offsets per row
            let mut row_y_offsets: BumpVec<f32> = BumpVec::new_in(arena);
            row_y_offsets.resize(n_rows, 0.0f32);
            let mut y_acc = 0.0f32;
            for r in 0..n_rows {
                row_y_offsets[r] = y_acc;
                y_acc += row_heights[r] + spacing_y_frac;
            }

            // Place children inside their assigned grid cell
            for_each_child!(self, self.sys.nodes[i], child, {
                if self.sys.nodes[child].params.free_placement {
                    self.place_child_free(child, i);
                } else {
                    let actual_col = self.sys.nodes[child].grid_element_column_i as usize;
                    let actual_row = self.sys.nodes[child].grid_element_row_i as usize;
                    let child_size = self.sys.nodes[child].size;

                    let x0 = parent_rect.x[0] + padding.x + actual_col as f32 * (cell_w + spacing_x_frac);
                    let y0 = parent_rect.y[0] + padding.y + row_y_offsets[actual_row];

                    self.sys.nodes[child].layout_rect.x = [x0, x0 + child_size.x];
                    self.sys.nodes[child].layout_rect.y = [y0, y0 + child_size.y];

                    self.set_local_layout_rect(child, i);
                    self.init_enter_animations(child);
                    self.update_content_bounds(i, self.sys.nodes[child].layout_rect);
                }
            });
        });
    }

    fn place_child_free(&mut self, child: NodeI, parent: NodeI) {
        let parent_rect = self.sys.nodes[parent].layout_rect;
        let padding = self.pixels_to_frac2(self.sys.nodes[parent].params.layout.padding);
        let child_size = self.sys.nodes[child].size;

        for axis in [X, Y] {
            match self.sys.nodes[child].params.layout.position[axis] {
                Pos::Start => {
                    let origin = parent_rect[axis][0] + padding[axis];
                    self.sys.nodes[child].layout_rect[axis] = [origin, origin + child_size[axis]];
                },
                Pos::Pixels(pixels) => {
                    let static_pos = self.pixels_to_frac(pixels, axis);
                    let anchor_offset = match self.sys.nodes[child].params.layout.anchor[axis] {
                        Anchor::Start => 0.0,
                        Anchor::Center => -child_size[axis] / 2.0,
                        Anchor::End => -child_size[axis],
                        Anchor::Frac(f) => -child_size[axis] * f,
                    };
                    let origin = parent_rect[axis][0] + padding[axis] + static_pos + anchor_offset;
                    self.sys.nodes[child].layout_rect[axis] = [origin, origin + child_size[axis]];
                }
                Pos::Frac(frac) => {
                    let inner_size = parent_rect.size()[axis] - 2.0 * padding[axis];
                    let static_pos = frac * inner_size;
                    let anchor_offset = match self.sys.nodes[child].params.layout.anchor[axis] {
                        Anchor::Start => 0.0,
                        Anchor::Center => -child_size[axis] / 2.0,
                        Anchor::End => -child_size[axis],
                        Anchor::Frac(f) => -child_size[axis] * f,
                    };
                    let origin = parent_rect[axis][0] + padding[axis] + static_pos + anchor_offset;
                    self.sys.nodes[child].layout_rect[axis] = [origin, origin + child_size[axis]];
                }
                Pos::End => {
                    let origin = parent_rect[axis][1] - padding[axis];
                    self.sys.nodes[child].layout_rect[axis] = [origin - child_size[axis], origin];
                },
                Pos::Center => {
                    let origin = (parent_rect[axis][0] + parent_rect[axis][1]) / 2.0;
                    self.sys.nodes[child].layout_rect[axis] = [
                        origin - child_size[axis] / 2.0 ,
                        origin + child_size[axis] / 2.0 ,
                    ];
                },
            }
        }

        self.set_local_layout_rect(child, parent);
        self.init_enter_animations(child);
        self.update_content_bounds(parent, self.sys.nodes[child].layout_rect);
    }

    fn place_children_free(&mut self, i: NodeI) {
        for_each_child!(self, self.sys.nodes[i], child, {
            self.place_child_free(child, i);
        });
    }

    fn set_local_layout_rect(&mut self, i: NodeI, parent: NodeI) {       
        let parent_rect = self.sys.nodes[parent].layout_rect;
        let child_rect = self.sys.nodes[i].layout_rect;
        
        self.sys.nodes[i].local_layout_rect = XyRect::new(
            [child_rect.x[0] - parent_rect.x[0], child_rect.x[1] - parent_rect.x[0]],
            [child_rect.y[0] - parent_rect.y[0], child_rect.y[1] - parent_rect.y[0]]
        );

        if ! self.sys.nodes[i].params.animation.state_transition.animate_position
            // && ! self.sys.nodes[i].exit_animation_still_going // this one is not needed, because exiting nodes don't get layouted.
                && ! self.sys.nodes[i].enter_animation_still_going {
            self.sys.nodes[i].local_animated_rect = self.sys.nodes[i].local_layout_rect;
            // might still be adjusted later for enter/exit animations.
        }
    }

    pub(crate) fn init_enter_animations(&mut self, i: NodeI) {
        if self.sys.nodes[i].frame_added != self.current_frame() {
            return;
        }

        self.sys.nodes[i].local_animated_rect = self.sys.nodes[i].local_layout_rect;

        match self.sys.nodes[i].params.animation.enter {
            EnterAnimation::None => {}
            EnterAnimation::Slide { edge, direction: _ } => {
                use SlideEdge::*;
                let rect = self.sys.nodes[i].local_layout_rect;
                let size = rect.size();

                let (offset_x, offset_y) = match edge {
                    Top => (0.0, -size.y.abs()),
                    Bottom => (0.0, size.y.abs()),
                    Left => (-size.x.abs(), 0.0),
                    Right => (size.x.abs(), 0.0),
                };

                self.sys.nodes[i].local_animated_rect.x[0] += offset_x;
                self.sys.nodes[i].local_animated_rect.x[1] += offset_x;
                self.sys.nodes[i].local_animated_rect.y[0] += offset_y;
                self.sys.nodes[i].local_animated_rect.y[1] += offset_y;
                self.sys.nodes[i].enter_animation_still_going = true;
            }
            EnterAnimation::GrowShrink { axis, origin } => {
                use Pos::*;
                let rect = self.sys.nodes[i].local_layout_rect;

                match axis {
                    Axis::X => {
                        // todo: this was dumb actually, static doesn't do anything
                        let origin_x = match origin {
                            Center | Pixels(_) | Frac(_) => (rect.x[0] + rect.x[1]) / 2.0,
                            Start => rect.x[0],
                            End => rect.x[1],
                        };
                        self.sys.nodes[i].local_animated_rect.x[0] = origin_x;
                        self.sys.nodes[i].local_animated_rect.x[1] = origin_x;
                    }
                    Axis::Y => {
                        let origin_y = match origin {
                            Center | Pixels(_) | Frac(_) => (rect.y[0] + rect.y[1]) / 2.0,
                            Start => rect.y[0],
                            End => rect.y[1],
                        };
                        self.sys.nodes[i].local_animated_rect.y[0] = origin_y;
                        self.sys.nodes[i].local_animated_rect.y[1] = origin_y;
                    }
                }
                self.sys.nodes[i].enter_animation_still_going = true;
            }
        }
    }

    pub(crate) fn init_exit_animations(&mut self, i: NodeI) {
        // If already exiting, don't restart another anim.
        if self.sys.nodes[i].exiting { return; }
        // Set exiting even if we don't have an exiting animation, because the node might need to stick around for a parent's exit animation.
        self.sys.nodes[i].exiting = true;
        self.sys.nodes[i].exit_animation_still_going = true;

        // set the whole branch to exiting.
        with_arena(|a| {
            let mut stack = BumpVec::with_capacity_in(20, a);
            for_each_child_including_lingering_reverse!(self, &self.sys.nodes[i], child, {
                stack.push(child);
            });
            while let Some(node) = stack.pop() {
                if self.sys.nodes[node].exiting { continue; }
                self.sys.nodes[node].exiting = true;
                self.sys.nodes[node].exit_animation_still_going = true;
                for_each_child_including_lingering_reverse!(self, &self.sys.nodes[node], child, {
                    stack.push(child);
                });
            }
        });

        match self.sys.nodes[i].params.animation.exit {
            ExitAnimation::None => {}
            ExitAnimation::Slide { edge, direction: _ } => {
                use SlideEdge::*;
                let rect = self.sys.nodes[i].local_layout_rect;
                let size = rect.size();

                let (offset_x, offset_y) = match edge {
                    Top => (0.0, -size.y.abs()),
                    Bottom => (0.0, size.y.abs()),
                    Left => (-size.x.abs(), 0.0),
                    Right => (size.x.abs(), 0.0),
                };

                // Change the layout_rect to move the "target" position.
                // This works because exiting nodes are excluded from layout, so the layout_rect is not updated further.
                self.sys.nodes[i].local_layout_rect.x[0] += offset_x;
                self.sys.nodes[i].local_layout_rect.x[1] += offset_x;
                self.sys.nodes[i].local_layout_rect.y[0] += offset_y;
                self.sys.nodes[i].local_layout_rect.y[1] += offset_y;
            }
            ExitAnimation::GrowShrink { axis, origin } => {
                use Pos::*;
                let rect = self.sys.nodes[i].local_layout_rect;

                match axis {
                    Axis::X => {
                        let origin_x = match origin {
                            Center | Pixels(_) | Frac(_) => (rect.x[0] + rect.x[1]) / 2.0,
                            Start => rect.x[0],
                            End => rect.x[1],
                        };
                        self.sys.nodes[i].local_layout_rect.x[0] = origin_x;
                        self.sys.nodes[i].local_layout_rect.x[1] = origin_x;
                    }
                    Axis::Y => {
                        let origin_y = match origin {
                            Center | Pixels(_) | Frac(_) => (rect.y[0] + rect.y[1]) / 2.0,
                            Start => rect.y[0],
                            End => rect.y[1],
                        };
                        self.sys.nodes[i].local_layout_rect.y[0] = origin_y;
                        self.sys.nodes[i].local_layout_rect.y[1] = origin_y;
                    }
                }
            }
        }

    }

    #[inline]
    fn update_content_bounds(&mut self, i: NodeI, content_rect: XyRect) {
        for axis in [X, Y] {
            let c_bounds = &mut self.sys.nodes[i].content_bounds[axis];
            c_bounds[0] = c_bounds[0].min(content_rect[axis][0]);
            c_bounds[1] = c_bounds[1].max(content_rect[axis][1]);
        }
    }

    pub(crate) fn set_clip_rect(&mut self, i: NodeI) {
        // Start from the parent's clip rect.
        // If nobody wants to clip children, this will always be [0.0, 1.0], passed down from root to everything else. 
        let parent_clip_rect = if i == ROOT_I {
            Xy::new_symm([0.0, 1.0])
        } else {
            let parent = self.sys.nodes[i].parent;
            self.sys.nodes[parent].clip_rect
        };

        let mut clip_rect = parent_clip_rect;
        for axis in [X, Y] {
            if self.sys.nodes[i].params.clip_children[axis] {
                let own_rect = self.sys.nodes[i].real_rect;
                clip_rect[axis] = intersect(own_rect[axis], parent_clip_rect[axis])
            }
        }

        self.sys.nodes[i].clip_rect = clip_rect;
    }

    pub(crate) fn rebuild_render_data(&mut self) {
        self.sys.renderer.begin_frame();

        // This is another separate traversal:
        // - separate from layout because of no-relayout animations
        // - separate from push_render_data so that prepare_text() can run after it knows whether any textbox changed, but before push_render_data.
        self.resolve_all_animations_and_scrolling();

        with_timer("prepare_text", Some(std::time::Duration::from_micros(500)), || {

            self.sys.renderer.prepare_text();
        
        });

        self.push_all_render_and_click_data();
    }

    pub(crate) fn resolve_all_animations_and_scrolling(&mut self) {
        self.sys.click_rects.clear();

        self.sys.changes.unfinished_animations = false;

        self.sys.depth_traversal_queue.clear();
        self.sys.depth_traversal_queue.push(ROOT_I);
        while let Some(i) = self.sys.depth_traversal_queue.pop() {
            self.resolve_animations_and_scrolling(i);
            self.update_text_boxes(i);

            // This loop should be fine even without z-ordering.
            for_each_child_including_lingering_reverse!(self, self.sys.nodes[i], child, {
                self.sys.depth_traversal_queue.push(child);
            });
        }
    }

    pub(crate) fn push_all_render_and_click_data(&mut self) {
        self.sys.custom_render_commands.clear();
        let mut keru_range_start: Option<usize> = None;

        self.sys.z_cursor = Z_START;
        self.sys.depth_traversal_queue.clear();
        self.sys.depth_traversal_queue.push(ROOT_I);

        while let Some(i) = self.sys.depth_traversal_queue.pop() {
            // Assign z values here so they reflect z_index-sorted order.
            self.sys.z_cursor += Z_STEP;
            self.sys.nodes[i].z = self.sys.z_cursor;
            if let Some(text_i) = &self.sys.nodes[i].text_i {
                let z = self.sys.nodes[i].z;
                match text_i {
                    TextI::TextBox(h) => {
                        self.sys.renderer.text.get_text_box_mut(h).set_depth(z);
                    }
                    TextI::TextEdit(h) => {
                        self.sys.renderer.text.get_text_edit_mut(h).set_depth(z);
                    }
                }
            }

            let is_custom = self.sys.nodes[i].params.custom_render;
            let instance_index_before = self.sys.renderer.instance_count();

            self.push_render_and_click_data(i);

            let instance_index_after = self.sys.renderer.instance_count();

            if !is_custom {
                if keru_range_start.is_none() && instance_index_after > instance_index_before {
                    keru_range_start = Some(instance_index_before);
                }
            } else {
                self.add_custom_render_command(i, instance_index_before, instance_index_after, &mut keru_range_start,);
            }

            // Sort z-ordering
            with_arena(|arena| {
                let n_children = self.sys.nodes[i].n_children as usize + 5; // not sure if lingering children are counted, it's free anyway
                let mut scratch = BumpVec::with_capacity_in(n_children, arena);
                let mut current = self.sys.nodes[i].last_child;
                while let Some(child) = current {
                    scratch.push((child, self.sys.nodes[child].params.z_index));
                    current = self.sys.nodes[child].prev_sibling;
                }
                scratch.sort_by(|x, y| {
                    y.1.partial_cmp(&x.1).unwrap_or(std::cmp::Ordering::Equal)
                });
                for (child, _) in scratch {
                    self.sys.depth_traversal_queue.push(child);
                }
            });
        }

        // Close final Keru range if any
        if let Some(start) = keru_range_start {
            let final_count = self.sys.renderer.instance_count();
            if start < final_count {
                self.sys.custom_render_commands.push(RenderCommand::Keru(KeruElementRange::new(start, final_count)));
            }
        }

        self.sys.changes.should_rebuild_render_data = self.sys.changes.unfinished_animations;
    }

    fn add_custom_render_command(
        &mut self,
        i: NodeI,
        instance_index_before: usize,
        instance_index_after: usize,
        keru_range_start: &mut Option<usize>,
    ) {
        // Close any open keru range
        if let Some(start) = *keru_range_start {
            if start < instance_index_before {
                self.sys.custom_render_commands.push(RenderCommand::Keru(
                    KeruElementRange::new(start, instance_index_before),
                ));
            }
            *keru_range_start = None;
        }
    
        // Add custom render command with the node's rectangle
        self.sys.custom_render_commands.push(RenderCommand::CustomRenderingArea {
            key: self.sys.nodes[i].original_key,
            rect: self.sys.nodes[i].real_rect,
        });
    
        // Start a new range
        if instance_index_after > instance_index_before {
            *keru_range_start = Some(instance_index_before);
        }
    }
    
    pub(crate) fn resolve_animations_and_scrolling(&mut self, i: NodeI) {
        // do animations in local space
        let target = self.sys.nodes[i].local_layout_rect;

        // Don't do animations on resizes, unless the flag is not set
        let skip_animations = self.sys.disable_animations_on_resize && self.sys.changes.resize;

        let mut l;
        if skip_animations {
            l = target;
        } else {
            l = self.sys.nodes[i].local_animated_rect;

            let speed = self.sys.global_animation_speed * self.sys.nodes[i].params.animation.speed;

            let dt = 1.0 / 60.0; // todo use real frame time

            let rate = 5.0 * speed * dt;

            let const_speed_pixels = 3.0 * speed;
            let diff = target - l;

            for i in 0..2 {
                // convert normalized diff into pixel space
                let dx_px = diff[X][i] * self.sys.size.x;
                let dy_px = diff[Y][i] * self.sys.size.y;
            
                let dist_px = (dx_px * dx_px + dy_px * dy_px).sqrt();
            
                if dist_px < const_speed_pixels {
                    l[X][i] = target[X][i];
                    l[Y][i] = target[Y][i];
                } else {
                    // normalized direction in pixel space
                    let dir_x = dx_px / dist_px;
                    let dir_y = dy_px / dist_px;
            
                    // same math concept as before but applied along straight-line distance
                    let step_px = (dist_px * rate).ceil();
            
                    l[X][i] += (step_px * dir_x) / self.sys.size.x;
                    l[Y][i] += (step_px * dir_y) / self.sys.size.y;
                }
            }
        };

        self.sys.nodes[i].local_animated_rect = l;

        // add the parent offset
        let parent = self.sys.nodes[i].parent;
        // todo: pick a side depending on the parent stack and stuff like that, separate translation and resize, etc
        let parent_offset = self.sys.nodes[parent].real_rect.top_left();
        self.sys.nodes[i].real_rect = self.sys.nodes[i].local_animated_rect + parent_offset;


        // add scroll
        let scroll = self.local_node_scroll(i);
        self.sys.nodes[i].real_rect += scroll;


        let parent = self.sys.nodes[i].parent;
        let expected_final_parent_offset = self.sys.nodes[parent].expected_final_rect.top_left();

        // set the new target (expected_final_rect)
        self.sys.nodes[i].expected_final_rect = self.sys.nodes[i].local_layout_rect + expected_final_parent_offset + scroll;

        // Accumulate transforms from parent
        self.compute_accumulated_transform(i);

        if ! self.node_or_parent_has_ongoing_animation(i) {
            if self.sys.nodes[i].exiting {
                self.sys.nodes[i].exit_animation_still_going = false;
                // todo: think harder
                self.set_new_ui_input();
            }
            if self.sys.nodes[i].enter_animation_still_going {
                self.sys.nodes[i].enter_animation_still_going = false;
            }
        } else {
            self.sys.changes.unfinished_animations = true;
        }

        self.set_clip_rect(i);
    }

    pub(crate) fn local_node_scroll(&self, i: NodeI) -> Xy<f32> {
        if i == ROOT_I {
            return Xy::new(0.0, 0.0);
        }
        let parent = self.sys.nodes[i].parent;
        if self.sys.nodes.get_node_if_it_still_exists(parent).is_none() {
            panic!("Surely this check isn't needed?");
        }
        if ! self.sys.nodes[parent].params.is_scrollable() {
            return Xy::new(0.0, 0.0);
        }

        let mut res = Xy::new(0.0, 0.0);
        for axis in [X, Y] {
            if self.sys.nodes[parent].params.layout.scrollable[axis] {
                let scroll_offset = self.scroll_offset(parent, axis);
                res[axis] = scroll_offset;
            }
        }
        return res;
    }

    pub(crate) fn compute_accumulated_transform(&mut self, i: NodeI) {
        if i == ROOT_I {
            self.sys.nodes[i].accumulated_transform = Transform::IDENTITY;
            return;
        }
        let parent = self.sys.nodes[i].parent;


        let parent_transform = self.sys.nodes[parent].accumulated_transform;
        let own_transform = self.sys.nodes[i].params.transform;
        let accumulated_transform;

        if own_transform != Transform::IDENTITY {
            // Get node center in pixels for centered scaling
            let rect = self.sys.nodes[i].real_rect;
            let center = rect.center();
            let center_px_x = center.x * self.sys.size[X];
            let center_px_y = center.y * self.sys.size[Y];

            // Center the child's scale around the node's center
            // to scale around C, add C * (1 - scale) to offset
            let factor = (1.0 - own_transform.scale) * parent_transform.scale;
            let scale_center_offset = vec2(center_px_x * factor, center_px_y * factor);

            let acc_offset = parent_transform.offset
                + own_transform.offset * parent_transform.scale
                + scale_center_offset;

            let acc_scale = parent_transform.scale * own_transform.scale;
            
            accumulated_transform = Transform {
                offset: acc_offset,
                scale: acc_scale,
            }

        } else {
            accumulated_transform = parent_transform;
        }

        self.sys.nodes[i].accumulated_transform = accumulated_transform;
    }
}

impl Xy<f32> {
    pub(crate) fn update_for_child(&mut self, child_size: Xy<f32>, stack_axis: Option<Axis>) {
        match stack_axis {
            None => {
                for axis in [X, Y] {
                    if child_size[axis] > self[axis] {
                        self[axis] = child_size[axis];
                    }
                }
            },
            Some(axis) => {
                let cross = axis.other();

                self[axis] += child_size[axis];
                if child_size[cross] > self[cross] {
                    self[cross] = child_size[cross];
                }
            },
        }
    }
    pub(crate) fn update_for_content(&mut self, content_size: Xy<f32>) {
        for axis in [X, Y] {
            if content_size[axis] > self[axis] {
                self[axis] = content_size[axis];
            }
        }
    }

}

// todo remove?
#[derive(Debug)]
pub(crate) struct Scroll {
    relative_offset: Xy<f32>,
}
impl Scroll {
    pub const ZERO: Scroll = Scroll {
        relative_offset: Xy::new(0.0, 0.0),
    };
}

impl Ui {
    pub(crate) fn update_container_scroll(&mut self, i: NodeI, delta: f32, axis: Axis) {       
        let container_rect = self.sys.nodes[i].layout_rect;

        let content_bounds = self.sys.nodes[i].content_bounds;
        let content_rect_size = content_bounds.size()[axis];

        if content_rect_size <= 0.0 {
            self.sys.nodes[i].scroll.relative_offset[axis] = 0.0;
            return;
        }

        // min scroll is the negative/upwards scroll that corrects the bottom end of content ending up below the container's bottom
        let min_scroll = if content_bounds[axis][1] > container_rect[axis][1] {
            container_rect[axis][1] - content_bounds[axis][1]
        } else {
            0.0
        };

        // max scroll is the positive/downwards scroll that corrects the top end of content overflowing above the container's top
        let max_scroll = if content_bounds[axis][0] < container_rect[axis][0] {
            container_rect[axis][0] - content_bounds[axis][0]
        } else {
            0.0
        };
                
        if min_scroll < max_scroll {                
            if self.sys.nodes[i].frame_added == self.sys.current_frame && delta == 0.0 {
                if let ChildrenLayout::Stack { axis: stack_axis, arrange, .. } = self.sys.nodes[i].params.children_layout {
                    if stack_axis == axis {
                        self.sys.nodes[i].scroll.relative_offset[axis] = match arrange {
                            Arrange::End => min_scroll,
                            _ => max_scroll,
                        };
                    }
                }
            } else {
                // Normal scroll update
                self.sys.nodes[i].scroll.relative_offset[axis] += delta;
            }
            
            let rel_offset = &mut self.sys.nodes[i].scroll.relative_offset[axis];
            *rel_offset = rel_offset.clamp(min_scroll, max_scroll);

        } else {
            self.sys.nodes[i].scroll.relative_offset[axis] = 0.0;
        }

    }
    
    pub(crate) fn _clamp_scroll(&mut self, i: NodeI) {
        // todo: this was hella wrong, figure it out from scratch
        // this was mainly for resizing, I think. when the scroll values and bounds get messed up by resizing or relayouts, we should at least reclamp the offset, or set it to zero if all content fits now.
        // for resizing, we should also store the offset as relative. Actually, for text it should be way more advanced, like  keeping track of the position in the text
        for axis in [X, Y] {
            self.update_container_scroll(i, 0.0, axis);
        }
    }

    pub(crate) fn scroll_offset(&self, i: NodeI, axis: Axis) -> f32 {
        let scroll_offset = self.sys.nodes[i].scroll.relative_offset[axis];

        // round it to whole pixels to avoid wobbling
        // account for transform scale to round to real screen pixels
        let size = self.sys.size[axis];
        let scale = self.sys.nodes[i].accumulated_transform.scale;
        let scroll_offset = (scroll_offset * size * scale).round() / scale / size;

        return scroll_offset;
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct ProposedSizes {
    to_this_child: Xy<f32>, // the size that was proposed to a child specifically after dividing between children
    to_all_children: Xy<f32>, // the whole size that the parent proposed to ALL its children collectively
}
impl ProposedSizes {
    pub(crate) const fn stack(to_this_child: Xy<f32>, to_all_children: Xy<f32>) -> ProposedSizes {
        return ProposedSizes {
            to_this_child,
            to_all_children,
        }
    }
    pub(crate) const fn container(size: Xy<f32>) -> ProposedSizes {
        return ProposedSizes {
            to_this_child: size,
            to_all_children: size,
        }
    }
}
