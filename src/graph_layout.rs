use keru_layout::TreeAdapter;

use crate::*;
use crate::inner_node::*;
use crate::Axis::{X, Y};

impl System {
    /// Push the node's final inner size into its text box or text edit.
    fn write_text_size(&mut self, i: NodeI) {
        let logical_window = self.logical_size();
        let node = &self.nodes[i];
        let Some(text_i) = &node.text_i else {
            return;
        };
        let padding = node.params.layout.total_padding();
        let inner_x = (node.size.x - padding.x / logical_window.x) * self.size.x;
        let inner_y = (node.size.y - padding.y / logical_window.y) * self.size.y;

        match text_i {
            TextI::TextBox(handle) => {
                self.renderer.text.get_text_box_mut(handle).set_width(inner_x);
            }
            TextI::TextEdit(handle) => {
                self.renderer.text.get_text_edit_mut(handle).set_size((inner_x, inner_y));
            }
        }
    }

    /// Updates the layout params of a node from its params, and registers the change for the next relayout.
    pub(crate) fn sync_layout(&mut self, i: NodeI) {
        let new = layout_params(&self.nodes[i]);
        let old = std::mem::replace(&mut self.nodes.layout_params[i], new);
        if old != new {
            self.register_node_change(i, &old);
            self.changes.relayout = true;
        }
    }

    pub(crate) fn set_exiting(&mut self, i: NodeI, exiting: bool) {
        self.nodes[i].exiting = exiting;
        let old = self.nodes.layout_params[i];
        if old.disabled != exiting {
            self.nodes.layout_params[i].disabled = exiting;
            self.register_node_change(i, &old);
            self.changes.relayout = true;
        }
    }

    /// Registers that the text of a node changed, for the next relayout.
    pub(crate) fn push_content_change(&mut self, i: NodeI) {
        self.register_content_change(i);
        self.changes.relayout = true;
    }

    #[inline]
    fn update_content_bounds(&mut self, i: NodeI, content_rect: XyRect) {
        for axis in [X, Y] {
            let c_bounds = &mut self.nodes[i].content_bounds[axis];
            c_bounds[0] = c_bounds[0].min(content_rect[axis][0]);
            c_bounds[1] = c_bounds[1].max(content_rect[axis][1]);
        }
    }
}

/// The keru tree, as the layout crate sees it.
impl keru_layout::TreeAdapter for System {
    type NodeId = NodeI;

    #[inline(always)]
    fn root(&self) -> NodeI {
        ROOT_I
    }

    #[inline(always)]
    fn parent(&self, i: NodeI) -> NodeI {
        self.nodes.links[i].parent
    }

    #[inline(always)]
    fn first_child(&self, i: NodeI) -> Option<NodeI> {
        self.nodes.links[i].first_child
    }

    #[inline(always)]
    fn next_sibling(&self, i: NodeI) -> Option<NodeI> {
        self.nodes.links[i].next_sibling
    }

    #[inline(always)]
    fn node_layout_params(&self, i: NodeI) -> &keru_layout::LayoutParams {
        &self.nodes.layout_params[i]
    }

    #[inline(always)]
    fn node_layout_data(&self, i: NodeI) -> &keru_layout::NodeLayoutData {
        &self.nodes.layout_data[i]
    }

    #[inline(always)]
    fn node_layout_data_mut(&mut self, i: NodeI) -> &mut keru_layout::NodeLayoutData {
        &mut self.nodes.layout_data[i]
    }

    #[inline(always)]
    fn layout_data(&mut self) -> &mut keru_layout::LayoutData<NodeI> {
        &mut self.nodes.tree_layout_data
    }

    fn measure_text_width(&mut self, i: NodeI) -> f32 {
        measure_text_width(&mut self.renderer.text, self.nodes[i].text_i.as_ref().unwrap(), self.scale_factor)
    }

    fn measure_text_height(&mut self, i: NodeI, wrap_width: Option<f32>) -> f32 {
        measure_text_height(&mut self.renderer.text, self.nodes[i].text_i.as_ref().unwrap(), wrap_width, self.scale_factor)
    }

    fn window_size(&self) -> keru_layout::Xy<f32> {
        let window = self.logical_size();
        keru_layout::Xy::new(window.x, window.y)
    }

    fn write_output_rect(&mut self, i: NodeI, rect: keru_layout::XyRect, local_rect: keru_layout::XyRect) {
        // The rest of keru works in fractions of the window size.
        let window = self.logical_size();
        let rect = rect_to_frac(rect, window);
        let local_rect = rect_to_frac(local_rect, window);
        let size = Xy::new(rect.x[1] - rect.x[0], rect.y[1] - rect.y[0]);
        let parent = self.nodes.links[i].parent;

        if i != ROOT_I {
            let parent_node = &self.nodes[parent];
            let can_hide = match parent_node.params.children_can_hide {
                ChildrenCanHide::Yes => true,
                ChildrenCanHide::No => false,
                ChildrenCanHide::Inherit => parent_node.can_hide,
            };
            self.nodes[i].can_hide = can_hide;
        }

        let node = &mut self.nodes[i];
        node.size = size;
        node.layout_rect = XyRect::new(rect.x, rect.y);
        node.local_layout_rect = XyRect::new(local_rect.x, local_rect.y);
        // The children extend this when they are placed.
        node.content_bounds = XyRect::new_symm([f32::MAX, f32::MIN]);
        if ! node.params.animation.state_transition.animate_layout
            && ! node.exit_animation_still_going
            && ! node.enter_animation_still_going {

            node.local_animated_rect = node.local_layout_rect;
        }
        if node.svg_data.is_some() {
            self.svg_rasterize_queue.push(i);
        }
        self.write_text_size(i);

        if i == ROOT_I {
            return;
        }

        self.nodes[i].init_enter_animations(self.current_frame);

        let params = &self.nodes[i].params;
        let free = params.free_placement || self.nodes[parent].params.children_layout == ChildrenLayout::Free;
        if ! (free && params.ignore_parent_scroll) {
            let layout_rect = self.nodes[i].layout_rect;
            self.update_content_bounds(parent, layout_rect);
        }
    }
}

pub(crate) const EMPTY_PARAMS: keru_layout::LayoutParams = keru_layout::LayoutParams {
    disabled: true,
    ..keru_layout::LayoutParams::DEFAULT
};

#[inline(always)]
fn gl_xy<T, U>(xy: Xy<T>, f: impl Fn(T) -> U) -> keru_layout::Xy<U> {
    keru_layout::Xy::new(f(xy.x), f(xy.y))
}

fn gl_axis(axis: Axis) -> keru_layout::Axis {
    match axis {
        X => keru_layout::Axis::X,
        Y => keru_layout::Axis::Y,
    }
}

fn gl_size(size: Size) -> keru_layout::Size {
    match size {
        Size::Pixels(p) => keru_layout::Size::Pixels(p),
        Size::Frac(f) => keru_layout::Size::Frac(f),
        Size::Fill => keru_layout::Size::Fill,
        Size::FitContent => keru_layout::Size::FitContent,
        Size::AspectRatio(r) => keru_layout::Size::AspectRatio(r),
    }
}

fn gl_pos(pos: Pos) -> keru_layout::Pos {
    match pos {
        Pos::Center => keru_layout::Pos::Center,
        Pos::Start => keru_layout::Pos::Start,
        Pos::End => keru_layout::Pos::End,
        Pos::Pixels(p) => keru_layout::Pos::Pixels(p),
        Pos::Frac(f) => keru_layout::Pos::Frac(f),
    }
}

fn gl_anchor(anchor: Anchor) -> keru_layout::Anchor {
    match anchor {
        Anchor::Start => keru_layout::Anchor::Start,
        Anchor::Center => keru_layout::Anchor::Center,
        Anchor::End => keru_layout::Anchor::End,
        Anchor::Frac(f) => keru_layout::Anchor::Frac(f),
    }
}

fn gl_window_overflow(window_overflow: WindowOverflow) -> keru_layout::WindowOverflow {
    match window_overflow {
        WindowOverflow::Ignore => keru_layout::WindowOverflow::Ignore,
        WindowOverflow::Clamp => keru_layout::WindowOverflow::Clamp,
        WindowOverflow::Flip => keru_layout::WindowOverflow::Flip,
    }
}

fn gl_arrange(arrange: Arrange) -> keru_layout::Arrange {
    match arrange {
        Arrange::Start => keru_layout::Arrange::Start,
        Arrange::End => keru_layout::Arrange::End,
        Arrange::Center => keru_layout::Arrange::Center,
        Arrange::SpaceBetween => keru_layout::Arrange::SpaceBetween,
        Arrange::SpaceAround => keru_layout::Arrange::SpaceAround,
        Arrange::SpaceEvenly => keru_layout::Arrange::SpaceEvenly,
    }
}

fn gl_direction(direction: Direction) -> keru_layout::Direction {
    match direction {
        Direction::LeftToRight => keru_layout::Direction::LeftToRight,
        Direction::RightToLeft => keru_layout::Direction::RightToLeft,
    }
}

fn gl_children_layout(children_layout: ChildrenLayout) -> keru_layout::ChildrenLayout {
    match children_layout {
        ChildrenLayout::Free => keru_layout::ChildrenLayout::Free,
        ChildrenLayout::Stack { arrange, axis, spacing } => keru_layout::ChildrenLayout::Stack {
            arrange: gl_arrange(arrange),
            axis: gl_axis(axis),
            spacing,
        },
        ChildrenLayout::Grid { columns, spacing_x, spacing_y, flow } => keru_layout::ChildrenLayout::Grid {
            columns: match columns {
                MainAxisCellSize::Count(n) => keru_layout::MainAxisCellSize::Count(n),
                MainAxisCellSize::Width(w) => keru_layout::MainAxisCellSize::Width(w),
            },
            spacing_x,
            spacing_y,
            flow: keru_layout::GridFlow {
                main_axis: gl_axis(flow.main_axis),
                x_fill_direction: gl_direction(flow.x_fill_direction),
                y_fill_direction: gl_direction(flow.y_fill_direction),
                backfill: flow.backfill,
            },
        },
    }
}

pub(crate) fn layout_params(node: &InnerNode) -> keru_layout::LayoutParams {
    let params = &node.params;
    let layout = &params.layout;
    let text = match node.text_i {
        Some(TextI::TextBox(_)) => keru_layout::TextContent::Box,
        Some(TextI::TextEdit(_)) => keru_layout::TextContent::Edit,
        None => keru_layout::TextContent::None,
    };
    keru_layout::LayoutParams {
        size: gl_xy(layout.size, gl_size),
        min_size: gl_xy(layout.min_size, |s| s.map(gl_size)),
        max_size: gl_xy(layout.max_size, |s| s.map(gl_size)),
        fill_weight: 1.0,
        padding: gl_xy(layout.padding, |p| p),
        children_layout: gl_children_layout(params.children_layout),
        free_placement: params.free_placement,
        grid_span: keru_layout::Xy::new(params.grid_element.column_span, params.grid_element.row_span),
        text,
        image: image_size(node),
        disabled: node.exiting,
        position: gl_xy(layout.position, gl_pos),
        anchor: gl_xy(layout.anchor, gl_anchor),
        children_origin_flipped: keru_layout::Xy::new(
            layout.children_origin_x == HorizontalOrigin::Right,
            layout.children_origin_y == VerticalOrigin::Bottom,
        ),
        window_overflow: gl_xy(layout.window_overflow, gl_window_overflow),
    }
}

pub(crate) fn image_size(node: &InnerNode) -> Option<keru_layout::Xy<f32>> {
    match &node.imageref {
        Some(ImageRef::Raster(loaded)) => Some(keru_layout::Xy::new(loaded.width as f32, loaded.height as f32)),
        _ => None,
    }
}

impl Ui {
    pub(crate) fn compute_layout(&mut self) {
        if self.sys.changes.full_relayout {
            log::debug!("Full relayout");
            self.sys.solve_layout();
        } else {
            match self.sys.solve_layout_incremental() {
                keru_layout::IncrementalRelayoutOutcome::Success { computed } => log::debug!("Incremental relayout: {computed} slots computed"),
                fallback => log::debug!("Full relayout: {fallback:?}"),
            }
        }
    }
}

const TEXT_WIDTH_TOLERANCE: f32 = 0.05;

/// Converts a rect from logical pixels to fractions of the window size.
fn rect_to_frac(rect: keru_layout::XyRect, window: Xy<f32>) -> keru_layout::XyRect {
    keru_layout::Xy::new([rect.x[0] / window.x, rect.x[1] / window.x], [rect.y[0] / window.y, rect.y[1] / window.y])
}

/// Converts a rect from fractions of the window size to logical pixels.
pub(crate) fn rect_to_pixels(rect: keru_layout::XyRect, window: Xy<f32>) -> keru_layout::XyRect {
    keru_layout::Xy::new([rect.x[0] * window.x, rect.x[1] * window.x], [rect.y[0] * window.y, rect.y[1] * window.y])
}

// The text system measures in physical pixels, and the layout wants logical pixels.
fn measure_text_width(text: &mut keru_draw::Text, text_i: &TextI, scale_factor: f32) -> f32 {
    match text_i {
        TextI::TextBox(handle) => {
            let widths = text.get_text_box_mut(handle).content_widths();
            let unwrapped = widths.max.max(widths.min);
            (unwrapped + TEXT_WIDTH_TOLERANCE) / scale_factor
        }
        // A text edit can be scrolled, so it never asks for any width.
        TextI::TextEdit(_) => 0.0,
    }
}

fn measure_text_height(text: &mut keru_draw::Text, text_i: &TextI, wrap_width: Option<f32>, scale_factor: f32) -> f32 {
    match text_i {
        TextI::TextBox(handle) => {
            let text_box = text.get_text_box_mut(handle);

            let widths = text_box.content_widths();
            let unwrapped = widths.max.max(widths.min);
            let width = match wrap_width {
                Some(width) => (width * scale_factor).clamp(widths.min, unwrapped),
                None => unwrapped,
            };

            text_box.set_width(width);
            text_box.layout().height() / scale_factor
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
            height / scale_factor
        }
    }
}
