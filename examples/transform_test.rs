use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {
    pub zoom: f32,
    pub pan_x: f32,
    pub pan_y: f32,
    pub click_count: i32,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const TRANSFORMED_CONTAINER: NodeKey;
    #[node_key] const CLICK_COUNTER_BUTTON: NodeKey;

    ui.v_stack().nest(|| {
        // Control panel
        ui.label("Transform Controls");

        // Zoom slider
        ui.h_stack().nest(|| {
            ui.label("Zoom:");
            ui.add_component(SliderParams::new(&mut state.zoom, 0.5, 3.0));
            ui.label(&format!("{:.2}", state.zoom));
        });

        // Pan X slider
        ui.h_stack().nest(|| {
            ui.label("Pan X:");
            ui.add_component(SliderParams::new(&mut state.pan_x, -200.0, 200.0));
            ui.label(&format!("{:.0}", state.pan_x));
        });

        // Pan Y slider
        ui.h_stack().nest(|| {
            ui.label("Pan Y:");
            ui.add_component(SliderParams::new(&mut state.pan_y, -200.0, 200.0));
            ui.label(&format!("{:.0}", state.pan_y));
        });

        ui.spacer();

    });

    // // Transformed content area
    ui.add(
        PANEL
            .size_symm(Size::Fill)
            .color(Color::rgba(30, 30, 40, 255))
            .key(TRANSFORMED_CONTAINER)
            .translate(state.pan_x, state.pan_y)
            .zoom(state.zoom)
    ).nest(|| {
        ui.v_stack().nest(|| {
            // Title
            ui.label("Transformed Content");

            // Interactive button
            if ui.add(BUTTON.text(&format!("Click me! ({})", state.click_count)).key(CLICK_COUNTER_BUTTON)).is_clicked(ui) {
                state.click_count += 1;
            }

            // Some visual elements
            ui.h_stack().nest(|| {
                ui.add(PANEL.color(Color::RED).size_symm(Size::Pixels(50)));
                ui.add(PANEL.color(Color::GREEN).size_symm(Size::Pixels(50)));
                ui.add(PANEL.color(Color::BLUE).size_symm(Size::Pixels(50)));
            });

            ui.label("This entire panel is transformed!");
            ui.label("Notice how clicks still work correctly.");
        });
    });
}

fn main() {
    let state = State {
        zoom: 1.0,
        pan_x: 0.0,
        pan_y: 0.0,
        click_count: 0,
    };
    run_example_loop(state, update_ui);
}
