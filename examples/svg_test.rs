use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {}

fn update_ui(_state: &mut State, ui: &mut Ui) {
    ui.v_stack().nest(|| {
        ui.static_paragraph("SVG Display Test");

        let svg_data = include_bytes!("../assets/tiger.svg");
        let svg_node = IMAGE
            .static_svg(svg_data)
            .size(Size::Fill, Size::Fill);
        ui.add(svg_node);
    });
}

fn main() {
    let state = State::default();
    run_example_loop(state, update_ui);
}
