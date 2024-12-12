use std::time::{Duration, Instant};

use winit::{event::{ElementState, Event, KeyEvent, MouseButton, WindowEvent}, keyboard::{Key, NamedKey}};

use crate::*;


impl Ui {
    /// Returns all [`MouseEvent`]s from the last frame.
    pub fn all_mouse_events(&self) -> impl DoubleEndedIterator<Item = &MouseEvent> {
        return self.sys.last_frame_mouse_events.iter();
    }

    /// Returns all [`MouseEvent`]s for a specific button on the node corresponding to `node_key`, or an empty iterator if the node is currently not part of the tree or if it doesn't exist.
    pub fn mouse_events(&self, mouse_button: MouseButton, node_key: NodeKey) -> impl DoubleEndedIterator<Item = &MouseEvent> {
        return self
            .all_mouse_events()
            .filter(move |c| c.originally_pressed.hit_node_id == Some(node_key.id) && c.button == mouse_button);
    }

    /// Returns `true` if the left mouse button was clicked on the node corresponding to `node_key`, or `false` if the node is currently not part of the tree or if it doesn't exist.
    pub fn is_clicked(&self, node_key: NodeKey) -> bool {
        let clicked_times = self.is_mouse_button_clicked(MouseButton::Left, node_key);
        return clicked_times > 0;
    }

    /// Returns the number of times `mouse_button` was clicked on the node corresponding to `node_key`, or `0` if the node is currently not part of the tree or if it doesn't exist.
    pub fn is_mouse_button_clicked(&self, mouse_button: MouseButton, node_key: NodeKey) -> usize {
        let all_events = self.mouse_events(mouse_button, node_key);
        return all_events.filter(|c| c.is_just_clicked()).count();
    }

    /// Returns `true` if a left mouse button click was released on the node corresponding to `node_key`, or `false` if the node is currently not part of the tree or if it doesn't exist.
    pub fn is_click_released(&self, node_key: NodeKey) -> bool {
        let clicked_times = self.is_mouse_button_click_released(MouseButton::Left, node_key);
        return clicked_times > 0;
    }

    /// Returns the number of times a click of `mouse_button` was released on the node corresponding to `node_key`, or `0` if the node is currently not part of the tree or if it doesn't exist.
    pub fn is_mouse_button_click_released(&self, mouse_button: MouseButton, node_key: NodeKey) -> usize {
        let all_events = self.mouse_events(mouse_button, node_key);
        return all_events.filter(|c| c.is_click_release()).count();
    }

    /// Returns the drag distance for a mouse button on a node, or None if there was no drag.
    ///
    /// In the case where the user dragged, released, and redragged all in one frame,
    /// this sums the distances.
    pub fn is_mouse_button_dragged(&self, mouse_button: MouseButton, node_key: NodeKey) -> Option<(f64, f64)> {
        let all_events = self.mouse_events(mouse_button, node_key);
        
        // I doubt anyone cares, but in the case the user dragged, released, and redragged, all in one frame, let's find all the distances and sum them.
        let mut dist = Xy::new_symm(0.0);
        
        for e in all_events {
            dist = dist + e.drag_distance();
        }

        if dist == Xy::new_symm(0.0) {
            return None;
        } else {
            return Some((dist.x as f64, dist.y as f64));
        }
        // or just return the (0.0, 0.0)?
    }

    /// Returns the drag distance for the left mouse button on a node, or `None` if there was no drag.
    pub fn is_dragged(&self, node_key: NodeKey) -> Option<(f64, f64)> {
        return self.is_mouse_button_dragged(MouseButton::Left, node_key);
    }

    /// Returns the time a mouse button was held on a node and its last position, or `None` if it wasn’t held.
    pub fn is_mouse_button_held(&self, mouse_button: MouseButton, node_key: NodeKey) -> Option<(Duration, Xy<f32>)> {
        let all_events = self.mouse_events(mouse_button, node_key);

        let mut time_held = Duration::ZERO;
        let mut last_pos = Xy::new(0.0, 0.0);

        for e in all_events {
            time_held += e.time_held();
            // todo: this is not good... but iterators are hard
            last_pos = e.currently_at.position;
        }

        if time_held == Duration::ZERO {
            return None;
        } else {
            return Some((time_held, last_pos));
        }
    }

    /// Returns the time the left mouse button was held on a node and its last position, or `None` if it wasn’t held.
    pub fn is_held(&self, node_key: NodeKey) -> Option<(Duration, Xy<f32>)> {
        return self.is_mouse_button_held(MouseButton::Left, node_key);
    }

    /// Returns `true` if a node is currently hovered by the cursor.
    pub fn is_hovered(&self, node_key: NodeKey) -> bool {
        return self.sys.hovered.last() == Some(&node_key.id);
    }

    // todo: think if it's really worth it to do this on every mouse movement.
    // maybe add a global setting to do it just once per frame
    pub(crate) fn resolve_hover(&mut self) {
        let topmost_mouse_hit = self.scan_mouse_hits();

        if let Some(hovered_id) = topmost_mouse_hit {
            if self.sys.hovered.contains(&hovered_id) {
                // nothing changed, do nothing
            } else {
                // newly entered
                self.end_all_hovering();
                self.start_hovering(hovered_id);
            }

        } else {
            self.end_all_hovering();
        }

    }

    pub(crate) fn start_hovering(&mut self, hovered_id: Id) {
        self.sys.hovered.push(hovered_id);
        
        // todo: yuck
        let hovered_node_i = self.nodes.node_hashmap.get(&hovered_id).unwrap().slab_i;
        let hovered_node = &mut self.nodes.nodes[hovered_node_i];

        if hovered_node.params.interact.click_animation {
            hovered_node.hovered = true;
            hovered_node.hover_timestamp = ui_time_f32();
            
            self.sys.changes.cosmetic_rect_updates.push(hovered_node_i);
            // todo: maybe cleaner to make this pass through the cosmetic updates
            self.sys.changes.animation_rerender_time = Some(1.0);
        }

    }

    pub(crate) fn end_all_hovering(&mut self) {
        for hovered_id in &self.sys.hovered {
            let hovered_nodemap_entry = self.nodes.node_hashmap.get(&hovered_id);
            
            if let Some(entry) = hovered_nodemap_entry {
                // check that the node is currently part of the tree...
                // this is a bit scary, and it will need to change with `assume_unchanged` and friends
                if entry.last_frame_touched == self.sys.part.current_frame {

                    let hovered_node_i = entry.slab_i;
                    let hovered_node = &mut self.nodes.nodes[hovered_node_i];
                    
                    if hovered_node.params.interact.click_animation {
                        hovered_node.hovered = false;
                        hovered_node.hover_timestamp = ui_time_f32();
                        self.sys.changes.cosmetic_rect_updates.push(hovered_node_i);
                        self.sys.changes.animation_rerender_time = Some(1.0);
                    }
                }
            }
        }
        self.sys.hovered.clear();
    }

    pub(crate) fn end_frame_resolve_inputs(&mut self) {
        // clicks
        self.sys.last_frame_mouse_events.clear();

        self.sys.unresolved_click_presses.retain(|click| click.already_released == false);

        // for each unresolved clickdown, push a partial drag/hold diff and update last_seen
        let mouse_current_status = self.scan_current_mouse_status();

        for click_pressed in self.sys.unresolved_click_presses.iter_mut().rev() {

            let mouse_happening = MouseEvent {
                button: click_pressed.button,
                originally_pressed: click_pressed.pressed_at,
                last_seen: click_pressed.last_seen,
                currently_at: mouse_current_status,
                kind: IsMouseReleased::StillDownButFrameEnded,
            };

            self.sys.last_frame_mouse_events.push(mouse_happening);

            click_pressed.last_seen = mouse_current_status;
        }
    }

    pub(crate) fn resolve_click_release(&mut self, button: MouseButton) {
        // look for a mouse press to match and resolve
        let mut matched = None;
        for click_pressed in self.sys.unresolved_click_presses.iter_mut().rev() {
            if click_pressed.button == button {
                click_pressed.already_released = true;
                // this copy is a classic borrow checker skill issue.
                matched = Some(*click_pressed);
                break;
            }
        };

        if let Some(matched) = matched {
            // check for hits.
            let released_at = self.scan_current_mouse_status();

            let full_mouse_event = MouseEvent {
                button,
                originally_pressed: matched.pressed_at,
                last_seen: matched.last_seen,
                currently_at: released_at,
                kind: IsMouseReleased::MouseReleased,
            };

            self.sys.last_frame_mouse_events.push(full_mouse_event);
        }
    }

    // returns if the ui consumed the mouse press, or if it should be passed down.   
    pub(crate) fn resolve_click_press(&mut self, button: MouseButton) -> bool {
        // defocus, so that we defocus when clicking anywhere outside.
        // if we're clicking something we'll re-focus below.
        self.sys.focused = None;
        
        // check for hits.
        let current_mouse_status = self.scan_current_mouse_status();
        let topmost_mouse_hit = current_mouse_status.hit_node_id;

        // if nothing is hit, we're done.
        let Some(clicked_id) = topmost_mouse_hit else {
            return false;
        };

        let pending_press = PendingMousePress::new(current_mouse_status, button);

        self.sys.unresolved_click_presses.push(pending_press);
        
        // hardcoded stuff with animations, focusing nodes, spawning cursors, etc
        if button == MouseButton::Left {
            // the default animation and the "focused" flag are hardcoded to work on left click only, I guess.
            let t = T0.elapsed();

            // todo: yuck
            let clicked_node_i = self.nodes.node_hashmap.get(&clicked_id).unwrap().slab_i;
            let clicked_node = &mut self.nodes.nodes[clicked_node_i];

            if clicked_node.params.interact.click_animation {

                clicked_node.last_click = t.as_secs_f32();
                
                self.sys.changes.cosmetic_rect_updates.push(clicked_node_i);
                
                // todo: maybe cleaner to make this pass through the cosmetic updates
                self.sys.changes.animation_rerender_time = Some(1.0);
            }
                
            if clicked_node.text_id.is_some() {
                if let Some(text) = clicked_node.params.text_params{
                    if text.editable {
                        self.sys.focused = Some(clicked_id);
                    }
                }
            }

            if let Some(id) = clicked_node.text_id {
                let text_area = &mut self.sys.text.text_areas[id];
                let (x, y) = (
                    self.sys.part.mouse_pos.x - text_area.params.left,
                    self.sys.part.mouse_pos.y - text_area.params.top,
                );

                text_area.buffer.hit(x, y);
            }

        }
   
        let consumed = topmost_mouse_hit.is_some();
        return consumed;
    }

    pub(crate) fn scan_current_mouse_status(&mut self) -> MouseState {
        let topmost_mouse_hit = self.scan_mouse_hits();

        return MouseState {
            hit_node_id: topmost_mouse_hit,
            timestamp: Instant::now(),
            position: Xy::new(self.sys.part.mouse_pos.x, self.sys.part.mouse_pos.y),
        };
    }

    pub(crate) fn scan_mouse_hits(&mut self) -> Option<Id> {
        self.sys.mouse_hit_stack.clear();

        for rect in &self.sys.rects {
            if self.sys.part.mouse_hit_rect(rect) {
                self.sys.mouse_hit_stack.push((rect.id, rect.z));
            }
        }

        for rect in &self.sys.invisible_but_clickable_rects {
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

    /// Handles window events and updates the UI state.
    ///
    /// You should pass all events from winit to this method, unless they are "consumed" by something "above" the GUI.
    ///
    /// Returns `true` if the event was "consumed" by the `Ui`, e.g. if a mouse click hit an opaque panel.
    /// 
    pub fn handle_events(&mut self, full_event: &Event<()>) -> bool {
        if let Event::WindowEvent { event, .. } = full_event {
            match event {
                WindowEvent::CursorMoved { position, .. } => {
                    self.sys.part.mouse_pos.x = position.x as f32;
                    self.sys.part.mouse_pos.y = position.y as f32;
                    self.resolve_hover();
                    // cursormoved is never consumed
                }
                WindowEvent::MouseInput { button, state, .. } => {
                    // We have to test against all clickable rectangles immediately to know if the input is consumed or not
                    match state {
                        ElementState::Pressed => {
                            let consumed = self.resolve_click_press(*button);
                            return consumed;
                        },
                        ElementState::Released => {
                            self.resolve_click_release(*button);
                            // Consuming mouse releases can very easily mess things up for whoever is below us.
                            // Some unexpected mouse releases probably won't be too annoying.
                            return false
                        },
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
                // todo: 
                WindowEvent::Resized(size) => self.resize(size),
                _ => {}
            }
        }

        return false;
    }

    pub(crate) fn handle_keyboard_event(&mut self, event: &KeyEvent) -> bool {
        // todo: remove line.reset(); and do it only once per frame via change watcher guy

        if let Key::Named(named_key) = &event.logical_key { if named_key == &NamedKey::F1 {
            if event.state.is_pressed() && self.sys.debug_key_pressed == false {
                #[cfg(debug_assertions)]
                {

                    self.set_debug_mode(! self.debug_mode());
                }
            }

            self.sys.debug_key_pressed = event.state.is_pressed();
        } }

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

/// 
/// 

// this gets used for both presses and releases, but it doesn't keep a field to distinguish them, because it's always clear from the context.
// hit_node_id will always we Some for click presses, because otherwise they're fully ignored.
// Splitting them would probably be clearer.
#[derive(Clone, Copy, Debug)]
pub struct MouseState {
    pub position: Xy<f32>,
    pub timestamp: Instant,
    pub hit_node_id: Option<Id>,
}

#[derive(Clone, Copy, Debug)]
pub struct PendingMousePress {
    pub button: MouseButton,
    pub pressed_at: MouseState,
    pub last_seen: MouseState,
    pub already_released: bool,
}
impl PendingMousePress {
    pub fn new(event: MouseState, button: MouseButton) -> Self {
        return Self {
            button,
            pressed_at: event,
            last_seen: event,
            already_released: false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IsMouseReleased {
    MouseReleased,
    StillDownButFrameEnded,
}


/// A full description of a mouse event, from click to release.
#[derive(Clone, Copy, Debug)]
pub struct MouseEvent {
    pub button: MouseButton,
    pub originally_pressed: MouseState,
    pub last_seen: MouseState,
    pub currently_at: MouseState,
    pub kind: IsMouseReleased,
}
impl MouseEvent {
    // maybe a bit stupid compared to storing it explicitly, but should work.
    // if it stays there for more than 1 frame, the last_seen timestamp gets updated to the end of the frame.
    pub fn is_just_clicked(&self) -> bool {
        return self.originally_pressed.timestamp == self.last_seen.timestamp;
    }

    pub fn is_click_release(&self) -> bool {
        let is_click_release = self.kind == IsMouseReleased::MouseReleased;
        let is_on_same_node = self.originally_pressed.hit_node_id == self.currently_at.hit_node_id;
        return is_click_release && is_on_same_node;
    }

    pub fn drag_distance(&self) -> Xy<f32> {
        return self.last_seen.position - self.currently_at.position;
    }

    pub fn time_held(&self) -> Duration {
        return self.currently_at.timestamp.duration_since(self.last_seen.timestamp);
    }
}


// pub fn cursor_pos_from_byte_offset(buffer: &GlyphonBuffer, byte_offset: usize) -> (f32, f32) {
//     let line = &buffer.lines[0];
//     let buffer_line = line.layout_opt().as_ref().unwrap();
//     let glyphs = &buffer_line[0].glyphs;

//     // todo: faster?
//     for g in glyphs {
//         if g.start >= byte_offset {
//             return (g.x, g.y);
//         }
//     }

//     if let Some(glyph) = glyphs.last() {
//         return (glyph.x + glyph.w, glyph.y);
//     }

//     // string is empty
//     return (0.0, 0.0);
// }