use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {}

fn update_ui(_state: &mut State, ui: &mut Ui) {
    ui.v_stack().nest(|| {
        ui.static_paragraph("Image Display Test - Using keru_draw");

        ui.h_stack().nest(|| {
            // Display debug image
            let img1 = IMAGE.static_image(include_bytes!("../src/textures/debug.png"));
            ui.add(img1);

            // Display E image
            let img2 = IMAGE.static_image(include_bytes!("../src/textures/E.png"));
            ui.add(img2);
        });

        // Display clouds image
        let img3 = IMAGE.static_image(include_bytes!("../src/textures/clouds.png"));
        ui.add(img3);
    });
}

fn main() {
    let state = State::default();
    run_example_loop(state, update_ui);
}
