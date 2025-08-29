use basic_window_loop::Context;
use glam::Vec2;
use keru::*;
use wgpu::RenderPass;
use crate::color_picker_render::ColorPickerRenderRect;
use crate::oklab::*;

use wgpu::BindGroup;
use wgpu::Buffer;
use wgpu::RenderPipeline;

use crate::paint_ui::KERU_PANEL;

pub struct ColorPickerRenderer {
    pub vertex_buffer: Buffer,
    pub render_pipeline: RenderPipeline,
    pub bind_group: BindGroup,
}

pub struct ColorPicker {
    pub key: NodeKey,
    pub oklch_color: OkLchColor,
    pub renderer: ColorPickerRenderer,
    pub need_rerender: bool,
}

const NEUTRAL_GREY: Color = Color::rgba_f(0.09, 0.09, 0.09, 1.0);

#[node_key] pub const OKLAB_HUE_WHEEL: NodeKey;
#[node_key] pub const OKLAB_SQUARE: NodeKey;
#[node_key] pub const SMALL_RING: NodeKey;
#[node_key] pub const PADDING_SQUARE: NodeKey;
#[node_key] pub const COLOR_PICKER_CONTAINER: NodeKey;

pub trait ColorPickerUi {
    fn place_color_picker(&mut self, color_picker: &mut ColorPicker); 
}
impl ColorPickerUi for Ui {
    fn place_color_picker(&mut self, color_picker: &mut ColorPicker) {
        self.named_subtree(color_picker.key).start(|| {

            // DVec2 hell
            if let Some(_) = self.is_held(OKLAB_HUE_WHEEL) {
                let abs_pos: Vec2 = self.cursor_position().as_vec2();
                let center: Vec2 = self.center(OKLAB_HUE_WHEEL).unwrap().into();
                let pos: Vec2 = abs_pos - center;
                let angle = pos.x.atan2(pos.y);
                
                color_picker.oklch_color.hue = angle;
                color_picker.need_rerender = true;
            };

            if let Some(_) = self.is_held(OKLAB_SQUARE) {
                let abs_pos: Vec2 = self.cursor_position().as_vec2();
                let size_pixels: Vec2 = self.rect(OKLAB_SQUARE).unwrap().size().into();
                let bottom_left: Vec2 = self.bottom_left(OKLAB_SQUARE).unwrap().into();
                let mut pos: Vec2 = abs_pos - bottom_left;
                pos.y = - pos.y;
                let pos: Vec2 = pos / size_pixels;
                
                color_picker.oklch_color.chroma = pos.y * 0.33;
                color_picker.oklch_color.lightness = pos.x;
                color_picker.need_rerender = true;
            };

            let container = KERU_PANEL
                .key(COLOR_PICKER_CONTAINER)
                .size_x(Size::Frac(0.18))
                .size_y(Size::AspectRatio(1.0));
            
            let oklab_hue_wheel = CUSTOM_RENDERED_PANEL
                .size_symm(Size::Fill)
                .shape(Shape::Ring { width: 60.0 })
                .sense_hold(true)
                .key(OKLAB_HUE_WHEEL);
        
            let padding_square = PANEL
            .key(PADDING_SQUARE)
                .color(NEUTRAL_GREY)
                .size_symm(Size::Fill)
                // .shape(Shape::Rectangle { corner_radius: 0.5 })
                .padding((60.0 * 2.0f32.sqrt() / 2.0) as u32);

            let oklab_square = CUSTOM_RENDERED_PANEL
                .shape(Shape::Rectangle { corner_radius: 0.0 })
                .size_symm(Size::Frac(0.7071))
                .sense_hold(true)
                .key(OKLAB_SQUARE);


            let ring_y = 1.0 - color_picker.oklch_color.chroma / 0.33;
            let ring_x = color_picker.oklch_color.lightness;
            let ring_y = ring_y.clamp(0.0, 1.0);
            let ring_x = ring_x.clamp(0.0, 1.0);
            let small_ring = PANEL
                .key(SMALL_RING)
                .size_symm(Size::Pixels(5))
                .color(Color::WHITE)
                .shape(Shape::Circle)
                .position_x(Position::Static(Len::Frac(ring_x)))
                .position_y(Position::Static(Len::Frac(ring_y)));

            // layout
            self.add(container).nest(|| {
                self.add(padding_square).nest(|| {
                    self.add(oklab_square).nest(|| {
                        self.add(small_ring);
                    });
                });
                self.add(oklab_hue_wheel);
            });
        });

    }
}


impl ColorPicker {
    pub fn new(key: NodeKey, ctx: &Context, base_uniforms: &Buffer) -> ColorPicker {
        return ColorPicker {
            key,
            oklch_color: OkLchColor {
                lightness: 0.75,
                chroma: 0.1254,
                hue: 0.3,
            },
            need_rerender: true,
            renderer: ColorPickerRenderer::new(ctx, base_uniforms),
        }
    }

    pub fn render(&mut self, render_pass: &mut RenderPass) {
        render_pass.set_pipeline(&self.renderer.render_pipeline);
        render_pass.set_vertex_buffer(0, self.renderer.vertex_buffer.slice(..));
        render_pass.set_bind_group(0, &self.renderer.bind_group, &[]);
        render_pass.draw(0..4, 0..2);

        self.need_rerender = false;
    }

    pub fn prepare(&self, ui: &mut Ui, queue: &wgpu::Queue) -> Option<()> {
        ui.named_subtree(self.key).start(|| {
            let wheel_info = ui.render_rect(OKLAB_HUE_WHEEL)?;
            let wheel_rect = ColorPickerRenderRect {
                rect: wheel_info.rect,
                z: wheel_info.z,
                oklch_color: self.oklch_color.into(),
            };
            
            let square_info = ui.render_rect(OKLAB_SQUARE)?;
            let square_rect = ColorPickerRenderRect {
                rect: square_info.rect,
                z: square_info.z,
                oklch_color: self.oklch_color.into(),
            };
            
            // to keep the rust-side boilerplate to a minimum, we use the same pipeline for all rects (wheel and main square) and have the shader do different things based on the instance index.
            // this means that the order here matters.
            let coords = [wheel_rect, square_rect];
            
            queue.write_buffer(&self.renderer.vertex_buffer, 0, bytemuck::cast_slice(&coords));
            
            return Some(());
        })
    }

    pub fn needs_rerender(&self) -> bool {
        return self.need_rerender;
    }

    // todo: in the future, these should give the range between the farthest and closest custom rendered nodes. right now the z layering is dumb so it's not possible (the hue wheel is closer than the small_ring/dot). But if we do that we eliminate dumb stuff like UI quads things being right on the edge (because the edges are things that the ui doesn't render)
    pub(crate) fn z_range(&self, ui: &mut Ui) -> Option<[f32; 2]> {
        ui.named_subtree(self.key).start(|| {
            let bg = ui.render_rect(PADDING_SQUARE)?;
            let small_ring = ui.render_rect(SMALL_RING)?;
            return Some([bg.z, small_ring.z])
        })
    }
}