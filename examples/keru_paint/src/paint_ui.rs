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
            self.ui.add_color_picker(&mut self.color_picker);
            
            let min_radius = 1.0;
            let max_radius = 100.0;
            let slider_val = self.add_log_slider(self.canvas.radius as f32, min_radius, max_radius);
            self.canvas.radius = slider_val as f64;
        });
            
        self.ui.place(LEFT_BAR).nest(|| {
            self.add_tools();
            self.add_pixel_info_ui();
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
        if self.ui.is_in_tree(TOOLS_PANEL) && ! changed {
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
            self.ui.h_stack().nest(|| {
                self.ui.v_stack().nest(|| {
                    self.ui.place(BRUSH);
                    self.ui.place(ERASER);
                });
            });
        });
    }
    
    pub fn add_log_slider(&mut self, linear_value: f32, min: f32, max: f32) -> f32 {
        assert!(min > 0.0 && max > min, "Log sliders require positive min and max values");

        // Convert linear value to logarithmic for slider representation
        let log_min = min.log10();
        let log_max = max.log10();
        let mut log_value = linear_value.log10();

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
            .size_y(Fixed(Frac((log_value - log_min) / (log_max - log_min))))
            .color(Color::FLGR_RED)
            .position_y(End)
            .padding_y(Pixels(1));


        let slider_height = self.ui.get_node(SLIDER_CONTAINER).unwrap().inner_size().y as f32;

        if let Some((_, y)) = self.ui.is_dragged(SLIDER_CONTAINER) {
            log_value += (y as f32) / slider_height * (log_max - log_min);
        }
        if let Some((_, y)) = self.ui.is_dragged(SLIDER_FILL) {
            log_value += (y as f32) / slider_height * (log_max - log_min);
        }

        log_value = log_value.clamp(log_min, log_max);
        log_value = 10f32.powf(log_value);

        // There's 2 reasons why can can't just pass the f32 and let the UI format it:
        // - we're using a custom format. This could be solved by making a text!() macro. but it wouldn't be nice.
        // - f32 doesn't implement Hash!!!!!!!!!!!!! So we still couldn't skip the formatting when it's unchanged
        self.format_scratch.clear();
        let _ = write!(&mut self.format_scratch, "{:.2}", log_value);

        self.ui.place(SLIDER_CONTAINER).nest(|| {
            self.ui.place(SLIDER_FILL);
            self.ui.text(&self.format_scratch);
        });

        self.format_scratch.clear();
        return log_value;
    }
}
