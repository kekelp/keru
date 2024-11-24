use blue::*;
use blue::Size::*;
use blue::Len::*;

use blue::{Ui, XyRect};
use wgpu::BindGroup;
use wgpu::Buffer;
use wgpu::RenderPipeline;

use crate::main_ui::FLGR_PANEL;

// Struct that holds the render pipeline and a buffer for rectangle vertices
pub struct ColorPicker {
    pub(crate) vertex_buffer: Buffer,
    pub(crate) render_pipeline: RenderPipeline,
    pub(crate) bind_group: BindGroup,
    pub coords: [XyRect; 1],
}

const NEUTRAL_GREY: Color = Color::rgba_f(0.09, 0.09, 0.09, 1.0);

impl ColorPicker {
    #[node_key] pub const OKLAB_HUE_WHEEL: NodeKey;
    #[node_key] pub const OKLAB_SQUARE: NodeKey;
    #[node_key] pub const PADDING_SQUARE: NodeKey;
    #[node_key] const PANEL: NodeKey;
}

pub trait ColorPickerUi {
    fn add_color_picker(&mut self, color_picker: &mut ColorPicker); 
}
impl ColorPickerUi for Ui {
    fn add_color_picker(&mut self, _color_picker: &mut ColorPicker) {
        self.add(ColorPicker::PANEL)
            .params(FLGR_PANEL)
            .size_x(Size::Fixed(Frac(0.18)))
            .size_y(Size::AspectRatio(1.0));
        
        self.add(ColorPicker::OKLAB_HUE_WHEEL)
            .params(CUSTOM_RENDERED_PANEL)
            .size_symm(Fill)
            .shape(Shape::Ring { width: 60.0 });
    
        self.add(ColorPicker::PADDING_SQUARE)
            .params(PANEL)
            .color(NEUTRAL_GREY)
            .size_symm(Fill)
            // .shape(Shape::Rectangle { corner_radius: 0.5 })
            .padding(Pixels((60.0 * 2.0f32.sqrt() / 2.0) as u32));

        self.add(ColorPicker::OKLAB_SQUARE)
            .params(CUSTOM_RENDERED_PANEL)
            // .shape(Shape::Rectangle { corner_radius: 0.0 })
            .size_symm(Fixed(Frac(0.7071)));
    
        self.place(ColorPicker::PANEL).nest(|| {
            self.place(ColorPicker::OKLAB_HUE_WHEEL);
            self.place(ColorPicker::PADDING_SQUARE).nest(|| {
                self.place(ColorPicker::OKLAB_SQUARE);
            });
        });

        // let clicks = self.get_clicks(MouseButton::Left, ColorPicker::OKLAB_HUE_WHEEL);
        // for c in clicks {
        //     println!(" {:?}", c);
        // }
    }
}