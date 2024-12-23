use std::time::{Duration, Instant};

use winit::{event::{KeyEvent, MouseButton}, keyboard::{Key, NamedKey}};
use winit_mouse_events::MouseInput;

use crate::*;

pub(crate) const ANIMATION_RERENDER_TIME: f32 = 0.5;

impl Ui {
    pub fn mouse_input(&self) -> &MouseInput<Id> {
        return &self.sys.mouse_input;
    }

    pub fn is_clicked(&self, node_key: NodeKey) -> bool {
        return self.sys.mouse_input.clicked(Some(MouseButton::Left), Some(node_key.id));
    }

    pub fn all_key_events(&self) -> impl DoubleEndedIterator<Item = &FullKeyEvent> {
        return self.sys.last_frame_key_events.iter();
    }

    pub fn key_events(&self, key: Key) -> impl DoubleEndedIterator<Item = &FullKeyEvent> {
        return self
            .all_key_events()
            .filter(move |c| c.key == key);    }

    pub fn key_pressed(&self, key: Key) -> bool {
        let all_events = self.key_events(key);
        let count = all_events.filter(|c| c.is_just_pressed()).count();
        return count > 0;
    }

    pub fn time_key_held(&self, key: Key) -> Option<Duration> {
        let all_events = self.key_events(key);

        let mut time_held = Duration::ZERO;

        for e in all_events {
            time_held += e.time_held();
        }

        if time_held == Duration::ZERO {
            return None;
        } else {
            return Some(time_held);
        }
    }

    // todo: could simplify
    pub fn key_held(&self, key: Key) -> bool {
        let duration = self.time_key_held(key);
        return duration > Some(Duration::ZERO);
    }

    /// Returns `true` if a node is currently hovered by the cursor.
    pub fn is_hovered(&self, node_key: NodeKey) -> bool {
        return self.sys.hovered.last() == Some(&node_key.id);
    }

    // todo: think if it's really worth it to do this on every mouse movement.
    // maybe add a global setting to do it just once per frame
    pub(crate) fn resolve_hover(&mut self) {

        if let Some(hovered_id) = self.sys.mouse_input.current_tag() {
            if self.sys.hovered.contains(&hovered_id) {
                // nothing changed, do nothing
            } else {
                // newly entered
                self.end_all_hovering();
                self.start_hovering(hovered_id);
                self.sys.new_ui_input = true;
            }

        } else {
            self.end_all_hovering();
        }

        if self.sys.is_anything_dragged {
            self.sys.new_ui_input = true;
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
            self.sys.anim_render_timer.push_new(Duration::from_secs_f32(ANIMATION_RERENDER_TIME));
        }

    }

    pub(crate) fn end_all_hovering(&mut self) {
        if ! self.sys.hovered.is_empty() {
            self.sys.anim_render_timer.push_new(Duration::from_secs_f32(ANIMATION_RERENDER_TIME));
        }

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
                    }

                    self.sys.new_ui_input = true;
                }
            }
        }
        self.sys.hovered.clear();
    }

    pub(crate) fn begin_frame_resolve_inputs(&mut self) {
        self.sys.mouse_input.begin_new_frame();

        if self.sys.mouse_input.dragged(Some(MouseButton::Left), None) != (0.0, 0.0) {
            self.sys.is_anything_dragged = true;
        } else {
            self.sys.is_anything_dragged = false;
        }
    }

    pub(crate) fn resolve_click_release(&mut self, button: MouseButton) {
        self.sys.new_ui_input = true;
    }

    // returns if the ui consumed the mouse press, or if it should be passed down. 
    pub(crate) fn resolve_click_press(&mut self, button: MouseButton) -> bool {
        self.sys.new_ui_input = true;

        // defocus, so that we defocus when clicking anywhere outside.
        // if we're clicking something we'll re-focus below.
        self.sys.focused = None;

        // if nothing is hit, we're done.
        let Some(clicked_id) = self.sys.mouse_input.current_tag() else {
            return false;
        };
        
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
                
                self.sys.anim_render_timer.push_new(Duration::from_secs_f32(ANIMATION_RERENDER_TIME));
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
   
        let consumed = self.sys.mouse_input.current_tag().is_some();
        return consumed;
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

    pub(crate) fn handle_keyboard_event(&mut self, event: &KeyEvent) -> bool {
        let now = Instant::now();
        if event.state.is_pressed() {
            let pending_press = PendingKeyPress::new(now, &event);
            self.sys.unresolved_key_presses.push(pending_press);
        } else {
            // look for a mouse press to match and resolve
            let mut matched = None;
            for key_pressed in self.sys.unresolved_key_presses.iter_mut().rev() {
                if key_pressed.key == event.logical_key {
                    key_pressed.already_released = true;
                    matched = Some(key_pressed.clone());
                    break;
                }
            };

            self.sys.new_ui_input = true;

            if let Some(matched) = matched {
                let full_key_event = FullKeyEvent {
                    key: event.logical_key.clone(),
                    originally_pressed: matched.pressed_at,
                    last_seen: matched.last_seen,
                    currently_at: now,
                    kind: IsKeyReleased::KeyReleased,
                };

                self.sys.last_frame_key_events.push(full_key_event);
            }
        }

        if let Key::Named(named_key) = &event.logical_key {
            if named_key == &NamedKey::F1 {
                if event.state.is_pressed() && self.sys.debug_key_pressed == false {
                    #[cfg(debug_assertions)]
                    {
                        self.set_debug_mode(!self.debug_mode());
                        self.sys.new_ui_input = true;
                    }
                }

                self.sys.debug_key_pressed = event.state.is_pressed();
            }
        }

        return false;
    }
}



#[derive(Clone, Debug)]
pub(crate) struct PendingKeyPress {
    pub key: Key,
    pub pressed_at: Instant,
    pub last_seen: Instant,
    pub already_released: bool,
}
impl PendingKeyPress {
    pub fn new(timestamp: Instant, key: &KeyEvent) -> Self {
        return Self {
            key: key.logical_key.clone(),
            pressed_at: timestamp,
            last_seen: timestamp,
            already_released: false,
        }
    }
}

// todo: merge with the mouse one??
/// Information about a [`FullKeyEvent`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IsKeyReleased {
    /// The key was released, and this event will be reported for the last time on this frame.
    KeyReleased,
    /// The key is still being held down, and it was reported at the end of the frame.
    StillDownButFrameEnded,
}


#[derive(Clone, Debug)]
pub struct FullKeyEvent {
    pub key: Key,
    pub originally_pressed: Instant,
    pub last_seen: Instant,
    // rename to current_time or something, or maybe remove?
    pub currently_at: Instant,
    pub kind: IsKeyReleased,
}
impl FullKeyEvent {
    // if it stays there for more than 1 frame, the last_seen timestamp gets updated to the end of the frame.
    pub fn is_just_pressed(&self) -> bool {
        return self.originally_pressed == self.last_seen;
    }

    pub fn is_pressed_release(&self) -> bool {
        let is_pressed_release = self.kind == IsKeyReleased::KeyReleased;
        return is_pressed_release;
    }

    pub fn time_held(&self) -> Duration {
        return self.currently_at.duration_since(self.last_seen);
    }
}
