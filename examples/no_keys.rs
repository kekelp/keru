use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {
    pub count: i32,
    pub show: bool,
}

impl ExampleLoop for State {
    // This example is equivalent to the "counter" example, but it doesn't use NodeKeys.
    // Since we can't refer to nodes using keys, we have to do all operations for a node (creating it, setting parameters, placing it in the layout, and running effects) all in a single method chain.
    // This might actually be more familiar, since that's how it works in many other declarative GUI libraries.
    // But in my opinion, it makes things harder to read: the layout is defined by the nesting of the function calls, but since we're doing so much other stuff in the same place, the layout structure becomes hard to understand at a glance.

    fn update_ui(&mut self, ui: &mut Ui) {
        fn count_color(count: i32) -> Color {
            let red = 0.1 * count as f32;
            return Color::rgba_f(red, 0.10196, 0.59608, 0.80392);
        }
        let show_button_text = match self.show {
            true => "Hide Counter",
            false => "Show Counter",
        };

        let increase_button_params = BUTTON
            .color(count_color(self.count))
            .text("Increase");

        let show_button_params = BUTTON
            .color(Color::RED)
            .text(show_button_text);

        let decrease_button_params = BUTTON
            .text("Decrease");

        ui.v_stack().nest(|| {
            if self.show {
                if ui
                    .add_anon().params(BUTTON)
                    .params(increase_button_params)
                    .is_clicked()
                {
                    self.count += 1;
                };

                ui.label(self.count);

                if ui
                    .add_anon().params(BUTTON)
                    .params(decrease_button_params)
                    .is_clicked()
                {
                    self.count -= 1;
                }
            }

            if ui
                .add_anon().params(BUTTON)
                .params(show_button_params)
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
