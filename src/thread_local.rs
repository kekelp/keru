use std::cell::RefCell;

use crate::*;

pub struct Stacks {
    pub parents: Vec<NodeI>,
    pub subtrees: Vec<Id>,
    pub reactive: i32,
}
impl Stacks {
    pub fn initialize() -> Stacks {
        return Stacks {
            parents: Vec::with_capacity(25),
            subtrees: Vec::with_capacity(10),
            reactive: 0,
        };
    }
}

thread_local! {
    /// Thread local stacks
    pub(crate) static THREAD_STACKS: RefCell<Stacks> = RefCell::new(Stacks::initialize());
}

pub fn push_parent(new_parent: &UiParent) {
    THREAD_STACKS.with(|stack| {
        stack.borrow_mut().parents.push(new_parent.i);       
    });
}

pub fn pop_parent() {
    THREAD_STACKS.with(|stack| {
        stack.borrow_mut().parents.pop().unwrap();
    })
}

// get the last parent slab i and the current depth ()
pub fn current_parent() -> NodeWithDepth {
    return THREAD_STACKS.with(
        |stack| {
            let parent_i = stack.borrow().parents.last().unwrap().clone();
            let depth = stack.borrow().parents.len();
            return NodeWithDepth{ i: parent_i, depth };
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

pub fn push_skip_block() {
    THREAD_STACKS.with(|stack| {
        stack.borrow_mut().reactive += 1;
    });
}

pub fn pop_skip_block() {
    THREAD_STACKS.with(|stack| {
        stack.borrow_mut().reactive -= 1;
    });
}
