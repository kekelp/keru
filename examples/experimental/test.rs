use keru::*;
use keru::node_library::*;

pub struct State {
    pub count: i32,
}

/// A row of three 50px squares, in a stack of the given width.
fn row(ui: &mut Ui, arrange: Arrange, width: f32) {
    let stack = PANEL
        .color(Color::KERU_PINK)
        .size(Size::Pixels(width), Size::Pixels(70.0))
        .padding(0.0)
        .stack(Axis::X, arrange, 40.0);

    let square = PANEL.size(Size::Pixels(50.0), Size::Pixels(50.0)).padding(0.0);

    ui.add(stack).nest(|| {
        ui.add(square.color(Color::RED));
        ui.add(square.color(Color::GREEN));
        ui.add(square.color(Color::BLUE));
    });
}

fn update_ui(_state: &mut State, ui: &mut Ui) {
    // In the wide rows, the old code ignores the 40px spacing: the gaps are smaller and the edge margins are bigger.
    // In the narrow rows, the old code gives negative gaps and the squares overlap. The new code keeps 40px gaps and lets the squares overflow to the right.
    ui.add(V_STACK.stack_spacing(20.0)).nest(|| {
        row(ui, Arrange::Start, 400.0);
        row(ui, Arrange::SpaceBetween, 400.0);
        row(ui, Arrange::SpaceAround, 400.0);
        row(ui, Arrange::SpaceEvenly, 400.0);

        row(ui, Arrange::Start, 120.0);
        row(ui, Arrange::SpaceBetween, 120.0);
        row(ui, Arrange::SpaceAround, 120.0);
        row(ui, Arrange::SpaceEvenly, 120.0);
    });
}

fn main() {
    let state = State { count: 0 };
    example_window_loop::run_example_loop(state, update_ui);
}
