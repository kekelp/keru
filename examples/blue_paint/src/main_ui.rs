use blue::*;
use blue::Position::*;
use blue::Size::*;
use blue::Len::*;

use crate::color_picker::ColorPickerUi;
use crate::PixelColorF32;
use crate::State;

const COLOR1: Color = Color::rgba(50, 13, 100, 240);
const COLOR2: Color = Color::rgba(100, 13, 50, 240);
const GRAD1: VertexColors = VertexColors::diagonal_gradient_forward_slash(COLOR1, COLOR2);
const FLGR_PANEL: NodeParams = PANEL.vertex_colors(GRAD1);

impl State {

    pub fn declare_ui(&mut self) {

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
            .size_x(FitContent);

        self.ui.begin_tree();

        self.ui.place(RIGHT_BAR).nest(|| {
            self.slider_value = self.add_slider(self.slider_value);
            self.ui.add_color_picker(&mut self.color_picker);
        });

        self.ui.place(LEFT_BAR).nest(|| {
            self.add_pixel_info_ui();
            self.add_tools();
        });

        self.ui.finish_tree();
    }
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

        #[node_key] const PIXEL_PANEL2: NodeKey;
        self.ui.add(PIXEL_PANEL2)
            .params(FLGR_PANEL)
            .position_x(Start)
            .position_y(Start)
            .size_x(FitContentOrMinimum(Pixels(100)));

        self.ui.place(PIXEL_PANEL2).nest(|| {
            self.ui.v_stack().nest(|| {
                self.ui.text(&x);
                self.ui.text(&y);
            });
        });
    }

    pub fn add_tools(&mut self) {
        #[node_key] const BRUSH: NodeKey;
        self.ui.add(BRUSH).params(ICON_BUTTON).static_image(include_bytes!("icons/brush.png"));

        #[node_key] const ERASER: NodeKey;
        self.ui.add(ERASER).params(ICON_BUTTON).static_image(include_bytes!("icons/eraser.png"));

        #[node_key] const TOOLS_PANEL: NodeKey;
        self.ui.add(TOOLS_PANEL)
            .params(FLGR_PANEL)
            .position_x(Start)
            .position_y(Start)
            .circle()
            .size_x(FitContent);

        self.ui.place(TOOLS_PANEL).nest(|| {
            self.ui.place_h_stack().nest(|| {
                self.ui.v_stack().nest(|| {
                    self.ui.place(BRUSH);
                    self.ui.place(ERASER);
                });
            });
        });

        if self.ui.is_clicked(BRUSH) {
            self.canvas.paint_color = PixelColorF32::new(0.2, 0.8, 0.2, 1.0);
        }

        if self.ui.is_clicked(ERASER) {
            self.canvas.paint_color = PixelColorF32::new(1.0, 1.0, 1.0, 0.0);
        }
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

        let size = self.ui.get_node(SLIDER_CONTAINER).unwrap().get_inner_size().y as f32;

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
