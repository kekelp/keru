use crate::changes::NodeWithDepth;
use crate::*;
use crate::keys::*;
use crate::math::*;
use crate::param_library::*;
use crate::text::*;
use crate::node::*;
use crate::color::*;
use crate::render_rect::*;
use crate::thread_local::{clear_thread_local_parent_stack, thread_local_hash_new_child, thread_local_peek_parent, thread_local_pop_parent, thread_local_push_parent};
use glyphon::cosmic_text::Align;
use glyphon::{AttrsList, Color as GlyphonColor, TextBounds, Viewport};

use glyphon::Cache as GlyphonCache;

use rustc_hash::FxHasher;
use thread_local::thread_local_peek_tree_position_hash;
use wgpu::*;
use winit::event::MouseButton;

use std::collections::hash_map::Entry;
use std::sync::LazyLock;
use std::{
    hash::Hasher,
    marker::PhantomData,
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

pub(crate) static T0: LazyLock<Instant> = LazyLock::new(Instant::now);
pub fn ui_time_f32() -> f32 {
    return T0.elapsed().as_secs_f32();
}


#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, Pod, Zeroable)]
#[repr(C)]
pub struct Id(pub u64);

// this is what you get from FxHasher::default().finish()
pub(crate) const EMPTY_HASH: u64 = 0;

pub const FIRST_FRAME: u64 = 1;




impl PartialBorrowStuff {
    pub fn mouse_hit_rect(&self, rect: &RenderRect) -> bool {
        // rects are rebuilt from scratch every render, so this isn't needed, for now.
        // if rect.last_frame_touched != self.current_frame {
        //     return (false, false);
        // }

        let mut mouse_pos = (
            self.mouse_pos.x / self.unifs.size[X],
            1.0 - (self.mouse_pos.y / self.unifs.size[Y]),
        );

        // transform mouse_pos into "opengl" centered coordinates
        mouse_pos.0 = (mouse_pos.0 * 2.0) - 1.0;
        mouse_pos.1 = (mouse_pos.1 * 2.0) - 1.0;

        let hovered = rect.rect[X][0] < mouse_pos.0
            && mouse_pos.0 < rect.rect[X][1]
            && rect.rect[Y][0] < mouse_pos.1
            && mouse_pos.1 < rect.rect[Y][1];

        return hovered;
    }
}

// another stupid sub struct for dodging partial borrows
pub struct TextSystem {
    pub font_system: FontSystem,
    pub cache: SwashCache,
    pub atlas: TextAtlas,
    pub text_renderer: TextRenderer,
    pub text_areas: Vec<FullText>,
    pub glyphon_viewport: Viewport,
    pub glyphon_cache: GlyphonCache,
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

    pub(crate) fn set_text_hashed(&mut self, text_id: usize, text: &str) {
        let hash = fx_hash(&text);
        let area = &mut self.text_areas[text_id];
        if hash != area.params.last_hash {
            area.params.last_hash = hash;
            area.buffer.set_text(
                &mut self.font_system,
                text,
                Attrs::new().family(Family::SansSerif),
                Shaping::Advanced,
            );

            let yellow = "this path should be pushing a relayout change, but we're not in the main Ui struct (note how modularity lost again)";
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

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, Pod, Zeroable)]
#[repr(C)]
pub struct Idx(pub(crate) u64);

impl Ui {
    pub fn format_into_scratch(&mut self, value: impl Display) {
        self.format_scratch.clear();
        let _ = write!(self.format_scratch, "{}", value);
    }

    // don't expect this to give you twin nodes automatically
    pub fn get_ref<T: NodeType>(&mut self, key: TypedKey<T>) -> Option<UiNode<Any>> {
        let node_i = self.nodes.node_hashmap.get(&key.id())?.slab_i;
        return Some(self.get_ref_unchecked(node_i, &key));
    }

    // only for the macro, use get_ref
    pub fn get_ref_unchecked<T: NodeType>(&mut self, i: usize, _key: &TypedKey<T>) -> UiNode<Any> {
        return UiNode {
            node_i: i,
            ui: self,
            nodetype_marker: PhantomData::<Any>,
        };
    }

    pub fn add_or_update_node(&mut self, key: NodeKey) -> usize {
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
                        let yellow = "refresh should be in place(), not here";
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

                        let yellow = "refresh should be in place(), not here";
                        let real_final_i = old_twin_map_entry.refresh(frame);

                        (real_final_i, twin_key.id())
                    }
                }
            }
        };

        return real_final_i;
    }

    // #[track_caller]
    pub fn place(&mut self, key: NodeKey) -> UiPlacedNode {
        // println!("Albeit {}", std::panic::Location::caller());
        let yellow = "don't unwrap here! Just return an empty UiPlacedNode or something. Figure something out";
        let real_key = self.get_latest_twin_key(key).unwrap();
        let node_i = self.nodes.node_hashmap.get(&real_key.id()).unwrap().slab_i;

        return self.place_by_i(node_i);
    }

    pub fn place_by_i(&mut self, i: usize) -> UiPlacedNode {

        // refresh last_frame_touched. 
        // refreshing the node should go here as well.
        // but maybe not? the point was always that untouched nodes stay out of the tree and they get skipped automatically.
        // unless we still need the frame for things like pruning?
        let frame = self.sys.part.current_frame;
        self.sys.text.refresh_last_frame(self.nodes[i].text_id, frame);

        // update the in-tree links and the thread-local state based on the current parent.
        let NodeWithDepth { i: parent_i, depth } = thread_local_peek_parent();
        self.set_tree_links(i, parent_i, depth);
        let children_hash_so_far = thread_local_hash_new_child(i);
        self.nodes[parent_i].old_children_hash = children_hash_so_far;

        let old_children_hash = self.nodes[i].old_children_hash;


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

        return UiPlacedNode::new(i, old_children_hash);
    }

    pub(crate) fn get_latest_twin_key<T: NodeType>(&self, key: TypedKey<T>) -> Option<TypedKey<T>> {
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
        // clean old state
        self.nodes[new_node_i].last_child = None;
        self.nodes[new_node_i].prev_sibling = None;
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
    }

    fn add_sibling(&mut self, new_node_i: usize, old_last_child: usize, parent_i: usize) {
        self.nodes[new_node_i].prev_sibling = Some(old_last_child);
        self.nodes[parent_i].last_child = Some(new_node_i);
    }

    pub fn resize(&mut self, size: &PhysicalSize<u32>, queue: &Queue) {
        self.sys.changes.full_relayout = true;
        
        self.sys.part.unifs.size[X] = size.width as f32;
        self.sys.part.unifs.size[Y] = size.height as f32;

        self.sys.text.glyphon_viewport.update(
            queue,
            glyphon::Resolution {
                width: self.sys.part.unifs.size.x as u32,
                height: self.sys.part.unifs.size.y as u32,
            },
        );

        let yellow = "change this";
        queue.write_buffer(
            &self.sys.base_uniform_buffer,
            0,
            &bytemuck::bytes_of(&self.sys.part.unifs)[..16],
        );
    }

    pub fn update_time(&mut self) {
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

    pub fn needs_rerender(&self) -> bool {
        return self.sys.changes.need_rerender || self.sys.changes.animation_rerender_time.is_some();
    }

    pub fn push_rect(&mut self, node: usize) {
        let node = &mut self.nodes.nodes[node];

        let draw_even_if_invisible = self.sys.debug_mode;
        if let Some(rect) = node.render_rect(draw_even_if_invisible) {
            self.sys.rects.push(rect);
            node.last_rect_i = self.sys.rects.len() - 1;
        }

        if let Some(image_rect) = node.image_rect() {
            self.sys.rects.push(image_rect);
        }
    }

    pub fn update_rect(&mut self, node: usize) {
        let node = &mut self.nodes.nodes[node];

        let draw_even_if_invisible = self.sys.debug_mode;
        if let Some(rect) = node.render_rect(draw_even_if_invisible) {
            let old_i = node.last_rect_i;
            self.sys.rects[old_i] = rect;
        }
            
        // todo: update images?
    }

    pub fn push_cursor_rect(&mut self) -> Option<()> {
        // cursor
        // how to make it appear at the right z? might be impossible if there are overlapping rects at the same z.
        // one epic way could be to increase the z sequentially when rendering, so that all rects have different z's, so the cursor can have the z of its rect plus 0.0001.
        // anyone doing custom rendering won't mind having to fetch a dynamic Z since they're fetching dynamic x's and y's all the time.

        // it's a specific choice by me to keep cursors for every string at all times, but only display (and use) the one on the currently focused ui node.
        // someone might want multi-cursor in the same node, multi-cursor on different nodes, etc.
        // let focused_id = &self.sys.focused?;
        // let focused_node = self.nodes.get_by_id(focused_id)?;
        // let text_id = focused_node.text_id?;
        // let focused_text_area = self.sys.text.text_areas.get(text_id)?;

        // match focused_text_area.buffer.lines[0].text.cursor() {
        //     StringCursor::Point(cursor) => {
        //         let rect_x0 = focused_node.rect[X][0];
        //         let rect_y1 = focused_node.rect[Y][1];

        //         let (x, y) = cursor_pos_from_byte_offset(&focused_text_area.buffer, *cursor);

        //         let cursor_width = focused_text_area.buffer.metrics().font_size / 20.0;
        //         let cursor_height = focused_text_area.buffer.metrics().font_size;
        //         // we're counting on this always happening after layout. which should be safe.
        //         let x0 = ((x - 1.0) / self.sys.part.unifs.size[X]) * 2.0;
        //         let x1 = ((x + cursor_width) / self.sys.part.unifs.size[X]) * 2.0;
        //         let x0 = x0 + (rect_x0 * 2. - 1.);
        //         let x1 = x1 + (rect_x0 * 2. - 1.);

        //         let y0 = ((-y - cursor_height) / self.sys.part.unifs.size[Y]) * 2.0;
        //         let y1 = ((-y) / self.sys.part.unifs.size[Y]) * 2.0;
        //         let y0 = y0 + (rect_y1 * 2. - 1.);
        //         let y1 = y1 + (rect_y1 * 2. - 1.);

        //         let cursor_rect = RenderRect {
        //             rect: XyRect::new([x0, x1], [y0, y1]),
        //             vertex_colors: VertexColors::flat(Color::rgba(128, 77, 128, 230)),
        //             last_hover: 0.0,
        //             last_click: 0.0,
        //             click_animation: 0,
        //             z: 0.0,
        //             id: Id(0),
        //             filled: 1,
        //             radius: 0.0,
        //             tex_coords: Xy::new([0.0, 0.0], [0.0, 0.0]),
        //         };

        //         self.sys.rects.push(cursor_rect);
        //     }
        //     StringCursor::Selection(selection) => {
        //         let rect_x0 = focused_node.rect[X][0];
        //         let rect_y1 = focused_node.rect[Y][1];

        //         let (x0, y0) =
        //             cursor_pos_from_byte_offset(&focused_text_area.buffer, selection.start);
        //         let (x1, y1) =
        //             cursor_pos_from_byte_offset(&focused_text_area.buffer, selection.end);

        //         // let cursor_width = focused_text_area.buffer.metrics().font_size / 20.0;
        //         let cursor_height = focused_text_area.buffer.metrics().font_size;
        //         let x0 = ((x0 - 1.0) / self.sys.part.unifs.size[X]) * 2.0;
        //         let x1 = ((x1 + 1.0) / self.sys.part.unifs.size[X]) * 2.0;
        //         let x0 = x0 + (rect_x0 * 2. - 1.);
        //         let x1 = x1 + (rect_x0 * 2. - 1.);

        //         let y0 = ((-y0 - cursor_height) / self.sys.part.unifs.size[Y]) * 2.0;
        //         let y1 = ((-y1) / self.sys.part.unifs.size[Y]) * 2.0;
        //         let y0 = y0 + (rect_y1 * 2. - 1.);
        //         let y1 = y1 + (rect_y1 * 2. - 1.);

        //         let cursor_rect = RenderRect {
        //             rect: XyRect::new([x0, x1], [y0, y1]),
        //             vertex_colors: VertexColors::flat(Color::rgba(128, 77, 128, 230)),
        //             last_hover: 0.0,
        //             last_click: 0.0,
        //             click_animation: 0,
        //             z: 0.0,
        //             id: Id(0),
        //             filled: 1,
        //             radius: 0.0,

        //             tex_coords: Xy::new([0.0, 0.0], [0.0, 0.0]),
        //         };

        //         self.sys.rects.push(cursor_rect);
        //     }
        // }

        return Some(());
    }

    // todo: actually call this once in a while
    pub fn prune(&mut self) {
        self.nodes.node_hashmap.retain(|k, v| {
            // the > is to always keep the root node without having to refresh it
            let should_retain = v.last_frame_touched >= self.sys.part.current_frame;
            if !should_retain {
                // side effect happens inside this closure... weird
                self.nodes.nodes.remove(v.slab_i);
                // remember to remove text areas and such ...
                println!(" PRUNING {:?} {:?}", k, v);
            }
            should_retain
        });
    }

    pub fn get_node(&mut self, key: TypedKey<Any>) -> Option<UiNode<Any>> {
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
}


impl Ui {
    // in case of partial declarative stuff, think of another name
    pub fn begin_tree(&mut self) {
        self.sys.part.current_frame += 1;
        clear_thread_local_parent_stack();
    }

    pub fn finish_tree(&mut self) {
        self.relayout();

        self.end_frame_check_inputs();

        self.sys.last_frame_clicks.clear();

        self.update_time();

        // .......
        self.nodes[self.sys.root_i].last_child = None;
    }
}


use std::hash::Hash;
fn fx_hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = FxHasher::default();
    value.hash(&mut hasher);
    hasher.finish()
}

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

    pub fn nest(&self, content: impl FnOnce()) {
        thread_local_push_parent(self);

        content();

        thread_local_pop_parent();
    }
}

impl<'a, T: NodeType> UiNode<'a, T> {
    pub fn place(&mut self) -> UiPlacedNode {
        self.ui.place_by_i(self.node_i);
        let old_children_hash = self.node().old_children_hash;
        return UiPlacedNode::new(self.node_i, old_children_hash);
    }
}

impl Ui {
    pub fn add(&mut self, key: NodeKey) -> UiNode<Any> {
        let i = self.add_or_update_node(key);
        return self.get_ref_unchecked(i, &key);
    }

    pub fn add_anon(&mut self, params: NodeParams) -> UiNode<Any> {
        let id_from_tree_position = thread_local_peek_tree_position_hash();
        // the params usually change after the add(), so no point in trying to get a debug name from the params
        let anonymous_key = NodeKey::new(Id(id_from_tree_position), "");
        
        let i = self.add_or_update_node(anonymous_key);

        let mut uinode = self.get_ref_unchecked(i, &anonymous_key);
        uinode.params(params);
        return uinode; 
    }

    pub fn v_stack(&mut self) -> UiPlacedNode {
        self.add(ANON_VSTACK).params(V_STACK);
        return self.place(ANON_VSTACK);
    }

    pub fn place_h_stack(&mut self) -> UiPlacedNode {
        self.add(ANON_HSTACK).params(H_STACK);
        return self.place(ANON_HSTACK);
    }

    pub fn text(&mut self, text: impl Display + Hash) -> UiPlacedNode {
        return self.add_anon(TEXT).text(text).place();
    }

    pub fn label(&mut self, text: impl Display + Hash) -> UiPlacedNode {
        return self.add_anon(LABEL).text(text).place();
    }
}
