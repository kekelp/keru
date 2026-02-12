/// Experiment: drag and drop list as a component.
///
/// Notes for future work:
/// - Need a "jump" method to add a node as the nth child (for inserting spacer at correct position)
/// - For now: spacer is appended at the end, but hover detection uses correct index

use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {
    pub items: Vec<String>,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    ui.add(LABEL.text("Drag to reorder (component experiment)").position_y(Pos::End));

    #[node_key] const ITEM: NodeKey;

    let item_style = BUTTON
        .size_x(Size::Pixels(150.0))
        .anchor_symm(Anchor::Center)
        .sense_drag(true)
        .absorbs_clicks(false)
        .animate_position(true);

    let container = CONTAINER
        .size(Size::Frac(0.4), Size::Frac(0.8))
        .position(Pos::Center, Pos::Center)
        .padding(20.0);

    // Find which item is being dragged (if any)
    let mut dragged_key: Option<NodeKey> = None;
    for item in &state.items {
        let key = ITEM.sibling(item);
        if ui.is_dragged(key).is_some() {
            dragged_key = Some(key);
            break;
        }
    }

    ui.add(container).nest(|| {
        let drag_list = DragList { dragged_key };
        let (list_parent, dragged_parent) = ui.add_component(drag_list);

        list_parent.nest(|| {
            for item in &state.items {
                let key = ITEM.sibling(item);
                // Don't add the dragged item to the normal list position
                if dragged_key != Some(key) {
                    ui.add(item_style.text(item).key(key));
                }
            }
        });

        dragged_parent.nest(|| {
            // Render dragged item at mouse position
            for item in &state.items {
                let key = ITEM.sibling(item);
                if let Some(drag) = ui.is_dragged(key) {
                    let (x, y) = (Pos::Pixels(drag.absolute_pos.x), Pos::Pixels(drag.absolute_pos.y));
                    ui.add(item_style.text(item).key(key).position(x, y));
                }
            }
        });
    });
}

fn main() {
    let mut state = State::default();
    state.items = vec![
        "Item 1".into(),
        "Item 2".into(),
        "Item 3".into(),
        "Item 4".into(),
        "Item 5".into(),
    ];
    run_example_loop(state, update_ui);
}

// ============ Component ============

pub struct DragList {
    /// The key of the item currently being dragged (if any)
    pub dragged_key: Option<NodeKey>,
}

#[derive(Default)]
pub struct DragListState {
    /// Insertion index when hovering (from previous frame's calculation)
    pub hover_index: Option<usize>,
}

impl Component for DragList {
    type AddResult = (UiParent, UiParent);
    type ComponentOutput = ();
    type State = DragListState;

    fn add_to_ui(&mut self, ui: &mut Ui, state: &mut Self::State) -> Self::AddResult {
        #[node_key] const STACK: NodeKey;
        #[node_key] const SPACER: NodeKey;
        #[node_key] const DRAGGED_CONTAINER: NodeKey;

        let stack = V_STACK
            .size_y(Size::Fill)
            .position_y(Pos::Start)
            .stack_arrange(Arrange::Start)
            .sense_drag_drop_target(true)
            .key(STACK);

        let spacer = SPACER
            .size_x(Size::Pixels(150.0))
            .size_y(Size::Pixels(40.0))
            .key(SPACER)
            .animate_position(true);

        // Get children rects from previous frame to calculate hover index
        let children_rects = ui.children_rects(STACK);

        // Calculate hover index if we have a dragged key
        state.hover_index = None;
        if let Some(dragged_key) = self.dragged_key {
            if let Some(drag) = ui.is_drag_hovered_onto(dragged_key, STACK) {
                let cursor_y = drag.absolute_pos.y;
                let screen_h = ui.screen_size().1;

                // Find insertion index based on cursor position
                let mut found = children_rects.len();
                for (i, rect) in children_rects.iter().enumerate() {
                    let midpoint_y = ((rect.y[0] + rect.y[1]) / 2.0) * screen_h;
                    if cursor_y < midpoint_y {
                        found = i;
                        break;
                    }
                }
                state.hover_index = Some(found);
            }
        }

        // Add the stack
        let list_parent = ui.add(stack);

        // Add spacer at the end if hovering
        // (todo: need jump_to_nth_child to insert at correct position)
        if state.hover_index.is_some() {
            list_parent.nest(|| {
                ui.add(spacer);
            });
        }

        // Container for dragged items (at root)
        let dragged_parent = ui.jump_to_root().nest(|| {
            ui.add(CONTAINER.key(DRAGGED_CONTAINER).invisible())
        });

        (list_parent, dragged_parent)
    }
}
