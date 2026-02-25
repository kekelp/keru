use std::time::Instant;

use glam::{vec2, Vec2};
use winit::event::{MouseButton, WindowEvent};

use crate::Id;

pub(crate) type SmallVec<T> = smallvec::SmallVec<[T; 8]>;

#[derive(Clone, Debug)]
pub enum InputEvent {
    /// Mouse button was just pressed
    Click(PressEvent),

    /// Mouse button was pressed and released on the same node(s)
    ClickRelease(ClickEvent),

    /// Ongoing drag - emitted each frame while dragging
    Drag(DragEvent),

    /// Drag just ended
    DragRelease(DragReleaseEvent),

    /// Scroll wheel
    Scroll(ScrollEvent),
}

#[derive(Clone, Debug)]
pub struct PressEvent {
    pub targets: SmallVec<Id>,
    pub position: Vec2,
    pub button: MouseButton,
    pub timestamp: Instant,
}

#[derive(Clone, Debug)]
pub struct ClickEvent {
    pub targets: SmallVec<Id>,
    pub position: Vec2,
    pub button: MouseButton,
    pub press_time: Instant,
}

#[derive(Clone, Debug)]
pub struct DragEvent {
    pub targets: SmallVec<Id>,
    pub button: MouseButton,
    pub start_pos: Vec2,
    pub current_pos: Vec2,
    pub frame_delta: Vec2,
    pub total_delta: Vec2,
    pub start_time: Instant,
}

#[derive(Clone, Debug)]
pub struct DragReleaseEvent {
    pub targets: SmallVec<Id>,
    pub button: MouseButton,
    pub start_pos: Vec2,
    pub end_pos: Vec2,
    pub total_delta: Vec2,
    pub start_time: Instant,
}

#[derive(Clone, Debug)]
pub struct ScrollEvent {
    pub target: Id,
    pub delta: Vec2,
    pub position: Vec2,
    pub timestamp: Instant,
}

// Pending state

#[derive(Clone, Debug)]
pub(crate) enum Pending {
    /// Tracks a potential click - resolved on release
    Click {
        button: MouseButton,
        press_pos: Vec2,
        press_time: Instant,
        targets: SmallVec<Id>,
    },

    /// Tracks an ongoing drag - emits Drag events each frame
    Drag {
        button: MouseButton,
        start_pos: Vec2,
        start_time: Instant,
        last_pos: Vec2,
        targets: SmallVec<Id>,
    },
}

impl Pending {
    fn button(&self) -> MouseButton {
        match self {
            Pending::Click { button, .. } => *button,
            Pending::Drag { button, .. } => *button,
        }
    }
}

pub struct MouseInput {
    pub events: Vec<InputEvent>,
    pending: Vec<Pending>,
    pub cursor_position: Vec2,
    pub prev_cursor_position: Vec2,
}

impl Default for MouseInput {
    fn default() -> Self {
        Self {
            events: Vec::with_capacity(20),
            pending: Vec::with_capacity(5),
            cursor_position: Vec2::ZERO,
            prev_cursor_position: Vec2::ZERO,
        }
    }
}

impl MouseInput {
    /// Called at the start of each frame to generate Drag events for ongoing drags
    pub fn begin_new_frame(&mut self) {
        for pending in &mut self.pending {
            if let Pending::Drag { button, start_pos, start_time, last_pos, targets } = pending {
                let frame_delta = self.cursor_position - *last_pos;
                let total_delta = self.cursor_position - *start_pos;

                self.events.push(InputEvent::Drag(DragEvent {
                    targets: targets.clone(),
                    button: *button,
                    start_pos: *start_pos,
                    current_pos: self.cursor_position,
                    frame_delta,
                    total_delta,
                    start_time: *start_time,
                }));

                *last_pos = self.cursor_position;
            }
        }
    }

    /// Called at the end of each frame to clear events
    pub fn finish_frame(&mut self) {
        self.events.clear();
    }

    pub fn window_event(&mut self, event: &WindowEvent) {
        if let WindowEvent::CursorMoved { position, .. } = event {
            self.prev_cursor_position = self.cursor_position;
            self.cursor_position = vec2(position.x as f32, position.y as f32);
        }
    }

    /// Called when mouse button is pressed
    pub fn push_press(
        &mut self,
        button: MouseButton,
        click_targets: SmallVec<Id>,
        drag_targets: SmallVec<Id>,
    ) {
        let now = Instant::now();

        // Emit Press event immediately
        if !click_targets.is_empty() {
            self.events.push(InputEvent::Click(PressEvent {
                targets: click_targets.clone(),
                position: self.cursor_position,
                button,
                timestamp: now,
            }));

            // Track for potential Click on release
            self.pending.push(Pending::Click {
                button,
                press_pos: self.cursor_position,
                press_time: now,
                targets: click_targets,
            });
        }

        // Push PendingDrag if there are drag targets
        if !drag_targets.is_empty() {
            self.pending.push(Pending::Drag {
                button,
                start_pos: self.cursor_position,
                start_time: now,
                last_pos: self.cursor_position,
                targets: drag_targets,
            });
        }
    }

    /// Called when mouse button is released
    pub fn push_release(&mut self, button: MouseButton, current_click_targets: SmallVec<Id>) {
        // Collect all pending entries for this button
        let mut i = 0;
        while i < self.pending.len() {
            if self.pending[i].button() == button {
                let pending = self.pending.remove(i);
                match pending {
                    Pending::Click { press_pos, press_time, targets, .. } => {
                        // Click if released on same targets as pressed
                        if targets == current_click_targets {
                            self.events.push(InputEvent::ClickRelease(ClickEvent {
                                targets,
                                position: self.cursor_position,
                                button,
                                press_time,
                            }));
                        }
                        let _ = press_pos; // unused for now, might use for threshold later
                    }
                    Pending::Drag { start_pos, start_time, targets, .. } => {
                        self.events.push(InputEvent::DragRelease(DragReleaseEvent {
                            targets,
                            button,
                            start_pos,
                            end_pos: self.cursor_position,
                            total_delta: self.cursor_position - start_pos,
                            start_time,
                        }));
                    }
                }
            } else {
                i += 1;
            }
        }
    }

    pub fn push_scroll(&mut self, delta: Vec2, target: Id) {
        self.events.push(InputEvent::Scroll(ScrollEvent {
            target,
            delta,
            position: self.cursor_position,
            timestamp: Instant::now(),
        }));
    }
    /// Returns IDs of nodes currently being dragged
    pub fn currently_dragging(&self) -> impl Iterator<Item = (&Id, MouseButton)> + '_ {
        self.pending.iter().filter_map(|p| match p {
            Pending::Drag { targets, button, .. } => targets.first().map(|id| (id, *button)),
            _ => None,
        })
    }
}

impl MouseInput {
    pub fn presses(&self) -> impl Iterator<Item = &PressEvent> {
        self.events.iter().filter_map(|e| match e {
            InputEvent::Click(ev) => Some(ev),
            _ => None,
        })
    }

    pub fn clicks(&self) -> impl Iterator<Item = &ClickEvent> {
        self.events.iter().filter_map(|e| match e {
            InputEvent::ClickRelease(ev) => Some(ev),
            _ => None,
        })
    }

    pub fn drags(&self) -> impl Iterator<Item = &DragEvent> {
        self.events.iter().filter_map(|e| match e {
            InputEvent::Drag(ev) => Some(ev),
            _ => None,
        })
    }

    pub fn drag_releases(&self) -> impl Iterator<Item = &DragReleaseEvent> {
        self.events.iter().filter_map(|e| match e {
            InputEvent::DragRelease(ev) => Some(ev),
            _ => None,
        })
    }

    pub fn scrolls(&self) -> impl Iterator<Item = &ScrollEvent> {
        self.events.iter().filter_map(|e| match e {
            InputEvent::Scroll(ev) => Some(ev),
            _ => None,
        })
    }
}
