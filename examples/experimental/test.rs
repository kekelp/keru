#![allow(unused)]
use keru::*;
use keru::node_library::*;
use keru::example_window_loop::*;

struct State {}

struct Button2<'a> {
    text: &'a str,
    layout: Layout,
    key: NodeKey,
}
impl<'a> Button2<'a> {

}
impl<'a> Component for Button2<'a> {
    type State = ();
    type AddResult = ();
    type ComponentOutput = ();

    fn add_to_ui(&mut self, ui: &mut Ui, state: &mut Self::State) -> Self::AddResult {
        todo!()
    }
}


fn update_ui(state: &mut State, ui: &mut Ui) {
    let image = IMAGE
        .static_image(include_bytes!("../assets/glitch.jpg"))
        .shape(Shape::Circle)
        .shape(Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: 10.0 })
        .shape(Shape::HexGrid { lattice_size: 20.0, offset: (0.0, 0.0), line_thickness: 1.0 })
        .color(Color::from_hex(0xee8031).with_alpha(1.0))
        .image_options(ImageOptions {
            nine_slice: None,
            tile_x: TileMode::Tile,
            tile_y: TileMode::Tile,
        })
        .size(Size::Frac(0.7), Size::Frac(0.7));

    ui.add(image);

    const BACKGROUND: Color = Color::new(0.878, 0.878, 0.878 + 0.03, 1.0);
    const DARK: Color = Color::new(0.45, 0.45, 0.45 + 0.03, 1.0);
    const LIGHT: Color = Color::new(1.0, 1.0, 1.0, 1.0);

    let left_vstack = V_STACK.size_x(Size::Frac(0.3)).position_x(Pos::Start).stack_spacing(30.0);
    ui.add(left_vstack).nest(|| {
        ui.add(BUTTON.text("Cope"));
        ui.add(BUTTON.text("Seethe"));
        ui.add(BUTTON.text("Sneed"));
    });

}

fn main() {
    let state = State {};
    example_window_loop::run_example_loop(state, update_ui);
}




