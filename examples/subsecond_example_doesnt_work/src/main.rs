/// This is an experiment with `subsecond` hot reload.
/// As of right now, it doesn't work at all.
/// 
/// 17:37:50 [linux] thread '<unnamed>' (41095) panicked at /home/kekelp/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/dioxus-devtools-0.7.3/src/lib.rs:81:64:
/// 17:37:50 [linux] called `Result::unwrap()` on an `Err` value: Dlopen("/home/kekelp/.cargo/target/dx/keru_subsecond/debug/linux/app/libkeru_subsecond-patch-1770482269349.so: undefined symbol: _ZN14wayland_client8protocol12__interfaces21WL_REGISTRY_INTERFACE17h4d09845aa2df5533E")
/// 
/// Similar error when trying to run the unofficial example `egui-subsecond-example`


use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {
    pub count: i32,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    dioxus_devtools::subsecond::call(|| {

        #[node_key] const INCREASE: NodeKey;
        
        let increase_button = BUTTON
            .color(Color::RED)
            .text("Increase2122")
            .key(INCREASE);

        ui.v_stack().nest(|| {
            ui.add(increase_button);
            ui.label(&state.count.to_string());
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


