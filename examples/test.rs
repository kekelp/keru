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

    let h_group = H_STACK.size_x(Size::Fill).stack_arrange(Arrange::Start);

    let element = LABEL.size_x(Size::Fill).slide();

    ui.add(left_bar).nest(|| {
        ui.add(h_group).nest(|| {
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
}


fn main() {
    basic_env_logger_init();
    let state = State {
        expanded: true,
    };
    run_example_loop(state, update_ui);
}
