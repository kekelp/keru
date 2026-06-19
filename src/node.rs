use glam::vec2;
use bumpalo::collections::Vec as BumpVec;
use bumpalo::collections::String as BumpString;
use keru_draw::parley::Alignment;
use keru_draw::parley::StyleProperty;
use keru_draw::parley::{FontStyle, FontWeight, FontFamily, FontFamilyName, GenericFamily};

const BOLD: TextStyleProperty = TextStyleProperty::FontWeight(FontWeight::new(800.0));
const ITALIC: TextStyleProperty = TextStyleProperty::FontStyle(FontStyle::Italic);
const MONOSPACE: TextStyleProperty = TextStyleProperty::FontFamily(FontFamily::Single(FontFamilyName::Generic(GenericFamily::Monospace)));

/// An individual text style property, 
pub type TextStyleProperty = keru_draw::parley::StyleProperty<'static, ColorBrush>;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct TextStyleFlags: u8 {
        const BOLD = 0b001;
        const ITALIC = 0b010;
        const MONOSPACE = 0b100;
    }
}

use crate::*;
use crate::node_library::*;
use std::{hash::{Hash, Hasher}, ops::Range};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerticalTextAlignment {
    #[default]
    Center,
    Top,
    Bottom,
}

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
/// # use keru::*; use keru::node_library::*; let mut ui: Ui = unimplemented!();
/// # const MY_BUTTON: Node = BUTTON
/// #     .color(Color::RED)
/// #     .shape(Shape::Circle);
/// #
/// ui.add(MY_BUTTON);
/// ```
///
///  You can start with one of the preset constants ([`BUTTON`], [`LABEL`], [`TEXT`], ...), then use the builder methods to customize it:
///
/// ```no_run
/// # use keru::*; use keru::node_library::*; let mut ui: Ui = unimplemented!();
/// const MY_BUTTON: Node = BUTTON
///     .color(Color::RED)
///     .shape(Shape::Circle);
/// ```
/// 
/// [`Node`] is a plain-old-data struct. Methods like [`Self::text()`] allow to associate borrowed data like a `&str` to a [`Node`].
/// 
/// The result is a [`Node`], a version of this struct that can hold borrowed data. Both versions can be used in the same ways.
#[derive(Debug, Copy, Clone)]
pub struct Node<'a> {
    pub key: Option<NodeKey>,
    pub text_options: TextOptions,
    pub children_layout: ChildrenLayout,
    pub shape: Shape,
    pub blur: Option<f32>,
    pub shadow: Option<Shadow>,
    pub second_shadow: Option<Shadow>,
    pub stroke: Option<Stroke>,
    pub color: ColorFill2,
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
    /// If this node is a child of a Grid element, customize its positioning inside the grid.
    pub grid_element: GridElement,
    /// If true and the parent uses Stack or Grid layout, this node ignores that layout and is placed freely within the parent instead.
    pub free_placement: bool,
    /// If true, this node is not shifted by the parent's scroll offset.
    pub ignore_parent_scroll: bool,

    pub text_alignment: Alignment,
    pub vertical_text_alignment: VerticalTextAlignment,

    pub text: Option<NodeText<'a>>,
    pub text_size: Option<f32>,
    pub text_color: Option<Color>,
    pub text_properties: &'a [TextStyleProperty],
    pub text_style_flags: TextStyleFlags,

    pub image: Option<Image<'a>>,
    pub image_options: ImageOptions,
    pub placeholder_text: Option<NodeText<'a>>,

    /// If true, when running in release mode, this node will never be hashed to detect differences and never trigger relayouts or rerenders.
    /// 
    /// In debug mode, the node will still be hashed and diffed, and an error will be reported if it's found to not actually be constant. 
    /// 
    /// Can be used as an extreme micro-optimization for programs that have many nodes that don't depend on any dynamic data.
    pub constant: bool,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Shadow {
    pub blur: f32,
    pub offset: Xy<f32>,
    pub color: Option<Color>,
}

impl Hash for Shadow {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.blur.to_bits().hash(state);
        self.offset.x.to_bits().hash(state);
        self.offset.y.to_bits().hash(state);
        if let Some(c) = self.color {
            c.r.to_bits().hash(state);
            c.g.to_bits().hash(state);
            c.b.to_bits().hash(state);
            c.a.to_bits().hash(state);
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum SlideEdge {
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
        std::mem::discriminant(self).hash(state);
        match self {
            Pixels(len) => len.to_bits().hash(state),
            Frac(len) => len.to_bits().hash(state),
            Fill => {},
            FitContent => {},
            AspectRatio(ratio) => ratio.to_bits().hash(state),
        }
    }
}

/// Anchor point within a node for positioning.
///
/// Determines which point of the node is positioned at the given coordinates
/// when using [`Pos::Frac`] or [`Pos::Pixels`]. For example, with `Anchor::Center`, the
/// center of the node will be placed at the specified position.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Anchor {
    /// Anchor at the start (left/top)
    Start,
    /// Anchor at the center
    Center,
    /// Anchor at the end (right/bottom)
    End,
    /// Anchor at a relative position (0.0 = start, 1.0 = end)
    Frac(f32),
}

impl Hash for Anchor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Anchor::Start => {},
            Anchor::Center => {},
            Anchor::End => {},
            Anchor::Frac(f) => f.to_bits().hash(state),
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
        std::mem::discriminant(self).hash(state);
        match self {
            Center => {},
            Start => {},
            End => {},
            Pixels(p) => p.to_bits().hash(state),
            Frac(f) => f.to_bits().hash(state),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HorizontalOrigin {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VerticalOrigin {
    Top,
    Bottom,
}

/// Determines how the children of the node are laid out in its space.
#[derive(Debug, Clone, Copy)]
pub enum ChildrenLayout {
    /// Children can position themselves freely according to their [`Size`] and [`Pos`] values. 
    Free,
    /// Children are arranged in a stack. Children's [`Pos`] values along the stack axis are ignored.
    Stack {
        arrange: Arrange,
        axis: Axis,
        spacing: f32,
    },
    /// Children are arranged in a grid. Children's [`Pos`] values on both axes are ignored.
    /// 
    /// To give an element a size and position relative to the grid cell, you can add a [`CONTAINER`] node as the direct child of the grid, then add the element as a child of the Container.
    Grid {
        columns: MainAxisCellSize,
        spacing_x: f32,
        spacing_y: f32,
        flow: GridFlow,
    },
}

impl Hash for ChildrenLayout {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);

        match self {
            ChildrenLayout::Free => {},
            ChildrenLayout::Stack { arrange, axis, spacing } => {
                arrange.hash(state);
                axis.hash(state);
                spacing.to_bits().hash(state);
            },
            ChildrenLayout::Grid { columns, spacing_x, spacing_y, flow } => {
                columns.hash(state);
                spacing_x.to_bits().hash(state);
                spacing_y.to_bits().hash(state);
                flow.hash(state);
            },
        }

    }
}

/// How many cells of a grid the node occupies.
/// 
/// Only works if the node is added as a child of a [`ChildrenLayout::Grid`] node, 
#[derive(Debug, Clone, Copy, Hash)]
pub struct GridElement {
    pub row_span: u16,
    pub column_span: u16,
}
impl GridElement {
    pub const ONE_BY_ONE: GridElement = GridElement {
        row_span: 1,
        column_span: 1,
    };
}

/// Controls in which direction grid children are placed.
///
/// `main_axis` determines whether items fill horizontally first (rows) or vertically first (columns).
/// `x_fill_direction: Direction::RightToLeft` places items right-to-left; `y_fill_direction: Direction::RightToLeft` places items bottom-to-top.
#[derive(Debug, Clone, Copy, Hash)]
pub struct GridFlow {
    pub main_axis: Axis,
    pub x_fill_direction: Direction,
    pub y_fill_direction: Direction,
    /// If true, the placement algorithm restarts from the beginning of the grid for each item,
    /// filling gaps left by earlier items with spans (like `grid-auto-flow: dense` in CSS).
    /// If false, the cursor only moves forward and gaps are left unfilled.
    pub backfill: bool,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Direction {
    LeftToRight,
    RightToLeft,
}

impl GridFlow {
    pub const DEFAULT: Self = Self { main_axis: Axis::X, x_fill_direction: Direction::LeftToRight, y_fill_direction: Direction::LeftToRight, backfill: false };
}

/// Specifies how cells are sized along the main axis of a grid layout.
#[derive(Debug, Clone, Copy)]
pub enum MainAxisCellSize {
    /// Fixed number of cells.
    Count(u32),
    /// Target cells width in pixels.
    Width(f32),
}

impl std::hash::Hash for MainAxisCellSize {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            MainAxisCellSize::Count(n) => n.hash(state),
            MainAxisCellSize::Width(w) => w.to_bits().hash(state),
        }
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
    /// Whether the default keyboard focus indicator rect is drawn when this node is focused.
    pub focus_indicator: bool,
    /// Whether the node can receive the keyboard-navigation focus.
    pub focusable: bool,
}

/// The node's layout, size and position.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Layout {
    pub size: Xy<Size>,
    pub padding: Xy<f32>,
    pub position: Xy<Pos>,
    pub anchor: Xy<Anchor>,
    pub pos_origin_x: HorizontalOrigin,
    pub pos_origin_y: VerticalOrigin,
    pub scrollable: Xy<bool>,
}
impl Hash for Layout {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.size.hash(state);
        self.padding.x.to_bits().hash(state);
        self.padding.y.to_bits().hash(state);
        self.position.hash(state);
        self.anchor.hash(state);
        self.pos_origin_x.hash(state);
        self.pos_origin_y.hash(state);
        self.scrollable.hash(state);
    }
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            size: Xy::new_symm(Size::FitContent),
            padding: Xy::new_symm(10.0),
            position: Xy::new_symm(Pos::Center),
            anchor: Xy::new_symm(Anchor::Start),
            pos_origin_x: HorizontalOrigin::Left,
            pos_origin_y: VerticalOrigin::Top,
            scrollable: Xy::new(false, false),
        }
    }
}

impl Layout {
    pub const fn size(mut self, size_x: Size, size_y: Size) -> Self {
        self.size.x = size_x;
        self.size.y = size_y;
        return self;
    }

    pub const fn size_x(mut self, size_x: Size) -> Self {
        self.size.x = size_x;
        return self;
    }

    pub const fn size_y(mut self, size_y: Size) -> Self {
        self.size.y = size_y;
        return self;
    }

    pub const fn size_symm(mut self, size: Size) -> Self {
        self.size.x = size;
        self.size.y = size;
        return self;
    }

    pub const fn position(mut self, position_x: Pos, position_y: Pos) -> Self {
        self.position.x = position_x;
        self.position.y = position_y;
        return self;
    }

    pub const fn position_x(mut self, position: Pos) -> Self {
        self.position.x = position;
        return self;
    }

    pub const fn position_y(mut self, position: Pos) -> Self {
        self.position.y = position;
        return self;
    }

    pub const fn position_symm(mut self, position: Pos) -> Self {
        self.position.x = position;
        self.position.y = position;
        return self;
    }

    pub const fn pos_origin(mut self, origin_x: HorizontalOrigin, origin_y: VerticalOrigin) -> Self {
        self.pos_origin_x = origin_x;
        self.pos_origin_y = origin_y;
        return self;
    }

    pub const fn pos_origin_x(mut self, origin: HorizontalOrigin) -> Self {
        self.pos_origin_x = origin;
        return self;
    }

    pub const fn pos_origin_y(mut self, origin: VerticalOrigin) -> Self {
        self.pos_origin_y = origin;
        return self;
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
        std::mem::discriminant(self).hash(state);
        match self {
            NoShape => {},
            Rectangle { rounded_corners, corner_radius } => {
                rounded_corners.hash(state);
                corner_radius.to_bits().hash(state);
            }
            Circle => {},
            Ring { width } => {
                width.to_bits().hash(state);
            }
            Arc { start_angle, end_angle, width } => {
                start_angle.to_bits().hash(state);
                end_angle.to_bits().hash(state);
                width.to_bits().hash(state);
            }
            Pie { start_angle, end_angle } => {
                start_angle.to_bits().hash(state);
                end_angle.to_bits().hash(state);
            }
            Segment { start, end, dash_length } => {
                start.0.to_bits().hash(state);
                start.1.to_bits().hash(state);
                end.0.to_bits().hash(state);
                end.1.to_bits().hash(state);
                dash_length.map(|len| len.to_bits()).hash(state);
            }
            HorizontalLine => {},
            VerticalLine => {},
            Triangle { rotation, width } => {
                rotation.to_bits().hash(state);
                width.to_bits().hash(state);
            }
            SquareGrid { lattice_size, offset, line_thickness } => {
                lattice_size.to_bits().hash(state);
                offset.0.to_bits().hash(state);
                offset.1.to_bits().hash(state);
                line_thickness.to_bits().hash(state);
            }
            HexGrid { lattice_size, offset, line_thickness } => {
                lattice_size.to_bits().hash(state);
                offset.0.to_bits().hash(state);
                offset.1.to_bits().hash(state);
                line_thickness.to_bits().hash(state);
            }
            Hexagon { size, rotation } => {
                size.to_bits().hash(state);
                rotation.to_bits().hash(state);
            }
        }
    }
}

/// The node's visual appearance.
#[derive(Debug, Copy, Clone, PartialEq, Hash)]
pub struct Rect {
    pub shape: Shape,
}

/// Linear gradient defined relative to the node's bounding box.
/// `angle_deg`: 0 = left→right, 90 = top→bottom.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearGradient {
    pub color_start: Color,
    pub color_end: Color,
    /// Degrees; 0 = left→right, 90 = top→bottom.
    pub angle_deg: f32,
}

impl LinearGradient {
    pub const fn new(color_start: Color, color_end: Color, angle_deg: f32) -> Self {
        Self { color_start, color_end, angle_deg }
    }

    pub const fn with_alpha(self, alpha: f32) -> Self {
        let mut new = self;
        new.color_start = new.color_start.with_alpha(alpha);
        new.color_end = new.color_end.with_alpha(alpha);
        new
    }
}

impl Hash for LinearGradient {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.color_start.r.to_bits().hash(state);
        self.color_start.g.to_bits().hash(state);
        self.color_start.b.to_bits().hash(state);
        self.color_start.a.to_bits().hash(state);
        self.color_end.r.to_bits().hash(state);
        self.color_end.g.to_bits().hash(state);
        self.color_end.b.to_bits().hash(state);
        self.color_end.a.to_bits().hash(state);
        self.angle_deg.to_bits().hash(state);
    }
}

/// Color fill for Nodes.
#[derive(Debug, Clone, Copy, PartialEq)]
// todo: rename
pub enum ColorFill2 {
    Color(Color),
    /// Linear gradient at the given angle (degrees; 0 = left→right, 90 = top→bottom).
    LinearGradient(LinearGradient),
    /// Radial gradient centered in the node; `color_inner` is the center color.
    RadialGradient {
        color_inner: Color,
        color_outer: Color,
    },
    /// Use the linear gradient of another node, identified by the [`NodeKey`].
    SharedGradient(NodeKey),
}

impl ColorFill2 {
    pub(crate) fn resolve(self, x0: f32, y0: f32, x1: f32, y1: f32) -> keru_draw::ColorFill {
        match self {
            ColorFill2::Color(c) => keru_draw::ColorFill::Color(c),
            ColorFill2::LinearGradient(lg) => {
                let cx = (x0 + x1) * 0.5;
                let cy = (y0 + y1) * 0.5;
                let w = x1 - x0;
                let h = y1 - y0;
                let rad = lg.angle_deg.to_radians();
                // half-length of the gradient line so it covers the entire box
                let half_len = (w * 0.5 * rad.cos()).abs() + (h * 0.5 * rad.sin()).abs();
                let dx = rad.cos() * half_len;
                let dy = rad.sin() * half_len;
                let p0 = [cx - dx, cy - dy];
                let p1 = [cx + dx, cy + dy];
                keru_draw::ColorFill::Gradient(keru_draw::Gradient::linear(p0, p1, lg.color_start, lg.color_end))
            },
            ColorFill2::RadialGradient { color_inner, color_outer } => {
                let cx = (x0 + x1) * 0.5;
                let cy = (y0 + y1) * 0.5;
                let w = x1 - x0;
                let h = y1 - y0;
                let outer_radius = w.min(h) * 0.5;
                keru_draw::ColorFill::Gradient(keru_draw::Gradient::radial([cx, cy], outer_radius, 0.0, color_inner, color_outer))
            },
            ColorFill2::SharedGradient(_) => keru_draw::ColorFill::Color(Color::TRANSPARENT),
        }
    }

    /// Like `resolve`, but for circular shapes that have explicit inner/outer radii.
    /// For `RadialGradient`, uses those radii directly instead of deriving them from the bounding box.
    pub(crate) fn resolve_radial(self, cx: f32, cy: f32, inner_r: f32, outer_r: f32, x0: f32, y0: f32, x1: f32, y1: f32) -> keru_draw::ColorFill {
        match self {
            ColorFill2::RadialGradient { color_inner, color_outer } => {
                keru_draw::ColorFill::Gradient(keru_draw::Gradient::radial([cx, cy], outer_r, inner_r, color_inner, color_outer))
            }
            other => other.resolve(x0, y0, x1, y1),
        }
    }

    pub(crate) fn darken(self, factor: f32) -> Self {
        let d = |c: Color| Color::new(c.r * factor, c.g * factor, c.b * factor, c.a);
        match self {
            ColorFill2::Color(c) => ColorFill2::Color(d(c)),
            ColorFill2::LinearGradient(lg) => ColorFill2::LinearGradient(LinearGradient {
                color_start: d(lg.color_start),
                color_end: d(lg.color_end),
                angle_deg: lg.angle_deg,
            }),
            ColorFill2::RadialGradient { color_inner, color_outer } =>
                ColorFill2::RadialGradient { color_inner: d(color_inner), color_outer: d(color_outer) },
            ColorFill2::SharedGradient(_) => panic!("darken called on SharedGradient; resolve first"),
        }
    }
}

// todo: is the size of this really ok?
/// The visual style of a stroke.
impl Hash for ColorFill2 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            ColorFill2::Color(c) => {
                c.r.to_bits().hash(state);
                c.g.to_bits().hash(state);
                c.b.to_bits().hash(state);
                c.a.to_bits().hash(state);
            }
            ColorFill2::LinearGradient(lg) => lg.hash(state),
            ColorFill2::RadialGradient { color_inner, color_outer } => {
                color_inner.r.to_bits().hash(state);
                color_inner.g.to_bits().hash(state);
                color_inner.b.to_bits().hash(state);
                color_inner.a.to_bits().hash(state);
                color_outer.r.to_bits().hash(state);
                color_outer.g.to_bits().hash(state);
                color_outer.b.to_bits().hash(state);
                color_outer.a.to_bits().hash(state);
            }
            ColorFill2::SharedGradient(k) => {
                k.hash(state);
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stroke {
    /// Width of the stroke.
    pub width: f32,
    /// Color of the stroke.
    pub color: ColorFill2,
    /// Lengths of dashes.
    pub dash_length: f32,
    /// Dash offset.
    pub dash_offset: f32,
}

impl Stroke {
    pub const fn new(width: f32) -> Self {
        Self {
            width,
            color: ColorFill2::Color(Color::KERU_GREEN),
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
        self.color = ColorFill2::Color(color);
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
    pub use_pointer_comparison: bool,
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
            use_pointer_comparison: false,
        }
    }
}

// The corner rounding of most default nodes.
pub const DEFAULT_CORNER_RADIUS: f32 = 9.0;

impl<'a> Node<'a> {
    pub(crate) fn cosmetic_hash(&self) -> u64 {
        let mut h = ahasher();
        self.shape.hash(&mut h);
        self.z_index.to_bits().hash(&mut h);
        self.color.hash(&mut h);
        self.blur.map(|v| v.to_bits()).hash(&mut h);
        self.shadow.hash(&mut h);
        self.second_shadow.hash(&mut h);
        self.stroke.hash(&mut h);
        self.animation.speed.to_bits().hash(&mut h);
        std::mem::discriminant(&self.animation.enter).hash(&mut h);
        match self.animation.enter {
            EnterAnimation::None => {},
            EnterAnimation::Slide { edge, direction } => { edge.hash(&mut h); direction.hash(&mut h); },
            EnterAnimation::GrowShrink { axis, origin } => { axis.hash(&mut h); origin.hash(&mut h); },
        }
        std::mem::discriminant(&self.animation.exit).hash(&mut h);
        match self.animation.exit {
            ExitAnimation::None => {},
            ExitAnimation::Slide { edge, direction } => { edge.hash(&mut h); direction.hash(&mut h); },
            ExitAnimation::GrowShrink { axis, origin } => { axis.hash(&mut h); origin.hash(&mut h); },
        }
        self.animation.state_transition.animate_position.hash(&mut h);
        self.transform.offset.x.to_bits().hash(&mut h);
        self.transform.offset.y.to_bits().hash(&mut h);
        self.transform.scale.to_bits().hash(&mut h);
        self.custom_render.hash(&mut h);
        self.interact.hash(&mut h);
        (self.text_alignment as u8).hash(&mut h);
        self.vertical_text_alignment.hash(&mut h);
        self.text_color.map(|c| (c.r.to_bits(), c.g.to_bits(), c.b.to_bits(), c.a.to_bits())).hash(&mut h);
        self.text_style_flags.hash(&mut h);
        return h.finish();
    }

    pub(crate) fn layout_hash(&self) -> u64 {
        let mut h = ahasher();
        self.layout.hash(&mut h);
        self.children_layout.hash(&mut h);
        self.text_options.hash(&mut h);
        self.text_size.map(|v| v.to_bits()).hash(&mut h);
        self.grid_element.hash(&mut h);
        self.free_placement.hash(&mut h);
        self.clip_children.hash(&mut h);
        self.visible.hash(&mut h);
        self.children_can_hide.hash(&mut h);
        self.ignore_parent_scroll.hash(&mut h);
        return h.finish();
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

    /// Set the stroke width.
    pub const fn stroke_width(mut self, width: f32) -> Self {
        if let Some(stroke) = &mut self.stroke {
            stroke.width = width;
        } else {
            self.stroke = Some(Stroke::new(width))
        }
        return self;
    }

    /// Set the stroke width.
    pub const fn stroke_linear_gradient(mut self, gradient: LinearGradient) -> Self {
        if let Some(stroke) = &mut self.stroke {
            stroke.color = ColorFill2::LinearGradient(gradient);
        } else {
            self.stroke = Some(Stroke::new(5.0));
            self.stroke.unwrap().color = ColorFill2::LinearGradient(gradient);
        }
        return self;
    }

    /// Set the draw order priority among siblings.
    /// 
    /// Siblings with a higher value will be drawn on top. The default value is zero.
    pub const fn z_index(mut self, z_index: f32) -> Self {
        self.z_index = z_index;
        return self;
    }

    /// Set children layout to a grid.
    pub const fn grid(mut self, cells: MainAxisCellSize, spacing_x: f32, spacing_y: f32, flow: GridFlow) -> Self {
        self.children_layout = ChildrenLayout::Grid { columns: cells, spacing_x, spacing_y, flow };
        return self;
    }

    /// Set the number of grid rows this node spans when it is added as a child of a `Grid` node.
    pub const fn grid_row_span(mut self, span: u16) -> Self {
        self.grid_element.row_span = span;
        return self;
    }

    /// Set the number of grid columns this node spans when it is added as a child of a `Grid` node.
    pub const fn grid_column_span(mut self, span: u16) -> Self {
        self.grid_element.column_span = span;
        return self;
    }

    /// Sets whether a node's children stay hidden or get removed when they get excluded from the tree.
    /// 
    /// If a node stays hidden, it retains its internal state (scroll offset, text input, ...), and it is slightly less expensive to bring them back into view. If it gets removed, its memory can be reused for other nodes. 
    /// 
    /// For example, the panel with the main content in a tabbed application should use [`children_can_hide(true)`](`Node::children_can_hide`), so that all state is retained when switching tabs.
    ///
    /// On the other hand, if a panel that contains dynamic content, it should stick to the default [`children_can_hide(false)`](`Node::children_can_hide`), so that when old elements are removed their memory can be reused for the new ones.
    pub fn children_can_hide(mut self, value: bool) -> Self {
        self.children_can_hide = if value { ChildrenCanHide::Yes } else { ChildrenCanHide::No };
        return self;
    }

    /// Set the translation part of the node's transform.
    pub const fn translate(mut self, x: f32, y: f32) -> Self {
        self.transform.offset = vec2(x, y);
        return self;
    }

    /// Apply a zoom centered at the center of the node's rect.
    pub const fn scale(mut self, scale: f32) -> Self {
        self.transform.scale = scale;
        return self;
    }

    /// Enable or disable the default click/hover animation.
    pub fn click_animation(mut self, value: bool) -> Self {
        self.interact.click_animation = value;
        return self;
    }

    /// Enable or disable the default keyboard focus indicator rect.
    ///
    /// Disable it to draw a custom focus effect using [`UiNode::is_keyboard_focused`].
    pub fn focus_indicator(mut self, value: bool) -> Self {
        self.interact.focus_indicator = value;
        return self;
    }

    /// Set whether the node can receive the keyboard-navigation focus at all (e.g. via Tab).
    pub fn focusable(mut self, value: bool) -> Self {
        self.interact.focusable = value;
        return self;
    }


    /// Mark a node as "constant".
    /// 
    /// When running in release mode, constant nodes will never be hashed to detect differences and never trigger relayouts or rerenders.
    /// 
    /// In debug mode, the node will still be hashed and diffed, and an error will be reported if it's found to not actually be constant. 
    /// 
    /// Can be used as an extreme micro-optimization for programs that have many nodes that don't depend on any dynamic data.
    pub const fn constant(mut self, value: bool) -> Self {
        self.constant = value;
        return self;
    }
}


#[derive(Copy, Clone, Hash, Debug)]
pub struct NodeText<'a>(pub &'a str);

impl<'a> NodeText<'a> {
    pub fn as_str(&self) -> &str {
        self.0
    }
}

/// Data for an image to be displayed
#[derive(Copy, Clone, Debug)]
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

impl<'a> Node<'a> {
    /// If the node has an editable text box, make it single-line.
    pub const fn single_line_text(mut self, value: bool) -> Self {
        self.text_options.single_line = value;
        return self;
    }

    /// Set whether the node's text is selectable by the user.
    pub const fn text_selectable(mut self, value: bool) -> Self {
        self.text_options.selectable = value;
        return self;
    }

    /// Set the node's position on both axes.
    pub const fn position(mut self, position_x: Pos, position_y: Pos) -> Self {
        self.layout.position.x = position_x;
        self.layout.position.y = position_y;
        return self;
    }

    /// Set the node's position to the same value on both axes.
    pub const fn position_symm(mut self, position: Pos) -> Self {
        self.layout.position.x = position;
        self.layout.position.y = position;
        return self;
    }

    /// Set the node's horizontal position.
    pub const fn position_x(mut self, position: Pos) -> Self {
        self.layout.position.x = position;
        return self;
    }

    /// Set the node's vertical position.
    pub const fn position_y(mut self, position: Pos) -> Self {
        self.layout.position.y = position;
        return self;
    }

    /// Set the anchor point on both axes.
    pub const fn anchor(mut self, anchor_x: Anchor, anchor_y: Anchor) -> Self {
        self.layout.anchor.x = anchor_x;
        self.layout.anchor.y = anchor_y;
        return self;
    }

    /// Set the same anchor on both axes.
    pub const fn anchor_symm(mut self, anchor: Anchor) -> Self {
        self.layout.anchor.x = anchor;
        self.layout.anchor.y = anchor;
        return self;
    }

    /// Set the horizontal anchor point.
    pub const fn anchor_x(mut self, anchor: Anchor) -> Self {
        self.layout.anchor.x = anchor;
        return self;
    }

    /// Set the vertical anchor point.
    pub const fn anchor_y(mut self, anchor: Anchor) -> Self {
        self.layout.anchor.y = anchor;
        return self;
    }

    /// Set the origin edges for this node's children's positions.
    pub const fn pos_origin(mut self, origin_x: HorizontalOrigin, origin_y: VerticalOrigin) -> Self {
        self.layout.pos_origin_x = origin_x;
        self.layout.pos_origin_y = origin_y;
        return self;
    }

    /// Set the horizontal origin edge for this node's children's positions.
    pub const fn pos_origin_x(mut self, origin: HorizontalOrigin) -> Self {
        self.layout.pos_origin_x = origin;
        return self;
    }

    /// Set the vertical origin edge for this node's children's positions.
    pub const fn pos_origin_y(mut self, origin: VerticalOrigin) -> Self {
        self.layout.pos_origin_y = origin;
        return self;
    }

    /// Set the node's [`Layout`].
    pub const fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        return self;
    }

    /// Set the node's size on both axes.
    pub const fn size(mut self, size_x: Size, size_y: Size) -> Self {
        self.layout.size.x = size_x;
        self.layout.size.y = size_y;
        return self;
    }

    /// Set the node's width.
    pub const fn size_x(mut self, size_x: Size) -> Self {
        self.layout.size.x = size_x;
        return self;
    }

    /// Set the node's height.
    pub const fn size_y(mut self, size_y: Size) -> Self {
        self.layout.size.y = size_y;
        return self;
    }

    /// Set the same size on both axes.
    pub const fn size_symm(mut self, size: Size) -> Self {
        self.layout.size.x = size;
        self.layout.size.y = size;
        return self;
    }

    /// Make the node visible.
    pub const fn visible(mut self) -> Self {
        self.visible = true;
        return self;
    }
    /// Make the node invisible.
    pub const fn invisible(mut self) -> Self {
        self.visible = false;
        return self;
    }

    /// Apply a blur effect to the node's shape with the given radius.
    pub const fn blur(mut self, radius: f32) -> Self {
        self.blur = Some(radius);
        return self;
    }

    /// Add a drop shadow.
    pub const fn shadow(mut self, shadow: Shadow) -> Self {
        self.shadow = Some(shadow);
        return self;
    }

    /// Add a second drop shadow.
    pub const fn second_shadow(mut self, shadow: Shadow) -> Self {
        self.second_shadow = Some(shadow);
        return self;
    }

    /// Add a stroke with the given width.
    pub const fn stroke(mut self, width: f32) -> Self {
        match &mut self.stroke {
            Some(stroke) => stroke.width = width,
            None => {
                self.stroke = Some(Stroke::new(width))
            },
        }
        return self;
    }

    /// Set the dash pattern for the stroke.
    pub const fn stroke_dashes(mut self, dash_length: f32, dash_offset: f32) -> Self {
        if let Some(stroke) = self.stroke {
            self.stroke = Some(stroke.with_dashes(dash_length, dash_offset));
        }
        return self;
    }

    /// Set the stroke color.
    pub const fn stroke_color(mut self, color: Color) -> Self {
        if let Some(stroke) = self.stroke {
            self.stroke = Some(stroke.with_color(color));
        }
        return self;
    }

    /// Set the stroke fill to a shared gradient.
    pub const fn stroke_fill(mut self, fill: ColorFill2) -> Self {
        if let Some(old_stroke) = self.stroke {
            self.stroke = Some(Stroke {
                color: fill,
                ..old_stroke
            });
        }
        return self;
    }

    /// Set the fill color.
    pub const fn color(mut self, color: Color) -> Self {
        self.color = ColorFill2::Color(color);
        return self;
    }

    /// Set the fill to a linear gradient relative to the node's bounds.
    pub const fn linear_gradient(mut self, gradient: LinearGradient) -> Self {
        self.color = ColorFill2::LinearGradient(gradient);
        return self;
    }

    /// Set the fill to use another node's linear gradient at its absolute position.
    pub const fn shared_gradient(mut self, key: NodeKey) -> Self {
        self.color = ColorFill2::SharedGradient(key);
        return self;
    }

    /// Set the fill to a [`ColorFill2`].
    pub const fn fill(mut self, fill: ColorFill2) -> Self {
        self.color = fill;
        return self;
    }

    /// Set the node's shape.
    pub const fn shape(mut self, shape: Shape) -> Self {
        self.shape = shape;
        return self;
    }

    /// Set the shape to a circle.
    pub const fn circle(mut self) -> Self {
        self.shape = Shape::Circle;
        return self;
    }

    /// Set children layout to a stack.
    pub const fn stack(mut self, axis: Axis, arrange: Arrange, spacing: f32) -> Self {
        self.children_layout = ChildrenLayout::Stack {
            arrange,
            axis,
            spacing,
        };
        return self;
    }

    /// Set the arrangement of children in a stack.
    pub const fn stack_arrange(mut self, arrange: Arrange) -> Self {
        let (axis, spacing) = match self.children_layout {
            ChildrenLayout::Stack { axis, spacing, .. } => (axis, spacing),
            _ => (Axis::Y, 8.0),
        };
        self.children_layout = ChildrenLayout::Stack { arrange, axis, spacing };
        return self;
    }

    /// Set the spacing between children in a stack.
    pub const fn stack_spacing(mut self, spacing: f32) -> Self {
        let (arrange, axis) = match self.children_layout {
            ChildrenLayout::Stack { arrange, axis, .. } => (arrange, axis),
            _ => (Arrange::Center, Axis::Y),
        };
        self.children_layout = ChildrenLayout::Stack { arrange, axis, spacing };
        return self;
    }

    /// Set the axis of a stack layout.
    pub const fn stack_axis(mut self, axis: Axis) -> Self {
        let (arrange, spacing) = match self.children_layout {
            ChildrenLayout::Stack { arrange, spacing, .. } => (arrange, spacing),
            _ => (Arrange::Center, 8.0),
        };
        self.children_layout = ChildrenLayout::Stack { arrange, axis, spacing };
        return self;
    }

    /// Set the number of columns in a grid layout.
    pub const fn grid_columns(mut self, count: u32) -> Self {
        let (spacing_x, spacing_y, flow) = match self.children_layout {
            ChildrenLayout::Grid { spacing_x, spacing_y, flow, .. } => (spacing_x, spacing_y, flow),
            _ => (8.0, 8.0, GridFlow::DEFAULT),
        };
        self.children_layout = ChildrenLayout::Grid { columns: MainAxisCellSize::Count(count), spacing_x, spacing_y, flow };
        return self;
    }

    /// Set the target cell width in a grid layout, letting the number of columns adjust automatically.
    pub const fn grid_column_width(mut self, width: f32) -> Self {
        let (spacing_x, spacing_y, flow) = match self.children_layout {
            ChildrenLayout::Grid { spacing_x, spacing_y, flow, .. } => (spacing_x, spacing_y, flow),
            _ => (8.0, 8.0, GridFlow::DEFAULT),
        };
        self.children_layout = ChildrenLayout::Grid { columns: MainAxisCellSize::Width(width), spacing_x, spacing_y, flow };
        return self;
    }

    /// Set the horizontal spacing between grid cells.
    pub const fn grid_spacing_x(mut self, spacing_x: f32) -> Self {
        let (columns, spacing_y, flow) = match self.children_layout {
            ChildrenLayout::Grid { columns, spacing_y, flow, .. } => (columns, spacing_y, flow),
            _ => (MainAxisCellSize::Count(3), 8.0, GridFlow::DEFAULT),
        };
        self.children_layout = ChildrenLayout::Grid { columns, spacing_x, spacing_y, flow };
        return self;
    }

    /// Set the vertical spacing between grid cells.
    pub const fn grid_spacing_y(mut self, spacing_y: f32) -> Self {
        let (columns, spacing_x, flow) = match self.children_layout {
            ChildrenLayout::Grid { columns, spacing_x, flow, .. } => (columns, spacing_x, flow),
            _ => (MainAxisCellSize::Count(3), 8.0, GridFlow::DEFAULT),
        };
        self.children_layout = ChildrenLayout::Grid { columns, spacing_x, spacing_y, flow };
        return self;
    }

    /// Set the flow direction of a grid layout.
    pub const fn grid_flow(mut self, flow: GridFlow) -> Self {
        let (columns, spacing_x, spacing_y) = match self.children_layout {
            ChildrenLayout::Grid { columns, spacing_x, spacing_y, .. } => (columns, spacing_x, spacing_y),
            _ => (MainAxisCellSize::Count(3), 8.0, 8.0),
        };
        self.children_layout = ChildrenLayout::Grid { columns, spacing_x, spacing_y, flow };
        return self;
    }

    /// Set symmetric padding on both axes.
    pub const fn padding(mut self, padding: f32) -> Self {
        self.layout.padding = Xy::new_symm(padding);
        return self;
    }

    /// Set horizontal padding.
    pub const fn padding_x(mut self, padding: f32) -> Self {
        self.layout.padding.x = padding;
        return self;
    }

    /// Set vertical padding.
    pub const fn padding_y(mut self, padding: f32) -> Self {
        self.layout.padding.y = padding;
        return self;
    }

    /// Enable or disable horizontal scrolling.
    pub const fn scrollable_x(mut self, scrollable_x: bool) -> Self {
        self.layout.scrollable.x = scrollable_x;
        return self;
    }

    /// Enable or disable vertical scrolling.
    pub const fn scrollable_y(mut self, scrollable_y: bool) -> Self {
        self.layout.scrollable.y = scrollable_y;
        return self;
    }

    /// Enable or disable automatic Markdown rendering.
    pub const fn auto_markdown(mut self, auto_markdown: bool) -> Self {
        self.text_options.auto_markdown = auto_markdown;
        return self;
    }

    /// Set the font size.
    pub const fn text_size(mut self, font_size: f32) -> Self {
        self.text_size = Some(font_size);
        return self;
    }

    /// Set the horizontal text alignment.
    pub const fn text_alignment(mut self, alignment: Alignment) -> Self {
        self.text_alignment = alignment;
        return self;
    }

    /// Set the vertical text alignment.
    pub const fn vertical_text_alignment(mut self, alignment: VerticalTextAlignment) -> Self {
        self.vertical_text_alignment = alignment;
        return self;
    }

    /// Set the text color.
    pub const fn text_color(mut self, color: Color) -> Self {
        self.text_color = Some(color);
        return self;
    }

    /// Set additional text style properties.
    pub const fn text_properties(mut self, properties: &'a [TextStyleProperty]) -> Self {
        self.text_properties = properties;
        return self;
    }

    /// Make the text bold.
    pub const fn bold(mut self) -> Self {
        self.text_style_flags = self.text_style_flags.union(TextStyleFlags::BOLD);
        return self;
    }

    /// Make the text italic.
    pub const fn italic(mut self) -> Self {
        self.text_style_flags = self.text_style_flags.union(TextStyleFlags::ITALIC);
        return self;
    }

    /// Use a monospace font.
    pub const fn monospace(mut self) -> Self {
        self.text_style_flags = self.text_style_flags.union(TextStyleFlags::MONOSPACE);
        return self;
    }

    /// Control whether this node consumes mouse events or is transparent to them.
    pub const fn absorbs_clicks(mut self, absorbs_clicks: bool) -> Self {
        self.interact.absorbs_mouse_events = absorbs_clicks;
        return self;
    }

    /// Enable or disable click sensing.
    pub const fn sense_click(mut self, value: bool) -> Self {
        let senses = &mut self.interact.senses;
        if value {
            *senses = senses.union(Sense::CLICK);
        } else {
            *senses = senses.intersection(Sense::CLICK.complement());
        }
        return self;
    }

    /// Enable or disable drag sensing.
    pub const fn sense_drag(mut self, value: bool) -> Self {
        let senses = &mut self.interact.senses;
        if value {
            *senses = senses.union(Sense::DRAG);
        } else {
            *senses = senses.intersection(Sense::DRAG.complement());
        }
        return self;
    }

    /// Make a node able to react to hovering, in the sense of every mouse movement that hovers over it.
    /// 
    /// Consider using [`Node::sense_hover_enter_or_exit()`] if the node only needs to react to the mouse entering or exiting it.
    pub const fn sense_hover(mut self, value: bool) -> Self {
        let senses = &mut self.interact.senses;
        if value {
            *senses = senses.union(Sense::HOVER);
        } else {
            *senses = senses.intersection(Sense::HOVER.complement());
        }
        return self;
    }

    /// Make a node able to react to hovering, in the sense of the mouse moving inside or outside of it.
    pub const fn sense_hover_enter_or_exit(mut self, value: bool) -> Self {
        let senses = &mut self.interact.senses;
        if value {
            *senses = senses.union(Sense::HOVER_ENTER_OR_EXIT);
        } else {
            *senses = senses.intersection(Sense::HOVER_ENTER_OR_EXIT.complement());
        }
        return self;
    }

    /// Enable or disable hold sensing.
    pub const fn sense_hold(mut self, value: bool) -> Self {
        let senses = &mut self.interact.senses;
        if value {
            *senses = senses.union(Sense::HOLD);
        } else {
            *senses = senses.intersection(Sense::HOLD.complement());
        }
        return self;
    }

    /// Enable or disable scroll sensing.
    pub const fn sense_scroll(mut self, value: bool) -> Self {
        let senses = &mut self.interact.senses;
        if value {
            *senses = senses.union(Sense::SCROLL);
        } else {
            *senses = senses.intersection(Sense::SCROLL.complement());
        }
        return self;
    }

    /// Enable or disable drag-and-drop target sensing.
    pub const fn sense_drag_drop_target(mut self, value: bool) -> Self {
        let senses = &mut self.interact.senses;
        if value {
            *senses = senses.union(Sense::DRAG_DROP_TARGET);
        } else {
            *senses = senses.intersection(Sense::DRAG_DROP_TARGET.complement());
        }
        return self;
    }

    /// Enable or disable time-based sensing (node receives updates every frame).
    pub const fn sense_time(mut self, value: bool) -> Self {
        let senses = &mut self.interact.senses;
        if value {
            *senses = senses.union(Sense::TIME);
        } else {
            *senses = senses.intersection(Sense::TIME.complement());
        }
        return self;
    }

    /// Add a [`NodeKey`] to the [`Node`].
    /// 
    pub fn key(mut self, key: NodeKey) -> Self {
        self.key = Some(key);
        return self;
    }

    /// Set the full animation config.
    pub const fn animation(mut self, animation: Animation) -> Self {
        self.animation = animation;
        return self;
    }

    /// Set the animation speed multiplier.
    pub const fn animation_speed(mut self, speed: f32) -> Self {
        self.animation.speed = speed;
        return self;
    }

    // Enter animation methods
    /// Set the enter slide animation.
    pub const fn enter_slide(mut self, edge: SlideEdge, direction: SlideDirection) -> Self {
        self.animation.enter = EnterAnimation::Slide { edge, direction };
        return self;
    }

    /// Set a grow-in enter animation.
    pub const fn enter_grow(mut self, axis: Axis, origin: Pos) -> Self {
        self.animation.enter = EnterAnimation::GrowShrink { axis, origin };
        return self;
    }

    // Exit animation methods
    /// Set the exit slide animation.
    pub const fn exit_slide(mut self, edge: SlideEdge, direction: SlideDirection) -> Self {
        self.animation.exit = ExitAnimation::Slide { edge, direction };
        return self;
    }

    /// Set a shrink-out exit animation.
    pub const fn exit_shrink(mut self, axis: Axis, origin: Pos) -> Self {
        self.animation.exit = ExitAnimation::GrowShrink { axis, origin };
        return self;
    }

    /// Slide in from and out to the top edge.
    pub const fn slide_from_top(mut self) -> Self {
        self.animation.enter = EnterAnimation::Slide { edge: SlideEdge::Top, direction: SlideDirection::In };
        self.animation.exit = ExitAnimation::Slide { edge: SlideEdge::Top, direction: SlideDirection::Out };
        return self;
    }

    /// Slide in from and out to the bottom edge.
    pub const fn slide_from_bottom(mut self) -> Self {
        self.animation.enter = EnterAnimation::Slide { edge: SlideEdge::Bottom, direction: SlideDirection::In };
        self.animation.exit = ExitAnimation::Slide { edge: SlideEdge::Bottom, direction: SlideDirection::Out };
        return self;
    }

    /// Slide in from and out to the left edge.
    pub const fn slide_from_left(mut self) -> Self {
        self.animation.enter = EnterAnimation::Slide { edge: SlideEdge::Left, direction: SlideDirection::In };
        self.animation.exit = ExitAnimation::Slide { edge: SlideEdge::Left, direction: SlideDirection::Out };
        return self;
    }

    /// Slide in from and out to the right edge.
    pub const fn slide_from_right(mut self) -> Self {
        self.animation.enter = EnterAnimation::Slide { edge: SlideEdge::Right, direction: SlideDirection::In };
        self.animation.exit = ExitAnimation::Slide { edge: SlideEdge::Right, direction: SlideDirection::Out };
        return self;
    }

    /// Grow in and shrink out along an axis from an origin.
    pub const fn grow_shrink(mut self, axis: Axis, origin: Pos) -> Self {
        self.animation.enter = EnterAnimation::GrowShrink { axis, origin };
        self.animation.exit = ExitAnimation::GrowShrink { axis, origin };
        return self;
    }

    pub const fn grow_from_top(mut self) -> Self {
        self.animation.enter = EnterAnimation::GrowShrink { axis: Axis::Y, origin: Pos::Start };
        return self;
    }
    pub const fn grow_from_bottom(mut self) -> Self {
        self.animation.enter = EnterAnimation::GrowShrink { axis: Axis::Y, origin: Pos::End };
        return self;
    }
    pub const fn grow_from_left(mut self) -> Self {
        self.animation.enter = EnterAnimation::GrowShrink { axis: Axis::X, origin: Pos::Start };
        return self;
    }
    pub const fn grow_from_center_along_x(mut self) -> Self {
        self.animation.enter = EnterAnimation::GrowShrink { axis: Axis::X, origin: Pos::Center };
        return self;
    }
    pub const fn grow_from_center_along_y(mut self) -> Self {
        self.animation.enter = EnterAnimation::GrowShrink { axis: Axis::Y, origin: Pos::Center };
        return self;
    }
    pub const fn grow_from_right(mut self) -> Self {
        self.animation.enter = EnterAnimation::GrowShrink { axis: Axis::X, origin: Pos::End };
        return self;
    }

    pub const fn shrink_to_top(mut self) -> Self {
        self.animation.exit = ExitAnimation::GrowShrink { axis: Axis::Y, origin: Pos::Start };
        return self;
    }
    pub const fn shrink_to_bottom(mut self) -> Self {
        self.animation.exit = ExitAnimation::GrowShrink { axis: Axis::Y, origin: Pos::End };
        return self;
    }
    pub const fn shrink_to_left(mut self) -> Self {
        self.animation.exit = ExitAnimation::GrowShrink { axis: Axis::X, origin: Pos::Start };
        return self;
    }
    pub const fn shrink_to_right(mut self) -> Self {
        self.animation.exit = ExitAnimation::GrowShrink { axis: Axis::X, origin: Pos::End };
        return self;
    }
    pub const fn shrink_to_center_along_x(mut self) -> Self {
        self.animation.exit = ExitAnimation::GrowShrink { axis: Axis::X, origin: Pos::Center };
        return self;
    }
    pub const fn shrink_to_center_along_y(mut self) -> Self {
        self.animation.exit = ExitAnimation::GrowShrink { axis: Axis::Y, origin: Pos::Center };
        return self;
    }

    /// Animate position changes when this node moves.
    pub const fn animate_position(mut self, value: bool) -> Self {
        self.animation.state_transition.animate_position = value;
        return self;
    }

    pub fn is_fit_content(&self) -> bool {
        let Xy { x, y } = self.layout.size;
        return x == Size::FitContent || y == Size::FitContent
    }

    pub const fn is_scrollable(&self) -> bool {
        return self.layout.scrollable.x || self.layout.scrollable.y
    }

    /// Inherit the `children_can_hide` setting from the parent.
    pub fn children_can_hide_inherit(mut self) -> Self {
        self.children_can_hide = ChildrenCanHide::Inherit;
        return self;
    }

    /// Clip children to the node's rectangle on both axes.
    pub const fn clip_children(mut self, value: bool) -> Self {
        self.clip_children.x = value;
        self.clip_children.y = value;
        return self;
    }

    /// Clip children to the node's horizontal bounds.
    pub const fn clip_children_x(mut self, value: bool) -> Self {
        self.clip_children.x = value;
        return self;
    }

    /// Clip children to the node's vertical bounds.
    pub const fn clip_children_y(mut self, value: bool) -> Self {
        self.clip_children.y = value;
        return self;
    }

    /// Enable custom rendering for this node.
    pub const fn custom_render(mut self, value: bool) -> Self {
        self.custom_render = value;
        return self;
    }

    /// Ignore parent stack/grid layout and place freely within the parent.
    pub const fn free_placement(mut self, value: bool) -> Self {
        self.free_placement = value;
        return self;
    }

    /// Prevent this node from being shifted by the parent's scroll offset.
    pub const fn ignore_parent_scroll(mut self, value: bool) -> Self {
        self.ignore_parent_scroll = value;
        return self;
    }

    /// Set placeholder text for a text edit that will be shown when the text edit is empty.
    /// This only works with editable text nodes.
    pub fn placeholder_text(mut self, placeholder: &'a str) -> Node<'a> {
        self.placeholder_text = Some(NodeText(placeholder));
        return self;
    }

    /// Add text to the [`Node`] from a `&'static str`.
    ///
    /// Uses pointer equality to determine if the text needs updating.
    pub const fn static_text(mut self, text: &'static str) -> Node<'a> {
        self.text_options.use_pointer_comparison = true;
        self.text = Some(NodeText(text));
        return self;
    }

    /// Set a raster image from static bytes.
    pub fn static_image(mut self, image: &'static [u8]) -> Node<'a> {
        self.image = Some(Image::RasterStatic(image));
        return self;
    }

    /// Set a raster image from a filesystem path.
    pub fn image_path(mut self, path: &'a str) -> Node<'a> {
        self.image = Some(Image::RasterPath(path));
        return self;
    }

    /// Set an SVG image from static bytes.
    pub fn static_svg(mut self, svg: &'static [u8]) -> Node<'a> {
        self.image = Some(Image::SvgStatic(svg));
        return self;
    }

    /// Set an SVG image from a filesystem path.
    pub fn svg_path(mut self, path: &'a str) -> Node<'a> {
        self.image = Some(Image::SvgPath(path));
        return self;
    }

    /// Add text to the [`Node`].
    pub fn text(mut self, text: &'a str) -> Node<'a> {
        self.text = Some(NodeText(text));
        return self;
    }

    /// Set image options, such as tiling mode or 9-slice borders.
    pub const fn image_options(mut self, options: ImageOptions) -> Node<'a> {
        self.image_options = options;
        return self;
    }
}

impl Node<'_> {
    #[track_caller]
    pub(crate) fn key_or_anon_key(&self) -> NodeKey {
        return match self.key {
            Some(key) => key,
            None => NodeKey::new(Id(caller_location_id()), "Anon node"),
        };
    }
}

type MarkdownStyleRange = (TextStyleProperty, Range<usize>);

fn apply_markdown<'a>(text: &str, arena: &'a bumpalo::Bump) -> (BumpString<'a>, BumpVec<'a, MarkdownStyleRange>) {
    use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

    let mut string = BumpString::with_capacity_in(text.len(), arena);
    let mut style_ranges = BumpVec::with_capacity_in(8, arena);

    let mut em_start: Option<usize> = None;
    let mut strong_start: Option<usize> = None;

    for event in Parser::new_ext(text, Options::empty()) {
        match event {
            Event::Text(t) => string.push_str(&t),
            Event::Code(t) => {
                let start = string.len();
                string.push_str(&t);
                let end = string.len();
                let grey = StyleProperty::Brush(ColorBrush([150, 150, 150, 255]));
                style_ranges.push((MONOSPACE, start..end));
                style_ranges.push((grey, start..end));
            }
            Event::Start(Tag::Emphasis) => em_start = Some(string.len()),
            Event::End(TagEnd::Emphasis) => {
                if let Some(start) = em_start.take() {
                    style_ranges.push((ITALIC, start..string.len()));
                }
            }
            Event::Start(Tag::Strong) => strong_start = Some(string.len()),
            Event::End(TagEnd::Strong) => {
                if let Some(start) = strong_start.take() {
                    style_ranges.push((BOLD, start..string.len()));
                }
            }
            Event::SoftBreak => string.push(' '),
            Event::HardBreak => string.push('\n'),
            Event::End(TagEnd::Paragraph) => string.push_str("\n\n"),
            _ => {}
        }
    }

    let trimmed_len = string.trim_end_matches('\n').len();
    string.truncate(trimmed_len);

    return (string, style_ranges);
}

impl Ui {
    pub(crate) fn set_params_text(&mut self, i: NodeI, node: &Node) {
        with_arena(|arena| {

            let text_options = node.text_options;

            if node.text.is_none() && !text_options.editable {
                return;
            }
            let raw_text = node.text.unwrap_or(NodeText(""));

            let new_fingerprint = TextFingerprint::new(raw_text.as_str(), text_options.use_pointer_comparison);

            let needs_new_widget = match (&self.sys.nodes[i].text_i, text_options.editable) {
                (None, _) => true,
                (Some(TextI::TextEdit(_)), true) => false,
                (Some(TextI::TextBox(_)), false) => false,
                _ => true, // need to switch
            };

            let content_needs_update = needs_new_widget || self.sys.nodes[i].text_fingerprint != new_fingerprint;

            if content_needs_update {
                self.sys.nodes[i].text_fingerprint = new_fingerprint;

                // Run markdown transform only when the content actually changed.
                // The fingerprint is based on the pre-transform text, so this is skipped too.
                let run_markdown = !text_options.editable && text_options.auto_markdown;

                let (markdown_string, mut style_ranges) = if run_markdown {
                    apply_markdown(raw_text.as_str(), arena)
                } else {
                    (BumpString::new_in(arena), BumpVec::new_in(arena))
                };
                let display_text: &str = if run_markdown { &markdown_string } else { raw_text.as_str() };

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
                    // Tag the text widget with the id of the node it belongs to,
                    // so we can map back from a text box/edit to its node.
                    let node_tag = Some(self.sys.nodes[i].id.0);
                    // Create new widget
                    let new_text_i = if text_options.editable {
                        let handle = self.sys.renderer.text.add_text_edit(display_text.to_string(), None, (500.0, 500.0), z);
                        self.sys.renderer.text.get_text_edit_mut(&handle).set_custom_tag(node_tag);
                        TextI::TextEdit(handle)
                    } else {
                        let handle = self.sys.renderer.text.add_text_box(display_text.to_string(), None, (500.0, 500.0), z);
                        self.sys.renderer.text.get_text_box_mut(&handle).set_custom_tag(node_tag);
                        for (prop, range) in style_ranges.drain(..) {
                            self.sys.renderer.text.get_text_box_mut(&handle).push_ranged_style_property(prop, range);
                        }
                        TextI::TextBox(handle)
                    };

                    self.sys.nodes[i].text_i = Some(new_text_i);
                } else {
                    match &self.sys.nodes[i].text_i {
                        Some(TextI::TextEdit(_)) => {
                            // do nothing, content in a text edit box is not reset declaratively every frame, obviously.
                        },
                        Some(TextI::TextBox(handle)) => {
                            let text_box = self.sys.renderer.text.get_text_box_mut(&handle);

                            text_box.set_text(display_text);

                            if ! style_ranges.is_empty() {
                                self.sys.renderer.text.get_text_box_mut(&handle).clear_ranged_style_properties();
                                for (prop, range) in style_ranges.drain(..) {
                                    self.sys.renderer.text.get_text_box_mut(&handle).push_ranged_style_property(prop, range);
                                }
                            }
                        },
                        None => unreachable!("Should have created a new widget above"),
                    }
                }

            }

            if let Some(text_i) = &self.sys.nodes[i].text_i {

                let mut properties_opt: Option<BumpVec<StyleProperty<_>>> = None;
                let flags = node.text_style_flags;
                let has_any_style = !flags.is_empty() || node.text_size.is_some() || node.text_color.is_some() || !node.text_properties.is_empty();
                if has_any_style {
                    let flag_count = flags.bits().count_ones() as usize;
                    let mut properties = BumpVec::with_capacity_in(node.text_properties.len() + flag_count + 2, arena);
                    if flags.contains(TextStyleFlags::BOLD) { properties.push(BOLD); }
                    if flags.contains(TextStyleFlags::ITALIC) { properties.push(ITALIC); }
                    if flags.contains(TextStyleFlags::MONOSPACE) { properties.push(MONOSPACE); }
                    properties.extend_from_slice(node.text_properties);
                    if let Some(font_size) = node.text_size {
                        properties.push(TextStyleProperty::FontSize(font_size));
                    }
                    if let Some(color) = node.text_color {
                        properties.push(TextStyleProperty::Brush(keru_draw::ColorBrush(color.to_u8_array())));
                    }
                    properties_opt = Some(properties);
                }

                match text_i {
                    TextI::TextEdit(handle) => {
                        let text_edit = self.sys.renderer.text.get_text_edit_mut(handle);
                        text_edit.set_disabled(text_options.edit_disabled);
                        text_edit.set_single_line(text_options.single_line);
                        if let Some(placeholder) = node.placeholder_text {
                            text_edit.set_placeholder_hashed(placeholder.as_str());
                        }

                        if let Some(properties) = properties_opt {
                            text_edit.set_style_property_overrides(&properties);
                        }
                        text_edit.set_alignment(node.text_alignment);

                    },
                    TextI::TextBox(handle) => {
                        let text_box = self.sys.renderer.text.get_text_box_mut(handle);
                        if let Some(properties) = properties_opt {
                            text_box.set_style_property_overrides(&properties);
                        }
                        text_box.set_alignment(node.text_alignment);
                        text_box.set_selectable(node.text_options.selectable);
                    },
                }
            }

            // Link this text box into the global cross-box selection chain.
            // Relink every frame so that links are always updated.
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

        });

    }


    pub(crate) fn set_params(&mut self, i: NodeI, node: &Node) {
        #[cfg(not(debug_assertions))]
        if reactive::is_in_skipped_reactive_block() {
            return;
        }
        
        // What if we did all the hashing and diffing in a single sequential scan of the stored Nodes?
        // (we'd store them in a separate Soa vec rather than inline in the InnerNode in that case.)
        // The linear scan vs random order would probably be good, but on the other hand, the pre-copy Node is already hot in the cache either way, because we have to copy it.

        #[cfg(not(debug_assertions))]
        if node.constant && self.sys.nodes[i].frame_added != self.sys.current_frame {
            return;
        }

        if let Some(image_data) = node.image {
            match image_data {
                Image::RasterStatic(image) => self.set_static_image(i, image),
                Image::RasterPath(path) => self.set_path_image(i, path),
                Image::SvgStatic(svg) => self.set_static_svg(i, svg),
                Image::SvgPath(path) => self.set_path_svg(i, path),
            };
        }

        let new_cosmetic_hash = node.cosmetic_hash();
        let new_layout_hash = node.layout_hash();
        
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

        #[cfg(debug_assertions)]
        if node.constant && self.sys.nodes[i].frame_added != self.sys.current_frame {
            if cosmetic_changed || layout_changed {
                let kind = match (layout_changed, cosmetic_changed) {
                    (true, true) => "layout and appearance",
                    (true, false) => "layout",
                    (false, true) => "appearance",
                    _ => unreachable!()
                };
                log::error!("Keru: node \"{}\" is marked constant but its {kind} params changed", self.node_debug_name(i));
            }
            return;
        }
        
        self.sys.nodes[i].params = node.remove_borrowed_data_and_copy();

        self.sys.nodes[i].last_cosmetic_hash = new_cosmetic_hash;
        self.sys.nodes[i].last_layout_hash = new_layout_hash;

        if layout_changed {
            self.sys.push_partial_relayout(i);
        }
        if cosmetic_changed{
            self.sys.changes.rebuild_render_data = true;
        }
    }
}


impl<'a> Node<'a> {
    fn remove_borrowed_data_and_copy(self) -> Node<'static> {
        let staticized: Node<'static> = Node {
            key: self.key,
            text_options: self.text_options,
            children_layout: self.children_layout,
            shape: self.shape,
            blur: self.blur,
            shadow: self.shadow,
            second_shadow: self.second_shadow,
            stroke: self.stroke,
            color: self.color,
            visible: self.visible,
            interact: self.interact,
            layout: self.layout,
            children_can_hide: self.children_can_hide,
            clip_children: self.clip_children,
            animation: self.animation,
            transform: self.transform,
            custom_render: self.custom_render,
            z_index: self.z_index,
            grid_element: self.grid_element,
            free_placement: self.free_placement,
            ignore_parent_scroll: self.ignore_parent_scroll,
            text_size: self.text_size,
            text_color: self.text_color,
            text_alignment: self.text_alignment,
            vertical_text_alignment: self.vertical_text_alignment,

            text: None,
            placeholder_text: None,
            image: None,
            image_options: self.image_options,
            text_properties: &[],
            text_style_flags: TextStyleFlags::empty(),
            constant: self.constant,
        };
        return staticized;
    }
}