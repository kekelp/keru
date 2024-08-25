// crate::* is needed to fix some crap with macros: https://github.com/rust-lang/rust/pull/52234#issuecomment-894851497
// when ui will be in its own crate, this won't happen anymore
use crate::node_params::*;
use crate::ui::Len::*;
use crate::ui::Position::*;
use crate::ui::Size::*;
use crate::ui::*;
use crate::*;
use glyphon::{cosmic_text::Align, Attrs, Color as GlyphonColor, Family, Weight};
use view_derive::node_key;

const COLOR1: Color = Color::rgba(50, 13, 100, 240);
const COLOR2: Color = Color::rgba(100, 13, 50, 240);
const GRAD1: VertexColors = VertexColors::diagonal_gradient_forward_slash(COLOR1, COLOR2);
const FLGR_PANEL: NodeParams = PANEL.vertex_colors(GRAD1);

impl State {
    pub fn update_ui(&mut self) {
        tree!(self.ui, {
            #[node_key(PANEL)]
            const SLIDER: NodeKey;

            if let Some((x, _y)) = self.ui.is_dragged(SLIDER) {
                self.slider_value -= x as f32;
            }

            add!(self.ui, SLIDER, {
                text!(self.ui, "Slider");
            })
            .set_position_x(Position::Static(Pixels(self.slider_value as u32)));

            #[node_key(V_STACK.position_x(Position::End).size_y(Fill).size_x(FitContent).stack_arrange(Arrange::Center))]
            const SIDEBAR: TypedKey<Stack>;
            add!(self.ui, SIDEBAR, {
                self.add_tools();

                self.add_pixel_info_ui();
            });

            // self.add_counter_ui();
        });

        // effects
        self.counter_on_click();
    }
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
            counter_mode: false,
        };
    }

    #[node_key(BUTTON.text("Increase").color(Color::FLGR_GREEN))]
    pub const INCREASE_BUTTON: NodeKey;

    #[node_key(BUTTON.text("Decrease").color(Color::FLGR_RED))]
    pub const DECREASE_BUTTON: NodeKey;

    #[node_key(BUTTON.text("Show Counter").color(Color::rgba(128, 26, 179, 102)))]
    pub const SHOW_COUNTER_BUTTON: NodeKey;

    #[node_key(LABEL)]
    pub const COUNT_LABEL: TypedKey<Text>;

    #[node_key(LABEL)]
    pub const COUNT_LABEL_2: NodeKey;

    // pub const COUNT_LABEL: TypedKey<Text> =
    // TypedKey::<Text>::new(&LABEL.debug_name("COUNT_LABEL"), Id(8379459943087886814)).validate();

    // pub const COUNT_LABEL: TypedKey<Text> = TypedKey::new(&LABEL.debug_name("COUNT_LABEL"), Id(4286411996384850605)).validate();

    // pub const COUNT_LABEL: TypedKey<Text> = TypedKey::new(&H_STACK.debug_name("COUNT_LABEL56"), Id(4286411996384850605)).validate();
}

pub fn count_color(count: i32) -> Color {
    let red = (0.1 * (count as f32) * 255.0) as u8;
    return Color::rgba(red, 26, 52, 205);
}

impl State {
    pub fn add_twin_thing_ui(&mut self) {
        #[node_key(FLGR_PANEL.size_y(Fixed(Frac(0.5))).position_x(Position::Start))]
        const PIXEL_PANEL: NodeKey;
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
            #[node_key(H_STACK.position_x(Position::Center))]
            pub const CENTER_ROW: NodeKey;
            h_stack!(self.ui, CENTER_ROW, {
                v_stack!(self.ui, {
                    if self.counter_state.counter_mode {
                        let new_color = count_color(self.counter_state.count);
                        add!(self.ui, CounterState::INCREASE_BUTTON).set_color(new_color);

                        let count = &self.counter_state.count.to_string();
                        add!(self.ui, CounterState::COUNT_LABEL)
                            .set_text(count)
                            .set_text_attrs(
                                Attrs::new()
                                    .family(Family::SansSerif)
                                    .color(GlyphonColor::rgb(255, 76, 23))
                                    .weight(Weight::EXTRA_BOLD),
                            )
                            .set_text_align(Align::Center);

                        add!(self.ui, CounterState::DECREASE_BUTTON);
                    }
                });

                let text = match self.counter_state.counter_mode {
                    true => "Hide useless counter",
                    false => "Show Counter\nl\nl\nl\nl\nl\nllllllllllllllllllllÞÞÞÞÞÞÞÞÞÞÞÞÞÞÞÞÞÞÞÞØØ↑ı¥§",
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

    pub fn add_pixel_info_ui(&mut self) {
        let pixel_info = &self.canvas.pixel_info();

        let (x, y) = match pixel_info {
            Some(pixel_info) => (
                format!("x: {}", pixel_info.x),
                format!("y: {}", pixel_info.y),
            ),
            None => ("x:  ".to_owned(), "y:  ".to_owned()),
        };

        // let (r, g, b, a) = match pixel_info {
        //     Some(pixel_info) => (
        //         format!("{:.2}", pixel_info.color.r),
        //         format!("{:.2}", pixel_info.color.g),
        //         format!("{:.2}", pixel_info.color.b),
        //         format!("{:.2}", pixel_info.color.a),
        //     ),
        //     None => ("".to_owned(), "".to_owned(), "".to_owned(), "".to_owned()),
        // };

        // todo:::::: I don't want strings, I want to write!() directly into the buffer
        #[node_key(FLGR_PANEL.position_x(End).position_y(Start).size_x(FitContentOrMinimum(Pixels(100))))]
        const PIXEL_PANEL2: NodeKey;
        add!(self.ui, PIXEL_PANEL2, {
            v_stack!(self.ui, {
                text!(self.ui, &x);
                text!(self.ui, &y);
            });
        });
    }

    pub fn add_tools(&mut self) {
        #[node_key(ICON_BUTTON.image(include_bytes!("icons/brush.png")))]
        const BRUSH_ICON: NodeKey;

        #[node_key(ICON_BUTTON.image(include_bytes!("icons/eraser.png")))]
        const ERASER_ICON: NodeKey;

        #[node_key(FLGR_PANEL.position_x(End).position_y(Start).size_x(FitContent))]
        const TOOLS_PANEL: NodeKey;

        add!(self.ui, TOOLS_PANEL, {
            h_stack!(self.ui, {
                v_stack!(self.ui, {
                    add!(self.ui, BRUSH_ICON);
                    add!(self.ui, ERASER_ICON);
                });
            });
        });

        if self.ui.is_clicked(BRUSH_ICON) {
            self.canvas.paint_color = PixelColorF32::new(0.2, 0.8, 0.2, 1.0);
        }

        if self.ui.is_clicked(ERASER_ICON) {
            self.canvas.paint_color = PixelColorF32::new(1.0, 1.0, 1.0, 0.0);
        }
    }

    pub fn add_stacks_test(&mut self) {
        #[node_key(H_STACK.stack_arrange(Arrange::Start))]
        const HSTACK1: NodeKey;
        add!(self.ui, HSTACK1, {
            #[node_key(LABEL)]
            const LABEL7: TypedKey<Text>;

            #[node_key(V_STACK.stack_arrange(Arrange::Start))]
            const VSTACK_A: NodeKey;
            add!(self.ui, VSTACK_A, {
                add!(self.ui, LABEL7).set_text("a1");
                add!(self.ui, LABEL7).set_text("a2");
                add!(self.ui, LABEL7).set_text("a3");
            });

            #[node_key(V_STACK.stack_arrange(Arrange::Center))]
            const VSTACK_B: NodeKey;
            add!(self.ui, VSTACK_B, {
                add!(self.ui, LABEL7).set_text("b1");
                add!(self.ui, LABEL7).set_text("b2");
                add!(self.ui, LABEL7).set_text("b3");
            });

            #[node_key(V_STACK.stack_arrange(Arrange::End))]
            const VSTACK_C: NodeKey;

            add!(self.ui, VSTACK_C, {
                add!(self.ui, LABEL7).set_text("c1");
                add!(self.ui, LABEL7).set_text("c2");
                add!(self.ui, LABEL7).set_text("c3");
            });

            #[node_key(V_STACK.stack_arrange(Arrange::Start))]
            const VSTACK5567: NodeKey;
            add!(self.ui, VSTACK5567, {
                #[node_key(H_STACK.stack_arrange(Arrange::Start))]
                const HSTACK_A: NodeKey;
                add!(self.ui, HSTACK_A, {
                    add!(self.ui, LABEL7).set_text("x1");
                    add!(self.ui, LABEL7).set_text("x2");
                    add!(self.ui, LABEL7).set_text("x3");
                });

                #[node_key(H_STACK.stack_arrange(Arrange::Center))]
                const HSTACK_B: NodeKey;
                add!(self.ui, HSTACK_B, {
                    add!(self.ui, LABEL7).set_text("y1");
                    add!(self.ui, LABEL7).set_text("y2");
                    add!(self.ui, LABEL7).set_text("y3");
                });

                #[node_key(H_STACK.stack_arrange(Arrange::End))]
                const HSTACK_C: NodeKey;
                add!(self.ui, HSTACK_C, {
                    add!(self.ui, LABEL7).set_text("z1");
                    add!(self.ui, LABEL7).set_text("z2");
                    add!(self.ui, LABEL7).set_text("z3");
                });
            });
        });
    }

    pub fn color_box_or_something(&mut self) {
        margin!(self.ui, {
            #[node_key(V_STACK.size_x(Fixed(Frac(0.7))).size_y(Fixed(Frac(0.5))).position_x(Center).position_y(Start))]
            const SIDEBAR2: NodeKey;
            #[node_key(TEXT_INPUT.text("Color").size_y(Fixed(Frac(0.2))).position_y(Start))]
            pub const PAINT_COLOR: NodeKey;
            add!(self.ui, SIDEBAR2, {
                // todo: function for doing get_text from other places
                let mut color = add!(self.ui, PAINT_COLOR).get_text();

                if let Some(color) = &mut color {
                    color.make_ascii_lowercase();
                    match color.as_str() {
                        "blue" => {
                            self.canvas.paint_color = PixelColorF32::BLUE;
                        }
                        "red" => {
                            self.canvas.paint_color = PixelColorF32::RED;
                        }
                        "green" => {
                            self.canvas.paint_color = PixelColorF32::GREEN;
                        }
                        _ => {}
                    }
                }
            });
        });
    }
}
