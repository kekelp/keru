#![allow(unused)]
use keru::*;
use keru::node_library::*;
use keru::example_window_loop::*;

#[derive(Default)]
struct State {
    show: bool
}

#[node_key] const TEXT_EDIT_LINE_KEY: NodeKey;
#[node_key] const OTHER_CONTENT_KEY: NodeKey;
#[node_key] const V_STACK_KEY: NodeKey;

fn update_ui(state: &mut State, ui: &mut Ui) {


    // ui.add(PANEL.children_can_hide(true)).nest(|| {
    //     if state.show {
    //         // // The text edit line stays interactable even when hidden
    //         ui.add(V_STACK.key(V_STACK_KEY)).nest(|| {
    //             ui.add(TEXT_EDIT_LINE.key(TEXT_EDIT_LINE_KEY).placeholder_text("Text ddit"));
    //         });

    //         // // works fine without the stack
    //         // ui.add(TEXT_EDIT_LINE.key(TEXT_EDIT_LINE_KEY).placeholder_text("Text edit"));

    //     } else {
    //         ui.add(LABEL.key(OTHER_CONTENT_KEY).static_text("Other content"));
    //     }
    // });

    ui.add(V_STACK.size(Size::Pixels(200.0), Size::Pixels(200.0)).clip_children(true).children_can_hide(false)).nest(|| {
        if state.show {
            ui.add(PANEL.slide_from_top()).nest(|| {
                ui.add(V_STACK).nest(|| {
                    ui.add(LABEL.text("Sneed2"));
                    ui.add(LABEL.text("Sneed3"));
                    ui.add(LABEL.text("Sneed4"));
                });
            });
        }

        ui.add(LABEL.text("Feed").position_y(Pos::End).animate_position(true))
    });

    if ui.add(BUTTON.text("Switch").position_y(Pos::End)).is_clicked(ui) {
        state.show = !state.show;
    }

}

fn main() {
    let state = State { show: true };
    example_window_loop::run_example_loop(state, update_ui);
}


