use blue::*;
use blue::Size::*;
use blue::Len::*;
use crate::oklab::*;

use wgpu::BindGroup;
use wgpu::Buffer;
use wgpu::RenderPipeline;

use crate::paint_ui::FLGR_PANEL;

pub struct ColorPicker {
    pub(crate) vertex_buffer: Buffer,
    pub(crate) render_pipeline: RenderPipeline,
    pub(crate) bind_group: BindGroup,
    pub(crate) oklch_color: OkLchColor,
}

const NEUTRAL_GREY: Color = Color::rgba_f(0.09, 0.09, 0.09, 1.0);

#[node_key] pub const OKLAB_HUE_WHEEL: NodeKey;
#[node_key] pub const OKLAB_SQUARE: NodeKey;
#[node_key] pub const PADDING_SQUARE: NodeKey;
#[node_key] const CONTAINER: NodeKey;

pub trait ColorPickerUi {
    fn add_color_picker(&mut self, color_picker: &mut ColorPicker); 
}
impl ColorPickerUi for Ui {
    fn add_color_picker(&mut self, color_picker: &mut ColorPicker) {

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
            // .shape(Shape::Rectangle { corner_radius: 0.0 })
            .size_symm(Fixed(Frac(0.7071)));
    
        self.place(CONTAINER).nest(|| {
            self.place(OKLAB_HUE_WHEEL);
            self.place(PADDING_SQUARE).nest(|| {
                self.place(OKLAB_SQUARE);
            });
        });

        if let Some((_time_held, abs_pos)) = self.is_held(OKLAB_HUE_WHEEL) {
            
            let center = self.get_node(OKLAB_HUE_WHEEL).unwrap().center();
            let pos = abs_pos - center;
            let angle = pos.x.atan2(pos.y);
            
            color_picker.oklch_color.hue = angle;
        };
    }
}