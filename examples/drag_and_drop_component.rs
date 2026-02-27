/// This is an example of an advanced component.
/// 
/// As the users of the `ReorderStack` component we add children to it as it it was a regular stack.
/// 
/// In `run_component`, the component uses the advanced `Ui` functions like `children()`, `rect()`, `jump_to_nth_child()`, `remove_and_readd()` to inspect and manipulates the tree after the children have been added to it. If the user is dragging an element:
/// 
/// - it calculates where the user is hovering it by going through the elements and measuring their height.
/// - it inserts an invisible spacer in that position with the same height as the dragged element.
/// - it removes the dragged elements from its place in the tree and it re-adds it as a child of the root node at the cursor position.
/// - animations just work with just the standard `.animate_position(true)` on the elements.
/// - when the dragged element is released, it returns a tuple of indices that indicate that it should be moved to the new position in the user's state.
/// 
/// This way, the `ReorderStack` works for a perfectly animated rearrangeable list that can hold any kind of elements, even complicated nested subtrees of different sizes.

use keru::*;

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
    let items = vec!["A", "special\nA", "B\nA\nA", "C\nA\nA\nA\nA", "xxxxxx\nxxxxxx\nxxxxxx", "D", "E"];

    let state = State {
        items,
    };

    example_window_loop::run_example_loop(state, update_ui);
}
