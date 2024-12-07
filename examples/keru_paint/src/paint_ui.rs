use std::fmt::Write;

use keru::*;
use keru::Position::*;
use keru::Size::*;
use keru::Len::*;

use crate::color_picker::ColorPickerUi;
use crate::oklab::*;
use crate::PixelColorF32;
use crate::State;

const COLOR1: Color = Color::rgba(50, 13, 100, 240);
const COLOR2: Color = Color::rgba(100, 13, 50, 240);
const GRAD1: VertexColors = VertexColors::diagonal_gradient_forward_slash(COLOR1, COLOR2);
pub const FLGR_PANEL: NodeParams = PANEL.vertex_colors(GRAD1);

impl State {

    pub fn update_ui(&mut self) {
        #[node_key] const RIGHT_BAR: NodeKey;
        self.ui.add(RIGHT_BAR)
            .params(V_STACK)
            .position_x(Position::End)
            .size_y(Fill)
            .size_x(FitContent);

        #[node_key] const LEFT_BAR: NodeKey;
        self.ui.add(LEFT_BAR)
            .params(V_STACK)
            .position_x(Position::Start)
            .size_y(Fill)
            .size_x(Fixed(Frac(0.1)));

        self.ui.place(RIGHT_BAR).nest(|| {
            self.slider_value = self.add_slider(self.slider_value);
            self.ui.add_color_picker(&mut self.color_picker);
        });
            
        self.ui.place(LEFT_BAR).nest(|| {
            self.add_pixel_info_ui();
            self.add_tools();
        });
        
        let picked_color = oklch_to_linear_srgb(self.color_picker.oklch_color);
        self.canvas.paint_color = PixelColorF32 {
            r: picked_color.r,
            g: picked_color.g,
            b: picked_color.b,
            a: 1.0,
        };
    }
}

impl State {
    pub fn add_pixel_info_ui(&mut self) {
        // todo: think of a lazier way that's still zero-allocation but doesn't require the user to keep his own format_scratch
        // one way is impl'ing Display, maybe
        self.format_scratch.clear();
        if let Some(pixel_info) = &self.canvas.pixel_info() {
            let _ = write!(&mut self.format_scratch, "{}:{}", pixel_info.x, pixel_info.y);
        } else {
            let _ = write!(&mut self.format_scratch, "  :  ");
        }
        
        #[node_key] const PIXEL_PANEL2: NodeKey;
        self.ui.add(PIXEL_PANEL2)
            .params(FLGR_PANEL)
            .position_x(Start)
            .position_y(Start)
            // without a fixed size here, we get way too much partial relayouting to do on every frame
            .size_x(Fixed(Pixels(100)))
            .size_y(Fixed(Pixels(50)));

        self.ui.place(PIXEL_PANEL2).nest(|| {
            self.ui.v_stack().nest(|| {
                self.ui.text(&self.format_scratch);
            });
        });
    }

    pub fn add_tools(&mut self) {
        #[node_key] const TOOLS_PANEL: NodeKey;
        #[node_key] const BRUSH: NodeKey;
        #[node_key] const ERASER: NodeKey;

        if self.ui.is_clicked(BRUSH) {
            self.canvas.eraser_mode = false;
        }

        if self.ui.is_clicked(ERASER) {
            self.canvas.eraser_mode = true;
        }

        // This never changes
        let changed = false;
        if self.ui.already_exists(TOOLS_PANEL) && ! changed {
            self.ui.place_and_assume_unchanged(TOOLS_PANEL);
            return;
        }
        
        self.ui.add(BRUSH).params(ICON_BUTTON).static_image(include_bytes!("icons/brush.png"));
        self.ui.add(ERASER).params(ICON_BUTTON).static_image(include_bytes!("icons/eraser.png"));

        self.ui.add(TOOLS_PANEL)
            .params(FLGR_PANEL)
            .position_x(Start)
            .position_y(Start)
            .size_x(FitContent);

        self.ui.place(TOOLS_PANEL).nest(|| {
            self.ui.place_h_stack().nest(|| {
                self.ui.v_stack().nest(|| {
                    self.ui.place(BRUSH);
                    self.ui.place(ERASER);
                });
            });
        });
    }

    pub fn add_slider(&mut self, value: f32) -> f32 {
        let mut value = value;

        #[node_key] pub const SLIDER_CONTAINER: NodeKey;
        self.ui.add(SLIDER_CONTAINER)
            .params(FLGR_PANEL)
            .position_x(End)
            .size_y(Size::Fixed(Frac(0.7)))
            .size_x(Fixed(Pixels(50)));

        #[node_key] pub const SLIDER_FILL: NodeKey;
        self.ui.add(SLIDER_FILL)
            .params(FLGR_PANEL)
            .size_x(Fill)
            .size_y(Fixed(Frac(value)))
            .color(Color::FLGR_RED)
            .position_y(End)
            .padding_y(Pixels(1));


        self.ui.place(SLIDER_CONTAINER).nest(|| {
            self.ui.place(SLIDER_FILL);
        });

        let size = self.ui.get_node(SLIDER_CONTAINER).unwrap().inner_size().y as f32;

        if let Some((_, y)) = self.ui.is_dragged(SLIDER_CONTAINER) {
            value += (y as f32) / size;
            value = value.clamp(0.0, 1.0);
        }
        if let Some((_, y)) = self.ui.is_dragged(SLIDER_FILL) {
            value += (y as f32) / size;
            value = value.clamp(0.0, 1.0);
        }

        return value;
    }
}
