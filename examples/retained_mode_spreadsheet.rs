/// A big grid of cells that uses """retained mode""" to update the cells.

use keru::*;
use keru::node_library::*;
use keru::example_window_loop::*;
use winit::event::MouseButton;

const COLUMNS: usize = 90;
const ROWS: usize = 33;
const N_CELLS: usize = COLUMNS * ROWS;

const CELL_ID_BASE: u64 = 182428736534;

struct Cells {
    values: Vec<i32>,
    changed: Vec<usize>,
}

impl Cells {
    fn new(len: usize) -> Self {
        return Self { values: vec![0; len], changed: Vec::new() };
    }

    fn get(&self, i: usize) -> i32 {
        return self.values[i];
    }

    fn set(&mut self, i: usize, value: i32) {
        if self.values[i] != value {
            self.values[i] = value;
            self.changed.push(i);
        }
    }
}

struct State {
    cells: Cells,
    last_update_size: usize,
    random_seed: u32,
}

#[node_key] const GRID_ROOT: NodeKey;
#[node_key] const RANDOMIZE: NodeKey;
#[node_key] const CLEAR: NodeKey;

impl State {
    fn next_random(&mut self) -> usize {
        self.random_seed = self.random_seed.wrapping_mul(1664525).wrapping_add(1013904223);
        return (self.random_seed >> 8) as usize;
    }

    fn update_ui(&mut self, ui: &mut Ui) {

        // Change the values before the grid update, because the `changed` list is cleared at the end of this function.
        for i in ui.get_clicked_ids(MouseButton::Left) {
            let i = (i - CELL_ID_BASE) as usize;
            self.cells.set(i, self.cells.get(i) + 1);
        }
        for i in ui.get_clicked_ids(MouseButton::Right) {
            let i = (i - CELL_ID_BASE) as usize;
            self.cells.set(i, self.cells.get(i) - 1);
        }

        if ui.is_clicked(RANDOMIZE) {
            for _ in 0..50 {
                let i = self.next_random() % N_CELLS;
                let value = (self.next_random() % 1000) as i32;
                self.cells.set(i, value);
            }
        }

        if ui.is_clicked(CLEAR) {
            for i in 0..N_CELLS {
                self.cells.set(i, 0);
            }
        }

        let grid_is_built = ui.get_node(GRID_ROOT).is_some();

        ui.add(V_STACK.size_symm(Size::Fill)).nest(|| {
            ui.add(H_STACK).nest(|| {
                ui.add(BUTTON.static_text("Randomize").key(RANDOMIZE));
                ui.add(BUTTON.static_text("Clear all").key(CLEAR));
            });

            ui.add(V_SCROLL_STACK.scrollable_x(true).size_x(Size::Fill)).nest(|| {
                if grid_is_built {
                    ui.readd_branch(GRID_ROOT);
                    self.do_minimal_grid_updates(ui);
                } else {
                    self.build_grid(ui);
                }
            });
        });

        if !self.cells.changed.is_empty() {
            self.last_update_size = self.cells.changed.len();
        }



        self.cells.changed.clear();
    }

    fn do_minimal_grid_updates(&self, ui: &mut Ui) {
        with_arena(|arena| {
            for &i in &self.cells.changed {
                let key = NodeKey::from_globally_unique_id(CELL_ID_BASE + i as u64);
                let text = bumpalo::format!(in arena, "{}", &self.cells.get(i).to_string());
                ui.get_node_mut(key).unwrap().set_text(&text);
            }
        });
    }

    fn build_grid(&self, ui: &mut Ui) {
        let cell = BUTTON
            .size_x(Size::Pixels(50.0))
            .size_y(Size::Pixels(40.0))
            .text_size(16.0);

        let grid = GRID
            .grid(Axis::X, MainAxisCellSize::Count(COLUMNS as u32), 2.0, 2.0, false)
            .position_x(Pos::Start)
            .key(GRID_ROOT);

        ui.add(grid).nest(|| {
            with_arena(|arena| {
                for i in 0..N_CELLS {
                    let text = bumpalo::format!(in arena, "{}", &self.cells.get(i).to_string());
                    let key = NodeKey::from_globally_unique_id(CELL_ID_BASE + i as u64);
                    let cell = cell.text(&text).key(key);
                    ui.add(cell);
                }
            });
        });
    }
}

fn main() {
    basic_env_logger_init();

    let state = State {
        cells: Cells::new(N_CELLS),
        last_update_size: 0,
        random_seed: 1,
    };
    run_example_loop(state, State::update_ui);
}
