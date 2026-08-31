// This example shows how to create a virtual list manually.
// We could also separate this logic into a Component or a function that takes a |ui, i| { ... } closure.
use keru::*;
use keru::node_library::*;
use keru::example_window_loop::*;

#[node_key] const LIST: NodeKey;

const ITEM_COUNT: usize = 1_000_000;
const ROW_HEIGHT: f32 = 40.0;
const VIEWPORT_H: f32 = 500.0;

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
    let scroll = if let Some(offset) = ui.scroll_offset(LIST) { offset.y } else { 0.0 };

    // Calculate what elements are within the viewport.
    let first = (scroll / ROW_HEIGHT) as usize;
    let visible_count = (VIEWPORT_H / ROW_HEIGHT).ceil() as usize + 2;
    let last = (first + visible_count).min(ITEM_COUNT);

    let list = V_SCROLL_STACK
        .stack_spacing(0.0)
        .size(Size::Pixels(320.0), Size::Pixels(500.0))
        .key(LIST);

    // Add spacer elements instead of all the offscreen ones.
    let top_spacer = SPACER.size_y(Size::Pixels(first as f32 * ROW_HEIGHT));
    let bottom_spacer = SPACER.size_y(Size::Pixels((ITEM_COUNT - last) as f32 * ROW_HEIGHT));

    let element = BUTTON.sense_hover(true).size(Size::Fill, Size::Pixels(ROW_HEIGHT));

    ui.add(list).nest(|| {
        ui.add(top_spacer);
        for i in first..last {
            let color = hue_color((i % 12) as f32 / 12.0);
            // We should still use an arena, but this only runs a handful of times now.
            let text = format!("Item {i}");
            let row = element.color(color).text(&text);

            ui.add(row);
        }
        ui.add(bottom_spacer);
    });
}

fn main() {
    basic_env_logger_init();
    run_example_loop((), update_ui);
}
