use keru::*;
use keru::node_library::{V_STACK, TEXT, SPACER};

struct State {}

const DARK: Color = Color::new(0.95, 0.95, 0.95, 1.0);

const BIG_TEXT: Node<'_> = TEXT.text_color(Color::BLACK).text_size(20.0);
const NORMAL_TEXT: Node<'_> = TEXT.text_color(Color::new(0.67, 0.67, 0.67, 1.0)).text_size(18.0);

const PANEL: Node = keru::node_library::PANEL
    .shape(Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: 10.0 })
    .color(Color::WHITE)
    .stroke_width(1.0)
    .stroke_color(Color::new(0.93, 0.93, 0.93, 1.0))
    .shadow(Shadow { blur: 2.5, offset: Xy::new(2.5, 2.5), color: Some(DARK) });

fn update_ui(_state: &mut State, ui: &mut Ui) {
    let background = PANEL.color(Color::WHITE).size(Size::Fill, Size::Fill);

    ui.add(background);
    ui.add(BIG_TEXT.text("Sneed"));

    
    ui.add(PANEL).nest(|| {
        ui.add(V_STACK.stack_spacing(5.0)).nest(|| {
            ui.add(BIG_TEXT.text("Modern Webslop").position_x(Pos::Start));
            ui.add(NORMAL_TEXT.text("I hecking love it").position_x(Pos::Start));

            ui.add(SPACER.size_y(Size::Pixels(30.0)).size_x(Size::Pixels(50.0)));
        });
    });
}

fn main() {
    let state = State {};
    example_window_loop::run_example_loop(state, update_ui);
}
