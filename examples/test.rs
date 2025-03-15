use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {
    pub current_tab: usize,
    pub show: bool,
}

impl ExampleLoop for State {
    fn update_ui(&mut self, ui: &mut Ui) {
        let moving_node = BUTTON.text(&"Sneed");

        ui.add(BUTTON.text(&"My child will type sneed2")).nest(|| {
            if self.show {
                ui.add(moving_node);
            }
        });
        ui.add(BUTTON.text(&"My child will type sneed1")).nest(|| {
            if ! self.show {
                ui.add(moving_node);
            }
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
