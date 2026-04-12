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

pub(crate) const Z_START: f32 = 0.5;
pub const Z_STEP: f32 = -0.000_030_517_578;

impl Ui {
    /// Add a node to the `Ui`.
    /// 
    /// `node` can be a [`Node`] or a [`FullNode`].
    /// 
    /// ```no_run
    /// # use keru::*;
    /// # let mut ui: Ui = unimplemented!();
    /// let red_label = LABEL
    ///     .color(RED)
    ///     .text("Increase");
    /// 
    /// ui.add(red_label);
    /// ```
    /// 
    /// Buttons, images, text elements, stack containers, etc. are all created by `add`ing a [`Node`] with the right fields.
    #[track_caller]
    pub fn add<'a>(&mut self, node: impl Into<FullNode<'a>>) -> UiParent
    {
        let params = node.into();
        let key = params.key_or_anon_key();
        let (i, _id) = self.add_or_update_node(key);
        self.set_params(i, &params);
        self.set_params_text(i, &params);
        return UiParent { i, sibling_cursor: SiblingCursor::None, ui_instance_id: self.sys.unique_id };
    }

    /// Returns an [`UiParent`] for the root node, that you can use to nest children directly into the root node, regardless of where you are in the [`nest`] structure.
    /// 
    /// This is sort of a crazy thing to do, but here's an example of why it might be useful:
    /// 
    /// ```no_run
    /// # use keru::*;
    /// # let mut ui: Ui = unimplemented!();
    /// // A list of elements that can be dragged away from the container
    /// #[node_key] pub const SOME_KEY: NodeKey;
    /// ui.add(V_STACK).nest(|| {
    ///     for i in 0..10 {
    ///         let key = SOME_KEY.sibling(i);
    ///         let element = LABEL.text("Element").key(key);
    ///     
    ///         if let Some(drag) = ui.is_dragged(SOME_KEY) {
    ///             // Add the element that's being dragged to the root
    ///             ui.jump_to_root().nest(|| {
    ///                 let mouse_pos = todo!();
    ///                 ui.add(element.position_symm(mouse_pos));
    ///             });
    ///         } else {
    ///             // add all the other elements to the stack normally
    ///             ui.add(element);
    ///         }
    ///     }
    /// });
    /// // Adding it to the root is an easy way to make the element follow the mouse without doing any math.
    /// // Not adding it to the stack means that the other elements get the correct animations, without any sort of special-casing.
    /// ```
    pub fn jump_to_root(&self) -> UiParent {
        return UiParent { i: ROOT_I, sibling_cursor: SiblingCursor::None, ui_instance_id: self.sys.unique_id }
    }


    /// If the node corresponding to `parent_key` exists, get a [`UiParent`] that can be used to break the normal nesting structure and add children to it.
    ///
    /// This is like [`jump_to_root`](Self::jump_to_root) but for any node.
    pub fn jump_to_parent(&self, parent_key: NodeKey) -> Option<UiParent> {
        let parent_i = self.sys.nodes.node_hashmap.get(&parent_key.id_with_subtree())?.slab_i;
        Some(UiParent {
            i: parent_i,
            sibling_cursor: SiblingCursor::None,
            ui_instance_id: self.sys.unique_id,
        })
    }

    /// If the node corresponding to `jump_key` exists, get a [`UiParent`] that can be used to break the normal nesting structure and add nodes after it.
    /// 
    /// The nested nodes will be added to `jump_key`'s parent, right after `jump_key`.
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use keru::*;
    /// # let mut ui: Ui = unimplemented!();
    /// #[node_key] const ITEM: NodeKey;
    /// let items = ["A", "B", "C", "D", "E"];
    /// ui.add(H_STACK).nest(|| {
    ///     ui.add(V_STACK).nest(|| {
    ///         for item in items {
    ///             ui.add(BUTTON.text(&item).key(ITEM.sibling(&item)));
    ///         }
    ///     });
    ///     
    ///     // Add a red "X" between "B" and "C"
    ///     let jump_key = ITEM.sibling("B");
    ///     ui.jump_to_sibling(jump_key).unwrap().nest(|| {
    ///         ui.add(BUTTON.text("X").color(RED));
    ///     });
    /// });
    /// ```
    pub fn jump_to_sibling(&self, jump_key: NodeKey) -> Option<UiParent> {
        let sibling_i = self.sys.nodes.node_hashmap.get(&jump_key.id_with_subtree())?.slab_i;
        let parent_i = self.sys.nodes[sibling_i].parent;
        Some(UiParent {
            i: parent_i,
            sibling_cursor: SiblingCursor::After(sibling_i),
            ui_instance_id: self.sys.unique_id,
        })
    }

    /// If the node corresponding to `jump_key` exists, get a [`UiParent`] that can be used to break the normal nesting structure and add nodes before it.
    /// 
    /// The nested nodes will be added to `jump_key`'s parent, right before `jump_key`.
    /// 
    /// # Example
    /// 
    /// ```no_run
    /// # use keru::*;
    /// # let mut ui: Ui = unimplemented!();
    /// #[node_key] const ITEM: NodeKey;
    /// let items = ["A", "B", "C", "D", "E"];
    /// ui.add(H_STACK).nest(|| {
    ///     ui.add(V_STACK).nest(|| {
    ///         for item in items {
    ///             ui.add(BUTTON.text(&item).key(ITEM.sibling(&item)));
    ///         }
    ///     });
    ///     
    ///     // Add a red "X" between "B" and "C"
    ///     let jump_key = ITEM.sibling("C");
    ///     ui.jump_to_before_sibling(jump_key).unwrap().nest(|| {
    ///         ui.add(BUTTON.text("X").color(RED));
    ///     });
    /// });
    /// ```
    pub fn jump_to_before_sibling(&self, jump_key: NodeKey) -> Option<UiParent> {
        let sibling_i = self.sys.nodes.node_hashmap.get(&jump_key.id_with_subtree())?.slab_i;
        let parent_i = self.sys.nodes[sibling_i].parent;
        let sibling_cursor = match self.sys.nodes[sibling_i].prev_sibling {
            Some(prev) => SiblingCursor::After(prev),
            None => SiblingCursor::AtStart,
        };
        Some(UiParent {
            i: parent_i,
            sibling_cursor,
            ui_instance_id: self.sys.unique_id,
        })
    }

    /// If the node corresponding to `parent_key` exists, get a [`UiParent`] positioned after its nth child.
    ///
    /// `n = 0` means insert before the first child, `n = 1` means insert after the first child, etc.
    /// If `n` is greater than the number of children, inserts at the end.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use keru::*;
    /// # let mut ui: Ui = unimplemented!();
    /// #[node_key] const ITEM: NodeKey;
    /// #[node_key] const MY_STACK: NodeKey;
    /// let items = ["A", "B", "C", "D", "E"];
    /// ui.add(H_STACK).nest(|| {
    ///     ui.add(V_STACK.key(MY_STACK)).nest(|| {
    ///         for item in items {
    ///             ui.add(BUTTON.text(&item).key(ITEM.sibling(&item)));
    ///         }
    ///     });
    ///
    ///     // Add a red "X" between "B" and "C"
    ///     ui.jump_to_nth_child(MY_STACK, 2).unwrap().nest(|| {
    ///         ui.add(BUTTON.text("X").color(RED));
    ///     });
    /// });
    /// ```
    pub fn jump_to_nth_child(&self, parent_key: NodeKey, n: usize) -> Option<UiParent> {
        let parent_i = self.sys.nodes.node_hashmap.get(&parent_key.id_with_subtree())?.slab_i;

        if n == 0 {
            return Some(UiParent {
                i: parent_i,
                sibling_cursor: SiblingCursor::AtStart,
                ui_instance_id: self.sys.unique_id,
            });
        }

        // Walk to the nth child
        let mut current = self.sys.nodes[parent_i].first_child;
        for _ in 1..n {
            match current {
                Some(child_i) => current = self.sys.nodes[child_i].next_sibling,
                None => break,
            }
        }

        let sibling_cursor = match current {
            Some(child_i) => SiblingCursor::After(child_i),
            None => SiblingCursor::None,
        };

        Some(UiParent {
            i: parent_i,
            sibling_cursor,
            ui_instance_id: self.sys.unique_id,
        })
    }

    /// Remove a node from its current position in the tree and re-add it at the current parent context.
    ///
    /// This is useful for moving nodes between parents, for example moving a dragged item to a floating layer.
    ///
    /// Returns `None` if the node doesn't exist.
    pub fn remove_and_readd(&mut self, key: NodeKey) -> Option<()> {
        let node_i = self.sys.nodes.node_hashmap.get(&key.id_with_subtree())?.slab_i;
        self.unlink_from_tree(node_i);

        let (parent_i, sibling_cursor, depth) = thread_local::current_parent(self.sys.unique_id);
        self.link_node_to_parent(node_i, parent_i, depth, sibling_cursor);

        Some(())
    }

    /// Unlink a node from its current parent's child list.
    fn unlink_from_tree(&mut self, node_i: NodeI) {
        let old_parent = self.sys.nodes[node_i].parent;
        let prev = self.sys.nodes[node_i].prev_sibling;
        let next = self.sys.nodes[node_i].next_sibling;

        match prev {
            Some(prev_i) => self.sys.nodes[prev_i].next_sibling = next,
            None => self.sys.nodes[old_parent].first_child = next,
        }

        match next {
            Some(next_i) => self.sys.nodes[next_i].prev_sibling = prev,
            None => self.sys.nodes[old_parent].last_child = prev,
        }

        // Decrement parent's child count
        self.sys.nodes[old_parent].n_children = self.sys.nodes[old_parent].n_children.saturating_sub(1);

        // Clear the node's sibling pointers
        self.sys.nodes[node_i].prev_sibling = None;
        self.sys.nodes[node_i].next_sibling = None;
    }

    #[track_caller]
    pub(crate) fn add_or_update_node(&mut self, key: NodeKey) -> (NodeI, Id) {
        let frame = self.sys.current_frame;
        let mut new_node_should_relayout = false;

        // Check the node corresponding to the key's id.
        // We might find that the key has already been used in this same frame:
        //      in this case, we take note, and calculate a twin key to use to add a "twin" in the next section.
        // Otherwise, we add or refresh normally, and take note of the final i.
        let twin_check_result = match self.sys.nodes.node_hashmap.entry(key.id_with_subtree()) {
            // Add a new normal node (no twins).
            Entry::Vacant(v) => {
                let new_node = InnerNode::new(&key, None, Location::caller(), frame);
                let final_i = NodeI::from(self.sys.nodes.nodes.insert(new_node));
                v.insert(NodeMapEntry::new(final_i));

                new_node_should_relayout = true;

                UpdatedNormal { final_i }
            }
            Entry::Occupied(o) => {
                let old_map_entry = o.into_mut();
                let old_i = old_map_entry.slab_i.as_usize();
                let last_frame_touched = self.sys.nodes.nodes[old_i].last_frame_touched;

                match should_refresh_or_add_twin(frame, last_frame_touched) {
                    // Refresh a normal node from the previous frame (no twins).
                    Refresh => {
                        old_map_entry.refresh();
                        self.sys.nodes.nodes[old_i].last_frame_touched = frame;
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
                match self.sys.nodes.node_hashmap.entry(twin_key.id_with_subtree()) {
                    // Add new twin.
                    Entry::Vacant(v) => {
                        let new_twin_node = InnerNode::new(&twin_key, Some(twin_n), Location::caller(), frame);
                        let real_final_i = NodeI::from(self.sys.nodes.nodes.insert(new_twin_node));
                        v.insert(NodeMapEntry::new(real_final_i));
                        new_node_should_relayout = true;
                        (real_final_i, twin_key.id_with_subtree())
                    }
                    // Refresh a twin from the previous frame.
                    Entry::Occupied(o) => {
                        let old_twin_map_entry = o.into_mut();

                        let real_final_i = old_twin_map_entry.refresh();
                        self.sys.nodes.nodes[real_final_i.as_usize()].last_frame_touched = frame;

                        (real_final_i, twin_key.id_with_subtree())
                    }
                }
            }
        };

        // update the in-tree links and the thread-local state based on the current parent.
        let (parent, insert_after, depth) = thread_local::current_parent(self.sys.unique_id);
        self.set_tree_links(real_final_i, parent, depth, insert_after);

        self.sys.nodes[real_final_i].exiting = false;

        self.refresh_node(real_final_i);

        if new_node_should_relayout {
            self.push_partial_relayout(real_final_i);
        }

        return (real_final_i, real_final_id);
    }

    fn refresh_node(&mut self, i: NodeI) {
        if let Some(text_i) = &self.sys.nodes[i].text_i {
            match text_i {
                TextI::TextBox(handle) => {
                    self.sys.renderer.text.refresh_text_box(handle);
                }
                TextI::TextEdit(handle) => {
                    self.sys.renderer.text.refresh_text_edit(handle);
                }
            }
        }

        self.sys.nodes[i].canvas_instances = None;
    }

    // this function also detects new nodes and reorderings, and pushes partial relayouts for them. For deleted nodes, partial relayouts will be pushed in cleanup_and_stuff.
    fn set_tree_links(&mut self, new_node_i: NodeI, parent_i: NodeI, depth: usize, sibling_cursor: SiblingCursor) {
        self.clear_node_children(new_node_i);
        self.link_node_to_parent(new_node_i, parent_i, depth, sibling_cursor);
    }

    fn clear_node_children(&mut self, new_node_i: NodeI) {
        // Reset old links
        self.sys.nodes[new_node_i].old_first_child = self.sys.nodes[new_node_i].first_child;
        self.sys.nodes[new_node_i].old_next_sibling = self.sys.nodes[new_node_i].next_sibling;

        self.sys.nodes[new_node_i].first_child = None;
        self.sys.nodes[new_node_i].last_child = None;
        self.sys.nodes[new_node_i].n_children = 0;
    }

    fn link_node_to_parent(&mut self, new_node_i: NodeI, parent_i: NodeI, depth: usize, sibling_cursor: SiblingCursor) {
        assert!(new_node_i != parent_i, "Keru: Internal error: tried to add a node as child of itself ({}). This shouldn't be possible.", self.sys.nodes[new_node_i].debug_name());

        self.sys.nodes[new_node_i].depth = depth;
        self.sys.nodes[new_node_i].currently_hidden = false;

        // If parent changed, convert local_animated_rect to the new parent's coordinate space using screen-space positions from the previous frame.
        let old_parent = self.sys.nodes[new_node_i].parent;
        let is_new_node = self.sys.nodes[new_node_i].frame_added == self.sys.current_frame;
        if !is_new_node && old_parent != parent_i {
            let screen_pos = self.sys.nodes[new_node_i].real_rect;
            let new_parent_offset = self.sys.nodes[parent_i].real_rect.top_left();
            self.sys.nodes[new_node_i].local_animated_rect = screen_pos - new_parent_offset;
        }

        // Add new child
        self.sys.nodes[new_node_i].parent = parent_i;
        self.sys.nodes[new_node_i].prev_sibling = None;
        self.sys.nodes[new_node_i].next_sibling = None;

        self.sys.nodes[parent_i].n_children += 1;

        match sibling_cursor {
            // Add after last sibling (no jump)
            SiblingCursor::None => {
                match self.sys.nodes[parent_i].last_child {
                    None => {
                        // First child
                        self.sys.nodes[parent_i].first_child = Some(new_node_i);
                        self.sys.nodes[parent_i].last_child = Some(new_node_i);

                        if self.sys.nodes[parent_i].first_child != self.sys.nodes[parent_i].old_first_child {
                            self.push_partial_relayout(parent_i);
                        }
                    },
                    Some(last_child) => {
                        let prev_sibling = last_child;
                        // Append after last_child
                        self.sys.nodes[new_node_i].prev_sibling = Some(prev_sibling);
                        self.sys.nodes[prev_sibling].next_sibling = Some(new_node_i);
                        self.sys.nodes[parent_i].last_child = Some(new_node_i);

                        if self.sys.nodes[prev_sibling].old_next_sibling != self.sys.nodes[prev_sibling].next_sibling {
                            self.push_partial_relayout(parent_i);
                        }
                    },
                }
            },
            // Add at the start (before first child)
            SiblingCursor::AtStart => {
                let old_first = self.sys.nodes[parent_i].first_child;

                self.sys.nodes[new_node_i].next_sibling = old_first;
                self.sys.nodes[parent_i].first_child = Some(new_node_i);

                match old_first {
                    Some(old_first_i) => {
                        self.sys.nodes[old_first_i].prev_sibling = Some(new_node_i);
                    }
                    None => {
                        self.sys.nodes[parent_i].last_child = Some(new_node_i);
                    }
                }

                // Manually advance the thread-local cursor
                thread_local::set_sibling_cursor(SiblingCursor::After(new_node_i));

                self.push_partial_relayout(parent_i);
            },
            // Add after a specific sibling
            SiblingCursor::After(after_i) => {
                let old_next = self.sys.nodes[after_i].next_sibling;

                self.sys.nodes[new_node_i].prev_sibling = Some(after_i);
                self.sys.nodes[new_node_i].next_sibling = old_next;
                self.sys.nodes[after_i].next_sibling = Some(new_node_i);

                match old_next {
                    Some(old_next_i) => {
                        self.sys.nodes[old_next_i].prev_sibling = Some(new_node_i);
                    }
                    None => {
                        self.sys.nodes[parent_i].last_child = Some(new_node_i);
                    }
                }

                // Manually advance the thread-local cursor
                thread_local::set_sibling_cursor(SiblingCursor::After(new_node_i));

                self.push_partial_relayout(parent_i);
            },
        };

        self.set_relayout_chain_root(new_node_i, parent_i);

        self.remove_hidden_child_if_it_exists(new_node_i, parent_i);
    }

    fn remove_hidden_child_if_it_exists(&mut self, child_i: NodeI, parent_i: NodeI) {
        if let Some(first_hidden_child) = self.sys.nodes[parent_i].first_hidden_child {
            if first_hidden_child == child_i {
                self.sys.nodes[parent_i].first_hidden_child = self.sys.nodes[child_i].next_hidden_sibling;
                self.sys.nodes[child_i].next_hidden_sibling = None;
                return;
            }
            
            let mut prev = first_hidden_child;
            for_each_hidden_child!(self, self.sys.nodes[parent_i], child, {
                if child == child_i {
                    self.sys.nodes[prev].next_hidden_sibling = self.sys.nodes[child].next_hidden_sibling;
                    self.sys.nodes[child].next_hidden_sibling = None;
                    return;
                }
                prev = child;
            });
        }
    }

    fn add_hidden_child(&mut self, new_node_i: NodeI, parent_i: NodeI) {
        match self.sys.nodes[parent_i].first_hidden_child {
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
        self.sys.nodes[parent_i].first_hidden_child = Some(new_node_i);
    }
    
    fn add_hidden_sibling(&mut self, new_node_i: NodeI, old_last_child: NodeI, _parent_i: NodeI) {
        self.sys.nodes[old_last_child].next_hidden_sibling = Some(new_node_i);
    }

    pub(crate) fn node_or_parent_has_ongoing_animation(&self, i: NodeI) -> bool {
        // todo: what about non-position exit animations, like fading away.

        // this works, but only if this function is called in the right pattern.
        // does it mean that some of the offset-inheriting wasn't needed? probably not.
        let parent = self.sys.nodes[i].parent;
        if self.sys.nodes[parent].exit_animation_still_going {
            return true;
        }

        let target = &self.sys.nodes[i].expected_final_rect;
        let current = &self.sys.nodes[i].real_rect;
        let tolerance = 0.0005;
        
        let is_at_target = (current.x[0] - target.x[0]).abs() < tolerance
            && (current.x[1] - target.x[1]).abs() < tolerance
            && (current.y[0] - target.y[0]).abs() < tolerance
            && (current.y[1] - target.y[1]).abs() < tolerance;

        return !is_at_target;
    }

    pub(crate) fn update_text_boxes(&mut self, i: NodeI) {
        if !self.sys.nodes[i].params.visible {
            return;
        }

        let Some(text_i) = &self.sys.nodes[i].text_i else {
            return;
        };

        let node_clip_rect = self.sys.nodes[i].clip_rect;

        // Update text position using animated rect
        let animated_rect = self.sys.nodes[i].get_animated_rect();
        let padding = self.sys.nodes[i].params.layout.padding;
        let left = (animated_rect[X][0] * self.sys.size[X]) as f64 + padding[X] as f64;

        // Calculate node height in pixels
        let node_height = (animated_rect[Y][1] - animated_rect[Y][0]) * self.sys.size[Y];
        let available_height = node_height - (2.0 * padding[Y] as f32);

        // Round to screen pixels using the transform scale
        let scale = self.sys.nodes[i].accumulated_transform.scale as f64;

        match text_i {
            TextI::TextBox(text_box_handle) => {
                let text_box = self.sys.renderer.text.get_text_box_mut(&text_box_handle);
                let layout = text_box.layout();
                let text_height = layout.height() as f32;

                // Center vertically if text is smaller than available height
                let vertical_offset = if text_height < available_height {
                    (available_height - text_height) / 2.0
                } else {
                    0.0
                };

                let top = (animated_rect[Y][0] * self.sys.size[Y]) as f64 + padding[Y] as f64 + vertical_offset as f64;

                text_box.set_pos(((left * scale).round() / scale, (top * scale).round() / scale));

                // Set hitbox to cover the whole node (in local space relative to text position)
                let node_width = (animated_rect[X][1] - animated_rect[X][0]) * self.sys.size[X];
                let hitbox = (
                    -padding[X],                                    // min_x
                    -padding[Y] - vertical_offset,                  // min_y
                    node_width - padding[X],                        // max_x
                    node_height - padding[Y] - vertical_offset,     // max_y
                );
                text_box.set_hitbox(Some(hitbox));

                // Set the screen-space clip rect
                let clip = BoundingBox {
                    x0: (node_clip_rect.x[0] * self.sys.size[X]) as f64,
                    y0: (node_clip_rect.y[0] * self.sys.size[Y]) as f64,
                    x1: (node_clip_rect.x[1] * self.sys.size[X]) as f64,
                    y1: (node_clip_rect.y[1] * self.sys.size[Y]) as f64,
                };
                self.sys.renderer.text.get_text_box_mut(&text_box_handle).set_clip_rect(Some(clip));
            },
            TextI::TextEdit(text_edit_handle) => {
                let text_edit = self.sys.renderer.text.get_text_edit_mut(&text_edit_handle);
                let (_width, text_edit_height) = text_edit.size();

                // Center vertically based on the text edit widget size
                let vertical_offset = if text_edit_height < available_height {
                    (available_height - text_edit_height) / 2.0
                } else {
                    0.0
                };

                let top = (animated_rect[Y][0] * self.sys.size[Y]) as f64 + padding[Y] as f64 + vertical_offset as f64;

                text_edit.set_pos(((left * scale).round() / scale, (top * scale).round() / scale));

                // Set hitbox to cover the whole node (in local space relative to text position)
                let node_width = (animated_rect[X][1] - animated_rect[X][0]) * self.sys.size[X];
                let hitbox = (
                    -padding[X],                                    // min_x
                    -padding[Y] - vertical_offset,                  // min_y
                    node_width - padding[X],                        // max_x
                    node_height - padding[Y] - vertical_offset,     // max_y
                );
                text_edit.set_hitbox(Some(hitbox));
            },
        }
    }

    pub(crate) fn push_render_and_click_data(&mut self, i: NodeI) {
        let debug = cfg!(debug_assertions);
        let is_scrollable = self.sys.nodes[i].params.is_scrollable();
        let push_click_rect = if debug && self.inspect_mode() {
            true
        } else {
            let opaque = self.sys.nodes[i].params.interact.absorbs_mouse_events;
            let has_senses = self.sys.nodes[i].params.interact.senses != Sense::NONE;
            let editable = if let Some(text_i) = &self.sys.nodes[i].text_i {
                match text_i {
                    TextI::TextEdit(_) => true,
                    TextI::TextBox(_) => false,
                }
            } else { false };
            opaque || editable || has_senses || is_scrollable
        };

        if push_click_rect {
            let click_rect = self.click_rect(i);
            self.sys.click_rects.push(click_rect);
        }

        // Get the clip rect for this node
        // todo: only insert a new one it if it's non-zero

        let node_clip_rect = self.sys.nodes[i].clip_rect;
        let screen_size = self.sys.size;
        let x_clip = [node_clip_rect.x[0] * screen_size.x, node_clip_rect.x[1] * screen_size.x];
        let y_clip = [node_clip_rect.y[0] * screen_size.y, node_clip_rect.y[1] * screen_size.y,];
        let clip_rect = keru_draw::ClipRect { x_clip, y_clip };

        let clip_rect_handle = match self.sys.nodes[i].clip_rect_handle {
            Some(h) => {
                self.sys.renderer.update_clip_rect(h, clip_rect);
                h
            }
            None => {
                let h = self.sys.renderer.insert_clip_rect(clip_rect);
                self.sys.nodes[i].clip_rect_handle = Some(h);
                h
            }
        };
        self.sys.renderer.set_current_clip_rect(clip_rect_handle);

        // Apply accumulated_transform for regular shapes
        if self.sys.nodes[i].accumulated_transform != Transform::IDENTITY {
            let accumulated = &self.sys.nodes[i].accumulated_transform;
            let transform = keru_draw::Transform {
                offset: [accumulated.offset.x, accumulated.offset.y],
                scale: accumulated.scale,
                _padding: 0.0,
            };
            let handle = match self.sys.nodes[i].accumulated_transform_handle {
                Some(h) => {
                    self.sys.renderer.update_transform(h, transform);
                    h
                }
                None => {
                    let h = self.sys.renderer.insert_transform(transform);
                    self.sys.nodes[i].accumulated_transform_handle = Some(h);
                    h
                }
            };
            self.sys.renderer.set_current_transform(handle);
        }

        let texture = self.sys.nodes[i].imageref.as_ref().map(|imageref| {
            match imageref {
                ImageRef::Raster(loaded) => loaded.clone(),
                ImageRef::Svg(loaded) => loaded.clone(),
            }
        });

        if self.sys.inspect_mode {
            self.render_node_shape_to_scene(i, texture, true);
        }

        if self.sys.nodes[i].params.visible {
            self.render_node_shape_to_scene(i, texture, false);

            if let Some(text_i) = &self.sys.nodes[i].text_i {
                match text_i {
                    TextI::TextBox(text_box_handle) => {
                        self.sys.renderer.draw_text_box(&text_box_handle);
                    },
                    TextI::TextEdit(text_edit_handle) => {
                        self.sys.renderer.draw_text_edit(&text_edit_handle);
                    },
                }
            }
        }

        // Clear current transform for regular shapes
        self.sys.renderer.clear_current_clip_rect();
        if self.sys.nodes[i].accumulated_transform != Transform::IDENTITY {
            self.sys.renderer.clear_current_transform();
        }

        // Draw canvas with combined transform (accumulated + canvas offset * scale)
        if let Some(canvas_instances) = self.sys.nodes[i].canvas_instances 
        && let Some((canvas_transform, canvas_clip_rect)) = self.sys.nodes[i].canvas_transform_and_clip {
            let accumulated = &self.sys.nodes[i].accumulated_transform;
            let rect = &self.sys.nodes[i].real_rect;
            let size = self.sys.size;

            // Canvas offset needs to be scaled by accumulated scale
            let canvas_offset_x = rect[X][0] * size.x * accumulated.scale;
            let canvas_offset_y = rect[Y][0] * size.y * accumulated.scale;

            let combined = keru_draw::Transform {
                offset: [
                    accumulated.offset.x + canvas_offset_x,
                    accumulated.offset.y + canvas_offset_y,
                ],
                scale: accumulated.scale,
                _padding: 0.0,
            };

            self.sys.renderer.update_transform(canvas_transform, combined);
            self.sys.renderer.update_clip_rect(canvas_clip_rect, clip_rect);
            self.sys.renderer.draw_deferred_elements(canvas_instances);
        }
    }


    fn set_relayout_chain_root(&mut self, new_node_i: NodeI, parent_i: NodeI) {
        let is_fit_content = self.sys.nodes[new_node_i].params.is_fit_content();
        match self.sys.nodes[parent_i].relayout_chain_root {
            Some(root_of_parent) => match is_fit_content {
                true => self.sys.nodes[new_node_i].relayout_chain_root = Some(root_of_parent), // continue chain
                false => self.sys.nodes[new_node_i].relayout_chain_root = None, // break chain
            },
            None => match is_fit_content {
                true => self.sys.nodes[new_node_i].relayout_chain_root = Some(new_node_i), // start chain
                false => self.sys.nodes[new_node_i].relayout_chain_root = None, // do nothing
            },
        };
    }

    pub(crate) fn push_partial_relayout(&mut self, _i: NodeI) {
        self.sys.changes.full_relayout = true;
        // let relayout_chain_root = match self.sys.nodes[i].relayout_chain_root {
        //     Some(root) => root,
        //     None => i,
        // };

        // // even after the chain, we still have to go one layer up, because a different sized child probably means that the parent wants to place the node differently, and maybe pick a different size and position for the other children as well
        // // In practice, the first half of that is basically always true, but the second half is only true for Stacks. I don't really feel like adding a distinction for that right now.
        // let relayout_target = self.sys.nodes[relayout_chain_root].parent;

        // // try skipping some duplicates
        // if self.sys.changes.partial_relayouts.last().map(|x| x.i) == Some(relayout_target) {
        //     return;
        // }

        // let relayout_entry = NodeWithDepth {
        //     i: relayout_target,
        //     depth: self.sys.nodes[relayout_target].depth,
        // };
        // self.sys.changes.partial_relayouts.push(relayout_entry);
    }

    /// Clear the old Ui tree and start declaring another one.
    /// 
    /// Use together with [`Ui::finish_frame()`], at most once per frame.
    /// 
    /// ```no_run
    /// # use keru::*;
    /// # let mut ui: Ui = unimplemented!();
    /// ui.begin_frame();
    /// // declare the GUI and update state: ui.add(...)
    /// ui.finish_frame();
    /// ```
    pub fn begin_frame(&mut self) {
        self.reset_root();

        self.sys.current_frame += 1;
        self.sys.last_linked_text_box_node = None;
        self.sys.renderer.text.advance_frame_and_hide_boxes();
        self.sys.renderer.clear_for_new_frame();

        thread_local::clear_parent_stack();
        self.sys.changes.unfinished_animations = false;

        thread_local::push_parent(ROOT_I, SiblingCursor::None, self.sys.unique_id);

        self.begin_frame_resolve_inputs();
    }
    
    fn reset_root(&mut self) {
        self.sys.nodes[ROOT_I].last_child = None;
        self.sys.nodes[ROOT_I].first_child = None;
        self.sys.nodes[ROOT_I].prev_sibling = None;
        self.sys.nodes[ROOT_I].next_sibling = None;
        self.sys.nodes[ROOT_I].n_children = 0;
    }

    /// Finish declaring the current GUI tree.
    ///
    /// This function will also relayout the nodes that need it, and do some bookkeeping.
    ///
    /// This function must be called once per frame, after calling [`Ui::begin_frame()`] and running your ui declaration code.
    pub fn finish_frame(&mut self) {
        log::trace!("Finished Ui update");
        // pop the root node
        thread_local::pop_parent(self.sys.unique_id);

        self.cleanup_and_stuff();

        self.relayout();

        self.sys.third_last_frame_end_fake_time = self.sys.second_last_frame_end_fake_time;
        self.sys.second_last_frame_end_fake_time = self.sys.last_frame_end_fake_time;
        self.sys.last_frame_end_fake_time = observer_timestamp();

        if self.sys.update_frames_needed > 0 {
            self.sys.update_frames_needed -= 1;
        }

        self.sys.new_external_events = false;
        self.sys.changes.resize = false;

        // not sure if still needed
        self.sys.needs_update.store(false, std::sync::atomic::Ordering::Relaxed);

        self.sys.mouse_input.finish_frame();
        reset_arena();
        
        self.arena_for_wrapper_structs.reset();
    }

    /// Returns `true` if a node corresponding to `key` exists and if it is currently part of the GUI tree. 
    pub fn is_in_tree(&self, key: NodeKey) -> bool {
        let node_i = self.sys.nodes.node_hashmap.get(&key.id_with_subtree());
        if let Some(entry) = node_i {
            // todo: also return true if it's retained
            return self.sys.nodes[entry.slab_i].last_frame_touched == self.sys.current_frame;
        } else {
            return false;
        }
    }

    fn cleanup_and_stuff(&mut self) {
        with_arena(|a| {
            let mut non_fresh_nodes = bumpalo::collections::Vec::with_capacity_in(20, a);
            let mut to_cleanup = bumpalo::collections::Vec::with_capacity_in(20, a);
            let mut hidden_branch_parents = bumpalo::collections::Vec::with_capacity_in(20, a);
            let mut exiting_nodes = bumpalo::collections::Vec::with_capacity_in(20, a);

            for (i, _) in self.sys.nodes.nodes.iter().skip(2) {
                let i = NodeI::from(i);
                let freshly_added = self.sys.nodes[i].last_frame_touched == self.sys.current_frame;

                if !freshly_added {
                    non_fresh_nodes.push(i);
                }
            }

            // Start exit animations for all nodes that need them
            for &i in &non_fresh_nodes {
                let old_parent = self.sys.nodes[i].parent;
                let old_parent_still_exists = self.sys.nodes.get(old_parent).is_some();

                if old_parent_still_exists {
                    self.init_exit_animations(i);
                }
            }

            // the top-level nodes in hidden branches need to be attached to their children_can_hide parents as hidden nodes, so that when that parent node is removed, we can also remove the hidden branch. Otherwise we'd just forget about them and leave them in memory forever.
            for &i in &non_fresh_nodes {
                let can_hide = self.sys.nodes[i].can_hide;
                let currently_hidden = self.sys.nodes[i].currently_hidden;
                let old_parent = self.sys.nodes[i].parent;
                let old_parent_still_exists = self.sys.nodes.get(old_parent).is_some();

                let is_first_child_in_hidden_branch = match self.sys.nodes.get(old_parent) {
                    Some(old_parent) => old_parent.params.children_can_hide == ChildrenCanHide::Yes,
                    None => false,
                };
                let children_can_hide = self.sys.nodes[i].params.children_can_hide == ChildrenCanHide::Yes;

                if old_parent_still_exists && self.sys.nodes[i].exiting && self.sys.nodes[i].exit_animation_still_going {

                    exiting_nodes.push(NodeWithDepth { i, depth: self.sys.nodes[i].depth });

                } else if ! can_hide {
                    to_cleanup.push(i);
                    if old_parent_still_exists {
                        self.push_partial_relayout(old_parent);
                    }

                    if children_can_hide {
                        hidden_branch_parents.push(i);
                    }

                } else if ! currently_hidden {

                    self.sys.nodes[i].currently_hidden = true;

                    if is_first_child_in_hidden_branch {
                        self.add_hidden_child(i, old_parent);
                        if old_parent_still_exists {
                            self.push_partial_relayout(old_parent);
                        }
                    }
                }

            }

            // Add lingering nodes back into the tree.
            // todo: don't just add them at the end, try to put them after their old prev_sibling. 
            exiting_nodes.sort_by_key(|n| n.depth);
            for &NodeWithDepth { i, .. } in &exiting_nodes {
                let old_parent = self.sys.nodes[i].parent;
                self.set_tree_links(i, old_parent, self.sys.nodes[i].depth, SiblingCursor::None);
                self.refresh_node(i);
                self.sys.nodes[i].exiting = true;
                // todo not in this retarded way
                self.sys.nodes[old_parent].n_children -= 1;
            }

            // This is delayed so that hidden children are all added
            for &i in &hidden_branch_parents {
                for_each_hidden_child!(self, self.sys.nodes[i], hidden_child, {
                    self.add_branch_to_cleanup(hidden_child, &mut to_cleanup);
                });
            }

            // finally cleanup
            for &k in &to_cleanup {
                self.cleanup_node(k);
            }
        });
    }

    fn add_branch_to_cleanup(&mut self, i: NodeI, vec: &mut bumpalo::collections::Vec<'_, NodeI>) {
        vec.push(i);
        for_each_child!(self, self.sys.nodes[i], child, {
            self.add_branch_to_cleanup(child, vec);
        });
    }

    fn cleanup_node(&mut self, i: NodeI) {
        if ! self.sys.nodes.nodes.contains(i.as_usize()) {
            log::error!("Keru: Internal error: tried to cleanup the same node twice. ({:?})", i);
            // we could cheat and just return. instead we continue, so we can see the panic clearly in case there's any bugs.
        }
        let id = self.sys.nodes[i].id;
        
        // skip the nodes that have last_frame_touched = now, because that means that they were not really removed, but just moved somewhere else in the tree.
        // Kind of weird to do this so late.
        // todo: with the new system we can delete this.
        if self.sys.nodes[i].last_frame_touched == self.sys.current_frame {
            log::trace!("Not removing: {}, as it was moved around and not removed", self.node_debug_name(i));
            return;
        }

        let old_handle = self.sys.nodes[i].text_i.take();
        if let Some(text_i) = old_handle {
            match text_i {
                TextI::TextBox(handle) => {
                    self.sys.renderer.text.remove_text_box(handle);
                }
                TextI::TextEdit(handle) => {
                    self.sys.renderer.text.remove_text_edit(handle);
                }
            }
        }

        // Clean up retained transforms and clip rects
        if let Some(handle) = self.sys.nodes[i].accumulated_transform_handle {
            self.sys.renderer.remove_transform(handle);
            self.sys.nodes[i].accumulated_transform_handle = None;
        }
        if let Some(handle) = self.sys.nodes[i].clip_rect_handle {
            self.sys.renderer.remove_clip_rect(handle);
            self.sys.nodes[i].clip_rect_handle = None;
        }

        if let Some((canvas_transform, canvas_clip_rect)) = self.sys.nodes[i].canvas_transform_and_clip {
            self.sys.renderer.remove_transform(canvas_transform);
            self.sys.renderer.remove_clip_rect(canvas_clip_rect);
            self.sys.nodes[i].canvas_transform_and_clip = None;
        }

        self.sys.nodes.node_hashmap.remove(&id);
        self.sys.nodes.nodes.remove(i.as_usize());
    }

    pub(crate) fn current_tree_hash(&mut self) -> u64 {
        let (parent, sibling_cursor, _depth) = thread_local::current_parent(self.sys.unique_id);

        let current_last_child = match sibling_cursor {
            SiblingCursor::None => self.sys.nodes[parent].last_child,
            SiblingCursor::AtStart => None,
            SiblingCursor::After(node) => Some(node),
        };

        let mut hasher = ahasher();

        parent.hash(&mut hasher);
        current_last_child.hash(&mut hasher);

        return hasher.finish()
    }

    pub fn debug_print_tree(&self) {
        let mut prefix = String::new();
        self.debug_print_node_recursive(ROOT_I, &mut prefix, true, false);
    }

    fn debug_print_node_recursive(&self, node_i: NodeI, prefix: &mut String, is_last: bool, is_hidden: bool) {
        let hidden_marker = if is_hidden { " [HIDDEN]" } else { "" };
        let currently_hidden = if self.sys.nodes[node_i].currently_hidden { " (currently_hidden=true)" } else { "" };
        let exiting = if self.sys.nodes[node_i].exiting { " (exiting=true)" } else { "" };
        
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
            self.sys.nodes[node_i].debug_name(),
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
        for_each_child_including_lingering!(self, self.sys.nodes[node_i], _child, {
            regular_count += 1;
        });
        for_each_hidden_child!(self, self.sys.nodes[node_i], _hidden_child, {
            hidden_count += 1;
        });
        let total_count = regular_count + hidden_count;

        // Traverse regular children
        let mut current_index = 0;
        for_each_child_including_lingering!(self, self.sys.nodes[node_i], child, {
            let is_child_last = current_index == total_count - 1;
            self.debug_print_node_recursive(child, prefix, is_child_last, false);
            current_index += 1;
        });

        // Traverse hidden children
        for_each_hidden_child!(self, self.sys.nodes[node_i], hidden_child, {
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
#[derive(Clone, Copy, Debug)]
pub struct UiParent {
    // todo: add a debug-mode frame number to check that it's not held and reused across frames
    pub(crate) i: NodeI,
    pub(crate) sibling_cursor: SiblingCursor,
    pub(crate) ui_instance_id: u32,
}
impl UiParent {
    /// Start a nested block in the GUI tree.
    /// 
    /// Inside the nested block, new nodes will be added as a child of the node that `self` refers to.
    /// 
    /// ```no_run
    /// # use keru::*;
    /// # let mut ui: Ui = unimplemented!();
    /// # let parent = V_STACK;
    /// # let child = BUTTON;
    /// #
    /// //           ↓ returns a `UiParent`
    /// ui.add(parent).nest(|| {
    ///     ui.add(child);
    /// });
    /// ```
    /// 
    /// Since the `content` closure doesn't borrow or move anything, it sets no restrictions on what code can be ran inside it.
    /// You can keep accessing and mutating both the `Ui` and the rest of the program state freely, as you would outside of the closure. 
    pub fn nest<T>(&self, content: impl FnOnce() -> T ) -> T {
        thread_local::push_parent(self.i, self.sibling_cursor, self.ui_instance_id);

        let result = content();

        thread_local::pop_parent(self.ui_instance_id);
    
        return result;
    }
}

#[allow(dead_code)]
#[track_caller]
pub(crate) fn with_timer<T>(operation_name: &str, if_more_than: Option<std::time::Duration>, f: impl FnOnce() -> T) -> T {
        let start = std::time::Instant::now();
        let result = f();
        let elapsed = start.elapsed();

        if let Some(if_more_than) = if_more_than {
            if elapsed > if_more_than {
                println!("{}: {:?}", operation_name, elapsed);
            }
        } else {
            println!("{}: {:?}", operation_name, elapsed);
        }

        result
}
