// todo: move some more stuff out of this file
use crate::changes::NodeWithDepth;
use crate::*;
use crate::node_key::*;
use crate::math::*;
use crate::param_library::*;
use crate::text::*;
use crate::node::*;
use crate::thread_local::*;
use glyphon::cosmic_text::Align;
use glyphon::{AttrsList, Color as GlyphonColor, TextBounds, Viewport};

use rustc_hash::FxHasher;
use thread_local::thread_local_peek_tree_position_hash;

use std::collections::hash_map::Entry;
use std::{
    hash::Hasher,
    time::Instant,
};

use bytemuck::{Pod, Zeroable};
use glyphon::{
    Attrs, Buffer as GlyphonBuffer, Family, FontSystem, Metrics, Shaping, SwashCache,
    TextAtlas, TextRenderer,
};
use winit::dpi::PhysicalSize;
use Axis::{X, Y};

use crate::twin_nodes::RefreshOrClone::*;
use crate::twin_nodes::TwinCheckResult::*;
use crate::twin_nodes::*;
use std::fmt::{Display, Write};


/// An `u64` identifier for a GUI node.
/// 
/// Usually this is only used as part of [`NodeKey`] structs, which are created with the [`node_key`] macro or with [`NodeKey::sibling`].
#[doc(hidden)]
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, Pod, Zeroable)]
#[repr(C)]
pub struct Id(pub u64);

// this is what you get from FxHasher::default().finish()
pub(crate) const EMPTY_HASH: u64 = 0;

pub(crate) const FIRST_FRAME: u64 = 1;

// todo: make this stuff configurable
pub(crate) const Z_BACKDROP: f32 = 0.5;
// This one has to be small, but not small enough to get precision issues.
// And I think it's probably good if it's a rounded binary number (0x38000000)? Not sure.
pub(crate) const Z_STEP: f32 = -0.000030517578125;

// another stupid sub struct for dodging partial borrows
pub(crate) struct TextSystem {
    pub font_system: FontSystem,
    pub cache: SwashCache,
    pub atlas: TextAtlas,
    pub text_renderer: TextRenderer,
    pub text_areas: Vec<FullText>,
    pub glyphon_viewport: Viewport,
}
const GLOBAL_TEXT_METRICS: Metrics = Metrics::new(24.0, 24.0);
impl TextSystem {
    pub(crate) fn maybe_new_text_area(
        &mut self,
        text: Option<&str>,
        current_frame: u64,
    ) -> Option<usize> {
        let text = match text {
            Some(text) => text,
            None => return None,
        };

        let mut buffer = GlyphonBuffer::new(&mut self.font_system, GLOBAL_TEXT_METRICS);
        buffer.set_size(&mut self.font_system, Some(500.), Some(500.));

        let mut hasher = FxHasher::default();
        text.hash(&mut hasher);
        let hash = hasher.finish();

        // todo: maybe remove duplication with set_text_hashed (the branch in refresh_node that updates the text without creating a new entry here)
        // buffer.set_wrap(&mut self.font_system, glyphon::Wrap::Word);
        buffer.set_text(
            &mut self.font_system,
            text,
            Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );

        for line in &mut buffer.lines {
            line.set_align(Some(glyphon::cosmic_text::Align::Center));
        }

        let params = TextAreaParams {
            left: 10.0,
            top: 10.0,
            scale: 1.0,
            bounds: TextBounds {
                left: 0,
                top: 0,
                right: 10000,
                bottom: 10000,
            },
            default_color: GlyphonColor::rgb(255, 255, 255),
            last_frame_touched: current_frame,
            last_hash: hash,
        };
        self.text_areas.push(FullText { buffer, params });
        let text_id = self.text_areas.len() - 1;

        return Some(text_id);
    }

    pub(crate) fn refresh_last_frame(&mut self, text_id: Option<usize>, current_frame: u64) {
        if let Some(text_id) = text_id {
            self.text_areas[text_id].params.last_frame_touched = current_frame;
        }
    }

    pub(crate) fn set_text_unchecked(&mut self, text_id: usize, text: &str) {
        let area = &mut self.text_areas[text_id];
        area.buffer.set_text(
            &mut self.font_system,
            text,
            Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );
    }

    pub(crate) fn set_text_attrs(&mut self, text_id: usize, attrs: Attrs) {
        let area = &mut self.text_areas[text_id];

        // Define new attributes
        // Apply new attributes to the entire text
        for line in &mut area.buffer.lines {
            line.set_attrs_list(AttrsList::new(attrs));
        }
    }

    pub(crate) fn set_text_align(&mut self, text_id: usize, align: Align) {
        for line in &mut self.text_areas[text_id].buffer.lines {
            line.set_align(Some(align));
        }
    }
}

impl Ui {
    pub(crate) fn format_into_scratch(&mut self, value: impl Display) {
        self.format_scratch.clear();
        let _ = write!(self.format_scratch, "{}", value);
    }

    // don't expect this to give you twin nodes automatically
    pub(crate) fn get_ref(&mut self, key: NodeKey) -> Option<UiNode> {
        let node_i = self.nodes.node_hashmap.get(&key.id())?.slab_i;
        return Some(self.get_ref_unchecked(node_i, &key));
    }

    // only for the macro, use get_ref
    pub(crate) fn get_ref_unchecked(&mut self, i: usize, _key: &NodeKey) -> UiNode {
        return UiNode {
            node_i: i,
            ui: self,
        };
    }

    pub(crate) fn add_or_update_node(&mut self, key: NodeKey) -> usize {
        let frame = self.sys.part.current_frame;

        // Check the node corresponding to the key's id.
        // We might find that the key has already been used in this same frame:
        //      in this case, we take note, and calculate a twin key to use to add a "twin" in the next section.
        // Otherwise, we add or refresh normally, and take note of the final i.
        let twin_check_result = match self.nodes.node_hashmap.entry(key.id()) {
            // Add a normal node (no twins).
            Entry::Vacant(v) => {
                let new_node = Node::new(&key, None);
                let final_i = self.nodes.nodes.insert(new_node);
                v.insert(NodeMapEntry::new(frame, final_i));

                UpdatedNormal { final_i }
            }
            Entry::Occupied(o) => {
                let old_map_entry = o.into_mut();

                match refresh_or_add_twin(frame, old_map_entry.last_frame_touched) {
                    // Refresh a normal node from the previous frame (no twins).
                    Refresh => {
                        let warning = "todo: refresh should be in place(), not here";
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
            UpdatedNormal { final_i } => (final_i, key.id()),
            NeedToUpdateTwin { twin_key, twin_n } => {
                match self.nodes.node_hashmap.entry(twin_key.id()) {
                    // Add new twin.
                    Entry::Vacant(v) => {
                        let new_twin_node = Node::new(&twin_key, Some(twin_n));
                        let real_final_i = self.nodes.nodes.insert(new_twin_node);
                        v.insert(NodeMapEntry::new(frame, real_final_i));
                        (real_final_i, twin_key.id())
                    }
                    // Refresh a twin from the previous frame.
                    Entry::Occupied(o) => {
                        let old_twin_map_entry = o.into_mut();

                        let warning = "todo: refresh should be in place(), not here";
                        let real_final_i = old_twin_map_entry.refresh(frame);

                        (real_final_i, twin_key.id())
                    }
                }
            }
        };

        return real_final_i;
    }

    /// Add a node to the `Ui` corresponding to `key` and returns an [`UiNode`] pointing to it.
    /// 
    /// Adding the node adds it to the `Ui`, but it won't be visible until it is "placed" in the tree. You can do this by calling [`Ui::place`] with the same key, or by calling [`place()`](UiNode::place) directly on the returned [`UiNode`].
    /// 
    /// The returned [`UiNode`] can also be used to set the appearance, size, text, etc. of the node, using [`UiNode`]'s builder methods.
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
    /// #[node_key] const RED_BUTTON: NodeKey;
    /// ui.add(RED_BUTTON)
    ///     .params(BUTTON)
    ///     .color(Color::RED)
    ///     .text("Increase");
    /// #
    /// #   }
    /// # }
    /// ```
    /// 
    /// # Details
    ///  
    /// - If a node corresponding to `key` was already added in a previous frame, then it will return a [`UiNode`] pointing to the old one.
    /// 
    /// - If one or more nodes corresponding to `key` were already added in the *same* frame, then it will create a "twin" node.
    /// It's usually clearer to use different keys, or to create sibling keys explicitely with [`NodeKey::sibling`], rather than to rely on this behavior.
    /// 
    /// # Similar Functions
    /// 
    /// - [`Ui::add_anon`] can also add a node, but without requiring a key.
    /// 
    /// - Shorthand functions like [`Ui::text`] and [`Ui::label`] can `add` and [place](`Ui::place`) simple nodes all in once without requiring a key.
    pub fn add(&mut self, key: NodeKey) -> UiNode {
        let i = self.add_or_update_node(key);
        return self.get_ref_unchecked(i, &key);
    }

    /// Exactly like [`Ui::add`], but without a key.
    /// 
    /// The added node will be anonymous, and it won't be reachable by methods like [`Ui::place`] or [`Ui::get_node`] that use a key.
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
    /// ui.add_anon()
    ///     .params(LABEL)
    ///     .color(Color::RED)
    ///     .text("Hello World")
    ///     .place();
    /// #
    /// #   }
    /// # }    
    /// ```
    pub fn add_anon(&mut self) -> UiNode {
        let id_from_tree_position = thread_local_peek_tree_position_hash();
        let anonymous_key = NodeKey::new(Id(id_from_tree_position), "");
        
        let i = self.add_or_update_node(anonymous_key);

        let uinode = self.get_ref_unchecked(i, &anonymous_key);
        return uinode; 
    }

    /// Place the node corresponding to `key` at a specific position in the GUI tree.
    /// 
    /// The position is defined by the position of the [`place`](Ui::place) call relative to [`nest`](UiPlacedNode::nest) calls.
    /// 
    /// Panics if it is called with a key that doesn't correspond to any previously added node, through either [Ui::add], [Ui::add_anon], or [Ui::text] or similar functions.
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
    /// # #[node_key] pub const PARENT: NodeKey;
    /// # #[node_key] pub const CHILD: NodeKey;
    /// #
    /// ui.add(PARENT).params(CONTAINER);
    /// ui.add(CHILD).params(BUTTON);
    /// 
    /// ui.place(PARENT).nest(|| {
    ///     ui.place(CHILD);
    /// });
    /// #
    /// #   }
    /// # }
    /// ```
    /// 
    /// [`UiNode::place`] does the same thing. It is called instead directly on an [`UiNode`], so it doesn't need a `NodeKey` argument to identify the node.
    /// 
    /// Compared to [`UiNode::place`], this function allows separating the code that adds the node and sets the params from the `place` code. This usually makes the tree layout much easier to read.
    ///
    // #[track_caller]
    pub fn place(&mut self, key: NodeKey) -> UiPlacedNode {
        // todo: panic bad
        let real_key = self.get_latest_twin_key(key).expect("Error: `place()`ing a node that was never `add()`ed.");
        let node_i = self.nodes.node_hashmap.get(&real_key.id()).unwrap().slab_i;

        return self.place_by_i(node_i);
    }

    pub(crate) fn place_by_i(&mut self, i: usize) -> UiPlacedNode {

        // refresh last_frame_touched. 
        // refreshing the node should go here as well.
        // but maybe not? the point was always that untouched nodes stay out of the tree and they get skipped automatically.
        // unless we still need the frame for things like pruning?
        let frame = self.sys.part.current_frame;
        self.sys.text.refresh_last_frame(self.nodes[i].text_id, frame);


        let old_children_hash = self.nodes[i].children_hash;
        // reset the children hash to keep it in sync with the thread local one (which will be created new in push_parent)
        self.nodes[i].children_hash = EMPTY_HASH;

        // update the in-tree links and the thread-local state based on the current parent.
        let NodeWithDepth { i: parent_i, depth } = thread_local_peek_parent();
        self.set_tree_links(i, parent_i, depth);

        // update the parent's **THREAD_LOCAL** children_hash with ourselves. (when the parent gets popped, it will be compared to the old one, which we passed to nest() before starting to add children)
        // AND THEN, sync the thread local hash value with the one on the node as well, so that we'll be able to use it for the old value next frame
        let children_hash_so_far = thread_local_hash_new_child(i);
        self.nodes[parent_i].children_hash = children_hash_so_far;

        let cosmetic_params_hash = self.nodes[i].params.cosmetic_update_hash();
        let layout_params_hash = self.nodes[i].params.partial_relayout_hash();

        let param_cosmetic_update = cosmetic_params_hash != self.nodes[i].last_cosmetic_params_hash;
        let param_partial_relayout = layout_params_hash != self.nodes[i].last_layout_params_hash;

        
        if self.nodes[i].needs_partial_relayout | param_partial_relayout {
            self.push_partial_relayout(i);
            self.nodes[i].last_layout_params_hash = layout_params_hash;
            self.nodes[i].needs_partial_relayout = false;
        }
        
        // push cosmetic updates
        if self.nodes[i].needs_cosmetic_update | param_cosmetic_update{
            self.push_cosmetic_update(i);
            self.nodes[i].last_cosmetic_params_hash = cosmetic_params_hash;
            self.nodes[i].needs_cosmetic_update = false;
        }

        // return the child_hash that this node had in the last frame, so that can new children can check against it.
        return UiPlacedNode::new(i, old_children_hash);
    }

    // todo: probably remove
    pub(crate) fn get_latest_twin_key(&self, key: NodeKey) -> Option<NodeKey> {
        let map_entry = self.nodes.node_hashmap.get(&key.id())?;

        if map_entry.n_twins == 0 {
            return Some(key);
        }

        // todo: yell a very loud warning here. latest_twin is more like a best-effort way to deal with dumb code.
        // the proper way is to just use unique keys, or to use the returned noderef, if that becomes a thing.
        let twin_key = key.sibling(map_entry.n_twins);

        return Some(twin_key);
    }

    fn set_tree_links(&mut self, new_node_i: usize, parent_i: usize, depth: usize) {
        // clear old tree links
        self.nodes[new_node_i].last_child = None;
        self.nodes[new_node_i].first_child = None;
        self.nodes[new_node_i].prev_sibling = None;
        self.nodes[new_node_i].next_sibling = None;
        // self.nodes[new_node_i].prev_sibling = None;
        self.nodes[new_node_i].n_children = 0;

        self.nodes[new_node_i].depth = depth;
        self.nodes[new_node_i].parent = parent_i;

        self.set_relayout_chain_root(new_node_i, parent_i);

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
    }

    fn add_first_child(&mut self, new_node_i: usize, parent_i: usize) {
        self.nodes[parent_i].last_child = Some(new_node_i);
        self.nodes[parent_i].first_child = Some(new_node_i);
    }
    
    fn add_sibling(&mut self, new_node_i: usize, old_last_child: usize, parent_i: usize) {
        self.nodes[new_node_i].prev_sibling = Some(old_last_child);
        self.nodes[old_last_child].next_sibling = Some(new_node_i);
        self.nodes[parent_i].last_child = Some(new_node_i);
    }

    /// Resize the `Ui`. 
    /// Updates the `Ui`'s internal state, and schedules a full relayout to adapt to the new size.
    /// Called by [`Ui::handle_events`].
    pub(crate) fn resize(&mut self, size: &PhysicalSize<u32>) {
        self.sys.changes.full_relayout = true;
        
        self.sys.part.unifs.size[X] = size.width as f32;
        self.sys.part.unifs.size[Y] = size.height as f32;

        self.sys.changes.resize = true;
    }

    pub(crate) fn update_time(&mut self) {
        self.sys.frame_t = ui_time_f32();
        
        let frame_time = self.sys.last_frame_timestamp.elapsed();

        if let Some(time) = &mut self.sys.changes.animation_rerender_time {
            *time = *time - frame_time.as_secs_f32();
        }
        if let Some(time) = self.sys.changes.animation_rerender_time {
            if time < 0.0 {
                self.sys.changes.animation_rerender_time = None;
            }
        }

        self.sys.last_frame_timestamp = Instant::now();
    }

    pub(crate) fn push_rect(&mut self, node: usize) {
        let node = &mut self.nodes.nodes[node];
        
        // really only need to do this whenever a custom-rendered rect shows up. But that would require custom rendered rects to be specifically marked, as opposed to just being the same as any other visible-only-in-debug rect, which means that you can forget to mark it and mess everything up. There's no real disadvantage to just always doing it.
        self.sys.z_cursor += Z_STEP;
        node.z = self.sys.z_cursor;

        let draw_even_if_invisible = self.sys.debug_mode;
        if let Some(rect) = node.render_rect(draw_even_if_invisible, None) {
            self.sys.rects.push(rect);
            node.last_rect_i = self.sys.rects.len() - 1;
        } else if node.params.interact.absorbs_mouse_events {
            // if it wasn't added to the regular rects but still needs to be clickable, add it to the lidl rects
            let just_give_me_a_rect = true;
            if let Some(rect) = node.render_rect(just_give_me_a_rect, None) {
                self.sys.invisible_but_clickable_rects.push(rect);
            }
        }

        if let Some(image) = node.imageref {
            if let Some(image_rect) = node.render_rect(draw_even_if_invisible, Some(image.tex_coords)) {
                self.sys.rects.push(image_rect);
            }
        }
    }

    pub(crate) fn update_rect(&mut self, node: usize) {
        let node = &mut self.nodes.nodes[node];

        let draw_even_if_invisible = self.sys.debug_mode;
        if let Some(rect) = node.render_rect(draw_even_if_invisible, None) {
            let old_i = node.last_rect_i;
            // if someone tries to update a rect that's no longer there, we could just ignore that instead of panicking.
            // however, it's probably a bug that we should fix.
            self.sys.rects[old_i] = rect;
        }
            
        // todo: update images?
    }

    // todo: actually call this once in a while
    pub(crate) fn prune(&mut self) {
        self.nodes.node_hashmap.retain(|_k, v| {
            // the > is to always keep the root node without having to refresh it
            let should_retain = v.last_frame_touched >= self.sys.part.current_frame;
            if !should_retain {
                let name = self.nodes.nodes[v.slab_i].debug_name();
                // side effect happens inside this closure? idk if this even works
                self.nodes.nodes.remove(v.slab_i);
                // remember to remove text areas and such ...
                println!("[{:?}] PRUNING {:?}", T0.elapsed(), name);
            }
            should_retain
        });
    }

    /// Returns an [`UiNode`] corresponding to `key`, if it exists.
    /// 
    /// This function ignores whether the node is currently inside the tree or not.
    /// 
    /// The returned [`UiNode`] can be used to get information about the node, through functions like [`UiNode::inner_size`] or [`UiNode::render_rect`]
    /// 
    /// To see if a node was clicked, use [`Ui::is_clicked`] and friends. In the future, those functions might be moved to [`UiNode`] as well.

    // todo: non-mut version of this?
    pub fn get_node(&mut self, key: NodeKey) -> Option<UiNode> {
        let real_key = self.get_latest_twin_key(key)?;
        return self.get_ref(real_key);
    }

    fn set_relayout_chain_root(&mut self, new_node_i: usize, parent_i: usize) {
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

    pub(crate) fn set_partial_relayout_flag(&mut self, node_i: usize) {
        self.nodes[node_i].needs_partial_relayout = true;
    }

    pub(crate) fn push_partial_relayout(&mut self, i: usize) {
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

    // this will be still needed for things like image/texture updates, I think. 
    pub(crate) fn _set_cosmetic_update_flag(&mut self, node_i: usize) {
        self.nodes[node_i].needs_cosmetic_update = true;
    }

    pub(crate) fn push_cosmetic_update(&mut self, i: usize) {
        self.sys.changes.cosmetic_rect_updates.push(i);
    }

    // todo: need_rerender should be on a flag too, not instant, right?
    // because the text might change on a node that ends up not being place()d
    pub(crate) fn push_text_change(&mut self, i: usize) {
        if self.nodes[i].params.is_fit_content() {
            self.set_partial_relayout_flag(i);
        } else {
            self.sys.changes.need_rerender = true;
        }
    }

    /// Clear the old GUI tree and start declaring another one.
    /// 
    /// Use together with [`Ui::finish_tree()`], at most once per frame.
    /// 
    /// ```rust
    /// # use keru::*;
    /// # use keru::*;
    /// 
    /// # pub struct State {
    /// #     pub ui: Ui,
    /// # }
    /// 
    /// # impl State {
    /// #   fn declare_ui(&mut self) {
    /// self.ui.begin_tree();
    /// // declare the GUI and update state
    /// self.ui.finish_tree();
    /// 
    /// #   }
    /// # }
    /// 
    /// ```
    pub fn begin_tree(&mut self) {
        // clear root
        self.nodes[ROOT_I].last_child = None;
        self.nodes[ROOT_I].first_child = None;        
        self.nodes[ROOT_I].n_children = 0;

        self.sys.part.current_frame += 1;
        clear_thread_local_parent_stack();
        self.format_scratch.clear();

        // messy manual equivalent of what we'd do when place()ing the root
        let old_root_hash = self.nodes[ROOT_I].children_hash;
        let root_parent = UiPlacedNode::new(ROOT_I, old_root_hash);
        self.nodes[ROOT_I].children_hash = EMPTY_HASH;

        thread_local_push_parent(&root_parent);
    }
    
    /// Finish declaring the current GUI tree.
    /// 
    /// This function will also relayout the nodes that need it, and do some bookkeeping.
    /// 
    /// Use at most once per frame, after calling [`Ui::begin_tree()`] and running your tree declaration code.
    pub fn finish_tree(&mut self) {
        // pop the root node
        thread_local_pop_parent();
        
        self.relayout();
        
        self.end_frame_resolve_inputs();
        
        self.update_time();
    }

    /// Add and place an anonymous vertical stack container.
    pub fn v_stack(&mut self) -> UiPlacedNode {
        self.add(ANON_VSTACK).params(V_STACK);
        return self.place(ANON_VSTACK);
    }
    
    /// Add and place an anonymous horizontal stack container.
    pub fn h_stack(&mut self) -> UiPlacedNode {
        self.add(ANON_HSTACK).params(H_STACK);
        return self.place(ANON_HSTACK);
    }

    /// Add and place an anonymous text element.
    pub fn text(&mut self, text: impl Display + Hash) -> UiPlacedNode {
        return self.add_anon().params(TEXT).text(text).place();
    }

    /// Add and place an anonymous label.
    pub fn label(&mut self, text: impl Display + Hash) -> UiPlacedNode {
        return self.add_anon().params(LABEL).text(text).place();
    }

    /// Returns `true` if a node corresponding to `key` exists and if it is currently part of the GUI tree. 
    pub fn is_in_tree(&self, key: NodeKey) -> bool {
        let node_i = self.nodes.node_hashmap.get(&key.id());
        if let Some(entry) = node_i {
            // todo: also return true if it's retained
            return entry.last_frame_touched == self.sys.part.current_frame;
        } else {
            return false;
        }
    }

    /// Experimental function for skipping declaration code when the underlying state is unchanged.
    pub fn place_and_assume_unchanged(&mut self, key: NodeKey) {
        let node_i = self.nodes.node_hashmap.get(&key.id());
        if let Some(entry) = node_i {
            // also set a retained flag
            self.place_by_i(entry.slab_i);
        }
    }
}


use std::hash::Hash;
pub(crate) fn fx_hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = FxHasher::default();
    value.hash(&mut hasher);
    hasher.finish()
}

/// A struct referring to a node that was [placed](Ui::place) on the tree. Allows adding nested children.
///  
/// Can be used to call [nest](UiPlacedNode::nest) and add more nodes as a parent of this one.
/// 
/// ```rust
/// # use keru::*;
/// # use keru::*;
/// # pub struct State {
/// #     pub ui: Ui,
/// # }
/// #
/// # impl State {
/// #    fn declare_ui(&mut self) {
/// #    let ui = &mut self.ui; 
/// #
/// # #[node_key] pub const PARENT: NodeKey;
/// # #[node_key] pub const CHILD: NodeKey;
/// #
///             // ↓ returns a `UiPlacedNode`
/// ui.place(PARENT).nest(|| {
///     ui.place(CHILD);
/// });
/// #
/// #   }
/// # }
/// ```
/// 
/// The nesting mechanism uses a bit of magic to avoid having to pass a [`Ui`] parameter into the closure.
/// Because of this, `UiPlacedNode` is actually a plain-old-data struct and doesn't contain a reference to the main [`Ui`] object, so it can technically be freely assigned to a variable and stored.
/// 
/// While there's nothing unsafe about that, it will almost surely lead to weird unreadable code. The intended use is to just call [nest](UiPlacedNode::nest) immediately after getting this struct from [`UiNode::place`], like in the example.
/// 
pub struct UiPlacedNode {
    pub(crate) node_i: usize,
    pub(crate) old_children_hash: u64,
}
impl UiPlacedNode {
    pub(crate) fn new(node_i: usize, old_children_hash: u64) -> UiPlacedNode {
        return UiPlacedNode {
            node_i,
            old_children_hash,
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
    /// # #[node_key] pub const PARENT: NodeKey;
    /// # #[node_key] pub const CHILD: NodeKey;
    /// #
    /// ui.place(PARENT).nest(|| {
    ///     ui.place(CHILD);
    /// });
    /// #
    /// #   }
    /// # }
    /// ```
    /// 
    /// Since the `content` closure doesn't borrow or move anything, it sets no restrictions at all on what code can be ran inside it.
    /// You can keep access and mutate both the `Ui` object and the rest of the program state freely, as you'd outside of the closure. 
    ///  
    pub fn nest(&self, content: impl FnOnce()) {
        thread_local_push_parent(self);

        content();

        thread_local_pop_parent();
    }
}

impl<'a> UiNode<'a> {
    /// Place the node at a specific position in the Ui tree.
    /// 
    /// The position is defined by the position of the [`place`](UiNode::place) call relative to [`nest`](UiPlacedNode::nest) calls.
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
    /// # #[node_key] pub const BUTTON_KEY: NodeKey;
    /// ui.add_anon().params(PANEL).place().nest(|| {
    ///     ui.add(BUTTON_KEY).place();
    /// });
    /// #
    /// #   }
    /// # }
    /// ```
    /// 
    /// [`Ui::place`] does the same thing, using a `NodeKey` argument to identify the node to place.
    /// 
    /// Compared to [`Ui::place`], this function doesn't allow separating the code that `add`s the node and sets the [`NodeParams`] and the code that defines the layout. 
    /// 
    /// However, it is fully panic-safe. 
    pub fn place(&mut self) -> UiPlacedNode {
        self.ui.place_by_i(self.node_i);
        let old_children_hash = self.node().children_hash;
        return UiPlacedNode::new(self.node_i, old_children_hash);
    }
}

