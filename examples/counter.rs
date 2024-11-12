use blue::example_window_loop::*;
use blue::{Color, NodeKey, Ui, BUTTON};
use blue::node_key;
use winit::error::EventLoopError;

#[derive(Default)]
pub struct State {
    pub count: i32,
    pub show: bool,
}

impl ExampleLoop for State {
    fn declare_ui(&mut self, ui: &mut Ui) {
        fn count_color(count: i32) -> Color {
            let red = (0.1 * (count as f32) * 255.0) as u8;
            return Color::rgba(red, 26, 52, 205);
        }

        let show_button_text = match self.show {
            true => "Hide Counter",
            false => "Show Counter",
        };

        // Declare unique identities for out Ui elements with #[node_key]
        // Using a NodeKey to assign a stable identity to each element is almost always a good idea, but it's not always necessary.
        #[node_key] const INCREASE: NodeKey;
        #[node_key] const DECREASE: NodeKey;
        #[node_key] const SHOW: NodeKey;

        ui.add(INCREASE)
            .params(BUTTON)
            .color(count_color(self.count))
            .static_text("Increase");

        ui.add(SHOW)
            .params(BUTTON)
            .color(Color::RED)
            .static_text(show_button_text);

        ui.add(DECREASE)
            .params(BUTTON)
            .static_text("Decrease");


        ui.v_stack().nest(|| {
            if self.show {
                ui.place(INCREASE);
                ui.label(self.count);
                ui.place(DECREASE);

            }
            ui.place(SHOW);
        });

        // Change our state according to the Ui events.
        // Since we use the unique stable identities provided by the node_key constants, we could run this same code from wherever we want.
        // This is also true of the definitions of the #[node_key] consts themselves.
        if ui.is_clicked(SHOW) {
            self.show = !self.show;
        }
        if ui.is_clicked(INCREASE) {
            self.count += 1;
        }
        if ui.is_clicked(DECREASE) {
            self.count -= 1;
        }
    }
}

fn main() -> Result<(), EventLoopError> {
    run_with_example_loop::<State>()
}
