use crate::{Arrange, Color, Interact, Layout, Len, NodeKey, NodeParams, Position, Rect, Size, Stack, TextNodeType, TextOptions, TypedKey, VertexColors};
use crate::math::{Axis, Xy};
use view_derive::node_key;
use Size::*;
use Position::*;
use Len::*;

pub(crate) const NODE_ROOT_PARAMS: NodeParams = NodeParams {
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
};

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
};

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
};

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
};

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
};

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
};

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
};

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
};

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
};

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
};


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
};

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
};

#[node_key] pub(crate) const ANON_NODE: NodeKey;
#[node_key] pub(crate) const ANON_TEXT: TypedKey<TextNodeType>;
#[node_key] pub(crate) const ANON_VSTACK: TypedKey<Stack>;
#[node_key] pub(crate) const ANON_HSTACK: TypedKey<Stack>;