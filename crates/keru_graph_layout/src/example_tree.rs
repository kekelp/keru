use std::ops::{Index, IndexMut};

use crate::*;

/// The index of a node in an `ExampleTree`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeI(u32);

impl NodeI {
    #[inline(always)]
    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

pub const ROOT_I: NodeI = NodeI(0);

/// A `Vec` indexed by `NodeI`.
#[derive(Debug)]
struct NodeVec<T>(Vec<T>);

impl<T> Index<NodeI> for NodeVec<T> {
    type Output = T;
    #[inline(always)]
    fn index(&self, i: NodeI) -> &T {
        &self.0[i.as_usize()]
    }
}

impl<T> IndexMut<NodeI> for NodeVec<T> {
    #[inline(always)]
    fn index_mut(&mut self, i: NodeI) -> &mut T {
        &mut self.0[i.as_usize()]
    }
}

#[derive(Debug, Clone, Copy)]
struct Links {
    parent: NodeI,
    first_child: Option<NodeI>,
    last_child: Option<NodeI>,
    next_sibling: Option<NodeI>,
}

/// A simple standalone tree that uses the layout. Each node keeps its data in `Vec`s, at the position of its `NodeI`.
#[derive(Debug)]
pub struct ExampleTree {
    links: NodeVec<Links>,
    params: NodeVec<LayoutParams>,
    layouts: NodeVec<NodeLayout>,
    /// The output rects, in pixels.
    rects: NodeVec<XyRect>,
    n_deferred_solves: usize,
}

impl ExampleTree {
    /// Makes a tree that contains only the root.
    pub fn new() -> Self {
        let mut tree = ExampleTree {
            links: NodeVec(Vec::new()),
            params: NodeVec(Vec::new()),
            layouts: NodeVec(Vec::new()),
            rects: NodeVec(Vec::new()),
            n_deferred_solves: 0,
        };
        tree.push(ROOT_I, LayoutParams::DEFAULT);
        tree
    }

    fn push(&mut self, parent: NodeI, params: LayoutParams) -> NodeI {
        let i = NodeI(self.links.0.len() as u32);
        self.links.0.push(Links { parent, first_child: None, last_child: None, next_sibling: None });
        self.params.0.push(params);
        self.layouts.0.push(NodeLayout::EMPTY);
        self.rects.0.push(Xy::new([0.0, 0.0], [0.0, 0.0]));
        i
    }

    /// Adds a node as the last child of `parent`.
    pub fn add(&mut self, parent: NodeI, params: LayoutParams) -> NodeI {
        let i = self.push(parent, params);
        match self.links[parent].last_child {
            Some(last) => self.links[last].next_sibling = Some(i),
            None => self.links[parent].first_child = Some(i),
        }
        self.links[parent].last_child = Some(i);
        i
    }

    /// The number of nodes, including the root.
    pub fn len(&self) -> usize {
        self.links.0.len()
    }

    /// Solves the sizes and places all the nodes.
    pub fn compute(&mut self, window_size: Xy<f32>) {
        let mut adapter = Adapter { tree: self, window_size };
        self.n_deferred_solves = compute(&mut adapter, window_size);
    }

    #[doc(hidden)]
    pub fn deferred_solves(&self) -> usize {
        self.n_deferred_solves
    }

    /// The rect of a node after `compute`, in pixels.
    pub fn rect_px(&self, i: NodeI) -> XyRect {
        self.rects[i]
    }
}

impl Default for ExampleTree {
    fn default() -> Self {
        Self::new()
    }
}

/// An `ExampleTree` during a `compute` call, with the values that only the call has.
struct Adapter<'a> {
    tree: &'a mut ExampleTree,
    window_size: Xy<f32>,
}

impl TreeAdapter for Adapter<'_> {
    type NodeId = NodeI;

    #[inline(always)]
    fn root(&self) -> NodeI {
        ROOT_I
    }

    #[inline(always)]
    fn parent(&self, i: NodeI) -> NodeI {
        self.tree.links[i].parent
    }

    #[inline(always)]
    fn first_child(&self, i: NodeI) -> Option<NodeI> {
        self.tree.links[i].first_child
    }

    #[inline(always)]
    fn next_sibling(&self, i: NodeI) -> Option<NodeI> {
        self.tree.links[i].next_sibling
    }

    #[inline(always)]
    fn params(&self, i: NodeI) -> &LayoutParams {
        &self.tree.params[i]
    }

    #[inline(always)]
    fn layout_data(&self, i: NodeI) -> &NodeLayout {
        &self.tree.layouts[i]
    }

    #[inline(always)]
    fn layout_data_mut(&mut self, i: NodeI) -> &mut NodeLayout {
        &mut self.tree.layouts[i]
    }

    #[inline(always)]
    fn measure_text(&mut self, _i: NodeI, _width: Option<f32>) -> Xy<f32> {
        Xy::new(0.0, 0.0)
    }

    #[inline(always)]
    fn write_output_rect(&mut self, i: NodeI, rect: XyRect, _local_rect: XyRect) {
        let window = self.window_size;
        self.tree.rects[i] = Xy::new(
            [rect.x[0] * window.x, rect.x[1] * window.x],
            [rect.y[0] * window.y, rect.y[1] * window.y],
        );
    }
}
