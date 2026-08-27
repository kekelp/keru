

use keru::*;
use keru::node_library::*;
use keru::deterministic_keys::node_key;

pub struct State {
    pub count: i32,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    // Define a unique identity for the button
    #[node_key] const INCREASE: NodeKey;

    dbg!(INCREASE);
    
    // Create a Node struct describing a button
    let increase_button: Node = BUTTON
        .color(Color::RED)
        .text("Increase")
        .key(INCREASE);

    // Add the nodes to the tree
    ui.add(V_STACK).nest(|| {
        ui.add(increase_button);
        ui.add(LABEL.text(&state.count.to_string()));
    });

    // Change the state in response to events
    if ui.is_clicked(INCREASE) {
        state.count += 1;
    }
}

fn main() {
    let state = State { count: 0 };
    example_window_loop::run_example_loop(state, update_ui);
}




