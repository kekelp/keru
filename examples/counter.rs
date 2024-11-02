use blue::{
    example_window_loop::{run_with_example_loop, ExampleLoop},
    ui_node_params::{BUTTON, LABEL},
    Color, NodeKey, Ui,
};
use change_watcher::Watcher;
use view_derive::node_key;
use winit::error::EventLoopError;

#[derive(Default)]
pub struct State {
    pub count: Watcher<i32>,
    pub show: bool,
}

impl ExampleLoop for State {
    fn declare_ui(&mut self, ui: &mut Ui) {
        #[node_key] const INCREASE: NodeKey;
        let increase = BUTTON.key(INCREASE);

        #[node_key] const DECREASE: NodeKey;
        let decrease = BUTTON.key(DECREASE);

        #[node_key] const SHOW: NodeKey;
        let show = BUTTON.color(Color::RED).key(SHOW);

        fn count_color(count: i32) -> Color {
            let red = (0.1 * (count as f32) * 255.0) as u8;
            return Color::rgba(red, 26, 52, 205);
        }
        let color = count_color(*self.count);

        let show_button_text = match self.show {
            true => "Hide Counter",
            false => "Show Counter",
        };

        ui.v_stack().nest(|| {
            if self.show {
                ui.add(&increase).static_text("Increase").set_color(color);
                ui.add(&LABEL).dyn_text(self.count.if_changed());
                ui.add(&decrease).static_text("Decrease");
            }
            ui.add(&show).static_text(&show_button_text);
        });

        if ui.is_clicked(SHOW) {
            self.show = !self.show;
        }
        if ui.is_clicked(INCREASE) {
            *self.count += 1;
        }
        if ui.is_clicked(DECREASE) {
            *self.count -= 1;
        }
    }
}

fn main() -> Result<(), EventLoopError> {
    run_with_example_loop::<State>()
}
