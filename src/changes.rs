use crate::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct NodeWithDepth {
    pub i: NodeI,
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
    pub cosmetic_rect_updates: Vec<NodeI>,
    pub partial_relayouts: Vec<NodeWithDepth>,
    pub tree_changed: bool,
    pub full_relayout: bool,

    pub need_gpu_rect_update: bool,

    pub need_rerender: bool,

    pub resize: bool,
}
impl PartialChanges {
    pub fn new() -> PartialChanges {
        return PartialChanges { 
            partial_relayouts: Vec::with_capacity(15),
            cosmetic_rect_updates: Vec::with_capacity(15),
            tree_changed: false,
            full_relayout: true,

            need_rerender: true,
            need_gpu_rect_update: true,

            resize: false,
        }
    }

    pub fn reset_layout_changes(&mut self) {
        self.partial_relayouts.clear();
        self.cosmetic_rect_updates.clear();
        self.full_relayout = false;
        self.tree_changed = false;
    }
}
