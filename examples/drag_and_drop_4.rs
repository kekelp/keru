use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {
    pub left_strings: Vec<String>,
    pub right_strings: Vec<String>,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const INCREASE: NodeKey;
    
    let increase_button = BUTTON
        .color(Color::RED)
        .text("Increase")
        .key(INCREASE);

    let (dragged_container, v_stack) = ui.add_component(DragNDropList);
    
    v_stack.nest(|| {
        ui.add(increase_button);
        ui.add(LABEL.text("1"));
        ui.add(LABEL.text("2"));
        ui.add(LABEL.text("3"));
    });

    dragged_container.nest(|| {

    });
}

fn main() {
    let mut state = State::default();
    state.left_strings = vec!["1".into(), "2".into(), "3".into(), "4".into()];
    state.right_strings = vec!["a".into(), "b".into(), "c".into(), "d".into()];
    run_example_loop(state, update_ui);
}


struct DragNDropList;

impl Component for DragNDropList {
    type AddResult = (UiParent, UiParent);
    type ComponentOutput = ();
    type State = ();

    fn add_to_ui(&mut self, ui: &mut Ui, state: &mut ()) -> Self::AddResult {
        
        let dragged_container = ui.jump_to_root().nest(|| {
            let cont = ui.add(PANEL);
            cont
        });

        let vstack = ui.add(V_STACK);

        return (dragged_container, vstack);
    }
}