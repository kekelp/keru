use crate::{NodeParams, NodeRef, NodeType, TypedKey, Ui};

pub trait AddParentManual {
    fn add_parent<T: NodeType>(&mut self, key: TypedKey<T>, params: &NodeParams) -> NodeRef<T>;
}
impl AddParentManual for Ui {
    fn add_parent<T: NodeType>(&mut self, key: TypedKey<T>, params: &NodeParams) -> NodeRef<T> {
        let i = self.update_node(key, params, true);
        return self.get_ref_unchecked(i, &key)
    }
}