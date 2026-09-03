#![allow(dead_code)]

use keru::*;
use keru::node_library::*;
use keru::example_window_loop::*;
use std::f32::consts::PI;

#[node_key] const SHAPE: NodeKey;

pub struct State {
    last_clicked: Option<String>,
}

impl State {
    fn update_ui(&mut self, ui: &mut Ui) {
        let teal = Color::from_hex_str("#06d6a0");
        let pink = Color::from_hex_str("#f72585");
        let blue = Color::from_hex_str("#4361ee");
        let yellow = Color::from_hex_str("#ffd166");

        let shapes: [(&str, Shape, Color); 12] = [
            ("rectangle", Shape::Rectangle { rounded_corners: RoundedCorners::NONE, corner_radius: 0.0 }, teal),
            ("rounded rect", Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: 45.0 }, teal),
            ("circle", Shape::Circle, pink),
            ("ring", Shape::Ring { width: 28.0 }, pink),
            ("arc", Shape::Arc { start_angle: 0.0, end_angle: PI * 1.3, width: 30.0 }, blue),
            ("pie", Shape::Pie { start_angle: PI * 0.2, end_angle: PI * 1.4 }, blue),
            ("triangle", Shape::Triangle { rotation: -PI / 2.0, width: 1.0 }, yellow),
            ("hexagon", Shape::Hexagon { size: 1.0, rotation: 0.0 }, yellow),
            ("segment", Shape::Segment { start: (0.1, 0.1), end: (0.9, 0.9), dash_length: None }, teal),
            ("h line", Shape::HorizontalLine, pink),
            ("v line", Shape::VerticalLine, blue),
            ("square grid", Shape::SquareGrid { lattice_size: (24.0, 24.0), offset: (0.0, 0.0), line_thickness: 3.0 }, yellow),
        ];

        let tile = PANEL
            .size_symm(Size::Pixels(130.0))
            .sense_click(true)
            .sense_hover(true)
            .click_animation(true)
            .hover_cursor_icon(CursorIcon::Pointer);
        let cell = V_STACK.size(Size::Pixels(150.0), Size::Pixels(170.0)).stack_spacing(6.0);
        let label = TEXT.text_size(15.0);

        let root = V_STACK.size_symm(Size::Fill).stack_arrange(Arrange::Center).stack_spacing(16.0);

        for (i, (name, _, _)) in shapes.iter().enumerate() {
            if ui.is_clicked(SHAPE.sibling(i)) {
                self.last_clicked = Some(name.to_string());
            }
        }

        let status = match &self.last_clicked {
            Some(n) => format!("Last clicked: {n}"),
            None => "Click inside a shape".to_string(),
        };

        ui.add(root).nest(|| {
            ui.add(TEXT.text(&status).text_size(24.0));

            for (chunk_i, chunk) in shapes.chunks(4).enumerate() {
                let row = H_STACK.size(Size::FitContent, Size::FitContent).stack_spacing(12.0);
                ui.add(row).nest(|| {
                    for (j, (name, shape, color)) in chunk.iter().enumerate() {
                        let i = chunk_i * 4 + j;
                        ui.add(cell).nest(|| {
                            ui.add(tile.key(SHAPE.sibling(i)).fill(ColorFill::Color(*color)).shape(*shape));
                            ui.add(label.text(name));
                        });
                    }
                });
            }
        });
    }
}

fn main() {
    basic_env_logger_init();
    let state = State { last_clicked: None };
    run_example_loop(state, State::update_ui);
}
