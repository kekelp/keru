use crate::{node_params::{ANON_HSTACK, ANON_VSTACK, H_STACK, V_STACK}, NodeParams, NodeRef, NodeType, Stack, TypedKey, Ui};

pub trait AddParentManual {
    fn add_parent<T: NodeType>(&mut self, key: TypedKey<T>, params: &NodeParams) -> NodeRef<T>;
    fn end_parent<T: NodeType>(&mut self, key: TypedKey<T>);
    // fn add_anon_parent(&mut self, params: &NodeParams, content_code: impl FnOnce(&mut Self)) -> NodeRef<Any>;
    fn v_stack(&mut self) -> NodeRef<Stack>;
    fn end_v_stack(&mut self);

    fn h_stack(&mut self) -> NodeRef<Stack>;
    fn end_h_stack(&mut self);
}
impl AddParentManual for Ui {
    fn add_parent<T: NodeType>(&mut self, key: TypedKey<T>, params: &NodeParams) -> NodeRef<T> {
        let i = self.update_node(key, params, true);
        return self.get_ref_unchecked(i, &key)
    }

    // todo: I wanted to add this checked version, but there is a twin-related problem here.
    // if the key got twinned, ended_parent will lead to the node with the twinned id, but the key will have the non-twinned id.
    // sounds stupid to store the non-twinned id just for this stupid check.

    // I think what we want is the still the Latest Twin Id, but should think some more about it.  
    fn end_parent<T: NodeType>(&mut self, key: TypedKey<T>) {
        let ended_parent = self.sys.parent_stack.pop();

        #[cfg(debug_assertions)] {
            let ended_parent = ended_parent.expect(&format!("Misplaced end_parent: {}", key.debug_name));
            let ended_parent_id = self.nodes[ended_parent].id;

            let twin_key = self.get_latest_twin_key(key).unwrap();
            debug_assert!(ended_parent_id == twin_key.id(),
            "Misplaced end_parent: tried to end {:?}, but {:?} was the latest parent", self.nodes[ended_parent].debug_name(), twin_key.debug_name
            );
        }

        self.sys.last_child_stack.pop();
    }

    fn v_stack(&mut self) -> NodeRef<Stack> {
        let i = self.update_node(ANON_VSTACK, &V_STACK, true);
        return self.get_ref_unchecked(i, &ANON_VSTACK)
    }

    fn end_v_stack(&mut self) {
        let ended_parent = self.sys.parent_stack.pop();

        #[cfg(debug_assertions)] {
            let ended_parent = ended_parent.expect(&format!("Misplaced end_parent: {}", ANON_VSTACK.debug_name));
            let ended_parent_id = self.nodes[ended_parent].id;

            let twin_key = self.get_latest_twin_key(ANON_VSTACK).unwrap();
            debug_assert!(ended_parent_id == twin_key.id(),
            "Misplaced end_parent: tried to end {:?}, but {:?} was the latest parent", self.nodes[ended_parent].debug_name(), twin_key.debug_name
            );
        }

        self.sys.last_child_stack.pop();
    }

    fn h_stack(&mut self) -> NodeRef<Stack> {
        let i = self.update_node(ANON_HSTACK, &H_STACK, true);
        return self.get_ref_unchecked(i, &ANON_HSTACK)
    }

    fn end_h_stack(&mut self) {
        let ended_parent = self.sys.parent_stack.pop();

        #[cfg(debug_assertions)] {
            let ended_parent = ended_parent.expect(&format!("Misplaced end_parent: {}", ANON_HSTACK.debug_name));
            let ended_parent_id = self.nodes[ended_parent].id;

            let twin_key = self.get_latest_twin_key(ANON_HSTACK).unwrap();
            debug_assert!(ended_parent_id == twin_key.id(),
            "Misplaced end_parent: tried to end {:?}, but {:?} was the latest parent", self.nodes[ended_parent].debug_name(), twin_key.debug_name
            );
        }

        self.sys.last_child_stack.pop();
    }
}