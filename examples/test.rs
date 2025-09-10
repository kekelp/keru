use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {

}

impl State {
    fn update_ui(&mut self, ui: &mut Ui) {

        let red = PANEL
            .color(Color::rgba_f(0.5, 0.0, 0.0, 1.0))
            .size(Size::Pixels(300), Size::Pixels(300));
        
        let blue = PANEL
            .color(Color::rgba_f(0.01, 0.015, 0.5, 1.0))
            .size(Size::Pixels(200), Size::Pixels(400));

        ui.add(TEXT.static_text("Test"));
        ui.add(blue);
        ui.add(red);

    }
}

fn main() {
    let state = State::default();
    run_example_loop(state, State::update_ui);
}
