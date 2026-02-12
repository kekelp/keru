use std::time::{Duration, Instant};
use std::fmt::Debug;

use glam::{vec2, Vec2};
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};

// todo: rewrite all doc comments

pub trait Tag: Clone + Copy + PartialEq + Debug {}
impl<T: Clone + Copy + PartialEq + Debug> Tag for T {}

// Overflows if more than 6 elements should receive the event.
// Normally the elements that receive events also absorb them. They only stack if a node has absorb_click_events = false but also has active Senses.
// For example, invisible overlay panels.
// This is only used for Id which is an u64.
pub(crate) type SmallVec<T> = smallvec::SmallVec<[T; 8]>;

pub struct MouseInput<T: Tag> {
    unresolved_click_presses: Vec<PendingMousePress<T>>,
    pub last_frame_mouse_events: Vec<FullMouseEvent<T>>,
    pub current_frame_mouse_events: Vec<FullMouseEvent<T>>,
    pub last_frame_scroll_events: Vec<ScrollEvent<T>>,
    pub current_frame_scroll_events: Vec<ScrollEvent<T>>,
    pub currently_hovered_tags: SmallVec<T>,
    pub currently_hovered_tag_for_scroll: SmallVec<T>,
    pub cursor_position: Vec2,
    pub prev_cursor_position: Vec2,
}


impl<T: Tag> Default for MouseInput<T> {
    fn default() -> Self {
        return Self {
            unresolved_click_presses: Vec::with_capacity(20),
            last_frame_mouse_events: Vec::with_capacity(20),
            current_frame_mouse_events: Vec::with_capacity(20),
            last_frame_scroll_events: Vec::with_capacity(10),
            current_frame_scroll_events: Vec::with_capacity(10),
            currently_hovered_tags: SmallVec::with_capacity(5),
            currently_hovered_tag_for_scroll: SmallVec::with_capacity(5),
            cursor_position: Default::default(),
            prev_cursor_position: Default::default(),
        }
    }
}

impl<T: Tag> MouseInput<T> {
    pub fn begin_new_frame(&mut self) {
        let current_mouse_status = MouseRecord {
            position: self.cursor_position,
            timestamp: Instant::now(),
            tag: self.currently_hovered_tags.clone(),
        };

        // Swap events for double buffering
        std::mem::swap(&mut self.last_frame_mouse_events, &mut self.current_frame_mouse_events);
        self.current_frame_mouse_events.clear();

        std::mem::swap(&mut self.last_frame_scroll_events, &mut self.current_frame_scroll_events);
        self.current_frame_scroll_events.clear();

        self.unresolved_click_presses.retain(|click| click.already_released == false);

        for click_pressed in self.unresolved_click_presses.iter_mut().rev() {

            let mouse_happening = FullMouseEvent {
                button: click_pressed.button,
                originally_pressed: click_pressed.pressed_at.clone(),
                last_seen: click_pressed.last_seen.clone(),
                currently_at: current_mouse_status.clone(),
                kind: IsMouseReleased::StillDownButFrameEnded,
            };

            self.last_frame_mouse_events.push(mouse_happening);

            click_pressed.last_seen = current_mouse_status.clone();
        }
    }

    pub fn window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.prev_cursor_position = self.cursor_position;
                self.cursor_position = vec2(position.x as f32, position.y as f32);
            },
            WindowEvent::MouseInput { button, state, .. } => {
                let tags = self.currently_hovered_tags.clone();
                match state {
                    ElementState::Pressed => {
                        self.push_click_press(*button, tags)
                    },
                    ElementState::Released => {
                        self.push_click_release(*button);
                    },
                }
            },
            _ => {}
        }
    }

    pub fn update_current_tag(&mut self, new_tag: SmallVec<T>) {
        self.currently_hovered_tags = new_tag;
    }

    pub fn cursor_position(&self) -> Vec2 {
        return self.cursor_position;
    }

    pub fn push_click_press(&mut self, button: MouseButton, current_tag: SmallVec<T>) {
        let current_mouse_status = MouseRecord {
            position: self.cursor_position,
            timestamp: Instant::now(),
            tag: current_tag,
        };
        let pending_press = PendingMousePress::new(current_mouse_status, button);
        self.unresolved_click_presses.push(pending_press);
    }

    pub fn push_click_release(&mut self, button: MouseButton) {
        // Collect all pending presses for this button and mark them as released
        for click_pressed in self.unresolved_click_presses.iter_mut() {
            if click_pressed.button == button && !click_pressed.already_released {
                click_pressed.already_released = true;

                let current_mouse_status = MouseRecord {
                    position: self.cursor_position,
                    timestamp: Instant::now(),
                    tag: self.currently_hovered_tags.clone(),
                };

                let full_mouse_event = FullMouseEvent {
                    button,
                    originally_pressed: click_pressed.pressed_at.clone(),
                    last_seen: click_pressed.last_seen.clone(),
                    currently_at: current_mouse_status,
                    kind: IsMouseReleased::MouseReleased,
                };

                self.current_frame_mouse_events.push(full_mouse_event);
            }
        }
    }

    pub fn push_scroll_event(&mut self, delta: &MouseScrollDelta, tag: T) {
        let (x, y) = match delta {
            MouseScrollDelta::LineDelta(x, y) => (x * 0.1 , y * 0.1),
            MouseScrollDelta::PixelDelta(pos) => (pos.x as f32, pos.y as f32),
        };

        let mut tag2 = SmallVec::new();
        tag2.push(tag);
        let scroll_event = ScrollEvent {
            delta: Vec2::new(x, y),
            position: self.cursor_position,
            timestamp: Instant::now(),
            tag: tag2,
        };

        self.current_frame_scroll_events.push(scroll_event);
    }
    
    // querying

    pub fn all_mouse_events(&self) -> impl DoubleEndedIterator<Item = &FullMouseEvent<T>> {
        self.last_frame_mouse_events.iter()
    }  

    /// Returns all [`FullMouseEvent`]s for a specific button on the node corresponding to `tag`, or an empty iterator if the node is currently not part of the tree or if it doesn't exist.
    pub fn mouse_events(&self, mouse_button: Option<MouseButton>, tag: Option<T>) -> impl DoubleEndedIterator<Item = &FullMouseEvent<T>> {
        self.last_frame_mouse_events.iter().filter(move |c| {
            let button_matches = match mouse_button {
                Some(btn) => c.button == btn,
                None => true,
            };
            let tag_matches = match tag {
                Some(t) => c.originally_pressed.tag.contains(&t),
                None => true,
            };
            button_matches && tag_matches
        })
    }

    /// Returns `true` if the left mouse button was clicked on the node corresponding to `tag`, or `false` if the node is currently not part of the tree or if it doesn't exist.
    pub fn clicked(&self, mouse_button: Option<MouseButton>, tag: Option<T>) -> bool {
        let n_clicks = self.clicks(mouse_button, tag);
        return n_clicks > 0;
    }

    pub fn clicked_at(&self, mouse_button: Option<MouseButton>, tag: Option<T>) -> Option<MouseRecord<T>> {
        let last_click = self.mouse_events(mouse_button, tag).last()?;
        return Some(last_click.last_seen.clone());
    }

    pub fn clicks(&self, mouse_button: Option<MouseButton>, tag: Option<T>) -> usize {
        let all_events = self.mouse_events(mouse_button, tag);
        return all_events.filter(|c| c.is_just_clicked()).count();
    }

    /// Returns `true` if a left mouse button click was released on the node corresponding to `tag`, or `false` if the node is currently not part of the tree or if it doesn't exist.
    pub fn click_released(&self, mouse_button: Option<MouseButton>, tag: Option<T>) -> bool {
        let n_clicks = self.click_releases(mouse_button, tag);
        return n_clicks > 0;
    }

    pub fn click_releases(&self, mouse_button: Option<MouseButton>, tag: Option<T>) -> usize {
        let all_events = self.mouse_events(mouse_button, tag);
        return all_events.filter(|c| c.is_click_release()).count();
    }

    /// Returns `true` if a left mouse button drag on the node corresponding to `tag` was released, or `false` if the node is currently not part of the tree or if it doesn't exist.
    pub fn drag_released(&self, mouse_button: Option<MouseButton>, tag: Option<T>) -> bool {
        let n_clicks = self.drag_releases(mouse_button, tag);
        return n_clicks > 0;
    }

    /// Returns `true` if a left button mouse drag on the node corresponding to the `src` key was just released onto the node corresponding to the `dest` key.
    pub fn drag_released_onto(&self, mouse_button: Option<MouseButton>, src_tag: Option<T>, dest_tag: Option<T>) -> bool {
        let all_events = self.mouse_events(mouse_button, src_tag);
        let n_clicks = all_events.filter(|c| {
            let is_release = c.is_drag_release();
            let ends_on_dest = match dest_tag {
                Some(dest) => c.currently_at.tag.contains(&dest),
                None => true,
            };
            is_release && ends_on_dest
        }).count();
        return n_clicks > 0;
    }

    pub fn drag_releases(&self, mouse_button: Option<MouseButton>, tag: Option<T>) -> usize {
        let all_events = self.mouse_events(mouse_button, tag);
        return all_events.filter(|c| c.is_drag_release()).count();
    }

    /// Returns `true` if the mouse button was just pressed on the node corresponding to `tag` (first frame of press), or `false` if the node is currently not part of the tree or if it doesn't exist.
    pub fn just_clicked(&self, mouse_button: Option<MouseButton>, tag: Option<T>) -> bool {
        self.mouse_events(mouse_button, tag).any(|e| e.is_just_clicked())
    }

    /// Returns the drag distance for a mouse button on a node.
    pub fn dragged(&self, mouse_button: Option<MouseButton>, tag: Option<T>) -> (f32, f32) {
        let all_events = self.mouse_events(mouse_button, tag);

        let mut dist = vec2(0.0, 0.0);
        
        for e in all_events {
            dist += e.drag_distance();
        }

        return (dist.x, dist.y);
    }

    pub fn dragged_at(&self, mouse_button: Option<MouseButton>, tag: Option<T>) -> Option<FullMouseEvent<T>> {
        let last_drag = self.mouse_events(mouse_button, tag).last()?;
        return Some(last_drag.clone());
    }

    /// Returns the time a mouse button was held on a node and its last position, or `None` if it wasn’t held.
    pub fn held(&self, mouse_button: Option<MouseButton>, tag: Option<T>) -> Option<Duration> {
        // this used to return a more accurate position, but I doubt anybody cares
        let all_events = self.mouse_events(mouse_button, tag);

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

    /// Returns all scroll events for a specific node tag, or all scroll events if tag is None.
    pub fn scroll_events(&self, tag: Option<T>) -> impl Iterator<Item = &ScrollEvent<T>> {
        self.last_frame_scroll_events.iter().filter(move |s| {
            match tag {
                Some(tag) => s.tag.contains(&tag),
                None => true,
            }
        })
    }

    /// Returns the total scroll delta for a specific node tag, or None if no scroll events occurred.
    pub fn scrolled(&self, tag: Option<T>) -> Option<Vec2> {
        let mut total_delta = Vec2::ZERO;
        let mut found_any = false;
        
        for event in self.scroll_events(tag) {
            total_delta += event.delta;
            found_any = true;
        }
        
        if found_any { Some(total_delta) } else { None }
    }

    /// Returns the most recent scroll event for a specific node tag.
    pub fn last_scroll_event(&self, tag: Option<T>) -> Option<&ScrollEvent<T>> {
        self.scroll_events(tag).last()
    }


    /// Returns the total scroll delta for all scroll events that occurred this frame, regardless of which node they occurred on.
    /// This is useful for global scroll handling like Ctrl+wheel for font size adjustment.
    pub fn global_scroll_delta(&self) -> Option<Vec2> {
        return self.scrolled(None);
    }

    pub(crate) fn prev_cursor_position(&self) -> Vec2 {
        self.prev_cursor_position
    }

    /// Returns an iterator over all currently pressed mouse buttons and their associated tags (node IDs).
    /// This is useful for checking if any nodes are currently being dragged.
    /// Returns the first tag in the SmallVec for each press (if any).
    pub fn currently_pressed_tags(&self) -> impl Iterator<Item = (Option<T>, MouseButton)> + '_ {
        self.unresolved_click_presses.iter().map(|press| (press.pressed_at.tag.first().copied(), press.button))
    }
}


/// A record describing where and when a mouse event occurred.
/// 
/// The `tag` field can be used for any extra information. For example, `Keru` uses it to store the `id` of the clicked node, 
/// 
/// This can represent either a mouse click or a mouse release. This is only used inside `FullMouseEvent`, where this is always clear from the context.
#[derive(Clone, Debug)]
pub struct MouseRecord<T: Tag> {
    pub position: Vec2,
    pub timestamp: Instant,
    pub tag: SmallVec<T>,
}

/// A record describing a scroll event and which node it occurred on.
#[derive(Clone, Debug)]
pub struct ScrollEvent<T: Tag> {
    pub delta: Vec2,
    pub position: Vec2,
    pub timestamp: Instant,
    pub tag: SmallVec<T>,
}

/// A mouse press that has to be matched to a future mouse release.
/// 
/// Not part of the public API.
#[derive(Clone, Debug)]
pub(crate) struct PendingMousePress<T: Tag> {
    pub button: MouseButton,
    pub pressed_at: MouseRecord<T>,
    pub last_seen: MouseRecord<T>,
    pub already_released: bool,
}
impl<T: Tag> PendingMousePress<T> {
    pub fn new(event: MouseRecord<T>, button: MouseButton) -> Self {
        return Self {
            button,
            pressed_at: event.clone(),
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
/// You can use the [`FullMouseEvent::is_just_clicked`] and the other methods to map these events into more familiar concepts.
#[derive(Clone, Debug)]
pub struct FullMouseEvent<T: Tag> {
    pub button: MouseButton,
    pub originally_pressed: MouseRecord<T>,
    /// The last position the mouse was seen at before the event's conclusion
    pub last_seen: MouseRecord<T>,
    pub currently_at: MouseRecord<T>,
    pub kind: IsMouseReleased,
}
impl<T: Tag> FullMouseEvent<T> {
    // maybe a bit stupid compared to storing it explicitly, but should work.
    // if it stays there for more than 1 frame, the last_seen timestamp gets updated to the end of the frame.
    pub fn is_just_clicked(&self) -> bool {
        return self.originally_pressed.timestamp == self.last_seen.timestamp;
    }

    pub fn is_click_release(&self) -> bool {
        let is_click_release = self.kind == IsMouseReleased::MouseReleased;
        let is_on_same_node = self.originally_pressed.tag == self.currently_at.tag;
        return is_click_release && is_on_same_node;
    }

    // Less strict release that works if the pointer is not on the same node when it releases
    pub fn is_drag_release(&self) -> bool {
        let is_click_release = self.kind == IsMouseReleased::MouseReleased;
        return is_click_release;
    }

    pub fn drag_distance(&self) -> Vec2 {
        return self.last_seen.position - self.currently_at.position;
    }

    pub fn time_held(&self) -> Duration {
        return self.currently_at.timestamp.duration_since(self.last_seen.timestamp);
    }
}