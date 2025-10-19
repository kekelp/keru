use crate::*;
use std::collections::hash_map::Entry;
use std::hash::Hasher;
use std::mem;
use std::panic::Location;
use bytemuck::{Pod, Zeroable};

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
    pub fn add<'a>(&mut self, params: impl Into<FullNodeParams<'a>>) -> UiParent
    {
        let params = params.into();
        let key = params.key_or_anon_key();
        let (i, _id) = self.add_or_update_node(key);
        self.set_params(i, &params);
        self.set_params_text(i, &params);
        return UiParent::new(i);
    }

    #[track_caller]
    pub(crate) fn add_or_update_node(&mut self, key: NodeKey) -> (NodeI, Id) {
        let frame = self.sys.current_frame;

        // todo: at least when using non-anonymous keys, I think there's no legit use case for twins anymore. it's always a mistake, I think. it should log out a warning or panic.

        // Check the node corresponding to the key's id.
        // We might find that the key has already been used in this same frame:
        //      in this case, we take note, and calculate a twin key to use to add a "twin" in the next section.
        // Otherwise, we add or refresh normally, and take note of the final i.
        let twin_check_result = match self.nodes.node_hashmap.entry(key.id_with_subtree()) {
            // Add a normal node (no twins).
            Entry::Vacant(v) => {
                let new_node = Node::new(&key, None, Location::caller(), frame);
                let final_i = NodeI::from(self.nodes.nodes.insert(new_node));
                v.insert(NodeMapEntry::new(final_i));

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

        return (real_final_i, real_final_id);
    }

    fn refresh_node(&mut self, i: NodeI) {
        self.nodes[i].animation_start_time = ui_time_f32();
        
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

    // todo split this in "reset_old_tree_links" and "add_child"?
    fn set_tree_links(&mut self, new_node_i: NodeI, parent_i: NodeI, depth: usize) {
        assert!(new_node_i != parent_i, "Keru: Internal error: tried to add a node as child of itself ({}). This shouldn't be possible.", self.nodes[new_node_i].debug_name());

        // clear old tree links
        self.nodes[new_node_i].old_first_child = self.nodes[new_node_i].first_child;
        self.nodes[new_node_i].old_next_sibling = self.nodes[new_node_i].next_sibling;
        

        self.nodes[new_node_i].last_child = None;
        self.nodes[new_node_i].first_child = None;
        self.nodes[new_node_i].n_children = 0;

        self.nodes[new_node_i].depth = depth;

        self.nodes[new_node_i].currently_hidden = false;

        self.nodes[new_node_i].user_states.clear();

        self.set_relayout_chain_root(new_node_i, parent_i);

        self.add_child(new_node_i, parent_i);
    }

    fn add_child(&mut self, new_node_i: NodeI, parent_i: NodeI) {
        self.nodes[new_node_i].parent = parent_i;
        self.nodes[new_node_i].prev_sibling = None;
        self.nodes[new_node_i].next_sibling = None;

        self.nodes[parent_i].n_children += 1;

        match self.nodes[parent_i].last_child {
            None => {
                {
                    let this = &mut *self;
                    this.nodes[parent_i].last_child = Some(new_node_i);
                    this.nodes[parent_i].first_child = Some(new_node_i);
                }
            },
            Some(last_child) => {
                let old_last_child = last_child;
                {
                    let this = &mut *self;
                    this.nodes[new_node_i].prev_sibling = Some(old_last_child);
                    this.nodes[old_last_child].next_sibling = Some(new_node_i);
                    this.nodes[parent_i].last_child = Some(new_node_i);
                }
            },
        };

        self.remove_hidden_child_if_it_exists(new_node_i, parent_i);
    }

    fn remove_hidden_child_if_it_exists(&mut self, child_i: NodeI, parent_i: NodeI) {
        if let Some(first_hidden_child) = self.nodes[parent_i].first_hidden_child {
            if first_hidden_child == child_i {
                self.nodes[parent_i].first_hidden_child = self.nodes[child_i].next_hidden_sibling;
                self.nodes[child_i].next_hidden_sibling = None;
                return;
            }
            
            // Track previous node while iterating through siblings
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
        let scroll = self.local_node_scroll(i);

        let parent_offset = if i != ROOT_I {
            let parent = self.nodes[i].parent;
            self.nodes[parent].animated_rect.top_left()
        } else {
            Xy::new(0.0, 0.0)
        };
        
        let current = &self.nodes[i].animated_rect;
        let target = self.nodes[i].local_layout_rect + parent_offset + scroll;
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
            self.nodes[i].last_click_rect_i = Some(self.sys.click_rects.len() - 1);
        } else {
            self.nodes[i].last_click_rect_i = None;
        }

        if self.nodes[i].params.is_scrollable() {
            let click_rect = self.click_rect(i);
            self.sys.scroll_rects.push(click_rect);
        }
        
        self.sys.z_cursor += Z_STEP;
        let z = self.sys.z_cursor;
        self.nodes[i].z = z;

        let draw_even_if_invisible = self.sys.inspect_mode;
        if let Some(rect) = self.render_rect_i(i, draw_even_if_invisible, None, false) {
            self.sys.rects.push(rect);
            self.nodes[i].last_rect_i = Some(self.sys.rects.len() - 1);
        } else {
            self.nodes[i].last_rect_i = None;
        }

        if let Some(image) =  self.nodes[i].imageref {
            if let Some(image_rect) = self.render_rect_i(i, draw_even_if_invisible, Some(image.tex_coords), true) {
                self.sys.rects.push(image_rect);
                 self.nodes[i].last_image_rect_i = Some(self.sys.rects.len() - 1);
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
                    let mut text_box = self.sys.text.get_text_box_mut(&text_box_handle);
                    text_box.set_depth(z);
                    text_box.set_pos((left, top));
                },
                TextI::TextEdit(text_edit_handle) => {
                    let mut text_edit = self.sys.text.get_text_edit_mut(&text_edit_handle);
                    text_edit.set_depth(z);
                    text_edit.set_pos((left, top));
                },
            }
        }
    }

    pub(crate) fn update_rect(&mut self, i: NodeI) {
        if let Some(old_i) = self.nodes[i].last_click_rect_i {
            let click_rect = self.click_rect(i);
            self.sys.click_rects[old_i] = click_rect;
        }

        // todo: update scroll rect
        // at this point, maybe split the cosmetic or size (click,scroll) updates?

        let draw_even_if_invisible = self.sys.inspect_mode;
        if let Some(old_i) = self.nodes[i].last_rect_i {
            if let Some(rect) = self.render_rect_i(i, draw_even_if_invisible, None, false) {
                self.sys.rects[old_i] = rect;
            }
        }
        
        if let Some(imageref) = self.nodes[i].imageref {
            if let Some(image_rect) = self.render_rect_i(i, draw_even_if_invisible, Some(imageref.tex_coords), true) {
                let old_i = self.nodes[i].last_image_rect_i.unwrap();
                self.sys.rects[old_i] = image_rect;
            }
        }

        // this kind of makes sense, but apparently not needed? I guess someone else is calling it?
        // self.sys.changes.need_gpu_rect_update = true;
        // todo: update images?
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

    pub(crate) fn push_partial_relayout(&mut self, i: NodeI) {
        let relayout_chain_root = match self.nodes[i].relayout_chain_root {
            Some(root) => root,
            None => i,
        };

        // even after the chain, we still have to go one layer up, because a different sized child probably means that the parent wants to place the node differently, and maybe pick a different size and position for the other children as well
        // In practice, the first half of that is basically always true, but the second half is only true for Stacks. I don't really feel like adding a distinction for that right now.
        let relayout_target = self.nodes[relayout_chain_root].parent;

        let relayout_entry = NodeWithDepth {
            i: relayout_target,
            depth: self.nodes[relayout_target].depth,
        };
        self.sys.changes.partial_relayouts.push(relayout_entry);
    }

    pub(crate) fn push_cosmetic_update(&mut self, i: NodeI) {
        self.sys.changes.cosmetic_rect_updates.push(i);
    }

    pub(crate) fn push_text_change(&mut self, i: NodeI) {
        if self.nodes[i].params.is_fit_content() {
            self.push_partial_relayout(i);
        } else {
            self.sys.changes.need_rerender = true;
        }
        self.sys.changes.text_changed = true;
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

        let root_parent = UiParent::new(ROOT_I);
        thread_local::push_parent(&root_parent);

        self.begin_frame_resolve_inputs();
    }
    
    fn reset_root(&mut self) {
        self.nodes[ROOT_I].old_first_child = self.nodes[ROOT_I].first_child;
        self.nodes[ROOT_I].old_next_sibling = self.nodes[ROOT_I].next_sibling;
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
                let animation_not_over = self.node_or_parent_has_ongoing_animation(i);
                if old_parent_still_exists && self.nodes[i].exiting && animation_not_over {

                    exiting_nodes.push(NodeWithDepth { i, depth: self.nodes[i].depth });
                    
                } else if ! can_hide {

                    to_cleanup.push(i);

                    if children_can_hide {
                        hidden_branch_parents.push(i);
                    }

                } else if ! currently_hidden {
                    
                    self.nodes[i].currently_hidden = true;

                    if is_first_child_in_hidden_branch {
                        self.add_hidden_child(i, old_parent);
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

        // todo: push partial relayouts instead.
        self.sys.changes.full_relayout = true;

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

        for state_id in &self.nodes[i].user_states {
            self.sys.user_state.remove(state_id);
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
