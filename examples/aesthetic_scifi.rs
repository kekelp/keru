use keru::*;
use keru::node_library::*;
use keru::keru_text::parley::{FontFamily, FontFamilyName};

const ORANGE: Color = Color::from_hex(0xee8031);
const RED: Color = Color::from_hex(0xff6702);

struct State {}

struct Button<'a> {
    text: &'a str,
    key: Option<ComponentKey<Button<'a>>>,
}

impl<'a> Button<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            key: None,
        }
    }

    pub fn key(mut self, key: ComponentKey<Button<'a>>) -> Self {
        self.key = Some(key);
        self
    }

    #[node_key] const CLICK_AREA: NodeKey;
}

impl<'a> Component for Button<'a> {
    type State = ();
    type AddResult = ();
    type ComponentOutput = bool;

    fn component_key(&self) -> Option<ComponentKey<Self>> {
        self.key
    }

    fn add_to_ui(&mut self, ui: &mut Ui, _: &mut ()) -> Self::AddResult {

        let hovered = ui.is_hovered(Self::CLICK_AREA);
        
        // This animation is still stateless. 
        // We could also store the hover or click timestamps in the State. Then we could do do whatever we want.
        let base_size = 200.0;
        let circle_size = if let Some(_) = hovered {
            Size::Pixels(base_size + 70.0)
        } else {
            Size::Pixels(-50.0)
        };

        let button_width = if let Some(_) = hovered {
            Size::Pixels(base_size)
        } else {
            Size::Pixels(base_size)
        };

        let circle = DEFAULT
            .shape(Shape::Circle)
            .color(RED.with_alpha(0.4))
            .animate_position(true)
            .anchor_symm(Anchor::Center)
            .absorbs_clicks(false)
            .static_image(include_bytes!("assets/glitch.jpg"))
            .image_options(ImageOptions {
                nine_slice: None,
                tile_x: TileMode::Tile,
                tile_y: TileMode::Tile,
            })
            .size_symm(circle_size);


        let button = LABEL
            .color(ORANGE.with_alpha(0.4))
            .stroke(3.0)
            .stroke_color(ORANGE.with_alpha(0.4))
            .size_x(button_width)
            .size_y(Size::Pixels(50.0))
            .shape(Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: 0.0 })
            .padding(12.0)
            .sense_hover_enter_or_exit(true)
            .sense_click(true)
            .clip_children(true)
            .animate_position(true)

            .static_image(include_bytes!("assets/glitch.jpg"))
            .image_options(ImageOptions {
                nine_slice: None,
                tile_x: TileMode::Tile,
                tile_y: TileMode::Tile,
            })

            .key(Self::CLICK_AREA);

        ui.add(button).nest(|| {
            ui.add(circle);
            ui.add(TEXT.text(self.text));
        });
    }

    fn run_component(ui: &mut Ui) -> Self::ComponentOutput {
        ui.is_clicked(Self::CLICK_AREA)
    }
}


fn update_ui(_: &mut State, ui: &mut Ui) {
    // The simplified example loop doesn't have a nice way to run code at setup only...
    if ui.current_frame() == 1 {
        let a = ui.load_font(include_bytes!("assets/airstrikegrad.ttf"));
        ui.default_text_style_mut().font_family = FontFamily::Single(FontFamilyName::Named("Airstrike Gradient".into()));
        dbg!(a);
        ui.default_text_style_mut().font_size = 32.0;
        ui.default_text_style_mut().brush = ColorBrush(ORANGE.to_u8_array());
    }

    let background = IMAGE
        .shape(Shape::Circle)
        .shape(Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: 10.0 })
        .shape(Shape::HexGrid { lattice_size: 20.0, offset: (0.0, 0.0), line_thickness: 1.0 })
        .color(Color::from_hex(0xf14845).with_alpha(1.0))
        .static_image(include_bytes!("assets/glitch.jpg"))
        .image_options(ImageOptions {
            nine_slice: None,
            tile_x: TileMode::Tile,
            tile_y: TileMode::Tile,
        })
        .size_symm(Size::Frac(0.6));

    ui.add(background);

    #[component_key] const BUTTON1:ComponentKey<Button<'_>>; 
    #[component_key] const BUTTON2:ComponentKey<Button<'_>>; 
    #[component_key] const BUTTON3:ComponentKey<Button<'_>>; 

    let left_vstack = V_STACK.size_x(Size::Frac(0.3)).position_x(Pos::Frac(0.07)).stack_spacing(15.0).animate_position(true);
    ui.add(left_vstack).nest(|| {
        ui.add_component(Button::new("登録して").key(BUTTON1));
        ui.add_component(Button::new("函数の函数").key(BUTTON2));
        ui.add_component(Button::new("類群の指標").key(BUTTON3));
    });

    if ui.run_component(BUTTON1) {
        println!("Sneed1");
    }
    if ui.run_component(BUTTON2) {
        println!("Sneed2");
    }
    if ui.run_component(BUTTON3) {
        println!("Sneed3");
    }

}

fn main() {
    let state = State {};
    example_window_loop::run_example_loop(state, update_ui);
}


