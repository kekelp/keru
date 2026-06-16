use std::f32::consts::PI;

use keru::*;
use keru::node_library::*;

#[node_key] const GRADIENT_SOURCE: NodeKey;
#[node_key] const KNOB_PAN_KEY: NodeKey;
#[node_key] const KNOB_SEND_KEY: NodeKey;
#[node_key] const FADER_KEY: NodeKey;
#[node_key] const FADER_TRACK_KEY: NodeKey;
#[node_key] const MUTE_KEY: NodeKey;
#[node_key] const SOLO_KEY: NodeKey;

const BG: Color = Color::new(0.06, 0.06, 0.10, 1.0);
const STRIP_BG: Color = Color::new(0.10, 0.10, 0.16, 1.0);
const TRACK_BG: Color = Color::new(0.13, 0.13, 0.20, 1.0);
const GROOVE: Color = Color::new(0.18, 0.18, 0.28, 1.0);
const DIM: Color = Color::new(0.45, 0.45, 0.60, 1.0);
const BRIGHT: Color = Color::new(0.88, 0.88, 1.00, 1.0);

const GRAD: LinearGradient = LinearGradient {
    color_start: Color::from_hex_str("#9b5de5"),
    color_end: Color::from_hex_str("#00f5d4"),
    angle_deg: 0.0,
};

const KNOB_START: f32 = PI * 0.75;   // bottom-left  (≈ 7:30)
const KNOB_END: f32 = PI * 2.25;     // bottom-right (≈ 4:30), 270° CCW from start
const KNOB_DRAG_RANGE: f32 = 120.0;


struct Channel {
    name: &'static str,
    gain: f32,
    pan: f32,
    send: f32,
    muted: bool,
    soloed: bool,
}

struct State {
    channels: Vec<Channel>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            channels: vec![
                Channel { name: "KICK",   gain: 0.80, pan: 0.50, send: 0.40, muted: false, soloed: false },
                Channel { name: "SNARE",  gain: 0.70, pan: 0.55, send: 0.55, muted: false, soloed: false },
                Channel { name: "HIHAT",  gain: 0.50, pan: 0.40, send: 0.20, muted: false, soloed: false },
                Channel { name: "BASS",   gain: 0.75, pan: 0.48, send: 0.30, muted: true,  soloed: false },
                Channel { name: "SYNTH",  gain: 0.65, pan: 0.60, send: 0.70, muted: false, soloed: false },
                Channel { name: "PAD",    gain: 0.45, pan: 0.35, send: 0.85, muted: false, soloed: false },
                Channel { name: "VOX",    gain: 0.90, pan: 0.50, send: 0.60, muted: false, soloed: true  },
                Channel { name: "FX BUS", gain: 0.60, pan: 0.50, send: 1.00, muted: false, soloed: false },
            ],
        }
    }
}

// Draw a knob with a drag-able container.
fn knob(ui: &mut Ui, value: f32, size: f32, key: NodeKey) {
    let sweep = KNOB_START + (KNOB_END - KNOB_START) * value;

    let container = CONTAINER
        .key(key)
        .size_symm(Size::Pixels(size))
        .position_x(Pos::Center)
        .sense_drag(true);

    let groove_arc = DEFAULT
        .color(GROOVE)
        .size_symm(Size::Frac(1.0))
        .shape(Shape::Arc { start_angle: KNOB_START, end_angle: KNOB_END, width: size * 0.12 })
        .absorbs_clicks(false);

    let value_arc = DEFAULT
        .shared_gradient(GRADIENT_SOURCE)
        .size_symm(Size::Frac(1.0))
        .shape(Shape::Arc { start_angle: KNOB_START, end_angle: sweep, width: size * 0.12 })
        .absorbs_clicks(false);

    let dot = DEFAULT
        .color(BRIGHT)
        .shape(Shape::Circle)
        .size_symm(Size::Pixels(size * 0.14))
        .position(Pos::Center, Pos::Center)
        .anchor_symm(Anchor::Center)
        .absorbs_clicks(false);

    ui.add(container).nest(|| {
        ui.add(groove_arc);
        ui.add(value_arc);
        ui.add(dot);
    });
}

// A rounded label pill — uses node_gradient when active.
fn badge(ui: &mut Ui, label: &'static str, active: bool, key: NodeKey) {
    let base = DEFAULT
        .key(key)
        .size_x(Size::FitContent)
        .size_y(Size::Pixels(20.0))
        .shape(Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: 10.0 })
        .padding_x(10.0)
        .text(label)
        .text_size(10.0)
        .bold()
        .sense_click(true);

    let pill = if active {
        base.shared_gradient(GRADIENT_SOURCE).text_color(Color::new(0.05, 0.05, 0.10, 1.0))
    } else {
        base.color(GROOVE).text_color(DIM)
    };
    ui.add(pill);
}

fn channel_strip(ui: &mut Ui, ch: &Channel, ch_i: usize) {
    let strip = V_STACK
        .color(STRIP_BG)
        .size_x(Size::Pixels(88.0))
        .size_y(Size::Fill)
        .shape(Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: 10.0 })
        .stack_spacing(10.0)
        .stack_arrange(Arrange::Start)
        .padding(10.0);

    let name_bg = if ch.soloed {
        DEFAULT
            .shared_gradient(GRADIENT_SOURCE)
            .size_x(Size::Fill).size_y(Size::Pixels(22.0))
            .shape(Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: 6.0 })
            .text(ch.name).text_color(Color::new(0.05, 0.05, 0.10, 1.0)).text_size(11.0).bold()
    } else {
        DEFAULT
            .color(TRACK_BG)
            .size_x(Size::Fill).size_y(Size::Pixels(22.0))
            .shape(Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: 6.0 })
            .text(ch.name).text_color(BRIGHT).text_size(11.0).bold()
    };

    let badge_row = H_STACK
        .size_x(Size::Fill)
        .stack_arrange(Arrange::Center)
        .stack_spacing(6.0);

    let fill_frac = ch.gain;

    let fader_fill = DEFAULT
        .shared_gradient(GRADIENT_SOURCE)
        .size_x(Size::Pixels(6.0))
        .size_y(Size::Frac(fill_frac))
        .shape(Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: 3.0 })
        .position_y(Pos::End)
        .anchor_y(Anchor::End)
        .absorbs_clicks(false);

    let fader_thumb = DEFAULT
        .key(FADER_KEY.sibling(ch_i))
        .color(BRIGHT)
        .size_x(Size::Pixels(22.0))
        .size_y(Size::Pixels(14.0))
        .shape(Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: 4.0 })
        .position_x(Pos::Center)
        .position_y(Pos::Frac(1.0 - fill_frac))
        .anchor_symm(Anchor::Center)
        .sense_drag(true);

    let fader_container = PANEL
        .key(FADER_TRACK_KEY.sibling(ch_i))
        .color(GROOVE)
        .size_x(Size::Pixels(6.0))
        .size_y(Size::Pixels(90.0))
        .shape(Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: 3.0 })
        .position_x(Pos::Center)
        .sense_drag(true)
        .sense_click(true)
        .padding(0.0);

    let gain_db = (ch.gain * 12.0 - 6.0) as i32;
    let gain_str = format!("{:+}dB", gain_db);
    let gain_label = TEXT.text(&gain_str).text_color(DIM).text_size(10.0).position_x(Pos::Center);

    let knob_label = |s: &'static str| TEXT.static_text(s).text_color(DIM).text_size(9.0).position_x(Pos::Center);
    let knob_section = V_STACK.size_x(Size::Fill).stack_spacing(2.0).stack_arrange(Arrange::Start);

    ui.add(strip).nest(|| {
        ui.add(name_bg);

        ui.add(badge_row).nest(|| {
            badge(ui, "M", ch.muted, MUTE_KEY.sibling(ch_i));
            badge(ui, "S", ch.soloed, SOLO_KEY.sibling(ch_i));
        });

        ui.add(knob_section).nest(|| {
            ui.add(knob_label("PAN"));
            knob(ui, ch.pan, 46.0, KNOB_PAN_KEY.sibling(ch_i));
        });

        ui.add(knob_section).nest(|| {
            ui.add(knob_label("SEND"));
            knob(ui, ch.send, 46.0, KNOB_SEND_KEY.sibling(ch_i));
        });

        ui.add(fader_container).nest(|| {
            ui.add(fader_fill);
            ui.add(fader_thumb);
        });

        ui.add(gain_label);
    });
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    for (ch_i, ch) in state.channels.iter_mut().enumerate() {
        if let Some(drag) = ui.is_dragged(KNOB_PAN_KEY.sibling(ch_i)) {
            ch.pan = (ch.pan - drag.absolute_delta.y / KNOB_DRAG_RANGE).clamp(0.0, 1.0);
        }
        if let Some(drag) = ui.is_dragged(KNOB_SEND_KEY.sibling(ch_i)) {
            ch.send = (ch.send - drag.absolute_delta.y / KNOB_DRAG_RANGE).clamp(0.0, 1.0);
        }
        if let Some(click) = ui.clicked_at(FADER_TRACK_KEY.sibling(ch_i)) {
            ch.gain = (1.0 - click.relative_position.y as f32).clamp(0.0, 1.0);
        }
        if let Some(drag) = ui.is_dragged(FADER_KEY.sibling(ch_i)) {
            ch.gain = (ch.gain - drag.absolute_delta.y / 90.0).clamp(0.0, 1.0);
        }
        if ui.is_clicked(MUTE_KEY.sibling(ch_i)) {
            ch.muted = !ch.muted;
        }
        if ui.is_clicked(SOLO_KEY.sibling(ch_i)) {
            ch.soloed = !ch.soloed;
        }
    }

    // Node with the invisible gradient
    let gradient_node = DEFAULT
            .key(GRADIENT_SOURCE)
            .linear_gradient(GRAD)
            .invisible()
            .size(Size::Frac(1.0), Size::Frac(1.0));
    ui.add(gradient_node);

    let background = PANEL
        .color(BG)
        .size_symm(Size::Fill)
        .shape(Shape::Rectangle { rounded_corners: RoundedCorners::NONE, corner_radius: 0.0 });

    ui.add(background);

    let header = H_STACK
        .size_x(Size::Fill)
        .size_y(Size::Pixels(44.0))
        .color(STRIP_BG)
        .stack_arrange(Arrange::Start)
        .stack_spacing(12.0)
        .padding(14.0);

    let title = TEXT.static_text("MIXER").text_color(BRIGHT).text_size(14.0).bold();
    let sub = TEXT.static_text("8 channels · 44.1 kHz · 24-bit")
        .text_color(DIM).text_size(11.0)
        .anchor_y(Anchor::Center).position_y(Pos::Center);

    let rec_badge = DEFAULT
        .shared_gradient(GRADIENT_SOURCE)
        .size_x(Size::FitContent).size_y(Size::Pixels(20.0))
        .shape(Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: 10.0 })
        .padding_x(10.0)
        .static_text("● REC")
        .text_color(Color::new(0.05, 0.05, 0.10, 1.0))
        .text_size(10.0).bold()
        .anchor_y(Anchor::Center)
        .position(Pos::End, Pos::Center).anchor_x(Anchor::End);

    let spacer = CONTAINER.size_x(Size::Fill);

    let strips = H_STACK
        .size_x(Size::Fill).size_y(Size::Fill)
        .stack_arrange(Arrange::Center)
        .stack_spacing(8.0)
        .padding(16.0);

    let layout = V_STACK
        .size_symm(Size::Fill)
        .stack_spacing(0.0)
        .stack_arrange(Arrange::Start);

    ui.add(layout).nest(|| {
        ui.add(header).nest(|| {
            ui.add(title);
            ui.add(sub);
            ui.add(spacer);
            ui.add(rec_badge);
        });
        ui.add(strips).nest(|| {
            for (ch_i, ch) in state.channels.iter().enumerate() {
                channel_strip(ui, ch, ch_i);
            }
        });
    });
}

fn main() {
    let state = State::default();
    example_window_loop::run_example_loop(state, update_ui);
}
