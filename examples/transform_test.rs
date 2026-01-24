use keru::example_window_loop::*;
use keru::*;
use winit::keyboard::{Key, NamedKey};

#[derive(Default)]
pub struct State {
    pub zoom: f32,
    pub pan_x: f32,
    pub pan_y: f32,
    pub click_count: i32,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const PAN_OVERLAY: NodeKey;
    #[node_key] const TRANSFORMED_AREA: NodeKey;
    #[node_key] const CLICK_COUNTER_BUTTON: NodeKey;

    let pan_overlay = PANEL
        .padding(0)
        .color(Color::TRANSPARENT)
        .sense_drag(true)
        .size(Size::Fill, Size::Fill)
        .key(PAN_OVERLAY);

    let transform_area = PANEL
        .size_symm(Size::Pixels(1000000))
        .color(Color::rgba(30, 30, 40, 255))
        .key(TRANSFORMED_AREA)
        .translate(state.pan_x, state.pan_y)
        .zoom(state.zoom)
        .clip_children(true);

    let bg_panel = PANEL.size_symm(Size::Pixels(600));
    let clip_area = CONTAINER.size_symm(Size::Fill).clip_children(true);

    ui.add(bg_panel).nest(|| {
        ui.add(clip_area).nest(|| {

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
                ui.add(pan_overlay);
            }
        });
    });


    ui.add(V_STACK.stack_arrange(Arrange::Start)).nest(|| {
        ui.add(H_STACK).nest(|| {
            // ui.label(&format!("Zoom: {:.2}", state.zoom));
            ui.label("Zoom:");
            ui.add_component(SliderParams::new(&mut state.zoom, 0.5, 3.0, false));
        });

        ui.add(H_STACK).nest(|| {
            // ui.label(&format!("Pan X: {:.2}", state.pan_x));
            ui.label("Pan X:");
            ui.add_component(SliderParams::new(&mut state.pan_x, -800.0, 800.0, false));
        });

        ui.add(H_STACK).nest(|| {
            // ui.label(&format!("Pan Y: {:.2}", state.pan_y));
            ui.label("Pan Y:");
            ui.add_component(SliderParams::new(&mut state.pan_y, -800.0, 800.0, false));
        });
        ui.add(SPACER);
    });

    if let Some(drag) = ui.is_dragged(PAN_OVERLAY) {
        state.pan_x -= drag.absolute_delta.x as f32;
        state.pan_y -= drag.absolute_delta.y as f32;
    }

    if let Some(scroll_event) = ui.scrolled_at(PAN_OVERLAY) {
        let old_zoom = state.zoom;
        let zoom_delta = scroll_event.delta.y as f32;
        state.zoom += zoom_delta;

        // Adjust pan to keep the content under the mouse at the same position
        let mouse_pos = scroll_event.relative_position;
        dbg!(mouse_pos);
        dbg!((mouse_pos.x as f32) * (1.0 / state.zoom - 1.0 / old_zoom));
        
        state.pan_x += (mouse_pos.x as f32) * (1.0 / state.zoom - 1.0 / old_zoom) * 600.0;
        state.pan_y += (mouse_pos.y as f32) * (1.0 / state.zoom - 1.0 / old_zoom) * 600.0;
    }

    if state.zoom <= 0.001 { state.zoom = 0.001; }
}

fn main() {
    // basic_env_logger_init();
    let state = State {
        zoom: 1.0,
        pan_x: 0.0,
        pan_y: 0.0,
        click_count: 0,
    };
    run_example_loop(state, update_ui);
}
