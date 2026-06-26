#![allow(unused)]
use keru::*;
use keru::node_library::*;
use keru::example_window_loop::*;

#[derive(Default)]
struct State {
    on: bool,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const TOGGLE: NodeKey;
    #[node_key] const BOX: NodeKey;

    // Slow things down so the cosmetic property animations are easy to see.
    ui.set_global_animation_speed(0.15);

    let toggle = BUTTON
        .key(TOGGLE)
        .text("Toggle");

    let (color, alpha) = if state.on {
        (Color::rgba_u8(230, 80, 80, 255), 1.0)
    } else {
        (Color::rgba_u8(60, 110, 220, 255), 0.25)
    };

    let panel = PANEL
        .key(BOX)
        .size_x(Size::Pixels(200.0))
        .size_y(Size::Pixels(200.0))
        .color(color)
        .alpha(alpha)
        .animate_properties(true);

    let vstack = V_STACK
        .size_x(Size::Pixels(220.0))
        .size_y(Size::Pixels(500.0))
        .stack_arrange(Arrange::Start);

    ui.add(vstack).nest(|| {
        ui.add(toggle);
        ui.add(panel);
    });

    if ui.is_clicked(TOGGLE) {
        state.on = !state.on;
    }
}

fn main() {
    run_example_loop(State::default(), update_ui);
}
