use std::path::Path;
use std::task::Poll;
use std::thread;
use std::time::Duration;

use keru::*;
use keru::example_window_loop::*;
use keru::thread_future::*;

pub struct State {
    pub file: Option<ThreadFuture<String>>,
}

fn load_file_slowly() -> String {
    thread::sleep(Duration::from_millis(800));
    let cargo_dir = Path::new(env!("CARGO_MANIFEST_DIR")).canonicalize().unwrap();
    let contents = std::fs::read_to_string(cargo_dir.join("src/lib.rs")).unwrap();
    return contents;
}

// This example needs access to the window to be able to wake up the event loop. 
fn update_ui(state: &mut State, ui: &mut Ui) {

    // Setup a waker callback that can both wake up the winit event loop and tell the Ui that an update is needed.
    // Calling `set_update_needed()`` it will cause `Ui::should_update()` to return `true` on the next call, 
    // which is the method used in the window loop to decide whether to rerun the ui logic.
    let uiwaker = ui.ui_waker();


    match &mut state.file {
        None => {
            let load = BUTTON.static_text("Click to load the file");
            if ui.add(load).is_clicked(ui) {
                state.file = Some(run_in_background(load_file_slowly, move || uiwaker.set_update_needed()));
            }
        }
        Some(future) => {
            match future.poll() {
                Poll::Pending => {
                    ui.add(LABEL.static_text("Loading..."));
                },
                Poll::Ready(file) => {
                    ui.add(V_SCROLL_STACK.size_symm(Size::Frac(0.75))).nest(|| {
                        ui.add(LABEL.text(&file));
                    });
                }
            };
        }
    }
}

fn main() {
    basic_env_logger_init();
    let state = State {
        file: None,
    };
    run_example_loop(state, update_ui);
}
