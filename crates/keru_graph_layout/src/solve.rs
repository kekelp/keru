use crate::*;
use crate::Axis::{X, Y};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Regular,
    Min,
    Max,
    Final,
    ContentMin,
    ContentMax,
    /// Has to be the last element of the enum!
    EvenShare,
}

const N_KINDS: usize = 7;
/// The kinds that have a slot on each axis.
const N_AXIS_KINDS: usize = N_KINDS - 1;
/// The kinds that have a `Size`. For `ContentMin` and `ContentMax`, the size is always `FitContent`.
pub(crate) const DECLARED: [Kind; 5] = [Kind::Regular, Kind::Min, Kind::Max, Kind::ContentMin, Kind::ContentMax];
pub(crate) const N_SLOTS: usize = 2 * N_AXIS_KINDS + 1;

#[inline(always)]
pub(crate) const fn slot_index(axis: Axis, kind: Kind) -> usize {
    match kind {
        Kind::EvenShare => 2 * N_AXIS_KINDS,
        _ => axis as usize * N_AXIS_KINDS + kind as usize,
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Slot<N> {
    pub node: N,
    pub axis: Axis,
    pub kind: Kind,
}

/// The state of each slot takes 2 bits in `NodeFlags`.
const UNSOLVED: u32 = 0;
/// The slot is solving the slots that it reads. Reaching it again means that there is a cycle.
const IN_PROGRESS: u32 = 1;
const SOLVED: u32 = 2;

/// The resolve flags of each axis come after the slot states.
const RESOLVED_FLAGS_SHIFT: u32 = 2 * N_SLOTS as u32;
const RESOLVED_FLAGS_BITS: u32 = 3;
const _: () = assert!(RESOLVED_FLAGS_SHIFT + 2 * RESOLVED_FLAGS_BITS <= 32, "The slot states and the resolve flags don't fit in NodeFlags");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct NodeFlags(u32);

impl NodeFlags {
    pub(crate) const EMPTY: NodeFlags = NodeFlags(0);

    /// The `FitContent` size becomes `Fill`, and the content is a lower bound (`Kind::ContentMin`).
    pub(crate) const FIT_IS_FILL: u32 = 1 << 0;
    /// The content is also an upper bound (`Kind::ContentMax`).
    pub(crate) const CONTENT_MAX: u32 = 1 << 1;
    /// The `Fill` min and max sizes don't apply, because the node is in a stack that has no space to share.
    pub(crate) const FILL_BOUNDS_DROPPED: u32 = 1 << 2;

    #[inline(always)]
    pub(crate) fn check_state(self, axis: Axis, kind: Kind) -> u32 {
        let k = slot_index(axis, kind);
        (self.0 >> (2 * k)) & 3
    }

    #[inline(always)]
    pub(crate) fn set_state(&mut self, axis: Axis, kind: Kind, state: u32) {
        let k = slot_index(axis, kind);
        self.0 = (self.0 & ! (3 << (2 * k))) | (state << (2 * k));
    }

    #[inline(always)]
    pub(crate) fn check_flag(self, axis: Axis, flag: u32) -> bool {
        self.0 & (flag << (RESOLVED_FLAGS_SHIFT + RESOLVED_FLAGS_BITS * axis as u32)) != 0
    }

    #[inline(always)]
    pub(crate) fn set_flag(&mut self, axis: Axis, flag: u32) {
        self.0 |= flag << (RESOLVED_FLAGS_SHIFT + RESOLVED_FLAGS_BITS * axis as u32);
    }
}

const EMPTY_VALUES: [f32; N_SLOTS] = {
    // A Max slot that is read before it is solved, in a cycle, means "no max".
    let mut values = [0.0; N_SLOTS];
    values[slot_index(X, Kind::Max)] = f32::INFINITY;
    values[slot_index(Y, Kind::Max)] = f32::INFINITY;
    values[slot_index(X, Kind::ContentMax)] = f32::INFINITY;
    values[slot_index(Y, Kind::ContentMax)] = f32::INFINITY;
    values
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
    ($solver:expr, $i:expr, $child:ident, $body:block) => {{
        let mut next = $solver.first_child($i);
        while let Some($child) = next {
            next = $solver.next_sibling($child);
            if ! $solver.params($child).disabled {
                $body
            }
        }
    }};
}

/// Per-node data used by the layout solver.
// todo: squeeze out 4 more bytes to get it to 64.
// This could be done by merging the EvenShare slot with the grid data, since a node can't be a stack and a grid at the same time.
// note also that it would drop far below if it didn't have the MinContent and MaxContent slots, which are only needed for the special fill-fitcontent interaction.
#[derive(Debug, Clone, Copy)]
pub struct LayoutData {
    /// The value of every slot, by `slot_index`. Only the slots that exist are used.
    pub(crate) values: [f32; N_SLOTS],
    /// The state of every slot and the result of `resolve_sizes`.
    pub(crate) flags: NodeFlags,
    /// How many children aren't free-placement.
    pub(crate) n_placed_children: u32,
    /// The cell of this node in its parent's grid.
    pub(crate) grid_cell: Xy<u16>,
    /// The number of columns and rows, if this node is a grid.
    pub(crate) grid_lines: Xy<u16>,
}

impl LayoutData {
    pub const EMPTY: LayoutData = LayoutData {
        values: EMPTY_VALUES,
        flags: NodeFlags::EMPTY,
        n_placed_children: 0,
        grid_cell: Xy { x: 0, y: 0 },
        grid_lines: Xy { x: 0, y: 0 },
    };

    pub(crate) fn size(&self) -> Xy<f32> {
        let values = &self.values;
        Xy::new(values[slot_index(X, Kind::Final)], values[slot_index(Y, Kind::Final)])
    }
}

impl Default for LayoutData {
    fn default() -> Self {
        Self::EMPTY
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

pub(crate) trait Solve: TreeAdapter {
    #[inline(always)]
    fn tree_traversal_next(&self, i: Self::NodeId) -> Option<Self::NodeId> {
        if ! self.params(i).disabled && let Some(child) = self.first_child(i) {
            return Some(child);
        }
        let root = self.root();
        let mut i = i;
        loop {
            if i == root {
                return None;
            }
            if let Some(sibling) = self.next_sibling(i) {
                return Some(sibling);
            }
            i = self.parent(i);
        }
    }

    #[inline(always)]
    fn read_size(&self, i: Self::NodeId, axis: Axis, kind: Kind) -> Option<Size> {
        let node = self.params(i);
        let flags = self.layout_data(i).flags;
        let bound = |bound: Option<Size>| match bound {
            Some(Size::Fill) if flags.check_flag(axis, NodeFlags::FILL_BOUNDS_DROPPED) => None,
            bound => bound,
        };
        match kind {
            Kind::Regular if flags.check_flag(axis, NodeFlags::FIT_IS_FILL) => Some(Size::Fill),
            Kind::Regular => Some(node.size[axis]),
            Kind::Min => bound(node.min_size[axis]),
            Kind::Max => bound(node.max_size[axis]),
            Kind::ContentMin => flags.check_flag(axis, NodeFlags::FIT_IS_FILL).then_some(Size::FitContent),
            Kind::ContentMax => flags.check_flag(axis, NodeFlags::CONTENT_MAX).then_some(Size::FitContent),
            Kind::Final | Kind::EvenShare => unreachable!(),
        }
    }

    #[inline(always)]
    fn any_size_is_fill(&self, i: Self::NodeId, axis: Axis) -> bool {
        DECLARED.iter().any(|&kind| self.read_size(i, axis, kind) == Some(Size::Fill))
    }

    #[inline(always)]
    fn text_wraps(&self, i: Self::NodeId, axis: Axis) -> bool {
        axis == Y && self.params(i).text == TextContent::Box && ! matches!(self.params(i).size[X], Size::AspectRatio(_))
    }

    #[inline(always)]
    fn stack_axis(&self, i: Self::NodeId) -> Option<Axis> {
        match self.params(i).children_layout {
            ChildrenLayout::Stack { axis, .. } => Some(axis),
            _ => None,
        }
    }

    #[inline(always)]
    fn is_stacked(&self, i: Self::NodeId, axis: Axis) -> bool {
        let parent = self.parent(i);
        self.stack_axis(parent) == Some(axis) && ! self.params(i).free_placement
    }

    #[inline(always)]
    fn has_even_share(&self, i: Self::NodeId, axis: Axis) -> bool {
        self.stack_axis(i) == Some(axis) && self.read_size(i, axis, Kind::Regular) != Some(Size::FitContent)
    }

    fn prepare(&mut self, i: Self::NodeId) {
        *self.layout_data_mut(i) = LayoutData::EMPTY;
        if self.params(i).disabled {
            return;
        }
        let mut n_placed_children = 0;
        let mut next = self.first_child(i);
        while let Some(child) = next {
            next = self.next_sibling(child);
            self.prepare(child);
            let params = self.params(child);
            if ! params.disabled && ! params.free_placement {
                n_placed_children += 1;
            }
        }
        self.layout_data_mut(i).n_placed_children = n_placed_children;

        if let ChildrenLayout::Grid { columns: MainAxisCellSize::Count(n), flow, .. } = self.params(i).children_layout {
            self.assign_grid_cells(i, (n as usize).max(1), flow);
        }
        self.resolve_sizes(i);
    }

    #[inline(always)]
    fn width_grid_main_axis(&self, i: Self::NodeId) -> Option<Axis> {
        match self.params(i).children_layout {
            ChildrenLayout::Grid { columns: MainAxisCellSize::Width(_), flow, .. } => Some(flow.main_axis),
            _ => None,
        }
    }

    fn solve_all_dependencies(&mut self, slot: Slot<Self::NodeId>) {
        let Slot { node: i, axis, kind } = slot;
        match kind {
            Kind::Final => {
                for kind in DECLARED {
                    if self.read_size(i, axis, kind).is_some() {
                        self.solve_slot(i, axis, kind);
                    }
                }
            }
            Kind::EvenShare => {
                self.solve_slot(i, axis, Kind::Final);
                for_each_layout_child!(self, i, child, {
                    if self.params(child).free_placement {
                        // Nothing
                    } else if self.any_size_is_fill(child, axis) {
                        for kind in DECLARED {
                            if let Some(size) = self.read_size(child, axis, kind) && size.not_fill() {
                                self.solve_slot(child, axis, kind);
                            }
                        }
                    } else {
                        self.solve_slot(child, axis, Kind::Final);
                    }
                });
            }
            declared => match self.read_size(i, axis, declared).unwrap() {
                Size::Pixels(_) => {}
                _ if i == self.root() && kind == Kind::Regular => {}
                Size::AspectRatio(_) => self.solve_slot(i, axis.other(), Kind::Final),
                Size::Frac(_) => self.read_available_from_parent(slot),
                Size::Fill if self.is_stacked(i, axis) => {
                    let parent = self.parent(i);
                    if self.has_even_share(parent, axis) {
                        self.solve_slot(parent, axis, Kind::EvenShare);
                    }
                }
                Size::Fill => self.read_available_from_parent(slot),
                Size::FitContent => {
                    for_each_layout_child!(self, i, child, {
                        if ! self.params(child).free_placement {
                            for kind in DECLARED {
                                if let Some(size) = self.read_size(child, axis, kind) && ! size.is_flex() {
                                    self.solve_slot(child, axis, kind);
                                }
                            }
                        }
                    });
                    if axis == Y && self.text_wraps(i, Y) {
                        self.solve_slot(i, X, Kind::Final);
                    }

                    if let Some(main) = self.width_grid_main_axis(i) && main != axis {
                        self.solve_slot(i, main, Kind::Final);
                    }
                }
            },
        }
    }

    /// Solves what `available_from_parent` reads.
    #[inline(always)]
    fn read_available_from_parent(&mut self, slot: Slot<Self::NodeId>) {
        let i = slot.node;
        if i == self.root() {
            return;
        }
        let parent = self.parent(i);
        self.solve_slot(parent, slot.axis, Kind::Final);
        if ! self.params(i).free_placement && let Some(main) = self.width_grid_main_axis(parent) {
            self.solve_slot(parent, main, Kind::Final);
        }
    }

    fn measure_sizes(&mut self) {
        reset_arena();
        self.prepare(self.root());
        self.solve();
    }

    fn solve(&mut self) {
        let mut next = Some(self.root());
        while let Some(node) = next {
            next = self.tree_traversal_next(node);
            if self.params(node).disabled {
                continue;
            }
            self.solve_slot(node, X, Kind::Final);
            self.solve_slot(node, Y, Kind::Final);
        }
    }

    fn solve_slot(&mut self, node: Self::NodeId, axis: Axis, kind: Kind) {
        let slot = Slot { node, axis, kind };
        match self.layout_data(node).flags.check_state(axis, kind) {
            SOLVED | IN_PROGRESS => return,
            _ => {}
        }
        self.layout_data_mut(node).flags.set_state(axis, kind, IN_PROGRESS);
        self.solve_all_dependencies(slot);
        let size = self.solve_slot_inner(slot);
        let data = self.layout_data_mut(node);
        data.values[slot_index(axis, kind)] = size;
        data.flags.set_state(axis, kind, SOLVED);
        if kind == Kind::Final && self.width_grid_main_axis(node) == Some(axis) {
            self.assign_width_grid_cells(node);
        }
    }

    #[inline(always)]
    fn get_solved_value(&self, i: Self::NodeId, axis: Axis, kind: Kind) -> f32 {
        let data = self.layout_data(i);
        debug_assert!(data.flags.check_state(axis, kind) != UNSOLVED, "Slot {axis:?} {kind:?} of node {i:?} read before it is solved.");
        data.values[slot_index(axis, kind)]
    }

    #[inline(always)]
    fn pixels_to_frac(&self, pixels: f32, axis: Axis) -> f32 {
        pixels / self.window_size()[axis]
    }

    fn solve_slot_inner(&mut self, slot: Slot<Self::NodeId>) -> f32 {
        let i = slot.node;
        let axis = slot.axis;
        match slot.kind {
            Kind::Regular if i == self.root() => 1.0,
            Kind::Final => {
                let (min, max) = self.solved_bounds(i, axis, |_| false);
                clamp(self.get_solved_value(i, axis, Kind::Regular), min, max)
            }
            Kind::EvenShare => self.solve_even_share(i, axis),
            declared => match self.read_size(i, axis, declared).unwrap() {
                Size::Pixels(pixels) => self.pixels_to_frac(pixels, axis),
                Size::AspectRatio(aspect) => {
                    let other = self.get_solved_value(i, axis.other(), Kind::Final);
                    let window_aspect = self.window_size().x / self.window_size().y;
                    match axis {
                        X => other * aspect / window_aspect,
                        Y => other * window_aspect / aspect,
                    }
                }
                Size::FitContent => {
                    let is_max_bound = matches!(declared, Kind::Max | Kind::ContentMax);
                    self.solve_fitcontent(i, axis, is_max_bound)
                },
                Size::Frac(frac) => self.available_from_parent(i, axis) * frac,
                Size::Fill if self.is_stacked(i, axis) => {
                    let parent = self.parent(i);
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
    fn solved_bounds(&self, i: Self::NodeId, axis: Axis, ignore_bounds_rule: impl Fn(Size) -> bool) -> (Option<f32>, Option<f32>) {
        let min = if let Some(bound) = self.read_size(i, axis, Kind::Min) && ! ignore_bounds_rule(bound) {
            Some(self.get_solved_value(i, axis, Kind::Min))
        } else { None };
        let max = if let Some(bound) = self.read_size(i, axis, Kind::Max) && ! ignore_bounds_rule(bound) {
            Some(self.get_solved_value(i, axis, Kind::Max))
        } else { None };
        let flags = self.layout_data(i).flags;
        let final_min = if flags.check_flag(axis, NodeFlags::FIT_IS_FILL) {
            Some(clamp(self.get_solved_value(i, axis, Kind::ContentMin), min, max))
        } else { min };
        let final_max = if flags.check_flag(axis, NodeFlags::CONTENT_MAX) {
            Some(clamp(self.get_solved_value(i, axis, Kind::ContentMax), min, max))
        } else { max };
        (final_min, final_max)
    }

    fn available_from_parent(&self, i: Self::NodeId, axis: Axis) -> f32 {
        let parent = self.parent(i);
        let inner = inner(self.get_solved_value(parent, axis, Kind::Final), self.padding(parent, axis));
        if self.params(i).free_placement {
            return inner;
        }
        match self.params(parent).children_layout {
            ChildrenLayout::Grid { spacing_x, spacing_y, .. } => {
                let spacing = self.pixels_to_frac(Xy::new(spacing_x, spacing_y)[axis], axis);
                let n = self.layout_data(parent).grid_lines[axis] as f32;
                let cell = ((inner - spacing * (n - 1.0)) / n.max(1.0)).max(0.0);
                let span = self.params(i).grid_span[axis].max(1) as f32;
                span * cell + (span - 1.0) * spacing
            }
            ChildrenLayout::Stack { axis: stack_axis, .. } if stack_axis == axis => {
                (inner - self.stack_gaps(parent, axis)).max(0.0)
            }
            _ => inner,
        }
    }

    fn solve_fitcontent(&mut self, i: Self::NodeId, axis: Axis, is_max_bound: bool) -> f32 {
        let (children, gaps) = match self.params(i).children_layout {
            ChildrenLayout::Grid { columns, spacing_x, spacing_y, flow } => {
                let spacing = self.pixels_to_frac(Xy::new(spacing_x, spacing_y)[axis], axis);
                let (n, cell) = match columns {
                    // The number of columns depends on the final size, so a grid fits one column.
                    MainAxisCellSize::Width(w) if axis == flow.main_axis => (1.0, self.pixels_to_frac(w, axis)),
                    _ => {
                        let mut cell = 0.0f32;
                        for_each_layout_child!(self, i, child, {
                            if ! self.params(child).free_placement {
                                let span = self.params(child).grid_span[axis].max(1) as f32;
                                cell = cell.max((self.pre_grow_size(child, axis, is_max_bound) - (span - 1.0) * spacing) / span);
                            }
                        });
                        (self.layout_data(i).grid_lines[axis] as f32, cell)
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
                    if ! self.params(child).free_placement {
                        biggest = biggest.max(self.pre_grow_size(child, axis, is_max_bound));
                    }
                });
                (biggest, 0.0)
            }
        };

        let mut own_content = match self.params(i).text {
            TextContent::None => 0.0,
            _ if self.text_wraps(i, axis) => {
                let width = inner(self.get_solved_value(i, X, Kind::Final), self.padding(i, X));
                self.measure_text(i, Some(width))[Y]
            }
            _ => self.measure_text(i, None)[axis],
        };
        if let Some(image) = self.params(i).image {
            own_content = own_content.max(self.pixels_to_frac(image[axis], axis));
        }

        children.max(own_content) + gaps + 2.0 * self.padding(i, axis)
    }

    fn fit_stack_main(&self, i: Self::NodeId, axis: Axis, at_max: bool) -> f32 {
        let mut estimate = 0.0f32;
        for _ in 0..MAX_FIT_STACK_STEPS {
            let mut fixed = 0.0f32;
            let mut free_frac = 0.0f32;
            let mut has_frac = false;
            for_each_layout_child!(self, i, child, {
                if self.params(child).free_placement {
                    // Nothing
                } else if let Size::Frac(frac) = self.params(child).size[axis] {
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
    fn pre_grow_size(&self, child: Self::NodeId, axis: Axis, at_max: bool) -> f32 {
        let size = match self.read_size(child, axis, Kind::Regular).unwrap() {
            Size::Fill if at_max => f32::INFINITY,
            size if size.is_flex() => 0.0,
            _ => self.get_solved_value(child, axis, Kind::Regular),
        };
        let (min, max) = self.solved_bounds(child, axis, |bound| bound.is_flex());
        clamp(size, min, max)
    }

    fn solve_even_share(&self, i: Self::NodeId, axis: Axis) -> f32 {
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
                let node = self.params(child);
                if node.free_placement {
                    // Nothing
                } else if self.any_size_is_fill(child, axis) {
                    let content = if self.read_size(child, axis, Kind::Regular) != Some(Size::Fill) {
                        Some(self.get_solved_value(child, axis, Kind::Regular))
                    } else { None };
                    let (min, max) = self.solved_bounds(child, axis, |bound| bound == Size::Fill);
                    fills.push(FillChild {
                        content,
                        min,
                        max,
                        capped_by_content: content.is_some() && self.read_size(child, axis, Kind::Max) == Some(Size::Fill),
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

    /// Assigns the cells of a grid with `MainAxisCellSize::Width` columns, from its `Final` size on the main axis.
    fn assign_width_grid_cells(&mut self, i: Self::NodeId) {
        let ChildrenLayout::Grid { columns: MainAxisCellSize::Width(w), spacing_x, spacing_y, flow } = self.params(i).children_layout else {
            unreachable!();
        };
        let main = flow.main_axis;
        let spacing = self.pixels_to_frac(Xy::new(spacing_x, spacing_y)[main], main);
        let width = self.pixels_to_frac(w, main);
        let inner_main = inner(self.get_solved_value(i, main, Kind::Final), self.padding(i, main));
        let n_main = ((inner_main + spacing) / (width + spacing)).floor().max(1.0) as usize;
        self.assign_grid_cells(i, n_main, flow);
    }

    fn assign_grid_cells(&mut self, i: Self::NodeId, n_main: usize, flow: GridFlow) {
        let n_lines = with_arena(|arena| {
            let mut occupied = BumpVec::new_in(arena);
            let mut cursor_line = 0;

            for_each_layout_child!(self, i, child, {
                if ! self.params(child).free_placement {
                    let span = self.params(child).grid_span;
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
                    self.layout_data_mut(child).grid_cell = cell;
                }
            });

            occupied.len() / n_main
        });

        let mut lines = Xy::new(0, 0);
        lines[flow.main_axis] = n_main as u16;
        lines[flow.main_axis.other()] = n_lines as u16;
        self.layout_data_mut(i).grid_lines = lines;

        for axis in [X, Y] {
            let reversed = match axis {
                X => flow.x_fill_direction == Direction::RightToLeft,
                Y => flow.y_fill_direction == Direction::RightToLeft,
            };
            if reversed {
                for_each_layout_child!(self, i, child, {
                    if ! self.params(child).free_placement {
                        let span = self.params(child).grid_span[axis].max(1);
                        let cell = &mut self.layout_data_mut(child).grid_cell[axis];
                        *cell = lines[axis] - span - *cell;
                    }
                });
            }
        }
    }

    #[inline(always)]
    fn padding(&self, i: Self::NodeId, axis: Axis) -> f32 {
        self.pixels_to_frac(self.params(i).padding[axis], axis)
    }

    #[inline(always)]
    fn stack_gaps(&self, i: Self::NodeId, axis: Axis) -> f32 {
        let ChildrenLayout::Stack { axis: stack_axis, spacing, .. } = self.params(i).children_layout else {
            return 0.0;
        };
        if stack_axis != axis {
            return 0.0;
        }
        let n = self.layout_data(i).n_placed_children as f32;
        self.pixels_to_frac(spacing, axis) * (n - 1.0).max(0.0)
    }
}

impl<T: TreeAdapter + ?Sized> Solve for T {}

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

