use std::num::NonZeroU16;
use std::ops::{Index, IndexMut};

use rustc_hash::FxHashMap;
use slab::Slab;

use crate::*;

#[derive(Debug)]
pub(crate) struct Nodes {
    // todo: make faster or something
    pub(crate) node_hashmap: FxHashMap<Id, NodeMapEntry>,
    pub(crate) nodes: Slab<Node>,
}

/// An index for nodes in the slab.
/// 
/// This has the same guarantees as a `usize` slab key/index: if the corresponding element gets removed, any dangling NodeIs can point to arbitrary other nodes that might have taken its place, or it can just point outside of the slab's current length, in which case it will panic on access.
/// 
/// For this reason, NodeIs should never be held for longer than one frame.
/// 
/// Obviously this can never be pub.
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
    type Output = Node;
    fn index(&self, i: NodeI) -> &Self::Output {
        return &self.nodes[i.as_usize()];
    }
}

impl IndexMut<NodeI> for Nodes {
    fn index_mut(&mut self, i: NodeI) -> &mut Self::Output {
        return &mut self.nodes[i.as_usize()];
    }
}

impl Nodes {
    // todo: doesn't this kind of suck?
    pub(crate) fn get_by_id(&mut self, id: &Id) -> Option<(&mut Node, NodeI)> {
        let i = self.node_hashmap.get(&id)?;
        return Some((&mut self.nodes[i.slab_i.as_usize()], i.slab_i));
    }

    pub(crate) fn new() -> Self {        
        let mut nodes = Slab::with_capacity(100);
        // Insert a dummy node at position zero and never remove it, so that real nodes can be indexed by NonZeroU16
        nodes.insert(ZERO_NODE_DUMMY);
        nodes.insert(NODE_ROOT);

        let root_map_entry = NodeMapEntry {
            last_frame_touched: u64::MAX,
            slab_i: ROOT_I,
            n_twins: 0,
        };

        let mut node_hashmap = FxHashMap::with_capacity_and_hasher(100, Default::default());
        
        node_hashmap.insert(NODE_ROOT_ID, root_map_entry);

        return Nodes {
            node_hashmap,
            nodes,
        };
    }

    // todo: actually call this once in a while
    pub(crate) fn prune(&mut self, current_frame: u64) {
        // remember to not delete the zero dummy node
        self.node_hashmap.retain(|_k, v| {
            // the > is to always keep the root node without having to refresh it
            let should_retain = v.last_frame_touched >= current_frame;
            if !should_retain {
                let i: usize = v.slab_i.as_usize();
                // let name = self.format_node_debug_name(i);
                // side effect happens inside this closure? idk if this even works
                self.nodes.remove(i);
                // remember to remove text areas and such ...
                // log::info!("pruning node {:?}", name);
            }
            should_retain
        });
    }
}