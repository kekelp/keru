#![allow(unused)]
use std::time::Instant;

use keru::*;
use keru_draw::{Box as DrawBox, ColorFill, Segment};

struct State {
    items: Vec<&'static str>,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    // Simple canvas test
    #[node_key] const CANVAS: NodeKey;
    let canvas_node = CONTAINER
        .size_x(Size::Pixels(300.0))
        .size_y(Size::Pixels(200.0))
        .color(Color::rgba_u8(50, 50, 50, 255))
        .key(CANVAS);

    ui.add(canvas_node);

    ui.canvas_drawing(CANVAS, |renderer| {
        // Draw a simple red box at (10, 10)
        renderer.draw_box(DrawBox {
            top_left: [10.0, 10.0],
            size: [100.0, 50.0],
            corner_radius: 5.0,
            rounded_corners: keru_draw::RoundedCorners::ALL,
            border_thickness: 0.0,
            fill: ColorFill::Color(Color::RED),
            texture: None,
        });

        // Draw a simple line
        renderer.draw_segment(Segment {
            start: [20.0, 100.0],
            end: [200.0, 150.0],
            thickness: 5.0,
            fill: ColorFill::Color(Color::KERU_GREEN),
            dash_length: None,
            dash_offset: 0.0,
            texture: None,
        });
    });

}

fn main() {
    let items = vec!["A", "special", "B", "C", "xxxxxx\nxxxxxx\nxxxxxx", "D", "E"];

    let state = State {
        items,
    };

    example_window_loop::run_example_loop(state, update_ui);
}
