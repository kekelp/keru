use keru::*;

struct State {}

const BACKGROUND: Color = Color::new(0.878, 0.878, 0.878 + 0.03, 1.0);
const DARK: Color = Color::new(0.45, 0.45, 0.45 + 0.03, 1.0);
const LIGHT: Color = Color::new(1.0, 1.0, 1.0, 1.0);

const NEUMORPHIC_BUTTON: Node = BUTTON
    .color(BACKGROUND)
    .shape(Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: 16.0 })
    .shadow(Shadow { blur: 18.0, offset: Xy::new(6.0, 6.0), color: Some(DARK) })
    .second_shadow(Shadow { blur: 18.0, offset: Xy::new(-6.0, -6.0), color: Some(LIGHT) });

const NEUMORPHIC_BUTTON_CIRCLE: Node = NEUMORPHIC_BUTTON.shape(Shape::Circle);
const NEUMORPHIC_BUTTON_HEXAGON: Node = NEUMORPHIC_BUTTON.shape(Shape::Hexagon { size: 1.0, rotation: 0.0 });

fn update_ui(_state: &mut State, ui: &mut Ui) {
    let background = PANEL.color(BACKGROUND).size(Size::Fill, Size::Fill);

    ui.add(background);

    ui.add(V_STACK.stack_spacing(50.0)).nest(|| {
        
        ui.add(NEUMORPHIC_BUTTON.static_text("Neumorphism button"));
        ui.add(NEUMORPHIC_BUTTON_CIRCLE.size_symm(Size::Pixels(150.0)));
        ui.add(NEUMORPHIC_BUTTON_HEXAGON.size_symm(Size::Pixels(150.0)));
    });
}

fn main() {
    let state = State {};
    example_window_loop::run_example_loop(state, update_ui);
}
