use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {
    elements: Vec<u32>,
    next_id: u32,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const ADD: NodeKey;
    #[node_key] const DELETE: NodeKey;
    
    ui.add(V_STACK.size_symm(Size::Fill).padding(50)).nest(|| {
        for &id in &state.elements {
            ui.add(H_STACK.slide()).nest(|| {
                ui.add(LABEL.text(&format!("Element {}", id)));
                ui.add(BUTTON.static_text("Delete").key(DELETE.s(id)));
            });
        }
        ui.add(BUTTON.key(ADD).static_text("SEETHE"));
    });
    
    // effects
    state.elements.retain(|&id| !ui.is_clicked(DELETE.s(id)));
    
    if ui.is_clicked(ADD) {
        state.elements.push(state.next_id);
        state.next_id += 1;
    }
}

fn main() {
    basic_env_logger_init();
    let state = State {
        elements: vec![0, 1, 2],
        next_id: 3,
    };
    run_example_loop(state, update_ui);
}