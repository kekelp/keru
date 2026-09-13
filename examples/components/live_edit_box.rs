//! Example using a [`LiveEditBox`] [`Component`].

use keru::*;
use keru::node_library::*;
use keru::example_window_loop::*;

#[derive(Default)]
struct State {
    value: f32,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    with_arena(|arena| {
        
        ui.add(V_STACK.size_x(Size::Pixels(500.0))).nest(|| {
            let edit_box = LiveEditBox::new(
                |v: &f32| bumpalo::format!(in arena, "{:.2}", v),
                |text: &str| text.trim().parse().ok(),
            );
            ui.add_component_with_state(edit_box, &mut state.value);
            
            ui.add_component_with_state(Slider::new(0.0, 100.0, true), &mut state.value);
        });

    });
}

fn main() {
    basic_env_logger_init();
    run_example_loop(State::default(), update_ui);
}
