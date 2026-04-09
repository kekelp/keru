#![allow(unused)]
use keru::*;

struct State {
    swap: bool,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const TOGGLE: NodeKey;
    #[node_key] const RED: NodeKey;
    #[node_key] const GREEN: NodeKey;
    #[node_key] const BLUE: NodeKey;

    let container = CONTAINER
        .size_x(Size::Pixels(300.0))
        .size_y(Size::Pixels(300.0))
        .visible()
        .color(Color::rgba_u8(30, 30, 30, 255));

    ui.add(container).nest(|| {
        let z_red   = if state.swap { 0.0 } else { 2.0 };
        let z_green = 1.0;
        let z_blue  = if state.swap { 2.0 } else { 0.0 };

        // Red square — declared first, so without z_index it would be behind.
        ui.add(DEFAULT
            .key(RED)
            .color(Color::rgba_u8(200, 60, 60, 255))
            .size_symm(Size::Pixels(150.0))
            .position_symm(Pos::Pixels(20.0))
            .anchor_symm(Anchor::Start)
            .z_index(z_red)
        );

        // Green square — offset slightly.
        ui.add(DEFAULT
            .key(GREEN)
            .color(Color::rgba_u8(60, 180, 60, 255))
            .size_symm(Size::Pixels(150.0))
            .position_symm(Pos::Pixels(70.0))
            .anchor_symm(Anchor::Start)
            .z_index(z_green)
        );

        // Blue square — declared last, so without z_index it would be on top.
        ui.add(DEFAULT
            .key(BLUE)
            .color(Color::rgba_u8(60, 80, 210, 255))
            .size_symm(Size::Pixels(150.0))
            .position_symm(Pos::Pixels(120.0))
            .anchor_symm(Anchor::Start)
            .z_index(z_blue)
        );
    });

    // Toggle button: swaps red and blue z_index values.
    let label = if state.swap {
        "Red: 0, Blue: 2 (blue on top)"
    } else {
        "Red: 2, Blue: 0 (red on top)"
    };
    ui.add(BUTTON.key(TOGGLE).text(label).position_y(Pos::End).anchor_y(Anchor::End));
    if ui.is_clicked(TOGGLE) {
        state.swap = !state.swap;
    }
}

fn main() {
    let state = State { swap: false };
    example_window_loop::run_example_loop(state, update_ui);
}
