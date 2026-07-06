use keru::*;
use keru::node_library::*;
use std::f32::consts::PI;

struct State {
    on: bool,
    speed: f32,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const TOGGLE: NodeKey;

    ui.set_global_animation_speed(state.speed);

    let on = state.on;

    let green = Color::from_hex_str("#4ade80");
    let pink = Color::from_hex_str("#f72585");
    let cyan = Color::from_hex_str("#4cc9f0");
    let blue = Color::from_hex_str("#4361ee");
    let yellow = Color::from_hex_str("#ffd166");
    let purple = Color::from_hex_str("#7209b7");
    let coral = Color::from_hex_str("#ef476f");
    let teal = Color::from_hex_str("#06d6a0");
    let orange = Color::from_hex_str("#f9844a");

    let color_to_gradient = if on {
        ColorFill2::LinearGradient(LinearGradient { color_start: pink, color_end: cyan, angle_deg: 45.0 })
    } else {
        ColorFill2::Color(green)
    };

    let gradient_angle = ColorFill2::LinearGradient(LinearGradient {
        color_start: pink,
        color_end: cyan,
        angle_deg: if on { 135.0 } else { 0.0 },
    });

    let rect_grid = Shape::SquareGrid {
        lattice_size: if on { (48.0, 16.0) } else { (16.0, 48.0) },
        offset: if on { (22.0, 12.0) } else { (0.0, 0.0) },
        line_thickness: if on { 4.0 } else { 2.0 },
    };

    let color_to_radial = if on {
        ColorFill2::RadialGradient { color_inner: yellow, color_outer: purple }
    } else {
        ColorFill2::Color(blue)
    };

    let corner_radius = Shape::Rectangle {
        rounded_corners: RoundedCorners::ALL,
        corner_radius: if on { 55.0 } else { 4.0 },
    };

    let ring_fill = if on {
        ColorFill2::RadialGradient { color_inner: cyan, color_outer: blue }
    } else {
        ColorFill2::Color(cyan)
    };
    let ring = Shape::Ring { width: if on { 44.0 } else { 8.0 } };

    let triangle = Shape::Triangle {
        rotation: if on { PI } else { 0.0 },
        width: if on { 0.6 } else { 1.0 },
    };

    let hexagon_fill = if on {
        ColorFill2::LinearGradient(LinearGradient { color_start: teal, color_end: blue, angle_deg: 90.0 })
    } else {
        ColorFill2::Color(teal)
    };
    let hexagon = Shape::Hexagon {
        size: if on { 1.0 } else { 0.55 },
        rotation: if on { PI / 2.0 } else { 0.0 },
    };

    let arc = Shape::Arc {
        start_angle: 0.0,
        end_angle: if on { PI * 1.8 } else { PI * 0.3 },
        width: if on { 34.0 } else { 14.0 },
    };

    let orange_purple = ColorFill2::LinearGradient(LinearGradient { color_start: orange, color_end: purple, angle_deg: 45.0 });
    let coral_teal = ColorFill2::LinearGradient(LinearGradient { color_start: coral, color_end: teal, angle_deg: 90.0 });
    let yellow_coral = ColorFill2::LinearGradient(LinearGradient { color_start: yellow, color_end: coral, angle_deg: 90.0 });
    let pink_yellow = ColorFill2::LinearGradient(LinearGradient { color_start: pink, color_end: yellow, angle_deg: 0.0 });

    let tile = PANEL.size_symm(Size::Pixels(140.0)).animate_properties(true);
    // Fixed-width cells so the columns line up and labels can't change the layout.
    let cell = V_STACK.size(Size::Pixels(160.0), Size::Pixels(180.0)).stack_spacing(6.0);
    let row = H_STACK.size(Size::FitContent, Size::FitContent).stack_spacing(12.0);

    let tile_text = TEXT.text_size(18.0);

    let tiles_column = V_STACK.size(Size::FitContent, Size::FitContent).stack_spacing(12.0);
    let controls_column = V_STACK.size(Size::Pixels(130.0), Size::Fill).stack_arrange(Arrange::Start).stack_spacing(12.0);

    let root = H_STACK
        .size_symm(Size::Fill)
        .stack_arrange(Arrange::Center)
        .stack_spacing(28.0);

    ui.add(root).nest(|| {
        ui.add(tiles_column).nest(|| {
            ui.add(row).nest(|| {
                ui.add(cell).nest(|| {
                    ui.add(tile.fill(color_to_gradient));
                    ui.add(tile_text.text("color -> gradient"));
                });
                ui.add(cell).nest(|| {
                    ui.add(tile.fill(gradient_angle));
                    ui.add(tile_text.text("gradient angle"));
                });
                ui.add(cell).nest(|| {
                    ui.add(tile.fill(orange_purple).shape(rect_grid));
                    ui.add(tile_text.text("rect grid"));
                });
            });

            ui.add(row).nest(|| {
                ui.add(cell).nest(|| {
                    ui.add(tile.fill(color_to_radial).shape(Shape::Circle));
                    ui.add(tile_text.text("color -> radial"));
                });
                ui.add(cell).nest(|| {
                    ui.add(tile.fill(coral_teal).shape(corner_radius));
                    ui.add(tile_text.text("corner radius"));
                });
                ui.add(cell).nest(|| {
                    ui.add(tile.fill(ring_fill).shape(ring));
                    ui.add(tile_text.text("ring width"));
                });
            });

            ui.add(row).nest(|| {
                ui.add(cell).nest(|| {
                    ui.add(tile.fill(yellow_coral).shape(triangle));
                    ui.add(tile_text.text("triangle spin"));
                });
                ui.add(cell).nest(|| {
                    ui.add(tile.fill(hexagon_fill).shape(hexagon));
                    ui.add(tile_text.text("hexagon"));
                });
                ui.add(cell).nest(|| {
                    ui.add(tile.fill(pink_yellow).shape(arc));
                    ui.add(tile_text.text("arc sweep"));
                });
            });
        });

        ui.add(controls_column).nest(|| {
            ui.add(BUTTON.key(TOGGLE).size_x(Size::Fill).text(if on { "Reset" } else { "Toggle" }));
            ui.add(TEXT.text("Speed"));
            ui.vertical_slider(&mut state.speed, 0.05, 2.0);
        });
    });

    if ui.is_clicked(TOGGLE) {
        state.on = !state.on;
    }
}

fn main() {
    let state = State { on: false, speed: 1.0 };
    example_window_loop::run_example_loop(state, update_ui);
}
