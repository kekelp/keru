use crate::*;
use Size::*;
use Pos::*;

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

const DEBUG_ONLY_SHAPE: Shape = Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: DEFAULT_CORNER_RADIUS };

/// [`Node`] for a node_root_params.
pub(crate) const NODE_ROOT_PARAMS: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    text_params: None,
    stack: None,
    visible: false,
    stroke: None,
    color: VertexColors::flat(Color::TRANSPARENT),
    shape: Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: DEFAULT_CORNER_RADIUS },
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
    transform: Transform::IDENTITY,
    custom_render: false,
};
/// [`Node`] for a default.
pub const DEFAULT: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    text_params: None,
    stack: None,
    visible: true,
    stroke: None,
    color: VertexColors::flat(Color::KERU_BLUE),
    shape: Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: DEFAULT_CORNER_RADIUS },
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
    transform: Transform::IDENTITY,
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
    visible: true,
    color: VertexColors::flat(Color::TRANSPARENT),
    stroke: None,
    shape: DEBUG_ONLY_SHAPE,
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
    transform: Transform::IDENTITY,
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
    visible: true,
    color: VertexColors::flat(Color::TRANSPARENT),
    stroke: None,
    shape: DEBUG_ONLY_SHAPE,
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
    transform: Transform::IDENTITY,
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
        visible: true,
            color: VertexColors::flat(Color::TRANSPARENT),
    stroke: None,
        shape: DEBUG_ONLY_SHAPE,
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
        transform: Transform::IDENTITY,
    custom_render: false,
    };

/// [`Node`] for a margin.
pub const MARGIN: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    stack: None,
    text_params: None,
    visible: true,
        color: VertexColors::flat(Color::TRANSPARENT),
    stroke: None,
    shape: DEBUG_ONLY_SHAPE,
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
    transform: Transform::IDENTITY,
    custom_render: false,
};
/// [`Node`] for an icon button.
pub const ICON_BUTTON: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    stack: None,
    text_params: None,
    visible: true,
    stroke: None,
    color: VertexColors::flat(Color::WHITE),
    shape: Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: 0.0 },
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
    transform: Transform::IDENTITY,
    custom_render: false,
};
/// [`Node`] for an image. 
pub const IMAGE: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    stack: None,
    text_params: None,
    visible: true,
    stroke: None,
    color: VertexColors::flat(Color::WHITE),
    shape: Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: DEFAULT_CORNER_RADIUS },
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
    transform: Transform::IDENTITY,
    custom_render: false,
};
/// [`Node`] for an icon button. 
pub const IMAGE_BUTTON: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    stack: None,
    text_params: None,
    visible: true,
    stroke: None,
    color: VertexColors::flat(Color::WHITE),
    shape: Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: DEFAULT_CORNER_RADIUS },
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
    transform: Transform::IDENTITY,
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
    visible: true,
    stroke: None,
    color: VertexColors::flat(Color::KERU_PINK),
    shape: Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: DEFAULT_CORNER_RADIUS },
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
    transform: Transform::IDENTITY,
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
    visible: true,
    stroke: None,
    color: VertexColors::KERU_GRAD,
    shape: Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: DEFAULT_CORNER_RADIUS },
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
    transform: Transform::IDENTITY,
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
    visible: true,
    stroke: None,
    color: VertexColors::KERU_GRAD,
    shape: Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: DEFAULT_CORNER_RADIUS },
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
    transform: Transform::IDENTITY,
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
    visible: true,
        color: VertexColors::flat(Color::TRANSPARENT),
    stroke: None,
    shape: DEBUG_ONLY_SHAPE,
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
    transform: Transform::IDENTITY,
    custom_render: false,
};

/// [`Node`] for an icon element.
pub const ICON: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    stack: None,
    text_params: None,
    visible: true,
    stroke: None,
    color: VertexColors::flat(Color::WHITE),
    shape: Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: 0.0 },
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
    transform: Transform::IDENTITY,
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
    visible: true,
    stroke: None,
    color: VertexColors::flat(Color::GREY),
    shape: Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: DEFAULT_CORNER_RADIUS },
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
    transform: Transform::IDENTITY,
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
    visible: true,
    stroke: None,
    color: VertexColors::flat(Color::GREY),
    shape: Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: DEFAULT_CORNER_RADIUS },
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
    transform: Transform::IDENTITY,
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
    visible: true,
        color: VertexColors::flat(Color::TRANSPARENT),
    stroke: None,
    shape: DEBUG_ONLY_SHAPE,
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
    transform: Transform::IDENTITY,
    custom_render: false,
};

/// [`Node`] for a panel. 
pub const PANEL: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    stack: None,
    text_params: None,
    visible: true,
    stroke: None,
    color: VertexColors::KERU_GRAD_FW,
    shape: Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: DEFAULT_CORNER_RADIUS },
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
    transform: Transform::IDENTITY,
    custom_render: false,
};

/// [`Node`] for a container. 
pub const CONTAINER: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    stack: None,
    text_params: None,
    visible: false,
    stroke: None,
    color: VertexColors::flat(Color::TRANSPARENT),
    shape: Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: DEFAULT_CORNER_RADIUS },
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: false,
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
    transform: Transform::IDENTITY,
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
    visible: false,
    stroke: None,
    color: VertexColors::GREENSCREEN,
    shape: Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: DEFAULT_CORNER_RADIUS },
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
    transform: Transform::IDENTITY,
    custom_render: false,
};

/// [`Node`] for a spacer element.
pub const SPACER: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    stack: None,
    text_params: None,
    visible: true,
        color: VertexColors::flat(Color::TRANSPARENT),
    stroke: None,
    shape: DEBUG_ONLY_SHAPE,
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
    transform: Transform::IDENTITY,
    custom_render: false,
};

/// [`Node`] for a invisible spacer element that fills all the available space in the X direction.
pub const H_SPACER: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    stack: None,
    text_params: None,
        color: VertexColors::flat(Color::TRANSPARENT),
    stroke: None,
    shape: DEBUG_ONLY_SHAPE,
    visible: true,
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
    transform: Transform::IDENTITY,
    custom_render: false,
};

/// [`Node`] for a invisible spacer element that fills all the available space in the Y direction.
pub const V_SPACER: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    stack: None,
    text_params: None,
    visible: true,
        color: VertexColors::flat(Color::TRANSPARENT),
    stroke: None,
    shape: DEBUG_ONLY_SHAPE,
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
    transform: Transform::IDENTITY,
    custom_render: false,
};

/// [`Node`] for a horizontal divider line.
pub const H_LINE: Node = Node {
    animation: NO_ANIMATION,
    key: None,
    stack: None,
    text_params: None,
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
    color: VertexColors::flat(Color::TRANSPARENT),
    shape: Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: DEFAULT_CORNER_RADIUS },
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
    transform: Transform::IDENTITY,
    custom_render: false,
};

pub(crate) const COMPONENT_ROOT: Node = Node {
    // todo remove
    animation: Animation { speed: 1.0, enter: EnterAnimation::None, exit: ExitAnimation::None, state_transition: StateTransition { animate_position: true } },
    key: None,
    stack: None,
    text_params: None,
    visible: false,
    stroke: None,
    color: VertexColors::flat(Color::KERU_DEBUG_BLUE),
    shape: Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: DEFAULT_CORNER_RADIUS },
    interact: Interact {
        senses: Sense::NONE,
        absorbs_mouse_events: false,
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
    transform: Transform::IDENTITY,
    custom_render: false,
};
