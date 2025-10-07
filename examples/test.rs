use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {
    expanded: bool,
    expanded2: bool,
    expanded3: bool,
    expanded4: bool,
    expanded5: bool,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const EXPAND: NodeKey;
    #[node_key] const EXPAND2: NodeKey;
    #[node_key] const EXPAND3: NodeKey;
    #[node_key] const EXPAND4: NodeKey;
    #[node_key] const EXPAND5: NodeKey;

    if ui.is_clicked(EXPAND) {
        state.expanded = ! state.expanded;
    }
    if ui.is_clicked(EXPAND2) {
        state.expanded2 = ! state.expanded2;
    }
    if ui.is_clicked(EXPAND3) {
        state.expanded3 = ! state.expanded3;
    }
    if ui.is_clicked(EXPAND4) {
        state.expanded4 = ! state.expanded4;
    }
    if ui.is_clicked(EXPAND5) {
        state.expanded5 = ! state.expanded5;
    }


    
    let expand_base = BUTTON
        .slide() // All animations in any direction
        .position_x(Position::Start)
        .position_y(Position::Start)
        .size_x(Size::FitContent);

    let h_group = H_STACK.size_x(Size::Fill).stack_arrange(Arrange::Start);

    #[node_key] const ELEMENT: NodeKey;

    let element_all = LABEL.size_x(Size::Fill).slide().key(ELEMENT);
    

    ui.add(h_group).nest(|| {
        ui.add(expand_base.text("All Animations").key(EXPAND));
        if state.expanded {
            ui.add(element_all.text("Slide in/out/move - any direction"));
        }
    });

    ui.debug_print_tree();
}


fn main() {
    // basic_env_logger_init();
    let state = State::default();
    run_example_loop(state, update_ui);
}
