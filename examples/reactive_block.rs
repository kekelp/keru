use keru::*;
use keru::example_window_loop::*;

#[derive(Default)]
pub struct State {
    pub count: Observer<i32>,
}

pub trait CustomComponents {
    fn counter(&mut self, count: &mut Observer<i32>);
}

impl CustomComponents for Ui {
    fn counter(&mut self, count: &mut Observer<i32>) {
        #[node_key] const INCREASE: NodeKey;
        #[node_key] const DECREASE: NodeKey;
        #[node_key] const SHOW: NodeKey;

        self.subtree().start(|| {

            let changed = self.check_changes(count);
            reactive(changed, || {

                if is_in_skipped_reactive_block() {
                    log::warn!("Reactive block: dependencies unchanged");
                } else {
                    log::warn!("Reactive block: dependencies changed");
                }

                let red = 0.1 * (**count as f32);
                let count_color = Color::rgba_f(red, 0.10196, 0.59608, 0.80392);
                let increase_button = BUTTON
                    .color(count_color)
                    .static_text("Increase")
                    .key(INCREASE);

                self.add(increase_button);
                
                if self.is_clicked(INCREASE) {
                    *count += 1;
                }


            });


        });
    }
}


impl ExampleLoop for State {
    fn update_ui(&mut self, ui: &mut Ui) {
        
        ui.counter(&mut self.count);

    }
}

fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Warn)
        .filter_module("keru::reactive", log::LevelFilter::Trace)
        .init();
    
    let state = State::default();
    run_example_loop(state);
}
