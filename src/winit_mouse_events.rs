use std::time::{Duration, Instant};

use glam::{vec2, Vec2};
use winit::event::{ElementState, MouseButton, WindowEvent};

use crate::Id;

pub(crate) type SmallVec<T> = smallvec::SmallVec<[T; 8]>;

pub struct MouseInput {
    pending_presses: Vec<PendingPress>,
    pub events: Vec<MouseEvent>,
    pub scroll_events: Vec<ScrollEvent>,
    pub cursor_position: Vec2,
    pub prev_cursor_position: Vec2,
}

impl Default for MouseInput {
    fn default() -> Self {
        Self {
            pending_presses: Vec::with_capacity(5),
            events: Vec::with_capacity(20),
            scroll_events: Vec::with_capacity(10),
            cursor_position: Vec2::ZERO,
            prev_cursor_position: Vec2::ZERO,
        }
    }
}

impl MouseInput {
    pub fn begin_new_frame(&mut self, hovered_ids: &SmallVec<Id>) {
        // Remove released presses, generate "still down" events for the rest
        self.pending_presses.retain(|p| !p.released);

        let now = Instant::now();
        for press in &mut self.pending_presses {
            self.events.push(MouseEvent {
                button: press.button,
                press_pos: press.press_pos,
                press_time: press.press_time,
                click_ids: press.click_ids.clone(),
                drag_ids: press.drag_ids.clone(),
                last_pos: press.last_pos,
                last_time: press.last_time,
                current_pos: self.cursor_position,
                current_time: now,
                current_ids: hovered_ids.clone(),
                released: false,
            });
            press.last_pos = self.cursor_position;
            press.last_time = now;
        }
    }

    pub fn finish_frame(&mut self) {
        self.events.clear();
        self.scroll_events.clear();
    }

    pub fn window_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.prev_cursor_position = self.cursor_position;
                self.cursor_position = vec2(position.x as f32, position.y as f32);
            }
            WindowEvent::MouseInput { button, state, .. } => {
                match state {
                    ElementState::Pressed => {
                        // Press handling is done in Ui::ui_input with the resolved hovered IDs
                    }
                    ElementState::Released => {
                        // Release handling is done in Ui::ui_input with the resolved hovered IDs
                    }
                }
                let _ = button; // Used externally
            }
            _ => {}
        }
    }

    pub fn push_press(&mut self, button: MouseButton, click_ids: SmallVec<Id>, drag_ids: SmallVec<Id>) {
        let now = Instant::now();
        self.pending_presses.push(PendingPress {
            button,
            press_pos: self.cursor_position,
            press_time: now,
            click_ids,
            drag_ids,
            last_pos: self.cursor_position,
            last_time: now,
            released: false,
        });
    }

    pub fn push_release(&mut self, button: MouseButton, current_ids: SmallVec<Id>) {
        let now = Instant::now();
        for press in &mut self.pending_presses {
            if press.button == button && !press.released {
                press.released = true;
                self.events.push(MouseEvent {
                    button,
                    press_pos: press.press_pos,
                    press_time: press.press_time,
                    click_ids: press.click_ids.clone(),
                    drag_ids: press.drag_ids.clone(),
                    last_pos: press.last_pos,
                    last_time: press.last_time,
                    current_pos: self.cursor_position,
                    current_time: now,
                    current_ids: current_ids.clone(),
                    released: true,
                });
            }
        }
    }

    pub fn push_scroll(&mut self, delta: Vec2, target_id: Id) {
        self.scroll_events.push(ScrollEvent {
            delta,
            position: self.cursor_position,
            timestamp: Instant::now(),
            target_id,
        });
    }

    pub fn cursor_position(&self) -> Vec2 {
        self.cursor_position
    }

    pub fn prev_cursor_position(&self) -> Vec2 {
        self.prev_cursor_position
    }

    /// Returns IDs of nodes currently being dragged
    pub fn currently_pressed(&self) -> impl Iterator<Item = (Id, MouseButton)> + '_ {
        self.pending_presses.iter()
            .filter(|p| !p.released)
            .filter_map(|p| p.drag_ids.first().map(|id| (*id, p.button)))
    }
}

struct PendingPress {
    button: MouseButton,
    press_pos: Vec2,
    press_time: Instant,
    click_ids: SmallVec<Id>,
    drag_ids: SmallVec<Id>,
    last_pos: Vec2,
    last_time: Instant,
    released: bool,
}

#[derive(Clone, Debug)]
pub struct MouseEvent {
    pub button: MouseButton,
    pub press_pos: Vec2,
    pub press_time: Instant,
    pub click_ids: SmallVec<Id>,
    pub drag_ids: SmallVec<Id>,
    pub last_pos: Vec2,
    pub last_time: Instant,
    pub current_pos: Vec2,
    pub current_time: Instant,
    pub current_ids: SmallVec<Id>,
    pub released: bool,
}

impl MouseEvent {
    pub fn is_just_pressed(&self) -> bool {
        self.press_time == self.last_time
    }

    pub fn is_click(&self) -> bool {
        self.released && self.click_ids == self.current_ids
    }

    pub fn is_drag_release(&self) -> bool {
        self.released && self.total_drag() != Vec2::ZERO
    }

    pub fn frame_drag(&self) -> Vec2 {
        self.last_pos - self.current_pos
    }

    pub fn total_drag(&self) -> Vec2 {
        self.press_pos - self.current_pos
    }

    pub fn time_held(&self) -> Duration {
        self.current_time.duration_since(self.last_time)
    }
}

#[derive(Clone, Debug)]
pub struct ScrollEvent {
    pub delta: Vec2,
    pub position: Vec2,
    pub timestamp: Instant,
    pub target_id: Id,
}
