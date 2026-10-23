use crate::*;
use crate::Axis::{X, Y};

/// How the solver reads the declared sizes of a node on one axis, after `resolve_sizes`. With no flags set, it reads the `LayoutParams` as they are.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Resolved(u8);

impl Resolved {
    pub(crate) const NONE: Resolved = Resolved(0);
    pub(crate) const FIT_IS_FILL: Resolved = Resolved(1 << 0);
    pub(crate) const CONTENT_MAX: Resolved = Resolved(1 << 1);
    pub(crate) const FILL_BOUNDS_DROPPED: Resolved = Resolved(1 << 2);

    #[inline(always)]
    pub(crate) fn has(self, flag: Resolved) -> bool {
        self.0 & flag.0 != 0
    }

    #[inline(always)]
    pub(crate) fn set(&mut self, flag: Resolved) {
        self.0 |= flag.0;
    }
}

pub(crate) trait Resolve: Solve {
    fn resolve_sizes(&mut self, i: Self::NodeId) {
        for axis in [X, Y] {
            if self.fit_becomes_fill(i, axis) {
                let content_max = self.growth_is_capped(i, axis);
                let resolved = &mut self.layout_data_mut(i).resolved[axis];
                // Keep the content as a lower bound, so that the node doesn't get smaller than its children
                resolved.set(Resolved::FIT_IS_FILL);
                // If no child can grow to infinity, the content is also an upper bound.
                if content_max {
                    resolved.set(Resolved::CONTENT_MAX);
                }
            }
        }

        // A stack without even-share has no space to share, so the Fill bounds of its stacked children don't apply. After this, every Fill bound reads the parent.
        for axis in [X, Y] {
            if self.stack_axis(i) == Some(axis) && ! self.has_even_share(i, axis) {
                for_each_layout_child!(self, i, child, {
                    if ! self.params(child).free_placement {
                        self.layout_data_mut(child).resolved[axis].set(Resolved::FILL_BOUNDS_DROPPED);
                    }
                });
            }
        }
    }

    fn fit_becomes_fill(&self, i: Self::NodeId, axis: Axis) -> bool {
        let params = self.params(i);
        if params.size[axis] != Size::FitContent {
            return false;
        }
        let stacked = match params.children_layout {
            ChildrenLayout::Stack { axis: stack_axis, .. } => stack_axis == axis,
            ChildrenLayout::Free => false,
            ChildrenLayout::Grid { .. } => return false,
        };

        let mut any_grows = false;
        let mut all_grow = true;
        for_each_layout_child!(self, i, child, {
            if ! self.params(child).free_placement {
                let grows = self.grows(child, axis);
                any_grows |= grows;
                all_grow &= grows;
            }
        });

        match stacked {
            // Stacked Fill children always collapse in a FitContent stack.
            true => any_grows,
            // On the cross axis and in free layouts, Fill children stretch to the biggest sibling. They collapse only if there is no other content.
            false => any_grows && all_grow && params.text == TextContent::None && params.image.is_none(),
        }
    }

    /// A child that would grow into the space given by a `FitContent` parent that becomes `Fill`.
    #[inline(always)]
    fn grows(&self, child: Self::NodeId, axis: Axis) -> bool {
        self.read_size(child, axis, Kind::Regular) == Some(Size::Fill)
    }

    fn growth_is_capped(&self, i: Self::NodeId, axis: Axis) -> bool {
        let mut capped = true;
        for_each_layout_child!(self, i, child, {
            if ! self.params(child).free_placement && self.grows(child, axis) {
                capped &= self.read_size(child, axis, Kind::ContentMax).is_some() || self.read_size(child, axis, Kind::Max).is_some_and(|max| ! max.is_flex());
            }
        });
        capped
    }
}

impl<T: TreeAdapter + ?Sized> Resolve for T {}
