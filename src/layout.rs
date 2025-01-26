use crate::*;
use crate::node::*;

use glyphon::Buffer as GlyphonBuffer;
use Axis::{X, Y};

const BIG_FLOAT: f32 = 1000.0;

/// Iterate on the children linked list.
macro_rules! for_each_child {
    ($ui:expr, $start:expr, $child:ident, $body:block) => {
        {
            let mut current_child = $start.first_child;
            while let Some($child) = current_child {
                $body
                current_child = $ui.nodes[$child].next_sibling;
            }
        }
    };
}

impl Ui {

    pub(crate) fn do_cosmetic_rect_updates(&mut self) {
        for idx in 0..self.sys.changes.cosmetic_rect_updates.len() {
            let update = self.sys.changes.cosmetic_rect_updates[idx];
            self.update_rect(update);
            log::info!("Visual rectangle update ({:?})", self.nodes[update].debug_name());
        }
    }

    // this gets called even when zero relayouts are needed. in that case it just does nothing. I guess it's to make the layout() logic more readable
    pub(crate) fn do_partial_relayouts(&mut self, update_rects_while_relayouting: bool) {
        self.sys.relayouts_scrath.clear();
        for n in &self.sys.changes.swapped_tree_changes {
            self.sys.relayouts_scrath.push(*n);
        }
        for n in &self.sys.changes.partial_relayouts {
            self.sys.relayouts_scrath.push(*n);
        }

        // sort by depth
        // todo: there was something about it being close to already sorted, except in reverse
        // the plan was to sort it in reverse and then use it in reverse
        self.sys.relayouts_scrath.sort();
        self.sys.partial_relayout_count = 0;

        for idx in 0..self.sys.relayouts_scrath.len() {
            // in partial_relayout(), we will check for overlaps.
            // todo: is that works as expected, maybe we can skip the limit/full relayout thing, or at least raise the limit by a lot.
            let relayout = self.sys.relayouts_scrath[idx];
            
            self.partial_relayout(relayout.i, update_rects_while_relayouting);
        }

        if self.sys.partial_relayout_count != 0 {
            let nodes = if self.sys.partial_relayout_count == 1 {
                "node"
            } else {
                "nodes"
            };
            log::info!("Partial relayout ({:?} {nodes})", self.sys.partial_relayout_count);
        }

        self.sys.partial_relayout_count = 0;
    }

    pub(crate) fn relayout(&mut self) {
        self.sys.changes.swap_thread_local_tree_changes();

        let tree_changed = ! self.sys.changes.swapped_tree_changes.is_empty();
        let rebuild_all_rects = tree_changed || self.sys.changes.rebuild_all_rects;
        let partial_relayouts = ! self.sys.changes.partial_relayouts.is_empty();
        let rect_updates = ! self.sys.changes.cosmetic_rect_updates.is_empty();
        let full_relayout = self.sys.changes.full_relayout;

        let nothing_to_do = ! tree_changed
            && ! partial_relayouts
            && ! rect_updates
            && ! full_relayout
            && ! rebuild_all_rects;
        if nothing_to_do {
            return;
        }

        // if anything happened at all, we'll need to rerender.
        self.sys.changes.need_gpu_rect_update = true;
        self.sys.changes.need_rerender = true;

        if full_relayout {
            self.relayout_from_root();
        } else {           
            if rebuild_all_rects {
                self.do_partial_relayouts(false);
            } else {
                self.do_partial_relayouts(true);
            }    
        }
        
        // reset these early, but resolve_hover has a chance to turn them back on
        // self.sys.new_ui_input = false;
        // self.sys.new_external_events = false;

        // after doing a relayout, we might be moving the hovered node away from the cursor.
        // So we run resolve_hover again, possibly causing another relayout next frame
        if tree_changed || partial_relayouts || full_relayout {
            self.resolve_hover();
        }

        // these ones are after the second-order-effect resolve_hover, just to see the update sooner.
        if full_relayout || rebuild_all_rects {
            self.rebuild_all_rects();

        } else {
            self.do_cosmetic_rect_updates();
        }

        self.sys.changes.reset_layout_changes();

        if tree_changed {
            // pruning here seems like an ok idea, but I haven't thought about it super hard yet.
            // in general, we could use info in tree_changes to do better pruning.
            // self.prune();
        }
    }

    pub(crate) fn relayout_from_root(&mut self) {
        // 1st recursive tree traversal: start from the root and recursively determine the size of all nodes
        // For the first node, assume that the proposed size that we got from the parent last frame is valid. (except for root, in which case just use the whole screen. todo: should just make full_relayout a separate function.)
        let starting_proposed_size = Xy::new(1.0, 1.0);

        self.recursive_determine_size(ROOT_I, starting_proposed_size);
        
        // 2nd recursive tree traversal: now that all nodes have a calculated size, place them.
        // we don't do update_rects here because the first frame you can't update... but maybe just special-case the first frame, then should be faster
        self.recursive_place_children(ROOT_I, false);
        
        self.nodes[ROOT_I].last_layout_frame = self.sys.current_frame;

    }

    pub(crate) fn partial_relayout(&mut self, node: usize, update_rects: bool) {
        // if the node has already been layouted on the current frame, stop immediately, and don't even recurse.
        // when doing partial layouts, this avoids overlap, but it means that we have to sort the partial relayouts cleanly from least depth to highest depth in order to get it right. This is done in `relayout()`.
        let current_frame = self.sys.current_frame;
        if self.nodes[node].last_layout_frame >= current_frame {
            return;
        }

        // 1st recursive tree traversal: start from the root and recursively determine the size of all nodes
        // For the first node, use the proposed size that we got from the parent last frame.
        let starting_proposed_size = self.nodes[node].last_proposed_size;
        self.recursive_determine_size(node, starting_proposed_size);
        
        // 2nd recursive tree traversal: now that all nodes have a calculated size, place them.
        self.recursive_place_children(node, update_rects);
    }

    fn get_proposed_size(&mut self, node: usize, proposed_size: Xy<f32>) -> Xy<f32> {
        let padding = self.to_frac2(self.nodes[node].params.layout.padding);
        let mut proposed_size = proposed_size;

        for axis in [X, Y] {
            // adjust proposed size based on padding
            proposed_size[axis] -= 2.0 * padding[axis];

            // adjust proposed size based on our own size
            match self.nodes[node].params.layout.size[axis] {
                Size::FitContent | Size::FitContentOrMinimum(_) => {}, // propose the whole available size. We will shrink our final size later if they end up using less or more 
                Size::Fill => {}, // propose the whole available size
                Size::Fixed(len) => {
                    proposed_size[axis] = self.len_to_frac_of_size(len, proposed_size, axis);
                }
                Size::AspectRatio(_aspect) => {
                    const ASPECT_RATIO_DEFAULT: f32 = 0.5;
                    proposed_size[axis] *= ASPECT_RATIO_DEFAULT;
                },
            }
        }

        // apply AspectRatio
        for axis in [X, Y] {
            match self.nodes[node].params.layout.size[axis] {
                Size::AspectRatio(aspect) => {
                    match self.nodes[node].params.layout.size[axis.other()] {
                        Size::AspectRatio(_second_aspect) => {
                            let debug_name = self.nodes[node].debug_name();
                            log::warn!("A Size shouldn't be AspectRatio in both dimensions. (node: {:?})", debug_name);
                        }
                        _ => {
                            let window_aspect = self.sys.unifs.size.x / self.sys.unifs.size.y;
                            let mult = match axis {
                                X => 1.0 / (window_aspect * aspect),
                                Y => window_aspect * aspect,
                            };
                            proposed_size[axis] = proposed_size[axis.other()] * mult;
                        }
                    }
                }
                _ => {}
            }
        }


        if let Some(stack) = self.nodes[node].params.stack {
            let main = stack.axis;
            let n_children = self.nodes[node].n_children as f32;
            let spacing = self.to_frac(stack.spacing, stack.axis);

            // adjust proposed size based on spacing
            if n_children > 1.5 {
                proposed_size[main] -= spacing * (n_children - 1.0);
            }
        }

        return proposed_size;
    }

    fn get_children_proposed_size(&mut self, node: usize, proposed_size: Xy<f32>) -> Xy<f32> {
        let mut child_proposed_size = proposed_size;

        if let Some(stack) = self.nodes[node].params.stack {
            let main = stack.axis;
            let n_children = self.nodes[node].n_children as f32;

            // divide between children
            child_proposed_size[main] /= n_children;
        }
        return child_proposed_size
    }

    fn recursive_determine_size(&mut self, node: usize, proposed_size: Xy<f32>) -> Xy<f32> {
        self.nodes[node].last_proposed_size = proposed_size;

        let stack = self.nodes[node].params.stack;
        
        // calculate the total size to propose to children
        let proposed_size = self.get_proposed_size(node, proposed_size);
        // divide it across children (if Stack)
        let child_proposed_size = self.get_children_proposed_size(node, proposed_size);

        let mut content_size = Xy::new(0.0, 0.0);
        // Propose a size to the children and let them decide
        for_each_child!(self, self.nodes[node], child, {
            let child_size = self.recursive_determine_size(child, child_proposed_size);
            content_size.update_for_child(child_size, stack);
        });

        // Propose the whole proposed_size (regardless of stack) to the contents, and let them decide.
        if self.nodes[node].text_id.is_some() {
            let text_size = self.determine_text_size(node, proposed_size);
            content_size.update_for_content(text_size);
        }
        if self.nodes[node].imageref.is_some() {
            let image_size = self.determine_image_size(node, proposed_size);
            content_size.update_for_content(image_size);
        }

        self.nodes[node].content_size = content_size;

        // Decide our own size. 
        //   We either use the proposed_size that we proposed to the children,
        //   or we change our mind to based on children.
        // todo: is we're not fitcontenting, we can skip the update_for_* calls instead, and then remove this, I guess.
        let mut final_size = proposed_size;
        for axis in [X, Y] {
            match self.nodes[node].params.layout.size[axis] {
                Size::FitContent => {
                    final_size[axis] = content_size[axis];
                }
                Size::FitContentOrMinimum(min_size) => {
                    let min_size = self.len_to_frac_of_size(min_size, proposed_size, axis);
                    final_size[axis] = content_size[axis].max(min_size);
                }
                _ => {},
            }
        }

        // add back padding to get the real final size
        final_size = self.adjust_final_size(node, final_size);


        self.nodes[node].size = final_size;
        return final_size;
    }

    fn determine_image_size(&mut self, node: usize, _proposed_size: Xy<f32>) -> Xy<f32> {
        let image_ref = self.nodes[node].imageref.unwrap();
        let size = image_ref.original_size;
        return self.f32_pixels_to_frac2(size);
    }

    fn determine_text_size(&mut self, node: usize, proposed_size: Xy<f32>) -> Xy<f32> {
        let text_id = self.nodes[node].text_id.unwrap();
        let buffer = &mut self.sys.text.text_areas[text_id].buffer;

        // this is for FitContent on both directions, basically.
        // todo: the rest.
        // also, note: the set_align trick might not be good if we expose the ability to set whatever align the user wants.

        let h = match self.nodes[node].params.layout.size[Y] {
            Size::FitContent => BIG_FLOAT,
            Size::FitContentOrMinimum(_min_size) => todo!(""),
            _ => proposed_size.x * self.sys.unifs.size[X],
        };

        let w = match self.nodes[node].params.layout.size[X] {
            Size::FitContent => {
                match self.nodes[node].params.text_params.unwrap_or(TextOptions::default()).single_line {
                    true => BIG_FLOAT,
                    false => proposed_size.x * self.sys.unifs.size[X],
                }
            },
            Size::FitContentOrMinimum(_min_size) => todo!(""),
            _ => proposed_size.x * self.sys.unifs.size[X],
        };

        for line in &mut buffer.lines {
            line.set_align(Some(glyphon::cosmic_text::Align::Left));
        }

        buffer.set_size(&mut self.sys.text.font_system, Some(w), Some(h));

        // let now = std::time::Instant::now();
        buffer.shape_until_scroll(&mut self.sys.text.font_system, false);
        // log::trace!("Shape text buffer {:?}", now.elapsed());

        let trimmed_size = buffer.measure_text_pixels();


        // idk if this line is needed
        buffer.set_size(&mut self.sys.text.font_system, Some(trimmed_size.x), Some(trimmed_size.y));

        // self.sys.text.text_areas[text_id].buffer.set_size(&mut self.sys.text.font_system, trimmed_size.x, trimmed_size.y);
        // self.sys.text.text_areas[text_id]
        //     .buffer
        //     .shape_until_scroll(&mut self.sys.text.font_system, false);

        // for axis in [X, Y] {
        //     trimmed_size[axis] *= 2.0;
        // }

        return self.f32_pixels_to_frac2(trimmed_size);
    }

    fn adjust_final_size(&mut self, node: usize, final_size: Xy<f32>) -> Xy<f32> {
        // re-add spacing and padding to the final size we calculated
        let mut final_size = final_size;

        let padding = self.to_frac2(self.nodes[node].params.layout.padding);
        for axis in [X, Y] {
            final_size[axis] += 2.0 * padding[axis];
        }

        if let Some(stack) = self.nodes[node].params.stack {
            let spacing = self.to_frac(stack.spacing, stack.axis);
            let n_children = self.nodes[node].n_children as f32;
            let main = stack.axis;

            if n_children > 1.0 {
                final_size[main] += spacing * (n_children - 1.0);
            }
        }

        return final_size;
    }

    pub(crate) fn recursive_place_children(&mut self, node: usize, also_update_rects: bool) {
        self.sys.partial_relayout_count += 1;
        if let Some(stack) = self.nodes[node].params.stack {
            self.place_children_stack(node, stack);
        } else {
            self.place_children_container(node);
        };

        // self.place_image(node); // I think there's nothing to place? right now it's always the full rect
        self.place_text_inside(node, self.nodes[node].rect);
    
        if also_update_rects {
            self.update_rect(node);
        }

        self.set_clip_rect(node);
            
        self.nodes[node].last_layout_frame = self.sys.current_frame;

        for_each_child!(self, self.nodes[node], child, {
            self.recursive_place_children(child, also_update_rects);
        });
    }

    fn place_children_stack(&mut self, node: usize, stack: Stack) {
        let (main, cross) = (stack.axis, stack.axis.other());
        let mut containing_rect = self.nodes[node].rect;
        
        for axis in [X, Y] {
            if self.nodes[node].params.layout.scrollable[axis] {
                let scroll_offset = self.nodes[node].scroll.absolute_offset(axis);
                containing_rect[axis][0] += scroll_offset;
                containing_rect[axis][1] += scroll_offset;
            }
        }

        let padding = self.to_frac2(self.nodes[node].params.layout.padding);
        let spacing = self.to_frac(stack.spacing, stack.axis);
        
        // On the main axis, totally ignore the children's chosen Position's and place them according to our own Stack::Arrange value.

        // collect all the children sizes in a vec
        let n = self.nodes[node].n_children;
        self.sys.size_scratch.clear();
        for_each_child!(self, self.nodes[node], child, {
            self.sys.size_scratch.push(self.nodes[child].size[main]);
        });

        let mut total_size = 0.0;
        for s in &self.sys.size_scratch {
            total_size += s;
        }
        if n > 0 {
            total_size += spacing * (n - 1) as f32;
        }

        let mut main_origin = match stack.arrange {
            Arrange::Start => containing_rect[main][0] + padding[main],
            Arrange::End => containing_rect[main][1] + padding[main] - total_size,
            Arrange::Center => {
                let center = (containing_rect[main][1] + containing_rect[main][0]) / 2.0 - 2.0 * padding[main];
                center - total_size / 2.0
            },
            _ => todo!(),
        };

        for_each_child!(self, self.nodes[node], child, {
            let child_size = self.nodes[child].size;

            match self.nodes[child].params.layout.position[cross] {
                Position::Center => {
                    let origin = (containing_rect[cross][1] + containing_rect[cross][0]) / 2.0;
                    self.nodes[child].rect[cross] = [
                        origin - child_size[cross] / 2.0 ,
                        origin + child_size[cross] / 2.0 ,
                    ];  
                },
                Position::Start => {
                    let origin = containing_rect[cross][0] + padding[cross];
                    self.nodes[child].rect[cross] = [origin, origin + child_size[cross]];         
                },
                Position::Static(len) => {
                    let static_pos = self.len_to_frac_of_size(len, containing_rect.size(), cross);
                    let origin = containing_rect[cross][0] + padding[cross] + static_pos;
                    self.nodes[child].rect[cross] = [origin, origin + child_size[cross]];  
                },
                Position::End => {
                    let origin = containing_rect[cross][1] - padding[cross];
                    self.nodes[child].rect[cross] = [origin - child_size[cross], origin];
                },
            }

            self.nodes[child].rect[main] = [main_origin, main_origin + child_size[main]];

            main_origin += self.nodes[child].size[main] + spacing;
        });
    }

    fn place_children_container(&mut self, node: usize) {

        let parent_rect = self.nodes[node].rect;

        let padding = self.to_frac2(self.nodes[node].params.layout.padding);

        let mut content_bounding_rect = XyRect::new([f32::MAX, -f32::MAX], [f32::MAX, -f32::MAX]);
        let mut origin = Xy::<f32>::default();

        for_each_child!(self, self.nodes[node], child, {
            let child_size = self.nodes[child].size;

            // check the children's chosen Position's and place them.
            for axis in [X, Y] {
                match self.nodes[child].params.layout.position[axis] {
                    Position::Start => {
                        origin[axis] = parent_rect[axis][0] + padding[axis];
                        self.nodes[child].rect[axis] = [origin[axis], origin[axis] + child_size[axis]];         
                    },
                    Position::Static(len) => {
                        let static_pos = self.len_to_frac_of_size(len, parent_rect.size(), axis);
                        origin[axis] = parent_rect[axis][0] + padding[axis] + static_pos;
                        self.nodes[child].rect[axis] = [origin[axis], origin[axis] + child_size[axis]];
                    }
                    Position::End => {
                        origin[axis] = parent_rect[axis][1] - padding[axis];
                        self.nodes[child].rect[axis] = [origin[axis] - child_size[axis], origin[axis]];
                    },
                    Position::Center => {
                        origin[axis] = (parent_rect[axis][0] + parent_rect[axis][1]) / 2.0;
                        self.nodes[child].rect[axis] = [
                            origin[axis] - child_size[axis] / 2.0 ,
                            origin[axis] + child_size[axis] / 2.0 ,
                        ];           
                    },
                }
    
                if self.nodes[node].params.layout.scrollable[axis] {
                    // todo: try using the remembered content_size and origin instead of doing this
                    content_bounding_rect.update_bounding_rect(axis, self.nodes[child].rect);
                }
            }
        });

        self.nodes[node].scroll.limits = self.scroll_limits(node, content_bounding_rect);

        self.set_children_scroll(node);
    }

    // doesnt work lol
    // pub(crate) fn recursive_set_scroll(&mut self, node: usize) {
    //     self.set_children_scroll(node);

    //     for_each_child!(self, self.nodes[node], child, {
    //         self.recursive_set_scroll(child);
    //     });
    // }

    fn set_children_scroll(&mut self, node: usize) {
        if ! self.nodes[node].params.is_scrollable() {
            return;
        }

        for_each_child!(self, self.nodes[node], child, {
            for axis in [X, Y] {
                if self.nodes[node].params.layout.scrollable[axis] {
                    let scroll_offset = self.nodes[node].scroll.absolute_offset(axis);
                    self.nodes[child].rect[axis][0] += scroll_offset;
                    self.nodes[child].rect[axis][1] += scroll_offset;
                }
            }
            self.update_rect(child);

        });
        
    }
    fn set_clip_rect(&mut self, node: usize) {
        let parent_clip_rect;
        if node == ROOT_I {
            parent_clip_rect = Xy::new_symm([0.0, 1.0]);
        } else {
            let parent = self.nodes[node].parent;
            parent_clip_rect = self.nodes[parent].clip_rect;
        }

        let own_rect = self.nodes[node].rect;
        for axis in [X, Y] {
            if self.nodes[node].params.layout.scrollable[axis] {
                self.nodes[node].clip_rect[axis] = intersect(own_rect[axis], parent_clip_rect[axis])
            } else {
                self.nodes[node].clip_rect = parent_clip_rect;
            }
        }

        // text
        let left = self.nodes[node].clip_rect[X][0] * self.sys.unifs.size[X];
        let right = self.nodes[node].clip_rect[X][1] * self.sys.unifs.size[X];
        let top = self.nodes[node].clip_rect[Y][0] * self.sys.unifs.size[Y];
        let bottom = self.nodes[node].clip_rect[Y][1] * self.sys.unifs.size[Y];

        if let Some(text_id) = self.nodes[node].text_id {
            self.sys.text.text_areas[text_id].params.bounds.left = left as i32;
            self.sys.text.text_areas[text_id].params.bounds.top = top as i32;
            self.sys.text.text_areas[text_id].params.bounds.right = right as i32;
            self.sys.text.text_areas[text_id].params.bounds.bottom = bottom as i32;
        }
    }

    #[allow(dead_code)]
    pub(crate) fn place_image(&mut self, _node: usize) {     
        // might be something here in the future
    }

    fn place_text_inside(&mut self, node: usize, rect: XyRect) {
        let padding = self.to_pixels2(self.nodes[node].params.layout.padding);

        let mut containing_rect = rect;
        let mut content_bounding_rect = XyRect::new([f32::MAX, -f32::MAX], [f32::MAX, -f32::MAX]);

        for axis in [X, Y] {
            if self.nodes[node].params.layout.scrollable[axis] {
                let scroll_offset = self.nodes[node].scroll.absolute_offset(axis);
                containing_rect[axis][0] += scroll_offset;
                containing_rect[axis][1] += scroll_offset;
            }
        }
        
        let text_id = self.nodes[node].text_id;
        if let Some(text_id) = text_id {
            let left = containing_rect[X][0] * self.sys.unifs.size[X];
            let top = containing_rect[Y][0] * self.sys.unifs.size[Y];

            // let right = rect[X][1] * self.sys.unifs.size[X];
            // let bottom =     rect[Y][1] * self.sys.unifs.size[Y];

            self.sys.text.text_areas[text_id].params.left = left + padding[X] as f32;
            self.sys.text.text_areas[text_id].params.top = top + padding[Y] as f32;
           
            // todo: different align? 
            // self.sys.text.text_areas[text_id].bounds.left = left as i32 + padding[X] as i32;
            // self.sys.text.text_areas[text_id].bounds.top = top as i32 + padding[Y] as i32;

            // self.sys.text.text_areas[text_id].bounds.right = right as i32;
            // self.sys.text.text_areas[text_id].bounds.bottom = bottom as i32;
        }
    }

    pub(crate) fn rebuild_all_rects(&mut self) {
        log::info!("Rebuilding all rectangles");
        self.sys.rects.clear();
        self.sys.invisible_but_clickable_rects.clear();
        self.sys.scroll_rects.clear();
        self.sys.z_cursor = Z_BACKDROP;
        self.recursive_push_rects(ROOT_I);
    }

    fn recursive_push_rects(&mut self, node: usize) {
        // 3nd recursive tree traversal: now that all nodes have a calculated size, place them.
        self.push_rect(node);

        for_each_child!(self, self.nodes[node], child, {
            self.recursive_push_rects(child);
        });
    }
}



impl Xy<f32> {
    pub(crate) fn update_for_child(&mut self, child_size: Xy<f32>, stack: Option<Stack>) {
        match stack {
            None => {
                for axis in [X, Y] {
                    if child_size[axis] > self[axis] {
                        self[axis] = child_size[axis];
                    }
                }
            },
            Some(stack) => {
                let (main, cross) = (stack.axis, stack.axis.other());

                self[main] += child_size[main];
                if child_size[cross] > self[cross] {
                    self[cross] = child_size[cross];
                }
            },
        }
    }
    pub(crate) fn update_for_content(&mut self, content_size: Xy<f32>) {
        for axis in [X, Y] {
            if content_size[axis] > self[axis] {
                self[axis] = content_size[axis];
            }
        }
    }

}

impl XyRect {
    fn update_bounding_rect(&mut self, axis: Axis, child_rect: XyRect) {
        let diff = child_rect[axis][0];
        if diff < self[axis][0] {
            self[axis][0] = diff;
        }

        let diff = child_rect[axis][1];
        if diff > self[axis][1] {
            self[axis][1] = diff;
        }
    }
}

#[derive(Debug)]
pub(crate) struct Scroll {
    limits: ScrollLimits,
    relative_offset: Xy<f32>,
}
impl Scroll {
    pub const ZERO: Scroll = Scroll {
        limits: ScrollLimits(XyRect::new([0.0, 0.0], [0.0, 0.0])),
        relative_offset: Xy::new(0.0, 0.0),
    };

    pub fn update(&mut self, delta: f32, axis: Axis) {
        let scroll_space = self.limits.scroll_space(axis);
        self.relative_offset[axis] += delta / scroll_space;

        let scroll_space = self.limits.scroll_space(axis);
        let min_scroll = self.limits.min_scroll(axis) / scroll_space;
        let max_scroll = self.limits.max_scroll(axis) / scroll_space;
        let scroll = &mut self.relative_offset[axis];
        *scroll = scroll.clamp(min_scroll, max_scroll);
    }

    pub fn absolute_offset(&self, axis: Axis) -> f32 {
        let scroll_offset = self.relative_offset[axis];
        let scroll_space = self.limits.scroll_space(axis);
        return scroll_offset * scroll_space;
    }
}

#[derive(Debug)]
pub(crate) struct ScrollLimits(XyRect);
impl ScrollLimits {
    pub fn min_scroll(&self, axis: Axis) -> f32 {
        return self.0[axis][0];
    }
    pub fn max_scroll(&self, axis: Axis) -> f32 {
        return self.0[axis][1];
    }
    pub fn scroll_space(&self, axis: Axis) -> f32 {
        return self.max_scroll(axis) - self.min_scroll(axis);
    }
}


impl Ui {
    fn scroll_limits(&mut self, node: usize, content_bounding_rect: XyRect) -> ScrollLimits {
        let mut scroll_limits = XyRect::new([0.0, 0.0], [0.0, 0.0]);

        for axis in [X, Y] {
            if self.nodes[node].params.layout.scrollable[axis] {
                let min_scroll = self.nodes[node].rect[axis][1] - content_bounding_rect[axis][1];
                let min_scroll = min_scroll.clamp(-f32::MAX, 0.0);
                
                let max_scroll = self.nodes[node].rect[axis][0] - content_bounding_rect[axis][0];
                let max_scroll = max_scroll.clamp(0.0, f32::MAX);

                scroll_limits[axis][0] = min_scroll;
                scroll_limits[axis][1] = max_scroll;
            }
        }
        return ScrollLimits(scroll_limits);
    }
}

pub trait MeasureText {
    fn measure_text_pixels(&self) -> Xy<f32>;
}
impl MeasureText for GlyphonBuffer {
    fn measure_text_pixels(&self) -> Xy<f32> {
        let layout_runs = self.layout_runs();
        let mut total_width: f32 = 0.;
        let mut total_height: f32 = 0.;
        for run in layout_runs {
            total_width = total_width.max(run.line_w);
            total_height += run.line_height;

        }
        return Xy::new(total_width.ceil(), total_height)
    }
}