use keru::*;
use keru::node_library::*;

// This example shows how to use the NodeKey::sibling() method to create dynamic keys at runtime.

// In this example, our State holds a Vec of items, and we'll add some buttons to add and remove them.
pub struct State {
    pub items: Vec<String>,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const CREATE_BUTTON: NodeKey;
    #[node_key] const REMOVE_BUTTON: NodeKey;

    let item_label = LABEL.size_x(Size::Pixels(150.0));

    let create_button = BUTTON
        .text("Create item")
        .animate_position(true)
        .color(Color::GREEN)
        .key(CREATE_BUTTON);

    let remove_button = BUTTON
        .size_x(Size::Pixels(150.0))
        .text("Remove")
        .color(Color::RED);

    ui.add(V_STACK.animate_position(true)).nest(|| {
        // Use a key for the button that creates a new element.
        ui.add(create_button);

        for (i, item) in state.items.iter().enumerate() {
            ui.add(H_STACK).nest(|| {
                // We can easily add Nodes based on dynamic data.
                ui.add(item_label.text(item.as_str()));
            
                // We want a remove button for each item. But we can't create compile-time keys for all of them in advance.
                // With the `sibling` method, we can start on a base NodeKey and create new ones dynamically from a hashable value:
                let key = REMOVE_BUTTON.sibling(i);
                let remove_button = remove_button.key(key);
                ui.add(remove_button);
            });
        }
    });

    // Outside the loop, we can call `sibling` with the same arguments, and we'll deterministically end up with the same key.
    // We can use it to point to the remove buttons and check for clicks on them. 
    for i in 0..state.items.len() {
        if ui.is_clicked(REMOVE_BUTTON.sibling(i)) {
            state.items.remove(i);
        }
    }
    // Using keys in this way is usually more readable, but this time we also happened to dodge a borrow issue.
    // if we tried to do the removal immediately after adding the node, 
    // the compiler wouldn't have let us modify the Vec while we were iterating on it.

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
