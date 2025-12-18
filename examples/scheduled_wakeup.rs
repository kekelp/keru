use std::time::{Duration, Instant};

use keru::*;
use keru::example_window_loop::*;

pub struct State {
    pub banner_last_shown: Instant,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const SHOW: NodeKey;
    let button = BUTTON.static_text("Show Banner").key(SHOW);
    let label = LABEL
        .static_text("Showing up for a while")
        .position_y(Position::End)
        .slide_from_bottom();

    ui.add(button);
    if state.banner_last_shown.elapsed() < Duration::from_secs(1) {
        ui.add(label);
    }

    if ui.is_clicked(SHOW) {
        state.banner_last_shown = Instant::now();
        ui.schedule_wakeup(Duration::from_millis(1500));
    }
}

fn main() {
    let state = State { banner_last_shown: Instant::now() - Duration::from_secs(90000), };
    run_example_loop(state, update_ui);
}
