use keru::*;
use keru::Size::*;
use keru::example_window_loop::*;

#[derive(Default)]
pub struct State {
    pub count: i32,
    pub show: bool,
}

impl ExampleLoop for State {
    fn update_ui(&mut self, ui: &mut Ui) {

        ui.subtree().start(|| {

            
            ui.add(PANEL.size_symm(FitContent)).nest(|| {
                ui.add(BUTTON.size_symm(Size::Fill).text("Sneed"));                
            });
            
        });
    }
}

fn main() {
    basic_env_logger_init();
    let state = State::default();
    run_example_loop(state);
}
