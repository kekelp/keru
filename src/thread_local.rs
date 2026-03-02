use std::cell::RefCell;

use crate::*;

#[derive(Clone, Copy, Debug)]
pub(crate) enum SiblingCursor {
    /// No cursor - append after last child (normal behavior).
    None,
    /// Insert at the beginning, before first child.
    AtStart,
    /// Insert after a specific sibling.
    After(NodeI),
}

#[derive(Clone, Copy)]
pub(crate) struct ParentCtx {
    /// add()ing new children places them as children of this parent.
    pub parent: NodeI,
    /// Normally this is Append and new children are added after the last child of the current parent automatically.
    /// When using [`Ui::jump_to_sibling()`], this is After, and new children are added after the sibling_cursor node.
    /// Then [`Ui::set_tree_links()`] advances the sibling_cursor manually.
    pub sibling_cursor: SiblingCursor,
    /// To allow for multiple Uis to be nested at the same time. Nobody should ever want to do this.
    pub ui_instance_id: u32,
}

pub struct Stacks {
    pub parents: Vec<ParentCtx>,
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
    pub(crate) static THREAD_STACKS: RefCell<Stacks> = RefCell::new(Stacks::initialize());
}

pub(crate) fn push_parent(parent: NodeI, sibling_cursor: SiblingCursor, ui_instance_id: u32) {
    THREAD_STACKS.with(|stack| {
        stack.borrow_mut().parents.push(ParentCtx { parent, sibling_cursor, ui_instance_id });
    });
}

pub(crate) fn pop_parent(ui_instance_id: u32) {
    THREAD_STACKS.with(|stack| {
        let mut stack = stack.borrow_mut();

        let last = stack.parents.iter().rposition(|ctx| ctx.ui_instance_id == ui_instance_id);
        if let Some(pos) = last {
            stack.parents.remove(pos);
        } else {
            unreachable!();
        }
    });
}

pub(crate) fn current_parent(ui_instance_id: u32) -> (NodeI, SiblingCursor, usize) {
    THREAD_STACKS.with(|stack| {
        let stack = stack.borrow();

        let parent_ctx = stack
            .parents
            .iter()
            .rfind(|ctx| ctx.ui_instance_id == ui_instance_id)
            .expect("No parent_ctx found for current ui_instance_id");

        (parent_ctx.parent, parent_ctx.sibling_cursor, stack.parents.len())
    })
}

pub(crate) fn set_sibling_cursor(cursor: SiblingCursor) {
    THREAD_STACKS.with(|stack| {
        if let Some(last) = stack.borrow_mut().parents.last_mut() {
            last.sibling_cursor = cursor;
        }
    });
}

pub fn clear_parent_stack() {
    THREAD_STACKS.with(|stack| {
        stack.borrow_mut().parents.clear();
    });
}


pub fn push_subtree(subtree_id: Id) {
    THREAD_STACKS.with(|stack| {
        stack.borrow_mut().subtrees.push(subtree_id);
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
