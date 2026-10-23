use crate::*;
use crate::Axis::{X, Y};

/// How the solver reads the declared sizes of a node on one axis, after `resolve_sizes`. With all flags false, it reads the `LayoutParams` as they are.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Resolved {
    /// The `FitContent` size becomes `Fill`, because the node has growing `Fill` children.
    pub(crate) fit_is_fill: bool,
    /// The `ContentMin` slot exists: the content is a lower bound.
    pub(crate) content_min: bool,
    /// The `ContentMax` slot exists: the content is also an upper bound, because no child can grow to infinity.
    pub(crate) content_max: bool,
    /// The `Fill` min and max sizes don't apply, because the node is in a stack that has no space to share.
    pub(crate) fill_bounds_dropped: bool,
}

impl Resolved {
    pub(crate) const NONE: Resolved = Resolved { fit_is_fill: false, content_min: false, content_max: false, fill_bounds_dropped: false };
}

pub(crate) trait Resolve: Solve {
    fn resolve_sizes(&mut self, i: Self::NodeId) {
        for axis in [X, Y] {
            if self.fit_becomes_fill(i, axis) {
                let content_max = self.growth_is_capped(i, axis);
                let resolved = &mut self.layout_data_mut(i).resolved[axis];
                resolved.fit_is_fill = true;
                // Keep the content as a lower bound, so that the node doesn't get smaller than its children
                resolved.content_min = true;
                // If no child can grow to infinity, the content is also an upper bound.
                resolved.content_max = content_max;
            }
        }

        // A stack without even-share has no space to share, so the Fill bounds of its stacked children don't apply. After this, every Fill bound reads the parent.
        for axis in [X, Y] {
            if self.stack_axis(i) == Some(axis) && ! self.has_even_share(i, axis) {
                for_each_layout_child!(self, i, child, {
                    if ! self.params(child).free_placement {
                        self.layout_data_mut(child).resolved[axis].fill_bounds_dropped = true;
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
