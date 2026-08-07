use crate::*;

use bumpalo::collections::Vec as BumpVec;

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
    GridCells,
}
pub(crate) const N_SIZE_TYPES: usize = 6;
pub(crate) const SIZE_TYPES: [SizeType; N_SIZE_TYPES] = [SizeType::Regular, SizeType::Min, SizeType::Max, SizeType::Final, SizeType::EvenShareForFillChildren, SizeType::GridCells];

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
    /// The main axis of the children's grid flow, if the children are laid out in a grid.
    pub grid_main_axis: Option<Axis>,
    pub has_sized_children: Xy<bool>,
    pub has_fill_children: Xy<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct LayoutDependency {
    pub dependent: GraphElement,
    pub depends_on: GraphElement,
}

impl Ui {
    pub(crate) fn l2_calculate_sizes(&mut self) {
        self.sys.layout_solve_queue.clear();
        self.sys.layout_deferred_queue.clear();
        self.reset_root();
        self.collapse_underdetermined_fitcontents(ROOT_I);
        self.push_dependencies_recursive(ROOT_I);
        self.solve_layout();
    }

    fn reset_root(&mut self) {
        self.clear_node_dependencies(ROOT_I);
        self.sys.layout_solve_queue.push(GraphElement { node: ROOT_I, axis: X, size_type: SizeType::Final });
        self.sys.layout_solve_queue.push(GraphElement { node: ROOT_I, axis: Y, size_type: SizeType::Final });
        self.sys.nodes[ROOT_I].size = Xy::new(1.0, 1.0);
        self.sys.nodes[ROOT_I].l2_solved[X][SizeType::Final as usize] = Some(1.0);
        self.sys.nodes[ROOT_I].l2_solved[Y][SizeType::Final as usize] = Some(1.0);
    }

    fn clear_node_dependencies(&mut self, i: NodeI) {
        self.sys.nodes[i].layout_dependents.clear();
        self.sys.nodes[i].n_unsolved_layout_dependencies = Xy::new([0; N_SIZE_TYPES], [0; N_SIZE_TYPES]);
        self.sys.nodes[i].l2_solved = Xy::new([None; N_SIZE_TYPES], [None; N_SIZE_TYPES]);
    }

    fn push_dependencies_recursive(&mut self, i: NodeI) {
        let info = self.get_parent_node_info(i);

        self.push_parent_dependencies(i, &info);

        for_each_child!(self, self.sys.nodes[i], child, {
            self.clear_node_dependencies(child);
            self.push_node_dependencies(child, info);
            self.push_dependencies_recursive(child);
        });

        self.push_solvable_nodes(i);
    }

    fn push_parent_dependencies(&mut self, i: NodeI, info: &ParentNodeInfo) {
        for axis in [X, Y] {
            if self.has_even_share_for_fill_children(info, axis) {
                self.push_dependency(LayoutDependency {
                    dependent: GraphElement { node: i, axis, size_type: SizeType::EvenShareForFillChildren },
                    depends_on: GraphElement { node: i, axis, size_type: SizeType::Final },
                });
            }
        }

        if let Some(main) = info.grid_main_axis {
            let cells = GraphElement { node: i, axis: main, size_type: SizeType::GridCells };

            if let ChildrenLayout::Grid { columns: MainAxisCellSize::Width(_), .. } = self.sys.nodes[i].params.children_layout {
                self.push_dependency(LayoutDependency {
                    dependent: cells,
                    depends_on: GraphElement { node: i, axis: main, size_type: SizeType::Final },
                });
            }

            for axis in [X, Y] {
                for size_type in DECLARED_SIZES {
                    if info.fitcontent_sizes[axis][size_type as usize] {
                        let slot = match size_type {
                            SizeType::Regular => self.regular_slot(i, axis),
                            other => other,
                        };
                        self.push_dependency(LayoutDependency {
                            dependent: GraphElement { node: i, axis, size_type: slot },
                            depends_on: cells,
                        });
                    }
                }
            }
        }
    }

    fn get_parent_node_info(&mut self, i: NodeI) -> ParentNodeInfo {
        let stack_axis = match self.sys.nodes[i].params.children_layout {
            ChildrenLayout::Stack { axis, .. } => Some(axis),
            _ => None,
        };
        let mut regular_is_fitcontent = Xy::new(false, false);
        let mut fitcontent_sizes = Xy::new([false; N_DECLARED_SIZES], [false; N_DECLARED_SIZES]);
        for axis in [X, Y] {
            // A flex-collapsing node acts as Fill, not FitContent: it fills its parent, so to its own children it's a parent with real space to hand out, and it no longer waits on them for its own regular size.
            regular_is_fitcontent[axis] = matches!(self.effective_size(i, axis), Size::FitContent);
            // A `FitContent` bound is sized from the content just like a `FitContent` size is, so each of them waits for the children on its own.
            for size_type in DECLARED_SIZES {
                let size = match size_type {
                    SizeType::Regular => self.effective_size(i, axis),
                    other => self.declared_size(i, axis, other).unwrap_or(Size::Fill),
                };
                fitcontent_sizes[axis][size_type as usize] = matches!(size, Size::FitContent);
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

        ParentNodeInfo { node: i, regular_is_fitcontent, fitcontent_sizes, stack_axis, grid_main_axis: self.grid_main_axis(i), has_sized_children, has_fill_children }
    }


    fn push_solvable_nodes(&mut self, i: NodeI) {
        for axis in [X, Y] {
            // Write fixed Pixel values
            for size_type in DECLARED_SIZES {
                if let Some(Size::Pixels(px)) = self.declared_size(i, axis, size_type) {
                    let slot = match size_type {
                        SizeType::Regular => self.regular_slot(i, axis),
                        other => other,
                    };
                    self.sys.nodes[i].l2_solved[axis][slot as usize] = Some(self.pixels_to_frac(px, axis));
                }
            }

            // Push solvable nodes in the solver queue
            for size_type in SIZE_TYPES {
                if self.size_type_exists(i, axis, size_type) {
                    let slot = GraphElement { node: i, axis, size_type };
                    if self.sys.nodes[i].l2_solved[axis][size_type as usize].is_some() {
                        self.sys.nodes[i].n_unsolved_layout_dependencies[axis][size_type as usize] = 0;
                        self.sys.layout_solve_queue.push(slot);
                    } else if self.sys.nodes[i].n_unsolved_layout_dependencies[axis][size_type as usize] == 0 {
                        self.solve_element(slot, false);
                    }
                }
            }
        }
    }

    fn push_node_dependencies(&mut self, i: NodeI, parent: ParentNodeInfo) {
        // Dependency of this node's final size on its regular, min, max sizes, if they exist.
        for axis in [X, Y] {
            for size_type in DECLARED_SIZES {
                if self.declared_size(i, axis, size_type).is_some() && self.size_type_exists(i, axis, size_type) {
                    self.push_dependency(LayoutDependency {
                        dependent: GraphElement { node: i, axis, size_type: SizeType::Final },
                        depends_on: GraphElement { node: i, axis, size_type }
                    });
                }
            }
        }
        
        // Dependencies on other nodes.
        for axis in [X, Y] {
            let stack_main = parent.stack_axis == Some(axis) && ! self.sys.nodes[i].params.free_placement;
            let parent_has_even_share_for_fill_children = self.has_even_share_for_fill_children(&parent, axis);

            if stack_main && parent_has_even_share_for_fill_children {
                for size_type in DECLARED_SIZES {
                    match self.declared_size(i, axis, size_type) {
                        None | Some(Size::Fill) => continue,
                        _ => {}
                    }
                    // A regular size lives in the Final slot when there are no bounds to clamp it against.
                    let child_slot = match size_type {
                        SizeType::Regular => self.regular_slot(i, axis),
                        other => other,
                    };
                    self.push_dependency(LayoutDependency {
                        dependent: GraphElement { node: parent.node, axis, size_type: SizeType::EvenShareForFillChildren },
                        depends_on: GraphElement { node: i, axis, size_type: child_slot },
                    });
                }
            }

            if ! self.sys.nodes[i].params.free_placement {
                for child_size_type in DECLARED_SIZES {
                    // A flex-collapsing child fills its FitContent parent instead of sizing it, so it's Fill-like here and contributes nothing (it goes through the even share like any Fill).
                    let child_size = match child_size_type {
                        SizeType::Regular => self.effective_size(i, axis),
                        other => self.declared_size(i, axis, other).unwrap_or(Size::Fill),
                    };
                    match child_size {
                        Size::Fill | Size::Frac(_) => continue,
                        _ => {}
                    }
                    let child_slot = match child_size_type {
                        SizeType::Regular => self.regular_slot(i, axis),
                        other => other,
                    };
                    for parent_size_type in DECLARED_SIZES {
                        if parent.fitcontent_sizes[axis][parent_size_type as usize] {
                            let parent_slot = match parent_size_type {
                                SizeType::Regular => self.regular_slot(parent.node, axis),
                                other => other,
                            };
                            self.push_dependency(LayoutDependency {
                                dependent: GraphElement { node: parent.node, axis, size_type: parent_slot },
                                depends_on: GraphElement { node: i, axis, size_type: child_slot },
                            });
                        }
                    }
                }
            }

            for size_type in DECLARED_SIZES {
                // A flex-collapsing node's regular size behaves as Fill: it fills its parent instead of summing its children.
                let size = match size_type {
                    SizeType::Regular => Some(self.effective_size(i, axis)),
                    other => self.declared_size(i, axis, other),
                };
                let Some(size) = size else { continue };
                if self.is_phantom_fill_bound(i, axis, size_type) {
                    continue;
                }
                let size_type = match size_type {
                    SizeType::Regular => self.regular_slot(i, axis),
                    other => other,
                };

                let mut dependency_on_other_axis = false;
                let mut dependency_on_parent = false;

                let fill_or_frac = matches!(size, Size::Fill | Size::Frac(_));

                if axis == Y && self.text_wraps_from_width(i, size) {
                    dependency_on_other_axis = true;
                }

                if matches!(size, Size::AspectRatio(_)) {
                    if matches!(self.sys.nodes[i].params.layout.size[axis.other()], Size::AspectRatio(_)) {
                        log::warn!("A node shouldn't be AspectRatio on both axes. (node: {})", self.sys.nodes[i].debug_name());
                    } else {
                        dependency_on_other_axis = true;
                    }
                }

                if fill_or_frac {
                    if ! parent.regular_is_fitcontent[axis] {
                        dependency_on_parent = true;
                    } else if ! parent.has_sized_children[axis] {
                        log::warn!("A FitContent node has no children that could give it a size: all of them ask for a share of it with no bound to fall back on, so they all stay at zero. (node: {}, axis: {:?})", self.sys.nodes[parent.node].debug_name(), axis);
                    } else if ! stack_main || matches!(size, Size::Frac(_)) {
                        dependency_on_parent = true;
                    } else {
                        log::warn!("A Fill child of a FitContent stack along its main axis has nothing to fill: the stack is only as big as its children, so there is never anything left over. It stays at its min size instead. (node: {}, axis: {:?})", self.sys.nodes[i].debug_name(), axis);
                    }
                }

                if dependency_on_other_axis {
                    self.push_dependency(LayoutDependency {
                        dependent: GraphElement { node: i, axis, size_type },
                        depends_on: GraphElement { node: i, axis: axis.other(), size_type: SizeType::Final }
                    });
                }
                if dependency_on_parent {
                    // A Fill on the stack's main axis comes out of the even share rather than straight out of the parent's size. The share waits for the parent itself, so nothing is lost by going through it.
                    let depends_on = match stack_main && parent_has_even_share_for_fill_children && matches!(size, Size::Fill) {
                        true => SizeType::EvenShareForFillChildren,
                        false => SizeType::Final,
                    };
                    self.push_dependency(LayoutDependency {
                        dependent: GraphElement { node: i, axis, size_type },
                        depends_on: GraphElement { node: parent.node, axis, size_type: depends_on }
                    });
                }
                // In a grid, what a Fill or Frac child gets is its cells, so it has to wait for them to be assigned.
                if fill_or_frac && let Some(main) = parent.grid_main_axis
                    && ! self.sys.nodes[i].params.free_placement {
                    self.push_dependency(LayoutDependency {
                        dependent: GraphElement { node: i, axis, size_type },
                        depends_on: GraphElement { node: parent.node, axis: main, size_type: SizeType::GridCells }
                    });
                }
            }
        }
    }

    fn push_dependency(&mut self, dependency: LayoutDependency) {
        self.sys.nodes[dependency.dependent.node].n_unsolved_layout_dependencies[dependency.dependent.axis][dependency.dependent.size_type as usize] += 1;
        self.sys.nodes[dependency.depends_on.node].layout_dependents.push(dependency);
    }

    pub(crate) fn solve_layout(&mut self) {
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

    fn solve_element(&mut self, slot: GraphElement, _deferred: bool) {
        if _deferred {
            log::warn!("Layout: solving {} with partial information due to a cycle in its layout dependencies.", self.sys.nodes[slot.node].debug_name());
        }
        if self.sys.nodes[slot.node].l2_solved[slot.axis][slot.size_type as usize].is_some() {
            return;
        }
        let solved = match slot.size_type {
            SizeType::Final => match self.regular_slot(slot.node, slot.axis) {
                SizeType::Final => {
                    let size = self.effective_size(slot.node, slot.axis);
                    self.l2_solve_size(slot.node, slot.axis, size, SizeType::Regular)
                }
                _ => {
                    let regular = self.l2_regular_or_guess(slot.node, slot.axis);
                    self.l2_clamp(slot.node, slot.axis, regular)
                }
            }
            SizeType::EvenShareForFillChildren => {
                let available = self.available_space_in_stack(slot.node, slot.axis);
                self.even_share_for_fill_children(slot.node, slot.axis, available)
            }
            SizeType::GridCells => self.assign_grid_cells(slot.node) as f32,
            _ => {
                let size = match slot.size_type {
                    SizeType::Regular => self.effective_size(slot.node, slot.axis),
                    other => self.declared_size(slot.node, slot.axis, other).unwrap(),
                };
                self.l2_solve_size(slot.node, slot.axis, size, slot.size_type)
            }
        };
        self.sys.nodes[slot.node].l2_solved[slot.axis][slot.size_type as usize] = Some(solved);
        self.sys.layout_solve_queue.push(slot);
    }

    /// Work out how many cells a grid has along its main axis and drop every child into one. Returns the number of cells along the main axis.
    fn assign_grid_cells(&mut self, i: NodeI) -> usize {
        let ChildrenLayout::Grid { columns, spacing_x, spacing_y, flow } = self.sys.nodes[i].params.children_layout else {
            unreachable!("A GridCells element only exists on a grid.");
        };
        let main = flow.main_axis;

        let inner_main = match columns {
            // The count is the answer on its own, so there's no size to look at.
            MainAxisCellSize::Count(_) => 0.0,
            MainAxisCellSize::Width(_) => {
                let size = self.l2_size_or_guess(i, main);
                let padding = self.pixels_to_frac2(self.sys.nodes[i].params.layout.padding)[main];
                (size - 2.0 * padding).max(0.0)
            }
        };
        let spacing_main = self.pixels_to_frac(Xy::new(spacing_x, spacing_y)[main], main);

        let n_main = self.grid_n_main(columns, flow, inner_main, spacing_main);
        self.grid_assign_cells(i, n_main, flow);
        n_main
    }

    pub(crate) fn l2_write_size(&mut self, i: NodeI) {
        for axis in [X, Y] {
            self.sys.nodes[i].size[axis] = self.l2_size_or_guess(i, axis);
        }
        self.l2_write_text_size(i);
    }

    pub(crate) fn l2_write_children_sizes(&mut self, i: NodeI) {
        // Also do this annoying hidden thing while we're at it.
        let children_can_hide = match self.sys.nodes[i].params.children_can_hide {
            ChildrenCanHide::Yes => true,
            ChildrenCanHide::No => false,
            ChildrenCanHide::Inherit => self.sys.nodes[i].can_hide,
        };

        for_each_child!(self, self.sys.nodes[i], child, {
            self.sys.nodes[child].can_hide = children_can_hide;
            self.l2_write_size(child);
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
        // Zero is the only honest answer when the solve hasn't reached this size yet: any other number would be made up, and whatever reads it would build on it.
        self.sys.nodes[i].l2_solved[axis][SizeType::Regular as usize].unwrap_or(0.0)
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

    fn l2_solve_size(&mut self, i: NodeI, axis: Axis, size: Size, size_type: SizeType) -> f32 {
        let padding = self.pixels_to_frac2(self.sys.nodes[i].params.layout.padding)[axis];

        match size {
            Size::Pixels(_) => panic!("A Pixels slot shouldn't be showing up as a dependent node that we have to solve."),

            Size::AspectRatio(aspect) => {
                let other = self.l2_size_or_guess(i, axis.other());
                let window_aspect = self.sys.size.x / self.sys.size.y;
                other * aspect / window_aspect
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
                self.l2_solve_fill_or_frac_from_parent(i, axis, size, size_type)
            },
        }
    }

    /// How tall a wrapping text node is once it's laid out at the width it actually got.
    fn l2_wrapped_text_height(&mut self, i: NodeI) -> f32 {
        let width = self.l2_size_or_guess(i, X);
        let padding = self.pixels_to_frac2(self.sys.nodes[i].params.layout.padding);
        let inner_x = (width - 2.0 * padding[X]).max(0.0);

        self.l2_text_size(i, Some(inner_x))[Y]
    }

    

    fn solve_fitcontent_size(&mut self, i: NodeI, axis: Axis) -> f32 {
        let mut own_content = 0.0f32;
        if self.sys.nodes[i].text_i.is_some() {
            own_content = self.l2_text_size(i, None)[axis];
        }
        if let Some(ImageRef::Raster(loaded)) = &self.sys.nodes[i].imageref {
            let size_pixels = Xy::new(loaded.width as f32, loaded.height as f32);
            own_content = own_content.max(self.pixels_to_frac(size_pixels[axis], axis));
        }

        match self.sys.nodes[i].params.children_layout {
            // A grid is as big as its cells, and it counts its own gaps.
            ChildrenLayout::Grid { columns, spacing_x, spacing_y, flow } => {
                let cells = self.l2_solve_grid_fitcontent(i, axis, columns, Xy::new(spacing_x, spacing_y), flow);
                cells.max(own_content)
            }

            ChildrenLayout::Stack { axis: stack_axis, .. } if stack_axis == axis => {
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

                current_size_estimate.max(own_content) + self.l2_stack_gaps(i, axis)
            }

            ChildrenLayout::Stack { .. } | ChildrenLayout::Free => {
                let mut biggest = 0.0f32;
                for_each_child!(self, self.sys.nodes[i], child, {
                    if ! self.sys.nodes[child].params.free_placement {
                        let child_size = match self.sys.nodes[child].params.layout.size[axis] {
                            Size::Fill | Size::Frac(_) => self.l2_clamp(child, axis, 0.0),
                            _ => self.l2_size_or_guess(child, axis),
                        };
                        biggest = biggest.max(child_size);
                    }
                });

                // No gaps: across a stack, or in a free layout, nothing is laid out one after the other.
                biggest.max(own_content)
            }
        }
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
    fn l2_solve_fill_or_frac_from_parent(&mut self, i: NodeI, axis: Axis, size: Size, size_type: SizeType) -> f32 {
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

        let available = self.available_space_in_stack(parent, axis);

        if let Size::Frac(f) = size {
            return available * f;
        }

        // Normally the share was solved as its own graph element, which is what makes it independent of the order the Fill children happen to come off the queue. It can still be missing if the graph never reached it, e.g. because it sits in a cycle, and then there's nothing better to do than what we used to do always: work it out here and now, out of whatever is solved so far.
        let share = match self.sys.nodes[parent].l2_solved[axis][SizeType::EvenShareForFillChildren as usize] {
            Some(share) => share,
            None => {
                let share = self.even_share_for_fill_children(parent, axis, available);
                self.sys.nodes[parent].l2_solved[axis][SizeType::EvenShareForFillChildren as usize] = Some(share);
                share
            }
        };

        // A `max_size(Fill)` bound is a cap that comes straight out of the share, so it resolves to the share itself: `min(content, share)` then happens when the node's regular size clamps against this bound. Raising it by the content the way a regular Fill is raised would turn the cap into the content and defeat it.
        if size_type == SizeType::Max {
            return share;
        }

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


    fn available_space_in_stack(&self, parent: NodeI, axis: Axis) -> f32 {
        let parent_size = self.l2_size_or_guess(parent, axis);
        let parent_padding = self.pixels_to_frac2(self.sys.nodes[parent].params.layout.padding)[axis];
        let inner = (parent_size - 2.0 * parent_padding).max(0.0);
        (inner - self.l2_stack_gaps(parent, axis)).max(0.0)
    }
    // Todo: should try to pre-count the free-placement children.
    fn l2_stack_gaps(&self, i: NodeI, axis: Axis) -> f32 {
        let ChildrenLayout::Stack { axis: stack_axis, spacing, .. } = self.sys.nodes[i].params.children_layout else {
            return 0.0;
        };
        if stack_axis != axis {
            return 0.0;
        }

        let mut n = 0;
        for_each_child!(self, self.sys.nodes[i], child, {
            if ! self.sys.nodes[child].params.free_placement {
                n += 1;
            }
        });

        self.pixels_to_frac(spacing, axis) * (n as f32 - 1.0).max(0.0)
    }

    fn clamp_fill_child_with_share_estimate(&self, child: NodeI, axis: Axis, share: f32) -> f32 {
        let content = self.l2_size_or_guess(child, axis);
        // Where the Fill sits decides whether the node's own content is a floor or a ceiling on what the share can do to it. A plain `Fill` (or a `min_size(Fill)` on a fixed node) grows the node up to the share, so the content is a floor: `content.max(share)`. A `max_size(Fill)` on an otherwise fixed node instead caps the node at the share, so the content is a ceiling: `content.min(share)`.
        let capped_by_content = ! matches!(self.sys.nodes[child].params.layout.size[axis], Size::Fill)
            && matches!(self.declared_size(child, axis, SizeType::Max), Some(Size::Fill));
        let raw = match capped_by_content {
            true => content.min(share),
            false => content.max(share),
        };
        self.l2_clamp(child, axis, raw)
    }

    fn even_share_for_fill_children(&mut self, parent: NodeI, axis: Axis, available: f32) -> f32 {
        let mut budget = available;

        return with_arena(|arena| {
            let mut fills: BumpVec<(NodeI, bool)> = BumpVec::new_in(arena);

            for_each_child!(self, self.sys.nodes[parent], child, {
                if self.sys.nodes[child].params.free_placement {
                    continue;
                }
                if self.any_size_is_fill(child, axis) {
                    fills.push((child, false));
                } else {
                    budget -= self.l2_size_or_guess(child, axis);
                }
            });

            let mut share = budget / fills.len().max(1) as f32;
            let mut frozen_total = 0.0;
            let mut n_frozen = 0;

            while n_frozen < fills.len() {
                share = (budget - frozen_total) / (fills.len() - n_frozen) as f32;

                let mut total_violation = 0.0;
                for (child, frozen) in fills.iter() {
                    if *frozen {
                        continue;
                    }
                    total_violation += self.clamp_fill_child_with_share_estimate(*child, axis, share) - share;
                }

                if total_violation.abs() < 1e-9 {
                    break;
                }

                let mut froze_any = false;
                for (child, frozen) in fills.iter_mut() {
                    if *frozen {
                        continue;
                    }
                    let size = self.clamp_fill_child_with_share_estimate(*child, axis, share);
                    let violation = size - share;
                    let stuck = match total_violation > 0.0 {
                        true => violation > 1e-9,
                        false => violation < -1e-9,
                    };
                    if stuck {
                        *frozen = true;
                        n_frozen += 1;
                        frozen_total += size;
                        froze_any = true;
                    }
                }

                if ! froze_any {
                    break;
                }
            }

            return share;
        });
    }

    /// The size the node's text takes at a given width, or at its own unwrapped width if none is given.
    fn l2_text_size(&mut self, i: NodeI, width: Option<f32>) -> Xy<f32> {
        let window = self.sys.size;
        let text_i = self.sys.nodes[i].text_i.as_ref().unwrap();

        const TEXT_WIDTH_TOLERANCE: f32 = 0.05;

        match text_i {
            TextI::TextBox(handle) => {
                let text_box = self.sys.renderer.text.get_text_box_mut(handle);

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
                let text_edit = self.sys.renderer.text.get_text_edit_mut(handle);
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


    pub(crate) fn declared_size(&self, i: NodeI, axis: Axis, size_type: SizeType) -> Option<Size> {
        let layout = &self.sys.nodes[i].params.layout;
        match size_type {
            SizeType::Min => layout.min_size[axis],
            SizeType::Max => layout.max_size[axis],
            SizeType::Regular => Some(layout.size[axis]),
            SizeType::Final | SizeType::EvenShareForFillChildren | SizeType::GridCells => None,
        }
    }

    pub(crate) fn size_type_exists(&self, i: NodeI, axis: Axis, size_type: SizeType) -> bool {
        let layout = &self.sys.nodes[i].params.layout;
        match size_type {
            SizeType::Final => true,
            SizeType::Regular => self.regular_slot(i, axis) == SizeType::Regular,
            SizeType::Min => layout.min_size[axis].is_some() && ! self.has_valid_bound(i, axis, SizeType::Min),
            SizeType::Max => layout.max_size[axis].is_some() && ! self.has_valid_bound(i, axis, SizeType::Max),
            // It isn't one of the node's own sizes, so nothing that walks the node's sizes should pick it up. It's solved from the graph alone.
            SizeType::EvenShareForFillChildren => false,
            // Only a grid has cells, and they're assigned once, on the flow's main axis.
            SizeType::GridCells => self.grid_main_axis(i) == Some(axis),
        }
    }

    pub(crate) fn grid_main_axis(&self, i: NodeI) -> Option<Axis> {
        match self.sys.nodes[i].params.children_layout {
            ChildrenLayout::Grid { flow, .. } => Some(flow.main_axis),
            _ => None,
        }
    }


    pub(crate) fn regular_slot(&self, i: NodeI, axis: Axis) -> SizeType {
        let layout = &self.sys.nodes[i].params.layout;
        let has_valid_min_bound = layout.min_size[axis].is_some() && ! self.has_valid_bound(i, axis, SizeType::Min);
        let has_valid_max_bound = layout.max_size[axis].is_some() && ! self.has_valid_bound(i, axis, SizeType::Max);
        let unbounded = ! has_valid_min_bound && ! has_valid_max_bound;
        match unbounded {
            true => SizeType::Final,
            false => SizeType::Regular,
        }
    }

    fn has_valid_bound(&self, i: NodeI, axis: Axis, size_type: SizeType) -> bool {
        self.is_phantom_fill_bound(i, axis, size_type) || self.is_underdetermined_grid_bound(i, axis, size_type)
    }

    fn is_underdetermined_grid_bound(&self, i: NodeI, axis: Axis, size_type: SizeType) -> bool {
        if ! matches!(self.declared_size(i, axis, size_type), Some(Size::FitContent)) {
            return false;
        }
        let ChildrenLayout::Grid { columns: MainAxisCellSize::Width(_), flow, .. } = self.sys.nodes[i].params.children_layout else {
            return false;
        };
        axis == flow.main_axis && self.sys.nodes[i].fitcontent_that_acts_as_fill[axis]
    }

    fn is_phantom_fill_bound(&self, i: NodeI, axis: Axis, size_type: SizeType) -> bool {
        if ! matches!(size_type, SizeType::Min | SizeType::Max) {
            return false;
        }
        if ! matches!(self.declared_size(i, axis, size_type), Some(Size::Fill)) {
            return false;
        }
        if self.sys.nodes[i].params.free_placement {
            return false;
        }
        let parent = self.sys.nodes[i].parent;
        let stack_main = matches!(
            self.sys.nodes[parent].params.children_layout,
            ChildrenLayout::Stack { axis: a, .. } if a == axis
        );
        stack_main && matches!(self.sys.nodes[parent].params.layout.size[axis], Size::FitContent)
    }

    pub(crate) fn collapse_underdetermined_fitcontents(&mut self, i: NodeI) {
        for_each_child!(self, self.sys.nodes[i], child, {
            self.collapse_underdetermined_fitcontents(child);
        });

        for axis in [X, Y] {
            let node = &self.sys.nodes[i];
            let is_fitcontent = matches!(node.params.layout.size[axis], Size::FitContent);
            let has_own_content = node.text_i.is_some() || node.imageref.is_some();
            if ! is_fitcontent || has_own_content {
                self.sys.nodes[i].fitcontent_that_acts_as_fill[axis] = false;
                continue;
            }

            if let ChildrenLayout::Grid { columns: MainAxisCellSize::Width(_), flow, .. } = node.params.children_layout {
                if axis == flow.main_axis {
                    log::warn!("Layout: A `FitContent` grid using `MainAxisCellSize::Width` has no way to determine its size.");
                    self.sys.nodes[i].fitcontent_that_acts_as_fill[axis] = true;
                    continue;
                }
            }

            let mut has_hard_child = false;
            let mut has_flex_child = false;
            for_each_child!(self, self.sys.nodes[i], child, {
                if ! self.sys.nodes[child].params.free_placement {
                    let (hard, flex) = self.child_hard_flex(child, axis);
                    has_hard_child |= hard;
                    has_flex_child |= flex;
                }
            });

            self.sys.nodes[i].fitcontent_that_acts_as_fill[axis] = ! has_hard_child && has_flex_child;
        }
    }

    fn child_hard_flex(&self, child: NodeI, axis: Axis) -> (bool, bool) {
        let layout = &self.sys.nodes[child].params.layout;
        let min_hard = matches!(layout.min_size[axis], Some(Size::Pixels(_) | Size::FitContent));
        let max_flex = matches!(layout.max_size[axis], Some(Size::Fill));
        let (base_hard, base_flex) = match self.effective_size(child, axis) {
            Size::Pixels(_) => (true, false),
            // Aspect Ratio on both axes is undetermined either way.
            Size::AspectRatio(_) if matches!(layout.size[axis.other()], Size::AspectRatio(_)) => (true, false),
            Size::AspectRatio(_) => self.child_hard_flex(child, axis.other()),
            Size::FitContent => (! matches!(layout.max_size[axis], Some(Size::Fill)), false),
            Size::Fill | Size::Frac(_) => (false, true),
        };
        (min_hard || base_hard, max_flex || base_flex)
    }

    fn effective_size(&self, i: NodeI, axis: Axis) -> Size {
        match self.sys.nodes[i].fitcontent_that_acts_as_fill[axis] {
            true => Size::Fill,
            false => self.sys.nodes[i].params.layout.size[axis],
        }
    }

    fn has_even_share_for_fill_children(&self, parent: &ParentNodeInfo, axis: Axis) -> bool {
        parent.stack_axis == Some(axis) && parent.has_fill_children[axis] && ! parent.regular_is_fitcontent[axis]
    }

    fn any_size_is_fill(&self, i: NodeI, axis: Axis) -> bool {
        DECLARED_SIZES.iter().any(|&size_type| matches!(self.declared_size(i, axis, size_type), Some(Size::Fill)))
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
        // A flex-collapsing child fills the parent rather than sizing it, so it counts as Fill here.
        match self.effective_size(child, axis) {
            _ if self.sys.nodes[child].params.free_placement => false,
            Size::Fill | Size::Frac(_) => matches!(layout.min_size[axis], Some(Size::Pixels(_) | Size::FitContent)),
            Size::Pixels(_) | Size::FitContent | Size::AspectRatio(_) => true,
        }
    }
}
