use std::mem;

use crate::*;
use crate::node::*;

pub(crate) const BIG_FLOAT: f32 = 100000.0;

pub(crate) const BASE_DURATION: f32 = 0.1;


/// Iterate on the children linked list.
#[macro_export]
#[doc(hidden)] // Ideally these wouldn't even be public
macro_rules! for_each_child {
    ($ui:expr, $start:expr, $child:ident, $body:block) => {
        {
            let mut current_child = $start.first_child;
            while let Some($child) = current_child {
                if ! $ui.nodes[$child].just_lingering {
                    $body
                }
                current_child = $ui.nodes[$child].next_sibling;
            }
        }
    };
}

/// Iterate on the children linked list.
#[macro_export]
#[doc(hidden)] // Ideally these wouldn't even be public
macro_rules! for_each_child_including_lingering {
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

/// Iterate on the linked list of hidden children
#[macro_export]
#[doc(hidden)]
macro_rules! for_each_hidden_child {
    ($ui:expr, $start:expr, $child:ident, $body:block) => {
        {
            let mut current_child = $start.first_hidden_child;
            while let Some($child) = current_child {
                $body
                current_child = $ui.nodes[$child].next_hidden_sibling;
            }
        }
    };
}

impl Ui {
    pub(crate) fn relayout(&mut self) {
        let tree_changed = self.sys.changes.tree_changed;
        let partial_relayouts = ! self.sys.changes.partial_relayouts.is_empty();
        let rect_updates = ! self.sys.changes.cosmetic_rect_updates.is_empty();
        let full_relayout = self.sys.changes.full_relayout;
        let text_changed = self.sys.changes.text_changed;

        let nothing_to_do = !partial_relayouts && !rect_updates && !full_relayout && !text_changed && !tree_changed;
        if nothing_to_do {
            return;
        }

        // if anything happened at all, we'll need to rerender.
        self.sys.changes.need_gpu_rect_update = true;
        self.sys.changes.need_rerender = true;

        if full_relayout {
            self.relayout_from_root();
        } else {           
            if tree_changed {
                self.do_partial_relayouts(false);
            } else {
                self.do_partial_relayouts(true);
            }
        }

        // these ones are after the second-order-effect resolve_hover, just to see the update sooner.
        if full_relayout || tree_changed {
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
    }

    pub(crate) fn do_cosmetic_rect_updates(&mut self) {
        if !self.sys.changes.cosmetic_rect_updates.is_empty() {
            self.sys.changes.need_gpu_rect_update = true;
        }

        // Is this even better than iterating with an index?
        let cosmetic_rect_updates = mem::take(&mut self.sys.changes.cosmetic_rect_updates);

        for &update in &cosmetic_rect_updates {
            self.update_rect(update);
            log::info!("Visual rectangle update ({})", self.node_debug_name_fmt_scratch(update));
        }

        self.sys.changes.cosmetic_rect_updates = cosmetic_rect_updates;
        self.sys.changes.cosmetic_rect_updates.clear();
    }

    // this gets called even when zero relayouts are needed. in that case it just does nothing. I guess it's to make the layout() logic more readable
    pub(crate) fn do_partial_relayouts(&mut self, update_rects_while_relayouting: bool) {
        // sort by depth
        // todo: there was something about it being close to already sorted, except in reverse
        // the plan was to sort it in reverse and then use it in reverse
        self.sys.changes.partial_relayouts.sort();
        self.sys.partial_relayout_count = 0;

        for idx in 0..self.sys.changes.partial_relayouts.len() {
            // in partial_relayout(), we will check for overlaps.
            // todo: if that works as expected, maybe we can skip the limit/full relayout thing, or at least raise the limit by a lot.
            let relayout = self.sys.changes.partial_relayouts[idx];
            
            self.partial_relayout(relayout.i, update_rects_while_relayouting);
        }

        if self.sys.partial_relayout_count != 0 {
            let nodes = if self.sys.partial_relayout_count == 1 { "node" } else { "nodes" };
            log::info!("Partial relayout ({:?} {nodes})", self.sys.partial_relayout_count);
        }

        self.sys.partial_relayout_count = 0;
    }

    pub(crate) fn relayout_from_root(&mut self) {
        // 1st recursive tree traversal: start from the root and recursively determine the size of all nodes
        let starting_proposed_size = Xy::new(1.0, 1.0);

        self.recursive_determine_size_and_hidden(ROOT_I, ProposedSizes::container(starting_proposed_size), false);
        
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
        let hidden_branch = if i == ROOT_I {
            false
        } else {
            match self.nodes[self.nodes[i].parent].params.children_can_hide {
                ChildrenCanHide::Yes => true,
                ChildrenCanHide::No => false,
                ChildrenCanHide::Inherit => false, // This should be determined by traversing up, but for partial relayout we simplify
            }
        };
        self.recursive_determine_size_and_hidden(i, starting_proposed_size, hidden_branch);
        
        // 2nd recursive tree traversal: now that all nodes have a calculated size, place them.
        self.recursive_place_children(i, update_rects);

        self.nodes[i].last_layout_frame = self.sys.current_frame;
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
            if let Size::AspectRatio(aspect) = self.nodes[i].params.layout.size[axis] {
                match self.nodes[i].params.layout.size[axis.other()] {
                    Size::AspectRatio(_second_aspect) => {
                        let debug_name = self.node_debug_name_fmt_scratch(i);
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

    fn recursive_determine_size_and_hidden(
        &mut self,
        i: NodeI,
        proposed_sizes: ProposedSizes,
        hideable_branch: bool,
    ) -> Xy<f32> {
        self.nodes[i].last_proposed_sizes = proposed_sizes;
        
        // Set can_hide flag based on parent's children_can_hide setting
        self.nodes[i].can_hide = hideable_branch;
        
        // Determine this node's children_can_hide setting for its children
        let children_can_hide = match self.nodes[i].params.children_can_hide {
            ChildrenCanHide::Yes => true,
            ChildrenCanHide::No => false,
            ChildrenCanHide::Inherit => hideable_branch,
        };

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
                    let child_size = self.recursive_determine_size_and_hidden(child, ProposedSizes::stack(available_size_left, size_to_propose), children_can_hide);
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
                        let child_size = self.recursive_determine_size_and_hidden(child, ProposedSizes::stack(size_per_child, size_to_propose), children_can_hide);
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
                let child_size = self.recursive_determine_size_and_hidden(child, ProposedSizes::container(size_to_propose), children_can_hide);
                content_size.update_for_child(child_size, stack); // this is None
            });            
            
            // Propose the whole size_to_propose to the contents, and let them decide.
            if self.nodes[i].text_i.is_some() {
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
        let text_i = self.nodes[i].text_i.as_ref().unwrap();
        
        match text_i {
            TextI::TextEdit(handle) => {
                let mut text_edit = self.sys.text.get_text_edit_mut(&handle);
                
                if text_edit.single_line() {
                    let layout = text_edit.layout();
                    let text_height = if let Some(first_line) = layout.lines().next() {
                        first_line.metrics().line_height
                    } else {
                        0.0
                    };
                    
                    // let text_width = layout.full_width();
                    let text_width = proposed_size.x * self.sys.unifs.size[X];

                    text_edit.set_size((text_width, text_height));

                    let text_size_pixels = Xy::new(text_width, text_height);
                    return self.f32_pixels_to_frac2(text_size_pixels);
                    
                } else {
                    let w = proposed_size.x * self.sys.unifs.size[X];
                    let h = proposed_size.y * self.sys.unifs.size[Y];
                    
                    text_edit.set_size((w, h));
                    return proposed_size;
                }
                
            }
            TextI::TextBox(handle) => {
                
                let fit_content_y = self.nodes[i].params.layout.size[Y] == Size::FitContent;
                let fit_content_x = self.nodes[i].params.layout.size[X] == Size::FitContent;

                let h = if fit_content_y {
                    BIG_FLOAT
                } else {
                    proposed_size.y * self.sys.unifs.size[Y]
                };

                let w = if fit_content_x {
                    if fit_content_y {
                        proposed_size.x * self.sys.unifs.size[X]
                    } else {
                        BIG_FLOAT
                    }
                } else {
                    proposed_size.x * self.sys.unifs.size[X]
                };

                let mut text_box = self.sys.text.get_text_box_mut(&handle);
                text_box.set_size((w, h));
                
                let layout = text_box.layout();
                let size_pixels = Xy::new(layout.width(), layout.height());
                let size = self.f32_pixels_to_frac2(size_pixels);        

                return size;
            }
        }
    }

    pub(crate) fn recursive_place_children(&mut self, i: NodeI, also_update_rects: bool) {
        self.nodes[i].content_bounds = XyRect::new_symm([f32::MAX, f32::MIN]);

        self.sys.partial_relayout_count += 1;
        if let Some(stack) = self.nodes[i].params.stack {
            self.place_children_stack(i, stack);
        } else {
            self.place_children_container(i);
        };

        if also_update_rects {
            self.update_rect(i);
        }

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

            // Store current animated position for position change detection
            let old_animated_rect = self.nodes[child].get_animated_rect();

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

            // Initialize slide animation for newly appearing elements or position changes
            self.init_animations(child, i, Some(old_animated_rect));

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

            let old_animated_rect = self.nodes[child].get_animated_rect();

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

            // Initialize slide animation for newly appearing elements or position changes
            self.init_animations(child, i, Some(old_animated_rect));

            self.update_content_bounds(i, self.nodes[child].rect);
        });

        self.set_children_scroll(i);
    }

    fn init_animations(&mut self, child: NodeI, parent: NodeI, old_animated_rect: Option<XyRect>) {
        let slide_flags = self.nodes[child].params.animation.slide;
        if slide_flags.is_empty() {
            return;
        }
        
        // for now bail if an animation is already ongoing
        if self.nodes[child].animation_start_time.is_some() {
            return;
        }

        let current_time = ui_time_f32();
        let current_frame = self.sys.current_frame;
        let child_frame_added = self.nodes[child].frame_added;
        let target_rect = self.nodes[child].rect;
        let mut starting_offset = Xy::new(0.0, 0.0);
        let mut should_animate = false;
        
        if child_frame_added == current_frame && slide_flags.contains(SlideFlags::APPEARING) {
            // Newly appearing element - calculate offset from configured edge
            let parent_rect = self.nodes[parent].rect;
            let slide_edge = self.nodes[child].params.animation.slide_edge;
            
            match slide_edge {
                SlideEdge::Top => {
                    let element_size = target_rect.size().y;
                    starting_offset.y = parent_rect.y[0] - element_size - target_rect.y[0];
                    should_animate = true;
                },
                SlideEdge::Bottom => {
                    starting_offset.y = parent_rect.y[1] - target_rect.y[0];
                    should_animate = true;
                },
                SlideEdge::Left => {
                    let element_size = target_rect.size().x;
                    starting_offset.x = parent_rect.x[0] - element_size - target_rect.x[0];
                    should_animate = true;
                },
                SlideEdge::Right => {
                    starting_offset.x = parent_rect.x[1] - target_rect.x[0];
                    should_animate = true;
                },
            }
            
        } else if let Some(old_rect) = old_animated_rect
            && slide_flags.contains(SlideFlags::MOVING)
            && ! self.nodes[child].exiting {

            let position_threshold = 0.01;
            let x_moved = (target_rect.x[0] - old_rect.x[0]).abs() > position_threshold;
            let y_moved = (target_rect.y[0] - old_rect.y[0]).abs() > position_threshold;
            
            if x_moved && slide_flags.contains(SlideFlags::HORIZONTAL) {
                starting_offset.x = old_rect.x[0] - target_rect.x[0];
                should_animate = true;
            }
            if y_moved && slide_flags.contains(SlideFlags::VERTICAL) {
                starting_offset.y = old_rect.y[0] - target_rect.y[0];
                should_animate = true;
            }
        }
        
        if should_animate {
            // Initialize animation state with the offset
            let child_node = &mut self.nodes[child];
            child_node.animation_offset_start = starting_offset;
            child_node.animation_start_time = Some(current_time);
        }
    }

    #[inline]
    fn update_content_bounds(&mut self, i: NodeI, content_rect: XyRect) {
        for axis in [X, Y] {
            let c_bounds = &mut self.nodes[i].content_bounds[axis];
            c_bounds[0] = c_bounds[0].min(content_rect[axis][0]);
            c_bounds[1] = c_bounds[1].max(content_rect[axis][1]);
        }
    }

    fn set_children_scroll(&mut self, i: NodeI) {
        if ! self.nodes[i].params.is_scrollable() {
            return;
        }

        for_each_child!(self, self.nodes[i], child, {
            for axis in [X, Y] {
                if self.nodes[i].params.layout.scrollable[axis] {
                    let scroll_offset = self.scroll_offset(i, axis);
                    self.nodes[child].rect[axis][0] += scroll_offset;
                    self.nodes[child].rect[axis][1] += scroll_offset;
                }
            }
        });
    }

    pub(crate) fn set_clip_rect(&mut self, i: NodeI) {
        // Start from keep the parent's clip rect.
        // If nobody wants to clip children, this will always be [0.0, 1.0], passed down from root to everything else. 
        let parent_clip_rect = if i == ROOT_I {
            Xy::new_symm([0.0, 1.0])
        } else {
            let parent = self.nodes[i].parent;
            self.nodes[parent].clip_rect
        };

        let mut clip_rect = parent_clip_rect;
        for axis in [X, Y] {
            if self.nodes[i].params.clip_children[axis] {
                let own_rect = self.nodes[i].animated_rect;
                clip_rect[axis] = intersect(own_rect[axis], parent_clip_rect[axis])
            }
        }

        self.nodes[i].clip_rect = clip_rect;

        if let Some(text_i) = &self.nodes[i].text_i {
            let left = clip_rect[X][0] * self.sys.unifs.size[X];
            let right = clip_rect[X][1] * self.sys.unifs.size[X];
            let top = clip_rect[Y][0] * self.sys.unifs.size[Y];
            let bottom = clip_rect[Y][1] * self.sys.unifs.size[Y];

            let padding = self.nodes[i].params.layout.padding;
            let text_left = (self.nodes[i].animated_rect[X][0] * self.sys.unifs.size[X]) as f64 + padding[X] as f64;
            let text_top = (self.nodes[i].animated_rect[Y][0] * self.sys.unifs.size[Y]) as f64 + padding[Y] as f64;
            
            let text_clip_rect = Some(textslabs::Rect {
                x0: left as f64 - text_left,
                y0: top as f64 - text_top,
                x1: right as f64 - text_left,
                y1: bottom as f64 - text_top,
            });
            
            match text_i {
                TextI::TextBox(handle) => {
                    self.sys.text.get_text_box_mut(&handle).set_clip_rect(text_clip_rect);
                }
                TextI::TextEdit(handle) => {
                    self.sys.text.get_text_edit_mut(&handle).set_clip_rect(text_clip_rect);
                }
            }
        }
    }

    pub(crate) fn rebuild_all_rects(&mut self) {
        log::info!("Rebuilding all rectangles");
        self.sys.rects.clear();

        self.sys.click_rects.clear();
        self.sys.scroll_rects.clear();
        self.sys.z_cursor = Z_START;

        self.breadth_first_push_rects();

        self.sys.editor_rects_i = (self.sys.rects.len()) as u16;
    }

    fn breadth_first_push_rects(&mut self) {
        self.sys.breadth_traversal_queue.clear();
        self.sys.breadth_traversal_queue.push_back(ROOT_I);

        while let Some(i) = self.sys.breadth_traversal_queue.pop_front() {
            self.update_animations(i);
            self.push_render_data(i);
            
            for_each_child_including_lingering!(self, self.nodes[i], child, {
                self.sys.breadth_traversal_queue.push_back(child);
            });
        }
    }
    
    /// update local animations, storing the total offset in the node.
    pub(crate) fn update_animations(&mut self, i: NodeI) {
        self.nodes[i].animated_rect = self.nodes[i].rect;

        if ! self.node_or_parent_has_ongoing_animation(i) {
            self.nodes[i].animation_start_time = None;
        }

        let parent_offset = if i == ROOT_I {
            Xy::new(0.0, 0.0)
        } else {
            let parent = self.nodes[i].parent;
            self.nodes[parent].animation_offset
        };
        
        let mut local_offset = self.nodes[i].animation_offset_target;
        
        if let Some(animation_start_time) = self.nodes[i].animation_start_time {
            let elapsed = ui_time_f32() - animation_start_time;
    
            let speed = self.sys.global_animation_speed * self.nodes[i].params.animation.speed;
            let duration = BASE_DURATION / speed;
            
            if elapsed < duration {
                let t = elapsed / duration;
                let ease_t = 1.0 - (1.0 - t) * (1.0 - t);
                
                local_offset = Xy::new(
                    self.nodes[i].animation_offset_target.x * ease_t + self.nodes[i].animation_offset_start.x * (1.0 - ease_t),
                    self.nodes[i].animation_offset_target.y * ease_t + self.nodes[i].animation_offset_start.y * (1.0 - ease_t),
                );
            }
        }
        
        let total_offset = parent_offset + local_offset;
    
        self.nodes[i].animated_rect.x[0] = self.nodes[i].rect.x[0] + total_offset.x;
        self.nodes[i].animated_rect.x[1] = self.nodes[i].rect.x[1] + total_offset.x;
        self.nodes[i].animated_rect.y[0] = self.nodes[i].rect.y[0] + total_offset.y;
        self.nodes[i].animated_rect.y[1] = self.nodes[i].rect.y[1] + total_offset.y;
        
        // Store the total offset for children to use
        self.nodes[i].animation_offset = total_offset;

        self.set_clip_rect(i);
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
    pub(crate) fn update_container_scroll(&mut self, i: NodeI, delta: f32, axis: Axis) {       
        let container_rect = self.nodes[i].rect;

        let content_bounds = self.nodes[i].content_bounds;
        let content_rect_size = content_bounds.size()[axis];

        if content_rect_size <= 0.0 {
            self.nodes[i].scroll.relative_offset[axis] = 0.0;
            return;
        }

        // min scroll is the negative/upwards scroll that corrects the bottom end of content ending up below the container's bottom
        let min_scroll = if content_bounds[axis][1] > container_rect[axis][1] {
            container_rect[axis][1] - content_bounds[axis][1]
        } else {
            0.0
        };

        // max scroll is the positive/downwards scroll that corrects the top end of content overflowing above the container's top
        let max_scroll = if content_bounds[axis][0] < container_rect[axis][0] {
            container_rect[axis][0] - content_bounds[axis][0]
        } else {
            0.0
        };
                
        if min_scroll < max_scroll {                
            if self.nodes[i].frame_added == self.sys.current_frame && delta == 0.0 {
                if let Some(stack) = self.nodes[i].params.stack {
                    if stack.axis == axis {
                        self.nodes[i].scroll.relative_offset[axis] = match stack.arrange {
                            Arrange::End => min_scroll,
                            _ => max_scroll,
                        };
                    }
                }
            } else {
                // Normal scroll update
                self.nodes[i].scroll.relative_offset[axis] += delta;
            }
            
            let rel_offset = &mut self.nodes[i].scroll.relative_offset[axis];
            *rel_offset = rel_offset.clamp(min_scroll, max_scroll);

        } else {
            self.nodes[i].scroll.relative_offset[axis] = 0.0;
        }

    }
    
    pub(crate) fn _clamp_scroll(&mut self, i: NodeI) {
        // todo: this was hella wrong, figure it out from scratch
        // this was mainly for resizing, I think. when the scroll values and bounds get messed up by resizing or relayouts, we should at least reclamp the offset, or set it to zero if all content fits now.
        // for resizing, we should also store the offset as relative. Actually, for text it should be way more advanced, like  keeping track of the position in the text
        for axis in [X, Y] {
            self.update_container_scroll(i, 0.0, axis);
        }
    }

    pub(crate) fn scroll_offset(&self, i: NodeI, axis: Axis) -> f32 {
        let scroll_offset = self.nodes[i].scroll.relative_offset[axis];

        // round it to whole pixels to avoid wobbling
        let size = self.sys.unifs.size[axis];
        let scroll_offset = (scroll_offset * size).round() / size;

        return scroll_offset;
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
