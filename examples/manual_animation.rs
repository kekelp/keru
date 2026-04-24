//! A manual animation example using [Sense::TIME].
//!
//! A node with [`Sense::TIME`] forces the event loop to keep calling `update()` every frame
//! while that node is visible, so you can drive animations using elapsed time.
//! This example animates a bar width back and forth using a sine wave.

use std::time::Instant;

use keru::*;
use keru::example_window_loop::*;

pub struct State {
    pub start: Instant,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const ANIM_BAR: NodeKey;

    let t = state.start.elapsed().as_secs_f32();
    let fraction = (t.sin() * 0.5 + 0.5) as f32;

    let bar = LABEL
        .key(ANIM_BAR)
        .static_text("animating")
        .size_x(Size::Frac(fraction))
        .size_y(Size::Pixels(60.0))
        .sense_time(true);

    ui.add(bar);
}

fn main() {
    let state = State { start: Instant::now() };
    run_example_loop(state, update_ui);
}
