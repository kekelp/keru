use glam::vec2;
use keru_draw::StyleHandle;

use crate::*;
use std::{hash::{Hash, Hasher}, ops::Range};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChildrenCanHide {
    Yes,
    No,
    Inherit,
}

/// A struct describing the params of a GUI node.
/// 
/// Pass it to [`Ui::add`] to create a node with the given params:
/// ```no_run
/// # use keru::*;
/// # let mut ui: Ui = unimplemented!();
/// # const MY_BUTTON: Node = keru::BUTTON
/// #     .color(RED)
/// #     .shape(Shape::Circle);
/// #
/// ui.add(MY_BUTTON);
/// ```
///
///  You can start with one of the preset constants ([`BUTTON`], [`LABEL`], [`TEXT`], ...), then use the builder methods to customize it:
///
/// ```rust
/// # use keru::*;
/// const MY_BUTTON: Node = keru::BUTTON
///     .color(RED)
///     .shape(Shape::Circle);
/// ```
/// 
/// [`Node`] is a plain-old-data struct. Methods like [`Self::text()`] allow to associate borrowed data like a `&str` to a [`Node`].
/// 
/// The result is a [`FullNode`], a version of this struct that can hold borrowed data. Both versions can be used in the same ways.
#[derive(Debug, Copy, Clone)]
pub struct Node {
    pub key: Option<NodeKey>,
    pub text_params: TextOptions,
    pub stack: Option<Stack>,
    pub shape: Shape,
    pub stroke: Option<Stroke>,
    pub color: ColorFill,
    pub visible: bool, // skip both the shape, node and text
    pub interact: Interact,
    pub layout: Layout,
    pub children_can_hide: ChildrenCanHide,
    /// Clip all children of the node to the node's bounding box (not its shape).
    pub clip_children: Xy<bool>,
    pub animation: Animation,
    pub transform: Transform,
    pub custom_render: bool,
    /// Draw order priority among siblings. Higher value = drawn on top. Default is 0.0.
    /// When two siblings have the same z_index, declaration order is used (later = on top).
    pub z_index: f32,
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
    GrowShrink { axis: Axis, origin: Pos },
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ExitAnimation {
    None,
    Slide { edge: SlideEdge, direction: SlideDirection },
    GrowShrink { axis: Axis, origin: Pos },
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
    Pixels(f32),
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
            Pixels(len) => (0u8, len.to_bits()).hash(state),
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pos {
    Center,
    Start,
    End,
    Pixels(f32),
    Frac(f32),
}

impl Hash for Pos {
    fn hash<H: Hasher>(&self, state: &mut H) {
        use Pos::*;
        match self {
            Center => 0u8.hash(state),
            Start => 1u8.hash(state),
            End => 2u8.hash(state),
            Pixels(p) => (3u8, p.to_bits()).hash(state),
            Frac(f) => (4u8, f.to_bits()).hash(state),
        }
    }
}

/// Options for stack container nodes.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Stack {
    pub arrange: Arrange,
    pub axis: Axis,
    pub spacing: f32,
}
impl Hash for Stack {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.arrange.hash(state);
        self.axis.hash(state);
        self.spacing.to_bits().hash(state);
    }
}
impl Stack {
    pub const DEFAULT: Stack = Stack {
        arrange: Arrange::Center,
        axis: Axis::Y,
        spacing: 5.0,
    };
    pub const fn arrange(mut self, arrange: Arrange) -> Self {
        self.arrange = arrange;
        return self;
    }
    pub const fn spacing(mut self, spacing: f32) -> Self {
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
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Layout {
    pub size: Xy<Size>,
    pub padding: Xy<f32>,
    pub position: Xy<Pos>,
    pub anchor: Xy<Anchor>,
    pub scrollable: Xy<bool>,
}
impl Hash for Layout {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.size.hash(state);
        self.padding.x.to_bits().hash(state);
        self.padding.y.to_bits().hash(state);
        self.position.hash(state);
        self.anchor.hash(state);
        self.scrollable.hash(state);
    }
}

pub use keru_draw::RoundedCorners;

/// The node's shape.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Shape {
    NoShape,
    Rectangle {
        rounded_corners: RoundedCorners,
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
    SquareGrid {
        lattice_size: f32,
        offset: (f32, f32),
        line_thickness: f32,
    },
    HexGrid {
        lattice_size: f32,
        offset: (f32, f32),
        line_thickness: f32,
    },
    /// Regular hexagon shape.
    /// size: 0.0-1.0 relative to node dimensions (1.0 = fills the node)
    /// rotation: in radians, 0 = flat-top
    Hexagon {
        size: f32,
        rotation: f32,
    },
}

impl Hash for Shape {
    fn hash<H: Hasher>(&self, state: &mut H) {
        use Shape::*;
        match self {
            Rectangle { rounded_corners, corner_radius } => {
                0u8.hash(state);
                rounded_corners.hash(state);
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
            SquareGrid { lattice_size, offset, line_thickness } => {
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
            Hexagon { size, rotation } => {
                11u8.hash(state);
                size.to_bits().hash(state);
                rotation.to_bits().hash(state);
            }
            NoShape => 99u8.hash(state),
        }
    }
}

/// The node's visual appearance.
#[derive(Debug, Copy, Clone, PartialEq, Hash)]
pub struct Rect {
    pub shape: Shape,
}

// todo: is the size of this really ok?
/// The visual style of a stroke.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stroke {
    /// Width of the stroke.
    pub width: f32,
    /// Color of the stroke.
    pub color: ColorFill,
    /// Lengths of dashes.
    pub dash_length: f32,
    /// Dash offset.
    pub dash_offset: f32,
}

impl Stroke {
    pub const fn new(width: f32) -> Self {
        Self {
            width,
            color: ColorFill::Color(Color::KERU_GREEN),
            dash_length: 0.0,
            dash_offset: 0.0,
        }
    }

    pub const fn with_dashes(mut self, dash_length: f32,dash_offset: f32) -> Self {
        self.dash_length = dash_length;
        self.dash_offset = dash_offset;
        self
    }

    pub const fn with_color(mut self, color: Color) -> Self {
        self.color = ColorFill::Color(color);
        self
    }
}

impl Hash for Stroke {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.width.to_bits().hash(state);
        self.color.hash(state);
        self.dash_length.to_bits().hash(state);
    }
}

impl Rect {
    pub const DEFAULT: Self = Self {
        shape: Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: DEFAULT_CORNER_RADIUS },
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
    pub auto_markdown: bool,
}

impl Default for TextOptions {
    fn default() -> Self {
        Self::const_default()
    }
}

impl TextOptions {
    pub const fn const_default() -> Self {
        Self {
            editable: false,
            single_line: false,
            selectable: true,
            edit_disabled: false,
            auto_markdown: false,
        }
    }
}

// The corner rounding of most default nodes.
pub const DEFAULT_CORNER_RADIUS: f32 = 9.0;

impl Node {
    pub(crate) fn cosmetic_hash(&self) -> u64 {
        let mut hasher = ahasher();
        self.shape.hash(&mut hasher);
        self.z_index.to_bits().hash(&mut hasher);
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

    /// Create a node for a line segment between two points specified in fractional coordinates (0.0 to 1.0).
    pub fn segment_frac(start: (f32, f32), end: (f32, f32), dash_length: Option<f32>) -> Self {
        let (x1, y1) = start;
        let (x2, y2) = end;

        // Calculate bounding box
        let min_x = x1.min(x2);
        let max_x = x1.max(x2);
        let min_y = y1.min(y2);
        let max_y = y1.max(y2);

        let width = max_x - min_x;
        let height = max_y - min_y;

        // Determine which diagonal
        let x1_is_min = x1 == min_x;
        let y1_is_min = y1 == min_y;

        let seg_start = (if x1_is_min { 0.0 } else { 1.0 }, if y1_is_min { 0.0 } else { 1.0 });
        let seg_end = (if x1_is_min { 1.0 } else { 0.0 }, if y1_is_min { 1.0 } else { 0.0 });

        DEFAULT
            .shape(Shape::Segment {
                start: seg_start,
                end: seg_end,
                dash_length,
            })
            .position_x(Pos::Frac(min_x))
            .position_y(Pos::Frac(min_y))
            .size_x(Size::Frac(width))
            .size_y(Size::Frac(height))
    }

    /// Create a node for a line segment between two points specified in pixel coordinates.
    pub fn segment_px(start: (f32, f32), end: (f32, f32), dash_length: Option<f32>) -> Self {
        let (x1, y1) = start;
        let (x2, y2) = end;

        // Calculate bounding box
        let min_x = x1.min(x2);
        let max_x = x1.max(x2);
        let min_y = y1.min(y2);
        let max_y = y1.max(y2);

        let width = max_x - min_x;
        let height = max_y - min_y;

        // Determine which diagonal
        let x1_is_min = x1 == min_x;
        let y1_is_min = y1 == min_y;

        let seg_start = (if x1_is_min { 0.0 } else { 1.0 }, if y1_is_min { 0.0 } else { 1.0 });
        let seg_end = (if x1_is_min { 1.0 } else { 0.0 }, if y1_is_min { 1.0 } else { 0.0 });

        DEFAULT
            .shape(Shape::Segment {
                start: seg_start,
                end: seg_end,
                dash_length,
            })
            .position_x(Pos::Pixels(min_x))
            .position_y(Pos::Pixels(min_y))
            .size_x(Size::Pixels(width))
            .size_y(Size::Pixels(height))
    }

    pub const fn position(mut self, position_x: Pos, position_y: Pos) -> Self {
        self.layout.position.x = position_x;
        self.layout.position.y = position_y;
        return self;
    }

    pub const fn position_symm(mut self, position: Pos) -> Self {
        self.layout.position.x = position;
        self.layout.position.y = position;
        return self;
    }

    pub const fn position_x(mut self, position: Pos) -> Self {
        self.layout.position.x = position;
        return self;
    }

    pub const fn position_y(mut self, position: Pos) -> Self {
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
        self.visible = true;
        return self;
    }
    pub const fn invisible(mut self) -> Self {
        self.visible = false;
        return self;
    }

    pub const fn filled(mut self) -> Self {
        self.stroke = None;
        return self;
    }

    pub const fn stroke_width(mut self, width: f32) -> Self {
        if let Some(stroke) = &mut self.stroke {
            stroke.width = width;
        } else {
            self.stroke = Some(Stroke::new(width))
        }
        return self;
    }

    pub const fn stroke_dashes(mut self, dash_length: f32, dash_offset: f32) -> Self {
        if let Some(stroke) = self.stroke {
            self.stroke = Some(stroke.with_dashes(dash_length, dash_offset));
        }
        return self;
    }

    pub const fn stroke_color(mut self, color: Color) -> Self {
        if let Some(stroke) = self.stroke {
            self.stroke = Some(stroke.with_color(color));
        }
        return self;
    }

    pub const fn color(mut self, color: Color) -> Self {
        self.color = ColorFill::Color(color);
        return self;
    }

    pub const fn gradient(mut self, gradient: Gradient) -> Self {
        self.color = ColorFill::Gradient(gradient);
        return self;
    }

    pub const fn fill(mut self, fill: ColorFill) -> Self {
        self.color = fill;
        return self;
    }

    pub const fn shape(mut self, shape: Shape) -> Self {
        self.shape = shape;
        return self;
    }

    pub const fn circle(mut self) -> Self {
        self.shape = Shape::Circle;
        return self;
    }

    /// Set the draw order priority among siblings.
    /// 
    /// Siblings with a higher value will be drawn on top. The default value is zero.
    pub const fn z_index(mut self, z_index: f32) -> Self {
        self.z_index = z_index;
        return self;
    }

    pub const fn stack(mut self, axis: Axis, arrange: Arrange, spacing: f32) -> Self {
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

    pub const fn stack_spacing(mut self, spacing: f32) -> Self {
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

    pub const fn padding(mut self, pixels: f32) -> Self {
        self.layout.padding = Xy::new_symm(pixels);
        return self;
    }

    pub const fn padding_x(mut self, padding: f32) -> Self {
        self.layout.padding.x = padding;
        return self;
    }

    pub const fn padding_y(mut self, padding: f32) -> Self {
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

    pub const fn auto_markdown(mut self, auto_markdown: bool) -> Self {
        self.text_params.auto_markdown = auto_markdown;
        return self;
    }

    // todo: rename to opaque or something like that
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

    pub const fn enter_grow(mut self, axis: Axis, origin: Pos) -> Self {
        self.animation.enter = EnterAnimation::GrowShrink { axis, origin };
        return self;
    }

    // Exit animation methods
    pub const fn exit_slide(mut self, edge: SlideEdge, direction: SlideDirection) -> Self {
        self.animation.exit = ExitAnimation::Slide { edge, direction };
        return self;
    }

    pub const fn exit_shrink(mut self, axis: Axis, origin: Pos) -> Self {
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

    pub const fn grow_shrink(mut self, axis: Axis, origin: Pos) -> Self {
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
    pub fn children_can_hide(mut self, value: bool) -> Self {
        self.children_can_hide = if value { ChildrenCanHide::Yes } else { ChildrenCanHide::No };
        return self;
    }

    pub fn children_can_hide_inherit(mut self) -> Self {
        self.children_can_hide = ChildrenCanHide::Inherit;
        return self;
    }

    pub const fn sense_click_release(mut self, value: bool) -> Self {
        let senses = &mut self.interact.senses;
        if value {
            *senses = senses.union(Sense::CLICK_RELEASE);
        } else {
            *senses = senses.intersection(Sense::CLICK_RELEASE.complement());
        }
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

    pub const fn sense_scroll(mut self, value: bool) -> Self {
        let senses = &mut self.interact.senses;
        if value {
            *senses = senses.union(Sense::SCROLL);
        } else {
            *senses = senses.intersection(Sense::SCROLL.complement());
        }
        return self;
    }

    pub const fn sense_drag_drop_target(mut self, value: bool) -> Self {
        let senses = &mut self.interact.senses;
        if value {
            *senses = senses.union(Sense::DRAG_DROP_TARGET);
        } else {
            *senses = senses.intersection(Sense::DRAG_DROP_TARGET.complement());
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

    pub const fn clip_children(mut self, value: bool) -> Self {
        self.clip_children = Xy::new(value, value);
        return self;
    }

    pub const fn translate(mut self, x: f32, y: f32) -> Self {
        self.transform.offset = vec2(x, y);
        return self;
    }

    /// Apply a zoom centered at the center of the node's rect.
    pub const fn scale(mut self, scale: f32) -> Self {
        self.transform.scale = scale;
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

    pub fn click_animation(mut self, value: bool) -> Self {
        self.interact.click_animation = value;
        return self;
    }
}


#[derive(Copy, Clone, Hash)]
pub enum NodeText<'a> {
    /// Text that may change between frames and will be hashed to detect changes.
    Dynamic(&'a str),
    /// Text that doesn't change between frames that can be compared using pointer equality.
    /// If the text is changed, the [`Ui`] will probably miss it.
    Immut(&'a str),
    /// Static text that can use pointer-equality and can be stored as a reference without copying.
    /// If the text is changed using unsafe code, the [`Ui`] will probably miss it.
    Static(&'static str),

    // /// Rich text that may change between frames and will be hashed to detect changes.
    // RichDynamic(&'a RichText<'a>),
    // /// Rich ext that doesn't change between frames that can be compared using pointer equality.
    // /// If the text is changed, the [`Ui`] will probably miss it.
    // RichImmut(&'a RichText<'a>),
    // /// Static rich text that can use pointer-equality and can be stored as a reference without copying.
    // /// If the text is changed using unsafe code, the [`Ui`] will probably miss it.
    // RichStatic(&'a RichText<'static>),
}

#[derive(Copy, Clone, Hash)]
pub struct RichText<'a> {
    spans: &'a [StyledSpan<'a>],
}

#[derive(Copy, Clone, Hash)]
pub struct StyledSpan<'a> {
    text: &'a str,
    style: i32,
}

impl<'a> NodeText<'a> {
    pub fn as_str(&self) -> &str {
        match self {
            NodeText::Dynamic(s) => s,
            NodeText::Static(s) => s,
            NodeText::Immut(s) => s,
        }
    }
}

/// Data for an image to be displayed
#[derive(Copy, Clone)]
pub enum Image<'a> {
    /// Raster image from static bytes (PNG, JPEG, etc.)
    RasterStatic(&'static [u8]),
    /// Raster image from filesystem path
    RasterPath(&'a str),
    /// SVG image from static bytes
    SvgStatic(&'static [u8]),
    /// SVG image from filesystem path
    SvgPath(&'a str),
}

/// An extended version of [`Node`] that can hold text or other borrowed data.
///
/// Created starting from a [`Node`] and using methods like [`Node::text()`].
///
/// Can be used in the same way as [`Node`].
#[derive(Copy, Clone)]
pub struct FullNode<'a> {
    pub node: Node,
    pub text: Option<NodeText<'a>>,
    pub text_style: Option<StyleHandle>,
    pub image: Option<Image<'a>>,
    pub placeholder_text: Option<NodeText<'a>>,
}

impl<'a> FullNode<'a> {
    pub const fn single_line_text(mut self, value: bool) -> Self {
        self.node.text_params.single_line = value;
        return self;
    }

    pub const fn position(mut self, position_x: Pos, position_y: Pos) -> Self {
        self.node.layout.position.x = position_x;
        self.node.layout.position.y = position_y;
        return self;
    }

    pub const fn position_symm(mut self, position: Pos) -> Self {
        self.node.layout.position.x = position;
        self.node.layout.position.y = position;
        return self;
    }

    pub const fn position_x(mut self, position: Pos) -> Self {
        self.node.layout.position.x = position;
        return self;
    }

    pub const fn position_y(mut self, position: Pos) -> Self {
        self.node.layout.position.y = position;
        return self;
    }

    pub const fn anchor(mut self, anchor_x: Anchor, anchor_y: Anchor) -> Self {
        self.node.layout.anchor.x = anchor_x;
        self.node.layout.anchor.y = anchor_y;
        return self;
    }

    pub const fn anchor_symm(mut self, anchor: Anchor) -> Self {
        self.node.layout.anchor.x = anchor;
        self.node.layout.anchor.y = anchor;
        return self;
    }

    pub const fn anchor_x(mut self, anchor: Anchor) -> Self {
        self.node.layout.anchor.x = anchor;
        return self;
    }

    pub const fn anchor_y(mut self, anchor: Anchor) -> Self {
        self.node.layout.anchor.y = anchor;
        return self;
    }

    pub const fn size(mut self, size_x: Size, size_y: Size) -> Self {
        self.node.layout.size.x = size_x;
        self.node.layout.size.y = size_y;
        return self;
    }

    pub const fn size_x(mut self, size_x: Size) -> Self {
        self.node.layout.size.x = size_x;
        return self;
    }

    pub const fn size_y(mut self, size_y: Size) -> Self {
        self.node.layout.size.y = size_y;
        return self;
    }

    pub const fn size_symm(mut self, size: Size) -> Self {
        self.node.layout.size.x = size;
        self.node.layout.size.y = size;
        return self;
    }

    pub const fn visible(mut self) -> Self {
        self.node.visible = true;
        return self;
    }
    pub const fn invisible(mut self) -> Self {
        self.node.visible = false;
        return self;
    }

    pub const fn filled(mut self) -> Self {
        self.node.stroke = None;
        return self;
    }

    pub const fn stroke(mut self, width: f32) -> Self {
        match &mut self.node.stroke {
            Some(stroke) => stroke.width = width,
            None => {
                self.node.stroke = Some(Stroke::new(width))
            },
        }
        return self;
    }

    pub const fn stroke_dashes(mut self, dash_length: f32, dash_offset: f32) -> Self {
        if let Some(stroke) = self.node.stroke {
            self.node.stroke = Some(stroke.with_dashes(dash_length, dash_offset));
        }
        return self;
    }

    pub const fn stroke_color(mut self, color: Color) -> Self {
        if let Some(stroke) = self.node.stroke {
            self.node.stroke = Some(stroke.with_color(color));
        }
        return self;
    }

    pub const fn color(mut self, color: Color) -> Self {
        self.node.color = ColorFill::Color(color);
        return self;
    }

    pub const fn gradient(mut self, gradient: Gradient) -> Self {
        self.node.color = ColorFill::Gradient(gradient);
        return self;
    }

    pub const fn fill(mut self, fill: ColorFill) -> Self {
        self.node.color = fill;
        return self;
    }

    pub const fn shape(mut self, shape: Shape) -> Self {
        self.node.shape = shape;
        return self;
    }

    pub const fn circle(mut self) -> Self {
        self.node.shape = Shape::Circle;
        return self;
    }

    pub const fn stack(mut self, axis: Axis, arrange: Arrange, spacing: f32) -> Self {
        self.node.stack = Some(Stack {
            arrange,
            axis,
            spacing,
        });
        return self;
    }

    pub const fn stack_arrange(mut self, arrange: Arrange) -> Self {
        let stack = match self.node.stack {
            Some(stack) => stack,
            None => Stack::DEFAULT,
        };
        self.node.stack = Some(stack.arrange(arrange));
        return self;
    }

    pub const fn stack_spacing(mut self, spacing: f32) -> Self {
        let stack = match self.node.stack {
            Some(stack) => stack,
            None => Stack::DEFAULT,
        };
        self.node.stack = Some(stack.spacing(spacing));
        return self;
    }

    // todo: if we don't mind sacrificing symmetry, it could make sense to just remove this one.
    pub const fn stack_axis(mut self, axis: Axis) -> Self {
        let stack = match self.node.stack {
            Some(stack) => stack,
            None => Stack::DEFAULT,
        };
        self.node.stack = Some(stack.axis(axis));
        return self;
    }

    pub const fn padding(mut self, padding: f32) -> Self {
        self.node.layout.padding = Xy::new_symm(padding);
        return self;
    }

    pub const fn padding_x(mut self, padding: f32) -> Self {
        self.node.layout.padding.x = padding;
        return self;
    }

    pub const fn padding_y(mut self, padding: f32) -> Self {
        self.node.layout.padding.y = padding;
        return self;
    }

    pub const fn scrollable_x(mut self, scrollable_x: bool) -> Self {
        self.node.layout.scrollable.x = scrollable_x;
        return self;
    }

    pub const fn scrollable_y(mut self, scrollable_y: bool) -> Self {
        self.node.layout.scrollable.y = scrollable_y;
        return self;
    }

    pub const fn auto_markdown(mut self, auto_markdown: bool) -> Self {
        self.node.text_params.auto_markdown = auto_markdown;
        return self;
    }

    pub const fn absorbs_clicks(mut self, absorbs_clicks: bool) -> Self {
        self.node.interact.absorbs_mouse_events = absorbs_clicks;
        return self;
    }

    pub const fn sense_click(mut self, value: bool) -> Self {
        let senses = &mut self.node.interact.senses;
        if value {
            *senses = senses.union(Sense::CLICK);
        } else {
            *senses = senses.intersection(Sense::CLICK.complement());
        }
        return self;
    }

    pub const fn sense_drag(mut self, value: bool) -> Self {
        let senses = &mut self.node.interact.senses;
        if value {
            *senses = senses.union(Sense::DRAG);
        } else {
            *senses = senses.intersection(Sense::DRAG.complement());
        }
        return self;
    }

    pub const fn sense_hover(mut self, value: bool) -> Self {
        let senses = &mut self.node.interact.senses;
        if value {
            *senses = senses.union(Sense::HOVER);
        } else {
            *senses = senses.intersection(Sense::HOVER.complement());
        }
        return self;
    }

    pub const fn sense_hold(mut self, value: bool) -> Self {
        let senses = &mut self.node.interact.senses;
        if value {
            *senses = senses.union(Sense::HOLD);
        } else {
            *senses = senses.intersection(Sense::HOLD.complement());
        }
        return self;
    }

    pub const fn sense_scroll(mut self, value: bool) -> Self {
        let senses = &mut self.node.interact.senses;
        if value {
            *senses = senses.union(Sense::SCROLL);
        } else {
            *senses = senses.intersection(Sense::SCROLL.complement());
        }
        return self;
    }

    pub const fn sense_drag_drop_target(mut self, value: bool) -> Self {
        let senses = &mut self.node.interact.senses;
        if value {
            *senses = senses.union(Sense::DRAG_DROP_TARGET);
        } else {
            *senses = senses.intersection(Sense::DRAG_DROP_TARGET.complement());
        }
        return self;
    }

    /// Add a [`NodeKey`] to the [`Node`].
    /// 
    pub fn key(mut self, key: NodeKey) -> Self {
        self.node.key = Some(key);
        return self;
    }

    pub const fn animation(mut self, animation: Animation) -> Self {
        self.node.animation = animation;
        return self;
    }

    pub const fn animation_speed(mut self, speed: f32) -> Self {
        self.node.animation.speed = speed;
        return self;
    }

    // Enter animation methods
    pub const fn enter_slide(mut self, edge: SlideEdge, direction: SlideDirection) -> Self {
        self.node.animation.enter = EnterAnimation::Slide { edge, direction };
        return self;
    }

    pub const fn enter_grow(mut self, axis: Axis, origin: Pos) -> Self {
        self.node.animation.enter = EnterAnimation::GrowShrink { axis, origin };
        return self;
    }

    // Exit animation methods
    pub const fn exit_slide(mut self, edge: SlideEdge, direction: SlideDirection) -> Self {
        self.node.animation.exit = ExitAnimation::Slide { edge, direction };
        return self;
    }

    pub const fn exit_shrink(mut self, axis: Axis, origin: Pos) -> Self {
        self.node.animation.exit = ExitAnimation::GrowShrink { axis, origin };
        return self;
    }

    pub const fn slide_from_top(mut self) -> Self {
        self.node.animation.enter = EnterAnimation::Slide { edge: SlideEdge::Top, direction: SlideDirection::In };
        self.node.animation.exit = ExitAnimation::Slide { edge: SlideEdge::Top, direction: SlideDirection::Out };
        return self;
    }

    pub const fn slide_from_bottom(mut self) -> Self {
        self.node.animation.enter = EnterAnimation::Slide { edge: SlideEdge::Bottom, direction: SlideDirection::In };
        self.node.animation.exit = ExitAnimation::Slide { edge: SlideEdge::Bottom, direction: SlideDirection::Out };
        return self;
    }

    pub const fn slide_from_left(mut self) -> Self {
        self.node.animation.enter = EnterAnimation::Slide { edge: SlideEdge::Left, direction: SlideDirection::In };
        self.node.animation.exit = ExitAnimation::Slide { edge: SlideEdge::Left, direction: SlideDirection::Out };
        return self;
    }

    pub const fn slide_from_right(mut self) -> Self {
        self.node.animation.enter = EnterAnimation::Slide { edge: SlideEdge::Right, direction: SlideDirection::In };
        self.node.animation.exit = ExitAnimation::Slide { edge: SlideEdge::Right, direction: SlideDirection::Out };
        return self;
    }

    pub const fn grow_shrink(mut self, axis: Axis, origin: Pos) -> Self {
        self.node.animation.enter = EnterAnimation::GrowShrink { axis, origin };
        self.node.animation.exit = ExitAnimation::GrowShrink { axis, origin };
        return self;
    }

    pub const fn animate_position(mut self, value: bool) -> Self {
        self.node.animation.state_transition.animate_position = value;
        return self;
    }

    pub fn is_fit_content(&self) -> bool {
        let Xy { x, y } = self.node.layout.size;
        return x == Size::FitContent || y == Size::FitContent
    }

    pub const fn is_scrollable(&self) -> bool {
        return self.node.layout.scrollable.x || self.node.layout.scrollable.y
    }

    pub fn children_can_hide(mut self, value: bool) -> Self {
        self.node.children_can_hide = if value { ChildrenCanHide::Yes } else { ChildrenCanHide::No };
        return self;
    }

    pub fn children_can_hide_inherit(mut self) -> Self {
        self.node.children_can_hide = ChildrenCanHide::Inherit;
        return self;
    }

    /// Set the text style for this node.
    pub fn text_style(mut self, style: StyleHandle) -> Self {
        self.text_style = Some(style);
        return self;
    }

    /// Set placeholder text for a text edit that will be shown when the text edit is empty.
    /// This only works with editable text nodes.
    ///
    /// Uses hashing to detect if the text has changed.
    pub fn placeholder_text(mut self, placeholder_text: &'a str) -> Self {
        self.placeholder_text = Some(NodeText::Dynamic(placeholder_text));
        self
    }

    /// Set placeholder text from a `&'static str`.
    ///
    /// Uses pointer equality to determine if the text needs updating.
    pub fn placeholder_text_static(mut self, placeholder_text: &'static str) -> FullNode<'a> {
        self.placeholder_text = Some(NodeText::Static(placeholder_text));
        self
    }

    /// Set placeholder text from a `&str` that is known to not change during its lifetime.
    ///
    /// Uses pointer equality to determine if the text needs updating.
    pub fn placeholder_text_immut(mut self, placeholder_text: &'a str) -> Self {
        self.placeholder_text = Some(NodeText::Immut(placeholder_text));
        self
    }

    pub const fn clip_children(mut self, value: Xy<bool>) -> Self {
        self.node.clip_children = value;
        return self;
    }

    pub const fn clip_children_x(mut self, value: bool) -> Self {
        self.node.clip_children.x = value;
        return self;
    }

    pub const fn clip_children_y(mut self, value: bool) -> Self {
        self.node.clip_children.y = value;
        return self;
    }

    pub const fn custom_render(mut self, value: bool) -> Self {
        self.node.custom_render = value;
        return self;
    }
}

impl Node {
    /// Set placeholder text for a text edit that will be shown when the text edit is empty.
    /// This only works with editable text nodes.
    ///
    /// Uses hashing to detect if the text has changed.
    pub fn placeholder_text<'a>(self, placeholder: &'a str) -> FullNode<'a> {
        return FullNode {
            node: self,
            text: None,
            text_style: None,
            image: None,
            placeholder_text: Some(NodeText::Dynamic(placeholder)),
        }
    }

    /// Set placeholder text from a `&'static str`.
    ///
    /// Uses pointer equality to determine if the text needs updating.
    pub fn placeholder_text_static(self, placeholder: &'static str) -> FullNode<'static> {
        return FullNode {
            node: self,
            text: None,
            text_style: None,
            image: None,
            placeholder_text: Some(NodeText::Static(placeholder)),
        }
    }

    /// Set placeholder text from a `&str` that is known to not change during its lifetime.
    ///
    /// Uses pointer equality to determine if the text needs updating.
    pub fn placeholder_text_immut<'a>(self, placeholder: &'a str) -> FullNode<'a> {
        return FullNode {
            node: self,
            text: None,
            text_style: None,
            image: None,
            placeholder_text: Some(NodeText::Immut(placeholder)),
        }
    }

    /// Add text to the [`Node`] from a `&'static str`.
    ///
    /// Uses pointer equality to determine if the text needs updating.
    pub fn static_text(self, text: &'static str) -> FullNode<'static> {
        return FullNode {
            node: self,
            text: Some(NodeText::Static(text)),
            text_style: None,
            image: None,
            placeholder_text: None,
        }
    }

    /// Add text to the [`Node`] from a `&str` that is known to not change during its lifetime.
    ///
    /// Uses pointer equality to determine if the text needs updating. The user must ensure
    /// that the text content doesn't change, otherwise the display will get out of sync.
    pub fn immut_text(self, text: &str) -> FullNode<'_> {
        return FullNode {
            node: self,
            text: Some(NodeText::Immut(text)),
            text_style: None,
            image: None,
            placeholder_text: None,
        }
    }

    pub fn static_image(self, image: &'static [u8]) -> FullNode<'static> {
        return FullNode {
            node: self,
            text: None,
            text_style: None,
            image: Some(Image::RasterStatic(image)),
            placeholder_text: None,
        }
    }

    pub fn image_path<'a>(self, path: &'a str) -> FullNode<'a> {
        return FullNode {
            node: self,
            text: None,
            text_style: None,
            image: Some(Image::RasterPath(path)),
            placeholder_text: None,
        }
    }

    pub fn static_svg(self, svg: &'static [u8]) -> FullNode<'static> {
        return FullNode {
            node: self,
            text: None,
            text_style: None,
            image: Some(Image::SvgStatic(svg)),
            placeholder_text: None,
        }
    }

    pub fn svg_path<'a>(self, path: &'a str) -> FullNode<'a> {
        return FullNode {
            node: self,
            text: None,
            text_style: None,
            image: Some(Image::SvgPath(path)),
            placeholder_text: None,
        }
    }
}

impl From<Node> for FullNode<'_> {
    fn from(val: Node) -> Self {
        FullNode {
            node: val,
            text: None,
            text_style: None,
            image: None,
            placeholder_text: None,
        }
    }
}

impl FullNode<'_> {
    #[track_caller]
    pub(crate) fn key_or_anon_key(&self) -> NodeKey {
        return match self.node.key {
            Some(key) => key,
            None => NodeKey::new(Id(caller_location_id()), "Anon node"),
        };
    }
}

impl Ui {
    pub(crate) fn set_params_text(&mut self, i: NodeI, params: &FullNode) {
        let Some(text) = params.text else {
            return
        };

        let text_options = params.node.text_params;
        let style = params.text_style.as_ref();

        let needs_new_widget = match (&self.sys.nodes[i].text_i, text_options.editable) {
            (None, _) => true,
            (Some(TextI::TextEdit(_)), true) => false,
            (Some(TextI::TextBox(_)), false) => false,
            _ => true, // need to switch
        };

        if needs_new_widget {
            // Remove old widget
            if let Some(old_text_i) = self.sys.nodes[i].text_i.take() {
                match old_text_i {
                    TextI::TextBox(handle) => self.sys.renderer.text.remove_text_box(handle),
                    TextI::TextEdit(handle) => self.sys.renderer.text.remove_text_edit(handle),
                }
            }

            // this z doesn't matter, it's set when preparing render data. todo: cleanup.
            let z = 0.0;
            // Create new widget
            let new_text_i = if text_options.editable {
                let handle = self.sys.renderer.text.add_text_edit(text.as_str().to_string(), (0.0, 0.0), (500.0, 500.0), z);
                if let Some(style) = style {
                    self.sys.renderer.text.get_text_edit_mut(&handle).set_style(style);
                }
                TextI::TextEdit(handle)
            } else {
                let handle = match text {
                    NodeText::Static(s) => {
                        self.sys.renderer.text.add_text_box(s, (0.0, 0.0), (500.0, 500.0), z)
                    },
                    NodeText::Dynamic(s) | NodeText::Immut(s) => {
                        self.sys.renderer.text.add_text_box(s.to_string(), (0.0, 0.0), (500.0, 500.0), z)
                    }
                };
                if let Some(style) = style {
                    self.sys.renderer.text.get_text_box_mut(&handle).set_style(style);
                }
                TextI::TextBox(handle)
            };

            self.sys.nodes[i].text_i = Some(new_text_i);
        } else {
            // Same type - just update content and style
            match &self.sys.nodes[i].text_i {
                Some(TextI::TextEdit(handle)) => {
                    // don't update the content. content in a text edit box is not reset declaratively every frame, obviously.
                    if let Some(style) = style {
                        self.sys.renderer.text.get_text_edit_mut(&handle).set_style(style);
                    }
                },
                Some(TextI::TextBox(handle)) => {
                    match text {
                        NodeText::Static(s) => {
                            self.sys.renderer.text.get_text_box_mut(&handle).set_static_text_with_pointer_check(s);
                        },
                        NodeText::Dynamic(s) => {
                            self.sys.renderer.text.get_text_box_mut(&handle).set_text_hashed(s);
                        },
                        NodeText::Immut(s) => {
                            self.sys.renderer.text.get_text_box_mut(&handle).set_text_with_pointer_check(s);
                        }
                    }
                    if let Some(style) = style {
                        self.sys.renderer.text.get_text_box_mut(&handle).set_style(style);
                    }
                },
                None => unreachable!("Should have created a new widget above"),
            }
        }

        // Apply text options
        if let Some(text_i) = &self.sys.nodes[i].text_i {
            match text_i {
                TextI::TextEdit(handle) => {
                    self.sys.renderer.text.get_text_edit_mut(handle).set_disabled(text_options.edit_disabled);
                    self.sys.renderer.text.get_text_edit_mut(handle).set_single_line(text_options.single_line);
                    if let Some(placeholder) = params.placeholder_text {
                        match placeholder {
                            NodeText::Static(s) => {
                                self.sys.renderer.text.get_text_edit_mut(handle).set_placeholder_static_with_pointer_check(s);
                            },
                            NodeText::Dynamic(s) => {
                                self.sys.renderer.text.get_text_edit_mut(handle).set_placeholder_hashed(s);
                            },
                            NodeText::Immut(s) => {
                                self.sys.renderer.text.get_text_edit_mut(handle).set_placeholder_with_pointer_check(s);
                            }
                        }
                    }
                },
                TextI::TextBox(handle) => {
                    self.sys.renderer.text.get_text_box_mut(handle).set_selectable(text_options.selectable);
                },
            }
        }

        // Link this text box into the global cross-box selection chain.
        // Runs every frame so that links are always up-to-date regardless of structural changes.
        if !text_options.editable && text_options.selectable {
            if let Some(TextI::TextBox(current_handle)) = &self.sys.nodes[i].text_i {
                self.sys.renderer.text.unlink_text_box(current_handle);

                if let Some(prev_node_i) = self.sys.last_linked_text_box_node {
                    if let Some(TextI::TextBox(prev_handle)) = &self.sys.nodes[prev_node_i].text_i {
                        self.sys.renderer.text.link_text_boxes(prev_handle, current_handle);
                    }
                }

                self.sys.last_linked_text_box_node = Some(i);
            }
        }
    }


    pub(crate) fn set_params(&mut self, i: NodeI, params: &FullNode) {
        #[cfg(not(debug_assertions))]
        if reactive::is_in_skipped_reactive_block() {
            return;
        }
        
        if let Some(image_data) = params.image {
            match image_data {
                Image::RasterStatic(image) => self.set_static_image(i, image),
                Image::RasterPath(path) => self.set_path_image(i, path),
                Image::SvgStatic(svg) => self.set_static_svg(i, svg),
                Image::SvgPath(path) => self.set_path_svg(i, path),
            };
        }
        
        let new_cosmetic_hash = params.node.cosmetic_hash();
        let new_layout_hash = params.node.layout_hash();
        
        let cosmetic_changed = new_cosmetic_hash != self.sys.nodes[i].last_cosmetic_hash;
        let layout_changed = new_layout_hash != self.sys.nodes[i].last_layout_hash;

        #[cfg(debug_assertions)]
        if reactive::is_in_skipped_reactive_block() {
            if cosmetic_changed || layout_changed {
                let kind = match (layout_changed, cosmetic_changed) {
                    (true, true) => "layout and appearance",
                    (true, false) => "layout",
                    (false, true) => "appearance",
                    _ => unreachable!()
                };
                log::error!("Keru: incorrect reactive block: the {kind} params of node \"{}\" changed, but reactive thought they didn't", self.node_debug_name(i));
                // log::error!("Keru: incorrect reactive block: the {kind} params of node \"{}\" changed, even if a reactive block declared that it shouldn't have.\n Check that the reactive block is correctly checking all the runtime variables that can affect the node's params.", self.node_debug_name(i));
            }
            return;
        }
        
        self.sys.nodes[i].params = params.node.clone();

        self.sys.nodes[i].last_cosmetic_hash = new_cosmetic_hash;
        self.sys.nodes[i].last_layout_hash = new_layout_hash;

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
    pub fn text(self, text: &str) -> FullNode<'_> {
        return FullNode {
            node: self,
            text: Some(NodeText::Dynamic(text)),
            text_style: None,
            image: None,
            placeholder_text: None,
        }
    }
}

impl<'a> FullNode<'a> {
    /// Add text to the [`Node`].
    pub fn text(mut self, text: &'a str) -> FullNode<'a> {
        self.text = Some(NodeText::Dynamic(text));
        return self;
    }
}