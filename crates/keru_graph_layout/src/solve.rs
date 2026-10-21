use crate::*;
use crate::Axis::{X, Y};

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
    pub padding: Xy<f32>,
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
}

impl LayoutParams {
    pub const DEFAULT: LayoutParams = LayoutParams {
        size: Xy { x: Size::FitContent, y: Size::FitContent },
        min_size: Xy { x: None, y: None },
        max_size: Xy { x: None, y: None },
        padding: Xy { x: 0.0, y: 0.0 },
        children_layout: ChildrenLayout::Free,
        free_placement: false,
        grid_span: Xy { x: 1, y: 1 },
        text: TextContent::None,
        image: None,
        disabled: false,
        position: Xy { x: Pos::Center, y: Pos::Center },
        anchor: Xy { x: Anchor::Start, y: Anchor::Start },
        children_origin_flipped: Xy { x: false, y: false },
    };
}

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
pub(crate) const DECLARED: [Kind; 3] = [Kind::Regular, Kind::Min, Kind::Max];
pub(crate) const N_SLOTS: usize = 2 * N_KINDS;

#[inline(always)]
pub(crate) const fn slot_index(axis: Axis, kind: Kind) -> usize {
    axis as usize * N_KINDS + kind as usize
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Slot {
    pub node: NodeI,
    pub axis: Axis,
    pub kind: Kind,
}

/// The state of each slot takes 2 bits in `SolverData::states`.
const UNSOLVED: u32 = 0;
/// The slot is solving the slots that it reads. Reaching it again means that there is a cycle.
const IN_PROGRESS: u32 = 1;
const SOLVED: u32 = 2;

#[derive(Debug, Clone, Copy)]
pub(crate) struct SolverData {
    /// The value of every slot, by `slot_index`. Only the slots that exist are used.
    pub values: [f32; N_SLOTS],
    /// The state of every slot, 2 bits each, by `slot_index`.
    pub states: u32,
    /// How many children aren't free-placement.
    pub n_placed_children: u32,
    /// The cell of this node in its parent's grid.
    pub grid_cell: Xy<u16>,
    /// The number of columns and rows, if this node is a grid.
    pub grid_lines: Xy<u16>,
}

const EMPTY_VALUES: [f32; N_SLOTS] = {
    // A Max slot that is read before it is solved, in a cycle, means "no max".
    let mut values = [0.0; N_SLOTS];
    values[slot_index(X, Kind::Max)] = f32::INFINITY;
    values[slot_index(Y, Kind::Max)] = f32::INFINITY;
    values
};

pub(crate) const EMPTY_SOLVER_DATA: SolverData = SolverData {
    values: EMPTY_VALUES,
    states: 0,
    n_placed_children: 0,
    grid_cell: Xy { x: 0, y: 0 },
    grid_lines: Xy { x: 0, y: 0 },
};


impl Size {
    #[inline(always)]
    pub(crate) fn not_fill(&self) -> bool {
        *self != Size::Fill
    }
    
    #[inline(always)]
    pub(crate) fn is_flex(&self) -> bool {
        matches!(self, Size::Frac(_) | Size::Fill)
    }
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
    pub n_deferred_solves: usize,
}

impl LayoutTree {
    pub fn new() -> Self {
        LayoutTree {
            params: NodeVec(Vec::with_capacity(100)),
            window_size: Xy::new(0.0, 0.0),
            solver_data: NodeVec(Vec::with_capacity(100)),
            n_deferred_solves: 0,
        }
    }

    pub fn size(&self, i: NodeI) -> Xy<f32> {
        let values = &self.solver_data[i].values;
        Xy::new(values[slot_index(X, Kind::Final)], values[slot_index(Y, Kind::Final)])
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
        axis == Y && self.layout.params[i].text == TextContent::Box && ! matches!(self.layout.params[i].size[X], Size::AspectRatio(_))
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

impl Nodes {
    pub(crate) fn reset_solver_data(&mut self) {
        self.layout.solver_data.0.clear();
        self.layout.solver_data.0.resize(self.layout.params.0.len(), EMPTY_SOLVER_DATA);
        self.layout.n_deferred_solves = 0;
        for i in 1..self.layout.params.0.len() {
            let params = &self.layout.params.0[i];
            if ! params.disabled && ! params.free_placement {
                let parent = self.links.0[i].parent;
                self.layout.solver_data[parent].n_placed_children += 1;
            }
        }
    }

    #[inline(always)]
    fn grid_main_axis(&self, i: NodeI) -> Option<Axis> {
        match self.layout.params[i].children_layout {
            ChildrenLayout::Grid { flow, .. } => Some(flow.main_axis),
            _ => None,
        }
    }

    fn solve_all_dependencies(&mut self, slot: Slot, measure_text: &mut MeasureText) {
        let Slot { node: i, axis, kind } = slot;
        match kind {
            Kind::Final => {
                for kind in DECLARED {
                    if self.declared(i, axis, kind).is_some() {
                        self.solve_slot(i, axis, kind, measure_text);
                    }
                }
            }
            Kind::EvenShare => {
                self.solve_slot(i, axis, Kind::Final, measure_text);
                for_each_layout_child!(self, i, child, {
                    if self.layout.params[child].free_placement {
                        // Nothing
                    } else if self.any_size_is_fill(child, axis) {
                        for kind in DECLARED {
                            if let Some(size) = self.declared(child, axis, kind) && size.not_fill() {
                                self.solve_slot(child, axis, kind, measure_text);
                            }
                        }
                    } else {
                        self.solve_slot(child, axis, Kind::Final, measure_text);
                    }
                });
            }
            Kind::GridCells => {
                if matches!(self.layout.params[i].children_layout, ChildrenLayout::Grid { columns: MainAxisCellSize::Width(_), .. }) {
                    self.solve_slot(i, axis, Kind::Final, measure_text);
                }
            }
            declared => match self.declared(i, axis, declared).unwrap() {
                Size::Pixels(_) => {}
                _ if i == ROOT_I && kind == Kind::Regular => {}
                Size::AspectRatio(_) => self.solve_slot(i, axis.other(), Kind::Final, measure_text),
                Size::Frac(_) => self.read_available_from_parent(slot, measure_text),
                Size::Fill if self.is_stacked(i, axis) => {
                    let parent = self.links[i].parent;
                    if self.has_even_share(parent, axis) {
                        self.solve_slot(parent, axis, Kind::EvenShare, measure_text);
                    }
                }
                Size::Fill => self.read_available_from_parent(slot, measure_text),
                Size::FitContent => {
                    for_each_layout_child!(self, i, child, {
                        if ! self.layout.params[child].free_placement {
                            for kind in DECLARED {
                                if let Some(size) = self.declared(child, axis, kind) && ! size.is_flex() {
                                    self.solve_slot(child, axis, kind, measure_text);
                                }
                            }
                        }
                    });
                    if axis == Y && self.text_wraps(i, Y) {
                        self.solve_slot(i, X, Kind::Final, measure_text);
                    }
                    if let Some(main) = self.grid_main_axis(i) {
                        self.solve_slot(i, main, Kind::GridCells, measure_text);
                    }
                }
            },
        }
    }

    /// Solves what `available_from_parent` reads.
    #[inline(always)]
    fn read_available_from_parent(&mut self, slot: Slot, measure_text: &mut MeasureText) {
        let i = slot.node;
        if i == ROOT_I {
            return;
        }
        let parent = self.links[i].parent;
        self.solve_slot(parent, slot.axis, Kind::Final, measure_text);
        if ! self.layout.params[i].free_placement && let Some(main) = self.grid_main_axis(parent) {
            self.solve_slot(parent, main, Kind::GridCells, measure_text);
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

/// Measures the text of a node. The width and the result are in fractions of the window size.
pub type MeasureText<'a> = dyn FnMut(NodeI, Option<f32>) -> Xy<f32> + 'a;

impl Nodes {
    pub(crate) fn solve_layout(&mut self, window_size: Xy<f32>, measure_text: &mut MeasureText) {
        self.layout.window_size = window_size;
        self.reset_solver_data();
        self.solve(measure_text);
    }

    fn solve(&mut self, measure_text: &mut MeasureText) {
        for i in 0..self.layout.params.0.len() {
            let node = NodeI::from_usize(i);
            if self.layout.params[node].disabled {
                continue;
            }
            self.solve_slot(node, X, Kind::Final, measure_text);
            self.solve_slot(node, Y, Kind::Final, measure_text);
            if let Some(main) = self.grid_main_axis(node) {
                self.solve_slot(node, main, Kind::GridCells, measure_text);
            }
        }
    }

    fn solve_slot(&mut self, node: NodeI, axis: Axis, kind: Kind, measure_text: &mut MeasureText) {
        let slot = Slot { node, axis, kind };
        let k = slot_index(slot.axis, slot.kind);
        match (self.layout.solver_data[slot.node].states >> (2 * k)) & 3 {
            SOLVED => return,
            IN_PROGRESS => {
                self.layout.n_deferred_solves += 1;
                return;
            }
            _ => {}
        }
        self.layout.solver_data[slot.node].states |= IN_PROGRESS << (2 * k);
        self.solve_all_dependencies(slot, measure_text);
        let size = self.solve_slot_inner(slot, measure_text);
        let data = &mut self.layout.solver_data[slot.node];
        data.values[k] = size;
        data.states = (data.states & ! (3 << (2 * k))) | (SOLVED << (2 * k));
        if slot.kind == Kind::Final {
            self.rects[slot.node][slot.axis] = [0.0, size];
        }
    }

    #[inline(always)]
    fn get_solved_value(&self, i: NodeI, axis: Axis, kind: Kind) -> f32 {
        let k = slot_index(axis, kind);
        let data = &self.layout.solver_data[i];
        debug_assert!((data.states >> (2 * k)) & 3 != UNSOLVED, "Slot {axis:?} {kind:?} of node {i:?} read before it is solved.");
        data.values[k]
    }

    #[inline(always)]
    fn pixels_to_frac(&self, pixels: f32, axis: Axis) -> f32 {
        pixels / self.layout.window_size[axis]
    }

    fn solve_slot_inner(&mut self, slot: Slot, measure_text: &mut MeasureText) -> f32 {
        let i = slot.node;
        let axis = slot.axis;
        match slot.kind {
            Kind::Regular if i == ROOT_I => 1.0,
            Kind::Final => {
                let (min, max) = self.solved_bounds(i, axis, |_| false);
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
                Size::FitContent => {
                    let is_max_bound = declared == Kind::Max;
                    self.solve_fitcontent(i, axis, is_max_bound, measure_text)
                },
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

    #[inline(always)]
    fn solved_bounds(&self, i: NodeI, axis: Axis, ignore_bounds_rule: impl Fn(Size) -> bool) -> (Option<f32>, Option<f32>) {
        let node = &self.layout.params[i];
        let min = if let Some(bound) = node.min_size[axis] && ! ignore_bounds_rule(bound) {
            Some(self.get_solved_value(i, axis, Kind::Min))
        } else { None };
        let max = if let Some(bound) = node.max_size[axis] && ! ignore_bounds_rule(bound) {
            Some(self.get_solved_value(i, axis, Kind::Max))
        } else { None };
        (min, max)
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

    fn solve_fitcontent(&mut self, i: NodeI, axis: Axis, is_max_bound: bool, measure_text: &mut MeasureText) -> f32 {
        let (children, gaps) = match self.layout.params[i].children_layout {
            ChildrenLayout::Grid { columns, spacing_x, spacing_y, flow } => {
                let spacing = self.pixels_to_frac(Xy::new(spacing_x, spacing_y)[axis], axis);
                let n = self.layout.solver_data[i].grid_lines[axis] as f32;
                let cell = match columns {
                    MainAxisCellSize::Width(w) if axis == flow.main_axis => self.pixels_to_frac(w, axis),
                    _ => {
                        let mut cell = 0.0f32;
                        for_each_layout_child!(self, i, child, {
                            if ! self.layout.params[child].free_placement {
                                let span = self.layout.params[child].grid_span[axis].max(1) as f32;
                                cell = cell.max((self.pre_grow_size(child, axis, is_max_bound) - (span - 1.0) * spacing) / span);
                            }
                        });
                        cell
                    }
                };
                ((n * cell + spacing * (n - 1.0)).max(0.0), 0.0)
            }
            ChildrenLayout::Stack { axis: stack_axis, .. } if stack_axis == axis => {
                (self.fit_stack_main(i, axis, is_max_bound), self.stack_gaps(i, axis))
            }
            ChildrenLayout::Stack { .. } | ChildrenLayout::Free => {
                let mut biggest = 0.0f32;
                for_each_layout_child!(self, i, child, {
                    if ! self.layout.params[child].free_placement {
                        biggest = biggest.max(self.pre_grow_size(child, axis, is_max_bound));
                    }
                });
                (biggest, 0.0)
            }
        };

        let mut own_content = match self.layout.params[i].text {
            TextContent::None => 0.0,
            _ if self.text_wraps(i, axis) => {
                let width = inner(self.get_solved_value(i, X, Kind::Final), self.padding(i, X));
                measure_text(i, Some(width))[Y]
            }
            _ => measure_text(i, None)[axis],
        };
        if let Some(image) = self.layout.params[i].image {
            own_content = own_content.max(self.pixels_to_frac(image[axis], axis));
        }

        children.max(own_content) + gaps + 2.0 * self.padding(i, axis)
    }

    fn fit_stack_main(&self, i: NodeI, axis: Axis, at_max: bool) -> f32 {
        let mut estimate = 0.0f32;
        for _ in 0..MAX_FIT_STACK_STEPS {
            let mut fixed = 0.0f32;
            let mut free_frac = 0.0f32;
            let mut has_frac = false;
            for_each_layout_child!(self, i, child, {
                if self.layout.params[child].free_placement {
                    // Nothing
                } else if let Size::Frac(frac) = self.layout.params[child].size[axis] {
                    has_frac = true;
                    let wanted = estimate * frac;
                    let (min, max) = self.solved_bounds(child, axis, |bound| bound.is_flex());
                    let allowed = clamp(wanted, min, max);
                    match allowed == wanted {
                        true => free_frac += frac,
                        false => fixed += allowed,
                    }
                } else {
                    fixed += self.pre_grow_size(child, axis, at_max);
                }
            });

            if ! has_frac || fixed == f32::INFINITY {
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
    fn pre_grow_size(&self, child: NodeI, axis: Axis, at_max: bool) -> f32 {
        let size = match self.layout.params[child].size[axis] {
            Size::Fill if at_max => f32::INFINITY,
            size if size.is_flex() => 0.0,
            _ => self.get_solved_value(child, axis, Kind::Regular),
        };
        let (min, max) = self.solved_bounds(child, axis, |bound| bound.is_flex());
        clamp(size, min, max)
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
                    let content = if node.size[axis].not_fill() {
                        Some(self.get_solved_value(child, axis, Kind::Regular))
                    } else { None };
                    let (min, max) = self.solved_bounds(child, axis, |bound| bound == Size::Fill);
                    fills.push(FillChild {
                        content,
                        min,
                        max,
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

