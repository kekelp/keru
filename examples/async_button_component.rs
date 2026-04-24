//! Example using an [AsyncButton] [Component].
//! 
//! The component draws a button that starts a background function when clicked. 
//! 
//! When the function completes, it gives us an owned String, and resets the button state so that the computation can start again.
//! 
//! This is different than the async_thread example, but it's what you want for something like running a system file picker.
//! 
//! Note that AsyncButton is a stateful component: it manages its own thread communication channel internally without us having to make space for it inside State.

use std::path::Path;
use std::task::Poll;
use std::thread;
use std::time::Duration;

use keru::*;
use keru::example_window_loop::*;

#[derive(Default)]
struct State {
    file: Option<String>,
}

fn update_ui(state: &mut State, ui: &mut Ui) {

    let path = "src/lib.rs";
    let load_file_slowly = move ||  {
        thread::sleep(Duration::from_millis(800));
        let cargo_dir = Path::new(env!("CARGO_MANIFEST_DIR")).canonicalize().unwrap();
        let contents = std::fs::read_to_string(cargo_dir.join(path)).unwrap();
        return contents;
    };

    ui.add(V_STACK).nest(|| {
        let result = ui.add_component(AsyncButton::new(load_file_slowly, "Click to load the file", "Loading slowly..."));
        
        if let Poll::Ready(file) = result {
            state.file = Some(file);
        }

        if let Some(file) = &state.file {
            ui.add(TEXT.text(&file[0..200]));
            ui.add(TEXT.text("..."));
        }
    });

}

fn main() {
    basic_env_logger_init();
    run_example_loop(State::default(), update_ui);
}
