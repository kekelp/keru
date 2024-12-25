use std::{cell::RefCell, hash::{Hash, Hasher}};

use rustc_hash::FxHasher;

use crate::{changes::NodeWithDepth, Id, UiPlacedNode};

pub struct StackParent {
    i: usize,
    old_children_hash: u64,
    children_hash: FxHasher,
}
impl StackParent {
    fn new(i: usize, old_children_hash: u64) -> StackParent {
        return StackParent {
            i,
            old_children_hash,
            children_hash: FxHasher::default(),
        }
    }
}

pub struct Stacks {
    pub parents: Vec<StackParent>,
    pub tree_changes: Vec<NodeWithDepth>,
    pub subtrees: Vec<Id>,
}
impl Stacks {
    pub fn initialize() -> Stacks {
        return Stacks {
            parents: Vec::with_capacity(25),
            subtrees: Vec::with_capacity(10),
            tree_changes: Vec::with_capacity(25),
        };
    }
}

// Global stacks
thread_local! {
    pub static THREAD_STACKS: RefCell<Stacks> = RefCell::new(Stacks::initialize());
}

pub fn push_parent(new_parent: &UiPlacedNode) {
    THREAD_STACKS.with(|stack| {
        let mut stack = stack.borrow_mut();
        stack.parents.push(StackParent::new(new_parent.node_i, new_parent.old_children_hash));       
    });
}

pub fn pop_parent() {
    THREAD_STACKS.with(|stack| {
        let mut stack = stack.borrow_mut();
        
        let parent = stack.parents.pop().unwrap();

        if parent.children_hash.finish() != parent.old_children_hash {
            // we just popped the parent, so its real depth was +1, I think
            let current_depth = stack.parents.len() + 1;

            stack.tree_changes.push(NodeWithDepth {
                i: parent.i,
                depth: current_depth,
            });
        }
    })
}

pub fn hash_new_child(child_i: usize) -> u64 {
    return THREAD_STACKS.with(|stack| {
        let mut stack = stack.borrow_mut();
        let children_hash = &mut stack.parents.last_mut().unwrap().children_hash;
        children_hash.write_usize(child_i);
        // For this hasher, `finish()` just returns the current value. It doesn't actually finish anything. We can continue using it.
        return children_hash.finish()
    });
}

// get the last parent slab i and the current depth ()
pub fn peek_parent() -> NodeWithDepth {
    return THREAD_STACKS.with(
        |stack| {
            let parent_i = stack.borrow().parents.last().unwrap().i;
            let depth = stack.borrow().parents.len();
            return NodeWithDepth{ i: parent_i, depth };
        }
    );
}

// this could be calculated on push/pop instead of every time
pub fn current_tree_hash() -> u64 {
    return THREAD_STACKS.with(
        |stack| {
            let parent_stack = &stack.borrow().parents;

            let mut hasher = FxHasher::default();
        
            for ancestor in parent_stack {
                ancestor.i.hash(&mut hasher); // Write each element into the same hasher
            }
        
            return hasher.finish();
        }
    );
}

pub fn clear_parent_stack() {
    THREAD_STACKS.with(|stack| {
        let mut stack = stack.borrow_mut();
        stack.parents.clear();
    });
}


pub fn push_subtree(subtree_id: Id) {
    THREAD_STACKS.with(|stack| {
        let mut stack = stack.borrow_mut();
        stack.subtrees.push(subtree_id);
    });
}

pub fn pop_subtree() {
    THREAD_STACKS.with(|stack| {
        stack.borrow_mut().subtrees.pop();
    });
}

pub fn last_subtree() -> Option<Id> {
    return THREAD_STACKS.with(|stack| {
        return stack.borrow_mut().subtrees.last().copied();
    });
}
