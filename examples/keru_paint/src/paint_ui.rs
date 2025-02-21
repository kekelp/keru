use std::fmt::Write;

use keru::*;
use keru::Position::*;
use keru::Size::*;

use crate::color_picker::ColorPickerUi;
use crate::oklab::*;
use crate::PixelColorF32;
use crate::State;

const COLOR1: Color = Color::rgba(50, 13, 100, 240);
const COLOR2: Color = Color::rgba(100, 13, 50, 240);
const GRAD1: VertexColors = VertexColors::diagonal_gradient_forward_slash(COLOR1, COLOR2);
pub const KERU_PANEL: NodeParams = PANEL.vertex_colors(GRAD1);

const MIN_RADIUS: f32 = 1.0;
const MAX_RADIUS: f32 = 100.0;

impl State {

    pub fn update_ui(&mut self) {
        let right_bar = V_STACK
            .position_x(Position::End)
            .size_y(Fill)
            .size_x(FitContent);

        let left_bar = V_STACK
            .position_x(Position::Start)
            .size_y(Fill)
            .size_x(Size::Frac(0.1));

        self.ui.add(right_bar).nest(|| {
            self.ui.place_color_picker(&mut self.color_picker);
            
            // todo: who asked for this functional crap? just pass a in reference
            let slider_val = self.add_log_slider(self.canvas.radius as f32, MIN_RADIUS, MAX_RADIUS);
            self.canvas.radius = slider_val as f64;
        });
            
        self.ui.add(left_bar).nest(|| {
            self.place_tools();
            self.place_pixel_info_ui();
        });
        
        let picked_color = oklch_to_linear_srgb(self.color_picker.oklch_color);

        let paint_color = PixelColorF32 {
            r: picked_color.r,
            g: picked_color.g,
            b: picked_color.b,
            a: 1.0,
        };

        if paint_color != self.canvas.paint_color {
            self.canvas.eraser_mode = false;
        }
        self.canvas.paint_color = paint_color;
    }
}

impl State {
    pub fn place_pixel_info_ui(&mut self) {
        // todo: think of a lazier way that's still zero-allocation but doesn't require the user to keep his own format_scratch
        // one way is impl'ing Display, maybe
        self.format_scratch.clear();
        if let Some(pixel_info) = &self.canvas.pixel_info() {
            let _ = write!(&mut self.format_scratch, "{}:{}", pixel_info.x, pixel_info.y);
        } else {
            let _ = write!(&mut self.format_scratch, "  :  ");
        }

        let pixel_panel_2 = KERU_PANEL
            .position_x(Start)
            .position_y(Start)
            // without a fixed size here, we get way too much partial relayouting to do on every frame
            .size_x(Size::Pixels(100))
            .size_y(Size::Pixels(50));

        self.ui.add(pixel_panel_2).nest(|| {
            self.ui.v_stack().nest(|| {
                self.ui.text(&self.format_scratch);
            });
        });
    }

    pub fn place_tools(&mut self) {
        #[node_key] const BRUSH: NodeKey;
        #[node_key] const ERASER: NodeKey;

        if self.ui.is_clicked(BRUSH) {
            self.canvas.eraser_mode = false;
        }

        if self.ui.is_clicked(ERASER) {
            self.canvas.eraser_mode = true;
        }
        
        let brush = ICON_BUTTON.static_image(include_bytes!("icons/brush.png")).key(BRUSH);
        let eraser = ICON_BUTTON.static_image(include_bytes!("icons/eraser.png")).key(ERASER);

        let tools_panel = KERU_PANEL
            .position_x(Start)
            .position_y(Start)
            // todo: something is off here
            .padding_y(20)
            .size_x(FitContent);

        self.ui.add(tools_panel).nest(|| {
            self.ui.h_stack().nest(|| {
                self.ui.v_stack().nest(|| {
                    self.ui.add(brush);
                    self.ui.add(eraser);
                });
            });
        });
    }
    
    pub fn add_log_slider(&mut self, linear_value: f32, min: f32, max: f32) -> f32 {
        debug_assert!(min > 0.0 && max > min, "Log sliders require positive min and max values");

        // Convert linear value to logarithmic for slider representation
        let log_min = min.log10();
        let log_max = max.log10();
        let mut log_value = linear_value.log10();

        let slider_height = match self.ui.get_node(SLIDER_CONTAINER) {
            Some(container) => container.inner_size().y as f32,
            // this is just for the first frame. awkward.
            None => 1.0,
        };

        let (_, y) = self.ui.is_dragged(SLIDER_CONTAINER);
        log_value += (y as f32) / slider_height * (log_max - log_min);
        
        let (_, y) = self.ui.is_dragged(SLIDER_FILL);
        log_value += (y as f32) / slider_height * (log_max - log_min);

        log_value = log_value.clamp(log_min, log_max);
        let filled_frac = (log_value - log_min) / (log_max - log_min);

        #[node_key] const SLIDER_CONTAINER: NodeKey;
        let slider_container = KERU_PANEL
            .position_x(End)
            .size_y(Size::Frac(0.7))
            .size_x(Size::Pixels(60))
            .key(SLIDER_CONTAINER);

        #[node_key] const SLIDER_FILL: NodeKey;
        let slider_fill = KERU_PANEL
            .size_x(Fill)
            .size_y(Size::Frac(filled_frac))
            .color(Color::KERU_RED)
            .position_y(End)
            .padding_y(1)
            .key(SLIDER_FILL);

        let new_lin_value = 10f32.powf(log_value);

        // There's 2 reasons why can can't just pass the f32 and let the UI format it:
        // - we're using a custom format. This could be solved by making a text!() macro. but it wouldn't be nice.
        // - f32 doesn't implement Hash!!!!!!!!!!!!! So we still couldn't skip the formatting when it's unchanged
        self.format_scratch.clear();
        let _ = write!(&mut self.format_scratch, "{:.2}", new_lin_value);

        self.ui.add(slider_container).nest(|| {
            self.ui.add(slider_fill);
            self.ui.text(&self.format_scratch);
        });

        self.format_scratch.clear();
        return new_lin_value;
    }
}
