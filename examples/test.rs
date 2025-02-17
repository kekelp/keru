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

        let content_panel = PANEL.size_y(Size::Fill);
        let v_stack = V_STACK.size_y(Fill).stack_arrange(Arrange::End);
        let tabs_h_stack = H_STACK.size_y(Size::FitContent);

        ui.subtree().start(|| {

            
            ui.add(v_stack).nest(|| {
                ui.add(tabs_h_stack).nest(|| {
                    ui.add(BUTTON.text("Sneed"));
                    ui.add(BUTTON.text("Feed"));
                    ui.add(BUTTON.text("Nasheed"));
                });
                
                ui.add(content_panel).nest(|| {
                    ui.text("o algo")
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
