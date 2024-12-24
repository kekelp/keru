use std::time::{Duration, Instant};
use std::fmt::Debug;

use winit::event::{ElementState, WindowEvent};
use winit::keyboard::Key;

pub struct KeyInput {
    unresolved_key_presses: Vec<PendingKeyPress>,
    last_frame_key_events: Vec<FullKeyEvent>,
    key_repeats: Vec<Key>,
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

impl Default for KeyInput {
    fn default() -> Self {
        Self {
            unresolved_key_presses: Vec::with_capacity(20),
            last_frame_key_events: Vec::with_capacity(20),
            key_repeats: Vec::with_capacity(10),
        }
    }
}

impl KeyInput {
    // updating
    pub fn begin_new_frame(&mut self) {
        let current_mouse_status = Instant::now();

        self.key_repeats.clear();
        
        self.last_frame_key_events.clear();

        self.unresolved_key_presses.retain(|click| click.already_released == false);

        for key_pressed in self.unresolved_key_presses.iter_mut().rev() {

            let mouse_happening = FullKeyEvent {
                key: key_pressed.key.clone(),
                originally_pressed: key_pressed.pressed_at,
                last_seen: key_pressed.last_seen,
                currently_at: current_mouse_status,
                kind: IsKeyReleased::StillDownButFrameEnded,
            };

            self.last_frame_key_events.push(mouse_happening);

            key_pressed.last_seen = current_mouse_status;
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event, is_synthetic, .. } => {
                if ! is_synthetic {
                    if event.state == ElementState::Pressed {
                        if ! event.repeat {
                            self.push_key_press(&event.logical_key);
                        } else {
                            self.push_key_repeat(&event.logical_key);
                        }
                    } else {
                        self.push_key_release(&event.logical_key);
                    }
                }
            }
            _ => {}
        }
    }

    fn push_key_press(&mut self, key: &Key) {
        let timestamp = Instant::now();
        let pending_press = PendingKeyPress::new(timestamp, &key);
        self.unresolved_key_presses.push(pending_press);
    }

    fn push_key_repeat(&mut self, key: &Key) {
        self.key_repeats.push(key.clone());
    }

    fn push_key_release(&mut self, key: &Key) {
        // look for a mouse press to match and resolve
        let mut matched = None;
        for click_pressed in self.unresolved_key_presses.iter_mut().rev() {
            if click_pressed.key == *key {
                click_pressed.already_released = true;
                // this copy is a classic borrow checker skill issue.
                matched = Some(click_pressed.clone());
                break;
            }
        };

        let timestamp = Instant::now();

        if let Some(matched) = matched {
            let full_mouse_event = FullKeyEvent {
                key: key.clone(),
                originally_pressed: matched.pressed_at,
                last_seen: matched.last_seen,
                currently_at: timestamp,
                kind: IsKeyReleased::KeyReleased,
            };

            self.last_frame_key_events.push(full_mouse_event);
        }
    }

    // querying
    pub fn all_key_events(&self) -> impl DoubleEndedIterator<Item = &FullKeyEvent> {
        return self.last_frame_key_events.iter();
    }

    pub fn key_events(&self, key: &Key) -> impl DoubleEndedIterator<Item = &FullKeyEvent> {
        let key = key.clone();
        return self
            .all_key_events()
            .filter(move |c| c.key == key);    }

    pub fn key_pressed(&self, key: &Key) -> bool {
        let all_events = self.key_events(key);
        let count = all_events.filter(|c| c.is_just_pressed()).count();
        return count > 0;
    }

    pub fn key_repeated(&self, key: &Key) -> bool {
        let count = self.key_repeats.iter().filter(|c| *c == key).count(); // ???
        return count > 0;
    }

    pub fn key_pressed_or_repeated(&self, key: &Key) -> bool {
        return self.key_pressed(key) || self.key_repeated(key);
    }

    pub fn time_key_held(&self, key: &Key) -> Option<Duration> {
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
    pub fn key_held(&self, key: &Key) -> bool {
        let duration = self.time_key_held(key);
        return duration > Some(Duration::ZERO);
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
    pub fn new(timestamp: Instant, key: &Key) -> Self {
        return Self {
            key: key.clone(),
            pressed_at: timestamp,
            last_seen: timestamp,
            already_released: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FullKeyEvent {
    pub key: Key,
    pub originally_pressed: Instant,
    pub last_seen: Instant,
    // rename to current_time or something, or maybe remove? it's just Instant::now() 
    pub currently_at: Instant,
    pub kind: IsKeyReleased,
}
impl FullKeyEvent {
    // if it stays there for more than 1 frame, the last_seen timestamp gets updated to the end of the frame.
    pub fn is_just_pressed(&self) -> bool {
        return self.originally_pressed == self.last_seen;
    }

    pub fn is_released(&self) -> bool {
        let is_released = self.kind == IsKeyReleased::KeyReleased;
        return is_released;
    }

    pub fn time_held(&self) -> Duration {
        return self.currently_at.duration_since(self.last_seen);
    }
}
