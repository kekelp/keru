use keru::*;
use keru::example_window_loop::*;

#[derive(Default)]
pub struct State {
    pub count: i32,
    pub show: bool,
}

impl State {
    fn update_ui(&mut self, ui: &mut Ui) {
        #[node_key] const INCREASE: NodeKey;
        #[node_key] const DECREASE: NodeKey;
        #[node_key] const SHOW: NodeKey;
        
        if ui.is_clicked(SHOW) {
            self.show = !self.show;
        }
        if ui.is_clicked(INCREASE) {
            self.count += 1;
        }
        if ui.is_clicked(DECREASE) {
            self.count -= 1;
        }

        let red = 0.1 * self.count as f32;
        let count_color = Color::rgba_f(red, 0.10196, 0.59608, 0.80392);

        let show_button_text = match self.show {
            true => "Hide Counter",
            false => "Show Counter",
        };

        let v_stack = V_STACK
            .clip_children_y(true)
            .slide();

        let increase_button = BUTTON
            .color(count_color)
            .text("Increase")
            .key(INCREASE);

        let show_button = BUTTON
            .color(Color::RED)
            .text(show_button_text)
            .slide_when_moving()
            .key(SHOW);

        let decrease_button = BUTTON
            .text("Decrease")
            .key(DECREASE);

        // In a real program, you should use a frame arena to avoid useless allocations like these.
        let fmt_count = self.count.to_string();
        let count = LABEL.text(&fmt_count).slide_when_moving();

        ui.add(v_stack).nest(|| {
            ui.add(show_button);
            if self.show {
                ui.add(increase_button);
                ui.add(count);
                ui.add(decrease_button);
            }
        });

    }

}

fn main() {
    // basic_env_logger_init();
    let mut state = State::default();
    state.show = true;
    run_example_loop(state, State::update_ui);
}
