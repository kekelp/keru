#![allow(dead_code)]

use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {
    elements: Vec<u32>,
    next_id: u32,
    show: bool,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const ADD: NodeKey;
    #[node_key] const DELETE: NodeKey;


    
    // ui.add(V_STACK.size_symm(Size::Fill).padding(50)).nest(|| {
    //     for &id in &state.elements {
    //         ui.add(H_STACK.animation(animation)).nest(|| {
    //             ui.add(LABEL.text(&format!("Element {}", id)));
    //             ui.add(BUTTON.static_text("Delete").key(DELETE.s(id)));
    //         });
    //     }
    //     ui.add(BUTTON.key(ADD).static_text("SEETHE"));
    // });
    
    // // effects
    // state.elements.retain(|&id| !ui.is_clicked(DELETE.s(id)));

    // if ui.is_clicked(ADD) {
    //     state.elements.push(state.next_id);
    //     state.next_id += 1;
    // }



    ui.add(V_STACK.slide_from_top().size_x(Size::Pixels(200.0)).position_y(Position::Start)).nest(|| {

        if state.show {
            ui.add(V_STACK.grow_shrink(Axis::Y, Position::Start).clip_children_y(true)).nest(|| {
                ui.add(LABEL.text("SNEED"));
            });
        }

        ui.add(LABEL.text("FEED").animate_position(true));
        ui.add(H_LINE.animate_position(true));
        ui.add(LABEL.text("FEED2").animate_position(true));
        ui.add(H_LINE.animate_position(true));
        ui.add(LABEL.text("FEED3").animate_position(true));
    });

    ui.add(BUTTON.key(ADD).static_text("SEETHE").position_y(Position::End));
    if ui.is_clicked(ADD) {
        state.show = !state.show;
    }
}

fn main() {
    // basic_env_logger_init();
    let state = State {
        elements: vec![0, 1, 2],
        next_id: 3,
        show: true,
    };
    run_example_loop(state, update_ui);
}