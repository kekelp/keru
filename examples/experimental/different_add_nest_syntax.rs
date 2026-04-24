#![allow(unused)]
use keru::*;
use keru::example_window_loop::*;

struct State {}


fn update_ui(state: &mut State, ui: &mut Ui) {
    ui.add2(V_STACK).nest().enter(|| {

        ui.add2(BUTTON.text("Hello"));
        if ui.add2(BUTTON.text("World")).is_clicked() {
            println!("World");
        }

    });
}

fn main() {
    let state = State {};
    example_window_loop::run_example_loop(state, update_ui);
}
