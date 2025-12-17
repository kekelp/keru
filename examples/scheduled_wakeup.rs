use std::time::{Duration, Instant};

use keru::*;
use keru::example_window_loop::*;

pub struct State {
    pub last_changed: Instant,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const SHOW: NodeKey;
    let button = BUTTON.static_text("Show Banner").key(SHOW);
    let label = LABEL
        .static_text("Showing up for a while")
        .position_y(Position::End)
        .slide();

    ui.add(button);
    if state.last_changed.elapsed() < Duration::from_secs(1) {
        ui.add(label);
    }

    if ui.is_clicked(SHOW) {
        state.last_changed = Instant::now();
        ui.schedule_wakeup(Duration::from_millis(1500));
    }
}

fn main() {
    let state = State { last_changed: Instant::now() - Duration::from_secs(90000), };
    run_example_loop(state, update_ui);
}
