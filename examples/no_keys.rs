use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {
    pub count: i32,
    pub show: bool,
}

impl ExampleLoop for State {
    // This example is equivalent to the "counter" example, but it doesn't use NodeKeys.
    // In my opinion, mashing together style, layout and effects like this makes things very hard to read.

    fn update_ui(&mut self, ui: &mut Ui) {
        fn count_color(count: i32) -> Color {
            let red = (0.1 * (count as f32) * 255.0) as u8;
            return Color::rgba(red, 26, 52, 205);
        }
        let show_button_text = match self.show {
            true => "Hide Counter",
            false => "Show Counter",
        };

        ui.v_stack().nest(|| {
            if self.show {
                if ui
                    .add_anon(BUTTON)
                    .color(count_color(self.count))
                    .static_text("Increase")
                    .place()
                    .response(ui)
                    .is_clicked()
                {
                    self.count += 1;
                };

                ui.label(self.count);

                if ui
                    .add_anon(BUTTON)
                    .static_text("Decrease")
                    .place()
                    .response(ui)
                    .is_clicked()
                {
                    self.count -= 1;
                }
            }

            if ui
                .add_anon(BUTTON)
                .color(Color::RED)
                .static_text(show_button_text)
                .place()
                .response(ui)
                .is_clicked()
            {
                self.show = !self.show;
            }
        });
    }
}

fn main() {
    basic_env_logger_init();
    let state = State::default();
    run_example_loop(state);
}
