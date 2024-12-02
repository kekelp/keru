use basic_window_loop::Context;
use blue::*;
use blue::Size::*;
use blue::Len::*;
use wgpu::RenderPass;
use crate::color_picker_render::ColorPickerRenderRect;
use crate::oklab::*;

use wgpu::BindGroup;
use wgpu::Buffer;
use wgpu::RenderPipeline;

use crate::paint_ui::FLGR_PANEL;

pub struct ColorPickerRenderer {
    pub vertex_buffer: Buffer,
    pub render_pipeline: RenderPipeline,
    pub bind_group: BindGroup,
}

pub struct ColorPicker {
    pub oklch_color: OkLchColor,
    pub renderer: ColorPickerRenderer,
    pub need_rerender: bool,
}

const NEUTRAL_GREY: Color = Color::rgba_f(0.09, 0.09, 0.09, 1.0);

#[node_key] pub const OKLAB_HUE_WHEEL: NodeKey;
#[node_key] pub const OKLAB_SQUARE: NodeKey;
#[node_key] pub const SMALL_RING: NodeKey;
#[node_key] pub const PADDING_SQUARE: NodeKey;
#[node_key] const CONTAINER: NodeKey;

pub trait ColorPickerUi {
    fn add_color_picker(&mut self, color_picker: &mut ColorPicker); 
}
impl ColorPickerUi for Ui {
    fn add_color_picker(&mut self, color_picker: &mut ColorPicker) {

        if let Some((_time_held, abs_pos)) = self.is_held(OKLAB_HUE_WHEEL) {
            let center = self.get_node(OKLAB_HUE_WHEEL).unwrap().center();
            let pos = abs_pos - center;
            let angle = pos.x.atan2(pos.y);
            
            color_picker.oklch_color.hue = angle;
            color_picker.need_rerender = true;
        };

        if let Some((_time_held, abs_pos)) = self.is_held(OKLAB_SQUARE) {
            let size_pixels = self.get_node(OKLAB_SQUARE).unwrap().rect().size();
            let bottom_left = self.get_node(OKLAB_SQUARE).unwrap().bottom_left();
            let mut pos = abs_pos - bottom_left;
            pos.y = - pos.y;
            let pos = pos / size_pixels;
            
            color_picker.oklch_color.chroma = pos.y * 0.33;
            color_picker.oklch_color.lightness = pos.x;
            color_picker.need_rerender = true;
        };

        // Wanted reactivity anyway for a laugh?
        // let changed = color_picker.need_rerender;
        // Tell the ui that if `changed` is false, then the whole subtree starting from `CONTAINER` will stay the same as it was before
        // self.get_node(CONTAINER).assume_unchanged_if(!changed);
        // Now we can literally skip the rest of the function (but not the input handling: that's the main reason why we write it above)
        // if ! changed {
        //     return;
        // }
        // todo: actually implement this, and pair it with a nicer signal thing for the example

        self.add(CONTAINER)
            .params(FLGR_PANEL)
            .size_x(Size::Fixed(Frac(0.18)))
            .size_y(Size::AspectRatio(1.0));
        
        self.add(OKLAB_HUE_WHEEL)
            .params(CUSTOM_RENDERED_PANEL)
            .size_symm(Fill)
            .shape(Shape::Ring { width: 60.0 });
    
        self.add(PADDING_SQUARE)
            .params(PANEL)
            .color(NEUTRAL_GREY)
            .size_symm(Fill)
            // .shape(Shape::Rectangle { corner_radius: 0.5 })
            .padding(Pixels((60.0 * 2.0f32.sqrt() / 2.0) as u32));

        self.add(OKLAB_SQUARE)
            .params(CUSTOM_RENDERED_PANEL)
            .shape(Shape::Rectangle { corner_radius: 0.0 })
            .size_symm(Fixed(Frac(0.7071)));

        // let ring_y = 1.0 - color_picker.oklch_color.chroma / 0.33;
        // let ring_x = color_picker.oklch_color.lightness;
        // let ring_y = ring_y.clamp(0.0, 1.0);
        // let ring_x = ring_x.clamp(0.0, 1.0);
        // self.add(SMALL_RING)
        //     .params(PANEL)
        //     .size_symm(Size::Fixed(Pixels(4)))
        //     .color(Color::WHITE)
        //     .shape(Shape::Ring { width: 4.0 })
        //     .position_x(Position::Static(Frac(ring_x)))
        //     .position_y(Position::Static(Frac(ring_y)));

        // layout
        self.place(CONTAINER).nest(|| {
            // order ???
            // this seems like a child ordering issue
            self.place(OKLAB_HUE_WHEEL);
            self.place(PADDING_SQUARE).nest(|| {
                self.place(OKLAB_SQUARE).nest(|| {
                    // self.place(SMALL_RING);
                });
            });
        });

    }
}


impl ColorPicker {
    pub fn new(ctx: &Context, base_uniforms: &Buffer) -> ColorPicker {
        return ColorPicker {
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
        let wheel_info = ui.get_node(OKLAB_HUE_WHEEL)?.render_rect();
        let wheel_rect = ColorPickerRenderRect {
            rect: wheel_info.rect,
            z: wheel_info.z,
            oklch_color: self.oklch_color.into(),
        };

        let square_info = ui.get_node(OKLAB_SQUARE)?.render_rect();
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
    }

    pub fn needs_rerender(&self) -> bool {
        return self.need_rerender;
    }
}