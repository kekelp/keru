use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {
    pub current_tab: usize,
    pub show: bool,
}

impl ExampleLoop for State {
    fn update_ui(&mut self, ui: &mut Ui) {
        #[node_key] const MOVING_NODE: NodeKey;
        #[node_key] const V_STACK_KEY: NodeKey;
        #[node_key] const SHOW: NodeKey;
        #[node_key] const CONT_1: NodeKey;
        #[node_key] const CONT_2: NodeKey;

        let button = BUTTON.size_symm(Size::Pixels(200));

        ui.add(V_STACK.key(V_STACK_KEY)).nest(|| {
            ui.add(button);
            ui.add(button);
        });
    }
}

fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Warn)
        .filter_module("keru::tree", log::LevelFilter::Trace)
        .init();
    let mut state = State::default();
    state.show = true;
    run_example_loop(state);
}
