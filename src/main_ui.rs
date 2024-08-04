// crate::* is needed to fix some crap with macros: https://github.com/rust-lang/rust/pull/52234#issuecomment-894851497
// when ui will be in its own crate, this won't happen anymore
use crate::node_params::*;
use crate::ui::Position::*;
use crate::ui::*;
use crate::*;
use glyphon::{cosmic_text::Align, Attrs, Color as GlyphonColor, Family, Weight};
use view_derive::node_key;

impl State {
    pub fn update_ui(&mut self) {
        tree!(self.ui, {

            #[node_key(BUTTON.text("Increase").color(Color::GREEN))]
            pub const BUTTON_A: NodeKey;

            h_stack!(self.ui, {
                add!(self.ui, BUTTON_A);
                add!(self.ui, BUTTON_A);
                add!(self.ui, BUTTON_A);
            });


            // margin!(self.ui, {
            //     #[node_key(V_STACK.size_x(0.3).position_x(Position::End))]
            //     const SIDEBAR: Nodekey;
            //     add!(self.ui, SIDEBAR, {
            //         // todo: function for doing get_text from other places
            //         let mut color = add!(self.ui, PAINT_COLOR).get_text();

            //         if let Some(color) = &mut color {
            //             color.make_ascii_lowercase();
            //             match color.as_str() {
            //                 "blue" => {
            //                     self.canvas.paint_color = PixelColorF32::BLUE;
            //                 }
            //                 "red" => {
            //                     self.canvas.paint_color = PixelColorF32::RED;
            //                 }
            //                 "green" => {
            //                     self.canvas.paint_color = PixelColorF32::GREEN;
            //                 }
            //                 _ => {}
            //             }
            //         }

            //         let pixel_info = self.canvas.pixel_info();
            //         self.add_pixel_info_ui(&pixel_info);
            //         self.add_pixel_info_ui(&pixel_info);
            //         self.add_pixel_info_ui(&pixel_info);

            //         self.add_twin_thing_ui();
            //         self.add_twin_thing_ui();
            //         self.add_twin_thing_ui();
            //     });
            // });

            // self.add_counter_ui();
        });

        // effects
        // self.counter_on_click();
    }
}

#[node_key(TEXT_INPUT.text("Color").size_y(0.2).position_y(Start))]
pub const PAINT_COLOR: NodeKey;

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

    #[node_key(BUTTON.text("Increase").color(Color::GREEN))]
    pub const INCREASE_BUTTON: NodeKey;

    #[node_key(BUTTON.text("Decrease").color(Color::RED))]
    pub const DECREASE_BUTTON: NodeKey;

    #[node_key(BUTTON.text("Show Counter").color(Color::rgba(0.5, 0.1, 0.7, 0.4)))]
    pub const SHOW_COUNTER_BUTTON: NodeKey;

    #[node_key(LABEL)]
    pub const COUNT_LABEL: NodeKey;
}
pub fn count_color(count: i32) -> Color {
    let red = 0.1 * (count as f32);
    return Color::rgba(red, 0.1, 0.2, 0.8);
}


impl State {
    pub fn add_twin_thing_ui(&mut self) {
        #[node_key(PANEL.size_y(0.5).position_x(Position::Start))]
        const PIXEL_PANEL: Nodekey;
        add!(self.ui, PIXEL_PANEL, {

            v_stack!(self.ui, {
                for _ in 0..3 {
                    text!(self.ui, "Identical");
                }
            });
        });
    }
}

impl State {

    pub fn add_counter_ui(&mut self) {
        margin!(self.ui, {
            #[node_key(H_STACK.size_x(0.23).position_x(Position::Start))]
            pub const CENTER_ROW: NodeKey;
            h_stack!(self.ui, CENTER_ROW, {
                v_stack!(self.ui, {
                    if self.counter_state.counter_mode {
                        let new_color = count_color(self.counter_state.count);
                        add!(self.ui, CounterState::INCREASE_BUTTON).set_color(new_color);

                        let count = &self.counter_state.count.to_string();
                        add!(self.ui, CounterState::COUNT_LABEL).set_text(count).set_text_attrs(
                            Attrs::new().family(Family::SansSerif).color(GlyphonColor::rgb(255, 76, 23)).weight(Weight::EXTRA_BOLD)

                        )
                        .set_text_align(Align::Center);

                        add!(self.ui, CounterState::DECREASE_BUTTON);
                    }
                });

                let text = match self.counter_state.counter_mode {
                    true => "Hide counter",
                    false => "Show counter",
                };
                add!(self.ui, CounterState::SHOW_COUNTER_BUTTON).set_text(text);
            });
        });
    }

    pub fn counter_on_click(&mut self) {
        if self.ui.is_clicked(CounterState::INCREASE_BUTTON) {
            self.counter_state.count += 1;
        }

        if self.ui.is_clicked(CounterState::DECREASE_BUTTON) {
            self.counter_state.count -= 1;
        }

        if self.ui.is_clicked(CounterState::SHOW_COUNTER_BUTTON) {
            self.counter_state.counter_mode = !self.counter_state.counter_mode;
        }
    }

    pub fn add_pixel_info_ui(&mut self, pixel_info: &Option<PixelInfo>) {
        let (x, y) = match pixel_info {
            Some(pixel_info) => (format!("{}", pixel_info.x), format!("{}", pixel_info.y)),
            None => ("".to_owned(), "".to_owned()),
        };

        let (r, g, b, a) = match pixel_info {
            Some(pixel_info) => (
                format!("{:.2}", pixel_info.color.r),
                format!("{:.2}", pixel_info.color.g),
                format!("{:.2}", pixel_info.color.b),
                format!("{:.2}", pixel_info.color.a),
            ),
            None => ("".to_owned(), "".to_owned(), "".to_owned(), "".to_owned()),
        };

        // todo:::::: I don't want strings, I want to write!() directly into the buffer
        #[node_key(PANEL.size_y(0.5).position_x(Position::Start))]
        const PIXEL_PANEL: Nodekey;
        add!(self.ui, PIXEL_PANEL, {
            v_stack!(self.ui, {
                h_stack!(self.ui, {
                    text!(self.ui, "x:");
                    text!(self.ui, &x);
                    text!(self.ui, "y:");
                    text!(self.ui, &y);
                });

                h_stack!(self.ui, {
                    text!(self.ui, "r:");
                    text!(self.ui, &r);
                    text!(self.ui, "g:");
                    text!(self.ui, &g);
                });
                h_stack!(self.ui, {
                    text!(self.ui, "b:");
                    text!(self.ui, &b);
                    text!(self.ui, "a:");
                    text!(self.ui, &a);
                });
            });
        });
    }
}
