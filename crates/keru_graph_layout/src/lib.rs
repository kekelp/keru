#[macro_use]
mod solve;
mod place;

use std::cell::RefCell;
use std::ops::{Index, IndexMut};

pub use solve::{Layout, LayoutParams, MeasureText, TextContent};
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

/// The tree that the layout runs on. The nodes are indexed by `NodeI`, and every index is smaller than `index_bound`. The indices don't have to be contiguous.
pub trait TreeAdapter {
    fn root(&self) -> NodeI;
    /// The parent of a node. It is never called on the root.
    fn parent(&self, i: NodeI) -> NodeI;
    fn first_child(&self, i: NodeI) -> Option<NodeI>;
    fn next_sibling(&self, i: NodeI) -> Option<NodeI>;
    fn params(&self, i: NodeI) -> &LayoutParams;
    /// An upper bound for the indices of the nodes. The `Layout` keeps this many elements.
    fn index_bound(&self) -> usize;
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Links {
    pub parent: NodeI,
    pub first_child: Option<NodeI>,
    pub last_child: Option<NodeI>,
    pub next_sibling: Option<NodeI>,
}

/// A simple standalone tree that implements `TreeAdapter`.
#[derive(Debug)]
pub struct VecTree {
    pub(crate) links: NodeVec<Links>,
    pub(crate) params: NodeVec<LayoutParams>,
}

impl TreeAdapter for VecTree {
    #[inline(always)]
    fn root(&self) -> NodeI {
        ROOT_I
    }

    #[inline(always)]
    fn parent(&self, i: NodeI) -> NodeI {
        self.links[i].parent
    }

    #[inline(always)]
    fn first_child(&self, i: NodeI) -> Option<NodeI> {
        self.links[i].first_child
    }

    #[inline(always)]
    fn next_sibling(&self, i: NodeI) -> Option<NodeI> {
        self.links[i].next_sibling
    }

    #[inline(always)]
    fn params(&self, i: NodeI) -> &LayoutParams {
        &self.params[i]
    }

    #[inline(always)]
    fn index_bound(&self) -> usize {
        self.links.0.len()
    }
}

#[derive(Debug)]
pub struct Nodes {
    pub tree: VecTree,
    pub layout: Layout,
}

impl Nodes {
    pub fn new() -> Self {
        let mut nodes = Nodes {
            tree: VecTree {
                links: NodeVec(Vec::new()),
                params: NodeVec(Vec::new()),
            },
            layout: Layout::new(),
        };
        nodes.push(ROOT_I, LayoutParams::DEFAULT);
        nodes
    }

    fn push(&mut self, parent: NodeI, params: LayoutParams) -> NodeI {
        let i = NodeI(self.tree.links.0.len() as u32);
        self.tree.links.0.push(Links { parent, first_child: None, last_child: None, next_sibling: None });
        self.tree.params.0.push(params);
        i
    }

    pub fn add(&mut self, parent: NodeI, params: LayoutParams) -> NodeI {
        let i = self.push(parent, params);
        let links = &mut self.tree.links;
        match links[parent].last_child {
            Some(last) => links[last].next_sibling = Some(i),
            None => links[parent].first_child = Some(i),
        }
        links[parent].last_child = Some(i);
        i
    }

    pub fn len(&self) -> usize {
        self.tree.links.0.len()
    }

    pub fn compute(&mut self, window_size: Xy<f32>, measure_text: &mut MeasureText) {
        self.layout.compute(&self.tree, window_size, measure_text);
    }

    #[doc(hidden)]
    pub fn deferred_solves(&self) -> usize {
        self.layout.deferred_solves()
    }

    pub fn rect_px(&self, i: NodeI) -> XyRect {
        self.layout.rect_px(i)
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
