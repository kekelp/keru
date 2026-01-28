

use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {
    pub count: i32,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    // Define a unique identity for the button
    #[node_key] const INCREASE: NodeKey;
    
    // Create a Node struct describing a button
    let increase_button = BUTTON
        .color(Color::RED)
        .text("Increase")
        .key(INCREASE);

    // Place the nodes into the tree and define the layout
    ui.v_stack().nest(|| {
        ui.add(increase_button);
        ui.label(&state.count.to_string());
    });

    // Change the state in response to events
    if ui.is_clicked(INCREASE) {
        state.count += 1;
    }
    // `is_clicked()` can be also called as a chained method.
    // In that case, using a key wouldn't be necessary.
}

fn main() {
    let state = State::default();
    run_example_loop(state, update_ui);
}


