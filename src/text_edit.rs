use std::cmp;

use arboard::Clipboard;
use glam::Vec2;
use glyphon::{cosmic_text::{BorrowedWithFontSystem, Motion, Selection}, Action, Affinity, Cursor, Edit};
use unicode_segmentation::UnicodeSegmentation;
use winit::{event::{ElementState, KeyEvent, MouseButton, WindowEvent}, keyboard::{Key, ModifiersState, NamedKey}};

use crate::*;

/// Represents the result of handling an editor event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditorEventResult {
    /// Whether the event was absorbed by the editor
    pub absorbed: bool,
    /// Whether the cursor/selection decorations need to be redrawn
    pub redraw_cursor: bool,
    /// Whether the text content needs to be redrawn
    pub redraw_text: bool,
}

const IGNORED: EditorEventResult = EditorEventResult {
    absorbed: false,
    redraw_cursor: false,
    redraw_text: false,
};

const ABSORBED_BUT_NOTHING_CHANGED: EditorEventResult = EditorEventResult {
    absorbed: true,
    redraw_cursor: false,
    redraw_text: false,
};

const CURSOR_CHANGED: EditorEventResult = EditorEventResult {
    absorbed: true,
    redraw_cursor: true,
    redraw_text: false,
};

const TEXT_CHANGED: EditorEventResult = EditorEventResult {
    absorbed: true,
    redraw_cursor: true,
    redraw_text: true,
};

pub(crate) fn delete_selection_and_record<'buffer>(
    editor: &mut BorrowedWithFontSystem<impl Edit<'buffer>>,
    history: &mut TextEditHistory
) {
    let Some(selected_text) = editor.copy_selection() else {
        return;
    };
    
    editor.delete_selection();
    history.record_delete(&selected_text, editor.cursor());
}

pub(crate) fn delete_selection_and_record_special<'buffer>(
    editor: &mut BorrowedWithFontSystem<impl Edit<'buffer>>,
    history: &mut TextEditHistory,
    cursor: Cursor
) {
    let selected_text = editor.copy_selection();
    let deleted_text = match &selected_text {
        Some(text) => text.as_str(),
        None => &"",
    };
    history.record_delete(deleted_text, cursor);
    editor.delete_selection();
}

pub(crate) fn editor_window_event<'buffer>(
    editor: &mut BorrowedWithFontSystem<impl Edit<'buffer>>,
    history: &mut TextEditHistory,
    editor_rect_top_left: Vec2,
    event: &WindowEvent,
    modifiers: &ModifiersState,
    mouse_left_pressed: bool,
    mouse_x: f64,
    mouse_y: f64,
    clipboard: &mut Clipboard,
) -> EditorEventResult {
    match event {
        WindowEvent::KeyboardInput { event, .. } => {
            let KeyEvent {
                logical_key, state, ..
            } = event;

            if state.is_pressed() {
                match logical_key {
                    Key::Named(NamedKey::ArrowLeft) => {
                        if modifiers.shift_key() {
                            let cursor = editor.cursor();
                            if editor.selection() == Selection::None {
                                editor.set_selection(Selection::Normal(cursor));
                            }
                            if modifiers.control_key() {
                                editor.action(Action::Motion(Motion::PreviousWord));
                            } else {
                                editor.action(Action::Motion(Motion::Left));
                            }
                        } else if let Some((start, _)) = editor.selection_bounds() {
                            editor.set_cursor(start);
                            editor.set_selection(Selection::None);
                        } else {
                            if modifiers.control_key() {
                                editor.action(Action::Motion(Motion::PreviousWord));
                            } else {
                                editor.action(Action::Motion(Motion::Left));
                            }
                        }
                        return CURSOR_CHANGED;
                    }
                    Key::Named(NamedKey::ArrowRight) => {
                        if modifiers.shift_key() {
                            let cursor = editor.cursor();
                            if editor.selection() == Selection::None {
                                editor.set_selection(Selection::Normal(cursor));
                            }
                            if modifiers.control_key() {
                                editor.action(Action::Motion(Motion::NextWord));
                            } else {
                                editor.action(Action::Motion(Motion::Right));
                            }
                        } else if let Some((_, end)) = editor.selection_bounds() {
                            editor.set_cursor(end);
                            editor.set_selection(Selection::None);
                        } else {
                            if modifiers.control_key() {
                                editor.action(Action::Motion(Motion::NextWord));
                            } else {
                                editor.action(Action::Motion(Motion::Right));
                            }
                        }
                        return CURSOR_CHANGED;
                    }
                    Key::Named(NamedKey::ArrowUp) => {
                        if modifiers.shift_key() {
                            let cursor = editor.cursor();
                            if editor.selection() == Selection::None {
                                editor.set_selection(Selection::Normal(cursor));
                            }
                        } else {
                            editor.set_selection(Selection::None);
                        }
                        if editor.cursor().line == 0 {
                            editor.set_cursor(Cursor { line: 0, index: 0, affinity: Affinity::Before});
                        } else {
                            editor.action(Action::Motion(Motion::Up));
                        }
                        return CURSOR_CHANGED;
                    }
                    Key::Named(NamedKey::ArrowDown) => {
                        if modifiers.shift_key() {
                            let cursor = editor.cursor();
                            if editor.selection() == Selection::None {
                                editor.set_selection(Selection::Normal(cursor));
                            }
                        } else {
                            editor.set_selection(Selection::None);
                        }
                        let last_line = editor.with_buffer(|buffer| buffer.lines.len() - 1);
                        if editor.cursor().line == last_line {
                            let last_index = editor.with_buffer(|buffer| buffer.lines[last_line].text().chars().count());
                            editor.set_cursor(Cursor { line: last_line, index: last_index, affinity: Affinity::After });
                        } else {
                            editor.action(Action::Motion(Motion::Down));
                        }
                        return CURSOR_CHANGED;
                    }
                    Key::Named(NamedKey::Home) => {
                        if modifiers.shift_key() {
                            let cursor = editor.cursor();
                            if editor.selection() == Selection::None {
                                editor.set_selection(Selection::Normal(cursor));
                            }
                        } else {
                            editor.set_selection(Selection::None);
                        }
                        editor.action(Action::Motion(Motion::Home));
                        return CURSOR_CHANGED;
                    }
                    Key::Named(NamedKey::End) => {
                        if modifiers.shift_key() {
                            let cursor = editor.cursor();
                            if editor.selection() == Selection::None {
                                editor.set_selection(Selection::Normal(cursor));
                            }
                        } else {
                            editor.set_selection(Selection::None);
                        }
                        editor.action(Action::Motion(Motion::End));
                        return CURSOR_CHANGED;
                    }
                    Key::Named(NamedKey::PageUp) => {
                        editor.action(Action::Motion(Motion::PageUp));
                        return CURSOR_CHANGED;
                    }
                    Key::Named(NamedKey::PageDown) => {
                        editor.action(Action::Motion(Motion::PageDown));
                        return CURSOR_CHANGED;
                    }
                    Key::Named(NamedKey::Escape) => {
                        editor.action(Action::Escape);
                        return CURSOR_CHANGED;
                    }
                    Key::Named(NamedKey::Enter) => {
                        // ctrl + enter isn't even listened
                        if ! modifiers.control_key() {
                            if editor.selection() != Selection::None {
                                delete_selection_and_record(editor, history);
                            } else {
                                delete_selection_and_record(editor, history);
                                editor.action(Action::Insert('\n'));
                                history.record_insert('\n', editor.cursor());
                            }
                            return TEXT_CHANGED;
                        }
                    }
                    Key::Named(NamedKey::Backspace) => {
                        if editor.selection() != Selection::None {
                            delete_selection_and_record(editor, history);
                            return TEXT_CHANGED;
                        }
                        if modifiers.control_key() {
                            let cursor = editor.cursor();
                            editor.set_selection(Selection::Normal(cursor));
                            editor.action(Action::Motion(Motion::PreviousWord));
                            delete_selection_and_record(editor, history);
                            editor.set_selection(Selection::None);
                        } else {
                            let cursor = editor.cursor();
                            editor.set_selection(Selection::Normal(cursor));
                            editor.action(Action::Motion(Motion::Previous));
                            delete_selection_and_record(editor, history);
                            editor.set_selection(Selection::None);
                        }
                        return TEXT_CHANGED;
                    }
                    Key::Named(NamedKey::Delete) => {
                        if editor.selection() != Selection::None {
                            delete_selection_and_record(editor, history);
                            return TEXT_CHANGED;
                        }
                        if modifiers.control_key() {
                            let old_cursor = editor.cursor();
                            editor.set_selection(Selection::Normal(old_cursor));
                            editor.action(Action::Motion(Motion::NextWord));

                            delete_selection_and_record(editor, history);
                            editor.set_selection(Selection::None);

                        } else {
                            let old_cursor = editor.cursor();
                            editor.set_selection(Selection::Normal(old_cursor));
                            editor.action(Action::Motion(Motion::Next));

                            delete_selection_and_record(editor, history);
                            editor.set_selection(Selection::None);
                        }
                        return TEXT_CHANGED;
                    }
                    Key::Named(key) => {
                        if ! modifiers.control_key() {
                            if let Some(text) = key.to_text() {
                                for c in text.chars() {
                                    editor.action(Action::Insert(c));
                                    history.record_insert(c, editor.cursor());
                                }
                                return TEXT_CHANGED;
                            }
                        }
                    }
                    Key::Character(text) => {
                        if modifiers.control_key() {
                            match text.as_str() {
                                "z" => {
                                    if let Some(op) = history.undo() {
                                        match op {
                                            Undo::UndoInsert(undo_insert) => {
                                                editor.set_cursor(undo_insert.cursor);
                                                editor.action(Action::Backspace);
                                                return TEXT_CHANGED;
                                            },
                                            Undo::UndoDelete(undo_delete) => {
                                                let new_cursor = editor.insert_at(undo_delete.cursor, undo_delete.text, None);
                                                editor.set_cursor(new_cursor);
                                                return TEXT_CHANGED;
                                            },
                                        }
                                    }
                                }
                                "a" => {
                                    editor.set_cursor(Cursor::new_with_affinity(0, 0, Affinity::Before));
                                    let end_line = editor.with_buffer(|buffer| buffer.lines.len() - 1);
                                    let end_col = editor.with_buffer(|buffer| buffer.lines[end_line].text().len());
                                    editor.set_selection(Selection::Normal(Cursor::new_with_affinity(end_line, end_col, Affinity::After)));
                                    return CURSOR_CHANGED;
                                }
                                "c" => {
                                    if let Some(text) = editor.copy_selection() {                                        
                                        if let Err(err) = clipboard.set_text(text) {
                                            log::error!("Failed to copy text to clipboard: {}", err);
                                        }
                                    }
                                    return ABSORBED_BUT_NOTHING_CHANGED;
                                }
                                "v" => {
                                    // Paste text from clipboard
                                    if let Ok(text) = clipboard.get_text() {
                                        // Delete any selected text first
                                        delete_selection_and_record(editor, history);
                                        
                                        let cursor = editor.cursor();
                                        let new_cursor = editor.insert_at(cursor, &text, None);
                                        editor.set_cursor(new_cursor);
                                    }
                                    return TEXT_CHANGED;
                                }
                                _ => {},
                            }
                        } else {
                            for c in text.chars() {
                                delete_selection_and_record(editor, history);
                                editor.action(Action::Insert(c));
                                history.record_insert(c, editor.cursor());
                            }
                            return TEXT_CHANGED;
                        }
                    }
                    _ => {},
                }
            }
        }
        WindowEvent::CursorMoved {
            device_id: _,
            position,
        } => {
            // Implement dragging
            if mouse_left_pressed {
                // Execute Drag editor action (update selection)
                editor.action(Action::Drag {
                    x: position.x as i32 - editor_rect_top_left.x as i32,
                    y: position.y as i32 - editor_rect_top_left.y as i32,
                });
                return CURSOR_CHANGED;
            }
        }
        WindowEvent::MouseInput {
            device_id: _,
            state,
            button,
        } => {
            if *button == MouseButton::Left {
                if *state == ElementState::Pressed {
                    editor.action(Action::Click {
                        x: mouse_x as i32 - editor_rect_top_left.x as i32,
                        y: mouse_y as i32 - editor_rect_top_left.y as i32,
                    });
                    return CURSOR_CHANGED;
                }
            }
        }
        _ => {},
    }
    return IGNORED;
}

impl Ui {
    pub(crate) fn push_focused_editor_decorations(&mut self) -> Option<()> {
        let id = self.sys.focused?;
        let node_i = self.nodes.node_hashmap.get(&id)?.slab_i;

        let Some(TextI::TextEditI(edit_i)) = self.nodes[node_i].text_i else {
            return None
        };

        // todo: skip the reborrowing and rehashing
        let editor = &self.sys.text.slabs.editors.get(edit_i)?.editor;
        match editor.selection() {
            Selection::None => self.push_cursor_rect(),
            Selection::Normal(_) => self.push_selection_rects(),
            Selection::Line(_) => self.push_selection_rects(),
            Selection::Word(_) => self.push_selection_rects(),
        };
    
        Some(())
    }

    pub(crate) fn push_cursor_rect(&mut self) -> Option<()> {
        let id = self.sys.focused?;
        let node_i = self.nodes.node_hashmap.get(&id)?.slab_i;

        let Some(TextI::TextEditI(edit_i)) = self.nodes[node_i].text_i else {
            return None
        };

        // todo: get the one from the actual line
        let editor = &self.sys.text.slabs.editors.get(edit_i)?.editor;
        let mut line_height = 10.0;
        for run in editor.buffer().layout_runs() {
            line_height = run.line_height;
            break;
        }

        const CURSOR_WIDTH: f32 = 2.5;
        let size = self.sys.unifs.size;

        let (x, y) = editor.cursor_position()?;
        let (x, y) = (x as f32, y as f32);
        let mut cursor_rect = XyRect::new([x + 1.0, x + 1.0 + CURSOR_WIDTH], [y - 2.0, y + 5.0 + line_height]);
        
        cursor_rect.x[0] = cursor_rect.x[0] / size.x;
        cursor_rect.x[1] = cursor_rect.x[1] / size.x;
        cursor_rect.y[0] = cursor_rect.y[0] / size.y;
        cursor_rect.y[1] = cursor_rect.y[1] / size.y;
        
        let editor_rect = self.nodes[node_i].rect;
        
        let rect = XyRect::new(
            [editor_rect.x[0] + cursor_rect.x[0], editor_rect.x[0] + cursor_rect.x[1]],
            [editor_rect.y[0] + cursor_rect.y[0], editor_rect.y[0] + cursor_rect.y[1]],
        );

        self.sys.rects.push(RenderRect {
            rect: rect.to_graphics_space_rounded(size),
            tex_coords: DUMB_MAGIC_TEX_COORDS,
            vertex_colors: VertexColors::KERU_GRAD,
            z: self.nodes[node_i].z - 0.0001,
            last_hover: f32::MIN,
            last_click: f32::MIN,
            shape_data: 0.0,
            flags: RenderRect::EMPTY_FLAGS,
            _padding: 0,
            clip_rect: self.nodes[node_i].clip_rect.to_graphics_space_rounded(size),
        });
        Some(())
    }

    pub fn push_selection_rects(&mut self) -> Option<()> {
        let size = self.sys.unifs.size;
    
        let id = self.sys.focused?;
        let node_i = self.nodes.node_hashmap.get(&id)?.slab_i;
    
        let Some(TextI::TextEditI(edit_i)) = self.nodes[node_i].text_i else {
            return None
        };
    
        let editor = &self.sys.text.slabs.editors.get(edit_i)?.editor;
    
        let selection_bounds = editor.selection_bounds();
    
        let buffer = editor.buffer();

        const TASTEFUL_THICKNESS_H: f32 = 5.0;
        const TASTEFUL_THICKNESS_V: f32 = 5.0;
    
        for run in buffer.layout_runs() {
            let line_i = run.line_i;
            let line_top = run.line_top;
            let line_height = run.line_height;
    
            // Extract selection rectangles
            if let Some((start, end)) = selection_bounds {
                if line_i >= start.line && line_i <= end.line {
                    let mut range_opt = None;
                    
                    for glyph in run.glyphs.iter() {
                        let cluster = &run.text[glyph.start..glyph.end];
                        let total = cluster.grapheme_indices(true).count();
                        let mut c_x = glyph.x;
                        let c_w = glyph.w / total as f32;
                        
                        for (i, c) in cluster.grapheme_indices(true) {
                            let c_start = glyph.start + i;
                            let c_end = glyph.start + i + c.len();
                            
                            if (start.line != line_i || c_end > start.index)
                                && (end.line != line_i || c_start < end.index)
                            {
                                range_opt = match range_opt.take() {
                                    Some((min, max)) => Some((
                                        cmp::min(min, c_x as i32),
                                        cmp::max(max, (c_x + c_w) as i32),
                                    )),
                                    None => Some((c_x as i32, (c_x + c_w) as i32)),
                                };
                            } else if let Some((min, max)) = range_opt.take() {
                                let min_f = min as f32;
                                let max_f = max as f32;
                                let top_f = line_top;
                                let bottom_f = line_top + line_height;
                                
                                let selection_rect = XyRect::new(
                                    [min_f, max_f + TASTEFUL_THICKNESS_H],
                                    [top_f, bottom_f + TASTEFUL_THICKNESS_V]
                                );
                                
                                // Normalize to editor space
                                let editor_rect = self.nodes[node_i].rect;
                                
                                let rect = XyRect::new(
                                    [editor_rect.x[0] + selection_rect.x[0] / size.x, 
                                     editor_rect.x[0] + selection_rect.x[1] / size.x],
                                    [editor_rect.y[0] + selection_rect.y[0] / size.y, 
                                     editor_rect.y[0] + selection_rect.y[1] / size.y],
                                );
    
                                self.sys.rects.push(RenderRect {
                                    rect: rect.to_graphics_space_rounded(size),
                                    tex_coords: DUMB_MAGIC_TEX_COORDS,
                                    vertex_colors: VertexColors::flat(Color::KERU_PINK),
                                    z: self.nodes[node_i].z - 0.0001,
                                    last_hover: f32::MIN,
                                    last_click: f32::MIN,
                                    shape_data: 0.0,
                                    flags: RenderRect::EMPTY_FLAGS,
                                    _padding: 0,
                                    clip_rect: self.nodes[node_i].clip_rect.to_graphics_space_rounded(size),
                                });
                            }
                            c_x += c_w;
                        }
                    }
    
                    if run.glyphs.is_empty() && end.line > line_i {
                        let selection_rect = XyRect::new(
                            [0.0, TASTEFUL_THICKNESS_H],
                            [line_top, line_top + line_height + TASTEFUL_THICKNESS_V]
                        );
                        
                        // Normalize to editor space
                        let editor_rect = self.nodes[node_i].rect;
                        
                        let rect = XyRect::new(
                            [editor_rect.x[0] + selection_rect.x[0] / size.x, 
                             editor_rect.x[0] + selection_rect.x[1] / size.x],
                            [editor_rect.y[0] + selection_rect.y[0] / size.y, 
                             editor_rect.y[0] + selection_rect.y[1] / size.y],
                        );
    
                        self.sys.rects.push(RenderRect {
                            rect: rect.to_graphics_space_rounded(size),
                            tex_coords: DUMB_MAGIC_TEX_COORDS,
                            vertex_colors: VertexColors::flat(Color::KERU_PINK),
                            z: self.nodes[node_i].z - 0.0001,
                            last_hover: f32::MIN,
                            last_click: f32::MIN,
                            shape_data: 0.0,
                            flags: RenderRect::EMPTY_FLAGS,
                            _padding: 0,
                            clip_rect: self.nodes[node_i].clip_rect.to_graphics_space_rounded(size),
                        });
                    }
    
                    if let Some((min, max)) = range_opt.take() {
                        
                        // Convert PlainRect to RenderRect
                        let min_f = min as f32;
                        let max_f = max as f32;
                        let top_f = line_top;
                        let bottom_f = line_top + line_height;
                        
                        let selection_rect = XyRect::new(
                            [min_f, max_f + TASTEFUL_THICKNESS_H],
                            [top_f, bottom_f + TASTEFUL_THICKNESS_V]
                        );
                        
                        // Normalize to editor space
                        let editor_rect = self.nodes[node_i].rect;
                        
                        let rect = XyRect::new(
                            [editor_rect.x[0] + selection_rect.x[0] / size.x, 
                             editor_rect.x[0] + selection_rect.x[1] / size.x],
                            [editor_rect.y[0] + selection_rect.y[0] / size.y, 
                             editor_rect.y[0] + selection_rect.y[1] / size.y],
                        );
    
                        self.sys.rects.push(RenderRect {
                            rect: rect.to_graphics_space_rounded(size),
                            tex_coords: DUMB_MAGIC_TEX_COORDS,
                            vertex_colors: VertexColors::flat(Color::KERU_PINK),
                            z: self.nodes[node_i].z - 0.0001,
                            last_hover: f32::MIN,
                            last_click: f32::MIN,
                            shape_data: 0.0,
                            flags: RenderRect::EMPTY_FLAGS,
                            _padding: 0,
                            clip_rect: self.nodes[node_i].clip_rect.to_graphics_space_rounded(size),
                        });
                    }
                }
            }
        }
        
        Some(())
    }

}





pub(crate) struct TextEditHistory {
    stored_text: String,
    history: Vec<HistoryElem>,
    current_position: usize, // Cursor position in history
}

#[derive(Debug)]
enum HistoryElem {
    Delete(Delete),
    Insert(Insert)
}

#[derive(Debug)]
struct Delete {
    selection: Cursor,
    text: (usize, usize) // range into storedtext
}

#[derive(Debug)]
pub struct Insert {
    cursor: Cursor,
    text: (usize, usize)
}

#[derive(Debug)]
pub struct UndoInsert {
    cursor: Cursor,
    n_chars: usize,
}

#[derive(Debug)]
struct UndoDelete<'a> {
    cursor: Cursor,
    text: &'a str,
}

#[derive(Debug)]
pub enum Undo<'a> {
    UndoInsert(UndoInsert),
    UndoDelete(UndoDelete<'a>),
}

impl TextEditHistory {
    pub fn new() -> Self {
        TextEditHistory {
            stored_text: String::with_capacity(50),
            history: Vec::with_capacity(50),
            current_position: 0,
        }
    }

    pub fn record_delete<'buffer>(&mut self, deleted_text: &str, selection: Cursor) {
        // Store the deleted text in stored_text
        let start = self.stored_text.len();
        self.stored_text.push_str(deleted_text);
        let end = self.stored_text.len();
        
        // Truncate history if we're not at the end (discard future redos)
        if self.current_position < self.history.len() {
            self.history.truncate(self.current_position);
        }
        
        // Add new operation
        self.history.push(HistoryElem::Delete(Delete {
            selection,
            text: (start, end),
        }));
        self.current_position = self.history.len();
    }

    pub fn record_insert(&mut self, inserted_char: char, cursor: Cursor) {
        let start = self.stored_text.len();
        self.stored_text.push(inserted_char);
        let end = self.stored_text.len();

        // Truncate history if we're not at the end
        if self.current_position < self.history.len() {
            self.history.truncate(self.current_position);
        }
        
        // Add new operation
        self.history.push(HistoryElem::Insert(Insert {
            cursor,
            text: (start, end),
        }));
        self.current_position = self.history.len();
    }

    pub fn undo(&mut self) -> Option<Undo> {
        if self.current_position > 0 {
            self.current_position -= 1;
            let op = &self.history[self.current_position];
            match op {
                HistoryElem::Delete(delete) => {
                    // Reinsert the deleted text
                    let (start, end) = delete.text;
                    let deleted_text = &self.stored_text[start..end];
                    Some(Undo::UndoDelete(
                        UndoDelete {
                            cursor: delete.selection,
                            text: deleted_text,
                        }
                    ))
                },
                HistoryElem::Insert(insert) => {
                    let (start, end) = insert.text;
                    Some(Undo::UndoInsert(
                        UndoInsert {
                            cursor: insert.cursor,
                            n_chars: end - start,
                        }
                    ))
                }
            }
        } else {
            None
        }
    }
    
    pub fn redo(&mut self) -> Option<Undo> {
        // Check if there are operations to redo (we must be at a position less than the history length)
        if self.current_position < self.history.len() {
            // Get the operation to redo
            let op = &self.history[self.current_position];
            
            // Move forward in the history
            self.current_position += 1;
            
            // Return the appropriate HistoryOp based on the stored operation
            match op {
                HistoryElem::Delete(delete) => {
                    let (start, end) = delete.text;
                    Some(Undo::UndoInsert(
                        UndoInsert {
                            cursor: delete.selection,
                            n_chars: end - start,
                        }
                    ))
                },
                HistoryElem::Insert(insert) => {
                    // For an insert operation, we need to insert again
                    let (start, end) = insert.text;
                    let text_to_insert = &self.stored_text[start..end];
                    
                    Some(Undo::UndoDelete(
                        UndoDelete {
                            cursor: insert.cursor,
                            text: text_to_insert,
                        }
                    ))
                }
            }
        } else {
            None
        }
    }
}