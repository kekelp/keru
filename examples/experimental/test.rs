#![allow(unused)]
use keru::*;
use keru::node_library::*;
use keru::example_window_loop::*;

#[derive(Default)]
struct State {
    show: bool,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const TOGGLE: NodeKey;
    #[node_key] const BOX: NodeKey;

    let toggle = BUTTON
        .key(TOGGLE)
        .text("Toggle");

    let panel = PANEL
        .key(BOX)
        .grow_from_left()
        .shrink_to_right()
        .size_x(Size::Pixels(200.0))
        .size_y(Size::Pixels(200.0))
        .clip_children(true);

    let vstack = V_STACK.size_x(Size::Pixels(200.0)).size_y(Size::Pixels(500.0)).stack_arrange(Arrange::Start);
    
    ui.add(vstack).nest(|| { 
        ui.add(toggle);
        
        if state.show {
            ui.add(panel).nest(|| {
                ui.add(V_STACK).nest(|| {
                    ui.add(TEXT.text("aaaaaaa"));
                    ui.add(TEXT.text("aaaaaaa"));
                    ui.add(TEXT.text("aaaaaaa"));
                    ui.add(TEXT.text("aaaaaaa"));
                });
            });
        }
    });

    if ui.is_clicked(TOGGLE) {
        state.show = !state.show;
    }
}

fn main() {
    run_example_loop(State::default(), update_ui);
}
