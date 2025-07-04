use crate::*;
use std::collections::hash_map::Entry;
use std::hash::Hasher;
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

pub(crate) const Z_BACKDROP: f32 = 0.5;
// This one has to be small, but not small enough to get precision issues.
// And I think it's probably good if it's a rounded binary number (0x38000000)? Not sure.
pub(crate) const Z_STEP: f32 = -0.000_030_517_578;

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
        let i = self.add_or_update_node(key);
        self.set_params(i, &params);
        self.set_params_text(i, &params);
        return UiParent::new(i);
    }

    #[track_caller]
    pub(crate) fn add_or_update_node(&mut self, key: NodeKey) -> NodeI {
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
                v.insert(NodeMapEntry::new(frame, final_i));

                UpdatedNormal { final_i }
            }
            Entry::Occupied(o) => {
                let old_map_entry = o.into_mut();

                match should_refresh_or_add_twin(frame, old_map_entry.last_frame_touched) {
                    // Refresh a normal node from the previous frame (no twins).
                    Refresh => {
                        old_map_entry.refresh(frame);
                        // in this branch we don't really do anything now. there will be a separate thing for updating params
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
        let (real_final_i, _real_final_id) = match twin_check_result {
            UpdatedNormal { final_i } => (final_i, key.id_with_subtree()),
            NeedToUpdateTwin { twin_key, twin_n } => {
                match self.nodes.node_hashmap.entry(twin_key.id_with_subtree()) {
                    // Add new twin.
                    Entry::Vacant(v) => {
                        let new_twin_node = Node::new(&twin_key, Some(twin_n), Location::caller(), frame);
                        let real_final_i = NodeI::from(self.nodes.nodes.insert(new_twin_node));
                        v.insert(NodeMapEntry::new(frame, real_final_i));
                        (real_final_i, twin_key.id_with_subtree())
                    }
                    // Refresh a twin from the previous frame.
                    Entry::Occupied(o) => {
                        let old_twin_map_entry = o.into_mut();

                        let real_final_i = old_twin_map_entry.refresh(frame);

                        (real_final_i, twin_key.id_with_subtree())
                    }
                }
            }
        };

        // update the in-tree links and the thread-local state based on the current parent.
        let NodeWithDepth { i: parent_i, depth } = thread_local::current_parent();
        self.set_tree_links(real_final_i, parent_i, depth);

        // refresh the text box associated with this node if it has one
        if let Some(text_i) = &self.nodes[real_final_i].text_i {
            match text_i {
                TextI::TextBox(handle) => {
                    self.sys.text.refresh_text_box(handle);
                }
                TextI::_StaticTextBox(handle) => {
                    self.sys.text.refresh_static_text_box(handle);
                }
                TextI::TextEdit(handle) => {
                    self.sys.text.refresh_text_edit(handle);
                }
            }
        }

        return real_final_i;
    }

    fn set_tree_links(&mut self, new_node_i: NodeI, parent_i: NodeI, depth: usize) {
        assert!(new_node_i != parent_i, "Keru: Internal error: tried to add a node as child of itself ({}). This shouldn't be possible.", self.nodes[new_node_i].debug_name());

        // clear old tree links
        self.nodes[new_node_i].old_first_child = self.nodes[new_node_i].first_child;
        self.nodes[new_node_i].old_next_sibling = self.nodes[new_node_i].next_sibling;
        
        self.nodes[new_node_i].last_child = None;
        self.nodes[new_node_i].first_child = None;
        self.nodes[new_node_i].prev_sibling = None;
        self.nodes[new_node_i].next_sibling = None;
        self.nodes[new_node_i].n_children = 0;

        self.nodes[new_node_i].depth = depth;
        self.nodes[new_node_i].parent = parent_i;

        self.nodes[new_node_i].user_states.clear();

        self.set_relayout_chain_root(new_node_i, parent_i);

        self.add_child(new_node_i, parent_i);
    }

    fn add_child(&mut self, new_node_i: NodeI, parent_i: NodeI) {
        self.nodes[parent_i].n_children += 1;

        match self.nodes[parent_i].last_child {
            None => {
                self.add_first_child(new_node_i, parent_i)
            },
            Some(last_child) => {
                let old_last_child = last_child;
                self.add_sibling(new_node_i, old_last_child, parent_i)
            },
        };

        self.remove_hidden_child_if_it_exists(new_node_i, parent_i);
    }

    fn add_first_child(&mut self, new_node_i: NodeI, parent_i: NodeI) {
        self.nodes[parent_i].last_child = Some(new_node_i);
        self.nodes[parent_i].first_child = Some(new_node_i);
    }
    
    fn add_sibling(&mut self, new_node_i: NodeI, old_last_child: NodeI, parent_i: NodeI) {
        self.nodes[new_node_i].prev_sibling = Some(old_last_child);
        self.nodes[old_last_child].next_sibling = Some(new_node_i);
        self.nodes[parent_i].last_child = Some(new_node_i);
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

    pub(crate) fn push_rect(&mut self, i: NodeI) {
        let debug = cfg!(debug_assertions);
        let push_click_rect = if debug && self.inspect_mode() {
            true
        } else {
            let clickable = self.nodes[i].params.interact.senses != Sense::NONE;
            let editable = if let Some(text_i) = &self.nodes[i].text_i {
                match text_i {
                    TextI::TextEdit(_) => true,
                    TextI::TextBox(_) | TextI::_StaticTextBox(_) => false,
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
        
        // really only need to do this whenever a custom-rendered rect shows up. But that would require custom rendered rects to be specifically marked, as opposed to just being the same as any other visible-only-in-debug rect, which means that you can forget to mark it and mess everything up. There's no real disadvantage to just always doing it.
        self.sys.z_cursor += Z_STEP;
         self.nodes[i].z = self.sys.z_cursor;

        let draw_even_if_invisible = self.sys.inspect_mode;
        if let Some(rect) = self.render_rect_i(i, draw_even_if_invisible, None) {
            self.sys.rects.push(rect);
             self.nodes[i].last_rect_i = Some(self.sys.rects.len() - 1);
        } else {
             self.nodes[i].last_rect_i = None;
        }

        if let Some(image) =  self.nodes[i].imageref {
            if let Some(image_rect) = self.render_rect_i(i, draw_even_if_invisible, Some(image.tex_coords)) {
                self.sys.rects.push(image_rect);
                 self.nodes[i].last_image_rect_i = Some(self.sys.rects.len() - 1);
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
            if let Some(rect) = self.render_rect_i(i, draw_even_if_invisible, None) {
                self.sys.rects[old_i] = rect;
            }
        }
        
        if let Some(imageref) = self.nodes[i].imageref {
            if let Some(image_rect) = self.render_rect_i(i, draw_even_if_invisible, Some(imageref.tex_coords)) {
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

        // Note: default_text_style_changed logic removed in new centralized text system
        // Style management is now handled centrally by the Text struct

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

        self.diff_children();
        self.relayout();
        self.cleanup();
        
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
            return entry.last_frame_touched == self.sys.current_frame;
        } else {
            return false;
        }
    }

    fn diff_children(&mut self) {
        self.sys.added_nodes.clear();
        self.sys.direct_removed_nodes.clear();
        self.sys.indirect_removed_nodes.clear();
        self.sys.very_indirect_removed_nodes.clear();
        self.sys.hidden_nodes.clear();
        self.sys.hidden_stack.clear();
        
        self.recursive_diff_children(ROOT_I);

        // if the tree changes, all rects have to be rebuilt. this might change if the rects become densemaps or whatever
        if !self.sys.added_nodes.is_empty()
            || !self.sys.direct_removed_nodes.is_empty()
            || !self.sys.hidden_nodes.is_empty()
        {
            self.sys.changes.tree_changed = true;
        }

        // push partial relayouts
        for k in 0..self.sys.added_nodes.len() {
            // the recursive_diff_children traversal uses the old tree, so it can miss a lot of added rects that are added to new children directly. This is fine though because added_rects is just for relayouts.
            let i = self.sys.added_nodes[k];
            self.push_partial_relayout(i);
        }
        for k in 0..self.sys.direct_removed_nodes.len() {
            let i = self.sys.direct_removed_nodes[k];
            self.push_partial_relayout(i);
        }
        for k in 0..self.sys.hidden_nodes.len() {
            let i = self.sys.hidden_nodes[k];
            self.push_partial_relayout(i);
        }
    }

    fn recursive_diff_children(&mut self, i: NodeI) {
        let id = self.nodes[i].id;
        let freshly_added = self.nodes.node_hashmap[&id].last_frame_touched == self.sys.current_frame;

        let new_hidden_branch = freshly_added && self.nodes[i].params.children_can_hide;
        if new_hidden_branch {
            self.sys.hidden_stack.push(i);
        }

        if i == ROOT_I || freshly_added {
            // collect old and new children
            self.sys.new_child_collect.clear();        
            self.sys.old_child_collect.clear();
            
            for_each_child!(self, self.nodes[i], child, {
                self.sys.new_child_collect.push(child);
            });
            for_each_old_child!(self, self.nodes[i], child, {
                self.sys.old_child_collect.push(child);
            });

            // diff the arrays
            // todo: use hashsets? probably not needed at all
            for &new_child in &self.sys.new_child_collect {
                if !self.sys.old_child_collect.contains(&new_child) {
                    self.sys.added_nodes.push(new_child);
                }
            }
            for k in 0..self.sys.old_child_collect.len() {
                let old_child = self.sys.old_child_collect[k];
                if !self.sys.new_child_collect.contains(&old_child) {

                    if new_hidden_branch {
                        self.sys.hidden_nodes.push(old_child);

                        let old_child_id = self.nodes[old_child].id;
                        // as usual, if the hidden node is actually freshly added, that means that it wasn't hidden, but just moved somewhere else in the frame. In that case if we did add_hidden_child it would be pretty bad.
                        if self.nodes.node_hashmap[&old_child_id].last_frame_touched != self.sys.current_frame {
                            
                            self.add_hidden_child(old_child, i);

                            log::trace!("Not removing {:?}, as its parent has can_hide_children = true", self.node_debug_name_fmt_scratch(old_child));
                        } else {
                            log::trace!("Not removing {:?}, as its parent has can_hide_children = true. But not setting it as hidden either, as it has merely moved to another position in the tree. Wow, what an edge case!", self.node_debug_name_fmt_scratch(old_child));
                        }

                    } else {
                        // no hidden crap, just remove. We could to the moved-somewhere-else check here for symmetry, but it's working fine down in garbage_collect_node
                        self.sys.direct_removed_nodes.push(old_child);
                    }
                }
            }

            // continue recursion on old children
            
            // in a hidden branch, there should be no chance of any node being freshly added, and the nodes don't get garbage collected, so there's no need to recurse at all.
            // (if we recursed anyway, presumably we could do retained-mode hiding text boxes instead of the last-frame-touched thing. But we would have to pass a "in_hidden_branch" parameter.)
            // (Why don't we just do everything last-frame-touched style anyway? Maybe so that we can do more accurate partial relayouts? It would also be a bit complicated to tell if orphaned children should be removed or hidden. But would it be more complicated than this?)
            if ! new_hidden_branch {
                for_each_old_child!(self, self.nodes[i], child, {
                    self.recursive_diff_children(child);
                });
            }

        } else {
            // orphaned children of old nodes
            // these ones were never visited, so their tree links weren't even updated. so the traversal uses for_each_child, not for_each_old_child

            // Add all their nodes to removed without diffing
            for_each_child!(self, self.nodes[i], child, {
                // this check isn't needed because hidden branches aren't even traversed.
                // however it's still here for safety.
                let in_hidden_branch = ! self.sys.hidden_stack.is_empty();
                if ! in_hidden_branch {
                    self.sys.indirect_removed_nodes.push(child);
                }
            });

            // continue recursion. down here, all nodes should all be not freshly added
            for_each_child!(self, self.nodes[i], child, {
                self.recursive_diff_children(child);
            });
        }

        if let Some(pop_hidden) = self.sys.hidden_stack.last() {
            if *pop_hidden == i {
                self.sys.hidden_stack.pop();
            }
        }
    }

    fn cleanup(&mut self) {
        // if any of the removed nodes have hidden children, also add those nodes (and their whole branch) to the cleanup.
        for k in 0..self.sys.direct_removed_nodes.len() {
            let i = self.sys.direct_removed_nodes[k];
            for_each_hidden_child!(self, self.nodes[i], hidden_child, {
                self.recursive_set_as_toremove_indirect_hidden_children(hidden_child);
            });
        }
        for k in 0..self.sys.indirect_removed_nodes.len() {
            let i = self.sys.indirect_removed_nodes[k];
            for_each_hidden_child!(self, self.nodes[i], hidden_child, {
                self.recursive_set_as_toremove_indirect_hidden_children(hidden_child);
            });
        }
        // no need to clear 

        for k in 0..self.sys.direct_removed_nodes.len() {
            let i = self.sys.direct_removed_nodes[k];
            self.cleanup_node(i);
        }
        for k in 0..self.sys.indirect_removed_nodes.len() {
            let i = self.sys.indirect_removed_nodes[k];
            self.cleanup_node(i);
        }
        for k in 0..self.sys.very_indirect_removed_nodes.len() {
            let i = self.sys.very_indirect_removed_nodes[k];
            self.cleanup_node(i);
        }
    }

    fn cleanup_node(&mut self, i: NodeI) {
        if ! self.nodes.nodes.contains(i.as_usize()) {
            log::error!("Keru: Internal error: tried to cleanup the same node twice. ({:?})", i);
            // we could cheat and just return. instead we continue, so we can see the panic clearly in case there's any bugs.
        }
        let id = self.nodes[i].id;
        
        // skip the nodes that have last_frame_touched = now, because that means that they were not really removed, but just moved somewhere else in the tree.
        // Kind of weird to do this so late.
        if self.nodes.node_hashmap[&id].last_frame_touched == self.sys.current_frame {
            log::trace!("Not removing: {:?}, as it was moved around and not removed", self.node_debug_name_fmt_scratch(i));
            return;
        }

        log::trace!("Removing {:?} ({:?})", self.node_debug_name_fmt_scratch(i), i);
        let old_handle = self.nodes[i].text_i.take();
        if let Some(text_i) = old_handle {
            match text_i {
                TextI::TextBox(handle) => {
                    self.sys.text.remove_text_box(handle);
                }
                TextI::_StaticTextBox(handle) => {
                    self.sys.text.remove_static_text_box(handle);
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

    fn recursive_set_as_toremove_indirect_hidden_children(&mut self, i: NodeI) {
        self.sys.very_indirect_removed_nodes.push(i);
        for_each_child!(self, self.nodes[i], child, {
            self.recursive_set_as_toremove_indirect_hidden_children(child);
        });
    }

    pub(crate) fn current_tree_hash(&mut self) -> u64 {
        let current_parent = thread_local::current_parent();
        let current_last_child = self.nodes[current_parent.i].last_child;

        let mut hasher = ahasher();
            
        current_parent.hash(&mut hasher);
        current_last_child.hash(&mut hasher);
        
        return hasher.finish()   
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
