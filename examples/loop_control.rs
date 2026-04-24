//! This example shows some of the ways in which we can indirectly control the winit loop and whether our Ui declaration code in `update_ui()` is executed or not.
//! 
//! This assumes that the winit loop is being controlled according to [Ui::should_request_redraw()], like in `run_example_loop` and in the `window_loop.rs` example:
//! ```
//! if state.ui.should_request_redraw() {
//!     window.request_redraw();
//! }
//! ```
//! 
//! The banner uses [Ui::schedule_wakeup()] to make the loop wake up when it has to go away, without having the loop tick continuously.
//!
//! Similarly, the async function uses a [Ui::ui_waker()] to wake the loop from another thread when the background task completes. The loop can sleep while the task is ongoing.
//!
//! On the other hand, the manually-animated node *wants* the loop to tick continuously, so that it can update its size on every frame.
//! So, it uses [Node::sense_time()] to tell the Ui to keep the winit loop awake and to rerunning the update code as long as it is visible.
//!
//! (Note that this isn't needed for "built-in" animations like the banner's [Node::slide_from_bottom()], 
//!   which can advance automatically when rerendering even without rerunning the update code.)

use std::task::Poll;
use std::thread;
use std::time::{Duration, Instant};

use keru::thread_future::*;

use keru::*;
use keru::example_window_loop::*;

pub struct State {
    pub banner_last_shown: Instant,
    pub async_task: Option<ThreadFuture<String>>,
    pub start: Instant,
    pub show_anim: bool,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    println!("Tick {:?}", Instant::now());

    #[node_key] const SHOW_BANNER: NodeKey;
    #[node_key] const TOGGLE_ANIM: NodeKey;
    #[node_key] const ANIM_NODE: NodeKey;
    #[node_key] const START_ASYNC: NodeKey;
    #[node_key] const RESET_ASYNC: NodeKey;

    let banner_duration = Duration::from_millis(1000);

    let t = state.start.elapsed().as_secs_f32();
    let sx = (t * 1.3).sin() * 0.15 + 0.25;
    let sy = (t * 1.9 + 1.5).sin() * 30.0 + 80.0;

    let toggle_label = if state.show_anim { "Hide Animation" } else { "Show Animation" };
    let banner = LABEL
        .static_text("Showing up for a while")
        .position_y(Pos::End)
        .slide_from_bottom();

    let animated_node = LABEL
        .key(ANIM_NODE)
        .anchor_symm(Anchor::Center)
        .static_text("Animated node")
        .size_x(Size::Frac(sx))
        .size_y(Size::Pixels(sy))
        .sense_time(true);
    
    let uiwaker = ui.ui_waker();

    ui.add(V_STACK).nest(|| {
        ui.add(BUTTON.static_text("Show Banner").key(SHOW_BANNER));
        ui.add(BUTTON.static_text(toggle_label).key(TOGGLE_ANIM));

        match &mut state.async_task {
            None => {
                ui.add(BUTTON.static_text("Start Async Function").key(START_ASYNC));
            }
            Some(future) => match future.poll() {
                Poll::Pending => { ui.add(LABEL.static_text("Working...")); }
                Poll::Ready(msg) => {
                    ui.add(BUTTON.static_text("Reset Async Value").key(RESET_ASYNC));
                    ui.add(LABEL.text(&msg));
                }
            },
        }

        if state.show_anim {
            ui.add(animated_node);
        }
    });

    if ui.is_clicked(START_ASYNC) {
        let slow_function = || {
            thread::sleep(Duration::from_millis(1200));
            String::from("Background work done!")
        };
        state.async_task = Some(run_in_background(slow_function, move || uiwaker.set_update_needed()));
    }
    if ui.is_clicked(RESET_ASYNC) {
        state.async_task = None;
    }

    if state.banner_last_shown.elapsed() < banner_duration {
        ui.add(banner);
    }

    if ui.is_clicked(SHOW_BANNER) {
        state.banner_last_shown = Instant::now();
        ui.schedule_wakeup(banner_duration);
    }

    if ui.is_clicked(TOGGLE_ANIM) {
        state.show_anim = !state.show_anim;
    }
}

fn main() {
    let state = State {
        banner_last_shown: Instant::now() - Duration::from_secs(90000),
        async_task: None,
        start: Instant::now(),
        show_anim: false,
    };
    run_example_loop(state, update_ui);
}
