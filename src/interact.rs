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
    /// Position relative to the node (0.0 to 1.0 in each dimension)
    pub relative_position: glam::Vec2,
    /// Timestamp of the latest hover-enter or hover-exit event on this node
    pub last_enter_or_exit: Option<std::time::Instant>,
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
pub struct Scroll {
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

/// A single node hit by the cursor, recorded by [`System::scan_all_hits`].
/// Ordered topmost-first. Carries enough info that callers can filter for the
/// senses they care about without rescanning.
#[derive(Debug, Copy, Clone)]
pub(crate) struct HitNode {
    pub id: Id,
    pub senses: Sense,
    pub absorbs_mouse_events: bool,
}

/// Pick out the nodes that have a given sense from a [`System::scan_all_hits`]
/// result, stopping at the first absorbing node that has the sense (which
/// consumes the event). Reproduces [`System::scan_hits_with_sense`].
pub(crate) fn filter_hits_by_sense(hits: &[HitNode], sense: Sense) -> SmallVec<Id> {
    let mut result = SmallVec::new();
    for hit in hits {
        if hit.senses.contains(sense) {
            result.push(hit.id);
        }
        if hit.absorbs_mouse_events && hit.senses.contains(sense) {
            break;
        }
    }
    result
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct Sense: u16 {
        const CLICK = 1 << 0;
        const DRAG  = 1 << 1;
        const HOVER = 1 << 2;
        const SCROLL = 1 << 3;
        const HOLD  = 1 << 4;
        const CLICK_RELEASE = 1 << 5;
        const DRAG_DROP_TARGET = 1 << 6;
        /// Hints that the winit loop should never go to sleep as long as this node is visible.
        const TIME = 1 << 7;
        /// Like HOVER, but only wakes up the event loop when the hover state changes (enter or exit),
        /// not on every mouse move while already hovering.
        const HOVER_ENTER_OR_EXIT = 1 << 8;

        const NONE = 0;
    }
}

impl Ui {
    /// Scan for any interactive node under cursor (for general hover detection)
    pub(crate) fn scan_opaque_hits(&self) -> SmallVec<Id> {
        let mut result = SmallVec::new();

        for clk_i in (0..self.sys.click_rects.len()).rev() {
            let rect = &self.sys.click_rects[clk_i];

            if ! self.sys.hit_click_rect(rect) {
                continue;
            }

            let is_interactive = rect.senses != Sense::NONE
                || rect.scrollable[X] || rect.scrollable[Y]
                || rect.absorbs_mouse_events;

            if is_interactive {
                result.push(self.sys.nodes[rect.i].id);
            }

            if rect.absorbs_mouse_events {
                break;
            }
        }

        result
    }

    #[cfg(debug_assertions)]
    pub(crate) fn scan_any_node_hits(&self) -> SmallVec<Id> {
        let mut result = SmallVec::new();

        for clk_i in (0..self.sys.click_rects.len()).rev() {
            let rect = &self.sys.click_rects[clk_i];

            if self.sys.hit_click_rect(rect) {
                result.push(self.sys.nodes[rect.i].id);

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
                // Already hovered - only wake up for HOVER (not HOVER_ENTER_OR_EXIT, which is enter/exit only)
                if let Some(i) = self.sys.nodes.get_by_id(id) {
                    if self.sys.nodes[i].params.interact.senses.contains(Sense::HOVER) {
                        self.set_new_ui_input();
                    }
                }
            }
        }

        self.sys.hovered.retain(|id| hovered_ids.contains(id));

        // Debug mode: track all hits for inspection
        #[cfg(debug_assertions)]
        if self.inspect_mode() {
            let all_hits = self.scan_any_node_hits();
            if let Some(&new_id) = all_hits.first() {
                if self.sys.inspect_hovered.first() != Some(&new_id) {
                    if let Some(i) = self.sys.nodes.get_by_id(new_id) {
                        log::info!("Inspect mode: hovering {}", self.node_debug_name(i));
                    }
                }
            }
            self.sys.inspect_hovered = all_hits;
        }
    }

    fn start_hovering(&mut self, id: Id) {
        self.sys.hovered.push(id);

        let (has_hover_sense, has_click_animation) = {
            if let Some(i) = self.sys.nodes.get_by_id(id) {
                let node = &mut self.sys.nodes[i];
                let senses = node.params.interact.senses;
                let has_hover = senses.intersects(Sense::HOVER | Sense::HOVER_ENTER_OR_EXIT);
                let has_anim = node.params.interact.click_animation;
                if has_anim {
                    node.hovered = true;
                    node.hover_timestamp = slow_accurate_timestamp_for_events_only();
                }
                node.hover_enter_exit_instant = Some(std::time::Instant::now());
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
        if let Some(i) = self.sys.nodes.get_by_id(id) {
            let node = &mut self.sys.nodes[i];
            let senses = node.params.interact.senses;
            if node.last_frame_touched == self.sys.current_frame && node.params.interact.click_animation {
                node.hovered = false;
                node.hover_timestamp = slow_accurate_timestamp_for_events_only();
                self.sys.changes.rebuild_render_data = true;
                self.sys.anim_render_timer.push_new(Duration::from_secs_f32(ANIMATION_RERENDER_TIME));
            }
            node.hover_enter_exit_instant = Some(std::time::Instant::now());
            if senses.contains(Sense::HOVER_ENTER_OR_EXIT) {
                self.set_new_ui_input();
            }
        }
    }

    pub(crate) fn begin_frame_resolve_inputs(&mut self) {
        self.sys.mouse_input.begin_new_frame();
        self.sys.key_input.begin_new_frame();
    }

    pub(crate) fn handle_mouse_press(&mut self, button: MouseButton, window: &Window) -> bool {
        // Single scan of everything under the cursor. The code below picks out
        // the senses it cares about from this one list.
        let hits = self.sys.scan_all_hits();

        let click_ids = filter_hits_by_sense(&hits, Sense::CLICK);
        let drag_ids = filter_hits_by_sense(&hits, Sense::DRAG);

        self.sys.mouse_input.push_press(button, click_ids.clone(), drag_ids);

        self.resolve_focus_on_press(hits.first());

        let mut any_consumed = false;
        for &id in &click_ids {
            if let Some(i) = self.sys.nodes.get_by_id(id) {
                let consumed = self.resolve_click_press(button, window, i);
                any_consumed = any_consumed || consumed;
            }
        }

        return any_consumed;
    }

    pub(crate) fn handle_mouse_release(&mut self, button: MouseButton) {
        let click_ids = self.sys.scan_hits_with_sense(Sense::CLICK);
        self.sys.mouse_input.push_release(button, click_ids.clone());

        // todo: instead of re-iterating, maybe do this while scanning?
        // Signal update if any relevant nodes
        for &id in &click_ids {
            if let Some(i) = self.sys.nodes.get_by_id(id) {
                let senses = self.sys.nodes[i].params.interact.senses;
                if senses.intersects(Sense::CLICK_RELEASE | Sense::DRAG | Sense::DRAG_DROP_TARGET) {
                    self.set_new_ui_input();
                }
            }
        }
    }

    /// Move focus to a node on press. Runs for the topmost hit node regardless
    /// of its senses. Clicking empty space (no hit) clears the focus.
    fn resolve_focus_on_press(&mut self, hit: Option<&HitNode>) {
        let Some(hit) = hit else {
            self.sys.focused = None;
            self.sys.renderer.text.clear_focus();
            return;
        };
        let Some(i) = self.sys.nodes.get_by_id(hit.id) else {
            return;
        };

        let prev_focused = self.sys.focused;

        let interactable = self.is_interactable_for_focus(i);
        if interactable {
            let currently_showing_indicator = self.sys.show_focus_indicator;
            self.set_focus_node(i, currently_showing_indicator);
        } else {
            // "Focus" the node anyway so that tab navigation can start from here.
            // It's harmless to focus it if it's non-interactable.
            self.set_focus_node(i, false);
        }

        if self.sys.focused != prev_focused {
            self.set_new_ui_input();
        }
    }

    fn resolve_click_press(&mut self, button: MouseButton, _window: &Window, i: NodeI) -> bool {
        if self.sys.nodes[i].params.interact.senses.contains(Sense::CLICK) {
            self.set_new_ui_input();
        }

        if button == MouseButton::Left {
            let t = T0.elapsed().as_secs_f32();

            if self.sys.nodes[i].params.interact.click_animation {
                self.sys.nodes[i].last_click = t;
                self.sys.changes.rebuild_render_data = true;
                self.sys.anim_render_timer.push_new(Duration::from_secs_f32(ANIMATION_RERENDER_TIME));
            }
        }

        return self.sys.nodes[i].params.interact.absorbs_mouse_events;
    }

    pub(crate) fn handle_keyboard_event(&mut self, event: &KeyEvent) -> bool {
        if let Key::Named(NamedKey::F1) = &event.logical_key {
            #[cfg(debug_assertions)]
            if event.state.is_pressed() && !self.sys.debug_key_pressed {
                self.set_inspect_mode(!self.inspect_mode());
                self.set_new_ui_input();
            }
            self.sys.debug_key_pressed = event.state.is_pressed();
        }

        if let Key::Named(NamedKey::Tab) = &event.logical_key {
            if event.state.is_pressed() {
                let forward = !self.sys.key_input.key_mods().shift_key();
                self.move_keyboard_focus(forward);
                return true;
            }
        }

        if let Key::Named(NamedKey::Escape) = &event.logical_key {
            if event.state.is_pressed() && self.sys.show_focus_indicator {
                // Hide the focus indicator without losing the focus itself, so a
                // subsequent Tab resumes navigation from the same node.
                self.sys.show_focus_indicator = false;
                self.sys.changes.should_rebuild_render_data = true;
                self.set_new_ui_input();
                return true;
            }
        }

        if let Key::Named(NamedKey::Space | NamedKey::Enter) = &event.logical_key {
            if event.state.is_pressed() {
                if let Some(i) = self.sys.focused.and_then(|id| self.sys.nodes.get_by_id(id)) {
                    // Don't activate a focused text edit: Space/Enter are text
                    // input there (handled by keru_text), not activation. Also
                    // skip non-interactable nodes, which can only be focused as a
                    // navigation anchor.
                    let is_text_edit = matches!(self.sys.nodes[i].text_i, Some(TextI::TextEdit(_)));
                    if self.is_interactable_for_focus(i) && !is_text_edit {
                        self.activate_focused_node(i);
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Activate a node via the keyboard, as if it had been clicked.
    fn activate_focused_node(&mut self, i: NodeI) {
        self.sys.push_synthetic_click(i);

        // Mirror the click animation that resolve_click_press triggers.
        if self.sys.nodes[i].params.interact.click_animation {
            self.sys.nodes[i].last_click = T0.elapsed().as_secs_f32();
            self.sys.changes.rebuild_render_data = true;
            self.sys.anim_render_timer.push_new(Duration::from_secs_f32(ANIMATION_RERENDER_TIME));
        }

        self.set_new_ui_input();
    }

    /// Set the keyboard focus to the node corresponding to `key`.
    pub fn focus(&mut self, key: NodeKey) {
        let id = key.id_with_key_scope();
        if let Some(i) = self.sys.nodes.get_by_id(id) {
            self.set_focus_node(i, true);
        };
    }

    /// Move the keyboard focus to the next interactable node. If nothing is focused yet, focuses the first interactable node.
    pub fn focus_next(&mut self) {
        self.move_keyboard_focus(true);
    }

    /// Move the keyboard focus to the previous interactable node. If nothing is focused yet, focuses the last interactable node.
    pub fn focus_previous(&mut self) {
        self.move_keyboard_focus(false);
    }

    /// Clear the keyboard focus.
    pub fn unfocus(&mut self) {
        self.sys.focused = None;
        self.sys.show_focus_indicator = false;
        self.sys.renderer.text.clear_focus();
        self.sys.changes.should_rebuild_render_data = true;
    }

    /// Move the keyboard focus to the next (or previous) interactable node in
    /// depth-first order, wrapping around at the ends.
    ///
    /// If nothing is focused yet, focuses the first interactable node.
    pub(crate) fn move_keyboard_focus(&mut self, forward: bool) {
        // Using the keyboard always reveals the focus indicator, even if the
        // focus doesn't end up moving (e.g. a single interactable node).
        self.sys.show_focus_indicator = true;
        self.sys.changes.should_rebuild_render_data = true;

        let Some(first) = self.first_node() else { return; };
        let Some(last) = self.last_node() else { return; };

        // The node we start scanning from. When nothing is focused (or the
        // focused node no longer exists), the first step should land on the
        // very first (or last) node of the tree.
        let start = match self.sys.focused.and_then(|id| self.sys.nodes.get_by_id(id)) {
            Some(i) => i,
            None => {
                let candidate = if forward { first } else { last };
                if self.is_interactable_for_focus(candidate) {
                    self.set_focus_node(candidate, true);
                    return;
                }
                candidate
            }
        };

        let mut cursor = start;
        loop {
            cursor = if forward {
                self.next_node(cursor).unwrap_or(first)
            } else {
                self.prev_node(cursor).unwrap_or(last)
            };

            if self.is_interactable_for_focus(cursor) {
                self.set_focus_node(cursor, true);
                return;
            }

            // Walked the whole tree without finding anything interactable.
            if cursor == start {
                return;
            }
        }
    }

    pub(crate) fn set_focus_node(&mut self, i: NodeI, show_indicator: bool) {
        self.sys.focused = Some(self.sys.nodes[i].id);
        self.sys.show_focus_indicator = show_indicator;
        self.sys.changes.should_rebuild_render_data = true;

        match &self.sys.nodes[i].text_i {
            Some(TextI::TextEdit(handle)) => {
                self.sys.renderer.text.get_text_edit_mut(handle).set_focus();
            }
            Some(TextI::TextBox(handle)) => {
                self.sys.renderer.text.get_text_box_mut(handle).set_focus();
            }
            None => {
                // self.sys.renderer.text.clear_focus();
            }
        }
    }

    fn is_interactable_for_focus(&self, i: NodeI) -> bool {
        return self.sys.nodes[i].params.interact.focusable;
    }

    fn first_node(&self) -> Option<NodeI> {
        self.sys.nodes[ROOT_I].first_child
    }

    /// The last node of the tree in depth-first order (deepest last descendant).
    fn last_node(&self) -> Option<NodeI> {
        let mut cursor = self.sys.nodes[ROOT_I].last_child?;
        while let Some(child) = self.sys.nodes[cursor].last_child {
            cursor = child;
        }
        Some(cursor)
    }

    fn next_node(&self, i: NodeI) -> Option<NodeI> {
        if let Some(child) = self.sys.nodes[i].first_child {
            return Some(child);
        }
        let mut cursor = i;
        loop {
            if let Some(sibling) = self.sys.nodes[cursor].next_sibling {
                return Some(sibling);
            }
            let parent = self.sys.nodes[cursor].parent;
            if parent == ROOT_I {
                return None;
            }
            cursor = parent;
        }
    }

    fn prev_node(&self, i: NodeI) -> Option<NodeI> {
        if let Some(sibling) = self.sys.nodes[i].prev_sibling {
            // Deepest last descendant of the previous sibling.
            let mut cursor = sibling;
            while let Some(child) = self.sys.nodes[cursor].last_child {
                cursor = child;
            }
            return Some(cursor);
        }
        let parent = self.sys.nodes[i].parent;
        if parent == ROOT_I {
            return None;
        }
        Some(parent)
    }

    pub(crate) fn handle_scroll_event(&mut self, delta: &MouseScrollDelta) {
        // Find the topmost hit node, then walk up to find scroll target
        let hovered_ids = self.scan_opaque_hits();
        let Some(&first_id) = hovered_ids.first() else {
            return;
        };
        let Some(hover_i) = self.sys.nodes.get_by_id(first_id) else {
            return;
        };

        let scale = self.sys.scale_factor;
        let (dx, dy) = match delta {
            MouseScrollDelta::LineDelta(x, y) => (x * 0.1, y * 0.1),
            MouseScrollDelta::PixelDelta(PhysicalPosition { x, y }) => (*x as f32 / scale, *y as f32 / scale),
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
                if self.sys.nodes[current_i].params.interact.senses.contains(Sense::SCROLL) {
                    scroll_target = Some((current_i, true));
                    break;
                }

                // Then check for scrollable container
                if self.sys.nodes[current_i].params.layout.scrollable[axis] {
                    scroll_target = Some((current_i, false));
                    break;
                }

                let parent_i = self.sys.nodes[current_i].parent;
                if parent_i == ROOT_I {
                    break;
                }
                current_i = parent_i;
            }
        }

        if let Some((target_i, is_sense)) = scroll_target {
            if is_sense {
                // if the node has the scroll sense, we have to do set_new_ui_input and do a full rebuild, and everything will sort itself out automatically.
                let id = self.sys.nodes[target_i].id;
                let scroll_delta = match delta {
                    MouseScrollDelta::LineDelta(x, y) => Vec2::new(*x * 0.1, *y * 0.1),
                    MouseScrollDelta::PixelDelta(p) => Vec2::new(p.x as f32 / scale, p.y as f32 / scale),
                };
                self.sys.mouse_input.push_scroll(scroll_delta, id);
                self.set_new_ui_input();
            } else {
                // otherwise, do atomic updates on the scroll value and the scrollbar state, and schedule just a rerender.
                self.update_container_scroll(target_i, fdelta[X], X);
                self.update_container_scroll(target_i, fdelta[Y], Y);

                self.update_scrollbar_handle_params(target_i);
                self.partial_relayout_for_scrollbar(target_i);
                // scrolling can cause the cursor to end up on top of a new node.
                self.resolve_hover();

                self.sys.changes.should_rebuild_render_data = true;
                self.sys.changes.need_rerender = true;
            }
        }
    }
}

// Methods that need to be reachable by the UiNode wrapper need to implemented for the inner System and not the Ui, because of the wrapper struct arena trick.
// It doesn't make much difference. Maybe we should implement all private functions as methods of System for consistency.
impl System {
    /// Emit a synthetic click (and matching click-release) on a node, as if the
    /// mouse had pressed and released on it. Used for keyboard activation. The
    /// click position is placed at the middle of the node's rect.
    pub(crate) fn push_synthetic_click(&mut self, i: NodeI) {
        let id = self.nodes[i].id;

        let logical_size = self.logical_size();
        let rect = self.nodes[i].get_animated_rect();
        let position = Vec2::new(
            (rect[X][0] + rect[X][1]) / 2.0 * logical_size[X],
            (rect[Y][0] + rect[Y][1]) / 2.0 * logical_size[Y],
        );

        let now = std::time::Instant::now();
        let mut targets = SmallVec::new();
        targets.push(id);
        self.mouse_input.events.push(crate::mouse_events::InputEvent::Click(crate::mouse_events::ClickEvent {
            targets: targets.clone(),
            position,
            button: MouseButton::Left,
            timestamp: now,
        }));
        self.mouse_input.events.push(crate::mouse_events::InputEvent::ClickRelease(crate::mouse_events::ClickReleaseEvent {
            targets,
            position,
            button: MouseButton::Left,
            press_time: now,
        }));
    }

    pub(crate) fn click_rect(&self, i: NodeI) -> ClickRect {
        let real_rect = self.nodes[i].real_rect;
        let transform = self.nodes[i].accumulated_transform;
        let size = self.size;

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

    /// Hit test with the current stored cursor position and a click rect
    pub(crate) fn hit_click_rect(&self, rect: &ClickRect) -> bool {
        let logical_size = self.logical_size();
        let size = self.size;

        // Get cursor position and convert to normalized coordinates.
        // cursor_position is in logical pixels; divide by logical screen size.
        let cursor_pos = (
            self.mouse_input.cursor_position.x / logical_size[X],
            self.mouse_input.cursor_position.y / logical_size[Y],
        );

        let node_i = rect.i;

        let aabb_hit = rect.rect[X][0] < cursor_pos.0
            && cursor_pos.0 < rect.rect[X][1]
            && rect.rect[Y][0] < cursor_pos.1
            && cursor_pos.1 < rect.rect[Y][1];

        if aabb_hit == false {
            return false;
        }

        // todo more accurate clicks
        match self.nodes[node_i].params.shape {
            Shape::NoShape => {
                return false; // weird...
            }
            Shape::Rectangle { .. } => {
                return true;
            }
            Shape::Circle => {
                // Calculate the circle center and radius
                let center_x = (rect.rect[X][0] + rect.rect[X][1]) / 2.0;
                let center_y = (rect.rect[Y][0] + rect.rect[Y][1]) / 2.0;
                let radius = (rect.rect[X][1] - rect.rect[X][0]) / 2.0;

                // Check if the mouse is within the circle
                let dx = cursor_pos.0 - center_x;
                let dy = cursor_pos.1 - center_y;
                return dx * dx + dy * dy <= radius * radius;
            }
            Shape::Ring { width } => {
                // scale to correct coordinates
                // width should have been a Len anyway so this will have to change
                let width = width / size[X];

                let aspect = size[X] / size[Y];
                // Calculate the ring's center and radii
                let center_x = (rect.rect[X][0] + rect.rect[X][1]) / 2.0;
                let center_y = (rect.rect[Y][0] + rect.rect[Y][1]) / 2.0;
                let outer_radius = (rect.rect[X][1] - rect.rect[X][0]) / 2.0;
                let inner_radius = outer_radius - width;

                // Check if the mouse is within the ring
                let dx = cursor_pos.0 - center_x;
                let dy = (cursor_pos.1 - center_y) / aspect;
                let distance_squared = dx * dx + dy * dy;
                return distance_squared <= outer_radius * outer_radius
                    && distance_squared >= inner_radius * inner_radius;

            }
            Shape::Arc { .. } => {
                let center_x = (rect.rect[X][0] + rect.rect[X][1]) / 2.0;
                let center_y = (rect.rect[Y][0] + rect.rect[Y][1]) / 2.0;
                let radius = (rect.rect[X][1] - rect.rect[X][0]) / 2.0;

                let dx = cursor_pos.0 - center_x;
                let dy = cursor_pos.1 - center_y;
                return dx * dx + dy * dy <= radius * radius;
            }
            Shape::Pie { .. } => {
                let center_x = (rect.rect[X][0] + rect.rect[X][1]) / 2.0;
                let center_y = (rect.rect[Y][0] + rect.rect[Y][1]) / 2.0;
                let radius = (rect.rect[X][1] - rect.rect[X][0]) / 2.0;

                let dx = cursor_pos.0 - center_x;
                let dy = cursor_pos.1 - center_y;
                return dx * dx + dy * dy <= radius * radius;
            }
            Shape::Hexagon { size: size_param, rotation } => {
                let screen_width = size[X];
                let screen_height = size[Y];

                // Convert rect to pixels
                let x0 = rect.rect[X][0] * screen_width;
                let x1 = rect.rect[X][1] * screen_width;
                let y0 = rect.rect[Y][0] * screen_height;
                let y1 = rect.rect[Y][1] * screen_height;

                // Cursor in pixels
                let cursor_px = cursor_pos.0 * screen_width;
                let cursor_py = cursor_pos.1 * screen_height;

                // Calculate hexagon parameters (matching render.rs)
                let cx = (x0 + x1) / 2.0;
                let cy = (y0 + y1) / 2.0;
                let max_radius = ((x1 - x0) / 2.0).min((y1 - y0) / 2.0);
                let hex_radius = max_radius * size_param;

                // Transform cursor to hexagon-local coordinates
                let dx = cursor_px - cx;
                let dy = cursor_py - cy;

                // Apply inverse rotation (rotate by -rotation)
                let cos_r = rotation.cos();
                let sin_r = rotation.sin();
                let local_x = dx * cos_r + dy * sin_r;
                let local_y = -dx * sin_r + dy * cos_r;

                // Point-in-hexagon test using 3-band method for flat-top hexagon
                // A regular hexagon can be described as the intersection of 3 pairs of parallel lines
                let sqrt3 = 3.0_f32.sqrt();
                let sqrt3_r = sqrt3 * hex_radius;
                let inradius = sqrt3_r / 2.0; // distance from center to edge midpoint

                // Check 3 constraints:
                // 1. Top/bottom edges: |y| <= inradius
                // 2. Upper-right/lower-left edges: |√3*x + y| <= √3*R
                // 3. Lower-right/upper-left edges: |√3*x - y| <= √3*R
                return local_y.abs() <= inradius
                    && (sqrt3 * local_x + local_y).abs() <= sqrt3_r
                    && (sqrt3 * local_x - local_y).abs() <= sqrt3_r;
            }
            Shape::Segment { .. } | Shape::HorizontalLine | Shape::VerticalLine | Shape::Triangle { .. } | Shape::SquareGrid { .. } | Shape::HexGrid { .. } => {
                // For segments, triangles, and grids, use simple rectangle hit test
                return true;
            }
        }

    }

    /// Scan once for every node under the cursor, recording enough info that
    /// callers can later filter by whichever sense they care about (see
    /// [`HitNode::filter_by_sense`]).
    ///
    /// The result is topmost-first. The scan visits all hit nodes in z-order
    /// until it reaches an absorbing node; from there it only follows that
    /// node's ancestor chain (the nodes an event could still pass through),
    /// stopping at the first absorbing ancestor. This mirrors the visiting set
    /// of [`scan_hits_with_sense`](Self::scan_hits_with_sense), so filtering the
    /// result by a sense reproduces that function's output.
    pub(crate) fn scan_all_hits(&self) -> SmallVec<HitNode> {
        let mut result = SmallVec::new();

        for clk_i in (0..self.click_rects.len()).rev() {
            let rect = &self.click_rects[clk_i];

            if ! self.hit_click_rect(rect) {
                continue;
            }

            result.push(HitNode {
                id: self.nodes[rect.i].id,
                senses: rect.senses,
                absorbs_mouse_events: rect.absorbs_mouse_events,
            });

            if rect.absorbs_mouse_events {
                // Once we hit an absorbing node, events can only keep passing
                // through to its ancestors. Walk up until the next absorbing one.
                let mut current_i = self.nodes[rect.i].parent;
                while current_i != ROOT_I {
                    let parent_rect = self.click_rect(current_i);
                    if self.hit_click_rect(&parent_rect) {
                        result.push(HitNode {
                            id: self.nodes[current_i].id,
                            senses: parent_rect.senses,
                            absorbs_mouse_events: parent_rect.absorbs_mouse_events,
                        });
                        if parent_rect.absorbs_mouse_events {
                            break;
                        }
                    }
                    current_i = self.nodes[current_i].parent;
                }
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

        for clk_i in (0..self.click_rects.len()).rev() {
            let rect = &self.click_rects[clk_i];

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

    pub(crate) fn check_clicked(&self, id: Id, button: MouseButton) -> bool {
        #[cfg(debug_assertions)] {
            if let Some(i) = self.nodes.get_by_id(id) {
                if !self.check_node_sense(i, Sense::CLICK, "is_clicked()", "Node::sense_click()") {
                    return false;
                }
            }
        }
        
        self.mouse_input.clicks()
            .any(|e| e.button == button && e.targets.contains(&id))
    }

    pub(crate) fn check_dragged(&self, id: Id, button: MouseButton) -> Option<&mouse_events::DragEvent> {
        #[cfg(debug_assertions)] {
            if let Some(i) = self.nodes.get_by_id(id) {
                if !self.check_node_sense(i, Sense::DRAG, "is_dragged()", "Node::sense_drag()") {
                    return None;
                }
            }
        }
        self.mouse_input.drags()
            .find(|e| e.button == button && e.targets.contains(&id))
    }

    pub(crate) fn check_hovered(&self, id: Id) -> bool {
        #[cfg(debug_assertions)] {
            if let Some(i) = self.nodes.get_by_id(id) {
                let senses = self.nodes[i].params.interact.senses;
                if !senses.intersects(Sense::HOVER | Sense::HOVER_ENTER_OR_EXIT) {
                    eprintln!(
                        "Keru: Debug mode check: \"is_hovered()\" was called for node {}, but the node doesn't have the HOVER or HOVER_ENTER_OR_EXIT sense. In release mode, this event will be silently ignored! You can add the sense with \"Node::sense_hover()\" or \"Node::sense_hover_enter_or_exit()\".",
                        self.nodes[i].debug_name(),
                    );
                    return false;
                }
            }
        }
        self.hovered.contains(&id)
    }

    pub(crate) fn check_clicked_at(&self, id: Id, button: MouseButton) -> Option<&mouse_events::ClickEvent> {
        #[cfg(debug_assertions)] {
            if let Some(i) = self.nodes.get_by_id(id) {
                if !self.check_node_sense(i, Sense::CLICK, "clicked_at()", "Node::sense_click()") {
                    return None;
                }
            }
        }
        self.mouse_input.clicks()
            .filter(|e| e.button == button && e.targets.contains(&id))
            .last()
    }

    pub(crate) fn check_click_released(&self, id: Id, button: MouseButton) -> bool {
        #[cfg(debug_assertions)] {
            if let Some(i) = self.nodes.get_by_id(id) {
                if !self.check_node_sense(i, Sense::CLICK_RELEASE, "is_click_released()", "Node::sense_click()") {
                    return false;
                }
            }
        }
        self.mouse_input.click_releases()
            .any(|e| e.button == button && e.targets.contains(&id))
    }

    pub(crate) fn check_drag_released(&self, id: Id, button: MouseButton) -> bool {
        #[cfg(debug_assertions)] {
            if let Some(i) = self.nodes.get_by_id(id) {
                if !self.check_node_sense(i, Sense::DRAG, "is_drag_released()", "Node::sense_drag()") {
                    return false;
                }
            }
        }
        self.mouse_input.drag_releases()
            .any(|e| e.button == button && e.targets.contains(&id))
    }

    pub(crate) fn check_drag_released_onto(&self, src_id: Id, dest_id: Id, button: MouseButton) -> Option<&mouse_events::DragReleaseEvent> {
        // Check if dest is reachable as a drop target
        let drop_targets = self.scan_hits_with_sense(Sense::DRAG_DROP_TARGET);
        if !drop_targets.contains(&dest_id) {
            return None;
        }

        self.mouse_input.drag_releases()
            .find(|e| e.button == button && e.targets.contains(&src_id))
    }

    pub(crate) fn check_drag_hovered_onto(&self, src_id: Id, dest_id: Id, button: MouseButton) -> Option<&mouse_events::DragEvent> {
        // Check if dest is reachable as a drop target
        let drop_targets = self.scan_hits_with_sense(Sense::DRAG_DROP_TARGET);
        if !drop_targets.contains(&dest_id) {
            return None;
        }

        self.mouse_input.drags()
            .find(|e| e.button == button && e.targets.contains(&src_id))
    }

    pub(crate) fn check_held_duration(&self, id: Id, button: MouseButton) -> Option<Duration> {
        #[cfg(debug_assertions)] {
            if let Some(i) = self.nodes.get_by_id(id) {
                if !self.check_node_sense(i, Sense::HOLD, "is_held()", "Node::sense_hold()") {
                    return None;
                }
            }
        }
        // Hold is tracked via drag events - duration since start
        self.mouse_input.drags()
            .find(|e| e.button == button && e.targets.contains(&id))
            .map(|e| e.start_time.elapsed())
    }

    pub(crate) fn check_scrolled(&self, id: Id) -> Option<Vec2> {
        #[cfg(debug_assertions)] {
            if let Some(i) = self.nodes.get_by_id(id) {
                if !self.check_node_sense(i, Sense::SCROLL, "is_scrolled()", "Node::sense_scroll()") {
                    return None;
                }
            }
        }
        let mut total = Vec2::ZERO;
        let mut found = false;
        for e in self.mouse_input.scrolls() {
            if e.target == id {
                total += e.delta;
                found = true;
            }
        }
        if found { Some(total) } else { None }
    }

    pub(crate) fn check_last_scroll_event(&self, id: Id) -> Option<&mouse_events::ScrollEvent> {
        #[cfg(debug_assertions)] {
            if let Some(i) = self.nodes.get_by_id(id) {
                if !self.check_node_sense(i, Sense::SCROLL, "scrolled_at()", "Node::sense_scroll()") {
                    return None;
                }
            }
        }
        self.mouse_input.scrolls()
            .filter(|e| e.target == id)
            .last()
    }

    pub(crate) fn global_scroll_delta(&self) -> Option<Vec2> {
        let mut total = Vec2::ZERO;
        let mut found = false;
        for e in self.mouse_input.scrolls() {
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

        self.mouse_input.drags()
            .find(|e| e.button == button)
    }

    /// Find any drag released onto dest (from any source)
    pub(crate) fn check_any_drag_released_onto(&self, dest_id: Id, button: MouseButton) -> Option<&mouse_events::DragReleaseEvent> {
        let drop_targets = self.scan_hits_with_sense(Sense::DRAG_DROP_TARGET);
        if !drop_targets.contains(&dest_id) {
            return None;
        }

        self.mouse_input.drag_releases()
            .find(|e| e.button == button)
    }
}

