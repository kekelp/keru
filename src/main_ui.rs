// crate::* is needed to fix some crap with macros: https://github.com/rust-lang/rust/pull/52234#issuecomment-894851497
// when ui will be in its own crate, this won't happen anymore
use crate::*;
use crate::ui::*;
use crate::ui::Axis::*;
use view_derive::{add_anon, derive_view};
use crate::ui::Position::*;

impl State {
    pub fn update_ui(&mut self) {
        let ui = &mut self.ui;
        ui.begin_tree();

        #[derive_view(MARGIN.size_y(0.95).size_x(1.0).position_x(Position::Center))]
        pub struct Margin;

        add!(ui, Margin, {

            // #[derive_view(V_STACK.size_x(0.3).position_x(Position::End))]
            // pub struct SideBar;
            // add!(ui, SideBar, {

            // let SIDEBAR = V_STACK.size_x(0.3).position_x(Position::End);
            const SIDEBAR: NodeParams = V_STACK.size_x(0.3).position_x(Position::End);
            add_anon!(ui, SIDEBAR, {

                let mut color = ui.add(PaintColor).get_text();

                if let Some(color) = &mut color {
                    color.make_ascii_lowercase();
                    match color.as_str() {
                        "blue" => {
                            self.canvas.paint_color = PixelColorF32::BLUE;
                        },
                        "red" => {
                            self.canvas.paint_color = PixelColorF32::RED;
                        },
                        "green" => {
                            self.canvas.paint_color = PixelColorF32::GREEN;
                        },
                        _ => {}
                    }
                } 

            });
        });


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



#[derive_view(H_STACK.color(Color::BLUE).size_x(0.5).position_x(Position::Start))]
pub struct CenterRow;

#[derive_view(BUTTON.text("Increase").color(Color::GREEN))]
pub struct IncreaseButton;

#[derive_view(BUTTON.text("Decrease").color(Color::RED))]
pub struct DecreaseButton;

#[derive_view(BUTTON.text("Show Counter").color(Color::rgba(0.5, 0.1, 0.7, 0.7)))]
pub struct ShowCounterButton;

#[derive_view(LABEL)]
pub struct CountLabel;

#[derive_view(
    H_STACK
    .size_y(0.95)
    .size_x(0.8)
    .stack(Y, Arrange::End)
    .color(Color::BLUE)
)]
pub struct CommandLineRow;

#[derive_view(TEXT_INPUT.text("Color").size_y(0.2).position_y(Start))]
pub struct PaintColor;

#[derive_view(TEXT_INPUT.text("RERER"))]
pub struct CommandLine;

#[derive_view(LABEL)]
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
