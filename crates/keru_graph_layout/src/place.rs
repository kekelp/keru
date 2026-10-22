use crate::*;

#[derive(Clone, Copy)]
struct ParentFrame {
    rect: XyRect,
    padding: Xy<f32>,
    flipped: Xy<bool>,
}

impl<T: TreeAdapter> Solver<'_, T> {
    pub(crate) fn place_all(&mut self) {
        self.layout.rects[self.tree.root()] = Xy::new([0.0, 1.0], [0.0, 1.0]);
        self.recursive_place_children(self.tree.root());
    }

    fn recursive_place_children(&mut self, i: NodeI) {
        match self.tree.params(i).children_layout {
            ChildrenLayout::Free => self.place_children_free(i),
            ChildrenLayout::Stack { arrange, axis, spacing } => self.place_children_stack(i, axis, arrange, spacing),
            ChildrenLayout::Grid { spacing_x, spacing_y, .. } => self.place_children_grid(i, spacing_x, spacing_y),
        }

        for_each_layout_child!(self, i, child, {
            self.recursive_place_children(child);
        });
    }

    /// The solved size of a node that isn't placed yet, in fractions of the window size. Solving a `Final` slot writes `[0, size]` into the node's rect.
    #[inline(always)]
    fn child_size(&self, child: NodeI) -> Xy<f32> {
        let rect = self.layout.rects[child];
        Xy::new(rect.x[1], rect.y[1])
    }

    #[inline(always)]
    fn parent_frame(&self, i: NodeI) -> ParentFrame {
        let params = self.tree.params(i);
        let window = self.layout.window_size;
        ParentFrame {
            rect: self.layout.rects[i],
            padding: Xy::new(params.padding.x / window.x, params.padding.y / window.y),
            flipped: params.children_origin_flipped,
        }
    }

    fn place_children_stack(&mut self, i: NodeI, axis: Axis, arrange: Arrange, spacing: f32) {
        let (main, cross) = (axis, axis.other());
        let frame = self.parent_frame(i);
        let stack_rect = frame.rect;
        let padding = frame.padding;
        let spacing = spacing / self.layout.window_size[axis];

        // On the main axis, totally ignore the children's chosen Position's and place them according to our own Stack::Arrange value.

        let mut n: u32 = 0;
        let mut children_size = 0.0;
        for_each_layout_child!(self, i, child, {
            if ! self.tree.params(child).free_placement {
                children_size += self.child_size(child)[main];
                n += 1;
            }
        });

        let total_size = if n > 0 {
            children_size + spacing * (n - 1) as f32
        } else {
            children_size
        };

        let inner = (stack_rect[main][1] - stack_rect[main][0]) - 2.0 * padding[main];
        let free = inner - children_size;

        let (mut walking_position, gap) = match arrange {
            Arrange::Start => (stack_rect[main][0] + padding[main], spacing),
            Arrange::End => (stack_rect[main][1] - padding[main] - total_size, spacing),
            Arrange::Center => {
                let center = (stack_rect[main][1] + stack_rect[main][0]) / 2.0;
                (center - total_size / 2.0, spacing)
            },
            Arrange::SpaceBetween => {
                let gap = if n > 1 { free / (n - 1) as f32 } else { 0.0 };
                (stack_rect[main][0] + padding[main], gap)
            },
            Arrange::SpaceAround => {
                let gap = if n > 0 { free / n as f32 } else { 0.0 };
                (stack_rect[main][0] + padding[main] + gap / 2.0, gap)
            },
            Arrange::SpaceEvenly => {
                let gap = if n > 0 { free / (n + 1) as f32 } else { 0.0 };
                (stack_rect[main][0] + padding[main] + gap, gap)
            },
        };

        for_each_layout_child!(self, i, child, {
            let child_size = self.child_size(child);
            if self.tree.params(child).free_placement {
                self.place_child_free(child, &frame, child_size);
            } else {
                self.layout.rects[child][cross] = self.resolve_pos_on_axis(&frame, child, cross, child_size[cross]);
                self.layout.rects[child][main] = [walking_position, walking_position + child_size[main]];
                walking_position += child_size[main] + gap;
            }
        });
    }

    fn place_children_grid(&mut self, i: NodeI, spacing_x: f32, spacing_y: f32) {
        let lines = self.layout.solver_data[i].grid_lines;

        let frame = self.parent_frame(i);
        let spacing = Xy::new(spacing_x / self.layout.window_size.x, spacing_y / self.layout.window_size.y);
        let size = self.layout.size(i);
        let mut cell = Xy::new(0.0, 0.0);
        for axis in [X, Y] {
            let n = lines[axis] as f32;
            cell[axis] = ((size[axis] - 2.0 * frame.padding[axis] - spacing[axis] * (n - 1.0)) / n.max(1.0)).max(0.0);
        }

        for_each_layout_child!(self, i, child, {
            let child_size = self.child_size(child);
            if self.tree.params(child).free_placement {
                self.place_child_free(child, &frame, child_size);
            } else {
                let child_cell = self.layout.solver_data[child].grid_cell;
                for axis in [X, Y] {
                    let start = frame.rect[axis][0] + frame.padding[axis] + child_cell[axis] as f32 * (cell[axis] + spacing[axis]);
                    self.layout.rects[child][axis] = [start, start + child_size[axis]];
                }
            }
        });
    }

    #[inline(always)]
    fn resolve_pos_on_axis(&self, frame: &ParentFrame, child: NodeI, axis: Axis, child_size: f32) -> [f32; 2] {
        let rect = frame.rect;
        let padding = frame.padding[axis];
        let flipped = frame.flipped[axis];
        let params = self.tree.params(child);

        let anchor_frac = match params.anchor[axis] {
            Anchor::Start => 0.0,
            Anchor::Center => 0.5,
            Anchor::End => 1.0,
            Anchor::Frac(f) => f,
        };

        let place_at = |reference: f32| {
            if ! flipped {
                let low = reference - anchor_frac * child_size;
                [low, low + child_size]
            } else {
                let high = reference + anchor_frac * child_size;
                [high - child_size, high]
            }
        };

        let origin_edge = if ! flipped { rect[axis][0] + padding } else { rect[axis][1] - padding };
        let far_edge = if ! flipped { rect[axis][1] - padding } else { rect[axis][0] + padding };

        match params.position[axis] {
            Pos::Start => {
                if ! flipped { [origin_edge, origin_edge + child_size] } else { [origin_edge - child_size, origin_edge] }
            },
            Pos::End => {
                if ! flipped { [far_edge - child_size, far_edge] } else { [far_edge, far_edge + child_size] }
            },
            Pos::Pixels(pixels) => {
                let static_pos = pixels / self.layout.window_size[axis];
                place_at(if ! flipped { origin_edge + static_pos } else { origin_edge - static_pos })
            },
            Pos::Frac(frac) => {
                let inner_size = (rect[axis][1] - rect[axis][0]) - 2.0 * padding;
                let static_pos = frac * inner_size;
                place_at(if ! flipped { origin_edge + static_pos } else { origin_edge - static_pos })
            },
            Pos::Center => {
                let center = (rect[axis][0] + rect[axis][1]) / 2.0;
                [center - child_size / 2.0, center + child_size / 2.0]
            },
        }
    }

    #[inline(always)]
    fn place_child_free(&mut self, child: NodeI, frame: &ParentFrame, child_size: Xy<f32>) {
        for axis in [X, Y] {
            self.layout.rects[child][axis] = self.resolve_pos_on_axis(frame, child, axis, child_size[axis]);
        }
    }

    fn place_children_free(&mut self, i: NodeI) {
        let frame = self.parent_frame(i);
        for_each_layout_child!(self, i, child, {
            let child_size = self.child_size(child);
            self.place_child_free(child, &frame, child_size);
        });
    }
}
