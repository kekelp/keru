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

        // these ones are after the second-order-effect resolve_hover, just to see the update sooner.
        if full_relayout || rebuild_all_rects {
            self.rebuild_all_rects();

        } else {
            self.do_cosmetic_rect_updates();
        }

        self.sys.changes.reset_layout_changes();

        // after doing a relayout, we might be moving the hovered node away from the cursor.
        // So we run resolve_hover again, possibly causing another relayout next frame
        if tree_changed || partial_relayouts || full_relayout {
            self.resolve_hover();
        }

        if tree_changed {
            // pruning here seems like an ok idea, but I haven't thought about it super hard yet.
            // in general, we could use info in tree_changes to do better pruning.
            // self.prune();
        }
    }

    pub(crate) fn do_cosmetic_rect_updates(&mut self) {
        for idx in 0..self.sys.changes.cosmetic_rect_updates.len() {
            let update = self.sys.changes.cosmetic_rect_updates[idx];
            self.update_rect(update);
            log::info!("Visual rectangle update ({})", self.node_debug_name(update));
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
            // todo: if that works as expected, maybe we can skip the limit/full relayout thing, or at least raise the limit by a lot.
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

    pub(crate) fn relayout_from_root(&mut self) {
        // 1st recursive tree traversal: start from the root and recursively determine the size of all nodes
        // For the first node, assume that the proposed size that we got from the parent last frame is valid. (except for root, in which case just use the whole screen. todo: should just make full_relayout a separate function.)
        let starting_proposed_size = Xy::new(1.0, 1.0);

        self.recursive_determine_size(ROOT_I, ProposedSizes::container(starting_proposed_size));
        
        // 2nd recursive tree traversal: now that all nodes have a calculated size, place them.
        // we don't do update_rects here because the first frame you can't update... but maybe just special-case the first frame, then should be faster
        self.recursive_place_children(ROOT_I, false);
        
        self.nodes[ROOT_I].last_layout_frame = self.sys.current_frame;

    }

    pub(crate) fn partial_relayout(&mut self, i: NodeI, update_rects: bool) {
        // if the node has already been layouted on the current frame, stop immediately, and don't even recurse.
        // when doing partial layouts, this avoids overlap, but it means that we have to sort the partial relayouts cleanly from least depth to highest depth in order to get it right. This is done in `relayout()`.
        let current_frame = self.sys.current_frame;
        if self.nodes[i].last_layout_frame >= current_frame {
            return;
        }

        // 1st recursive tree traversal: start from the root and recursively determine the size of all nodes
        // For the first node, use the proposed size that we got from the parent last frame.
        let starting_proposed_size = self.nodes[i].last_proposed_sizes;
        self.recursive_determine_size(i, starting_proposed_size);
        
        // 2nd recursive tree traversal: now that all nodes have a calculated size, place them.
        self.recursive_place_children(i, update_rects);
    }

    fn get_size(
        &mut self,
        i: NodeI,    
        child_proposed_size: Xy<f32>, // the size that was proposed to us specifically after dividing between children
        whole_parent_proposed_size: Xy<f32>, // the whole size that the parent proposed to ALL its children collectively
    ) -> Xy<f32> {
        let mut size = child_proposed_size; // this default value is mostly useless

        for axis in [X, Y] {
            match self.nodes[i].params.layout.size[axis] {
                Size::FitContent => {
                    size[axis] = child_proposed_size[axis];
                }, // propose the whole available size. We will shrink our final size later if they end up using less or more 
                Size::Fill => {
                    size[axis] = child_proposed_size[axis]; // use the whole available size
                },
                Size::Pixels(pixels) => {
                    size[axis] = self.pixels_to_frac(pixels, axis); // ignore the proposed size and force our pixel size
                },
                Size::Frac(frac) => {
                    size[axis] = whole_parent_proposed_size[axis] * frac;
                }
                Size::AspectRatio(_aspect) => {} // do nothing
            }
        }

        // apply AspectRatio
        for axis in [X, Y] {
            match self.nodes[i].params.layout.size[axis] {
                Size::AspectRatio(aspect) => {
                    match self.nodes[i].params.layout.size[axis.other()] {
                        Size::AspectRatio(_second_aspect) => {
                            let debug_name = self.node_debug_name(i);
                            log::warn!("A Size shouldn't be AspectRatio in both dimensions. (node: {})", debug_name);
                        }
                        _ => {
                            let window_aspect = self.sys.unifs.size.x / self.sys.unifs.size.y;
                            let mult = match axis {
                                X => 1.0 / (window_aspect * aspect),
                                Y => window_aspect * aspect,
                            };
                            size[axis] = size[axis.other()] * mult;
                        }
                    }
                }
                _ => {}
            }
        }

        return size;
    }

    fn get_inner_size(&mut self, i: NodeI, size: Xy<f32>) -> Xy<f32> {
        let mut inner_size = size;

        // remove padding
        let padding = self.pixels_to_frac2(self.nodes[i].params.layout.padding);
        for axis in [X, Y] {
            inner_size[axis] -= 2.0 * padding[axis];
        }

        // remove stack spacing
        if let Some(stack) = self.nodes[i].params.stack {
            let n_children = self.nodes[i].n_children as f32;
            let spacing = self.pixels_to_frac(stack.spacing, stack.axis);

            if n_children > 1.5 {
                inner_size[stack.axis] -= spacing * (n_children - 1.0);
            }
        }

        return inner_size;
    }

    fn recursive_determine_size(
        &mut self,
        i: NodeI,
        proposed_sizes: ProposedSizes,
    ) -> Xy<f32> {
        self.nodes[i].last_proposed_sizes = proposed_sizes;

        let size = self.get_size(i, proposed_sizes.to_this_child, proposed_sizes.to_all_children);
        let size_to_propose = self.get_inner_size(i, size);

        let stack = self.nodes[i].params.stack;
        let padding = self.pixels_to_frac2(self.nodes[i].params.layout.padding);
        let mut content_size = Xy::new(0.0, 0.0);

        if let Some(stack) = stack {
            let spacing = self.pixels_to_frac(stack.spacing, stack.axis);

            let mut available_size_left = size_to_propose;
            let mut n_added_children = 0;
            let mut n_fill_children = 0;
            // First, do all non-Fill children
            for_each_child!(self, self.nodes[i], child, {
                if self.nodes[child].params.layout.size[stack.axis] != Size::Fill {
                    let child_size = self.recursive_determine_size(child, ProposedSizes::stack(available_size_left, size_to_propose));
                    content_size.update_for_child(child_size, Some(stack));
                    if n_added_children != 0 {
                        content_size[stack.axis] += spacing;
                    }
                    available_size_left[stack.axis] -= child_size[stack.axis];
                    available_size_left[stack.axis] -= padding[stack.axis];
                    n_added_children += 1;
                } else {
                    n_fill_children += 1;
                }
            });

            
            if n_fill_children > 0 {
                // then, divide the remaining space between the Fill children
                let mut size_per_child = available_size_left;
                if n_fill_children > 1 {
                    available_size_left[stack.axis] -= ((n_fill_children - 1) as f32) * padding[stack.axis];
                }

                size_per_child[stack.axis] /= n_fill_children as f32;
                for_each_child!(self, self.nodes[i], child, {
                    if self.nodes[child].params.layout.size[stack.axis] == Size::Fill {
                        let child_size = self.recursive_determine_size(child, ProposedSizes::stack(size_per_child, size_to_propose));
                        content_size.update_for_child(child_size, Some(stack));
                        if n_added_children != 0 {
                            content_size[stack.axis] += spacing;
                        }
                        available_size_left[stack.axis] -= child_size[stack.axis];
                        n_added_children += 1;
                    }
                });
            }

        } else {
            // Propose a size to the children and let them decide
            for_each_child!(self, self.nodes[i], child, {
                let child_size = self.recursive_determine_size(child, ProposedSizes::container(size_to_propose));
                content_size.update_for_child(child_size, stack); // this is None
            });            
            
            // Propose the whole size_to_propose to the contents, and let them decide.
            if self.nodes[i].text_id.is_some() {
                let text_size = self.determine_text_size(i, size_to_propose);
                content_size.update_for_content(text_size);
            }
            if self.nodes[i].imageref.is_some() {
                let image_size = self.determine_image_size(i, size_to_propose);
                content_size.update_for_content(image_size);
            }
        }

        // Decide our own size. 
        //   We either use the size that we decided before, or we change our mind to based on children.
        // todo: is we're not fitcontenting, we can skip the update_for_* calls instead, and then remove this, I guess.
        let mut final_size = size;

        for axis in [X, Y] {
            match self.nodes[i].params.layout.size[axis] { // todo if let
                Size::FitContent => {
                    // if we use content_size instead of the size above, then content_size doesn't have padding in
                    let mut content_size_with_padding = content_size;
                    content_size_with_padding[axis] += 2.0 * padding[axis];
                    final_size[axis] = content_size_with_padding[axis];
                }
                _ => {},
            }
        }

        self.nodes[i].size = final_size;
        return final_size;
    }

    fn determine_image_size(&mut self, i: NodeI, _proposed_size: Xy<f32>) -> Xy<f32> {
        let image_ref = self.nodes[i].imageref.unwrap();
        let size = image_ref.original_size;
        return self.f32_pixels_to_frac2(size);
    }

    fn determine_text_size(&mut self, i: NodeI, proposed_size: Xy<f32>) -> Xy<f32> {
        let text_id = self.nodes[i].text_id.unwrap();
        let buffer = &mut self.sys.text.text_areas[text_id].buffer;

        // this is for FitContent on both directions, basically.
        // todo: the rest.
        // also, note: the set_align trick might not be good if we expose the ability to set whatever align the user wants.

        let h = match self.nodes[i].params.layout.size[Y] {
            Size::FitContent => BIG_FLOAT,
            _ => proposed_size.x * self.sys.unifs.size[X],
        };

        let w = match self.nodes[i].params.layout.size[X] {
            Size::FitContent => {
                match self.nodes[i].params.text_params.unwrap_or(TextOptions::default()).single_line {
                    true => BIG_FLOAT,
                    false => proposed_size.x * self.sys.unifs.size[X],
                }
            },
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

    pub(crate) fn recursive_place_children(&mut self, i: NodeI, also_update_rects: bool) {
        self.nodes[i].content_bounds = XyRect::new_symm([f32::MAX, f32::MIN]);


        self.sys.partial_relayout_count += 1;
        if let Some(stack) = self.nodes[i].params.stack {
            self.place_children_stack(i, stack);
        } else {
            self.place_children_container(i);
        };

        // self.place_image(i); // I think there's nothing to place? right now it's always the full rect
        self.place_text_inside(i, self.nodes[i].rect);
    
        if also_update_rects {
            self.update_rect(i);
        }

        self.set_clip_rect(i);
            
        self.nodes[i].last_layout_frame = self.sys.current_frame;

        for_each_child!(self, self.nodes[i], child, {
            self.recursive_place_children(child, also_update_rects);
        });
    }

    fn place_children_stack(&mut self, i: NodeI, stack: Stack) {
        let (main, cross) = (stack.axis, stack.axis.other());
        let stack_rect = self.nodes[i].rect;

        let padding = self.pixels_to_frac2(self.nodes[i].params.layout.padding);
        let spacing = self.pixels_to_frac(stack.spacing, stack.axis);
        
        // On the main axis, totally ignore the children's chosen Position's and place them according to our own Stack::Arrange value.
        
        let n = self.nodes[i].n_children;
        let mut total_size = 0.0;
        for_each_child!(self, self.nodes[i], child, {
            total_size += self.nodes[child].size[main];
        });

        if n > 0 {
            total_size += spacing * (n - 1) as f32;
        }

        let mut walking_position = match stack.arrange {
            Arrange::Start => stack_rect[main][0] + padding[main],
            Arrange::End => stack_rect[main][1] + padding[main] - total_size,
            Arrange::Center => {
                let center = (stack_rect[main][1] + stack_rect[main][0]) / 2.0 - 2.0 * padding[main];
                center - total_size / 2.0
            },
            _ => todo!(),
        };

        for_each_child!(self, self.nodes[i], child, {
            let child_size = self.nodes[child].size;

            match self.nodes[child].params.layout.position[cross] {
                Position::Center => {
                    let origin = (stack_rect[cross][1] + stack_rect[cross][0]) / 2.0;
                    self.nodes[child].rect[cross] = [
                        origin - child_size[cross] / 2.0 ,
                        origin + child_size[cross] / 2.0 ,
                    ];  
                },
                Position::Start => {
                    let origin = stack_rect[cross][0] + padding[cross];
                    self.nodes[child].rect[cross] = [origin, origin + child_size[cross]];         
                },
                Position::Static(len) => {
                    let static_pos = self.len_to_frac_of_size(len, stack_rect.size(), cross);
                    let origin = stack_rect[cross][0] + padding[cross] + static_pos;
                    self.nodes[child].rect[cross] = [origin, origin + child_size[cross]];  
                },
                Position::End => {
                    let origin = stack_rect[cross][1] - padding[cross];
                    self.nodes[child].rect[cross] = [origin - child_size[cross], origin];
                },
            }

            self.nodes[child].rect[main] = [walking_position, walking_position + child_size[main]];

            walking_position += self.nodes[child].size[main] + spacing;

            self.update_content_bounds(i, self.nodes[child].rect);
        });

        self.set_children_scroll(i);
    }

    fn place_children_container(&mut self, i: NodeI) {

        let parent_rect = self.nodes[i].rect;

        let padding = self.pixels_to_frac2(self.nodes[i].params.layout.padding);

        let mut origin = Xy::<f32>::default();

        for_each_child!(self, self.nodes[i], child, {
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
            }

            self.update_content_bounds(i, self.nodes[child].rect);
        });

        self.set_children_scroll(i);
    }

    #[inline]
    fn update_content_bounds(&mut self, i: NodeI, content_rect: XyRect) {
        for axis in [X, Y] {
            let c_bounds = &mut self.nodes[i].content_bounds[axis];
            c_bounds[0] = c_bounds[0].min(content_rect[axis][0]);
            c_bounds[1] = c_bounds[1].max(content_rect[axis][1]);
        }
    }

    // doesnt work lol
    // pub(crate) fn recursive_set_scroll(&mut self, i: NodeI) {
    //     self.set_children_scroll(i);

    //     for_each_child!(self, self.nodes[i], child, {
    //         self.recursive_set_scroll(child);
    //     });
    // }

    fn set_children_scroll(&mut self, i: NodeI) {
        if ! self.nodes[i].params.is_scrollable() {
            return;
        }
        self.clamp_scroll(i);

        for_each_child!(self, self.nodes[i], child, {
            for axis in [X, Y] {
                if self.nodes[i].params.layout.scrollable[axis] {
                    let scroll_offset = self.scroll_offset(i, axis);
                    self.nodes[child].rect[axis][0] += scroll_offset;
                    self.nodes[child].rect[axis][1] += scroll_offset;
                }
            }
            // self.update_rect(child);

        });
        
    }
    fn set_clip_rect(&mut self, i: NodeI) {
        let parent_clip_rect;
        if i == ROOT_I {
            parent_clip_rect = Xy::new_symm([0.0, 1.0]);
        } else {
            let parent = self.nodes[i].parent;
            parent_clip_rect = self.nodes[parent].clip_rect;
        }

        let own_rect = self.nodes[i].rect;
        for axis in [X, Y] {
            if self.nodes[i].params.layout.scrollable[axis] {
                self.nodes[i].clip_rect[axis] = intersect(own_rect[axis], parent_clip_rect[axis])
            } else {
                self.nodes[i].clip_rect = parent_clip_rect;
            }
        }

        // text
        let left = self.nodes[i].clip_rect[X][0] * self.sys.unifs.size[X];
        let right = self.nodes[i].clip_rect[X][1] * self.sys.unifs.size[X];
        let top = self.nodes[i].clip_rect[Y][0] * self.sys.unifs.size[Y];
        let bottom = self.nodes[i].clip_rect[Y][1] * self.sys.unifs.size[Y];

        if let Some(text_id) = self.nodes[i].text_id {
            self.sys.text.text_areas[text_id].params.bounds.left = left as i32;
            self.sys.text.text_areas[text_id].params.bounds.top = top as i32;
            self.sys.text.text_areas[text_id].params.bounds.right = right as i32;
            self.sys.text.text_areas[text_id].params.bounds.bottom = bottom as i32;
        }
    }

    #[allow(dead_code)]
    pub(crate) fn place_image(&mut self, _i: NodeI) {     
        // might be something here in the future
    }

    fn place_text_inside(&mut self, i: NodeI, rect: XyRect) {
        let padding = self.nodes[i].params.layout.padding;

        // for axis in [X, Y] {
        //     if self.nodes[i].params.layout.scrollable[axis] {
        //         let scroll_offset = self.nodes[i].scroll.absolute_offset(axis);
        //         containing_rect[axis][0] += scroll_offset;
        //         containing_rect[axis][1] += scroll_offset;
        //     }
        // }
        
        let text_id = self.nodes[i].text_id;
        if let Some(text_id) = text_id {
            let left = rect[X][0] * self.sys.unifs.size[X];
            let top = rect[Y][0] * self.sys.unifs.size[Y];

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
        self.sys.click_rects.clear();
        self.sys.invisible_but_clickable_rects.clear();
        self.sys.scroll_rects.clear();
        self.sys.z_cursor = Z_BACKDROP;
        self.recursive_push_rects(ROOT_I);
    }

    fn recursive_push_rects(&mut self, i: NodeI) {
        // 3nd recursive tree traversal: now that all nodes have a calculated size, place them.
        self.push_rect(i);

        for_each_child!(self, self.nodes[i], child, {
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

#[derive(Debug)]
pub(crate) struct Scroll {
    relative_offset: Xy<f32>,
}
impl Scroll {
    pub const ZERO: Scroll = Scroll {
        relative_offset: Xy::new(0.0, 0.0),
    };
}

impl Ui {
    pub(crate) fn update_scroll(&mut self, i: NodeI, delta: f32, axis: Axis) {       
        let real_rect = self.nodes[i].rect;
        
        let content_rect = self.nodes[i].content_bounds;
        let content_rect_size = content_rect.size()[axis];

        if content_rect_size <= 0.0 {
            self.nodes[i].scroll.relative_offset[axis] = 0.0;
            return;
        }

        let min_scroll = (content_rect[axis][0] - real_rect[axis][0] ) / content_rect_size;
        let max_scroll = (content_rect[axis][1] - real_rect[axis][1] ) / content_rect_size;
        
        if min_scroll < max_scroll {                
            self.nodes[i].scroll.relative_offset[axis] += delta * (max_scroll - min_scroll);
            
            let rel_offset = &mut self.nodes[i].scroll.relative_offset[axis];
            if min_scroll < max_scroll {
                *rel_offset = rel_offset.clamp(min_scroll, max_scroll);
            }
        } else {
            self.nodes[i].scroll.relative_offset[axis] = 0.0;
        }
    
    }
    
    pub(crate) fn clamp_scroll(&mut self, i: NodeI) {       
        for axis in [X, Y] {
            self.update_scroll(i, 0.0, axis);
        }
    }

    pub(crate) fn scroll_offset(&self, i: NodeI, axis: Axis) -> f32 {
        let scroll_offset = self.nodes[i].scroll.relative_offset[axis];
        let scroll_space = self.nodes[i].content_bounds.size()[axis];
        return scroll_offset * scroll_space;
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct ProposedSizes {
    to_this_child: Xy<f32>, // the size that was proposed to a child specifically after dividing between children
    to_all_children: Xy<f32>, // the whole size that the parent proposed to ALL its children collectively
}
impl ProposedSizes {
    pub(crate) const fn stack(to_this_child: Xy<f32>, to_all_children: Xy<f32>) -> ProposedSizes {
        return ProposedSizes {
            to_this_child,
            to_all_children,
        }
    }
    pub(crate) const fn container(size: Xy<f32>) -> ProposedSizes {
        return ProposedSizes {
            to_this_child: size,
            to_all_children: size,
        }
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
