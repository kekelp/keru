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
        .anchor_symm(Anchor::Center);

    let special_panel = PANEL
        .animate_position(true)
        .sense_drag(true);

    let (stack, floater) = ui.add_component(DragAndDropStack { key: STACK });

    for &item in &state.items {
        let key = ITEM.sibling(&item);

        let stack_or_floater = if ui.is_dragged(key).is_some() { floater } else { stack };

        stack_or_floater.nest(|| {

            match item {
                "special" => {
                    ui.add(special_panel.key(key)).nest(|| {
                        ui.add(H_STACK).nest(|| {
                            ui.add(PANEL.color(Color::RED).size_symm(Size::Pixels(30.0)));
                            ui.add(PANEL.color(Color::GREEN).size_symm(Size::Pixels(30.0)));
                            ui.add(PANEL.color(Color::BLUE).size_symm(Size::Pixels(30.0)));
                        });
                    })
                }
                _ =>  {
                    let node = item_base.text(&item).key(key);
                    ui.add(node);
                }
            };

        });

    }


    let mut released_idx = None;
    for (i, item) in state.items.iter().enumerate() {
        if ui.is_drag_released(ITEM.sibling(&item)) {
            released_idx = Some(i);
        }
    }

    if let Some(drop) = ui.run_component(STACK) {
        if let Some(old_idx) = released_idx {
            let item = state.items.remove(old_idx);
            state.items.insert(drop.insertion_index.min(state.items.len()), item);
        }
    }
}

fn main() {
    let items = vec!["A", "special", "B", "C", "xxxxxx\nxxxxxx\nxxxxxx", "D", "E"];

    let state = State {
        items,
    };

    example_window_loop::run_example_loop(state, update_ui);
}
