use std::{cell::RefCell, hash::Hasher};

use rustc_hash::FxHasher;

use crate::{changes::NodeWithDepth, Parent, EMPTY_HASH};

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

// now there's a single stack here. but now that I wrote the struct I might as well leave it.
pub struct Stacks {
    pub parents: Vec<StackParent>,
    pub tree_changes: Vec<NodeWithDepth>,
}
impl Stacks {
    pub fn initialize() -> Stacks {
        return Stacks {
            parents: Vec::with_capacity(25),
            tree_changes: Vec::with_capacity(25),
        };
    }
}

// Global stacks
thread_local! {
    pub static THREAD_STACKS: RefCell<Stacks> = RefCell::new(Stacks::initialize());
}

pub fn thread_local_push(new_parent: &Parent) {
    THREAD_STACKS.with(|stack| {
        let mut stack = stack.borrow_mut();
        stack.parents.push(StackParent::new(new_parent.node_i, new_parent.old_children_hash));       
    });
}

pub fn thread_local_pop() {
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

pub fn thread_local_hash_new_child(child_i: usize) -> u64 {
    THREAD_STACKS.with(|stack| {
        let mut stack = stack.borrow_mut();
        let children_hash = &mut stack.parents.last_mut().unwrap().children_hash;
        children_hash.write_usize(child_i);
        // For this hasher, `finish()` just returns the current value. It doesn't actually finish anything. We can continue using it.
        return children_hash.finish()
    })
}

// get the last parent slab i and the current depth ()
pub fn thread_local_peek_parent() -> NodeWithDepth {
    THREAD_STACKS.with(
        |stack| {
            let parent_i = stack.borrow().parents.last().unwrap().i;
            let depth = stack.borrow().parents.len();
            return NodeWithDepth{ i: parent_i, depth };
        }
    )
}

pub fn clear_thread_local_parent_stack() {
    THREAD_STACKS.with(|stack| {
        let mut stack = stack.borrow_mut();
        stack.parents.clear();
        // todo: this should be `root_i`, but whatever
        stack.parents.push(StackParent::new(0, EMPTY_HASH));
    })
}
