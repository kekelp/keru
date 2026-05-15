use std::f32::consts::PI;
use std::time::Instant;

use keru::*;
use keru::node_library::*;
use keru::keru_text::parley::{FontFamily, FontFamilyName, GenericFamily};

const RED: Color = Color::from_hex_str("#f26032");
const TEXT_COLOR: Color = Color::from_hex_str("#f26032");

const GRAD1: Gradient = Gradient {
    color_start: Color::from_hex_str("#f26032"),
    color_end: Color::from_hex_str("#57360e"),
    gradient_type: keru_draw::GradientType::Linear,
    angle: PI / 2.0,
};
const GRAD2: Gradient = Gradient {
    color_start: Color::from_hex_str("#f37c5d"),
    color_end: Color::from_hex_str("#d34425").with_alpha(-0.8),
    gradient_type: keru_draw::GradientType::Linear,
    angle: 0.45 * PI,
};

struct State {
    start: Instant,
}

struct ButtonState {
    last_click: Option<Instant>,
    click_pos: (f32, f32),
}

impl Default for ButtonState {
    fn default() -> Self {
        Self { last_click: None, click_pos: (0.5, 0.5) }
    }
}

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
    type State = ButtonState;
    type AddResult = ();
    type ComponentOutput = bool;

    fn component_key(&self) -> Option<ComponentKey<Self>> {
        self.key
    }

    fn add_to_ui(&mut self, ui: &mut Ui, state: &mut ButtonState) -> Self::AddResult {

        if let Some(click) = ui.clicked_at(Self::CLICK_AREA) {
            state.last_click = Some(click.timestamp);
            state.click_pos = (click.relative_position.x, click.relative_position.y);
        }

        let click_t = state.last_click.map(|t| {
            (t.elapsed().as_secs_f32() / 0.25).min(1.0)
        });

        let hovered = ui.is_hovered(Self::CLICK_AREA);
        
        // The hover animation is fully stateless, and could be done without adding state to the component.
        let base_width = 270.0;
        let hover_circle_size = if hovered {
            Size::Pixels(base_width + 30.0)
        } else {
            Size::Pixels(-10.0)
        };

        let click_ripple = click_t.filter(|&t| t < 1.0).map(|t| {
            let blink = |center: f32| (-(( t - center) / 0.04).powi(2)).exp();
            let base_alpha = (1.0 - t) * 0.85;
            let ripple_alpha = (base_alpha - blink(0.3) - blink(0.6)).max(0.0);
            let ripple_size = 20.0 + t * 550.0;
            PANEL
                .color(Color::from_hex_str("#ffccaa").with_alpha(ripple_alpha))
                .anchor_symm(Anchor::Center)
                .position_x(Pos::Frac(state.click_pos.0))
                .position_y(Pos::Frac(state.click_pos.1))
                .absorbs_clicks(false)
                // Since this animation is driven manually by our update function, we need to mark this node as "sensitive to the passage of time".
                // With this setting turned on, the Ui will know that as long as this node is visible, it will have to keep rerunning the update function on every frame, and prevent the window loop from going idle.
                // (Actually, the Ui doesn't directly control the window loop. This setting will just cause [Ui::should_request_redraw()] to return `true`),
                // and it's up to us or to the `run_example_loop` helper to only call `winit::window::Window::request_redraw()` only when it's true.)
                // (...actually, in this example, the 3D wireframe will keep the loop awake either way. See the `loop_control.rs` example to see this in action.)
                .sense_time(true)
                .shape(Shape::Circle)
                .static_image(include_bytes!("assets/noise.jpg"))
                .image_options(ImageOptions {
                    nine_slice: None,
                    tile_x: TileMode::Tile,
                    tile_y: TileMode::Tile,
                })
                .size_symm(Size::Pixels(ripple_size))
        });

        let circle = DEFAULT
            .color(RED.with_alpha(0.4))
            .animate_position(true)
            .anchor_symm(Anchor::Center)
            .absorbs_clicks(false)
            .static_image(include_bytes!("assets/noise.jpg"))
            .image_options(ImageOptions {
                nine_slice: None,
                tile_x: TileMode::Tile,
                tile_y: TileMode::Tile,
            })
            .size_symm(hover_circle_size);


        let button = LABEL
            .gradient(GRAD1.with_alpha(0.4))
            .stroke(3.0)
            .stroke_gradient(GRAD2)
            .size_x(Size::Pixels(base_width))
            .size_y(Size::Pixels(50.0))
            .shape(Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: 0.0 })
            .padding(12.0)
            .sense_hover_enter_or_exit(true)
            .sense_click(true)
            .clip_children(true)
            .animate_position(true)
            .static_image(include_bytes!("assets/noise.jpg"))
            .image_options(ImageOptions {
                nine_slice: None,
                tile_x: TileMode::Tile,
                tile_y: TileMode::Tile,
            })
            .key(Self::CLICK_AREA);

            let text = TEXT
                .text(self.text)
                .text_selectable(false)
                .position_x(Pos::Start);

        ui.add(button).nest(|| {
            ui.add(circle);
            if let Some(ripple) = click_ripple { ui.add(ripple); }
            ui.add(text);
        });
    }

    fn run_component(ui: &mut Ui) -> Self::ComponentOutput {
        ui.is_clicked(Self::CLICK_AREA)
    }
}


fn update_ui(state: &mut State, ui: &mut Ui) {
    // The simplified example loop doesn't have a nice way to run code at setup only...
    if ui.current_frame() == 1 {
        ui.default_text_style_mut().font_family = FontFamily::Single(FontFamilyName::Generic(GenericFamily::Serif));
        ui.default_text_style_mut().font_size = 30.0;
        ui.default_text_style_mut().brush = ColorBrush(TEXT_COLOR.to_u8_array());
    }

    let background = IMAGE
        .shape(Shape::HexGrid { lattice_size: 20.0, offset: (0.0, 0.0), line_thickness: 2.0 })
        .gradient(GRAD1.with_alpha(0.6))
        .static_image(include_bytes!("assets/noise.jpg"))
        .image_options(ImageOptions {
            nine_slice: None,
            tile_x: TileMode::Tile,
            tile_y: TileMode::Tile,
        })
        .size_symm(Size::Frac(0.6));

    ui.add(background);

    #[component_key] const 決定: ComponentKey<Button<'_>>; 
    #[component_key] const ログ: ComponentKey<Button<'_>>; 
    #[component_key] const 設定: ComponentKey<Button<'_>>; 

    let left_vstack = V_STACK.size_x(Size::Frac(0.3)).position_x(Pos::Frac(0.07)).stack_spacing(20.0).animate_position(true);
    ui.add(left_vstack).nest(|| {
        ui.add_component(Button::new("決定 \\\\ Enter").key(決定));
        ui.add_component(Button::new("ログ \\\\ Log").key(ログ));
        ui.add_component(Button::new("設定 \\\\ Settings").key(設定));
    });

    if ui.run_component(決定) {
        println!("決定");
    }
    if ui.run_component(ログ) {
        println!("ログ");
    }
    if ui.run_component(設定) {
        println!("設定");
    }

    #[node_key] const WIREFRAME_CANVAS: NodeKey;
    let canvas_container = CONTAINER
        .size_symm(Size::Pixels(520.0))
        .padding(0.0)
        .position_x(Pos::Frac(0.62))
        .position_y(Pos::Frac(0.5))
        .anchor_x(Anchor::Center)
        .anchor_y(Anchor::Center)
        .sense_time(true)
        .key(WIREFRAME_CANVAS);

    ui.add(canvas_container);

    let t = state.start.elapsed().as_secs_f32();

    ui.canvas_drawing(WIREFRAME_CANVAS, |renderer| {
        use keru_draw::{Segment, ColorFill};

        let s = 0.5f32;
        let verts: [[f32; 3]; 8] = [
            [-s, -s, -s], [ s, -s, -s], [ s,  s, -s], [-s,  s, -s],
            [-s, -s,  s], [ s, -s,  s], [ s,  s,  s], [-s,  s,  s],
        ];
        let edges: [(usize, usize); 12] = [
            (0, 1), (1, 2), (2, 3), (3, 0),
            (4, 5), (5, 6), (6, 7), (7, 4),
            (0, 4), (1, 5), (2, 6), (3, 7),
        ];

        let ay = t * 0.8;
        let ax = 0.45f32;
        let (cy, sy) = (ay.cos(), ay.sin());
        let (cx, sx) = (ax.cos(), ax.sin());

        let center = 260.0f32;
        let scale = 150.0f32;
        let fov = 3.0f32;

        let project = |v: [f32; 3]| -> [f32; 2] {
            let x1 =  cy * v[0] + sy * v[2];
            let y1 = v[1];
            let z1 = -sy * v[0] + cy * v[2];
            let x2 = x1;
            let y2 = cx * y1 - sx * z1;
            let z2 = sx * y1 + cx * z1;
            let d = fov + z2;
            let px = center + x2 * scale * fov / d;
            let py = center + y2 * scale * fov / d;
            [px, py]
        };

        let noise_period = 3.0f32;
        let phase = t % noise_period;
        let blink = |start: f32| if phase >= start && phase < start + 0.05 { 1.0 } else { 0.0 };
        let noise_alpha = (0.6f32 - blink(0.0) - blink(0.1)).max(0.0);

        for (a, b) in edges {
            let p0 = project(verts[a]);
            let p1 = project(verts[b]);
            let color = RED.with_alpha(noise_alpha);
            renderer.draw_segment(Segment {
                start: p0,
                end: p1,
                thickness: 3.0,
                fill: ColorFill::Color(color),
                dash_length: None,
                dash_offset: 0.0,
                blur: 0.0,
                texture: None,
                texture_options: None,
            });
        }
    });

}

fn main() {
    let state = State { start: Instant::now() };
    example_window_loop::run_example_loop(state, update_ui);
}


