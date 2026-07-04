use keru::*;
use keru::node_library::*;
use keru::example_window_loop::*;

#[derive(Default)]
struct State {
    on: bool,
    fade_on: bool,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const TOGGLE: NodeKey;
    #[node_key] const BOX: NodeKey;
    #[node_key] const FADE_TOGGLE: NodeKey;
    #[node_key] const FADE_BOX: NodeKey;
    #[node_key] const FADE_CANVAS: NodeKey;

    ui.set_global_animation_speed(1.0);

    let fade_toggle = BUTTON
        .key(FADE_TOGGLE)
        .text("Fade in/out");

    let fade_panel = PANEL
        .key(FADE_BOX)
        .size_x(Size::Pixels(280.0))
        .size_y(Size::FitContent)
        .color(Color::rgba_u8(80, 200, 120, 255))
        .stack_arrange(Arrange::Start)
        .fade();

    let inner_label = BUTTON
        .text("child button");

    let inner_box = PANEL
        .size_x(Size::Pixels(120.0))
        .size_y(Size::Pixels(60.0))
        .color(Color::rgba_u8(220, 180, 60, 255));

    let inner_box2 = PANEL
        .size_x(Size::Pixels(80.0))
        .size_y(Size::Pixels(40.0))
        .color(Color::rgba_u8(40, 40, 40, 255))
        .alpha(0.5);

    let inner_image = IMAGE
        .static_image(include_bytes!("../../src/textures/clouds.png"))
        .size(Size::Pixels(200.0), Size::Pixels(100.0));

    let inner_canvas = CONTAINER
        .key(FADE_CANVAS)
        .size_x(Size::Pixels(200.0))
        .size_y(Size::Pixels(120.0))
        .padding(0.0)
        .color(Color::rgba_u8(30, 30, 40, 255));

    let v_stack = V_STACK.size_y(Size::Fill).size_x(Size::Fill).stack_arrange(Arrange::Start);

    ui.add(v_stack).nest(|| {
        ui.add(fade_toggle);
        if state.fade_on {
            ui.add(fade_panel).nest(|| {
                ui.add(inner_label);
                ui.add(inner_box);
                ui.add(inner_box2);
                ui.add(inner_image);
                ui.add(inner_canvas);
                ui.get_node_mut(FADE_CANVAS).unwrap().canvas_drawing(|canvas| {
                    use keru_draw::{Segment, ColorFill};

                    let num_points = 120;
                    let width = 200.0;
                    let height = 120.0;
                    let margin = 20.0;
                    let num_coils = 6.0;
                    let coil_radius = 30.0;

                    let points: Vec<[f32; 2]> = (0..num_points)
                        .map(|i| {
                            let t = i as f32 / (num_points - 1) as f32;
                            let angle = t * num_coils * 2.0 * std::f32::consts::PI;
                            let x = margin + t * (width - 2.0 * margin) + coil_radius * 0.3 * angle.cos();
                            let y = height / 2.0 + coil_radius * angle.sin();
                            [x, y]
                        })
                        .collect();

                    for i in 0..points.len() - 1 {
                        let t = i as f32 / (num_points - 1) as f32;
                        let angle = t * num_coils * 2.0 * std::f32::consts::PI;
                        let depth = angle.cos();
                        let thickness = 3.0 + 2.5 * (depth + 1.0) / 2.0;

                        let color = if depth > 0.0 {
                            Color::KERU_PINK
                        } else {
                            Color::rgba_u8(200, 100, 150, 255)
                        };

                        canvas.draw_segment(Segment {
                            start: points[i],
                            end: points[i + 1],
                            thickness,
                            stroke_thickness: 0.0,
                            fill: ColorFill::Color(color),
                            dash_length: None,
                            dash_offset: 0.0,
                            blur: 0.0,
                            texture: None,
                            texture_options: None,
                        });
                    }
                });
            });
        }
    });

    if ui.is_clicked(TOGGLE) {
        state.on = !state.on;
    }
    if ui.is_clicked(FADE_TOGGLE) {
        state.fade_on = !state.fade_on;
    }
}

fn main() {
    run_example_loop(State::default(), update_ui);
}
