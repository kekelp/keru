use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {
    pub current_tab: usize,
    pub show: bool,
}

impl ExampleLoop for State {
    fn update_ui(&mut self, ui: &mut Ui) {
        #[node_key] const MOVING_NODE: NodeKey;
        #[node_key] const V_STACK_KEY: NodeKey;
        #[node_key] const SHOW: NodeKey;
        #[node_key] const CONT_1: NodeKey;
        #[node_key] const CONT_2: NodeKey;

        let moving_node = BUTTON
            .color(Color::RED)
            .shape(Shape::Circle)
            .key(MOVING_NODE);
        let cont_1 = BUTTON
            .text("My child will type sneed1\n.\n.\n.")
            .key(CONT_1);
        let cont_2 = BUTTON
            .text("My child will type sneed2\n.\n.\n.")
            .key(CONT_2);

        ui.add(V_STACK.key(V_STACK_KEY)).nest(|| {
            ui.add(cont_1).nest(|| {
                if self.show {
                    ui.add(moving_node);
                }
            });
            ui.add(cont_2).nest(|| {
                if !self.show {
                    ui.add(moving_node);
                }
            });
            if ui.add(BUTTON.text("Show").key(SHOW)).is_clicked(ui) {
                self.show = !self.show;
            }
        });
    }
}

fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Warn)
        .filter_module("keru::tree", log::LevelFilter::Trace)
        .init();
    let mut state = State::default();
    state.show = true;
    run_example_loop(state);
}
