use crate::*;

#[derive(Clone, Copy)]
pub(crate) struct ParentFrame {
    rect: XyRect,
    padding: Xy<[f32; 2]>,
    flipped: Xy<bool>,
}

/// The placement part of the layout. Every `TreeAdapter` gets it from the blanket impl.
pub(crate) trait Place: Solve {
    fn place_nodes(&mut self) {
        let window = self.window_size();
        let window = Xy::new([0.0, window.x], [0.0, window.y]);
        self.place(self.root(), window, window);
    }

    /// Gives a node its final rect, then places its children inside it.
    fn place(&mut self, i: Self::NodeId, rect: XyRect, parent_rect: XyRect) {
        let local_rect = Xy::new(
            [rect.x[0] - parent_rect.x[0], rect.x[1] - parent_rect.x[0]],
            [rect.y[0] - parent_rect.y[0], rect.y[1] - parent_rect.y[0]],
        );
        self.write_output_rect(i, rect, local_rect);
        match self.node_layout_params(i).children_layout {
            ChildrenLayout::Free => self.place_children_free(i, rect),
            ChildrenLayout::Stack { arrange, axis, spacing } => self.place_children_stack(i, rect, axis, arrange, spacing),
            ChildrenLayout::Grid { spacing_x, spacing_y, .. } => self.place_children_grid(i, rect, spacing_x, spacing_y),
        }
    }

    /// Places a freely placed node and its subtree, inside the rect of its parent. It panics if the node is not freely placed.
    fn place_free_node(&mut self, node: Self::NodeId, parent_rect: XyRect) {
        assert!(node != self.root(), "place_free_placed_subtree called on the root, which is not freely placed");
        let parent = self.parent(node);
        let is_free = self.node_layout_params(node).free_placement || self.node_layout_params(parent).children_layout == ChildrenLayout::Free;
        assert!(is_free, "place_free_placed_subtree called on {node:?}, which is not freely placed");
        let frame = self.parent_frame(parent, parent_rect);
        self.place_child_free(node, &frame, self.node_layout_data(node).size());
    }

    #[inline(always)]
    fn parent_frame(&self, i: Self::NodeId, rect: XyRect) -> ParentFrame {
        let params = self.node_layout_params(i);
        ParentFrame {
            rect,
            padding: params.padding,
            flipped: params.children_origin_flipped,
        }
    }

    fn place_children_stack(&mut self, i: Self::NodeId, rect: XyRect, axis: Axis, arrange: Arrange, spacing: f32) {
        let (main, cross) = (axis, axis.other());
        let frame = self.parent_frame(i, rect);
        let stack_rect = frame.rect;
        let [pad_start, pad_end] = frame.padding[main];

        // On the main axis, totally ignore the children's chosen Position's and place them according to our own Stack::Arrange value.

        let mut n: u32 = 0;
        let mut children_size = 0.0;
        for_each_layout_child!(self, i, child, {
            if ! self.node_layout_params(child).free_placement {
                children_size += self.node_layout_data(child).size()[main];
                n += 1;
            }
        });

        let total_size = if n > 0 {
            children_size + spacing * (n - 1) as f32
        } else {
            children_size
        };

        let (inner_start, inner_end) = (stack_rect[main][0] + pad_start, stack_rect[main][1] - pad_end);
        let free = (inner_end - inner_start) - children_size;

        let (mut walking_position, gap) = match arrange {
            Arrange::Start => (inner_start, spacing),
            Arrange::End => (inner_end - total_size, spacing),
            Arrange::Center => {
                let center = (inner_start + inner_end) / 2.0;
                (center - total_size / 2.0, spacing)
            },
            Arrange::SpaceBetween => {
                let gap = if n > 1 { free / (n - 1) as f32 } else { 0.0 };
                (inner_start, gap)
            },
            Arrange::SpaceAround => {
                let gap = if n > 0 { free / n as f32 } else { 0.0 };
                (inner_start + gap / 2.0, gap)
            },
            Arrange::SpaceEvenly => {
                let gap = if n > 0 { free / (n + 1) as f32 } else { 0.0 };
                (inner_start + gap, gap)
            },
        };

        for_each_layout_child!(self, i, child, {
            let child_size = self.node_layout_data(child).size();
            if self.node_layout_params(child).free_placement {
                self.place_child_free(child, &frame, child_size);
            } else {
                let mut child_rect = Xy::new([0.0, 0.0], [0.0, 0.0]);
                child_rect[cross] = self.resolve_pos_on_axis(&frame, child, cross, child_size[cross]);
                child_rect[main] = [walking_position, walking_position + child_size[main]];
                walking_position += child_size[main] + gap;
                self.place(child, child_rect, frame.rect);
            }
        });
    }

    fn place_children_grid(&mut self, i: Self::NodeId, rect: XyRect, spacing_x: f32, spacing_y: f32) {
        let lines = self.node_layout_data(i).grid_lines;

        let frame = self.parent_frame(i, rect);
        let spacing = Xy::new(spacing_x, spacing_y);
        let size = self.node_layout_data(i).size();
        let mut cell = Xy::new(0.0, 0.0);
        for axis in [X, Y] {
            let n = lines[axis] as f32;
            let [pad_start, pad_end] = frame.padding[axis];
            cell[axis] = ((size[axis] - pad_start - pad_end - spacing[axis] * (n - 1.0)) / n.max(1.0)).max(0.0);
        }

        for_each_layout_child!(self, i, child, {
            let child_size = self.node_layout_data(child).size();
            if self.node_layout_params(child).free_placement {
                self.place_child_free(child, &frame, child_size);
            } else {
                let child_cell = self.node_layout_data(child).grid_cell;
                let mut child_rect = Xy::new([0.0, 0.0], [0.0, 0.0]);
                for axis in [X, Y] {
                    let start = frame.rect[axis][0] + frame.padding[axis][0] + child_cell[axis] as f32 * (cell[axis] + spacing[axis]);
                    child_rect[axis] = [start, start + child_size[axis]];
                }
                self.place(child, child_rect, frame.rect);
            }
        });
    }

    #[inline(always)]
    fn resolve_pos_on_axis(&self, frame: &ParentFrame, child: Self::NodeId, axis: Axis, child_size: f32) -> [f32; 2] {
        let rect = frame.rect;
        let [pad_start, pad_end] = frame.padding[axis];
        let flipped = frame.flipped[axis];
        let params = self.node_layout_params(child);

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

        let (inner_start, inner_end) = (rect[axis][0] + pad_start, rect[axis][1] - pad_end);
        let origin_edge = if ! flipped { inner_start } else { inner_end };
        let far_edge = if ! flipped { inner_end } else { inner_start };

        match params.position[axis] {
            Pos::Start => {
                if ! flipped { [origin_edge, origin_edge + child_size] } else { [origin_edge - child_size, origin_edge] }
            },
            Pos::End => {
                if ! flipped { [far_edge - child_size, far_edge] } else { [far_edge, far_edge + child_size] }
            },
            Pos::Pixels(pixels) => {
                place_at(if ! flipped { origin_edge + pixels } else { origin_edge - pixels })
            },
            Pos::Frac(frac) => {
                let static_pos = frac * (inner_end - inner_start);
                place_at(if ! flipped { origin_edge + static_pos } else { origin_edge - static_pos })
            },
            Pos::Center => {
                let center = (inner_start + inner_end) / 2.0;
                [center - child_size / 2.0, center + child_size / 2.0]
            },
        }
    }

    #[inline(always)]
    fn place_child_free(&mut self, child: Self::NodeId, frame: &ParentFrame, child_size: Xy<f32>) {
        let mut child_rect = Xy::new([0.0, 0.0], [0.0, 0.0]);
        for axis in [X, Y] {
            let rect = self.resolve_pos_on_axis(frame, child, axis, child_size[axis]);
            child_rect[axis] = self.apply_window_overflow(frame, child, axis, rect);
        }
        self.place(child, child_rect, frame.rect);
    }

    fn apply_window_overflow(&self, frame: &ParentFrame, child: Self::NodeId, axis: Axis, rect: [f32; 2]) -> [f32; 2] {
        let params = self.node_layout_params(child);
        let behavior = params.window_overflow[axis];
        if behavior == WindowOverflow::Ignore {
            return rect;
        }

        let win_low = 0.0;
        let win_high = self.window_size()[axis];

        let [lo, hi] = rect;
        let size = hi - lo;

        // If the node is larger than the window, just give up.
        if size >= (win_high - win_low) {
            return [win_low, win_low + size];
        }

        let mut out = rect;

        if behavior == WindowOverflow::Flip {
            // The anchor point (in absolute coords) is the point that should stay fixed, e.g. the cursor.
            let anchor_frac = anchor_frac(params.anchor[axis]);
            let a = if ! frame.flipped[axis] { lo + anchor_frac * size } else { hi - anchor_frac * size };

            let reflected = [2.0 * a - hi, 2.0 * a - lo];
            if hi > win_high && reflected[0] >= win_low {
                out = reflected;
            } else if lo < win_low && reflected[1] <= win_high {
                out = reflected;
            }
        }

        // Clamp (also the fallback when a flip wouldn't fit either).
        let [mut lo, mut hi] = out;
        let size = hi - lo;
        if hi > win_high {
            hi = win_high;
            lo = hi - size;
        }
        if lo < win_low {
            lo = win_low;
            hi = lo + size;
        }
        [lo, hi]
    }

    fn place_children_free(&mut self, i: Self::NodeId, rect: XyRect) {
        let frame = self.parent_frame(i, rect);
        for_each_layout_child!(self, i, child, {
            let child_size = self.node_layout_data(child).size();
            self.place_child_free(child, &frame, child_size);
        });
    }
}

impl<T: TreeAdapter + ?Sized> Place for T {}

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
