// crate::* is needed to fix some crap with macros: https://github.com/rust-lang/rust/pull/52234#issuecomment-894851497
// when ui will be in its own crate, this won't happen anymore
use crate::node_params::*;
use crate::ui::Len::*;
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
    pub fn update_ui(&mut self) {
        tree!(self.ui, {

            #[node_key]
            const RIGHT_BAR: TypedKey<Stack>;
            let sidebar_params = V_STACK
                .position_x(Position::End)
                .size_y(Fill)
                .size_x(FitContent)
                .stack_arrange(Arrange::Center);

            add!(self.ui, RIGHT_BAR, sidebar_params, {

                self.slider_value = self.ui.add_slider(self.slider_value);

            });


            let left_bar_params = V_STACK
                .position_x(Position::Start)
                .size_y(Fill)
                .size_x(FitContent)
                .stack_arrange(Arrange::Center);

            add_anon!(self.ui, left_bar_params, {
                self.add_tools();
                self.add_pixel_info_ui();
            });

            // // self.add_counter_ui();
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

        #[node_key]
        const PIXEL_PANEL2: NodeKey;
        let pixel_panel_params = FLGR_PANEL
            .position_x(Start)
            .position_y(Start)
            .size_x(FitContentOrMinimum(Pixels(100)));

        use add_parent_closure::AddParentClosure;

        self.ui.add_parent(PIXEL_PANEL2, &pixel_panel_params, |ui| {
            ui.v_stack(|ui| {
                ui.text(&x);
                ui.text(&y);
            });
        });
    }

    pub fn add_tools(&mut self) {
        #[node_key]
        const BRUSH: NodeKey;
        let brush_params = ICON_BUTTON;

        #[node_key]
        const ERASER: NodeKey;
        let eraser_params = ICON_BUTTON;

        #[node_key]
        const TOOLS_PANEL: NodeKey;
        let tools_params = FLGR_PANEL
            .position_x(Start)
            .position_y(Start)
            .size_x(FitContent);


        use add_parent_manual::AddParentManual;
        self.ui.add_parent(TOOLS_PANEL, &tools_params);
        {
            self.ui.h_stack();
            {
                self.ui.v_stack();
                {
                    self.ui.add(BRUSH, &brush_params).image(include_bytes!("icons/brush.png"));
                    self.ui.add(ERASER, &eraser_params).image(include_bytes!("icons/eraser.png"));
                }
                self.ui.end_v_stack();
            }
            self.ui.end_h_stack();
        }
        self.ui.end_parent(TOOLS_PANEL);

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
        const FIXED_LEN: u32 = 200;

        #[node_key]
        pub const SLIDER_CONTAINER: NodeKey;
        let slider_container_params = FLGR_PANEL.size_y(Fixed(Pixels(FIXED_LEN))).size_x(Fixed(Pixels(50)));
        
        #[node_key]
        pub const SLIDER_FILL: NodeKey;
        let slider_fill_params = FLGR_PANEL
        .size_x(Fill)
        .size_y(Fixed(Frac(0.4)))
        .color(Color::FLGR_RED)
        .position_y(End)
        .padding_y(Pixels(2));
    
        let mut value = value;
        
        use add_parent_manual::AddParentManual;
        self.add_parent(SLIDER_CONTAINER, &slider_container_params);
        {
            self.add(
                SLIDER_FILL,
                &slider_fill_params.size_y(Fixed(Pixels(value as u32))),
            );
        }
        self.end_parent(SLIDER_CONTAINER);
        
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
// fn text_params(pixel_info: Option<&PixelInfo>) -> NodeParams {

//     let (x, y) = match pixel_info {
//         Some(pixel_info) => (
//             format!("x: {}", pixel_info.x),
//             format!("y: {}", pixel_info.y),
//         ),
//         None => ("x:  ".to_owned(), "y:  ".to_owned()),
//     };

//     return TEXT.text(&x);
// }
