#![allow(unused)]
use keru::*;
use keru::example_window_loop::*;

struct State {}


fn update_ui(state: &mut State, ui: &mut Ui) {
    ui.add(BUTTON.text("Hello"));
}

fn main() {
    let state = State {};
    example_window_loop::run_example_loop(state, update_ui);
}
