use crate::{node_params::{ANON_HSTACK, ANON_NODE, ANON_VSTACK, H_STACK, V_STACK}, Any, NodeParams, NodeRef, NodeType, Stack, TypedKey, Ui};

pub trait AddParentClosure {
    fn add_parent<T: NodeType>(&mut self, key: TypedKey<T>, params: &NodeParams, content_code: impl FnOnce(&mut Self)) -> NodeRef<T>;
    fn add_anon_parent(&mut self, params: &NodeParams, content_code: impl FnOnce(&mut Self)) -> NodeRef<Any>;
    fn v_stack(&mut self, content_code: impl FnOnce(&mut Self)) -> NodeRef<Stack>;
    fn h_stack(&mut self, content_code: impl FnOnce(&mut Self)) -> NodeRef<Stack>;
}

impl AddParentClosure for Ui {
    fn add_parent<T: NodeType>(&mut self, key: TypedKey<T>, params: &NodeParams, content_code: impl FnOnce(&mut Self)) -> NodeRef<T> {
        let i = self.update_node(key, params, true);

        content_code(self);

        self.end_parent_unchecked();

        return self.get_ref_unchecked(i, &key)
    }

    fn add_anon_parent(&mut self, params: &NodeParams, content_code: impl FnOnce(&mut Self)) -> NodeRef<Any> {
        let i = self.update_node(ANON_NODE, params, true);

        content_code(self);

        self.end_parent_unchecked();

        return self.get_ref_unchecked(i, &ANON_NODE)
    }

    fn v_stack(&mut self, content_code: impl FnOnce(&mut Self)) -> NodeRef<Stack> {
        let i = self.update_node(ANON_VSTACK, &V_STACK, true);

        content_code(self);

        self.end_parent_unchecked();

        return self.get_ref_unchecked(i, &ANON_VSTACK)
    }

    fn h_stack(&mut self, content_code: impl FnOnce(&mut Self)) -> NodeRef<Stack> {
        let i = self.update_node(ANON_HSTACK, &H_STACK, true);

        content_code(self);

        self.end_parent_unchecked();

        return self.get_ref_unchecked(i, &ANON_HSTACK)
    }
}