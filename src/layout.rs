use glam::vec2;

use crate::*;
use crate::inner_node::*;

use bumpalo::collections::Vec as BumpVec;

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
        let nothing_to_do = !partial_relayouts && !full_relayout && !text_changed;
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

    /// Relayout only the scrollbar nodes for `container_i`, without touching the container or any other children.
    pub(crate) fn partial_relayout_for_scrollbar(&mut self, container_i: NodeI) {
        let container_key = self.sys.nodes[container_i].original_key;

        for key in [
            container_key.sibling(SCROLL_RAIL_Y), container_key.sibling(SCROLL_HANDLE_Y),
            container_key.sibling(SCROLL_RAIL_X), container_key.sibling(SCROLL_HANDLE_X),
        ] {
            let Some(node_i) = self.sys.nodes.get_by_id(key.id_with_key_scope()) else {
                continue;
            };
            // self.recursive_determine_size_and_hidden(node_i, proposed_size, false);
            self.place_child_free(node_i, container_i);
        }
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
                            X => aspect / window_aspect,
                            Y => window_aspect / aspect,
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
            },
            ChildrenLayout::Stack { axis, spacing, arrange: _ } => {
                let spacing = self.pixels_to_frac(spacing, axis);

                // Subtract stack spacing
                let mut n_stack_children: f32 = 0.0;
                for_each_child!(self, self.sys.nodes[i], child, {
                    if !self.sys.nodes[child].params.free_placement {
                        n_stack_children += 1.0;
                    }
                });
                let mut available_size_left = size_to_propose;
                if n_stack_children > 1.5 {
                    available_size_left[axis] -= spacing * (n_stack_children - 1.0);
                }

                let mut n_added_children = 0;
                let mut n_fill_children = 0;
                // First, do all fixed-size children
                for_each_child!(self, self.sys.nodes[i], child, {
                    if self.sys.nodes[child].params.free_placement {
                        // (for free_placement children, do the recursion without partecipating in the stack calculation)
                        self.recursive_determine_size_and_hidden(child, ProposedSizes::container(size_to_propose), children_can_hide);
                    } else {
                        let size_on_axis = self.sys.nodes[child].params.layout.size[axis];
                        if size_on_axis != Size::Fill && !matches!(size_on_axis, Size::Frac(_)) {
                            let child_size = self.recursive_determine_size_and_hidden(child, ProposedSizes::stack(available_size_left, size_to_propose), children_can_hide);
                            content_size.update_for_child(child_size, Some(axis));
                            if n_added_children != 0 {
                                content_size[axis] += spacing;
                            }
                            available_size_left[axis] -= child_size[axis];
                            n_added_children += 1;
                        } else if size_on_axis == Size::Fill {
                            n_fill_children += 1;
                        }
                    }
                });

                // Second, do Frac children - they get a fraction of the remaining space after fixed children.
                // All Frac children share the same base (snapshot before this pass), so Frac(0.5) always
                // means 50% of the post-fixed remainder regardless of how many Frac siblings there are.
                let remaining_after_fixed = available_size_left;
                for_each_child!(self, self.sys.nodes[i], child, {
                    if !self.sys.nodes[child].params.free_placement {
                        if matches!(self.sys.nodes[child].params.layout.size[axis], Size::Frac(_)) {
                            // Pass remaining space as to_all_children on the stack axis, full size on cross axis
                            let mut frac_all_children = size_to_propose;
                            frac_all_children[axis] = remaining_after_fixed[axis];
                            let child_size = self.recursive_determine_size_and_hidden(child, ProposedSizes::stack(available_size_left, frac_all_children), children_can_hide);
                            content_size.update_for_child(child_size, Some(axis));
                            if n_added_children != 0 {
                                content_size[axis] += spacing;
                            }
                            available_size_left[axis] -= child_size[axis];
                            n_added_children += 1;
                        }
                    }
                });

                if n_fill_children > 0 {
                    // then, divide the remaining space between the Fill children
                    if n_fill_children > 1 {
                        available_size_left[axis] -= ((n_fill_children - 1) as f32) * spacing;
                    }
                    let mut size_per_child = available_size_left;
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
        // We either use the size that we decided before, or we change our mind to based on children if we are FitContent.
        let mut final_size = size;

        let fit_content_x = self.sys.nodes[i].params.layout.size[X] == Size::FitContent;
        let fit_content_y = self.sys.nodes[i].params.layout.size[Y] == Size::FitContent;

        if fit_content_x || fit_content_y {
            // Propose the whole size_to_propose to any inline text/image, and let them decide.
            if self.sys.nodes[i].text_i.is_some() {
                let text_size = self.determine_text_size(i, size_to_propose);
                content_size.update_for_content(text_size);
            }
            if self.sys.nodes[i].imageref.is_some() {
                let image_size = self.determine_image_size(i, size_to_propose);
                content_size.update_for_content(image_size);
            }

            if fit_content_x {
                let content_size_with_padding = content_size.x + 2.0 * padding.x;
                final_size.x = content_size_with_padding;
            }
            if fit_content_y {
                let content_size_with_padding = content_size.y + 2.0 * padding.y;
                final_size.y = content_size_with_padding;
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

    // This is only relevant when the parent is FitContent.
    // Should reorganize
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

                    return Xy::new(text_width / self.sys.size[X], text_height / self.sys.size[Y]);

                } else {
                    let w = proposed_size.x * self.sys.size[X];
                    let h = proposed_size.y * self.sys.size[Y];

                    text_edit.set_size((w, h));
                    return proposed_size;
                }

            }
            TextI::TextBox(handle) => {

                let mut size = proposed_size;
                let proposed_size_pixels = proposed_size * self.sys.size;

                let fit_content_x = self.sys.nodes[i].params.layout.size[X] == Size::FitContent;
                let fit_content_y = self.sys.nodes[i].params.layout.size[Y] == Size::FitContent;

                if fit_content_x || fit_content_y {
                    let text_box = self.sys.renderer.text.get_text_box_mut(&handle);

                    if text_box.needs_relayout() {
                        // layout in the whole available space
                        text_box.set_size((proposed_size_pixels.x, proposed_size_pixels.y));
                        // after, it would make sense to also shrink the text box size... but that would mean that needs_relayout() would be true again on the next frame.
                        // and it's probably okay without. selection already requires a click on the actual layout bounds, not on the whole textbox.
                        // It should be fine to shrink just the node
                    }

                    let layout = text_box.layout();
                    if fit_content_x {
                        size.x = layout.width() / self.sys.size[X];
                    }
                    if fit_content_y {
                        size.y = layout.height() / self.sys.size[Y];
                    }
                }

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

        let mut n: u32 = 0;
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

                self.sys.nodes[child].layout_rect[cross] = self.resolve_pos_on_axis(i, child, cross);

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

    fn resolve_pos_on_axis(&self, parent: NodeI, child: NodeI, axis: Axis) -> [f32; 2] {
        let rect = self.sys.nodes[parent].layout_rect;
        let padding = self.pixels_to_frac(self.sys.nodes[parent].params.layout.padding[axis], axis);
        let flipped = match axis {
            X => self.sys.nodes[parent].params.layout.pos_origin_x == HorizontalOrigin::Right,
            Y => self.sys.nodes[parent].params.layout.pos_origin_y == VerticalOrigin::Bottom,
        };

        let child_size = self.sys.nodes[child].size[axis];

        // Anchor as a fraction of the child measured from its origin-side edge.
        let anchor_frac = match self.sys.nodes[child].params.layout.anchor[axis] {
            Anchor::Start => 0.0,
            Anchor::Center => 0.5,
            Anchor::End => 1.0,
            Anchor::Frac(f) => f,
        };

        // Place a child whose anchor point lands on `reference`, where `reference`
        // is an offset measured from the origin edge growing inwards.
        let place_at = |reference: f32| {
            if !flipped {
                let low = reference - anchor_frac * child_size;
                [low, low + child_size]
            } else {
                let high = reference + anchor_frac * child_size;
                [high - child_size, high]
            }
        };
        // Flush against the origin edge (`Pos::Start`) or the far edge (`Pos::End`).
        let origin_edge = if !flipped { rect[axis][0] + padding } else { rect[axis][1] - padding };
        let far_edge = if !flipped { rect[axis][1] - padding } else { rect[axis][0] + padding };

        match self.sys.nodes[child].params.layout.position[axis] {
            Pos::Start => {
                if !flipped { [origin_edge, origin_edge + child_size] } else { [origin_edge - child_size, origin_edge] }
            },
            Pos::End => {
                if !flipped { [far_edge - child_size, far_edge] } else { [far_edge, far_edge + child_size] }
            },
            Pos::Pixels(pixels) => {
                let static_pos = self.pixels_to_frac(pixels, axis);
                place_at(if !flipped { origin_edge + static_pos } else { origin_edge - static_pos })
            },
            Pos::Frac(frac) => {
                let inner_size = rect.size()[axis] - 2.0 * padding;
                let static_pos = frac * inner_size;
                place_at(if !flipped { origin_edge + static_pos } else { origin_edge - static_pos })
            },
            Pos::Center => {
                let center = (rect[axis][0] + rect[axis][1]) / 2.0;
                [center - child_size / 2.0, center + child_size / 2.0]
            },
        }
    }

    pub(crate) fn place_child_free(&mut self, child: NodeI, parent: NodeI) {
        for axis in [X, Y] {
            self.sys.nodes[child].layout_rect[axis] = self.resolve_pos_on_axis(parent, child, axis);
        }

        self.set_local_layout_rect(child, parent);
        self.init_enter_animations(child);
        if !self.sys.nodes[child].params.ignore_parent_scroll {
            self.update_content_bounds(parent, self.sys.nodes[child].layout_rect);
        }
    }

    pub(crate) fn place_children_free(&mut self, i: NodeI) {
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
            && ! self.sys.nodes[i].exit_animation_still_going
            && ! self.sys.nodes[i].enter_animation_still_going {

            self.sys.nodes[i].local_animated_rect = self.sys.nodes[i].local_layout_rect;
        }
    }

    pub(crate) fn init_enter_animations(&mut self, i: NodeI) {
        let is_just_added_or_dehidden = self.sys.nodes[i].frame_added == self.current_frame();
        if ! is_just_added_or_dehidden {
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
            EnterAnimation::FadeIn => {
                self.sys.nodes[i].fade_alpha = 0.0;
                // self.sys.nodes[i].enter_animation_still_going = true;
            }
        }
    }

    pub(crate) fn init_exit_animations(&mut self, i: NodeI) {
        // If already exiting, don't restart another anim.
        if self.sys.nodes[i].exiting {
            return;
        }
        // Set exiting even if we don't have an exiting animation, because the node might need to stick around for a parent's exit animation.
        self.sys.nodes[i].exiting = true;

        if self.sys.nodes[i].params.animation.exit == ExitAnimation::None {
            return;
        }

        self.sys.nodes[i].exit_animation_still_going = true;

        // set the whole branch to exiting.
        with_arena(|a| {
            let mut stack = BumpVec::with_capacity_in(20, a);
            for_each_child_including_lingering_reverse!(self, &self.sys.nodes[i], child, {
                stack.push(child);
            });
            while let Some(node) = stack.pop() {
                if self.sys.nodes[node].exit_animation_still_going { continue; }
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
            ExitAnimation::FadeOut => {}
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

        self.update_property_animations();

        with_timer("prepare_text", Some(std::time::Duration::from_micros(500)), || {
            self.sys.renderer.prepare_text();
        });

        self.push_all_render_and_click_data();
    }

    pub(crate) fn resolve_all_animations_and_scrolling(&mut self) {
        self.sys.click_rects.clear();

        self.sys.changes.unfinished_animations = false;

        struct AnimationTraversalNode {
            node: NodeI,
            parent_scroll: Xy<f32>,
            parent_expected_final_rect: XyRect,
        }

        with_arena(|arena| {
            let mut traversal_queue: BumpVec<AnimationTraversalNode> = BumpVec::with_capacity_in(64, arena);
            traversal_queue.push(AnimationTraversalNode {
                node: ROOT_I,
                parent_scroll: Xy::new(0.0, 0.0),
                parent_expected_final_rect: XyRect::new_symm([0.0, 0.0]),
            });

            while let Some(entry) = traversal_queue.pop() {
                let i = entry.node;
                self.update_scroll_animation(i);
                let expected_final_rect = self.resolve_animations_and_scrolling(i, entry.parent_scroll, entry.parent_expected_final_rect);

                // This could also be gated by ! self.node_is_offscreen(i), but it's a bit scary. Technically text boxes can overflow the node rect. And if the text box doesn't know its real location, it might not realize that it's offscreen and can cull itself, and it might end up being counterproductive.
                self.update_text_boxes(i);

                let child_scroll = self.scroll_for_children(i);

                // This loop should be fine even without z-ordering.
                for_each_child_including_lingering_reverse!(self, self.sys.nodes[i], child, {
                    traversal_queue.push(AnimationTraversalNode {
                        node: child,
                        parent_scroll: child_scroll,
                        parent_expected_final_rect: expected_final_rect,
                    });
                });
            }
        });
    }

    fn scroll_for_children(&self, i: NodeI) -> Xy<f32> {
        let mut res = Xy::new(0.0, 0.0);
        for axis in [X, Y] {
            if self.sys.nodes[i].params.layout.scrollable[axis] {
                res[axis] = self.scroll_offset(i, axis);
            }
        }
        res
    }

    pub(crate) fn push_all_render_and_click_data(&mut self) {
        self.sys.custom_render_commands.clear();
        let mut keru_range_start: Option<usize> = None;

        self.sys.z_cursor = Z_START;

        with_arena(|arena| {
            let mut z_ordering_vec: BumpVec<(NodeI, f32)> = BumpVec::with_capacity_in(20, arena);
            let mut traversal_queue: BumpVec<(NodeI, f32)> = BumpVec::with_capacity_in(64, arena);
            traversal_queue.push((ROOT_I, 1.0));

            while let Some((i, inherited_alpha)) = traversal_queue.pop() {
                // Assign z values here so they reflect z_index-sorted order.
                self.sys.z_cursor += Z_STEP;
                self.sys.nodes[i].z = self.sys.z_cursor;

                // Cascade the node's opacity multiplicatively down the tree.
                let effective_alpha = inherited_alpha * self.sys.nodes[i].params.alpha * self.sys.nodes[i].fade_alpha;

                if ! self.node_is_offscreen(i) {
                    let is_custom = self.sys.nodes[i].params.custom_render;
                    let instance_index_before = self.sys.renderer.instance_count();

                    self.push_render_and_click_data(i, effective_alpha);

                    let instance_index_after = self.sys.renderer.instance_count();

                    if !is_custom {
                        if keru_range_start.is_none() && instance_index_after > instance_index_before {
                            keru_range_start = Some(instance_index_before);
                        }
                    } else {
                        self.add_custom_render_command(i, instance_index_before, instance_index_after, &mut keru_range_start,);
                    }
                }

                // Sort z-ordering
                z_ordering_vec.clear();
                let mut current = self.sys.nodes[i].last_child;
                while let Some(child) = current {
                    z_ordering_vec.push((child, self.sys.nodes[child].params.z_index));
                    current = self.sys.nodes[child].prev_sibling;
                }
                z_ordering_vec.sort_by(|x, y| {
                    y.1.partial_cmp(&x.1).unwrap_or(std::cmp::Ordering::Equal)
                });
                for (child, _) in &z_ordering_vec {
                    traversal_queue.push((*child, effective_alpha));
                }
            }
        });


        if self.sys.show_focus_indicator {
            if let Some(i) = self.sys.focused.and_then(|id| self.sys.nodes.get_by_id(id)) {
                if self.sys.nodes[i].params.interact.show_focus_indicator {
                    let transformed = self.sys.nodes[i].accumulated_transform != Transform::IDENTITY;
                    if transformed {
                        if let Some(handle) = self.sys.nodes[i].accumulated_transform_handle {
                            self.sys.renderer.set_current_transform(handle);
                        }
                    }

                    self.draw_focus_rect(i);

                    if transformed {
                        self.sys.renderer.clear_current_transform();
                    }
                }
            }
        }

        self.sys.renderer.draw_text_decorations();

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
    
    // original check: (rect * screen * scale).round() / scale compared to screen * threshold
    // the * scale / scale cancel, leaving just rect compared to threshold in normalized coords
    pub(crate) fn node_is_offscreen(&self, i: NodeI) -> bool {
        let rect = self.sys.nodes[i].real_rect;
        rect[X][1] < -2.0
            || rect[X][0] > 3.0
            || rect[Y][1] < -2.0
            || rect[Y][0] > 3.0
    }

    pub(crate) fn resolve_animations_and_scrolling(&mut self, i: NodeI, parent_scroll: Xy<f32>, parent_expected_final_rect: XyRect) -> XyRect {
        let still_moving = self.resolve_animation(i);

        // add the parent offset
        let parent = self.sys.nodes[i].parent;

        let real_tl = self.sys.nodes[parent].real_rect.top_left();
        let mut parent_offset = real_tl;

        // Heuristics to use a better parent_offset in specific cases.
        // I don't know if it's possible to solve this generally.
        let parent_enter_going = self.sys.nodes[parent].enter_animation_still_going;
        let parent_exit_going = self.sys.nodes[parent].exit_animation_still_going;
        let parent_exiting = self.sys.nodes[parent].exiting;
        if parent_enter_going || parent_exit_going || parent_exiting {

            let parent_enter_anim = &self.sys.nodes[parent].params.animation.enter;
            let parent_exit_anim = &self.sys.nodes[parent].params.animation.exit;

            if parent_enter_going {
                let layout_tl = parent_expected_final_rect.top_left();
                if let EnterAnimation::GrowShrink { axis, origin } = *parent_enter_anim {
                    match origin {
                        Pos::End | Pos::Center => match axis {
                            Axis::X => parent_offset.x = layout_tl.x,
                            Axis::Y => parent_offset.y = layout_tl.y,
                        },
                        _ => {}
                    }
                }
            }
            if parent_exiting {
                let parent_size = self.sys.nodes[parent].layout_rect.size();
                if let ExitAnimation::GrowShrink { axis, origin } = *parent_exit_anim {
                    match origin {
                        Pos::End => match axis {
                            Axis::X => parent_offset.x = self.sys.nodes[parent].real_rect.x[1] - parent_size.x,
                            Axis::Y => parent_offset.y = self.sys.nodes[parent].real_rect.y[1] - parent_size.y,
                        },
                        // Midpoint is stable: original start = stable midpoint - original size/2
                        Pos::Center => match axis {
                            Axis::X => { let r = self.sys.nodes[parent].real_rect.x; parent_offset.x = (r[0] + r[1]) / 2.0 - parent_size.x / 2.0; },
                            Axis::Y => { let r = self.sys.nodes[parent].real_rect.y; parent_offset.y = (r[0] + r[1]) / 2.0 - parent_size.y / 2.0; },
                        },
                        _ => {}
                    }
                }
            };
        }

        // let parent_offset = self.sys.nodes[parent].real_rect.top_left();

        self.sys.nodes[i].real_rect = self.sys.nodes[i].local_animated_rect + parent_offset;


        // add scroll
        let scroll = if self.sys.nodes[i].params.ignore_parent_scroll {
            Xy::new(0.0, 0.0)
        } else {
            parent_scroll
        };
        self.sys.nodes[i].real_rect += scroll;


        // compute the settled target rect (local_layout_rect in world space, with scroll)
        let expected_final_rect = self.sys.nodes[i].local_layout_rect + parent_expected_final_rect.top_left() + scroll;

        // Accumulate transforms from parent
        self.compute_accumulated_transform(i);

        let parent = self.sys.nodes[i].parent;
        let parent_exiting = self.sys.nodes[parent].exit_animation_still_going;
        if !still_moving && !parent_exiting {
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

        expected_final_rect
    }

    pub(crate) fn resolve_animation(&mut self, i: NodeI) -> bool {
        // do animations in local space
        let target = self.sys.nodes[i].local_layout_rect;

        // Todo: try a bruteforce optimization for offscreen nodes.
        let mut l = target;
        let mut still_moving = false;
        let animate_position = self.sys.nodes[i].params.animation.state_transition.animate_position;
        let enter_anim = self.sys.nodes[i].enter_animation_still_going;
        let exit_anim = self.sys.nodes[i].exit_animation_still_going;
        let skip_animations = (!animate_position && !enter_anim && !exit_anim) || (self.sys.disable_animations_on_resize && self.sys.changes.resize);

        if ! skip_animations {
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
                    still_moving = true;
                    // normalized direction in pixel space
                    let dir_x = dx_px / dist_px;
                    let dir_y = dy_px / dist_px;

                    // same math concept as before but applied along straight-line distance
                    let step_px = (dist_px * rate).ceil();

                    l[X][i] += (step_px * dir_x) / self.sys.size.x;
                    l[Y][i] += (step_px * dir_y) / self.sys.size.y;
                }
            }
        }

        self.sys.nodes[i].local_animated_rect = l;

        let fade_exiting_animation = self.sys.nodes[i].params.animation.exit == ExitAnimation::FadeOut;
        let fade_target = if self.sys.nodes[i].exiting && fade_exiting_animation { 0.0 } else { 1.0 };
        if self.sys.nodes[i].fade_alpha != fade_target {
            let speed = self.sys.global_animation_speed * self.sys.nodes[i].params.animation.speed;
            let rate = (5.0 * speed / 60.0).clamp(0.0, 1.0);
            let (new_fade, fade_done) = step_f32(self.sys.nodes[i].fade_alpha, fade_target, rate);
            self.sys.nodes[i].fade_alpha = new_fade;
            if ! fade_done {
                still_moving = true;
            }
        }

        still_moving
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

impl Ui {
    // Todo: maybe deduplicate the math
    pub(crate) fn update_scroll_animation(&mut self, i: NodeI) {
        let mut moved = false;
        for axis in [X, Y] {
            let current = self.sys.nodes[i].scroll[axis];
            let target = self.sys.nodes[i].scroll_animation_target[axis];
            let diff = target - current;
            if diff == 0.0 {
                continue;
            }
            moved = true;

            let speed = self.sys.global_animation_speed * self.sys.nodes[i].params.animation.speed;
            let rate = 5.0 * speed * (1.0 / 60.0);
            let const_speed_pixels = 3.0 * speed;

            let diff_px = diff * self.sys.size[axis];
            let dist_px = diff_px.abs();

            if dist_px < const_speed_pixels {
                self.sys.nodes[i].scroll[axis] = target;
            } else {
                let dir = diff_px / dist_px;
                let step_px = (dist_px * rate).ceil();
                self.sys.nodes[i].scroll[axis] += (step_px * dir) / self.sys.size[axis];
                self.sys.changes.unfinished_animations = true;
            }
        }

        // Keep the scrollbar thumb in sync with the displayed offset as it animates.
        // The thumb nodes are children of `i`, visited later in this same traversal.
        if moved {
            self.sys.update_scrollbar_handle_params(i);
            self.partial_relayout_for_scrollbar(i);
        }
    }

    pub(crate) fn scroll_offset(&self, i: NodeI, axis: Axis) -> f32 {
        let scroll_offset = self.sys.nodes[i].scroll[axis];

        // round it to whole pixels to avoid wobbling
        // account for transform scale to round to real screen pixels
        let size = self.sys.size[axis];
        let scale = self.sys.nodes[i].accumulated_transform.scale;
        let scroll_offset = (scroll_offset * size * scale).round() / scale / size;

        return scroll_offset;
    }
}

impl System {

    /// Adjust the scroll offsets of all scrollable ancestors of `i` so that node
    /// `i` ends up inside their visible rects. Used by keyboard focus navigation
    /// to scroll the focused node into view.
    pub(crate) fn scroll_node_into_view(&mut self, i: NodeI, padding_px: f32, animate: bool) {
        let target_rect = self.nodes[i].real_rect;

        let mut adjusted = false;
        let mut current = self.nodes[i].parent;
        while current != ROOT_I {
            for axis in [X, Y] {
                if !self.nodes[current].params.layout.scrollable[axis] {
                    continue;
                }

                let viewport = self.nodes[current].real_rect[axis];
                let target = target_rect[axis];

                // Leave a gap between the node and the viewport edge instead of
                // aligning it exactly against the boundary.
                let pad = padding_px / self.size[axis];

                let mut delta = 0.0;
                if target[0] < viewport[0] {
                    // target starts before the viewport: scroll content forward (down/right).
                    delta = viewport[0] + pad - target[0];
                } else if target[1] > viewport[1] {
                    // target ends after the viewport: scroll content backward (up/left).
                    // If the target is taller/wider than the viewport, prefer aligning the
                    // start edge rather than overshooting it.
                    let delta_end = viewport[1] - pad - target[1];
                    let delta_start = viewport[0] + pad - target[0];
                    delta = delta_end.max(delta_start);
                }

                if delta != 0.0 {
                    self.update_container_scroll(current, delta, axis, animate);
                    if ! animate {
                        self.update_scrollbar_handle_params(current);
                        // The scroll and focus rect change will probably cause a full relayout anyway, I think.
                        // self.partial_relayout_for_scrollbar(current);
                    }
                    adjusted = true;
                }
            }
            current = self.nodes[current].parent;
        }

        if adjusted {
            self.changes.should_rebuild_render_data = true;
            self.changes.need_rerender = true;
        }
    }

    pub(crate) fn update_container_scroll(&mut self, i: NodeI, delta: f32, axis: Axis, animate: bool) {
        let container_rect = self.nodes[i].layout_rect;

        let content_bounds = self.nodes[i].content_bounds;
        let content_rect_size = content_bounds.size()[axis];

        if content_rect_size <= 0.0 {
            self.nodes[i].scroll[axis] = 0.0;
            self.nodes[i].scroll_animation_target[axis] = 0.0;
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
            if self.nodes[i].frame_added == self.current_frame && delta == 0.0 {
                if let ChildrenLayout::Stack { axis: stack_axis, arrange, .. } = self.nodes[i].params.children_layout {
                    if stack_axis == axis {
                        let init = match arrange {
                            Arrange::End => min_scroll,
                            _ => max_scroll,
                        };
                        self.nodes[i].scroll[axis] = init;
                        self.nodes[i].scroll_animation_target[axis] = init;
                    }
                }
            } else {
                // Normal scroll update
                let base = if animate {
                    self.nodes[i].scroll[axis]
                } else {
                    self.nodes[i].scroll_animation_target[axis]
                };
                self.nodes[i].scroll_animation_target[axis] = base + delta;
            }

            let target = &mut self.nodes[i].scroll_animation_target[axis];
            *target = target.clamp(min_scroll, max_scroll);

            if ! animate {
                self.nodes[i].scroll[axis] = self.nodes[i].scroll_animation_target[axis];
            }

        } else {
            self.nodes[i].scroll[axis] = 0.0;
            self.nodes[i].scroll_animation_target[axis] = 0.0;
        }

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
