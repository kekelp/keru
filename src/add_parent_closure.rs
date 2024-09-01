use std::marker::PhantomData;

use crate::{NodeParams, NodeRef, NodeType, TypedKey, Ui};

pub trait AddParentClosure {
    fn add_parent<T: NodeType>(&mut self, key: TypedKey<T>, defaults: &NodeParams, content_code: impl FnOnce(&mut Self)) -> NodeRef<T>;
}
impl AddParentClosure for Ui {
    fn add_parent<T: NodeType>(&mut self, key: TypedKey<T>, defaults: &NodeParams, content_code: impl FnOnce(&mut Self)) -> NodeRef<T> {
        let i = self.update_node(key, defaults, true);

        content_code(self);

        self.end_parent_unchecked();

        return NodeRef {
            node: &mut self.nodes[i],
            text: &mut self.sys.text,
            nodetype_marker: PhantomData::<T>,
        };
    }
}