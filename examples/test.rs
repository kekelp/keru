#![allow(dead_code)]
#![allow(unused_variables)]

use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {
    expanded: bool,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const EXPAND: NodeKey;
    
    let left_bar = V_STACK
        .size_x(Size::Pixels(300))
        .size_y(Size::Fill)
        .stack_arrange(Arrange::Start)
        .position_x(Position::Start);
    
    let expand = BUTTON
        .text("Expand")
        .position_x(Position::Start)
        .position_y(Position::Start)
        .size_x(Size::FitContent)
        .key(EXPAND);

    let element = LABEL.size_x(Size::Fill);

    ui.add(left_bar).nest(|| {
        
        let h_stack = H_STACK.size_x(Size::Fill).stack_arrange(Arrange::Start);
        ui.add(h_stack).nest(|| {
            ui.add(expand);
            if state.expanded {
                ui.add(V_STACK).nest(|| {
                    ui.add(element.text("1"));
                    ui.add(element.text("2"));
                    ui.add(element.text("3"));
                });    
            }
        });
    });

    if ui.is_clicked(EXPAND) {
        state.expanded = ! state.expanded;
    }
}


fn main() {
    let state = State {
        expanded: true,
    };
    run_example_loop(state, update_ui);
}
