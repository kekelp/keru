use crate::*;
use crate::Axis::{X, Y};
use bumpalo::collections::Vec as BumpVec;

/// What `solve_layout_incremental` did. The `Fallback` variants give the reason for a full solve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncrementalRelayoutOutcome {
    /// The relayout was incremental.
    Success {
        /// Slot computations.
        computed: u32,
    },
    /// In `solve_layout_incremental_checked()`, the incremental relayout gave a different result than the full relayout. The tree keeps the result of the full relayout. This should only be possible if the tree had a structural change that wasn't registered with `register_node_change()`.
    CheckedIncrementalRelayoutMismatch,
    /// The incremental relayout fell back to a full relayout because the full layout was never solved before.
    FallbackNeverSolved,
    /// The incremental relayout fell back to a full relayout because the tree had a structural change. Structural changes are not supported by the incremental solver.
    FallbackStructuralChange,
    /// The incremental relayout fell back to a full relayout because the last full solve found a cycle. With cycles, the incremental solver can't guarantee identical results to the full solver in all cases.
    FallbackCycle,
    /// The incremental relayout fell back to a full relayout because the changes affected a large part of the tree, and the solver estimated it would be faster to fall back to a full relayout.
    FallbackTooMuchWork,
}

/// The queued slots of an incremental relayout, in one bucket per level.
pub(crate) struct Queue<'a, N> {
    arena: &'a bumpalo::Bump,
    buckets: BumpVec<'a, BumpVec<'a, Slot<N>>>,
    /// The level of the bucket that is being emptied. The slots below it are done.
    cursor: usize,
    /// The number of computed slots above which the relayout stops.
    max_computed: u32,
    /// Slot computations.
    computed: u32,
}

/// Layout data for the whole tree. The tree keeps it next to the nodes, and gives it to the solver with `TreeAdapter::layout_data`.
#[derive(Debug, Clone)]
pub struct LayoutData<N> {
    /// Nodes whose params changed, with their old params. It can contain duplicates.
    pub(crate) changes: Vec<(N, LayoutParams)>,
    /// Nodes whose text or image changed. It can contain duplicates.
    pub(crate) contents: Vec<N>,
    /// The reason for the next relayout to be a full solve: the layout was never solved, or a structural change happened. A full solve clears it.
    pub(crate) pending_full_solve: Option<IncrementalRelayoutOutcome>,
    /// The window size of the last solve.
    pub(crate) window_size: Xy<f32>,
    /// True if the last full solve found a cycle. Then every relayout is a full solve.
    pub(crate) has_cycle: bool,
    /// The number of enabled nodes at the last full solve.
    pub(crate) n_nodes: u32,
    /// During a full solve, 1 + the highest level of the dependencies of the slot that is being solved.
    pub(crate) current_level: u16,
}

impl<N> LayoutData<N> {
    pub const fn new() -> Self {
        LayoutData { changes: Vec::new(), contents: Vec::new(), pending_full_solve: Some(IncrementalRelayoutOutcome::FallbackNeverSolved), window_size: Xy::new(0.0, 0.0), has_cycle: false, n_nodes: 0, current_level: 0 }
    }

    /// Returns the `Fallback` outcome if the next relayout has to be a full solve.
    pub(crate) fn fallback(&self) -> Option<IncrementalRelayoutOutcome> {
        self.pending_full_solve.or(self.has_cycle.then_some(IncrementalRelayoutOutcome::FallbackCycle))
    }

    pub(crate) fn clear(&mut self) {
        self.changes.clear();
        self.contents.clear();
    }
}

impl<N> Default for LayoutData<N> {
    fn default() -> Self {
        Self::new()
    }
}

/// True if the change can change the dependencies between the slots.
fn is_structural(old: &LayoutParams, new: &LayoutParams) -> bool {
    use std::mem::discriminant;
    let same_kind = |a: Option<Size>, b: Option<Size>| a.map(|a| discriminant(&a)) == b.map(|b| discriminant(&b));
    let same_layout = match (old.children_layout, new.children_layout) {
        (ChildrenLayout::Free, ChildrenLayout::Free) => true,
        (ChildrenLayout::Stack { axis: a, .. }, ChildrenLayout::Stack { axis: b, .. }) => a == b,
        // The `Count` grid cells are assigned in `prepare`.
        (ChildrenLayout::Grid { columns: MainAxisCellSize::Count(a), flow: flow_a, .. }, ChildrenLayout::Grid { columns: MainAxisCellSize::Count(b), flow: flow_b, .. }) => a == b && flow_a == flow_b,
        (ChildrenLayout::Grid { columns: MainAxisCellSize::Width(_), flow: flow_a, .. }, ChildrenLayout::Grid { columns: MainAxisCellSize::Width(_), flow: flow_b, .. }) => flow_a == flow_b,
        _ => false,
    };
    old.disabled != new.disabled
        || old.free_placement != new.free_placement
        || old.grid_span != new.grid_span
        || old.text != new.text
        || old.image.is_some() != new.image.is_some()
        || ! same_layout
        || [X, Y].into_iter().any(|axis| {
            ! same_kind(Some(old.size[axis]), Some(new.size[axis]))
                || ! same_kind(old.min_size[axis], new.min_size[axis])
                || ! same_kind(old.max_size[axis], new.max_size[axis])
        })
}

pub(crate) trait Incremental: Solve {
    fn measure_sizes_incremental(&mut self) -> IncrementalRelayoutOutcome {
        let mut data = std::mem::take(self.layout_data());
        let outcome = with_arena(|arena| {
            let mut q = Queue {
                arena,
                buckets: BumpVec::new_in(arena),
                cursor: 0,
                max_computed: data.n_nodes / 2,
                computed: 0,
            };

            // Too many changes, so give up on incremental relayout.
            // Most changes queue at least one slot, so with this many changes the relayout would probably stop anyway.
            if data.changes.len() + data.contents.len() > q.max_computed as usize {
                return IncrementalRelayoutOutcome::FallbackTooMuchWork;
            }

            let window_size = self.window_size();
            let root = self.root();
            for axis in [X, Y] {
                if data.window_size[axis] != window_size[axis] {
                    self.queue(Slot { node: root, axis, kind: Kind::Regular }, &mut q);
                }
            }
            data.window_size = window_size;
            for (i, old) in &data.changes {
                self.queue_node_change(*i, old, &mut q);
            }
            for &i in &data.contents {
                self.queue_content_change(i, &mut q);
            }
            self.run_queue(&mut q)
        });
        // Give back the allocations.
        data.clear();
        *self.layout_data() = data;
        reset_arena(self.max_retained_scratch_memory());
        outcome
    }

    /// Records a change in the params of a node.
    fn incr_register_node_change(&mut self, i: Self::NodeId, old: &LayoutParams) {
        if is_structural(old, self.node_layout_params(i)) {
            self.layout_data().pending_full_solve.get_or_insert(IncrementalRelayoutOutcome::FallbackStructuralChange);
        } else {
            self.layout_data().changes.push((i, *old));
        }
    }

    /// Records that the text or the image of a node changed.
    fn incr_register_content_change(&mut self, i: Self::NodeId) {
        self.layout_data().contents.push(i);
    }

    /// Does an incremental relayout, or a full solve if the registered changes need one. If `verify` is true, it also does a full solve and compares the sizes.
    fn solve_incremental(&mut self, verify: bool) -> IncrementalRelayoutOutcome {
        let outcome = match self.layout_data().fallback() {
            Some(fallback) => fallback,
            None => self.measure_sizes_incremental(),
        };
        match outcome {
            IncrementalRelayoutOutcome::Success { .. } => match verify {
                true => {
                    if ! self.run_full_solve_and_compare() {
                        return IncrementalRelayoutOutcome::CheckedIncrementalRelayoutMismatch;
                    }
                }
                false => self.place_nodes(),
            },
            _ => self.solve_layout(),
        }
        outcome
    }

    fn run_full_solve_and_compare(&mut self) -> bool {
        let mut incremental = Vec::new();
        let mut next = Some(self.root());
        while let Some(i) = next {
            incremental.push((i, self.node_layout_data(i).values.map(f32::to_bits)));
            next = self.tree_traversal_next(i);
        }
        self.solve_layout();
        for (i, values) in incremental {
            if self.node_layout_data(i).values.map(f32::to_bits) != values {
                return false;
            }
        }
        true
    }

    /// Queues the slots that a change in the params of a node can affect.
    fn queue_node_change(&mut self, i: Self::NodeId, old: &LayoutParams, q: &mut Queue<'_, Self::NodeId>) {
        let new_params = *self.node_layout_params(i);
        let new_params = &new_params;
        // If a node went from disabled to enabled or viceversa, that's a structural change, and it won't reach this function at all.
        // Seeing it here just means that the node was disabled both before and after and can be ignored.
        if new_params.disabled {
            return;
        }

        for axis in [X, Y] {
            for kind in DECLARED_SIZES {
                let read = |params: &LayoutParams| match kind {
                    Kind::Min => params.min_size[axis],
                    Kind::Max => params.max_size[axis],
                    _ => Some(params.size[axis]),
                };
                if read(old) != read(new_params) && ! (kind == Kind::Regular && i == self.root()) {
                    self.queue(Slot { node: i, axis, kind }, q);
                }
            }
            // A `FitContent` stack reads the `Frac` numbers of its children from their params, without a slot.
            if old.size[axis] != new_params.size[axis] && matches!(new_params.size[axis], Size::Frac(_)) && i != self.root() && ! new_params.free_placement {
                let parent = self.parent(i);
                self.queue_declared(parent, axis, |size| size == Size::FitContent, q);
            }
        }

        for axis in [X, Y] {
            if old.padding[axis] != new_params.padding[axis] {
                self.queue_inner_size_readers(i, axis, q);
            }
        }

        if old.fill_weight != new_params.fill_weight && i != self.root() && ! new_params.free_placement {
            let parent = self.parent(i);
            if let Some(axis) = self.stack_axis(parent) {
                self.queue_declared(i, axis, |size| size.is_fill(), q);
                self.queue_even_share(parent, q);
            }
        }

        // Only the spacing or the cell size can change. `arrange` changes only the placement.
        match (old.children_layout, new_params.children_layout) {
            (ChildrenLayout::Stack { spacing: a, .. }, ChildrenLayout::Stack { axis, spacing: b, .. }) if a != b => {
                self.queue_inner_size_readers(i, axis, q);
            }
            (old_layout @ ChildrenLayout::Grid { .. }, new_layout @ ChildrenLayout::Grid { .. }) if old_layout != new_layout => {
                self.queue_inner_size_readers(i, X, q);
                self.queue_inner_size_readers(i, Y, q);
            }
            _ => {}
        }

        if old.image != new_params.image {
            self.queue_content_change(i, q);
        }
    }

    fn queue_content_change(&mut self, i: Self::NodeId, q: &mut Queue<'_, Self::NodeId>) {
        if self.node_layout_params(i).disabled {
            return;
        }
        for axis in [X, Y] {
            self.queue_declared(i, axis, |size| size == Size::FitContent, q);
        }
    }

    fn queue_inner_size_readers(&mut self, i: Self::NodeId, axis: Axis, q: &mut Queue<'_, Self::NodeId>) {
        let grid_main = self.width_grid_main_axis(i);
        // In a grid, the spacing and the cell size on one axis can change the cells on the other axis.
        let axes: &[Axis] = if grid_main.is_some() { &[X, Y] } else { &[axis] };
        for &axis in axes {
            self.queue_declared(i, axis, |size| size == Size::FitContent, q);
            for_each_layout_child!(self, i, child, {
                let in_stack = self.is_in_stack(child, axis);
                self.queue_declared(child, axis, |size| matches!(size, Size::Frac(_)) || (size.is_fill() && ! in_stack), q);
            });
        }
        if let Some(main) = grid_main {
            self.queue(Slot { node: i, axis: main, kind: Kind::Final }, q);
        }
        // Wrapped text reads the inner width for the height.
        if axes.contains(&X) && self.text_wraps(i, Y) {
            self.queue_declared(i, Y, |size| size == Size::FitContent, q);
        }
        self.queue_even_share(i, q);
    }

    fn queue_even_share(&mut self, i: Self::NodeId, q: &mut Queue<'_, Self::NodeId>) {
        if let Some(axis) = self.stack_axis(i) {
            self.queue(Slot { node: i, axis, kind: Kind::EvenShare }, q);
        }
    }

    fn queue(&mut self, slot: Slot<Self::NodeId>, q: &mut Queue<'_, Self::NodeId>) {
        let Slot { node, axis, kind } = slot;
        let data = self.node_layout_data_mut(node);
        if data.flags.check_state(axis, kind) != SOLVED {
            return;
        }
        data.flags.set_state(axis, kind, UNSOLVED);
        let level = data.levels[slot_index(axis, kind)] as usize;
        debug_assert!(level >= q.cursor, "Slot {slot:?} queued at level {level}, below the current level {}.", q.cursor);
        while q.buckets.len() <= level {
            q.buckets.push(BumpVec::new_in(q.arena));
        }
        q.buckets[level].push(slot);
    }

    /// Computes the queued slots in order of level, and queues the dependents of each slot that changes. The levels are exact, so each slot is computed once.
    fn run_queue(&mut self, q: &mut Queue<'_, Self::NodeId>) -> IncrementalRelayoutOutcome {
        while q.cursor < q.buckets.len() {
            let Some(slot) = q.buckets[q.cursor].pop() else {
                q.cursor += 1;
                continue;
            };
            q.computed += 1;
            if q.computed > q.max_computed {
                return IncrementalRelayoutOutcome::FallbackTooMuchWork;
            }
            let k = slot_index(slot.axis, slot.kind);
            let old = self.node_layout_data(slot.node);
            let (old_value, old_grid_lines) = (old.values[k], old.grid_lines);
            self.finish_slot(slot, q.cursor as u16);
            let new = self.node_layout_data(slot.node);
            // The cells of a width grid can change while its size stays the same.
            if new.values[k].to_bits() != old_value.to_bits() || new.grid_lines != old_grid_lines {
                self.queue_dependents(slot, q);
            }
        }
        IncrementalRelayoutOutcome::Success { computed: q.computed }
    }

    /// Queues every slot that reads `slot`. It is the reverse of `solve_all_dependencies`, and it has to stay in sync with it.
    fn queue_dependents(&mut self, slot: Slot<Self::NodeId>, q: &mut Queue<'_, Self::NodeId>) {
        let Slot { node: i, axis, kind } = slot;
        match kind {
            Kind::EvenShare => {
                for_each_layout_child!(self, i, child, {
                    if self.is_in_stack(child, axis) {
                        self.queue_declared(child, axis, |size| size.is_fill(), q);
                    }
                });
            }
            Kind::Final => {
                let other = axis.other();
                self.queue_declared(i, other, |size| matches!(size, Size::AspectRatio(_)), q);
                let grid_main = self.width_grid_main_axis(i) == Some(axis);
                if grid_main || (axis == X && self.text_wraps(i, Y)) {
                    self.queue_declared(i, other, |size| size == Size::FitContent, q);
                }
                if self.stack_axis(i) == Some(axis) {
                    self.queue(Slot { node: i, axis, kind: Kind::EvenShare }, q);
                }
                for_each_layout_child!(self, i, child, {
                    for child_axis in [X, Y] {
                        // A placed child of a width grid also reads the main axis for its other axis.
                        if child_axis == axis || (grid_main && ! self.node_layout_params(child).free_placement) {
                            let in_stack = self.is_in_stack(child, child_axis);
                            self.queue_declared(child, child_axis, |size| matches!(size, Size::Frac(_)) || (size.is_fill() && ! in_stack), q);
                        }
                    }
                });
                if i != self.root() && ! self.node_layout_params(i).free_placement {
                    let parent = self.parent(i);
                    if self.stack_axis(parent) == Some(axis) && ! self.any_size_is_fill(i, axis) {
                        self.queue(Slot { node: parent, axis, kind: Kind::EvenShare }, q);
                    }
                }
            }
            declared => {
                self.queue(Slot { node: i, axis, kind: Kind::Final }, q);
                if i != self.root() && ! self.node_layout_params(i).free_placement {
                    let size = self.read_size(i, axis, declared).unwrap();
                    let parent = self.parent(i);
                    if self.stack_axis(parent) == Some(axis) && self.any_size_is_fill(i, axis) && ! size.is_fill() {
                        self.queue(Slot { node: parent, axis, kind: Kind::EvenShare }, q);
                    }
                    if ! size.is_flex() {
                        self.queue_declared(parent, axis, |size| size == Size::FitContent, q);
                    }
                }
            }
        }
    }

    /// Queues the declared slots of a node on `axis` whose size matches `filter`.
    #[inline(always)]
    fn queue_declared(&mut self, i: Self::NodeId, axis: Axis, filter: impl Fn(Size) -> bool, q: &mut Queue<'_, Self::NodeId>) {
        for kind in DECLARED_SIZES {
            // The `Regular` slot of the root reads only the window size.
            if kind == Kind::Regular && i == self.root() {
                continue;
            }
            if let Some(size) = self.read_size(i, axis, kind) && filter(size) {
                self.queue(Slot { node: i, axis, kind }, q);
            }
        }
    }
}

impl<T: TreeAdapter + ?Sized> Incremental for T {}

#[cfg(test)]
#[path = "incremental_fuzz_tests.rs"]
mod incremental_fuzz_tests;
