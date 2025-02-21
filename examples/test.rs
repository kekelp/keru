use keru::example_window_loop::*;
use keru::Size::*;
use keru::*;

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

                ui.add(V_STACK).nest(|| {
                    ui.add(BUTTON.size_y(Size::Frac(0.3)).text("1"));
                    ui.add(BUTTON.size_y(Size::Frac(0.7)).text("2"));
                });
            });
        });
    }
}

fn main() {
    basic_env_logger_init();
    let state = State::default();
    run_example_loop(state);
}
