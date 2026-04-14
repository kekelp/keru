/// This is an example showing how to use `readd_branch` for "reactivity".
/// Note that this feature is experimental and it doesn't fit in the library all that well. In particular, you can't use it together with Components.
///
/// Normally, a GUI program has a state that can evolve in arbitrary ways. But once the state is set, it's usually not hard to go from the state to the GUI representation. In Keru, you just have to call a few `ui.add` functions, which are very cheap, and then the library can take it from there and do diffing, incremental relayouts, or incremental updates to the render data, if it wants to.
/// As long as this is the case, "reactivity" isn't really anything that we need to worry about, and it's not worth complicating the programming model for it.
///
/// But what if some reason we had to do some really expensive computation to convert the program state into something that can be shown in the GUI?
/// For example, imagine that we have a counter where our state is a `i32`, with values `0`, `1`, `2`, and imagine that converting it to the strings "Zero", "One", "Two" that we want to display was an expensive process worth worrying about.
///
/// The easy solution would be to just store the converted form in our state, and update when it changes. That would probably work great, so maybe we still don't need reactivity. But what if we were really convinced that this reactivity thing was the future? In that case, we might want a way to skip running that part of the GUI rebuilding code completely, except on the frames where the underlying `i32` changed.
///
/// To be precise, if it was so expensive that it makes the problem actually miss frames, that would be a separate issue: we'd still miss the frame whenever the `i32` did change. We'd need a real solution like computing it in a separate thread. (See the async_thread.rs example).
/// 
/// Also, if computing that expensive value was the ONLY thing that our program does, it wouldn't be a problem either. If the window loop is set correctly, it doesn't rerun the GUI rebuild code at all unless the `Ui` received an input that it cares about. If it only cares about the Increase button being clicked, that means that it's already only running it when the value actually needs to be recomputed, and there'd be nothing to skip.
/// 
/// What we're talking about here is the very specific case where we have a complex GUI with many individual parts, it's common for the user to interact with simpler parts of the GUI in ways that don't change the state that the expensive part depends on, and the cost of rerunning the expensive parts when not needed add up in terms of CPU usage or power consumption.
/// In this case it might finally make sense to think about rerunning the builder code for the simple part but skipping the code for the expensive part.
/// 
/// (But remember that it's never too late to just cache the result of the expensive calculation, stop thinking about reactivity, and move on to more interesting things.)
/// 
/// If we really decided that we want to do this, we can do it using just one function: `ui.readd_branch()`, which turns off the usual "clear the tree and redeclare it" mechanism and instead keeps the whole branch the way it was in the previous frame.
///
/// To know when to rebuild and when to skip, we need to do some basic change-tracking ourselves. In this example, we use an `Observer`, a simple wrapper struct that does it automatically using DerefMut. But we could also do it manually with a basic boolean flag.
///
/// Note that if we put the code that increases `reactive_count` inside the `if changed {}` block, it would be skipped as well! because of this, we lose some of the usual flexibility and we are forced to cleanly separate the GUI rebuild code from the effects.
///
/// This also means that we can't use Components and `readd_branch()` at the same time. To write a Component, you have to mix their GUI code and their effects into a single `add_to_ui` function, which makes it impossible to split it. To fix this, we'd have to change the Component trait and force everyone to split their code into two separate functions. Right now, I don't think it's worth it to complicate Component for this.

use std::thread;
use std::time::Duration;

use keru::*;
use keru::example_window_loop::*;

#[derive(Default)]
pub struct State {
    pub count: i32,
    pub reactive_count: Observer<i32>,
}

const NUMBERS: [&str; 8] = [
    "Zero",
    "One",
    "Two",
    "Three",
    "Four",
    "Five",
    "Six",
    "Too big...",
];

fn do_a_slow_calculation_to_convert_our_state_into_something_that_we_can_show_in_the_gui(count: &i32) -> &'static str {
    thread::sleep(Duration::from_secs(1));
    let index = (count.abs() as usize).min(5);
    return NUMBERS[index];
}

impl State {
    fn update_ui(&mut self, ui: &mut Ui) {
        #[node_key] const REACTIVE_ROOT: NodeKey;

        #[node_key] const INCREASE: NodeKey;
        let increase_button = BUTTON
            .static_text(&"Increase")
            .key(INCREASE);

        #[node_key] const INCREASE_2: NodeKey;
        let increase_button_2 = BUTTON
            .static_text(&"Increase")
            .key(INCREASE_2);

        if ui.is_clicked(INCREASE_2) {
            self.count += 1;
        }

        if ui.is_clicked(INCREASE) {
            self.reactive_count += 1;
        }

        ui.add(H_STACK).nest(|| {

            ui.add(V_STACK).nest(|| {
                ui.add(LABEL.static_text("Regular counter"));
                ui.add(LABEL.text(&self.count.to_string()));
                ui.add(increase_button_2);
            });

            let changed = ui.check_changes(&mut self.reactive_count);
            if changed {
                ui.add(V_STACK.key(REACTIVE_ROOT)).nest(|| {
                    ui.add(LABEL.static_text("Slow reactive counter"));
                    let text = do_a_slow_calculation_to_convert_our_state_into_something_that_we_can_show_in_the_gui(&self.reactive_count);
                    ui.add(LABEL.text(&text));
                    ui.add(increase_button);
                });
            } else {
                ui.readd_branch(REACTIVE_ROOT);
            }
            
        });
    }
}

fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Warn)
        .filter_module("keru::reactive", log::LevelFilter::Trace)
        .init();

    let state = State {
        count: 0,
        reactive_count: Observer::new(0),
    };
    run_example_loop(state, State::update_ui);
}
