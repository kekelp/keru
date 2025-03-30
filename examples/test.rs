use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {
    pub current_tab: usize,
    pub show: bool,
}

impl State {
    fn update_ui(&mut self, ui: &mut Ui) {
        let params = TEXT_EDIT.static_text("'moko");
        ui.add(params);
    }
}

fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Warn)
        .filter_module("keru::tree", log::LevelFilter::Trace)
        .init();
    let mut state = State::default();
    state.show = true;
    run_example_loop(state, State::update_ui);
}
