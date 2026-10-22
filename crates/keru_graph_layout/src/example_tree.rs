use crate::*;

/// The index of a node in an `ExampleTree`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeI(u32);

pub const ROOT_I: NodeI = NodeI(0);

#[derive(Debug, Clone, Copy)]
struct Node {
    parent: NodeI,
    first_child: Option<NodeI>,
    last_child: Option<NodeI>,
    next_sibling: Option<NodeI>,
    params: LayoutParams,
    layout: NodeLayout,
    /// The output rect, in fractions of the window size.
    rect: XyRect,
}

impl Node {
    fn new(parent: NodeI, params: LayoutParams) -> Self {
        Node {
            parent,
            first_child: None,
            last_child: None,
            next_sibling: None,
            params,
            layout: NodeLayout::EMPTY,
            rect: Xy::new([0.0, 0.0], [0.0, 0.0]),
        }
    }
}

/// A simple standalone tree that uses the layout.
#[derive(Debug)]
pub struct ExampleTree {
    nodes: Vec<Node>,
    window_size: Xy<f32>,
}

impl ExampleTree {
    /// Makes a tree that contains only the root. `window_size` is in logical pixels.
    pub fn new(window_size: Xy<f32>) -> Self {
        ExampleTree { nodes: vec![Node::new(ROOT_I, LayoutParams::DEFAULT)], window_size }
    }

    #[inline(always)]
    fn node(&self, i: NodeI) -> &Node {
        &self.nodes[i.0 as usize]
    }

    #[inline(always)]
    fn node_mut(&mut self, i: NodeI) -> &mut Node {
        &mut self.nodes[i.0 as usize]
    }

    /// Adds a node as the last child of `parent`.
    pub fn add(&mut self, parent: NodeI, params: LayoutParams) -> NodeI {
        let i = NodeI(self.nodes.len() as u32);
        self.nodes.push(Node::new(parent, params));
        match self.node(parent).last_child {
            Some(last) => self.node_mut(last).next_sibling = Some(i),
            None => self.node_mut(parent).first_child = Some(i),
        }
        self.node_mut(parent).last_child = Some(i);
        i
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
        self.node(i).parent
    }

    #[inline(always)]
    fn first_child(&self, i: NodeI) -> Option<NodeI> {
        self.node(i).first_child
    }

    #[inline(always)]
    fn next_sibling(&self, i: NodeI) -> Option<NodeI> {
        self.node(i).next_sibling
    }

    #[inline(always)]
    fn params(&self, i: NodeI) -> &LayoutParams {
        &self.node(i).params
    }

    #[inline(always)]
    fn layout_data(&self, i: NodeI) -> &NodeLayout {
        &self.node(i).layout
    }

    #[inline(always)]
    fn layout_data_mut(&mut self, i: NodeI) -> &mut NodeLayout {
        &mut self.node_mut(i).layout
    }

    #[inline(always)]
    fn measure_text(&mut self, _i: NodeI, _width: Option<f32>) -> Xy<f32> {
        Xy::new(0.0, 0.0)
    }

    #[inline(always)]
    fn window_size(&self) -> Xy<f32> {
        self.window_size
    }

    #[inline(always)]
    fn write_output_rect(&mut self, i: NodeI, rect: XyRect, _local_rect: XyRect) {
        self.node_mut(i).rect = rect;
    }
}
