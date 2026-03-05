use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {
    pub transform_state: TransformViewState,
    pub click_count: i32,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    with_arena(|a| {

        #[node_key] const CLICK_COUNTER_BUTTON: NodeKey;

        let bg_panel = PANEL.size_symm(Size::Frac(0.8));

        let text = &bumpalo::format!(in a, "Click ({})", state.click_count);
        let button = BUTTON.text(text).key(CLICK_COUNTER_BUTTON);
        let transform = TransformView::new(&mut state.transform_state);

        ui.add(bg_panel).nest(|| {
            ui.add_component(transform).nest(|| {
                ui.add(V_STACK).nest(|| {
                    ui.label("Transformed subtree");

                    ui.add(button);

                    ui.add(H_STACK).nest(|| {
                        ui.add(PANEL.color(Color::RED).size_symm(Size::Pixels(50.0)));
                        ui.add(PANEL.color(Color::GREEN).size_symm(Size::Pixels(50.0)));
                        ui.add(PANEL.color(Color::BLUE).size_symm(Size::Pixels(50.0)));
                    });

                    ui.label("Don't expect scaled text to look good, though. It uses the same texture and just scales the quads");
                });
            });
        });

        if ui.is_clicked(CLICK_COUNTER_BUTTON) {
            state.click_count += 1;
        }
    });

    ui.add(V_STACK.stack_arrange(Arrange::Start).position_y(Pos::Start)).nest(|| {
        ui.add(H_STACK).nest(|| {
            ui.label("Zoom:");
            ui.add_component(Slider::new(&mut state.transform_state.scale, 0.1, 5.0, false));
        });

        ui.add(H_STACK).nest(|| {
            ui.label("Pan X:");
            ui.add_component(Slider::new(&mut state.transform_state.pan_x, -800.0, 800.0, false));
        });

        ui.add(H_STACK).nest(|| {
            ui.label("Pan Y:");
            ui.add_component(Slider::new(&mut state.transform_state.pan_y, -800.0, 800.0, false));
        });
    });

    ui.add(V_STACK.stack_arrange(Arrange::End).position_y(Pos::End)).nest(|| {
        ui.static_label("Middle click drag / Space + drag to pan. Scroll or Space + middle click drag to zoom");
    });
}

fn main() {
    let state = State {
        transform_state: TransformViewState {
            scale: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            zoom_drag_anchor: None,
        },
        click_count: 0,
    };
    run_example_loop(state, update_ui);
}
