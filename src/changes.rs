use crate::thread_local::THREAD_STACKS;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct NodeWithDepth {
    pub i: usize,
    pub depth: usize,
}

impl Ord for NodeWithDepth {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.depth.cmp(&other.depth)
    }
}

impl PartialOrd for NodeWithDepth {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug)]
pub struct PartialChanges {
    pub cosmetic_rect_updates: Vec<usize>,
    pub partial_relayouts: Vec<NodeWithDepth>,
    pub swapped_tree_changes: Vec<NodeWithDepth>,
    pub rebuild_all_rects: bool,
    pub full_relayout: bool,

    pub need_rerender: bool,

    pub resize: bool,
}
impl PartialChanges {
    pub fn new() -> PartialChanges {
        return PartialChanges { 
            partial_relayouts: Vec::with_capacity(15),
            cosmetic_rect_updates: Vec::with_capacity(15),
            swapped_tree_changes: Vec::with_capacity(15),
            rebuild_all_rects: false,
            full_relayout: true,

            need_rerender: true,

            resize: false,
        }
    }

    pub fn reset_layout_changes(&mut self) {
        self.partial_relayouts.clear();
        self.cosmetic_rect_updates.clear();
        self.full_relayout = false;
        self.rebuild_all_rects = false;

        // ... and the thread local stuff gets automatically reset by take_thread_local_tree_changes
    }

    pub fn swap_thread_local_tree_changes(&mut self) {
        THREAD_STACKS.with(|stack| {
            let mut stack = stack.borrow_mut();
            
            // mem::swap the tree changes out of the thread_local into a normal vec.
            std::mem::swap(&mut self.swapped_tree_changes, &mut stack.tree_changes);

            stack.tree_changes.clear();

            // after this, the tree changes are stored in `swapped_tree_changes`, until they are swapped again.
        })
    }
}
