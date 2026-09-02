#![allow(dead_code)]

use keru::*;
use keru::node_library::*;
use keru::example_window_loop::*;
use Size::*;

#[derive(Default)]
pub struct State {
    f: f32,
}

#[node_key] const A: NodeKey;
#[node_key] const B: NodeKey;
#[node_key] const C: NodeKey;
#[node_key] const D: NodeKey;


impl State {
    fn update_ui(&mut self, ui: &mut Ui) {
        // Giraffe
        ui.add(CONTAINER.key(D).size_x(AspectRatio(1.0)).size_y(FitContent)).nest(|| {
            ui.add(DEFAULT.key(A).size(Pixels(50.0), Pixels(50.0)));
            ui.add(DEFAULT.key(B).size_x(Fill).size_y(AspectRatio(1.0)));
        });
    }
}

fn main() {
    basic_env_logger_init();
    let state = State::default();
    run_example_loop(state, State::update_ui);
}
