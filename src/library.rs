use crate::*;
use Size::*;
use Position::*;
use Shape::*;

// todo: not very discoverable from docs. there's a list of constants on the main page, maybe that's good? link to that or something?

/// [`NodeParams`] for a node_root_params. 
pub(crate) const NODE_ROOT_PARAMS: NodeParams = NodeParams {
    key: None,
    stack: None,
    text_params: None,
    rect: Rect {
        rounded_corners: RoundedCorners::ALL,
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: false,
        outline_only: true,
        vertex_colors: VertexColors::flat(Color::TRANSPARENT)
    },
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(Size::Frac(1.0)),
        position: Xy::new_symm(Start),
        padding: Xy::new_symm(0),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: false,
};
/// [`NodeParams`] for a default. 
pub const DEFAULT: NodeParams = NodeParams {
    key: None,
    stack: None,
    text_params: None,
    rect: Rect {
        rounded_corners: RoundedCorners::ALL,
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: true,
        outline_only: false,
        vertex_colors: VertexColors::flat(Color::KERU_BLUE),
    },
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(Size::Frac(1.0)),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(0),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: false,
};
/// [`NodeParams`] for a vertical stack. 
pub const V_STACK: NodeParams = NodeParams {
    key: None,
    stack: Some(Stack {
        arrange: Arrange::Center,
        axis: Axis::Y,
        spacing: 10,
    }),
    text_params: None,
    rect: Rect {
        rounded_corners: RoundedCorners::ALL,
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: false,
        outline_only: true,
        vertex_colors: VertexColors::flat(Color::KERU_DEBUG_RED),
    },
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new(Size::FitContent, Size::FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(0),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: false,
};
/// [`NodeParams`] for a horizontal stack. 
pub const H_STACK: NodeParams = NodeParams {
    key: None,
    stack: Some(Stack {
        arrange: Arrange::Center,
        axis: Axis::X,
        spacing: 5,
    }),
    text_params: None,
    rect: Rect {
        rounded_corners: RoundedCorners::ALL,
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: false,
        outline_only: true,
        vertex_colors: VertexColors::flat(Color::KERU_DEBUG_RED),
    },
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new(Size::FitContent, Size::FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(0),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: false,
};

/// [`NodeParams`] for a vertically scrollable vertical stack.
pub const V_SCROLL_STACK: NodeParams = NodeParams {
    key: None,
        stack: Some(Stack {
            arrange: Arrange::Center,
            axis: Axis::Y,
            spacing: 10,
        }),
        text_params: None,
        rect: Rect {
            rounded_corners: RoundedCorners::ALL,
            shape: Rectangle { corner_radius: BASE_RADIUS },
            visible: false,
            outline_only: true,
            vertex_colors: VertexColors::flat(Color::KERU_DEBUG_RED),
        },
        interact: Interact {
            senses: Sense::NONE,
            absorbs_mouse_events: false,
            click_animation: false,
        },
        layout: Layout {
            size: Xy::new(Size::FitContent, Size::Fill),
            position: Xy::new_symm(Center),
            padding: Xy::new_symm(0),
            scrollable: Xy::new(false, true),
        },
        children_can_hide: false,
    };

/// [`NodeParams`] for a margin. 
pub const MARGIN: NodeParams = NodeParams {
    key: None,
    stack: None,
    text_params: None,
    rect: Rect {
        rounded_corners: RoundedCorners::ALL,
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: false,
        outline_only: true,
        vertex_colors: VertexColors::flat(Color::KERU_DEBUG_RED),
    },
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(Size::Frac(0.9)),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(0),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: false,
};
/// [`NodeParams`] for an icon button. 
pub const ICON_BUTTON: NodeParams = NodeParams {
    key: None,
    stack: None,
    text_params: None,
    rect: Rect {
        rounded_corners: RoundedCorners::ALL,
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: true,
        outline_only: false,
        vertex_colors: VertexColors::KERU_GRAD,
    },
    interact: Interact {
        senses: Sense::CLICK,
        absorbs_mouse_events: true,
        click_animation: true,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(0),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: false,
};
/// [`NodeParams`] for an image. 
pub const IMAGE: NodeParams = NodeParams {
    key: None,
    stack: None,
    text_params: None,
    rect: Rect {
        rounded_corners: RoundedCorners::ALL,
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: true,
        outline_only: true,
        vertex_colors: VertexColors::flat(Color::WHITE),
    },
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: true,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(0),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: false,
};
/// [`NodeParams`] for an icon button. 
pub const IMAGE_BUTTON: NodeParams = NodeParams {
    key: None,
    stack: None,
    text_params: None,
    rect: Rect {
        rounded_corners: RoundedCorners::ALL,
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: true,
        outline_only: false,
        vertex_colors: VertexColors::flat(Color::WHITE),
    },
    interact: Interact {
        senses: Sense::CLICK,
        absorbs_mouse_events: true,
        click_animation: true,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(0),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: false,
};
/// [`NodeParams`] for a button. 
pub const BUTTON: NodeParams = NodeParams {
    key: None,
    stack: None,
    text_params: Some(TextOptions {
        single_line: true,
        editable: false,
    }),
    rect: Rect {
        rounded_corners: RoundedCorners::ALL,
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: true,
        outline_only: false,
        // vertex_colors: VertexColors::TEST,
        vertex_colors: VertexColors::diagonal_gradient_backslash(Color::KERU_BLUE, Color::KERU_RED),
    },
    interact: Interact {
        senses: Sense::CLICK,
        absorbs_mouse_events: true,
        click_animation: true,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(10),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: false,
};
/// [`NodeParams`] for a label. 
pub const LABEL: NodeParams = NodeParams {
    key: None,
    stack: None,
    text_params: Some(TextOptions {
        single_line: true,
        editable: false,
    }),
    rect: Rect {
        rounded_corners: RoundedCorners::ALL,
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: true,
        outline_only: false,
        vertex_colors: VertexColors::KERU_GRAD,
    },
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: true,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(10),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: false,
};

/// [`NodeParams`] for a label containing a multi-line paragraph. 
pub const MULTILINE_LABEL: NodeParams = NodeParams {
    key: None,
    stack: None,
    text_params: Some(TextOptions {
        single_line: false,
        editable: false,
    }),
    rect: Rect {
        rounded_corners: RoundedCorners::ALL,
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: true,
        outline_only: false,
        vertex_colors: VertexColors::KERU_GRAD,
    },
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: true,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(10),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: false,
};

/// [`NodeParams`] for a text element. 
pub const TEXT: NodeParams = NodeParams {
    key: None,
    stack: None,
    text_params: Some(TextOptions {
        single_line: true,
        editable: false,
    }),
    rect: Rect {
        rounded_corners: RoundedCorners::ALL,
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: false,
        outline_only: true,
        vertex_colors: VertexColors::flat(Color::KERU_DEBUG_RED),
    },
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(2),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: false,
};

/// [`NodeParams`] for a text element containing a multi-line paragraph.
pub const TEXT_PARAGRAPH: NodeParams = NodeParams {
    key: None,
    stack: None,
    text_params: Some(TextOptions {
        single_line: false,
        editable: false,
    }),
    rect: Rect {
        rounded_corners: RoundedCorners::ALL,
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: false,
        outline_only: true,
        vertex_colors: VertexColors::flat(Color::KERU_DEBUG_RED),
    },
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(2),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: false,
};

/// [`NodeParams`] for a panel. 
pub const PANEL: NodeParams = NodeParams {
    key: None,
    stack: None,
    text_params: None,
    rect: Rect {
        rounded_corners: RoundedCorners::ALL,
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: true,
        outline_only: false,
        vertex_colors: VertexColors::KERU_GRAD_FW,
    },
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: true,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(10),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: false,
};

/// [`NodeParams`] for a container. 
pub const CONTAINER: NodeParams = NodeParams {
    key: None,
    stack: None,
    text_params: None,
    rect: Rect {
        rounded_corners: RoundedCorners::ALL,
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: false,
        outline_only: true,
        vertex_colors: VertexColors::KERU_GRAD,
    },
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: true,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(10),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: false,
};

/// [`NodeParams`] for a custom rendered panel.
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
        rounded_corners: RoundedCorners::ALL,
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: false,
        outline_only: false,
        vertex_colors: VertexColors::GREENSCREEN,
    },
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: true,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(0),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: false,
};

/// [`NodeParams`] for a spacer element.
pub const SPACER: NodeParams = NodeParams {
    key: None,
    stack: None,
    text_params: None,
    rect: Rect {
        rounded_corners: RoundedCorners::ALL,
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: false,
        outline_only: true,
        vertex_colors: VertexColors::flat(Color::KERU_DEBUG_RED),
    },
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(Fill),
        position: Xy::new_symm(Center),
        padding: Xy::new_symm(0),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: false,
};
