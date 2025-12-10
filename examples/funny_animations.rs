use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {
    expanded: Vec<bool>,
    clip_children: bool,
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
        .slide()
        .clip_children_y(state.clip_children)
        .size_x(Size::Fill)
        .stack_arrange(Arrange::Start);
    
    let expand = BUTTON
        .text("Expand")
        .position_x(Position::Start)
        .position_y(Position::Start)
        .size_x(Size::FitContent);

    let elem = BUTTON
        .size_x(Size::Fill)
        .text("???");

    let elem_vstack = V_STACK
        .slide()
        .key(ELEM_VSTACK);

    let n = 4;
    let m = 4;

    ui.add(left_bar).nest(|| {
        for i in 0..n {
            ui.add(h_group).nest(|| {
                ui.add(expand.key(EXPAND.sibling(i)));

                if state.expanded[i] {
                    ui.add(elem_vstack.key(ELEM_VSTACK.sibling(i))).nest(|| {

                        for j in 0..m {
                            ui.add(elem.key(ELEM.sibling(i).sibling(j)));
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

    if ui.add(BUTTON.position_symm(Position::End).static_text("Toggle clipping")).is_clicked(ui) {
        state.clip_children = !state.clip_children;
    }
}


fn main() {
    basic_env_logger_init();
    let state = State {
        expanded: vec![false, false, false, false, false],
        clip_children: true,
    };
    run_example_loop(state, update_ui);
}
