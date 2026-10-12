use crate::*;
use crate::Axis::{X, Y};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextContent {
    None,
    Box,
    Edit,
}

/// The parts of a node that the layout reads.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct LayoutParams {
    pub size: Xy<Size>,
    pub min_size: Xy<Option<Size>>,
    pub max_size: Xy<Option<Size>>,
    pub padding: Xy<f32>,
    pub children_layout: ChildrenLayout,
    pub free_placement: bool,
    pub grid_span: Xy<u16>,
    pub text: TextContent,
    /// The size of the node's raster image in pixels.
    pub image: Option<Xy<f32>>,
    /// Exiting nodes stay in their parent's child list for the exit animation, but the layout skips them.
    pub disabled: bool,
}

const EMPTY_PARAMS: LayoutParams = LayoutParams {
    size: Xy { x: Size::Pixels(0.0), y: Size::Pixels(0.0) },
    min_size: Xy { x: None, y: None },
    max_size: Xy { x: None, y: None },
    padding: Xy { x: 0.0, y: 0.0 },
    children_layout: ChildrenLayout::Free,
    free_placement: false,
    grid_span: Xy { x: 1, y: 1 },
    text: TextContent::None,
    image: None,
    disabled: true,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Regular,
    Min,
    Max,
    Final,
    EvenShare,
    GridCells,
}

const N_KINDS: usize = 6;
pub(crate) const KINDS: [Kind; N_KINDS] = [Kind::Regular, Kind::Min, Kind::Max, Kind::Final, Kind::EvenShare, Kind::GridCells];
pub(crate) const DECLARED: [Kind; 3] = [Kind::Regular, Kind::Min, Kind::Max];
pub(crate) const N_SLOTS: usize = 2 * N_KINDS;

#[inline(always)]
pub(crate) fn slot_index(axis: Axis, kind: Kind) -> usize {
    axis as usize * N_KINDS + kind as usize
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Slot {
    pub node: NodeI,
    pub axis: Axis,
    pub kind: Kind,
}

/// The solver data of one node. It is reset at the start of every solve.
#[derive(Debug, Clone, Copy)]
pub(crate) struct SolverData {
    /// The solved value of each slot, or 0 if it isn't solved yet.
    pub solved: [f32; N_SLOTS],
    /// One bit for each slot that is solved.
    pub solved_mask: u16,
    /// How many dependencies of each slot are not solved yet.
    pub n_unsolved: [u32; N_SLOTS],
    /// Range of the dependents lists. The dependents of slot `k` are `dependents[dependents[k]..dependents[k + 1]]`.
    pub dependents: [u32; N_SLOTS + 1],
    /// How many children aren't free-placement.
    pub n_placed_children: u32,
    /// The cell of this node in its parent's grid.
    pub grid_cell: Xy<u16>,
    /// The number of columns and rows, if this node is a grid.
    pub grid_lines: Xy<u16>,
}

pub(crate) const EMPTY_SOLVER_DATA: SolverData = SolverData {
    solved: [0.0; N_SLOTS],
    solved_mask: 0,
    n_unsolved: [0; N_SLOTS],
    dependents: [0; N_SLOTS + 1],
    n_placed_children: 0,
    grid_cell: Xy { x: 0, y: 0 },
    grid_lines: Xy { x: 0, y: 0 },
};

#[inline(always)]
pub(crate) fn not_fill(size: Size) -> bool {
    size != Size::Fill
}

#[inline(always)]
pub(crate) fn not_flex(size: Size) -> bool {
    ! matches!(size, Size::Frac(_) | Size::Fill)
}

macro_rules! for_each_layout_child {
    ($nodes:expr, $i:expr, $child:ident, $body:block) => {{
        let mut next = $nodes.links[$i].first_child;
        while let Some($child) = next {
            next = $nodes.links[$child].next_sibling;
            if ! $nodes.layout.params[$child].disabled {
                $body
            }
        }
    }};
}
pub(crate) use for_each_layout_child;

#[derive(Debug)]
pub(crate) struct LayoutTree {
    pub params: NodeVec<LayoutParams>,
    /// The window size in logical pixels. All the results are fractions of it.
    pub window_size: Xy<f32>,
    pub solver_data: NodeVec<SolverData>,
    pub dependents: Vec<Slot>,
    pub solve_queue: Vec<Slot>,
}

impl LayoutTree {
    pub fn new() -> Self {
        LayoutTree {
            params: NodeVec(Vec::with_capacity(100)),
            window_size: Xy::new(0.0, 0.0),
            solver_data: NodeVec(Vec::with_capacity(100)),
            dependents: Vec::with_capacity(400),
            solve_queue: Vec::with_capacity(400),
        }
    }

    pub fn init(&mut self, i: NodeI, params: LayoutParams) {
        if self.params.0.len() <= i.as_usize() {
            self.params.0.resize(i.as_usize() + 1, EMPTY_PARAMS);
        }
        self.params[i] = params;
    }

    /// The final size of a node, as a fraction of the window.
    pub fn size(&self, i: NodeI) -> Xy<f32> {
        let solved = &self.solver_data[i].solved;
        Xy::new(solved[slot_index(X, Kind::Final)], solved[slot_index(Y, Kind::Final)])
    }

    /// The cell of a node in its parent's grid.
    pub fn grid_cell(&self, i: NodeI) -> Xy<u16> {
        self.solver_data[i].grid_cell
    }

    /// The number of columns and rows of a grid.
    pub fn grid_lines(&self, i: NodeI) -> Xy<u16> {
        self.solver_data[i].grid_lines
    }
}

impl Nodes {
    #[inline(always)]
    pub(crate) fn declared(&self, i: NodeI, axis: Axis, kind: Kind) -> Option<Size> {
        let node = &self.layout.params[i];
        match kind {
            Kind::Regular => Some(node.size[axis]),
            Kind::Min => node.min_size[axis],
            Kind::Max => node.max_size[axis],
            Kind::Final | Kind::EvenShare | Kind::GridCells => None,
        }
    }

    #[inline(always)]
    pub(crate) fn any_size_is_fill(&self, i: NodeI, axis: Axis) -> bool {
        DECLARED.iter().any(|&kind| self.declared(i, axis, kind) == Some(Size::Fill))
    }

    #[inline(always)]
    pub(crate) fn text_wraps(&self, i: NodeI, axis: Axis) -> bool {
        let node = &self.layout.params[i];
        // With an AspectRatio width, the width depends on the height, so the height can't depend on the width.
        axis == Y && node.text == TextContent::Box && ! matches!(node.size[X], Size::AspectRatio(_))
    }

    #[inline(always)]
    fn stack_axis(&self, i: NodeI) -> Option<Axis> {
        match self.layout.params[i].children_layout {
            ChildrenLayout::Stack { axis, .. } => Some(axis),
            _ => None,
        }
    }

    /// True if the node is one of the children that its parent stacks along this axis.
    #[inline(always)]
    pub(crate) fn is_stacked(&self, i: NodeI, axis: Axis) -> bool {
        let parent = self.links[i].parent;
        self.stack_axis(parent) == Some(axis) && ! self.layout.params[i].free_placement
    }

    #[inline(always)]
    pub(crate) fn has_even_share(&self, i: NodeI, axis: Axis) -> bool {
        self.stack_axis(i) == Some(axis) && self.layout.params[i].size[axis] != Size::FitContent
    }
}

pub(crate) fn layout_params(node: &InnerNode) -> LayoutParams {
    let params = &node.params;
    let text = match node.text_i {
        Some(TextI::TextBox(_)) => TextContent::Box,
        Some(TextI::TextEdit(_)) => TextContent::Edit,
        None => TextContent::None,
    };
    LayoutParams {
        size: params.layout.size,
        min_size: params.layout.min_size,
        max_size: params.layout.max_size,
        padding: params.layout.padding,
        children_layout: params.children_layout,
        free_placement: params.free_placement,
        grid_span: Xy::new(params.grid_element.column_span, params.grid_element.row_span),
        text,
        image: image_size(node),
        disabled: node.exiting,
    }
}

pub(crate) fn image_size(node: &InnerNode) -> Option<Xy<f32>> {
    match &node.imageref {
        Some(ImageRef::Raster(loaded)) => Some(Xy::new(loaded.width as f32, loaded.height as f32)),
        _ => None,
    }
}
