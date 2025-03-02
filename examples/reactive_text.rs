use keru::*;
use keru::example_window_loop::*;

// This example shows how by wrapping `text` in an Observer, the ui is able to avoid any unnecessary updates to the text layout, without hashing.

#[derive(Default)]
pub struct State {
    pub text: Observer<String>,
}

pub trait CustomComponents {
    fn string_push(&mut self, text: &mut Observer<String>);
}

impl CustomComponents for Ui {
    fn string_push(&mut self, text: &mut Observer<String>) {
        self.subtree().start(|| {
            #[node_key] const INCREASE: NodeKey;
            #[node_key] const RESET: NodeKey;

            let increase_button = BUTTON
                .color(Color::RED)
                .static_text("Push str")
                .key(INCREASE);
            let clear_button = BUTTON
                .color(Color::RED)
                .static_text("Clear")
                .key(RESET);

            self.v_stack().nest(|| {
                self.h_stack().nest(|| {
                    self.add(increase_button);
                    self.add(clear_button);
                });
                self.label(text);
            });

            if self.is_clicked(INCREASE) {
                text.push_str(" etc");
            }
            if self.is_clicked(RESET) {
                *text = Observer::new("Etc".to_string());
            }
        });    
    }
}


impl ExampleLoop for State {
    fn update_ui(&mut self, ui: &mut Ui) {
        ui.h_stack().nest(|| {
            ui.string_push(&mut self.text);
        });
    }
}

fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Warn)
        .filter_module("keru::node_params", log::LevelFilter::Trace)
        .init();

    let state = State {
        text: Observer::new("Etc".to_string()),
    };
    
    run_example_loop(state);
}
