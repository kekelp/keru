#![allow(unused)]
use keru::*;
use keru::example_window_loop::*;

struct State {
    count: usize,
    flow: GridFlow,
    columns: MainAxisCellSize,
}

fn update_ui(state: &mut State, ui: &mut Ui) {

}

fn main() {
    let state = State { count: 9, flow: GridFlow::DEFAULT, columns: MainAxisCellSize::Count(4) };
    example_window_loop::run_example_loop(state, update_ui);
}
