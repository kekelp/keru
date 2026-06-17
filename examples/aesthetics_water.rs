use std::f32::consts::PI;
use std::time::Instant;

use keru::*;
use keru::node_library::*;
use keru::keru_text::parley::{FontFamily, FontFamilyName};

const WATER_TEAL: Color = Color::from_hex_str("#4ab8c8");
const TEXT_COLOR: Color = Color::from_hex_str("#cceeff");

const GRAD_BG: LinearGradient = LinearGradient {
    color_start: Color::from_hex_str("#0a1a2e"),
    color_end: Color::from_hex_str("#0d3b55"),
    angle_deg: PI / 2.0,
};

const GRAD_BUTTON: LinearGradient = LinearGradient {
    color_start: Color::from_hex_str("#1a4a6e").with_alpha(0.6),
    color_end: Color::from_hex_str("#0d2a42").with_alpha(0.6),
    angle_deg: PI / 2.0,
};

const GRAD_STROKE: LinearGradient = LinearGradient {
    color_start: Color::from_hex_str("#6ad4f0"),
    color_end: Color::from_hex_str("#2a7a9a").with_alpha(-0.4),
    angle_deg: PI / 2.0,
};

struct State {}

struct ButtonState {
    clicks: Vec<(Instant, (f32, f32))>,
}

impl Default for ButtonState {
    fn default() -> Self {
        Self { clicks: Vec::new() }
    }
}

struct WaterButton<'a> {
    text: &'a str,
    key: Option<ComponentKey<WaterButton<'a>>>,
}

impl<'a> WaterButton<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { text, key: None }
    }

    pub fn key(mut self, key: ComponentKey<WaterButton<'a>>) -> Self {
        self.key = Some(key);
        self
    }

    #[node_key] const CLICK_AREA: NodeKey;
}

impl<'a> Component for WaterButton<'a> {
    type State = ButtonState;
    type AddResult = ();
    type ComponentOutput = bool;

    fn component_key(&self) -> Option<ComponentKey<Self>> {
        self.key
    }

    fn add_to_ui(&mut self, ui: &mut Ui, state: &mut ButtonState) -> Self::AddResult {

        if let Some(click) = ui.clicked_at(Self::CLICK_AREA) {
            state.clicks.push((click.timestamp, (click.relative_position.x, click.relative_position.y)));
        }

        let hovered = ui.is_hovered(Self::CLICK_AREA);

        let ripple_duration = 2.0;
        state.clicks.retain(|(t, _)| t.elapsed().as_secs_f32() < ripple_duration);

        let make_ripple = |click_t: f32, pos: (f32, f32), i: u32| -> Option<_> {
            let phase = i as f32 * 0.07;
            let t_ring = (click_t - phase).max(0.0) / (1.0 - phase);
            if t_ring <= 0.0 { return None; }

            let t_eased = 1.0 - (1.0 - t_ring).powi(2);

            let alpha = (1.0 - t_eased) * 0.9;
            let size = 30.0 + t_eased * 480.0;
            let blur = 6.0 + i as f32 * 5.0;

            let angle = PI * 0.3 + i as f32 * 0.4;
            let ripple_grad = LinearGradient {
                color_start: Color::from_hex_str("#f8f8ff").with_alpha(alpha),
                color_end: Color::from_hex_str("#1a6888").with_alpha(alpha * 0.2),
                angle_deg: angle * 180.0 / PI,
            };

            let size_symm = PANEL
                .linear_gradient(ripple_grad)
                .anchor_symm(Anchor::Center)
                .position_x(Pos::Frac(pos.0))
                .position_y(Pos::Frac(pos.1))
                .absorbs_clicks(false)
                .sense_time(true)
                .shape(Shape::Ring { width: 4.0 })
                .blur(blur)
                .size_symm(Size::Pixels(size));
            
            Some(size_symm)
        };

        // Soft ambient glow that pulses slowly when hovered
        let glow_alpha = if hovered { 0.18 } else { 0.06 };
        let glow = DEFAULT
            .color(WATER_TEAL.with_alpha(glow_alpha))
            .anchor_symm(Anchor::Center)
            .position_x(Pos::Frac(0.5))
            .position_y(Pos::Frac(0.5))
            .absorbs_clicks(false)
            .blur(24.0)
            .size_symm(Size::Frac(1.2));

        let button = LABEL
            .linear_gradient(GRAD_BUTTON)
            .stroke(2.0)
            .stroke_linear_gradient(GRAD_STROKE)
            .size_x(Size::Pixels(380.0))
            .size_y(Size::Pixels(90.0))
            .shape(Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: 8.0 })
            .padding(20.0)
            .sense_hover_enter_or_exit(true)
            .sense_click(true)
            .clip_children(true)
            .key(Self::CLICK_AREA);

        let text = TEXT
            .text(self.text)
            .text_selectable(false)
            .position_x(Pos::Center);

        ui.add(button).nest(|| {
            ui.add(glow);
            for (t, pos) in &state.clicks {
                let click_t = (t.elapsed().as_secs_f32() / ripple_duration).min(1.0);
                if let Some(r) = make_ripple(click_t, *pos, 0) { ui.add(r); }
                if let Some(r) = make_ripple(click_t, *pos, 1) { ui.add(r); }
            }
            ui.add(text);
        });
    }

    fn run_component(ui: &mut Ui) -> Self::ComponentOutput {
        ui.is_clicked(Self::CLICK_AREA)
    }
}


fn update_ui(_state: &mut State, ui: &mut Ui) {
    if ui.current_frame() == 1 {
        ui.default_text_style_mut().font_family = FontFamily::Single(FontFamilyName::Generic(keru_draw::parley::GenericFamily::SansSerif));
        ui.default_text_style_mut().font_size = 36.0;
        ui.default_text_style_mut().brush = ColorBrush(TEXT_COLOR.to_u8_array());
    }

    let background = PANEL
        .linear_gradient(GRAD_BG)
        .size_symm(Size::Frac(1.0))
        .absorbs_clicks(false);

    ui.add(background);

    #[component_key] const DIVE: ComponentKey<WaterButton<'_>>;
    #[component_key] const SURFACE: ComponentKey<WaterButton<'_>>;
    #[component_key] const CURRENT: ComponentKey<WaterButton<'_>>;

    let stack = V_STACK
        .size_x(Size::Frac(0.5))
        .position_x(Pos::Frac(0.5))
        .position_y(Pos::Frac(0.5))
        .anchor_x(Anchor::Center)
        .anchor_y(Anchor::Center)
        .stack_spacing(30.0);

    ui.add(stack).nest(|| {
        ui.add_component(WaterButton::new("Water Button 1").key(DIVE));
        ui.add_component(WaterButton::new("Water Button 2").key(SURFACE));
        ui.add_component(WaterButton::new("Water Button 3").key(CURRENT));
    });

    if ui.run_component(DIVE)    { println!("1"); }
    if ui.run_component(SURFACE) { println!("2"); }
    if ui.run_component(CURRENT) { println!("3"); }
}

fn main() {
    let state = State {};
    example_window_loop::run_example_loop(state, update_ui);
}
