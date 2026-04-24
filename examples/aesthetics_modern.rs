use keru::*;

struct State {}

const DARK: Color = Color::new(0.95, 0.95, 0.95, 1.0);

const MODERN_BUTTON: Node = BUTTON
    .shape(Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: 10.0 })
    .color(Color::WHITE)
    .stroke_width(1.0)
    .stroke_color(Color::new(0.93, 0.93, 0.93, 1.0))
    .shadow(Shadow { blur: 2.5, offset: Xy::new(2.5, 2.5), color: Some(DARK) });

fn update_ui(_state: &mut State, ui: &mut Ui) {
    let background = PANEL.color(Color::WHITE).size(Size::Fill, Size::Fill);

    ui.add(background);

    ui.add(V_STACK.stack_spacing(50.0)).nest(|| {
        
        ui.add(MODERN_BUTTON.static_text("Neumorphism button"));
    });
}

fn main() {
    let state = State {};
    example_window_loop::run_example_loop(state, update_ui);
}
