//! Without the `fit_fill` feature, a `FitContent` node always fits its content. These functions have the same signatures as the ones in `fit_fill.rs`, and they do nothing.
use crate::*;

pub(crate) const DECLARED_SIZES: [Kind; 3] = [Kind::Regular, Kind::Min, Kind::Max];

pub(crate) const fn init_content_values(values: [f32; N_SLOTS]) -> [f32; N_SLOTS] {
    values
}

pub(crate) type Growth = ();

#[inline(always)]
pub(crate) fn start_growth<S: Solve + ?Sized>(_solver: &S, _i: S::NodeId) -> Growth {}

#[inline(always)]
pub(crate) fn add_child_growth<S: Solve + ?Sized>(_solver: &S, _growth: &mut Growth, _child: S::NodeId) {}

#[inline(always)]
pub(crate) fn resolve<S: Solve + ?Sized>(_solver: &mut S, _i: S::NodeId, _growth: Growth) {}

#[inline(always)]
pub(crate) fn fit_is_fill<S: Solve + ?Sized>(_solver: &S, _i: S::NodeId, _axis: Axis) -> bool {
    false
}

#[inline(always)]
pub(crate) fn read_content_size<S: Solve + ?Sized>(_solver: &S, _i: S::NodeId, _axis: Axis, _kind: Kind) -> Option<Size> {
    unreachable!()
}

#[inline(always)]
pub(crate) fn content_bounds<S: Solve + ?Sized>(_solver: &S, _i: S::NodeId, _axis: Axis, min: Option<f32>, max: Option<f32>) -> (Option<f32>, Option<f32>) {
    (min, max)
}

#[inline(always)]
pub(crate) fn is_max_bound(kind: Kind) -> bool {
    kind == Kind::Max
}
