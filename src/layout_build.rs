use crate::*;
use crate::Axis::{X, Y};
use crate::layout_tree::*;

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

        // The children can be dependencies too, so the counts are complete only now.
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

    /// Appends every slot that reads this slot when it is solved to `dependents`.
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

    /// Adds the child's `Frac` sizes and the `Fill` sizes that the parent doesn't stack.
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
