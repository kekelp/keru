use keru::*;
use keru::node_library::*;

// This example shows how to create keys dynamically at runtime with the NodeKey::sibling() method.
// The State holds a Vec of items, and we'll add some buttons to add and remove them.
pub struct State {
    pub items: Vec<String>,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const CREATE_BUTTON: NodeKey;
    #[node_key] const REMOVE_BUTTON: NodeKey;

    let item_row = H_STACK.grow_from_top().clip_children_y(true);
    let item_label = LABEL.size_x(Size::Pixels(150.0));

    // There's only one Create button, so it works the same as in the previous example.
    let create_button = BUTTON
        .text("Create item")
        .color(Color::GREEN)
        .key(CREATE_BUTTON);

    let remove_button = BUTTON
        .text("Remove")
        .size_x(Size::Pixels(150.0))
        .color(Color::RED);

    ui.add(V_STACK.animate_position(true)).nest(|| {
        ui.add(create_button);
        
        for (n, item) in state.items.iter().enumerate() {
            ui.add(item_row).nest(|| {
                // We can add Nodes based on dynamic data.
               ui.add(item_label.text(&item));
                
                // We want a remove button for each item.
                // But we can't create compile-time keys for all of them in advance.
                // With the `sibling` method, we can start from a base NodeKey,
                // and create new "sibling keys" dynamically with a hashable value.
                let nth_key = REMOVE_BUTTON.sibling(n);
                // Create a Node by taking the base `remove_button` and assigning it the new key
                let nth_remove_button = remove_button.key(nth_key);
                ui.add(nth_remove_button);
            });
        }
    });

    // Outside the loop, we can call `sibling` with the same arguments,
    // and we'll deterministically end up with the same keys.
    // We can use them to point to the remove buttons and check for clicks on them. 
    for n in 0..state.items.len() {
        if ui.is_clicked(REMOVE_BUTTON.sibling(n)) {
            state.items.remove(n);
        }
    }
    // Using keys in this way allows to move the effects away from the layout,
    // which is usually more readable. But this time we also dodged a borrow issue.
    // If we tried to do the removal immediately after adding the node, 
    // the compiler wouldn't have let us modify the Vec while we were iterating on it,
    // and we'd have to move the effects out either way.

    if ui.is_clicked(CREATE_BUTTON) {
        const ITEMS: &[&str] = &["Apple", "Banana", "Cherry", "Dragonfruit"];
        let new_item = ITEMS[state.items.len() % ITEMS.len()].to_string();
        state.items.push(new_item);
    }
}

fn main() {
    let state = State {
        items: vec!["Apple".to_string(), "Banana".to_string()],
    };
    example_window_loop::run_example_loop(state, update_ui);
}

// The last tutorial example is `03_components.rs`, which shows the `Component` trait.
