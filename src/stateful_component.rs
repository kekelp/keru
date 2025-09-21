use std::collections::hash_map::Entry;

use crate as keru;
use keru::*;

pub trait StatefulComponentParams {
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

impl Ui {
    #[track_caller]
    pub fn add_stateful_component<T: StatefulComponentParams>(&mut self, component_params: T) -> T::AddResult {
        let key = match component_params.component_key() {
            Some(key) => key.as_normal_key(),
            None => NodeKey::new(Id(caller_location_id()), ""),
        };
        
        // Add the component. This should do twinning, with_subtree_id, and everything.
        let (i, id) = self.add_or_update_node(key);
        self.set_params(i, &COMPONENT_ROOT.into());

        // Initialize the state if it's not already there.
        if let Entry::Vacant(e) = self.sys.user_state.entry(id) {
            e.insert(Box::new(T::State::default()));
        }

        // Here, we have to pass the `&mut Ui` (`self`) and the reference to the state in `self.sys.user_state`.
        // Besides the dumb partial borrow issue, there's also a real issue: inside the `add_to_ui`, the user could re-add the same component and get a reference to the same state.
        // But that's impossible because of the subtree id system. If the user re-adds with the same *key*, he'd end up with a different id anyway because of `id_with_subtree()`.
        // So there can't be multiple references to the same state.
        // If we believe that, then we might as well use unsafe pointers. But we can also avoid the unsafe code and do this: remove the state from the hashmap, pass it to `add_to_ui` separately, then re-insert it. Since the state is inside a `Box` anyway, it can be moved in and out cheaply. We still do some extra hashing though.

        // Take the state out of the hashmap.
        let mut state = self.sys.user_state.remove(&id).unwrap();
        let state_ref = state.downcast_mut().expect("Keru: Internal error: Couldn't downcast component state to the expected type.");

        // todo: since there are no closures anymore, doesn't this mean that the subtree stack could be a normal field on the Ui, instead of being in the thread local?
        // yes, doing it this way is basically the same as reborrowing every time (|ui| { ... }, like in egui). Forcing the component's add_to_ui to borrow ui and state separately means that you don't have any problems.
        // todo: then I guess put it inside the Ui?
        thread_local::push_subtree(id);

        let res = T::add_to_ui(component_params, self, state_ref);

        thread_local::pop_subtree();

        // Put the state back in its place inside the Ui.
        match self.sys.user_state.entry(id) {
            Entry::Vacant(e) => e.insert(state),
            Entry::Occupied(_) => panic!("Keru: Internal error: different components ended up using the same state?"),
        };
    
        return res;
    }

    pub fn stateful_component_output<T: StatefulComponentParams>(&mut self, component_key: ComponentKey<T>) -> Option<T::ComponentOutput> {
        // No twinning here, so use this old closure one which does twinning
        self.component_key_subtree(component_key).start(|| {
            T::component_output(self)
        })
    }
}
