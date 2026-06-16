use keru::*;
use keru::node_library::*;

#[node_key] const GRADIENT_SOURCE: NodeKey;

const GRAD: LinearGradient = LinearGradient {
    color_start: Color::from_hex_str("#f72585"),
    color_end: Color::from_hex_str("#4cc9f0"),
    angle_deg: 45.0,
};

fn update_ui(_state: &mut (), ui: &mut Ui) {
    // add a fullscreen invisible node just to hold the gradient.
    let gradient_node = PANEL
        .key(GRADIENT_SOURCE)
        .linear_gradient(GRAD)
        .invisible()
        .size(Size::Fill, Size::Fill);

    let circle = BUTTON
        .shared_gradient(GRADIENT_SOURCE)
        .shape(Shape::Circle)
        .size_symm(Size::Pixels(85.0))
        .anchor_symm(Anchor::Center);

    let border = PANEL
        .color(Color::TRANSPARENT)
        .absorbs_clicks(false)
        .stroke(14.0)
        .stroke_fill(ColorFill2::SharedGradient(GRADIENT_SOURCE))
        .size_symm(Size::Frac(0.9))
        .anchor_symm(Anchor::Center)
        .position_symm(Pos::Frac(0.5));

    let text = LABEL.shared_gradient(GRADIENT_SOURCE).size_x(Size::Fill);

    ui.add(gradient_node);

    ui.add(V_STACK.size_symm(Size::Fill)).nest(|| {
        ui.add(CONTAINER.size_symm(Size::Fill)).nest(|| {
            ui.add(border);
            for i in 2..9 {
                let pos = 0.1 * i as f32;
                ui.add(circle.position_symm(Pos::Frac(pos)).text(&format!("{}", i - 1)));
            }
        });

        ui.add(text.static_text("Nodes can share a gradient with another node. This is an easy way to create richer gradient effects."));
    });
}

fn main() {
    example_window_loop::run_example_loop((), update_ui);
}
