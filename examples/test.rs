#![allow(unused)]
use keru::*;
use keru::example_window_loop::*;

struct State {
    count: usize,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const ADD: NodeKey;
    #[node_key] const REMOVE: NodeKey;

    if ui.is_clicked(ADD) {
        state.count += 1;
    }
    if ui.is_clicked(REMOVE) && state.count > 0 {
        state.count -= 1;
    }

    let grid = PANEL
        .size(Size::Frac(0.8), Size::FitContent)
        .grid(4, 8.0, 8.0)
        .padding(8.0);

    let count_str = format!("{} items", state.count);

    ui.add(V_STACK).nest(|| {
        ui.add(H_STACK).nest(|| {
            ui.add(BUTTON.text("Add").key(ADD));
            ui.add(BUTTON.text("Remove").key(REMOVE));
            ui.add(LABEL.text(&count_str));
        });
        ui.add(grid).nest(|| {
            for i in 0..state.count {
                let hue = (i as f32 * 0.13).rem_euclid(1.0);
                let color = Color::new(
                    (hue * 6.0).rem_euclid(1.0),
                    1.0 - (hue * 3.0).rem_euclid(0.5),
                    0.4 + (hue * 5.0).rem_euclid(0.4),
                    1.0,
                );

                ui.add(PANEL.color(color).size_symm(Size::Fill));

                if i == 5 {
                    ui.add(SPACER);
                }
            }
        });
    });
}

fn main() {
    let state = State { count: 9 };
    example_window_loop::run_example_loop(state, update_ui);
}
