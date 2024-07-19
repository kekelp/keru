// crate::* is needed to fix some crap with macros: https://github.com/rust-lang/rust/pull/52234#issuecomment-894851497
// when ui will be in its own crate, this won't happen anymore
use crate::*;
use crate::ui::*;
use view_derive::node_key;
use crate::ui::Position::*;

impl State {
    pub fn update_ui(&mut self) {
        let ui = &mut self.ui;
        ui.begin_tree();

        #[node_key(MARGIN.size_y(0.95).size_x(1.0).position_x(Position::Center))]
        const MARGIN2: Nodekey;
        add!(ui, MARGIN2, {

            #[node_key(V_STACK.size_x(0.3).position_x(Position::End))]            
            const SIDEBAR: Nodekey;
            add!(ui, SIDEBAR, {

                let mut color = add!(ui, PAINT_COLOR).get_text();

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

        self.counter_state.add(ui);
        
        
        ui.finish_tree();
        
        
        self.counter_state.interact(ui);

    }
}




// #[derive_view(
//     H_STACK
//     .size_y(0.95)
//     .size_x(0.8)
//     .stack(Y, Arrange::End)
//     .color(Color::BLUE)
// )]
// pub struct CommandLineRow;

#[node_key(TEXT_INPUT.text("Color").size_y(0.2).position_y(Start))]
pub const PAINT_COLOR: NodeKey;

// #[derive_view(TEXT_INPUT.text("RERER"))]
// pub struct CommandLine;

// #[derive_view(LABEL)]
// pub struct Label234;

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

    #[node_key(BUTTON.text("Show Counter").color(Color::rgba(0.5, 0.1, 0.7, 0.7)))]
    pub const SHOW_COUNTER_BUTTON: NodeKey;

    #[node_key(LABEL)]
    pub const COUNT_LABEL: NodeKey;


    pub fn add(&mut self, ui: &mut Ui) {
        margin!(ui, {
            
            #[node_key(H_STACK.size_x(0.5).position_x(Position::Start))]
            pub const CENTER_ROW: NodeKey;
            add!(ui, CENTER_ROW, {
                v_stack!(ui, {
                    if self.counter_mode {
                        let new_color = count_color(self.count);
                        ui.add(Self::INCREASE_BUTTON).set_color(new_color);
    
                        let count = &self.count.to_string();
                        ui.add(Self::COUNT_LABEL).set_text(count);
    
                        ui.add(Self::DECREASE_BUTTON);
                    }
                });
    
                v_stack!(ui, {
                    let text = match self.counter_mode {
                        true => "Hide counter",
                        false => "Show counter",
                    };
                    ui.add(Self::SHOW_COUNTER_BUTTON).set_text(text);
                });
            });
        });
    }

    pub fn interact(&mut self, ui: &mut Ui) {
        if ui.is_clicked(Self::INCREASE_BUTTON) {
            self.count += 1;
        }

        if ui.is_clicked(Self::DECREASE_BUTTON) {
            self.count -= 1;
        }

        if ui.is_clicked(Self::SHOW_COUNTER_BUTTON) {
            self.counter_mode = !self.counter_mode;
        }
    }
}
pub fn count_color(count: i32) -> Color {
    let red = 0.1 * (count as f32);
    return Color::rgba(red, 0.1, 0.2, 0.8);
}
