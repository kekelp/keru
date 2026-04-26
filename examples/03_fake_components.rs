use keru::*;
use keru::node_library::*;

// This example shows how GUI code can be organized and separated into helper functions.

// - it's always fine to move GUI code into helper functions for organization.
// - helper functions that are called multiple times need to wrap ui.subtree() to make their internal keys unique, if they have any.
// - there is also a more robust way to reuse GUI code: the Component trait. See the next example (04_real_components.rs). 

pub struct State {
    pub count: i32,
}

fn update_ui(state: &mut State, ui: &mut Ui) {

    ui.add(PANEL.size_x(Size::Frac(0.75))).nest(|| {
        ui.add(V_STACK).nest(|| {

            // It's usually fine to group a block of GUI code together in a helper function.
            // This is very useful for organizing code, and it's always fine if the function is called just once.
            fn counter_display(state: &State, ui: &mut Ui) {
                let count = format!("Count: {}", state.count);
                ui.add(V_STACK).nest(|| {                    
                    ui.add(LABEL.text("Count:"));
                    ui.add(LABEL.text(&count));
                });
            }

            // As long as we don't use `NodeKey`s inside the function, it's also fine to call it as many times as we want:
            ui.add(TEXT.text("Read-only counters:"));
            ui.add(H_STACK).nest(|| {
                counter_display(state, ui);
                counter_display(state, ui);
                counter_display(state, ui);
            });

            ui.add(H_LINE);

            // But using `NodeKey`s inside a helper that's called multiple times clearly can't work: 
            // the NodeKey is supposed to be an unique identity for a node, but now we're trying to use on many nodes.
            // If we try to use it, we'll see that the first Increase button increases the count by 3, and the other two are broken. 
            fn counter_bad(state: &mut State, ui: &mut Ui) {
                #[node_key] const INCREASE: NodeKey;

                let increase_button = BUTTON
                    .color(Color::RED)
                    .text("Increase")
                    .key(INCREASE);

                let count = format!("Count: {}", state.count);
                ui.add(V_STACK).nest(|| {                    
                    ui.add(increase_button);
                    ui.add(LABEL.text(&count));
                });

                // We're calling this 3 times with the same key, which points to the first `increase_button` added.
                if ui.is_clicked(INCREASE) {
                    state.count += 1;
                }
            }

            ui.add(TEXT.text("Broken counters:"));
            ui.add(H_STACK).nest(|| {
                counter_bad(state, ui);
                counter_bad(state, ui);
                counter_bad(state, ui);
            });

            ui.add(H_LINE);

            // To fix this, the quickest way is to use the subtree() function, which creates a private key-space.
            // As long as we wrap our helpers in a subtree, we can do whatever we want.
            // (subtree is a bad name for this and it will be changed.)
            fn counter_good(state: &mut State, ui: &mut Ui) {
                ui.subtree().start(|| {
                    counter_bad(state, ui);
                });
            }

            ui.add(TEXT.text("Fixed counters:"));
            ui.add(H_STACK).nest(|| {
                counter_good(state, ui);
                counter_good(state, ui);
                counter_good(state, ui);
            });

            // There is also a more advanced way: the Component trait.
            // Components not only 
        });
    });
}

fn main() {
    let state = State { count: 0 };
    example_window_loop::run_example_loop(state, update_ui);
}
