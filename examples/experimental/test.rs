#![allow(unused)]
use keru::*;
use keru::node_library::*;
use keru::example_window_loop::*;

#[derive(Default)]
struct State {}

fn update_ui(state: &mut State, ui: &mut Ui) {
    // V_STACK with mixed sizes: Pixels, Frac, Fill.
    // Expected (container is 600px tall, with two 5px spacers):
    //   - fixed:  100px  (Pixels)
    //   - frac:   0.5 * (600 - 100 - 5 - 5) = 245px  (Frac(0.5) of remaining)
    //   - fill:   600 - 100 - 245 - 5 - 5 = 245px  (Fill gets the rest)
    //   => frac == fill here, both 245px

    let container = V_STACK
        .size(Size::Pixels(400.0), Size::Pixels(600.0))
        .padding(0.0)
        .stack_spacing(5.0)
        .stack_arrange(Arrange::Start);

    ui.add(container).nest(|| {
        ui.add(PANEL.size_x(Size::Fill).size_y(Size::Pixels(100.0)).color(Color::RED));
        ui.add(PANEL.size_x(Size::Fill).size_y(Size::Frac(0.5)).color(Color::KERU_BLUE));
        ui.add(PANEL.size_x(Size::Fill).size_y(Size::Frac(0.3)).color(Color::GREEN));
        ui.add(PANEL.size_x(Size::Fill).size_y(Size::Fill).color(Color::RED));

    });
}

fn main() {
    run_example_loop(State::default(), update_ui);
}
