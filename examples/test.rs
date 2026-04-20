#![allow(unused)]
use keru::*;
use keru::example_window_loop::*;

struct State {
    count: usize,
    flow: GridFlow,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const ADD: NodeKey;
    #[node_key] const REMOVE: NodeKey;
    #[node_key] const TOGGLE_AXIS: NodeKey;
    #[node_key] const TOGGLE_X: NodeKey;
    #[node_key] const TOGGLE_Y: NodeKey;

    if ui.is_clicked(ADD) {
        state.count += 1;
    }
    if ui.is_clicked(REMOVE) && state.count > 0 {
        state.count -= 1;
    }
    if ui.is_clicked(TOGGLE_AXIS) {
        state.flow.main_axis = state.flow.main_axis.other();
    }
    if ui.is_clicked(TOGGLE_X) {
        state.flow.x_reversed = !state.flow.x_reversed;
    }
    if ui.is_clicked(TOGGLE_Y) {
        state.flow.y_reversed = !state.flow.y_reversed;
    }

    let axis_label = match state.flow.main_axis { Axis::X => "Axis: Row", Axis::Y => "Axis: Col" };
    let x_label = if state.flow.x_reversed { "X: RTL" } else { "X: LTR" };
    let y_label = if state.flow.y_reversed { "Y: BTT" } else { "Y: TTB" };
    let count_str = format!("{} items", state.count);

    let grid = PANEL
        .size(Size::Frac(0.8), Size::FitContent)
        .grid(4, 8.0, 8.0, state.flow)
        .padding(8.0);

    ui.add(V_STACK.position_y(Pos::Start)).nest(|| {
        ui.add(H_STACK).nest(|| {
            ui.add(BUTTON.text("Add").key(ADD));
            ui.add(BUTTON.text("Remove").key(REMOVE));
            ui.add(LABEL.text(&count_str));
            ui.add(BUTTON.text(axis_label).key(TOGGLE_AXIS));
            ui.add(BUTTON.text(x_label).key(TOGGLE_X));
            ui.add(BUTTON.text(y_label).key(TOGGLE_Y));
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
                let label = format!("{i}");
                let node = match i {
                    0 => PANEL.color(color).size_symm(Size::Pixels(80.0)).grid_column_span(2),
                    1 => PANEL.color(color).size_symm(Size::Pixels(80.0)).grid_row_span(2),
                    _ => PANEL.color(color).size_symm(Size::Pixels(80.0)),
                };
                ui.add(node).nest(|| {
                    ui.add(TEXT.text(&label));
                });
            }
        });
    });
}

fn main() {
    let state = State { count: 9, flow: GridFlow::DEFAULT };
    example_window_loop::run_example_loop(state, update_ui);
}
