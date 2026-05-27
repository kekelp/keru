#![allow(unused)]
use keru::*;
use keru::node_library::*;
use keru::example_window_loop::*;

struct State {}

fn update_ui(state: &mut State, ui: &mut Ui) {
    ui.add(V_STACK).nest(|| {
        // Vertical scroll
        ui.add(V_SCROLL_STACK.size_y(Size::Frac(0.5))).nest(|| {
            for i in 0..50 {
                let label = format!("Row {i}");
                let node = PANEL
                    .color(Color::BLUE)
                    .text(&label)
                    .size_y(Size::Pixels(50.0))
                    .size_x(Size::Pixels(200.0));

                ui.add(node);
            }
        });

        // Horizontal scroll
        ui.add(H_SCROLL_STACK).nest(|| {
            for i in 0..50 {
                let label = format!("Col {i}");
                let node = PANEL
                    .color(Color::KERU_GREEN)
                    .text(&label)
                    .size_y(Size::Pixels(80.0))
                    .size_x(Size::Pixels(120.0));

                ui.add(node);
            }
        });
    });
}

fn main() {
    let state = State {};
    example_window_loop::run_example_loop(state, update_ui);
}


