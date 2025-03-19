use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {
    pub current_tab: usize,
    pub show: bool,
}

impl State {
    fn update_ui(&mut self, ui: &mut Ui) {
        let string = "String".to_string();
        let str_ref = "string ref";
        let number: f32 = 17.5;

        let copy_this = BUTTON.text(&string);

        ui.v_stack().nest(|| {
            ui.add(BUTTON.text(str_ref));
            ui.add(BUTTON.text(&str_ref));
            ui.add(BUTTON.text(&string));
            ui.add(BUTTON.text(number.to_string().as_str()));

            ui.add(BUTTON.hashed_text(str_ref));
            ui.add(BUTTON.hashed_text(&string));
            ui.add(BUTTON.hashed_text(&number.to_string()));

            ui.add(copy_this);
            ui.add(copy_this);
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
    run_example_loop(state, State::update_ui);
}
