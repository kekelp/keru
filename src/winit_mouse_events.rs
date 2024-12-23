use std::time::{Duration, Instant};

use glam::{dvec2, DVec2};
use winit::event::{ElementState, MouseButton, WindowEvent};

// todo: rewrite all doc comments

pub trait Tag: Clone + Copy + PartialEq {}
impl<T: Clone + Copy + PartialEq> Tag for T {}

pub struct MouseInput<T: Tag> {
    unresolved_click_presses: Vec<PendingMousePress<T>>,
    last_frame_mouse_events: Vec<FullMouseEvent<T>>,
    current_tag: Option<T>,
    cursor_position: DVec2,
    // todo: add tagged scrolling
}


impl<T: Tag> Default for MouseInput<T> {
    fn default() -> Self {
        return Self {
            unresolved_click_presses: Vec::with_capacity(20),
            last_frame_mouse_events: Vec::with_capacity(20),
            current_tag: None,
            cursor_position: Default::default(),
        }
    }
}

impl<T: Tag> MouseInput<T> {
    // updating
    pub fn begin_new_frame(&mut self) {
        let current_mouse_status = MouseRecord {
            position: self.cursor_position,
            timestamp: Instant::now(),
            tag: self.current_tag,
        };

        self.last_frame_mouse_events.clear();

        self.unresolved_click_presses.retain(|click| click.already_released == false);

        for click_pressed in self.unresolved_click_presses.iter_mut().rev() {

            let mouse_happening = FullMouseEvent {
                button: click_pressed.button,
                originally_pressed: click_pressed.pressed_at,
                last_seen: click_pressed.last_seen,
                currently_at: current_mouse_status,
                kind: IsMouseReleased::StillDownButFrameEnded,
            };

            self.last_frame_mouse_events.push(mouse_happening);

            click_pressed.last_seen = current_mouse_status;
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position = dvec2(position.x, position.y);
            },
            WindowEvent::MouseInput { button, state, .. } => {
                let tag = self.current_tag;
                match state {
                    ElementState::Pressed => {
                        self.record_click_press(*button, tag)
                    },
                    ElementState::Released => {
                        self.record_click_release(*button, tag);
                    },
                }
            }
            _ => {}
        }
    }

    pub fn update_current_tag(&mut self, new_tag: Option<T>) {
        self.current_tag = new_tag;
    }

    pub fn current_tag(&self) -> Option<T> {
        return self.current_tag;
    }

    pub fn cursor_position(&self) -> DVec2 {
        return self.cursor_position;
    }

    fn record_click_press(&mut self, button: MouseButton, current_tag: Option<T>) {
        let current_mouse_status = MouseRecord {
            position: self.cursor_position,
            timestamp: Instant::now(),
            tag: current_tag,
        };
        let pending_press = PendingMousePress::new(current_mouse_status, button);
        self.unresolved_click_presses.push(pending_press);
    }

    fn record_click_release(&mut self, button: MouseButton, current_tag: Option<T>) {
        // look for a mouse press to match and resolve
        let mut matched = None;
        for click_pressed in self.unresolved_click_presses.iter_mut().rev() {
            if click_pressed.button == button {
                click_pressed.already_released = true;
                // this copy is a classic borrow checker skill issue.
                matched = Some(*click_pressed);
                break;
            }
        };

        let current_mouse_status = MouseRecord {
            position: self.cursor_position,
            timestamp: Instant::now(),
            tag: current_tag,
        };

        if let Some(matched) = matched {
            let full_mouse_event = FullMouseEvent {
                button,
                originally_pressed: matched.pressed_at,
                last_seen: matched.last_seen,
                currently_at: current_mouse_status,
                kind: IsMouseReleased::MouseReleased,
            };

            self.last_frame_mouse_events.push(full_mouse_event);
        }
    }
    
    // querying

    /// Returns all [`FullMouseEvent`]s for a specific button on the node corresponding to `tag`, or an empty iterator if the node is currently not part of the tree or if it doesn't exist.
    pub fn mouse_events(
        &self, 
        mouse_button: Option<MouseButton>, 
        tag: Option<T>
    ) -> impl DoubleEndedIterator<Item = &FullMouseEvent<T>> {
        self.last_frame_mouse_events.iter().filter(move |c| {
            (mouse_button.is_none() || c.button == mouse_button.unwrap())
                && (tag.is_none() || c.originally_pressed.tag == tag)
        })
    }    

    /// Returns `true` if the left mouse button was clicked on the node corresponding to `tag`, or `false` if the node is currently not part of the tree or if it doesn't exist.
    pub fn clicked(&self, mouse_button: Option<MouseButton>, tag: Option<T>) -> bool {
        let n_clicks = self.clicks(mouse_button, tag);
        return n_clicks > 0;
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
        return all_events.filter(|c| c.is_just_clicked()).count();
    }

    /// Returns the drag distance for a mouse button on a node, or None if there was no drag.
    ///
    /// In the case where the user dragged, released, and redragged all in one frame,
    /// this sums the distances.
    pub fn dragged(&self, mouse_button: Option<MouseButton>, tag: Option<T>) -> (f64, f64) {
        let all_events = self.mouse_events(mouse_button, tag);

        let mut dist = glam::dvec2(0.0, 0.0);
        
        for e in all_events {
            dist = dist + e.drag_distance();
        }

        return (dist.x, dist.y);
    }

    /// Returns the time a mouse button was held on a node and its last position, or `None` if it wasn’t held.
    pub fn is_mouse_button_held(&self, mouse_button: Option<MouseButton>, tag: Option<T>) -> Option<(Duration, glam::DVec2)> {
        let all_events = self.mouse_events(mouse_button, tag);

        let mut time_held = Duration::ZERO;
        let mut last_pos = glam::dvec2(0.0, 0.0);

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
}


/// A record describing where and when a mouse event occurred.
/// 
/// The `tag` field can be used for any extra information. For example, `Keru` uses it to store the `id` of the clicked node, 
/// 
/// This can represent either a mouse click or a mouse release. This is only used inside `FullMouseEvent`, where this is always clear from the context.
#[derive(Clone, Copy, Debug)]
pub struct MouseRecord<T: Tag> {
    pub position: glam::DVec2,
    pub timestamp: Instant,
    pub tag: Option<T>,
}

/// A mouse press that has to be matched to a future mouse release.
/// 
/// Not part of the public API.
#[derive(Clone, Copy, Debug)]
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
pub struct FullMouseEvent<T: Tag> {
    pub button: MouseButton,
    pub originally_pressed: MouseRecord<T>,
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

    pub fn drag_distance(&self) -> glam::DVec2 {
        return self.last_seen.position - self.currently_at.position;
    }

    pub fn time_held(&self) -> Duration {
        return self.currently_at.timestamp.duration_since(self.last_seen.timestamp);
    }
}