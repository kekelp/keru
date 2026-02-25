use keru::*;

struct State {
    items: Vec<&'static str>,
}

fn update_ui(state: &mut State, ui: &mut Ui) {

    #[node_key] const ITEM: NodeKey;
    #[component_key] const STACK: ComponentKey<DragAndDropStack>;

    let item_base = BUTTON
        .animate_position(true)
        .absorbs_clicks(false)
        .sense_drag(true)
        .size_x(Size::Pixels(100.0))
        .anchor_symm(Anchor::Center);

    let component = DragAndDropStack { key: STACK };
    
    ui.add_component(component).nest(|| {
        for &item in &state.items {
            let key = ITEM.sibling(&item);
            let node = item_base.text(&item).key(key);
            ui.add(node);
        }
    });

    if let Some((move_from, move_to)) = ui.run_component(STACK) {
        let item = state.items.remove(move_from);
        let adjusted = if move_to > move_from { move_to - 1 } else { move_to };
        state.items.insert(adjusted.min(state.items.len()), item);
    }
}

fn main() {
    let items = vec!["A", "special", "B", "C", "xxxxxx\nxxxxxx\nxxxxxx", "D", "E"];

    let state = State {
        items,
    };

    example_window_loop::run_example_loop(state, update_ui);
}
