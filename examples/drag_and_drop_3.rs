/// Another way to do advanced drag and drop using hitboxes that are invisible copies of items.

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

        #[node_key] const LEFT_STACK: NodeKey;
        #[node_key] const RIGHT_STACK: NodeKey;
        #[node_key] const RIGHT_SPACER: NodeKey;
        #[node_key] const HITBOX: NodeKey;
        #[node_key] const ITEM: NodeKey;

        // Have to control these manually to get the hitboxes to overlap correctly.
        let item_padding = 10.0;
        let stack_spacing = 10.0;

        let item = BUTTON
            .size_x(Size::Pixels(100.0))
            .anchor_symm(Anchor::Center)
            .sense_drag(true)
            .padding(item_padding)
            .absorbs_clicks(false)
            .animate_position(true);

        let stack = V_STACK
            .padding(50.0)
            .stack_spacing(stack_spacing)
            .size_y(Size::Fill)
            .position_y(Pos::Start)
            .stack_arrange(Arrange::Start);

        let spacer = SPACER
            .size_x(Size::Pixels(100.0))
            .key(RIGHT_SPACER)
            .animate_position(true);

        // Check which item is being hovered and where (top/bottom half)
        let mut insertion_index: Option<usize> = None;
        let mut release_info: Option<(usize, usize)> = None;

        for (left_i, left_string) in self.left_strings.iter().enumerate() {
            let drag_key = ITEM.sibling(left_string);

            for (right_i, right_string) in self.right_strings.iter().enumerate() {
                let hitbox_key = HITBOX.sibling(right_string);

                if let Some(drag) = ui.is_drag_hovered_onto(drag_key, hitbox_key) {
                    dbg!(drag.relative_position.y);
                    // Top half = insert before, bottom half = insert after
                    if drag.relative_position.y < 0.5 {
                        insertion_index = Some(right_i);
                    } else {
                        insertion_index = Some(right_i + 1);
                    }
                }

                if let Some(drag) = ui.is_drag_released_onto(drag_key, hitbox_key) {
                    let idx = if drag.relative_position.y < 0.5 {
                        right_i
                    } else {
                        right_i + 1
                    };
                    release_info = Some((left_i, idx));
                }
            }
        }

        if let Some((left_idx, insert_idx)) = release_info {
            let removed = self.left_strings.remove(left_idx);
            let clamped_idx = insert_idx.min(self.right_strings.len());
            self.right_strings.insert(clamped_idx, removed);
        }

        // Get height of dragged item for spacer
        let mut dragged_item_height: Option<f32> = None;
        for string in &self.left_strings {
            let key = ITEM.sibling(string);
            if ui.is_dragged(key).is_some() {
                if let Some(rect) = ui.rect(key) {
                    dragged_item_height = Some(rect.size().y);
                }
                break;
            }
        }

        // Left stack
        let left_stack = stack.position_x(Pos::Start).key(LEFT_STACK);
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
        let right_stack = stack.position_x(Pos::End).key(RIGHT_STACK);
        ui.add(right_stack).nest(|| {
            for (i, string) in self.right_strings.iter().enumerate() {
                if insertion_index == Some(i) {
                    let height = dragged_item_height.unwrap_or(30.0);
                    ui.add(spacer.size_y(Size::Pixels(height)));
                }

                let key = ITEM.sibling(string);
                ui.add(item.text(&string).key(key));
            }

            if insertion_index == Some(self.right_strings.len()) {
                let height = dragged_item_height.unwrap_or(30.0);
                ui.add(spacer.size_y(Size::Pixels(height)));
            }
        });

        // Invisible hitbox stack - same layout as right stack but without the spacer
        let hitbox_stack = stack.position_x(Pos::End).stack_spacing(0.0);
        ui.add(hitbox_stack).nest(|| {
            for string in &self.right_strings {
                let hitbox = item
                    .text(&string)
                    .key(HITBOX.sibling(string))
                    .padding(item_padding + stack_spacing / 2.0)
                    .sense_drag(false)
                    .sense_drag_drop_target(true)
                    .absorbs_clicks(false)
                    .invisible()
                    .color(Color::rgba_f(1.0, 0.2, 0.5, 0.5));
                ui.add(hitbox);
            }
        });
    }
}

fn main() {
    let mut state = State::default();
    state.left_strings = vec!["1".into(), "2".into(), "3".into(), "4".into()];
    state.right_strings = vec!["a".into(), "b".into(), "c".into(), "d".into()];
    run_example_loop(state, State::update_ui);
}
