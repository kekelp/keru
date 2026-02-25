use std::time::Instant;

use keru::*;

struct State {
    items: Vec<&'static str>,
}

fn update_ui(state: &mut State, ui: &mut Ui) {

    #[node_key] const BUTTON_KEY: NodeKey;

    let button = BUTTON
        .text("Click")
        .sense_drag(true)
        .key(BUTTON_KEY);
    
    ui.add(button);

    if ui.is_clicked(BUTTON_KEY) {
        dbg!(Instant::now());
    }
    
}

fn main() {
    let items = vec!["A", "special", "B", "C", "xxxxxx\nxxxxxx\nxxxxxx", "D", "E"];

    let state = State {
        items,
    };

    example_window_loop::run_example_loop(state, update_ui);
}
