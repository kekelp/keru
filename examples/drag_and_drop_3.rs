/// Manual implementation of a single rearrangeable stack.
/// This is the manual-style equivalent of test.rs, without using the DragAndDropStack component.

use keru::*;
use keru::example_window_loop::*;

#[derive(Default)]
pub struct State {
    pub items: Vec<String>,
}

impl State {
    fn update_ui(&mut self, ui: &mut Ui) {
        ui.add(LABEL.text("Drag to rearrange").position_y(Pos::End));

        #[node_key] const STACK: NodeKey;
        #[node_key] const ITEM: NodeKey;
        #[node_key] const SPACER_KEY: NodeKey;

        let item = BUTTON
            .size_x(Size::Pixels(100.0))
            .anchor_symm(Anchor::Center)
            .sense_drag(true)
            .absorbs_clicks(false)
            .animate_position(true);

        let stack = V_STACK
            .padding(50.0)
            .size_y(Size::Fill)
            .position(Pos::Center, Pos::Start)
            .sense_drag_drop_target(true)
            .stack_arrange(Arrange::Start)
            .key(STACK);

        let spacer = SPACER
            .size_x(Size::Pixels(100.0))
            .key(SPACER_KEY)
            .animate_position(true);

        // Helper to calculate insertion index from cursor position
        let calc_insertion_index = |ui: &Ui, cursor_y: f32, dragged_index: usize| -> usize {
            let mut found_index = self.items.len();
            for (i, item_str) in self.items.iter().enumerate() {
                // Skip the item being dragged in the calculation
                if i == dragged_index {
                    continue;
                }
                let key = ITEM.sibling(item_str);
                if let Some(node) = ui.get_node(key) {
                    let rect = node.rect();
                    let midpoint_y = (rect.y[0] + rect.y[1]) / 2.0;
                    if cursor_y < midpoint_y {
                        found_index = i;
                        break;
                    }
                }
            }
            found_index
        };

        // Find which item is being dragged and calculate insertion index
        let mut dragged_info: Option<(usize, usize, f32)> = None; // (dragged_index, insertion_index, height)

        for (i, item_str) in self.items.iter().enumerate() {
            let key = ITEM.sibling(item_str);
            if let Some(drag) = ui.is_drag_hovered_onto(key, STACK) {
                let height = ui.get_node(key).map(|node| node.rect().size().y).unwrap_or(30.0);
                let insertion_index = calc_insertion_index(ui, drag.absolute_pos.y, i);
                dragged_info = Some((i, insertion_index, height));
                break;
            }
        }

        // Check for drag release
        let mut release_info: Option<(usize, usize)> = None;
        for (i, item_str) in self.items.iter().enumerate() {
            let key = ITEM.sibling(item_str);
            if let Some(drag) = ui.is_drag_released_onto(key, STACK) {
                let insertion_index = calc_insertion_index(ui, drag.absolute_pos.y, i);
                release_info = Some((i, insertion_index));
                break;
            }
        }
        if let Some((old_index, new_index)) = release_info {
            let removed = self.items.remove(old_index);
            // Adjust insertion index if we removed from before it
            let adjusted_index = if old_index < new_index {
                (new_index - 1).min(self.items.len())
            } else {
                new_index.min(self.items.len())
            };
            self.items.insert(adjusted_index, removed);
        }

        // Render the stack
        ui.add(stack).nest(|| {
            for (i, item_str) in self.items.iter().enumerate() {
                // Insert spacer at the insertion point
                if let Some((_, insertion_index, height)) = dragged_info {
                    if i == insertion_index {
                        ui.add(spacer.size_y(Size::Pixels(height)));
                    }
                }

                let key = ITEM.sibling(item_str);
                let node = item.text(&item_str).key(key);

                // Check if this item is currently being dragged
                if let Some(drag) = ui.is_dragged(key) {
                    // Render at cursor position via root
                    let (x, y) = (Pos::Pixels(drag.absolute_pos.x), Pos::Pixels(drag.absolute_pos.y));
                    ui.jump_to_root().nest(|| {
                        ui.add(node.position(x, y));
                    });
                } else {
                    ui.add(node);
                }
            }

            // Spacer at the end if inserting at the end
            if let Some((_, insertion_index, height)) = dragged_info {
                if insertion_index == self.items.len() {
                    ui.add(spacer.size_y(Size::Pixels(height)));
                }
            }
        });
    }
}

fn main() {
    let mut state = State::default();
    state.items = vec!["A\nA".into(), "B\nA\nA".into(), "C\nA\nA\nA".into(), "D\nA\nA\nA\nA".into(), "E".into()];
    run_example_loop(state, State::update_ui);
}
