use keru::example_window_loop::*;
use keru::{Color, NodeKey, Ui, BUTTON};
use keru::node_key;

#[derive(Default)]
pub struct State {
    pub count: i32,
    pub show: bool,
}

impl ExampleLoop for State {
    fn declare_ui(&mut self, ui: &mut Ui) {
        // Declare unique identities for out Ui elements with #[node_key]
        // Using a NodeKey to assign a stable identity to each element is almost always a good idea, but it's not always necessary.
        #[node_key] const INCREASE: NodeKey;
        #[node_key] const DECREASE: NodeKey;
        #[node_key] const SHOW: NodeKey;
        
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

        fn count_color(count: i32) -> Color {
            let red = (0.1 * (count as f32) * 255.0) as u8;
            return Color::rgba(red, 26, 52, 205);
        }
        let show_button_text = match self.show {
            true => "Hide Counter",
            false => "Show Counter",
        };

        // Add nodes to the UI and set their parameters
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

        // Place the nodes into the tree. The nesting and order of these calls define the layout.
        ui.v_stack().nest(|| {
            if self.show {
                ui.place(INCREASE);
                ui.label(self.count);
                ui.place(DECREASE);
            }
            ui.place(SHOW);
        });
    }
}

fn main() -> Result<(), WinitEventLoopError> {
    // This one-line render loop is only intended for examples.
    // The library is meant to be used within a custom `winit`/`wgpu` loop.
    // See the `keru_paint` package for an example.
    run_with_example_loop::<State>()
}
