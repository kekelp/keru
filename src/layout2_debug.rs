#![allow(dead_code)]

use crate::*;

impl Ui {
    fn all_dependencies(&self) -> impl Iterator<Item = LayoutDependency> + '_ {
        std::iter::once(ROOT_I)
            .chain(self.sys.nodes.iter())
            .flat_map(|i| self.sys.nodes[i].layout_dependents.iter().copied())
    }

    pub(crate) fn layout_dependencies_dot(&self) -> String {
        use std::fmt::Write;

        let slot_id = |slot: GraphElement| {
            let raw = format!("{:?}_{:?}_{:?}", slot.node, slot.axis, slot.size_type);
            raw.replace(|c: char| ! c.is_alphanumeric(), "_")
        };
        let slot_label = |slot: GraphElement| {
            // The debug name ends in a "[file:line]" part, which is too long to put in a graph.
            let full = format!("{}", self.sys.nodes[slot.node].debug_name());
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

    pub(crate) fn write_layout_dependencies_dot(&self, path: &str) {
        match std::fs::write(path, self.layout_dependencies_dot()) {
            Ok(()) => log::info!("Wrote the layout dependency graph to {path}"),
            Err(e) => log::error!("Couldn't write the layout dependency graph to {path}: {e}"),
        }
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
                let _ = write!(guessed, "{}.{:?}.{:?}", self.sys.nodes[c.depends_on.node].debug_name(), c.depends_on.axis, c.depends_on.size_type);
            }
            how = format!("DEFERRED, guessed {}", guessed);
        }

        eprintln!("SOLVED: {}.{:?}.{:?} ({:?}) = {:.1}px  ({})",
            self.sys.nodes[slot.node].debug_name(),
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
                    self.sys.nodes[i].debug_name(),
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

}
