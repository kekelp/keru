use std::collections::hash_map::Entry;
use std::num::NonZeroU16;
use std::ops::{Index, IndexMut};
use std::panic::Location;

use ahash::AHashMap;
use slab::Slab;

use crate::*;

// The point of this data structure is that from "outside", the nodes can be referenced by stable Ids, but internally, nodes can refer to other nodes by holding a NodeI. A NodeI can be way smaller than both a pointer or an id, and you can use it to access nodes without hashing (as if they held a hashmap key), and without lifetime issues (as if they held references).
#[derive(Debug)]
pub(crate) struct Nodes {
    node_hashmap: AHashMap<Id, NodeI>,
    nodes: Slab<InnerNode>,
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) struct NodeI(NonZeroU16);

impl NodeI {
    const fn from(value: usize) -> Self {
        NodeI(NonZeroU16::new(value as u16).unwrap())
    }

    pub fn as_usize(&self) -> usize {
        self.0.get().into()
    }
}

pub const DUMMY_I: NodeI = NodeI::from(12312355);
pub const ROOT_I: NodeI = NodeI::from(1);

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
    pub(crate) fn get_with_subtree(&self, key: NodeKey) -> Option<NodeI> {
        let id = key.id_with_subtree();
        return self.node_hashmap.get(&id).copied();
    }

    pub(crate) fn get_by_id(&self, id: Id) -> Option<NodeI> {
        return self.node_hashmap.get(&id).copied();
    }

    pub(crate) fn new() -> Self {
        let mut nodes = Slab::with_capacity(100);
        // Insert a dummy node at position zero and never remove it, so that real nodes can be indexed by NonZeroU16
        nodes.insert(ZERO_NODE_DUMMY);
        nodes.insert(NODE_ROOT);

        let mut node_hashmap = AHashMap::with_capacity_and_hasher(100, Default::default());

        node_hashmap.insert(NODE_ROOT_ID, ROOT_I);

        return Nodes {
            node_hashmap,
            nodes,
        };
    }

    pub fn get_node_if_it_still_exists(&self, i: NodeI) -> Option<&InnerNode> {
        self.nodes.get(i.as_usize())
    }

    pub fn iter(&self) -> impl Iterator<Item = NodeI> {
        // Skip the dummy and the root
        self.nodes.iter().skip(2).map(|(i, _node)| NodeI::from(i))
    }

    pub fn remove(&mut self, id: Id) {
        let i = self.node_hashmap.remove(&id).unwrap();
        self.nodes.remove(i.as_usize());
    }
}

impl Ui {
    #[track_caller]
    pub(crate) fn add_or_update_node(&mut self, key: NodeKey) -> (NodeI, Id) {
        let frame = self.sys.current_frame;
        let mut new_node_should_relayout = false;

        // Check the node corresponding to the key's id.
        // We might find that the key has already been used in this same frame:
        //      in this case, we take note, and calculate a twin key to use to add a "twin" in the next section.
        // Otherwise, we add or refresh normally, and take note of the final i.
        let twin_check_result = match self.sys.nodes.node_hashmap.entry(key.id_with_subtree()) {
            // Add a new normal node (no twins).
            Entry::Vacant(v) => {
                let new_node = InnerNode::new(&key, None, Location::caller(), frame);
                let final_i = NodeI::from(self.sys.nodes.nodes.insert(new_node));
                v.insert(final_i);

                new_node_should_relayout = true;

                UpdatedNormal { final_i }
            }
            Entry::Occupied(o) => {
                let old_i = *o.get();
                let last_frame_touched = self.sys.nodes[old_i].last_frame_touched;

                match should_refresh_or_add_twin(frame, last_frame_touched) {
                    // Refresh a normal node from the previous frame (no twins).
                    Refresh => {
                        self.sys.nodes[old_i].last_frame_touched = frame;
                        self.sys.nodes[old_i].n_twins = 0;
                        UpdatedNormal { final_i: old_i }
                    }
                    // do nothing, just calculate the twin key and go to twin part below
                    AddTwin => {
                        self.sys.nodes[old_i].n_twins += 1;
                        let twin_key = key.sibling(self.sys.nodes[old_i].n_twins);

                        NeedToUpdateTwin {
                            twin_key,
                            twin_n: self.sys.nodes[old_i].n_twins,
                        }
                    }
                }
            }
        };

        // If twin_check_result is AddedNormal, the node was added in the section before,
        //      and there's nothing to do regarding twins, so we just confirm final_i.
        // If it's NeedToAddTwin, we repeat the same thing with the new twin_key.
        let (real_final_i, real_final_id) = match twin_check_result {
            UpdatedNormal { final_i } => (final_i, key.id_with_subtree()),
            NeedToUpdateTwin { twin_key, twin_n } => {
                match self.sys.nodes.node_hashmap.entry(twin_key.id_with_subtree()) {
                    // Add new twin.
                    Entry::Vacant(v) => {
                        let new_twin_node = InnerNode::new(&twin_key, Some(twin_n), Location::caller(), frame);
                        let real_final_i = NodeI::from(self.sys.nodes.nodes.insert(new_twin_node));
                        v.insert(real_final_i);
                        new_node_should_relayout = true;
                        (real_final_i, twin_key.id_with_subtree())
                    }
                    // Refresh a twin from the previous frame.
                    Entry::Occupied(o) => {
                        let real_final_i = *o.get();
                        self.sys.nodes.nodes[real_final_i.as_usize()].last_frame_touched = frame;

                        (real_final_i, twin_key.id_with_subtree())
                    }
                }
            }
        };

        // update the in-tree links and the thread-local state based on the current parent.
        let (parent, insert_after, depth) = thread_local::current_parent(self.sys.unique_id);
        self.set_tree_links(real_final_i, parent, depth, insert_after);

        self.sys.nodes[real_final_i].exiting = false;

        self.refresh_node(real_final_i);

        if new_node_should_relayout {
            self.push_partial_relayout(real_final_i);
        }

        return (real_final_i, real_final_id);
    }
}
