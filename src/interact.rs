use std::time::Duration;

use glam::Vec2;
use winit::{dpi::PhysicalPosition, event::{KeyEvent, MouseButton, MouseScrollDelta}, keyboard::{Key, NamedKey}, window::Window};

use crate::*;
use crate::Axis::{X, Y};
use crate::mouse_events::SmallVec;

pub(crate) const ANIMATION_RERENDER_TIME: f32 = 0.5;

/// A struct describing a click event on a GUI node.
#[derive(Clone, Copy, Debug)]
pub struct Click {
    /// Absolute screen position in pixels
    pub absolute_position: glam::Vec2,
    /// Position relative to the node (0.0 to 1.0 in each dimension)
    pub relative_position: glam::Vec2,
    /// Timestamp of the click
    pub timestamp: std::time::Instant,
}

/// A struct describing a hover event on a GUI node.
#[derive(Clone, Copy, Debug)]
pub struct Hover {
    /// Absolute screen position in pixels
    pub absolute_position: glam::Vec2,
}

/// A struct describing a drag event on a GUI node.
#[derive(Clone, Copy, Debug)]
pub struct Drag {
    /// Absolute screen position in pixels
    pub absolute_pos: Vec2,
    /// Position relative to the node (0.0 to 1.0 in each dimension)
    pub relative_position: Vec2,
    /// Absolute delta movement in pixels
    pub absolute_delta: Vec2,
    /// Delta movement relative to the node's dimensions (as a fraction)
    pub relative_delta: Vec2,
    /// Time when the drag event started
    pub pressed_timestamp: std::time::Instant,
    /// Total absolute drag in pixels since the start of the drag event
    pub total_drag_distance: Vec2,
}

/// A struct describing a scroll event on a GUI node.
#[derive(Clone, Copy, Debug)]
pub struct ScrollEvent {
    /// Absolute screen position in pixels where the scroll occurred
    pub absolute_position: glam::Vec2,
    /// Position relative to the node (0.0 to 1.0 in each dimension)
    pub relative_position: glam::Vec2,
    /// Scroll delta (positive Y is scroll up, negative Y is scroll down)
    pub delta: glam::Vec2,
    /// Timestamp of the scroll event
    pub timestamp: std::time::Instant,
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct ClickRect {
    pub rect: XyRect,
    pub i: NodeI,
    pub senses: Sense,
    pub scrollable: Xy<bool>,
    pub absorbs_mouse_events: bool,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct Sense: u8 {
        const CLICK = 1 << 0;
        const DRAG  = 1 << 1;
        const HOVER = 1 << 2;
        const SCROLL = 1 << 3;
        const HOLD  = 1 << 4;
        const CLICK_RELEASE = 1 << 5;
        const DRAG_DROP_TARGET = 1 << 6;

        const NONE = 0;
    }
}

impl Ui {
    pub(crate) fn click_rect(&self, i: NodeI) -> ClickRect {
        let real_rect = self.nodes[i].real_rect;
        let transform = self.nodes[i].accumulated_transform;
        let size = self.sys.size;

        // Apply transform
        let tx_norm = transform.offset.x / size[X];
        let ty_norm = transform.offset.y / size[Y];

        let transformed_rect = XyRect::new(
            [real_rect[X][0] * transform.scale + tx_norm, real_rect[X][1] * transform.scale + tx_norm],
            [real_rect[Y][0] * transform.scale + ty_norm, real_rect[Y][1] * transform.scale + ty_norm],
        );

        // Clip the transformed rect to the node's clip_rect
        let clip_rect = self.nodes[i].clip_rect;
        let clipped_rect = XyRect::new(
            intersect(transformed_rect[X], clip_rect[X]),
            intersect(transformed_rect[Y], clip_rect[Y]),
        );

        ClickRect {
            rect: clipped_rect,
            i,
            senses: self.nodes[i].params.interact.senses,
            scrollable: self.nodes[i].params.layout.scrollable,
            absorbs_mouse_events: self.nodes[i].params.interact.absorbs_mouse_events,
        }
    }

    /// Scan for any interactive node under cursor (for general hover detection)
    pub(crate) fn scan_opaque_hits(&self) -> SmallVec<Id> {
        let mut result = SmallVec::new();

        for clk_i in (0..self.sys.click_rects.len()).rev() {
            let rect = &self.sys.click_rects[clk_i];

            if ! self.hit_click_rect(rect) {
                continue;
            }

            let is_interactive = rect.senses != Sense::NONE
                || rect.scrollable[X] || rect.scrollable[Y]
                || rect.absorbs_mouse_events;

            if is_interactive {
                result.push(self.nodes[rect.i].id);
            }

            if rect.absorbs_mouse_events {
                break;
            }
        }

        result
    }

    /// Scan for nodes with a specific sense. Only stops at absorbing nodes that have the sense.
    /// If an absorbing node without the sense is encountered, walks up the parent tree instead
    /// of continuing to scan siblings/unrelated nodes.
    pub(crate) fn scan_hits_with_sense(&self, sense: Sense) -> SmallVec<Id> {
        let mut result = SmallVec::new();

        for clk_i in (0..self.sys.click_rects.len()).rev() {
            let rect = &self.sys.click_rects[clk_i];

            if ! self.hit_click_rect(rect) {
                continue;
            }

            // If this node has the sense, add it
            if rect.senses.contains(sense) {
                result.push(self.nodes[rect.i].id);
            }

            // If this is an absorbing node
            if rect.absorbs_mouse_events {
                if rect.senses.contains(sense) {
                    // Absorbing node with the sense - stop completely
                    break;
                } else {
                    // Absorbing node without the sense - walk up the parent tree
                    let mut current_i = self.nodes[rect.i].parent;
                    while current_i != ROOT_I {
                        let parent_rect = self.click_rect(current_i);
                        if self.hit_click_rect(&parent_rect) {
                            if parent_rect.senses.contains(sense) {
                                result.push(self.nodes[current_i].id);
                            }
                            if parent_rect.absorbs_mouse_events {
                                break;
                            }
                        }
                        current_i = self.nodes[current_i].parent;
                    }
                    break; // Exit main loop after parent walking
                }
            }
        }

        result
    }

    #[cfg(debug_assertions)]
    pub(crate) fn scan_any_node_hits(&self) -> SmallVec<Id> {
        let mut result = SmallVec::new();

        for clk_i in (0..self.sys.click_rects.len()).rev() {
            let rect = &self.sys.click_rects[clk_i];

            if self.hit_click_rect(rect) {
                result.push(self.nodes[rect.i].id);

                if rect.absorbs_mouse_events {
                    break;
                }
            }
        }

        result
    }

    pub(crate) fn resolve_hover(&mut self) {
        let hovered_ids = self.scan_opaque_hits();

        // Handle nodes that are no longer hovered
        for i in 0..self.sys.hovered.len() {
            let old_id = self.sys.hovered[i];
            if !hovered_ids.contains(&old_id) {
                self.end_hovering(old_id);
            }
        }

        // Handle newly hovered nodes
        for &id in &hovered_ids {
            if !self.sys.hovered.contains(&id) {
                self.start_hovering(id);
            } else {
                // Already hovered - check if we need to signal input
                if let Some(entry) = self.nodes.node_hashmap.get(&id) {
                    if self.nodes[entry.slab_i].params.interact.senses.contains(Sense::HOVER) {
                        self.set_new_ui_input();
                    }
                }
            }
        }

        self.sys.hovered.retain(|id| hovered_ids.contains(id));

        // Check for ongoing drags
        let has_drag = self.sys.mouse_input.currently_dragging().next().is_some();
        if has_drag {
            self.set_new_ui_input();
        }

        // Debug mode: track all hits for inspection
        #[cfg(debug_assertions)]
        if self.inspect_mode() {
            let all_hits = self.scan_any_node_hits();
            if let Some(&new_id) = all_hits.first() {
                if self.sys.inspect_hovered.first() != Some(&new_id) {
                    if let Some(entry) = self.nodes.node_hashmap.get(&new_id) {
                        log::info!("Inspect mode: hovering {}", self.node_debug_name_fmt_scratch(entry.slab_i));
                    }
                }
            }
            self.sys.inspect_hovered = all_hits;
        }
    }

    fn start_hovering(&mut self, id: Id) {
        self.sys.hovered.push(id);

        let (has_hover_sense, has_click_animation) = {
            if let Some((node, _)) = self.nodes.get_mut_by_id(&id) {
                let has_hover = node.params.interact.senses.contains(Sense::HOVER);
                let has_anim = node.params.interact.click_animation;
                if has_anim {
                    node.hovered = true;
                    node.hover_timestamp = slow_accurate_timestamp_for_events_only();
                }
                (has_hover, has_anim)
            } else {
                (false, false)
            }
        };

        if has_hover_sense {
            self.set_new_ui_input();
        }
        if has_click_animation {
            self.sys.changes.rebuild_render_data = true;
            self.sys.anim_render_timer.push_new(Duration::from_secs_f32(ANIMATION_RERENDER_TIME));
        }
    }

    fn end_hovering(&mut self, id: Id) {
        if let Some((node, _)) = self.nodes.get_mut_by_id(&id) {
            if node.last_frame_touched == self.sys.current_frame && node.params.interact.click_animation {
                node.hovered = false;
                node.hover_timestamp = slow_accurate_timestamp_for_events_only();
                self.sys.changes.rebuild_render_data = true;
                self.sys.anim_render_timer.push_new(Duration::from_secs_f32(ANIMATION_RERENDER_TIME));
            }
        }
    }

    pub(crate) fn begin_frame_resolve_inputs(&mut self) {
        self.sys.mouse_input.begin_new_frame();
        self.sys.key_input.begin_new_frame();

        self.sys.text_edit_changed_last_frame = self.sys.text_edit_changed_this_frame;
        self.sys.text_edit_changed_this_frame = None;
    }

    pub(crate) fn handle_mouse_press(&mut self, button: MouseButton, window: &Window) -> bool {
        self.sys.focused = None;

        let click_ids = self.scan_hits_with_sense(Sense::CLICK);
        let drag_ids = self.scan_hits_with_sense(Sense::DRAG);

        self.sys.mouse_input.push_press(button, click_ids.clone(), drag_ids);

        // todo: instead of re-iterating, maybe do this while scanning?
        let mut any_consumed = false;
        for &id in &click_ids {
            if let Some(entry) = self.nodes.node_hashmap.get(&id) {
                let i = entry.slab_i;
                let consumed = self.resolve_click_press(button, window, i);
                any_consumed = any_consumed || consumed;
            }
        }

        return any_consumed;
    }

    pub(crate) fn handle_mouse_release(&mut self, button: MouseButton) {
        let click_ids = self.scan_hits_with_sense(Sense::CLICK);
        self.sys.mouse_input.push_release(button, click_ids.clone());

        // todo: instead of re-iterating, maybe do this while scanning?
        // Signal update if any relevant nodes
        for &id in &click_ids {
            if let Some(entry) = self.nodes.node_hashmap.get(&id) {
                let senses = self.nodes[entry.slab_i].params.interact.senses;
                if senses.contains(Sense::CLICK_RELEASE) || senses.contains(Sense::DRAG) {
                    self.set_new_ui_input();
                }
            }
        }
    }

    fn resolve_click_press(&mut self, button: MouseButton, _window: &Window, i: NodeI) -> bool {
        let id = self.nodes[i].id;

        if self.nodes[i].params.interact.senses.contains(Sense::CLICK) {
            self.set_new_ui_input();
        }

        if button == MouseButton::Left {
            let t = T0.elapsed().as_secs_f32();

            if self.nodes[i].params.interact.click_animation {
                self.nodes[i].last_click = t;
                self.sys.changes.rebuild_render_data = true;
                self.sys.anim_render_timer.push_new(Duration::from_secs_f32(ANIMATION_RERENDER_TIME));
            }

            if let Some(text_i) = &self.nodes[i].text_i {
                if matches!(text_i, TextI::TextEdit(_)) {
                    self.sys.focused = Some(id);
                }
            }
        }

        return self.nodes[i].params.interact.absorbs_mouse_events;
    }

    pub(crate) fn handle_keyboard_event(&mut self, event: &KeyEvent) -> bool {
        if let Key::Named(NamedKey::F1) = &event.logical_key {
            if event.state.is_pressed() && !self.sys.debug_key_pressed {
                #[cfg(debug_assertions)]
                {
                    self.set_inspect_mode(!self.inspect_mode());
                    self.set_new_ui_input();
                }
            }
            self.sys.debug_key_pressed = event.state.is_pressed();
        }
        false
    }

    pub(crate) fn handle_scroll_event(&mut self, delta: &MouseScrollDelta) {
        // Find the topmost hit node, then walk up to find scroll target
        let hovered_ids = self.scan_opaque_hits();
        let Some(&first_id) = hovered_ids.first() else {
            return;
        };
        let Some(entry) = self.nodes.node_hashmap.get(&first_id) else {
            return;
        };
        let hover_i = entry.slab_i;

        let (dx, dy) = match delta {
            MouseScrollDelta::LineDelta(x, y) => (x * 0.1, y * 0.1),
            MouseScrollDelta::PixelDelta(PhysicalPosition { x, y }) => (*x as f32, *y as f32),
        };
        let fdelta = Xy::new(dx, dy);

        // Walk up to find a scroll target
        let mut scroll_target: Option<(NodeI, bool)> = None; // (index, is_sense_target)

        for axis in [X, Y] {
            if fdelta[axis] == 0.0 {
                continue;
            }

            let mut current_i = hover_i;
            loop {
                // Check for SCROLL sense first
                if self.nodes[current_i].params.interact.senses.contains(Sense::SCROLL) {
                    scroll_target = Some((current_i, true));
                    break;
                }

                // Then check for scrollable container
                if self.nodes[current_i].params.layout.scrollable[axis] {
                    scroll_target = Some((current_i, false));
                    break;
                }

                let parent_i = self.nodes[current_i].parent;
                if parent_i == ROOT_I {
                    break;
                }
                current_i = parent_i;
            }
        }

        if let Some((target_i, is_sense)) = scroll_target {
            if is_sense {
                let id = self.nodes[target_i].id;
                let scroll_delta = match delta {
                    MouseScrollDelta::LineDelta(x, y) => Vec2::new(*x * 0.1, *y * 0.1),
                    MouseScrollDelta::PixelDelta(p) => Vec2::new(p.x as f32, p.y as f32),
                };
                self.sys.mouse_input.push_scroll(scroll_delta, id);
                self.set_new_ui_input();
            } else {
                self.update_container_scroll(target_i, fdelta[Y], Y);
                self.recursive_place_children(target_i);
                self.sys.changes.text_changed = true;
                self.resolve_hover();
                self.sys.changes.need_gpu_rect_update = true;
                self.sys.changes.need_rerender = true;
            }
        }
    }

    // Query methods used by ui_node.rs

    pub(crate) fn check_hovered(&self, id: Id) -> bool {
        self.sys.hovered.contains(&id)
    }

    pub(crate) fn check_clicked(&self, id: Id, button: MouseButton) -> bool {
        self.sys.mouse_input.clicks()
            .any(|e| e.button == button && e.targets.contains(&id))
    }

    pub(crate) fn check_clicked_at(&self, id: Id, button: MouseButton) -> Option<&mouse_events::ClickEvent> {
        self.sys.mouse_input.clicks()
            .filter(|e| e.button == button && e.targets.contains(&id))
            .last()
    }

    pub(crate) fn check_click_released(&self, id: Id, button: MouseButton) -> bool {
        // Click events are emitted on release when released on same target
        self.check_clicked(id, button)
    }

    pub(crate) fn check_dragged(&self, id: Id, button: MouseButton) -> Option<&mouse_events::DragEvent> {
        self.sys.mouse_input.drags()
            .find(|e| e.button == button && e.targets.contains(&id))
    }

    pub(crate) fn check_drag_released(&self, id: Id, button: MouseButton) -> bool {
        self.sys.mouse_input.drag_releases()
            .any(|e| e.button == button && e.targets.contains(&id))
    }

    pub(crate) fn check_drag_released_onto(&self, src_id: Id, dest_id: Id, button: MouseButton) -> Option<&mouse_events::DragReleaseEvent> {
        // Check if dest is reachable as a drop target
        let drop_targets = self.scan_hits_with_sense(Sense::DRAG_DROP_TARGET);
        if !drop_targets.contains(&dest_id) {
            return None;
        }

        self.sys.mouse_input.drag_releases()
            .find(|e| e.button == button && e.targets.contains(&src_id))
    }

    pub(crate) fn check_drag_hovered_onto(&self, src_id: Id, dest_id: Id, button: MouseButton) -> Option<&mouse_events::DragEvent> {
        // Check if dest is reachable as a drop target
        let drop_targets = self.scan_hits_with_sense(Sense::DRAG_DROP_TARGET);
        if !drop_targets.contains(&dest_id) {
            return None;
        }

        self.sys.mouse_input.drags()
            .find(|e| e.button == button && e.targets.contains(&src_id))
    }

    pub(crate) fn check_held_duration(&self, id: Id, button: MouseButton) -> Option<Duration> {
        // Hold is tracked via drag events - duration since start
        self.sys.mouse_input.drags()
            .find(|e| e.button == button && e.targets.contains(&id))
            .map(|e| e.start_time.elapsed())
    }

    pub(crate) fn check_scrolled(&self, id: Id) -> Option<Vec2> {
        let mut total = Vec2::ZERO;
        let mut found = false;
        for e in self.sys.mouse_input.scrolls() {
            if e.target == id {
                total += e.delta;
                found = true;
            }
        }
        if found { Some(total) } else { None }
    }

    pub(crate) fn check_last_scroll_event(&self, id: Id) -> Option<&mouse_events::ScrollEvent> {
        self.sys.mouse_input.scrolls()
            .filter(|e| e.target == id)
            .last()
    }

    pub(crate) fn global_scroll_delta(&self) -> Option<Vec2> {
        let mut total = Vec2::ZERO;
        let mut found = false;
        for e in self.sys.mouse_input.scrolls() {
            total += e.delta;
            found = true;
        }
        if found { Some(total) } else { None }
    }

    /// Find any drag hovering onto dest (from any source)
    pub(crate) fn check_any_drag_hovered_onto(&self, dest_id: Id, button: MouseButton) -> Option<&mouse_events::DragEvent> {
        let drop_targets = self.scan_hits_with_sense(Sense::DRAG_DROP_TARGET);
        if !drop_targets.contains(&dest_id) {
            return None;
        }

        self.sys.mouse_input.drags()
            .find(|e| e.button == button)
    }

    /// Find any drag released onto dest (from any source)
    pub(crate) fn check_any_drag_released_onto(&self, dest_id: Id, button: MouseButton) -> Option<&mouse_events::DragReleaseEvent> {
        let drop_targets = self.scan_hits_with_sense(Sense::DRAG_DROP_TARGET);
        if !drop_targets.contains(&dest_id) {
            return None;
        }

        self.sys.mouse_input.drag_releases()
            .find(|e| e.button == button)
    }
}
