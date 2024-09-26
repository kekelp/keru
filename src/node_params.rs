use crate::{Arrange, Color, Interact, Layout, NodeKey, NodeParams, Position, Rect, Size, Stack, TextOptions, TypedKey, VertexColors};
use crate::math::*;
use view_derive::node_key;
use Size::*;
use Position::*;
use Len::*;
#[node_key] pub const ANON_NODE_ROOT_PARAMS: NodeKey;
pub const NODE_ROOT_PARAMS: NodeParams = NodeParams {
    stack: None,
    text_params: None,
    rect: Rect {
        visible: false,
        filled: false,
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
#[node_key] pub const ANON_DEFAULT: NodeKey;
pub const DEFAULT: NodeParams = NodeParams {
    stack: None,
    text_params: Some(TextOptions {
        editable: false,
    }),
    rect: Rect {
        visible: true,
        filled: true,
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
#[node_key] pub const ANON_V_STACK: NodeKey;
pub const V_STACK: NodeParams = NodeParams {
    stack: Some(Stack {
        arrange: Arrange::Center,
        axis: Axis::Y,
        spacing: Len::Pixels(10),
    }),
    text_params: None,
    rect: Rect {
        visible: false,
        filled: false,
        vertex_colors: VertexColors::flat(Color::FLGR_DEBUG_RED),
    },
    interact: Interact {
        absorbs_mouse_events: false,
        click_animation: true,
    },
    layout: Layout {
        size: Xy::new(Size::FitContent, Size::FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::ZERO),
    },
    key: ANON_V_STACK,
};
#[node_key] pub const ANON_H_STACK: NodeKey;
pub const H_STACK: NodeParams = NodeParams {
    stack: Some(Stack {
        arrange: Arrange::Start,
        axis: Axis::X,
        spacing: Len::Pixels(5),
    }),
    text_params: None,
    rect: Rect {
        visible: false,
        filled: false,
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
#[node_key] pub const ANON_MARGIN: NodeKey;
pub const MARGIN: NodeParams = NodeParams {
    stack: None,
    text_params: None,
    rect: Rect {
        visible: false,
        filled: false,
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
#[node_key] pub const ANON_ICON_BUTTON: NodeKey;
pub const ICON_BUTTON: NodeParams = NodeParams {
    stack: None,
    text_params: None,
    rect: Rect {
        visible: true,
        filled: true,
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
#[node_key] pub const ANON_BUTTON: NodeKey;
pub const BUTTON: NodeParams = NodeParams {
    stack: None,
    text_params: Some(TextOptions {
        editable: false,
    }),
    rect: Rect {
        visible: true,
        filled: true,
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
#[node_key] pub const ANON_LABEL: NodeKey;
pub const LABEL: NodeParams = NodeParams {
    stack: None,
    text_params: Some(TextOptions {
        editable: false,
    }),
    rect: Rect {
        visible: true,
        filled: true,
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
#[node_key] pub const ANON_TEXT: NodeKey;
pub const TEXT: NodeParams = NodeParams {
    stack: None,
    text_params: Some(TextOptions {
        editable: false,
    }),
    rect: Rect {
        visible: false,
        filled: false,
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
#[node_key] pub const ANON_EMPTY_TEXT: NodeKey;
pub const EMPTY_TEXT: NodeParams = NodeParams {
    stack: None,
    text_params: Some(TextOptions {
        editable: false,
    }),
    rect: Rect {
        visible: false,
        filled: false,
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

#[node_key] pub const ANON_TEXT_INPUT: NodeKey;
pub const TEXT_INPUT: NodeParams = NodeParams {
    stack: None,
    text_params: Some(TextOptions {
        editable: true,
    }),
    rect: Rect {
        visible: true,
        filled: true,
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

#[node_key] pub(crate) const ANON_PANEL: NodeKey;
pub const PANEL: NodeParams = NodeParams {
    stack: None,
    text_params: None,
    rect: Rect {
        visible: true,
        filled: true,
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
    key: ANON_PANEL,
};

#[node_key] pub(crate) const ANON_NODE: NodeKey;
// #[node_key] pub(crate) const ANON_TEXT: TypedKey<TextNodeType>;
#[node_key] pub(crate) const ANON_VSTACK: TypedKey<Stack>;
#[node_key] pub(crate) const ANON_HSTACK: TypedKey<Stack>;