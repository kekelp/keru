use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {
    pub click_count: i32,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const CLICK_COUNTER_BUTTON: NodeKey;
    #[component_key] const TRANSFORM_VIEW: ComponentKey<StatefulTransformView>;

    let bg_panel = PANEL.size_symm(Size::Frac(0.8));

    ui.add(bg_panel).nest(|| {
        ui.add_component(StatefulTransformView::new(TRANSFORM_VIEW).initial_zoom(1.0)).nest(|| {
            ui.add(V_STACK).nest(|| {
                ui.label("Transformed subtree (stateful component)");

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
    });

    ui.add(V_STACK.stack_arrange(Arrange::End).position_y(Position::End)).nest(|| {
        ui.static_label("Middle click drag / Space + drag to pan. Scroll or Space + middle click drag to zoom");
        ui.static_label("Transform state is managed internally by the component");
    });
}

fn main() {
    run_example_loop(State::default(), update_ui);
}
