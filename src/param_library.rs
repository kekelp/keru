use crate::{color::*, Shape, BASE_RADIUS};
use crate::*;
use Size::*;
use Position::*;
use Shape::*;

// todo: not very discoverable from docs. there's a list of constants on the main page, maybe that's good? link to that or something?

/// [`NodeParams`] for a node_root_params. 
///
/// You can use the "source" link to inspect the param values. 
pub(crate) const NODE_ROOT_PARAMS: NodeParams = NodeParams {
    key: None,
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
        size: Xy::new_symm(Size::Frac(1.0)),
        position: Xy::new_symm(Start),
        padding: Xy::new_symm(Len::ZERO),
        scrollable: Xy::new(false, false),
    },
};
/// [`NodeParams`] for a default. 
///
/// You can use the "source" link to inspect the param values. 
pub const DEFAULT: NodeParams = NodeParams {
    key: None,
    stack: None,
    text_params: None,
    rect: Rect {
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: true,
        outline_only: false,
        vertex_colors: VertexColors::flat(Color::KERU_BLUE),
    },
    interact: Interact {
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(Size::Frac(1.0)),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::ZERO),
        scrollable: Xy::new(false, false),
    },
};
/// [`NodeParams`] for a vertical stack. 
///
/// You can use the "source" link to inspect the param values. 
pub const V_STACK: NodeParams = NodeParams {
    key: None,
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
        vertex_colors: VertexColors::flat(Color::KERU_DEBUG_RED),
    },
    interact: Interact {
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new(Size::FitContent, Size::FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::ZERO),
        scrollable: Xy::new(false, false),
    },
};
/// [`NodeParams`] for a horizontal stack. 
///
/// You can use the "source" link to inspect the param values. 
pub const H_STACK: NodeParams = NodeParams {
    key: None,
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
        vertex_colors: VertexColors::flat(Color::KERU_DEBUG_RED),
    },
    interact: Interact {
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new(Size::FitContent, Size::FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::ZERO),
        scrollable: Xy::new(false, false),
    },
};

/// [`NodeParams`] for a vertically scrollable vertical stack.
///
/// You can use the "source" link to inspect the param values. 
pub const V_SCROLL_STACK: NodeParams = NodeParams {
    key: None,
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
            vertex_colors: VertexColors::flat(Color::KERU_DEBUG_RED),
        },
        interact: Interact {
            absorbs_mouse_events: false,
            click_animation: false,
        },
        layout: Layout {
            size: Xy::new(Size::FitContent, Size::Fill),
            position: Xy::new_symm(Center),
            padding: Xy::new_symm(Len::ZERO),
            scrollable: Xy::new(false, true),
        },
    };

/// [`NodeParams`] for a margin. 
///
/// You can use the "source" link to inspect the param values. 
pub const MARGIN: NodeParams = NodeParams {
    key: None,
    stack: None,
    text_params: None,
    rect: Rect {
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: false,
        outline_only: true,
        vertex_colors: VertexColors::flat(Color::KERU_DEBUG_RED),
    },
    interact: Interact {
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(Size::Frac(0.9)),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::ZERO),
        scrollable: Xy::new(false, false),
    },
};
/// [`NodeParams`] for an icon button. 
///
/// You can use the "source" link to inspect the param values. 
pub const ICON_BUTTON: NodeParams = NodeParams {
    key: None,
    stack: None,
    text_params: None,
    rect: Rect {
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: true,
        outline_only: false,
        vertex_colors: VertexColors::KERU_GRAD,
    },
    interact: Interact {
        absorbs_mouse_events: true,
        click_animation: true,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::ZERO),
        scrollable: Xy::new(false, false),
    },
};
/// [`NodeParams`] for an icon button. 
///
/// You can use the "source" link to inspect the param values. 
pub const IMAGE: NodeParams = NodeParams {
    key: None,
    stack: None,
    text_params: None,
    rect: Rect {
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: true,
        outline_only: false,
        vertex_colors: VertexColors::flat(Color::WHITE),
    },
    interact: Interact {
        absorbs_mouse_events: true,
        click_animation: true,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::ZERO),
        scrollable: Xy::new(false, false),
    },
};
/// [`NodeParams`] for a button. 
///
/// You can use the "source" link to inspect the param values. 
pub const BUTTON: NodeParams = NodeParams {
    key: None,
    stack: None,
    text_params: Some(TextOptions {
        single_line: true,
        editable: false,
    }),
    rect: Rect {
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: true,
        outline_only: false,
        // vertex_colors: VertexColors::TEST,
        vertex_colors: VertexColors::diagonal_gradient_backslash(Color::KERU_BLUE, Color::KERU_RED),
    },
    interact: Interact {
        absorbs_mouse_events: true,
        click_animation: true,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::Pixels(10)),
        scrollable: Xy::new(false, false),
    },
};
/// [`NodeParams`] for a label. 
///
/// You can use the "source" link to inspect the param values. 
pub const LABEL: NodeParams = NodeParams {
    key: None,
    stack: None,
    text_params: Some(TextOptions {
        single_line: true,
        editable: false,
    }),
    rect: Rect {
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: true,
        outline_only: false,
        vertex_colors: VertexColors::KERU_GRAD,
    },
    interact: Interact {
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::Pixels(10)),
        scrollable: Xy::new(false, false),
    },
};

/// [`NodeParams`] for a label containing a multi-line paragraph. 
///
/// You can use the "source" link to inspect the param values. 
pub const MULTILINE_LABEL: NodeParams = NodeParams {
    key: None,
    stack: None,
    text_params: Some(TextOptions {
        single_line: false,
        editable: false,
    }),
    rect: Rect {
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: true,
        outline_only: false,
        vertex_colors: VertexColors::KERU_GRAD,
    },
    interact: Interact {
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::Pixels(10)),
        scrollable: Xy::new(false, false),
    },
};

/// [`NodeParams`] for a text element. 
///
/// You can use the "source" link to inspect the param values. 
pub const TEXT: NodeParams = NodeParams {
    key: None,
    stack: None,
    text_params: Some(TextOptions {
        single_line: true,
        editable: false,
    }),
    rect: Rect {
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: false,
        outline_only: true,
        vertex_colors: VertexColors::flat(Color::KERU_DEBUG_RED),
    },
    interact: Interact {
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::Pixels(2)),
        scrollable: Xy::new(false, false),
    },
};

/// [`NodeParams`] for a text element containing a multi-line paragraph.
///
/// You can use the "source" link to inspect the param values. 
pub const TEXT_PARAGRAPH: NodeParams = NodeParams {
    key: None,
    stack: None,
    text_params: Some(TextOptions {
        single_line: false,
        editable: false,
    }),
    rect: Rect {
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: false,
        outline_only: true,
        vertex_colors: VertexColors::flat(Color::KERU_DEBUG_RED),
    },
    interact: Interact {
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::Pixels(2)),
        scrollable: Xy::new(false, false),
    },
};


// /// [`NodeParams`] for a text_input. 
// ///
// /// You can use the "source" link to inspect the param values. 
// pub const TEXT_INPUT: NodeParams = NodeParams {
    // key: None,
//     stack: None,
//     text_params: Some(TextOptions {
//     single_line: true,
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
    // scrollable: Xy::new(false, false),
//     },
// };

/// [`NodeParams`] for a panel. 
///
/// You can use the "source" link to inspect the param values. 
pub const PANEL: NodeParams = NodeParams {
    key: None,
    stack: None,
    text_params: None,
    rect: Rect {
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: true,
        outline_only: false,
        vertex_colors: VertexColors::KERU_GRAD_FW,
    },
    interact: Interact {
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::Pixels(10)),
        scrollable: Xy::new(false, false),
    },
};

/// [`NodeParams`] for a container. 
///
/// You can use the "source" link to inspect the param values. 
pub const CONTAINER: NodeParams = NodeParams {
    key: None,
    stack: None,
    text_params: None,
    rect: Rect {
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: false,
        outline_only: true,
        vertex_colors: VertexColors::KERU_GRAD,
    },
    interact: Interact {
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(Len::Pixels(10)),
        scrollable: Xy::new(false, false),
    },
};

/// [`NodeParams`] for a custom rendered panel.
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
/// # #[node_key] const HUE_WHEEL: NodeKey;
/// #
/// let render_data = ui.get_node(HUE_WHEEL).unwrap().render_rect();
/// #
/// #   }
/// # }
/// ```
/// See the color picker in the painter example.
pub const CUSTOM_RENDERED_PANEL: NodeParams = NodeParams {
    key: None,
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
        scrollable: Xy::new(false, false),
    },
};
