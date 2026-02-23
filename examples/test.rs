use keru::example_window_loop::*;
use keru::*;

fn update_ui(_state: &mut (), ui: &mut Ui) {

    #[node_key] const ITEM: NodeKey;
    #[component_key] const MY_STACK: ComponentKey<DragAndDropStack>;
    let items = ["A", "B", "C", "D", "E"];

    let my_stack = DragAndDropStack { key: MY_STACK };
    ui.add_component(my_stack).nest(|| {
        for item in items {
            ui.add(BUTTON.text(&item).key(ITEM.sibling(&item)));
        }
    });

    // don't mind the name, this is what should add the spacer.
    ui.component_output(MY_STACK);

}

fn main() {
    run_example_loop((), update_ui);
}
