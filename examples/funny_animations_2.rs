use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {
    expanded: Vec<bool>,
    sub_expanded: Vec<Vec<bool>>,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const EXPAND: NodeKey;
    #[node_key] const SUB_EXPAND: NodeKey;
    #[node_key] const ELEM: NodeKey;
    #[node_key] const ELEM_VSTACK: NodeKey;
    #[node_key] const SUB_ELEM_VSTACK: NodeKey;
    #[node_key] const HGROUP: NodeKey;
    
    let left_bar = V_STACK
        .size_x(Size::Pixels(500))
        .size_y(Size::Fill)
        .stack_arrange(Arrange::Start)
        .position_x(Position::Start);
    
    let h_group = H_STACK
        .slide()
        // .clip_children_y(true)
        .size_x(Size::Fill)
        .stack_arrange(Arrange::Start);
    
    let expand = BUTTON
        .text("Expand")
        .position_x(Position::Start)
        .position_y(Position::Start)
        .size_x(Size::FitContent);
    
    let sub_expand = BUTTON
        .text("Sub-Expand")
        .position_x(Position::Start)
        .position_y(Position::Start)
        .size_x(Size::FitContent);
    
    let elem = BUTTON
        .size_x(Size::Fill)
        .text("???");
    
    let elem_vstack = V_STACK
        .slide()
        .key(ELEM_VSTACK);
    
    let sub_elem_vstack = V_STACK
        .slide()
        .key(SUB_ELEM_VSTACK);
    
    let n = 4;
    let m = 4;
    let p = 4;
    
    ui.add(left_bar).nest(|| {
        for i in 0..n {
            ui.add(h_group).nest(|| {
                let key = EXPAND.sibling(i);
                ui.add(expand.key(key));
                
                if state.expanded[i] {
                    ui.add(elem_vstack.key(ELEM_VSTACK.sibling(i))).nest(|| {
                        for j in 0..m {
                            ui.add(h_group.key(HGROUP.sibling(i).sibling(j))).nest(|| {
                                ui.add(sub_expand.key(SUB_EXPAND.sibling(i).sibling(j)));
                                
                                if state.sub_expanded[i][j] {
                                    ui.add(sub_elem_vstack.key(SUB_ELEM_VSTACK.sibling(i).sibling(j))).nest(|| {
                                        for k in 0..p {
                                            ui.add(elem.key(ELEM.sibling(i).sibling(j).sibling(k)));
                                        }
                                    });
                                }
                            });
                        }
                    });
                }
            });
        }
    });
    
    for i in 0..n {
        if ui.is_clicked(EXPAND.sibling(i)) {
            state.expanded[i] = !state.expanded[i];
        }
        
        for j in 0..m {
            if ui.is_clicked(SUB_EXPAND.sibling(i).sibling(j)) {
                state.sub_expanded[i][j] = !state.sub_expanded[i][j];
            }
        }
    }
    
    // ui.debug_print_tree();
}

fn main() {
    // basic_env_logger_init();
    let state = State {
        expanded: vec![false, false, false, false],
        sub_expanded: vec![
            vec![false, false, false, false],
            vec![false, false, false, false],
            vec![false, false, false, false],
            vec![false, false, false, false],
        ],
    };
    run_example_loop(state, update_ui);
}