use crate::changes::NodeWithDepth;
use crate::*;
use crate::keys::*;
use crate::math::*;
use crate::param_library::*;
use crate::text::*;
use crate::texture_atlas::*;
use crate::thread_local::{clear_thread_local_parent_stack, thread_local_hash_new_child, thread_local_peek_parent, thread_local_pop, thread_local_push};
use glyphon::cosmic_text::Align;
use glyphon::{AttrsList, Color as GlyphonColor, TextBounds, Viewport};

use glyphon::Cache as GlyphonCache;

use rustc_hash::{FxHashMap, FxHasher};
use slab::Slab;
use wgpu::*;

use std::collections::hash_map::Entry;
use std::sync::LazyLock;
use std::{
    hash::Hasher,
    marker::PhantomData,
    ops::{Index, IndexMut},
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

// ...because it will be added first?
pub const ROOT_I: usize = 0;

pub const NODE_ROOT_ID: Id = Id(0);
pub const NODE_ROOT: Node = Node {
    id: NODE_ROOT_ID,
    depth: 0,
    rect: Xy::new_symm([0.0, 1.0]),
    size: Xy::new_symm(1.0),
    text_id: None,

    imageref: None,
    last_static_image_ptr: None,
    last_static_text_ptr: None,

    parent: usize::MAX,
    old_parent: usize::MAX,

    n_children: 0,
    last_child: None,
    prev_sibling: None,
    old_first_child: None,
    old_prev_sibling: None,

    is_twin: None,

    params: NODE_ROOT_PARAMS,
    debug_name: "Root",
    last_hover: f32::MIN,
    last_click: f32::MIN,
    z: -10000.0,
    last_rect_i: 0,
    relayout_chain_root: None,
    old_children_hash: EMPTY_HASH,
    last_layout_frame: 0,
};

// Despite the name, these are also used for hit-testing mouse clicks.
#[repr(C)]
#[derive(Default, Debug, Pod, Copy, Clone, Zeroable)]
pub struct RenderRect {
    pub rect: XyRect,               // (f32, f32) for each corner
    pub tex_coords: XyRect,         // (f32, f32) for texture coordinates
    pub vertex_colors: VertexColors, // (u8, u8, u8, u8) colors
    
    pub z: f32,                     // (f32) depth information
    pub last_hover: f32,            // (f32) hover timestamp
    pub last_click: f32,            // (f32) click timestamp
    pub radius: f32,                // (f32) radius
    
    pub flags: u32,                 // (u32) bitfield flags
    pub _padding: u32,        // (u32) next free block index

    // this is currently used for click resolution, but it's not used for anything on the gpu.
    // in the future, I would like to have a separate structure for click resolution, and remove the Id from this structure.
    pub id: Id,
}

impl RenderRect {
    pub fn buffer_desc() -> [VertexAttribute; 15] {
        vertex_attr_array![
            // rect (XyRect): 2 x Float32x2
            0 => Float32x2, // rect.x_min, rect.y_min
            1 => Float32x2, // rect.x_max, rect.y_max

            // tex_coords (XyRect): 2 x Float32x2
            2 => Float32x2, // tex_coords.x_min, tex_coords.y_min
            3 => Float32x2, // tex_coords.x_max, tex_coords.y_max

            // vertex_colors (VertexColors): 4 x Uint8x4
            4 => Uint8x4, // vertex_colors[0]
            5 => Uint8x4, // vertex_colors[1]
            6 => Uint8x4, // vertex_colors[2]
            7 => Uint8x4, // vertex_colors[3]

            8 => Float32,  // z
            9 => Float32,  // last_hover
            10 => Float32, // last_click
            11 => Float32, // radius

            12 => Uint32, // flags
            13 => Uint32, // slab_next_free
            
            14 => Uint32x2, // id. it's actually a u64, but it doesn't look like wgpu understands u64s.
        ]
    }
}

#[rustfmt::skip]
impl RenderRect {
    pub const CLICK_ANIMATION: u32 = 1 << 0;
    pub const OUTLINE_ONLY:    u32 = 1 << 1;

    pub const EMPTY_FLAGS: u32 = 0;
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Zeroable, Pod)]
#[repr(C)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
impl Color {
    pub const fn alpha(mut self, alpha: u8) -> Self {
        self.a = alpha;
        return self;
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Zeroable, Pod)]
#[repr(C)]
pub struct VertexColors {
    top_left: Color,
    top_right: Color,
    bottom_left: Color,
    bottom_right: Color,
}
impl VertexColors {
    pub const FLGR_SOVL_GRAD: Self =
        VertexColors::diagonal_gradient_backslash(Color::FLGR_BLUE, Color::FLGR_RED);

    pub const TEST: Self = Self {
        top_left: Color::rgba(255, 0, 0, 255),
        top_right: Color::rgba(0, 255, 0, 255),
        bottom_left: Color::rgba(0, 0, 255, 255),
        bottom_right: Color::rgba(255, 255, 255, 255),
    };
    pub const TEST2: Self = Self {
        top_left: Color::WHITE,
        top_right: Color::RED,
        bottom_left: Color::WHITE,
        bottom_right: Color::WHITE,
    };
    pub const fn new(tl: Color, tr: Color, bl: Color, br: Color) -> VertexColors {
        return VertexColors {
            top_left: tl,
            top_right: tr,
            bottom_left: bl,
            bottom_right: br,
        };
    }

    pub const fn flat(color: Color) -> VertexColors {
        return VertexColors::new(color, color, color, color);
    }

    pub const fn h_gradient(left: Color, right: Color) -> VertexColors {
        return VertexColors::new(left, right, left, right);
    }

    pub const fn v_gradient(top: Color, bottom: Color) -> VertexColors {
        return VertexColors::new(top, top, bottom, bottom);
    }

    // techinically, the blended corners shouldn't be blended with weight 0.5. The weight should depend on the aspect ratio, I think. I don't think that's practical though, and it looks okay like this.
    pub const fn diagonal_gradient_forward_slash(
        bottom_left: Color,
        top_right: Color,
    ) -> VertexColors {
        let blended = bottom_left.blend(top_right, 255 / 2);
        return VertexColors {
            top_left: blended,
            top_right,
            bottom_left,
            bottom_right: blended,
        };
    }

    pub const fn diagonal_gradient_backslash(top_left: Color, bottom_right: Color) -> VertexColors {
        let blended = top_left.blend(bottom_right, 255 / 2);
        return VertexColors {
            top_left,
            top_right: blended,
            bottom_left: blended,
            bottom_right,
        };
    }
}

impl Color {
    pub const FLGR_BLACK: Color = Color {
        r: (0.6 * 255.0) as u8,
        g: (0.3 * 255.0) as u8,
        b: (0.6 * 255.0) as u8,
        a: 255_u8,
    };

    pub const FLGR_DEBUG_RED: Color = Color::rgba(255, 0, 0, 77);

    pub const RED: Color = Color::rgba(255, 0, 0, 255);
    pub const GREEN: Color = Color::rgba(0, 255, 0, 255);
    pub const BLUE: Color = Color::rgba(0, 0, 255, 255);
    pub const BLACK: Color = Color::rgba(0, 0, 0, 255);

    pub const WHITE: Color = Color::rgba(255, 255, 255, 255);
    pub const TRANSPARENT: Color = Color::rgba(255, 255, 255, 0);

    pub const FLGR_BLUE: Color = Color::rgba(26, 26, 255, 255);
    pub const FLGR_RED: Color = Color::rgba(255, 26, 26, 255);
    pub const FLGR_GREEN: Color = Color::rgba(26, 255, 26, 255);

    pub const LIGHT_BLUE: Color = Color {
        r: (0.9 * 255.0) as u8,
        g: (0.7 * 255.0) as u8,
        b: (1.0 * 255.0) as u8,
        a: (0.6 * 255.0) as u8,
    };

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color { r, g, b, a }
    }

    pub const fn blend_channel(c1: u8, c2: u8, factor: u8) -> u8 {
        let inv_factor = 255 - factor;
        let res = (c1 as u16 * inv_factor as u16 + c2 as u16 * factor as u16) / 255;
        res as u8
    }

    // todo: in a future version of rust, rewrite with float factor
    // (can't use floats in const functions in current stable rust)
    pub const fn blend(self, other: Color, factor: u8) -> Color {
        Color {
            r: Color::blend_channel(self.r, other.r, factor),
            g: Color::blend_channel(self.g, other.g, factor),
            b: Color::blend_channel(self.b, other.b, factor),
            a: Color::blend_channel(self.a, other.a, factor),
        }
    }
}

pub struct UiNode<'a, T: NodeType> {
    pub(crate) node_i: usize,
    pub(crate) ui: &'a mut Ui,
    pub(crate) nodetype_marker: PhantomData<T>,
}

// why can't you just do it separately?
impl<'a, T: NodeType> UiNode<'a, T> {
    pub fn node_mut(&mut self) -> &mut Node {
        return &mut self.ui.nodes.nodes[self.node_i];
    }
    pub fn node(&self) -> &Node {
        return &self.ui.nodes.nodes[self.node_i];
    }

    pub fn static_image(&mut self, image: &'static [u8]) {
        let image_pointer: *const u8 = image.as_ptr();

        if let Some(last_pointer) = self.node().last_static_image_ptr {
            if image_pointer == last_pointer {
                return;
            }
        }

        let image = self.ui.sys.texture_atlas.allocate_image(image);
        self.node_mut().imageref = Some(image);
        self.node_mut().last_static_image_ptr = Some(image_pointer);
    }

    pub fn dynamic_image(&mut self, image: &[u8], changed: bool) {
        if self.node_mut().imageref.is_some() && changed == false {
            return;
        }

        let image = self.ui.sys.texture_atlas.allocate_image(image);
        self.node_mut().imageref = Some(image);
        self.node_mut().last_static_image_ptr = None;
    }

    // pub fn is_clicked(&self) -> bool {
    //     let id = self.node.id;
    //     return self.sys.clicked.contains(&real_key.id);

    // }

    // pub fn is_dragged(&self) -> Option<(f64, f64)> {
    //     if self.is_clicked(node_key) {
    //         return Some(self.sys.mouse_status.cursor_diff())
    //     } else {
    //         return None;
    //     }
    // }

    pub fn set_color(&mut self, color: Color) -> &mut Self {
        self.node_mut().params.rect.vertex_colors = VertexColors::flat(color);
        return self;
    }

    pub fn set_vertex_colors(&mut self, colors: VertexColors) -> &mut Self {
        self.node_mut().params.rect.vertex_colors = colors;
        return self;
    }

    pub fn set_position_x(&mut self, position: Position) -> &mut Self {
        self.node_mut().params.layout.position.x = position;
        return self;
    }

    pub fn set_position_y(&mut self, position: Position) -> &mut Self {
        self.node_mut().params.layout.position.y = position;
        return self;
    }

    pub fn set_size_x(&mut self, size: Size) -> &mut Self {
        self.node_mut().params.layout.size.x = size;
        return self;
    }

    pub fn set_size_y(&mut self, size: Size) -> &mut Self {
        self.node_mut().params.layout.size.y = size;
        return self;
    }

    pub fn inner_size(&self) -> Xy<u32> {
        let padding = self.node().params.layout.padding;
        let padding = self.ui.to_pixels2(padding);
        
        let size = self.node().size;
        let size = self.ui.f32_size_to_pixels2(size);

        return size - padding;
    }
}

impl<'a, T: TextTrait> UiNode<'a, T> {
    pub fn static_text(mut self, text: &'static str) -> Self {
        let text_pointer: *const u8 = text.as_ptr();

        if let Some(last_pointer) = self.node().last_static_text_ptr {
            if text_pointer == last_pointer {
                return self;
            }
        }

        if let Some(text_id) = self.node_mut().text_id {
            self.ui.sys.text.set_text_unchecked(text_id, text);
        } else {
            let text_id = self
                .ui
                .sys
                .text
                .maybe_new_text_area(Some(text), self.ui.sys.part.current_frame);
            self.node_mut().text_id = text_id;
        }

        self.node_mut().last_static_text_ptr = Some(text_pointer);

        self.ui.push_partial_relayout(self.node_i);

        return self;
    }

    pub fn text(mut self, text: &str) -> Self {
        if let Some(text_id) = self.node_mut().text_id {
            self.ui.sys.text.set_text_hashed(text_id, text);
        } else {
            let text_id = self
                .ui
                .sys
                .text
                .maybe_new_text_area(Some(text), self.ui.sys.part.current_frame);
            self.node_mut().text_id = text_id;
        }

        return self;
    }

    pub fn dyn_text(mut self, into_text: Option<impl Display>) -> Self {
        // if the text is None, return.
        let Some(into_text) = into_text else {
            return self;
        };
        
        self.ui.format_into_scratch(into_text);
        
        if let Some(text_id) = self.node_mut().text_id {
            self.ui.sys.text.set_text_unchecked(text_id, &self.ui.format_scratch);
        } else {
            let text_id = self
                .ui
                .sys
                .text
                .maybe_new_text_area(Some(&self.ui.format_scratch), self.ui.sys.part.current_frame);
            self.node_mut().text_id = text_id;
        }

        self.ui.push_partial_relayout(self.node_i);

        return self;
    }

    pub fn set_text_attrs(&mut self, attrs: Attrs) -> &mut Self {
        if let Some(text_id) = self.node_mut().text_id {
            self.ui.sys.text.set_text_attrs(text_id, attrs);
        } else {
            // todo: log a warning or something
            // or make these things type safe somehow
        }
        return self;
    }

    pub fn set_text_align(&mut self, align: Align) -> &mut Self {
        if let Some(text_id) = self.node_mut().text_id {
            self.ui.sys.text.set_text_align(text_id, align);
        } else {
            // todo: log a warning or something
            // or make these things type safe somehow
        }
        return self;
    }

    // todo: in a sane world, this wouldn't allocate.
    pub fn get_text(&self) -> Option<String> {
        // let text_id = self.node().text_id.unwrap();

        // let lines = self.ui.sys.text.text_areas[text_id].buffer.lines;
        
        // let text = lines.into_iter().map(|l| l.text()).collect();
        // return Some(text);
        return None;
    }
}


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

    fn refresh_last_frame(&mut self, text_id: Option<usize>, current_frame: u64) {
        if let Some(text_id) = text_id {
            self.text_areas[text_id].params.last_frame_touched = current_frame;
        }
    }

    fn set_text_hashed(&mut self, text_id: usize, text: &str) {
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

    fn set_text_unchecked(&mut self, text_id: usize, text: &str) {
        let area = &mut self.text_areas[text_id];
        area.buffer.set_text(
            &mut self.font_system,
            text,
            Attrs::new().family(Family::SansSerif),
            Shaping::Advanced,
        );
    }

    fn set_text_attrs(&mut self, text_id: usize, attrs: Attrs) {
        let area = &mut self.text_areas[text_id];

        // Define new attributes
        // Apply new attributes to the entire text
        for line in &mut area.buffer.lines {
            line.set_attrs_list(AttrsList::new(attrs));
        }
    }

    fn set_text_align(&mut self, text_id: usize, align: Align) {
        for line in &mut self.text_areas[text_id].buffer.lines {
            line.set_align(Some(align));
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, Pod, Zeroable)]
#[repr(C)]
pub struct Idx(pub(crate) u64);

#[derive(Debug, Clone, Copy)]
pub struct NodeMapEntry {
    pub last_parent: usize,
    pub last_frame_touched: u64,

    // keeping track of the twin situation.
    // This is the number of twins of a node that showed up SO FAR in the current frame. it gets reset every frame (on refresh().)
    // for the 0-th twin of a family, this will be the total number of clones of itself around. (not including itself, so starts at zero).
    // the actual twins ARE twins, but they don't HAVE twins, so this is zero.
    // for this reason, "clones" or "copies" would be better names, but those words are loaded in rust
    // reproduction? replica? imitation? duplicate? version? dupe? replication? mock? carbon?
    pub n_twins: u32,
    pub slab_i: usize,
}
impl NodeMapEntry {
    pub fn new(parent_id: usize, frame: u64, new_i: usize) -> Self {
        return Self {
            last_parent: parent_id,
            last_frame_touched: frame,
            n_twins: 0,
            slab_i: new_i,
        };
    }

    pub fn refresh(&mut self, parent_id: usize, frame: u64) -> usize {
        self.last_frame_touched = frame;
        self.last_parent = parent_id;
        self.n_twins = 0;
        return self.slab_i;
    }
}

#[derive(Debug)]
pub struct Nodes {
    // todo: make faster o algo
    pub node_hashmap: FxHashMap<Id, NodeMapEntry>,
    pub nodes: Slab<Node>,
}
impl Nodes {
    pub fn get_by_id(&mut self, id: &Id) -> Option<&mut Node> {
        let i = self.node_hashmap.get(id)?.slab_i;
        return self.nodes.get_mut(i);
    }
}
impl Index<usize> for Nodes {
    type Output = Node;
    fn index(&self, i: usize) -> &Self::Output {
        return &self.nodes[i];
    }
}
impl IndexMut<usize> for Nodes {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        return &mut self.nodes[i];
    }
}

impl System {
    pub fn build_new_node<T: NodeType>(
        &mut self,
        key: &TypedKey<T>,
        params: &NodeParams,
        twin_n: Option<u32>,
        depth: usize,
    ) -> Node {
        // add back somewhere

        return Node {
            id: key.id(),
            depth,
            rect: Xy::new_symm([0.0, 1.0]),
            size: Xy::new_symm(10.0),
            text_id: None,

            imageref: None,
            last_static_image_ptr: None,
            last_static_text_ptr: None,

            parent: 0, // just a wrong value which will be overwritten. it's even worse here.
            // but it's for symmetry with update_node, where all these values are old and are reset.
            old_parent: 0,

            n_children: 0,
            last_child: None, // will be overwritten later... not the cleanest
            prev_sibling: None, // will be overwritten later... not the cleanest
            old_first_child: None, // same here
            old_prev_sibling: None, // same

            is_twin: twin_n,
            params: *params,
            debug_name: key.debug_name,
            last_hover: f32::MIN,
            last_click: f32::MIN,
            z: 0.0,
            last_rect_i: 0,
            relayout_chain_root: None, // will be overwritten later... not the cleanest
            old_children_hash: EMPTY_HASH,
            last_layout_frame: 0,
        };
    }
}

impl Ui {
    /// Determine if the params change means that the Ui needs to be relayouted and/or rerendered
    fn watch_params_change(&mut self, node_i: usize, old: NodeParams, new: NodeParams) {
        // todo: maybe improve with hashes and stuff?
        if old.layout != new.layout {
            self.push_partial_relayout(node_i);
        }

        if old.rect != new.rect {
            self.push_cosmetic_rect_update(node_i);
        }
    }

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

    pub fn add_or_update_node<T: NodeType>(&mut self, key: TypedKey<T>, params: &NodeParams) -> usize {
        let NodeWithDepth { i: parent_i, depth } = thread_local_peek_parent();

        // todo: make build_new_node and update_node take the same params and pack it together

        let frame = self.sys.part.current_frame;

        // Check the node corresponding to the key's id.
        // We might find that the key has already been used in this same frame:
        //      in this case, we take note, and calculate a twin key to use to add a "twin" in the next section.
        // Otherwise, we add or refresh normally, and take note of the final i.
        let twin_check_result = match self.nodes.node_hashmap.entry(key.id()) {
            // Add a normal node (no twins).
            Entry::Vacant(v) => {
                let new_node = self.sys.build_new_node(&key, params, None, depth);
                let final_i = self.nodes.nodes.insert(new_node);
                v.insert(NodeMapEntry::new(parent_i, frame, final_i));

                UpdatedNormal { final_i }
            }
            Entry::Occupied(o) => {
                let old_map_entry = o.into_mut();

                match refresh_or_add_twin(frame, old_map_entry.last_frame_touched) {
                    // Refresh a normal node from the previous frame (no twins).
                    Refresh => {
                        old_map_entry.refresh(parent_i, frame);
                        // todo2: check the map_entry values and maybe skip reaching into the node
                        let final_i = old_map_entry.slab_i;
                        self.update_node(params, final_i, frame);

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
                        let new_twin_node =
                            self.sys.build_new_node(&twin_key, params, Some(twin_n), depth);
                        let real_final_i = self.nodes.nodes.insert(new_twin_node);
                        v.insert(NodeMapEntry::new(parent_i, frame, real_final_i));
                        (real_final_i, twin_key.id())
                    }
                    // Refresh a twin from the previous frame.
                    Entry::Occupied(o) => {
                        let old_twin_map_entry = o.into_mut();

                        // todo2: check the map_entry values and maybe skip reaching into the node
                        let real_final_i = old_twin_map_entry.refresh(parent_i, frame);

                        self.update_node(params, real_final_i, frame);
                        (real_final_i, twin_key.id())
                    }
                }
            }
        };

        let children_hash_so_far = thread_local_hash_new_child(real_final_i);
        self.nodes[parent_i].old_children_hash = children_hash_so_far;

        self.set_relayout_chain_root(params, real_final_i, parent_i);

        self.set_tree_links(real_final_i, parent_i);

        return real_final_i;
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

    fn set_tree_links(&mut self, new_node_i: usize, parent_i: usize) {
        self.nodes[new_node_i].parent = parent_i;

        self.nodes[parent_i].n_children += 1;

        // todo: maybe merge reset_children into this to get big premature optimization points 
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

    pub fn text(&mut self, text: &str) -> UiNode<Any> {
        self.add(&TEXT).text(text)
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
        // would definitely be very cringe for anyone doing custom rendering. but not really. nobody will ever want to stick his custom rendered stuff between a rectangle and another. when custom rendering INSIDE a rectangle, the user can get the z every time. might be annoying (very annoying even) but not deal breaking.

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

    fn update_node(&mut self, params: &NodeParams, node_i: usize, frame: u64) {
        self.watch_params_change(node_i, self.nodes[node_i].params, *params);
        
        let node = &mut self.nodes[node_i];

        node.params = *params;

        // we don't do this anymore, cuz we do it in set_tree_links
        // node.parent = parent_id;

        // dunno why this is commented now, I guess it's done somewhere else
        // self.last_frame_touched = frame;
        
        node.reset_children();

        self.sys.text.refresh_last_frame(node.text_id, frame);
    }

    pub fn get_node(&mut self, key: TypedKey<Any>) -> Option<UiNode<Any>> {
        let real_key = self.get_latest_twin_key(key)?;
        return self.get_ref(real_key);
    }

    fn set_relayout_chain_root(&mut self, new_node_params: &NodeParams, new_node_i: usize, parent_i: usize) {
        match self.nodes[parent_i].relayout_chain_root {
            Some(root_of_parent) => match new_node_params.is_fit_content() {
                true => self.nodes[new_node_i].relayout_chain_root = Some(root_of_parent), // continue chain
                false => self.nodes[new_node_i].relayout_chain_root = None, // break chain
            },
            None => match new_node_params.is_fit_content() {
                true => self.nodes[new_node_i].relayout_chain_root = Some(new_node_i), // start chain
                false => self.nodes[new_node_i].relayout_chain_root = None, // do nothing
            },
        };
    }

    fn push_partial_relayout(&mut self, node_i: usize) {
        let relayout_target_i = match self.nodes[node_i].relayout_chain_root {
            Some(root) => root,
            None => node_i,
        };

        // even after the chain, we still have to go one layer up, because a different sized child probably means that the parent wants to place the node differently, and maybe pick a different size and position for the other children as well
        // In practice, the first half of that is basically always true, but the second half is only true for Stacks. I don't really feel like adding a distinction for that right now.
        let relayout_target_parent = self.nodes[relayout_target_i].parent;

        let relayout_entry = NodeWithDepth {
            i: relayout_target_parent,
            depth: self.nodes[relayout_target_parent].depth,
        };
        self.sys.changes.partial_relayouts.push(relayout_entry);
    }

    fn push_cosmetic_rect_update(&mut self, node_i: usize) {
        // no chains here.
        self.sys.changes.cosmetic_rect_updates.push(node_i);
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

        self.nodes[self.sys.root_i].reset_children();
    }
}

#[derive(Debug)]
pub struct Node {
    pub id: Id,
    pub depth: usize,

    pub last_layout_frame: u64,

    // also for invisible rects, used for layout
    pub rect: XyRect,

    // partial result when layouting?
    // in probably in fraction of screen units or some trash 
    pub size: Xy<f32>,

    relayout_chain_root: Option<usize>,

    pub(crate) last_rect_i: usize,

    pub text_id: Option<usize>,

    pub imageref: Option<ImageRef>,
    pub last_static_image_ptr: Option<*const u8>,
    pub last_static_text_ptr: Option<*const u8>,

    pub parent: usize,
    pub old_parent: usize,

    // le epic inline linked list instead of a random Vec somewhere else on the heap
    // todo: Option<usize> is 128 bits, which is ridicolous. Use a NonMaxU32 or something
    pub n_children: u16,

    pub old_first_child: Option<usize>,
    pub old_prev_sibling: Option<usize>,
    pub last_child: Option<usize>,
    pub prev_sibling: Option<usize>,
    // prev_sibling is never used so far.
    // at some point I was iterating the children in reverse for z ordering purposes, but I don't think that actually makes any difference.
    // pub prev_sibling: Option<usize>,
    pub params: NodeParams,

    pub debug_name: &'static str,

    pub old_children_hash: u64,

    pub is_twin: Option<u32>,

    pub last_hover: f32,
    pub last_click: f32,
    pub z: f32,
}
impl Node {
    pub fn debug_name(&self) -> String {
        let debug_name = match self.is_twin {
            Some(n) => format!("{} (twin #{})", self.debug_name, n),
            None => self.debug_name.to_string(),
        };
        return debug_name;
    }

    fn reset_children(&mut self) {
        // keep these ones so we can detect changes
        self.old_parent = self.parent;
        self.old_first_child = self.last_child;
        self.old_prev_sibling = self.prev_sibling;

        self.last_child = None;
        self.prev_sibling = None;
        // self.prev_sibling = None;
        self.n_children = 0;
    }

    pub fn render_rect(&self, draw_even_if_invisible: bool) -> Option<RenderRect> {
        if ! draw_even_if_invisible && ! self.params.rect.visible {
            return None;
        }

        let mut flags = RenderRect::EMPTY_FLAGS;
        if self.params.interact.click_animation {
            flags |= RenderRect::CLICK_ANIMATION;
        }
        if self.params.rect.outline_only {
            flags |= RenderRect::OUTLINE_ONLY;
        }

        return Some(RenderRect {
            rect: self.rect.to_graphics_space(),
            vertex_colors: self.params.rect.vertex_colors,
            last_hover: self.last_hover,
            last_click: self.last_click,
            id: self.id,
            z: 0.0,
            radius: BASE_RADIUS,

            // magic coords
            // todo: demagic
            tex_coords: Xy {
                x: [0.9375, 0.9394531],
                y: [0.00390625 / 2.0, 0.0],
            },
            flags,
            _padding: 0,
        })
    }

    pub fn image_rect(&self) -> Option<RenderRect> {
        let mut image_flags = RenderRect::EMPTY_FLAGS;
        if self.params.interact.click_animation {
            image_flags |= RenderRect::CLICK_ANIMATION;
        }

        if let Some(image) = self.imageref {
            // in debug mode, draw invisible rects as well.
            // usually these have filled = false (just the outline), but this is not enforced.

            return Some(RenderRect {
                rect: self.rect.to_graphics_space(),
                vertex_colors: self.params.rect.vertex_colors,
                last_hover: self.last_hover,
                last_click: self.last_click,
                id: self.id,
                z: 0.0,
                radius: BASE_RADIUS,

                tex_coords: image.tex_coords,
                flags: image_flags,
                _padding: 0,
            });
        }

        return None;
    }

}

use std::hash::Hash;
fn fx_hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = FxHasher::default();
    value.hash(&mut hasher);
    hasher.finish()
}

pub struct Parent {
    pub(crate) node_i: usize,
    pub(crate) old_children_hash: u64,
}
impl Parent {
    pub(crate) fn new(node_i: usize, old_children_hash: u64) -> Parent {
        return Parent {
            node_i,
            old_children_hash,
        }
    }

    pub fn nest(&self, children_block: impl FnOnce()) {
        thread_local_push(self);

        children_block();

        thread_local_pop();
    }
}

impl<'a, T: NodeType> UiNode<'a, T> {
    pub fn parent(&self) -> Parent {
        let old_children_hash = self.node().old_children_hash;
        return Parent::new(self.node_i, old_children_hash);
    }
}

impl Ui {
    pub fn add(&mut self, params: &NodeParams) -> UiNode<Any> {
        let i = self.add_or_update_node(params.key, params);
        return self.get_ref_unchecked(i, &params.key);
    }

    pub fn add_parent(&mut self, params: &NodeParams) -> Parent {
        let node = self.add_or_update_node(params.key, params);
        let old_children_hash = self.nodes[node].old_children_hash;
        return Parent::new(node, old_children_hash);
    }

    pub fn v_stack(&mut self) -> Parent {
        let node = self.add_or_update_node(ANON_VSTACK, &V_STACK);
        let old_children_hash = self.nodes[node].old_children_hash;
        return Parent::new(node, old_children_hash);
    }

    pub fn h_stack(&mut self) -> Parent {
        let node = self.add_or_update_node(ANON_HSTACK, &H_STACK);
        let old_children_hash = self.nodes[node].old_children_hash;
        return Parent::new(node, old_children_hash);
    }
}
