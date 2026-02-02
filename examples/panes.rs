use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {
    pub left_pane_frac: f32,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const LEFT_PANE: NodeKey;
    #[node_key] const MIDDLING_TIER: NodeKey;
    #[node_key] const RIGHT_PANE: NodeKey;
    
    let left = LABEL.text("Left").key(LEFT_PANE)
        .size_x(Size::Frac(state.left_pane_frac))
        .size_y(Size::Fill);
    
    let middling = PANEL
        .key(MIDDLING_TIER)
        .size_y(Size::Fill)
        .size_x(Size::Pixels(16.0))
        .sense_drag(true);

    let right = LABEL.text("Right").key(RIGHT_PANE)
        .size_x(Size::Fill)
        .size_y(Size::Fill);

    ui.h_stack().nest(|| {
        ui.add(left);
        ui.add(middling);
        ui.add(right);
    });

    let width = 800.0;

    if let Some(drag) = ui.is_dragged(MIDDLING_TIER) {
        state.left_pane_frac -= drag.absolute_delta.x as f32 / width;
        state.left_pane_frac = state.left_pane_frac.clamp(0.05, 0.95);
    }
}

fn main() {
    let state = State {
        left_pane_frac: 0.3,
    };

    run_example_loop(state, update_ui);
}
