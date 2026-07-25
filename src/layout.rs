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
    /// Trigger a relayout of the UI.
    /// 
    /// Normally it's not necessary to call this function manually: the UI will relayout automatically in [`Ui::finish_frame()`].
    pub fn relayout(&mut self) {
        let full_relayout = self.sys.changes.full_relayout;
        let text_changed = self.sys.changes.text_changed;
        let nothing_to_do = !full_relayout && !text_changed;
        if nothing_to_do {
            return;
        }

        self.sys.changes.need_gpu_rect_update = true;
        self.sys.changes.need_rerender = true;

        // todo: bring back partial relayouts
        self.clay_relayout_from_root();

        self.rebuild_render_data();

        self.sys.changes.reset_layout_changes();

        // after doing a relayout, we might be moving the hovered node away from the cursor.
        // So we run resolve_hover again, possibly causing another relayout next frame
        self.resolve_hover();
    }

    pub(crate) fn clay_relayout_from_root(&mut self) {
        log::info!("Full relayout");

        self.clay_fit_sizing(ROOT_I, X);
        self.sys.nodes[ROOT_I].size[X] = 1.0;

        self.clay_grow_and_shrink(X);

        self.clay_wrap_text(ROOT_I);

        self.clay_fit_sizing(ROOT_I, Y);
        self.sys.nodes[ROOT_I].size[Y] = 1.0;
        self.clay_grow_and_shrink(Y);

        self.clay_aspect_ratio_widths(ROOT_I);

        self.clay_size_text_edits(ROOT_I);

        self.sys.nodes[ROOT_I].layout_rect = XyRect::new([0.0, 1.0], [0.0, 1.0]);
        self.recursive_place_children(ROOT_I);

        // self.dump_layout(ROOT_I, 0);
    }

    #[allow(dead_code)]
    fn dump_layout(&mut self, i: NodeI, depth: usize) {
        let name = self.node_debug_name(i);
        let s = self.sys.nodes[i].size;
        let r = self.sys.nodes[i].layout_rect;
        log::info!("{:indent$}{name}: size=({:.3},{:.3}) rect=x[{:.3},{:.3}] y[{:.3},{:.3}]",
            "", s.x, s.y, r.x[0], r.x[1], r.y[0], r.y[1], indent = depth * 2);
        for_each_child!(self, self.sys.nodes[i], child, {
            self.dump_layout(child, depth + 1);
        });
    }

    fn clay_fit_sizing(&mut self, i: NodeI, axis: Axis) {
        let children_can_hide = match self.sys.nodes[i].params.children_can_hide {
            ChildrenCanHide::Yes => true,
            ChildrenCanHide::No => false,
            ChildrenCanHide::Inherit => self.sys.nodes[i].can_hide,
        };

        for_each_child!(self, self.sys.nodes[i], child, {
            if axis == X {
                self.sys.nodes[child].can_hide = children_can_hide;
            }
            self.clay_fit_sizing(child, axis);
        });

        let padding = self.pixels_to_frac2(self.sys.nodes[i].params.layout.padding)[axis];

        // Aggregate the children into a content fit size and a content min size: sum along the
        // stack axis, max across it.
        let mut content = 0.0f32;
        let mut content_min = 0.0f32;
        match self.sys.nodes[i].params.children_layout {
            ChildrenLayout::Stack { axis: stack_axis, spacing, .. } if stack_axis == axis => {
                let spacing = self.pixels_to_frac(spacing, axis);
                let mut n = 0;
                for_each_child!(self, self.sys.nodes[i], child, {
                    if ! self.sys.nodes[child].params.free_placement {
                        content += self.sys.nodes[child].size[axis];
                        content_min += self.sys.nodes[child].min_content_size[axis];
                        if n != 0 { content += spacing; content_min += spacing; }
                        n += 1;
                    }
                });
            }
            ChildrenLayout::Grid { columns, spacing_x, spacing_y, flow } => {
                (content, content_min) = self.clay_grid_fit(i, axis, columns, spacing_x, spacing_y, flow);
            }
            // Across a stack, and for Free: the bounding extent.
            _ => {
                for_each_child!(self, self.sys.nodes[i], child, {
                    if ! self.sys.nodes[child].params.free_placement {
                        content = content.max(self.sys.nodes[child].size[axis]);
                        content_min = content_min.max(self.sys.nodes[child].min_content_size[axis]);
                    }
                });
            }
        }

        if self.sys.nodes[i].text_i.is_some() {
            let (fit, min) = self.text_content_size(i, axis);
            content = content.max(fit);
            content_min = content_min.max(min);
        }
        if self.sys.nodes[i].imageref.is_some() {
            let image = self.determine_image_size(i, Xy::new(1.0, 1.0))[axis];
            content = content.max(image);
            content_min = content_min.max(image);
        }

        let (size, min) = match self.sys.nodes[i].params.layout.size[axis] {
            Size::Pixels(px) => {
                let p = self.pixels_to_frac(px, axis);
                (p, p) // a fixed size can't shrink
            }

            Size::AspectRatio(ratio) if axis == Y => {
                let h = self.aspect_ratio_size(i, Y, ratio);
                (h, h)
            }

            other => {
                let min = if self.sys.nodes[i].params.layout.scrollable[axis] || matches!(other, Size::Frac(_)) {
                    0.0
                } else {
                    content_min
                };
                (content + 2.0 * padding, min + 2.0 * padding)
            }
        };

        self.sys.nodes[i].size[axis] = size;
        self.sys.nodes[i].min_content_size[axis] = min;
    }

    fn aspect_ratio_size(&self, i: NodeI, axis: Axis, ratio: f32) -> f32 {
        let other = axis.other();
        let other_pixels = self.sys.nodes[i].size[other] * self.sys.size[other];
        (ratio * other_pixels / self.sys.size[axis]).max(0.0)
    }

    fn clay_aspect_ratio_widths(&mut self, i: NodeI) {
        if let Size::AspectRatio(ratio) = self.sys.nodes[i].params.layout.size[X]
            && ! matches!(self.sys.nodes[i].params.layout.size[Y], Size::AspectRatio(_)) {
            self.sys.nodes[i].size[X] = self.aspect_ratio_size(i, X, ratio);
        }

        for_each_child!(self, self.sys.nodes[i], child, {
            self.clay_aspect_ratio_widths(child);
        });
    }

    fn text_content_size(&mut self, i: NodeI, axis: Axis) -> (f32, f32) {
        let window = self.sys.size[axis];
        let text_i = self.sys.nodes[i].text_i.as_ref().unwrap();

        const TEXT_WIDTH_TOLERANCE: f32 = 0.05;

        match text_i {
            TextI::TextBox(handle) => {
                let text_box = self.sys.renderer.text.get_text_box_mut(handle);
                match axis {
                    X => {
                        let widths = text_box.content_widths();
                        (
                            (widths.max + TEXT_WIDTH_TOLERANCE) / window,
                            (widths.min + TEXT_WIDTH_TOLERANCE) / window,
                        )
                    }
                    Y => {
                        let height = text_box.layout().height() / window;
                        (height, height)
                    }
                }
            }
            TextI::TextEdit(handle) => {
                let text_edit = self.sys.renderer.text.get_text_edit_mut(handle);
                if axis == Y && text_edit.single_line() {
                    let line_height = match text_edit.layout().lines().next() {
                        Some(first_line) => first_line.metrics().line_height,
                        None => 0.0,
                    };
                    return (line_height / window, 0.0);
                }
                (1.0, 0.0)
            }
        }
    }

    fn clay_wrap_text(&mut self, i: NodeI) {
        for_each_child!(self, self.sys.nodes[i], child, {
            self.clay_wrap_text(child);
        });

        if let Some(TextI::TextBox(handle)) = &self.sys.nodes[i].text_i {
            let padding = self.pixels_to_frac2(self.sys.nodes[i].params.layout.padding);
            let inner_width = (self.sys.nodes[i].size.x - 2.0 * padding.x) * self.sys.size[X];
            let text_box = self.sys.renderer.text.get_text_box_mut(handle);
            text_box.set_width(inner_width);
        }
    }

    fn clay_size_text_edits(&mut self, i: NodeI) {
        for_each_child!(self, self.sys.nodes[i], child, {
            self.clay_size_text_edits(child);
        });

        if let Some(TextI::TextEdit(handle)) = &self.sys.nodes[i].text_i {
            let padding = self.pixels_to_frac2(self.sys.nodes[i].params.layout.padding);
            let inner_x = (self.sys.nodes[i].size.x - 2.0 * padding.x) * self.sys.size[X];
            let inner_y = (self.sys.nodes[i].size.y - 2.0 * padding.y) * self.sys.size[Y];
            let text_edit = self.sys.renderer.text.get_text_edit_mut(handle);
            text_edit.set_size((inner_x, inner_y));
        }
    }

    fn clay_grow_and_shrink(&mut self, axis: Axis) {
        with_arena(|arena| {
            let mut bfs = BumpVec::new_in(arena);
            bfs.push(ROOT_I);
            let mut resizable = BumpVec::new_in(arena);

            let mut idx = 0;
            while idx < bfs.len() {
                let parent = bfs[idx];
                idx += 1;

                let padding2 = 2.0 * self.pixels_to_frac2(self.sys.nodes[parent].params.layout.padding)[axis];
                let parent_size = self.sys.nodes[parent].size[axis];
                let parent_inner = parent_size - padding2;

                let mut n_children = 0;
                for_each_child!(self, self.sys.nodes[parent], child, {
                    if self.sys.nodes[child].n_children > 0 {
                        bfs.push(child);
                    }
                    if self.sys.nodes[child].params.free_placement {
                        self.clay_size_child_in(child, axis, parent_inner);
                    } else {
                        n_children += 1;
                    }
                });

                if n_children == 0 { continue; }

                // A grid sizes its children from its cells, not by distributing slack.
                if let ChildrenLayout::Grid { columns, spacing_x, spacing_y, flow } = self.sys.nodes[parent].params.children_layout {
                    self.clay_grid_sizing(parent, axis, columns, spacing_x, spacing_y, flow);
                    continue;
                }

                // Some when we're distributing along this parent's stack axis, None when across it.
                let along = match self.sys.nodes[parent].params.children_layout {
                    ChildrenLayout::Stack { axis: a, spacing, .. } if a == axis =>
                        Some(self.pixels_to_frac(spacing, axis)),
                    _ => None,
                };

                if let Some(spacing) = along {
                    let gaps = spacing * (n_children as f32 - 1.0).max(0.0);

                    let mut inner = gaps;
                    for_each_child!(self, self.sys.nodes[parent], child, {
                        if ! self.sys.nodes[child].params.free_placement {
                            if let Size::Frac(f) = self.sys.nodes[child].params.layout.size[axis] {
                                self.sys.nodes[child].size[axis] = (parent_size - padding2 - gaps) * f;
                            }
                            inner += self.sys.nodes[child].size[axis];
                        }
                    });
                    let mut to_distribute = parent_size - padding2 - inner;

                    let scrolls = self.sys.nodes[parent].params.layout.scrollable[axis];
                    if to_distribute < 0.0 && !scrolls {
                        resizable.clear();
                        for_each_child!(self, self.sys.nodes[parent], child, {
                            if ! self.sys.nodes[child].params.free_placement && self.clay_can_shrink(child, axis) {
                                resizable.push(child);
                            }
                        });
                        self.clay_shrink(axis, &mut resizable, &mut to_distribute);
                    } else if to_distribute > 0.0 {
                        resizable.clear();
                        for_each_child!(self, self.sys.nodes[parent], child, {
                            if ! self.sys.nodes[child].params.free_placement
                                && self.sys.nodes[child].params.layout.size[axis] == Size::Fill {
                                resizable.push(child);
                            }
                        });
                        self.clay_grow(axis, &mut resizable, &mut to_distribute);
                    }
                } else {
                    let mut available = parent_inner;
                    if self.sys.nodes[parent].params.layout.scrollable[axis] {
                        for_each_child!(self, self.sys.nodes[parent], child, {
                            if ! self.sys.nodes[child].params.free_placement {
                                available = available.max(self.sys.nodes[child].size[axis]);
                            }
                        });
                    }

                    for_each_child!(self, self.sys.nodes[parent], child, {
                        if ! self.sys.nodes[child].params.free_placement {
                            self.clay_size_child_in(child, axis, available);
                        }
                    });
                }
            }
        });
    }

    fn clay_can_shrink(&self, child: NodeI, axis: Axis) -> bool {
        match self.sys.nodes[child].params.layout.size[axis] {
            Size::Fill | Size::FitContent => true,
            Size::AspectRatio(_) => axis == X,
            _ => false,
        }
    }

    fn clay_size_child_in(&mut self, child: NodeI, axis: Axis, available: f32) {
        match self.sys.nodes[child].params.layout.size[axis] {
            Size::Fill => self.sys.nodes[child].size[axis] = available,
            Size::Frac(f) => self.sys.nodes[child].size[axis] = available * f,
            Size::FitContent => {
                let size = self.sys.nodes[child].size[axis];
                let constrained = if axis == X { size.min(available) } else { size };
                self.sys.nodes[child].size[axis] = constrained.max(self.sys.nodes[child].min_content_size[axis]);
            }
            _ => {}
        }
    }

    fn grid_line(&self, child: NodeI, axis: Axis) -> usize {
        match axis {
            X => self.sys.nodes[child].grid_element_column_i as usize,
            Y => self.sys.nodes[child].grid_element_row_i as usize,
        }
    }

    fn grid_span(&self, child: NodeI, axis: Axis) -> usize {
        let span = match axis {
            X => self.sys.nodes[child].params.grid_element.column_span,
            Y => self.sys.nodes[child].params.grid_element.row_span,
        };
        (span as usize).max(1)
    }

    fn grid_n_lines(&self, i: NodeI, axis: Axis) -> usize {
        match axis {
            X => self.sys.nodes[i].grid_n_columns as usize,
            Y => self.sys.nodes[i].grid_n_rows as usize,
        }
    }

    /// The uniform size of one grid cell along an axis. Only valid once the cells are assigned.
    fn grid_cell_size(&self, i: NodeI, axis: Axis, spacing: f32) -> f32 {
        let padding = self.pixels_to_frac(self.sys.nodes[i].params.layout.padding[axis], axis);
        let inner = self.sys.nodes[i].size[axis] - 2.0 * padding;
        let n = self.grid_n_lines(i, axis) as f32;
        ((inner - spacing * (n - 1.0)) / n).max(0.0)
    }

    /// How many cells fit along the main axis.
    fn grid_n_main(&self, columns: MainAxisCellSize, flow: GridFlow, inner_main: f32, spacing_main: f32) -> usize {
        match columns {
            MainAxisCellSize::Count(n) => (n as usize).max(1),
            MainAxisCellSize::Width(w) => {
                let w = self.pixels_to_frac(w, flow.main_axis);
                ((inner_main + spacing_main) / (w + spacing_main)).floor().max(1.0) as usize
            }
        }
    }

    fn grid_spans(&self, child: NodeI) -> (usize, usize) {
        (self.grid_span(child, X), self.grid_span(child, Y))
    }

    fn grid_assign_cells(&mut self, i: NodeI, n_main: usize, flow: GridFlow) {
        with_arena(|arena| {
            let mut occ = GridOccupancy::new(n_main, arena);
            for_each_child!(self, self.sys.nodes[i], child, {
                if ! self.sys.nodes[child].params.free_placement {
                    let (col_span, row_span) = self.grid_spans(child);
                    let (span_line, span_pos) = to_occ_spans(col_span, row_span, flow);
                    let (occ_line, occ_pos) = occ.place_next(span_line, span_pos, flow.backfill);
                    let (col, row) = from_occ(occ_line, occ_pos, flow);
                    self.sys.nodes[child].grid_element_column_i = col as u16;
                    self.sys.nodes[child].grid_element_row_i = row as u16;
                }
            });

            let (n_cols, n_rows) = match flow.main_axis {
                Axis::X => (n_main, occ.n_lines),
                Axis::Y => (occ.n_lines, n_main),
            };
            self.sys.nodes[i].grid_n_columns = n_cols as u16;
            self.sys.nodes[i].grid_n_rows = n_rows as u16;
        });

        let reversed = flow.x_fill_direction == Direction::RightToLeft
            || flow.y_fill_direction == Direction::RightToLeft;
        if ! reversed { return; }

        let n_cols = self.sys.nodes[i].grid_n_columns as usize;
        let n_rows = self.sys.nodes[i].grid_n_rows as usize;
        for_each_child!(self, self.sys.nodes[i], child, {
            if ! self.sys.nodes[child].params.free_placement {
                let (col_span, row_span) = self.grid_spans(child);
                let logical_col = self.sys.nodes[child].grid_element_column_i as usize;
                let logical_row = self.sys.nodes[child].grid_element_row_i as usize;
                let (col, row) = apply_reversal(logical_col, logical_row, col_span, row_span, n_cols, n_rows, flow);
                self.sys.nodes[child].grid_element_column_i = col as u16;
                self.sys.nodes[child].grid_element_row_i = row as u16;
            }
        });
    }

    fn clay_grid_fit(&mut self, i: NodeI, axis: Axis, columns: MainAxisCellSize, spacing_x: f32, spacing_y: f32, flow: GridFlow) -> (f32, f32) {
        let spacing = match axis {
            X => self.pixels_to_frac(spacing_x, X),
            Y => self.pixels_to_frac(spacing_y, Y),
        };

        if axis == X {
            let n_main = match columns {
                MainAxisCellSize::Count(n) => (n as usize).max(1),
                MainAxisCellSize::Width(_) => 1,
            };
            self.grid_assign_cells(i, n_main, flow);
        }

        let n_lines = self.grid_n_lines(i, axis);
        if n_lines == 0 { return (0.0, 0.0); }

        // The biggest child sets the cell size, since cells are uniform.
        let mut cell = 0.0f32;
        let mut cell_min = 0.0f32;
        for_each_child!(self, self.sys.nodes[i], child, {
            if ! self.sys.nodes[child].params.free_placement {
                let span = self.grid_span(child, axis);
                let share = |v: f32| (v - (span - 1) as f32 * spacing) / span as f32;
                cell = cell.max(share(self.sys.nodes[child].size[axis]));
                cell_min = cell_min.max(share(self.sys.nodes[child].min_content_size[axis]));
            }
        });
        // A fixed cell size along the main axis overrides what the children asked for.
        if let MainAxisCellSize::Width(w) = columns {
            if axis == flow.main_axis {
                cell = self.pixels_to_frac(w, axis);
                cell_min = cell;
            }
        }

        let n = n_lines as f32;
        let extent = |cell: f32| (n * cell + spacing * (n - 1.0)).max(0.0);
        (extent(cell), extent(cell_min))
    }

    fn clay_grid_sizing(&mut self, parent: NodeI, axis: Axis, columns: MainAxisCellSize, spacing_x: f32, spacing_y: f32, flow: GridFlow) {
        let main = flow.main_axis;
        let spacing = Xy::new(self.pixels_to_frac(spacing_x, X), self.pixels_to_frac(spacing_y, Y));

        let assign_axis = match columns {
            MainAxisCellSize::Count(_) => X,
            MainAxisCellSize::Width(_) => main,
        };
        if axis == assign_axis {
            let padding = self.pixels_to_frac2(self.sys.nodes[parent].params.layout.padding);
            let inner_main = self.sys.nodes[parent].size[main] - 2.0 * padding[main];
            let n_main = self.grid_n_main(columns, flow, inner_main, spacing[main]);
            self.grid_assign_cells(parent, n_main, flow);
        }

        self.clay_grid_size_children(parent, axis, spacing);

        if axis == Y && assign_axis == Y {
            self.clay_grid_size_children(parent, X, spacing);
        }
    }

    fn clay_grid_size_children(&mut self, parent: NodeI, axis: Axis, spacing: Xy<f32>) {
        if self.grid_n_lines(parent, axis) == 0 { return; }
        let cell = self.grid_cell_size(parent, axis, spacing[axis]);

        for_each_child!(self, self.sys.nodes[parent], child, {
            if ! self.sys.nodes[child].params.free_placement {
                let span = self.grid_span(child, axis);
                let available = span as f32 * cell + (span - 1) as f32 * spacing[axis];
                self.clay_size_child_in(child, axis, available);
            }
        });
    }

    fn clay_shrink(&mut self, axis: Axis, resizable: &mut BumpVec<NodeI>, to_distribute: &mut f32) {
        while *to_distribute < -1e-6 && !resizable.is_empty() {
            let mut largest = 0.0f32;
            let mut second = 0.0f32;
            let mut delta = *to_distribute;
            for &c in resizable.iter() {
                let cs = self.sys.nodes[c].size[axis];
                if (cs - largest).abs() < 1e-9 { continue; }
                if cs > largest { second = largest; largest = cs; }
                if cs < largest { second = second.max(cs); delta = second - largest; }
            }
            delta = delta.max(*to_distribute / resizable.len() as f32);

            let mut k = 0;
            while k < resizable.len() {
                let c = resizable[k];
                let min = self.sys.nodes[c].min_content_size[axis];
                let prev = self.sys.nodes[c].size[axis];
                if (prev - largest).abs() < 1e-9 {
                    let mut new = prev + delta;
                    let mut done = false;
                    if new <= min { new = min; done = true; }
                    self.sys.nodes[c].size[axis] = new;
                    *to_distribute -= new - prev;
                    if done { resizable.swap_remove(k); continue; }
                }
                k += 1;
            }
        }
    }

    fn clay_grow(&mut self, axis: Axis, resizable: &mut BumpVec<NodeI>, to_distribute: &mut f32) {
        while *to_distribute > 1e-6 && !resizable.is_empty() {
            let mut smallest = f32::MAX;
            let mut second = f32::MAX;
            let mut delta = *to_distribute;
            for &c in resizable.iter() {
                let cs = self.sys.nodes[c].size[axis];
                if (cs - smallest).abs() < 1e-9 { continue; }
                if cs < smallest { second = smallest; smallest = cs; }
                if cs > smallest { second = second.min(cs); delta = second - smallest; }
            }
            delta = delta.min(*to_distribute / resizable.len() as f32);

            for k in 0..resizable.len() {
                let c = resizable[k];
                let prev = self.sys.nodes[c].size[axis];
                if (prev - smallest).abs() < 1e-9 {
                    self.sys.nodes[c].size[axis] = prev + delta;
                    *to_distribute -= delta;
                }
            }
        }
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
            self.place_child_free(node_i, container_i);
        }
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
}

impl Ui {
    pub(crate) fn recursive_place_children(&mut self, i: NodeI) {
        self.sys.nodes[i].content_bounds = XyRect::new_symm([f32::MAX, f32::MIN]);

        match self.sys.nodes[i].params.children_layout {
            ChildrenLayout::Free => self.place_children_free(i),
            ChildrenLayout::Stack { arrange, axis, spacing } => self.place_children_stack(i, axis, arrange, spacing),
            ChildrenLayout::Grid { spacing_x, spacing_y, .. } => self.place_children_grid(i, spacing_x, spacing_y),
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
        let mut children_size = 0.0;
        for_each_child!(self, self.sys.nodes[i], child, {
            if !self.sys.nodes[child].params.free_placement {
                children_size += self.sys.nodes[child].size[main];
                n += 1;
            }
        });

        let total_size = if n > 0 {
            children_size + spacing * (n - 1) as f32
        } else {
            children_size
        };

        let inner = (stack_rect[main][1] - stack_rect[main][0]) - 2.0 * padding[main];
        let free = inner - children_size;

        let (mut walking_position, gap) = match arrange {
            Arrange::Start => (stack_rect[main][0] + padding[main], spacing),
            Arrange::End => (stack_rect[main][1] - padding[main] - total_size, spacing),
            Arrange::Center => {
                let center = (stack_rect[main][1] + stack_rect[main][0]) / 2.0;
                (center - total_size / 2.0, spacing)
            },
            Arrange::SpaceBetween => {
                let gap = if n > 1 { free / (n - 1) as f32 } else { 0.0 };
                (stack_rect[main][0] + padding[main], gap)
            },
            Arrange::SpaceAround => {
                let gap = if n > 0 { free / n as f32 } else { 0.0 };
                (stack_rect[main][0] + padding[main] + gap / 2.0, gap)
            },
            Arrange::SpaceEvenly => {
                let gap = if n > 0 { free / (n + 1) as f32 } else { 0.0 };
                (stack_rect[main][0] + padding[main] + gap, gap)
            },
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

                walking_position += self.sys.nodes[child].size[main] + gap;

                self.update_content_bounds(i, self.sys.nodes[child].layout_rect);
            }
        });

        // self.set_children_scroll(i);
    }

    /// Place every child at the top-left corner of the cells it was assigned in
    /// [`Ui::clay_grid_sizing`]. Children's own `Pos` values are ignored.
    fn place_children_grid(&mut self, i: NodeI, spacing_x: f32, spacing_y: f32) {
        if self.grid_n_lines(i, X) == 0 || self.grid_n_lines(i, Y) == 0 { return; }

        let parent_rect = self.sys.nodes[i].layout_rect;
        let padding = self.pixels_to_frac2(self.sys.nodes[i].params.layout.padding);
        let spacing = Xy::new(self.pixels_to_frac(spacing_x, X), self.pixels_to_frac(spacing_y, Y));
        let cell = Xy::new(self.grid_cell_size(i, X, spacing.x), self.grid_cell_size(i, Y, spacing.y));

        for_each_child!(self, self.sys.nodes[i], child, {
            if self.sys.nodes[child].params.free_placement {
                self.place_child_free(child, i);
            } else {
                let child_size = self.sys.nodes[child].size;
                for axis in [X, Y] {
                    let line = self.grid_line(child, axis);
                    let start = parent_rect[axis][0] + padding[axis] + line as f32 * (cell[axis] + spacing[axis]);
                    self.sys.nodes[child].layout_rect[axis] = [start, start + child_size[axis]];
                }

                self.set_local_layout_rect(child, i);
                self.init_enter_animations(child);
                self.update_content_bounds(i, self.sys.nodes[child].layout_rect);
            }
        });
    }

    fn resolve_pos_on_axis(&self, parent: NodeI, child: NodeI, axis: Axis) -> [f32; 2] {
        let rect = self.sys.nodes[parent].layout_rect;
        let padding = self.pixels_to_frac(self.sys.nodes[parent].params.layout.padding[axis], axis);
        let flipped = match axis {
            X => self.sys.nodes[parent].params.layout.children_origin_x == HorizontalOrigin::Right,
            Y => self.sys.nodes[parent].params.layout.children_origin_y == VerticalOrigin::Bottom,
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
            let rect = self.resolve_pos_on_axis(parent, child, axis);
            self.sys.nodes[child].layout_rect[axis] = self.apply_spill_on_axis(child, parent, axis, rect);
        }

        self.set_local_layout_rect(child, parent);
        self.init_enter_animations(child);
        if !self.sys.nodes[child].params.ignore_parent_scroll {
            self.update_content_bounds(parent, self.sys.nodes[child].layout_rect);
        }
    }

    fn apply_spill_on_axis(&self, child: NodeI, parent: NodeI, axis: Axis, rect: [f32; 2]) -> [f32; 2] {
        let behavior = self.sys.nodes[child].params.layout.window_overflow[axis];
        if behavior == WindowOverflow::Ignore {
            return rect;
        }

        const WIN_LO: f32 = 0.0;
        const WIN_HI: f32 = 1.0;

        let [lo, hi] = rect;
        let size = hi - lo;

        // If the node is larger than the window, just give up.
        if size >= (WIN_HI - WIN_LO) {
            return [WIN_LO, WIN_LO + size];
        }

        let mut out = rect;

        if behavior == WindowOverflow::Flip {
            // The anchor point (in absolute coords) is the point that should stay fixed, e.g. the cursor.
            let flipped = match axis {
                X => self.sys.nodes[parent].params.layout.children_origin_x == HorizontalOrigin::Right,
                Y => self.sys.nodes[parent].params.layout.children_origin_y == VerticalOrigin::Bottom,
            };
            let anchor_frac = match self.sys.nodes[child].params.layout.anchor[axis] {
                Anchor::Start => 0.0,
                Anchor::Center => 0.5,
                Anchor::End => 1.0,
                Anchor::Frac(f) => f,
            };
            let a = if !flipped { lo + anchor_frac * size } else { hi - anchor_frac * size };

            let reflected = [2.0 * a - hi, 2.0 * a - lo];
            if hi > WIN_HI && reflected[0] >= WIN_LO {
                out = reflected;
            } else if lo < WIN_LO && reflected[1] <= WIN_HI {
                out = reflected;
            }
        }

        // Clamp (also the fallback when a flip wouldn't fit either).
        let [mut lo, mut hi] = out;
        let size = hi - lo;
        if hi > WIN_HI {
            hi = WIN_HI;
            lo = hi - size;
        }
        if lo < WIN_LO {
            lo = WIN_LO;
            hi = lo + size;
        }
        [lo, hi]
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

        if ! self.sys.nodes[i].params.animation.state_transition.animate_layout
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
            EnterAnimation::Grow { axis, origin } => {
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
                // We don't need to set enter_animation_still_going, as that's only needed for when enter/exit animations interact with the regular position interpolation ones
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
            ExitAnimation::Shrink { axis, origin } => {
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
                if let EnterAnimation::Grow { axis, origin } = *parent_enter_anim {
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
                if let ExitAnimation::Shrink { axis, origin } = *parent_exit_anim {
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
        let animate_layout = self.sys.nodes[i].params.animation.state_transition.animate_layout;
        let enter_anim = self.sys.nodes[i].enter_animation_still_going;
        let exit_anim = self.sys.nodes[i].exit_animation_still_going;
        let skip_animations = (!animate_layout && !enter_anim && !exit_anim) || (self.sys.disable_animations_on_resize && self.sys.changes.resize);

        if ! skip_animations {
            l = self.sys.nodes[i].local_animated_rect;

            let local_speed = self.sys.nodes[i].params.animation.speed;

            const SNAP_PX: f32 = 3.0;
            const MIN_STEP_PX: f32 = 1.0;

            let diff = target - l;
            // We could try to separate position and size changes by looking at an anchor, either the real Anchor or based on the parent's arrange when it's a Stack.
            // But I don't think it's common to want position to snap and size to animate or viceversa.

            for i in 0..2 {
                // convert normalized diff into pixel space
                let dx_px = diff[X][i] * self.sys.size.x;
                let dy_px = diff[Y][i] * self.sys.size.y;

                let dist_px = (dx_px * dx_px + dy_px * dy_px).sqrt();

                let (step_px, settled) = self.sys.exp_tail_step_dist(dist_px, local_speed, SNAP_PX, MIN_STEP_PX);

                if settled {
                    l[X][i] = target[X][i];
                    l[Y][i] = target[Y][i];
                } else {
                    still_moving = true;

                    // normalized direction in pixel space
                    let dir_x = dx_px / dist_px;
                    let dir_y = dy_px / dist_px;

                    l[X][i] += (step_px * dir_x) / self.sys.size.x;
                    l[Y][i] += (step_px * dir_y) / self.sys.size.y;
                }
            }
        }

        self.sys.nodes[i].local_animated_rect = l;

        let fade_exiting_animation = self.sys.nodes[i].params.animation.exit == ExitAnimation::FadeOut;
        let fade_target = if self.sys.nodes[i].exiting && fade_exiting_animation { 0.0 } else { 1.0 };
        if self.sys.nodes[i].fade_alpha != fade_target {
            let local_speed = self.sys.nodes[i].params.animation.speed;
            let (new_fade, fade_done) = self.sys.exp_tail_step(self.sys.nodes[i].fade_alpha, fade_target, local_speed);
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

impl Ui {
    pub(crate) fn update_scroll_animation(&mut self, i: NodeI) {
        let mut moved = false;
        for axis in [X, Y] {
            let current = self.sys.nodes[i].scroll[axis];
            let target = self.sys.nodes[i].scroll_animation_target[axis];
            if current == target {
                continue;
            }
            moved = true;

            let local_speed = self.sys.nodes[i].params.animation.speed;

            let snap = 0.5 / self.sys.size[axis];
            let (new, settled) = self.sys.pure_exp_step(current, target, local_speed, snap);
            self.sys.nodes[i].scroll[axis] = new;
            if !settled {
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
