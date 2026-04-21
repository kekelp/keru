#![allow(unused)]
use keru::*;
use keru::example_window_loop::*;

struct State {
    elements: Vec<Element>,

    next_element: Element,
    
    use_n_columns: bool,

    // todo: use ints after we figure out int sliders
    n_columns: f32,
    column_width: f32,

    flow: GridFlow,
}

#[derive(Clone, Copy)]
struct Element {
    // todo: use ints after we figure out int sliders
    row_span: f32,
    column_span: f32,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const ADD: NodeKey;
    #[node_key] const ADD_FIVE: NodeKey;
    #[node_key] const REMOVE: NodeKey;
    #[node_key] const REMOVE_FIVE: NodeKey;
    #[node_key] const TOGGLE_AXIS: NodeKey;
    #[node_key] const TOGGLE_X: NodeKey;
    #[node_key] const TOGGLE_Y: NodeKey;
    #[node_key] const TOGGLE_COLUMNS: NodeKey;
    #[node_key] const TOGGLE_BACKFILL: NodeKey;
    #[node_key] const ELEMENT: NodeKey;
    
    with_arena(|arena| {
    
        if ui.is_clicked(ADD) {
            state.elements.push(state.next_element);
        }
        if ui.is_clicked(ADD_FIVE) {
            for i in 0..5 {
                state.elements.push(state.next_element);
            }
        }
        if ui.is_clicked(REMOVE) && state.elements.len() > 0 {
            state.elements.pop();
        }
        if ui.is_clicked(REMOVE_FIVE) {
            for i in 0..5 {
                if state.elements.len() > 0 {
                    state.elements.pop();
                }
            }
        }
        if ui.is_clicked(TOGGLE_AXIS) {
            state.flow.main_axis = state.flow.main_axis.other();
        }
        if ui.is_clicked(TOGGLE_X) {
            state.flow.x_fill_direction = if state.flow.x_fill_direction == Direction::LeftToRight { Direction::RightToLeft } else { Direction::LeftToRight };
        }
        if ui.is_clicked(TOGGLE_Y) {
            state.flow.y_fill_direction = if state.flow.y_fill_direction == Direction::LeftToRight { Direction::RightToLeft } else { Direction::LeftToRight };
        }
        if ui.is_clicked(TOGGLE_BACKFILL) {
            state.flow.backfill = ! state.flow.backfill;
        }
        if ui.is_clicked(TOGGLE_COLUMNS) {
            state.use_n_columns = ! state.use_n_columns;
        }

        for (i, element) in state.elements.iter_mut().enumerate() {
            let key = ELEMENT.sibling(i);
            if ui.is_clicked(key) {
                if ui.key_input().key_mods().shift_key() {
                    element.row_span += 1.0;
                } else {
                    element.column_span += 1.0;
                }
            }
            if ui.is_right_clicked(key) {
                if ui.key_input().key_mods().shift_key() {
                    element.row_span -= 1.0;
                } else {
                    element.column_span -= 1.0;
                }
            }
            if element.row_span < 1.0 { element.row_span = 1.0 };
            if element.column_span < 1.0 { element.column_span = 1.0 };
        }

        let backfill_label = if state.flow.backfill { "Backfill: On" } else { "Backfill: Off" };
        let axis_label = match state.flow.main_axis { Axis::X => "Fill Rows First", Axis::Y => "Fill Columns First" };
        let x_label = if state.flow.x_fill_direction == Direction::RightToLeft { "Right to Left" } else { "Left to Right" };
        let y_label = if state.flow.y_fill_direction == Direction::RightToLeft { "Bottom to Top" } else { "Top to Bottom" };
        let columns_label = if state.use_n_columns { "Column size: specify Count" } else { "Column size: specify Width" };

        let columns = if state.use_n_columns { MainAxisCellSize::Count(state.n_columns as u32) } else { MainAxisCellSize::Width(state.column_width) };

        let grid = PANEL
            .size_symm(Size::Fill)
            .grid(columns, 8.0, 8.0, state.flow)
            .padding(8.0);

        ui.add(H_STACK.position_y(Pos::Start)).nest(|| {
            ui.add(V_SCROLL_STACK.position_y(Pos::Start).size_x(Size::Pixels(250.0))).nest(|| {

                ui.add(PANEL).nest(|| {
                    ui.add(V_STACK).nest(|| {
                        ui.add(TEXT.text("Grid properties:"));

                        ui.add(BUTTON.text(axis_label).key(TOGGLE_AXIS));
                        ui.add(BUTTON.text(x_label).key(TOGGLE_X));
                        ui.add(BUTTON.text(y_label).key(TOGGLE_Y));
                        ui.add(BUTTON.text(backfill_label).key(TOGGLE_BACKFILL));
                        
                        ui.add(BUTTON.text(&columns_label).key(TOGGLE_COLUMNS));
                        if state.use_n_columns {
                            ui.add(TEXT.text("Columns: (rounded)"));
                            ui.add_component(Slider::new(&mut state.n_columns, 0.0, 50.0, true))
                        } else {
                            ui.add(TEXT.text("Width: (rounded)"));
                            ui.add_component(Slider::new(&mut state.column_width, 0.0, 300.0, true))
                        }
                    });
                });

                ui.add(PANEL).nest(|| {
                    ui.add(V_STACK).nest(|| {

                        ui.add(H_STACK).nest(|| {
                            ui.add(BUTTON.size_x(Size::Fill).text("Push Element").key(ADD));
                            ui.add(BUTTON.size_x(Size::Fill).text("Push 5").key(ADD_FIVE));
                        });
                        ui.add(H_STACK).nest(|| {
                            ui.add(BUTTON.size_x(Size::Fill).text("Pop Element").key(REMOVE));
                            ui.add(BUTTON.size_x(Size::Fill).text("Pop 5").key(REMOVE_FIVE));
                        });
                        
                        ui.add(TEXT.text("\
                            Click on elements to change their row span. \n\n\
                            Left click / right click: increase / decrease span \n\n\
                            Hold Shift to change column span
                        "));
                    });
                });


            });

            ui.add(grid).nest(|| {
                for (i, element) in state.elements.iter().enumerate() {
                    let key = ELEMENT.sibling(i);
                    let hue = (i as f32 * 0.13).rem_euclid(1.0);
                    let color = Color::new(
                        (hue * 6.0).rem_euclid(1.0),
                        1.0 - (hue * 3.0).rem_euclid(0.5),
                        0.4 + (hue * 5.0).rem_euclid(0.4),
                        1.0,
                    );
                    let text = bumpalo::format!(in arena, "{}", i);

                    let node = PANEL
                        .color(color)
                        .stroke_color(Color::BLUE)
                        .sense_click(true)
                        .size_symm(Size::Fill)
                        .animate_position(true)
                        .grid_row_span(element.row_span as u16)
                        .grid_column_span(element.column_span as u16)
                        .key(key);

                    ui.add(node).nest(|| {
                        ui.add(TEXT.text(&text));
                    });
                }
            });
        });

    });
}

fn main() {
    let next_element = Element { row_span: 1.0, column_span: 1.0 };
    let elements = vec![next_element; 12];
    let state = State { elements, next_element, flow: GridFlow::DEFAULT, use_n_columns: true, n_columns: 4.0, column_width: 150.0  };
    example_window_loop::run_example_loop(state, update_ui);
}
