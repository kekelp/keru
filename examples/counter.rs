use blue::{example_window_loop::*, V_STACK};
use blue::{Color, NodeKey, Ui, BUTTON, LABEL};
use blue::node_key;
use winit::error::EventLoopError;

#[derive(Default)]
pub struct State {
    pub count: i32,
    pub show: bool,
}

impl ExampleLoop for State {
    fn declare_ui(&mut self, ui: &mut Ui) {
        // Some light calculations to turn the real state (self.count, self.show) into things that the Ui understands (String, Color)
        fn count_color(count: i32) -> Color {
            let red = (0.1 * (count as f32) * 255.0) as u8;
            return Color::rgba(red, 26, 52, 205);
        }
        let color = count_color(self.count);

        let show_button_text = match self.show {
            true => "Hide Counter",
            false => "Show Counter",
        };

        // Declare our Ui elements and set their params: color, size, position, ...
        // We're mostly just using the default params in BUTTON here.
        // Using a NodeKey to assign a stable identity to each element is almost always a good idea, but it's not always necessary.
        #[node_key] const INCREASE: NodeKey;
        // #[node_key] const DECREASE: NodeKey;
        #[node_key] const LABELKEY: NodeKey;
        #[node_key] const SHOW: NodeKey;

        #[node_key] const VSTACKKEY: NodeKey;

        // ui.add(DECREASE).params(BUTTON).static_text("Decrease");
        ui.add(INCREASE)
            .params(BUTTON)
            .color(color)
            .on_click(|| { self.count += 1 })
            .static_text("Increase");
        ui.add(LABELKEY)
            .params(LABEL)
            .color(Color::RED)
            .text(self.count);
        ui.add(SHOW)
            .params(BUTTON)
            .color(Color::RED)
            .static_text(show_button_text);

        ui.add(VSTACKKEY).params(V_STACK);


        // ui.place(SHOW);

        // Declare the layout.

        ui.v_stack().nest(|| {
            if self.show {
                ui.place(INCREASE);
                ui.place(LABELKEY); 

                // this one is all in one line cuz we're quirky like that
                ui.add_anon(BUTTON).static_text("Decrease").on_click(|| { self.count -= 1 }).place();

            }
            ui.place(SHOW);
        });

        // Change our state according to the Ui events.
        // Since we use the unique stable identities provided by the node_key constants, we could run this same code from wherever we want.
        // This is also true of the definitions of the #[node_key] consts themselves.
        if ui.is_clicked(SHOW) {
            self.show = !self.show;
        }
        // if ui.is_clicked(INCREASE) {
        //     self.count += 1;
        // }
        // if ui.is_clicked(DECREASE) {
        //     self.count -= 1;
        // }
    }
}

fn main() -> Result<(), EventLoopError> {
    run_with_example_loop::<State>()
}
