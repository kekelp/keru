#![allow(dead_code)]
#![allow(unused_variables)]

use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {
    expanded: bool,
    expanded2: bool,
    expanded3: bool,
    expanded4: bool,
    expanded5: bool,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const EXPAND: NodeKey;

    ui.add(CONTAINER.size_symm(Size::Fill).padding(50)).nest(|| {
        ui.add(LABEL.static_text("A").position_y(Position::End));
        // ui.add(LABEL.static_text("B"));
        // ui.add(LABEL.static_text("C"));
    });

}


fn main() {
    basic_env_logger_init();
    let state = State::default();
    run_example_loop(state, update_ui);
}
