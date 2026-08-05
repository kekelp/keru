use keru::*;
use keru::node_library::*;
use keru::example_window_loop::*;

#[derive(Default)]
pub struct State {
    f: f32,
}

#[node_key] const K1: NodeKey;
#[node_key] const K2: NodeKey;
#[node_key] const K3: NodeKey;
#[node_key] const VSTACKKEY: NodeKey;
#[node_key] const WRAP: NodeKey;
#[node_key] const REDFILL: NodeKey;
#[node_key] const K7: NodeKey;
#[node_key] const K8: NodeKey;
#[node_key] const PANELKEY: NodeKey;
#[node_key] const K10: NodeKey;

impl State {
    fn update_ui(&mut self, ui: &mut Ui) {

    }
}

fn main() {
    basic_env_logger_init();
    let state = State::default();
    run_example_loop(state, State::update_ui);
}
