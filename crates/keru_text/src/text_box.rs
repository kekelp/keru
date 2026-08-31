use std::{cell::RefCell, ptr::NonNull};

#[cfg(feature = "accessibility")]
use accesskit::{Node, NodeId};

use parley::*;
use winit::{
    event::WindowEvent, keyboard::{Key, NamedKey}, platform::modifier_supplement::KeyEventExtModifierSupplement, window::Window
};
use arboard::Clipboard;

use parley::{Affinity, Alignment, Selection};

use crate::*;
use std::hash::{Hash, Hasher};
use ahash::AHasher;

pub(crate) const X_TOLERANCE: f64 = 35.0;
/// How far a size has to move before it's worth acting on.
pub(crate) const SIZE_TOLERANCE: f32 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextIdentity {
    Hash(u64),
    Pointer(*const str),
}

pub(crate) fn hash_text(text: &str) -> u64 {
    let mut hasher = AHasher::default();
    text.hash(&mut hasher);
    return hasher.finish();
}

/// A text box stored inside a [`Text`] struct.
/// 
/// This struct can't be created directly. Instead, use [`Text::add_text_box()`] to create one within [`Text`] and get a [`TextBoxHandle`] back.
/// 
/// Then, pass the handle to [`Text::get_text_box_mut()`] to get a reference to it.
pub struct TextBox {
    pub(crate) text: Cow<'static, str>,
    pub(crate) text_identity: Option<TextIdentity>,
    pub(crate) style: StyleHandle,
    pub(crate) layout: Layout<ColorBrush>,

    #[cfg(feature = "accessibility")]
    pub(crate) layout_access: LayoutAccessibility,

    pub(crate) transform: Transform2D,
    // should get rid of it and have it only in the BoxData
    pub(crate) group_transform_index: Option<GroupTransformHandle>,
    pub(crate) depth: f32,
    pub(crate) opacity: f32,
    pub(crate) color_override: Option<ColorBrush>,
    pub(crate) selection: Selection,
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) alignment: Alignment,
    pub(crate) screen_space_clip_rect: Option<parley::BoundingBox>,
    pub(crate) auto_clip: bool,
    pub(crate) scroll_offset: (f32, f32),
    
    pub(crate) selectable: bool,

    /// A single-line box lays its text out with no wrapping width at all, so it never wraps and
    /// scrolls horizontally instead. Set by [`TextEdit::set_single_line()`].
    pub(crate) single_line: bool,
    
    pub(crate) hidden: bool,
    
    pub(crate) needs_reshape: bool,
    pub(crate) needs_line_break: bool,
    pub(crate) needs_quad_rebuild: bool,

    /// Cached result of [`TextBox::content_widths()`]. Only ever filled in if someone asks for
    /// it, and dropped whenever the layout is rebuilt.
    pub(crate) content_widths: Option<ContentWidths>,

    pub(crate) can_hide: bool,
    
    // Multi-window support
    pub(crate) window_id: Option<winit::window::WindowId>,
    
    pub(crate) render_data_info: RenderDataInfo,

    /// Optional explicit hitbox in local space (min_x, min_y, max_x, max_y).
    /// If set, hit_full_rect and hit_bounding_box will use this instead of the default behavior.
    pub(crate) explicit_hitbox: Option<(f32, f32, f32, f32)>,

    /// Unsafe raw pointer to the shared state
    pub(crate) shared_backref: NonNull<Shared>,
    /// Copy of the key that this textbox corresponds to. (or the parent text_edit_box! That's a bit messy).
    /// todo: try to get rid of this.
    pub(crate) key: usize,

    /// Key into `Shared::hit_tests` for this box's precomputed hit test entry.
    pub(crate) hit_test_key: usize,

    /// For cross-box selection: the next text box in the sequence.
    /// When selecting past the end of this box, selection continues into the next box.
    pub(crate) next_box: Option<usize>,
    /// For cross-box selection: the previous text box in the sequence.
    /// When selecting before the start of this box, selection continues into the previous box.
    pub(crate) prev_box: Option<usize>,

    /// Per-range style overrides applied on top of the base style during layout.
    pub(crate) ranged_style_properties: Vec<(StyleProperty<'static, ColorBrush>, std::ops::Range<usize>)>,
    pub(crate) style_property_overrides: Vec<StyleProperty<'static, ColorBrush>>,

    pub(crate) custom_tag: Option<u64>,
}

/// Metadata for the render data of a text box
#[derive(Clone)]
pub(crate) struct RenderDataInfo {
    /// Index into the text renderer's box_data array for this text box
    pub box_index: usize,
    /// The scroll offset when quads were prepared (for line-culling tolerance check)
    pub base_scroll: (f32, f32),
    /// Handle into the glyph quad heap for this box's current allocation.
    pub glyph_quad_handle: Option<Handle>,
}


thread_local! {
    static CLIPBOARD: RefCell<Clipboard> = RefCell::new(Clipboard::new().unwrap());
}

/// Runs the given closure with mutable access to the thread-local `Clipboard`.
pub fn with_clipboard<R>(f: impl FnOnce(&mut Clipboard) -> R) -> R {
    let res = CLIPBOARD.with_borrow_mut(|clipboard| f(clipboard));
    res
}

pub(crate) fn original_default_style() -> TextStyle2 { 
    TextStyle2 { 
        brush: ColorBrush([255,255,255,255]),
        font_size: 24.0,
        overflow_wrap: OverflowWrap::Normal,
        ..Default::default()
    }
}

// Helper to push a whole root style as individual properties.
fn push_root_style(builder: &mut RangedBuilder<'_, ColorBrush>, style: &TextStyle2) {
    builder.push_default(StyleProperty::FontFamily(style.font_family.clone()));
    builder.push_default(StyleProperty::FontSize(style.font_size));
    builder.push_default(StyleProperty::FontWidth(style.font_width));
    builder.push_default(StyleProperty::FontStyle(style.font_style));
    builder.push_default(StyleProperty::FontWeight(style.font_weight));
    builder.push_default(StyleProperty::FontVariations(style.font_variations.clone()));
    builder.push_default(StyleProperty::FontFeatures(style.font_features.clone()));
    builder.push_default(StyleProperty::Locale(style.locale.clone()));
    builder.push_default(StyleProperty::Brush(style.brush));
    builder.push_default(StyleProperty::Underline(style.has_underline));
    builder.push_default(StyleProperty::UnderlineOffset(style.underline_offset));
    builder.push_default(StyleProperty::UnderlineSize(style.underline_size));
    builder.push_default(StyleProperty::UnderlineBrush(style.underline_brush));
    builder.push_default(StyleProperty::Strikethrough(style.has_strikethrough));
    builder.push_default(StyleProperty::StrikethroughOffset(style.strikethrough_offset));
    builder.push_default(StyleProperty::StrikethroughSize(style.strikethrough_size));
    builder.push_default(StyleProperty::StrikethroughBrush(style.strikethrough_brush));
    builder.push_default(StyleProperty::LineHeight(style.line_height));
    builder.push_default(StyleProperty::WordSpacing(style.word_spacing));
    builder.push_default(StyleProperty::LetterSpacing(style.letter_spacing));
    builder.push_default(StyleProperty::WordBreak(style.word_break));
    builder.push_default(StyleProperty::OverflowWrap(style.overflow_wrap));
    builder.push_default(StyleProperty::TextWrapMode(style.text_wrap_mode));
}

impl TextBox {
    pub(crate) fn is_scroll_distance_above_tolerance(&self) -> bool {
        let distance_x = (self.scroll_offset.0 - self.render_data_info.base_scroll.0).abs();
        let distance_y = (self.scroll_offset.1 - self.render_data_info.base_scroll.1).abs();
    
        // Use the same tolerance as line culling
        const SCROLL_TOLERANCE: f32 = 200.0;
        let safe_scroll_tolerance = SCROLL_TOLERANCE - 5.0;
    
        if distance_x > safe_scroll_tolerance || distance_y > safe_scroll_tolerance {
            return true;
        } else {
            return false;
        }
    }

    pub(crate) fn new(
        text: impl Into<Cow<'static, str>>,
        pos: (f64, f64),
        size: (f32, f32),
        depth: f32,
        default_style_key: usize,
        shared_backref: NonNull<Shared>,
    ) -> Self {
        let position = Transform2D {
            translation: (pos.0 as f32, pos.1 as f32),
            rotation: 0.0,
            scale: 1.0,
        };
        Self {
            text: text.into(),
            text_identity: None,
            layout: Layout::new(),
            #[cfg(feature = "accessibility")]
            layout_access: LayoutAccessibility::default(),
            selectable: true,
            single_line: false,
            needs_reshape: true,
            needs_line_break: false,
            transform: position,
            group_transform_index: None,
            height: size.1,
            depth,
            opacity: 1.0,
            color_override: None,
            selection: Selection::default(),
            style: StyleHandle { key: default_style_key },
            width: size.0,
            alignment: Alignment::Center,
            screen_space_clip_rect: None,
            auto_clip: false,
            scroll_offset: (0.0, 0.0),
            hidden: false,
            needs_quad_rebuild: true,
            content_widths: None,
            can_hide: false,
            window_id: None,
            render_data_info: RenderDataInfo {
                box_index: 0,
                base_scroll: (0.0, 0.0),
                glyph_quad_handle: None,
            },
            explicit_hitbox: None,
            shared_backref,
            key: usize::MAX, // Remember to fill it in later, I guess.
            hit_test_key: usize::MAX, // Filled in after insertion into the slab.
            next_box: None,
            prev_box: None,
            ranged_style_properties: Vec::with_capacity(3),
            style_property_overrides: Vec::with_capacity(3),
            custom_tag: None,
        }
    }

    #[must_use]
    pub(crate) fn hit_full_rect(&self, cursor_pos: (f64, f64)) -> bool {
        // Transform cursor position to text box local space
        let local_pos = self.cursor_to_local(cursor_pos);

        let offset = (local_pos.x as f64, local_pos.y as f64);

        // If explicit hitbox is set, use it
        if let Some((min_x, min_y, max_x, max_y)) = self.explicit_hitbox {
            let hit = offset.0 >= min_x as f64
                && offset.0 <= max_x as f64
                && offset.1 >= min_y as f64
                && offset.1 <= max_y as f64;
            return hit;
        }

        // Default behavior
        let hit = offset.0 > -X_TOLERANCE
            && offset.0 < self.width as f64 + X_TOLERANCE
            && offset.1 > 0.0
            && offset.1 < self.height as f64;

        return hit;
    }

    /// Check if a screen-space cursor position is past the end of this text box.
    /// Used for determining when to extend selection to the next linked box.
    /// Returns true if the cursor is below the box, or on the last line and past the right edge.
    pub(crate) fn is_cursor_past_end(&self, cursor_pos: (f64, f64)) -> bool {
        let local_pos = self.cursor_to_local(cursor_pos);

        // Past the bottom of the box.
        let effective_height = self.height.min(self.layout.height());
        if local_pos.y > effective_height {
            return true;
        }

        // On or past the last line and past the right edge
        let text_height = self.layout.height();
        let last_line_y = text_height - self.layout.lines().last().map(|l| l.metrics().line_height).unwrap_or(0.0);
        if local_pos.y >= last_line_y && local_pos.x > self.layout.full_width() {
            return true;
        }

        false
    }

    /// Check if a screen-space cursor position is before the start of this text box.
    /// Used for determining when to extend selection to the previous linked box.
    /// Returns true if the cursor is above the box, or on the first line and before the left edge.
    pub(crate) fn is_cursor_before_start(&self, cursor_pos: (f64, f64)) -> bool {
        let local_pos = self.cursor_to_local(cursor_pos);

        // Before the top of the box
        if local_pos.y < 0.0 {
            return true;
        }

        // On the first line and before the left edge
        let first_line_height = self.layout.lines().next().map(|l| l.metrics().line_height).unwrap_or(self.height);
        if local_pos.y < first_line_height && local_pos.x < 0.0 {
            return true;
        }

        false
    }
}

impl TextBox {
    // SAFETY: only use if no other mutable references created with shared_mut() are live.
    // When using the public API, this is automatically enforced, because you can only get references to text boxes with Text::get_text_box() and similar functions, which borrow the whole Text struct.
    // In private functions, it should be easy enough. The idea is in the cases where there are multiple ways to get a mutable reference to Shared, there's always a single one that's "reasonable".
    // - If we're in a method on Text, just use &mut self.shared. It would be unreasonable to go fetch it in text.text_boxes[17].shared.
    // - If we're in a method on TextBox, use self.shared_mut().
    // - If we're in a random free function and the arguments are a random mixture of references to things that we don't remember the path to, delete the whole function, rewrite it as either a method on Text or a method on TextBox, and get the Shared from there.
    // Technically these functions should be marked as unsafe, but since they are private, we don't do it.
    // 
    // Note that the unsafe pointer isn't REALLY needed. We could make the library work the same way without it. (i.e. have the interface where the user calls get_text_box() and gets a single text box, which can also access the Shared state for styles and other shared stuff).
    // We'd just have to make a wrapper struct that holds a ref to the TextBox and a ref to Shared, and make get_text_box() return that.
    // However, such a wrapper struct has worse erdonomics: we would need to have separate structs for TextBox and TextBoxMut, the user would need to make the TextBoxMut binding itself mutable, etc.   
    // The point of the unsafe pointer is purely to avoid these ergonomic downsides, not to enable an intrinsecally un-Rust-y pattern.
    pub(crate) fn shared_mut(&mut self) -> &mut Shared {
        unsafe { self.shared_backref.as_mut() }
    }
    pub(crate) fn shared(&self) -> &Shared {
        unsafe { self.shared_backref.as_ref() }
    }

    pub(crate) fn rebuild_hit_test_data(&mut self) {
        let shape = compute_hit_test_shape(
            self.transform,
            self.group_transform_index,
            self.width,
            self.height,
            self.explicit_hitbox,
            &self.shared().render_data.group_transforms,
        );
        let key = self.hit_test_key;
        let depth = self.depth;
        let selectable = self.selectable;
        let hidden = self.hidden;
        let window_id = self.window_id;
        let entry = &mut self.shared_mut().hit_tests[key];
        entry.depth = depth;
        entry.selectable = selectable;
        entry.hidden = hidden;
        entry.window_id = window_id;
        entry.shape = shape;
    }

    /// Transforms a screen-space cursor position to text box local space,
    /// accounting for both the group transform (if any) and the per-box 
    /// transform.
    pub(crate) fn cursor_to_local(&self, cursor_pos: (f64, f64)) -> euclid::Point2D<f32, euclid::UnknownUnit> {
        let mut pos = euclid::Point2D::new(cursor_pos.0 as f32, cursor_pos.1 as f32);
         // First, inverse the group transform (if any)
        if let Some(handle) = self.group_transform_index 
        {
            let group = &self.shared().render_data.group_transforms[handle.
            0];
            // Forward transform is: pos = pos * scale + offset
            // Inverse is: pos = (pos - offset) / scale
            if group.scale != 0.0 {
                pos.x = (pos.x - group.offset[0]) / group.scale;                                   
                pos.y = (pos.y - group.offset[1]) / group.scale;
            }
        }
        // Then, inverse the per-box transform
        let inv_transform = self.transform().inverse().unwrap_or(Transform2D::identity());
        return inv_transform.transform_point(pos);
    }
    

    /// Returns a reference to the current style of the text box.
    pub fn style(&self) -> &TextStyle2 {
        &self.shared().styles[self.style.key].text_style
    }

    /// Returns `true` if the text box is currently hidden.
    pub fn hidden(&self) -> bool {
        self.hidden
    }

    /// Returns the current depth (z) of the text box.
    pub fn depth(&self) -> f32 {
        self.depth
    }

    /// Returns a reference to the text in the text nox. 
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the current position of the text box.
    pub fn position(&self) -> (f32, f32) {
        self.transform.translation
    }

    /// Returns the currently selected text, or `None` if no text is currently selected.
    pub fn selected_text(&self) -> Option<&str> {
        if !self.selection.is_collapsed() {
            self.text.get(self.selection.text_range())
        } else {
            None
        }
    }

    /// Returns the range of the currently selected text.
    /// 
    /// If the text in the text box changes, the result will become invalid, and might not even point
    /// to a valid UTF-8 substring of the new text anymore.
    pub fn selected_text_range(&self) -> Option<std::ops::Range<usize>> {
        if !self.selection.is_collapsed() {
            Some(self.selection.text_range())
        } else {
            None
        }
    }

    /// Returns the current selection of the text box.
    pub fn selection(&self) -> Selection {
        self.selection
    }

    /// Returns the current scroll offset of the text box.
    pub fn scroll_offset(&self) -> (f32, f32) {
        self.scroll_offset
    }

    /// Returns `true` if the text in the text box is currently selectable.
    pub fn selectable(&self) -> bool {
        self.selectable
    }

    #[doc(hidden)] 
    pub fn can_hide(&self) -> bool {
        self.can_hide
    }

    /// Returns the range of the glyph quads in the gpu render buffer.
    ///
    /// Must be called after [`Text::prepare_all()`].
    pub fn glyph_quad_range(&self) -> (usize, usize) {
        match self.render_data_info.glyph_quad_handle {
            Some(handle) => {
                let start = handle.vec_index(CHUNK_SIZE);
                let size = handle.size as usize;
                (start, start + size)
            }
            None => (0, 0),
        }
    }
}

impl TextBox {
    pub(crate) fn effective_clip_rect(&self) -> Option<parley::BoundingBox> {
        if self.auto_clip {
            Some(parley::BoundingBox {
                x0: self.scroll_offset.0 as f64,
                y0: self.scroll_offset.1 as f64,
                x1: (self.scroll_offset.0 + self.width) as f64,
                y1: (self.scroll_offset.1 + self.height) as f64,
            })
        } else {
            None
        }
    }

    #[cfg(feature = "accessibility")]
    /// Fills `parent_node` with TextRun children and pushes the child nodes into `out`.
    /// Call this from your accessibility tree builder after creating the parent node.
    pub fn build_accesskit_nodes(
        &mut self,
        parent_node: &mut Node,
        out: &mut Vec<(NodeId, Node)>,
        mut next_node_id: impl FnMut() -> NodeId,
    ) {
        self.refresh_layout();
        let (x, y) = self.position();
        let mut dummy = accesskit::TreeUpdate {
            nodes: Vec::new(),
            tree: None,
            tree_id: accesskit::TreeId::ROOT,
            focus: NodeId(0),
        };
        self.layout_access.build_nodes(
            &self.text,
            &self.layout,
            &mut dummy,
            parent_node,
            &mut next_node_id,
            x as f64,
            y as f64,
            |_, _| {},
        );
        out.extend(dummy.nodes);
    }

    pub(crate) fn handle_event(&mut self, event: &WindowEvent, _window: &Window, input_state: &TextInputState) -> bool {
        if self.hidden {
            return false;
        }

        let initial_selection = self.selection;

        let mut consumed = self.handle_event_no_edit(event, input_state, false);

        // Handle mouse wheel scrolling for multi-line text boxes with auto_clip
        if let WindowEvent::MouseWheel { delta, .. } = event {
            if self.auto_clip {
                let cursor_pos = input_state.mouse.cursor_pos;
                if self.hit_full_rect(cursor_pos) {
                    let scroll_amount = match delta {
                        winit::event::MouseScrollDelta::LineDelta(_x, y) => y * 30.0,
                        winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                    };

                    if scroll_amount.abs() > 0.1 {
                        let old_scroll = self.scroll_offset.1;
                        let new_scroll = old_scroll - scroll_amount;

                        self.refresh_layout();
                        let total_text_height = self.layout.height();
                        let text_height = self.height;
                        let max_scroll = (total_text_height - text_height).max(0.0).round();
                        let new_scroll = new_scroll.clamp(0.0, max_scroll).round();

                        if (new_scroll - old_scroll).abs() > 0.1 {
                            self.scroll_offset.1 = new_scroll;
                            self.shared_mut().scrolled = true;
                            consumed = true;
                        }
                    }
                }
            }
        }

        if selection_rects_changed(initial_selection, self.selection, false) {
            self.shared_mut().decorations_dirty = true;
        }

        return consumed;
    }

    /// The output bool says if the event was consumed by this text box.
    pub(crate) fn handle_event_no_edit(&mut self, event: &WindowEvent, input_state: &TextInputState, enable_auto_scroll: bool) -> bool {
        if self.hidden {
            return false;
        }
        if !self.selectable {
            self.reset_selection();
            return false;
        }

        let mut consumed = false;

        match event {
            WindowEvent::CursorMoved { position, .. } => {
                let cursor_pos = (position.x, position.y);
                // macOS seems to generate a spurious move after selecting word?
                if input_state.mouse.pointer_down {
                    // Transform cursor position to text box local space
                    let local_pos = self.cursor_to_local(cursor_pos);
                    let left = 0.0f32;
                    let top = 0.0f32;
                    let scroll_offset_x = self.scroll_offset.0;
                    let scroll_offset_y = self.scroll_offset.1;
                    let cursor_pos = (local_pos.x, local_pos.y);

                    // Check for auto-scroll when dragging near borders (only for text edits)
                    let mut new_scroll_x = scroll_offset_x;
                    let mut new_scroll_y = scroll_offset_y;

                    if enable_auto_scroll {
                        let scroll_margin = 20.0; // Distance from border to trigger auto-scroll
                        let scroll_speed = 5.0; // Scroll speed in pixels
                        let mut did_scroll = false;

                        // Check horizontal auto-scroll
                        if cursor_pos.0 - left < scroll_margin {
                            // Near left border - scroll left
                            new_scroll_x = (scroll_offset_x - scroll_speed).max(0.0);
                            if new_scroll_x != scroll_offset_x {
                                did_scroll = true;
                            }
                        } else if cursor_pos.0 > (left + self.width) - scroll_margin {
                            // Near right border - scroll right
                            let total_text_width = self.layout.full_width();
                            let max_scroll_x = (total_text_width - self.width).max(0.0);
                            new_scroll_x = (scroll_offset_x + scroll_speed).min(max_scroll_x);
                            if new_scroll_x != scroll_offset_x {
                                did_scroll = true;
                            }
                        }

                        // Check vertical auto-scroll
                        if cursor_pos.1 - top < scroll_margin {
                            // Near top border - scroll up
                            new_scroll_y = (scroll_offset_y - scroll_speed).max(0.0);
                            if new_scroll_y != scroll_offset_y {
                                did_scroll = true;
                            }
                        } else if cursor_pos.1 > (top + self.height) - scroll_margin {
                            // Near bottom border - scroll down
                            let total_text_height = self.layout.height();
                            let max_scroll_y = (total_text_height - self.height).max(0.0);
                            new_scroll_y = (scroll_offset_y + scroll_speed).min(max_scroll_y);
                            if new_scroll_y != scroll_offset_y {
                                did_scroll = true;
                            }
                        }
                        
                        // Apply scroll if needed
                        if did_scroll {
                            self.set_scroll_offset((new_scroll_x, new_scroll_y));
                            self.shared_mut().scrolled = true;
                        }
                    }

                    let cursor_pos = (
                        cursor_pos.0 - left + new_scroll_x,
                        cursor_pos.1 - top + new_scroll_y,
                    );
                    self.selection.extend_selection_to_point(
                        &self.layout,
                        cursor_pos.0,
                        cursor_pos.1,
                    );
                    consumed = true;
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let shift = input_state.modifiers.state().shift_key();
                if *button == winit::event::MouseButton::Left {
                    // Transform cursor position to text box local space
                    let local_pos = self.cursor_to_local(input_state.mouse.cursor_pos);
                    let cursor_pos = (
                        local_pos.x + self.scroll_offset.0,
                        local_pos.y + self.scroll_offset.1,
                    );

                    if state.is_pressed() {
                        let click_count = input_state.mouse.click_count;
                        match click_count {
                            2 => self.selection.select_word_at_point(&self.layout, cursor_pos.0, cursor_pos.1),
                            3 => self.selection.select_line_at_point(&self.layout, cursor_pos.0, cursor_pos.1),
                            _ => {
                                if ! shift {
                                    self.selection.move_to_point(&self.layout, cursor_pos.0, cursor_pos.1);
                                    self.shared_mut().reset_cursor_blink();
                                }
                                // Shift clicking is handled globally because of multi-box selections.
                            }
                        }
                        consumed = true;
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if !event.state.is_pressed() {
                    return consumed;
                }
                let mods_state = input_state.modifiers.state();
                let shift = mods_state.shift_key();
                let action_mod = if cfg!(target_os = "macos") {
                    mods_state.super_key()
                } else {
                    mods_state.control_key()
                };

                if shift {
                    match &event.logical_key {
                        Key::Named(NamedKey::Home) => {
                            if action_mod {
                                self.selection.select_to_text_start(&self.layout);
                            } else {
                                self.selection.select_to_line_start(&self.layout);
                            }
                            consumed = true;
                        }
                        Key::Named(NamedKey::End) => {
                            if action_mod {
                                self.selection.select_to_text_end(&self.layout);
                            } else {
                                self.selection.select_to_line_end(&self.layout);
                            }
                            consumed = true;
                        }
                        _ => (),
                    }
                }

                #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
                if action_mod {
                    match event.key_without_modifiers() {
                        Key::Character(c) => {
                            match c.as_str() {
                                "a" => {
                                    self.select_all();
                                    consumed = true;
                                }
                                _ => (),
                            }
                        }
                        _ => (),
                    };
                }
            }
            _ => {}
        }

        return consumed;
    }

    pub(crate) fn reset_selection(&mut self) {
        self.set_selection(self.selection.collapse());
    }

    /// Returns a mutable reference to the text content.
    /// 
    /// This returns a `Cow<'static, str>`, which can be set to a `String` or to `'static str`
    /// 
    /// To manipulate the text as a `String`, call `Cow::to_mut()` on the result, or use [`Self::text_mut_string()`]
    /// 
    /// After this method is called, the [`TextBox`] will assume that its text has changed. If you have to call this method many times with the same text every time, as when building a declarative or immediate mode interface, consider using a method like [`Self::set_text_hashed()`].
    pub fn text_mut(&mut self) -> &mut Cow<'static, str> {
        self.needs_reshape = true;
        self.text_identity = None;
        &mut self.text
    }

    /// Returns a mutable reference to the text content as a `String`. If the text was a borrowed `&str`, it will be cloned.
    ///
    /// This is a convenience method over [`Self::text_mut()`]
    pub fn text_mut_string(&mut self) -> &mut String {
        self.text_mut().to_mut()
    }

    /// Set the text in the text box.
    /// 
    /// If `new_text` is a `&`static` string, use [`Self::set_static_text()`].
    ///  
    /// After this method is called, the [`TextBox`] will assume that its text has changed. If you have to call this method many times with the same text every time, as when building a declarative or immediate mode interface, consider using a method like [`Self::set_text_hashed()`].
    pub fn set_text(&mut self, new_text: &str) {
        self.needs_reshape = true;
        self.text_identity = None;
        self.ranged_style_properties.clear();

        match &mut self.text {
            Cow::Owned(s) => {
                s.clear();
                s.push_str(new_text);
            }
            Cow::Borrowed(_) => {
                // We can't store &str references with arbitrary lifetimes, so if a text box that's currently holding a 'static str receives a non-'static string, we have to allocate a new string buffer.
                self.text = Cow::Owned(new_text.to_string());
            }
        }
    }

    /// Set the text in the text box.
    /// 
    /// This function will also hash the text and store a hash identity. If it is called again with the same text, it will detect that the text is unchanged and avoid doing an unnecessary relayout. This can be useful for building a declarative interface.
    pub fn set_text_hashed(&mut self, new_text: &str) {
        let text_hash = hash_text(new_text);
        let new_identity = TextIdentity::Hash(text_hash);
        if self.text_identity == Some(new_identity) {
            return;
        }
        self.text_identity = Some(new_identity);
        self.needs_reshape = true;
        self.ranged_style_properties.clear();

        match &mut self.text {
            Cow::Owned(s) => {
                s.clear();
                s.push_str(new_text);
            }
            Cow::Borrowed(_) => {
                self.text = Cow::Owned(new_text.to_string());
            }
        }
    }

    /// Sets the text in the text box, storing the value of the `text` pointer for comparison.
    /// 
    /// This function is similar to [`Self::set_static_text_with_pointer_check()`], but without an explicit `&'static` bound. It should only be used when `new_text` is an immutable string that won't be changed during its lifetime.
    /// 
    /// If this invariant holds, then the pointer allows this function to detect whether the text is changed or not if is called again with the same text, and potentially avoid doing an unnecessary relayout. This can be useful for building a declarative interface.
    /// 
    /// See also [`Self::set_text_hashed()`].
    pub fn set_text_with_pointer_check(&mut self, new_text: &str) {
        let new_identity = TextIdentity::Pointer(new_text as *const str);
        if self.text_identity == Some(new_identity) {
            return;
        }
        self.text_identity = Some(new_identity);
        self.needs_reshape = true;
        self.ranged_style_properties.clear();

        match &mut self.text {
            Cow::Owned(s) => {
                s.clear();
                s.push_str(new_text);
            }
            Cow::Borrowed(_) => {
                self.text = Cow::Owned(new_text.to_string());
            }
        }
    }

    /// Sets the text to a static string reference.
    pub fn set_static_text(&mut self, text: &'static str) {
        self.needs_reshape = true;
        self.text_identity = None;
        self.ranged_style_properties.clear();
        self.text = Cow::Borrowed(text);
    }

    /// Sets the text to a static string reference, storing the value of the `text` pointer for comparison.
    /// 
    /// If this function is called again with the same text, it will detect that the text is unchanged and avoid doing an unnecessary relayout. This can be useful for building a declarative interface.
    /// 
    /// This function assumes that an `&'static str` never change, and therefore it's safe to use pointer equality to compare them. If this is not the case, due to interior mutability or unsafe code, this function should not be used.
    pub fn set_static_text_with_pointer_check(&mut self, text: &'static str) {
        let new_identity = TextIdentity::Pointer(text as *const str);
        if self.text_identity == Some(new_identity) {
            return;
        }
        self.text_identity = Some(new_identity);
        self.needs_reshape = true;
        self.ranged_style_properties.clear();
        self.text = Cow::Borrowed(text);
    }

    /// Set a custom tag for this text box.
    pub fn set_custom_tag(&mut self, custom_tag: Option<u64>) {
        self.custom_tag = custom_tag;
    }

    /// Get the custom tag set to this text box, if any. 
    pub fn custom_tag(&mut self) -> Option<u64> {
        self.custom_tag
    }

    /// Sets the position of the text box.
    ///
    /// This function will only update the position if the new value is different from the current one.
    pub fn set_pos(&mut self, pos: (f64, f64)) {
        let new_translation = (pos.0 as f32, pos.1 as f32);
        if (self.transform.translation.0 - new_translation.0).abs() < 0.5
            && (self.transform.translation.1 - new_translation.1).abs() < 0.5
        {
            return;
        }
        self.transform.translation = new_translation;
        let i = self.render_data_info.box_index;
        self.shared_mut().render_data.box_data.get_mut(i).translation = [pos.0 as f32, pos.1 as f32];
    }

    /// Sets the transform of the text box.
    ///
    /// This function will only update the transform if the new value is different from the current one.
    pub fn set_transform(&mut self, transform: Transform2D) {
        if self.transform.translation == transform.translation
            && self.transform.rotation == transform.rotation
            && self.transform.scale == transform.scale
        {
            return;
        }
        self.transform.translation = transform.translation;
        self.transform.rotation = transform.rotation;
        self.transform.scale = transform.scale;
        let i = self.render_data_info.box_index;
        let box_data = self.shared_mut().render_data.box_data.get_mut(i);
        box_data.translation = [transform.translation.0, transform.translation.1];
        box_data.rotation = transform.rotation;
        box_data.scale = transform.scale;
        self.rebuild_hit_test_data();
    }

    /// Sets the text box to use a group transform in addition to its own one.
    pub fn set_group_transform(&mut self, transform: GroupTransformHandle) {
        if self.group_transform_index == Some(transform) {
            return;
        }

        let i = self.render_data_info.box_index;
        self.shared_mut().render_data.box_data.get_mut(i).group_transform_index = transform.0 as u32;
        self.group_transform_index = Some(transform);
        self.rebuild_hit_test_data();
    }

    /// Returns the current transform of the text box.
    pub fn transform(&self) -> Transform2D {
        Transform2D {
            translation: self.transform.translation,
            rotation: self.transform.rotation,
            scale: self.transform.scale,
        }
    }

    /// Sets the translation (position) of the text box.
    pub fn set_translation(&mut self, x: f32, y: f32) {
        if self.transform.translation == (x, y) {
            return;
        }
        self.transform.translation = (x, y);
        let i = self.render_data_info.box_index;
        self.shared_mut().render_data.box_data.get_mut(i).translation = [x, y];
        self.rebuild_hit_test_data();
    }

    /// Sets the rotation of the text box in radians.
    pub fn set_rotation(&mut self, radians: f32) {
        if self.transform.rotation == radians {
            return;
        }
        self.transform.rotation = radians;
        let i = self.render_data_info.box_index;
        self.shared_mut().render_data.box_data.get_mut(i).rotation = radians;
        self.rebuild_hit_test_data();
    }

    /// Hides or unhides the text box.
    pub fn set_hidden(&mut self, hidden: bool) {
        if self.hidden == hidden {
            return;
        }
        self.hidden = hidden;
        if hidden {
            self.reset_selection();
            let focused = self.shared().focused;
            if focused == Some(AnyBox::TextBox(self.key)) || focused == Some(AnyBox::TextEdit(self.key)) {
                self.shared_mut().focused = None;
            }
        }
        self.rebuild_hit_test_data();
    }

    /// Sets the depth (z-order) of the text box.
    ///
    /// This function will only update the depth if the new value is different from the current one.
    pub fn set_depth(&mut self, depth: f32) {
        if self.depth == depth {
            return;
        }
        self.depth = depth;
        let i = self.render_data_info.box_index;
        self.shared_mut().render_data.box_data.get_mut(i).depth = depth;
        self.rebuild_hit_test_data();
    }

    /// Sets the opacity multiplier of the text box, multiplied into every glyph's alpha.
    ///
    /// 1.0 = fully opaque. This function will only update the opacity if the new value is
    /// different from the current one. Opacity does not affect hit testing.
    pub fn set_opacity(&mut self, opacity: f32) {
        if self.opacity == opacity {
            return;
        }
        self.opacity = opacity;
        let i = self.render_data_info.box_index;
        self.shared_mut().render_data.box_data.get_mut(i).opacity = opacity;
    }

    /// Sets a screen-space clip rect.
    /// This is applied in screen space in the fragment shader,
    /// so it works correctly even when the text box is rotated.
    ///
    /// This function will only update the clip rect if the new value is different from the current one.
    pub fn set_clip_rect(&mut self, clip_rect: Option<parley::BoundingBox>) {
        if self.screen_space_clip_rect == clip_rect {
            return;
        }
        self.screen_space_clip_rect = clip_rect;
        let i = self.render_data_info.box_index;
        let box_data = self.shared_mut().render_data.box_data.get_mut(i);
        match clip_rect {
            Some(clip) => {
                box_data.screen_clip_x = [clip.x0 as f32, clip.x1 as f32];
                box_data.screen_clip_y = [clip.y0 as f32, clip.y1 as f32];
            }
            None => {
                box_data.screen_clip_x = [f32::NEG_INFINITY, f32::INFINITY];
                box_data.screen_clip_y = [f32::NEG_INFINITY, f32::INFINITY];
            }
        }
    }

    /// Sets whether automatic clipping to the text box bounds is enabled.
    /// When enabled, text is clipped to the rectangle defined by scroll_offset and the box size.
    ///
    /// This function will only update the auto-clip setting if the new value is different from the current one.
    pub fn set_auto_clip(&mut self, auto_clip: bool) {
        if self.auto_clip == auto_clip {
            return;
        }
        self.auto_clip = auto_clip;
        self.needs_quad_rebuild = true;
    }

    /// Sets an explicit hitbox for hit detection in local space (min_x, min_y, max_x, max_y).
    ///
    /// When set, `hit_full_rect` and `hit_bounding_box` will use this hitbox instead of
    /// computing one from the text box dimensions or layout.
    pub fn set_hitbox(&mut self, hitbox: Option<(f32, f32, f32, f32)>) {
        self.explicit_hitbox = hitbox;
        self.rebuild_hit_test_data();
    }

    /// Returns the explicit hitbox if set.
    pub fn hitbox(&self) -> Option<(f32, f32, f32, f32)> {
        self.explicit_hitbox
    }


    /// Sets the scroll offset for the text box.
    ///
    /// This function will only update the scroll offset if the new value is different from the current one.
    pub fn set_scroll_offset(&mut self, offset: (f32, f32)) {
        if self.scroll_offset == offset {
            return;
        }
        self.scroll_offset = offset;
        let i = self.render_data_info.box_index;
        let auto_clip = self.auto_clip;
        let width = self.width;
        let height = self.height;
        let box_data = self.shared_mut().render_data.box_data.get_mut(i);
        box_data.scroll_offset = [offset.0, offset.1];
        if auto_clip {
            box_data.clip_rect_x = [offset.0, offset.0 + width];
            box_data.clip_rect_y = [offset.1, offset.1 + height];
        }
    }

    /// Sets the style for the text box.
    ///
    /// This function will only trigger a relayout if the new style is different from the old one.
    pub fn set_style(&mut self, style: &StyleHandle) {
        if self.style.key == style.key {
            return;
        }
        self.style = style.sneak_clone();
        self.needs_reshape = true;
    }

    pub(crate) fn text_inner(&self) -> &str {
        &self.text
    }

    pub(crate) fn get_scale_factor(&self) -> f64 {
        let scale_factor = if let Some(window_id) = self.window_id {           
            self.shared().windows.iter().find(|info| info.window_id == window_id)
                .map(|info| info.scale_factor).unwrap_or(1.0)
        } else {
            self.shared().windows.first().map(|w| w.scale_factor).unwrap_or(1.0)
        };

        scale_factor * self.shared().explicit_scale_factor
    }

    pub(crate) fn rebuild_layout(&mut self) {
        let scale_factor = self.get_scale_factor();
        
        let k = self.style.key;
        // even sketchier partial borrow moment. self.shared_mut() borrows the whole self
        let shared = unsafe { self.shared_backref.as_mut() };
        let style = &mut shared.styles[k].text_style;
        
        let layout_cx = &mut shared.layout_cx;
        let font_cx = &mut shared.font_cx;

        let mut builder = layout_cx.ranged_builder(font_cx, &self.text, scale_factor as f32, true);
        push_root_style(&mut builder, style);

        for prop in &self.style_property_overrides {
            builder.push(prop.clone(), 0..usize::MAX);
        }

        for (prop, range) in &self.ranged_style_properties {
            // properties are usually lightweight Copy types. Even font names should usually be the & 'static str variant of Cow.
            let prop: StyleProperty<'static, ColorBrush> = prop.clone();
            builder.push(prop, range.clone());
        }

        let mut layout = builder.build(&self.text);

        // A single-line box has no wrapping width at all, so it never wraps.
        let max_advance = if self.single_line { None } else { Some(self.width) };

        layout.break_all_lines(max_advance);

        layout.align(
            self.alignment,
            AlignmentOptions::default(),
        );

        self.layout = layout;
        self.needs_reshape = false;
        self.needs_line_break = false;
        // The shaping changed, so the cached content widths are stale. A re-break alone can't
        // invalidate them: they're measured from the clusters, independently of any line breaks.
        self.content_widths = None;

        // todo: does this do anything?
        self.selection = self.selection.refresh(&self.layout);
    }

    /// Re-break the existing shaped text into lines, without rebuilding it.
    ///
    /// This is the cheap half of [`Self::rebuild_layout()`]: parley keeps the shaped runs and
    /// clusters in the layout and only recomputes the lines, so nothing touches the font stack.
    pub(crate) fn rebreak_lines(&mut self) {
        // A single-line box has no wrapping width at all, so it never wraps.
        let max_advance = if self.single_line { None } else { Some(self.width) };

        self.layout.break_all_lines(max_advance);

        self.layout.align(
            self.alignment,
            AlignmentOptions::default(),
        );

        self.needs_line_break = false;

        self.selection = self.selection.refresh(&self.layout);
    }


    /// Sets the size of the text box.
    pub fn set_size(&mut self, size: (f32, f32)) {
        self.set_width(size.0);
        self.set_height(size.1);
    }

    /// Sets the width of the text box, which is also the width its lines wrap at.
    ///
    /// This is the only thing that can change the line breaking, so it's the one to call from a
    /// layout pass, which has to know how tall the text ends up before it can know where the box
    /// goes. The height can then be set separately, once it's known.
    pub fn set_width(&mut self, width: f32) {
        if (self.width - width).abs() <= SIZE_TOLERANCE {
            return;
        }
        self.width = width;
        // A different width doesn't change the shaping, only where the lines break.
        self.needs_line_break = true;
        self.rebuild_hit_test_data();
    }

    /// Sets the height of the text box.
    ///
    /// The height doesn't affect the text layout at all: nothing about shaping or line breaking
    /// depends on it. It's only used for clipping, hit testing and scrolling.
    pub fn set_height(&mut self, height: f32) {
        if (self.height - height).abs() <= SIZE_TOLERANCE {
            return;
        }
        self.height = height;
        self.rebuild_hit_test_data();
    }

    /// Returns the size of the text box.
    /// 
    /// By default text boxes don't clip the text. Depending on the purpose, you might want to use the size of the [layout](`Self::layout()`) rather than the size of the text box itself.
    pub fn size(&self) -> (f32, f32) {
        (self.width, self.height)
    }

    /// Sets the text alignment.
    ///
    /// This function will only trigger a relayout if the new alignment is different from the old one.
    pub fn set_alignment(&mut self, alignment: Alignment) {
        if self.alignment == alignment {
            return;
        }
        self.alignment = alignment;
        self.needs_line_break = true;
    }

    /// Sets the text alignment.
    ///
    /// This function will only trigger a relayout if the new alignment is different from the old one.
    pub fn alignment(&self) -> Alignment {
        self.alignment
    }

    /// Adds a [`StyleProperty`] override for the given byte range of the text.
    ///
    /// The property is applied each time the layout is rebuilt, on top of the base style.
    /// Multiple overlapping ranges are applied in insertion order.
    /// Call [`needs_relayout`](Self::needs_relayout) or otherwise trigger a relayout after
    /// calling this to see the effect.
    pub fn push_ranged_style_property(&mut self, prop: StyleProperty<'static, ColorBrush>, range: std::ops::Range<usize>) {
        self.ranged_style_properties.push((prop, range));
        self.needs_reshape = true;
    }

    /// Sets the whole-box [`StyleProperty`] overrides, replacing any previously set overrides.
    ///
    /// This method won't cause a relayout unless the new properties are different than the previous ones.
    pub fn set_style_property_overrides(&mut self, properties: &[StyleProperty<'static, ColorBrush>]) {
        if self.style_property_overrides.as_slice() == properties {
            return;
        }
        self.style_property_overrides = properties.to_vec();
        self.needs_reshape = true;
    }

    /// Clears all per-range style property overrides.
    pub fn clear_ranged_style_properties(&mut self) {
        self.ranged_style_properties.clear();
        self.needs_reshape = true;
    }

    /// Clears all style properties of a range.
    pub fn clear_style_properties_in_range(&mut self, range_to_clear: std::ops::Range<usize>) {
        let c_start = range_to_clear.start;
        let c_end = range_to_clear.end;

        // handle internal splits where the entry fully contains the clear range,
        // and we need to split it it half. 
        let original_len = self.ranged_style_properties.len();
        for i in 0..original_len {
            let tail_end = {
                let (_, r) = &self.ranged_style_properties[i];
                if r.start < c_start && r.end > c_end { Some(r.end) } else { None }
            };
            if let Some(end) = tail_end {
                let prop = self.ranged_style_properties[i].0.clone();
                self.ranged_style_properties[i].1.end = c_start;  // head: [start, c_start)
                self.ranged_style_properties.push((prop, c_end..end));  // tail: [c_end, end)
            }
        }

        // Remove fully-covered entries and trim the ones that intersect at one side.
        // Entries already trimmed in pass 1 no longer intersect and pass through unchanged.
        self.ranged_style_properties.retain_mut(|(_, r)| {
            if r.end <= c_start || r.start >= c_end {
                true  // no intersection
            } else if r.start >= c_start && r.end <= c_end {
                false  // fully covered: drop
            } else if r.start < c_start {
                r.end = c_start;  // left-side overlap: trim end
                true
            } else {
                r.start = c_end;  // right-side overlap: trim start
                true
            }
        });

        self.needs_reshape = true;
    }

    /// Adjusts ranged style properties after a text edit.
    ///
    /// `edit_start..edit_end` is the byte range that was deleted; `inserted_len` is the byte
    /// length of the text inserted in its place. Ranges that collapse to zero length after the
    /// adjustment are removed.
    pub(crate) fn adjust_ranged_styles_for_edit(&mut self, edit_start: usize, edit_end: usize, inserted_len: usize) {
        if self.ranged_style_properties.is_empty() {
            return;
        }
        let delta: isize = inserted_len as isize - (edit_end - edit_start) as isize;
        self.ranged_style_properties.retain_mut(|(_, range)| {
            let new_start = if range.start < edit_start {
                range.start
            } else if range.start < edit_end {
                // start was inside the deleted region: clamp to the edit point
                edit_start
            } else {
                (range.start as isize + delta) as usize
            };
            let new_end = if range.end <= edit_start {
                range.end
            } else if range.end < edit_end {
                // end was inside the deleted region: clamp to the edit point
                edit_start
            } else {
                (range.end as isize + delta) as usize
            };
            range.start = new_start;
            range.end = new_end;
            new_start < new_end
        });
    }

    // todo: scale factor was meant to be a different thing?
    /// Sets the scale factor for the text.
    ///
    /// This function will only trigger a relayout if the new scale is different from the old one.
    pub fn set_scale(&mut self, scale: f32) {
        if self.transform.scale == scale {
            return;
        }
        self.transform.scale = scale;
        self.needs_reshape = true;
    }

    // #[cfg(feature = "accesskit")]
    // #[inline]
    // /// Perform an accessibility update if the layout is valid.
    // ///
    // /// Returns `None` if the layout is not up-to-date.
    // /// You can call [`refresh_layout`](Self::refresh_layout) before using this method,
    // /// to ensure that the layout is up-to-date.
    // /// The [`accessibility`](PlainEditorDriver::accessibility) method on the driver type
    // /// should be preferred if the contexts are available, which will do this automatically.
    // pub fn try_accessibility(
    //     &mut self,
    //     update: &mut TreeUpdate,
    //     node: &mut Node,
    //     next_node_id: impl FnMut() -> NodeId,
    //     x_offset: f64,
    //     y_offset: f64,
    // ) -> Option<()> {
    //     if self.needs_relayout {
    //         return None;
    //     }
    //     self.accessibility_unchecked(update, node, next_node_id, x_offset, y_offset);
    //     Some(())
    // }

    /// Update the selection, and nudge the `Generation` if something other than `h_pos` changed.
    pub(crate) fn set_selection(&mut self, new_sel: Selection) {

        // This debug code is quite useful when diagnosing selection problems.
        #[allow(clippy::print_stderr)] // reason = "unreachable debug code"
        if false {
            let focus = new_sel.focus();
            let cluster = focus.logical_clusters(&self.layout);
            let dbg = (
                cluster[0].as_ref().map(|c| &self.text[c.text_range()]),
                focus.index(),
                focus.affinity(),
                cluster[1].as_ref().map(|c| &self.text[c.text_range()]),
            );
            eprint!("{dbg:?}");
            let cluster = focus.visual_clusters(&self.layout);
            let dbg = (
                cluster[0].as_ref().map(|c| &self.text[c.text_range()]),
                cluster[0]
                    .as_ref()
                    .map(|c| if c.is_word_boundary() { " W" } else { "" })
                    .unwrap_or_default(),
                focus.index(),
                focus.affinity(),
                cluster[1].as_ref().map(|c| &self.text[c.text_range()]),
                cluster[1]
                    .as_ref()
                    .map(|c| if c.is_word_boundary() { " W" } else { "" })
                    .unwrap_or_default(),
            );
            eprintln!(" | visual: {dbg:?}");
        }
        self.selection = new_sel;
    }

    // #[cfg(feature = "accesskit")]
    // /// Perform an accessibility update, assuming that the layout is valid.
    // ///
    // /// The wrapper [`accessibility`](PlainEditorDriver::accessibility) on the driver type should
    // /// be preferred.
    // ///
    // /// You should always call [`refresh_layout`](Self::refresh_layout) before using this method,
    // /// with no other modifying method calls in between.
    // pub(crate) fn accessibility_unchecked(
    //     &mut self,
    //     update: &mut TreeUpdate,
    //     node: &mut Node,
    //     next_node_id: impl FnMut() -> NodeId,
    //     x_offset: f64,
    //     y_offset: f64,
    // ) {
    //     self.layout_access.build_nodes(
    //         &self.text,
    //         &self.layout,
    //         update,
    //         node,
    //         next_node_id,
    //         x_offset,
    //         y_offset,
    //     );
    //     if self.show_cursor {
    //         if let Some(selection) = self
    //             .selection
    //             .to_access_selection(&self.layout, &self.layout_access)
    //         {
    //             node.set_text_selection(selection);
    //         }
    //     } else {
    //         node.clear_text_selection();
    //     }
    //     node.add_action(accesskit::Action::SetTextSelection);
    // }


    /// Move the cursor to the cluster boundary nearest this point in the layout.
    pub(crate) fn move_to_point(&mut self, x: f32, y: f32) {
        self.set_selection(Selection::from_point(&self.layout, x, y));
    }

    /// Move the cursor to the start of the text.
    pub(crate) fn move_to_text_start(&mut self) {
        self.set_selection(
            self.selection.move_lines(&self.layout, isize::MIN, false),
        );
    }

    /// Move the cursor to the start of the physical line.
    pub(crate) fn move_to_line_start(&mut self) {
        self.set_selection(self.selection.line_start(&self.layout, false));
    }

    /// Move the cursor to the end of the text.
    pub(crate) fn move_to_text_end(&mut self) {
        self.set_selection(
            self.selection
                .move_lines(&self.layout, isize::MAX, false),
        );
    }

    /// Move the cursor to the end of the physical line.
    pub(crate) fn move_to_line_end(&mut self) {
        self.set_selection(self.selection.line_end(&self.layout, false));
    }

    /// Move up to the closest physical cluster boundary on the previous line, preserving the horizontal position for repeated movements.
    pub(crate) fn move_up(&mut self) {
        self.set_selection(self.selection.previous_line(&self.layout, false));
    }

    /// Move down to the closest physical cluster boundary on the next line, preserving the horizontal position for repeated movements.
    pub(crate) fn move_down(&mut self) {
        self.set_selection(self.selection.next_line(&self.layout, false));
    }

    /// Move to the next cluster left in visual order.
    pub(crate) fn move_left(&mut self) {
        self.set_selection(
            self.selection.previous_visual(&self.layout, false),
        );
    }

    /// Move to the next cluster right in visual order.
    pub(crate) fn move_right(&mut self) {
        self.set_selection(self.selection.next_visual(&self.layout, false));
    }

    /// Move to the next word boundary left.
    pub(crate) fn move_word_left(&mut self) {
        self.set_selection(
            self.selection.previous_visual_word(&self.layout, false),
        );
    }


    /// Move to the next word boundary right.
    pub(crate) fn move_word_right(&mut self) {
        self.set_selection(
            self.selection.next_visual_word(&self.layout, false),
        );
    }

    /// Select the whole text.
    pub(crate) fn select_all(&mut self) {
        self.set_selection(
            Selection::from_byte_index(&self.layout, 0_usize, Affinity::default()).move_lines(
                &self.layout,
                isize::MAX,
                true,
            ),
        );
    }

    /// Collapse selection into caret.
    pub(crate) fn collapse_selection(&mut self) {
        self.set_selection(self.selection.collapse());
    }

    /// Move the selection focus point to the cluster boundary closest to point.
    pub(crate) fn extend_selection_to_point(&mut self, x: f32, y: f32) {
        self.selection.extend_selection_to_point(&self.layout, x, y);
    }

    /// Returns the layout, refreshing it if needed.
    pub fn layout(&mut self) -> &Layout<ColorBrush> {
        self.refresh_layout();
        &self.layout
    }

    /// Rebuild the layout if needed.
    pub fn refresh_layout(&mut self) {
        if self.needs_reshape {
            self.rebuild_layout();
            self.needs_quad_rebuild = true;
        } else if self.needs_line_break {
            self.rebreak_lines();
            self.needs_quad_rebuild = true;
        }
    }

    /// Set a color override for the whole text box.
    pub fn set_color_override(&mut self, color_override: Option<ColorBrush>) {
        if self.color_override != color_override {
            self.color_override = color_override;
            // The box data is rewritten every frame, so this only has to make sure that a frame
            // happens at all.
            self.shared_mut().decorations_dirty = true;
        }
    }

    /// The width the text wants, independently of the box's current size.
    pub fn content_widths(&mut self) -> ContentWidths {
        self.refresh_layout();
        if let Some(cached) = self.content_widths {
            return cached;
        }
        let content_widths = self.layout.calculate_content_widths();
        self.content_widths = Some(content_widths);
        content_widths
    }

    /// Returns `true` if the text box layout will have to be recomputed because of changes to the text content, size, alignment, font size, etc.
    ///
    /// The layout will automatically be recomputed when preparing the text for render, or when calling [TextBox::layout()].
    pub fn needs_relayout(&mut self) -> bool {
        return self.needs_reshape || self.needs_line_break;
    }

    /// Sets whether the text is selectable.
    pub fn set_selectable(&mut self, selectable: bool) {
        self.selectable = selectable;
    }
    
    /// Sets focus to this text box.
    pub fn set_focus(&mut self) {
        let k = AnyBox::TextBox(self.key);
        self.shared_mut().refocus(Some(k));
    }
}

pub use parley::BoundingBox;

pub(crate) trait SelectionExt {
    fn move_to_point(&mut self, layout: &Layout<ColorBrush>, x: f32, y: f32);
    fn select_word_at_point(&mut self, layout: &Layout<ColorBrush>, x: f32, y: f32);
    fn select_line_at_point(&mut self, layout: &Layout<ColorBrush>, x: f32, y: f32);
    fn extend_selection_to_point(&mut self, layout: &Layout<ColorBrush>, x: f32, y: f32);
    fn select_to_text_start(&mut self, layout: &Layout<ColorBrush>);
    fn select_to_line_start(&mut self, layout: &Layout<ColorBrush>);
    fn select_to_text_end(&mut self, layout: &Layout<ColorBrush>);
    fn select_to_line_end(&mut self, layout: &Layout<ColorBrush>);
    fn select_up(&mut self, layout: &Layout<ColorBrush>);
    fn select_down(&mut self, layout: &Layout<ColorBrush>);
    fn select_left(&mut self, layout: &Layout<ColorBrush>);
    fn select_right(&mut self, layout: &Layout<ColorBrush>);
    fn select_word_left(&mut self, layout: &Layout<ColorBrush>);
    fn select_word_right(&mut self, layout: &Layout<ColorBrush>);
}

impl SelectionExt for Selection {
    fn move_to_point(&mut self, layout: &Layout<ColorBrush>, x: f32, y: f32) {
        *self = Selection::from_point(layout, x, y);
    }

    fn select_word_at_point(&mut self, layout: &Layout<ColorBrush>, x: f32, y: f32) {
        *self = Selection::word_from_point(layout, x, y);
    }

    fn select_line_at_point(&mut self, layout: &Layout<ColorBrush>, x: f32, y: f32) {
        *self = Selection::line_from_point(layout, x, y);
    }

    fn extend_selection_to_point(&mut self, layout: &Layout<ColorBrush>, x: f32, y: f32) {
        *self = self.extend_to_point(layout, x, y);
    }

    fn select_to_text_start(&mut self, layout: &Layout<ColorBrush>) {
        *self = self.move_lines(layout, isize::MIN, true);
    }

    fn select_to_line_start(&mut self, layout: &Layout<ColorBrush>) {
        *self = self.line_start(layout, true);
    }

    fn select_to_text_end(&mut self, layout: &Layout<ColorBrush>) {
        *self = self.move_lines(layout, isize::MAX, true);
    }

    fn select_to_line_end(&mut self, layout: &Layout<ColorBrush>) {
        *self = self.line_end(layout, true);
    }

    fn select_up(&mut self, layout: &Layout<ColorBrush>) {
        *self = self.previous_line(layout, true);
    }

    fn select_down(&mut self, layout: &Layout<ColorBrush>) {
        *self = self.next_line(layout, true);
    }

    fn select_left(&mut self, layout: &Layout<ColorBrush>) {
        *self = self.previous_visual(layout, true);
    }

    fn select_right(&mut self, layout: &Layout<ColorBrush>) {
        *self = self.next_visual(layout, true);
    }

    fn select_word_left(&mut self, layout: &Layout<ColorBrush>) {
        *self = self.previous_visual_word(layout, true);
    }

    fn select_word_right(&mut self, layout: &Layout<ColorBrush>) {
        *self = self.next_visual_word(layout, true);
    }
}



