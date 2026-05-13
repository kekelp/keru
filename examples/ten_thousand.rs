use keru::*;
use keru::node_library::*;

#[node_key] const KEY: NodeKey;

fn hue_color(hue: f32) -> Color {
    let h = (hue * 6.0).rem_euclid(6.0);
    let f = h.fract();
    let (r, g, b) = match h as u32 {
        0 => (1.0, f,   0.0),
        1 => (1.0 - f, 1.0, 0.0),
        2 => (0.0, 1.0, f),
        3 => (0.0, 1.0 - f, 1.0),
        4 => (f,   0.0, 1.0),
        _ => (1.0, 0.0, 1.0 - f),
    };
    let s = 0.55;
    Color::new(r * s + (1.0 - s), g * s + (1.0 - s), b * s + (1.0 - s), 1.0)
}

fn update_ui(_: &mut (), ui: &mut Ui) {
    with_arena(|arena| {

        ui.add(V_SCROLL_STACK).nest(|| {
            for i in 0..10_000 {

                let key = KEY.sibling(i);

                let height = if ui.is_hovered(key) { 90.0 } else { 50.0 };
                let text = bumpalo::format!(in arena, "{}", i);

                let color = hue_color((i % 10) as f32 / 10.0);

                let node = PANEL
                    .color(color)
                    .sense_hover_enter_or_exit(true)
                    .text(&text)
                    .size_y(Size::Pixels(height))
                    .size_x(Size::Pixels(100.0))
                    .animate_position(true)
                    .key(key);

                ui.add(node);
            }
        });
    });
}

fn main() {
    example_window_loop::run_example_loop((), update_ui);
}



