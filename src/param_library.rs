use crate::{color::*, Shape, BASE_RADIUS};
use crate::*;
use Size::*;
use Position::*;
use Len::*;
use Shape::*;

// todo: not very discoverable from docs. there's a list of constants on the main page, maybe that's good? link to that or something?

const ANON_NODE_ROOT_PARAMS: NodeKey = <NodeKey>::new(Id(0), "ANON_NODE_ROOT_PARAMS");
/// Preset [`NodeParams`] for a node_root_params. 
///
/// You can use the "source" link to inspect the param values. 
pub(crate) const NODE_ROOT_PARAMS: NodeParams = NodeParams {
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
const ANON_DEFAULT: NodeKey = <NodeKey>::new(Id(1), "ANON_DEFAULT");
/// Preset [`NodeParams`] for a default. 
///
/// You can use the "source" link to inspect the param values. 
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
const ANON_V_STACK: NodeKey = <NodeKey>::new(Id(2), "ANON_V_STACK");
/// Preset [`NodeParams`] for a vertical stack. 
///
/// You can use the "source" link to inspect the param values. 
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
const ANON_H_STACK: NodeKey = <NodeKey>::new(Id(3), "ANON_H_STACK");
/// Preset [`NodeParams`] for a horizontal stack. 
///
/// You can use the "source" link to inspect the param values. 
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
const ANON_MARGIN: NodeKey = <NodeKey>::new(Id(4), "ANON_MARGIN");
/// Preset [`NodeParams`] for a margin. 
///
/// You can use the "source" link to inspect the param values. 
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
const ANON_ICON_BUTTON: NodeKey = <NodeKey>::new(Id(5), "ANON_ICON_BUTTON");
/// Preset [`NodeParams`] for an icon button. 
///
/// You can use the "source" link to inspect the param values. 
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
const ANON_BUTTON: NodeKey = <NodeKey>::new(Id(6), "ANON_BUTTON");
/// Preset [`NodeParams`] for a button. 
///
/// You can use the "source" link to inspect the param values. 
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
const ANON_LABEL: NodeKey = <NodeKey>::new(Id(7), "ANON_LABEL");
/// Preset [`NodeParams`] for a label. 
///
/// You can use the "source" link to inspect the param values. 
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
const ANON_TEXT: NodeKey = <NodeKey>::new(Id(8), "ANON_TEXT");
/// Preset [`NodeParams`] for a text element. 
///
/// You can use the "source" link to inspect the param values. 
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

// const ANON_TEXT_INPUT: NodeKey = <NodeKey>::new(Id(10), "ANON_TEXT_INPUT");
// /// Preset [`NodeParams`] for a text_input. 
// ///
// /// You can use the "source" link to inspect the param values. 
// pub const TEXT_INPUT: NodeParams = NodeParams {
//     stack: None,
//     text_params: Some(TextOptions {
//         editable: true,
//     }),
//     rect: Rect {
//         shape: Rectangle { corner_radius: BASE_RADIUS },
//         visible: true,
//         outline_only: false,
//         vertex_colors: VertexColors::flat(Color::rgba(26, 0, 26, 230)),
//     },
//     interact: Interact {
//         absorbs_mouse_events: true,
//         click_animation: true,
//     },
//     layout: Layout {
//         size: Xy::new_symm(Fill),
//         position: Xy::new_symm(Center),
//         padding: Xy::new_symm(Len::Pixels(5)),
//     },
//     key: ANON_TEXT_INPUT,
// };

pub(crate) const ANON_PANEL: NodeKey = <NodeKey>::new(Id(11), "ANON_PANEL");
/// Preset [`NodeParams`] for a panel. 
///
/// You can use the "source" link to inspect the param values. 
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

/// Preset [`NodeParams`] for a container. 
///
/// You can use the "source" link to inspect the param values. 
pub const CONTAINER: NodeParams = NodeParams {
    stack: None,
    text_params: None,
    rect: Rect {
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: false,
        outline_only: true,
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

/// Preset [`NodeParams`] for a custom rendered panel.
///
/// You can use the "source" link to inspect the param values. 
/// 
/// Use [`UiNode::render_rect`] to get the render data for your node, then render it in a separate draw call.
/// ```rust
/// # use keru::*;
/// # pub struct State {
/// #     pub ui: Ui,
/// # }
/// # 
/// # impl State {
/// #   fn declare_ui(&mut self) {
/// #
/// # let ui = &mut self.ui;
/// #
/// # #[node_key] pub const HUE_WHEEL: NodeKey;
/// #
/// let render_data = ui.get_node(HUE_WHEEL).unwrap().render_rect();
/// #
/// #   }
/// # }
/// ```
/// See the color picker in the painter example.
pub const CUSTOM_RENDERED_PANEL: NodeParams = NodeParams {
    stack: None,
    text_params: None,
    rect: Rect {
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: false,
        outline_only: false,
        vertex_colors: VertexColors::GREENSCREEN,
    },
    interact: Interact {
        absorbs_mouse_events: true,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::Pixels(0)),
    },
    key: ANON_PANEL,
};

// pub(crate) const ANON_TEXT: TypedKey<TextNodeType> = <NodeKey>::new(Id(13), "ANON_TEXT");
pub(crate) const ANON_VSTACK: NodeKey = <NodeKey>::new(Id(14), "ANON_VSTACK");
pub(crate) const ANON_HSTACK: NodeKey = <NodeKey>::new(Id(15), "ANON_HSTACK");