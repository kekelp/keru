use std::time::{Duration, Instant};

use winit::{event::{KeyEvent, MouseButton}, keyboard::{Key, NamedKey}};

use crate::*;

pub(crate) const ANIMATION_RERENDER_TIME: f32 = 0.5;

impl Ui {
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

    /// Returns all [`FullMouseEvent`]s from the last frame.
    pub fn all_mouse_events(&self) -> impl DoubleEndedIterator<Item = &FullMouseEvent> {
        return self.sys.last_frame_mouse_events.iter();
    }

    /// Returns all [`FullMouseEvent`]s for a specific button on the node corresponding to `node_key`, or an empty iterator if the node is currently not part of the tree or if it doesn't exist.
    pub fn mouse_events(&self, mouse_button: MouseButton, node_key: NodeKey) -> impl DoubleEndedIterator<Item = &FullMouseEvent> {
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

    pub fn is_anything_dragged(&self) -> bool {
        let all_events = self.all_mouse_events();
        
        // I doubt anyone cares, but in the case the user dragged, released, and redragged, all in one frame, let's find all the distances and sum them.
        let mut dist = Xy::new_symm(0.0);
        
        for e in all_events {
            dist = dist + e.drag_distance();
        }

        if dist == Xy::new_symm(0.0) {
            return false;
        } else {
            return true;
        }
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

    pub fn mouse_held_in_general(&self, mouse_button: MouseButton) -> bool {
        let all_events = self
            .all_mouse_events();
            // .filter(move |c| c.button == mouse_button);

        
        let mut time_held = Duration::ZERO;
        
        println!("  We wuz panning");
        for e in all_events {
            println!("  {:?}", e);
            time_held += e.time_held();
        }

        if time_held == Duration::ZERO {
            return false;
        } else {
            return true;
        }
    }

    pub fn is_mouse_button_dragged_in_general(&self, mouse_button: MouseButton) -> (f64, f64) {
        let all_events = self
        .all_mouse_events()
        .filter(move |c| c.button == mouse_button);
        
        // I doubt anyone cares, but in the case the user dragged, released, and redragged, all in one frame, let's find all the distances and sum them.
        let mut dist = Xy::new_symm(0.0);
        
        for e in all_events {
            dist = dist + e.drag_distance();
        }

        if dist == Xy::new_symm(0.0) {
            return (0.0, 0.0);
        } else {
            return (dist.x as f64, dist.y as f64);
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
        // clicks
        self.sys.last_frame_mouse_events.clear();

        self.sys.unresolved_click_presses.retain(|click| click.already_released == false);

        // for each unresolved clickdown, push a partial drag/hold diff and update last_seen
        let mouse_current_status = self.scan_current_mouse_status();

        for click_pressed in self.sys.unresolved_click_presses.iter_mut().rev() {

            let mouse_happening = FullMouseEvent {
                button: click_pressed.button,
                originally_pressed: click_pressed.pressed_at,
                last_seen: click_pressed.last_seen,
                currently_at: mouse_current_status,
                kind: IsMouseReleased::StillDownButFrameEnded,
            };

            self.sys.last_frame_mouse_events.push(mouse_happening);

            click_pressed.last_seen = mouse_current_status;
        }

        if self.is_anything_dragged() {
            self.sys.is_anything_dragged = true;
        } else {
            self.sys.is_anything_dragged = false;
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

        self.sys.new_ui_input = true;

        if let Some(matched) = matched {
            // check for hits.
            let released_at = self.scan_current_mouse_status();

            let full_mouse_event = FullMouseEvent {
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
        self.sys.new_ui_input = true;

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
   
        let consumed = topmost_mouse_hit.is_some();
        return consumed;
    }

    pub(crate) fn scan_current_mouse_status(&mut self) -> MouseEvent {
        let topmost_mouse_hit = self.scan_mouse_hits();

        return MouseEvent {
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

/// A mouse event.
/// 
/// This can represent either a mouse click or a mouse release. This is only used inside `FullMouseEvent`, where this is always clear from the context.

// hit_node_id will always we Some for click presses, because otherwise they're fully ignored.
// Splitting them would probably be clearer.
#[derive(Clone, Copy, Debug)]
pub struct MouseEvent {
    pub position: Xy<f32>,
    pub timestamp: Instant,
    pub hit_node_id: Option<Id>,
}

/// A mouse press that has to be matched to a future mouse release.
/// 
/// Not part of the public API.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PendingMousePress {
    pub button: MouseButton,
    pub pressed_at: MouseEvent,
    pub last_seen: MouseEvent,
    pub already_released: bool,
}
impl PendingMousePress {
    pub fn new(event: MouseEvent, button: MouseButton) -> Self {
        return Self {
            button,
            pressed_at: event,
            last_seen: event,
            already_released: false,
        }
    }
}

/// Information about a [`FullMouseEvent`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IsMouseReleased {
    /// The mouse was released, and this event will be reported for the last time on this frame.
    MouseReleased,
    /// The mouse is still being held down, and it was reported at the end of the frame.
    StillDownButFrameEnded,
}


/// A full description of a mouse event tracked for multiple frames, from click to release.
/// 
/// Usually there's no need to use this struct directly, as you can use [`Ui::is_clicked`] and similar methods. But for advanced uses, you can obtain an iterator of `FullMouseEvent`s from [`Ui::all_mouse_events`] or [`Ui::mouse_events`].
/// 
/// You can use the [`FullMouseEvent::is_just_clicked`] and the other methods to map these events into more familiar concepts.
#[derive(Clone, Copy, Debug)]
pub struct FullMouseEvent {
    pub button: MouseButton,
    pub originally_pressed: MouseEvent,
    pub last_seen: MouseEvent,
    pub currently_at: MouseEvent,
    pub kind: IsMouseReleased,
}
impl FullMouseEvent {
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
