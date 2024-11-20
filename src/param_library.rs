use crate::{color::*, Shape, BASE_RADIUS};
use crate::*;
use Size::*;
use Position::*;
use Len::*;
use Shape::*;

pub const ANON_NODE_ROOT_PARAMS: NodeKey = <NodeKey>::new(Id(0), "ANON_NODE_ROOT_PARAMS");
pub const NODE_ROOT_PARAMS: NodeParams = NodeParams {
    stack: None,
    text_params: None,
    rect: Rect {
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: false,
        outline_only: true,
        vertex_colors: VertexColors::flat(Color::TRANSPARENT)
    },
    interact: Interact {
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(Fixed(Frac(1.0))),
        position: Xy::new_symm(Start),
        padding: Xy::new_symm(Len::ZERO),
    },
    key: ANON_NODE_ROOT_PARAMS,
};
pub const ANON_DEFAULT: NodeKey = <NodeKey>::new(Id(1), "ANON_DEFAULT");
pub const DEFAULT: NodeParams = NodeParams {
    stack: None,
    text_params: Some(TextOptions {
        editable: false,
    }),
    rect: Rect {
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: true,
        outline_only: false,
        vertex_colors: VertexColors::flat(Color::FLGR_BLUE),
    },
    interact: Interact {
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(Fixed(Frac(1.0))),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::ZERO),
    },
    key: ANON_DEFAULT,
};
pub const ANON_V_STACK: NodeKey = <NodeKey>::new(Id(2), "ANON_V_STACK");
pub const V_STACK: NodeParams = NodeParams {
    stack: Some(Stack {
        arrange: Arrange::Center,
        axis: Axis::Y,
        spacing: Len::Pixels(10),
    }),
    text_params: None,
    rect: Rect {
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: false,
        outline_only: true,
        vertex_colors: VertexColors::flat(Color::FLGR_DEBUG_RED),
    },
    interact: Interact {
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new(Size::FitContent, Size::FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::ZERO),
    },
    key: ANON_V_STACK,
};
pub const ANON_H_STACK: NodeKey = <NodeKey>::new(Id(3), "ANON_H_STACK");
pub const H_STACK: NodeParams = NodeParams {
    stack: Some(Stack {
        arrange: Arrange::Start,
        axis: Axis::X,
        spacing: Len::Pixels(5),
    }),
    text_params: None,
    rect: Rect {
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: false,
        outline_only: true,
        vertex_colors: VertexColors::flat(Color::FLGR_DEBUG_RED),
    },
    interact: Interact {
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new(Size::FitContent, Size::FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::ZERO),
    },
    key: ANON_H_STACK,
};
pub const ANON_MARGIN: NodeKey = <NodeKey>::new(Id(4), "ANON_MARGIN");
pub const MARGIN: NodeParams = NodeParams {
    stack: None,
    text_params: None,
    rect: Rect {
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: false,
        outline_only: true,
        vertex_colors: VertexColors::flat(Color::FLGR_DEBUG_RED),
    },
    interact: Interact {
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(Fixed(Frac(0.9))),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::ZERO),
    },
    key: ANON_MARGIN,  
};
pub const ANON_ICON_BUTTON: NodeKey = <NodeKey>::new(Id(5), "ANON_ICON_BUTTON");
pub const ICON_BUTTON: NodeParams = NodeParams {
    stack: None,
    text_params: None,
    rect: Rect {
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: true,
        outline_only: false,
        vertex_colors: VertexColors::FLGR_SOVL_GRAD,
    },
    interact: Interact {
        absorbs_mouse_events: true,
        click_animation: true,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::ZERO),
    },
    key: ANON_ICON_BUTTON,
};
pub const ANON_BUTTON: NodeKey = <NodeKey>::new(Id(6), "ANON_BUTTON");
pub const BUTTON: NodeParams = NodeParams {
    stack: None,
    text_params: Some(TextOptions {
        editable: false,
    }),
    rect: Rect {
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: true,
        outline_only: false,
        // vertex_colors: VertexColors::TEST,
        vertex_colors: VertexColors::diagonal_gradient_backslash(Color::FLGR_BLUE, Color::FLGR_RED),
    },
    interact: Interact {
        absorbs_mouse_events: true,
        click_animation: true,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::Pixels(10)),
    },
    key: ANON_BUTTON,   
};
pub const ANON_LABEL: NodeKey = <NodeKey>::new(Id(7), "ANON_LABEL");
pub const LABEL: NodeParams = NodeParams {
    stack: None,
    text_params: Some(TextOptions {
        editable: false,
    }),
    rect: Rect {
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: true,
        outline_only: false,
        vertex_colors: VertexColors::flat(Color::FLGR_BLUE),
    },
    interact: Interact {
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::Pixels(10)),
    },
    key: ANON_LABEL,
};
pub const ANON_TEXT: NodeKey = <NodeKey>::new(Id(8), "ANON_TEXT");
pub const TEXT: NodeParams = NodeParams {
    stack: None,
    text_params: Some(TextOptions {
        editable: false,
    }),
    rect: Rect {
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: false,
        outline_only: true,
        vertex_colors: VertexColors::flat(Color::FLGR_DEBUG_RED),
    },
    interact: Interact {
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::Pixels(2)),
    },
    key: ANON_TEXT,
};
pub const ANON_EMPTY_TEXT: NodeKey = <NodeKey>::new(Id(9), "ANON_EMPTY_TEXT");
pub const EMPTY_TEXT: NodeParams = NodeParams {
    stack: None,
    text_params: Some(TextOptions {
        editable: false,
    }),
    rect: Rect {
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: false,
        outline_only: true,
        vertex_colors: VertexColors::flat(Color::FLGR_DEBUG_RED),
    },
    interact: Interact {
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::Pixels(2)),
    },
    key: ANON_EMPTY_TEXT,
};

pub const ANON_TEXT_INPUT: NodeKey = <NodeKey>::new(Id(10), "ANON_TEXT_INPUT");
pub const TEXT_INPUT: NodeParams = NodeParams {
    stack: None,
    text_params: Some(TextOptions {
        editable: true,
    }),
    rect: Rect {
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: true,
        outline_only: false,
        vertex_colors: VertexColors::flat(Color::rgba(26, 0, 26, 230)),
    },
    interact: Interact {
        absorbs_mouse_events: true,
        click_animation: true,
    },
    layout: Layout {
        size: Xy::new_symm(Fill),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::Pixels(5)),
    },
    key: ANON_TEXT_INPUT,
};

pub(crate) const ANON_PANEL: NodeKey = <NodeKey>::new(Id(11), "ANON_PANEL");
pub const PANEL: NodeParams = NodeParams {
    stack: None,
    text_params: None,
    rect: Rect {
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: true,
        outline_only: false,
        vertex_colors: VertexColors::FLGR_SOVL_GRAD,
    },
    interact: Interact {
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::Pixels(10)),
    },
    key: ANON_PANEL,
};

pub const CUSTOM_RENDERED_PANEL: NodeParams = NodeParams {
    stack: None,
    text_params: None,
    rect: Rect {
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: true,
        outline_only: true,
        vertex_colors: VertexColors::flat(Color::rgba_f(0.0, 1.0, 0.0, 1.0)),
    },
    interact: Interact {
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::Pixels(10)),
    },
    key: ANON_PANEL,
};

// pub(crate) const ANON_TEXT: TypedKey<TextNodeType> = <NodeKey>::new(Id(13), "ANON_TEXT");
pub(crate) const ANON_VSTACK: NodeKey = <NodeKey>::new(Id(14), "ANON_VSTACK");
pub(crate) const ANON_HSTACK: NodeKey = <NodeKey>::new(Id(15), "ANON_HSTACK");