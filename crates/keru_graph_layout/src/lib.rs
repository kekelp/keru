#[macro_use]
mod solve;
mod place;
mod example_tree;

use std::cell::RefCell;
use std::fmt::Debug;
use std::ops::{Index, IndexMut};

pub use solve::{LayoutParams, NodeLayout, TextContent, compute, place_free_placed_subtree, solve};
pub(crate) use solve::*;
pub use example_tree::{ExampleTree, NodeI, ROOT_I};

pub trait TreeAdapter {
    type NodeId: Copy + Eq + Debug;
    fn root(&self) -> Self::NodeId;
    fn parent(&self, i: Self::NodeId) -> Self::NodeId;
    fn first_child(&self, i: Self::NodeId) -> Option<Self::NodeId>;
    fn next_sibling(&self, i: Self::NodeId) -> Option<Self::NodeId>;
    fn params(&self, i: Self::NodeId) -> &LayoutParams;
    fn layout_data(&self, i: Self::NodeId) -> &NodeLayout;
    fn layout_data_mut(&mut self, i: Self::NodeId) -> &mut NodeLayout;
    /// Measures the text of a node. The width and the result are in fractions of the window size. It is called only on nodes with text.
    fn measure_text(&mut self, i: Self::NodeId, width: Option<f32>) -> Xy<f32>;
    /// Called when the layout solver computes the end result for a node's rectangle.
    /// The output is provided in both absolute coordinates and relative to the parent.
    fn write_output_rect(&mut self, i: Self::NodeId, rect: XyRect, local_rect: XyRect);
}

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
