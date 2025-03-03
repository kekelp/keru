// todo: move some more stuff out of this file
use crate::*;

use glyphon::cosmic_text::Align;
use glyphon::{AttrsList, Color as GlyphonColor, TextBounds, Viewport};

use rustc_hash::FxHasher;

use std::collections::hash_map::Entry;
use std::hash::Hasher;
use std::panic::Location;

use bytemuck::{Pod, Zeroable};
use glyphon::{
    Attrs, Buffer as GlyphonBuffer, Family, FontSystem, Metrics, Shaping, SwashCache,
    TextAtlas, TextRenderer,
};
use winit::dpi::PhysicalSize;
use Axis::{X, Y};

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

        for line in &mut buffer.lines {
            line.set_align(Some(glyphon::cosmic_text::Align::Center));
        }

        // todo: maybe remove duplication with set_text_hashed (the branch in refresh_node that updates the text without creating a new entry here)
        // buffer.set_wrap(&mut self.font_system, glyphon::Wrap::Word);
        buffer.set_text(
            &mut self.font_system,
            text,
            Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );


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
    // todo: this function writes into format_scratch, doesn't tell anybody anything, and then expects people to get their string directly from self.format_scratch. is it really impossible to just return a reference? 
    pub(crate) fn format_into_scratch(&mut self, value: impl Display) {
        self.format_scratch.clear();
        let _ = write!(self.format_scratch, "{}", value);
    }


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
    pub fn add<'a, M, T>(&mut self, params: M) -> UiParent
    where
        M: Into<FullNodeParams<'a, T>>,
        T: Display + ?Sized + 'a,
    {
        let params = params.into();
        let key = params.key_or_anon_key();
        let i = self.add_or_update_node(key);
        self.set_params(i, &params);
        self.set_params_text(i, &params);
        return self.make_parent_from_i(i);
    }

    /// Returns an [`UiNode`] corresponding to `key`, if it exists.
    /// 
    /// This function ignores whether the node is currently inside the tree or not.
    // todo: why though? is that ever useful?
    /// 
    /// The returned [`UiNode`] can be used to get information about the node, through functions like [`UiNode::inner_size`] or [`UiNode::render_rect`]
    /// 
    /// To see if a node was clicked, use [`Ui::is_clicked`] and friends. In the future, those functions might be moved to [`UiNode`] as well.

    // todo: non-mut version of this?
    // todo: the pub version of this should give out a non-mut ref. You only get to change nodes by redeclaring them.
    // aka make a new UiNode that's not mutable and give that
    pub fn get_node(&mut self, key: NodeKey) -> Option<UiNode> {
        let node_i = self.nodes.node_hashmap.get(&key.id_with_subtree())?.slab_i;
        return Some(self.get_uinode(node_i));
    }

    // only for the macro, use get_ref
    pub(crate) fn get_uinode(&mut self, i: NodeI) -> UiNode {
        return UiNode {
            i,
            ui: self,
        };
    }

    #[track_caller]
    pub(crate) fn add_or_update_node(&mut self, key: NodeKey) -> NodeI {
        let frame = self.sys.current_frame;

        // todo: at least when using non-anonymous keys, I think there's no legit use case for twins anymore. it's always a mistake, I think. it should print out a warning or panic.

        // Check the node corresponding to the key's id.
        // We might find that the key has already been used in this same frame:
        //      in this case, we take note, and calculate a twin key to use to add a "twin" in the next section.
        // Otherwise, we add or refresh normally, and take note of the final i.
        let twin_check_result = match self.nodes.node_hashmap.entry(key.id_with_subtree()) {
            // Add a normal node (no twins).
            Entry::Vacant(v) => {
                let new_node = Node::new(&key, None, Location::caller());
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
                        let new_twin_node = Node::new(&twin_key, Some(twin_n), Location::caller());
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

        // refresh last_frame_touched. 
        // refreshing the node should go here as well.
        // but maybe not? the point was always that untouched nodes stay out of the tree and they get skipped automatically.
        // unless we still need the frame for things like pruning?
        let frame = self.sys.current_frame;
        self.sys.text.refresh_last_frame(self.nodes[real_final_i].text_id, frame);
        
        // update the in-tree links and the thread-local state based on the current parent.
        let NodeWithDepth { i: parent_i, depth } = thread_local::current_parent();
        self.set_tree_links(real_final_i, parent_i, depth);

        return real_final_i;
    }


    pub(crate) fn make_parent_from_i(&mut self, i: NodeI) -> UiParent {
        
        let old_children_hash = self.nodes[i].children_hash;
        // reset the children hash to keep it in sync with the thread local one (which will be created new in push_parent)
        self.nodes[i].children_hash = EMPTY_HASH;
        
        let parent_i = self.nodes[i].parent;

        // update the parent's **THREAD_LOCAL** children_hash with ourselves. (when the parent gets popped, it will be compared to the old one, which we passed to nest() before starting to add children)
        // AND THEN, sync the thread local hash value with the one on the node as well, so that we'll be able to use it for the old value next frame
        let children_hash_so_far = thread_local::hash_new_child(i);
        self.nodes[parent_i].children_hash = children_hash_so_far;


        // return the child_hash that this node had in the last frame, so that can new children can check against it.
        return UiParent::new(i, old_children_hash);
    }

    fn set_tree_links(&mut self, new_node_i: NodeI, parent_i: NodeI, depth: usize) {
        assert!(new_node_i != parent_i, "Internal error: tried to add a node as child of itself ({}).", self.nodes[new_node_i].debug_key_name);

        // clear old tree links
        self.nodes[new_node_i].last_child = None;
        self.nodes[new_node_i].first_child = None;
        self.nodes[new_node_i].prev_sibling = None;
        self.nodes[new_node_i].next_sibling = None;
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

    fn add_first_child(&mut self, new_node_i: NodeI, parent_i: NodeI) {
        self.nodes[parent_i].last_child = Some(new_node_i);
        self.nodes[parent_i].first_child = Some(new_node_i);
    }
    
    fn add_sibling(&mut self, new_node_i: NodeI, old_last_child: NodeI, parent_i: NodeI) {
        self.nodes[new_node_i].prev_sibling = Some(old_last_child);
        self.nodes[old_last_child].next_sibling = Some(new_node_i);
        self.nodes[parent_i].last_child = Some(new_node_i);
    }

    /// Resize the `Ui`. 
    /// Updates the `Ui`'s internal state, and schedules a full relayout to adapt to the new size.
    /// Called by [`Ui::window_event`].
    pub(crate) fn resize(&mut self, size: &PhysicalSize<u32>) {        
        self.sys.changes.full_relayout = true;
        
        self.sys.unifs.size[X] = size.width as f32;
        self.sys.unifs.size[Y] = size.height as f32;

        self.sys.changes.resize = true;
    }

    pub(crate) fn push_rect(&mut self, i: NodeI) {
        let node = &mut self.nodes[i];
        
        // really only need to do this whenever a custom-rendered rect shows up. But that would require custom rendered rects to be specifically marked, as opposed to just being the same as any other visible-only-in-debug rect, which means that you can forget to mark it and mess everything up. There's no real disadvantage to just always doing it.
        self.sys.z_cursor += Z_STEP;
        node.z = self.sys.z_cursor;

        let draw_even_if_invisible = self.sys.inspect_mode;
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

        if node.params.is_scrollable() {
            let just_give_me_a_rect = true;
            if let Some(rect) = node.render_rect(just_give_me_a_rect, None) {
                self.sys.scroll_rects.push(rect);
            }
        }

        if let Some(image) = node.imageref {
            if let Some(image_rect) = node.render_rect(draw_even_if_invisible, Some(image.tex_coords)) {
                self.sys.rects.push(image_rect);
            }
        }
    }

    pub(crate) fn update_rect(&mut self, i: NodeI) {
        let node = &mut self.nodes[i];

        let draw_even_if_invisible = self.sys.inspect_mode;
        if let Some(rect) = node.render_rect(draw_even_if_invisible, None) {
            let old_i = node.last_rect_i;
            // this panics all the time: big skill issue. solve with the dense maps thing, I guess.
            self.sys.rects[old_i] = rect;
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
        // clear root
        self.nodes[ROOT_I].last_child = None;
        self.nodes[ROOT_I].first_child = None;        
        self.nodes[ROOT_I].n_children = 0;

        self.sys.current_frame += 1;
        thread_local::clear_parent_stack();
        self.format_scratch.clear();

        // messy manual equivalent of what we'd do when add()ing the root
        let old_root_hash = self.nodes[ROOT_I].children_hash;
        let root_parent = UiParent::new(ROOT_I, old_root_hash);
        self.nodes[ROOT_I].children_hash = EMPTY_HASH;
        thread_local::push_parent(&root_parent);

        self.begin_frame_resolve_inputs();
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

        self.relayout();
        self.sys.third_last_frame_end_fake_time = self.sys.second_last_frame_end_fake_time;
        self.sys.second_last_frame_end_fake_time = self.sys.last_frame_end_fake_time;
        self.sys.last_frame_end_fake_time = fake_time_now();

        if self.sys.new_ui_input_1_more_frame {
            self.sys.new_ui_input_1_more_frame = false;
            self.sys.new_ui_input = true;
        } else {
            self.sys.new_ui_input = false;
        }

        self.sys.new_external_events = false;

        // let mut buffer = String::new();
        // std::io::stdin().read_line(&mut buffer).expect("Failed to read line");
    }

    /// Add a panel.
    #[track_caller]
    pub fn panel(&mut self) -> UiParent {
        return self.add(PANEL);
    }

    /// Add a vertical stack container.
    #[track_caller]
    pub fn v_stack(&mut self) -> UiParent {
        return self.add(V_STACK);
    }

    /// Add a spacer.
    #[track_caller]
    pub fn spacer(&mut self) -> UiParent {
        return self.add(SPACER);
    }
    
    /// Add a horizontal stack container.
    #[track_caller]
    pub fn h_stack(&mut self) -> UiParent {
        return self.add(H_STACK);
    }

    /// Add a single-line text element.
    #[track_caller]
    pub fn text_line<'a, T, M>(&mut self, text: &'a M) -> UiParent
    where
        T: Display + ?Sized,
        M: MaybeObserver<T> + ?Sized,
    {
        let params = TEXT.text(text);
        return self.add(params);
    }

    /// Add a single-line text element from a `'static str`.
    #[track_caller]
    pub fn static_text_line(&mut self, text: &'static str) -> UiParent {
        let params = TEXT.static_text(text);
        return self.add(params);
    }

    /// Add a multiline text paragraph.
    #[track_caller]
    pub fn paragraph<'a, T, M>(&mut self, text: &'a M) -> UiParent
    where
        T: Display + ?Sized,
        M: MaybeObserver<T> + ?Sized,
    {
        let params = TEXT_PARAGRAPH.text(text);
        return self.add(params);
    }

    /// Add a multiline text paragraph from a `'static str`.
    #[track_caller]
    pub fn static_paragraph(&mut self, text: &'static str) -> UiParent {
        let params = TEXT_PARAGRAPH.static_text(text);
        return self.add(params);
    }

    /// Add a label.
    #[track_caller]
    pub fn label<'a, T, M>(&mut self, text: &'a M) -> UiParent
    where
        T: Display + ?Sized,
        M: MaybeObserver<T> + ?Sized,
    {
        let params = MULTILINE_LABEL.text(text);
        return self.add(params);
    }

    /// Add a label from a `&static str`.
    #[track_caller]
    pub fn static_label(&mut self, text: &'static str) -> UiParent {
        let params = MULTILINE_LABEL.static_text(text);
        return self.add(params);
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
}


use std::hash::Hash;
pub(crate) fn fx_hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = FxHasher::default();
    value.hash(&mut hasher);
    hasher.finish()
}

#[track_caller]
pub(crate) fn caller_location_hash() -> u64 {
    let location = Location::caller();
    // it would be cool to avoid doing all these hashes at runtime, somehow.
    let caller_location_hash = fx_hash(location);
    return caller_location_hash;
}

/// A struct referring to a node that was [`added`](Ui::add) on the tree.
///  
/// Can be used to call [`nest()`](Self::nest()) and add more nodes as children of this one. 
pub struct UiParent {
    pub(crate) i: NodeI,
    pub(crate) old_children_hash: u64,
}
impl UiParent {
    pub(crate) fn new(node_i: NodeI, old_children_hash: u64) -> UiParent {
        return UiParent {
            i: node_i,
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

    /// Returns true if the added node was clicked.
    /// 
    /// This method allows to test for interactions right after adding a node, without needing to use a key.
    /// 
    /// This function needs to take an `&mut Ui` argument because `UiParent` doesn't hold a reference to the `Ui`, to allow greater flexibility when using [`UiParent::nest()`].
    pub fn is_clicked(&self, ui: &mut Ui) -> bool {
        return ui.get_uinode(self.i).is_clicked();
    }
}

pub(crate) fn start_info_log_timer() -> Option<std::time::Instant> {
    if log::max_level() >= log::LevelFilter::Info {
        Some(std::time::Instant::now())
    } else {
        None
    }
}
