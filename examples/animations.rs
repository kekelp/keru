//! The animation system still needs some improvements.

use keru::*;
use keru::node_library::*;

#[derive(Default)]
pub struct State {
    expanded: Vec<bool>,
    sub_expanded: Vec<Vec<bool>>,
    animation_speed: f32,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const EXPAND: NodeKey;
    #[node_key] const SUB_EXPAND: NodeKey;
    #[node_key] const ELEM: NodeKey;
    #[node_key] const ELEM_VSTACK: NodeKey;
    #[node_key] const SUB_ELEM_VSTACK: NodeKey;
    #[node_key] const HGROUP: NodeKey;
    
    let left_bar = V_STACK
        .size_x(Size::Pixels(500.0))
        .size_y(Size::Fill)
        .stack_arrange(Arrange::Start)
        .position_x(Pos::Start);
    
    let h_group = H_STACK
        .animate_position(true)
        .clip_children_y(true)
        .size_x(Size::Fill)
        .stack_arrange(Arrange::Start);

    let expand = BUTTON
        .text("Expand")
        .position_x(Pos::Start)
        .position_y(Pos::Start)
        .size_x(Size::FitContent);

    let sub_expand = BUTTON
        .text("Sub-Expand")
        .position_x(Pos::Start)
        .position_y(Pos::Start)
        .size_x(Size::FitContent);

    let elem = BUTTON
        .size_x(Size::Fill)
        .text("Element");

    let elem_vstack = V_STACK
        .grow_from_top().shrink_to_top()
        .animate_position(true)
        .clip_children_y(true)
        .key(ELEM_VSTACK);

    let sub_elem_vstack = V_STACK
        .grow_from_top().shrink_to_top()
        .animate_position(true)
        .clip_children_y(true)
        .key(SUB_ELEM_VSTACK);
    
    let n = 4;
    let m = 4;
    let p = 4;
    
    ui.set_global_animation_speed(state.animation_speed);

    ui.add(left_bar).nest(|| {
        for i in 0..n {
            ui.add(h_group).nest(|| {
                let key = EXPAND.sibling(i);
                ui.add(expand.key(key));
                
                if state.expanded[i] {
                    let key = ELEM_VSTACK.sibling(i);
                    ui.add(elem_vstack.key(key)).nest(|| {
                        for j in 0..m {

                            let key = HGROUP.sibling(i).sibling(j);
                            ui.add(h_group.key(key)).nest(|| {
                                let key = SUB_EXPAND.sibling(i).sibling(j);
                                ui.add(sub_expand.key(key));
                                
                                if state.sub_expanded[i][j] {
                                    let key = SUB_ELEM_VSTACK.sibling(i).sibling(j);
                                    ui.add(sub_elem_vstack.key(key)).nest(|| {

                                        for k in 0..p {
                                            let key = ELEM.sibling(i).sibling(j).sibling(k);
                                            ui.add(elem.key(key));
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

    ui.add(CONTAINER.position_y(Pos::End).size_x(Size::Frac(0.7))).nest(|| {
        ui.add(V_STACK).nest(|| {
            ui.add(TEXT.text("Global animation speed:"));
            ui.slider(&mut state.animation_speed, 0.02, 1.5);
        });
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
        animation_speed: 1.0,
    };
    example_window_loop::run_example_loop(state, update_ui);
}