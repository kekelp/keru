#![allow(unused)]
use keru::*;
use keru::node_library::*;
use keru::example_window_loop::*;


struct State {}

fn update_ui(state: &mut State, ui: &mut Ui) {
    ui.add(V_STACK).nest(|| {
        let image = IMAGE
            .static_image(include_bytes!("../assets/glitch.jpg"))
            .shape(Shape::Circle)
            .shape(Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: 10.0 })
            .shape(Shape::HexGrid { lattice_size: 20.0, offset: (0.0, 0.0), line_thickness: 1.0 })
            .color(Color::from_hex(0xee8031).with_alpha(0.9))
            .image_options(ImageOptions {
                nine_slice: None,
                tile_x: TileMode::Tile,
                tile_y: TileMode::Tile,
            })
            .size(Size::Frac(0.9), Size::Frac(0.9));

        ui.add(image);

    });
}

fn main() {
    let state = State {};
    example_window_loop::run_example_loop(state, update_ui);
}
