use crate::*;

impl Ui {
    fn all_dependencies(&self) -> impl Iterator<Item = LayoutDependency> + '_ {
        std::iter::once(ROOT_I)
            .chain(self.sys.nodes.iter())
            .flat_map(|i| self.sys.nodes[i].layout_dependents.iter().copied())
    }

    pub fn layout_dependencies_dot(&self) -> String {
        use std::fmt::Write;

        let slot_id = |slot: GraphElement| {
            let raw = format!("{:?}_{:?}_{:?}", slot.node, slot.axis, slot.size_type);
            raw.replace(|c: char| ! c.is_alphanumeric(), "_")
        };
        let slot_label = |slot: GraphElement| {
            // The debug name ends in a "[file:line]" part, which is too long to put in a graph.
            let full = format!("{}", self.node_debug_name(slot.node));
            let mut name = full.split('[').next().unwrap_or("").trim().to_string();
            if name.is_empty() {
                name = format!("{:?}", slot.node);
            }
            format!("{}\\n{:?} {:?}", name.replace('"', "'"), slot.axis, slot.size_type)
        };

        let mut out = String::new();
        let _ = writeln!(out, "digraph layout_dependencies {{");
        let _ = writeln!(out, "  rankdir=LR;");
        let _ = writeln!(out, "  node [shape=box, fontname=\"monospace\", fontsize=10];");
        let _ = writeln!(out, "  edge [fontname=\"monospace\", fontsize=8];");

        let mut seen: Vec<GraphElement> = Vec::new();
        for c in self.all_dependencies() {
            for slot in [c.dependent, c.depends_on] {
                if seen.contains(&slot) { continue; }
                seen.push(slot);
                let color = match slot.axis { X => "#e8f0ff", Y => "#fff0e8" };
                let _ = writeln!(out, "  {} [label=\"{}\", style=filled, fillcolor=\"{}\"];",
                    slot_id(slot), slot_label(slot), color);
            }
        }

        for c in self.all_dependencies() {
            let _ = writeln!(out, "  {} -> {};",
                slot_id(c.dependent), slot_id(c.depends_on));
        }

        let _ = writeln!(out, "}}");
        out
    }

    pub(crate) fn l2_dump_solved(&self, slot: GraphElement, size: f32, deferred: bool) {
        use std::fmt::Write;

        let mut how = "solved".to_string();
        if deferred {
            let mut guessed = String::new();
            for c in self.all_dependencies() {
                if c.dependent != slot {
                    continue;
                }
                // The ones that did arrive are what it was just solved from. The rest never came.
                if self.sys.nodes[c.depends_on.node].l2_solved[c.depends_on.axis][c.depends_on.size_type as usize].is_some() {
                    continue;
                }
                if ! guessed.is_empty() {
                    guessed.push_str(", ");
                }
                let _ = write!(guessed, "{}.{:?}.{:?}", self.node_debug_name(c.depends_on.node), c.depends_on.axis, c.depends_on.size_type);
            }
            how = format!("DEFERRED, guessed {}", guessed);
        }

        eprintln!("SOLVED: {}.{:?}.{:?} ({:?}) = {:.1}px  ({})",
            self.node_debug_name(slot.node),
            slot.axis,
            slot.size_type,
            self.declared_size(slot.node, slot.axis, slot.size_type),
            size * self.sys.size[slot.axis],
            how,
        );
    }

    pub(crate) fn l2_dump_unsolved(&mut self, i: NodeI) {
        for axis in [X, Y] {
            for size_type in SIZE_TYPES {
                if ! self.size_type_exists(i, axis, size_type) || self.sys.nodes[i].l2_solved[axis][size_type as usize].is_some() {
                    continue;
                }
                let waiting_for = self.sys.nodes[i].n_unsolved_layout_dependencies[axis][size_type as usize];
                let reason = if waiting_for > 0 {
                    // Everything it waits for is solved by now if the solve reached it at all, so a
                    // count that never came down means nobody ever showed up: a cycle, or an element
                    // upstream that was itself left unsolved.
                    format!("still waiting on {} of its dependencies", waiting_for)
                } else {
                    "nothing determines it: it never had a dependency to wait for".to_string()
                };

                eprintln!("UNSOLVED: {}.{:?}.{:?} ({:?}), {}. Falling back to {:.1}px",
                    self.node_debug_name(i),
                    axis,
                    size_type,
                    self.declared_size(i, axis, size_type),
                    reason,
                    self.l2_size_or_guess(i, axis) * self.sys.size[axis],
                );
            }
        }

        for_each_child!(self, self.sys.nodes[i], child, {
            self.l2_dump_unsolved(child);
        });
    }

    /// Experimental: every node's sizes after the solve, in pixels.
    #[allow(dead_code)]
    pub(crate) fn l2_dump_sizes(&mut self, i: NodeI, depth: usize) {
        let window = self.sys.size;
        let node = &self.sys.nodes[i];
        let px = |v: f32, axis: Axis| v * window[axis];
        let show = |v: Option<f32>, axis: Axis| match v {
            Some(v) => format!("{:.1}", px(v, axis)),
            None => "-".to_string(),
        };

        let slot = |axis: Axis, size_type: SizeType| show(node.l2_solved[axis][size_type as usize], axis);
        eprintln!("{:indent$}{}  x: {:?} guess {:.1} [reg {} lo {} hi {} fin {}] => {:.1}   |   y: {:?} guess {:.1} [reg {} lo {} hi {} fin {}] => {:.1}",
            "",
            node.debug_name(),
            node.params.layout.size[X], px(node.l2_base_guess[X], X),
            slot(X, SizeType::Regular), slot(X, SizeType::Min), slot(X, SizeType::Max), slot(X, SizeType::Final), px(node.size[X], X),
            node.params.layout.size[Y], px(node.l2_base_guess[Y], Y),
            slot(Y, SizeType::Regular), slot(Y, SizeType::Min), slot(Y, SizeType::Max), slot(Y, SizeType::Final), px(node.size[Y], Y),
            indent = depth * 2,
        );

        for_each_child!(self, self.sys.nodes[i], child, {
            self.l2_dump_sizes(child, depth + 1);
        });
    }

    #[allow(dead_code)]
    pub(crate) fn dump_layout_dependencies(&mut self) {
        for c in self.all_dependencies().collect::<Vec<_>>() {
            let dependent_name = self.node_debug_name(c.dependent.node);
            let depends_on_name = self.node_debug_name(c.depends_on.node);
            eprintln!("{dependent_name}.{:?}.{:?} <- {depends_on_name}.{:?}.{:?}",
                c.dependent.axis, c.dependent.size_type, c.depends_on.axis, c.depends_on.size_type);
        }
    }
}
