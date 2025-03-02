use keru::*;
use keru::example_window_loop::*;

#[derive(Default)]
pub struct State {
    pub count_1: Observer<i32>,
    pub show_1: Observer<bool>,

    pub count_2: Observer<i32>,
    pub show_2: Observer<bool>,
}

pub trait CustomComponents {
    fn counter(&mut self, count: &mut Observer<i32>, show: &mut Observer<bool>);
}

impl CustomComponents for Ui {
    fn counter(&mut self, count: &mut Observer<i32>, show: &mut Observer<bool>) {
        
        self.subtree().start(|| {

            let changed = self.check_changes(count) || self.check_changes(show);
            reactive(changed, || {

                #[node_key] const INCREASE: NodeKey;
                #[node_key] const DECREASE: NodeKey;
                #[node_key] const SHOW: NodeKey;

                if self.is_clicked(SHOW) {
                    *show = ! *show;
                }
                if self.is_clicked(INCREASE) {
                    *count += 1;
                }
                if self.is_clicked(DECREASE) {
                    *count -= 1;
                }

                let show_button_text = match **show {
                    true => "Hide Counter",
                    false => "Show Counter",
                };

                let red = 0.1 * (**count as f32);
                let count_color = Color::rgba_f(red, 0.10196, 0.59608, 0.80392);
                let increase_button = BUTTON
                    .color(count_color)
                    .static_text("Increase")
                    .key(INCREASE);
    
                let show_button = BUTTON
                    .color(Color::RED)
                    .static_text(show_button_text)
                    .key(SHOW);
        
                let decrease_button = BUTTON
                    .static_text("Decrease")
                    .key(DECREASE);


                let label = LABEL.text(count);

                self.v_stack().nest(|| {
                    if **show {
                        self.add(increase_button);
                        self.add(label);
                        self.add(decrease_button);
                    }
                    self.add(show_button);
                });
            });    
        });
    }
}


impl ExampleLoop for State {
    fn update_ui(&mut self, ui: &mut Ui) {
        
        ui.h_stack().nest(|| {
            ui.counter(&mut self.count_1, &mut self.show_1);
            ui.counter(&mut self.count_2, &mut self.show_2);
        });

    }
}

fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Warn)
        .filter_module("keru::node_params", log::LevelFilter::Trace)
        .init();
    
    let state = State::default();
    run_example_loop(state);
}
