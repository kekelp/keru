//! The fill-fitcontent interaction: a `FitContent` node with `Fill` children becomes `Fill`, and its content is a lower bound. If no child can grow to infinity, the content is also an upper bound.
use crate::*;

pub(crate) const DECLARED_SIZES: [Kind; 5] = [Kind::Regular, Kind::Min, Kind::Max, Kind::ContentMin, Kind::ContentMax];

/// The resolve flags of each axis come after the slot states.
const RESOLVED_FLAGS_SHIFT: u32 = 2 * N_SLOTS as u32;
const RESOLVED_FLAGS_BITS: u32 = 2;
const _: () = assert!(RESOLVED_FLAGS_SHIFT + 2 * RESOLVED_FLAGS_BITS <= 32, "The slot states and the resolve flags don't fit in NodeFlags");

/// The `FitContent` size becomes `Fill`, and the content is a lower bound (`Kind::ContentMin`).
const FIT_IS_FILL: u32 = 1 << 0;
/// The content is also an upper bound (`Kind::ContentMax`).
const CONTENT_MAX: u32 = 1 << 1;

impl NodeFlags {
    #[inline(always)]
    fn check_flag(self, axis: Axis, flag: u32) -> bool {
        self.0 & (flag << (RESOLVED_FLAGS_SHIFT + RESOLVED_FLAGS_BITS * axis as u32)) != 0
    }

    #[inline(always)]
    fn set_flag(&mut self, axis: Axis, flag: u32) {
        self.0 |= flag << (RESOLVED_FLAGS_SHIFT + RESOLVED_FLAGS_BITS * axis as u32);
    }
}

/// A ContentMax slot that is read before it is solved, in a cycle, means "no max".
pub(crate) const fn init_content_values(mut values: [f32; N_SLOTS]) -> [f32; N_SLOTS] {
    values[slot_index(X, Kind::ContentMax)] = f32::INFINITY;
    values[slot_index(Y, Kind::ContentMax)] = f32::INFINITY;
    values
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct GrowingChildren {
    any: bool,
    all: bool,
    all_capped: bool,
}

impl GrowingChildren {
    const NONE: GrowingChildren = GrowingChildren { any: false, all: true, all_capped: true };
}

/// The growing children on each axis where the node can become `Fill`.
pub(crate) type Growth = Xy<Option<GrowingChildren>>;

#[inline(always)]
fn can_become_fill<S: Solve + ?Sized>(solver: &S, i: S::NodeId, axis: Axis) -> bool {
    let params = solver.params(i);
    params.size[axis] == Size::FitContent && ! matches!(params.children_layout, ChildrenLayout::Grid { .. })
}

#[inline(always)]
pub(crate) fn start_growth<S: Solve + ?Sized>(solver: &S, i: S::NodeId) -> Growth {
    let mut growth = Xy::new(None, None);
    for axis in [X, Y] {
        if can_become_fill(solver, i, axis) {
            growth[axis] = Some(GrowingChildren::NONE);
        }
    }
    growth
}

#[inline(always)]
pub(crate) fn add_child_growth<S: Solve + ?Sized>(solver: &S, growth: &mut Growth, child: S::NodeId) {
    for axis in [X, Y] {
        if let Some(growing_children) = &mut growth[axis] {
            let child_can_grow = solver.read_size(child, axis, Kind::Regular).is_some_and(|size| size.is_fill());
            growing_children.any |= child_can_grow;
            growing_children.all &= child_can_grow;
            if child_can_grow {
                growing_children.all_capped &= solver.read_size(child, axis, Kind::ContentMax).is_some() || solver.read_size(child, axis, Kind::Max).is_some_and(|max| ! max.is_flex());
            }
        }
    }
}

pub(crate) fn resolve<S: Solve + ?Sized>(solver: &mut S, i: S::NodeId, growth: Growth) {
    for axis in [X, Y] {
        let Some(growing_children) = growth[axis] else {
            continue;
        };
        let params = solver.params(i);
        let becomes_fill = match solver.stack_axis(i) == Some(axis) {
            true => growing_children.any,
            false => growing_children.any && growing_children.all && params.text == TextContent::None && params.image.is_none(),
        };
        if becomes_fill {
            let flags = &mut solver.layout_data_mut(i).flags;
            // Keep the content as a lower bound, so that the node doesn't get smaller than its children
            flags.set_flag(axis, FIT_IS_FILL);
            // If no child can grow to infinity, the content is also an upper bound.
            if growing_children.all_capped {
                flags.set_flag(axis, CONTENT_MAX);
            }
        }
    }
}

#[inline(always)]
pub(crate) fn fit_is_fill<S: Solve + ?Sized>(solver: &S, i: S::NodeId, axis: Axis) -> bool {
    solver.layout_data(i).flags.check_flag(axis, FIT_IS_FILL)
}

#[inline(always)]
pub(crate) fn read_content_size<S: Solve + ?Sized>(solver: &S, i: S::NodeId, axis: Axis, kind: Kind) -> Option<Size> {
    let flags = solver.layout_data(i).flags;
    match kind {
        Kind::ContentMin => flags.check_flag(axis, FIT_IS_FILL).then_some(Size::FitContent),
        Kind::ContentMax => flags.check_flag(axis, CONTENT_MAX).then_some(Size::FitContent),
        _ => unreachable!(),
    }
}

/// Adds the content bounds to the declared bounds.
#[inline(always)]
pub(crate) fn content_bounds<S: Solve + ?Sized>(solver: &S, i: S::NodeId, axis: Axis, min: Option<f32>, max: Option<f32>) -> (Option<f32>, Option<f32>) {
    let flags = solver.layout_data(i).flags;
    let final_min = if flags.check_flag(axis, FIT_IS_FILL) {
        Some(clamp(solver.get_solved_value(i, axis, Kind::ContentMin), min, max))
    } else { min };
    let final_max = if flags.check_flag(axis, CONTENT_MAX) {
        Some(clamp(solver.get_solved_value(i, axis, Kind::ContentMax), min, max))
    } else { max };
    (final_min, final_max)
}

#[inline(always)]
pub(crate) fn is_max_bound(kind: Kind) -> bool {
    matches!(kind, Kind::Max | Kind::ContentMax)
}
