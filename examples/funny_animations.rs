use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {
    expanded: bool,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const EXPAND: NodeKey;

    if ui.is_clicked(EXPAND) {
        state.expanded = ! state.expanded;
    }

    let left_bar = V_STACK
        .size_x(Size::Pixels(500))
        .size_y(Size::Fill)
        .stack_arrange(Arrange::Start)
        .position_x(Position::Start);
    
    let expand_base = BUTTON
        .slide()
        .position_x(Position::Start)
        .position_y(Position::Start)
        .size_x(Size::FitContent);

    let h_group = H_STACK.size_x(Size::Fill).stack_arrange(Arrange::Start);
    
    let slower = CONTAINER
        .padding(0)
        .slide()
        .size_x(Size::Fill).stack_arrange(Arrange::Start);

    let elem = LABEL.size_x(Size::Fill).slide().text("Suh");

    // every node interpolates its position towards its parent. but if many parents are stacked and they're all animated, this is what happens.

    ui.add(left_bar).nest(|| {
        ui.add(h_group).nest(|| {
            ui.add(expand_base.text("Expand").key(EXPAND));
            if state.expanded {
                ui.add(V_STACK).nest(|| {
                    ui.add(elem);
                    ui.add(slower).nest(|| ui.add(elem));
                    ui.add(slower).nest(|| ui.add(slower).nest(|| ui.add(elem)));
                    ui.add(slower).nest(|| ui.add(slower).nest(|| ui.add(slower).nest(|| ui.add(elem))));
                    ui.add(slower).nest(|| ui.add(slower).nest(|| ui.add(slower).nest(|| ui.add(slower).nest(|| ui.add(elem)))));
                    ui.add(slower).nest(|| ui.add(slower).nest(|| ui.add(slower).nest(|| ui.add(slower).nest(|| ui.add(slower).nest(|| ui.add(elem))))));
                });
            }
        });
    });
}


fn main() {
    basic_env_logger_init();
    let state = State::default();
    run_example_loop(state, update_ui);
}
