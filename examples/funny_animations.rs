use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {
    expanded: Vec<bool>,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const EXPAND: NodeKey;
    #[node_key] const ELEM: NodeKey;
    #[node_key] const ELEM_VSTACK: NodeKey;

    let left_bar = V_STACK
        .size_x(Size::Pixels(500))
        .size_y(Size::Fill)
        .stack_arrange(Arrange::Start)
        .position_x(Position::Start);
    
    let h_group = H_STACK
        .size_x(Size::Fill)
        .stack_arrange(Arrange::Start);
    
    let expand = BUTTON
        .text("Expand")
        .slide()
        .position_x(Position::Start)
        .position_y(Position::Start)
        .size_x(Size::FitContent);

    let elem = LABEL
        .size_x(Size::Fill)
        // .slide()
        .text("Suh");

    let elem_vstack = V_STACK.slide().visible().outline_only(false).key(ELEM_VSTACK);

    let n = 4;
    let m = 4;

    ui.add(left_bar).nest(|| {
        for i in 0..n {
            ui.add(h_group).nest(|| {
                let key = EXPAND.sibling(i);
                ui.add(expand.key(key));

                if state.expanded[i] {
                    ui.add(elem_vstack).nest(|| {

                        for j in 0..m {
                            let key = ELEM.sibling(i).sibling(j);
                            ui.add(elem.key(key));
                        }

                    });
                }
            });
        }
    });

    for i in 0..n {
        if ui.is_clicked(EXPAND.sibling(i)) {
            state.expanded[i] = ! state.expanded[i];
        }
    }

    // ui.debug_print_tree();
}


fn main() {
    // basic_env_logger_init();
    let state = State {
        expanded: vec![false, false, false, false, false],
    };
    run_example_loop(state, update_ui);
}
