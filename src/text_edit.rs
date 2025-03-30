use std::cmp;

use arboard::Clipboard;
use glyphon::{cosmic_text::{BorrowedWithFontSystem, Motion, Selection}, Action, Affinity, Cursor, Edit};
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
    clipboard: &mut Clipboard,
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
                        // ctrl + enter isn't even listened
                        if ! modifiers.control_key() {
                            if editor.selection() != Selection::None {
                                editor.delete_selection();
                            } else {
                                editor.action(Action::Enter);
                            }
                            return true;
                        }
                    }
                    Key::Named(NamedKey::Backspace) => {
                        if editor.selection() != Selection::None {
                            editor.delete_selection();
                            return true;
                        }
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
                        if editor.selection() != Selection::None {
                            editor.delete_selection();
                            return true;
                        }
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
                        if ! modifiers.control_key() {
                            if let Some(text) = key.to_text() {
                                for c in text.chars() {
                                    editor.action(Action::Insert(c));
                                }
                                return true;
                            }
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
                                "c" => {
                                    // Copy selected text to clipboard
                                    if let Some((start, end)) = editor.selection_bounds() {
                                        let text = editor.with_buffer(|buffer| {
                                            let mut result = String::new();
                                            
                                            if start.line == end.line {
                                                // Single line selection
                                                let line_str = buffer.lines[start.line].text();
                                                // Use grapheme indices instead of char_to_byte
                                                let graphemes: Vec<&str> = line_str.graphemes(true).collect();
                                                let start_char = if start.index < graphemes.len() { start.index } else { graphemes.len() };
                                                let end_char = if end.index < graphemes.len() { end.index } else { graphemes.len() };
                                                
                                                for i in start_char..end_char {
                                                    if i < graphemes.len() {
                                                        result.push_str(graphemes[i]);
                                                    }
                                                }
                                            } else {
                                                // Multi-line selection
                                                // First line
                                                let first_line_str = buffer.lines[start.line].text();
                                                let first_graphemes: Vec<&str> = first_line_str.graphemes(true).collect();
                                                let start_char = if start.index < first_graphemes.len() { start.index } else { first_graphemes.len() };
                                                
                                                for i in start_char..first_graphemes.len() {
                                                    result.push_str(first_graphemes[i]);
                                                }
                                                result.push('\n');
                                                
                                                // Middle lines
                                                for line_idx in (start.line + 1)..end.line {
                                                    result.push_str(buffer.lines[line_idx].text());
                                                    result.push('\n');
                                                }
                                                
                                                // Last line
                                                let last_line_str = buffer.lines[end.line].text();
                                                let last_graphemes: Vec<&str> = last_line_str.graphemes(true).collect();
                                                let end_char = if end.index < last_graphemes.len() { end.index } else { last_graphemes.len() };
                                                
                                                for i in 0..end_char {
                                                    if i < last_graphemes.len() {
                                                        result.push_str(last_graphemes[i]);
                                                    }
                                                }
                                            }
                                            
                                            result
                                        });
                                        
                                        if let Err(err) = clipboard.set_text(text) {
                                            eprintln!("Failed to copy text to clipboard: {}", err);
                                        }
                                    }
                                    return true;
                                }
                                "v" => {
                                    // Paste text from clipboard
                                    if let Ok(text) = clipboard.get_text() {
                                        // Delete any selected text first
                                        editor.delete_selection();
                                        
                                        // Insert the clipboard text
                                        for line in text.lines().enumerate() {
                                            if line.0 > 0 {
                                                // For lines after the first one, insert a newline first
                                                editor.action(Action::Enter);
                                            }
                                            
                                            // Insert the line character by character
                                            for c in line.1.chars() {
                                                editor.action(Action::Insert(c));
                                            }
                                        }
                                        
                                        // Handle the case where the clipboard text ends with a newline
                                        if text.ends_with('\n') {
                                            editor.action(Action::Enter);
                                        }
                                    }
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

impl Ui {
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
            tex_coords: Xy {
                x: [0.9375, 0.9394531],
                y: [0.00390625 / 2.0, 0.0],
            },
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
                                    [top_f, bottom_f]
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
                                    tex_coords: Xy {
                                        x: [0.9375, 0.9394531],
                                        y: [0.00390625 / 2.0, 0.0],
                                    },
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
                            tex_coords: Xy {
                                x: [0.9375, 0.9394531],
                                y: [0.00390625 / 2.0, 0.0],
                            },
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
                            tex_coords: Xy {
                                x: [0.9375, 0.9394531],
                                y: [0.00390625 / 2.0, 0.0],
                            },
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
