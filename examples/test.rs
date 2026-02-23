use keru::example_window_loop::*;
use keru::*;

struct State {
    items: Vec<&'static str>,
    items2: Vec<&'static str>,
}

fn update_ui(state: &mut State, ui: &mut Ui) {

    #[node_key] const ITEM: NodeKey;
    #[component_key] const STACK_1: ComponentKey<DragAndDropStack>;
    #[component_key] const STACK_2: ComponentKey<DragAndDropStack>;

    let item_base = BUTTON
        .animate_position(true)
        .sense_drag(true)
        .absorbs_clicks(false)
        .anchor_symm(Anchor::Center);

    let (stack_1, floater_1) = ui.add_component(DragAndDropStack { key: STACK_1, pos: Pos::Start });

    let (stack_2, floater_2) = ui.add_component(DragAndDropStack { key: STACK_2, pos: Pos::End });

    stack_1.nest(|| {
        for item in &state.items {
            let key = ITEM.sibling(&item);
            let node = item_base.text(&item).key(key);

            if ui.is_dragged(key).is_some() {
                floater_2.nest(|| {
                    ui.add(node);
                });
            } else {
                ui.add(node);
            }
        }
    });

    stack_2.nest(|| {
        for item in &state.items2 {
            let key = ITEM.sibling(&item);
            let node = item_base.text(&item).key(key);

            if ui.is_dragged(key).is_some() {
                floater_1.nest(|| {
                    ui.add(node);
                });
            } else {
                ui.add(node);
            }
        }
    });

    let mut released_from_1 = None;
    for item in &state.items {
        if ui.is_drag_released(ITEM.sibling(&item)) {
            released_from_1 = Some(*item);
        }
    }
    let mut released_from_2 = None;
    for item in &state.items2 {
        if ui.is_drag_released(ITEM.sibling(&item)) {
            released_from_2 = Some(*item);
        }
    }

    if let Some(drop) = ui.component_output(STACK_1) {
        if let Some(item) = released_from_2 {
            state.items2.retain(|s| *s != item);
            let idx = drop.insertion_index.min(state.items.len());
            state.items.insert(idx, item);
        }
    }

    if let Some(drop) = ui.component_output(STACK_2) {
        if let Some(item) = released_from_1 {
            state.items.retain(|s| *s != item);
            let idx = drop.insertion_index.min(state.items2.len());
            state.items2.insert(idx, item);
        }
    }


}

fn main() {
    let items = vec!["A", "B", "C", "D", "E"];
    let items2 = vec!["A2", "B2", "C2", "D2", "E2"];

    let state = State {
        items,
        items2,
    };

    run_example_loop(state, update_ui);
}
