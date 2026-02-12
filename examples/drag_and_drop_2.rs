/// This one is a bit complicated...
/// Maybe it could be simpler by using invisible hitboxes instead of doing all that math by hand.

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

        #[node_key] const ITEM: NodeKey;

        let item = BUTTON
            .size_x(Size::Pixels(100.0))
            .anchor_symm(Anchor::Center)
            .sense_drag(true)
            .absorbs_clicks(false)
            .animate_position(true);

        let stack = V_STACK
            .padding(50.0)
            .size_y(Size::Fill)
            .position_y(Pos::Start)
            .sense_drag_drop_target(true)
            .stack_arrange(Arrange::Start);

        let spacer = SPACER
            .size_x(Size::Pixels(100.0))
            .key(RIGHT_SPACER)
            .animate_position(true);

        // Helper to calculate insertion index from cursor position
        let calc_insertion_index = |ui: &Ui, cursor_y: f32| -> usize {
            let mut found_index = self.right_strings.len();
            for (i, string) in self.right_strings.iter().enumerate() {
                let key = ITEM.sibling(string);
                // Any time we use functions like `rect`, we're also adding subtle one-frame-off imperfections.
                // The rect isn't calculated until the layout step at the end of the frame,
                // so we're using the rect from the last frame for this frame's calculation.
                if let Some(rect) = ui.rect(key) {
                    let midpoint_y = (rect.y[0] + rect.y[1]) / 2.0;
                    if cursor_y < midpoint_y {
                        found_index = i;
                        break;
                    }
                }
            }
            found_index
        };

        // Find hover info and calculate insertion index
        let mut insertion_index: Option<usize> = None;
        let mut dragged_item_height: Option<f32> = None;

        for (_, string) in self.left_strings.iter().enumerate() {
            let key = ITEM.sibling(string);
            if let Some(drag) = ui.is_drag_hovered_onto(key, RIGHT_STACK) {
                if let Some(rect) = ui.rect(key) {
                    dragged_item_height = Some(rect.size().y);
                }
                insertion_index = Some(calc_insertion_index(ui, drag.absolute_pos.y));
                break;
            }
        }

        // Check for drag release onto right stack
        let mut release_info: Option<(usize, usize)> = None;
        for (i, string) in self.left_strings.iter().enumerate() {
            let key = ITEM.sibling(string);
            if let Some(drag) = ui.is_drag_released_onto(key, RIGHT_STACK) {
                let insert_at = calc_insertion_index(ui, drag.absolute_pos.y);
                release_info = Some((i, insert_at));
                break;
            }
        }
        if let Some((left_idx, insert_at)) = release_info {
            let removed = self.left_strings.remove(left_idx);
            let clamped_idx = insert_at.min(self.right_strings.len());
            self.right_strings.insert(clamped_idx, removed);
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

        // Right stack
        let right_stack = stack.position_x(Pos::End).key(RIGHT_STACK);
        ui.add(right_stack).nest(|| {
            for (i, string) in self.right_strings.iter().enumerate() {
                // Insert spacer at the insertion point
                if Some(i) == insertion_index {
                    let height = dragged_item_height.unwrap_or(30.0);
                    ui.add(spacer.size_y(Size::Pixels(height)));
                }

                let key = ITEM.sibling(string);
                let item = item.text(&string).key(key);
                ui.add(item);
            }

            // Spacer at the end if inserting at the end
            if insertion_index == Some(self.right_strings.len()) {
                let height = dragged_item_height.unwrap_or(30.0);
                ui.add(spacer.size_y(Size::Pixels(height)));
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
