use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {}

fn update_ui(_state: &mut State, ui: &mut Ui) {
    ui.v_stack().nest(|| {
        ui.static_paragraph("Image Path Test - Testing filesystem path loading");

        ui.h_stack().nest(|| {
            // Display debug image from static bytes
            let img1 = IMAGE.static_image(include_bytes!("../src/textures/debug.png"));
            ui.add(img1);

            // Display same debug image from filesystem path
            let img2 = IMAGE.image_path("src/textures/debug.png");
            ui.add(img2);
        });

        // Display clouds image from path
        let img3 = IMAGE.image_path("src/textures/clouds.png");
        ui.add(img3);

        ui.static_paragraph("If you see 3 images above, filesystem path loading works!");
    });
}

fn main() {
    let state = State::default();
    run_example_loop(state, update_ui);
}
