// Reproduction of the AirBnB-style circular month slider from PanGui.
// Imitation is the best form of flattery...
// Keru's renderer can't do non-rectangular clipping, so we have to use some tricks to simulate it.
// PanGui uses a much more advanced SDF-based renderer that can do plenty of great things besides just nonrectangular clipping, but that's the only thing we're missing for this one example. 

use keru::*;
use keru::node_library::*;
use std::f32::consts::TAU;
use std::time::Instant;

const CONTAINER_SIZE: f32 = 500.0;
const INNER_RADIUS: f32 = 90.0;
const OUTER_RADIUS: f32 = 150.0;
const HALF_THICKNESS: f32 = (OUTER_RADIUS - INNER_RADIUS) / 2.0;
const TRACK_RADIUS: f32 = INNER_RADIUS + HALF_THICKNESS;
const HANDLE_RADIUS: f32 = HALF_THICKNESS - 10.0;
const SHADOW_BLEED: f32 = 500.0;

const BG: Color = Color::new(0.96, 0.95, 0.97, 1.0);

struct State {
    month: u32,
    t: f32,
    last_update: Instant,
}

impl Default for State {
    fn default() -> Self {
        Self {
            month: 1,
            t: 1.0 / 12.0,
            last_update: Instant::now(),
        }
    }
}

fn arc_pos(t: f32, radius: f32) -> [f32; 2] {
    let cx = CONTAINER_SIZE / 2.0;
    let cy = CONTAINER_SIZE / 2.0;
    [cx + (t * TAU).sin() * radius, cy - (t * TAU).cos() * radius]
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    let now = Instant::now();
    let dt = now.duration_since(state.last_update).as_secs_f32().min(0.1);
    state.last_update = now;

    #[node_key] const CONTAINER: NodeKey;
    #[node_key] const HANDLE: NodeKey;

    let maybe_drag = ui.is_dragged(HANDLE);
    let container_center = ui.get_node(CONTAINER).map(|n| n.center());

    if let (Some(drag), Some(center)) = (maybe_drag, container_center) {
        let dx = drag.absolute_pos.x - center.x;
        let dy = drag.absolute_pos.y - center.y;
        let new_t = f32::atan2(dx, -dy) / TAU;
        let new_t = if new_t < 0.0 { new_t + 1.0 } else { new_t };
        state.t = new_t.clamp(1.0 / 12.0, 1.0);
        state.month = ((state.t * 12.0).round() as u32).clamp(1, 12);
    } else {
        let target_t = state.month as f32 / 12.0;
        state.t += (target_t - state.t) * (1.0 - (-10.0 * dt).exp());
    }

    let is_dragging = maybe_drag.is_some();
    let is_hovered = ui.is_hovered(HANDLE);
    let handle_visual_radius = if is_hovered || is_dragging {
        HANDLE_RADIUS + 4.0
    } else {
        HANDLE_RADIUS
    };

    let [hx, hy] = arc_pos(state.t, TRACK_RADIUS);

    let container = DEFAULT
        .color(BG)
        .size_symm(Size::Pixels(CONTAINER_SIZE))
        .anchor_symm(Anchor::Center)
        .position_symm(Pos::Center)
        .sense_time(true)
        .key(CONTAINER);

    let handle = DEFAULT
        .shape(Shape::Circle)
        .color(Color::rgba_u8(255, 252, 255, 255))
        .size_symm(Size::Pixels(handle_visual_radius * 2.0))
        .anchor_symm(Anchor::Center)
        .position_x(Pos::Pixels(hx))
        .position_y(Pos::Pixels(hy))
        .sense_drag(true)
        .sense_hover(true)
        .shadow(Shadow { blur: 4.0, offset: Xy::new(0.0, 2.0), color: Some(Color::rgba_u8(0, 0, 0, 100)) })
        .absorbs_clicks(false)
        .animate_position(true)
        .key(HANDLE);

    let center_stack = V_STACK
        .size_symm(Size::FitContent)
        .anchor_symm(Anchor::Center)
        .position_symm(Pos::Center)
        .stack_spacing(2.0)
        .stack_arrange(Arrange::Center);

    with_arena(|a| {
        let month_str = bumpalo::format!(in a, "{}", state.month);
        let label_str = if state.month == 1 { "month" } else { "months" };

        ui.add(container).nest(|| {
            ui.add(center_stack).nest(|| {
                ui.add(TEXT.text(month_str.as_str()).text_size(80.0).bold().text_color(Color::rgba_u8(20, 20, 20, 255)));
                ui.add(TEXT.static_text(label_str).text_size(18.0).text_color(Color::rgba_u8(80, 80, 80, 255)));
            });
            ui.add(handle);
        });
    });

    if let Some(node) = ui.get_node_mut(CONTAINER) {
        let t = state.t;
        let month = state.month;
        node.canvas_drawing(move |canvas| {
            use keru::{Circle, CircleArc, CircleRing, ColorFill};

            let cx = CONTAINER_SIZE / 2.0;
            let cy = CONTAINER_SIZE / 2.0;
            let thickness = OUTER_RADIUS - INNER_RADIUS;

            // Background track ring
            canvas.draw_ring(CircleRing {
                center: [cx, cy],
                inner_radius: INNER_RADIUS,
                outer_radius: OUTER_RADIUS,
                fill: ColorFill::Color(Color::rgba_u8(0, 0, 0, 25)),
                texture: None,
                texture_options: None,
                dash_length: None,
                dash_offset: 0.0,
                blur: 0.0,
            });

            // Month markers
            for i in 0..12 {
                let pos = arc_pos(i as f32 / 12.0, TRACK_RADIUS);
                let is_current = i + 1 == month;
                canvas.draw_circle(Circle {
                    center: pos,
                    radius: if is_current { 3.5 } else { 2.5 },
                    fill: ColorFill::Color(Color::rgba_u8(0, 0, 0, if is_current { 140 } else { 70 })),
                    texture: None,
                    texture_options: None,
                    blur: 0.0,
                });
            }

            let start_angle = -TAU / 4.0;
            let end_angle = start_angle + t * TAU;

            // Track glow
            canvas.draw_arc(CircleArc {
                center: [cx, cy],
                radius: TRACK_RADIUS,
                start_angle,
                end_angle,
                thickness,
                fill: ColorFill::Color(Color::rgba_u8(186, 0, 87, 160)),
                texture: None,
                texture_options: None,
                dash_length: None,
                dash_offset: 0.0,
                blur: 30.0,
            });

            let arc_color = ColorFill::Gradient(Gradient::radial(
                Color::rgba_u8(249, 30, 80, 255),
                Color::rgba_u8(186, 0, 87, 255),
            ));

            // Track body
            canvas.draw_arc(CircleArc {
                center: [cx, cy],
                radius: TRACK_RADIUS,
                start_angle,
                end_angle,
                thickness,
                fill: arc_color,
                texture: None,
                texture_options: None,
                dash_length: None,
                dash_offset: 0.0,
                blur: 0.0,
            });

            let start_pos = arc_pos(0.0, TRACK_RADIUS);
            canvas.draw_circle(Circle {
                center: start_pos,
                radius: HALF_THICKNESS,
                fill: arc_color,
                texture: None,
                texture_options: None,
                blur: 0.0,
            });

            let end_pos = arc_pos(t, TRACK_RADIUS);
            canvas.draw_circle(Circle {
                center: end_pos,
                radius: HALF_THICKNESS,
                fill: arc_color,
                texture: None,
                texture_options: None,
                blur: 0.0,
            });

            // Mask the shadow bleed on the inside with a filled circle matching the background.
            canvas.draw_circle(Circle {
                center: [cx, cy],
                radius: INNER_RADIUS,
                fill: ColorFill::Color(BG),
                texture: None,
                texture_options: None,
                blur: 0.0,
            });

            // Mask the shadow bleed on the outside with an opaque ring matching the background.
            canvas.draw_ring(CircleRing {
                center: [cx, cy],
                inner_radius: OUTER_RADIUS,
                outer_radius: OUTER_RADIUS + SHADOW_BLEED,
                fill: ColorFill::Color(BG),
                texture: None,
                texture_options: None,
                dash_length: None,
                dash_offset: 0.0,
                blur: 0.0,
            });
        });
    }
}

fn main() {
    let state = State::default();
    example_window_loop::run_example_loop(state, update_ui);
}
