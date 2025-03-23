use glyphon::{cosmic_text::Motion, Action, Edit, Editor, FontSystem};
use winit::{event::{KeyEvent, WindowEvent}, keyboard::{Key, NamedKey}};

use crate::*;

pub trait RealEdit {
    fn actions_from_events(&mut self, event: &WindowEvent, font_system: &mut FontSystem);
}

impl RealEdit for Editor<'static> {
    fn actions_from_events(&mut self, event: &WindowEvent, font_system: &mut FontSystem) {
        let mut editor = self.borrow_with(font_system);
        match event {
            
            WindowEvent::KeyboardInput { event, .. } => {
                let KeyEvent {
                    logical_key, state, ..
                } = event;

                if state.is_pressed() {
                    match logical_key {
                        Key::Named(NamedKey::ArrowLeft) => {
                            editor.action(Action::Motion(Motion::Left))
                        }
                        Key::Named(NamedKey::ArrowRight) => {
                            editor.action(Action::Motion(Motion::Right))
                        }
                        Key::Named(NamedKey::ArrowUp) => {
                            editor.action(Action::Motion(Motion::Up))
                        }
                        Key::Named(NamedKey::ArrowDown) => {
                            editor.action(Action::Motion(Motion::Down))
                        }
                        Key::Named(NamedKey::Home) => {
                            editor.action(Action::Motion(Motion::Home))
                        }
                        Key::Named(NamedKey::End) => editor.action(Action::Motion(Motion::End)),
                        Key::Named(NamedKey::PageUp) => {
                            editor.action(Action::Motion(Motion::PageUp))
                        }
                        Key::Named(NamedKey::PageDown) => {
                            editor.action(Action::Motion(Motion::PageDown))
                        }
                        Key::Named(NamedKey::Escape) => editor.action(Action::Escape),
                        Key::Named(NamedKey::Enter) => editor.action(Action::Enter),
                        Key::Named(NamedKey::Backspace) => editor.action(Action::Backspace),
                        Key::Named(NamedKey::Delete) => editor.action(Action::Delete),
                        Key::Named(key) => {
                            if let Some(text) = key.to_text() {
                                for c in text.chars() {
                                    editor.action(Action::Insert(c));
                                }
                            }
                        }
                        Key::Character(text) => {
                            if false {
                               
                            } else {
                                for c in text.chars() {
                                    editor.action(Action::Insert(c));
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            _ => {},
        }
    }
}