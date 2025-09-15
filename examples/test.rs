use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {
    count: i32,
}

fn update_ui(state: &mut State, ui: &mut Ui) {

    #[node_key] const COLOR_BUTTON: NodeKey;
    let colors = ["Blue", "Azure", "Cerulean"];
    
    ui.add(V_STACK).nest(|| {
        for color in &colors {
            let dynamic_key = COLOR_BUTTON.sibling(color);
            let button = BUTTON.key(dynamic_key).text(color);
            ui.add(button);
        }
    });

}


fn main() {
    let state = State::default();
    run_example_loop(state, update_ui);
}
