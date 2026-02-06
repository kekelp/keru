#![allow(dead_code)]

use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {
    elements: Vec<u32>,
    next_id: u32,
    show: bool,
}

const INTRO_TAB: Tab = Tab("Intro");
const TEXT_TAB: Tab = Tab("Text");
const GRAPHICS_TAB: Tab = Tab("Graphics");

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const ADD: NodeKey;
    #[node_key] const DELETE: NodeKey;

    let vert_tabs = StatefulVerticalTabs { tabs: &[INTRO_TAB, TEXT_TAB, GRAPHICS_TAB] };

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