#![allow(unused)]
use keru::*;
use keru::example_window_loop::*;

struct State {
    pos_x: f32,
    pos_y: f32,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    let stack = H_STACK
        .size_y(Size::Pixels(80.0))
        .size_x(Size::Fill)
        .color(Color::GREY)
        .visible()
        .padding(8.0);

    let free = PANEL
        .shape(Shape::Circle)
        .color(Color::RED)
        .anchor_symm(Anchor::Center)
        .size(Size::Pixels(5.0), Size::Pixels(5.0))
        .position(Pos::Frac(state.pos_x), Pos::Frac(state.pos_y))
        .free_placement(true);

    ui.add(V_STACK.size_x(Size::Pixels(500.0))).nest(|| {
        ui.add(stack).nest(|| {
            ui.add(BUTTON.text("One"));
            ui.add(BUTTON.text("Two"));
            ui.add(BUTTON.text("Three"));
            ui.add(free);
        });
        ui.slider(&mut state.pos_x, 0.0, 1.0);
        ui.slider(&mut state.pos_y, 0.0, 1.0);
    });
}

fn main() {
    let state = State { pos_x: 0.0, pos_y: 1.0 };
    example_window_loop::run_example_loop(state, update_ui);
}
