

use keru::*;
use keru::example_window_loop::*;

#[derive(Default)]
pub struct State {
    pub count: i32,
}

impl ExampleLoop for State {
    fn update_ui(&mut self, ui: &mut Ui) {
        #[node_key] const INCREASE: NodeKey;
        
        if ui.is_clicked(INCREASE) {
            self.count += 1;
        }

        let increase_button = BUTTON
            .color(Color::RED)
            .text("Increase")
            .key(INCREASE);

        ui.v_stack().nest(|| {
            ui.add(increase_button);
            ui.label(&self.count.to_string());
        });

    }
}

fn main() {
    let state = State::default();
    run_example_loop(state);
}






