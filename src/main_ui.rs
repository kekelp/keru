// this is needed to fix some crap with macros: https://github.com/rust-lang/rust/pull/52234#issuecomment-894851497
// when ui will be in its own crate, this won't happen anymore
use crate::*;
use crate::ui::*;
use crate::ui::Axis::Y;
use view_derive::derive_view;


impl State {
    pub fn update_ui(&mut self) {
        let ui = &mut self.ui;
        ui.begin_tree();

        useless_counter(ui, &mut self.counter_state);

        ui.finish_tree();

        if ui.is_clicked(IncreaseButton) {
            self.counter_state.count += 1;
        }

        if ui.is_clicked(DecreaseButton) {
            self.counter_state.count -= 1;
        }

        if ui.is_clicked(ShowCounterButton) {
            self.counter_state.counter_mode = !self.counter_state.counter_mode;
        }

    }
}


#[derive_view(NodeParams::H_STACK.color(Color::BLUE).size_x(0.5).position_x(Position::Start))]
pub struct CenterRow;

#[derive_view(NodeParams::BUTTON.text("Increase").color(Color::GREEN))]
pub struct IncreaseButton;

#[derive_view(NodeParams::BUTTON.text("Decrease").color(Color::RED))]
pub struct DecreaseButton;

#[derive_view(NodeParams::BUTTON.text("Show Counter").color(Color::rgba(0.5, 0.1, 0.7, 0.7)))]
pub struct ShowCounterButton;

#[derive_view(NodeParams::LABEL)]
pub struct CountLabel;

#[derive_view(
    NodeParams::H_STACK
    .size_y(0.95)
    .size_x(0.8)
    .stack(Y, Arrange::End)
    .color(Color::BLUE)
)]
pub struct CommandLineRow;

#[derive_view(NodeParams::TEXT_INPUT.text("scalar"))]
pub struct ScalarInput;

#[derive_view(NodeParams::TEXT_INPUT.text("e12"))]
pub struct E12Input;

#[derive_view(NodeParams::TEXT_INPUT.text("RERER"))]
pub struct CommandLine;

#[derive_view(NodeParams::LABEL)]
pub struct Label234;

#[allow(dead_code)]
pub fn useless_counter(ui: &mut Ui, counter_state: &mut CounterState) {
    margin!(ui, {
        h_stack!(ui, CenterRow, {
            v_stack!(ui, {
                if counter_state.counter_mode {
                    let new_color = count_color(counter_state.count);
                    ui.add(IncreaseButton).set_color(new_color);

                    let count = &counter_state.count.to_string();
                    ui.add(CountLabel).set_text(count);

                    ui.add(DecreaseButton);
                }
            });

            v_stack!(ui, {
                let text = match counter_state.counter_mode {
                    true => "Hide counter",
                    false => "Show counter",
                };
                ui.add(ShowCounterButton).set_text(text);
            });
        });
    });
}

pub struct CounterState {
    pub count: i32,
    pub counter_mode: bool,
}
impl Default for CounterState {
    fn default() -> Self {
        Self::new()
    }
}

impl CounterState {
    pub fn new() -> Self {
        return CounterState {
            count: 0,
            counter_mode: true,
        };
    }
}
pub fn count_color(count: i32) -> Color {
    let red = 0.1 * (count as f32);
    return Color::rgba(red, 0.1, 0.2, 0.8);
}
