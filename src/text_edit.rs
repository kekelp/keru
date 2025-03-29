use std::cmp;

use glyphon::{cosmic_text::{BorrowedWithFontSystem, Motion, Selection}, Action, Affinity, Cursor, Edit, Editor, FontSystem};
use unicode_segmentation::UnicodeSegmentation;
use winit::{event::{ElementState, KeyEvent, MouseButton, WindowEvent}, keyboard::{Key, ModifiersState, NamedKey}};

use crate::*;


pub(crate) fn editor_window_event<'buffer>(
    editor: &mut BorrowedWithFontSystem<impl Edit<'buffer>>,
    event: &WindowEvent,
    modifiers: &ModifiersState,
    mouse_left_pressed: bool,
    mouse_x: f64,
    mouse_y: f64,
) -> bool {
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
                        return true;
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
                        return true;
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
                            editor.action(Action::Motion(Motion::Home));
                        } else {
                            editor.action(Action::Motion(Motion::Up));
                        }
                        return true;
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
                        if editor.cursor().line
                            == editor.with_buffer(|buffer| buffer.lines.len() - 1)
                        {
                            editor.action(Action::Motion(Motion::End));
                        } else {
                            editor.action(Action::Motion(Motion::Down));
                        }
                        return true;
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
                        return true;
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
                        return true;
                    }
                    Key::Named(NamedKey::PageUp) => {
                        editor.action(Action::Motion(Motion::PageUp));
                        return true;
                    }
                    Key::Named(NamedKey::PageDown) => {
                        editor.action(Action::Motion(Motion::PageDown));
                        return true;
                    }
                    Key::Named(NamedKey::Escape) => {
                        editor.action(Action::Escape);
                        return true;
                    }
                    Key::Named(NamedKey::Enter) => {
                        editor.action(Action::Enter);
                        return true;
                    }
                    Key::Named(NamedKey::Backspace) => {
                        if modifiers.control_key() {
                            let cursor = editor.cursor();
                            editor.action(Action::Motion(Motion::PreviousWord));
                            let start = editor.cursor();
                            editor.set_selection(Selection::Normal(start));
                            editor.set_cursor(cursor);
                            editor.delete_selection();
                        } else {
                            editor.action(Action::Backspace);
                        }
                        return true;
                    }
                    Key::Named(NamedKey::Delete) => {
                        if modifiers.control_key() {
                            let cursor = editor.cursor();
                            editor.action(Action::Motion(Motion::NextWord));
                            let end = editor.cursor();
                            editor.set_selection(Selection::Normal(cursor));
                            editor.set_cursor(end);
                            editor.delete_selection();
                        } else {
                            editor.action(Action::Delete);
                        }
                        return true;
                    }
                    Key::Named(key) => {
                        if let Some(text) = key.to_text() {
                            for c in text.chars() {
                                editor.action(Action::Insert(c));
                            }
                            return true;
                        }
                    }
                    Key::Character(text) => {
                        if modifiers.control_key() {
                            match text.as_str() {
                                "a" => {
                                    editor.set_cursor(Cursor::new_with_affinity(0, 0, Affinity::Before));
                                    let end_line = editor.with_buffer(|buffer| buffer.lines.len() - 1);
                                    let end_col = editor.with_buffer(|buffer| buffer.lines[end_line].text().len());
                                    editor.set_selection(Selection::Normal(Cursor::new_with_affinity(end_line, end_col, Affinity::After)));
                                    return true;
                                }
                                _ => {},
                            }
                        } else {
                            for c in text.chars() {
                                editor.action(Action::Insert(c));
                            }
                            return true;
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
                    x: position.x as i32,
                    y: position.y as i32,
                });
                return true;
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
                        x: mouse_x as i32,
                        y: mouse_y as i32,
                    });
                    return true;
                }
            }
        }
        _ => {},
    }
    return false;
}


#[derive(Debug, Clone, Copy)]
pub struct SelectionRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct CursorRect {
    pub x: i32,
    pub y: i32,
    pub height: u32,
    pub width: u32,
}

pub struct EditorDecorationData {
    pub selections: Vec<SelectionRect>,
    pub cursor: Option<CursorRect>,
}

impl EditorDecorationData {
    pub fn new() -> Self {
        Self {
            selections: Vec::new(),
            cursor: None,
        }
    }
}

pub fn get_editor_decorations(editor: &mut Editor<'static>) -> EditorDecorationData {
    let mut data = EditorDecorationData::new();
    let selection_bounds = editor.selection_bounds();

    let mut line_height = 10.0;
    for run in editor.rip_it_out().layout_runs() {
        line_height = run.line_height;
        break;
    }

    // Extract cursor position
    if let Some((x, y)) = editor.cursor_position() {
        data.cursor = Some(CursorRect {
            x,
            y,
            width: 1, // or your desired cursor width
            height: line_height as u32,
        });
    }

    let buffer = editor.rip_it_out();

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
                            data.selections.push(SelectionRect {
                                x: min,
                                y: line_top as i32,
                                width: cmp::max(0, max - min) as u32,
                                height: line_height as u32,
                            });
                        }
                        c_x += c_w;
                    }
                }

                if run.glyphs.is_empty() && end.line > line_i {
                    // Full line selection for empty lines
                    data.selections.push(SelectionRect {
                        x: 0,
                        y: line_top as i32,
                        width: buffer.size().0.unwrap_or(0.0) as u32,
                        height: line_height as u32,
                    });
                }

                if let Some((mut min, mut max)) = range_opt.take() {
                    if end.line > line_i {
                        // Extend to end of line
                        if run.rtl {
                            min = 0;
                        } else {
                            max = buffer.size().0.unwrap_or(0.0) as i32;
                        }
                    }
                    data.selections.push(SelectionRect {
                        x: min,
                        y: line_top as i32,
                        width: cmp::max(0, max - min) as u32,
                        height: line_height as u32,
                    });
                }
            }
        }
    }
    
    data
}