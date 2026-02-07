#![allow(dead_code)]

use keru::example_window_loop::*;
use keru::*;
use winit::keyboard::Key;

#[derive(Default)]
pub struct State {
    elements: Vec<u32>,
    next_id: u32,
    show: bool,
}

const INTRO_TAB: Tab = Tab("Intro");
const TEXT_TAB: Tab = Tab("Text");
const GRAPHICS_TAB: Tab = Tab("Graphics");

#[component_key] const MY_TABS: ComponentKey<TabContainer<'static>>;
#[component_key] const MY_TABS2: ComponentKey<TabContainer<'static>>;

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const ADD: NodeKey;
    #[node_key] const DELETE: NodeKey;

    // Press Z to go to the first tab
    if ui.key_input().key_pressed(&Key::Character("z".into())) {
        if let Some(tabs_state) = ui.component_state_mut(MY_TABS) {
            tabs_state.i = 0;
        }
    }
    // Press X to go to the first tab of the nested one
    if ui.key_input().key_pressed(&Key::Character("x".into())) {
        if let Some(tabs_state) = ui.component_state_mut(MY_TABS2) {
            tabs_state.i = 0;
        }
    }

    let vert_tabs = TabContainer::new(&[INTRO_TAB, TEXT_TAB, GRAPHICS_TAB]).key(MY_TABS);

    // instead of returning stuff like this, we could also use component_state_mut to get the state out, or other things.
    let (parent, current_tab) = ui.add_component(vert_tabs);
    parent.nest(|| {
        match current_tab {
            INTRO_TAB => {
                ui.add(LABEL.text("asdasd"));
            },
            TEXT_TAB => {

                // nest the same thing inside
                let vert_tabs = TabContainer::new(&[INTRO_TAB, TEXT_TAB, GRAPHICS_TAB]).key(MY_TABS2);

                let (parent, current_tab) = ui.add_component(vert_tabs);
                parent.nest(|| {
                    match current_tab {
                        INTRO_TAB => {
                            ui.add(LABEL.text("asdasd"));
                        },
                        TEXT_TAB => {
                            ui.add(BUTTON.text("Buttons"));
                        },
                        GRAPHICS_TAB => {
                            ui.add(BUTTON.text("Graphics o algo"));
                        },
                        _ => {}
                    }
                });
                
            },
            GRAPHICS_TAB => {
                ui.add(BUTTON.text("Graphics o algo"));
            },
            _ => {}
        }
    });
}

fn main() {
    // basic_env_logger_init();
    let state = State {
        elements: vec![0, 1, 2],
        next_id: 3,
        show: true,
    };
    run_example_loop(state, update_ui);
}