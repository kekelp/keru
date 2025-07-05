use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {
    pub current_tab: usize,
    pub show: bool,
}

impl State {
    fn update_ui(&mut self, ui: &mut Ui) {
        #[node_key] const BUTTON3: NodeKey;
        #[state_key] const WIDGET_STATE: StateKey<bool>;

        ui.h_stack().nest(|| {
            ui.add(BUTTON.key(BUTTON3));

            if ui.is_clicked(BUTTON3) {
                *ui.state_mut(WIDGET_STATE) = ! ui.state(WIDGET_STATE)
            }

            #[node_key] pub const KYS2: NodeKey;
            if *ui.state(WIDGET_STATE) {
                ui.add(LABEL.key(KYS2).static_text("Bool on"));
            } else {
                ui.add(LABEL.key(KYS2).static_text("Bool off"));
            }
        });
    }
}

fn main() {
    // env_logger::Builder::new()
    //     .filter_level(log::LevelFilter::Warn)
    //     .filter_module("keru", log::LevelFilter::Info)
    //     .init();
    let mut state = State::default();
    state.show = true;
    run_example_loop(state, State::update_ui);
}
