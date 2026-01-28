use std::{any::TypeId, collections::hash_map::Entry};

const DOWNCAST_ERROR: &str = "Keru: Internal error: Couldn't downcast component state to the expected type.";

use crate as keru;
use keru::*;

pub trait Component2 {
    type AddResult;
    type ComponentOutput;
    type State: Default + 'static;

    fn add_to_ui(self, ui: &mut Ui, state: &mut Self::State) -> Self::AddResult;

    // this returns an Option mostly just so that we can default impl it with None, but maybe that's useful in other ways?
    // as in, if the component is not currently added, maybe Ui::component_output can just see that and return None, instead of running the function anyway and (hopefully) getting a None?
    // todo: figure this out
    fn component_output(_ui: &mut Ui) -> Option<Self::ComponentOutput> {
        None
    }

    fn component_key(&self) -> Option<ComponentKey<Self>> {
        None
    }
}

/// A simpler version of [`StatefulComponent`] for stateless components that don't return a value.
///
/// This trait automatically implements [`StatefulComponent`] with all associated types set to `()`.
pub trait Component {
    fn add_to_ui(self, ui: &mut Ui);

    fn component_key(&self) -> Option<ComponentKey<Self>> {
        None
    }
}

impl<T: Component> Component2 for T {
    type AddResult = ();
    type ComponentOutput = ();
    type State = ();

    fn add_to_ui(self, ui: &mut Ui, _state: &mut Self::State) -> Self::AddResult {
        Component::add_to_ui(self, ui)
    }

    fn component_key(&self) -> Option<ComponentKey<Self>> {
        Component::component_key(self)
    }
}

impl Ui {
    #[track_caller]
    pub fn add_stateful_component<T: Component2>(&mut self, component_params: T) -> T::AddResult {        
        let key = match component_params.component_key() {
            Some(key) => key.as_normal_key(),
            None => NodeKey::new(Id(caller_location_id()), ""),
        };
        
        // Add the component. This should do twinning, with_subtree_id, and everything.
        // todo: try removing this.
        let (i, id) = self.add_or_update_node(key);
        self.set_params(i, &COMPONENT_ROOT.into());
        let parent = UiParent::new(i);

        // Here, we have to pass the `&mut Ui` (`self`) and the reference to the state in `self.sys.user_state`.
        // Besides the dumb partial borrow issue, there's also a real issue: inside the `add_to_ui`, the user could re-add the same component and get a reference to the same state.
        // But that's impossible because of the subtree id system. If the user re-adds with the same *key*, he'd end up with a different id anyway because of `id_with_subtree()`.
        // So there can't be multiple references to the same state.
        // If we really believe that, then we might as well use unsafe pointers. But we can also avoid the unsafe code and do this: remove the state from the hashmap, pass it to `add_to_ui` separately, then re-insert it. Since the state is inside a `Box` anyway, it can be moved in and out cheaply. We still do some extra hashing though.
        
        thread_local::push_parent(&parent);
        thread_local::push_subtree(id);

        let res;

        let stateless = TypeId::of::<T::State>() == TypeId::of::<()>();
        if stateless {
            // Safety: we know that T is () here because of the TypeId check. 
            let state_ref = unsafe { std::mem::transmute::<&mut (), &mut T::State>(&mut ()) };
            res = T::add_to_ui(component_params, self, state_ref);
                        
        } else {
            // Get the state or initialize it if it's not there yet.
            let mut state = match self.sys.user_state.entry(id) {
                Entry::Occupied(e) => e.remove(),
                Entry::Vacant(_) => Box::new(T::State::default()),
            };
            let state_ref = state.downcast_mut().expect(DOWNCAST_ERROR);
    
            res = T::add_to_ui(component_params, self, state_ref);
    
            // Put the state back in its place inside the Ui.
            let a = self.sys.user_state.insert(id, state);
            debug_assert!(a.is_none());
        };

        thread_local::pop_subtree();
        thread_local::pop_parent();

        return res;
    }

    pub fn stateful_component_output<T: Component2>(&mut self, component_key: ComponentKey<T>) -> Option<T::ComponentOutput> {
        // No twinning here, so use this old closure one.
        self.component_key_subtree(component_key).start(|| {
            T::component_output(self)
        })
    }
}
