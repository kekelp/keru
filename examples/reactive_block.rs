use keru::*;
use keru::example_window_loop::*;

#[derive(Default)]
pub struct State {
    pub count: Observer<i32>,
    pub useless_variable: Observer<i32>,
}

pub trait CustomComponents {
    fn counter(&mut self, count: &mut Observer<i32>, useless_variable: &mut Observer<i32>);
}

impl CustomComponents for Ui {
    fn counter(&mut self, count: &mut Observer<i32>, _useless_variable: &mut Observer<i32>) {
        #[node_key] const INCREASE: NodeKey;
        #[node_key] const DECREASE: NodeKey;

        self.subtree_old().start(|| {
            let changed = self.check_changes(count);
            // if we uncomment these two lines and comment the two below, the [`Ui`] will have a wrong idea of which variables the ui code depends on, and it will miss updates to `count`.
            // However, when running in debug mode, the [`Ui`] still checks for differences, so it can detect this mistake and print some error messages.
            // This does mean that any performance gains from the reactive block apply to release mode only: in debug mode, the [`Ui`] is still hashing and diffing everything.

            // let changed_wrong = self.check_changes(_useless_variable);
            // reactive(changed_wrong, || {
            self.reactive(changed).start(|| {

                let red = 0.1 * (**count as f32);
                let increase_color = Color::rgba_f(red, 0.10196, 0.59608, 0.80392);
                let increase_button = BUTTON
                    .color(increase_color)
                    .static_text(&"Increase")
                    .key(INCREASE);

                let green = 0.1 * ((10 - **count) as f32);
                let decrease_color = Color::rgba_f(0.2345, green, 0.59608, 0.80392);
                let decrease_button = BUTTON
                    .color(decrease_color)
                    .static_text(&"Decrease")
                    .key(DECREASE);
                
                self.h_stack().nest(|| {
                    self.add(decrease_button);
                    // When using a wrong state declaration in reactive(), this count label will still update, and you won't see any error messages for it. This is because functions taking an `Observer<text>` can use the Observer directly, so they always skip unneeded updates even without a reactive block.
                    self.label(&count.to_string());
                    self.add(increase_button);
                });
                
                if self.is_clicked(INCREASE) {
                    *count += 1;
                }
                if self.is_clicked(DECREASE) {
                    *count -= 1;
                }
            });


        });
    }
}


impl State {
    fn update_ui(&mut self, ui: &mut Ui) {
        
        ui.counter(&mut self.count, &mut self.useless_variable);

    }
}

fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Warn)
        .filter_module("keru::reactive", log::LevelFilter::Trace)
        .init();
    
    let state = State::default();
    run_example_loop(state, State::update_ui);
}
