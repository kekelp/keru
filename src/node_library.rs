use crate::*;
use Size::*;
use Position::*;
use Shape::*;

// todo remove this
// TODO: Re-add Cap and Join when implementing stroke features in keru_draw
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cap {
    Butt,
    Round,
    Square,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Join {
    Miter,
    Round,
    Bevel,
}

// todo: not very discoverable from docs. there's a list of constants on the main page, maybe that's good? link to that or something?

pub const ICON_RIGHT: &[u8] = include_bytes!("svg_icons/right.svg");
pub const ICON_LEFT: &[u8] = include_bytes!("svg_icons/left.svg");
pub const ICON_PLUS: &[u8] = include_bytes!("svg_icons/plus.svg");
pub const ICON_MINUS: &[u8] = include_bytes!("svg_icons/minus.svg");
pub const ICON_DELETE: &[u8] = include_bytes!("svg_icons/delete.svg");
pub const ICON_EDIT: &[u8] = include_bytes!("svg_icons/pencil.svg");

const DEBUG_ONLY_RECT: Rect = Rect {
    rounded_corners: RoundedCorners::ALL,
    shape: Rectangle { corner_radius: BASE_RADIUS },
    visible: false,
    stroke: Some(Stroke::new(4.0).with_color(Color::KERU_DEBUG_RED)),
    vertex_colors: VertexColors::flat(Color::TRANSPARENT),
};

/// [`Node`] for a node_root_params.
pub(crate) const NODE_ROOT_PARAMS: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    text_params: None,
    stack: None,
    rect: Rect {
        rounded_corners: RoundedCorners::ALL,
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: false,
        stroke: None,
        vertex_colors: VertexColors::flat(Color::TRANSPARENT)
    },
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(Size::Frac(1.0)),
        padding: Xy::new_symm(0.0),
        position: Xy::new_symm(Start),
        anchor: Xy::new_symm(Anchor::Start),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: ChildrenCanHide::Inherit,
    clip_children: Xy::new(false, false),
    translate: None,
    scale: None,
    custom_render: false,
};
/// [`Node`] for a default.
pub const DEFAULT: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    text_params: None,
    stack: None,
    rect: Rect {
        rounded_corners: RoundedCorners::ALL,
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: true,
        stroke: None,
        vertex_colors: VertexColors::flat(Color::KERU_BLUE),
    },
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(Size::Frac(1.0)),
        padding: Xy::new_symm(0.0),
        position: Xy::new_symm(Center),
        anchor: Xy::new_symm(Anchor::Start),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: ChildrenCanHide::Inherit,
    clip_children: Xy::new(false, false),
    translate: None,
    scale: None,
    custom_render: false,
};
/// [`Node`] for a vertical stack.
pub const V_STACK: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    text_params: None,
    stack: Some(Stack {
        arrange: Arrange::Center,
        axis: Axis::Y,
        spacing: 8.0,
    }),
    rect: DEBUG_ONLY_RECT,
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new(Size::FitContent, Size::FitContent),
        padding: Xy::new_symm(0.0),
        position: Xy::new_symm(Center),
        anchor: Xy::new_symm(Anchor::Start),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: ChildrenCanHide::Inherit,
    clip_children: Xy::new(false, false),
    translate: None,
    scale: None,
    custom_render: false,
};
/// [`Node`] for a horizontal stack.
pub const H_STACK: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    text_params: None,
    stack: Some(Stack {
        arrange: Arrange::Center,
        axis: Axis::X,
        spacing: 8.0,
    }),
    rect: DEBUG_ONLY_RECT,
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new(Size::FitContent, Size::FitContent),
        padding: Xy::new_symm(0.0),
        position: Xy::new_symm(Center),
        anchor: Xy::new_symm(Anchor::Start),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: ChildrenCanHide::Inherit,
    clip_children: Xy::new(false, false),
    translate: None,
    scale: None,
    custom_render: false,
};

/// [`Node`] for a vertically scrollable vertical stack.
pub const V_SCROLL_STACK: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    text_params: None,
        stack: Some(Stack {
            arrange: Arrange::Start,
            axis: Axis::Y,
            spacing: 10.0,
        }),
        rect: Rect {
            rounded_corners: RoundedCorners::ALL,
            shape: Rectangle { corner_radius: BASE_RADIUS },
            visible: false,
            stroke: Some(Stroke::new(4.0).with_color(Color::KERU_DEBUG_RED)),
            vertex_colors: VertexColors::flat(Color::TRANSPARENT),
        },
        interact: Interact {
            senses: Sense::NONE,
            absorbs_mouse_events: false,
            click_animation: false,
        },
        layout: Layout {
            size: Xy::new(Size::FitContent, Size::Fill),
            padding: Xy::new_symm(0.0),
            position: Xy::new_symm(Center),
            anchor: Xy::new_symm(Anchor::Start),
            scrollable: Xy::new(false, true),
        },
        children_can_hide: ChildrenCanHide::Inherit,
        clip_children: Xy::new(false, true),
        translate: None,
    scale: None,
    custom_render: false,
    };

/// [`Node`] for a margin.
pub const MARGIN: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    stack: None,
    text_params: None,
    rect: DEBUG_ONLY_RECT,
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(Size::Frac(0.9)),
        padding: Xy::new_symm(0.0),
        position: Xy::new_symm(Center),
        anchor: Xy::new_symm(Anchor::Start),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: ChildrenCanHide::Inherit,
    clip_children: Xy::new(false, false),
    translate: None,
    scale: None,
    custom_render: false,
};
/// [`Node`] for an icon button.
pub const ICON_BUTTON: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    stack: None,
    text_params: None,
    rect: Rect {
        rounded_corners: RoundedCorners::ALL,
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: true,
        stroke: None,
        vertex_colors: VertexColors::KERU_GRAD,
    },
    interact: Interact {
        senses: Sense::CLICK.union(Sense::CLICK_RELEASE),
        absorbs_mouse_events: true,
        click_animation: true,
    },
    layout: Layout {
        size: Xy::new_symm(Size::Pixels(40.0)),
        padding: Xy::new_symm(2.0),
        position: Xy::new_symm(Center),
        anchor: Xy::new_symm(Anchor::Start),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: ChildrenCanHide::Inherit,
    clip_children: Xy::new(false, false),
    translate: None,
    scale: None,
    custom_render: false,
};
/// [`Node`] for an image. 
pub const IMAGE: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    stack: None,
    text_params: None,
    rect: Rect {
        rounded_corners: RoundedCorners::ALL,
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: true,
        stroke: None,
        vertex_colors: VertexColors::flat(Color::WHITE),
    },
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: true,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        padding: Xy::new_symm(0.0),
        position: Xy::new_symm(Center),
        anchor: Xy::new_symm(Anchor::Start),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: ChildrenCanHide::Inherit,
    clip_children: Xy::new(false, false),
    translate: None,
    scale: None,
    custom_render: false,
};
/// [`Node`] for an icon button. 
pub const IMAGE_BUTTON: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    stack: None,
    text_params: None,
    rect: Rect {
        rounded_corners: RoundedCorners::ALL,
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: true,
        stroke: None,
        vertex_colors: VertexColors::flat(Color::WHITE),
    },
    interact: Interact {
        senses: Sense::CLICK.union(Sense::CLICK_RELEASE),
        absorbs_mouse_events: true,
        click_animation: true,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        padding: Xy::new_symm(0.0),
        position: Xy::new_symm(Center),
        anchor: Xy::new_symm(Anchor::Start),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: ChildrenCanHide::Inherit,
    clip_children: Xy::new(false, false),
    translate: None,
    scale: None,
    custom_render: false,
};
/// [`Node`] for a button. 
pub const BUTTON: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    stack: None,
    text_params: Some(TextOptions {
        single_line: true,
        editable: false,
        selectable: false,
        edit_disabled: false,
    }),
    rect: Rect {
        rounded_corners: RoundedCorners::ALL,
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: true,
        stroke: None,
        // vertex_colors: VertexColors::TEST,
        vertex_colors: VertexColors::diagonal_gradient_backslash(Color::KERU_BLUE, Color::KERU_RED),
    },
    interact: Interact {
        senses: Sense::CLICK.union(Sense::CLICK_RELEASE),
        absorbs_mouse_events: true,
        click_animation: true,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        padding: Xy::new_symm(10.0),
        position: Xy::new_symm(Center),
        anchor: Xy::new_symm(Anchor::Start),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: ChildrenCanHide::Inherit,
    clip_children: Xy::new(false, false),
    translate: None,
    scale: None,
    custom_render: false,
};
/// [`Node`] for a label. 
pub const LABEL: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    stack: None,
    text_params: Some(TextOptions {
        single_line: true,
        editable: false,
        selectable: true,
        edit_disabled: false,
    }),
    rect: Rect {
        rounded_corners: RoundedCorners::ALL,
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: true,
        stroke: None,
        vertex_colors: VertexColors::KERU_GRAD,
    },
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: true,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        padding: Xy::new_symm(10.0),
        position: Xy::new_symm(Center),
        anchor: Xy::new_symm(Anchor::Start),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: ChildrenCanHide::Inherit,
    clip_children: Xy::new(false, false),
    translate: None,
    scale: None,
    custom_render: false,
};

/// [`Node`] for a label containing a multi-line paragraph. 
pub const MULTILINE_LABEL: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    stack: None,
    text_params: Some(TextOptions {
        single_line: false,
        editable: false,
        selectable: true,
        edit_disabled: false,
    }),
    rect: Rect {
        rounded_corners: RoundedCorners::ALL,
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: true,
        stroke: None,
        vertex_colors: VertexColors::KERU_GRAD,
    },
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: true,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        padding: Xy::new_symm(10.0),
        position: Xy::new_symm(Center),
        anchor: Xy::new_symm(Anchor::Start),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: ChildrenCanHide::Inherit,
    clip_children: Xy::new(false, false),
    translate: None,
    scale: None,
    custom_render: false,
};

/// [`Node`] for a text element. 
pub const TEXT: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    stack: None,
    text_params: Some(TextOptions {
        single_line: true,
        editable: false,
        selectable: true,
        edit_disabled: false,
    }),
    rect: DEBUG_ONLY_RECT,
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        padding: Xy::new_symm(2.0),
        position: Xy::new_symm(Center),
        anchor: Xy::new_symm(Anchor::Start),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: ChildrenCanHide::Inherit,
    clip_children: Xy::new(false, false),
    translate: None,
    scale: None,
    custom_render: false,
};

/// [`Node`] for an icon element.
pub const ICON: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    stack: None,
    text_params: None,
    rect: DEBUG_ONLY_RECT,
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(Size::Pixels(40.0)),
        padding: Xy::new_symm(2.0),
        position: Xy::new_symm(Center),
        anchor: Xy::new_symm(Anchor::Start),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: ChildrenCanHide::Inherit,
    clip_children: Xy::new(false, false),
    translate: None,
    scale: None,
    custom_render: false,
};

/// [`Node`] for a multiline text edit box. 
pub const TEXT_EDIT: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    stack: None,
    text_params: Some(TextOptions {
        single_line: false,
        editable: true,
        selectable: true,
        edit_disabled: false,
    }),
    rect: Rect {
        rounded_corners: RoundedCorners::ALL,
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: true,
        stroke: None,
        vertex_colors: VertexColors::flat(Color::GREY),
    },
    interact: Interact {
        senses: Sense::CLICK,
        absorbs_mouse_events: true,
        click_animation: true,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        padding: Xy::new_symm(10.0),
        position: Xy::new_symm(Center),
        anchor: Xy::new_symm(Anchor::Start),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: ChildrenCanHide::Inherit,
    clip_children: Xy::new(false, false),
    translate: None,
    scale: None,
    custom_render: false,
};

/// [`Node`] for a single line text edit box. 
pub const TEXT_EDIT_LINE: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    stack: None,
    text_params: Some(TextOptions {
        single_line: true,
        editable: true,
        selectable: true,
        edit_disabled: false,
    }),
    rect: Rect {
        rounded_corners: RoundedCorners::ALL,
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: true,
        stroke: None,
        vertex_colors: VertexColors::flat(Color::GREY),
    },
    interact: Interact {
        senses: Sense::CLICK,
        absorbs_mouse_events: true,
        click_animation: true,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        padding: Xy::new_symm(10.0),
        position: Xy::new_symm(Center),
        anchor: Xy::new_symm(Anchor::Start),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: ChildrenCanHide::Inherit,
    clip_children: Xy::new(false, false),
    translate: None,
    scale: None,
    custom_render: false,
};

/// [`Node`] for a text element containing a multi-line paragraph.
pub const TEXT_PARAGRAPH: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    stack: None,
    text_params: Some(TextOptions {
        single_line: false,
        editable: false,
        selectable: true,
        edit_disabled: false,
    }),
    rect: DEBUG_ONLY_RECT,
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        padding: Xy::new_symm(2.0),
        position: Xy::new_symm(Center),
        anchor: Xy::new_symm(Anchor::Start),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: ChildrenCanHide::Inherit,
    clip_children: Xy::new(false, false),
    translate: None,
    scale: None,
    custom_render: false,
};

/// [`Node`] for a panel. 
pub const PANEL: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    stack: None,
    text_params: None,
    rect: Rect {
        rounded_corners: RoundedCorners::ALL,
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: true,
        stroke: None,
        vertex_colors: VertexColors::KERU_GRAD_FW,
    },
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: true,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        padding: Xy::new_symm(10.0),
        position: Xy::new_symm(Center),
        anchor: Xy::new_symm(Anchor::Start),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: ChildrenCanHide::Inherit,
    clip_children: Xy::new(false, false),
    translate: None,
    scale: None,
    custom_render: false,
};

/// [`Node`] for a container. 
pub const CONTAINER: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    stack: None,
    text_params: None,
    rect: Rect {
        rounded_corners: RoundedCorners::ALL,
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: false,
        stroke: None,
        vertex_colors: VertexColors::flat(Color::TRANSPARENT),
    },
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: true,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        padding: Xy::new_symm(10.0),
        position: Xy::new_symm(Center),
        anchor: Xy::new_symm(Anchor::Start),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: ChildrenCanHide::Inherit,
    clip_children: Xy::new(false, false),
    translate: None,
    scale: None,
    custom_render: false,
};

/// [`Node`] for a custom rendered node.
/// 
/// Use [`Ui::render_rect`] to get the render data for your node, then render it in a separate draw call.
/// ```no_run
/// # use keru::*;
/// # let mut ui: Ui = unimplemented!();
/// #
/// #[node_key] const CUSTOM_RENDERED_NODE: NodeKey;
/// let render_rect = ui.render_rect(CUSTOM_RENDERED_NODE);
/// ```
/// See the color picker in the painter example.
pub const CUSTOM_RENDERED_PANEL: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    stack: None,
    text_params: None,
    rect: Rect {
        rounded_corners: RoundedCorners::ALL,
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: false,
        stroke: None,
        vertex_colors: VertexColors::GREENSCREEN,
    },
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: true,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        padding: Xy::new_symm(0.0),
        position: Xy::new_symm(Center),
        anchor: Xy::new_symm(Anchor::Start),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: ChildrenCanHide::Inherit,
    clip_children: Xy::new(false, false),
    translate: None,
    scale: None,
    custom_render: false,
};

/// [`Node`] for a spacer element.
pub const SPACER: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    stack: None,
    text_params: None,
    rect: DEBUG_ONLY_RECT,
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(Fill),
        padding: Xy::new_symm(0.0),
        position: Xy::new_symm(Center),
        anchor: Xy::new_symm(Anchor::Start),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: ChildrenCanHide::Inherit,
    clip_children: Xy::new(false, false),
    translate: None,
    scale: None,
    custom_render: false,
};

/// [`Node`] for a invisible spacer element that fills all the available space in the X direction.
pub const H_SPACER: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    stack: None,
    text_params: None,
    rect: DEBUG_ONLY_RECT,
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new(Size::Fill, Size::FitContent),
        padding: Xy::new_symm(0.0),
        position: Xy::new_symm(Center),
        anchor: Xy::new_symm(Anchor::Start),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: ChildrenCanHide::Inherit,
    clip_children: Xy::new(false, false),
    translate: None,
    scale: None,
    custom_render: false,
};

/// [`Node`] for a invisible spacer element that fills all the available space in the Y direction.
pub const V_SPACER: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    stack: None,
    text_params: None,
    rect: DEBUG_ONLY_RECT,
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new(Size::FitContent, Size::Fill),
        padding: Xy::new_symm(0.0),
        position: Xy::new_symm(Center),
        anchor: Xy::new_symm(Anchor::Start),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: ChildrenCanHide::Inherit,
    clip_children: Xy::new(false, false),
    translate: None,
    scale: None,
    custom_render: false,
};

/// [`Node`] for a horizontal divider line.
pub const H_LINE: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    stack: None,
    text_params: None,
    rect: Rect {
        rounded_corners: RoundedCorners::ALL,
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: true,
        stroke: Some(Stroke {
            width: 2.0,
            color: Color::WHITE,
            join: Join::Miter,
            miter_limit: 4.0,
            start_cap: Cap::Round,
            end_cap: Cap::Round,
            dash_length: 0.0,
            dash_offset: 0.0,
        }),
        vertex_colors: VertexColors::flat(Color::TRANSPARENT),
    },
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: false,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new(Size::Fill, Size::Pixels(0.0)),
        padding: Xy::new_symm(0.0),
        position: Xy::new_symm(Center),
        anchor: Xy::new_symm(Anchor::Start),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: ChildrenCanHide::Inherit,
    clip_children: Xy::new(false, false),
    translate: None,
    scale: None,
    custom_render: false,
};

pub(crate) const COMPONENT_ROOT: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    stack: None,
    text_params: None,
    rect: Rect {
        rounded_corners: RoundedCorners::ALL,
        shape: Rectangle { corner_radius: BASE_RADIUS },
        visible: false,
        stroke: None,
        vertex_colors: VertexColors::flat(Color::KERU_DEBUG_BLUE),
    },
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: true,
        click_animation: false,
    },
    layout: Layout {
        size: Xy::new_symm(FitContent),
        padding: Xy::new_symm(0.0),
        position: Xy::new_symm(Center),
        anchor: Xy::new_symm(Anchor::Start),
        scrollable: Xy::new(false, false),
    },
    children_can_hide: ChildrenCanHide::Inherit,
    clip_children: Xy::new(false, false),
    translate: None,
    scale: None,
    custom_render: false,
};
