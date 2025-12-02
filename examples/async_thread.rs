use std::io::Read;
use std::sync::Arc;
use std::task::Poll;
use std::thread;
use std::time::Duration;

use keru::*;
use keru::example_window_loop::*;
use keru::thread_future::*;
use winit::window::Window;

pub struct State {
    pub file: Option<ThreadFuture<String>>,
}

fn load_file_slowly() -> String {
    thread::sleep(Duration::from_millis(800));
    let mut file = std::fs::File::open("src/lib.rs").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    return contents;
}

// This example needs access to the window to be able to wake up the event loop. 
fn update_ui(state: &mut State, ui: &mut Ui, window: Arc<Window>) {

    // Setup a waker callback that can both wake up the winit event loop and tell the Ui that an update is needed.
    // Calling `set_update_needed()`` it will cause `Ui::should_update()` to return `true` on the next call, 
    // which is the method used in the window loop to decide whether to rerun the ui logic.
    let uiwaker = ui.ui_waker();
    let waker = move || {
        window.request_redraw();
        uiwaker.set_update_needed();
    };

    match &mut state.file {
        None => {
            #[node_key] pub const LOAD: NodeKey;
            ui.add(BUTTON.key(LOAD).static_text("Click to load the file"));
            
            if ui.is_clicked(LOAD) {
                state.file = Some(run_in_background(load_file_slowly, waker));
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
    run_example_loop_with_window(state, update_ui);
}
