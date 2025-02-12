use crate::*;
use crate::color::*;
use std::hash::{Hash, Hasher};
use rustc_hash::FxHasher;

#[derive(Debug, Copy, Clone)]
/// A lightweight struct describing the params of a GUI node.
/// 
/// You can start with one of the preset constants ([`BUTTON`], [`LABEL`], [`TEXT`], ...), then use the builder methods to customize it:
/// 
/// ```rust
/// # use keru::*;
/// const MY_BUTTON: NodeParams = keru::BUTTON
///     .color(Color::RED)
///     .shape(Shape::Circle); 
/// ```
/// After adding a node to the [`Ui`] with [`Ui::add`], you can call the [`UiNode::params`] method on it to set its params.
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
/// # const MY_BUTTON: NodeParams = keru::BUTTON
/// #     .color(Color::RED)
/// #     .shape(Shape::Circle); 
/// ui.add(INCREASE).params(MY_BUTTON);
/// #
/// #   }
/// # }
/// ```
/// You can also call [`UiNode`]'s other builder methods to change the node's params it *after* you add it:
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
/// # const MY_BUTTON: NodeParams = keru::BUTTON
/// #     .color(Color::RED)
/// #     .shape(Shape::Circle); 
/// ui.add(INCREASE).params(MY_BUTTON).size_x(Size::Fill);
/// #
/// #   }
/// # }
/// ```
/// To see it show up, use [`UiNode::place`] or [`Ui::place`] to place it in the ui tree.
pub struct NodeParams {
    pub key: Option<NodeKey>,
    pub text_params: Option<TextOptions>,
    pub stack: Option<Stack>,
    pub rect: Rect,
    pub interact: Interact,
    pub layout: Layout,
}

/// A node's size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Size {
    // todo: use two variants instead of Len?
    Fixed(Len),
    Fill,
    FitContent,
    FitContentOrMinimum(Len),
    AspectRatio(f32),
}

// Get a load of this dogshit that I have to write
impl Hash for Size {
    fn hash<H: Hasher>(&self, state: &mut H) {
        use Size::*;
        match self {
            Fixed(len) => (0u8, len).hash(state),
            Fill => 1u8.hash(state),
            FitContent => 2u8.hash(state),
            FitContentOrMinimum(len) => (3u8, len).hash(state),
            AspectRatio(ratio) => (4u8, ratio.to_bits()).hash(state),
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
    pub spacing: Len,
}
impl Stack {
    pub const DEFAULT: Stack = Stack {
        arrange: Arrange::Center,
        axis: Axis::Y,
        spacing: Len::Pixels(5),
    };
    pub const fn arrange(mut self, arrange: Arrange) -> Self {
        self.arrange = arrange;
        return self;
    }
    pub const fn spacing(mut self, spacing: Len) -> Self {
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
}

/// The node's layout, size and position.
#[derive(Debug, Copy, Clone, PartialEq, Hash)]
pub struct Layout {
    pub size: Xy<Size>,
    pub padding: Xy<Len>,
    pub position: Xy<Position>,
    pub scrollable: Xy<bool>,
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
    }
}

impl Hash for Shape {
    fn hash<H: Hasher>(&self, state: &mut H) {
        use Shape::*;
        match self {
            Rectangle { corner_radius } => {
                0u8.hash(state); // Unique tag for Rectangle
                corner_radius.to_bits().hash(state); // Convert f32 to bits for hashing
            }
            Circle => {
                1u8.hash(state); // Unique tag for Circle
            }
            Ring { width } => {
                2u8.hash(state); // Unique tag for Ring
                width.to_bits().hash(state); // Convert f32 to bits for hashing
            }
        }
    }
}

/// The node's visual appearance.
#[derive(Debug, Copy, Clone, PartialEq, Hash)]
pub struct Rect {
    pub shape: Shape,
    pub visible: bool,
    pub outline_only: bool,
    pub vertex_colors: VertexColors,
    // ... crazy stuff like texture and NinePatchRect
}
impl Rect {
    pub const DEFAULT: Self = Self {
        shape: Shape::Rectangle { corner_radius: BASE_RADIUS }, 
        visible: true,
        outline_only: true,
        vertex_colors: VertexColors::flat(Color::KERU_BLUE),
    };
}

// rename
// todo: add greyed text for textinput
/// Options for text nodes.
#[derive(Debug, Default, Copy, Clone, Hash)]
pub struct TextOptions {
    pub editable: bool,
    pub single_line: bool,
}

pub(crate) const BASE_RADIUS: f32 = 20.0;

impl NodeParams {
    pub(crate) fn cosmetic_update_hash(&self) -> u64 {
        let mut hasher = FxHasher::default();
        self.rect.hash(&mut hasher);
        return hasher.finish();
    }

    pub(crate) fn partial_relayout_hash(&self) -> u64 {
        let mut hasher = FxHasher::default();
        self.layout.hash(&mut hasher);
        self.stack.hash(&mut hasher);
        self.text_params.hash(&mut hasher);
        return hasher.finish();
    }

    pub const fn const_default() -> Self {
        return DEFAULT;
    }

    // todo: in a future version of Rust that allows it, change these to take a generic Into<Size>
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
        self.rect.outline_only = false;
        self.rect.vertex_colors = VertexColors::flat(Color::KERU_DEBUG_RED);
        return self;
    }

    pub const fn filled(mut self, filled: bool) -> Self {
        self.rect.outline_only = filled;
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

    pub const fn vertex_colors(mut self, colors: VertexColors) -> Self {
        self.rect.vertex_colors = colors;
        return self;
    }

    pub const fn stack(mut self, axis: Axis, arrange: Arrange, spacing: Len) -> Self {
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

    pub const fn stack_spacing(mut self, spacing: Len) -> Self {
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

    pub const fn padding(mut self, padding: Len) -> Self {
        self.layout.padding = Xy::new_symm(padding);
        return self;
    }

    pub const fn padding_x(mut self, padding: Len) -> Self {
        self.layout.padding.x = padding;
        return self;
    }

    pub const fn padding_y(mut self, padding: Len) -> Self {
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

    pub fn is_fit_content(&self) -> bool {
        let Xy { x, y } = self.layout.size;
        return x == Size::FitContent || y == Size::FitContent
    }

    pub fn is_scrollable(&self) -> bool {
        return self.layout.scrollable.x || self.layout.scrollable.y
    }
}

pub struct FullNodeParams<'a> {
    pub params: NodeParams,
    pub text: Option<&'a str>,
    pub image: Option<&'static [u8]>,
}

impl<'a> NodeParamsTrait for FullNodeParams<'a> {
    fn get_params(&self) -> &NodeParams {
        return &self.params;
    }

    fn get_text(&self) -> Option<&str> {
        return self.text;
    }

    fn get_image(&self) -> Option<&'static [u8]> {
        return self.image;
    }
}

impl<'a> FullNodeParams<'a> {
    // todo: in a future version of Rust that allows it, change these to take a generic Into<Size>
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
        self.params.rect.outline_only = false;
        self.params.rect.vertex_colors = VertexColors::flat(Color::KERU_DEBUG_RED);
        return self;
    }

    pub const fn filled(mut self, filled: bool) -> Self {
        self.params.rect.outline_only = filled;
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

    pub const fn stack(mut self, axis: Axis, arrange: Arrange, spacing: Len) -> Self {
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

    pub const fn stack_spacing(mut self, spacing: Len) -> Self {
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

    pub const fn padding(mut self, padding: Len) -> Self {
        self.params.layout.padding = Xy::new_symm(padding);
        return self;
    }

    pub const fn padding_x(mut self, padding: Len) -> Self {
        self.params.layout.padding.x = padding;
        return self;
    }

    pub const fn padding_y(mut self, padding: Len) -> Self {
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

    pub fn key(mut self, key: NodeKey) -> Self {
        self.params.key = Some(key);
        return self;
    }

    pub fn is_fit_content(&self) -> bool {
        let Xy { x, y } = self.params.layout.size;
        return x == Size::FitContent || y == Size::FitContent
    }

    pub fn is_scrollable(&self) -> bool {
        return self.params.layout.scrollable.x || self.params.layout.scrollable.y
    }
}

// todo: static text
impl NodeParams {
    pub fn text<'a>(self, text: &'a str) -> FullNodeParams<'a> {
        return FullNodeParams {
            params: self,
            text: Some(text),
            image: None,
        }
    }

    pub fn static_image(self, image: &'static [u8]) -> FullNodeParams<'static> {
        return FullNodeParams {
            params: self,
            text: None,
            image: Some(image),
        }
    }
}
