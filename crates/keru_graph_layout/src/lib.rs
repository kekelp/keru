#[macro_use]
mod solve;
mod place;

use std::cell::RefCell;
use std::ops::{Index, IndexMut};

pub use solve::{LayoutParams, MeasureText, TextContent};
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

#[derive(Debug, Clone, Copy)]
pub(crate) struct Links {
    pub parent: NodeI,
    pub first_child: Option<NodeI>,
    pub last_child: Option<NodeI>,
    pub next_sibling: Option<NodeI>,
}

#[derive(Debug)]
pub struct Nodes {
    pub(crate) links: NodeVec<Links>,
    pub(crate) layout: LayoutTree,
    pub(crate) rects: NodeVec<XyRect>,
}

impl Nodes {
    pub fn new() -> Self {
        let mut nodes = Nodes {
            links: NodeVec(Vec::new()),
            layout: LayoutTree::new(),
            rects: NodeVec(Vec::new()),
        };
        nodes.push(ROOT_I, LayoutParams::DEFAULT);
        nodes
    }

    fn push(&mut self, parent: NodeI, params: LayoutParams) -> NodeI {
        let i = NodeI(self.links.0.len() as u32);
        self.links.0.push(Links { parent, first_child: None, last_child: None, next_sibling: None });
        self.layout.params.0.push(params);
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
        reset_arena();
        self.solve_layout(window_size, measure_text);
        self.place_all();
    }

    #[doc(hidden)]
    pub fn deferred_solves(&self) -> usize {
        self.layout.n_deferred_solves
    }

    pub fn rect_px(&self, i: NodeI) -> XyRect {
        let rect = self.rects[i];
        let window = self.layout.window_size;
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
