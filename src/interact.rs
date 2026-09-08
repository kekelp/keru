use std::time::Duration;

use glam::Vec2;
use winit::{dpi::PhysicalPosition, event::{KeyEvent, MouseButton, MouseScrollDelta}, keyboard::{Key, NamedKey}, window::Window};

use crate::*;
use crate::Axis::{X, Y};
use crate::mouse_events::SmallVec;

pub(crate) const ANIMATION_RERENDER_TIME: f32 = 0.5;
pub(crate) const SCROLL_INTO_VIEW_PADDING_PIXELS: f32 = 10.0;

pub(crate) const LINE_HIT_TOLERANCE: f32 = 5.0;

const TAU: f32 = std::f32::consts::TAU;

fn normalize_angle(angle: f32) -> f32 {
    let r = angle % TAU;
    if r < 0.0 { r + TAU } else { r }
}

fn sd_rounded_box(p: Vec2, half: Vec2, radii: [f32; 4]) -> f32 {
    let [tl, tr, bl, br] = radii;
    let r = match (p.x > 0.0, p.y > 0.0) {
        (true, true) => br,
        (true, false) => tr,
        (false, true) => bl,
        (false, false) => tl,
    };
    let q = p.abs() - half + Vec2::splat(r);
    q.max(Vec2::ZERO).length() + q.x.max(q.y).min(0.0) - r
}

fn sd_arc(p: Vec2, inner: f32, outer: f32, start_angle: f32, end_angle: f32) -> f32 {
    let angle = p.y.atan2(p.x);
    let angle_rel = normalize_angle(angle - start_angle);
    let span = normalize_angle(end_angle - start_angle);
    let mid_radius = (inner + outer) * 0.5;
    let half_thickness = (outer - inner) * 0.5;
    let nearest_rel = if angle_rel <= span {
        angle_rel
    } else {
        let dist_to_start = TAU - angle_rel;
        let dist_to_end = angle_rel - span;
        if dist_to_start < dist_to_end { 0.0 } else { span }
    };
    let nearest = mid_radius * Vec2::new((start_angle + nearest_rel).cos(), (start_angle + nearest_rel).sin());
    (p - nearest).length() - half_thickness
}

fn sd_pie(p: Vec2, radius: f32, start_angle: f32, end_angle: f32) -> f32 {
    let mut span = normalize_angle(end_angle - start_angle);
    if span < 0.001 { span = TAU; }
    if (span - TAU).abs() < 0.01 {
        return p.length() - radius;
    }
    let mid_angle = start_angle + span * 0.5;
    let rot = std::f32::consts::FRAC_PI_2 - mid_angle;
    let (sr, cr) = rot.sin_cos();
    let mut rp = Vec2::new(p.x * cr - p.y * sr, p.x * sr + p.y * cr);
    let half_span = span * 0.5;
    let c = Vec2::new(half_span.sin(), half_span.cos());
    rp.x = rp.x.abs();
    let l = rp.length() - radius;
    let m = (rp - c * rp.dot(c).clamp(0.0, radius)).length();
    l.max(m * (c.y * rp.x - c.x * rp.y).signum())
}

fn sd_hexagon(mut p: Vec2, r: f32) -> f32 {
    let k = Vec2::new(-0.866025404, 0.5);
    let kz = 0.577350269;
    p = p.abs();
    p -= 2.0 * k.dot(p).min(0.0) * k;
    p -= Vec2::new(p.x.clamp(-kz * r, kz * r), r);
    p.length() * p.y.signum()
}

fn sd_segment(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = (pa.dot(ba) / ba.dot(ba)).clamp(0.0, 1.0);
    (pa - ba * h).length()
}

fn sd_triangle(p: Vec2, p0: Vec2, p1: Vec2, p2: Vec2) -> f32 {
    let e0 = p1 - p0;
    let e1 = p2 - p1;
    let e2 = p0 - p2;
    let v0 = p - p0;
    let v1 = p - p1;
    let v2 = p - p2;
    let pq0 = v0 - e0 * (v0.dot(e0) / e0.dot(e0)).clamp(0.0, 1.0);
    let pq1 = v1 - e1 * (v1.dot(e1) / e1.dot(e1)).clamp(0.0, 1.0);
    let pq2 = v2 - e2 * (v2.dot(e2) / e2.dot(e2)).clamp(0.0, 1.0);
    let s = (e0.x * e2.y - e0.y * e2.x).signum();
    let d = Vec2::new(pq0.dot(pq0), s * (v0.x * e0.y - v0.y * e0.x))
        .min(Vec2::new(pq1.dot(pq1), s * (v1.x * e1.y - v1.y * e1.x)))
        .min(Vec2::new(pq2.dot(pq2), s * (v2.x * e2.y - v2.y * e2.x)));
    -d.x.sqrt() * d.y.signum()
}

/// A struct describing a click event on a GUI node.
#[derive(Clone, Copy, Debug)]
pub struct Click {
    /// Absolute screen position in pixels
    pub absolute_position: glam::Vec2,
    /// Position relative to the node (0.0 to 1.0 in each dimension)
    pub relative_position: glam::Vec2,
    /// Timestamp of the click
    pub timestamp: std::time::Instant,
    /// Number of clicks in the current chain: 1 for a single click, 2 for a double click, and so on.
    pub count: u32,
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

/// Result of a single mouse-press hit-test pass. See `scan_press_hits`.
pub(crate) struct PressHits {
    pub click_ids: SmallVec<Id>,
    pub drag_ids: SmallVec<Id>,
    pub topmost: Option<Id>,
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
    pub(crate) fn scan_opaque_hits(&self) -> SmallVec<Id> {
        self.sys.scan_hits(|rect| {
            rect.senses != Sense::NONE
                || rect.scrollable[X] || rect.scrollable[Y]
                || rect.absorbs_mouse_events
        }, false)
    }

    #[cfg(debug_assertions)]
    pub(crate) fn scan_any_node_hits(&self) -> SmallVec<Id> {
        self.sys.scan_hits(|_| true, false)
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

        // todo: this is not the best because hovered_ids is filtered on opaque nodes. In the future we should rewrite all this scanning code so it's easy to do a single scan that can check many different conditions without return-hell. 
        self.update_cursor_icon(hovered_ids.first().copied());

        // Debug mode: track all hits for inspection
        #[cfg(debug_assertions)]
        if self.inspect_mode() {
            let all_hits = self.scan_any_node_hits();
            if let Some(&new_id) = all_hits.first() {
                if self.sys.inspect_hovered.first() != Some(&new_id) {
                    if let Some(i) = self.sys.nodes.get_by_id(new_id) {
                        log::info!("Inspect mode: hovering {}", self.sys.nodes[i].debug_name());
                    }
                }
            }
            self.sys.inspect_hovered = all_hits;
        }
    }

    /// Set the window cursor from the topmost hovered node that specifies one, only when it changes.
    fn update_cursor_icon(&mut self, topmost: Option<Id>) {
        let icon = topmost
            .and_then(|id| self.sys.nodes.get_by_id(id))
            .and_then(|i| self.sys.nodes[i].params.interact.hover_cursor_icon)
            .unwrap_or(CursorIcon::Default);

        if icon != self.sys.last_cursor_icon {
            self.sys.last_cursor_icon = icon;
            if let Some(window) = self.sys.window_ref.as_ref().and_then(|w| w.upgrade()) {
                window.set_cursor(icon);
            }
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
            self.sys.changes.should_rebuild_render_data = true;
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
                self.sys.changes.should_rebuild_render_data = true;
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

        let anim_speed = self.sys.anim_exp_speed(1.0);
        self.sys.mouse_input.update_animated_scrolls(anim_speed);
    }

    pub(crate) fn handle_mouse_press(&mut self, button: MouseButton, window: &Window) -> bool {
        let PressHits { click_ids, drag_ids, topmost } = self.sys.scan_press_hits();

        self.sys.mouse_input.push_press(button, click_ids.clone(), drag_ids);

        self.resolve_focus_on_press(topmost);

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
        let click_ids = self.sys.scan_hits(|r| r.senses.contains(Sense::CLICK), true);
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
    fn resolve_focus_on_press(&mut self, hit: Option<Id>) {
        let Some(hit_id) = hit else {
            self.sys.focused = None;
            self.sys.renderer.text.clear_focus();
            self.sys.changes.focus_changed = true;
            return;
        };
        let Some(i) = self.sys.nodes.get_by_id(hit_id) else {
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
                self.sys.changes.should_rebuild_render_data = true;
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
                let mods = self.sys.key_input.key_mods();
                use winit::keyboard::ModifiersState;
                let no_other_mods = *mods & ! ModifiersState::SHIFT == ModifiersState::empty();
                if no_other_mods {
                    let forward = ! mods.shift_key();
                    self.move_keyboard_focus(forward);
                    return true;
                }
            }
        }

        if let Key::Named(NamedKey::Escape) = &event.logical_key {
            if event.state.is_pressed() && self.sys.show_focus_indicator {
                // Hide the focus indicator without losing the focus itself, so a subsequent Tab resumes navigation from the same node.
                self.sys.show_focus_indicator = false;
                self.sys.changes.should_rebuild_render_data = true;
                self.set_new_ui_input();
                return true;
            }
        }

        if let Key::Named(NamedKey::Space | NamedKey::Enter) = &event.logical_key {
            if event.state.is_pressed() {
                if let Some(i) = self.sys.focused.and_then(|id| self.sys.nodes.get_by_id(id)) {
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
            self.sys.changes.should_rebuild_render_data = true;
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
        self.sys.changes.focus_changed = true;
    }

    /// Move the keyboard focus to the next (or previous) interactable node in
    /// depth-first order, wrapping around at the ends.
    ///
    /// If nothing is focused yet, focuses the first interactable node.
    pub(crate) fn move_keyboard_focus(&mut self, forward: bool) {
        // Using the keyboard navigation always reveals the focus indicator even if the focus doesn't end up moving.
        self.sys.show_focus_indicator = true;
        self.sys.changes.should_rebuild_render_data = true;

        let Some(first) = self.first_node() else { return; };
        let Some(last) = self.last_node() else { return; };

        let focus_anchor = self.sys.focused
            .and_then(|id| self.sys.nodes.get_by_id(id))
            .filter(|&i| !self.sys.nodes[i].currently_hidden);
        let start = match focus_anchor {
            Some(i) => i,
            None => {
                let candidate = if forward { first } else { last };
                if self.is_interactable_for_focus(candidate) {
                    self.set_focus_node(candidate, true);
                    self.sys.scroll_node_into_view(candidate, SCROLL_INTO_VIEW_PADDING_PIXELS, true);
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
                self.sys.scroll_node_into_view(cursor, SCROLL_INTO_VIEW_PADDING_PIXELS, true);
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
        self.sys.changes.focus_changed = true;

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
        return self.sys.nodes[i].params.interact.focusable && !self.sys.nodes[i].currently_hidden;
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
        let (dx, dy, likely_scrollwheel) = match delta {
            MouseScrollDelta::LineDelta(x, y) => {
                let likely_scrollwheel = y.fract() == 0.0 && y.abs() > 0.2;
                let line_to_pixel_ratio = if likely_scrollwheel { 0.4 } else { 0.1 };
                (x * line_to_pixel_ratio, y * line_to_pixel_ratio, likely_scrollwheel)
            },
            MouseScrollDelta::PixelDelta(PhysicalPosition { x, y }) => (*x as f32 / scale, *y as f32 / scale, false),
        };
        let fdelta = Xy::new(dx, dy);

        // Each axis resolves its own scroll target; the two axes may land on the
        // same node or on different ones. Accumulate the per-axis deltas per target.
        let mut targets: SmallVec<(NodeI, bool, Xy<f32>)> = SmallVec::new();
        for axis in [X, Y] {
            if fdelta[axis] == 0.0 {
                continue;
            }
            let Some((target_i, is_sense)) = self.sys.find_scroll_target(hover_i, axis) else {
                continue;
            };
            if let Some(entry) = targets.iter_mut().find(|(i, _, _)| *i == target_i) {
                entry.2[axis] = fdelta[axis];
            } else {
                let mut delta = Xy::new(0.0, 0.0);
                delta[axis] = fdelta[axis];
                targets.push((target_i, is_sense, delta));
            }
        }

        let mut scrolled_any_container = false;
        for &(target_i, is_sense, delta) in &targets {
            if is_sense {
                let id = self.sys.nodes[target_i].id;
                self.sys.mouse_input.push_scroll(Vec2::new(delta.x, delta.y), id, likely_scrollwheel);
                self.set_new_ui_input();
            }
            let scrollable = self.sys.nodes[target_i].params.layout.scrollable;
            if scrollable[X] || scrollable[Y] {
                // Do atomic updates on the scroll value and the scrollbar state.
                let animate = likely_scrollwheel;
                self.sys.update_container_scroll(target_i, delta[X], X, animate);
                self.sys.update_container_scroll(target_i, delta[Y], Y, animate);
                self.sys.update_scrollbar_handle_params(target_i);
                if ! is_sense {
                    self.partial_relayout_for_scrollbar(target_i);
                    scrolled_any_container = true;
                }
            }
        }

        if scrolled_any_container {
            // scrolling can cause the cursor to end up on top of a new node.
            self.resolve_hover();
            self.sys.changes.should_rebuild_render_data = true;
            self.sys.changes.need_rerender = true;
        }
    }

    pub(crate) fn push_synthetic_scroll(&mut self, i: NodeI, fdelta: Xy<f32>) {
        self.sys.update_container_scroll(i, fdelta[X], X, false);
        self.sys.update_container_scroll(i, fdelta[Y], Y, false);

        self.sys.update_scrollbar_handle_params(i);
        self.partial_relayout_for_scrollbar(i);
        // Scrolling can move the cursor onto a different node.
        self.resolve_hover();

        self.sys.changes.should_rebuild_render_data = true;
        self.sys.changes.need_rerender = true;
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
        self.mouse_input.events.push(InputEvent::Click(ClickEvent {
            targets: targets.clone(),
            position,
            button: MouseButton::Left,
            timestamp: now,
            count: 1,
        }));
        self.mouse_input.events.push(InputEvent::ClickRelease(ClickReleaseEvent {
            targets,
            button: MouseButton::Left,
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

        let scale_factor = self.scale_factor;
        let px0 = rect.rect[X][0] * size[X];
        let px1 = rect.rect[X][1] * size[X];
        let py0 = rect.rect[Y][0] * size[Y];
        let py1 = rect.rect[Y][1] * size[Y];
        let center = Vec2::new((px0 + px1) / 2.0, (py0 + py1) / 2.0);
        let cursor = Vec2::new(cursor_pos.0 * size[X], cursor_pos.1 * size[Y]);
        let p = cursor - center;
        // Line-like shapes get a small extra hit tolerance so thin strokes stay clickable.
        let line_tolerance = LINE_HIT_TOLERANCE * scale_factor;
        let stroke_half_thickness = || {
            let w = self.nodes[node_i].params.stroke.map(|s| s.width).unwrap_or(1.0) * scale_factor;
            (w / 2.0).max(line_tolerance)
        };

        match self.nodes[node_i].params.shape {
            Shape::NoShape => {
                return false; // weird...
            }
            Shape::Rectangle { rounded_corners, corner_radius } => {
                let half = Vec2::new((px1 - px0) / 2.0, (py1 - py0) / 2.0);
                let r = corner_radius * scale_factor;
                let radii = [
                    if rounded_corners.contains(RoundedCorners::TOP_LEFT) { r } else { 0.0 },
                    if rounded_corners.contains(RoundedCorners::TOP_RIGHT) { r } else { 0.0 },
                    if rounded_corners.contains(RoundedCorners::BOTTOM_LEFT) { r } else { 0.0 },
                    if rounded_corners.contains(RoundedCorners::BOTTOM_RIGHT) { r } else { 0.0 },
                ];
                return sd_rounded_box(p, half, radii) <= 0.0;
            }
            Shape::Circle => {
                let radius = ((px1 - px0) / 2.0).min((py1 - py0) / 2.0);
                return p.length() - radius <= 0.0;
            }
            Shape::Ring { width } => {
                let outer = ((px1 - px0) / 2.0).min((py1 - py0) / 2.0);
                let inner = (outer - width * scale_factor).max(0.0);
                let d = p.length();
                return d >= inner && d <= outer;
            }
            Shape::Arc { start_angle, end_angle, width } => {
                let outer = ((px1 - px0) / 2.0).min((py1 - py0) / 2.0);
                let actual_width = width * scale_factor;
                let inner = (outer - actual_width).max(0.0);
                return sd_arc(p, inner, outer, start_angle, end_angle) <= 0.0;
            }
            Shape::Pie { start_angle, end_angle } => {
                let radius = ((px1 - px0) / 2.0).min((py1 - py0) / 2.0);
                return sd_pie(p, radius, start_angle, end_angle) <= 0.0;
            }
            Shape::Hexagon { size: size_param, rotation } => {
                let max_radius = ((px1 - px0) / 2.0).min((py1 - py0) / 2.0);
                let hex_size = max_radius * size_param;
                // Same rotation matrix as the shader.
                let (sin_r, cos_r) = rotation.sin_cos();
                let rp = Vec2::new(p.x * cos_r - p.y * sin_r, p.x * sin_r + p.y * cos_r);
                return sd_hexagon(rp, hex_size) <= 0.0;
            }
            Shape::Segment { start, end, .. } => {
                let a = Vec2::new(px0 + start.0 * (px1 - px0), py0 + start.1 * (py1 - py0));
                let b = Vec2::new(px0 + end.0 * (px1 - px0), py0 + end.1 * (py1 - py0));
                return sd_segment(cursor, a, b) - stroke_half_thickness() <= 0.0;
            }
            Shape::HorizontalLine => {
                let a = Vec2::new(px0, center.y);
                let b = Vec2::new(px1, center.y);
                return sd_segment(cursor, a, b) - stroke_half_thickness() <= 0.0;
            }
            Shape::VerticalLine => {
                let a = Vec2::new(center.x, py0);
                let b = Vec2::new(center.x, py1);
                return sd_segment(cursor, a, b) - stroke_half_thickness() <= 0.0;
            }
            Shape::Triangle { rotation, width } => {
                let radius = ((px1 - px0).min(py1 - py0)) / 2.0;
                let (sin_r, cos_r) = rotation.sin_cos();
                let tip_dist = radius;
                let base_back = radius * 0.5;
                let base_half_width = radius * 0.866 * width;
                let perp = Vec2::new(-sin_r, cos_r);
                let dir = Vec2::new(cos_r, sin_r);
                let p0 = center + dir * tip_dist;
                let p1 = center - dir * base_back + perp * base_half_width;
                let p2 = center - dir * base_back - perp * base_half_width;
                return sd_triangle(cursor, p0, p1, p2) <= 0.0;
            }
            Shape::SquareGrid { .. } | Shape::HexGrid { .. } => {
                return true;
            }
        }
    }


    fn find_scroll_target(&self, starting_i: NodeI, axis: Axis) -> Option<(NodeI, bool)> {
        let mut current_i = starting_i;
        loop {
            if self.nodes[current_i].params.interact.senses.contains(Sense::SCROLL) {
                return Some((current_i, true));
            }

            if self.nodes[current_i].params.layout.scrollable[axis] {
                return Some((current_i, false));
            }

            let parent_i = self.nodes[current_i].parent;
            if parent_i == ROOT_I {
                return None;
            }
            current_i = parent_i;
        }
    }

    pub(crate) fn scan_press_hits(&self) -> PressHits {
        let mut click_ids = SmallVec::new();
        let mut drag_ids: SmallVec<Id> = SmallVec::new();
        let mut topmost = None;

        let mut click_absorbed = false;
        let mut drag_absorbed = false;
        let mut hold_absorbed = false;

        for clk_i in (0..self.click_rects.len()).rev() {
            let rect = &self.click_rects[clk_i];

            if ! self.hit_click_rect(rect) {
                continue;
            }

            let id = self.nodes[rect.i].id;

            if topmost.is_none() {
                topmost = Some(id);
            }

            let click = rect.senses.contains(Sense::CLICK);
            let drag = rect.senses.contains(Sense::DRAG);
            let hold = rect.senses.contains(Sense::HOLD);

            if !click_absorbed && click {
                click_ids.push(id);
            }
            if ((!drag_absorbed && drag) || (!hold_absorbed && hold)) && !drag_ids.contains(&id) {
                drag_ids.push(id);
            }

            if rect.absorbs_mouse_events {
                if click { click_absorbed = true; }
                if drag { drag_absorbed = true; }
                if hold { hold_absorbed = true; }
            }
        }

        PressHits { click_ids, drag_ids, topmost }
    }

    pub(crate) fn scan_hits(&self, filter: impl Fn(&ClickRect) -> bool, _pass_to_parents: bool) -> SmallVec<Id> {
        let mut result = SmallVec::new();

        for clk_i in (0..self.click_rects.len()).rev() {
            let rect = &self.click_rects[clk_i];

            if ! self.hit_click_rect(rect) {
                continue;
            }

            let matched = filter(rect);
            if matched {
                result.push(self.nodes[rect.i].id);
            }

            if rect.absorbs_mouse_events {
                if matched {
                    break;
                }
                // For now we're not doing this, the users will have to understand it and use hitboxes or similar.
                // In the future we could bring this back and add a Node flag for the parent, something like "gets_events_through_opaque_children".
                // (for things like drag-drop areas or similar) 
                // if pass_to_parents {
                //     // Absorbing node doesn't pass the filter: the event keeps
                //     // passing up its ancestors until the next absorbing one.
                //     let mut current_i = self.nodes[rect.i].parent;
                //     while current_i != ROOT_I {
                //         // This is slow, btw.
                //         let parent_rect = self.click_rect(current_i);
                //         if self.hit_click_rect(&parent_rect) {
                //             if filter(&parent_rect) {
                //                 result.push(self.nodes[current_i].id);
                //             }
                //             if parent_rect.absorbs_mouse_events {
                //                 break;
                //             }
                //         }
                //         current_i = self.nodes[current_i].parent;
                //     }
                //     break;
                // }
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

    pub(crate) fn check_multi_clicked(&self, id: Id, button: MouseButton, count: u32) -> bool {
        #[cfg(debug_assertions)] {
            if let Some(i) = self.nodes.get_by_id(id) {
                if !self.check_node_sense(i, Sense::CLICK, "is_multi_clicked()", "Node::sense_click()") {
                    return false;
                }
            }
        }

        self.mouse_input.clicks()
            .any(|e| e.button == button && e.count == count && e.targets.contains(&id))
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
        let drop_targets = self.scan_hits(|r| r.senses.contains(Sense::DRAG_DROP_TARGET), true);
        if !drop_targets.contains(&dest_id) {
            return None;
        }

        self.mouse_input.drag_releases()
            .find(|e| e.button == button && e.targets.contains(&src_id))
    }

    pub(crate) fn check_drag_hovered_onto(&self, src_id: Id, dest_id: Id, button: MouseButton) -> Option<&mouse_events::DragEvent> {
        // Check if dest is reachable as a drop target
        let drop_targets = self.scan_hits(|r| r.senses.contains(Sense::DRAG_DROP_TARGET), true);
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

    pub(crate) fn check_scrolled_animated(&self, id: Id) -> Option<Vec2> {
        #[cfg(debug_assertions)] {
            if let Some(i) = self.nodes.get_by_id(id) {
                if !self.check_node_sense(i, Sense::SCROLL, "is_scrolled_animated()", "Node::sense_scroll()") {
                    return None;
                }
            }
        }
        let mut total = Vec2::ZERO;
        let mut found = false;
        for e in self.mouse_input.animated_scrolls() {
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

    pub(crate) fn check_last_animated_scroll_event(&self, id: Id) -> Option<&mouse_events::ScrollEvent> {
        #[cfg(debug_assertions)] {
            if let Some(i) = self.nodes.get_by_id(id) {
                if !self.check_node_sense(i, Sense::SCROLL, "scrolled_at_animated()", "Node::sense_scroll()") {
                    return None;
                }
            }
        }
        self.mouse_input.animated_scrolls()
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
        let drop_targets = self.scan_hits(|r| r.senses.contains(Sense::DRAG_DROP_TARGET), true);
        if !drop_targets.contains(&dest_id) {
            return None;
        }

        self.mouse_input.drags()
            .find(|e| e.button == button)
    }

    /// Find any drag released onto dest (from any source)
    pub(crate) fn check_any_drag_released_onto(&self, dest_id: Id, button: MouseButton) -> Option<&mouse_events::DragReleaseEvent> {
        let drop_targets = self.scan_hits(|r| r.senses.contains(Sense::DRAG_DROP_TARGET), true);
        if !drop_targets.contains(&dest_id) {
            return None;
        }

        self.mouse_input.drag_releases()
            .find(|e| e.button == button)
    }
}

