use keru::example_window_loop::*;
use keru::*;

fn update_ui(_state: &mut (), ui: &mut Ui) {

    let items = ["A", "B", "C", "D", "E"];

    #[node_key] const ITEM: NodeKey;
    #[node_key] const MY_STACK: NodeKey;
    let items = ["A", "B", "C", "D", "E"];
    ui.add(H_STACK).nest(|| {
        ui.add(V_STACK.key(MY_STACK)).nest(|| {
            for item in items {
                ui.add(BUTTON.text(&item).key(ITEM.sibling(&item)));
            }
        });

        // Add a red "X" between "B" and "C"
        ui.jump_to_nth_child(MY_STACK, 2).unwrap().nest(|| {
            ui.add(BUTTON.text("X").color(Color::RED));
            ui.add(BUTTON.text("Y").color(Color::RED));
        });
    });


}

fn main() {
    run_example_loop((), update_ui);
}
