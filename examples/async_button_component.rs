/// Example using an AsyncButton component.
/// 
/// The component draws a button that starts a background function when clicked. 
/// 
/// When the function completes, it gives us an owned String, and resets the button state so that the computation can start again.
/// 
/// This is different than the async_thread example, but it's what you want for something like running a system file picker.

use std::path::Path;
use std::task::Poll;
use std::thread;
use std::time::Duration;

use keru::*;
use keru::example_window_loop::*;

fn update_ui(_state: &mut (), ui: &mut Ui) {

    let path = "src/lib.rs";
    let load_file_slowly = move ||  {
        thread::sleep(Duration::from_millis(800));
        let cargo_dir = Path::new(env!("CARGO_MANIFEST_DIR")).canonicalize().unwrap();
        let contents = std::fs::read_to_string(cargo_dir.join(path)).unwrap();
        return contents;
    };

    let loader = AsyncButton::new(load_file_slowly, "Click to load the file", "Loading...");

    let result = ui.add_component(loader);

    if let Poll::Ready(file) = result {
        dbg!(file); // We can move it manually into a state variable, or do something else with it
    }

}

fn main() {
    basic_env_logger_init();
    run_example_loop((), update_ui);
}
