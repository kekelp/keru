// Reproduction of the AirBnB-style circular month slider from PanGui.
// Imitation is the best form of flattery...
// Keru's renderer can't do non-rectangular clipping, so we have to use some tricks to simulate it.
// PanGui uses a much more advanced SDF-based renderer that can do plenty of great things besides just nonrectangular clipping, but that's the only thing we're missing for this one example.

use keru::*;
use keru::node_library::*;
use std::f32::consts::TAU;

const CONTAINER_SIZE: f32 = 500.0;
const INNER_RADIUS: f32 = 90.0;
const OUTER_RADIUS: f32 = 150.0;
const THICKNESS: f32 = OUTER_RADIUS - INNER_RADIUS;
const TRACK_RADIUS: f32 = INNER_RADIUS + THICKNESS / 2.0;
const HANDLE_RADIUS: f32 = THICKNESS / 2.0 - 10.0;
const SHADOW_BLEED: f32 = 500.0;

const BG: Color = Color::new(0.96, 0.95, 0.97, 1.0);

struct State {
    month: u32,
    t: f32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            month: 1,
            t: 1.0 / 12.0,
        }
    }
}

fn arc_pos(t: f32, radius: f32) -> [f32; 2] {
    let cx = CONTAINER_SIZE / 2.0;
    let cy = CONTAINER_SIZE / 2.0;
    [cx + (t * TAU).sin() * radius, cy - (t * TAU).cos() * radius]
}

fn update_ui(state: &mut State, ui: &mut Ui) {
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
        state.t = state.month as f32 / 12.0;
    }

    let is_dragging = maybe_drag.is_some();
    let is_hovered = ui.is_hovered(HANDLE);
    let handle_visual_radius = if is_hovered || is_dragging {
        HANDLE_RADIUS + 4.0
    } else {
        HANDLE_RADIUS
    };

    let t = state.t;
    let month = state.month;
    let [hx, hy] = arc_pos(t, TRACK_RADIUS);

    let start_angle = -TAU / 4.0;
    let end_angle = start_angle + t * TAU;

    let clip_wrapper = DEFAULT
        .color(BG)
        .size_symm(Size::Pixels(360.0))
        .anchor_symm(Anchor::Center)
        .position_symm(Pos::Center)
        .clip_children_x(true)
        .clip_children_y(true);

    let container = DEFAULT
        .color(BG)
        .size_symm(Size::Pixels(CONTAINER_SIZE))
        .anchor_symm(Anchor::Center)
        .position_symm(Pos::Center)
        .sense_time(true)
        .key(CONTAINER);

    let bg_ring = DEFAULT
        .shape(Shape::Ring { width: THICKNESS })
        .color(Color::rgba_u8(0, 0, 0, 25))
        .size_symm(Size::Pixels(OUTER_RADIUS * 2.0))
        .anchor_symm(Anchor::Center)
        .position_symm(Pos::Center);

    let glow_arc = DEFAULT
        .shape(Shape::Arc { start_angle, end_angle, width: THICKNESS })
        .color(Color::rgba_u8(186, 0, 87, 200))
        .blur(60.0)
        .size_symm(Size::Pixels(TRACK_RADIUS * 2.0))
        .anchor_symm(Anchor::Center)
        .position_symm(Pos::Center);

    let track_arc = DEFAULT
        .shape(Shape::Arc { start_angle, end_angle, width: THICKNESS })
        .fill(ColorFill2::RadialGradient {
            color_inner: Color::rgba_u8(249, 30, 80, 200),
            color_outer: Color::rgba_u8(186, 0, 87, 200),
        })
        .size_symm(Size::Pixels(TRACK_RADIUS * 2.0))
        .anchor_symm(Anchor::Center)
        .position_symm(Pos::Center);

    let inner_mask = DEFAULT
        .shape(Shape::Circle)
        .color(BG)
        .size_symm(Size::Pixels(INNER_RADIUS * 2.0))
        .anchor_symm(Anchor::Center)
        .position_symm(Pos::Center);

    let outer_mask = DEFAULT
        .shape(Shape::Ring { width: SHADOW_BLEED })
        .color(BG)
        .size_symm(Size::Pixels((OUTER_RADIUS + SHADOW_BLEED) * 2.0))
        .anchor_symm(Anchor::Center)
        .position_symm(Pos::Center);

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
        .key(HANDLE);

    let center_stack = V_STACK
        .size_symm(Size::FitContent)
        .anchor_symm(Anchor::Center)
        .position_symm(Pos::Center)
        .stack_spacing(0.0)
        .stack_arrange(Arrange::Center);

    with_arena(|a| {
        let month_str = bumpalo::format!(in a, "{}", state.month);
        let label_str = if state.month == 1 { "month" } else { "months" };

        ui.add(PANEL.size_symm(Size::Fill).color(BG));

        ui.add(clip_wrapper).nest(|| {
            ui.add(container).nest(|| {
                ui.add(bg_ring);

                for i in 0..12u32 {
                    let pos = arc_pos(i as f32 / 12.0, TRACK_RADIUS);
                    let is_current = i + 1 == month;
                    let dot = DEFAULT
                        .shape(Shape::Circle)
                        .color(Color::rgba_u8(0, 0, 0, if is_current { 140 } else { 70 }))
                        .size_symm(Size::Pixels(if is_current { 7.0 } else { 5.0 }))
                        .anchor_symm(Anchor::Center)
                        .position_x(Pos::Pixels(pos[0]))
                        .position_y(Pos::Pixels(pos[1]));
                    ui.add(dot);
                }

                ui.add(glow_arc);
                ui.add(track_arc);
                ui.add(inner_mask);
                ui.add(outer_mask);

                ui.add(center_stack).nest(|| {
                    ui.add(TEXT.text(month_str.as_str()).text_size(80.0).bold().text_color(Color::rgba_u8(20, 20, 20, 255)));
                    ui.add(TEXT.static_text(label_str).text_size(18.0).bold().text_color(Color::rgba_u8(80, 80, 80, 255)));
                });

                ui.add(handle);
            });
        });
    });
}

fn main() {
    let state = State::default();
    example_window_loop::run_example_loop(state, update_ui);
}
