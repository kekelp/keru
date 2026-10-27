use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeI(pub u32);

pub const ROOT_I: NodeI = NodeI(0);

#[derive(Debug, Clone, Copy)]
pub struct Node {
    pub parent: NodeI,
    pub first_child: Option<NodeI>,
    pub last_child: Option<NodeI>,
    pub next_sibling: Option<NodeI>,
    pub params: LayoutParams,
    pub layout: NodeLayoutData,
    pub output_rect: XyRect,
}

impl Node {
    pub fn new(parent: NodeI, params: LayoutParams) -> Self {
        Node {
            parent,
            first_child: None,
            last_child: None,
            next_sibling: None,
            params,
            layout: NodeLayoutData::EMPTY,
            output_rect: Xy::new([0.0, 0.0], [0.0, 0.0]),
        }
    }
}

/// A simple standalone tree that uses the layout.
#[derive(Debug)]
pub struct ExampleTree {
    pub nodes: Vec<Node>,
    pub window_size: Xy<f32>,
    pub warnings: Vec<(NodeI, LayoutWarning)>,
}

impl ExampleTree {
    /// Makes a tree that contains only the root, with space for `capacity` nodes. `window_size` is in logical pixels.
    pub fn with_capacity(capacity: usize, window_size: Xy<f32>) -> Self {
        let mut nodes = Vec::with_capacity(capacity);
        nodes.push(Node::new(ROOT_I, LayoutParams::DEFAULT));
        ExampleTree { nodes, window_size, warnings: Vec::new() }
    }

    /// Adds a node as the last child of `parent`.
    pub fn add(&mut self, parent: NodeI, params: LayoutParams) -> NodeI {
        let i = NodeI(self.nodes.len() as u32);
        self.nodes.push(Node::new(parent, params));
        match self[parent].last_child {
            Some(last) => self[last].next_sibling = Some(i),
            None => self[parent].first_child = Some(i),
        }
        self[parent].last_child = Some(i);
        i
    }
}

impl std::ops::Index<NodeI> for ExampleTree {
    type Output = Node;
    #[inline(always)]
    fn index(&self, i: NodeI) -> &Node {
        &self.nodes[i.0 as usize]
    }
}

impl std::ops::IndexMut<NodeI> for ExampleTree {
    #[inline(always)]
    fn index_mut(&mut self, i: NodeI) -> &mut Node {
        &mut self.nodes[i.0 as usize]
    }
}

impl TreeAdapter for ExampleTree {
    type NodeId = NodeI;

    #[inline(always)]
    fn root(&self) -> NodeI {
        ROOT_I
    }

    #[inline(always)]
    fn parent(&self, i: NodeI) -> NodeI {
        self[i].parent
    }

    #[inline(always)]
    fn first_child(&self, i: NodeI) -> Option<NodeI> {
        self[i].first_child
    }

    #[inline(always)]
    fn next_sibling(&self, i: NodeI) -> Option<NodeI> {
        self[i].next_sibling
    }

    #[inline(always)]
    fn node_layout_params(&self, i: NodeI) -> &LayoutParams {
        &self[i].params
    }

    #[inline(always)]
    fn node_layout_data(&self, i: NodeI) -> &NodeLayoutData {
        &self[i].layout
    }

    #[inline(always)]
    fn node_layout_data_mut(&mut self, i: NodeI) -> &mut NodeLayoutData {
        &mut self[i].layout
    }

    #[inline(always)]
    fn measure_text_width(&mut self, _i: NodeI) -> f32 {
        0.0
    }

    #[inline(always)]
    fn measure_text_height(&mut self, _i: NodeI, _wrap_width: Option<f32>) -> f32 {
        0.0
    }

    #[inline(always)]
    fn window_size(&self) -> Xy<f32> {
        self.window_size
    }

    #[inline(always)]
    fn write_output_rect(&mut self, i: NodeI, rect: XyRect, _local_rect: XyRect) {
        self[i].output_rect = rect;
    }

    fn layout_warning(&mut self, i: NodeI, warning: LayoutWarning) {
        self.warnings.push((i, warning));
    }
}
