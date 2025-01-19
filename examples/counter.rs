use keru::*;
use keru::example_window_loop::*;

#[derive(Default)]
pub struct State {
    pub count: i32,
    pub show: bool,
}

impl ExampleLoop for State {
    fn update_ui(&mut self, ui: &mut Ui) {
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

        // Big number = more red
        fn count_color(count: i32) -> Color {
            let red = 0.1 * count as f32;
            return Color::rgba_f(red, 0.10196, 0.59608, 0.80392);
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
                // let variable_text = format!("{:?}", std::time::Instant::now());
                // ui.label(variable_text);
                ui.label(self.count);
                ui.place(DECREASE);
            }
            ui.place(SHOW);
        });
    }

}

fn main() {
    basic_env_logger_init();
    let state = State::default();
    run_example_loop(state);
}
