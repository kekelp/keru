#[macro_use]
mod solve;
mod place;

use std::cell::RefCell;
use std::fmt::Debug;
use std::ops::{Index, IndexMut};

pub use solve::{LayoutParams, NodeLayout, TextContent, compute, place_free_placed_subtree, solve};
pub(crate) use solve::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Axis {
    X,
    Y,
}

impl Axis {
    pub fn other(&self) -> Self {
        match self {
            Axis::X => Axis::Y,
            Axis::Y => Axis::X,
        }
    }
}

pub(crate) use Axis::{X, Y};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Xy<T> {
    pub x: T,
    pub y: T,
}

impl<T> Xy<T> {
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T: Copy> Xy<T> {
    pub const fn new_symm(v: T) -> Self {
        Self { x: v, y: v }
    }
}

impl<T> Index<Axis> for Xy<T> {
    type Output = T;
    #[inline(always)]
    fn index(&self, axis: Axis) -> &T {
        match axis {
            Axis::X => &self.x,
            Axis::Y => &self.y,
        }
    }
}

impl<T> IndexMut<Axis> for Xy<T> {
    #[inline(always)]
    fn index_mut(&mut self, axis: Axis) -> &mut T {
        match axis {
            Axis::X => &mut self.x,
            Axis::Y => &mut self.y,
        }
    }
}

pub type XyRect = Xy<[f32; 2]>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Size {
    Pixels(f32),
    Frac(f32),
    Fill,
    FitContent,
    AspectRatio(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Anchor {
    Start,
    Center,
    End,
    Frac(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pos {
    Center,
    Start,
    End,
    Pixels(f32),
    Frac(f32),
}

/// What to do if a freely placed node would extend past the window edge. It has no effect on the children of a `Stack` or a `Grid`, unless they are `free_placement`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WindowOverflow {
    /// Leave the node where it is, even if it extends past the window edge.
    #[default]
    Ignore,
    /// Push the node back inside the window.
    Clamp,
    /// Mirror the node to the other side of its anchor point, as a right-click context menu does. If the mirrored node also doesn't fit, push it back inside the window.
    Flip,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub enum Arrange {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum Direction {
    LeftToRight,
    RightToLeft,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridFlow {
    pub main_axis: Axis,
    pub x_fill_direction: Direction,
    pub y_fill_direction: Direction,
    pub backfill: bool,
}

impl GridFlow {
    pub const DEFAULT: Self = Self { main_axis: Axis::X, x_fill_direction: Direction::LeftToRight, y_fill_direction: Direction::LeftToRight, backfill: false };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MainAxisCellSize {
    Count(u32),
    Width(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChildrenLayout {
    Free,
    Stack {
        arrange: Arrange,
        axis: Axis,
        spacing: f32,
    },
    Grid {
        columns: MainAxisCellSize,
        spacing_x: f32,
        spacing_y: f32,
        flow: GridFlow,
    },
}

/// The tree that the layout runs on. The tree keeps a `NodeLayout` for each node, and the layout reads and writes it through `layout_data` and `layout_data_mut`. The storage can be of any form: for example, a `Vec`, a `SecondaryMap`, or a field in the node.
pub trait TreeAdapter {
    /// The handle of a node. It must stay valid during a `solve` or `compute` call.
    type Node: Copy + Eq + Debug;
    fn root(&self) -> Self::Node;
    /// The parent of a node. It is never called on the root.
    fn parent(&self, i: Self::Node) -> Self::Node;
    fn first_child(&self, i: Self::Node) -> Option<Self::Node>;
    fn next_sibling(&self, i: Self::Node) -> Option<Self::Node>;
    fn params(&self, i: Self::Node) -> &LayoutParams;
    fn layout_data(&self, i: Self::Node) -> &NodeLayout;
    fn layout_data_mut(&mut self, i: Self::Node) -> &mut NodeLayout;
    /// Measures the text of a node. The width and the result are in fractions of the window size. It is called only on nodes with text.
    fn measure_text(&mut self, i: Self::Node, width: Option<f32>) -> Xy<f32>;
    /// Receives the final rect of a node, in fractions of the window size. `compute` calls it once on each placed node, on the parent before its children. `solve` never calls it.
    fn write_output_rect(&mut self, i: Self::Node, rect: XyRect);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeI(u32);

impl NodeI {
    #[inline(always)]
    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }

    pub fn from_usize(i: usize) -> Self {
        NodeI(i as u32)
    }
}

pub const ROOT_I: NodeI = NodeI(0);

#[derive(Debug)]
pub(crate) struct NodeVec<T>(pub Vec<T>);

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
pub(crate) struct Links {
    pub parent: NodeI,
    pub first_child: Option<NodeI>,
    pub last_child: Option<NodeI>,
    pub next_sibling: Option<NodeI>,
}

/// Measures the text of a node in `Nodes`. The width and the result are in fractions of the window size.
pub type MeasureText<'a> = dyn FnMut(NodeI, Option<f32>) -> Xy<f32> + 'a;

/// A simple standalone tree, with the layout data in a `Vec`.
#[derive(Debug)]
pub struct Nodes {
    links: NodeVec<Links>,
    params: NodeVec<LayoutParams>,
    layouts: NodeVec<NodeLayout>,
    rects: NodeVec<XyRect>,
    window_size: Xy<f32>,
    n_deferred_solves: usize,
}

/// `Nodes` with a text measuring function, as a `TreeAdapter`.
struct NodesAdapter<'a, 'b> {
    nodes: &'a mut Nodes,
    measure_text: &'a mut MeasureText<'b>,
}

impl TreeAdapter for NodesAdapter<'_, '_> {
    type Node = NodeI;

    #[inline(always)]
    fn root(&self) -> NodeI {
        ROOT_I
    }

    #[inline(always)]
    fn parent(&self, i: NodeI) -> NodeI {
        self.nodes.links[i].parent
    }

    #[inline(always)]
    fn first_child(&self, i: NodeI) -> Option<NodeI> {
        self.nodes.links[i].first_child
    }

    #[inline(always)]
    fn next_sibling(&self, i: NodeI) -> Option<NodeI> {
        self.nodes.links[i].next_sibling
    }

    #[inline(always)]
    fn params(&self, i: NodeI) -> &LayoutParams {
        &self.nodes.params[i]
    }

    #[inline(always)]
    fn layout_data(&self, i: NodeI) -> &NodeLayout {
        &self.nodes.layouts[i]
    }

    #[inline(always)]
    fn layout_data_mut(&mut self, i: NodeI) -> &mut NodeLayout {
        &mut self.nodes.layouts[i]
    }

    #[inline(always)]
    fn measure_text(&mut self, i: NodeI, width: Option<f32>) -> Xy<f32> {
        (self.measure_text)(i, width)
    }

    #[inline(always)]
    fn write_output_rect(&mut self, i: NodeI, rect: XyRect) {
        self.nodes.rects[i] = rect;
    }
}

impl Nodes {
    pub fn new() -> Self {
        let mut nodes = Nodes {
            links: NodeVec(Vec::new()),
            params: NodeVec(Vec::new()),
            layouts: NodeVec(Vec::new()),
            rects: NodeVec(Vec::new()),
            window_size: Xy::new(0.0, 0.0),
            n_deferred_solves: 0,
        };
        nodes.push(ROOT_I, LayoutParams::DEFAULT);
        nodes
    }

    fn push(&mut self, parent: NodeI, params: LayoutParams) -> NodeI {
        let i = NodeI(self.links.0.len() as u32);
        self.links.0.push(Links { parent, first_child: None, last_child: None, next_sibling: None });
        self.params.0.push(params);
        self.layouts.0.push(NodeLayout::EMPTY);
        self.rects.0.push(Xy::new([0.0, 0.0], [0.0, 0.0]));
        i
    }

    pub fn add(&mut self, parent: NodeI, params: LayoutParams) -> NodeI {
        let i = self.push(parent, params);
        match self.links[parent].last_child {
            Some(last) => self.links[last].next_sibling = Some(i),
            None => self.links[parent].first_child = Some(i),
        }
        self.links[parent].last_child = Some(i);
        i
    }

    pub fn len(&self) -> usize {
        self.links.0.len()
    }

    pub fn compute(&mut self, window_size: Xy<f32>, measure_text: &mut MeasureText) {
        self.window_size = window_size;
        let mut adapter = NodesAdapter { nodes: self, measure_text };
        let n_deferred_solves = compute(&mut adapter, window_size);
        self.n_deferred_solves = n_deferred_solves;
    }

    #[doc(hidden)]
    pub fn deferred_solves(&self) -> usize {
        self.n_deferred_solves
    }

    pub fn rect_px(&self, i: NodeI) -> XyRect {
        let rect = self.rects[i];
        let window = self.window_size;
        Xy::new(
            [rect.x[0] * window.x, rect.x[1] * window.x],
            [rect.y[0] * window.y, rect.y[1] * window.y],
        )
    }
}

impl Default for Nodes {
    fn default() -> Self {
        Self::new()
    }
}

thread_local! {
    static THREAD_ARENA: RefCell<bumpalo::Bump> = RefCell::new(bumpalo::Bump::new());
}

pub(crate) fn with_arena<F, R>(f: F) -> R
where
    F: FnOnce(&bumpalo::Bump) -> R,
{
    THREAD_ARENA.with(|arena| f(&arena.borrow()))
}

pub(crate) fn reset_arena() {
    THREAD_ARENA.with(|arena| arena.borrow_mut().reset());
}
