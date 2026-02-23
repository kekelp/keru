use keru::example_window_loop::*;
use keru::*;

fn update_ui(_state: &mut (), ui: &mut Ui) {

    #[node_key] const ITEM: NodeKey;
    #[component_key] const STACK_1: ComponentKey<DragAndDropStack>;
    #[component_key] const STACK_2: ComponentKey<DragAndDropStack>;
    let items = ["A", "B", "C", "D", "E"];

    let container = CONTAINER.size_x(Size::Frac(0.3)).position_y(Pos::Start);

    let stack_2 = DragAndDropStack { key: STACK_1 };

    ui.add(container.position_x(Pos::Start)).nest(|| {
        ui.add_component(stack_2).nest(|| {
            for item in items {
                ui.add(BUTTON.animate_position(true).text(&item).key(ITEM.sibling(&item)));
            }
        });
    
        ui.component_output(STACK_1);
    });

    let stack_2 = DragAndDropStack { key: STACK_2 };

    ui.add(container.position_x(Pos::End)).nest(|| {
        ui.add_component(stack_2).nest(|| {
            for item in items {
                ui.add(BUTTON.animate_position(true).text(&item).key(ITEM.sibling(&item)));
            }
        });
    
        ui.component_output(STACK_2);
    });




}

fn main() {
    run_example_loop((), update_ui);
}
