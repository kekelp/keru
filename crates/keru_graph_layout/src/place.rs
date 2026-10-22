use crate::*;

#[derive(Clone, Copy)]
struct ParentFrame {
    rect: XyRect,
    padding: Xy<f32>,
    flipped: Xy<bool>,
}

impl<T: TreeAdapter> Solver<'_, T> {
    pub(crate) fn place_all(&mut self) {
        self.place(self.tree.root(), Xy::new([0.0, 1.0], [0.0, 1.0]));
    }

    /// Gives a node its final rect, then places its children inside it. The rects are not stored: each one goes to `write_output_rect`, and down the recursion.
    fn place(&mut self, i: T::Node, rect: XyRect) {
        self.tree.write_output_rect(i, rect);
        match self.tree.params(i).children_layout {
            ChildrenLayout::Free => self.place_children_free(i, rect),
            ChildrenLayout::Stack { arrange, axis, spacing } => self.place_children_stack(i, rect, axis, arrange, spacing),
            ChildrenLayout::Grid { spacing_x, spacing_y, .. } => self.place_children_grid(i, rect, spacing_x, spacing_y),
        }
    }

    #[inline(always)]
    fn parent_frame(&self, i: T::Node, rect: XyRect) -> ParentFrame {
        let params = self.tree.params(i);
        let window = self.window_size;
        ParentFrame {
            rect,
            padding: Xy::new(params.padding.x / window.x, params.padding.y / window.y),
            flipped: params.children_origin_flipped,
        }
    }

    fn place_children_stack(&mut self, i: T::Node, rect: XyRect, axis: Axis, arrange: Arrange, spacing: f32) {
        let (main, cross) = (axis, axis.other());
        let frame = self.parent_frame(i, rect);
        let stack_rect = frame.rect;
        let padding = frame.padding;
        let spacing = spacing / self.window_size[axis];

        // On the main axis, totally ignore the children's chosen Position's and place them according to our own Stack::Arrange value.

        let mut n: u32 = 0;
        let mut children_size = 0.0;
        for_each_layout_child!(self, i, child, {
            if ! self.tree.params(child).free_placement {
                children_size += self.size(child)[main];
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
            let child_size = self.size(child);
            if self.tree.params(child).free_placement {
                self.place_child_free(child, &frame, child_size);
            } else {
                let mut child_rect = Xy::new([0.0, 0.0], [0.0, 0.0]);
                child_rect[cross] = self.resolve_pos_on_axis(&frame, child, cross, child_size[cross]);
                child_rect[main] = [walking_position, walking_position + child_size[main]];
                walking_position += child_size[main] + gap;
                self.place(child, child_rect);
            }
        });
    }

    fn place_children_grid(&mut self, i: T::Node, rect: XyRect, spacing_x: f32, spacing_y: f32) {
        let lines = self.data(i).grid_lines;

        let frame = self.parent_frame(i, rect);
        let spacing = Xy::new(spacing_x / self.window_size.x, spacing_y / self.window_size.y);
        let size = self.size(i);
        let mut cell = Xy::new(0.0, 0.0);
        for axis in [X, Y] {
            let n = lines[axis] as f32;
            cell[axis] = ((size[axis] - 2.0 * frame.padding[axis] - spacing[axis] * (n - 1.0)) / n.max(1.0)).max(0.0);
        }

        for_each_layout_child!(self, i, child, {
            let child_size = self.size(child);
            if self.tree.params(child).free_placement {
                self.place_child_free(child, &frame, child_size);
            } else {
                let child_cell = self.data(child).grid_cell;
                let mut child_rect = Xy::new([0.0, 0.0], [0.0, 0.0]);
                for axis in [X, Y] {
                    let start = frame.rect[axis][0] + frame.padding[axis] + child_cell[axis] as f32 * (cell[axis] + spacing[axis]);
                    child_rect[axis] = [start, start + child_size[axis]];
                }
                self.place(child, child_rect);
            }
        });
    }

    #[inline(always)]
    fn resolve_pos_on_axis(&self, frame: &ParentFrame, child: T::Node, axis: Axis, child_size: f32) -> [f32; 2] {
        let rect = frame.rect;
        let padding = frame.padding[axis];
        let flipped = frame.flipped[axis];
        let params = self.tree.params(child);

        let anchor_frac = anchor_frac(params.anchor[axis]);

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
                let static_pos = pixels / self.window_size[axis];
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
    fn place_child_free(&mut self, child: T::Node, frame: &ParentFrame, child_size: Xy<f32>) {
        let mut child_rect = Xy::new([0.0, 0.0], [0.0, 0.0]);
        for axis in [X, Y] {
            let rect = self.resolve_pos_on_axis(frame, child, axis, child_size[axis]);
            child_rect[axis] = self.apply_window_overflow(frame, child, axis, rect);
        }
        self.place(child, child_rect);
    }

    fn apply_window_overflow(&self, frame: &ParentFrame, child: T::Node, axis: Axis, rect: [f32; 2]) -> [f32; 2] {
        let params = self.tree.params(child);
        let behavior = params.window_overflow[axis];
        if behavior == WindowOverflow::Ignore {
            return rect;
        }

        const WIN_LO: f32 = 0.0;
        const WIN_HI: f32 = 1.0;

        let [lo, hi] = rect;
        let size = hi - lo;

        // If the node is larger than the window, just give up.
        if size >= (WIN_HI - WIN_LO) {
            return [WIN_LO, WIN_LO + size];
        }

        let mut out = rect;

        if behavior == WindowOverflow::Flip {
            // The anchor point (in absolute coords) is the point that should stay fixed, e.g. the cursor.
            let anchor_frac = anchor_frac(params.anchor[axis]);
            let a = if ! frame.flipped[axis] { lo + anchor_frac * size } else { hi - anchor_frac * size };

            let reflected = [2.0 * a - hi, 2.0 * a - lo];
            if hi > WIN_HI && reflected[0] >= WIN_LO {
                out = reflected;
            } else if lo < WIN_LO && reflected[1] <= WIN_HI {
                out = reflected;
            }
        }

        // Clamp (also the fallback when a flip wouldn't fit either).
        let [mut lo, mut hi] = out;
        let size = hi - lo;
        if hi > WIN_HI {
            hi = WIN_HI;
            lo = hi - size;
        }
        if lo < WIN_LO {
            lo = WIN_LO;
            hi = lo + size;
        }
        [lo, hi]
    }

    fn place_children_free(&mut self, i: T::Node, rect: XyRect) {
        let frame = self.parent_frame(i, rect);
        for_each_layout_child!(self, i, child, {
            let child_size = self.size(child);
            self.place_child_free(child, &frame, child_size);
        });
    }
}

/// The anchor point of a node, as a fraction of its size from its origin-side edge.
#[inline(always)]
fn anchor_frac(anchor: Anchor) -> f32 {
    match anchor {
        Anchor::Start => 0.0,
        Anchor::Center => 0.5,
        Anchor::End => 1.0,
        Anchor::Frac(f) => f,
    }
}
