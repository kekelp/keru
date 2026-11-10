use keru::*;
use keru::node_library::*;
use keru::example_window_loop::*;

const MIN_BLOCK_H: f32 = 40.0;
const MAX_BLOCK_H: f32 = 240.0;
const VIEWPORT_H: f32 = 500.0;

const OVERDRAW: f32 = 100.0;
const WINDOW_H: f32 = 7000.0;
const JUMP_EDGE: f32 = 500.0;

struct Block {
    top: f32,
    height: f32,
}

impl Block {
    fn bottom(&self) -> f32 {
        self.top + self.height
    }
}

#[derive(Default)]
struct State {
    blocks: Vec<Block>,
    visible_window_start: f32,
}

#[node_key] const LIST: NodeKey;

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

fn pseudo_random(i: usize) -> f32 {
    let hash = (i as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    return (hash >> 40) as f32 / (1u64 << 24) as f32;
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    let real_scroll = if let Some(offset) = ui.scroll_offset(LIST) { offset.y } else { 0.0 };
    // The position in the full list of blocks.
    let scroll = real_scroll + state.visible_window_start;

    let near_top = real_scroll < JUMP_EDGE && state.visible_window_start > 0.0;
    let near_bottom = real_scroll > WINDOW_H - VIEWPORT_H - JUMP_EDGE;

    if near_top || near_bottom {
        if let Some(list) = ui.get_node_mut(LIST) {
            // Put the viewport in the middle of the window. The spacers change by the same amount in this frame, so the blocks stay in place on the screen.
            let new_window_start = (scroll - (WINDOW_H - VIEWPORT_H) / 2.0).max(0.0).round();
            list.shift_scroll_offset(Xy::new(0.0, state.visible_window_start - new_window_start));
            state.visible_window_start = new_window_start;
        }
    }

    let visible_start = scroll - OVERDRAW;
    let visible_end = scroll + VIEWPORT_H + OVERDRAW;

    let mut bottom = state.blocks.last().map_or(0.0, Block::bottom);
    while bottom < visible_end {
        let random = pseudo_random(state.blocks.len());
        let height = (MIN_BLOCK_H + random * (MAX_BLOCK_H - MIN_BLOCK_H)).round();
        state.blocks.push(Block { top: bottom, height });
        bottom += height;
    }

    let first = state.blocks.partition_point(|block| block.bottom() < visible_start);
    let last = state.blocks.partition_point(|block| block.top < visible_end);

    let list = V_SCROLL_STACK
        .sense_scroll(true)
        .stack_spacing(0.0)
        .size(Size::Pixels(320.0), Size::Pixels(VIEWPORT_H))
        .key(LIST)
        .children_can_hide(false);

    let visible_top = state.blocks[first].top;
    let visible_bottom = state.blocks[last - 1].bottom();
    let window_end = state.visible_window_start + WINDOW_H;
    let top_spacer = SPACER.size_y(Size::Pixels(visible_top - state.visible_window_start));
    let bottom_spacer = SPACER.size_y(Size::Pixels(window_end - visible_bottom));

    ui.add(list).nest(|| {
        ui.add(top_spacer);
        for i in first..last {
            let color = hue_color((i % 12) as f32 / 12.0);
            let text = format!("Block {}", i);
            let element = PANEL
                .size(Size::Fill, Size::Pixels(state.blocks[i].height))
                .color(color)
                .text(&text);

            ui.add(element);
        }
        ui.add(bottom_spacer);
    });
}

fn main() {
    basic_env_logger_init();
    run_example_loop(State::default(), update_ui);
}
