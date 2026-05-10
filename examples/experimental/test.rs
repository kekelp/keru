#![allow(unused)]
use keru::*;
use keru::node_library::*;
use keru::example_window_loop::*;

struct State {}

fn update_ui(state: &mut State, ui: &mut Ui) {
    ui.add(V_SCROLL_STACK).nest(|| {
        for _ in 0..10_000 {
            let node = PANEL
            .color(Color::BLUE)
            .text("Hello")
            .size_y(Size::FitContent)
            .size_x(Size::Pixels(100.0))
            ;

            ui.add(node);
        }
    });
}

fn main() {
    let state = State {};
    example_window_loop::run_example_loop(state, update_ui);
}




