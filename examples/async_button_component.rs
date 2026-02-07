use std::path::Path;
use std::task::Poll;
use std::thread;
use std::time::Duration;

use keru::*;
use keru::example_window_loop::*;

fn load_file_slowly() -> String {
    thread::sleep(Duration::from_millis(800));
    let cargo_dir = Path::new(env!("CARGO_MANIFEST_DIR")).canonicalize().unwrap();
    std::fs::read_to_string(cargo_dir.join("src/lib.rs")).unwrap()
}

fn update_ui(_state: &mut (), ui: &mut Ui) {
    let load_fn: fn() -> String = load_file_slowly;

    #[component_key] const FILE_LOADER: ComponentKey<AsyncButton<String>>;

    let loader = AsyncButton::new(load_fn, "Click to load the file", "Loading...").key(FILE_LOADER);

    ui.add(V_SCROLL_STACK).nest(|| {

        let result = ui.add_component(loader);

        // Show the result when ready
        if let Poll::Ready(file) = result {
            ui.add(V_SCROLL_STACK.size_symm(Size::Frac(0.75))).nest(|| {
                ui.add(LABEL.text(&file));
            });
        }
    
        // // Alternative: access state via component_state_mut
        // if let Some(state) = ui.component_state_mut(FILE_LOADER) {
        //     if let Some(future) = &state.future {
        //         if let Poll::Ready(file) = future.poll() {
        //             ui.add(V_SCROLL_STACK.size_symm(Size::Frac(0.75))).nest(|| {
        //                 ui.add(LABEL.text(&file));
        //             });
        //         }
        //     }
        // }

    });

}

fn main() {
    basic_env_logger_init();
    run_example_loop((), update_ui);
}
