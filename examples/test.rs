use keru::example_window_loop::*;
use keru::*;

fn update_ui(_state: &mut (), ui: &mut Ui) {
    #[node_key] const ITEM: NodeKey;

    let items = ["A", "B", "C", "D", "E"];

    ui.add(H_STACK).nest(|| {
        ui.add(V_STACK).nest(|| {
            for item in items {
                ui.add(BUTTON.text(&item).key(ITEM.sibling(&item)));
            }
        });

        // Add a red "X" between "B" and "C"
        let jump_key = ITEM.sibling("B");
        ui.jump_to_sibling(jump_key).unwrap().nest(|| {
            ui.add(BUTTON.text("X").color(Color::RED));
            ui.add(BUTTON.text("Y").color(Color::RED));
        });
    });


}

fn main() {
    run_example_loop((), update_ui);
}
