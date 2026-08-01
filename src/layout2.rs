use crate::*;
use crate::layout::DUMP_L2_SIZING;

const MAX_FIT_STACK_FRAC: f32 = 0.999;
const MAX_FIT_STACK_STEPS: usize = 64;
const FIT_STACK_EPSILON: f32 = 1e-7;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum SizeType {
    Regular,
    Min,
    Max,
    Final,
    EvenShareForFillChildren,
}
pub(crate) const N_SIZE_TYPES: usize = 5;
pub(crate) const SIZE_TYPES: [SizeType; N_SIZE_TYPES] = [SizeType::Regular, SizeType::Min, SizeType::Max, SizeType::Final, SizeType::EvenShareForFillChildren];

pub(crate) const N_DECLARED_SIZES: usize = 3;
pub(crate) const DECLARED_SIZES: [SizeType; N_DECLARED_SIZES] = [SizeType::Regular, SizeType::Min, SizeType::Max];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct GraphElement {
    pub node: NodeI,
    pub axis: Axis,
    pub size_type: SizeType,
}

// Info about a parent node that its children need to know about in order to push their dependencies.
// Some fields are determined by looking at all children, so they really have to be precomputed like this.
// Others could be grabbed directly from the parent, but since we have this struct anyway, we might as well pre-store them here to improve the memory accesses.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ParentNodeInfo {
    pub node: NodeI,
    /// Whether the parent's *regular* size is `FitContent`, which is what decides whether it has room of its own to hand out to a `Fill` child.
    pub regular_is_fitcontent: Xy<bool>,
    pub fitcontent_sizes: Xy<[bool; N_DECLARED_SIZES]>,
    pub stack_axis: Option<Axis>,
    pub has_sized_children: Xy<bool>,
    pub has_fill_children: Xy<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct TextSizes {
    pub min: Xy<f32>,
    pub preferred: Xy<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct LayoutDependency {
    pub dependent: GraphElement,
    pub depends_on: GraphElement,
}

fn l2_settle(preferred: Option<f32>, min: f32, available: f32) -> f32 {
    preferred.unwrap_or(min).min(available).max(min)
}

fn l2_max_into(slot: &mut Option<f32>, candidate: f32) {
    *slot = Some(match *slot {
        Some(so_far) => so_far.max(candidate),
        None => candidate,
    });
}

impl Ui {
    pub(crate) fn l2_calculate_sizes(&mut self) {
        self.sys.layout_solve_queue.clear();
        self.sys.layout_deferred_queue.clear();
        self.clear_node_dependencies(ROOT_I);
        self.push_dependencies_recursive(ROOT_I);
        let root = Xy::new(1.0, 1.0);
        let _ = self.determine_base_sizes_recursive(ROOT_I, root, root, Xy::new(true, true), None);

        if DUMP_L2_SIZING {
            self.dump_layout_dependencies();
        }
        self.l2_solve();
        self.l2_write_sizes(ROOT_I);
        if DUMP_L2_SIZING {
            self.l2_dump_unsolved(ROOT_I);
            self.l2_dump_sizes(ROOT_I, 0);
        }
    }

    fn clear_node_dependencies(&mut self, i: NodeI) {
        self.sys.nodes[i].layout_dependents.clear();
        self.sys.nodes[i].n_unsolved_layout_dependencies = Xy::new([0; N_SIZE_TYPES], [0; N_SIZE_TYPES]);
        self.sys.nodes[i].l2_solved = Xy::new([None; N_SIZE_TYPES], [None; N_SIZE_TYPES]);
    }

    fn push_dependencies_recursive(&mut self, i: NodeI) {
        let info = self.get_parent_node_info(i);

        // The even share for this node's Fill children is taken out of its own size, so it waits for it. The children it also waits for are pushed one by one in `push_node_dependencies`.
        for axis in [X, Y] {
            if self.l2_has_even_share_for_fill_children(&info, axis) {
                self.push_dependency(GraphElement { node: i, axis, size_type: SizeType::EvenShareForFillChildren }, GraphElement { node: i, axis, size_type: SizeType::Final });
            }
        }

        for_each_child!(self, self.sys.nodes[i], child, {
            self.clear_node_dependencies(child);

            self.push_node_dependencies(child, info);
            self.push_dependencies_recursive(child);
        });
    }

    fn get_parent_node_info(&mut self, i: NodeI) -> ParentNodeInfo {
        let stack_axis = match self.sys.nodes[i].params.children_layout {
            ChildrenLayout::Stack { axis, .. } => Some(axis),
            _ => None,
        };
        let mut regular_is_fitcontent = Xy::new(false, false);
        let mut fitcontent_sizes = Xy::new([false; N_DECLARED_SIZES], [false; N_DECLARED_SIZES]);
        for axis in [X, Y] {
            regular_is_fitcontent[axis] = matches!(self.sys.nodes[i].params.layout.size[axis], Size::FitContent);
            // A `FitContent` bound is sized from the content just like a `FitContent` size is, so each of them waits for the children on its own.
            for size_type in DECLARED_SIZES {
                fitcontent_sizes[axis][size_type as usize] = matches!(self.declared_size(i, axis, size_type), Some(Size::FitContent));
            }
        }

        let own_content = self.sys.nodes[i].text_i.is_some() || self.sys.nodes[i].imageref.is_some();
        let mut has_sized_children = Xy::new(own_content, own_content);
        let mut has_fill_children = Xy::new(false, false);

        for_each_child!(self, self.sys.nodes[i], child, {
            for axis in [X, Y] {
                if self.child_can_size_parent(child, axis) {
                    has_sized_children[axis] = true;
                }
                if ! self.sys.nodes[child].params.free_placement && self.any_size_is_fill(child, axis) {
                    has_fill_children[axis] = true;
                }
            }
        });

        ParentNodeInfo { node: i, regular_is_fitcontent, fitcontent_sizes, stack_axis, has_sized_children, has_fill_children }
    }

    fn push_node_dependencies(&mut self, i: NodeI, parent: ParentNodeInfo) {
        // Dependency of this node's final size on its regular, min, max sizes, if they exist.
        for axis in [X, Y] {
            for size_type in DECLARED_SIZES {
                if self.declared_size(i, axis, size_type).is_some() {
                    self.push_dependency(GraphElement { node: i, axis, size_type: SizeType::Final }, GraphElement { node: i, axis, size_type });
                }
            }
        }
        
        // Dependencies on other nodes.
        for axis in [X, Y] {
            let stack_main = parent.stack_axis == Some(axis) && ! self.sys.nodes[i].params.free_placement;
            let parent_has_even_share_for_fill_children = self.l2_has_even_share_for_fill_children(&parent, axis);

            if stack_main && parent_has_even_share_for_fill_children
                && ! self.any_size_is_fill(i, axis) {
                self.push_dependency(GraphElement { node: parent.node, axis, size_type: SizeType::EvenShareForFillChildren }, GraphElement { node: i, axis, size_type: SizeType::Final });
            }

            if ! self.sys.nodes[i].params.free_placement {
                let contributions: &[SizeType] = match self.sys.nodes[i].params.layout.size[axis] {
                    Size::Fill | Size::Frac(_) => &[SizeType::Min, SizeType::Max],
                    _ => &[SizeType::Final],
                };
                for &contribution in contributions {
                    match self.declared_size(i, axis, contribution) {
                        Some(Size::Fill | Size::Frac(_)) => continue,
                        None if contribution != SizeType::Final => continue,
                        _ => {}
                    }
                    for size_type in DECLARED_SIZES {
                        if parent.fitcontent_sizes[axis][size_type as usize] {
                            self.push_dependency(GraphElement { node: parent.node, axis, size_type }, GraphElement { node: i, axis, size_type: contribution });
                        }
                    }
                }
            }

            for size_type in DECLARED_SIZES {
                let Some(size) = self.declared_size(i, axis, size_type) else { continue };
                let element = GraphElement { node: i, axis, size_type };

                let mut dependency_on_other_axis = false;
                let mut dependency_on_parent = false;

                let fill_or_frac = matches!(size, Size::Fill | Size::Frac(_));

                if axis == Y && self.text_wraps_from_width(i, size) {
                    dependency_on_other_axis = true;
                }

                if matches!(size, Size::AspectRatio(_)) {
                    if matches!(self.sys.nodes[i].params.layout.size[axis.other()], Size::AspectRatio(_)) {
                        log::warn!("A node shouldn't be AspectRatio on both axes. (node: {})", self.node_debug_name(parent.node));
                    } else {
                        dependency_on_other_axis = true;
                    }
                }

                if fill_or_frac {
                    if ! parent.regular_is_fitcontent[axis] {
                        dependency_on_parent = true;
                    } else if ! parent.has_sized_children[axis] {
                        log::warn!("A FitContent node has no children that could give it a size: all of them ask for a share of it with no bound to fall back on, so they all stay at zero. (node: {}, axis: {:?})", self.node_debug_name(parent.node), axis);
                    } else if ! stack_main || matches!(size, Size::Frac(_)) {
                        dependency_on_parent = true;
                    } else {
                        log::warn!("A Fill child of a FitContent stack along its main axis has nothing to fill: the stack is only as big as its children, so there is never anything left over. It stays at its min size instead. (node: {}, axis: {:?})", self.node_debug_name(i), axis);
                    }
                }

                if dependency_on_other_axis {
                    self.push_dependency(element, GraphElement { node: i, axis: axis.other(), size_type: SizeType::Final });
                }
                if dependency_on_parent {
                    // A Fill on the stack's main axis comes out of the even share rather than straight out of the parent's size. The share waits for the parent itself, so nothing is lost by going through it.
                    let depends_on = match stack_main && parent_has_even_share_for_fill_children && matches!(size, Size::Fill) {
                        true => SizeType::EvenShareForFillChildren,
                        false => SizeType::Final,
                    };
                    self.push_dependency(element, GraphElement { node: parent.node, axis, size_type: depends_on });
                }
            }
        }
    }

    pub(crate) fn declared_size(&self, i: NodeI, axis: Axis, size_type: SizeType) -> Option<Size> {
        let layout = &self.sys.nodes[i].params.layout;
        match size_type {
            SizeType::Min => layout.min_size[axis],
            SizeType::Max => layout.max_size[axis],
            SizeType::Regular => Some(layout.size[axis]),
            SizeType::Final | SizeType::EvenShareForFillChildren => None,
        }
    }

    pub(crate) fn size_type_exists(&self, i: NodeI, axis: Axis, size_type: SizeType) -> bool {
        let layout = &self.sys.nodes[i].params.layout;
        match size_type {
            SizeType::Regular | SizeType::Final => true,
            SizeType::Min => layout.min_size[axis].is_some(),
            SizeType::Max => layout.max_size[axis].is_some(),
            // It isn't one of the node's own sizes, so nothing that walks the node's sizes should pick it up. It's solved from the graph alone.
            SizeType::EvenShareForFillChildren => false,
        }
    }

    /// Whether this node hands out an even share to Fill children on this axis: it has to be a stack along it, with something asking to fill it, and with room of its own to hand out.
    fn l2_has_even_share_for_fill_children(&self, parent: &ParentNodeInfo, axis: Axis) -> bool {
        parent.stack_axis == Some(axis) && parent.has_fill_children[axis] && ! parent.regular_is_fitcontent[axis]
    }

    fn any_size_is_fill(&self, i: NodeI, axis: Axis) -> bool {
        DECLARED_SIZES.iter().any(|&size_type| matches!(self.declared_size(i, axis, size_type), Some(Size::Fill)))
    }

    fn any_size_is_fitcontent(&self, i: NodeI, axis: Axis) -> bool {
        DECLARED_SIZES.iter().any(|&size_type| matches!(self.declared_size(i, axis, size_type), Some(Size::FitContent)))
    }

    fn text_wraps_from_width(&self, i: NodeI, size: Size) -> bool {
        if ! matches!(self.sys.nodes[i].text_i, Some(TextI::TextBox(_))) {
            return false;
        }
        if ! matches!(size, Size::FitContent) {
            return false;
        }
        ! matches!(self.sys.nodes[i].params.layout.size[X], Size::AspectRatio(_))
    }

    fn child_can_size_parent(&self, child: NodeI, axis: Axis) -> bool {
        let layout = &self.sys.nodes[child].params.layout;
        match layout.size[axis] {
            _ if self.sys.nodes[child].params.free_placement => false,
            Size::Fill | Size::Frac(_) => matches!(layout.min_size[axis], Some(Size::Pixels(_) | Size::FitContent)),
            Size::Pixels(_) | Size::FitContent | Size::AspectRatio(_) => true,
        }
    }

    fn push_dependency(&mut self, dependent: GraphElement, depends_on: GraphElement) {
        self.sys.nodes[dependent.node].n_unsolved_layout_dependencies[dependent.axis][dependent.size_type as usize] += 1;
        self.sys.nodes[depends_on.node].layout_dependents.push(LayoutDependency { dependent, depends_on });
    }

    /// Returns the node's minimum content size and its preferred size, which is all the parent needs out of it on the way back up.
    fn determine_base_sizes_recursive(&mut self, i: NodeI, proposed: Xy<f32>, parent_inner: Xy<f32>, parent_inner_final: Xy<bool>, parent_stack_axis: Option<Axis>) -> (Xy<f32>, Xy<Option<f32>>) {
        for axis in [X, Y] {
            for size_type in [SizeType::Min, SizeType::Max] {
                if let Some(Size::Pixels(px)) = self.declared_size(i, axis, size_type) {
                    self.sys.nodes[i].l2_solved[axis][size_type as usize] = Some(self.pixels_to_frac(px, axis));
                }
            }
        }

        // Determine the base sizes
        let (available, settled) = self.take_available_size(i, proposed, parent_inner, parent_inner_final, parent_stack_axis);

        self.sys.nodes[i].l2_stack_gaps = Xy::new(0.0, 0.0);

        let padding = self.pixels_to_frac2(self.sys.nodes[i].params.layout.padding);
        let mut inner = available;
        for axis in [X, Y] {
            inner[axis] = (inner[axis] - 2.0 * padding[axis]).max(0.0);
        }

        let mut content_min = Xy::new(0.0f32, 0.0f32);
        let mut content_preferred: Xy<Option<f32>> = Xy::new(None, None);

        // Recursively propose the size down to our children.
        // While doing it, keep track of the sizes of the content. When we get back to this node after the recursive dive, use those sizes to determine our own.
        match self.sys.nodes[i].params.children_layout {
            ChildrenLayout::Free => {
                for_each_child!(self, self.sys.nodes[i], child, {
                    let (child_min, child_preferred) = self.determine_base_sizes_recursive(child, inner, inner, settled, None);
                    for axis in [X, Y] {
                        content_min[axis] = content_min[axis].max(child_min[axis]);
                        if let Some(child_preferred) = child_preferred[axis] {
                            l2_max_into(&mut content_preferred[axis], child_preferred);
                        }
                    }
                });
            }

            ChildrenLayout::Stack { axis, spacing, .. } => {
                let cross = axis.other();
                let spacing = self.pixels_to_frac(spacing, axis);

                let mut n = 0;
                let mut fixed_total = 0.0;
                let mut frac_total = 0.0;
                for_each_child!(self, self.sys.nodes[i], child, {
                    if ! self.sys.nodes[child].params.free_placement {
                        n += 1;
                        match self.sys.nodes[child].params.layout.size[axis] {
                            Size::Pixels(px) => fixed_total += self.pixels_to_frac(px, axis),
                            Size::Frac(f) => frac_total += f,
                            _ => {}
                        }
                    }
                });
                let gaps = spacing * (n as f32 - 1.0).max(0.0);
                self.sys.nodes[i].l2_stack_gaps[axis] = gaps;
                if frac_total > MAX_FIT_STACK_FRAC && self.any_size_is_fitcontent(i, axis) {
                    log::warn!("A FitContent stack has no finite size that satisfies its Frac children: they ask for {} of it in total. (node: {}, axis: {:?})",
                        frac_total, self.node_debug_name(i), axis);
                }

                let mut children_inner = inner;
                children_inner[axis] = (inner[axis] - gaps).max(0.0);

                let mut n_added = 0;
                let mut preferred_sum = 0.0f32;
                let mut any_child_opinion = false;
                for_each_child!(self, self.sys.nodes[i], child, {
                    if self.sys.nodes[child].params.free_placement {
                        let _ = self.determine_base_sizes_recursive(child, inner, inner, settled, None);
                    } else {
                        let own_fixed = match self.sys.nodes[child].params.layout.size[axis] {
                            Size::Pixels(px) => self.pixels_to_frac(px, axis),
                            _ => 0.0,
                        };
                        let mut child_proposed = children_inner;
                        child_proposed[axis] = (children_inner[axis] - (fixed_total - own_fixed)).max(0.0);

                        let (child_min, child_preferred) = self.determine_base_sizes_recursive(child, child_proposed, children_inner, settled, Some(axis));

                        if n_added != 0 {
                            content_min[axis] += spacing;
                            preferred_sum += spacing;
                        }
                        content_min[axis] += child_min[axis];
                        preferred_sum += child_preferred[axis].unwrap_or(child_min[axis]);
                        any_child_opinion |= child_preferred[axis].is_some();

                        content_min[cross] = content_min[cross].max(child_min[cross]);
                        if let Some(cross_preferred) = child_preferred[cross] {
                            l2_max_into(&mut content_preferred[cross], cross_preferred);
                        }
                        n_added += 1;
                    }
                });

                if any_child_opinion {
                    content_preferred[axis] = Some(preferred_sum);
                }
            }

            ChildrenLayout::Grid { columns, spacing_x, spacing_y, flow } => {
                let spacing = Xy::new(self.pixels_to_frac(spacing_x, X), self.pixels_to_frac(spacing_y, Y));
                let main = flow.main_axis;

                // The cells are assigned once, here, out of the size we're proposing to ourselves. The solve never reassigns them: a grid's shape isn't supposed to keep changing under it the way a stack's sizes do.
                let n_main = self.grid_n_main(columns, flow, inner[main], spacing[main]);
                self.grid_assign_cells(i, n_main, flow);

                let n_lines = Xy::new(self.grid_n_lines(i, X) as f32, self.grid_n_lines(i, Y) as f32);
                let mut gaps = Xy::new(0.0f32, 0.0);
                for axis in [X, Y] {
                    gaps[axis] = spacing[axis] * (n_lines[axis] - 1.0).max(0.0);
                }
                self.sys.nodes[i].l2_stack_gaps = gaps;

                // What one cell would come out at if we handed out the size we have now. It's only a proposal: each child still settles on its own min and preferred.
                let mut cell_proposal = Xy::new(0.0f32, 0.0);
                for axis in [X, Y] {
                    cell_proposal[axis] = ((inner[axis] - gaps[axis]) / n_lines[axis].max(1.0)).max(0.0);
                }
                if let MainAxisCellSize::Width(w) = columns {
                    cell_proposal[main] = self.pixels_to_frac(w, main);
                }

                let mut cell_min = Xy::new(0.0f32, 0.0);
                let mut cell_preferred: Xy<Option<f32>> = Xy::new(None, None);
                for_each_child!(self, self.sys.nodes[i], child, {
                    if self.sys.nodes[child].params.free_placement {
                        let _ = self.determine_base_sizes_recursive(child, inner, inner, settled, None);
                    } else {
                        let mut spans = Xy::new(1.0f32, 1.0);
                        let mut child_proposed = Xy::new(0.0f32, 0.0);
                        for axis in [X, Y] {
                            spans[axis] = self.grid_span(child, axis) as f32;
                            child_proposed[axis] = spans[axis] * cell_proposal[axis] + (spans[axis] - 1.0) * spacing[axis];
                        }

                        let (child_min, child_preferred) = self.determine_base_sizes_recursive(child, child_proposed, child_proposed, settled, None);

                        // The cells are uniform, so a child that spans several of them only asks for its share of one.
                        for axis in [X, Y] {
                            let share = |v: f32| (v - (spans[axis] - 1.0) * spacing[axis]) / spans[axis];
                            cell_min[axis] = cell_min[axis].max(share(child_min[axis]));
                            if let Some(child_preferred) = child_preferred[axis] {
                                l2_max_into(&mut cell_preferred[axis], share(child_preferred));
                            }
                        }
                    }
                });

                for axis in [X, Y] {
                    // A fixed cell size along the main axis overrides what the children asked for.
                    if axis == main && let MainAxisCellSize::Width(w) = columns {
                        let w = self.pixels_to_frac(w, axis);
                        cell_min[axis] = w;
                        cell_preferred[axis] = Some(w);
                    }
                    content_min[axis] = n_lines[axis] * cell_min[axis] + gaps[axis];
                    if let Some(cell_preferred) = cell_preferred[axis] {
                        content_preferred[axis] = Some(n_lines[axis] * cell_preferred + gaps[axis]);
                    }
                }
            }
        }

        if self.sys.nodes[i].text_i.is_some() {
            let text = self.l2_text_min_size(i, inner);
            for axis in [X, Y] {
                content_min[axis] = content_min[axis].max(text.min[axis]);
                l2_max_into(&mut content_preferred[axis], text.preferred[axis]);
            }
        }
        if self.sys.nodes[i].imageref.is_some() {
            let image = self.determine_image_size(i, inner);
            for axis in [X, Y] {
                content_min[axis] = content_min[axis].max(image[axis]);
                // An image is the size it is, so what it needs at a minimum is also what it wants.
                l2_max_into(&mut content_preferred[axis], image[axis]);
            }
        }

        let mut min = Xy::new(0.0f32, 0.0f32);
        let mut preferred: Xy<Option<f32>> = Xy::new(None, None);
        let mut final_size = Xy::new(None, None);
        for axis in [X, Y] {
            match self.sys.nodes[i].params.layout.size[axis] {
                Size::Pixels(px) => {
                    min[axis] = self.pixels_to_frac(px, axis);
                    preferred[axis] = Some(min[axis]);
                    final_size[axis] = Some(min[axis]);
                }
                Size::FitContent => {
                    min[axis] = content_min[axis] + 2.0 * padding[axis];
                    preferred[axis] = content_preferred[axis].map(|p| p + 2.0 * padding[axis]);
                }
                Size::Fill | Size::Frac(_) => {
                    min[axis] = 2.0 * padding[axis];
                    if settled[axis] {
                        final_size[axis] = Some(available[axis]);
                    }
                }
                Size::AspectRatio(_) => {}
            }
        }
        for axis in [X, Y] {
            if let Size::AspectRatio(aspect) = self.sys.nodes[i].params.layout.size[axis] {
                if matches!(self.sys.nodes[i].params.layout.size[axis.other()], Size::AspectRatio(_)) {
                    log::warn!("A Size shouldn't be AspectRatio in both dimensions. (node: {})", self.node_debug_name(i));
                } else {
                    let mult = self.l2_aspect_mult(axis, aspect);
                    min[axis] = min[axis.other()] * mult;
                    preferred[axis] = preferred[axis.other()].map(|other| other * mult);
                    final_size[axis] = final_size[axis.other()].map(|other| other * mult);
                }
            }
        }

        for axis in [X, Y] {
            if let Some(min_bound) = self.sys.nodes[i].l2_solved[axis][SizeType::Min as usize] {
                min[axis] = min[axis].max(min_bound);
            }
            preferred[axis] = preferred[axis].map(|p| p.max(min[axis]));
        }

        let mut guess = Xy::new(0.0f32, 0.0f32);
        for axis in [X, Y] {
            guess[axis] = l2_settle(preferred[axis], min[axis], available[axis]);
        }
        self.sys.nodes[i].l2_base_guess = guess;

        for axis in [X, Y] {
            if final_size[axis].is_some() {
                self.sys.nodes[i].l2_solved[axis][SizeType::Regular as usize] = final_size[axis];
            }
        }

        for axis in [X, Y] {
            for size_type in SIZE_TYPES {
                if ! self.size_type_exists(i, axis, size_type) {
                    continue;
                }
                if self.sys.nodes[i].l2_solved[axis][size_type as usize].is_some() {
                    self.sys.nodes[i].n_unsolved_layout_dependencies[axis][size_type as usize] = 0;
                } else if self.sys.nodes[i].n_unsolved_layout_dependencies[axis][size_type as usize] != 0 {
                    continue;
                } else {
                    self.sys.nodes[i].l2_solved[axis][size_type as usize] = Some(guess[axis]);
                }
                self.sys.layout_solve_queue.push(GraphElement { node: i, axis, size_type });
            }
        }

        (min, preferred)
    }


    fn take_available_size(&mut self, i: NodeI, proposed: Xy<f32>, parent_inner: Xy<f32>, parent_inner_final: Xy<bool>, parent_stack_axis: Option<Axis>) -> (Xy<f32>, Xy<bool>) {
        let mut available = proposed;
        let mut settled = Xy::new(false, false);

        for axis in [X, Y] {
            match self.sys.nodes[i].params.layout.size[axis] {
                Size::Pixels(px) => {
                    available[axis] = self.pixels_to_frac(px, axis);
                    settled[axis] = true;
                }
                Size::Frac(frac) => {
                    available[axis] = parent_inner[axis] * frac;
                    settled[axis] = parent_inner_final[axis];
                }
                Size::Fill => {
                    settled[axis] = parent_inner_final[axis] && parent_stack_axis != Some(axis);
                }
                Size::FitContent => {}
                Size::AspectRatio(_) => {}
            }
        }

        // Aspect ratio is settled if the other axis is.
        for axis in [X, Y] {
            if let Size::AspectRatio(aspect) = self.sys.nodes[i].params.layout.size[axis]
                && ! matches!(self.sys.nodes[i].params.layout.size[axis.other()], Size::AspectRatio(_)) {
                available[axis] = available[axis.other()] * self.l2_aspect_mult(axis, aspect);
                settled[axis] = settled[axis.other()];
            }
        }

        for axis in [X, Y] {
            available[axis] = self.l2_clamp(i, axis, available[axis]);
        }

        (available, settled)
    }


    pub(crate) fn l2_solve(&mut self) {
        let mut next_slot = 0;
        let mut next_deferred = 0;
        loop {
            while next_slot < self.sys.layout_solve_queue.len() {
                let slot = self.sys.layout_solve_queue[next_slot];
                next_slot += 1;

                for ci in 0..self.sys.nodes[slot.node].layout_dependents.len() {
                    let c = self.sys.nodes[slot.node].layout_dependents[ci];
                    if c.depends_on != slot {
                        continue;
                    }
                    let dep = c.dependent;

                    let deps_left = &mut self.sys.nodes[dep.node].n_unsolved_layout_dependencies[dep.axis][dep.size_type as usize];
                    if *deps_left == 0 {
                        continue;
                    }
                    *deps_left -= 1;
                    if *deps_left != 0 {
                        // Some input did flow into this element, but it also has other dependencies. If these other dependencies are never solved, e.g. because they are part of a cycle, we should still try to solve it using the input that we did get
                        self.sys.layout_deferred_queue.push(dep);
                        continue;
                    }

                    self.solve_element(dep, false);
                }
            }

            let Some(slot) = self.l2_next_deferred(&mut next_deferred) else { return };

            self.sys.nodes[slot.node].n_unsolved_layout_dependencies[slot.axis][slot.size_type as usize] = 0;
            self.solve_element(slot, true);
        }
    }

    fn l2_next_deferred(&mut self, next: &mut usize) -> Option<GraphElement> {
        while *next < self.sys.layout_deferred_queue.len() {
            let slot = self.sys.layout_deferred_queue[*next];
            *next += 1;
            if self.sys.nodes[slot.node].l2_solved[slot.axis][slot.size_type as usize].is_none() {
                return Some(slot);
            }
        }
        None
    }

    fn solve_element(&mut self, slot: GraphElement, deferred: bool) {
        if self.sys.nodes[slot.node].l2_solved[slot.axis][slot.size_type as usize].is_some() {
            return;
        }
        let solved = match slot.size_type {
            SizeType::Final => {
                let regular = self.l2_regular_or_guess(slot.node, slot.axis);
                self.l2_clamp(slot.node, slot.axis, regular)
            }
            SizeType::EvenShareForFillChildren => {
                let available = self.l2_stack_available(slot.node, slot.axis);
                self.l2_even_share_for_fill_children(slot.node, slot.axis, available)
            }
            _ => {
                let size = self.declared_size(slot.node, slot.axis, slot.size_type).unwrap();
                self.l2_solve_size(slot.node, slot.axis, size)
            }
        };
        self.sys.nodes[slot.node].l2_solved[slot.axis][slot.size_type as usize] = Some(solved);
        if DUMP_L2_SIZING {
            self.l2_dump_solved(slot, solved, deferred);
        }
        self.sys.layout_solve_queue.push(slot);
    }

    pub(crate) fn l2_write_sizes(&mut self, i: NodeI) {
        // Also do this annoying hidden thing while we're at it.
        let children_can_hide = match self.sys.nodes[i].params.children_can_hide {
            ChildrenCanHide::Yes => true,
            ChildrenCanHide::No => false,
            ChildrenCanHide::Inherit => self.sys.nodes[i].can_hide,
        };

        for axis in [X, Y] {
            self.sys.nodes[i].size[axis] = self.l2_size_or_guess(i, axis);
        }

        self.l2_write_text_size(i);

        for_each_child!(self, self.sys.nodes[i], child, {
            self.sys.nodes[child].can_hide = children_can_hide;
            self.l2_write_sizes(child);
        });
    }

    /// Push the node's final inner size into its text box or text edit.
    fn l2_write_text_size(&mut self, i: NodeI) {
        if self.sys.nodes[i].text_i.is_none() {
            return;
        }
        let padding = self.pixels_to_frac2(self.sys.nodes[i].params.layout.padding);
        let inner_x = (self.sys.nodes[i].size.x - 2.0 * padding.x) * self.sys.size[X];
        let inner_y = (self.sys.nodes[i].size.y - 2.0 * padding.y) * self.sys.size[Y];

        match self.sys.nodes[i].text_i.as_ref().unwrap() {
            TextI::TextBox(handle) => {
                self.sys.renderer.text.get_text_box_mut(handle).set_width(inner_x);
            }
            TextI::TextEdit(handle) => {
                self.sys.renderer.text.get_text_edit_mut(handle).set_size((inner_x, inner_y));
            }
        }
    }

    /// What the node really comes out at on this axis, or the best guess at it if the solve never got that far.
    pub(crate) fn l2_size_or_guess(&self, i: NodeI, axis: Axis) -> f32 {
        if let Some(size) = self.sys.nodes[i].l2_solved[axis][SizeType::Final as usize] {
            return size;
        }
        self.l2_clamp(i, axis, self.l2_regular_or_guess(i, axis))
    }

    fn l2_regular_or_guess(&self, i: NodeI, axis: Axis) -> f32 {
        let node = &self.sys.nodes[i];
        if let Some(size) = node.l2_solved[axis][SizeType::Regular as usize] {
            return size;
        }
        node.l2_base_guess[axis]
    }

    fn l2_clamp(&self, i: NodeI, axis: Axis, size: f32) -> f32 {
        let solved = &self.sys.nodes[i].l2_solved[axis];
        let mut size = size;
        if let Some(max) = solved[SizeType::Max as usize] {
            size = size.min(max);
        }
        if let Some(min) = solved[SizeType::Min as usize] {
            size = size.max(min);
        }
        size
    }

    fn l2_solve_size(&mut self, i: NodeI, axis: Axis, size: Size) -> f32 {
        let padding = self.pixels_to_frac2(self.sys.nodes[i].params.layout.padding)[axis];

        match size {
            Size::Pixels(_) => panic!("A Pixels slot shouldn't be showing up as a dependent node that we have to solve."),

            Size::AspectRatio(aspect) => {
                let other = self.l2_size_or_guess(i, axis.other());
                other * self.l2_aspect_mult(axis, aspect)
            }

            Size::FitContent => {
                let content = self.solve_fitcontent_size(i, axis);

                if axis == Y && self.text_wraps_from_width(i, size) {
                    let text = self.l2_wrapped_text_height(i);
                    return content.max(text) + 2.0 * padding;
                }

                // No clamp against the min: this is a sum over the same children the min is a sum over, and each one's solved size is at least its own min.
                content + 2.0 * padding
            }

            Size::Fill | Size::Frac(_) => {
                self.l2_solve_fill_or_frac_from_parent(i, axis, size)
            },
        }
    }

    /// How tall a wrapping text node is once it's laid out at the width it actually got.
    fn l2_wrapped_text_height(&mut self, i: NodeI) -> f32 {
        let width = self.l2_size_or_guess(i, X);
        let padding = self.pixels_to_frac2(self.sys.nodes[i].params.layout.padding);
        // The Y is whatever comes out of the wrapping, so there's nothing to offer on that axis.
        let inner = Xy::new((width - 2.0 * padding[X]).max(0.0), 0.0);

        self.l2_text_min_size(i, inner).min[Y]
    }

    fn solve_fitcontent_size(&mut self, i: NodeI, axis: Axis) -> f32 {
        if let ChildrenLayout::Grid { columns, spacing_x, spacing_y, flow } = self.sys.nodes[i].params.children_layout {
            return self.l2_solve_grid_fitcontent(i, axis, columns, Xy::new(spacing_x, spacing_y), flow);
        }

        let stack_main = matches!(
            self.sys.nodes[i].params.children_layout,
            ChildrenLayout::Stack { axis: a, .. } if a == axis
        );
        let gaps = self.sys.nodes[i].l2_stack_gaps[axis];

        if ! stack_main {
            let mut total = 0.0f32;
            for_each_child!(self, self.sys.nodes[i], child, {
                if ! self.sys.nodes[child].params.free_placement {
                    let child_size = match self.sys.nodes[child].params.layout.size[axis] {
                        Size::Fill | Size::Frac(_) => self.l2_clamp(child, axis, 0.0),
                        _ => self.l2_size_or_guess(child, axis),
                    };
                    total = total.max(child_size);
                }
            });
            return total + gaps;
        }

        let mut current_size_estimate = 0.0f32;
        for _ in 0..MAX_FIT_STACK_STEPS {
            let mut fixed = 0.0f32;
            let mut free_frac = 0.0f32;
            for_each_child!(self, self.sys.nodes[i], child, {
                if self.sys.nodes[child].params.free_placement {
                    continue;
                }
                match self.l2_fit_demand_stacked(child, axis, current_size_estimate) {
                    FitDemand::Scaling { frac } => free_frac += frac,
                    FitDemand::Fixed(size) => fixed += size,
                }
            });

            let next = match free_frac <= MAX_FIT_STACK_FRAC {
                true => fixed / (1.0 - free_frac),
                false => fixed,
            };
            let settled = (next - current_size_estimate).abs() <= FIT_STACK_EPSILON;
            current_size_estimate = next;
            if settled {
                break;
            }
        }

        current_size_estimate + gaps
    }

    /// A grid that fits its content is as big as its cells, and the cells are as big as the biggest child in them.
    fn l2_solve_grid_fitcontent(&mut self, i: NodeI, axis: Axis, columns: MainAxisCellSize, spacing_px: Xy<f32>, flow: GridFlow) -> f32 {
        let spacing = self.pixels_to_frac(spacing_px[axis], axis);
        let n_lines = self.grid_n_lines(i, axis) as f32;
        if n_lines == 0.0 {
            return 0.0;
        }

        let mut cell = 0.0f32;
        if axis == flow.main_axis && let MainAxisCellSize::Width(w) = columns {
            cell = self.pixels_to_frac(w, axis);
        } else {
            for_each_child!(self, self.sys.nodes[i], child, {
                if ! self.sys.nodes[child].params.free_placement {
                    let child_size = match self.sys.nodes[child].params.layout.size[axis] {
                        // A child that only asks for a share of the cell can't be what decides how big the cell is.
                        Size::Fill | Size::Frac(_) => self.l2_clamp(child, axis, 0.0),
                        _ => self.l2_size_or_guess(child, axis),
                    };
                    let span = self.grid_span(child, axis) as f32;
                    cell = cell.max((child_size - (span - 1.0) * spacing) / span);
                }
            });
        }

        n_lines * cell + spacing * (n_lines - 1.0)
    }

    fn l2_fit_demand_stacked(&self, child: NodeI, axis: Axis, current_parent_size_estimate: f32) -> FitDemand {
        match self.sys.nodes[child].params.layout.size[axis] {
            Size::Frac(frac) => {
                let wanted = current_parent_size_estimate * frac;
                let allowed = self.l2_clamp(child, axis, wanted);

                match allowed == wanted {
                    true => FitDemand::Scaling { frac },
                    false => FitDemand::Fixed(allowed),
                }
            }
            Size::Fill => FitDemand::Fixed(self.l2_clamp(child, axis, 0.0)),
            _ => FitDemand::Fixed(self.l2_size_or_guess(child, axis)),
        }
    }
}

enum FitDemand {
    Fixed(f32),
    Scaling { frac: f32 },
}

impl Ui {

    /// A Fill or Frac slot taking its size out of the parent. `size` is the slot's own `Size`, which is a bound's when it's a bound being solved.
    fn l2_solve_fill_or_frac_from_parent(&mut self, i: NodeI, axis: Axis, size: Size) -> f32 {
        let parent = self.sys.nodes[i].parent;
        let parent_size = self.l2_size_or_guess(parent, axis);
        let parent_padding = self.pixels_to_frac2(self.sys.nodes[parent].params.layout.padding)[axis];
        let inner = (parent_size - 2.0 * parent_padding).max(0.0);

        // In a grid, what the child gets is its cells, not the whole inner size.
        if let ChildrenLayout::Grid { spacing_x, spacing_y, .. } = self.sys.nodes[parent].params.children_layout
            && ! self.sys.nodes[i].params.free_placement {
            let spacing = self.pixels_to_frac(Xy::new(spacing_x, spacing_y)[axis], axis);
            let cell = self.l2_grid_cell_size(parent, axis, spacing);
            let span = self.grid_span(i, axis) as f32;
            let available = span * cell + (span - 1.0) * spacing;
            return match size {
                Size::Frac(f) => available * f,
                _ => available,
            };
        }

        let stack_main = matches!(
            self.sys.nodes[parent].params.children_layout,
            ChildrenLayout::Stack { axis: a, .. } if a == axis
        );

        if ! stack_main || self.sys.nodes[i].params.free_placement {
            return match size {
                Size::Frac(f) => inner * f,
                _ => inner,
            };
        }

        let available = self.l2_stack_available(parent, axis);

        if let Size::Frac(f) = size {
            return available * f;
        }

        // Normally the share was solved as its own graph element, which is what makes it independent of the order the Fill children happen to come off the queue. It can still be missing if the graph never reached it, e.g. because it sits in a cycle, and then there's nothing better to do than what we used to do always: work it out here and now, out of whatever is solved so far.
        let share = match self.sys.nodes[parent].l2_solved[axis][SizeType::EvenShareForFillChildren as usize] {
            Some(share) => share,
            None => {
                let share = self.l2_even_share_for_fill_children(parent, axis, available);
                self.sys.nodes[parent].l2_solved[axis][SizeType::EvenShareForFillChildren as usize] = Some(share);
                share
            }
        };

        // The share is what the child gets unless its own size is already past it: it raises a child up to the common size, it never cuts one down to it.
        self.l2_size_or_guess(i, axis).max(share)
    }

    /// The size of one of a grid's uniform cells along an axis, out of the size the grid has so far.
    fn l2_grid_cell_size(&self, parent: NodeI, axis: Axis, spacing: f32) -> f32 {
        let n = self.grid_n_lines(parent, axis) as f32;
        if n == 0.0 {
            return 0.0;
        }
        let parent_size = self.l2_size_or_guess(parent, axis);
        let parent_padding = self.pixels_to_frac2(self.sys.nodes[parent].params.layout.padding)[axis];
        let inner = (parent_size - 2.0 * parent_padding).max(0.0);
        ((inner - spacing * (n - 1.0)) / n).max(0.0)
    }

    /// What a stack has left over on its main axis for its children to divide, once its padding and its gaps are out of the way.
    fn l2_stack_available(&self, parent: NodeI, axis: Axis) -> f32 {
        let parent_size = self.l2_size_or_guess(parent, axis);
        let parent_padding = self.pixels_to_frac2(self.sys.nodes[parent].params.layout.padding)[axis];
        let inner = (parent_size - 2.0 * parent_padding).max(0.0);
        (inner - self.sys.nodes[parent].l2_stack_gaps[axis]).max(0.0)
    }

    /// The size every `Fill` child of this stack comes out at, except the ones whose own minimum is bigger: those keep their minimum, and the rest evenly divide whatever is left once the children that aren't asking for a share have taken theirs. Each round drops the children that are already past the current share out of the division, which can only lower it, so the split settles after at most one round per `Fill` child.
    fn l2_even_share_for_fill_children(&mut self, parent: NodeI, axis: Axis, available: f32) -> f32 {
        let mut budget = available;
        let mut n_fills = 0;
        for_each_child!(self, self.sys.nodes[parent], child, {
            if self.sys.nodes[child].params.free_placement {
                continue;
            }
            if self.sys.nodes[child].params.layout.size[axis] == Size::Fill {
                n_fills += 1;
            } else {
                budget -= self.l2_size_or_guess(child, axis);
            }
        });

        let mut share = budget / n_fills.max(1) as f32;
        loop {
            let mut over_share = 0.0f32;
            let mut n_under = 0;
            for_each_child!(self, self.sys.nodes[parent], child, {
                if self.sys.nodes[child].params.free_placement
                    || self.sys.nodes[child].params.layout.size[axis] != Size::Fill {
                    continue;
                }
                let base = self.l2_size_or_guess(child, axis);
                if base > share {
                    over_share += base;
                } else {
                    n_under += 1;
                }
            });

            if n_under == n_fills {
                break;
            }
            let new_share = (budget - over_share) / n_under.max(1) as f32;
            if (new_share - share).abs() < 1e-9 {
                break;
            }
            share = new_share;
        }

        share
    }

    fn l2_aspect_mult(&self, axis: Axis, aspect: f32) -> f32 {
        let window_aspect = self.sys.size.x / self.sys.size.y;
        match axis {
            X => aspect / window_aspect,
            Y => window_aspect / aspect,
        }
    }

    fn l2_text_min_size(&mut self, i: NodeI, inner: Xy<f32>) -> TextSizes {
        let window = self.sys.size;
        let text_i = self.sys.nodes[i].text_i.as_ref().unwrap();

        const TEXT_WIDTH_TOLERANCE: f32 = 0.05;

        match text_i {
            TextI::TextBox(handle) => {
                let inner_x_pixels = inner.x * window[X];
                let text_box = self.sys.renderer.text.get_text_box_mut(handle);

                let widths = text_box.content_widths();
                let width = inner_x_pixels.clamp(widths.min, widths.max.max(widths.min));

                text_box.set_width(width);
                let height = text_box.layout().height();

                TextSizes {
                    min: Xy::new(
                        (widths.min + TEXT_WIDTH_TOLERANCE) / window[X],
                        height / window[Y],
                    ),
                    preferred: Xy::new(
                        (widths.max.max(widths.min) + TEXT_WIDTH_TOLERANCE) / window[X],
                        height / window[Y],
                    ),
                }
            }
            TextI::TextEdit(handle) => {
                let text_edit = self.sys.renderer.text.get_text_edit_mut(handle);
                let height = if text_edit.single_line() {
                    match text_edit.layout().lines().next() {
                        Some(first_line) => first_line.metrics().line_height,
                        None => 0.0,
                    }
                } else {
                    0.0
                };
                // A text edit can be scrolled, so it never asks for any width, and there's nothing it wants beyond what it needs.
                let size = Xy::new(0.0, height / window[Y]);
                TextSizes {
                    min: size,
                    preferred: size,
                }
            }
        }
    }
}
