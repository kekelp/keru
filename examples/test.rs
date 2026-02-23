use keru::example_window_loop::*;
use keru::*;

struct State {
    items: Vec<&'static str>,
}

fn update_ui(state: &mut State, ui: &mut Ui) {

    #[node_key] const ITEM: NodeKey;
    #[component_key] const STACK: ComponentKey<DragAndDropStack>;

    let item_base = BUTTON
        .animate_position(true)
        .sense_drag(true)
        .size_x(Size::Pixels(100.0))
        .absorbs_clicks(false)
        .anchor_symm(Anchor::Center);

    let (stack, floater) = ui.add_component(DragAndDropStack { key: STACK, pos: Pos::Center });

    stack.nest(|| {
        for item in &state.items {
            let key = ITEM.sibling(&item);
            let node = item_base.text(&item).key(key);

            if ui.is_dragged(key).is_some() {
                floater.nest(|| {
                    ui.add(node);
                });
            } else {
                ui.add(node);
            }
        }
    });

    let mut released_idx = None;
    for (i, item) in state.items.iter().enumerate() {
        if ui.is_drag_released(ITEM.sibling(&item)) {
            released_idx = Some(i);
        }
    }

    if let Some(drop) = ui.component_output(STACK) {
        if let Some(old_idx) = released_idx {
            let item = state.items.remove(old_idx);
            state.items.insert(drop.insertion_index.min(state.items.len()), item);
        }
    }
}

fn main() {
    let items = vec!["A", "B", "C", "D", "E"];

    let state = State {
        items,
    };

    run_example_loop(state, update_ui);
}
