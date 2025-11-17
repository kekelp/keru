use crate::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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
pub struct Changes {
    pub rebuild_render_data: bool,
    pub partial_relayouts: Vec<NodeWithDepth>,
    // todo: bitflags, or just less bools
    pub full_relayout: bool,
    pub text_changed: bool,
    pub unfinished_animations: bool,


    pub need_gpu_rect_update: bool,

    pub need_rerender: bool,
    pub should_rebuild_render_data: bool,

    pub resize: bool,
}
impl Changes {
    pub fn new() -> Changes {
        return Changes {
            partial_relayouts: Vec::with_capacity(15),
            rebuild_render_data: false,
            text_changed: false,
            full_relayout: true,
            unfinished_animations: false,

            should_rebuild_render_data: true,
            need_rerender: true,
            need_gpu_rect_update: true,

            resize: false,
        }
    }

    pub fn reset_layout_changes(&mut self) {
        self.partial_relayouts.clear();
        self.rebuild_render_data = false;
        self.full_relayout = false;
    }
}
