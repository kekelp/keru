// crate::* is needed to fix some crap with macros: https://github.com/rust-lang/rust/pull/52234#issuecomment-894851497
// when ui will be in its own crate, this won't happen anymore
use crate::node_params::*;
use crate::math::Len::*;
use crate::ui::Position::*;
use crate::ui::Size::*;
use crate::ui::*;
use crate::*;

use view_derive::node_key;

const COLOR1: Color = Color::rgba(50, 13, 100, 240);
const COLOR2: Color = Color::rgba(100, 13, 50, 240);
const GRAD1: VertexColors = VertexColors::diagonal_gradient_forward_slash(COLOR1, COLOR2);
const FLGR_PANEL: NodeParams = PANEL.vertex_colors(GRAD1);

impl State {
    #[rustfmt::skip]
    pub fn update_ui(&mut self) {
        self.ui.begin_tree();

        #[node_key] const INCREASE: NodeKey;
        #[node_key] const DECREASE: NodeKey;
        let increase = BUTTON.key(INCREASE);
        let decrease = BUTTON.key(DECREASE);
        let count_label = LABEL;

        let count_str = &self.count_state.count.to_string();

        self.ui.parent(&V_STACK).children(&[
            self.ui.add2(&increase, "Increase"),
            self.ui.add2(&count_label, count_str),
            self.ui.add2(&decrease, "Decrease"),
        ]).build(&mut self.ui);

        if self.ui.is_clicked(INCREASE) {
            self.count_state.count += 1;
        }
        if self.ui.is_clicked(DECREASE) {
            self.count_state.count -= 1;
        }

        self.ui.finish_tree();
    }

    pub fn update_ui_old(&mut self) {
        tree!(self.ui, {

            println!("  {:?}", "Among us");

            #[node_key] const RIGHT_BAR: NodeKey;

            let sidebar = V_STACK
                .key(RIGHT_BAR)
                .position_x(Position::End)
                .size_y(Fill)
                .size_x(FitContent)
                .stack_arrange(Arrange::Center);

            add!(self.ui, sidebar, {
                self.slider_value = self.ui.add_slider(self.slider_value);
            });


            let left_bar = V_STACK
                .position_x(Position::Start)
                .size_y(Fill)
                .size_x(FitContent)
                .stack_arrange(Arrange::Center);

            add!(self.ui, left_bar, {
                self.add_tools();
                self.add_pixel_info_ui();
            });

        });
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
}

pub fn count_color(count: i32) -> Color {
    let red = (0.1 * (count as f32) * 255.0) as u8;
    return Color::rgba(red, 26, 52, 205);
}

impl State {
    pub fn add_pixel_info_ui(&mut self) {
        let pixel_info = &self.canvas.pixel_info();

        let (x, y) = match pixel_info {
            Some(pixel_info) => (
                format!("x: {}", pixel_info.x),
                format!("y: {}", pixel_info.y),
            ),
            None => ("x:  ".to_owned(), "y:  ".to_owned()),
        };

        #[node_key] const PIXEL_PANEL2: NodeKey;
        let pixel_panel = FLGR_PANEL
            .key(PIXEL_PANEL2)
            .position_x(Start)
            .position_y(Start)
            .size_x(FitContentOrMinimum(Pixels(100)));

        use add_parent_closure::AddParentClosure;

        self.ui.add(&pixel_panel).children(|ui| {
            ui.v_stack(|ui| {
                ui.text(&x);
                ui.text(&y);
            });
        });
    }

    pub fn add_tools(&mut self) {
        #[node_key] const BRUSH: NodeKey;
        let brush = ICON_BUTTON.key(BRUSH);

        #[node_key] const ERASER: NodeKey;
        let eraser = ICON_BUTTON.key(ERASER);

        #[node_key] const TOOLS_PANEL: NodeKey;
        let tools = FLGR_PANEL
            .key(TOOLS_PANEL)
            .position_x(Start)
            .position_y(Start)
            .size_x(FitContent);


        use add_parent_manual::AddParentManual;
        self.ui.add_parent(&tools);
        {
            self.ui.h_stack();
            {
                self.ui.v_stack();
                {
                    self.ui.add(&brush).static_image(include_bytes!("icons/brush.png"));
                    self.ui.add(&eraser).static_image(include_bytes!("icons/eraser.png"));
                }
                self.ui.end_v_stack();
            }
            self.ui.end_h_stack();
        }
        self.ui.end_parent(&tools);

        if self.ui.is_clicked(BRUSH) {
            self.canvas.paint_color = PixelColorF32::new(0.2, 0.8, 0.2, 1.0);
        }

        if self.ui.is_clicked(ERASER) {
            self.canvas.paint_color = PixelColorF32::new(1.0, 1.0, 1.0, 0.0);
        }
    }
}

impl Ui {

    pub fn add_slider(&mut self, value: f32) -> f32 {
        let mut value = value;

        const FIXED_LEN: u32 = 200;

        #[node_key] pub const SLIDER_CONTAINER: NodeKey;
        let slider_container = FLGR_PANEL
            .size_y(Fixed(Pixels(FIXED_LEN)))
            .size_x(Fixed(Pixels(50)))
            .key(SLIDER_CONTAINER);
        
        #[node_key] pub const SLIDER_FILL: NodeKey;
        let slider_fill = FLGR_PANEL
        .key(SLIDER_FILL)
        .size_x(Fill)
        .size_y(Fixed(Pixels(value as u32)))
        .color(Color::FLGR_RED)
        .position_y(End)
        .padding_y(Pixels(2));
    
        
        use add_parent_manual::AddParentManual;
        self.add_parent(&slider_container);
        {
            self.add(&slider_fill);
        }
        self.end_parent(&slider_container);
        
        if let Some((_x, y)) = self.is_dragged(SLIDER_CONTAINER) {
            value += y as f32;
            value = value.clamp(0.0, FIXED_LEN as f32);
        }
        if let Some((_x, y)) = self.is_dragged(SLIDER_FILL) {
            value += y as f32;
            value = value.clamp(0.0, FIXED_LEN as f32);
        }
        
        return value;
    }
}

// // // this is what you cannot do in lifetime soup mode
// fn text(pixel_info: Option<&PixelInfo>) -> NodeParams {

//     let (x, y) = match pixel_info {
//         Some(pixel_info) => (
//             format!("x: {}", pixel_info.x),
//             format!("y: {}", pixel_info.y),
//         ),
//         None => ("x:  ".to_owned(), "y:  ".to_owned()),
//     };

//     return TEXT.text(&x);
// }
