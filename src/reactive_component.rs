use crate as keru;
use keru::*;

/// A [`Component`] that can skip rerunning the UI declaration code if its state didn't change. The effects still run on every frame.
///
/// An easy way to implement the required `state_changed` method is by using the `Observer` struct.
/// 
/// Reactive components can't be nested.
pub trait ReactiveComponent {
    /// See [`Component::State`].
    type State;

    /// The type returned by [`ReactiveComponent::run_effects()`]. The component user will receive it back when calling [`Ui::add_component()`].
    ///
    /// Do not use this to return a [`UiParent`]: children nested into a kept branch would be added a second time.
    type AddResult;

    /// See [`Component::ComponentOutput`].
    type ComponentOutput: Default;

    /// Return `true` if the nodes must be declared again in this frame. Changes done in [`ReactiveComponent::run_effects()`] are already visible here.
    fn state_changed(&self, ui: &Ui, state: &Self::State) -> bool;

    /// Run the component's side effects, such as reacting to clicks and drags.
    /// 
    /// This function runs on every frame, because the component has to respond to input even when its state didn't change.
    fn run_effects(&mut self, ui: &mut Ui, state: &mut Self::State) -> Self::AddResult;

    /// Add the component's nodes to the `Ui`.
    fn add_to_ui(&mut self, ui: &mut Ui, state: &Self::State);

    /// See [`Component::component_key()`].
    fn component_key(&self) -> Option<ComponentKey<Self>> {
        None
    }

    /// See [`Component::run_component()`].
    fn run_component(_ui: &mut Ui) -> Self::ComponentOutput {
        Self::ComponentOutput::default()
    }
}

impl<T: ReactiveComponent> Component for T {
    type State = T::State;
    type AddResult = T::AddResult;
    type ComponentOutput = T::ComponentOutput;

    fn add_to_ui(&mut self, ui: &mut Ui, state: &mut T::State) -> T::AddResult {
        let result = self.run_effects(ui, state);

        // The state check alone is not enough: on the first frame, or after the nodes were hidden or removed, there is nothing valid to keep.
        if self.state_changed(ui, state) || !ui.can_keep_component_nodes() {
            self.add_to_ui(ui, state);
            ui.save_component_last_node();
        } else {
            ui.keep_component_nodes();
        }

        return result;
    }

    fn component_key(&self) -> Option<ComponentKey<Self>> {
        ReactiveComponent::component_key(self)
    }

    fn run_component(ui: &mut Ui) -> T::ComponentOutput {
        <T as ReactiveComponent>::run_component(ui)
    }
}

impl Ui {
    fn current_component_root(&self) -> Option<NodeI> {
        return self.sys.nodes.get_by_id(thread_local::last_key_scope()?);
    }

    fn component_last_node(&self, root: NodeI) -> Option<NodeI> {
        return self.sys.nodes.get_by_id(self.sys.nodes[root].component_last_node?);
    }

    fn can_keep_component_nodes(&self) -> bool {
        let Some(root) = self.current_component_root() else { return false };
        let Some(last) = self.component_last_node(root) else { return false };
        return self.sys.nodes[last].last_frame_touched + 1 == self.sys.current_frame;
    }

    fn save_component_last_node(&mut self) {
        let Some(root) = self.current_component_root() else { return };
        let (parent, sibling_cursor, _depth) = thread_local::current_parent(self.sys.unique_id);
        let last = match sibling_cursor {
            SiblingCursor::After(i) => Some(i),
            _ => self.sys.nodes.links[parent].last_child,
        };
        self.sys.nodes[root].component_last_node = match last {
            Some(i) if i != root => Some(self.sys.nodes[i].id),
            _ => None,
        };
    }

    fn keep_component_nodes(&mut self) {
        let Some(root) = self.current_component_root() else { return };
        let Some(last) = self.component_last_node(root) else { return };
        let mut current = self.sys.nodes[root].old_next_sibling;
        while let Some(i) = current {
            // Save next before readd_branch_at clears it.
            current = self.sys.nodes.links[i].next_sibling;
            self.readd_nodei_branch(i);
            if i == last {
                break;
            }
        }
    }
}
