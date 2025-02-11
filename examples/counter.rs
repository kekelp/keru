use keru::*;
use keru::example_window_loop::*;

#[derive(Default)]
pub struct State {
    pub count: i32,
    pub show: bool,
}

impl ExampleLoop for State {
    fn update_ui(&mut self, ui: &mut Ui) {
        #[node_key] const INCREASE: NodeKey;
        #[node_key] const DECREASE: NodeKey;
        #[node_key] const SHOW: NodeKey;
        
        if ui.is_clicked(SHOW) {
            self.show = !self.show;
        }
        if ui.is_clicked(INCREASE) {
            self.count += 1;
        }
        if ui.is_clicked(DECREASE) {
            self.count -= 1;
        }

        fn count_color(count: i32) -> Color {
            let red = 0.1 * count as f32;
            return Color::rgba_f(red, 0.10196, 0.59608, 0.80392);
        }
        let show_button_text = match self.show {
            true => "Hide Counter",
            false => "Show Counter",
        };

        let increase_button = BUTTON
            .color(count_color(self.count))
            .text("Increase")
            .key(INCREASE);

        let show_button = BUTTON
            .color(Color::RED)
            .text(show_button_text)
            .key(SHOW);

        let decrease_button = BUTTON
            .text("Decrease")
            .key(DECREASE);

        ui.v_stack().nest(|| {
            if self.show {
                ui.add(increase_button);
                ui.label(self.count);
                ui.add(decrease_button);
            }
            ui.add(show_button);
        });
    }

}

fn main() {
    basic_env_logger_init();
    let state = State::default();
    run_example_loop(state);
}
