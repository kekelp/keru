use keru::*;
use keru::node_library::*;
use keru::example_window_loop::*;

const COLUMNS: usize = 90;
const ROWS: usize = 33;
const N_CELLS: usize = COLUMNS * ROWS;

#[node_key] const RANDOMIZE: NodeKey;
#[node_key] const CLEAR: NodeKey;
#[node_key] const CELL_BUTTON: NodeKey;
#[component_key] const CELL: ComponentKey<Cell>;

struct Cell {
    index: usize,
}

impl ReactiveComponent for Cell {
    type State = Observer<i32>;
    type AddResult = ();
    type ComponentOutput = ();

    fn component_key(&self) -> Option<ComponentKey<Self>> {
        return Some(CELL.sibling(self.index));
    }

    fn state_changed(&self, ui: &Ui, value: &Observer<i32>) -> bool {
        return ui.check_if_observer_is_changed(value);
    }

    fn run_effects(&mut self, ui: &mut Ui, value: &mut Observer<i32>) {
        if ui.is_clicked(CELL_BUTTON) {
            *value += 1;
        }
        if ui.is_right_clicked(CELL_BUTTON) {
            *value -= 1;
        }
    }

    fn add_to_ui(&mut self, ui: &mut Ui, value: &Observer<i32>) {
        with_arena(|arena| {
            let text = bumpalo::format!(in arena, "{}", **value);
            let cell = BUTTON
                .size_x(Size::Pixels(50.0))
                .size_y(Size::Pixels(40.0))
                .text_size(16.0)
                .text(&text)
                .key(CELL_BUTTON);
            ui.add(cell);
        });
    }
}

struct State {
    cells: Vec<Observer<i32>>,
    random_seed: u32,
}

impl State {
    fn next_random(&mut self) -> usize {
        self.random_seed = self.random_seed.wrapping_mul(1664525).wrapping_add(1013904223);
        return (self.random_seed >> 8) as usize;
    }

    fn update_ui(&mut self, ui: &mut Ui) {
        if ui.is_clicked(RANDOMIZE) {
            for _ in 0..50 {
                let i = self.next_random() % N_CELLS;
                let value = (self.next_random() % 1000) as i32;
                *self.cells[i] = value;
            }
        }

        if ui.is_clicked(CLEAR) {
            for cell in &mut self.cells {
                **cell = 0;
            }
        }

        let grid = GRID
            .grid(Axis::X, MainAxisCellSize::Count(COLUMNS as u32), 2.0, 2.0, false)
            .position_x(Pos::Start);

        ui.add(V_STACK.size_symm(Size::Fill)).nest(|| {
            ui.add(H_STACK).nest(|| {
                ui.add(BUTTON.static_text("Randomize some cells").key(RANDOMIZE));
                ui.add(BUTTON.static_text("Clear all").key(CLEAR));
            });

            ui.add(V_SCROLL_STACK.scrollable_x(true).size_x(Size::Fill)).nest(|| {
                ui.add(grid).nest(|| {
                    for (index, value) in self.cells.iter_mut().enumerate() {
                        ui.add_component_with_state(Cell { index }, value);
                    }
                });
            });
        });
    }
}

fn main() {
    basic_env_logger_init();

    let state = State {
        cells: (0..N_CELLS).map(|_| Observer::new(0)).collect(),
        random_seed: 1,
    };
    run_example_loop(state, State::update_ui);
}
