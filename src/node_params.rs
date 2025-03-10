use crate::*;
use crate::color::*;
use std::{fmt::Display, hash::{Hash, Hasher}};
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
    /// Which types of input the node can respond to
    pub senses: Sense,
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
    pub(crate) fn cosmetic_hash(&self) -> u64 {
        let mut hasher = FxHasher::default();
        self.rect.hash(&mut hasher);
        return hasher.finish();
    }

    pub(crate) fn layout_hash(&self) -> u64 {
        let mut hasher = FxHasher::default();
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

    pub fn is_scrollable(&self) -> bool {
        return self.layout.scrollable.x || self.layout.scrollable.y
    }

    pub const fn corners(mut self, corners: RoundedCorners) -> Self {
        self.rect.rounded_corners = corners;
        return self;
    }
}

/// An extended version of [`NodeParams`] that can hold borrowed data.
/// 
/// Created starting from a [`NodeParams`] and using methods that add text, images, etc. to it, like [`NodeParams::text()`].
/// 
/// Can be used in the same way as [`NodeParams`].
pub struct FullNodeParams<'a, T: Display + ?Sized> {
    pub params: NodeParams,
    pub text: Option<&'a T>,
    pub text_changed: Changed,
    pub text_ptr: usize,
    pub image: Option<&'static [u8]>,
}

impl<'a, T: Display + ?Sized> FullNodeParams<'a, T> {
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

    /// Add a [`NodeKey`] to the [`NodeParams`].
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

impl<'a, T: Display + ?Sized> FullNodeParams<'a, T> {
    /// Add text to the [`NodeParams`] from a `&'static str`.
    /// 
    /// `text` is assumed to be unchanged, so the [`Ui`] uses pointer equality to determine if it needs to update the text shown on screen.
    /// 
    /// If `text` changes, due to interior mutability or unsafe code, then the [`Ui`] will miss it.  
    pub fn static_text(self, text: &'static str) -> FullNodeParams<'static, str> {
        return FullNodeParams {
            params: self.params,
            image: self.image,
            text: Some(text),
            text_changed: Changed::Static,
            text_ptr: (&raw const text) as usize,
        }
    }
}

impl NodeParams {
    /// Add text to the [`NodeParams`] from a `&'static str`.
    /// 
    /// The [`Ui`] will have to hash `text` to determine if it needs to update the text shown on the screen. To avoid this performance penalty, use [`NodeParams::smart_text`], or [`NodeParams::static_text`] if `text` is an unchanging `'static str`. 
    pub fn hashed_text<'a>(self, text: &'a str) -> FullNodeParams<'a, str> {
        return FullNodeParams {
            params: self,
            text: Some(text),
            image: None,
            text_changed: Changed::NeedsHash,
            text_ptr: (&raw const text) as usize,
        }
    }

    /// Add text to the [`NodeParams`] from a `&'static str`.
    /// 
    /// `text` is assumed to be unchanged, so the [`Ui`] uses pointer equality to determine if it needs to update the text shown on screen.
    /// 
    /// If `text` changes, due to interior mutability or unsafe code, then the [`Ui`] will miss it.  
    pub fn static_text(self, text: &'static str) -> FullNodeParams<'static, str> {
        return FullNodeParams {
            params: self,
            text: Some(text),
            image: None,
            text_changed: Changed::Static,
            text_ptr: (&raw const text) as usize,
        }
    }

    pub fn smart_text<'a>(self, text: &'a Observer<impl AsRef<str>>) -> FullNodeParams<'a, str> {
        return FullNodeParams {
            params: self,
            text: Some(text.as_ref()),
            text_changed: text.changed_at(),
            text_ptr: (&raw const text) as usize,
            image: None,
        }
    }

    pub fn static_image(self, image: &'static [u8]) -> FullNodeParams<'static, str> {
        return FullNodeParams {
            params: self,
            text: None,
            image: Some(image),
            text_changed: Changed::Static,
            text_ptr: 0,
        }
    }
}

#[derive(Debug)]
pub enum Changed {
    ChangedAt(u64),
    NeedsHash,
    // isn't this about the same as ChangedAt(0)?
    Static,
}

impl<'a> Into<FullNodeParams<'a, str>> for NodeParams {
    fn into(self) -> FullNodeParams<'a, str> {
        FullNodeParams {
            params: self,
            text: None,
            text_changed: Changed::Static,
            text_ptr: 0,
            image: None,
        }
    }
}

impl<'a, T: Display + ?Sized> FullNodeParams<'a, T> {
    #[track_caller]
    pub(crate) fn key_or_anon_key(&self) -> NodeKey {
        return match self.params.key {
            Some(key) => key,
            None => NodeKey::new(Id(caller_location_hash()), ""),
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
    fn check_text_situation<T: Display + ?Sized>(&self, i: NodeI, params: &FullNodeParams<T>) -> TextVerdict {
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

    pub(crate) fn set_params_text<T: Display + ?Sized>(&mut self, i: NodeI, params: &FullNodeParams<T>) {       
        let Some(text) = params.text else {
            return
        };

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
        self.format_into_scratch(text);

        // todo: we're doing the hash twice in debug mode to do this check
        #[cfg(debug_assertions)]
        if reactive::is_in_skipped_reactive_block() {
            let hash = fx_hash(&self.format_scratch);
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

        match text_verdict {
            TextVerdict::Skip => { unreachable!("lol") },
            TextVerdict::HashAndSee => {
                if let Some(_) = self.nodes[i].text_id {
                    let hash = fx_hash(&self.format_scratch);
                    if let Some(last_hash) = self.nodes[i].last_text_hash {
                        if hash != last_hash {

                            log::trace!("Updating after hash");
                            self.nodes[i].last_text_hash = Some(hash);                    
                            self.get_uinode(i).text_from_fmtscratch();
                        } else {
                            log::trace!("Skipping after hash");
                        }
                        
                    } else {
                        self.get_uinode(i).text_from_fmtscratch();
                        self.nodes[i].last_text_hash = Some(hash);                    
                    }
                } else {
                    log::trace!("Updating (node had no text)");
                    self.get_uinode(i).text_from_fmtscratch();
                }
            },
            TextVerdict::UpdateWithoutHashing => {
                log::trace!("Updating without hash");
                self.get_uinode(i).text_from_fmtscratch();
                self.nodes[i].last_text_hash = None;
                // todo, think about this a bit more. we lose the hash.
            },
        };
    }


    pub(crate) fn set_params<T: Display + ?Sized>(&mut self, i: NodeI, params: &FullNodeParams<T>) {
        #[cfg(not(debug_assertions))]
        if reactive::is_in_skipped_reactive_block() {
            return;
        }
        
        if let Some(image) = params.image {
            self.get_uinode(i).static_image(image);
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
        
        // some off-by-one-frame errors or something. see notes.
        self.nodes[i].params = params.params;

        self.nodes[i].last_cosmetic_hash = new_cosmetic_hash;
        self.nodes[i].last_layout_hash = new_layout_hash;

        if layout_changed {
            self.push_partial_relayout(i);
        }
        if cosmetic_changed{
            self.push_cosmetic_update(i);
        }
    }
}

/// A trait for types that can *optionally* observe changes to themselves and report them to an [`Ui`] for more efficient displaying.
/// 
/// This is implemented for regular untracked types (no optimization) and [`Observer`] types. Todo: add some `Static` or `Immut` wrappers and implement this.
/// 
/// ```
/// # use keru::*;
/// let regular_string = "regular string".to_string();
/// 
/// let observed_string = Observer::new("observed string".to_string());
/// 
/// // NodeParams::text()'s argument is a MaybeObserver, so can take both a regular String and an Observed<String> 
/// let label_params = LABEL.text(&regular_string); // no optimization
/// let label_params = LABEL.text(&observed_string); // when this is added to the Ui, it will check if it has changed, and potentially skip some work.
/// ```
/// 
/// # Notes
/// 
/// The logical thing would be to implement `MaybeObserver` for any `T` and any `Observer<T>`, but this is not possible in current Rust. This problem is mostly solved by implementing it only for the types and traits that are exposed by functions like [`NodeParams::text()`], plus some compromises on [`Observer`]'s `Deref`ing abilities.
pub trait MaybeObserver<T: ?Sized> {
    fn value(&self) -> &T;
    fn changed_at(&self) -> Changed;
}

impl NodeParams {
    /// Add text to the [`NodeParams`].
    /// 
    /// The `text` argument can be a `&str`, a `String`, or any type that implements [`Display`], possibly wrapped by an [`Observer`], [`Static`] or [`Immut`] for efficiency.
    /// 
    /// 
    /// If a non-[`Observer`] type is used, the [`Ui`] will fall back to hashing the string to determine if the text needs updating.
    /// 
    /// This single generic function might be replaced by three separate functions: `hashed_text()`, `static_text()`, `observed_text()`, or similar. 
    pub fn text<'a, T, M>(self, text: &'a M) -> FullNodeParams<'a, T>
    where
        M: MaybeObserver<T> + ?Sized,
        T: Display + ?Sized,
    {
        return FullNodeParams {
            params: self,
            text: Some(&text.value()),
            image: None,
            text_changed: text.changed_at(),
            text_ptr: (&raw const text) as usize,
        };
    }
}


impl<T: Display + ?Sized> MaybeObserver<T> for T {
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
pub struct Static<T: ?Sized + 'static>(pub &'static T);

impl<T: Display + ?Sized + 'static> MaybeObserver<T> for Static<T> {
    fn value(&self) -> &T {
        &self.0
    }
    
    fn changed_at(&self) -> Changed {
        Changed::Static
    }
}


/// Same as `Static`, but without an explicit ``static` bound.
/// 
/// This struct can wrap any value: it is up to the programmer to ensure that wrapped variables never change. If this assumption is broken, the values displayed in the Ui will get out of sync with the real value of `T`.
/// 
/// You can always use an [`Observer<T>`](`Observer`) or a raw `T` to avoid this risk. If a raw `T` is passed, the [`Ui`] will hash the resulting text to make sure it stays synced.
pub struct Immut<T: ?Sized>(pub T);

impl<T: Display + ?Sized> MaybeObserver<T> for Immut<T> {
    fn value(&self) -> &T {
        &self.0
    }
    
    fn changed_at(&self) -> Changed {
        Changed::Static
    }
}
