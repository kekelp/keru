use crate::{node_params::{ANON_HSTACK, ANON_VSTACK, H_STACK, V_STACK}, Any, UiNode, Ui};

pub trait AddParentClosure {
    fn v_stack(&mut self, content_code: impl FnOnce(&mut Self)) -> UiNode<Any>;
    fn h_stack(&mut self, content_code: impl FnOnce(&mut Self)) -> UiNode<Any>;
}

impl AddParentClosure for Ui {
    fn v_stack(&mut self, content_code: impl FnOnce(&mut Self)) -> UiNode<Any> {
        let i = self.update_node(ANON_VSTACK, &V_STACK, true, false);

        content_code(self);

        self.end_parent_unchecked();

        return self.get_ref_unchecked(i, &ANON_VSTACK)
    }

    fn h_stack(&mut self, content_code: impl FnOnce(&mut Self)) -> UiNode<Any> {
        let i = self.update_node(ANON_HSTACK, &H_STACK, true, false);

        content_code(self);

        self.end_parent_unchecked();

        return self.get_ref_unchecked(i, &ANON_HSTACK)
    }
}