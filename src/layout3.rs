use crate::*;
use crate::Axis::{X, Y};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextContent {
    None,
    Box,
    Edit,
}

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
    pub image: Option<Xy<f32>>,
    /// For "exiting nodes".
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

#[derive(Debug)]
pub(crate) struct LayoutTree {
    pub params: NodeVec<LayoutParams>,
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

    pub fn size(&self, i: NodeI) -> Xy<f32> {
        let solved = &self.solver_data[i].solved;
        Xy::new(solved[slot_index(X, Kind::Final)], solved[slot_index(Y, Kind::Final)])
    }

    pub fn grid_cell(&self, i: NodeI) -> Xy<u16> {
        self.solver_data[i].grid_cell
    }

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
        axis == Y && node.text == TextContent::Box && ! matches!(node.size[X], Size::AspectRatio(_))
    }

    #[inline(always)]
    fn stack_axis(&self, i: NodeI) -> Option<Axis> {
        match self.layout.params[i].children_layout {
            ChildrenLayout::Stack { axis, .. } => Some(axis),
            _ => None,
        }
    }

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

impl Nodes {
    pub(crate) fn build_dependency_graph(&mut self) {
        self.layout.solver_data.0.clear();
        self.layout.solver_data.0.resize(self.layout.params.0.len(), EMPTY_SOLVER_DATA);
        self.layout.dependents.clear();
        self.layout.solve_queue.clear();
        self.build_dependency_graph_recursive(ROOT_I);
    }

    fn build_dependency_graph_recursive(&mut self, i: NodeI) {
        self.layout.solver_data[i].dependents[0] = self.layout.dependents.len() as u32;
        let mut existing: u16 = 0;
        for axis in [X, Y] {
            for kind in KINDS {
                if self.slot_exists(i, axis, kind) {
                    existing |= 1 << slot_index(axis, kind);
                    let start = self.layout.dependents.len();
                    self.push_dependents(i, axis, kind);
                    for d in start..self.layout.dependents.len() {
                        let dependent = self.layout.dependents[d];
                        self.layout.solver_data[dependent.node].n_unsolved[slot_index(dependent.axis, dependent.kind)] += 1;
                    }
                }
                self.layout.solver_data[i].dependents[slot_index(axis, kind) + 1] = self.layout.dependents.len() as u32;
            }
        }

        for_each_layout_child!(self, i, child, {
            if ! self.layout.params[child].free_placement {
                self.layout.solver_data[i].n_placed_children += 1;
            }
            self.build_dependency_graph_recursive(child);
        });

        for axis in [X, Y] {
            for kind in KINDS {
                let k = slot_index(axis, kind);
                if existing & (1 << k) != 0 && self.layout.solver_data[i].n_unsolved[k] == 0 {
                    self.layout.solve_queue.push(Slot { node: i, axis, kind });
                }
            }
        }
    }

    #[inline(always)]
    fn slot_exists(&self, i: NodeI, axis: Axis, kind: Kind) -> bool {
        match kind {
            Kind::Regular | Kind::Final => true,
            Kind::Min | Kind::Max => self.declared(i, axis, kind).is_some(),
            Kind::EvenShare => self.has_even_share(i, axis),
            Kind::GridCells => self.grid_main_axis(i) == Some(axis),
        }
    }

    #[inline(always)]
    fn grid_main_axis(&self, i: NodeI) -> Option<Axis> {
        match self.layout.params[i].children_layout {
            ChildrenLayout::Grid { flow, .. } => Some(flow.main_axis),
            _ => None,
        }
    }

    #[inline(always)]
    fn add_dependent(&mut self, node: NodeI, axis: Axis, kind: Kind) {
        self.layout.dependents.push(Slot { node, axis, kind });
    }

    fn push_dependents(&mut self, i: NodeI, axis: Axis, kind: Kind) {
        let parent = self.links[i].parent;
        let in_parent = i != ROOT_I && ! self.layout.params[i].free_placement;
        let flex = ! not_flex(self.layout.params[i].size[axis]);

        match kind {
            Kind::Regular | Kind::Min | Kind::Max => {
                self.add_dependent(i, axis, Kind::Final);
                let declared = self.declared(i, axis, kind).unwrap();
                if in_parent && self.has_even_share(parent, axis) && self.any_size_is_fill(i, axis) && not_fill(declared) {
                    self.add_dependent(parent, axis, Kind::EvenShare);
                }
                if in_parent && flex && kind != Kind::Regular && not_flex(declared) {
                    self.add_fitcontents(parent, axis);
                }
            }
            Kind::Final => {
                if self.has_even_share(i, axis) {
                    self.add_dependent(i, axis, Kind::EvenShare);
                }
                if self.grid_main_axis(i) == Some(axis) && matches!(self.layout.params[i].children_layout, ChildrenLayout::Grid { columns: MainAxisCellSize::Width(_), .. }) {
                    self.add_dependent(i, axis, Kind::GridCells);
                }
                for other in DECLARED {
                    if matches!(self.declared(i, axis.other(), other), Some(Size::AspectRatio(_))) {
                        self.add_dependent(i, axis.other(), other);
                    }
                }
                if axis == X && self.text_wraps(i, Y) {
                    self.add_fitcontents(i, Y);
                }
                if in_parent && self.has_even_share(parent, axis) && ! self.any_size_is_fill(i, axis) {
                    self.add_dependent(parent, axis, Kind::EvenShare);
                }
                if in_parent && ! flex {
                    self.add_fitcontents(parent, axis);
                }
                for_each_layout_child!(self, i, child, {
                    self.add_available_from_parent_readers(child, axis);
                });
            }
            Kind::EvenShare => {
                for_each_layout_child!(self, i, child, {
                    for child_kind in DECLARED {
                        if self.declared(child, axis, child_kind) == Some(Size::Fill) && self.is_stacked(child, axis) {
                            self.add_dependent(child, axis, child_kind);
                        }
                    }
                });
            }
            Kind::GridCells => {
                self.add_fitcontents(i, X);
                self.add_fitcontents(i, Y);
                for_each_layout_child!(self, i, child, {
                    if ! self.layout.params[child].free_placement {
                        self.add_available_from_parent_readers(child, X);
                        self.add_available_from_parent_readers(child, Y);
                    }
                });
            }
        }
    }

    #[inline(always)]
    fn add_fitcontents(&mut self, i: NodeI, axis: Axis) {
        for kind in DECLARED {
            if self.declared(i, axis, kind) == Some(Size::FitContent) {
                self.add_dependent(i, axis, kind);
            }
        }
    }

    #[inline(always)]
    fn add_available_from_parent_readers(&mut self, child: NodeI, axis: Axis) {
        for kind in DECLARED {
            let reads = match self.declared(child, axis, kind) {
                Some(Size::Frac(_)) => true,
                Some(Size::Fill) => ! self.is_stacked(child, axis),
                _ => false,
            };
            if reads {
                self.add_dependent(child, axis, kind);
            }
        }
    }
}

use bumpalo::collections::Vec as BumpVec;

const MAX_FIT_STACK_FRAC: f32 = 0.999;
const MAX_FIT_STACK_STEPS: usize = 64;
const FIT_STACK_EPSILON: f32 = 1e-7;

#[inline(always)]
fn clamp(size: f32, min: Option<f32>, max: Option<f32>) -> f32 {
    let size = max.map_or(size, |max| size.min(max));
    min.map_or(size, |min| size.max(min))
}

#[inline(always)]
fn inner(size: f32, padding: f32) -> f32 {
    (size - 2.0 * padding).max(0.0)
}

pub(crate) type MeasureText<'a> = dyn FnMut(&InnerNode, Option<f32>) -> Xy<f32> + 'a;

impl Nodes {
    pub(crate) fn solve_layout(&mut self, window_size: Xy<f32>, measure_text: &mut MeasureText) {
        self.layout.window_size = window_size;
        self.build_dependency_graph();
        self.solve(measure_text);
    }

    fn solve(&mut self, measure_text: &mut MeasureText) {
        let mut next = 0;
        while next < self.layout.solve_queue.len() {
            let slot = self.layout.solve_queue[next];
            next += 1;
            let size = self.solve_slot(slot, measure_text);
            let k = slot_index(slot.axis, slot.kind);
            self.layout.solver_data[slot.node].solved[k] = size;
            self.layout.solver_data[slot.node].solved_mask |= 1 << k;

            let range = self.layout.solver_data[slot.node].dependents;
            for d in range[k]..range[k + 1] {
                let dependent = self.layout.dependents[d as usize];
                let count = &mut self.layout.solver_data[dependent.node].n_unsolved[slot_index(dependent.axis, dependent.kind)];
                *count -= 1;
                if *count == 0 {
                    self.layout.solve_queue.push(dependent);
                }
            }
        }
    }

    #[inline(always)]
    fn try_get_solved_value(&self, i: NodeI, axis: Axis, kind: Kind) -> Option<f32> {
        let k = slot_index(axis, kind);
        let slots = &self.layout.solver_data[i];
        (slots.solved_mask & (1 << k) != 0).then_some(slots.solved[k])
    }

    #[inline(always)]
    fn get_solved_value(&self, i: NodeI, axis: Axis, kind: Kind) -> f32 {
        self.layout.solver_data[i].solved[slot_index(axis, kind)]
    }

    #[inline(always)]
    fn pixels_to_frac(&self, pixels: f32, axis: Axis) -> f32 {
        pixels / self.layout.window_size[axis]
    }

    fn solve_slot(&mut self, slot: Slot, measure_text: &mut MeasureText) -> f32 {
        let i = slot.node;
        let axis = slot.axis;
        match slot.kind {
            Kind::Regular if i == ROOT_I => 1.0,
            Kind::Final => {
                let min = self.try_get_solved_value(i, axis, Kind::Min);
                let max = self.try_get_solved_value(i, axis, Kind::Max);
                clamp(self.get_solved_value(i, axis, Kind::Regular), min, max)
            }
            Kind::EvenShare => self.solve_even_share(i, axis),
            Kind::GridCells => self.solve_grid_cells(i),
            declared => match self.declared(i, axis, declared).unwrap() {
                Size::Pixels(pixels) => self.pixels_to_frac(pixels, axis),
                Size::AspectRatio(aspect) => {
                    let other = self.get_solved_value(i, axis.other(), Kind::Final);
                    let window_aspect = self.layout.window_size.x / self.layout.window_size.y;
                    match axis {
                        X => other * aspect / window_aspect,
                        Y => other * window_aspect / aspect,
                    }
                }
                Size::FitContent => self.solve_fitcontent(i, axis, measure_text),
                Size::Frac(frac) => self.available_from_parent(i, axis) * frac,
                Size::Fill if self.is_stacked(i, axis) => {
                    let parent = self.links[i].parent;
                    match self.has_even_share(parent, axis) {
                        true => self.get_solved_value(parent, axis, Kind::EvenShare),
                        false => 0.0,
                    }
                }
                Size::Fill => self.available_from_parent(i, axis),
            },
        }
    }

    fn available_from_parent(&self, i: NodeI, axis: Axis) -> f32 {
        let parent = self.links[i].parent;
        let inner = inner(self.get_solved_value(parent, axis, Kind::Final), self.padding(parent, axis));
        if self.layout.params[i].free_placement {
            return inner;
        }
        match self.layout.params[parent].children_layout {
            ChildrenLayout::Grid { spacing_x, spacing_y, .. } => {
                let spacing = self.pixels_to_frac(Xy::new(spacing_x, spacing_y)[axis], axis);
                let n = self.layout.solver_data[parent].grid_lines[axis] as f32;
                let cell = ((inner - spacing * (n - 1.0)) / n.max(1.0)).max(0.0);
                let span = self.layout.params[i].grid_span[axis].max(1) as f32;
                span * cell + (span - 1.0) * spacing
            }
            ChildrenLayout::Stack { axis: stack_axis, .. } if stack_axis == axis => {
                (inner - self.stack_gaps(parent, axis)).max(0.0)
            }
            _ => inner,
        }
    }

    fn solve_fitcontent(&mut self, i: NodeI, axis: Axis, measure_text: &mut MeasureText) -> f32 {
        let (children, gaps) = match self.layout.params[i].children_layout {
            ChildrenLayout::Grid { columns, spacing_x, spacing_y, flow } => {
                let spacing = self.pixels_to_frac(Xy::new(spacing_x, spacing_y)[axis], axis);
                let n = self.layout.solver_data[i].grid_lines[axis] as f32;
                let cell = match columns {
                    MainAxisCellSize::Width(w) if axis == flow.main_axis => self.pixels_to_frac(w, axis),
                    // Each cell is as big as the biggest child in it.
                    _ => {
                        let mut cell = 0.0f32;
                        for_each_layout_child!(self, i, child, {
                            if ! self.layout.params[child].free_placement {
                                let span = self.layout.params[child].grid_span[axis].max(1) as f32;
                                cell = cell.max((self.child_fit_size(child, axis) - (span - 1.0) * spacing) / span);
                            }
                        });
                        cell
                    }
                };
                ((n * cell + spacing * (n - 1.0)).max(0.0), 0.0)
            }
            ChildrenLayout::Stack { axis: stack_axis, .. } if stack_axis == axis => {
                (self.fit_stack_main(i, axis), self.stack_gaps(i, axis))
            }
            ChildrenLayout::Stack { .. } | ChildrenLayout::Free => {
                let mut biggest = 0.0f32;
                for_each_layout_child!(self, i, child, {
                    if ! self.layout.params[child].free_placement {
                        biggest = biggest.max(self.child_fit_size(child, axis));
                    }
                });
                (biggest, 0.0)
            }
        };

        let mut own_content = match self.layout.params[i].text {
            TextContent::None => 0.0,
            _ if self.text_wraps(i, axis) => {
                let width = inner(self.get_solved_value(i, X, Kind::Final), self.padding(i, X));
                measure_text(&self[i], Some(width))[Y]
            }
            _ => measure_text(&self[i], None)[axis],
        };
        if let Some(image) = self.layout.params[i].image {
            own_content = own_content.max(self.pixels_to_frac(image[axis], axis));
        }

        children.max(own_content) + gaps + 2.0 * self.padding(i, axis)
    }

    fn fit_stack_main(&self, i: NodeI, axis: Axis) -> f32 {
        let mut estimate = 0.0f32;
        for _ in 0..MAX_FIT_STACK_STEPS {
            let mut fixed = 0.0f32;
            let mut free_frac = 0.0f32;
            let mut has_frac = false;
            for_each_layout_child!(self, i, child, {
                if self.layout.params[child].free_placement {
                    // Free children aren't part of the stack.
                } else if let Size::Frac(frac) = self.layout.params[child].size[axis] {
                    has_frac = true;
                    let wanted = estimate * frac;
                    let allowed = clamp(wanted, self.try_get_solved_value(child, axis, Kind::Min), self.try_get_solved_value(child, axis, Kind::Max));
                    match allowed == wanted {
                        true => free_frac += frac,
                        false => fixed += allowed,
                    }
                } else {
                    fixed += self.child_fit_size(child, axis);
                }
            });

            if ! has_frac {
                return fixed;
            }

            let next = match free_frac <= MAX_FIT_STACK_FRAC {
                true => fixed / (1.0 - free_frac),
                false => fixed,
            };
            let settled = (next - estimate).abs() <= FIT_STACK_EPSILON;
            estimate = next;
            if settled {
                break;
            }
        }
        estimate
    }

    #[inline(always)]
    fn child_fit_size(&self, child: NodeI, axis: Axis) -> f32 {
        match not_flex(self.layout.params[child].size[axis]) {
            true => self.get_solved_value(child, axis, Kind::Final),
            false => clamp(0.0, self.try_get_solved_value(child, axis, Kind::Min), self.try_get_solved_value(child, axis, Kind::Max)),
        }
    }

    fn solve_even_share(&self, i: NodeI, axis: Axis) -> f32 {
        #[derive(Debug)]
        struct FillChild {
            content: Option<f32>,
            min: Option<f32>,
            max: Option<f32>,
            capped_by_content: bool,
            frozen: bool,
        }

        impl FillChild {
            #[inline(always)]
            fn size(&self, share: f32) -> f32 {
                let raw = match self.content {
                    Some(content) if self.capped_by_content => content.min(share),
                    Some(content) => content.max(share),
                    None => share,
                };
                clamp(raw, self.min, self.max)
            }
        }

        let own = inner(self.get_solved_value(i, axis, Kind::Final), self.padding(i, axis));
        let mut budget = (own - self.stack_gaps(i, axis)).max(0.0);

        with_arena(|arena| {
            let mut fills = BumpVec::new_in(arena);
            for_each_layout_child!(self, i, child, {
                let node = &self.layout.params[child];
                if node.free_placement {
                    // Nothing
                } else if self.any_size_is_fill(child, axis) {
                    let content = not_fill(node.size[axis]).then(|| self.get_solved_value(child, axis, Kind::Regular));
                    fills.push(FillChild {
                        content,
                        min: self.try_get_solved_value(child, axis, Kind::Min),
                        max: self.try_get_solved_value(child, axis, Kind::Max),
                        capped_by_content: content.is_some() && node.max_size[axis] == Some(Size::Fill),
                        frozen: false,
                    });
                } else {
                    budget -= self.get_solved_value(child, axis, Kind::Final);
                }
            });

            let budget = budget.max(0.0);
            let mut n_frozen = 0;
            let mut frozen_total = 0.0;
            let mut share = budget / fills.len().max(1) as f32;

            while n_frozen < fills.len() {
                share = (budget - frozen_total) / (fills.len() - n_frozen) as f32;

                let mut total_violation = 0.0;
                for child in fills.iter() {
                    if ! child.frozen {
                        total_violation += child.size(share) - share;
                    }
                }
                if total_violation.abs() < 1e-9 {
                    break;
                }

                let mut froze_any = false;
                for child in fills.iter_mut() {
                    let size = child.size(share);
                    let stuck = match total_violation > 0.0 {
                        true => size - share > 1e-9,
                        false => size - share < -1e-9,
                    };
                    if ! child.frozen && stuck {
                        child.frozen = true;
                        n_frozen += 1;
                        frozen_total += size;
                        froze_any = true;
                    }
                }
                if ! froze_any {
                    break;
                }
            }


            share
        })
    }

    fn solve_grid_cells(&mut self, i: NodeI) -> f32 {
        let ChildrenLayout::Grid { columns, spacing_x, spacing_y, flow } = self.layout.params[i].children_layout else {
            unreachable!();
        };
        let main = flow.main_axis;
        let n_main = match columns {
            MainAxisCellSize::Count(n) => (n as usize).max(1),
            MainAxisCellSize::Width(w) => {
                let spacing = self.pixels_to_frac(Xy::new(spacing_x, spacing_y)[main], main);
                let width = self.pixels_to_frac(w, main);
                let inner_main = inner(self.get_solved_value(i, main, Kind::Final), self.padding(i, main));
                ((inner_main + spacing) / (width + spacing)).floor().max(1.0) as usize
            }
        };
        self.assign_grid_cells(i, n_main, flow);
        n_main as f32
    }

    fn assign_grid_cells(&mut self, i: NodeI, n_main: usize, flow: GridFlow) {
        let n_lines = with_arena(|arena| {
            let mut occupied = BumpVec::new_in(arena);
            let mut cursor_line = 0;

            for_each_layout_child!(self, i, child, {
                if ! self.layout.params[child].free_placement {
                    let span = self.layout.params[child].grid_span;
                    let span_pos = (span[flow.main_axis].max(1) as usize).min(n_main);
                    let span_line = span[flow.main_axis.other()].max(1) as usize;

                    let mut line = if flow.backfill { 0 } else { cursor_line };
                    let pos = 'search: loop {
                        for pos in 0..=n_main - span_pos {
                            if grid_area_is_free(&occupied, n_main, line, pos, span_line, span_pos) {
                                break 'search pos;
                            }
                        }
                        line += 1;
                    };

                    let needed = (line + span_line) * n_main;
                    if occupied.len() < needed {
                        occupied.resize(needed, false);
                    }
                    for l in line..line + span_line {
                        occupied[l * n_main + pos..l * n_main + pos + span_pos].fill(true);
                    }
                    if ! flow.backfill {
                        cursor_line = line;
                    }

                    let mut cell = Xy::new(0, 0);
                    cell[flow.main_axis] = pos as u16;
                    cell[flow.main_axis.other()] = line as u16;
                    self.layout.solver_data[child].grid_cell = cell;
                }
            });

            occupied.len() / n_main
        });

        let mut lines = Xy::new(0, 0);
        lines[flow.main_axis] = n_main as u16;
        lines[flow.main_axis.other()] = n_lines as u16;
        self.layout.solver_data[i].grid_lines = lines;

        for axis in [X, Y] {
            let reversed = match axis {
                X => flow.x_fill_direction == Direction::RightToLeft,
                Y => flow.y_fill_direction == Direction::RightToLeft,
            };
            if reversed {
                for_each_layout_child!(self, i, child, {
                    if ! self.layout.params[child].free_placement {
                        let span = self.layout.params[child].grid_span[axis].max(1);
                        let cell = &mut self.layout.solver_data[child].grid_cell[axis];
                        *cell = lines[axis] - span - *cell;
                    }
                });
            }
        }
    }

    #[inline(always)]
    fn padding(&self, i: NodeI, axis: Axis) -> f32 {
        self.pixels_to_frac(self.layout.params[i].padding[axis], axis)
    }

    #[inline(always)]
    fn stack_gaps(&self, i: NodeI, axis: Axis) -> f32 {
        let ChildrenLayout::Stack { axis: stack_axis, spacing, .. } = self.layout.params[i].children_layout else {
            return 0.0;
        };
        if stack_axis != axis {
            return 0.0;
        }
        let n = self.layout.solver_data[i].n_placed_children as f32;
        self.pixels_to_frac(spacing, axis) * (n - 1.0).max(0.0)
    }
}

fn grid_area_is_free(occupied: &[bool], n_main: usize, line: usize, pos: usize, span_line: usize, span_pos: usize) -> bool {
    for l in line..line + span_line {
        for p in pos..pos + span_pos {
            if occupied.get(l * n_main + p) == Some(&true) {
                return false;
            }
        }
    }
    true
}

impl Ui {
    pub(crate) fn layout_calculate_sizes(&mut self) {
        let window_size = self.sys.logical_size();
        let System { nodes, renderer, size, .. } = &mut self.sys;
        nodes.solve_layout(window_size, &mut |node, width| {
            measure_text(&mut renderer.text, node.text_i.as_ref().unwrap(), width, *size)
        });
        self.write_sizes_recursive(ROOT_I);
    }

    fn write_sizes_recursive(&mut self, i: NodeI) {
        let layout = &self.sys.nodes.layout;
        let size = layout.size(i);
        let cell = layout.grid_cell(i);
        let lines = layout.grid_lines(i);
        let node = &mut self.sys.nodes[i];
        node.l2_solved[X][SizeType::Final as usize] = Some(size.x);
        node.l2_solved[Y][SizeType::Final as usize] = Some(size.y);
        node.grid_element_column_i = cell.x;
        node.grid_element_row_i = cell.y;
        node.grid_n_columns = lines.x;
        node.grid_n_rows = lines.y;

        for_each_child!(self, i, child, {
            self.write_sizes_recursive(child);
        });
    }
}

const TEXT_WIDTH_TOLERANCE: f32 = 0.05;

fn measure_text(text: &mut keru_draw::Text, text_i: &TextI, width: Option<f32>, window: Xy<f32>) -> Xy<f32> {
    match text_i {
        TextI::TextBox(handle) => {
            let text_box = text.get_text_box_mut(handle);

            let widths = text_box.content_widths();
            let unwrapped = widths.max.max(widths.min);
            let width = match width {
                Some(width) => (width * window[X]).clamp(widths.min, unwrapped),
                None => unwrapped,
            };

            text_box.set_width(width);
            let height = text_box.layout().height();

            Xy::new((width + TEXT_WIDTH_TOLERANCE) / window[X], height / window[Y])
        }
        TextI::TextEdit(handle) => {
            let text_edit = text.get_text_edit_mut(handle);
            let height = if text_edit.single_line() {
                match text_edit.layout().lines().next() {
                    Some(first_line) => first_line.metrics().line_height,
                    // In this empty case, we could probably get the height by looking at the metrics default font, but it's kind of complicated.
                    None => 0.0, 
                }
            } else {
                0.0
            };
            // A text edit can be scrolled, so it never asks for any width.
            Xy::new(0.0, height / window[Y])
        }
    }
}
