use keru_draw::StyleHandle;

use crate::*;
use crate::color::*;
use std::{hash::{Hash, Hasher}, ops::Deref};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChildrenCanHide {
    Yes,
    No,
    Inherit,
}

/// A struct describing the params of a GUI node.
/// 
/// Pass it to [`Ui::add`] to create a node with the given params:
/// ```rust
/// # use keru::*;
/// # pub struct State {
/// #     pub ui: Ui,
/// # }
/// #
/// # impl State {
/// #    fn declare_ui(&mut self) {
/// #    let ui = &mut self.ui; 
/// #
/// # #[node_key] const INCREASE: NodeKey;
/// # const MY_BUTTON: Node = keru::BUTTON
/// #     .color(Color::RED)
/// #     .shape(Shape::Circle); 
/// ui.add(MY_BUTTON);
/// #
/// #   }
/// # }
/// ```
/// 
///  You can start with one of the preset constants ([`BUTTON`], [`LABEL`], [`TEXT`], ...), then use the builder methods to customize it:
/// 
/// ```rust
/// # use keru::*;
/// const MY_BUTTON: Node = keru::BUTTON
///     .color(Color::RED)
///     .shape(Shape::Circle); 
/// ```
/// 
/// [`Node`] is a plain-old-data struct. Methods like [`Self::text()`] allow to associate borrowed data like a `&str` to a [`Node`].
/// 
/// The result is a [`FullNode`], a version of this struct that can hold borrowed data. Both versions can be used in the same ways.
#[derive(Debug, Copy, Clone)]
pub struct Node {
    pub key: Option<NodeKey>,
    pub text_params: Option<TextOptions>,
    pub stack: Option<Stack>,
    pub rect: Rect,
    pub interact: Interact,
    pub layout: Layout,
    pub children_can_hide: ChildrenCanHide,
    pub clip_children: Xy<bool>,
    pub animation: Animation,
    pub translate: Option<(f32, f32)>,
    pub scale: Option<(f32, f32)>,
    pub custom_render: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SlideEdge {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SlideDirection {
    In,
    Out,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum EnterAnimation {
    None,
    Slide { edge: SlideEdge, direction: SlideDirection },
    GrowShrink { axis: Axis, origin: Position },
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ExitAnimation {
    None,
    Slide { edge: SlideEdge, direction: SlideDirection },
    GrowShrink { axis: Axis, origin: Position },
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StateTransition {
    // For now, just position-based transitions (placeholder)
    pub animate_position: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct Animation {
    pub speed: f32,
    pub enter: EnterAnimation,
    pub exit: ExitAnimation,
    pub state_transition: StateTransition,
}

pub const NO_ANIMATION: Animation = Animation {
    speed: 1.0,
    enter: EnterAnimation::None,
    exit: ExitAnimation::None,
    state_transition: StateTransition {
        animate_position: false,
    },
};

/// A node's size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Size {
    Pixels(u32),
    Frac(f32),
    Fill,
    FitContent,
    AspectRatio(f32),
}

// Get a load of this crap that I have to write
impl Hash for Size {
    fn hash<H: Hasher>(&self, state: &mut H) {
        use Size::*;
        match self {
            Pixels(len) => (0u8, len).hash(state),
            Frac(len) => (1u8, len.to_bits()).hash(state),
            Fill => 2u8.hash(state),
            FitContent => 3u8.hash(state),
            AspectRatio(ratio) => (5u8, ratio.to_bits()).hash(state),
        }
    }
}

/// Anchor point within a node for positioning.
///
/// Determines which point of the node is positioned at the given coordinates
/// when using `Position::Static`. For example, with `Anchor::Center`, the
/// center of the node will be placed at the specified position.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Anchor {
    /// Anchor at the start (left for X, top for Y)
    Start,
    /// Anchor at the center
    Center,
    /// Anchor at the end (right for X, bottom for Y)
    End,
    /// Anchor at a relative position (0.0 = start, 1.0 = end)
    Frac(f32),
}

impl Hash for Anchor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Anchor::Start => 0u8.hash(state),
            Anchor::Center => 1u8.hash(state),
            Anchor::End => 2u8.hash(state),
            Anchor::Frac(f) => {
                3u8.hash(state);
                f.to_bits().hash(state);
            }
        }
    }
}

/// A node's position relative to its parent.
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub enum Position {
    Center,
    Start,
    End,
    // todo: this should be named "Fixed", but the name conflicts with Size when exporting everything together...
    // FixedPos and FixedSize??
    // besides, this is missing anchors and the "self center"
    Static(Len),
}

/// Options for stack container nodes.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Hash)]
pub struct Stack {
    pub arrange: Arrange,
    pub axis: Axis,
    pub spacing: u32,
}
impl Stack {
    pub const DEFAULT: Stack = Stack {
        arrange: Arrange::Center,
        axis: Axis::Y,
        spacing: 5,
    };
    pub const fn arrange(mut self, arrange: Arrange) -> Self {
        self.arrange = arrange;
        return self;
    }
    pub const fn spacing(mut self, spacing: u32) -> Self {
        self.spacing = spacing;
        return self;
    }
    pub const fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        return self;
    }
}

/// Options for the arrangement of child nodes within a stack node.
#[derive(Debug, Clone, Copy, Hash)]
pub enum Arrange {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

// might as well move to Rect? but maybe there's issues with non-clickable stuff absorbing the clicks.
/// The node's interact behavior.
#[derive(Debug, Copy, Clone, Hash)]
pub struct Interact {
    /// Whether the node displays the default animation when clicked and hovered.
    pub click_animation: bool,
    /// Whether the node consumes mouse events, or is transparent to them.
    pub absorbs_mouse_events: bool,
    /// Which types of input the node can respond to
    pub senses: Sense,
}

/// The node's layout, size and position.
#[derive(Debug, Copy, Clone, PartialEq, Hash)]
pub struct Layout {
    pub size: Xy<Size>,
    pub padding: Xy<u32>,
    pub position: Xy<Position>,
    pub anchor: Xy<Anchor>,
    pub scrollable: Xy<bool>,
}

bitflags::bitflags! {
    /// A bitflag struct defining which corners of a rectangle are rounded
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct RoundedCorners: u8 {
        const TOP_RIGHT    = 1 << 0;
        const TOP_LEFT     = 1 << 1;
        const BOTTOM_RIGHT = 1 << 2;
        const BOTTOM_LEFT  = 1 << 3;
        
        const TOP          = Self::TOP_LEFT.bits() | Self::TOP_RIGHT.bits();
        const BOTTOM       = Self::BOTTOM_LEFT.bits() | Self::BOTTOM_RIGHT.bits();
        const LEFT         = Self::TOP_LEFT.bits() | Self::BOTTOM_LEFT.bits();
        const RIGHT        = Self::TOP_RIGHT.bits() | Self::BOTTOM_RIGHT.bits();        
        const ALL = Self::TOP.bits() | Self::BOTTOM.bits();
        const NONE = 0;
    }

}

/// The node's shape.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Shape {
    Rectangle {
        corner_radius: f32,
    },
    Circle,
    Ring {
        width: f32,
    },
    /// Arc segment of a circle. Angles are in radians, starting from the right (0) and going counter-clockwise.
    Arc {
        start_angle: f32,
        end_angle: f32,
        width: f32,
    },
    /// Pie/wedge slice of a circle. Angles are in radians, starting from the right (0) and going counter-clockwise.
    Pie {
        start_angle: f32,
        end_angle: f32,
    },
    /// Line segment. Coordinates are normalized (0.0 to 1.0) within the node's rect.
    /// (0, 0) is top-left, (1, 1) is bottom-right.
    /// dash_length: None = solid line, Some(length) = dashed line with specified dash length.
    // todo, this is terrible.
    Segment {
        start: (f32, f32),
        end: (f32, f32),
        dash_length: Option<f32>,
    },
    /// Convenience for a horizontal line from left to right at vertical center.
    HorizontalLine,
    /// Convenience for a vertical line from top to bottom at horizontal center.
    VerticalLine,
    /// Triangle pointing in a direction. Rotation in radians, 0 = pointing right, π/2 = pointing up, etc.
    /// Width controls the base width: 1.0 = equilateral, <1.0 = narrower.
    Triangle {
        rotation: f32,
        width: f32,
    },
    /// Grid pattern filling the node's rect.
    Grid {
        lattice_size: f32,
        offset: (f32, f32),
        line_thickness: f32,
    },
    HexGrid {
        lattice_size: f32,
        offset: (f32, f32),
        line_thickness: f32,
    },
}

impl Hash for Shape {
    fn hash<H: Hasher>(&self, state: &mut H) {
        use Shape::*;
        match self {
            Rectangle { corner_radius } => {
                0u8.hash(state);
                corner_radius.to_bits().hash(state);
            }
            Circle => {
                1u8.hash(state);
            }
            Ring { width } => {
                2u8.hash(state);
                width.to_bits().hash(state);
            }
            Arc { start_angle, end_angle, width } => {
                3u8.hash(state);
                start_angle.to_bits().hash(state);
                end_angle.to_bits().hash(state);
                width.to_bits().hash(state);
            }
            Pie { start_angle, end_angle } => {
                4u8.hash(state);
                start_angle.to_bits().hash(state);
                end_angle.to_bits().hash(state);
            }
            Segment { start, end, dash_length } => {
                5u8.hash(state);
                start.0.to_bits().hash(state);
                start.1.to_bits().hash(state);
                end.0.to_bits().hash(state);
                end.1.to_bits().hash(state);
                match dash_length {
                    None => 0u8.hash(state),
                    Some(len) => {
                        1u8.hash(state);
                        len.to_bits().hash(state);
                    }
                }
            }
            HorizontalLine => {
                6u8.hash(state);
            }
            VerticalLine => {
                7u8.hash(state);
            }
            Triangle { rotation, width } => {
                8u8.hash(state);
                rotation.to_bits().hash(state);
                width.to_bits().hash(state);
            }
            Grid { lattice_size, offset, line_thickness } => {
                9u8.hash(state);
                lattice_size.to_bits().hash(state);
                offset.0.to_bits().hash(state);
                offset.1.to_bits().hash(state);
                line_thickness.to_bits().hash(state);
            }
            HexGrid { lattice_size, offset, line_thickness } => {
                10u8.hash(state);
                lattice_size.to_bits().hash(state);
                offset.0.to_bits().hash(state);
                offset.1.to_bits().hash(state);
                line_thickness.to_bits().hash(state);
            }
        }
    }
}

/// The node's visual appearance.
#[derive(Debug, Copy, Clone, PartialEq, Hash)]
pub struct Rect {
    pub shape: Shape,
    pub rounded_corners: RoundedCorners,
    pub visible: bool,
    pub stroke: Option<Stroke>,
    pub vertex_colors: VertexColors,
    // ... crazy stuff like texture and NinePatchRect
}

// todo: is the size of this really ok?
/// The visual style of a stroke.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stroke {
    /// Width of the stroke.
    pub width: f32,
    /// Color of the stroke.
    pub color: Color,
    /// Style for connecting segments of the stroke.
    pub join: Join,
    /// Limit for miter joins.
    pub miter_limit: f32,
    /// Style for capping the beginning of an open subpath.
    pub start_cap: Cap,
    /// Style for capping the end of an open subpath.
    pub end_cap: Cap,
    /// Lengths of dashes in alternating on/off order.
    pub dash_length: f32,
    /// Offset of the first dash.
    pub dash_offset: f32,
}

impl Stroke {
    pub const fn new(width: f32) -> Self {
        Self {
            width,
            color: Color::KERU_GREEN,
            join: Join::Miter,
            miter_limit: 4.0,
            start_cap: Cap::Butt,
            end_cap: Cap::Butt,
            dash_length: 0.0,
            dash_offset: 0.0,
        }
    }

    pub const fn with_join(mut self, join: Join) -> Self {
        self.join = join;
        self
    }

    pub const fn with_caps(mut self, cap: Cap) -> Self {
        self.start_cap = cap;
        self.end_cap = cap;
        self
    }

    pub const fn with_start_cap(mut self, cap: Cap) -> Self {
        self.start_cap = cap;
        self
    }

    pub const fn with_end_cap(mut self, cap: Cap) -> Self {
        self.end_cap = cap;
        self
    }

    pub const fn with_miter_limit(mut self, limit: f32) -> Self {
        self.miter_limit = limit;
        self
    }

    pub const fn with_dashes(mut self, dash_length: f32, dash_offset: f32) -> Self {
        self.dash_length = dash_length;
        self.dash_offset = dash_offset;
        self
    }

    pub const fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

}

impl Hash for Stroke {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.width.to_bits().hash(state);
        self.color.hash(state);
        std::mem::discriminant(&self.join).hash(state);
        self.miter_limit.to_bits().hash(state);
        std::mem::discriminant(&self.start_cap).hash(state);
        std::mem::discriminant(&self.end_cap).hash(state);
        self.dash_length.to_bits().hash(state);
        self.dash_offset.to_bits().hash(state);
    }
}

impl Rect {
    pub const DEFAULT: Self = Self {
        shape: Shape::Rectangle { corner_radius: BASE_RADIUS },
        visible: true,
        stroke: None,
        vertex_colors: VertexColors::flat(Color::KERU_BLUE),
        rounded_corners: RoundedCorners::ALL,
    };
}

// rename
// todo: add greyed text for textinput
/// Options for text nodes.
#[derive(Debug, Copy, Clone, Hash)]
pub struct TextOptions {
    pub editable: bool,
    pub single_line: bool,
    pub selectable: bool,
    pub edit_disabled: bool,
}

impl Default for TextOptions {
    fn default() -> Self {
        Self::const_default()
    }
}

impl TextOptions {
    const fn const_default() -> Self {
        Self {
            editable: false,
            single_line: false,
            selectable: true,
            edit_disabled: false,
        }
    }
}

pub(crate) const BASE_RADIUS: f32 = 9.0;

impl Node {
    pub(crate) fn cosmetic_hash(&self) -> u64 {
        let mut hasher = ahasher();
        self.rect.hash(&mut hasher);
        return hasher.finish();
    }

    pub(crate) fn layout_hash(&self) -> u64 {
        let mut hasher = ahasher();
        self.layout.hash(&mut hasher);
        self.stack.hash(&mut hasher);
        self.text_params.hash(&mut hasher);
        return hasher.finish();
    }

    pub const fn const_default() -> Self {
        return DEFAULT;
    }

    pub const fn position(mut self, position_x: Position, position_y: Position) -> Self {
        self.layout.position.x = position_x;
        self.layout.position.y = position_y;
        return self;
    }

    pub const fn position_symm(mut self, position: Position) -> Self {
        self.layout.position.x = position;
        self.layout.position.y = position;
        return self;
    }

    pub const fn position_x(mut self, position: Position) -> Self {
        self.layout.position.x = position;
        return self;
    }

    pub const fn position_y(mut self, position: Position) -> Self {
        self.layout.position.y = position;
        return self;
    }

    pub const fn anchor(mut self, anchor_x: Anchor, anchor_y: Anchor) -> Self {
        self.layout.anchor.x = anchor_x;
        self.layout.anchor.y = anchor_y;
        return self;
    }

    pub const fn anchor_symm(mut self, anchor: Anchor) -> Self {
        self.layout.anchor.x = anchor;
        self.layout.anchor.y = anchor;
        return self;
    }

    pub const fn anchor_x(mut self, anchor: Anchor) -> Self {
        self.layout.anchor.x = anchor;
        return self;
    }

    pub const fn anchor_y(mut self, anchor: Anchor) -> Self {
        self.layout.anchor.y = anchor;
        return self;
    }

    pub const fn size(mut self, size_x: Size, size_y: Size) -> Self {
        self.layout.size.x = size_x;
        self.layout.size.y = size_y;
        return self;
    }

    pub const fn size_x(mut self, size_x: Size) -> Self {
        self.layout.size.x = size_x;
        return self;
    }

    pub const fn size_y(mut self, size_y: Size) -> Self {
        self.layout.size.y = size_y;
        return self;
    }

    pub const fn size_symm(mut self, size: Size) -> Self {
        self.layout.size.x = size;
        self.layout.size.y = size;
        return self;
    }

    pub const fn visible(mut self) -> Self {
        self.rect.visible = true;
        return self;
    }
    pub const fn invisible(mut self) -> Self {
        self.rect.visible = false;
        self.rect.stroke = Some(Stroke::new(4.0).with_color(Color::KERU_DEBUG_RED));
        self.rect.vertex_colors = VertexColors::flat(Color::TRANSPARENT);
        return self;
    }

    pub const fn filled(mut self) -> Self {
        self.rect.stroke = None;
        return self;
    }

    pub const fn stroke(mut self, width: f32) -> Self {
        self.rect.stroke = Some(Stroke::new(width));
        return self;
    }

    pub const fn stroke_join(mut self, join: Join) -> Self {
        if let Some(stroke) = self.rect.stroke {
            self.rect.stroke = Some(stroke.with_join(join));
        }
        return self;
    }

    pub const fn stroke_caps(mut self, cap: Cap) -> Self {
        if let Some(stroke) = self.rect.stroke {
            self.rect.stroke = Some(stroke.with_caps(cap));
        }
        return self;
    }

    pub const fn stroke_start_cap(mut self, cap: Cap) -> Self {
        if let Some(stroke) = self.rect.stroke {
            self.rect.stroke = Some(stroke.with_start_cap(cap));
        }
        return self;
    }

    pub const fn stroke_end_cap(mut self, cap: Cap) -> Self {
        if let Some(stroke) = self.rect.stroke {
            self.rect.stroke = Some(stroke.with_end_cap(cap));
        }
        return self;
    }

    pub const fn stroke_miter_limit(mut self, limit: f32) -> Self {
        if let Some(stroke) = self.rect.stroke {
            self.rect.stroke = Some(stroke.with_miter_limit(limit));
        }
        return self;
    }

    pub const fn stroke_dashes(mut self, dash_length: f32, dash_offset: f32) -> Self {
        if let Some(stroke) = self.rect.stroke {
            self.rect.stroke = Some(stroke.with_dashes(dash_length, dash_offset));
        }
        return self;
    }

    pub const fn stroke_color(mut self, color: Color) -> Self {
        if let Some(stroke) = self.rect.stroke {
            self.rect.stroke = Some(stroke.with_color(color));
        }
        return self;
    }

    pub const fn color(mut self, color: Color) -> Self {
        self.rect.vertex_colors = VertexColors::flat(color);
        return self;
    }

    pub const fn shape(mut self, shape: Shape) -> Self {
        self.rect.shape = shape;
        return self;
    }

    pub const fn circle(mut self) -> Self {
        self.rect.shape = Shape::Circle;
        return self;
    }

    pub const fn colors(mut self, colors: VertexColors) -> Self {
        self.rect.vertex_colors = colors;
        return self;
    }

    pub const fn stack(mut self, axis: Axis, arrange: Arrange, spacing: u32) -> Self {
        self.stack = Some(Stack {
            arrange,
            axis,
            spacing,
        });
        return self;
    }

    pub const fn stack_arrange(mut self, arrange: Arrange) -> Self {
        let stack = match self.stack {
            Some(stack) => stack,
            None => Stack::DEFAULT,
        };
        self.stack = Some(stack.arrange(arrange));
        return self;
    }

    pub const fn stack_spacing(mut self, spacing: u32) -> Self {
        let stack = match self.stack {
            Some(stack) => stack,
            None => Stack::DEFAULT,
        };
        self.stack = Some(stack.spacing(spacing));
        return self;
    }

    // todo: if we don't mind sacrificing symmetry, it could make sense to just remove this one.
    pub const fn stack_axis(mut self, axis: Axis) -> Self {
        let stack = match self.stack {
            Some(stack) => stack,
            None => Stack::DEFAULT,
        };
        self.stack = Some(stack.axis(axis));
        return self;
    }

    pub const fn padding(mut self, pixels: u32) -> Self {
        self.layout.padding = Xy::new_symm(pixels);
        return self;
    }

    pub const fn padding_x(mut self, padding: u32) -> Self {
        self.layout.padding.x = padding;
        return self;
    }

    pub const fn padding_y(mut self, padding: u32) -> Self {
        self.layout.padding.y = padding;
        return self;
    }

    pub const fn scrollable_x(mut self, scrollable_x: bool) -> Self {
        self.layout.scrollable.x = scrollable_x;
        return self;
    }

    pub const fn scrollable_y(mut self, scrollable_y: bool) -> Self {
        self.layout.scrollable.y = scrollable_y;
        return self;
    }

    pub const fn absorbs_clicks(mut self, absorbs_clicks: bool) -> Self {
        self.interact.absorbs_mouse_events = absorbs_clicks;
        return self;
    }

    pub fn key(mut self, key: NodeKey) -> Self {
        self.key = Some(key);
        return self;
    }

    pub const fn animation(mut self, animation: Animation) -> Self {
        self.animation = animation;
        return self;
    }

    pub const fn animation_speed(mut self, speed: f32) -> Self {
        self.animation.speed = speed;
        return self;
    }

    // Enter animation methods
    pub const fn enter_slide(mut self, edge: SlideEdge, direction: SlideDirection) -> Self {
        self.animation.enter = EnterAnimation::Slide { edge, direction };
        return self;
    }

    pub const fn enter_grow(mut self, axis: Axis, origin: Position) -> Self {
        self.animation.enter = EnterAnimation::GrowShrink { axis, origin };
        return self;
    }

    // Exit animation methods
    pub const fn exit_slide(mut self, edge: SlideEdge, direction: SlideDirection) -> Self {
        self.animation.exit = ExitAnimation::Slide { edge, direction };
        return self;
    }

    pub const fn exit_shrink(mut self, axis: Axis, origin: Position) -> Self {
        self.animation.exit = ExitAnimation::GrowShrink { axis, origin };
        return self;
    }

    // Convenience methods for common patterns
    pub const fn slide_from_top(mut self) -> Self {
        self.animation.enter = EnterAnimation::Slide { edge: SlideEdge::Top, direction: SlideDirection::In };
        self.animation.exit = ExitAnimation::Slide { edge: SlideEdge::Top, direction: SlideDirection::Out };
        return self;
    }

    pub const fn slide_from_bottom(mut self) -> Self {
        self.animation.enter = EnterAnimation::Slide { edge: SlideEdge::Bottom, direction: SlideDirection::In };
        self.animation.exit = ExitAnimation::Slide { edge: SlideEdge::Bottom, direction: SlideDirection::Out };
        return self;
    }

    pub const fn slide_from_left(mut self) -> Self {
        self.animation.enter = EnterAnimation::Slide { edge: SlideEdge::Left, direction: SlideDirection::In };
        self.animation.exit = ExitAnimation::Slide { edge: SlideEdge::Left, direction: SlideDirection::Out };
        return self;
    }

    pub const fn slide_from_right(mut self) -> Self {
        self.animation.enter = EnterAnimation::Slide { edge: SlideEdge::Right, direction: SlideDirection::In };
        self.animation.exit = ExitAnimation::Slide { edge: SlideEdge::Right, direction: SlideDirection::Out };
        return self;
    }

    pub const fn grow_shrink(mut self, axis: Axis, origin: Position) -> Self {
        self.animation.enter = EnterAnimation::GrowShrink { axis, origin };
        self.animation.exit = ExitAnimation::GrowShrink { axis, origin };
        return self;
    }

    pub const fn animate_position(mut self, value: bool) -> Self {
        self.animation.state_transition.animate_position = value;
        return self;
    }

    /// Sets whether a node's children stay hidden or get removed when they get excluded from the tree.
    /// 
    /// If a node stays hidden, it retains its internal state (scroll offset, text input, ...), and it is slightly less expensive to bring them back into view. If it gets removed, its memory can be reused for other nodes. 
    /// 
    /// For example, the panel with the main content in a tabbed application should use [`children_can_hide(true)`], so that all state is retained when switching tabs.
    /// 
    /// On the other hand, a panel that contains thumbnails for files, or similar highly dynamic content, should use [`children_can_hide(false)`], so that when the thumbnails for the old elements are switched out, their memory can be reused for the new ones.
    /// 
    /// By default, almost all [`Node`] values have [`children_can_hide(false)`].
    pub fn children_can_hide(mut self, value: bool) -> Self {
        self.children_can_hide = if value { ChildrenCanHide::Yes } else { ChildrenCanHide::No };
        return self;
    }

    pub fn children_can_hide_inherit(mut self) -> Self {
        self.children_can_hide = ChildrenCanHide::Inherit;
        return self;
    }

    pub const fn sense_click(mut self, value: bool) -> Self {
        let senses = &mut self.interact.senses;
        if value {
            *senses = senses.union(Sense::CLICK);
        } else {
            *senses = senses.intersection(Sense::CLICK.complement());
        }
        return self;
    }

    pub const fn sense_drag(mut self, value: bool) -> Self {
        let senses = &mut self.interact.senses;
        if value {
            *senses = senses.union(Sense::DRAG);
        } else {
            *senses = senses.intersection(Sense::DRAG.complement());
        }
        return self;
    }

    pub const fn sense_hover(mut self, value: bool) -> Self {
        let senses = &mut self.interact.senses;
        if value {
            *senses = senses.union(Sense::HOVER);
        } else {
            *senses = senses.intersection(Sense::HOVER.complement());
        }
        return self;
    }

    pub const fn sense_hold(mut self, value: bool) -> Self {
        let senses = &mut self.interact.senses;
        if value {
            *senses = senses.union(Sense::HOLD);
        } else {
            *senses = senses.intersection(Sense::HOLD.complement());
        }
        return self;
    }

    pub fn is_fit_content(&self) -> bool {
        let Xy { x, y } = self.layout.size;
        return x == Size::FitContent || y == Size::FitContent
    }

    pub const fn is_scrollable(&self) -> bool {
        return self.layout.scrollable.x || self.layout.scrollable.y
    }

    pub const fn corners(mut self, corners: RoundedCorners) -> Self {
        self.rect.rounded_corners = corners;
        return self;
    }

    pub const fn clip_children(mut self, value: bool) -> Self {
        self.clip_children = Xy::new(value, value);
        return self;
    }

    pub const fn translate(mut self, x: f32, y: f32) -> Self {
        self.translate = Some((x, y));
        return self;
    }

    /// Apply a zoom centered at the center of the node's rect.
    pub const fn zoom(mut self, scale: f32) -> Self {
        self.scale = Some((scale, scale));
        return self;
    }

    pub const fn clip_children_x(mut self, value: bool) -> Self {
        self.clip_children.x = value;
        return self;
    }

    pub const fn clip_children_y(mut self, value: bool) -> Self {
        self.clip_children.y = value;
        return self;
    }

    pub const fn custom_render(mut self, value: bool) -> Self {
        self.custom_render = value;
        return self;
    }
}

#[derive(Copy, Clone, Hash)]
pub enum NodeText<'a> {
    Dynamic(&'a str),
    Static(&'static str),
}

impl<'a> NodeText<'a> {
    pub fn as_str(&self) -> &str {
        match self {
            NodeText::Dynamic(s) => s,
            NodeText::Static(s) => s,
        }
    }
    
    pub fn is_static(&self) -> bool {
        matches!(self, NodeText::Static(_))
    }
}

/// Data for an image to be displayed
#[derive(Copy, Clone)]
pub enum ImageData {
    /// Raster image (PNG, JPEG, etc.)
    Raster(&'static [u8]),
    /// SVG image data
    Svg(&'static [u8]),
}

/// An extended version of [`Node`] that can hold text or other borrowed data.
///
/// Created starting from a [`Node`] and using methods like [`Node::text()`].
///
/// Can be used in the same way as [`Node`].
#[derive(Copy, Clone)]
pub struct FullNode<'a> {
    pub params: Node,
    pub text: Option<NodeText<'a>>,
    pub text_style: Option<StyleHandle>,
    pub(crate) text_changed: Changed,
    // todo: why store it here? just do text.ptr()?
    pub(crate) text_ptr: usize,
    pub image: Option<ImageData>,
    pub placeholder: Option<&'a str>,
}

impl<'a> FullNode<'a> {
    pub const fn single_line_text(mut self, value: bool) -> Self {
        let text_params = match self.params.text_params {
            Some(mut tp) => {
                tp.single_line = value;
                tp
            },
            None => TextOptions {
                single_line: value,
                ..TextOptions::const_default()
            }
        };
        self.params.text_params = Some(text_params);
        return self;
    }

    pub const fn position(mut self, position_x: Position, position_y: Position) -> Self {
        self.params.layout.position.x = position_x;
        self.params.layout.position.y = position_y;
        return self;
    }

    pub const fn position_symm(mut self, position: Position) -> Self {
        self.params.layout.position.x = position;
        self.params.layout.position.y = position;
        return self;
    }

    pub const fn position_x(mut self, position: Position) -> Self {
        self.params.layout.position.x = position;
        return self;
    }

    pub const fn position_y(mut self, position: Position) -> Self {
        self.params.layout.position.y = position;
        return self;
    }

    pub const fn anchor(mut self, anchor_x: Anchor, anchor_y: Anchor) -> Self {
        self.params.layout.anchor.x = anchor_x;
        self.params.layout.anchor.y = anchor_y;
        return self;
    }

    pub const fn anchor_symm(mut self, anchor: Anchor) -> Self {
        self.params.layout.anchor.x = anchor;
        self.params.layout.anchor.y = anchor;
        return self;
    }

    pub const fn anchor_x(mut self, anchor: Anchor) -> Self {
        self.params.layout.anchor.x = anchor;
        return self;
    }

    pub const fn anchor_y(mut self, anchor: Anchor) -> Self {
        self.params.layout.anchor.y = anchor;
        return self;
    }

    pub const fn size(mut self, size_x: Size, size_y: Size) -> Self {
        self.params.layout.size.x = size_x;
        self.params.layout.size.y = size_y;
        return self;
    }

    pub const fn size_x(mut self, size_x: Size) -> Self {
        self.params.layout.size.x = size_x;
        return self;
    }

    pub const fn size_y(mut self, size_y: Size) -> Self {
        self.params.layout.size.y = size_y;
        return self;
    }

    pub const fn size_symm(mut self, size: Size) -> Self {
        self.params.layout.size.x = size;
        self.params.layout.size.y = size;
        return self;
    }

    pub const fn visible(mut self) -> Self {
        self.params.rect.visible = true;
        return self;
    }
    pub const fn invisible(mut self) -> Self {
        self.params.rect.visible = false;
        self.params.rect.stroke = Some(Stroke::new(4.0).with_color(Color::KERU_DEBUG_RED));
        self.params.rect.vertex_colors = VertexColors::flat(Color::TRANSPARENT);
        return self;
    }

    pub const fn filled(mut self) -> Self {
        self.params.rect.stroke = None;
        return self;
    }

    pub const fn stroke(mut self, width: f32) -> Self {
        self.params.rect.stroke = Some(Stroke::new(width));
        return self;
    }

    pub const fn stroke_join(mut self, join: Join) -> Self {
        if let Some(stroke) = self.params.rect.stroke {
            self.params.rect.stroke = Some(stroke.with_join(join));
        }
        return self;
    }

    pub const fn stroke_caps(mut self, cap: Cap) -> Self {
        if let Some(stroke) = self.params.rect.stroke {
            self.params.rect.stroke = Some(stroke.with_caps(cap));
        }
        return self;
    }

    pub const fn stroke_start_cap(mut self, cap: Cap) -> Self {
        if let Some(stroke) = self.params.rect.stroke {
            self.params.rect.stroke = Some(stroke.with_start_cap(cap));
        }
        return self;
    }

    pub const fn stroke_end_cap(mut self, cap: Cap) -> Self {
        if let Some(stroke) = self.params.rect.stroke {
            self.params.rect.stroke = Some(stroke.with_end_cap(cap));
        }
        return self;
    }

    pub const fn stroke_miter_limit(mut self, limit: f32) -> Self {
        if let Some(stroke) = self.params.rect.stroke {
            self.params.rect.stroke = Some(stroke.with_miter_limit(limit));
        }
        return self;
    }

    pub const fn stroke_dashes(mut self, dash_length: f32, dash_offset: f32) -> Self {
        if let Some(stroke) = self.params.rect.stroke {
            self.params.rect.stroke = Some(stroke.with_dashes(dash_length, dash_offset));
        }
        return self;
    }

    pub const fn stroke_color(mut self, color: Color) -> Self {
        if let Some(stroke) = self.params.rect.stroke {
            self.params.rect.stroke = Some(stroke.with_color(color));
        }
        return self;
    }

    pub const fn color(mut self, color: Color) -> Self {
        self.params.rect.vertex_colors = VertexColors::flat(color);
        return self;
    }

    pub const fn shape(mut self, shape: Shape) -> Self {
        self.params.rect.shape = shape;
        return self;
    }

    pub const fn circle(mut self) -> Self {
        self.params.rect.shape = Shape::Circle;
        return self;
    }

    pub const fn vertex_colors(mut self, colors: VertexColors) -> Self {
        self.params.rect.vertex_colors = colors;
        return self;
    }

    pub const fn stack(mut self, axis: Axis, arrange: Arrange, spacing: u32) -> Self {
        self.params.stack = Some(Stack {
            arrange,
            axis,
            spacing,
        });
        return self;
    }

    pub const fn stack_arrange(mut self, arrange: Arrange) -> Self {
        let stack = match self.params.stack {
            Some(stack) => stack,
            None => Stack::DEFAULT,
        };
        self.params.stack = Some(stack.arrange(arrange));
        return self;
    }

    pub const fn stack_spacing(mut self, spacing: u32) -> Self {
        let stack = match self.params.stack {
            Some(stack) => stack,
            None => Stack::DEFAULT,
        };
        self.params.stack = Some(stack.spacing(spacing));
        return self;
    }

    // todo: if we don't mind sacrificing symmetry, it could make sense to just remove this one.
    pub const fn stack_axis(mut self, axis: Axis) -> Self {
        let stack = match self.params.stack {
            Some(stack) => stack,
            None => Stack::DEFAULT,
        };
        self.params.stack = Some(stack.axis(axis));
        return self;
    }

    pub const fn padding(mut self, padding: u32) -> Self {
        self.params.layout.padding = Xy::new_symm(padding);
        return self;
    }

    pub const fn padding_x(mut self, padding: u32) -> Self {
        self.params.layout.padding.x = padding;
        return self;
    }

    pub const fn padding_y(mut self, padding: u32) -> Self {
        self.params.layout.padding.y = padding;
        return self;
    }

    pub const fn scrollable_x(mut self, scrollable_x: bool) -> Self {
        self.params.layout.scrollable.x = scrollable_x;
        return self;
    }

    pub const fn scrollable_y(mut self, scrollable_y: bool) -> Self {
        self.params.layout.scrollable.y = scrollable_y;
        return self;
    }

    pub const fn absorbs_clicks(mut self, absorbs_clicks: bool) -> Self {
        self.params.interact.absorbs_mouse_events = absorbs_clicks;
        return self;
    }

    pub const fn sense_click(mut self, value: bool) -> Self {
        let senses = &mut self.params.interact.senses;
        if value {
            *senses = senses.union(Sense::CLICK);
        } else {
            *senses = senses.intersection(Sense::CLICK.complement());
        }
        return self;
    }

    pub const fn sense_drag(mut self, value: bool) -> Self {
        let senses = &mut self.params.interact.senses;
        if value {
            *senses = senses.union(Sense::DRAG);
        } else {
            *senses = senses.intersection(Sense::DRAG.complement());
        }
        return self;
    }

    pub const fn sense_hover(mut self, value: bool) -> Self {
        let senses = &mut self.params.interact.senses;
        if value {
            *senses = senses.union(Sense::HOVER);
        } else {
            *senses = senses.intersection(Sense::HOVER.complement());
        }
        return self;
    }

    pub const fn sense_hold(mut self, value: bool) -> Self {
        let senses = &mut self.params.interact.senses;
        if value {
            *senses = senses.union(Sense::HOLD);
        } else {
            *senses = senses.intersection(Sense::HOLD.complement());
        }
        return self;
    }

    /// Add a [`NodeKey`] to the [`Node`].
    /// 
    pub fn key(mut self, key: NodeKey) -> Self {
        self.params.key = Some(key);
        return self;
    }

    pub const fn animation(mut self, animation: Animation) -> Self {
        self.params.animation = animation;
        return self;
    }

    pub const fn animation_speed(mut self, speed: f32) -> Self {
        self.params.animation.speed = speed;
        return self;
    }

    // Enter animation methods
    pub const fn enter_slide(mut self, edge: SlideEdge, direction: SlideDirection) -> Self {
        self.params.animation.enter = EnterAnimation::Slide { edge, direction };
        return self;
    }

    pub const fn enter_grow(mut self, axis: Axis, origin: Position) -> Self {
        self.params.animation.enter = EnterAnimation::GrowShrink { axis, origin };
        return self;
    }

    // Exit animation methods
    pub const fn exit_slide(mut self, edge: SlideEdge, direction: SlideDirection) -> Self {
        self.params.animation.exit = ExitAnimation::Slide { edge, direction };
        return self;
    }

    pub const fn exit_shrink(mut self, axis: Axis, origin: Position) -> Self {
        self.params.animation.exit = ExitAnimation::GrowShrink { axis, origin };
        return self;
    }

    pub const fn slide_from_top(mut self) -> Self {
        self.params.animation.enter = EnterAnimation::Slide { edge: SlideEdge::Top, direction: SlideDirection::In };
        self.params.animation.exit = ExitAnimation::Slide { edge: SlideEdge::Top, direction: SlideDirection::Out };
        return self;
    }

    pub const fn slide_from_bottom(mut self) -> Self {
        self.params.animation.enter = EnterAnimation::Slide { edge: SlideEdge::Bottom, direction: SlideDirection::In };
        self.params.animation.exit = ExitAnimation::Slide { edge: SlideEdge::Bottom, direction: SlideDirection::Out };
        return self;
    }

    pub const fn slide_from_left(mut self) -> Self {
        self.params.animation.enter = EnterAnimation::Slide { edge: SlideEdge::Left, direction: SlideDirection::In };
        self.params.animation.exit = ExitAnimation::Slide { edge: SlideEdge::Left, direction: SlideDirection::Out };
        return self;
    }

    pub const fn slide_from_right(mut self) -> Self {
        self.params.animation.enter = EnterAnimation::Slide { edge: SlideEdge::Right, direction: SlideDirection::In };
        self.params.animation.exit = ExitAnimation::Slide { edge: SlideEdge::Right, direction: SlideDirection::Out };
        return self;
    }

    pub const fn grow_shrink(mut self, axis: Axis, origin: Position) -> Self {
        self.params.animation.enter = EnterAnimation::GrowShrink { axis, origin };
        self.params.animation.exit = ExitAnimation::GrowShrink { axis, origin };
        return self;
    }

    pub const fn animate_position(mut self, value: bool) -> Self {
        self.params.animation.state_transition.animate_position = value;
        return self;
    }

    pub fn is_fit_content(&self) -> bool {
        let Xy { x, y } = self.params.layout.size;
        return x == Size::FitContent || y == Size::FitContent
    }

    pub const fn is_scrollable(&self) -> bool {
        return self.params.layout.scrollable.x || self.params.layout.scrollable.y
    }

    pub const fn corners(mut self, corners: RoundedCorners) -> Self {
        self.params.rect.rounded_corners = corners;
        return self;
    }

    pub fn children_can_hide(mut self, value: bool) -> Self {
        self.params.children_can_hide = if value { ChildrenCanHide::Yes } else { ChildrenCanHide::No };
        return self;
    }

    pub fn children_can_hide_inherit(mut self) -> Self {
        self.params.children_can_hide = ChildrenCanHide::Inherit;
        return self;
    }

    /// Set the text style for this node.
    pub fn text_style(mut self, style: StyleHandle) -> Self {
        self.text_style = Some(style);
        return self;
    }

    /// Set placeholder text for a text edit that will be shown when the text edit is empty.
    /// This only works with editable text nodes.
    pub fn placeholder_text(mut self, placeholder: &'a str) -> Self {
        self.placeholder = Some(placeholder);
        self
    }

    pub const fn clip_children(mut self, value: Xy<bool>) -> Self {
        self.params.clip_children = value;
        return self;
    }

    pub const fn clip_children_x(mut self, value: bool) -> Self {
        self.params.clip_children.x = value;
        return self;
    }

    pub const fn clip_children_y(mut self, value: bool) -> Self {
        self.params.clip_children.y = value;
        return self;
    }

    pub const fn custom_render(mut self, value: bool) -> Self {
        self.params.custom_render = value;
        return self;
    }
}

// impl FullNode<'_> {
//     /// Add text to the [`Node`] from a `&'static str`.
//     /// 
//     /// `text` is assumed to be unchanged, so the [`Ui`] uses pointer equality to determine if it needs to update the text shown on screen.
//     /// 
//     /// If `text` changes, due to interior mutability or unsafe code, then the [`Ui`] will miss it.  
//     pub fn static_text(self, text: &'static str) -> FullNode<'static> {
//         return FullNode {
//             params: self.params,
//             image: self.image,
//             text: Some(text),
//             text_style: self.text_style,
//             text_changed: Changed::Static,
//             text_ptr: text.as_ref().as_ptr() as usize,
//         }
//     }
// }

impl Node {
    /// Add text to the [`Node`] from a `&'static str`.
    /// 
    /// The [`Ui`] will have to hash `text` to determine if it needs to update the text shown on the screen. To avoid this performance penalty, use [`Node::observed_text`], or [`Node::static_text`] if `text` is an unchanging `'static str`. 
    
    // pub fn text<'a, T, M>(self, text: &'a M) -> FullNode<'a>
    // where
    //     M: MaybeObserver<T> + ?Sized,
    //     T: AsRef<str> + ?Sized + 'a,
    // {
    pub fn hashed_text(self, text: &(impl AsRef<str> + ?Sized)) -> FullNode<'_> {
        return FullNode {
            params: self,
            text: Some(NodeText::Dynamic(text.as_ref())),
            text_style: None,
            image: None,
            text_changed: Changed::NeedsHash,
            text_ptr: text.as_ref().as_ptr() as usize,
            placeholder: None,
        }
    }

    /// Set placeholder text for a text edit that will be shown when the text edit is empty.
    /// This only works with editable text nodes.
    pub fn placeholder_text<'a>(self, placeholder: &'a str) -> FullNode<'a> {
        return FullNode {
            params: self,
            text: None,
            text_style: None,
            image: None,
            text_changed: Changed::NeedsHash,
            text_ptr: 0,
            placeholder: Some(placeholder),
        }
    }

    /// Add text to the [`Node`] from a `&'static str`.
    /// 
    /// `text` is assumed to be unchanged, so the [`Ui`] uses pointer equality to determine if it needs to update the text shown on screen.
    /// 
    /// If `text` changes, due to interior mutability or unsafe code, then the [`Ui`] will miss it.  
    pub fn static_text(self, text: &'static (impl AsRef<str> + ?Sized)) -> FullNode<'static> {
        return FullNode {
            params: self,
            text: Some(NodeText::Static(text.as_ref())),
            text_style: None,
            image: None,
            text_changed: Changed::Static,
            text_ptr: text.as_ref().as_ptr() as usize,
            placeholder: None,
        }
    }

    /// Add text to the [`Node`] from a `&str` that is known to not be mutated during its lifetime.
    /// 
    /// Since the text is assumed to never change, the [`Ui`] can use pointer equality to determine if it needs to update the text shown on screen.
    /// 
    /// If `text` changes anyway, then the [`Ui`] will miss it.  
    pub fn immut_text(self, text: &(impl AsRef<str> + ?Sized)) -> FullNode<'_> {
        return FullNode {
            params: self,
            text: Some(NodeText::Dynamic(text.as_ref())),
            text_style: None,
            image: None,
            text_changed: Changed::Static,
            text_ptr: text.as_ref().as_ptr() as usize,
            placeholder: None,
        }
    }

    pub fn observed_text(self, text: Observer<&(impl AsRef<str> + ?Sized)>) -> FullNode<'_> {
        return FullNode {
            params: self,
            text: Some(NodeText::Dynamic(text.as_ref())),
            text_style: None,
            text_changed: text.changed_at(),
            text_ptr: text.as_ref().as_ptr() as usize,
            image: None,
            placeholder: None,
        }
    }

    pub fn static_image(self, image: &'static [u8]) -> FullNode<'static> {
        return FullNode {
            params: self,
            text: None,
            text_style: None,
            image: Some(ImageData::Raster(image)),
            text_changed: Changed::Static,
            text_ptr: 0,
            placeholder: None,
        }
    }

    pub fn static_svg(self, svg: &'static [u8]) -> FullNode<'static> {
        return FullNode {
            params: self,
            text: None,
            text_style: None,
            image: Some(ImageData::Svg(svg)),
            text_changed: Changed::Static,
            text_ptr: 0,
            placeholder: None,
        }
    }
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub enum Changed {
    ChangedAt(u64),
    NeedsHash,
    // isn't this about the same as ChangedAt(0)?
    Static,
}

impl From<Node> for FullNode<'_> {
    fn from(val: Node) -> Self {
        FullNode {
            params: val,
            text: None,
            text_style: None,
            text_changed: Changed::Static,
            text_ptr: 0,
            image: None,
            placeholder: None,
        }
    }
}

impl FullNode<'_> {
    #[track_caller]
    pub(crate) fn key_or_anon_key(&self) -> NodeKey {
        return match self.params.key {
            Some(key) => key,
            None => NodeKey::new(Id(caller_location_id()), "Anon node"),
        };
    }
}

#[derive(PartialEq, Debug)]
enum TextVerdict {
    Skip,
    HashAndSee,
    UpdateWithoutHashing,
}

impl Ui {
    fn check_text_situation(&self, i: NodeI, params: &FullNode) -> TextVerdict {
        let same_pointer = params.text_ptr == self.nodes[i].last_text_ptr;
        let verdict = if same_pointer {
             match params.text_changed {
                Changed::NeedsHash => TextVerdict::HashAndSee,
                Changed::ChangedAt(change_frame) => {
                    if change_frame > self.sys.second_last_frame_end_fake_time {
                        TextVerdict::UpdateWithoutHashing
                    } else {
                        TextVerdict::Skip
                    }
                },
                Changed::Static => TextVerdict::Skip,
            }

        } else { // different pointer 
            // probably not worth even hashing here
            TextVerdict::UpdateWithoutHashing
        };
        return verdict;
    }

    pub(crate) fn set_params_text(&mut self, i: NodeI, params: &FullNode) {       
        let Some(text) = params.text else {
            return
        };
        
        let text_options = params.params.text_params.as_ref();
        let edit = text_options.map(|tp| tp.editable).unwrap_or(false);
        
        if edit {
            // For editable text, always update if content changed
            if self.nodes[i].last_text_ptr != params.text_ptr {
                // todo: this as_ref() is dumb, should this be changed in textslabs?
                self.set_text2(i, text, text_options, params.text_style.as_ref(), params.placeholder);
                self.nodes[i].last_text_ptr = params.text_ptr;
            }
            return;
        }

        #[cfg(not(debug_assertions))]
        if reactive::is_in_skipped_reactive_block() {
            return;
        }
        // todo: the error-logging brother of that
        
        // todo: if text attributes have changed, go straight to relayout anyway.

        let text_verdict = self.check_text_situation(i, params);
        if text_verdict == TextVerdict::Skip {
            log::trace!("Skipping text update");
            return;
        }
        
        self.nodes[i].last_text_ptr = params.text_ptr;

        #[cfg(debug_assertions)]
        let hash: u64;

        #[cfg(debug_assertions)] {
            hash = ahash(&text);
            if reactive::is_in_skipped_reactive_block() {
                let mut error = false;
                if let Some(last_hash) = self.nodes[i].last_text_hash {
                    if last_hash != hash {
                        error = true;
                    }
                    self.nodes[i].last_text_hash = Some(hash); 
                } else {
                    // this is probably wrong too
                    error = true;
                }
                if error {
                    log::error!("Keru: incorrect reactive block: the text on node \"{}\" changed, but reactive thought they didn't", self.node_debug_name_fmt_scratch(i));
                    return;
                    
                }
            }
        }

        match text_verdict {
            TextVerdict::Skip => unreachable!("Already handled above"),
            TextVerdict::HashAndSee => {
                if self.nodes[i].text_i.is_some() {
                    #[cfg(not(debug_assertions))]
                    let hash = ahash(&text);

                    if let Some(last_hash) = self.nodes[i].last_text_hash {
                        if hash != last_hash {
                            log::trace!("Updating after hash");
                            self.nodes[i].last_text_hash = Some(hash);
                            self.set_text2(i, text, text_options, params.text_style.as_ref(), params.placeholder);
                        } else {
                            log::trace!("Skipping after hash");
                        }
                    } else {
                        self.set_text2(i, text, text_options, params.text_style.as_ref(), params.placeholder);
                        if !text.is_static() {
                            self.nodes[i].last_text_hash = Some(hash);
                        }
                    }
                } else {
                    log::trace!("Updating (node had no text)");
                    self.set_text2(i, text, text_options, params.text_style.as_ref(), params.placeholder);
                }
            },
            TextVerdict::UpdateWithoutHashing => {
                log::trace!("Updating without hash");
                self.set_text2(i, text, text_options, params.text_style.as_ref(), params.placeholder);
                self.nodes[i].last_text_hash = None;
            },
        };
    }


    pub(crate) fn set_params(&mut self, i: NodeI, params: &FullNode) {
        #[cfg(not(debug_assertions))]
        if reactive::is_in_skipped_reactive_block() {
            return;
        }
        
        if let Some(image_data) = params.image {
            match image_data {
                ImageData::Raster(image) => self.set_static_image(i, image),
                ImageData::Svg(svg) => self.set_static_svg(i, svg),
            };
        }
        
        let new_cosmetic_hash = params.params.cosmetic_hash();
        let new_layout_hash = params.params.layout_hash();
        
        let cosmetic_changed = new_cosmetic_hash != self.nodes[i].last_cosmetic_hash;
        let layout_changed = new_layout_hash != self.nodes[i].last_layout_hash;

        #[cfg(debug_assertions)]
        if reactive::is_in_skipped_reactive_block() {
            if cosmetic_changed || layout_changed {
                let kind = match (layout_changed, cosmetic_changed) {
                    (true, true) => "layout and appearance",
                    (true, false) => "layout",
                    (false, true) => "appearance",
                    _ => unreachable!()
                };
                log::error!("Keru: incorrect reactive block: the {kind} params of node \"{}\" changed, but reactive thought they didn't", self.node_debug_name_fmt_scratch(i));
                // log::error!("Keru: incorrect reactive block: the {kind} params of node \"{}\" changed, even if a reactive block declared that it shouldn't have.\n Check that the reactive block is correctly checking all the runtime variables that can affect the node's params.", self.node_debug_name(i));
            }
            return;
        }
        
        self.nodes[i].params = params.params.clone();

        self.nodes[i].last_cosmetic_hash = new_cosmetic_hash;
        self.nodes[i].last_layout_hash = new_layout_hash;

        if layout_changed {
            self.push_partial_relayout(i);
        }
        if cosmetic_changed{
            self.sys.changes.rebuild_render_data = true;
        }
    }
}

impl Node {
    /// Add text to the [`Node`].
    /// 
    /// The `text` argument can be a `&str`, a `String`, or any type that implements [`AsRef<str>`].
    /// 
    /// It can optionally wrapped by an [`Observer`], [`Static`] or [`Immut`] for efficiency.
    /// 
    /// If a plain non-[`Observer`] type is used, the [`Ui`] will fall back to hashing the text to determine if the text needs updating.
    /// 
    /// Instead of this single generic function, you can also use [`Self::hashed_text()`], [`Self::static_text()`], [`Self::immut_text()`], or [`Self::observed_text()`].
    pub fn text(self, text: &(impl MaybeObservedText + ?Sized)) -> FullNode<'_> {
        return FullNode {
            params: self,
            text: Some(NodeText::Dynamic(text.as_text())),
            text_style: None,
            image: None,
            text_changed: text.changed_at(),
            text_ptr: text.as_text().as_ptr() as usize,
            placeholder: None,
        }
    }
}

impl<'a> FullNode<'a> {
    /// Add text to the [`Node`].
    /// 
    /// The `text` argument can be a `&str`, a `String`, or any type that implements [`AsRef<str>`].
    /// 
    /// It can optionally wrapped by an [`Observer`], [`Static`] or [`Immut`] for efficiency.
    /// 
    /// If a plain non-[`Observer`] type is used, the [`Ui`] will fall back to hashing the text to determine if the text needs updating.
    /// 
    /// Instead of this single generic function, you can also use [`Self::hashed_text()`], [`Self::static_text()`], [`Self::immut_text()`], or [`Self::observed_text()`].
    pub fn text(mut self, text: &'a (impl MaybeObservedText + ?Sized)) -> FullNode<'a> {
        self.text = Some(NodeText::Dynamic(text.as_text()));
        self.text_changed = text.changed_at();
        self.text_ptr = text.as_text().as_ptr() as usize;

        return self;
    }
}


/// A wrapper struct for a `'static` value that will never change during its lifetime.
/// 
/// `'static` values can't be mutated except through interior mutability or unsafe code, so this struct is relatively hard to misuse. 
/// 
/// ```rust
/// # use keru::*;
/// let string: &'static str = "this will never change";
/// 
/// // Rust doesn't know that `string` is static.
/// // If we use `params1` to create a node and add it to the Ui,
/// // the Ui will need to hash the text on every update to make sure it's not changing.  
/// let params1 = LABEL.text(string); 
/// 
/// // If the string is wrapped in `Static`,
/// // `text()` can tell that this string can never change, and skip some updates. 
/// let params2 = LABEL.text(&Static(string));
/// ```
/// 
/// If you can guarantee that a non-`'static` variable will not be mutated through its lifetime, you can use [`Immut`]: it works the same way as [`Static`], but without an explicit `'static` bound.
/// 
/// # Notes
/// 
/// This is needed because Rust doesn't support lifetime specialization.
pub struct Static<T: 'static + ?Sized>(pub &'static T);

impl<T: ?Sized> Deref for Static<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

/// A wrapper struct for a value that will never change during its lifetime.
/// 
/// Same as `Static`, but without an explicit ``static` bound.
/// 
/// This struct can wrap any value: it is up to the user to ensure that wrapped variables actually never change. If this assumption is broken, the values displayed in the GUI will get out of sync with the real value of `T`.
/// 
/// You can always use an [`Observer<T>`](`Observer`) or a raw `T` to avoid this risk. If a raw `T` is passed, the [`Ui`] will hash the resulting text to make sure it stays synced.
pub struct Immut<T: ?Sized>(pub T);

impl<T: ?Sized> Deref for Immut<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


pub trait MaybeObservedText {
    // Get the text content
    fn as_text(&self) -> &str;
    
    // Check if the text has changed
    fn changed_at(&self) -> Changed;
}

// Generic implementation for any type that implements AsRef<str>
impl<T: AsRef<str> + ?Sized> MaybeObservedText for T {
    fn as_text(&self) -> &str {
        self.as_ref()
    }
    
    fn changed_at(&self) -> Changed {
        Changed::NeedsHash
    }
}

// Observer can't be ?Sized because it physically holds the T as a field
impl<T: AsRef<str>> MaybeObservedText for Observer<T> {
    fn as_text(&self) -> &str {
        self.as_ref()
    }
    
    fn changed_at(&self) -> Changed {
        self.changed_at()
    }
}

impl<T: AsRef<str> + ?Sized> MaybeObservedText for Static<T> {
    fn as_text(&self) -> &str {
        self.as_ref()
    }
    
    fn changed_at(&self) -> Changed {
        Changed::Static
    }
}

impl<T: AsRef<str> + ?Sized> MaybeObservedText for Immut<T> {
    fn as_text(&self) -> &str {
        self.as_ref()
    }
    
    fn changed_at(&self) -> Changed {
        Changed::Static
    }
}