use keru::*;
use keru::node_library::*;

use crate::color_picker::ColorPickerUi;
use crate::oklab::*;
use crate::canvas::PixelColorF32;
use crate::State;

const COLOR1: Color = Color::KERU_PINK.with_alpha(0.94);
pub const KERU_PANEL: Node = PANEL.color(COLOR1);

const MIN_RADIUS: f32 = 1.0;
const MAX_RADIUS: f32 = 100.0;

impl State {

    pub fn update_ui(&mut self) {
        let right_bar = V_STACK
            .position_x(Pos::End)
            .size_y(Size::Fill)
            .size_x(Size::FitContent);

        let left_bar = V_STACK
            .position_x(Pos::Start)
            .size_y(Size::Fill)
            .size_x(Size::Frac(0.1));

        self.ui.add(right_bar).nest(|| {
            self.ui.place_color_picker(&mut self.color_picker);
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
        with_arena(|arena| {

            let mut text = bumpalo::format!(in arena, "{}", "");
            if let Some(pixel_info) = &self.canvas.pixel_info() {
                text = bumpalo::format!(in arena, "{}:{}", pixel_info.x, pixel_info.y);
            };

            let pixel_panel_2 = KERU_PANEL
                .position_x(Pos::Start)
                .position_y(Pos::Start)
                .size_x(Size::Pixels(110.0))
                .size_y(Size::Pixels(50.0));

            self.ui.add(pixel_panel_2).nest(|| {
                self.ui.v_stack().nest(|| {
                    self.ui.text_line(&text);
                });
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
            .position_x(Pos::Start)
            .position_y(Pos::Start)
            .size_symm(Size::FitContent);

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

        #[node_key] const SLIDER_CONTAINER: NodeKey;
        #[node_key] const SLIDER_FILL: NodeKey;

        let slider_height = match self.ui.get_node(SLIDER_CONTAINER).map(|n| n.inner_size()) {
            Some(container) => container.y as f32,
            // this is just for the first frame. awkward.
            None => 1.0,
        };

        if let Some(drag) = self.ui.is_dragged(SLIDER_CONTAINER) {
            log_value -= (drag.absolute_delta.y as f32) / slider_height * (log_max - log_min);
        }
        if let Some(drag) = self.ui.is_dragged(SLIDER_FILL) {
            log_value -= (drag.absolute_delta.y as f32) / slider_height * (log_max - log_min);
        }

        log_value = log_value.clamp(log_min, log_max);
        let filled_frac = (log_value - log_min) / (log_max - log_min);

        let slider_container = KERU_PANEL
            .position_x(Pos::End)
            .size_y(Size::Fill)
            .size_x(Size::Pixels(60.0))
            .sense_drag(true)
            .key(SLIDER_CONTAINER);

        let slider_fill = KERU_PANEL
            .size_x(Size::Fill)
            .size_y(Size::Frac(filled_frac))
            .color(Color::RED)
            .position_y(Pos::End)
            .sense_drag(true)
            .padding_y(1.0)
            .key(SLIDER_FILL);

        let new_lin_value = 10f32.powf(log_value);

        with_arena(|arena| {
            let text = bumpalo::format!(in arena, "{:.2}", new_lin_value);
            
            self.ui.add(slider_container).nest(|| {
                self.ui.add(slider_fill);
                self.ui.text_line(&text);
            });
            
        });
        return new_lin_value;
    }
}
