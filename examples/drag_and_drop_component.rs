//! This is an example of an advanced component.
//! 
//! We create the `ReorderStack` struct and assign a key to it, we add it with `ui.add_compontent()`, and we add children to it as it it was a regular stack.
//! 
//! Then, we call `ui.run_component()` to run the component's additional logic. It will use manipulate and interact the children that we already added to implement the drag-and-drop logic, in a completely transparent way.
//! 
//! - it calculates where the user is hovering it by going through the elements and measuring their height.
//! - it inserts an invisible spacer in that position with the same height as the dragged element.
//! - it removes the dragged elements from its place in the tree and it re-adds it as a free-floating node at the cursor position.
//! - If the children that we add use the standard `.animate_position(true)`, they will animate smoothly as the component moves them around.
//! - when the dragged element is released, it returns a tuple of indices that indicate that it should be moved to the new position in the user's state.
//! 
//! Note that the advanced APIs for manipulating the tree are experimental.

use keru::*;
use keru::node_library::*;

struct State {
    items: Vec<&'static str>,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const ITEM: NodeKey;
    #[component_key] const STACK: ComponentKey<ReorderStack>;

    let item_base = BUTTON
        .animate_position(true)
        .absorbs_clicks(false)
        .sense_drag(true)
        .size_x(Size::Pixels(100.0))
        .anchor_symm(Anchor::Center);

    let component = ReorderStack { key: STACK };
    
    ui.add_component(component).nest(|| {
        for &item in &state.items {
            let key = ITEM.sibling(&item);
            let node = item_base.text(&item).key(key);
            ui.add(node);
        }
    });

    if let Some((move_from, move_to)) = ui.run_component(STACK) {
        let item = state.items.remove(move_from);
        let adjusted = if move_to > move_from { move_to - 1 } else { move_to };
        state.items.insert(adjusted.min(state.items.len()), item);
    }
}

fn main() {
    let items = vec!["A", "B\nB", "C\nC\nC", "D\nD\nD", "E\nE", "F"];
    let state = State { items };
    example_window_loop::run_example_loop(state, update_ui);
}
