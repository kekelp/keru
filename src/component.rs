use crate as keru;
use keru::*;
use std::{any::TypeId, collections::hash_map::Entry};


/// Trait for a reusable Ui component.
///
/// This trait is simpler version of [`Component`] for simple components that don't use [`Component::State`], [`Component::AddResult`] or [`Component::ComponentOutput`].
pub trait SimpleComponent {
    /// Add the component's nodes to the `Ui` and run any side effects.
    /// 
    /// When the component's user calls [`Ui::add_component()`], the [`Ui`] will do some setup, then call this function.
    fn add_to_ui(&mut self, ui: &mut Ui);
}

/// Trait for a reusable Ui component.
pub trait Component {
    /// State that the [`Ui`] will automatically associate with each instance of this component.
    /// 
    /// If you don't need this, you can set it to the empty type `()`. Unfortunately, Rust doesn't allow traits to provide default values for their associated types.
    /// Consider also using [`SimpleComponent`].
    type State: Default + 'static;

    /// The type returned by [`Component::add_to_ui()`]. The component user will receive it back when calling [`Ui::add_component()`].
    /// 
    /// It can be used in two main ways:
    /// - return an [`UiParent`] to allow the component user to nest children into one of the element's nodes.
    /// - return the result of the app user's interaction with a node within the component.
    /// 
    /// If you don't need this, you can set it to the empty type `()`. Unfortunately, Rust doesn't allow traits to provide default values for their associated types.
    /// Consider also using [`SimpleComponent`].
    type AddResult;

    /// The type returned by [`Component::run_component()`]. The component user will receive it back when calling [`Ui::run_component()`].
    /// 
    /// If you don't need this, you can set it to the empty type `()`. Unfortunately, Rust doesn't allow traits to provide default values for their associated types.
    /// Consider also using [`SimpleComponent`].
    type ComponentOutput;

    /// Add the component's nodes to the `Ui` and run any side effects.
    /// 
    /// When the component's user calls [`Ui::add_component()`], the [`Ui`] will do some setup, then call this function.
    /// 
    /// If [`Component::State`] is not `()`, the [`Ui`] will initialize it as `Default` it when the component is first added, store it in its internal memory, and pass it to this function every time it's called. Then, it will drop the value when the component is removed from the tree.
    /// 
    /// If the same Component is added to the Ui multiple times in the same frame, each instance will get its own "private key space", so [`NodeKeys`] used inside this functions will "just work" without conflicts.
    fn add_to_ui(&mut self, ui: &mut Ui, state: &mut Self::State) -> Self::AddResult;

    /// Allow a component to be associated with a key. This will allow code in [`Component::run_component()`] to enter the component's "key space" and access its nodes using the same keys used in the [`Component::add_to_ui()`].
    /// 
    /// Without this system, if we used the same `Component` multiple times, its internal keys would become ambiguous and it would be impossible to refer to them from outside.
    /// 
    /// 
    fn component_key(&self) -> Option<ComponentKey<Self>> {
        None
    }

    /// Enter a component's space and run some additional logic for it. Requires [`Component::component_key()`] 
    /// 
    /// The user can call [`ui.run_component(key)`](`Ui::run_component()`) with the same key 
    /// 
    /// In advanced components, this can be used in different ways, such as adjusting the component based on the children that have been added to it.
    /// 
    /// It's also possible to enter the component space manually 
    /// 
    /// See the "drag_and_drop_component" example for an example.
    fn run_component(_ui: &mut Ui) -> Option<Self::ComponentOutput> {
        None
    }
}

impl<T: SimpleComponent> Component for T {
    type AddResult = ();
    type State = ();
    type ComponentOutput = ();

    fn add_to_ui(&mut self, ui: &mut Ui, _state: &mut Self::State) -> Self::AddResult {
        SimpleComponent::add_to_ui(self, ui)
    }
}

impl Ui {
    #[track_caller]
    /// Add a component to the `Ui`.
    pub fn add_component<T: Component>(&mut self, mut component: T) -> T::AddResult {        
        let key = match component.component_key() {
            Some(key) => key.as_normal_key(),
            None => NodeKey::new(Id(caller_location_id()), "Anon component"),
        };
        
        // Add the component. This should do twinning, with_subtree_id, and everything.
        // todo: try removing this node.
        let (i, id) = self.add_or_update_node(key);
        self.set_params(i, &COMPONENT_ROOT.into());
        // Here, we have to pass the `&mut Ui` (`self`) and the reference to the state in `self.sys.user_state`.
        // Besides the dumb partial borrow issue, there's also a real issue: inside the `add_to_ui`, the user could re-add the same component and get a reference to the same state.
        // But that's impossible because of the subtree id system. If the user re-adds with the same *key*, he'd end up with a different *id* anyway because of `id_with_subtree()` (inside `add_or_update_node()`).
        //
        // So there can't be multiple references to the same state.
        //
        // If we really believe that, then we might as well use unsafe pointers. But we can also avoid the unsafe code and do this: remove the state from the hashmap, pass it to `add_to_ui` separately, then re-insert it. Since the state is inside a `Box` anyway, it can be moved in and out cheaply. We still do some extra hashing though.
        //
        // (When adding the same component multiple times, they are deduplicated by track_caller or by key twinning, but that wouldn't be a safety issue for the state anyway.)

        thread_local::push_parent(i, SiblingCursor::None, self.sys.unique_id);
        thread_local::push_subtree(id);

        let res;

        let stateless = TypeId::of::<T::State>() == TypeId::of::<()>();
        if stateless {
            // Safety: we know that T is () here because of the TypeId check. 
            let state_ref = unsafe { std::mem::transmute::<&mut (), &mut T::State>(&mut ()) };
            res = T::add_to_ui(&mut component, self, state_ref);

        } else {
            // Get the state or initialize it if it's not there yet.
            let mut state = match self.sys.user_state.entry(id) {
                Entry::Occupied(e) => e.remove(),
                // todo: try smallbox
                Entry::Vacant(_) => Box::new(T::State::default()),
            };
            let state_ref = state.downcast_mut().expect(DOWNCAST_ERROR);
    
            res = T::add_to_ui(&mut component, self, state_ref);
    
            // Put the state back in its place inside the Ui.
            let a = self.sys.user_state.insert(id, state);
            debug_assert!(a.is_none());
        };

        thread_local::pop_subtree();
        thread_local::pop_parent(self.sys.unique_id);

        return res;
    }

    /// Run additional logic for a component. Simple components don't use this method, but some advanced components might use it for various reasons:
    /// 
    /// - as a way to return a value from the component, such as whether an internal button is clicked.
    /// 
    /// - to modify the component after it has been added, for example after children have been added to it. 
    pub fn run_component<T: Component>(&mut self, component_key: ComponentKey<T>) -> Option<T::ComponentOutput>{
        // No twinning here, so use this old closure one.
        self.component_key_subtree(component_key).start(|| {
            T::run_component(self)
        })
    }

    /// Get a mutable reference to a component's state by its key.
    ///
    /// Returns `None` if the component is not currently a part of the Ui tree.
    pub fn component_state_mut<T: Component>(&mut self, component_key: ComponentKey<T>) -> Option<&mut T::State> {
        let id = component_key.as_normal_key().id_with_subtree();
        self.sys.user_state.get_mut(&id)?.downcast_mut()
    }
}

const DOWNCAST_ERROR: &str = "Keru: Internal error: Couldn't downcast component state to the expected type.";
