use crate::*;
use std::collections::hash_map::Entry;
use std::hash::Hasher;
use std::mem;
use std::panic::Location;
use bytemuck::{Pod, Zeroable};
use vello_common::peniko::Extend;
use vello_common::kurbo::Rect as VelloRect;
use vello_common::kurbo::Shape as VelloShape;
use vello_common::paint::{ImageSource, Image as VelloImage};
use vello_common::peniko::ImageSampler;
use vello_common::kurbo::Affine;

/// An `u64` identifier for a GUI node.
/// 
/// Usually this is only used as part of [`NodeKey`] structs, which are created with the [`node_key`] macro or with [`NodeKey::sibling`].
#[doc(hidden)]
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, Pod, Zeroable)]
#[repr(C)]
pub struct Id(pub u64);

pub(crate) const FIRST_FRAME: u64 = 1;

pub(crate) const Z_START: f32 = 0.5;
// This one has to be small, but not small enough to get precision issues.
// And I think it's probably good if it's a rounded binary number (0x38000000)? Not sure.
pub const Z_STEP: f32 = -0.000_030_517_578;

impl Ui {
    /// Add a node to the `Ui` with the properties described by `params`.
    /// 
    /// `params` can be a basic [`NodeParams`] or a [`FullNodeParams`] created from it.
    /// 
    /// ```rust
    /// # use keru::*;
    /// # fn declare_ui(ui: &mut Ui) {
    /// let red_label = LABEL
    ///     .color(Color::RED)
    ///     .text("Increase");
    /// 
    /// ui.add(red_label);
    /// # }
    /// ```
    /// 
    ///  Buttons, images, text elements, stack containers, etc. are all created by `add`ing a node with the right [`NodeParams`].
    #[track_caller]
    pub fn add<'a>(&mut self, node: impl Into<FullNodeParams<'a>>) -> UiParent
    {
        let params = node.into();
        let key = params.key_or_anon_key();
        let (i, _id) = self.add_or_update_node(key);
        self.set_params(i, &params);
        self.set_params_text(i, &params);
        return UiParent::new(i);
    }

    #[track_caller]
    pub(crate) fn add_or_update_node(&mut self, key: NodeKey) -> (NodeI, Id) {
        let frame = self.sys.current_frame;
        let mut new_node_should_relayout = false;

        // todo: at least when using non-anonymous keys, I think there's no legit use case for twins anymore. it's always a mistake, I think. it should log out a warning or panic.

        // Check the node corresponding to the key's id.
        // We might find that the key has already been used in this same frame:
        //      in this case, we take note, and calculate a twin key to use to add a "twin" in the next section.
        // Otherwise, we add or refresh normally, and take note of the final i.
        let twin_check_result = match self.nodes.node_hashmap.entry(key.id_with_subtree()) {
            // Add a new normal node (no twins).
            Entry::Vacant(v) => {
                let new_node = Node::new(&key, None, Location::caller(), frame);
                let final_i = NodeI::from(self.nodes.nodes.insert(new_node));
                v.insert(NodeMapEntry::new(final_i));

                new_node_should_relayout = true;

                UpdatedNormal { final_i }
            }
            Entry::Occupied(o) => {
                let old_map_entry = o.into_mut();
                let old_i = old_map_entry.slab_i.as_usize();
                let last_frame_touched = self.nodes.nodes[old_i].last_frame_touched;

                match should_refresh_or_add_twin(frame, last_frame_touched) {
                    // Refresh a normal node from the previous frame (no twins).
                    Refresh => {
                        old_map_entry.refresh();
                        self.nodes.nodes[old_i].last_frame_touched = frame;
                        let final_i = old_map_entry.slab_i;
                        UpdatedNormal { final_i }
                    }
                    // do nothing, just calculate the twin key and go to twin part below
                    AddTwin => {
                        old_map_entry.n_twins += 1;
                        let twin_key = key.sibling(old_map_entry.n_twins);

                        NeedToUpdateTwin {
                            twin_key,
                            twin_n: old_map_entry.n_twins,
                        }
                    }
                }
            }
        };

        // If twin_check_result is AddedNormal, the node was added in the section before,
        //      and there's nothing to do regarding twins, so we just confirm final_i.
        // If it's NeedToAddTwin, we repeat the same thing with the new twin_key.
        let (real_final_i, real_final_id) = match twin_check_result {
            UpdatedNormal { final_i } => (final_i, key.id_with_subtree()),
            NeedToUpdateTwin { twin_key, twin_n } => {
                match self.nodes.node_hashmap.entry(twin_key.id_with_subtree()) {
                    // Add new twin.
                    Entry::Vacant(v) => {
                        let new_twin_node = Node::new(&twin_key, Some(twin_n), Location::caller(), frame);
                        let real_final_i = NodeI::from(self.nodes.nodes.insert(new_twin_node));
                        v.insert(NodeMapEntry::new(real_final_i));
                        new_node_should_relayout = true;
                        (real_final_i, twin_key.id_with_subtree())
                    }
                    // Refresh a twin from the previous frame.
                    Entry::Occupied(o) => {
                        let old_twin_map_entry = o.into_mut();

                        let real_final_i = old_twin_map_entry.refresh();
                        self.nodes.nodes[real_final_i.as_usize()].last_frame_touched = frame;

                        (real_final_i, twin_key.id_with_subtree())
                    }
                }
            }
        };

        // update the in-tree links and the thread-local state based on the current parent.
        let NodeWithDepth { i: parent_i, depth } = thread_local::current_parent();
        self.set_tree_links(real_final_i, parent_i, depth);

        self.nodes[real_final_i].exiting = false;

        self.refresh_node(real_final_i);

        if new_node_should_relayout {
            self.push_partial_relayout(real_final_i);
        }

        return (real_final_i, real_final_id);
    }

    fn refresh_node(&mut self, i: NodeI) {        
        // refresh the text box associated with this node if it has one
        if let Some(text_i) = &self.nodes[i].text_i {
            match text_i {
                TextI::TextBox(handle) => {
                    self.sys.text.refresh_text_box(handle);
                }
                TextI::TextEdit(handle) => {
                    self.sys.text.refresh_text_edit(handle);
                }
            }
        }
    }

    // this function also detects new nodes and reorderings, and pushes partial relayouts for them. For deleted nodes, partial relayouts will be pushed in cleanup_and_stuff.
    fn set_tree_links(&mut self, new_node_i: NodeI, parent_i: NodeI, depth: usize) {
        assert!(new_node_i != parent_i, "Keru: Internal error: tried to add a node as child of itself ({}). This shouldn't be possible.", self.nodes[new_node_i].debug_name());

        self.nodes[new_node_i].depth = depth;
        self.nodes[new_node_i].currently_hidden = false;

        // Reset old links
        self.nodes[new_node_i].old_first_child = self.nodes[new_node_i].first_child;
        self.nodes[new_node_i].old_next_sibling = self.nodes[new_node_i].next_sibling;
        
        self.nodes[new_node_i].first_child = None;
        self.nodes[new_node_i].last_child = None;
        self.nodes[new_node_i].n_children = 0;

        // Add new child
        self.nodes[new_node_i].parent = parent_i;
        self.nodes[new_node_i].prev_sibling = None;
        self.nodes[new_node_i].next_sibling = None;

        self.nodes[parent_i].n_children += 1;

        match self.nodes[parent_i].last_child {
            None => {
                self.nodes[parent_i].first_child = Some(new_node_i);
                self.nodes[parent_i].last_child = Some(new_node_i);

                if self.nodes[parent_i].first_child != self.nodes[parent_i].old_first_child {
                    self.push_partial_relayout(parent_i);
                }
            },
            Some(last_child) => {
                let prev_sibling = last_child;
                self.nodes[new_node_i].prev_sibling = Some(prev_sibling);
                self.nodes[prev_sibling].next_sibling = Some(new_node_i);
                self.nodes[parent_i].last_child = Some(new_node_i);

                if self.nodes[prev_sibling].old_next_sibling != self.nodes[prev_sibling].next_sibling {
                    self.push_partial_relayout(parent_i);
                }
            },
        };

        self.set_relayout_chain_root(new_node_i, parent_i);

        self.remove_hidden_child_if_it_exists(new_node_i, parent_i);
    }

    fn remove_hidden_child_if_it_exists(&mut self, child_i: NodeI, parent_i: NodeI) {
        if let Some(first_hidden_child) = self.nodes[parent_i].first_hidden_child {
            if first_hidden_child == child_i {
                self.nodes[parent_i].first_hidden_child = self.nodes[child_i].next_hidden_sibling;
                self.nodes[child_i].next_hidden_sibling = None;
                return;
            }
            
            let mut prev = first_hidden_child;
            for_each_hidden_child!(self, self.nodes[parent_i], child, {
                if child == child_i {
                    self.nodes[prev].next_hidden_sibling = self.nodes[child].next_hidden_sibling;
                    self.nodes[child].next_hidden_sibling = None;
                    return;
                }
                prev = child;
            });
        }
    }

    fn add_hidden_child(&mut self, new_node_i: NodeI, parent_i: NodeI) {
        match self.nodes[parent_i].first_hidden_child {
            None => {
                self.add_hidden_first_child(new_node_i, parent_i)
            },
            Some(last_hidden_child) => {
                let old_last_hidden_child = last_hidden_child;
                self.add_hidden_sibling(new_node_i, old_last_hidden_child, parent_i)
            },
        };
    }

    fn add_hidden_first_child(&mut self, new_node_i: NodeI, parent_i: NodeI) {
        self.nodes[parent_i].first_hidden_child = Some(new_node_i);
    }
    
    fn add_hidden_sibling(&mut self, new_node_i: NodeI, old_last_child: NodeI, _parent_i: NodeI) {
        self.nodes[old_last_child].next_hidden_sibling = Some(new_node_i);
    }

    pub(crate) fn node_or_parent_has_ongoing_animation(&self, i: NodeI) -> bool {
        // todo: what about non-position exit animations, like fading away.

        // this works, but only if this function is called in the right pattern.
        // does it mean that some of the offset-inheriting wasn't needed? probably not.
        let parent = self.nodes[i].parent;
        if self.nodes[parent].exit_animation_still_going {
            return true;
        }

        let target = &self.nodes[i].expected_final_rect;
        let current = &self.nodes[i].real_rect;
        let tolerance = 0.0005;
        
        let is_at_target = (current.x[0] - target.x[0]).abs() < tolerance
            && (current.x[1] - target.x[1]).abs() < tolerance
            && (current.y[0] - target.y[0]).abs() < tolerance
            && (current.y[1] - target.y[1]).abs() < tolerance;

        return !is_at_target;
    }

    pub(crate) fn push_render_data(&mut self, i: NodeI) {
        let debug = cfg!(debug_assertions);
        let push_click_rect = if debug && self.inspect_mode() {
            true
        } else {
            let clickable = self.nodes[i].params.interact.senses != Sense::NONE;
            let editable = if let Some(text_i) = &self.nodes[i].text_i {
                match text_i {
                    TextI::TextEdit(_) => true,
                    TextI::TextBox(_) => false,
                }
            } else { false };
            clickable || editable
        };

        if push_click_rect {
            let click_rect = self.click_rect(i);
            self.sys.click_rects.push(click_rect);
        }

        if self.nodes[i].params.is_scrollable() {
            let click_rect = self.click_rect(i);
            self.sys.scroll_rects.push(click_rect);
        }
        
        self.sys.z_cursor += Z_STEP;
        let z = self.sys.z_cursor;
        self.nodes[i].z = z;

        let draw_even_if_invisible = self.sys.inspect_mode;

        // Push clip layers.
        // Todo: not sure this is a reasonable way to do it, but I don't understand how it's supposed to work.
        // I need breadth-first-traversal for painter's algorithm, but I can't stack clip layers unless I'm doing depth-first.
        let node_clip_rect = self.nodes[i].clip_rect;
        let is_clipping = node_clip_rect != Xy::new_symm([0.0, 1.0]);
        if is_clipping {
            let screen_size = self.sys.unifs.size;
            let clip_x0 = (node_clip_rect.x[0] * screen_size.x) as f64;
            let clip_y0 = (node_clip_rect.y[0] * screen_size.y) as f64;
            let clip_x1 = (node_clip_rect.x[1] * screen_size.x) as f64;
            let clip_y1 = (node_clip_rect.y[1] * screen_size.y) as f64;

            let clip_rect = VelloRect::new(clip_x0, clip_y0, clip_x1, clip_y1);
            self.sys.vello_scene.push_clip_path(&clip_rect.to_path(0.1));
        }

        // Render node's shape directly to vello scene
        if draw_even_if_invisible || self.nodes[i].params.rect.visible {
            self.render_node_shape_to_scene(i);
        }

        // Images
        if self.nodes[i].imageref.is_some() {
            let animated_rect = self.nodes[i].get_animated_rect();
            let screen_size = self.sys.unifs.size;
            let x0 = (animated_rect.x[0] * screen_size.x) as f64;
            let y0 = (animated_rect.y[0] * screen_size.y) as f64;
            let x1 = (animated_rect.x[1] * screen_size.x) as f64;
            let y1 = (animated_rect.y[1] * screen_size.y) as f64;

            match &self.nodes[i].imageref {
                Some(ImageRef::Raster { image_id, .. }) => {
                    // Create an image brush from the uploaded image ID
                    let image_source = ImageSource::OpaqueId(*image_id);
                    let sampler = ImageSampler::default().with_extend(Extend::Repeat);
                    let image_brush = VelloImage {
                        image: image_source,
                        sampler,
                    };

                    self.sys.vello_scene.set_paint_transform(Affine::translate((x0, y0)));
                    self.sys.vello_scene.set_paint(image_brush);

                    // Draw the rect at the actual screen position
                    let rect = vello_common::kurbo::Rect::new(x0, y0, x1, y1);
                    self.sys.vello_scene.fill_rect(&rect);

                    // Reset paint transform
                    self.sys.vello_scene.reset_paint_transform();
                }
                Some(ImageRef::Svg { svg_index, original_size }) => {
                    // Calculate scale to fit SVG in the node's rect
                    let node_width = x1 - x0;
                    let node_height = y1 - y0;
                    let scale_x = node_width / original_size.x as f64;
                    let scale_y = node_height / original_size.y as f64;

                    // Use the smaller scale to maintain aspect ratio
                    let scale = scale_x.min(scale_y);

                    // Calculate centering offset
                    let scaled_width = original_size.x as f64 * scale;
                    let scaled_height = original_size.y as f64 * scale;
                    let offset_x = (node_width - scaled_width) / 2.0;
                    let offset_y = (node_height - scaled_height) / 2.0;

                    // Create transform: translate to position, then scale
                    let transform = Affine::translate((x0 + offset_x, y0 + offset_y)) * Affine::scale(scale);

                    // Render SVG items directly with the transform
                    self.render_svg_items(*svg_index, transform);
                }
                None => {}
            }
        }

        if let Some(text_i) = &self.nodes[i].text_i {
            // Update text position using animated rect
            let animated_rect = self.nodes[i].get_animated_rect();
            let padding = self.nodes[i].params.layout.padding;
            let left = (animated_rect[X][0] * self.sys.unifs.size[X]) as f64 + padding[X] as f64;
            let top = (animated_rect[Y][0] * self.sys.unifs.size[Y]) as f64 + padding[Y] as f64;

            match text_i {
                TextI::TextBox(text_box_handle) => {
                    let text_box = self.sys.text.get_text_box_mut(&text_box_handle);
                    text_box.set_depth(z);
                    text_box.set_pos((left, top));
                    // Render text to scene
                    text_box.render_to_scene(&mut self.sys.vello_scene);
                },
                TextI::TextEdit(text_edit_handle) => {
                    let text_edit = self.sys.text.get_text_edit_mut(&text_edit_handle);
                    text_edit.set_depth(z);
                    text_edit.set_pos((left, top));
                    // Render text to scene
                    text_edit.render_to_scene(&mut self.sys.vello_scene);
                },
            }
        }

        if is_clipping {
            self.sys.vello_scene.pop_clip_path();
        }
    }

    fn render_svg_items(&mut self, svg_index: usize, transform: Affine) {
        use vello_common::pico_svg::Item;
        use vello_common::kurbo::Stroke;

        self.sys.vello_scene.set_transform(transform);

        let item_count = self.sys.svg_storage[svg_index].len();
        for i in 0..item_count {
            match &self.sys.svg_storage[svg_index][i] {
                Item::Fill(fill_item) => {
                    self.sys.vello_scene.set_paint(fill_item.color);
                    self.sys.vello_scene.fill_path(&fill_item.path);
                }
                Item::Stroke(stroke_item) => {
                    let style = Stroke::new(stroke_item.width);
                    self.sys.vello_scene.set_stroke(style);
                    self.sys.vello_scene.set_paint(stroke_item.color);
                    self.sys.vello_scene.stroke_path(&stroke_item.path);
                }
                Item::Group(group_item) => {
                    let new_transform = transform * group_item.affine;
                    render_svg_group(&mut self.sys.vello_scene, &group_item.children, new_transform);
                    self.sys.vello_scene.set_transform(transform);
                }
            }
        }

        self.sys.vello_scene.reset_transform();
    }

    fn set_relayout_chain_root(&mut self, new_node_i: NodeI, parent_i: NodeI) {
        let is_fit_content = self.nodes[new_node_i].params.is_fit_content();
        match self.nodes[parent_i].relayout_chain_root {
            Some(root_of_parent) => match is_fit_content {
                true => self.nodes[new_node_i].relayout_chain_root = Some(root_of_parent), // continue chain
                false => self.nodes[new_node_i].relayout_chain_root = None, // break chain
            },
            None => match is_fit_content {
                true => self.nodes[new_node_i].relayout_chain_root = Some(new_node_i), // start chain
                false => self.nodes[new_node_i].relayout_chain_root = None, // do nothing
            },
        };
    }

    pub(crate) fn push_partial_relayout(&mut self, _i: NodeI) {
        // let relayout_chain_root = match self.nodes[i].relayout_chain_root {
        //     Some(root) => root,
        //     None => i,
        // };

        // // even after the chain, we still have to go one layer up, because a different sized child probably means that the parent wants to place the node differently, and maybe pick a different size and position for the other children as well
        // // In practice, the first half of that is basically always true, but the second half is only true for Stacks. I don't really feel like adding a distinction for that right now.
        // let relayout_target = self.nodes[relayout_chain_root].parent;

        // // try skipping some duplicates
        // if self.sys.changes.partial_relayouts.last().map(|x| x.i) == Some(relayout_target) {
        //     return;
        // }

        // let relayout_entry = NodeWithDepth {
        //     i: relayout_target,
        //     depth: self.nodes[relayout_target].depth,
        // };
        // self.sys.changes.partial_relayouts.push(relayout_entry);
    }

    /// Clear the old GUI tree and start declaring another one.
    /// 
    /// Use together with [`Ui::finish_frame()`], at most once per frame.
    /// 
    /// ```rust
    /// # use keru::*;
    /// # pub struct State {
    /// #     pub ui: Ui,
    /// # }
    /// #
    /// # impl State {
    /// #   fn declare_ui(&mut self) {
    /// self.ui.begin_frame();
    /// // declare the GUI and update state
    /// self.ui.finish_frame();
    /// #
    /// #   }
    /// # }
    /// ```
    pub fn begin_frame(&mut self) {
        self.reset_root();

        self.sys.current_frame += 1;
        self.sys.text.advance_frame_and_hide_boxes();
        thread_local::clear_parent_stack();
        self.format_scratch.clear();
        self.sys.changes.unfinished_animations = false;

        let root_parent = UiParent::new(ROOT_I);
        thread_local::push_parent(&root_parent);

        self.begin_frame_resolve_inputs();
    }
    
    fn reset_root(&mut self) {
        self.nodes[ROOT_I].last_child = None;
        self.nodes[ROOT_I].first_child = None;
        self.nodes[ROOT_I].prev_sibling = None;
        self.nodes[ROOT_I].next_sibling = None;
        self.nodes[ROOT_I].n_children = 0;
    }

    /// Finish declaring the current GUI tree.
    /// 
    /// This function will also relayout the nodes that need it, and do some bookkeeping.
    /// 
    /// Use at most once per frame, after calling [`Ui::begin_frame()`] and running your tree declaration code.
    pub fn finish_frame(&mut self) {
        log::trace!("Finished Ui update");
        // pop the root node
        thread_local::pop_parent();

        self.cleanup_and_stuff();

        self.relayout();

        self.sys.third_last_frame_end_fake_time = self.sys.second_last_frame_end_fake_time;
        self.sys.second_last_frame_end_fake_time = self.sys.last_frame_end_fake_time;
        self.sys.last_frame_end_fake_time = observer_timestamp();

        if self.sys.update_frames_needed > 0 {
            self.sys.update_frames_needed -= 1;
        }

        self.sys.new_external_events = false;

        if let Some(waker) = &self.sys.waker {
            waker.needs_update.store(false, std::sync::atomic::Ordering::Relaxed);
        }

        // let mut buffer = String::new();
        // std::io::stdin().read_line(&mut buffer).expect("Failed to read line");
    }

    /// Returns `true` if a node corresponding to `key` exists and if it is currently part of the GUI tree. 
    pub fn is_in_tree(&self, key: NodeKey) -> bool {
        let node_i = self.nodes.node_hashmap.get(&key.id_with_subtree());
        if let Some(entry) = node_i {
            // todo: also return true if it's retained
            return self.nodes[entry.slab_i].last_frame_touched == self.sys.current_frame;
        } else {
            return false;
        }
    }

    fn cleanup_and_stuff(&mut self) {
        let mut non_fresh_nodes: Vec<NodeI> = take_buffer_and_clear(&mut self.sys.non_fresh_nodes);
        let mut to_cleanup: Vec<NodeI> = take_buffer_and_clear(&mut self.sys.to_cleanup);
        let mut hidden_branch_parents: Vec<NodeI> = take_buffer_and_clear(&mut self.sys.hidden_branch_parents);
        let mut exiting_nodes: Vec<NodeWithDepth> = take_buffer_and_clear(&mut self.sys.lingering_nodes);


        for (i, _) in self.nodes.nodes.iter().skip(2) {
            let i = NodeI::from(i);
            let freshly_added = self.nodes[i].last_frame_touched == self.sys.current_frame;

            if !freshly_added {
                non_fresh_nodes.push(i);
            }
        }

        // Start exit animations for all nodes that need them
        for &i in &non_fresh_nodes {
            let old_parent = self.nodes[i].parent;
            let old_parent_still_exists = self.nodes.get(old_parent).is_some();

            if old_parent_still_exists {
                self.init_exit_animations(i);
            }
        }

        // the top-level nodes in hidden branches need to be attached to their children_can_hide parents as hidden nodes, so that when that parent node is removed, we can also remove the hidden branch. Otherwise we'd just forget about them and leave them in memory forever.
        // The nodes with 
        for &i in &non_fresh_nodes {
            let freshly_added = self.nodes[i].last_frame_touched == self.sys.current_frame;
            let can_hide = self.nodes[i].can_hide;
            let currently_hidden = self.nodes[i].currently_hidden;
            let old_parent = self.nodes[i].parent;
            let old_parent_still_exists = self.nodes.get(old_parent).is_some();

            let is_first_child_in_hidden_branch = match self.nodes.get(old_parent) {
                Some(old_parent) => old_parent.params.children_can_hide == ChildrenCanHide::Yes,
                None => false,
            };
            let children_can_hide = self.nodes[i].params.children_can_hide == ChildrenCanHide::Yes;

            if ! freshly_added {
                if old_parent_still_exists && self.nodes[i].exiting && self.nodes[i].exit_animation_still_going {

                    exiting_nodes.push(NodeWithDepth { i, depth: self.nodes[i].depth });
                    
                } else if ! can_hide {
                    to_cleanup.push(i);
                    if old_parent_still_exists {
                        self.push_partial_relayout(old_parent);
                    }

                    if children_can_hide {
                        hidden_branch_parents.push(i);
                    }

                } else if ! currently_hidden {
                    
                    self.nodes[i].currently_hidden = true;

                    if is_first_child_in_hidden_branch {
                        self.add_hidden_child(i, old_parent);
                        if old_parent_still_exists {
                            self.push_partial_relayout(old_parent);
                        }
                    }
                }
            }
        }

        // Add lingering nodes back into the tree.
        // todo: don't just add them at the end, try to put them after their old prev_sibling. 
        exiting_nodes.sort_by_key(|n| n.depth);
        for &NodeWithDepth { i, .. } in &exiting_nodes {
            let old_parent = self.nodes[i].parent;
            self.set_tree_links(i, old_parent, self.nodes[i].depth);
            self.refresh_node(i);
            self.nodes[i].exiting = true;
            // todo not in this retarded way
            self.nodes[old_parent].n_children -= 1;
        }

        // This is delayed so that hidden children are all added
        for &i in &hidden_branch_parents {
            for_each_hidden_child!(self, self.nodes[i], hidden_child, {
                self.add_branch_to_cleanup(hidden_child, &mut to_cleanup);
            });
        }

        // finally cleanup
        for &k in &to_cleanup {
            self.cleanup_node(k);
        }
        
        self.sys.lingering_nodes = exiting_nodes;
        self.sys.non_fresh_nodes = non_fresh_nodes;
        self.sys.to_cleanup = to_cleanup;
        self.sys.hidden_branch_parents = hidden_branch_parents;
    }

    fn add_branch_to_cleanup(&mut self, i: NodeI, vec: &mut Vec<NodeI>) {
        vec.push(i);
        for_each_child!(self, self.nodes[i], child, {
            self.add_branch_to_cleanup(child, vec);
        });
    }

    fn cleanup_node(&mut self, i: NodeI) {
        if ! self.nodes.nodes.contains(i.as_usize()) {
            log::error!("Keru: Internal error: tried to cleanup the same node twice. ({:?})", i);
            // we could cheat and just return. instead we continue, so we can see the panic clearly in case there's any bugs.
        }
        let id = self.nodes[i].id;
        
        // skip the nodes that have last_frame_touched = now, because that means that they were not really removed, but just moved somewhere else in the tree.
        // Kind of weird to do this so late.
        // todo: with the new system we can delete this.
        if self.nodes[i].last_frame_touched == self.sys.current_frame {
            log::trace!("Not removing: {:?}, as it was moved around and not removed", self.node_debug_name_fmt_scratch(i));
            return;
        }

        let old_handle = self.nodes[i].text_i.take();
        if let Some(text_i) = old_handle {
            match text_i {
                TextI::TextBox(handle) => {
                    self.sys.text.remove_text_box(handle);
                }
                TextI::TextEdit(handle) => {
                    self.sys.text.remove_text_edit(handle);
                }
            }
        }

        self.nodes.node_hashmap.remove(&id);
        self.nodes.nodes.remove(i.as_usize());
    }

    pub(crate) fn current_tree_hash(&mut self) -> u64 {
        let current_parent = thread_local::current_parent();
        let current_last_child = self.nodes[current_parent.i].last_child;

        let mut hasher = ahasher();
            
        current_parent.hash(&mut hasher);
        current_last_child.hash(&mut hasher);
        
        return hasher.finish()   
    }

    pub fn debug_print_tree(&self) {
        let mut prefix = String::new();
        self.debug_print_node_recursive(ROOT_I, &mut prefix, true, false);
    }

    fn debug_print_node_recursive(&self, node_i: NodeI, prefix: &mut String, is_last: bool, is_hidden: bool) {
        let hidden_marker = if is_hidden { " [HIDDEN]" } else { "" };
        let currently_hidden = if self.nodes[node_i].currently_hidden { " (currently_hidden=true)" } else { "" };
        let exiting = if self.nodes[node_i].exiting { " (exiting=true)" } else { "" };
        
        let connector = if prefix.is_empty() {
            String::new()
        } else if is_last {
            "└── ".to_string()
        } else {
            "├── ".to_string()
        };
        
        println!("{}{}{}{}{}{}",
            prefix,
            connector,
            self.nodes[node_i].debug_name(),
            hidden_marker,
            currently_hidden,
            exiting,
        );

        let old_len = prefix.len();
        if is_last {
            prefix.push_str("    ");
        } else {
            prefix.push_str("│   ");
        }

        // Count children first to determine which is last
        let mut regular_count = 0;
        let mut hidden_count = 0;
        for_each_child_including_lingering!(self, self.nodes[node_i], _child, {
            regular_count += 1;
        });
        for_each_hidden_child!(self, self.nodes[node_i], _hidden_child, {
            hidden_count += 1;
        });
        let total_count = regular_count + hidden_count;

        // Traverse regular children
        let mut current_index = 0;
        for_each_child_including_lingering!(self, self.nodes[node_i], child, {
            let is_child_last = current_index == total_count - 1;
            self.debug_print_node_recursive(child, prefix, is_child_last, false);
            current_index += 1;
        });

        // Traverse hidden children
        for_each_hidden_child!(self, self.nodes[node_i], hidden_child, {
            let is_child_last = current_index == total_count - 1;
            self.debug_print_node_recursive(hidden_child, prefix, is_child_last, true);
            current_index += 1;
        });

        prefix.truncate(old_len);
    }
}


pub(crate) fn ahasher() -> ahash::AHasher {
    ahash::RandomState::with_seeds(567899776617, 113565788, 68634584565675377, 54345456222646).build_hasher()
}

use std::hash::Hash;
use std::hash::BuildHasher;
pub(crate) fn ahash<T: Hash>(value: &T) -> u64 {
    let mut hasher = ahasher();
    value.hash(&mut hasher);
    hasher.finish()
}

#[track_caller]
pub(crate) fn caller_location_id() -> u64 {
    let location = Location::caller();
    // Pointer equality probably works?
    // https://rustc-dev-guide.rust-lang.org/backend/implicit-caller-location.html#generating-code-for-track_caller-callees
    // This relies on `Location::internal_constructor` being const folded, and also other things. It's definitely not guaranteed.
    // Neither false positives nor false negatives are the end of the world though.
    // If this turns out to be dumb, just go back to hashing it.
    // Ideally the magic track_caller mechanism would just insert a compile-time hash.
    return &raw const (*location) as u64;
}

#[cfg(test)]
mod test_caller_location_id {
    use crate::caller_location_id;

    #[test]
    fn test_different() {
        fn no_duplicates<T: Eq + std::hash::Hash>(items: &[T]) -> bool {
            let mut seen = std::collections::HashSet::new();
            for item in items {
                if seen.contains(item) {
                    return false;
                }
                seen.insert(item);
            }
            true
        }

        let mut vec = Vec::with_capacity(50);
        vec.push(caller_location_id());
        vec.push(caller_location_id());
        vec.push(caller_location_id());
        vec.push(caller_location_id());
        vec.push(caller_location_id());
        vec.push(caller_location_id());
        vec.push(caller_location_id());
        vec.push(caller_location_id());
        vec.push(caller_location_id());
        vec.push(caller_location_id());
        vec.push(caller_location_id());
        vec.push(caller_location_id());
        vec.push(caller_location_id());
        vec.push(caller_location_id());
        vec.push(caller_location_id());
        vec.push(caller_location_id());
        vec.push(caller_location_id());
        vec.push(caller_location_id());
        vec.push(caller_location_id());
        vec.push(caller_location_id());
        vec.push(caller_location_id());
        vec.push(caller_location_id());

        assert!(no_duplicates(&vec));
    }

    #[test]
    fn test_same() {
        fn all_same<T: PartialEq>(vec: &[T]) -> bool {   
            let first = &vec[0];
            vec.iter().all(|item| item == first)
        }

        let mut vec = Vec::with_capacity(50);
        for _ in 0..200 {
            vec.push(caller_location_id());
        }

        assert!(all_same(&vec));
    }
}

/// A struct referring to a node that was [`added`](Ui::add) on the tree.
///  
/// Can be used to call [`nest()`](Self::nest()) and add more nodes as children of this one. 
pub struct UiParent {
    pub(crate) i: NodeI,
}
impl UiParent {
    pub(crate) fn new(node_i: NodeI) -> UiParent {
        return UiParent {
            i: node_i,
        }
    }

    /// Start a nested block in the GUI tree.
    /// 
    /// Inside the nested block, new nodes will be added as a child of the node that `self` refers to.
    /// 
    /// ```rust
    /// # use keru::*;
    /// # pub struct State {
    /// #     pub ui: Ui,
    /// # }
    /// #
    /// # impl State {
    /// #    fn declare_ui(&mut self) {
    /// #    let ui = &mut self.ui; 
    /// #
    /// # let parent = V_STACK;
    /// # let child = BUTTON;
    /// #
    /// //             ↓ returns a `UiParent`
    /// ui.add(parent).nest(|| {
    ///     ui.add(child);
    /// });
    /// #
    /// #   }
    /// # }
    /// ```
    /// 
    /// Since the `content` closure doesn't borrow or move anything, it sets no restrictions at all on what code can be ran inside it.
    /// You can keep accessing and mutating both the `Ui` object and the rest of the program state freely, as you'd outside of the closure. 
    ///  
    pub fn nest<T>(&self, content: impl FnOnce() -> T ) -> T {
        thread_local::push_parent(self);

        let result = content();

        thread_local::pop_parent();
    
        return result;
    }
}

#[allow(dead_code)]
#[track_caller]
pub(crate) fn with_info_log_timer<T>(operation_name: &str, f: impl FnOnce() -> T) -> T {
    if log::max_level() >= log::LevelFilter::Info {
        let start = std::time::Instant::now();
        let result = f();
        let elapsed = start.elapsed();
        log::info!("{} took {:?}", operation_name, elapsed);
        if elapsed > std::time::Duration::from_millis(2) {
        }
        result
    } else {
        f()
    }
}

// New partial borrow cope just dropped.
// Remember to but the vec back in place!
pub(crate) fn take_buffer_and_clear<T>(buf: &mut Vec<T>) -> Vec<T> {
    buf.clear();
    return mem::take(buf)
}

fn render_svg_group(scene: &mut vello_hybrid::Scene, items: &[vello_common::pico_svg::Item], transform: Affine) {
    use vello_common::pico_svg::Item;
    use vello_common::kurbo::Stroke;

    scene.set_transform(transform);

    for item in items {
        match item {
            Item::Fill(fill_item) => {
                scene.set_paint(fill_item.color);
                scene.fill_path(&fill_item.path);
            }
            Item::Stroke(stroke_item) => {
                let style = Stroke::new(stroke_item.width);
                scene.set_stroke(style);
                scene.set_paint(stroke_item.color);
                scene.stroke_path(&stroke_item.path);
            }
            Item::Group(group_item) => {
                let new_transform = transform * group_item.affine;
                render_svg_group(scene, &group_item.children, new_transform);
                scene.set_transform(transform);
            }
        }
    }
}