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

    let stack_1 = DragAndDropStack { key: STACK_1 };

    ui.add(container.position_x(Pos::Start)).nest(|| {
        ui.add_component(stack_1).nest(|| {
            for item in &state.items {
                ui.add(BUTTON.animate_position(true).text(&item).key(ITEM.sibling(&item)));
            }
        });
    
        ui.component_output(STACK_1);
    });

    let stack_2 = DragAndDropStack { key: STACK_2 };

    ui.add(container.position_x(Pos::End)).nest(|| {
        ui.add_component(stack_2).nest(|| {
            for item in &state.items2 {
                ui.add(BUTTON.animate_position(true).text(&item).key(ITEM.sibling(&item)));
            }
        });
    
        ui.component_output(STACK_2);
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
