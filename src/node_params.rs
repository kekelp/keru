use crate::*;
use crate::color::*;
use std::{fmt::Display, hash::{Hash, Hasher}, ops::Deref};
use rustc_hash::FxHasher;

/// A lightweight struct describing the params of a Ui node.
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
/// # const MY_BUTTON: NodeParams = keru::BUTTON
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
/// const MY_BUTTON: NodeParams = keru::BUTTON
///     .color(Color::RED)
///     .shape(Shape::Circle); 
/// ```
/// 
/// [`NodeParams`] is a lightweight plain-old-data struct. Methods like [`Self::text()`] allow to associate borrowed data like a `&str` to a [`NodeParams`].
/// 
/// The result is a [`FullNodeParams`], a version of this struct that can hold borrowed data. Both versions can be used in the same ways.
#[derive(Debug, Copy, Clone)]
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
    Pixels(u32),
    Frac(f32),
    Fill, // todo, same as Frac(1), remove?
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
}

/// The node's layout, size and position.
#[derive(Debug, Copy, Clone, PartialEq, Hash)]
pub struct Layout {
    pub size: Xy<Size>,
    pub padding: Xy<u32>,
    pub position: Xy<Position>,
    pub scrollable: Xy<bool>,
}

bitflags::bitflags! {
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
    pub rounded_corners: RoundedCorners,
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
        rounded_corners: RoundedCorners::ALL,
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

pub(crate) const BASE_RADIUS: f32 = 15.0;

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

    pub const fn padding(mut self, padding: u32) -> Self {
        self.layout.padding = Xy::new_symm(padding);
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

    pub fn is_fit_content(&self) -> bool {
        let Xy { x, y } = self.layout.size;
        return x == Size::FitContent || y == Size::FitContent
    }

    pub fn is_scrollable(&self) -> bool {
        return self.layout.scrollable.x || self.layout.scrollable.y
    }

    pub const fn corners(mut self, corners: RoundedCorners) -> Self {
        self.rect.rounded_corners = corners;
        return self;
    }
}

/// A version of [`NodeParams`] that can hold borrowed data.
/// 
/// Can be used in the same way as [`NodeParams`].
pub struct FullNodeParams<'a> {
    pub params: NodeParams,
    pub text: Option<&'a str>,
    pub text_changed: Changed,
    pub text_ptr: usize,
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

    /// Associate a [`NodeKey`] to the [`NodeParams`].
    /// 
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

    pub const fn corners(mut self, corners: RoundedCorners) -> Self {
        self.params.rect.rounded_corners = corners;
        return self;
    }
}

// todo: static text
impl NodeParams {
    pub fn text<'a>(self, text: &'a str) -> FullNodeParams<'a> {
        return FullNodeParams {
            params: self,
            text: Some(text),
            image: None,
            text_changed: Changed::NeedsHash,
            text_ptr: (&raw const text) as usize,
        }
    }

    pub fn static_text(self, text: &'static str) -> FullNodeParams<'static> {
        return FullNodeParams {
            params: self,
            text: Some(text),
            image: None,
            text_changed: Changed::Static,
            text_ptr: (&raw const text) as usize,
        }
    }

    // pub fn smart_text<'a>(self, text: &'a Observer<impl AsRef<str>>) -> FullNodeParams<'a> {
    //     return FullNodeParams {
    //         params: self,
    //         text: Some(text.as_ref()),
    //         text_changed: text.changed_at(),
    //         text_ptr: (&raw const text) as usize,
    //         image: None,
    //     }
    // }

    pub fn static_image(self, image: &'static [u8]) -> FullNodeParams<'static> {
        return FullNodeParams {
            params: self,
            text: None,
            image: Some(image),
            text_changed: Changed::Static,
            text_ptr: 0,
        }
    }
}


pub enum Changed {
    Static,
    ChangedAt(u64),
    NeedsHash,
}


pub struct FullNodeParams2<'a, T: Display + ?Sized> {
    pub params: NodeParams,
    pub text: Option<&'a T>,
    pub text_changed: Changed,
    pub text_ptr: usize,
    pub image: Option<&'static [u8]>,
}

impl NodeParams {
    pub fn text2<'a, T: Display + ?Sized>(self, text: &'a T) -> FullNodeParams2<'a, T> {
        return FullNodeParams2 {
            params: self,
            text: Some(&text),
            image: None,
            text_changed: Changed::NeedsHash,
            text_ptr: (&raw const text) as usize,
        }
    }

    pub fn smart_text2<'a, T: Display>(self, text: &'a Observer<T>) -> FullNodeParams2<'a, T> {
        return FullNodeParams2 {
            params: self,
            text: Some(&text),
            image: None,
            text_changed: text.changed_at(),
            text_ptr: (&raw const text) as usize,
        }
    }
}

impl Ui {
    #[track_caller]
    pub fn add2<T: Display + ?Sized>(&mut self, params: FullNodeParams2<T>) -> UiParent {
        let key = match params.params.key {
            Some(key) => key,
            None => NodeKey::new(Id(caller_location_hash()), ""),
        };
        
        let i = self.add_or_update_node(key);
        self.get_uinode(i).set_params2(params);
        return self.make_parent_from_i(i);
    }
}

impl<'a> UiNode<'a> {
    pub(crate) fn set_params2<T: Display + ?Sized>(&mut self, params: FullNodeParams2<T>) -> &mut Self {
        self.node_mut().params = params.params;
        if let Some(text) = params.text {
            let text_changed = match params.text_changed {
                Changed::Static => false,
                Changed::ChangedAt(frame) => frame > self.ui.sys.second_last_frame_end_fake_time,
                Changed::NeedsHash => true,
            };

            let did_we_even_get_the_same_text_variable = params.text_ptr == self.node().last_text_ptr;

            let can_we_skip_it = did_we_even_get_the_same_text_variable && (text_changed == false);

            if can_we_skip_it == false {
                log::warn!("Actually writing");
                // todo: skip the hashing inside text() when we're sure it changed
                self.text(text);
                self.node_mut().last_text_ptr = params.text_ptr;
            } else {
                log::warn!("Skipping unchanged display value");
            }

        }
        
        if let Some(image) = params.image {
            self.static_image(image);
        }

        self.ui.check_param_changes(self.node_i);

        return self;
    }
}

// to avoid conflicting impls, MaybeObserver<T> is implemented for all T: Display, not all T in general.
// In this way, Observer<T> doesn't implement Display on its own, and it works.
// I might be wrong about this. Pretty good chance actually.
// However, if that is true, this name is kind of a lie. But it ends up being pretty ok, as long as this is only exposed for Display types anyway.
pub trait MaybeObserver<T> {
    fn value(&self) -> &T;
    fn changed_at(&self) -> Changed;
}

impl NodeParams {
    pub fn just_unbelievably_smart_text2<'a, T: Display>(
        self,
        text: &'a impl MaybeObserver<T>,
    ) -> FullNodeParams2<'a, T> {
        return FullNodeParams2 {
            params: self,
            text: Some(&text.value()),
            image: None,
            text_changed: text.changed_at(),
            text_ptr: (&raw const text) as usize,
        };
    }
}


impl<T: Display> MaybeObserver<T> for T {
    fn value(&self) -> &T {
        self
    }

    fn changed_at(&self) -> Changed {
        Changed::NeedsHash
    }
}

impl<T: Display> MaybeObserver<T> for Observer<T> {
    fn value(&self) -> &T {
        self
    }

    fn changed_at(&self) -> Changed {
        self.changed_at()
    }
}

