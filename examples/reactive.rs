use keru::*;
use keru::example_window_loop::*;

#[derive(Default)]
pub struct State {
    pub count_1: Observer<i32>,
    pub show_1: Observer<bool>,

    pub count_2: Observer<i32>,
    pub show_2: Observer<bool>,
}

fn count_color(count: i32) -> Color {
    let red = 0.1 * count as f32;
    return Color::rgba_f(red, 0.10196, 0.59608, 0.80392);
}

pub trait CustomWidgets {
    fn counter(&mut self, count: &mut Observer<i32>, show: &mut Observer<bool>, number: i32);
}

impl CustomWidgets for Ui {
    fn counter(&mut self, count: &mut Observer<i32>, show: &mut Observer<bool>, number: i32) {
        
        subtree(|| {

            let changed = count.changed() || show.changed();
            reactive(changed, || {

                if can_skip() {
                    log::warn!("Counter #{number} is soft-skipped. `ui` methods will be able to skip most expensive operations");
                } else {
                    log::warn!("Counter #{number} updated");
                }

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

                self.add(INCREASE)
                    .params(BUTTON)
                    .color(count_color(**count))
                    .static_text("Increase");

                self.add(SHOW)
                    .params(BUTTON)
                    .color(Color::RED)
                    .static_text(show_button_text);

                self.add(DECREASE)
                    .params(BUTTON)
                    .static_text("Decrease");

                self.v_stack().nest(|| {
                    if **show {
                        self.place(INCREASE);
                        self.label(*count);
                        self.place(DECREASE);
                    }
                    self.place(SHOW);
                });
            });    
        });
    }
}


impl ExampleLoop for State {
    fn update_ui(&mut self, ui: &mut Ui) {

        ui.h_stack().nest(|| {
            ui.counter(&mut self.count_1, &mut self.show_1, 1);
            ui.counter(&mut self.count_2, &mut self.show_2, 2);
        });

    }
}

fn main() {
    basic_env_logger_init();
    let state = State::default();
    run_example_loop(state);
}
