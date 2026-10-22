use keru_graph_layout as gl;

use crate::*;
use crate::inner_node::*;
use crate::Axis::{X, Y};

/// The keru tree, as the layout crate sees it.
pub(crate) struct KeruTree<'a> {
    pub nodes: &'a mut Nodes,
    pub text: &'a mut keru_draw::Text,
    pub window: Xy<f32>,
}

impl gl::TreeAdapter for KeruTree<'_> {
    type Node = NodeI;

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
    fn layout_data(&self, i: NodeI) -> &gl::NodeLayout {
        &self.nodes.layout_data[i]
    }

    #[inline(always)]
    fn layout_data_mut(&mut self, i: NodeI) -> &mut gl::NodeLayout {
        &mut self.nodes.layout_data[i]
    }

    fn measure_text(&mut self, i: NodeI, width: Option<f32>) -> gl::Xy<f32> {
        let size = measure_text(self.text, self.nodes[i].text_i.as_ref().unwrap(), width, self.window);
        gl::Xy::new(size.x, size.y)
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
    }
}

pub(crate) fn image_size(node: &InnerNode) -> Option<gl::Xy<f32>> {
    match &node.imageref {
        Some(ImageRef::Raster(loaded)) => Some(gl::Xy::new(loaded.width as f32, loaded.height as f32)),
        _ => None,
    }
}

impl Ui {
    pub(crate) fn calculate_sizes(&mut self) {
        let window_size = self.sys.logical_size();
        let System { nodes, renderer, size, .. } = &mut self.sys;
        let mut tree = KeruTree { nodes, text: &mut renderer.text, window: *size };
        gl::solve(&mut tree, gl::Xy::new(window_size.x, window_size.y));
        self.write_sizes_recursive(ROOT_I);
    }

    fn write_sizes_recursive(&mut self, i: NodeI) {
        let layout = &self.sys.nodes.layout_data[i];
        let size = layout.size();
        let cell = layout.grid_cell();
        let lines = layout.grid_lines();
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
