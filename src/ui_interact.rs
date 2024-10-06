use std::time::Instant;

use wgpu::Queue;
use winit::{dpi::PhysicalPosition, event::{ElementState, Event, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent}};

use crate::{ui_math::Xy, ui_time_f32, Id, NodeKey, Ui, T0};

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

    pub fn resolve_click(&mut self, button: MouseButton, state: ElementState) -> bool {
        let topmost_mouse_hit = self.scan_mouse_hits();
        
        // defocus when use clicking anywhere outside.
        self.sys.focused = None;
        
        let Some(clicked_id) = topmost_mouse_hit else {
            return false;
        };

        self.sys.waiting_for_click_release = true;

        let t = T0.elapsed();
        let node = self.nodes.get_by_id(&clicked_id).unwrap();
        node.last_click = t.as_secs_f32();
        
        self.sys.last_frame_clicks.push(StoredClick {
            hit_node: clicked_id,

            button,
            timestamp: Instant::now(),
            position: Xy::new(self.sys.part.mouse_pos.x, self.sys.part.mouse_pos.y),
            state,
        });

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
    
        let consumed = topmost_mouse_hit.is_some();
        return consumed;
    }

    pub fn resolve_click_release(&mut self) -> bool {
        self.sys.waiting_for_click_release = false;
        let topmost_mouse_hit = self.scan_mouse_hits();
        let consumed = topmost_mouse_hit.is_some();
        self.sys.last_frame_clicks.clear();
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
                            let consumed = self.resolve_click(*button, *state);
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
        let Some(real_key) = real_key else {
            return false;
        };
        return self.sys.last_frame_clicks.ids.contains(&real_key.id);
    }

    pub fn is_dragged_abs(&mut self, node_key: NodeKey) -> Option<(f64, f64)> {
        if self.is_clicked(node_key) {
            return Some(self.sys.mouse_status.cursor_diff())
        } else {
            return None;
        }
    }

    pub fn is_dragged(&mut self, node_key: NodeKey) -> Option<(f64, f64)> {
        let diff = self.is_dragged_abs(node_key)?;
        return Some(diff);
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


#[derive(Debug, Default)]
pub struct MouseButtons {
    pub left: bool,
    pub right: bool,
    pub middle: bool,
    pub back: bool,
    pub forward: bool,
    pub other: u16, // 16-bit field for other buttons
}
impl MouseButtons {
    pub fn is_other_button_pressed(&self, id: u16) -> bool {
        if id < 16 {
            return self.other & (1 << id) != 0;
        } else {
            panic!("Mouse button id must be between 0 and 15")
        }
    }
}

#[derive(Debug)]
pub struct MouseInputState {
    pub position: PhysicalPosition<f64>,
    pub buttons: MouseButtons,
    pub scroll_delta: (f32, f32),

    // previous for diffs
    pub prev_position: PhysicalPosition<f64>,
}

impl Default for MouseInputState {
    fn default() -> Self {
        return Self {
            position: PhysicalPosition::new(0.0, 0.0),
            buttons: MouseButtons::default(),
            scroll_delta: (0.0, 0.0),

            prev_position: PhysicalPosition::new(0.0, 0.0),
        };
    }
}

impl MouseInputState {
    pub fn update(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.position = *position;
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let pressed = *state == ElementState::Pressed;
                match button {
                    MouseButton::Left => self.buttons.left = pressed,
                    MouseButton::Right => self.buttons.right = pressed,
                    MouseButton::Middle => self.buttons.middle = pressed,
                    MouseButton::Back => self.buttons.back = pressed,
                    MouseButton::Forward => self.buttons.forward = pressed,
                    MouseButton::Other(id) => {
                        if *id < 16 {
                            if pressed {
                                self.buttons.other |= 1 << id;
                            } else {
                                self.buttons.other &= !(1 << id);
                            }
                        }
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(x, y) => {
                    self.scroll_delta.0 += x;
                    self.scroll_delta.1 += y;
                }
                MouseScrollDelta::PixelDelta(pos) => {
                    self.scroll_delta.0 += pos.x as f32;
                    self.scroll_delta.1 += pos.y as f32;
                }
            },
            _ => {}
        }
    }

    pub fn clear_frame(&mut self) {
        self.prev_position = self.position;
    }

    pub fn cursor_diff(&self) -> (f64, f64) {
        return (
            self.prev_position.x - self.position.x,
            self.prev_position.y - self.position.y,
        );
    }

    pub fn reset_scroll(&mut self) {
        self.scroll_delta = (0.0, 0.0);
    }
}


#[derive(Clone, Copy, Debug)]
pub struct StoredClick {
    pub button: MouseButton,
    pub timestamp: Instant,
    pub position: Xy<f32>,
    pub state: ElementState,
    pub hit_node: Id,
}

pub struct LastFrameClicks {
    pub ids: Vec<Id>,
    pub clicks: Vec<StoredClick>,
}
impl LastFrameClicks {
    pub fn new() -> LastFrameClicks {
        return LastFrameClicks {
            ids: Vec::with_capacity(20),
            clicks: Vec::with_capacity(20),
        }
    }

    fn push(&mut self, info: StoredClick) {
        self.ids.push(info.hit_node);
        self.clicks.push(info);
    }

    pub fn clear(&mut self) {
        self.ids.clear();
        self.clicks.clear();
    }
}
