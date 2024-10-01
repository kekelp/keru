use wgpu::Queue;
use winit::event::{Event, KeyEvent, MouseButton, WindowEvent};

use crate::{ui_time_f32, Id, NodeKey, Ui};

use glyphon::{Affinity, Cursor as GlyphonCursor};


#[derive(Debug, Copy, Clone)]
pub struct BlinkyLine {
    pub index: usize,
    pub affinity: Affinity,
}

#[derive(Debug, Copy, Clone)]
pub enum Cursor {
    BlinkyLine(BlinkyLine),
    Selection((GlyphonCursor, GlyphonCursor)),
}


impl Ui {

    // called on every mouse movement AND on every frame.
    // todo: think if it's really worth it to do this on every mouse movement.
    pub fn resolve_hover(&mut self) {
        let topmost_mouse_hit = self.scan_mouse_hits();

        if let Some(hovered_id) = topmost_mouse_hit {
            self.sys.hovered.push(hovered_id);
            let t = ui_time_f32();
            let node = self.nodes.get_by_id(&hovered_id).unwrap();
            node.last_hover = t;
        }
    }

    pub fn resolve_click(&mut self) -> bool {
        let topmost_mouse_hit = self.scan_mouse_hits();

        // defocus when use clicking anywhere outside.
        self.sys.focused = None;

        if let Some(clicked_id) = topmost_mouse_hit {
            self.sys.waiting_for_click_release = true;

            self.sys.clicked.push(clicked_id);
            let t = ui_time_f32();
            let node = self.nodes.get_by_id(&clicked_id).unwrap();
            node.last_click = t;

            if node.text_id.is_some() {
                if let Some(text) = node.params.text_params{
                    if text.editable {
                        self.sys.focused = Some(clicked_id);
                    }
                }
            }

            if let Some(id) = node.text_id {
                let text_area = &mut self.sys.text.text_areas[id];
                let (x, y) = (
                    self.sys.part.mouse_pos.x - text_area.params.left,
                    self.sys.part.mouse_pos.y - text_area.params.top,
                );

                // todo: with how I'm misusing cosmic-text, this might become "unsafe" soon (as in, might be incorrect or cause panics, not actually unsafe).
                // I think in general, there should be a safe version of hit() that just forces a rerender just to be sure that the offset is safe to use.
                // But in this case, calling this in resolve_mouse_input() and not on every winit mouse event probably means it's safe

                // actually, the enlightened way is that cosmic_text exposes an "unsafe" hit(), but we only ever see the string + cursor + buffer struct, and call that hit(), which doesn't return an offset but just mutates the one inside.
                text_area.buffer.hit(x, y);
            }
        }

        let consumed = topmost_mouse_hit.is_some();
        return consumed;
    }

    pub fn resolve_click_release(&mut self) -> bool {
        self.sys.waiting_for_click_release = false;
        let topmost_mouse_hit = self.scan_mouse_hits();
        let consumed = topmost_mouse_hit.is_some();
        self.sys.clicked.clear();
        return consumed;
    }

    pub fn scan_mouse_hits(&mut self) -> Option<Id> {
        self.sys.mouse_hit_stack.clear();

        for rect in &self.sys.rects {
            if self.sys.part.mouse_hit_rect(rect) {
                self.sys.mouse_hit_stack.push((rect.id, rect.z));
            }
        }

        // only the one with the highest z is actually clicked.
        // in practice, nobody ever sets the Z. it depends on the order.
        let mut topmost_hit = None;

        let mut max_z = f32::MAX;
        for (id, z) in self.sys.mouse_hit_stack.iter().rev() {
            if *z < max_z {
                max_z = *z;
                topmost_hit = Some(*id);
            }
        }

        return topmost_hit;
    }


    // returns: is the event consumed?
    pub fn handle_events(&mut self, full_event: &Event<()>, queue: &Queue) -> bool {
        if let Event::NewEvents(_) = full_event {
            self.sys.mouse_status.clear_frame();
        }


        if let Event::WindowEvent { event, .. } = full_event {
            match event {
                WindowEvent::CursorMoved { position, .. } => {
                    self.sys.part.mouse_pos.x = position.x as f32;
                    self.sys.part.mouse_pos.y = position.y as f32;
                    self.resolve_hover();
                    // cursormoved is never consumed
                }
                WindowEvent::MouseInput { button, state, .. } => {
                    if *button == MouseButton::Left {
                        let is_pressed = state.is_pressed();
                        if is_pressed {
                            let consumed = self.resolve_click();
                            return consumed;
                        } else {
                            let waiting_for_click_release = self.sys.waiting_for_click_release;
                            let on_rect = self.resolve_click_release();
                            let consumed = on_rect && waiting_for_click_release;
                            return consumed;
                        }
                    }
                }
                WindowEvent::ModifiersChanged(modifiers) => {
                    self.sys.key_mods = modifiers.state();
                }
                WindowEvent::KeyboardInput {
                    event,
                    is_synthetic,
                    ..
                } => {
                    if !is_synthetic {
                        let consumed = self.handle_keyboard_event(event);
                        return consumed;
                    }
                }
                WindowEvent::Resized(size) => self.resize(size, queue),
                _ => {}
            }

            self.sys.mouse_status.update(event);

        }

        return false;
    }


    pub fn is_clicked(&self, node_key: NodeKey) -> bool {
        let real_key = self.get_latest_twin_key(node_key);
        if let Some(real_key) = real_key {
            return self.sys.clicked.contains(&real_key.id);
        } else {
            return false;
        }
        
    }

    pub fn is_dragged(&self, node_key: NodeKey) -> Option<(f64, f64)> {
        if self.is_clicked(node_key) {
            return Some(self.sys.mouse_status.cursor_diff())
        } else {
            return None;
        }
    }

    // todo: is_clicked_advanced

    pub fn is_hovered(&self, node_key: NodeKey) -> bool {
        return self.sys.hovered.last() != Some(&node_key.id);
    }

    // todo: is_hovered_advanced



    pub fn handle_keyboard_event(&mut self, _event: &KeyEvent) -> bool {
        // todo: remove line.reset(); and do it only once per frame via change watcher guy

        // if let Key::Named(named_key) = &event.logical_key { if named_key == &NamedKey::F1 {
        //     if event.state.is_pressed() && self.sys.debug_key_pressed == false {
        //         #[cfg(debug_assertions)]
        //         {
        //             self.sys.debug_mode = !self.sys.debug_mode;
        //         }
        //     }

        //     self.sys.debug_key_pressed = event.state.is_pressed();
        // } }

        // // if there is no focused text node, return consumed: false
        // let id = unwrap_or_return!(self.sys.focused, false);
        // let node = unwrap_or_return!(self.nodes.get_by_id(&id), false);
        // let text_id = unwrap_or_return!(node.text_id, false);

        // // return consumed: true in each of these cases. Still don't consume keys that the UI doesn't use.
        // if event.state.is_pressed() {
        //     let buffer = &mut self.sys.text.text_areas[text_id].buffer;
        //     let line = &mut buffer.lines[0];

        //     match &event.logical_key {
        //         // todo: ctrl + Z
        //         Key::Named(named_key) => match named_key {
        //             NamedKey::ArrowLeft => {
        //                 match (self.sys.key_mods.shift_key(), self.sys.key_mods.control_key()) {
        //                     (true, true) => line.text.control_shift_left_arrow(),
        //                     (true, false) => line.text.shift_left_arrow(),
        //                     (false, true) => line.text.control_left_arrow(),
        //                     (false, false) => line.text.left_arrow(),
        //                 }
        //                 return true;
        //             }
        //             NamedKey::ArrowRight => {
        //                 match (self.sys.key_mods.shift_key(), self.sys.key_mods.control_key()) {
        //                     (true, true) => line.text.control_shift_right_arrow(),
        //                     (true, false) => line.text.shift_right_arrow(),
        //                     (false, true) => line.text.control_right_arrow(),
        //                     (false, false) => line.text.right_arrow(),
        //                 }
        //                 return true;
        //             }
        //             NamedKey::Backspace => {
        //                 if self.sys.key_mods.control_key() {
        //                     line.text.ctrl_backspace();
        //                 } else {
        //                     line.text.backspace();
        //                 }
        //                 line.reset();
        //                 return true;
        //             }
        //             NamedKey::End => {
        //                 match self.sys.key_mods.shift_key() {
        //                     true => line.text.shift_end(),
        //                     false => line.text.go_to_end(),
        //                 }
        //                 line.reset();
        //                 return true;
        //             }
        //             NamedKey::Home => {
        //                 match self.sys.key_mods.shift_key() {
        //                     false => line.text.go_to_start(),
        //                     true => line.text.shift_home(),
        //                 }
        //                 line.reset();
        //                 return true;
        //             }
        //             NamedKey::Delete => {
        //                 if self.sys.key_mods.control_key() {
        //                     line.text.ctrl_delete();
        //                 } else {
        //                     line.text.delete();
        //                 }
        //                 line.reset();
        //                 return true;
        //             }
        //             NamedKey::Space => {
        //                 line.text.insert_str_at_cursor(" ");
        //                 line.reset();
        //                 return true;
        //             }
        //             _ => {}
        //         },
        //         Key::Character(new_char) => {
        //             if !self.sys.key_mods.control_key()
        //                 && !self.sys.key_mods.alt_key()
        //                 && !self.sys.key_mods.super_key()
        //             {
        //                 line.text.insert_str_at_cursor(new_char);
        //                 line.reset();
        //                 return true;
        //             } else if self.sys.key_mods.control_key() {
        //                 match new_char.as_str() {
        //                     "c" => {
        //                         let selected_text = line.text.selected_text().to_owned();
        //                         if let Some(text) = selected_text {
        //                             let _ = self.sys.clipboard.set_contents(text.to_string());
        //                         }
        //                         return true;
        //                     }
        //                     "v" => {
        //                         if let Ok(pasted_text) = self.sys.clipboard.get_contents() {
        //                             line.text.insert_str_at_cursor(&pasted_text);
        //                             line.reset();
        //                         }
        //                         return true;
        //                     }
        //                     _ => {}
        //                 }
        //             }
        //         }
        //         Key::Unidentified(_) => {}
        //         Key::Dead(_) => {}
        //     };
        // }

        return false;
    }

}