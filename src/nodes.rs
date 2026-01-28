use std::num::NonZeroU16;
use std::ops::{Index, IndexMut};

use ahash::AHashMap;
use slab::Slab;

use crate::*;

#[derive(Debug)]
// The point of this weird data structure is that from "outside", the nodes can be referenced by stable Ids, but internally, nodes can refer to other nodes by holding a NodeI. A NodeI can be way smaller than both a pointer or an id, and you can use it to access nodes without hashing (as if they held a hashmap key), and without lifetime issues (as if they held references).
pub(crate) struct Nodes {
    // todo: make faster or something
    pub(crate) node_hashmap: AHashMap<Id, NodeMapEntry>,
    pub(crate) nodes: Slab<InnerNode>,
}

/// An index for nodes in the slab.
/// 
/// This has the same guarantees as a `usize` slab key/index: if the corresponding element gets removed, any dangling NodeIs can point to arbitrary other nodes that might have taken its place, or it can just point outside of the slab's current length, in which case it will panic on access.
/// 
/// For this reason, NodeIs should never be held for longer than one frame.
/// 
/// This is mostly automatic given the declarative structure, but for example things like Hovered or Focused have to hold an Id and not a NodeI for this reason.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct NodeI(NonZeroU16);

impl NodeI {
    pub const fn from(value: usize) -> Self {
        NodeI(NonZeroU16::new(value as u16).unwrap())
    }

    pub fn as_usize(&self) -> usize {
        self.0.get().into()
    }
}

impl Index<NodeI> for Nodes {
    type Output = InnerNode;
    fn index(&self, i: NodeI) -> &Self::Output {
        return &self.nodes[i.as_usize()];
        // unsafe {
        //     return &self.nodes.get_unchecked(i.as_usize());
        // }
    }
}

impl IndexMut<NodeI> for Nodes {
    fn index_mut(&mut self, i: NodeI) -> &mut Self::Output {
        return &mut self.nodes[i.as_usize()];
    }
}

impl Nodes {
    // todo: doesn't this kind of suck?
    pub(crate) fn get_mut_by_id(&mut self, id: &Id) -> Option<(&mut InnerNode, NodeI)> {
        let i = self.node_hashmap.get(id)?;
        return Some((&mut self.nodes[i.slab_i.as_usize()], i.slab_i));
    }

    #[allow(dead_code)]
    pub(crate) fn get_by_id_ick(&self, id: &Id) -> Option<(&InnerNode, NodeI)> {
        let i = self.node_hashmap.get(id)?;
        return Some((&self.nodes[i.slab_i.as_usize()], i.slab_i));
    }

    pub(crate) fn get_by_id(&mut self, id: Option<Id>) -> Option<NodeI> {
        let id = id?;
        let i = self.node_hashmap.get(&id)?;
        return Some(i.slab_i);
    }

    pub(crate) fn new() -> Self {        
        let mut nodes = Slab::with_capacity(100);
        // Insert a dummy node at position zero and never remove it, so that real nodes can be indexed by NonZeroU16
        nodes.insert(ZERO_NODE_DUMMY);
        nodes.insert(NODE_ROOT);

        let root_map_entry = NodeMapEntry {
            slab_i: ROOT_I,
            n_twins: 0,
        };

        let mut node_hashmap = AHashMap::with_capacity_and_hasher(100, Default::default());
        
        node_hashmap.insert(NODE_ROOT_ID, root_map_entry);

        return Nodes {
            node_hashmap,
            nodes,
        };
    }

    pub fn get(&self, i: NodeI) -> Option<&InnerNode> {
        self.nodes.get(i.as_usize())
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct NodeMapEntry {
    // keeping track of the twin situation.
    // This is the number of twins of a node that showed up SO FAR in the current frame. it gets reset every frame (on refresh().)
    // for the 0-th twin of a family, this will be the total number of clones of itself around. (not including itself, so starts at zero).
    // the actual twins ARE twins, but they don't HAVE twins, so this is zero.
    // for this reason, "clones" or "copies" would be better names, but those words are loaded in rust
    // reproduction? replica? imitation? duplicate? version? dupe? replication? mock? carbon?
    pub n_twins: u32,
    pub slab_i: NodeI,
}
impl NodeMapEntry {
    pub fn new(new_i: NodeI) -> Self {
        return Self {
            n_twins: 0,
            slab_i: new_i,
        };
    }

    pub fn refresh(&mut self) -> NodeI {
        self.n_twins = 0;
        return self.slab_i;
    }
}