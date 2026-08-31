use std::{
    fmt::Display, ops::Range, ptr::NonNull, time::{Duration, Instant}
};

use parley::*;
use winit::{
    event::{Ime, Touch, WindowEvent}, keyboard::{Key, NamedKey}, platform::modifier_supplement::KeyEventExtModifierSupplement, window::Window
};

pub(crate) const CURSOR_WIDTH: f32 = 3.0;
pub (crate) const CURSOR_COLOR: u32 = 0xee_ee_ee_ff;


use crate::*;

macro_rules! clear_placeholder_partial_borrows {
    ($self:expr) => {
        if $self.showing_placeholder {
            $self.text_box.text_mut_string().clear();
            $self.showing_placeholder = false;
            $self.text_box.set_color_override(None);
            $self.text_box.refresh_layout();
            $self.text_box.move_to_text_start();
        }
    };
}

/// Defines how newlines are entered in a text edit box.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewlineMode {
    /// Enter key inserts newlines (default for multi-line)
    Enter,
    /// Shift+Enter inserts newlines, Enter is ignored
    ShiftEnter,
    /// Ctrl+Enter inserts newlines, Enter is ignored (or Cmd+Enter on macOS)
    CtrlEnter,
    /// No newlines allowed (used automatically for single-line mode)
    None,
}

impl Default for NewlineMode {
    fn default() -> Self {
        NewlineMode::Enter
    }
}

/// A string that may be split into two parts (used for IME composition).
#[derive(Debug, Clone, Copy)]
pub struct SplitString<'source>(pub(crate) [&'source str; 2]);

impl<'source> SplitString<'source> {
    /// Get the characters of this string.
    pub fn chars(self) -> impl Iterator<Item = char> + 'source {
        self.into_iter().flat_map(str::chars)
    }
}

impl PartialEq<&'_ str> for SplitString<'_> {
    fn eq(&self, other: &&'_ str) -> bool {
        let [a, b] = self.0;
        let mid = a.len();
        // When our MSRV is 1.80 or above, use split_at_checked instead.
        // is_char_boundary checks bounds
        let (a_1, b_1) = if other.is_char_boundary(mid) {
            other.split_at(mid)
        } else {
            return false;
        };

        a_1 == a && b_1 == b
    }
}

impl Display for SplitString<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let [a, b] = self.0;
        write!(f, "{a}{b}")
    }
}

/// Iterate through the source strings.
impl<'source> IntoIterator for SplitString<'source> {
    type Item = &'source str;
    type IntoIter = <[&'source str; 2] as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

pub(crate) fn selection_rects_changed(initial_selection: Selection, new_selection: Selection, is_editable: bool) -> bool {
    // For non-editable boxes, if both selections are collapsed, no change
    if !is_editable && initial_selection.is_collapsed() && new_selection.is_collapsed() {
        return false;
    }
    
    // Compare selections ignoring affinity-only changes
    let initial_range = initial_selection.text_range();
    let new_range = new_selection.text_range();
    
    initial_range != new_range
}

/// A text edit box.
/// 
/// This struct can't be created directly. Instead, use [`Text::add_text_edit()`] or similar functions to create one within [`Text`] and get a [`TextEditHandle`] back.
/// 
/// Then, pass the handle to [`Text::get_text_edit_mut()`] to get a reference to it.
pub struct TextEdit {
    pub(crate) compose: Option<Range<usize>>,
    pub(crate) start_time: Option<Instant>,
    pub(crate) blink_period: Duration,
    pub(crate) history: TextEditHistory,
    pub(crate) newline_mode: NewlineMode,
    pub(crate) disabled: bool,
    pub(crate) text_box: TextBox,
    pub(crate) needs_scroll_update: bool,
    pub(crate) showing_placeholder: bool,
    pub(crate) placeholder_text: Option<Cow<'static, str>>,
    pub(crate) placeholder_text_identity: Option<TextIdentity>,
    /// Whether the text was changed by user input since the last frame.
    pub(crate) text_changed: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct ScrollAnimation {
    pub start_offset: f32,
    pub target_offset: f32,
    pub start_time: Instant,
    pub duration: Duration,
    pub direction: ScrollDirection,
    pub handle: ClonedTextEditHandle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScrollDirection {
    Horizontal,
    Vertical,
}

impl TextEdit {
    pub(crate) fn new(text: String, pos: (f64, f64), size: (f32, f32), depth: f32, default_style_key: usize, shared_backref: NonNull<Shared>) -> Self {
        let mut text_box = TextBox::new(text, pos, size, depth, default_style_key, shared_backref);
        text_box.auto_clip = true;
        text_box.alignment = Alignment::Start;
        return Self {
            compose: Default::default(),
            start_time: Default::default(),
            blink_period: Default::default(),
            history: TextEditHistory::new(),
            newline_mode: NewlineMode::default(),
            disabled: false,
            text_box,
            needs_scroll_update: false,
            showing_placeholder: false,
            placeholder_text: None,
            placeholder_text_identity: None,
            text_changed: false,
        }
    }
}


impl ScrollAnimation {

    pub fn get_current_offset(&self) -> f32 {
        let elapsed = self.start_time.elapsed();
        if elapsed >= self.duration {
            return self.target_offset;
        }

        let progress = elapsed.as_secs_f32() / self.duration.as_secs_f32();
        // Use smooth easing function (ease-out cubic)
        let eased_progress = 1.0 - (1.0 - progress).powi(3);
        
        self.start_offset + (self.target_offset - self.start_offset) * eased_progress
    }

    pub fn is_finished(&self) -> bool {
        self.start_time.elapsed() >= self.duration
    }
}

impl TextEdit {
    /// Sets whether the text edit is single-line or multi-line.
    pub fn set_single_line(&mut self, single_line: bool) {
        if self.text_box.single_line != single_line {
            self.text_box.single_line = single_line;
            // This only decides whether there's a wrapping width at all, so the shaped text is
            // still good and only the lines have to be redone.
            self.text_box.needs_line_break = true;
            
            // If switching to single line mode, remove any existing newlines and set newline mode to None
            if single_line {
                self.newline_mode = NewlineMode::None;
            } else {
                // When switching back to multi-line, restore default newline mode
                self.newline_mode = NewlineMode::Enter;
            }
        }
    }

    /// Sets the newline entry mode for multi-line text edits.
    pub fn set_newline_mode(&mut self, mode: NewlineMode) {
        if !self.text_box.single_line {
            self.newline_mode = mode;
        }
    }

    /// Sets whether the text edit is disabled.
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    pub(crate) fn handle_event_editable(&mut self, event: &WindowEvent, window: &Window, input_state: &TextInputState) -> bool {
        if self.text_box.hidden() || self.disabled() {
            return false;
        }

        // Capture initial state for comparison
        let initial_selection = self.text_box.selection();
        let mut consumed = false;

        if ! self.showing_placeholder {
            consumed = self.text_box.handle_event_no_edit(event, input_state, true);
        }

        match event {
            WindowEvent::KeyboardInput { event, .. } if !self.is_composing() => {
                if !event.state.is_pressed() {
                    return consumed;
                }
                consumed = true;
                #[allow(unused)]
                let mods_state = input_state.modifiers.state();
                let shift = mods_state.shift_key();
                let action_mod = if cfg!(target_os = "macos") {
                    mods_state.super_key()
                } else {
                    mods_state.control_key()
                };

                // edit action mods
                if action_mod {
                    match event.key_without_modifiers() {
                        Key::Character(c) => {
                            match c.as_str() {
                                "x" if !shift => {
                                    with_clipboard(|cb| {
                                        if let Some(text) = self.text_box.selected_text() {
                                            cb.set_text(text.to_owned()).ok();
                                            self.delete_selection();
                                        }
                                    });
                                }
                                "v" if !shift => {
                                    if !self.text_box.shared().pasted_this_frame {
                                        self.text_box.shared_mut().pasted_this_frame = true;
                                        with_clipboard(|cb| {
                                            let text = cb.get_text().unwrap_or_default();
                                            self.insert_or_replace_selection(&text);
                                        });
                                    }
                                }
                                "z" => {
                                    if shift {
                                        self.redo();
                                    } else {
                                        self.undo();
                                    }
                                }
                                _ => (),
                            }
                        }
                        _ => (),
                    };
                }

                match &event.logical_key {
                    Key::Named(NamedKey::ArrowLeft) => {
                        if !shift && ! self.showing_placeholder {
                            if action_mod {
                                self.text_box.move_word_left();
                            } else {
                                self.text_box.move_left();
                            }
                        }
                    }
                    Key::Named(NamedKey::ArrowRight) => {
                        if !shift && ! self.showing_placeholder {
                            if action_mod {
                                self.text_box.move_word_right();
                            } else {
                                self.text_box.move_right();
                            }
                        }
                    }
                    Key::Named(NamedKey::ArrowUp) => {
                        if !shift && ! self.showing_placeholder {
                            if self.text_box.single_line {
                                self.text_box.move_to_text_start();
                            } else {
                                self.text_box.move_up();
                            }
                        }
                    }
                    Key::Named(NamedKey::ArrowDown) => {
                        if !shift && ! self.showing_placeholder {
                            if self.text_box.single_line {
                                self.text_box.move_to_text_end();
                            } else {
                                self.text_box.move_down();
                            }
                        }
                    }
                    Key::Named(NamedKey::Home) => {
                        if !shift && ! self.showing_placeholder {
                            if action_mod {
                                self.text_box.move_to_text_start();
                            } else {
                                self.text_box.move_to_line_start();
                            }
                        }
                    }
                    Key::Named(NamedKey::End) => {
                        if !shift && ! self.showing_placeholder {
                            if action_mod {
                                self.text_box.move_to_text_end();
                            } else {
                                self.text_box.move_to_line_end();
                            }
                        }
                    }
                    Key::Named(NamedKey::Delete) => {
                        if ! self.showing_placeholder {
                            if action_mod {
                                self.delete_word();
                            } else {
                                self.delete();
                            }
                        }
                    }
                    Key::Named(NamedKey::Backspace) => {
                        if ! self.showing_placeholder {
                            if action_mod {
                                self.backdelete_word();
                            } else {
                                self.backdelete();
                            }
                        }
                    }
                    Key::Named(NamedKey::Enter) => {
                        let newline_mode_matches = match self.newline_mode {
                            NewlineMode::Enter => !action_mod && !shift,
                            NewlineMode::ShiftEnter => shift && !action_mod,
                            NewlineMode::CtrlEnter => action_mod && !shift,
                            NewlineMode::None => false,
                        };
                        
                        if newline_mode_matches && ! self.text_box.single_line {
                            self.insert_or_replace_selection("\n");
                        }
                    }
                    Key::Named(NamedKey::Space) => {
                        if ! action_mod {
                            self.insert_or_replace_selection(" ");
                        }
                    }
                    Key::Character(s) => {
                        if ! action_mod {
                            self.insert_or_replace_selection(&s);
                        }
                    }
                    _ => (),
                }
            }
            WindowEvent::Touch(Touch {
                phase, location, ..
            }) if !self.is_composing() => {
                // todo, this is all wrong (should probably scroll), but nobody cares
                use winit::event::TouchPhase::*;
                if ! self.showing_placeholder {
                    consumed = true;
                    match phase {
                        Started => {
                            // Transform touch position to text box local space
                            let local_pos = self.text_box.cursor_to_local((location.x, location.y));
                            let cursor_pos = (
                                local_pos.x as f64 + self.text_box.scroll_offset.0 as f64,
                                local_pos.y as f64 + self.text_box.scroll_offset.1 as f64,
                            );
                            self.text_box.move_to_point(cursor_pos.0 as f32, cursor_pos.1 as f32);
                        }
                        Cancelled => {
                            self.text_box.collapse_selection();
                        }
                        Moved => {
                            // Transform touch position to text box local space
                            let local_pos = self.text_box.cursor_to_local((location.x, location.y));
                            self.text_box.extend_selection_to_point(
                                local_pos.x + self.text_box.scroll_offset.0,
                                local_pos.y + self.text_box.scroll_offset.1,
                            );
                        }
                        Ended => (),
                    }
                }
            }
            WindowEvent::Ime(Ime::Disabled) => {
                consumed = true;
                self.clear_compose();
            }
            WindowEvent::Ime(Ime::Commit(text)) => {
                consumed = true;
                if self.showing_placeholder {
                    self.clear_placeholder()
                }
                self.insert_or_replace_selection(&text);
            }
            WindowEvent::Ime(Ime::Preedit(text, cursor)) => {
                consumed = true;
                if self.showing_placeholder {
                    self.clear_placeholder()
                }
                if text.is_empty() {
                    self.clear_compose();
                } else {
                    self.set_compose(&text, *cursor);
                    self.set_ime_cursor_area(window);
                }
            }
            _ => {}
        }

        self.restore_placeholder_if_any();

        let select_rects_changed = selection_rects_changed(initial_selection, self.text_box.selection(), !self.disabled);

        // Mark that we need to update scroll before rendering.
        // All these functions that rely on a fresh layout are deferred before a real render, otherwise when events come in too fast they cause too many unneeded layout rebuilds.
        if select_rects_changed {
            self.needs_scroll_update = true;
        }

        consumed
    }

    // #[cfg(feature = "accesskit")]
    // pub(crate) fn handle_accesskit_action_request(&mut self, req: &accesskit::ActionRequest) {
    //     if req.action == accesskit::Action::SetTextSelection {
    //         if let Some(accesskit::ActionData::SetTextSelection(selection)) = &req.data {
    //             self.select_from_accesskit(selection);
    //         }
    //     }
    // }

    /// Insert at cursor, or replace selection.
    fn replace_range_and_record(&mut self, range: Range<usize>, old_selection: Selection, s: &str) {
        let old_text = &self.text_box.text_inner()[range.clone()];

        let new_range_start = range.start;
        let new_range_end = range.start + s.len();

        self.history
            .record(&old_text, s, old_selection, new_range_start..new_range_end);

        self.text_box.adjust_ranged_styles_for_edit(range.start, range.end, s.len());
        self.text_box.text_mut_string().replace_range(range, s);
        self.text_changed = true;

        if self.text_box.single_line {
            self.remove_newlines();
        }
    }

    fn replace_selection_and_record(&mut self, s: &str) {
        let old_selection = self.text_box.selection();

        let range = self.text_box.selection().text_range();
        let old_text = &self.text_box.text_inner()[range.clone()];

        let new_range_start = range.start;
        let new_range_end = range.start + s.len();

        self.history.record(&old_text, s, old_selection, new_range_start..new_range_end);

        self.replace_selection_inner(s);
    }

    /// Insert at cursor, or replace selection.
    pub(crate) fn insert_or_replace_selection(&mut self, s: &str) {
        assert!(!self.is_composing());

        self.clear_placeholder();

        self.replace_selection_and_record(s);
        self.text_changed = true;
    }

    /// Replaces the current selection with the given string.
    pub fn replace_selection(&mut self, string: &str) {
        if ! self.is_composing() {
            self.insert_or_replace_selection(string);
        }
    }

    pub(crate) fn clear_placeholder(&mut self) {
        clear_placeholder_partial_borrows!(self);
    }

    pub(crate) fn restore_placeholder_if_any(&mut self) {
        if self.text_box.text_inner().is_empty() && ! self.showing_placeholder {
            if self.placeholder_text.is_some() {
                self.text_box.text_mut_string().clear();
                self.refresh_layout();
                self.text_box.move_to_text_start();
            }

            if let Some(placeholder) = &self.placeholder_text {
                self.text_box.text_mut_string().push_str(&placeholder);
                self.showing_placeholder = true;
                self.refresh_layout();
            }
        }

        
    }

    /// Delete the selection.
    pub(crate) fn delete_selection(&mut self) {
        assert!(!self.is_composing());

        self.insert_or_replace_selection("");
    }

    /// Delete the selection or the next cluster (typical ‘delete’ behavior).
    pub(crate) fn delete(&mut self) {
        assert!(!self.is_composing());

        if self.text_box.selection().is_collapsed() {
            // Upstream cluster range
            if let Some(range) = self
                .text_box.selection()
                .focus()
                .logical_clusters(&self.text_box.layout())[1]
                .as_ref()
                .map(|cluster| cluster.text_range())
                .and_then(|range| (!range.is_empty()).then_some(range))
            {
                self.replace_range_and_record(range, self.text_box.selection(), "");
                self.refresh_layout();
            }
        } else {
            self.delete_selection();
        }
    }

    /// Delete the selection or up to the next word boundary (typical 'ctrl + delete' behavior).
    pub(crate) fn delete_word(&mut self) {
        assert!(!self.is_composing());

        if self.text_box.selection().is_collapsed() {
            let focus = self.text_box.selection().focus();
            let start = focus.index();
            let end = focus.next_logical_word(&self.text_box.layout()).index();
            if self.text_box.text_inner().get(start..end).is_some() {
                self.replace_range_and_record(start..end, self.text_box.selection(), "");
                self.refresh_layout();
                self.text_box.set_selection(
                    Cursor::from_byte_index(&self.text_box.layout, start, Affinity::Downstream).into(),
                );
            }
        } else {
            self.delete_selection();
        }
    }

    /// Delete the selection or the previous cluster (typical ‘backspace’ behavior).
    pub(crate) fn backdelete(&mut self) {
        assert!(!self.is_composing());

        if self.text_box.selection().is_collapsed() {
            // Upstream cluster
            if let Some(cluster) = self
                .text_box.selection()
                .focus()
                .logical_clusters(&self.text_box.layout())[0]
                .clone()
            {
                let range = cluster.text_range();
                let end = range.end;
                let start = if cluster.is_hard_line_break() || cluster.is_emoji() {
                    // For newline sequences and emoji, delete the previous cluster
                    range.start
                } else {
                    // Otherwise, delete the previous character
                    let Some((start, _)) = self
                        .text_box.text_inner()
                        .get(..end)
                        .and_then(|str| str.char_indices().next_back())
                    else {
                        return;
                    };
                    start
                };
                self.replace_range_and_record(start..end, self.text_box.selection(), "");
                self.refresh_layout();
                self.text_box.set_selection(
                    Cursor::from_byte_index(&self.text_box.layout, start, Affinity::Downstream).into(),
                );
            }
        } else {
            self.delete_selection();
        }
    }

    /// Delete the selection or back to the previous word boundary (typical 'ctrl + backspace' behavior).
    pub(crate) fn backdelete_word(&mut self) {
        assert!(!self.is_composing());

        if self.text_box.selection().is_collapsed() {
            let focus = self.text_box.selection().focus();
            let end = focus.index();
            let start = focus.previous_logical_word(&self.text_box.layout()).index();
            if self.text_box.text_inner().get(start..end).is_some() {
                self.replace_range_and_record(start..end, self.text_box.selection(), "");
                self.refresh_layout();
                self.text_box.set_selection(
                    Cursor::from_byte_index(&self.text_box.layout, start, Affinity::Downstream).into(),
                );
            }
        } else {
            self.delete_selection();
        }
    }

    /// Set the IME preedit composing text.
    ///
    /// This starts composing. Composing is reset by calling [`clear_compose`](Self::clear_compose).
    /// While composing, it is a logic error to call anything other than
    /// [`Self::set_compose()`] or [`Self::clear_compose()`].
    ///
    /// The preedit text replaces the current selection if this call starts composing.
    ///
    /// The selection is updated based on `cursor`, which contains the byte offsets relative to the
    /// start of the preedit text. If `cursor` is `None`, the selection and caret are hidden.
    pub(crate) fn set_compose(&mut self, text: &str, cursor: Option<(usize, usize)>) {
        debug_assert!(!text.is_empty());
        debug_assert!(cursor.map(|cursor| cursor.1 <= text.len()).unwrap_or(true));

        let start = if let Some(preedit_range) = &self.compose {
            let (ps, pe) = (preedit_range.start, preedit_range.end);
            self.text_box.adjust_ranged_styles_for_edit(ps, pe, text.len());
            self.text_box.text_mut_string().replace_range(ps..pe, text);
            ps
        } else {
            let selection_start = self.text_box.selection().text_range().start;
            if self.text_box.selection().is_collapsed() {
                self.text_box.adjust_ranged_styles_for_edit(selection_start, selection_start, text.len());
                self.text_box.text_mut_string()
                    .insert_str(selection_start, text);

                if self.text_box.single_line {
                    self.remove_newlines();
                }
            } else {
                let range = self.text_box.selection().text_range();
                self.text_box.adjust_ranged_styles_for_edit(range.start, range.end, text.len());
                self.text_box.text_mut_string()
                    .replace_range(range, text);
            }
            selection_start
        };
        self.compose = Some(start..start + text.len());
        self.text_box.shared_mut().cursor_blink_animation_currently_visible = cursor.is_some();

        // Select the location indicated by the IME. If `cursor` is none, collapse the selection to
        // a caret at the start of the preedit text.

        self.refresh_layout();

        let cursor = cursor.unwrap_or((0, 0));
        self.text_box.set_selection(Selection::new(
            // In parley, the layout is updated first, then the checked version is used. This should be fine too.
            Cursor::from_byte_index(&self.text_box.layout, start + cursor.0, Affinity::Downstream),
            Cursor::from_byte_index(&self.text_box.layout, start + cursor.1, Affinity::Downstream),
        ));

        self.text_box.needs_reshape = true;
    }

    /// Stop IME composing.
    ///
    /// This removes the IME preedit text.
    pub(crate) fn clear_compose(&mut self) {
        if let Some(preedit_range) = self.compose.take() {
            self.text_box.adjust_ranged_styles_for_edit(preedit_range.start, preedit_range.end, 0);
            self.text_box.text_mut_string().replace_range(preedit_range.clone(), "");
            self.text_box.shared_mut().cursor_blink_animation_currently_visible = true;

            let (index, affinity) = if preedit_range.start >= self.text_box.text_inner().len() {
                (self.text_box.text_inner().len(), Affinity::Upstream)
            } else {
                (preedit_range.start, Affinity::Downstream)
            };

            self.refresh_layout();
            self.text_box.selection = Cursor::from_byte_index(&self.text_box.layout, index, affinity).into();
        }
    }

    // #[cfg(feature = "accesskit")]
    // /// Select inside the editor based on the selection provided by accesskit.
    // pub(crate) fn select_from_accesskit(&mut self, selection: &accesskit::TextSelection) {
    //     assert!(!self.is_composing());

    //     self.refresh_layout();
    //     if let Some(selection) =
    //         Selection::from_access_selection(selection, &self.layout, &self.layout_access)
    //     {
    //         self.set_selection(selection);
    //     }
    // }

    // #[cfg(feature = "accesskit")]
    // /// Perform an accessibility update.
    // pub(crate) fn accessibility(
    //     &mut self,
    //     update: &mut TreeUpdate,
    //     node: &mut Node,
    //     next_node_id: impl FnMut() -> NodeId,
    //     x_offset: f64,
    //     y_offset: f64,
    // ) -> Option<()> {
    //     self.refresh_layout();
    //     self.accessibility_unchecked(update, node, next_node_id, x_offset, y_offset);
    //     Some(())
    // }

    pub(crate) fn undo(&mut self) {
        if self.is_composing() {
            return;
        }

        if let Some(op) = self.history.undo(self.text_box.text_mut_string()) {

            if ! op.text_to_restore.is_empty() {
                clear_placeholder_partial_borrows!(self);
            }

            // The two operations together are equivalent to replacing range_to_clear with text_to_restore.
            self.text_box.adjust_ranged_styles_for_edit(op.range_to_clear.start, op.range_to_clear.end, op.text_to_restore.len());
            self
                .text_box.text_mut_string()
                .replace_range(op.range_to_clear.clone(), "");
            self
                .text_box.text_mut_string()
                .insert_str(op.range_to_clear.start, op.text_to_restore);

            let prev_selection = op.prev_selection;
            self.text_box.set_selection(prev_selection);

            if self.text_box.single_line {
                self.remove_newlines();
            }
            self.text_changed = true;
        }

        self.restore_placeholder_if_any();
    }

    pub(crate) fn redo(&mut self) {
        if self.is_composing() {
            return;
        }

        if let Some(op) = self.history.redo() {
            // The two operations together are equivalent to replacing range_to_clear with text_to_restore.
            self.text_box.adjust_ranged_styles_for_edit(op.range_to_clear.start, op.range_to_clear.end, op.text_to_restore.len());
            self
                .text_box.text_mut_string()
                .replace_range(op.range_to_clear.clone(), "");

            if ! op.text_to_restore.is_empty() {
                clear_placeholder_partial_borrows!(self);
            }

            self
                .text_box.text_mut_string()
                .insert_str(op.range_to_clear.start, op.text_to_restore);

            let end = op.range_to_clear.start + op.text_to_restore.len();

            self.refresh_layout();
            self.text_box.selection = Cursor::from_byte_index(&self.text_box.layout, end, Affinity::Upstream).into();

            if self.text_box.single_line {
                self.remove_newlines();
            }
            self.text_changed = true;
        }

        self.restore_placeholder_if_any();
    }

    pub(crate) fn replace_selection_inner(&mut self, s: &str) {
        let range = self.text_box.selection().text_range();
        let start = range.start;
        if self.text_box.selection().is_collapsed() {
            self.text_box.adjust_ranged_styles_for_edit(start, start, s.len());
            self.text_box.text_mut_string().insert_str(start, s);

            if self.text_box.single_line {
                self.remove_newlines();
            }
        } else {
            self.text_box.adjust_ranged_styles_for_edit(range.start, range.end, s.len());
            self.text_box.text_mut_string().replace_range(range, s);

            if self.text_box.single_line {
                self.remove_newlines();
            }
        }

        let index = start.saturating_add(s.len());
        let affinity = if s.ends_with("\n") {
            Affinity::Downstream
        } else {
            Affinity::Upstream
        };

        // With the new setup, we can do refresh_layout here and use the checked from_byte_index functions. However, the check is still completely useless, all it does is turn a potential explicit panic into a silent failure.
        self.refresh_layout();
        self.text_box.selection = Cursor::from_byte_index(&self.text_box.layout, index, affinity).into();
    }

    /// Returns the layout, refreshing it if needed.
    pub fn layout(&mut self) -> &Layout<ColorBrush> {
        self.text_box.layout()
    }

    /// Sets the size of the text edit box.
    pub fn set_size(&mut self, size: (f32, f32)) {
        self.text_box.set_size(size)
    }

    /// Sets the width of the text edit box, which is also the width its lines wrap at.
    pub fn set_width(&mut self, width: f32) {
        self.text_box.set_width(width)
    }

    /// Sets the height of the text edit box, which doesn't affect the text layout.
    pub fn set_height(&mut self, height: f32) {
        self.text_box.set_height(height)
    }

    /// Returns the size of the text edit box.
    pub fn size(&self) -> (f32, f32) {
        self.text_box.size()
    }
}


#[derive(Clone, Debug)]
pub(crate) struct TextEditHistory {
    undo_text: String,
    redo_text: String,
    history: Vec<RecordedOp>,
    current_position: usize,
    can_grow: GrowHint,
}

#[derive(Clone, Copy, Debug)]
enum GrowHint {
    CannotGrow,
    GrowableInsert(usize),
    GrowableInsertWhitespace(usize),
    GrowableDelete(usize),
    GrowableDeleteWhitespace(usize),
}

#[derive(Debug, Clone)]
struct RecordedOp {
    /// Data needed to undo this history element.
    undo: Ranges,
    /// Data needed to redo this history element.
    /// To save memory, the redo data only gets populated when the element is undone.
    redo: Option<Ranges>,
    /// State of the selection right before this operation.
    prev_selection: Selection,
}

/// Internal Data for an undo or redo operation.
#[derive(Debug, Clone)]
struct Ranges {
    /// A range into the editor's main buffer for text that was inserted as part of a replace.
    inserted_range: Range<usize>,
    /// A range into the `TextEditHistory`'s internal buffer for text was deleted as part of a replace and stored.
    deleted_range: Range<usize>,
}

impl Ranges {
    fn is_delete_only(&self) -> bool {
        return self.inserted_range.is_empty();
    }
    fn is_insert_only(&self) -> bool {
        return self.deleted_range.is_empty();
    }
}

/// The result of undoing or redoing a text replace operation.
#[derive(Debug, Clone)]
struct TextRestore<'a> {
    /// A range into the original buffer that should be cleared.
    range_to_clear: Range<usize>,
    /// Text that should be inserted in the place of the cleared range.
    text_to_restore: &'a str,
    /// The state of selection right before the operation was made.
    /// Typically, undo operations restore the selection to this stored value,
    /// while redo operations ignore it and place a collapsed selection at the end of the newly restored text.
    prev_selection: Selection,
}

impl TextEditHistory {
    pub(crate) fn new() -> TextEditHistory {
        Self {
            undo_text: String::with_capacity(64),
            redo_text: String::with_capacity(64),
            history: Vec::with_capacity(64),
            current_position: 0,
            can_grow: GrowHint::CannotGrow,
        }
    }
}

trait StringBuffer {
    fn store_str(&mut self, text: &str) -> Range<usize>;
}
impl StringBuffer for String {
    fn store_str(&mut self, text: &str) -> Range<usize> {
        let start = self.len();
        self.push_str(text);
        start..self.len()
    }
}
trait WhitespaceStr {
    fn is_whitespace(&self) -> bool;
}
impl WhitespaceStr for &str {
    fn is_whitespace(&self) -> bool {
        self.chars().all(|c| c.is_whitespace() || c.is_ascii_punctuation())
    }
}

impl TextEditHistory {
    const MAX_GROWABLE_SIZE: usize = 20;

    #[rustfmt::skip]
    pub fn record(
        &mut self,
        old_str: &str,
        new_str: &str,
        selection: Selection,
        inserted_range: Range<usize>,
    ) {
        if self.current_position < self.history.len() {
            let undo_trunc = self.history[self.current_position].undo.deleted_range.start;
            self.undo_text.truncate(undo_trunc);
            self.redo_text.clear();
            self.history.truncate(self.current_position);
        }

        if let Some(last) = self.history.last_mut() {
            match self.can_grow {
                GrowHint::GrowableInsert(size) 
                    if old_str.is_empty() && size < Self::MAX_GROWABLE_SIZE =>
                        last.undo.inserted_range.end = inserted_range.end,

                GrowHint::GrowableInsertWhitespace(size) 
                    if old_str.is_empty() && new_str.is_whitespace() && size < Self::MAX_GROWABLE_SIZE =>
                        last.undo.inserted_range.end = inserted_range.end,

                GrowHint::GrowableDelete(size)
                    if inserted_range.is_empty() && size < Self::MAX_GROWABLE_SIZE =>
                        self.merge_delete(old_str, inserted_range),

                GrowHint::GrowableDeleteWhitespace(size)
                    if inserted_range.is_empty() && old_str.is_whitespace() && size < Self::MAX_GROWABLE_SIZE =>
                        self.merge_delete(old_str, inserted_range),

                _ => {
                    self.push_new(old_str, selection, inserted_range);
                },
            };
        } else {
            self.push_new(old_str, selection, inserted_range);
        }

        self.set_grow_hint(new_str, old_str);
    }

    pub fn push_new(&mut self, old_str: &str, selection: Selection, inserted_range: Range<usize>) {
        let undo_range = self.undo_text.store_str(old_str);

        self.history.push(RecordedOp {
            prev_selection: selection,
            undo: Ranges {
                inserted_range,
                deleted_range: undo_range,
            },
            redo: None,
        });

        self.current_position += 1;
    }

    fn merge_delete(&mut self, old_str: &str, inserted_range: Range<usize>) {
        let last = self.history.last_mut().unwrap();
        let start = last.undo.deleted_range.start;
        // To keep the text stored in the proper order, the old text has to be shifted.
        self.undo_text.insert_str(start, old_str);
        let end = self.undo_text.len();
        last.undo.deleted_range = start..end;
        last.undo.inserted_range = inserted_range.clone();
    }

    fn set_grow_hint(&mut self, new_str: &str, old_str: &str) {
        let last_op = &self.history.last().unwrap().undo;

        self.can_grow = if last_op.is_insert_only() {
            let len = new_str.len();
            match new_str.chars().last() {
                Some(c) if c.is_whitespace() => GrowHint::GrowableInsertWhitespace(len),
                Some(_) => GrowHint::GrowableInsert(len),
                None => GrowHint::CannotGrow,
            }
        } else if last_op.is_delete_only() {
            let len = old_str.len();
            match old_str.chars().last() {
                Some(c) if c.is_whitespace() => GrowHint::GrowableDeleteWhitespace(len),
                Some(_) => GrowHint::GrowableDelete(len),
                None => GrowHint::CannotGrow,
            }
        } else {
            GrowHint::CannotGrow
        };
    }

    fn undo(&mut self, buffer: &String) -> Option<TextRestore<'_>> {
        if self.current_position > 0 {
            self.current_position -= 1;
            let last = &mut self.history[self.current_position];

            // Prepare the undo to return
            let undo_text = last.undo.deleted_range.clone();
            let undo = TextRestore {
                prev_selection: last.prev_selection,
                range_to_clear: last.undo.inserted_range.clone(),
                text_to_restore: &self.undo_text[undo_text.clone()],
            };

            // Fill the last element with the data that will be needed for the redo
            if last.redo.is_none() {
                let redo_text = &buffer[undo.range_to_clear.clone()];
                let a = undo.range_to_clear.start;
                let redo_range = self.redo_text.store_str(redo_text);

                last.redo = Some(Ranges {
                    inserted_range: a..(a + undo_text.len()),
                    deleted_range: redo_range,
                });
            }
            // todo: if possible, put a nice prev_selection here so the caller doesn't have to think about it

            Some(undo)
        } else {
            None
        }
    }

    fn redo(&mut self) -> Option<TextRestore<'_>> {
        let last = self.history.get_mut(self.current_position)?;

        self.current_position += 1;

        let redo = last.redo.as_ref().unwrap().clone();
        let old_text = redo.deleted_range;

        Some(TextRestore {
            prev_selection: last.prev_selection,
            range_to_clear: redo.inserted_range,
            text_to_restore: &self.redo_text[old_text],
        })
    }
}

/// Replace newlines with spaces in-place. This probably doesn't allocate.
fn remove_newlines_inplace(text: &mut String) -> bool {
    let mut changed = false;
    for i in 0..text.len() {
        let b = text.as_bytes()[i];
        if b == b'\n' || b == b'\r' {
            text.replace_range(i..=i, " ");
            changed = true;
        }
    }

    return changed;
}

impl TextEdit {
    /// Returns a reference to the text edit style of the text edit box.
    pub fn text_edit_style(&self) -> &TextEditStyle {
        &self.text_box.shared().styles[self.text_box.style.key].text_edit_style
    }

    /// Returns `true` if the text edit is currently composing IME text.
    pub fn is_composing(&self) -> bool {
        self.compose.is_some()
    }

    /// Returns `true` if the text edit is in single-line mode.
    pub fn single_line(&self) -> bool {
        self.text_box.single_line
    }

    /// Returns the newline entry mode.
    pub fn newline_mode(&self) -> NewlineMode {
        self.newline_mode
    }

    /// Returns `true` if the text edit is disabled.
    pub fn disabled(&self) -> bool {
        self.disabled
    }

    /// Returns `true` if placeholder text is currently showing.
    pub fn showing_placeholder(&self) -> bool {
        self.showing_placeholder
    }

    /// Returns `true` if the text was changed by user input since the last frame.
    ///
    /// This flag is cleared at the start of each frame (in `prepare_all`).
    pub fn text_changed(&self) -> bool {
        self.text_changed
    }

    /// Returns the next time the cursor should blink.
    pub fn next_blink_time(&self) -> Option<Instant> {
        self.start_time.map(|start_time| {
            let phase = Instant::now().duration_since(start_time);

            start_time
                + Duration::from_nanos(
                    ((phase.as_nanos() / self.blink_period.as_nanos() + 1)
                        * self.blink_period.as_nanos()) as u64,
                )
        })
    }

    /// Returns the text content. Returns an empty string if placeholder is showing.
    pub fn raw_text(&self) -> &str {
        if self.showing_placeholder {
            ""
        } else {
            self.text_box.text()
        }
    }

    /// Returns the placeholder text, if any.
    pub fn placeholder(&self) -> Option<&str> {
        self.placeholder_text.as_deref()
    }

    #[cfg(feature = "accessibility")]
    /// Fills `parent_node` with `TextRun` children (for the screen-reader text
    /// pattern) and sets the caret/selection on it. Mirrors
    /// [`TextBox::build_accesskit_nodes`], adding the selection that an editable
    /// field needs.
    pub fn build_accesskit_nodes(
        &mut self,
        parent_node: &mut accesskit::Node,
        out: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
        next_node_id: impl FnMut() -> accesskit::NodeId,
    ) {
        self.text_box.build_accesskit_nodes(parent_node, out, next_node_id);
        if let Some(selection) = self
            .text_box
            .selection
            .to_access_selection(&self.text_box.layout, &self.text_box.layout_access)
        {
            parent_node.set_text_selection(selection);
        }
    }
    
    /// Returns the currently selected text, if any. Returns `None` if placeholder is showing.
    pub fn selected_text(&self) -> Option<&str> {
        if self.showing_placeholder {
            None
        } else {
            self.text_box.selected_text()
        }
    }

    /// Returns the range of the currently selected text.
    /// 
    /// If the text in the text box changes, the result will become invalid, and might not even point
    /// to a valid UTF-8 substring of the new text anymore.
    pub fn selected_text_range(&self) -> Option<std::ops::Range<usize>> {
        if self.showing_placeholder {
            None
        } else {
            self.text_box.selected_text_range()
        }
    }
    
    /// Returns the position of the text edit box.
    pub fn pos(&self) -> (f32, f32) {
        self.text_box.position()
    }
    
    /// Returns `true` if the text edit box is hidden.
    pub fn hidden(&self) -> bool {
        self.text_box.hidden()
    }
    
    /// Returns the depth (z-order) of the text edit box.
    pub fn depth(&self) -> f32 {
        self.text_box.depth()
    }

    /// Returns `true` if automatic clipping is enabled.
    pub fn auto_clip(&self) -> bool {
        self.text_box.auto_clip
    }
    
    /// Returns the scroll offset.
    pub fn scroll_offset(&self) -> (f32, f32) {
        self.text_box.scroll_offset()
    }

    /// Returns the current text selection.
    pub fn selection(&self) -> Selection {
        self.text_box.selection()
    }

    /// Returns the glyph quad ranges for this text edit.
    pub fn glyph_quad_range(&self) -> (usize, usize) {
        self.text_box.glyph_quad_range()
    }
}

impl TextEdit {

    /// Returns a mutable reference to the text content.
    pub fn raw_text_mut(&mut self) -> &mut String {
        // clear the placeholder so that the user can edit the text. If the text is empty after the modifications, the placeholder text should get restored anyway at the next relayout.
        if self.showing_placeholder {
            self.clear_placeholder();
        }
        self.text_changed = true;
        self.text_box.text_mut_string()
    }

    /// Set a custom tag for this text box.
    pub fn set_custom_tag(&mut self, custom_tag: Option<u64>) {
        self.text_box.set_custom_tag(custom_tag);
    }

    /// Get the custom tag set to this text box, if any. 
    pub fn custom_tag(&mut self) -> Option<u64> {
        self.text_box.custom_tag()
    }

    /// Sets the position of the text edit box.
    pub fn set_pos(&mut self, pos: (f64, f64)) {
        self.text_box.set_pos(pos);
    }

    /// Sets whether the text edit box is hidden.
    pub fn set_hidden(&mut self, hidden: bool) {
        self.text_box.set_hidden(hidden);
    }
    
    /// Sets the depth (z-order) of the text edit box.
    pub fn set_depth(&mut self, value: f32) {
        self.text_box.set_depth(value);
    }

    /// Sets the opacity multiplier of the text edit box, multiplied into every glyph's alpha.
    pub fn set_opacity(&mut self, value: f32) {
        self.text_box.set_opacity(value);
    }

    /// Sets the screen-space clipping rectangle for the text edit box.
    pub fn set_clip_rect(&mut self, clip_rect: Option<parley::BoundingBox>) {
        self.text_box.set_clip_rect(clip_rect);
    }

    /// Sets an explicit hitbox for hit detection in local space (min_x, min_y, max_x, max_y).
    ///
    /// When set, hit detection will use this hitbox instead of computing one from
    /// the text edit box dimensions.
    pub fn set_hitbox(&mut self, hitbox: Option<(f32, f32, f32, f32)>) {
        self.text_box.set_hitbox(hitbox);
    }

    /// Returns the explicit hitbox if set.
    pub fn hitbox(&self) -> Option<(f32, f32, f32, f32)> {
        self.text_box.hitbox()
    }

    /// Sets the scroll offset for the text edit box.
    pub fn set_scroll_offset(&mut self, offset: (f32, f32)) {
        self.text_box.set_scroll_offset(offset);
    }
    
    /// Apply horizontal scroll with bounds checking and precision handling
    /// Returns true if scroll offset was changed
    fn apply_horizontal_scroll(&mut self, new_scroll: f32) -> bool {
        let old_scroll = self.text_box.scroll_offset.0;
        let total_text_width = self.text_box.layout.full_width();
        let text_width = self.text_box.width;
        let max_scroll = (total_text_width - text_width).max(0.0).round() + CURSOR_WIDTH;
        let clamped_scroll = new_scroll.clamp(0.0, max_scroll).round();
        
        if clamped_scroll != old_scroll {
            self.text_box.scroll_offset.0 = clamped_scroll;
            true
        } else {
            false
        }
    }

    /// Updates scroll offset to ensure cursor is visible.
    pub fn update_scroll_to_cursor(&mut self) -> bool {
        if let Some(cursor_rect) = self.cursor_geometry(1.0) {
            if self.text_box.single_line {
                // Horizontal scrolling for single-line edits
                let text_width = self.text_box.width;
                let cursor_left = cursor_rect.x0 as f32;
                let cursor_right = cursor_rect.x1 as f32;
                let current_scroll = self.text_box.scroll_offset().0;

                let visible_start = current_scroll;
                let visible_end = current_scroll + text_width;                
                if cursor_left < visible_start {
                    // Cursor left is too far left, scroll to show cursor fully at left edge
                    return self.apply_horizontal_scroll((cursor_left - CURSOR_WIDTH).max(0.0));
                } else if cursor_right > visible_end {
                    // Cursor right is too far right, scroll to show cursor fully at right edge
                    return self.apply_horizontal_scroll(CURSOR_WIDTH + cursor_right - text_width);
                }
            } else {
                // Vertical scrolling for multi-line edits
                let text_height = self.text_box.height;
                let cursor_top = cursor_rect.y0 as f32;
                let cursor_bottom = cursor_rect.y1 as f32;
                let current_scroll = self.text_box.scroll_offset().1;
                
                // Get the total text height to check if we're overflowing
                let total_text_height = self.text_box.layout.height();
                
                // Calculate visible range
                let visible_start = current_scroll;
                let visible_end = current_scroll + text_height;
                
                // Margin for cursor visibility - small buffer zone
                let margin = text_height * 0.05; // 5% margin
                
                // Check if cursor is outside visible range
                if cursor_top < visible_start + margin {
                    // Cursor top is too far up, scroll up
                    let new_scroll = (cursor_top - margin).max(0.0).round();
                    if (new_scroll - current_scroll).abs() > 0.5 {
                        self.text_box.set_scroll_offset((0.0, new_scroll));
                        return true;
                    }
                } else if cursor_bottom > visible_end - margin {
                    // Cursor bottom is too far down, scroll down
                    let new_scroll = cursor_bottom - text_height + margin;
                    let max_scroll = (total_text_height - text_height).max(0.0).round();
                    let new_scroll = new_scroll.min(max_scroll).round();
                    if (new_scroll - current_scroll).abs() > 0.5 {
                        self.text_box.set_scroll_offset((0.0, new_scroll));
                        return true;
                    }
                }
            }
        }
        
        false
    }
    
    /// Sets the style for the text edit box.
    ///
    /// This function will only trigger a relayout if the new style is different from the old one.
    pub fn set_style(&mut self, style: &StyleHandle) {
        self.text_box.set_style(style);
    }

    /// Adds a [`StyleProperty`] override for the given byte range of the text.
    ///
    /// See [`TextBox::push_ranged_style_property`] for details.
    pub fn push_style_property(&mut self, prop: StyleProperty<'static, ColorBrush>, range: std::ops::Range<usize>) {
        self.text_box.push_ranged_style_property(prop, range);
    }

    /// Sets the whole-box [`StyleProperty`] overrides, replacing any previously set overrides.
    ///
    /// This method won't cause a relayout unless the new properties are different than the previous ones.
    pub fn set_style_property_overrides(&mut self, props: &[StyleProperty<'static, ColorBrush>]) {
        self.text_box.set_style_property_overrides(props);
    }

    /// Sets the text alignment.
    ///
    /// This function will only trigger a relayout if the new alignment is different from the old one.
    pub fn set_alignment(&mut self, alignment: Alignment) {
        self.text_box.set_alignment(alignment);
    }

    /// Clears all per-range style property overrides.
    ///
    /// See [`TextBox::clear_ranged_style_properties`] for details.
    pub fn clear_style_properties(&mut self) {
        self.text_box.clear_ranged_style_properties();
    }

    /// Clears style property in a range.
    pub fn clear_style_properties_in_range(&mut self, range_to_clear: std::ops::Range<usize>) {
        self.text_box.clear_style_properties_in_range(range_to_clear);
    }

    /// Returns the cursor geometry if visible.
    pub fn cursor_geometry(&mut self, size: f32) -> Option<parley::BoundingBox> {
        if ! self.text_box.shared_mut().cursor_blink_animation_currently_visible {
            return None;
        }
        
        self.refresh_layout();
        Some(self.text_box.selection().focus().geometry(&self.text_box.layout, size))
    }

    /// Refresh the text layout if needed.
    pub fn refresh_layout(&mut self) {
        let color_override = if self.disabled {
            Some(self.text_edit_style().disabled_text_color)
        } else if self.showing_placeholder {
            Some(self.text_edit_style().placeholder_text_color)
        } else {
            None
        };

        self.text_box.set_color_override(color_override);
        self.text_box.refresh_layout();
    }

    /// Set the text of the text edit box.
    pub fn set_text(&mut self, new_text: &str) {
        self.text_box.set_text(new_text);
        self.text_box.move_to_text_end();
        // Clear any composition state
        self.compose = None;
        self.text_changed = true;
        self.restore_placeholder_if_any();
    }

    /// Set the text of the text edit box, using a hash to detect if unchanged.
    ///
    /// If called again with the same text, it will detect unchanged and skip work.
    /// Useful for declarative APIs.
    pub fn set_text_hashed(&mut self, new_text: &str) {
        if self.showing_placeholder && new_text == "" {
            return;
        }
        let new_identity = TextIdentity::Hash(hash_text(new_text));
        if self.text_box.text_identity == Some(new_identity) {
            return;
        }
        self.text_box.set_text_hashed(new_text);
        self.text_box.move_to_text_end();
        self.compose = None;
        self.text_changed = true;
        self.restore_placeholder_if_any();
    }

    /// Sets the text of the text edit box, using pointer identity to detect if unchanged.
    ///
    /// Only use when the string reference is stable (e.g. from an immutable source).
    /// If called again with the same pointer, it will skip work.
    pub fn set_text_with_pointer_check(&mut self, new_text: &str) {
        if self.showing_placeholder && new_text == "" {
            return;
        }
        let new_identity = TextIdentity::Pointer(new_text as *const str);
        if self.text_box.text_identity == Some(new_identity) {
            return;
        }
        self.text_box.set_text_with_pointer_check(new_text);
        self.text_box.move_to_text_end();
        self.compose = None;
        self.text_changed = true;
        self.restore_placeholder_if_any();
    }

    /// Sets the text to a static string reference.
    pub fn set_static_text(&mut self, text: &'static str) {
        if self.showing_placeholder && text == "" {
            return;
        }
        self.text_box.set_static_text(text);
        self.text_box.move_to_text_end();
        self.compose = None;
        self.text_changed = true;
        self.restore_placeholder_if_any();
    }

    /// Sets the text to a static string reference, using pointer identity to detect if unchanged.
    ///
    /// If called again with the same static string, it will skip work.
    /// Useful for declarative APIs.
    pub fn set_static_text_with_pointer_check(&mut self, text: &'static str) {
        if self.showing_placeholder && text == "" {
            return;
        }
        let new_identity = TextIdentity::Pointer(text as *const str);
        if self.text_box.text_identity == Some(new_identity) {
            return;
        }
        self.text_box.set_static_text_with_pointer_check(text);
        self.text_box.move_to_text_end();
        self.compose = None;
        self.text_changed = true;
        self.restore_placeholder_if_any();
    }

    /// Set placeholder text that will be shown when the text edit is empty.
    pub fn set_placeholder(&mut self, placeholder: &str) {
        self.placeholder_text_identity = None;
        // Store placeholder, reusing existing buffer if possible
        match &mut self.placeholder_text {
            Some(Cow::Owned(s)) => {
                s.clear();
                s.push_str(placeholder);
            }
            _ => {
                self.placeholder_text = Some(Cow::Owned(placeholder.to_string()));
            }
        }
        self.reapply_placeholder_if_needed();
    }

    /// Set placeholder text, using a hash to detect if unchanged.
    ///
    /// If called again with the same text, it will detect unchanged and skip work.
    /// Useful for declarative APIs.
    pub fn set_placeholder_hashed(&mut self, placeholder: &str) {
        let new_identity = TextIdentity::Hash(hash_text(placeholder));
        if self.placeholder_text_identity == Some(new_identity) {
            return;
        }
        self.placeholder_text_identity = Some(new_identity);
        match &mut self.placeholder_text {
            Some(Cow::Owned(s)) => {
                s.clear();
                s.push_str(placeholder);
            }
            _ => {
                self.placeholder_text = Some(Cow::Owned(placeholder.to_string()));
            }
        }
        self.reapply_placeholder_if_needed();
    }

    /// Set placeholder text, using pointer identity to detect if unchanged.
    ///
    /// Only use when the string reference is stable (e.g. from an immutable source).
    /// If called again with the same pointer, it will skip work.
    pub fn set_placeholder_with_pointer_check(&mut self, placeholder: &str) {
        let new_identity = TextIdentity::Pointer(placeholder as *const str);
        if self.placeholder_text_identity == Some(new_identity) {
            return;
        }
        self.placeholder_text_identity = Some(new_identity);
        match &mut self.placeholder_text {
            Some(Cow::Owned(s)) => {
                s.clear();
                s.push_str(placeholder);
            }
            _ => {
                self.placeholder_text = Some(Cow::Owned(placeholder.to_string()));
            }
        }
        self.reapply_placeholder_if_needed();
    }

    /// Set placeholder text to a static string reference.
    pub fn set_placeholder_static(&mut self, placeholder: &'static str) {
        self.placeholder_text_identity = None;
        self.placeholder_text = Some(Cow::Borrowed(placeholder));
        self.reapply_placeholder_if_needed();
    }

    /// Set placeholder text to a static string, using pointer identity to detect if unchanged.
    ///
    /// If called again with the same static string, it will skip work.
    /// Useful for declarative APIs.
    pub fn set_placeholder_static_with_pointer_check(&mut self, placeholder: &'static str) {
        let new_identity = TextIdentity::Pointer(placeholder as *const str);
        if self.placeholder_text_identity == Some(new_identity) {
            return;
        }
        self.placeholder_text_identity = Some(new_identity);
        self.placeholder_text = Some(Cow::Borrowed(placeholder));
        self.reapply_placeholder_if_needed();
    }

    fn reapply_placeholder_if_needed(&mut self) {
        if self.text_box.text_inner().is_empty() || self.showing_placeholder {
            if let Some(placeholder) = &self.placeholder_text {
                self.text_box.set_text(placeholder);
                self.showing_placeholder = true;
                self.text_box.reset_selection();
            }
        }
    }

    // todo: we could also pass a range to check only the newly inserted part.
    fn remove_newlines(&mut self) {
        let removed = remove_newlines_inplace(self.text_box.text_mut_string());
        if removed {
            self.text_box.needs_reshape = true;
        }
    }

    /// Sets the transform of the text edit box.
    pub fn set_transform(&mut self, transform: Transform2D) {
        self.text_box.set_transform(transform);
    }

    /// Sets the text edit box to use a group transform in addition to its own one
    pub fn set_group_transform(&mut self, transform: GroupTransformHandle) {
        self.text_box.group_transform_index = Some(transform);
    }

    /// Returns the current transform of the text box.
    pub fn transform(&self) -> Transform2D {
        self.text_box.transform()
    }

    /// Sets the IME cursor area for this text edit.
    pub fn set_ime_cursor_area(&mut self, window: &Window) {
        if let Some(area) = self.cursor_geometry(1.0) {
            // Note: on X11 `set_ime_cursor_area` may cause the exclusion area to be obscured
            // until https://github.com/rust-windowing/winit/pull/3966 is in the Winit release
            // used by this example.
            // Transform the IME cursor area to screen space
            let screen_pos = self.text_box.transform().transform_point(euclid::Point2D::new(area.x0 as f32, area.y0 as f32));
            window.set_ime_cursor_area(
                winit::dpi::PhysicalPosition::new(
                    screen_pos.x as f64,
                    screen_pos.y as f64,
                ),
                winit::dpi::PhysicalSize::new(area.width(), area.height()),
            );
        }
    }

    /// Sets focus to this text edit.
    pub fn set_focus(&mut self) {
        let k = AnyBox::TextEdit(self.text_box.key);
        self.text_box.shared_mut().refocus(Some(k));
    }
}

/// Determine if animation should be used based on delta type and which component is being used
pub(crate) fn should_use_animation(delta: &winit::event::MouseScrollDelta, vertical: bool) -> bool {
    match delta {
        // can't find a good way to tell apart touchpad and mouse wheel. They both show up as LineDelta.
        winit::event::MouseScrollDelta::LineDelta(x, y) => {
            if vertical {
                y.abs().fract() == 0.0
            } else {
                x.abs().fract() == 0.0
            }
        },
        winit::event::MouseScrollDelta::PixelDelta(_) => false,
    }
}

