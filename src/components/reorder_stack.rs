use crate as keru;
use keru::*;
use keru::node_library::*;

pub struct ReorderStack {
    pub key: ComponentKey<Self>,
}
impl ReorderStack {
    #[node_key] const STACK: NodeKey;
    #[node_key] const SPACER: NodeKey;
}

impl Component for ReorderStack {
    type AddResult = UiParent;
    type ComponentOutput = Option<(usize, usize)>;
    type State = ();

    fn add_to_ui(&mut self, ui: &mut Ui, _state: &mut Self::State) -> Self::AddResult {

        let stack = V_STACK
            .animate_layout(true)
            .size(Size::Pixels(100.0), Size::Fill)
            .position_y(Pos::Start)
            .stack_arrange(Arrange::Start)
            .sense_drag_drop_target(true)
            .key(Self::STACK);

        return ui.add(stack);
    }

    fn component_key(&self) -> Option<ComponentKey<Self>> {
        return Some(self.key);
    }

    fn run_component(ui: &mut Ui) -> Self::ComponentOutput {
        // Find the dragged item
        let mut dragged = None;
        if let Some(stack) = ui.get_node(Self::STACK) {
            for (index, child) in stack.children().enumerate() {
                let key = child.temp_key();
                if child.is_dragged().is_some() || ui.is_drag_released(key) {
                    let height = child.rect().size().y;
                    dragged = Some((key, height, index));
                    break;
                }
            }
        }

        if let Some((key, height, index)) = dragged {
            // Find where it's being hovered
            let stack_node = ui.get_node(Self::STACK).unwrap();
            let cursor_y = ui.cursor_position().y;
            let mut insertion_index = stack_node.children_count();
            for (i, child) in stack_node.children().enumerate() {
                if i == index {
                    continue;
                }
                if cursor_y < child.center().y {
                    insertion_index = i;
                    break;
                }
            }

            let hovered = ui.is_drag_hovered_onto(key, Self::STACK).is_some();
            let drag_released = ui.is_drag_released_onto(key, Self::STACK).is_some();
            if hovered || drag_released {
                // Insert spacer at the calculated position
                ui.jump_to_nth_child(Self::STACK, insertion_index).unwrap().nest(|| {
                    let spacer = SPACER
                        .key(Self::SPACER)
                        .size_y(Size::Pixels(height))
                        .absorbs_clicks(false)
                        .animate_layout(true);

                    ui.add(spacer);
                });
            }

            let cursor = ui.cursor_position();

            ui.jump_to_root().nest(|| {
                let node = ui.get_node_mut(key).unwrap();
                node.set_position(Pos::Pixels(cursor.x), Pos::Pixels(cursor.y));
                node.re_add();
            });

            // Return swap indices when drag is released
            if drag_released && index != insertion_index {
                return Some((index, insertion_index));
            }
        }

        None
    }
}
