#![allow(dead_code)]

use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {
    elements: Vec<u32>,
    next_id: u32,
    show: bool,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const ADD: NodeKey;
    #[node_key] const DELETE: NodeKey;

    let button = BUTTON.key(ADD).static_text("SEETHE").position_y(Position::End);

    ui.add(button);

    if ui.is_clicked(ADD) {
        dbg!("click");
    }

    if ui.is_click_released(ADD) {
        dbg!("click release");
    }

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