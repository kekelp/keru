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

    let container = CONTAINER.size_x(Size::Frac(0.3)).position_y(Pos::Start);

    let item_base = BUTTON
        .animate_position(true)
        .sense_drag(true)
        .absorbs_clicks(false)
        .anchor_symm(Anchor::Center);

    // Find which item (if any) was just released from a drag
    let mut released_from_1: Option<&'static str> = None;
    for item in &state.items {
        if ui.is_drag_released(ITEM.sibling(&item)) {
            released_from_1 = Some(*item);
            break;
        }
    }
    let mut released_from_2: Option<&'static str> = None;
    for item in &state.items2 {
        if ui.is_drag_released(ITEM.sibling(&item)) {
            released_from_2 = Some(*item);
            break;
        }
    }

    let stack_1 = DragAndDropStack { key: STACK_1 };

    ui.add(container.position_x(Pos::Start)).nest(|| {
        ui.add_component(stack_1).nest(|| {
            for item in &state.items {
                let key = ITEM.sibling(&item);
                let node = item_base.text(&item).key(key);

                if let Some(drag) = ui.is_dragged(key) {
                    let (x, y) = (Pos::Pixels(drag.absolute_pos.x), Pos::Pixels(drag.absolute_pos.y));
                    ui.jump_to_root().nest(|| {
                        ui.add(node.position(x, y));
                    });
                } else {
                    ui.add(node);
                }
            }
        });

        // If something was dropped onto stack_1, move it from items2 to items
        if let Some(drop) = ui.component_output(STACK_1) {
            if let Some(item) = released_from_2 {
                state.items2.retain(|s| *s != item);
                let idx = drop.insertion_index.min(state.items.len());
                state.items.insert(idx, item);
            }
        }
    });

    let stack_2 = DragAndDropStack { key: STACK_2 };

    ui.add(container.position_x(Pos::End)).nest(|| {
        ui.add_component(stack_2).nest(|| {
            for item in &state.items2 {
                let key = ITEM.sibling(&item);
                let node = item_base.text(&item).key(key);

                if let Some(drag) = ui.is_dragged(key) {
                    let (x, y) = (Pos::Pixels(drag.absolute_pos.x), Pos::Pixels(drag.absolute_pos.y));
                    ui.jump_to_root().nest(|| {
                        ui.add(node.position(x, y));
                    });
                } else {
                    ui.add(node);
                }
            }
        });

        // If something was dropped onto stack_2, move it from items to items2
        if let Some(drop) = ui.component_output(STACK_2) {
            if let Some(item) = released_from_1 {
                state.items.retain(|s| *s != item);
                let idx = drop.insertion_index.min(state.items2.len());
                state.items2.insert(idx, item);
            }
        }
    });

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
