/// Another way to do advanced drag and drop using hitboxes that are invisible copies of items.
/// It would be better for performance if, instead of doing copies of items, we grabbed the size of the rect of the real item. But this looks cooler.

use keru::*;
use keru::example_window_loop::*;

#[derive(Default)]
pub struct State {
    pub left_strings: Vec<String>,
    pub right_strings: Vec<String>,
}

impl State {
    fn update_ui(&mut self, ui: &mut Ui) {
        ui.add(LABEL.text("Drag from left to right").position_y(Pos::End));

        #[node_key] const RIGHT_SPACER: NodeKey;
        #[node_key] const HITBOX: NodeKey;
        #[node_key] const ITEM: NodeKey;

        // Have to control these manually to get the hitboxes to overlap correctly.
        let item_padding = 10.0;
        let stack_spacing = 10.0;

        let stack = V_STACK
            .padding(50.0)
            .stack_spacing(stack_spacing)
            .size_y(Size::Fill)
            .position_y(Pos::Start)
            .stack_arrange(Arrange::Start);

        let left_stack = stack.position_x(Pos::Start);
        let right_stack = stack.position_x(Pos::End);
        let hitbox_stack = stack.position_x(Pos::End).stack_spacing(0.0);

        let item = BUTTON
            .size_x(Size::Pixels(100.0))
            .anchor_symm(Anchor::Center)
            .sense_drag(true)
            .padding(item_padding)
            .absorbs_clicks(false)
            .animate_position(true);

        let spacer = item
            .size_x(Size::Pixels(100.0))
            .size_y(Size::Pixels(50.0))
            .invisible()
            .key(RIGHT_SPACER)
            .animate_position(true);

        let hitbox = item
            .padding(item_padding + stack_spacing / 2.0)
            .sense_drag(false)
            .sense_drag_drop_target(true)
            .absorbs_clicks(false)
            .invisible();

        // Check which item is being hovered and where (top/bottom half)
        let mut right_hovered_i: Option<usize> = None;
        let mut right_release_i: Option<usize> = None;

        let mut left_dragged_i: Option<usize> = None;
        let mut left_drag_key: Option<NodeKey> = None;

        for (left_i, left_string) in self.left_strings.iter().enumerate() {
            
            if ui.is_dragged(ITEM.sibling(left_string)).is_some() {
                left_dragged_i = Some(left_i);
                left_drag_key = Some(ITEM.sibling(left_string));
            }
        }

        if let Some(left_drag_key) = left_drag_key {

            for (right_i, right_string) in self.right_strings.iter().enumerate() {
                let hitbox_key = HITBOX.sibling(right_string);

                if let Some(drag) = ui.is_drag_hovered_onto(left_drag_key, hitbox_key) {
                    // Top half = insert before, bottom half = insert after
                    if drag.relative_position.y < 0.5 {
                        right_hovered_i = Some(right_i);
                    } else {
                        right_hovered_i = Some(right_i + 1);
                    }
                }

                if let Some(drag) = ui.is_drag_released_onto(left_drag_key, hitbox_key) {
                    let idx = if drag.relative_position.y < 0.5 {
                        right_i
                    } else {
                        right_i + 1
                    };
                    right_release_i = Some(idx);
                }
            }
        }

        // Left stack
        ui.add(left_stack).nest(|| {
            for string in &self.left_strings {
                let key = ITEM.sibling(string);
                let item = item.text(&string).key(key);

                if let Some(drag) = ui.is_dragged(key) {
                    let (x, y) = (Pos::Pixels(drag.absolute_pos.x), Pos::Pixels(drag.absolute_pos.y));
                    ui.jump_to_root().nest(|| {
                        ui.add(item.position(x, y));
                    });
                } else {
                    ui.add(item);
                }
            }
        });

        // Right stack with spacer
        ui.add(right_stack).nest(|| {
            for (i, string) in self.right_strings.iter().enumerate() {

                if let Some(d) = left_dragged_i && right_hovered_i == Some(i) {
                    ui.add(spacer.text(&self.left_strings[d]));
                }

                let key = ITEM.sibling(string);
                ui.add(item.text(&string).key(key));
            }

            if let Some(d) = left_dragged_i && right_hovered_i == Some(self.right_strings.len()) {
                ui.add(spacer.text(&self.left_strings[d]));
            }
        });

        // Invisible hitbox stack
        ui.add(hitbox_stack).nest(|| {
            let last = self.right_strings.len().saturating_sub(1);
            for (i, string) in self.right_strings.iter().enumerate() {
                
                let size = if i == last { Size::Fill } else { Size::FitContent };
            
                let hitbox = hitbox
                    .text(&string)
                    .size_y(size)
                    .key(HITBOX.sibling(string));
    
                ui.add(hitbox);
            }
        });


        
        if let Some(left_dragged_i) = left_dragged_i {
            if let Some(right_release_i) = right_release_i {
                let removed = self.left_strings.remove(left_dragged_i);
                let clamped_idx = right_release_i.min(self.right_strings.len());
                self.right_strings.insert(clamped_idx, removed);
            }
        }
    }
}

fn main() {
    let mut state = State::default();
    state.left_strings = vec!["1".into(), "2".into(), "3".into(), "4".into()];
    state.right_strings = vec!["a".into(), "b".into(), "c".into(), "d".into()];
    run_example_loop(state, State::update_ui);
}
