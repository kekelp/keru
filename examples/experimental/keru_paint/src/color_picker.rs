use keru::*;
use keru::node_library::*;
use wgpu::RenderPass;

use crate::color_picker_render::ColorPickerRenderer;
use crate::oklab::*;
use crate::paint_ui::KERU_PANEL;
use crate::window::Context;

pub struct ColorPicker {
    pub oklch_color: OkLchColor,
    pub renderer: ColorPickerRenderer,
    // Set when the picked color changes, so the custom-rendered wheel/square get redrawn.
    pub need_rerender: bool,
}

const NEUTRAL_GREY: Color = Color::new(0.09, 0.09, 0.09, 1.0);

// Width of the hue ring, in pixels. Must match `WIDTH` in shaders/color_picker.wgsl.
const RING_WIDTH: f32 = 28.0;

#[node_key] pub const OKLAB_HUE_WHEEL: NodeKey;
#[node_key] pub const OKLAB_SQUARE: NodeKey;
#[node_key] pub const SMALL_RING: NodeKey;
#[node_key] pub const WHEEL_HANDLE: NodeKey;
#[node_key] pub const PADDING_SQUARE: NodeKey;
#[node_key] pub const COLOR_PICKER_CONTAINER: NodeKey;

pub trait ColorPickerUi {
    fn place_color_picker(&mut self, color_picker: &mut ColorPicker);
}
impl ColorPickerUi for Ui {
    fn place_color_picker(&mut self, color_picker: &mut ColorPicker) {
        // Interaction. Uses last frame's node geometry (positions are only known after layout).
        let cursor = self.cursor_position();

        if self.is_dragged(OKLAB_HUE_WHEEL).is_some() {
            if let Some(node) = self.get_node(OKLAB_HUE_WHEEL) {
                let center = node.center();
                let angle = (cursor.x - center.x).atan2(cursor.y - center.y);
                color_picker.oklch_color.hue = angle;
                color_picker.need_rerender = true;
            }
        }

        if self.is_held(OKLAB_SQUARE).is_some() {
            if let Some(node) = self.get_node(OKLAB_SQUARE) {
                let size = node.rect().size();
                let bottom_left = node.bottom_left();
                let frac_x = (cursor.x - bottom_left.x) / size.x;
                // bottom_left.y is the bottom edge (larger y), so this is 0 at the bottom, 1 at the top.
                let frac_y = (bottom_left.y - cursor.y) / size.y;

                color_picker.oklch_color.chroma = (frac_y.clamp(0.0, 1.0)) * 0.33;
                color_picker.oklch_color.lightness = frac_x.clamp(0.0, 1.0);
                color_picker.need_rerender = true;
            }
        }

        // Layout.
        let container = KERU_PANEL
            .key(COLOR_PICKER_CONTAINER)
            .size_x(Size::Frac(0.18))
            .size_y(Size::AspectRatio(1.0));

        let oklab_hue_wheel = CUSTOM_RENDERED_PANEL
            .custom_render(true)
            .size_symm(Size::Fill)
            .shape(Shape::Ring { width: RING_WIDTH })
            .sense_drag(true)
            .key(OKLAB_HUE_WHEEL);

        // The handle is a radial segment (with rounded caps) spanning the ring band at the selected
        // hue. The handle node fills the wheel, so segment coords are fractions of the wheel rect
        // (0..1, y down). The ring goes from its outer edge (frac 0.5 from center) inward by
        // RING_WIDTH; `inner_frac` converts that pixel width using last frame's geometry.
        let inner_frac = match self.get_node(OKLAB_HUE_WHEEL) {
            Some(node) => 0.5 - RING_WIDTH / node.rect().size().x,
            None => 0.4,
        };
        let hue = color_picker.oklch_color.hue;
        // Matches the shader/drag convention: angle measured from +y (down), x to the right.
        let (dx, dy) = (hue.sin(), hue.cos());
        let wheel_handle = PANEL
            .key(WHEEL_HANDLE)
            .size_symm(Size::Fill)
            .color(Color::WHITE)
            .stroke(4.0)
            .shape(Shape::Segment {
                start: (0.5 + inner_frac * dx, 0.5 + inner_frac * dy),
                end: (0.5 + 0.5 * dx, 0.5 + 0.5 * dy),
                dash_length: None,
            });

        let padding_square = PANEL
            .key(PADDING_SQUARE)
            .color(NEUTRAL_GREY)
            .size_symm(Size::Fill)
            // Inset the square so its corners sit just inside the hue ring (ring is RING_WIDTH px wide).
            .padding(RING_WIDTH + 8.0);

        let oklab_square = CUSTOM_RENDERED_PANEL
            .custom_render(true)
            .shape(Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: 0.0 })
            .size_symm(Size::Frac(0.7071))
            .sense_hold(true)
            .key(OKLAB_SQUARE);

        let ring_y = (1.0 - color_picker.oklch_color.chroma / 0.33).clamp(0.0, 1.0);
        let ring_x = color_picker.oklch_color.lightness.clamp(0.0, 1.0);
        let small_ring = PANEL
            .key(SMALL_RING)
            .size_symm(Size::Pixels(5.0))
            .color(Color::WHITE)
            .shape(Shape::Circle)
            .position_x(Pos::Frac(ring_x))
            .position_y(Pos::Frac(ring_y));

        self.add(container).nest(|| {
            self.add(padding_square).nest(|| {
                self.add(oklab_square).nest(|| {
                    self.add(small_ring);
                });
            });
            self.add(oklab_hue_wheel).nest(|| {
                self.add(wheel_handle);
            });
        });
    }
}

impl ColorPicker {
    pub fn new(ctx: &Context) -> ColorPicker {
        return ColorPicker {
            oklch_color: OkLchColor {
                lightness: 0.75,
                chroma: 0.1254,
                hue: 0.3,
            },
            renderer: ColorPickerRenderer::new(ctx),
            need_rerender: true,
        };
    }

    /// Called from the custom render loop for each `CustomRenderingArea` command.
    pub fn render_custom(
        &self,
        render_pass: &mut RenderPass,
        key: NodeKey,
        rect: XyRect,
        window_size: [f32; 2],
    ) {
        let c = self.oklch_color;
        let hcl = [c.hue, c.chroma, c.lightness];
        if key == OKLAB_HUE_WHEEL {
            self.renderer.draw(render_pass, rect, hcl, window_size, 0);
        } else if key == OKLAB_SQUARE {
            self.renderer.draw(render_pass, rect, hcl, window_size, 1);
        }
    }
}
