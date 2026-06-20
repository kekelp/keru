/// This example shows how to implement richer keyboard navigation logic that is still compatible with the default tab/shift-tab stuff.
/// Of course it's also possible to disable the default logic and build a completely custom system.
use keru::*;
use keru::node_library::*;
use winit::keyboard::{Key, NamedKey};

const RADIUS: i32 = 2;
const STEP: f32 = 90.0;
const MARGIN: f32 = 40.0;

fn inside_diamond_shape(x: i32, y: i32) -> bool {
    x.abs() + y.abs() <= RADIUS
}

#[node_key] const CELL: NodeKey;

const ARROWS: [(NamedKey, i32, i32); 4] = [
    (NamedKey::ArrowRight, 1, 0),
    (NamedKey::ArrowLeft, -1, 0),
    (NamedKey::ArrowDown, 0, 1),
    (NamedKey::ArrowUp, 0, -1),
];

fn update_ui(_state: &mut (), ui: &mut Ui) {

    let mut focused_cell: Option<(i32, i32)> = None;

    ui.add(PANEL.size_symm(Size::Fill)).nest(|| {
        ui.add(TEXT.text("Use either the default Tab/Shift+Tab to navigate nodes in the default order, or the arrow keys to navigate in 2D.")
            .position(Pos::Pixels(MARGIN), Pos::End));

        for y in -RADIUS..=RADIUS {
            for x in -RADIUS..=RADIUS {
                if ! inside_diamond_shape(x, y) {
                    continue;
                }

                let key = CELL.sibling((x, y));
                let label = &format!("{x} {y}");
                let node = BUTTON
                    .key(key)
                    .text(label)
                    .size_symm(Size::Pixels(70.0))
                    .position(
                        Pos::Pixels(MARGIN + (x + RADIUS) as f32 * STEP),
                        Pos::Pixels(MARGIN + (y + RADIUS) as f32 * STEP),
                    );

                ui.add(node);

                if ui.is_focused(key) {
                    focused_cell = Some((x, y))
                }
            }
        }
    });

    for (key, dx, dy) in ARROWS {
        if ui.key_pressed_or_repeated(&Key::Named(key)) {
            match focused_cell {
                Some((x, y)) => {
                    let (mut nx, mut ny) = (x + dx, y + dy);
                    if ! inside_diamond_shape(nx, ny) {
                        let limit = RADIUS - (x * dy).abs() - (y * dx).abs();
                        nx = x * dy.abs() - dx * limit;
                        ny = y * dx.abs() - dy * limit;
                    }
                    ui.focus(CELL.sibling((nx, ny)));
                }
                None => {
                    ui.focus(CELL.sibling((0, 0)));
                }
            }
        }
    }
}

fn main() {
    example_window_loop::run_example_loop((), update_ui);
}
