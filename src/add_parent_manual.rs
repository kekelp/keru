use std::marker::PhantomData;

use crate::{NodeParams, NodeRef, NodeType, TypedKey, Ui};

pub trait AddParentManual {
    fn add_parent<T: NodeType>(&mut self, key: TypedKey<T>, defaults: &NodeParams) -> NodeRef<T>;
}
impl AddParentManual for Ui {
    fn add_parent<T: NodeType>(&mut self, key: TypedKey<T>, defaults: &NodeParams) -> NodeRef<T> {
        let i = self.update_node(key, defaults, true);
        return NodeRef {
            node: &mut self.nodes[i],
            text: &mut self.text,
            nodetype_marker: PhantomData::<T>,
        };
    }
}