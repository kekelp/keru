

use keru::*;
use keru::node_library::*;

pub struct State {
    pub count: i32,
}

fn update_ui(_state: &mut State, ui: &mut Ui) {

    let stack = PANEL.color(Color::GREEN)
        .size_x(Size::FitContent)
        // .min_size_x(Size::FitContent)
        ;

    let child = PANEL.color(Color::RED).size_y(Size::Pixels(100.0))
        .min_size_x(Size::Fill)
        .size_x(Size::Pixels(200.0))
        ;

    ui.add(stack).nest(|| {
        ui.add(child);
    });

}

fn main() {
    let state = State { count: 0 };
    example_window_loop::run_example_loop(state, update_ui);
}




