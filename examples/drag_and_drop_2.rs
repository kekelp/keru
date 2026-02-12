use keru::*;
use keru::example_window_loop::*;

#[derive(Default)]
pub struct State {
    pub left_strings: Vec<String>,
    pub right_strings: Vec<String>,
    // Persisted insertion index for when drag is released
    pub hover_insertion_index: Option<usize>,
    pub hover_item_height: Option<f32>,
}

impl State {
    fn update_ui(&mut self, ui: &mut Ui) {
        #[node_key] const LEFT_STACK: NodeKey;
        #[node_key] const RIGHT_STACK: NodeKey;
        #[node_key] const SPACER: NodeKey;

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

        // Find which left item is being dragged over the right stack, and get cursor position
        let mut dragged_item_info: Option<(usize, glam::Vec2)> = None;
        for (i, string) in self.left_strings.iter().enumerate() {
            let key = ITEM.sibling(string);
            if let Some(drag) = ui.is_dragged(key) {
                if ui.is_drag_hovered_onto(key, RIGHT_STACK) {
                    dragged_item_info = Some((i, drag.absolute_pos));
                }
            }
        }

        // Calculate insertion index based on cursor position vs item midpoints
        // Store in self so it persists when drag is released
        if let Some((dragged_idx, cursor_pos)) = &dragged_item_info {
            let dragged_key = ITEM.sibling(&self.left_strings[*dragged_idx]);
            if let Some(rect) = ui.rect(dragged_key) {
                self.hover_item_height = Some(rect.size().y);
            }

            let cursor_y = cursor_pos.y;
            let mut found_index = self.right_strings.len(); // default: insert at end

            for (i, string) in self.right_strings.iter().enumerate() {
                let key = ITEM.sibling(string);
                if let Some(rect) = ui.rect(key) {
                    let midpoint_y = (rect.y[0] + rect.y[1]) / 2.0;
                    if cursor_y < midpoint_y {
                        found_index = i;
                        break;
                    }
                }
            }
            self.hover_insertion_index = Some(found_index);
        } else {
            // Not hovering over right stack anymore, but keep the value for release frame
        }

        // Check for drag release onto right stack
        let mut release_info: Option<(usize, usize)> = None; // (left_idx, insert_at)
        for (i, string) in self.left_strings.iter().enumerate() {
            let key = ITEM.sibling(string);
            if ui.is_drag_released_onto(key, RIGHT_STACK) {
                // Use the persisted insertion index
                let insert_at = self.hover_insertion_index.unwrap_or(self.right_strings.len());
                release_info = Some((i, insert_at));
            }
        }

        if let Some((left_idx, insert_at)) = release_info {
            let removed = self.left_strings.remove(left_idx);
            let clamped_idx = insert_at.min(self.right_strings.len());
            self.right_strings.insert(clamped_idx, removed);
            // Clear hover state after insertion
            self.hover_insertion_index = None;
            self.hover_item_height = None;
        }

        // Left stack - items can be dragged from here
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

        // Right stack - items move apart to make space when hovering
        let right_stack = stack.position_x(Pos::End).key(RIGHT_STACK);
        let insertion_index = if dragged_item_info.is_some() { self.hover_insertion_index } else { None };
        let dragged_item_height = self.hover_item_height;

        ui.add(right_stack).nest(|| {
            for (i, string) in self.right_strings.iter().enumerate() {
                // Insert spacer at the insertion point
                if Some(i) == insertion_index {
                    let height = dragged_item_height.unwrap_or(30.0);
                    let spacer = PANEL
                        .key(SPACER)
                        .size_x(Size::Pixels(100.0))
                        .size_y(Size::Pixels(height))
                        .animate_position(true);
                    ui.add(spacer);
                }

                let key = ITEM.sibling(string);
                let item = item.text(&string).key(key);
                ui.add(item);
            }

            // Spacer at the end if inserting at the end
            if insertion_index == Some(self.right_strings.len()) {
                let height = dragged_item_height.unwrap_or(30.0);
                let spacer = PANEL
                    .key(SPACER)
                    .size_x(Size::Pixels(100.0))
                    .size_y(Size::Pixels(height))
                    .animate_position(true);

                ui.add(spacer);
            }
        });
    }
}

fn main() {
    let mut state = State::default();
    state.left_strings = vec!["1".into(), "2".into(), "3".into(), "4".into(), "5".into()];
    state.right_strings = vec!["a".into(), "b".into(), "c".into(), "d".into(), "e".into()];
    run_example_loop(state, State::update_ui);
}
