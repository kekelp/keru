use keru_graph_layout as gl;
use keru_graph_layout::TreeAdapter;

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
        let padding = node.params.layout.padding;
        let inner_x = (node.size.x - 2.0 * padding.x / logical_window.x) * self.size.x;
        let inner_y = (node.size.y - 2.0 * padding.y / logical_window.y) * self.size.y;

        match text_i {
            TextI::TextBox(handle) => {
                self.renderer.text.get_text_box_mut(handle).set_width(inner_x);
            }
            TextI::TextEdit(handle) => {
                self.renderer.text.get_text_edit_mut(handle).set_size((inner_x, inner_y));
            }
        }
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
impl gl::TreeAdapter for System {
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
    fn params(&self, i: NodeI) -> &gl::LayoutParams {
        &self.nodes.layout_params[i]
    }

    #[inline(always)]
    fn layout_data(&self, i: NodeI) -> &gl::LayoutData {
        &self.nodes.layout_data[i]
    }

    #[inline(always)]
    fn layout_data_mut(&mut self, i: NodeI) -> &mut gl::LayoutData {
        &mut self.nodes.layout_data[i]
    }

    fn measure_text(&mut self, i: NodeI, width: Option<f32>) -> gl::Xy<f32> {
        let size = measure_text(&mut self.renderer.text, self.nodes[i].text_i.as_ref().unwrap(), width, self.size);
        gl::Xy::new(size.x, size.y)
    }

    fn window_size(&self) -> gl::Xy<f32> {
        let window = self.logical_size();
        gl::Xy::new(window.x, window.y)
    }

    fn write_output_rect(&mut self, i: NodeI, rect: gl::XyRect, local_rect: gl::XyRect) {
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

pub(crate) const EMPTY_PARAMS: gl::LayoutParams = gl::LayoutParams {
    disabled: true,
    ..gl::LayoutParams::DEFAULT
};

#[inline(always)]
fn gl_xy<T, U>(xy: Xy<T>, f: impl Fn(T) -> U) -> gl::Xy<U> {
    gl::Xy::new(f(xy.x), f(xy.y))
}

fn gl_axis(axis: Axis) -> gl::Axis {
    match axis {
        X => gl::Axis::X,
        Y => gl::Axis::Y,
    }
}

fn gl_size(size: Size) -> gl::Size {
    match size {
        Size::Pixels(p) => gl::Size::Pixels(p),
        Size::Frac(f) => gl::Size::Frac(f),
        Size::Fill => gl::Size::Fill,
        Size::FitContent => gl::Size::FitContent,
        Size::AspectRatio(r) => gl::Size::AspectRatio(r),
    }
}

fn gl_pos(pos: Pos) -> gl::Pos {
    match pos {
        Pos::Center => gl::Pos::Center,
        Pos::Start => gl::Pos::Start,
        Pos::End => gl::Pos::End,
        Pos::Pixels(p) => gl::Pos::Pixels(p),
        Pos::Frac(f) => gl::Pos::Frac(f),
    }
}

fn gl_anchor(anchor: Anchor) -> gl::Anchor {
    match anchor {
        Anchor::Start => gl::Anchor::Start,
        Anchor::Center => gl::Anchor::Center,
        Anchor::End => gl::Anchor::End,
        Anchor::Frac(f) => gl::Anchor::Frac(f),
    }
}

fn gl_window_overflow(window_overflow: WindowOverflow) -> gl::WindowOverflow {
    match window_overflow {
        WindowOverflow::Ignore => gl::WindowOverflow::Ignore,
        WindowOverflow::Clamp => gl::WindowOverflow::Clamp,
        WindowOverflow::Flip => gl::WindowOverflow::Flip,
    }
}

fn gl_arrange(arrange: Arrange) -> gl::Arrange {
    match arrange {
        Arrange::Start => gl::Arrange::Start,
        Arrange::End => gl::Arrange::End,
        Arrange::Center => gl::Arrange::Center,
        Arrange::SpaceBetween => gl::Arrange::SpaceBetween,
        Arrange::SpaceAround => gl::Arrange::SpaceAround,
        Arrange::SpaceEvenly => gl::Arrange::SpaceEvenly,
    }
}

fn gl_direction(direction: Direction) -> gl::Direction {
    match direction {
        Direction::LeftToRight => gl::Direction::LeftToRight,
        Direction::RightToLeft => gl::Direction::RightToLeft,
    }
}

fn gl_children_layout(children_layout: ChildrenLayout) -> gl::ChildrenLayout {
    match children_layout {
        ChildrenLayout::Free => gl::ChildrenLayout::Free,
        ChildrenLayout::Stack { arrange, axis, spacing } => gl::ChildrenLayout::Stack {
            arrange: gl_arrange(arrange),
            axis: gl_axis(axis),
            spacing,
        },
        ChildrenLayout::Grid { columns, spacing_x, spacing_y, flow } => gl::ChildrenLayout::Grid {
            columns: match columns {
                MainAxisCellSize::Count(n) => gl::MainAxisCellSize::Count(n),
                MainAxisCellSize::Width(w) => gl::MainAxisCellSize::Width(w),
            },
            spacing_x,
            spacing_y,
            flow: gl::GridFlow {
                main_axis: gl_axis(flow.main_axis),
                x_fill_direction: gl_direction(flow.x_fill_direction),
                y_fill_direction: gl_direction(flow.y_fill_direction),
                backfill: flow.backfill,
            },
        },
    }
}

pub(crate) fn layout_params(node: &InnerNode) -> gl::LayoutParams {
    let params = &node.params;
    let layout = &params.layout;
    let text = match node.text_i {
        Some(TextI::TextBox(_)) => gl::TextContent::Box,
        Some(TextI::TextEdit(_)) => gl::TextContent::Edit,
        None => gl::TextContent::None,
    };
    gl::LayoutParams {
        size: gl_xy(layout.size, gl_size),
        min_size: gl_xy(layout.min_size, |s| s.map(gl_size)),
        max_size: gl_xy(layout.max_size, |s| s.map(gl_size)),
        padding: gl_xy(layout.padding, |p| p),
        children_layout: gl_children_layout(params.children_layout),
        free_placement: params.free_placement,
        grid_span: gl::Xy::new(params.grid_element.column_span, params.grid_element.row_span),
        text,
        image: image_size(node),
        disabled: node.exiting,
        position: gl_xy(layout.position, gl_pos),
        anchor: gl_xy(layout.anchor, gl_anchor),
        children_origin_flipped: gl::Xy::new(
            layout.children_origin_x == HorizontalOrigin::Right,
            layout.children_origin_y == VerticalOrigin::Bottom,
        ),
        window_overflow: gl_xy(layout.window_overflow, gl_window_overflow),
    }
}

pub(crate) fn image_size(node: &InnerNode) -> Option<gl::Xy<f32>> {
    match &node.imageref {
        Some(ImageRef::Raster(loaded)) => Some(gl::Xy::new(loaded.width as f32, loaded.height as f32)),
        _ => None,
    }
}

impl Ui {
    pub(crate) fn compute_layout(&mut self) {
        self.sys.solve_layout();
    }
}

const TEXT_WIDTH_TOLERANCE: f32 = 0.05;

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
