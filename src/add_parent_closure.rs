use crate::{NodeParams, NodeRef, NodeType, TypedKey, Ui};

pub trait AddParentClosure {
    fn add_parent<T: NodeType>(&mut self, key: TypedKey<T>, params: &NodeParams, content_code: impl FnOnce(&mut Self)) -> NodeRef<T>;
}
impl AddParentClosure for Ui {
    fn add_parent<T: NodeType>(&mut self, key: TypedKey<T>, params: &NodeParams, content_code: impl FnOnce(&mut Self)) -> NodeRef<T> {
        let i = self.update_node(key, params, true);

        content_code(self);

        self.end_parent_unchecked();

        return self.get_ref_unchecked(i, &key)
    }
}