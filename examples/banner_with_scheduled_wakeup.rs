//! This example uses [Ui::schedule_wakeup()] to make a banner show up and go away, without having the loop tick continuously.
//! 
//! This assumes that the winit loop is being controlled according to [Ui::should_request_redraw()], like in `run_example_loop` and in the `window_loop.rs` example.
//! 
//! ```
//! # use keru::*; let mut ui: Ui = unimplemented!();
//! if state.ui.should_request_redraw() {
//!     window.request_redraw();
//! }
//! ```
//! 
//! See also [Ui::ui_waker()] to wake up the loop from another thread.
//!
//! If you do want the loop to continuously, but only some nodes are visible, you can use [Node::sense_time()].
//! As long as a time-sensitive node is visible, [Ui::should_request_redraw()] will continue returning `true` every frame. See `manual_animation.rs` example.

use std::time::{Duration, Instant};

use keru::*;

pub struct State {
    pub banner_last_shown: Instant,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    println!("Tick {:?}", Instant::now());
    #[node_key] const SHOW: NodeKey;
    let button = BUTTON.static_text("Show Banner").key(SHOW);
    let label = LABEL
        .static_text("Showing up for a while")
        .position_y(Pos::End)
        .slide_from_bottom();

    ui.add(button);
    if state.banner_last_shown.elapsed() < Duration::from_secs(1) {
        ui.add(label);
    }

    if ui.is_clicked(SHOW) {
        state.banner_last_shown = Instant::now();
        ui.schedule_wakeup(Duration::from_millis(1000));
    }
}

fn main() {
    let state = State { banner_last_shown: Instant::now() - Duration::from_secs(90000), };
    example_window_loop::run_example_loop(state, update_ui);
}
