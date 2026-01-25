use glam::{DVec2, dvec2};
use keru::example_window_loop::*;
use keru::*;
use winit::event::MouseButton;
use winit::keyboard::{Key, NamedKey};

#[derive(Default)]
pub struct State {
    pub zoom: f32,
    pub pan_x: f32,
    pub pan_y: f32,
    pub click_count: i32,
    pub zoom_drag_anchor: Option<glam::DVec2>,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const PAN_OVERLAY: NodeKey;
    #[node_key] const SPACEBAR_PAN_OVERLAY: NodeKey;
    #[node_key] const TRANSFORMED_AREA: NodeKey;
    #[node_key] const CLICK_COUNTER_BUTTON: NodeKey;

    let spacebar_pan_overlay = PANEL
        .padding(0)
        .color(Color::TRANSPARENT)
        .sense_drag(true)
        .size(Size::Fill, Size::Fill)
        .key(SPACEBAR_PAN_OVERLAY);

    let pan_overlay = PANEL
        .padding(0)
        .color(Color::TRANSPARENT)
        .sense_drag(true)
        .absorbs_clicks(false)
        .size(Size::Fill, Size::Fill)
        .key(PAN_OVERLAY);

    let transform_area = PANEL
        .size_symm(Size::Pixels(1000000))
        .color(Color::rgba(30, 30, 40, 255))
        .key(TRANSFORMED_AREA)
        .translate(state.pan_x, state.pan_y)
        .zoom(state.zoom)
        .size_symm(Size::Fill)
        .clip_children(true);

    let bg_panel = PANEL.size_symm(Size::Frac(0.8));

    ui.add(bg_panel).nest(|| {

        ui.add(transform_area).nest(|| {

            ui.add(V_STACK).nest(|| {
                ui.label("Transformed subtree");

                if ui.add(BUTTON.text(&format!("Click ({})", state.click_count)).key(CLICK_COUNTER_BUTTON)).is_clicked(ui) {
                    state.click_count += 1;
                }

                ui.add(H_STACK).nest(|| {
                    ui.add(PANEL.color(Color::RED).size_symm(Size::Pixels(50)));
                    ui.add(PANEL.color(Color::GREEN).size_symm(Size::Pixels(50)));
                    ui.add(PANEL.color(Color::BLUE).size_symm(Size::Pixels(50)));
                });

                ui.label("Don't expect scaled text to look good, though. It uses the same texture and just scales the quads");
            });

        });
            
        if ui.key_input().key_held(&Key::Named(NamedKey::Space)) {
            ui.add(spacebar_pan_overlay);
        }
        
        ui.add(pan_overlay);
    
    });

    let size = ui.inner_size(TRANSFORMED_AREA).unwrap_or(Xy::new(600, 600));

    ui.add(V_STACK.stack_arrange(Arrange::Start).position_y(Position::Start)).nest(|| {
        ui.add(H_STACK).nest(|| {
            ui.label("Zoom:");
            ui.add_component(SliderParams::new(&mut state.zoom, 0.1, 5.0, false));
        });

        ui.add(H_STACK).nest(|| {
            ui.label("Pan X:");
            ui.add_component(SliderParams::new(&mut state.pan_x, -800.0, 800.0, false));
        });

        ui.add(H_STACK).nest(|| {
            ui.label("Pan Y:");
            ui.add_component(SliderParams::new(&mut state.pan_y, -800.0, 800.0, false));
        });
    });

    ui.add(V_STACK.stack_arrange(Arrange::End).position_y(Position::End)).nest(|| {
        ui.static_label("Middle click drag / Space + drag to pan. Scroll Space + middle click drag to zoom");
    });

    if ! ui.key_input().key_held(&Key::Named(NamedKey::Space)) {
        if let Some(drag) = ui.is_mouse_button_dragged(PAN_OVERLAY, MouseButton::Middle) {
            state.pan_x -= drag.absolute_delta.x as f32;
            state.pan_y -= drag.absolute_delta.y as f32;
        }
    }

    if let Some(drag) = ui.is_dragged(SPACEBAR_PAN_OVERLAY) {
        state.pan_x -= drag.absolute_delta.x as f32;
        state.pan_y -= drag.absolute_delta.y as f32;
    }

    let mut apply_zoom = |delta_y: f64, mouse_pos: DVec2| {
        let old_zoom = state.zoom;
        let curve_factor = ((0.01 + old_zoom).powf(1.1) - 0.01).abs();
        let new_zoom = old_zoom + delta_y as f32 * curve_factor;
        
        if new_zoom > 0.01 && !new_zoom.is_infinite() && !new_zoom.is_nan() {
            state.zoom = new_zoom;
            let zoom_ratio = state.zoom / old_zoom;
            let centered_pos = mouse_pos - dvec2(0.5, 0.5);
            state.pan_x = state.pan_x * zoom_ratio + size.x as f32 * centered_pos.x as f32 * (1.0 - zoom_ratio);
            state.pan_y = state.pan_y * zoom_ratio + size.y as f32 * centered_pos.y as f32 * (1.0 - zoom_ratio);
        }
    };

    if let Some(drag) = ui.is_mouse_button_dragged(SPACEBAR_PAN_OVERLAY, MouseButton::Middle) {
        if state.zoom_drag_anchor.is_none() {
            state.zoom_drag_anchor = Some(drag.relative_position);
        }

        apply_zoom(drag.absolute_delta.y * 0.01, state.zoom_drag_anchor.unwrap());

    } else {
        state.zoom_drag_anchor = None;
    }

    if let Some(scroll_event) = ui.scrolled_at(PAN_OVERLAY) {
        apply_zoom(scroll_event.delta.y, scroll_event.relative_position);
    }
}

fn main() {
    // basic_env_logger_init();
    let state = State {
        zoom: 1.0,
        pan_x: 0.0,
        pan_y: 0.0,
        click_count: 0,
        zoom_drag_anchor: None,
    };
    run_example_loop(state, update_ui);
}
