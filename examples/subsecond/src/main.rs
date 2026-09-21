// `subsecond` hot reload example.
// - Install `dioxus-cli` with `cargo binstall` or `cargo-install`
// - Run `dx serve --hotpatch --package keru_subsecond`, then edit the UI code in `update_ui`.
// - Edit the code in this file and save.

use keru::*;
use keru::node_library::*;
use keru::example_window_loop::*;

#[derive(Default)]
pub struct State {
    pub count: i32,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    // Tell dioxus to wake up the Ui when it applies a patch.
    if ui.current_frame() == 1 {
        let waker = ui.ui_waker();
        dioxus_devtools::subsecond::register_handler(std::sync::Arc::new(move || waker.set_update_needed()));
    }

    dioxus_devtools::subsecond::call(|| {
        #[node_key] const INCREASE: NodeKey;

        let increase_button = BUTTON
            .color(Color::RED)
            .text("Increase")
            .key(INCREASE);

        ui.add(V_STACK).nest(|| {
            ui.add(increase_button);
            ui.add(LABEL.text(&state.count.to_string()));
        });

        if ui.is_clicked(INCREASE) {
            state.count += 1;
        }
    });
}

fn main() {
    dioxus_devtools::connect_subsecond();

    dioxus_devtools::subsecond::call(|| {
        let state = State::default();
        run_example_loop(state, update_ui);
    });
}
