//! This example shows how to use the [Ui::key_scope()] function to allow using unique keys inside helper functions that are meant to be called multiple times.
//! 
//! See also the [Component] trait and the ``03_components.rs` example for a more robust way to create reusable components. 
//! Components can also manage their own state.

use keru::*;
use keru::node_library::*;

pub struct State {
    pub count: i32,
}

fn update_ui(state: &mut State, ui: &mut Ui) {

    ui.add(PANEL.size_x(Size::Frac(0.75))).nest(|| {
        ui.add(V_STACK).nest(|| {

            // A NodeKey is supposed to be an unique identity for a node.
            // If we define and use a NodeKey in a helper function that's meant to be reused, 
            // that doesn't make a lot of sense, because the node won't be unique anymore.
            // If you try running this, you'll see that the first of the 3 counters increases the count by 3, while the others do nothing.
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

                // It makes sense: we're calling this 3 times with the same key.
                // The same key always points to the same node, so it will always point to the first `increase_button` that we added.
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

            // To fix this, the quickest way is to use the `ui.key_scope()` function.
            // As long as we wrap our helpers in a key scope, we can do whatever we want.
            // The keys the we use inside the scope won't conflict with keys in other scopes.
            fn counter_good(state: &mut State, ui: &mut Ui) {
                ui.key_scope().start(|| {
                    counter_bad(state, ui);
                });
            }

            ui.add(TEXT.text("Fixed counters:"));
            ui.add(H_STACK).nest(|| {
                counter_good(state, ui);
                counter_good(state, ui);
                counter_good(state, ui);
            });

            ui.add(H_LINE);

            // If the code doesn't use any keys, or if it's not called multiple times, we can organize it however we want.
            fn counter_display(state: &State, ui: &mut Ui) {
                let count = format!("Count: {}", state.count);
                ui.add(V_STACK).nest(|| {                    
                    ui.add(LABEL.text("Count:"));
                    ui.add(LABEL.text(&count));
                });
            }

            ui.add(TEXT.text("Read-only counters:"));
            ui.add(H_STACK).nest(|| {
                counter_display(state, ui);
                counter_display(state, ui);
                counter_display(state, ui);
            });

        });
    });
}

fn main() {
    let state = State { count: 0 };
    example_window_loop::run_example_loop(state, update_ui);
}
