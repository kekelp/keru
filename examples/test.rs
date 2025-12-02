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

    let text = TEXT_EDIT_LINE
        // .text("aaa")
        .placeholder_text("Feed");

    ui.add(text);
    // ui.debug_print_tree();
}


fn main() {
    // basic_env_logger_init();
    let state = State::default();
    run_example_loop(state, update_ui);
}
