use crate::*;
use crate::node_library::*;
use std::hash::Hasher;
use std::panic::Location;
use bytemuck::{Pod, Zeroable};
use bumpalo::collections::Vec as BumpVec;

/// An `u64` identifier for a GUI node.
/// 
/// Usually this is only used as part of [`NodeKey`] structs, which are created with the [`node_key`] macro or with [`NodeKey::sibling()`].
#[doc(hidden)]
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, Pod, Zeroable)]
#[repr(C)]
pub struct Id(pub u64);

pub(crate) const FIRST_FRAME: u64 = 0;

pub(crate) const Z_START: f32 = 0.5;
pub const Z_STEP: f32 = -0.000_030_517_578;

pub(crate) const SCROLL_HANDLE_Y: NodeKey = NodeKey::new(Id(834694356), "[internal] Scroll Handle Y");
pub(crate) const SCROLL_RAIL_Y: NodeKey = NodeKey::new(Id(834694357), "[internal] Scroll Rail Y");
pub(crate) const SCROLL_HANDLE_X: NodeKey = NodeKey::new(Id(834694358), "[internal] Scroll Handle X");
pub(crate) const SCROLL_RAIL_X: NodeKey = NodeKey::new(Id(834694359), "[internal] Scroll Rail X");

impl Ui {
    /// Add a [`Node`] to the `Ui`.
    /// 
    /// ```no_run
    /// # use keru::*; use keru::node_library::*; let mut ui: Ui = unimplemented!();
    /// let red_label = LABEL
    ///     .color(Color::RED)
    ///     .text("Increase");
    /// 
    /// ui.add(red_label);
    /// ```
    /// 
    /// Buttons, images, text elements, stack containers, etc. are all created by `add`ing a [`Node`] with the right fields.
    #[track_caller]
    pub fn add<'a>(&mut self, node: Node<'a>) -> UiParent
    {
        let key = node.key_or_anon_key();
        let (i, _id) = self.add_or_update_node(key);
        self.set_params(i, &node);
        self.set_params_text(i, &node);

        if node.layout.scrollable.y {
            self.add_scrollbar(i, key, Y);
        }
        if node.layout.scrollable.x {
            self.add_scrollbar(i, key, X);
        }

        return UiParent { i, sibling_cursor: SiblingCursor::None, ui_instance_id: self.sys.unique_id };
    }

    /// Returns an [`UiParent`] for the root node, that you can use to nest children directly into the root node, regardless of where you are in the `Ui` tree structure.
    /// 
    /// This is sort of a crazy thing to do, but here's an example of why it might be useful:
    /// 
    /// ```no_run
    /// # use keru::*; use keru::node_library::*; let mut ui: Ui = unimplemented!();
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
    pub fn jump_to_node(&self, key: NodeKey) -> Option<UiParent> {
        let parent_i = self.sys.nodes.get_with_key_scope(key)?;
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
    /// # use keru::*; use keru::node_library::*; let mut ui: Ui = unimplemented!();
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
    ///         ui.add(BUTTON.text("X").color(Color::RED));
    ///     });
    /// });
    /// ```
    pub fn jump_to_sibling(&self, jump_key: NodeKey) -> Option<UiParent> {
        let sibling_i = self.sys.nodes.get_with_key_scope(jump_key)?;
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
    /// # use keru::*; use keru::node_library::*; let mut ui: Ui = unimplemented!();
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
    ///         ui.add(BUTTON.text("X").color(Color::RED));
    ///     });
    /// });
    /// ```
    pub fn jump_to_before_sibling(&self, jump_key: NodeKey) -> Option<UiParent> {
        let sibling_i = self.sys.nodes.get_with_key_scope(jump_key)?;
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
    /// # use keru::*; use keru::node_library::*; let mut ui: Ui = unimplemented!();
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
    ///         ui.add(BUTTON.text("X").color(Color::RED));
    ///     });
    /// });
    /// ```
    pub fn jump_to_nth_child(&self, parent_key: NodeKey, n: usize) -> Option<UiParent> {
        let parent_i = self.sys.nodes.get_with_key_scope(parent_key)?;

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

    // this function also detects new nodes and reorderings, and pushes partial relayouts for them. For deleted nodes, partial relayouts will be pushed in cleanup_and_stuff.
    pub(crate) fn set_tree_links(&mut self, new_node_i: NodeI, parent_i: NodeI, depth: usize, sibling_cursor: SiblingCursor) {
        self.clear_node_children(new_node_i);
        self.sys.link_node_to_parent(new_node_i, parent_i, depth, sibling_cursor);
    }

    fn clear_node_children(&mut self, new_node_i: NodeI) {
        // Reset old links
        self.sys.nodes[new_node_i].old_first_child = self.sys.nodes[new_node_i].first_child;
        self.sys.nodes[new_node_i].old_next_sibling = self.sys.nodes[new_node_i].next_sibling;

        self.sys.nodes[new_node_i].first_child = None;
        self.sys.nodes[new_node_i].last_child = None;
        self.sys.nodes[new_node_i].n_children = 0;
    }

    fn add_hidden_child(&mut self, new_node_i: NodeI, parent_i: NodeI) {
        match self.sys.nodes[parent_i].first_hidden_child {
            None => {
                // add hidden first child
                self.sys.nodes[parent_i].first_hidden_child = Some(new_node_i);
            },
            Some(last_hidden_child) => {
                // add hidden sibling
                let old_last_hidden_child = last_hidden_child;
                self.sys.nodes[old_last_hidden_child].next_hidden_sibling = Some(new_node_i);

            },
        };
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
        let node_width = (animated_rect[X][1] - animated_rect[X][0]) * self.sys.size[X];

        let available_height = node_height - (2.0 * padding[Y] as f32);
        let available_width = node_width - (2.0 * padding[X] as f32);

        // Round to screen pixels using the transform scale
        let scale = self.sys.nodes[i].accumulated_transform.scale as f64;

        match text_i {
            TextI::TextBox(text_box_handle) => {
                let text_box = self.sys.renderer.text.get_text_box_mut(&text_box_handle);
                let layout = text_box.layout();
                let text_height = layout.height() as f32;

                let vertical_offset = match self.sys.nodes[i].params.vertical_text_alignment {
                    VerticalTextAlignment::Center => if text_height < available_height { (available_height - text_height) / 2.0 } else { 0.0 },
                    VerticalTextAlignment::Top => 0.0,
                    VerticalTextAlignment::Bottom => if text_height < available_height { available_height - text_height } else { 0.0 },
                };

                let top = (animated_rect[Y][0] * self.sys.size[Y]) as f64 + padding[Y] as f64 + vertical_offset as f64;

                text_box.set_pos(((left * scale).round() / scale, (top * scale).round() / scale));

                // Set hitbox to cover the whole node (in local space relative to text position)
                let hitbox = (
                    -padding[X],                                    // min_x
                    -padding[Y] - vertical_offset,                  // min_y
                    node_width - padding[X],                        // max_x
                    node_height - padding[Y] - vertical_offset,     // max_y
                );
                // looks like it needs one pixel of breathing room, or the FitContent text will overflow to two lines
                text_box.set_size((available_width + 1.0, available_height));
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

                let vertical_offset = match self.sys.nodes[i].params.vertical_text_alignment {
                    VerticalTextAlignment::Center => if text_edit_height < available_height { (available_height - text_edit_height) / 2.0 } else { 0.0 },
                    VerticalTextAlignment::Top => {
                        0.0
                    },
                    VerticalTextAlignment::Bottom => if text_edit_height < available_height { available_height - text_edit_height } else { 0.0 },
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

    pub(crate) fn push_render_and_click_data(&mut self, i: NodeI, alpha: f32) {
        if let Some(text_i) = &self.sys.nodes[i].text_i {
            let z = self.sys.nodes[i].z;
            match text_i {
                TextI::TextBox(h) => {
                    let tb = self.sys.renderer.text.get_text_box_mut(h);
                    tb.set_depth(z);
                    tb.set_opacity(alpha);
                }
                TextI::TextEdit(h) => {
                    let te = self.sys.renderer.text.get_text_edit_mut(h);
                    te.set_depth(z);
                    te.set_opacity(alpha);
                }
            }
        }

        let is_scrollable = self.sys.nodes[i].params.is_scrollable();
        let push_click_rect = if self.inspect_mode() {
            true
        } else {
            let opaque = self.sys.nodes[i].params.interact.absorbs_mouse_events;
            let has_senses = self.sys.nodes[i].params.interact.senses != Sense::NONE;
            // Some noninteractable nodes still need click rects so that they can partecipate silently to keyboard focus. That is, they silently receive the focus so that the next Tab or Shift+Tab can focus the node closest to them.
            // It's currently extended just to non-editable text nodes, but maybe it should be all visible nodes?
            let is_text = self.sys.nodes[i].text_i.is_some();
            opaque || is_text || has_senses || is_scrollable
        };

        if push_click_rect {
            let click_rect = self.sys.click_rect(i);
            self.sys.click_rects.push(click_rect);

            if click_rect.senses.contains(Sense::TIME) {
                self.sys.has_any_time_sense_node = true;
            }
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
            self.draw_node_shape(i, texture, true, alpha);
        }

        if self.sys.nodes[i].params.visible {
            self.draw_node_shape(i, texture, false, alpha);

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
        // todo: keep the canvas for longer so we can remove the canvas_recorded_this_frame check and draw canvas stuff for exiting nodes
        let canvas_recorded_this_frame = self.sys.nodes[i].last_frame_touched == self.sys.current_frame;
        if canvas_recorded_this_frame
        && let Some(canvas_instances) = self.sys.nodes[i].canvas_instances
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
                scale: accumulated.scale * self.sys.scale_factor,
                _padding: 0.0,
            };

            self.sys.renderer.update_transform(canvas_transform, combined);
            self.sys.renderer.update_clip_rect(canvas_clip_rect, clip_rect);
            self.sys.renderer.draw_deferred_elements(canvas_instances);
        }
    }


    /// Clear the old Ui tree and start declaring another one.
    /// 
    /// Use together with [`Ui::finish_frame()`], at most once per frame.
    /// 
    /// ```no_run
    /// # use keru::*; use keru::node_library::*; let mut ui: Ui = unimplemented!();
    /// ui.begin_frame();
    /// // declare the GUI and update state: ui.add(...)
    /// ui.finish_frame();
    /// ```
    pub fn begin_frame(&mut self) {
        // reset root
        self.sys.nodes[ROOT_I].last_child = None;
        self.sys.nodes[ROOT_I].first_child = None;
        self.sys.nodes[ROOT_I].prev_sibling = None;
        self.sys.nodes[ROOT_I].next_sibling = None;
        self.sys.nodes[ROOT_I].n_children = 0;

        self.begin_frame_inner();
    }

    /// Start a "retained mode" frame.
    ///
    /// Unlike the normal [`Ui::begin_frame()`], this does not clear the tree, so you can do just the minimal imperative modifications to the tree instead of redeclaring a new one from scratch.
    /// 
    /// Call [`Ui::begin_retained_mode_frame()`] to finish the retained-mode frame.
    pub fn begin_retained_mode_frame(&mut self) {
        self.begin_frame_inner();
        // If we were more serious about retained mode, this could be done in slightly more efficient ways, probably.
        self.readd_branch_recursive(ROOT_I);
    }

    pub fn begin_frame_inner(&mut self) {
        self.sys.current_frame += 1;
        self.sys.last_linked_text_box_node = None;
        self.sys.renderer.clear_for_new_frame();

        thread_local::clear_parent_stack();
        self.sys.changes.unfinished_animations = false;

        thread_local::push_parent(ROOT_I, SiblingCursor::None, self.sys.unique_id);

        self.begin_frame_resolve_inputs();
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

        self.finish_frame_inner();
    }

    /// Finish a "retained mode" frame started with [`Ui::begin_retained_mode_frame()`].
    pub fn finish_retained_mode_frame(&mut self) {
        self.finish_frame_inner();
    }

    fn finish_frame_inner(&mut self) {
        self.cleanup_and_stuff();

        let update_accesskit_focus = self.sys.changes.focus_changed;
        let update_accesskit_tree = self.sys.changes.full_relayout
            || ! self.sys.changes.partial_relayouts.is_empty()
            || self.sys.changes.text_changed;
        // todo: we could also check accessibility roles, actions etc. on every node update.
        // But I think it's highly unlikely that they'd change in a frame that doesn't also change text or layout.

        self.relayout();

        self.sys.last_frame_end_fake_time = get_observer_timestamp();

        if self.sys.has_any_time_sense_node {
            self.sys.update_frames_needed = 2;
        } else if self.sys.update_frames_needed > 0 {
            self.sys.update_frames_needed -= 1;
        }
        self.sys.has_any_time_sense_node = false;

        self.sys.new_external_events = false;
        self.sys.changes.resize = false;

        // not sure if still needed
        self.sys.needs_update.store(false, std::sync::atomic::Ordering::Relaxed);

        self.sys.mouse_input.finish_frame();
        self.sys.accesskit_actions.clear();

        if update_accesskit_tree {
            // Partial borrows moment
            if let Some(mut accesskit) = self.sys.accesskit.take() {
                accesskit.update_if_active(|| self.build_accesskit_tree());
                self.sys.accesskit = Some(accesskit);
            }
        } else if update_accesskit_focus {
            self.update_accesskit_focus_if_active();
        }
        self.sys.changes.focus_changed = false;

        reset_arena();

        self.arena_for_wrapper_structs.reset();
    }

    /// Returns `true` if a node corresponding to `key` exists and if it is currently part of the GUI tree. 
    pub fn is_in_tree(&self, key: NodeKey) -> bool {
        if let Some(i) = self.sys.nodes.get_with_key_scope(key) {
            // todo: also return true if it's retained
            return self.sys.nodes[i].last_frame_touched == self.sys.current_frame;
        } else {
            return false;
        }
    }

    fn cleanup_and_stuff(&mut self) {
        with_arena(|a| {
            let mut non_fresh_nodes = BumpVec::with_capacity_in(20, a);
            let mut to_cleanup = BumpVec::with_capacity_in(20, a);
            let mut hidden_branch_parents = BumpVec::with_capacity_in(20, a);
            let mut exiting_nodes = BumpVec::with_capacity_in(20, a);

            for i in self.sys.nodes.iter() {
                let freshly_added = self.sys.nodes[i].last_frame_touched == self.sys.current_frame;

                if !freshly_added {
                    non_fresh_nodes.push(i);
                }
            }

            // Start exit animations for all nodes that need them
            for &i in &non_fresh_nodes {
                let old_parent = self.sys.nodes[i].parent;
                let old_parent_still_exists = self.sys.nodes.get_node_if_it_still_exists(old_parent).is_some();

                if old_parent_still_exists {
                    self.init_exit_animations(i);
                }
            }

            // the top-level nodes in hidden branches need to be attached to their children_can_hide parents as hidden nodes, so that when that parent node is removed, we can also remove the whole hidden branch. Otherwise we'd just forget about them and leave them in memory forever.
            for &i in &non_fresh_nodes {
                let can_hide = self.sys.nodes[i].can_hide;
                let currently_hidden = self.sys.nodes[i].currently_hidden;
                let old_parent_i = self.sys.nodes[i].parent;
                let old_parent_node = self.sys.nodes.get_node_if_it_still_exists(old_parent_i);
                let old_parent_still_exists = old_parent_node.is_some();
                let is_first_child_in_hidden_branch = old_parent_node.map_or(false, |p| p.params.children_can_hide == ChildrenCanHide::Yes);
                let parent_is_exiting = old_parent_node.map_or(false, |p| p.exit_animation_still_going);
                let children_can_hide = self.sys.nodes[i].params.children_can_hide == ChildrenCanHide::Yes;
                let has_exit_anim = !matches!(self.sys.nodes[i].params.animation.exit, ExitAnimation::None);

                if old_parent_still_exists && self.sys.nodes[i].exiting && self.sys.nodes[i].exit_animation_still_going && (has_exit_anim || parent_is_exiting) {

                    exiting_nodes.push(NodeWithDepth { i, depth: self.sys.nodes[i].depth });

                } else if ! can_hide {

                    to_cleanup.push(i);
                    if old_parent_still_exists {
                        self.sys.push_partial_relayout(old_parent_i);
                    }

                    if children_can_hide {
                        hidden_branch_parents.push(i);
                    }

                } else if ! currently_hidden {

                    self.sys.nodes[i].currently_hidden = true;
                    self.sys.set_text_hidden(i, true);

                    if is_first_child_in_hidden_branch {
                        self.add_hidden_child(i, old_parent_i);
                        if old_parent_still_exists {
                            self.sys.push_partial_relayout(old_parent_i);
                        }
                    }
                }

            }

            // Add lingering/exiting nodes back into the tree.
            // todo: don't just add them at the end, try to put them after their old prev_sibling.
            // (it only matters for z order, exiting nodes don't partecipate in layout)
            exiting_nodes.sort_by_key(|n| n.depth);
            for &NodeWithDepth { i, .. } in &exiting_nodes {
                let old_parent = self.sys.nodes[i].parent;
                self.set_tree_links(i, old_parent, self.sys.nodes[i].depth, SiblingCursor::None);
                self.sys.nodes[i].exiting = true;
                // we're reusing set_tree_links which also increases the parent's child count, but exiting nodes shouldn't be counted.
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

    fn add_branch_to_cleanup(&mut self, i: NodeI, vec: &mut BumpVec<'_, NodeI>) {
        vec.push(i);
        for_each_child!(self, self.sys.nodes[i], child, {
            self.add_branch_to_cleanup(child, vec);
        });
    }

    fn cleanup_node(&mut self, i: NodeI) {
        if self.sys.nodes.get_node_if_it_still_exists(i).is_none() {
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

        if self.sys.nodes[i].has_component_state {
            self.sys.user_state.remove(&id);
        }

        self.sys.nodes.remove(id);
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

    pub(crate) fn add_scrollbar(&mut self, i: NodeI, key: NodeKey, axis: Axis) {
        let (rail_key, handle_key) = scrollbar_keys(key, axis);

        // todo: without the "! released", it gets stuck to the wide size after dragging.
        let wide = self.is_hovered(rail_key) || self.is_hovered(handle_key)
            || (self.is_dragged(rail_key).is_some() && ! self.is_drag_released(rail_key))
            || (self.is_dragged(handle_key).is_some() && ! self.is_drag_released(handle_key));

        let width = if wide { 8.0 } else { 3.0 };
        let rail_width = if wide { 14.0 } else { 9.0 };

        let Some(ScrollbarState { thumb_frac, thumb_lead_frac, scroll_range, max_scroll, container_size, scroll }) = self.sys.scrollbar_state(i, axis) else {
            return;
        };

        let rail_color = if wide { Color::rgba_u8(80, 80, 80, 60) } else { Color::TRANSPARENT };
        let handle_color = if wide { Color::rgba_u8(80, 80, 80, 255) } else { Color::rgba_u8(80, 80, 80, 90) };

        let thumb_anchor_cross = Anchor::Frac((rail_width + width) / (2.0 * width));

        let (rail_size, rail_pos, handle_size, handle_pos, handle_anchor) = match axis {
            Y => (
                (Size::Pixels(rail_width), Size::Fill),
                (Pos::End, Pos::Start),
                (Size::Pixels(width), Size::Frac(thumb_frac)),
                (Pos::Frac(1.0), Pos::Frac(thumb_lead_frac)),
                (thumb_anchor_cross, Anchor::Start),
            ),
            X => (
                (Size::Fill, Size::Pixels(rail_width)),
                (Pos::Start, Pos::End),
                (Size::Frac(thumb_frac), Size::Pixels(width)),
                (Pos::Frac(thumb_lead_frac), Pos::Frac(1.0)),
                (Anchor::Start, thumb_anchor_cross),
            ),
        };

        let scroll_rail_node = PANEL
            .key(rail_key)
            .shape(Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: rail_width / 2.0 })
            .size(rail_size.0, rail_size.1)
            .position_x(rail_pos.0)
            .position_y(rail_pos.1)
            .sense_hover_enter_or_exit(true)
            .sense_click(true)
            .sense_drag(true)
            .focusable(false)
            .z_index(100.0)
            .free_placement(true)
            .ignore_parent_scroll(true)
            .color(rail_color);

        let scroll_handle_node = PANEL
            .key(handle_key)
            .shape(Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: width / 2.0 })
            .size(handle_size.0, handle_size.1)
            .position_x(handle_pos.0)
            .anchor_x(handle_anchor.0)
            .position_y(handle_pos.1)
            .anchor_y(handle_anchor.1)
            .sense_drag(true)
            .sense_hover_enter_or_exit(true)
            .focusable(false)
            .z_index(100.0)
            .animate_position(true)
            .animation_speed(2.5)
            .free_placement(true)
            .ignore_parent_scroll(true)
            .color(handle_color);

        thread_local::push_parent(i, SiblingCursor::None, self.sys.unique_id);

        self.add(scroll_rail_node);
        self.add(scroll_handle_node);

        thread_local::pop_parent(self.sys.unique_id);

        let container_i = i;

        if self.is_dragged(rail_key).is_none() && let Some(drag) = self.is_dragged(handle_key) {
            if scroll_range < 0.0 {
                let track_size = (1.0 - thumb_frac) * container_size;
                let logical_size = self.sys.logical_size();
                let delta_norm = vec2_axis(drag.absolute_delta, axis) / logical_size[axis];
                let scroll_delta = if track_size > 0.0 {
                    delta_norm / track_size * scroll_range
                } else {
                    0.0
                };
                self.sys.update_container_scroll(container_i, scroll_delta, axis, false);
            }
        } else {
            let rail_cursor =
            if let Some(click) = self.clicked_at(rail_key) {
                Some(vec2_axis(click.relative_position, axis))
            } else if let Some(drag) = self.is_dragged(rail_key) {
                Some(vec2_axis(drag.relative_position, axis))
            } else {
                None
            };

            if let Some(cursor) = rail_cursor {
                if scroll_range < 0.0 {
                    let progress = ((cursor - thumb_frac / 2.0) / (1.0 - thumb_frac)).clamp(0.0, 1.0);
                    let target_scroll = max_scroll + progress * scroll_range;
                    self.sys.update_container_scroll(container_i, target_scroll - scroll, axis, false);
                }
            }
        }
    }
}

fn scrollbar_keys(key: NodeKey, axis: Axis) -> (NodeKey, NodeKey) {
    match axis {
        Y => (key.sibling(SCROLL_RAIL_Y), key.sibling(SCROLL_HANDLE_Y)),
        X => (key.sibling(SCROLL_RAIL_X), key.sibling(SCROLL_HANDLE_X)),
    }
}

struct ScrollbarState {
    thumb_frac: f32,
    thumb_lead_frac: f32,
    scroll_range: f32,
    max_scroll: f32,
    container_size: f32,
    scroll: f32,
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
    /// # use keru::*; use keru::node_library::*; let mut ui: Ui = unimplemented!();
    /// # let parent = node_library::V_STACK;
    /// # let child = node_library::BUTTON;
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
                log::info!("{}: {:?}", operation_name, elapsed);
            }
        } else {
            log::info!("{}: {:?}", operation_name, elapsed);
        }

        result
}

impl Ui {
    /// Alternate form of [`Ui::add()`] that returns an [`UiNode`].
    /// 
    /// This way, we can call [`is_clicked`](UiNode::is_clicked()) and all the other [`UiNode`] directly after adding the node, without tricks.
    /// 
    /// However, nesting requires two separate calls to `nest()` and `enter()` instead of just one `nest()`.
    /// 
    /// # Example
    /// ```no_run
    /// # use keru::*; use keru::node_library::*; let mut ui: Ui = unimplemented!();
    /// ui.add2(V_STACK).nest().enter(|| {
    ///     ui.add2(BUTTON.text("Hello"));
    ///     ui.add2(BUTTON.text("World"));
    /// });
    /// ```
    #[track_caller]
    pub fn add2<'a>(&mut self, node: Node<'a>) -> &mut UiNode<'_>
    {
        let key = node.key_or_anon_key();
        let (i, _id) = self.add_or_update_node(key);
        self.set_params(i, &node);
        self.set_params_text(i, &node);

        if node.layout.scrollable.y {
            self.add_scrollbar(i, key, Y);
        }
        if node.layout.scrollable.x {
            self.add_scrollbar(i, key, X);
        }

        return self.get_node_mut(key).unwrap();
    }
}
impl System {
    pub(crate) fn update_scrollbar_handle_params(&mut self, container_i: NodeI) {
        let key = self.nodes[container_i].original_key;

        for axis in [Y, X] {
            let (_, handle_key) = scrollbar_keys(key, axis);
            let Some(handle_i) = self.nodes.get_by_id(handle_key.id_with_key_scope()) else {
                continue;
            };

            let Some(ScrollbarState { thumb_lead_frac, .. }) = self.scrollbar_state(container_i, axis) else {
                continue;
            };

            self.nodes[handle_i].params.layout.position[axis] = Pos::Frac(thumb_lead_frac);
        }
    }


    fn scrollbar_state(&self, i: NodeI, axis: Axis) -> Option<ScrollbarState> {
        let container_rect = self.nodes[i].layout_rect;
        let content_bounds = self.nodes[i].content_bounds;
        let scroll = self.nodes[i].scroll[axis];

        let container_size = container_rect.size()[axis];
        let content_size = content_bounds.size()[axis];

        if content_size <= container_size || container_size <= 0.0 {
            return None;
        }

        let thumb_frac = (container_size / content_size).clamp(0.05, 1.0);

        let min_scroll = if content_bounds[axis][1] > container_rect[axis][1] {
            container_rect[axis][1] - content_bounds[axis][1]
        } else {
            0.0
        };
        let max_scroll = if content_bounds[axis][0] < container_rect[axis][0] {
            container_rect[axis][0] - content_bounds[axis][0]
        } else {
            0.0
        };

        let scroll_range = min_scroll - max_scroll;
        let progress = if scroll_range < 0.0 {
            ((scroll - max_scroll) / scroll_range).clamp(0.0, 1.0)
        } else {
            0.0
        };

        Some(ScrollbarState {
            thumb_frac,
            thumb_lead_frac: progress * (1.0 - thumb_frac),
            scroll_range,
            max_scroll,
            container_size,
            scroll,
        })
    }

    pub(crate) fn clear_children_of_node(&mut self, i: NodeI) {
        self.mark_children_non_fresh(i);
        self.nodes[i].first_child = None;
        self.nodes[i].last_child = None;
        self.nodes[i].n_children = 0;
        self.push_partial_relayout(i);
    }

    fn mark_children_non_fresh(&mut self, i: NodeI) {
        let mut child = self.nodes[i].first_child;
        while let Some(c) = child {
            let next = self.nodes[c].next_sibling;
            self.nodes[c].last_frame_touched = 0;
            self.mark_children_non_fresh(c);
            child = next;
        }
    }

    /// Unlink a node from its current parent's child list.
    pub(crate) fn unlink_from_tree(&mut self, node_i: NodeI) {
        let old_parent = self.nodes[node_i].parent;
        let prev = self.nodes[node_i].prev_sibling;
        let next = self.nodes[node_i].next_sibling;

        match prev {
            Some(prev_i) => self.nodes[prev_i].next_sibling = next,
            None => self.nodes[old_parent].first_child = next,
        }

        match next {
            Some(next_i) => self.nodes[next_i].prev_sibling = prev,
            None => self.nodes[old_parent].last_child = prev,
        }

        // Decrement parent's child count
        debug_assert!(self.nodes[old_parent].n_children > 0);
        self.nodes[old_parent].n_children -= 1;

        // Clear the node's sibling pointers
        self.nodes[node_i].prev_sibling = None;
        self.nodes[node_i].next_sibling = None;
    }

    pub(crate) fn link_node_to_parent(&mut self, new_node_i: NodeI, parent_i: NodeI, _depth: usize, sibling_cursor: SiblingCursor) {
        assert!(new_node_i != parent_i, "Keru: Internal error: tried to add a node as child of itself ({}). This shouldn't be possible.", self.nodes[new_node_i].debug_name());

        // If parent changed, convert local_animated_rect to the new parent's coordinate space using screen-space positions from the previous frame.
        let old_parent = self.nodes[new_node_i].parent;
        let is_new_node = self.nodes[new_node_i].frame_added == self.current_frame;
        if !is_new_node && old_parent != parent_i {
            let screen_pos = self.nodes[new_node_i].real_rect;
            let new_parent_offset = self.nodes[parent_i].real_rect.top_left();
            self.nodes[new_node_i].local_animated_rect = screen_pos - new_parent_offset;
        }

        // Add new child
        self.nodes[new_node_i].parent = parent_i;
        self.nodes[new_node_i].prev_sibling = None;
        self.nodes[new_node_i].next_sibling = None;

        self.nodes[parent_i].n_children += 1;

        match sibling_cursor {
            // Add after last sibling (no jump)
            SiblingCursor::None => {
                match self.nodes[parent_i].last_child {
                    None => {
                        // First child
                        self.nodes[parent_i].first_child = Some(new_node_i);
                        self.nodes[parent_i].last_child = Some(new_node_i);

                        if self.nodes[parent_i].first_child != self.nodes[parent_i].old_first_child {
                            self.push_partial_relayout(parent_i);
                        }
                    },
                    Some(last_child) => {
                        let prev_sibling = last_child;
                        // Append after last_child
                        self.nodes[new_node_i].prev_sibling = Some(prev_sibling);
                        self.nodes[prev_sibling].next_sibling = Some(new_node_i);
                        self.nodes[parent_i].last_child = Some(new_node_i);

                        if self.nodes[prev_sibling].old_next_sibling != self.nodes[prev_sibling].next_sibling {
                            self.push_partial_relayout(parent_i);
                        }
                    },
                }
            },
            // Add at the start (before first child)
            SiblingCursor::AtStart => {
                let old_first = self.nodes[parent_i].first_child;

                self.nodes[new_node_i].next_sibling = old_first;
                self.nodes[parent_i].first_child = Some(new_node_i);

                match old_first {
                    Some(old_first_i) => {
                        self.nodes[old_first_i].prev_sibling = Some(new_node_i);
                    }
                    None => {
                        self.nodes[parent_i].last_child = Some(new_node_i);
                    }
                }

                // Manually advance the thread-local cursor
                thread_local::set_sibling_cursor(SiblingCursor::After(new_node_i));

                self.push_partial_relayout(parent_i);
            },
            // Add after a specific sibling
            SiblingCursor::After(after_i) => {
                let old_next = self.nodes[after_i].next_sibling;

                self.nodes[new_node_i].prev_sibling = Some(after_i);
                self.nodes[new_node_i].next_sibling = old_next;
                self.nodes[after_i].next_sibling = Some(new_node_i);

                match old_next {
                    Some(old_next_i) => {
                        self.nodes[old_next_i].prev_sibling = Some(new_node_i);
                    }
                    None => {
                        self.nodes[parent_i].last_child = Some(new_node_i);
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
        self.changes.full_relayout = true;

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

    fn remove_hidden_child_if_it_exists(&mut self, child_i: NodeI, parent_i: NodeI) {
        if let Some(first_hidden_child) = self.nodes[parent_i].first_hidden_child {
            if first_hidden_child == child_i {
                self.nodes[parent_i].first_hidden_child = self.nodes[child_i].next_hidden_sibling;
                self.nodes[child_i].next_hidden_sibling = None;
                return;
            }

            let mut prev = first_hidden_child;
            let mut current_child = self.nodes[parent_i].first_hidden_child;
            while let Some(child) = current_child {
                if child == child_i {
                    self.nodes[prev].next_hidden_sibling = self.nodes[child].next_hidden_sibling;
                    self.nodes[child].next_hidden_sibling = None;
                    return;
                }
                prev = child;
                current_child = self.nodes[child].next_hidden_sibling;
            }
        }
    }

    pub(crate) fn set_text_hidden(&mut self, i: NodeI, value: bool) {
        if let Some(i) = &self.nodes[i].text_i {
            match i {
                TextI::TextBox(text_box_handle) => {
                    self.renderer.text.get_text_box_mut(text_box_handle).set_hidden(value);
                },
                TextI::TextEdit(text_edit_handle) => {
                    self.renderer.text.get_text_edit_mut(text_edit_handle).set_hidden(value);
                },
            }
        }
    }
}

impl<'a> UiNode<'a> {
    /// Get a [`UiParent`] that can be used to add other nodes as children of this one.
    ///
    /// # Example
    /// ```no_run
    /// # use keru::*; use keru::node_library::*; let mut ui: Ui = unimplemented!();
    /// ui.add2(V_STACK).nest().enter(|| {
    ///     ui.add2(BUTTON.text("Hello"));
    ///     ui.add2(BUTTON.text("World"));
    /// });
    /// ```
    pub fn nest(&self) -> UiParent {
        let i = self.i;
        let ui_instance_id = self.sys().unique_id;
        return UiParent { i, sibling_cursor: SiblingCursor::None, ui_instance_id };
    }

    /// Remove this node from its current position in the tree and re-add it at the current parent context.
    pub fn re_add(&mut self) {
        let node_i = self.i;
        let sys = self.sys_mut();
        sys.unlink_from_tree(node_i);

        let (parent_i, sibling_cursor, depth) = thread_local::current_parent(sys.unique_id);
        sys.link_node_to_parent(node_i, parent_i, depth, sibling_cursor);
    }
}

impl UiParent {
    /// Alias of `nest` used for the alternative [`Ui::add2()`] syntax.
    pub fn enter<T>(&self, content: impl FnOnce() -> T ) -> T {
        self.nest(content)
    }
}
