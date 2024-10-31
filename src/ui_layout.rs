use crate::*;

use crate::ui_math::*;

use crate::for_each_child;


use glyphon::Buffer as GlyphonBuffer;
use Axis::{X, Y};

impl Ui {

    pub fn relayout(&mut self) {        
        // after calling this, the changes are swapped into `self.sys.changes.swapped_tree_changes`
        self.sys.changes.swap_thread_local_tree_changes();

        let nothing_to_do = self.sys.changes.swapped_tree_changes.len() == 0
            && self.sys.changes.partial_relayouts.len() == 0
            && self.sys.changes.cosmetic_rect_updates.len() == 0
            && self.sys.changes.full_relayout == false
            && self.sys.changes.rebuild_all_rects == false;

        if nothing_to_do {
            return;
        }

        self.sys.changes.need_rerender = true;


        // if there's a lot of tree changes and/or partial relayouts, it's probably cheaper to just do a full relayout rather than going through all of them. 
        let score = self.sys.changes.swapped_tree_changes.len() * 2 + self.sys.changes.partial_relayouts.len();
        const MAX_RELAYOUT_SCORE: usize = 30;
        if score > MAX_RELAYOUT_SCORE || self.sys.part.current_frame == FIRST_FRAME || self.sys.changes.full_relayout {
            // println!("  Full relayout (score: {:?})", score);
            self.full_relayout_and_rebuild();
            self.sys.changes.reset();
            return;
        }

        // if there are any tree changes, the rects are all invalidated.
        let tree_changed = ! self.sys.changes.swapped_tree_changes.is_empty();
        let rebuild_all_rects = tree_changed || self.sys.changes.rebuild_all_rects;


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

        let update_rects_while_relayouting = ! rebuild_all_rects;

        for idx in 0..self.sys.relayouts_scrath.len() {
            // in partial_relayout(), we will check for overlaps.
            // todo: is that works as expected, maybe we can skip the limit/full relayout thing, or at least raise the limit by a lot.
            let relayout = self.sys.relayouts_scrath[idx];
            
            self.partial_relayout(relayout.i, update_rects_while_relayouting);

            println!("[{:?}]  partial relayout ({:?})", T0.elapsed(), self.nodes[relayout.i].debug_name());
        }

        // rect updates
        if rebuild_all_rects {
            self.sys.rects.clear();
            self.rebuild_all_rects();
            println!("[{:?}]  rebuild all rects", T0.elapsed());
        } else {
            // cosmetic atomic rect updates
            for idx in 0..self.sys.changes.cosmetic_rect_updates.len() {
                let update = self.sys.changes.cosmetic_rect_updates[idx];
                self.update_rect(update);
                println!("[{:?}]  cosmetic update ({:?})", T0.elapsed(), self.nodes[update].debug_name());
            }
        }

        self.sys.changes.reset();
    }

    pub fn full_relayout_and_rebuild(&mut self) {
        self.sys.rects.clear();
        
        self.relayout_from_root();

        self.recursive_push_rects(ROOT_I);

        self.push_cursor_rect();
    }

    pub fn relayout_from_root(&mut self) {
        // 1st recursive tree traversal: start from the root and recursively determine the size of all nodes
        // For the first node, assume that the proposed size that we got from the parent last frame is valid. (except for root, in which case just use the whole screen. todo: should just make full_relayout a separate function.)
        let starting_proposed_size = Xy::new(1.0, 1.0);
        self.recursive_determine_size(ROOT_I, starting_proposed_size);
        
        // 2nd recursive tree traversal: now that all nodes have a calculated size, place them.
        // we don't do update_rects here because the first frame you can't update... but maybe just special-case the first frame, then should be faster
        self.recursive_place_children(ROOT_I, false);
        
        self.nodes[ROOT_I].last_layout_frame = self.sys.part.current_frame;
    }

    pub fn partial_relayout(&mut self, node: usize, update_rects: bool) {
        // if the node has already been layouted on the current frame, stop immediately, and don't even recurse.
        // when doing partial layouts, this avoids overlap, but it means that we have to sort the partial relayouts cleanly from least depth to highest depth in order to get it right. This is done in `relayout()`.
        let current_frame = self.sys.part.current_frame;
        if self.nodes[node].last_layout_frame >= current_frame {
            return;
        }

        // 1st recursive tree traversal: start from the root and recursively determine the size of all nodes
        // For the first node, assume that the proposed size that we got from the parent last frame is valid. (except for root, in which case just use the whole screen. todo: should just make full_relayout a separate function.)
        let starting_proposed_size = match node {
            ROOT_I => Xy::new(1.0, 1.0),
            _ => {
                let parent = self.nodes[node].parent;
                self.nodes[parent].size
            } 
        };
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
                Size::FitContent | Size::FitContentOrMinimum(_) => {}, // propose the whole size. We will shrink our own final size later if they end up using less or more 
                Size::Fill => {}, // keep the whole proposed_size
                Size::Fixed(len) => match len {
                    Len::Pixels(pixels) => {
                        proposed_size[axis] = self.pixels_to_frac(pixels, axis);
                    },
                    Len::Frac(frac) => {
                        proposed_size[axis] *= frac;
                    },
                }
            }
        }

        // just moved this from get_children_proposed_size(), haven't thought about it that hard, but it seems right.
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
        let stack = self.nodes[node].params.stack;
        
        // calculate the total size to propose to children
        let proposed_size = self.get_proposed_size(node, proposed_size);
        // divide it across children (if Stack)
        let child_proposed_size = self.get_children_proposed_size(node, proposed_size);

        // Propose a size to the children and let them decide
        let mut content_size = Xy::new(0.0, 0.0);
        for_each_child!(self, self.nodes[node], child, {
            let child_size = self.recursive_determine_size(child, child_proposed_size);
            content_size.update_for_child(child_size, stack);
        });

        // Propose the whole proposed_size (regardless of stack) to the contents, and let them decide.
        if self.nodes[node].text_id.is_some() {
            let text_size = self.determine_text_size(node, proposed_size);
            content_size.update_for_content(text_size, stack);
        }
        if self.nodes[node].imageref.is_some() {
            let image_size = self.determine_image_size(node, proposed_size);
            content_size.update_for_content(image_size, stack);
        }

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
                    let min_size = match min_size {
                        Len::Pixels(pixels) => {
                            self.pixels_to_frac(pixels, axis)
                        },
                        Len::Frac(frac) => proposed_size[axis] * frac
                    };

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

    fn determine_text_size(&mut self, node: usize, _proposed_size: Xy<f32>) -> Xy<f32> {
        let text_id = self.nodes[node].text_id.unwrap();
        let buffer = &mut self.sys.text.text_areas[text_id].buffer;

        // this is for FitContent on both directions, basically.
        // todo: the rest.
        // also, note: the set_align trick might not be good if we expose the ability to set whatever align the user wants.

        // let w = proposed_size.x * self.sys.part.unifs.size[X];
        // let h = proposed_size.y * self.sys.part.unifs.size[Y];
        let w = 999999.0;
        let h = 999999.0;

        for line in &mut buffer.lines {
            line.set_align(Some(glyphon::cosmic_text::Align::Left));
        }

        buffer.set_size(&mut self.sys.text.font_system, Some(w), Some(h));
        buffer.shape_until_scroll(&mut self.sys.text.font_system, false);

        let trimmed_size = buffer.measure_text_pixels();

        // self.sys.text.text_areas[text_id].buffer.set_size(&mut self.sys.text.font_system, trimmed_size.x, trimmed_size.y);
        // self.sys.text.text_areas[text_id]
        //     .buffer
        //     .shape_until_scroll(&mut self.sys.text.font_system, false);

        // for axis in [X, Y] {
        //     trimmed_size[axis] *= 2.0;
        // }

        // return proposed_size;
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

    fn recursive_place_children(&mut self, node: usize, update_rects: bool) {
        if let Some(stack) = self.nodes[node].params.stack {
            self.place_children_stack(node, stack, update_rects);
        } else {
            self.place_children_container(node, update_rects);
        };

        // self.place_image(node); // I think there's nothing to place? right now it's always the full rect
        self.place_text(node, self.nodes[node].rect);
    
        if update_rects {
            self.update_rect(node);
            println!("[{:?}] update rect while layouting {:?}", T0.elapsed(), self.nodes[node].debug_name());
        }

        self.nodes[node].last_layout_frame = self.sys.part.current_frame;
    }

    fn place_children_stack(&mut self, node: usize, stack: Stack, update_rects: bool) {
        let (main, cross) = (stack.axis, stack.axis.other());
        let parent_rect = self.nodes[node].rect;
        let padding = self.to_frac2(self.nodes[node].params.layout.padding);
        let spacing = self.to_frac(stack.spacing, stack.axis);
        
        // Totally ignore the children's chosen Position's and place them according to our own Stack::Arrange value.

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
            Arrange::Start => parent_rect[main][0] + padding[main],
            Arrange::End => parent_rect[main][1] + padding[main] - total_size,
            Arrange::Center => {
                let center = (parent_rect[main][1] + parent_rect[main][0]) / 2.0 - 2.0 * padding[main];
                center - total_size / 2.0
            },
            _ => todo!(),
        };

        for_each_child!(self, self.nodes[node], child, {
            let size = self.nodes[child].size;

            match self.nodes[child].params.layout.position[cross] {
                Position::Center => {
                    let origin = (parent_rect[cross][1] + parent_rect[cross][0]) / 2.0;
                    self.nodes[child].rect[cross] = [
                        origin - size[cross] / 2.0 ,
                        origin + size[cross] / 2.0 ,
                    ];  
                },
                Position::Start => {
                    let origin = parent_rect[cross][0] + padding[cross];
                    self.nodes[child].rect[cross] = [origin, origin + size[cross]];         
                },
                Position::Static(len) => {
                    let static_pos = self.to_frac(len, cross);
                    let origin = parent_rect[cross][0] + padding[cross] + static_pos;
                    self.nodes[child].rect[cross] = [origin, origin + size[cross]];         
                },
                Position::End => {
                    let origin = parent_rect[cross][1] - padding[cross];
                    self.nodes[child].rect[cross] = [origin - size[cross], origin];
                },
            }

            self.nodes[child].rect[main] = [main_origin, main_origin + size[main]];

            self.recursive_place_children(child, update_rects);

            main_origin += self.nodes[child].size[main] + spacing;
        });
    }

    fn place_children_container(&mut self, node: usize, update_rects: bool) {
        let parent_rect = self.nodes[node].rect;
        let padding = self.to_frac2(self.nodes[node].params.layout.padding);

        for_each_child!(self, self.nodes[node], child, {
            let size = self.nodes[child].size;

            // check the children's chosen Position's and place them.
            for ax in [X, Y] {
                match self.nodes[child].params.layout.position[ax] {
                    Position::Start => {
                        let origin = parent_rect[ax][0] + padding[ax];
                        self.nodes[child].rect[ax] = [origin, origin + size[ax]];         
                    },
                    Position::Static(len) => {
                        let static_pos = self.to_frac(len, ax);
                        let origin = parent_rect[ax][0] + padding[ax] + static_pos;
                        self.nodes[child].rect[ax] = [origin, origin + size[ax]];
                    }
                    Position::End => {
                        let origin = parent_rect[ax][1] - padding[ax];
                        self.nodes[child].rect[ax] = [origin - size[ax], origin];
                    },
                    Position::Center => {
                        let origin = (parent_rect[ax][1] + parent_rect[ax][0]) / 2.0;
                        self.nodes[child].rect[ax] = [
                            origin - size[ax] / 2.0 ,
                            origin + size[ax] / 2.0 ,
                        ];           
                    },
                }
            }

            self.recursive_place_children(child, update_rects);
        });
    }

    #[allow(dead_code)]
    pub fn place_image(&mut self, _node: usize) {     
        // might be something here in the future
    }

    pub fn place_text(&mut self, node: usize, rect: XyRect) {
        let padding = self.to_pixels2(self.nodes[node].params.layout.padding);
        let node = &mut self.nodes[node];
        let text_id = node.text_id;

        if let Some(text_id) = text_id {
            let left = rect[X][0] * self.sys.part.unifs.size[X];
            let top = rect[Y][0] * self.sys.part.unifs.size[Y];

            // let right = rect[X][1] * self.sys.part.unifs.size[X];
            // let bottom =     rect[Y][1] * self.sys.part.unifs.size[Y];

            self.sys.text.text_areas[text_id].params.left = left + padding[X] as f32;
            self.sys.text.text_areas[text_id].params.top = top + padding[Y] as f32;
           
            // self.sys.text.text_areas[text_id].bounds.left = left as i32 + padding[X] as i32;
            // self.sys.text.text_areas[text_id].bounds.top = top as i32 + padding[Y] as i32;

            // self.sys.text.text_areas[text_id].bounds.right = right as i32;
            // self.sys.text.text_areas[text_id].bounds.bottom = bottom as i32;
        }
    }

    fn rebuild_all_rects(&mut self) {
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
    pub(crate) fn update_for_content(&mut self, child_size: Xy<f32>, _stack: Option<Stack>) {
        for axis in [X, Y] {
            if child_size[axis] > self[axis] {
                self[axis] = child_size[axis];
            }
        }
    }

}


pub trait MeasureText {
    fn measure_text_pixels(&self) -> Xy<f32>;
}
impl MeasureText for GlyphonBuffer {
    fn measure_text_pixels(&self) -> Xy<f32> {
        let layout_runs = self.layout_runs();
        let mut run_width: f32 = 0.;
        let line_height = self.lines.len() as f32 * self.metrics().line_height;
        for run in layout_runs {
            run_width = run_width.max(run.line_w);
        }
        return Xy::new(run_width.ceil(), line_height)
    }
}