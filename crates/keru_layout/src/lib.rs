#[macro_use]
mod solve;
mod place;
pub mod example_tree;

use std::cell::RefCell;
use std::fmt::Debug;
use std::ops::{Index, IndexMut};

pub use solve::NodeLayoutData;
pub(crate) use solve::*;
pub(crate) use place::Place;

pub trait TreeAdapter {
    type NodeId: Copy + Eq + Debug;
    fn root(&self) -> Self::NodeId;
    fn parent(&self, i: Self::NodeId) -> Self::NodeId;
    fn first_child(&self, i: Self::NodeId) -> Option<Self::NodeId>;
    fn next_sibling(&self, i: Self::NodeId) -> Option<Self::NodeId>;
    fn node_layout_params(&self, i: Self::NodeId) -> &LayoutParams;
    fn node_layout_data(&self, i: Self::NodeId) -> &NodeLayoutData;
    fn node_layout_data_mut(&mut self, i: Self::NodeId) -> &mut NodeLayoutData;
    /// The width of the text in a node, without any wrapping, in logical pixels.
    fn measure_text_width(&mut self, i: Self::NodeId) -> f32;
    
    /// The height of the text in a node, with its width optionally fixed to `wrap_width`, in logical pixels.
    fn measure_text_height(&mut self, i: Self::NodeId, wrap_width: Option<f32>) -> f32;
    /// Called when the layout solver computes the end result for a node's rectangle.
    /// The output is provided in both absolute coordinates and relative to the parent, in logical pixels.
    fn write_output_rect(&mut self, i: Self::NodeId, rect: XyRect, local_rect: XyRect);

    /// The size of the window, in logical pixels.
    fn window_size(&self) -> Xy<f32>;
    
    /// Called when the layout solver finds that a node is part of a dependency cycle.
    /// 
    /// Depending on the use case, this can be reported to the user.
    fn cycle_found(&mut self, _i: Self::NodeId) {}

    /// The maximum amount of scratch memory that the layout solver can retain.
    fn max_retained_scratch_memory(&self) -> usize {
        1 << 20
    }

    /// Solves the layout on all nodes.
    fn solve_layout(&mut self) {
        self.measure_sizes();
        self.place_nodes();
    }

    /// Places a free-placement node into a new position. Can be used to move scroll bars without requiring a full relayout. `parent_rect` is in logical pixels.
    fn place_free_placed_subtree(&mut self, node: Self::NodeId, new_rect: XyRect) {
        let new_rect = self.rect_to_frac(new_rect);
        self.place_free_node(node, new_rect);
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextContent {
    None,
    Box,
    Edit,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutParams {
    pub size: Xy<Size>,
    pub min_size: Xy<Option<Size>>,
    pub max_size: Xy<Option<Size>>,
    /// In a stack, the `Fill` children share the free space in proportion to their weight. It has no effect on the other axis.
    pub fill_weight: f32,
    pub padding: Xy<[f32; 2]>,
    pub children_layout: ChildrenLayout,
    pub free_placement: bool,
    pub grid_span: Xy<u16>,
    pub text: TextContent,
    pub image: Option<Xy<f32>>,
    /// For "exiting nodes".
    pub disabled: bool,
    pub position: Xy<Pos>,
    pub anchor: Xy<Anchor>,
    /// True if the children are placed from the right or the bottom edge.
    pub children_origin_flipped: Xy<bool>,
    pub window_overflow: Xy<WindowOverflow>,
}

impl LayoutParams {
    pub const DEFAULT: LayoutParams = LayoutParams {
        size: Xy { x: Size::FitContent, y: Size::FitContent },
        min_size: Xy { x: None, y: None },
        max_size: Xy { x: None, y: None },
        fill_weight: 1.0,
        padding: Xy { x: [0.0, 0.0], y: [0.0, 0.0] },
        children_layout: ChildrenLayout::Free,
        free_placement: false,
        grid_span: Xy { x: 1, y: 1 },
        text: TextContent::None,
        image: None,
        disabled: false,
        position: Xy { x: Pos::Center, y: Pos::Center },
        anchor: Xy { x: Anchor::Start, y: Anchor::Start },
        children_origin_flipped: Xy { x: false, y: false },
        window_overflow: Xy { x: WindowOverflow::Ignore, y: WindowOverflow::Ignore },
    };
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

pub(crate) fn reset_arena(max_retained: usize) {
    THREAD_ARENA.with(|arena| {
        let mut arena = arena.borrow_mut();
        arena.reset();
        if arena.allocated_bytes() > max_retained {
            *arena = bumpalo::Bump::with_capacity(max_retained.saturating_sub(8192));
        }
    });
}
