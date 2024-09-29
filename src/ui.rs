use crate::ui_node_params::{ANON_HSTACK, ANON_VSTACK, DEFAULT, H_STACK, NODE_ROOT_PARAMS, TEXT, V_STACK};
use crate::ui_render::TypedGpuBuffer;
use crate::ui_texture_atlas::{ImageRef, TextureAtlas};
use copypasta::ClipboardContext;
use glyphon::cosmic_text::{Align, StringCursor};
use glyphon::{AttrsList, Color as GlyphonColor, TextBounds};
use rustc_hash::{FxHashMap, FxHasher};
use slab::Slab;
use wgpu::*;
use winit::event::{ElementState, MouseScrollDelta};
use crate::ui_math::*;

use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::sync::LazyLock;
use std::{
    hash::Hasher,
    marker::PhantomData,
    mem,
    ops::{Index, IndexMut},
    time::Instant,
};

use bytemuck::{Pod, Zeroable};
use glyphon::{
    Attrs, Buffer as GlyphonBuffer, Family, FontSystem, Metrics, Shaping,
    SwashCache, TextArea, TextAtlas, TextRenderer,
};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{MouseButton, WindowEvent},
    keyboard::ModifiersState,
};
use Axis::{X, Y};
use {
    util::{self, DeviceExt},
    vertex_attr_array, BindGroup, BufferAddress, BufferUsages, ColorTargetState, Device,
    MultisampleState, Queue, RenderPipeline, SurfaceConfiguration, VertexAttribute,
    VertexBufferLayout, VertexStepMode,
};

static T0: LazyLock<Instant> = LazyLock::new(Instant::now);
pub fn ui_time_f32() -> f32 {
    return T0.elapsed().as_secs_f32();
}

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, Pod, Zeroable)]
#[repr(C)]
pub struct Id(pub(crate) u64);

pub const NODE_ROOT_ID: Id = Id(0);
pub const NODE_ROOT: Node = Node {
    id: NODE_ROOT_ID,
    rect: Xy::new_symm([0.0, 1.0]),
    size: Xy::new_symm(1.0),
    text_id: None,

    imageref: None,
    last_static_image_ptr: None,

    parent: usize::MAX,

    n_children: 0,
    first_child: None,
    next_sibling: None,
    is_twin: None,

    params: NODE_ROOT_PARAMS,
    debug_name: "Root",
    last_frame_status: LastFrameStatus::Nothing,
    last_hover: f32::MIN,
    last_click: f32::MIN,
    z: -10000.0,
};

// might as well move to Rect? but maybe there's issues with non-clickable stuff absorbing the clicks.
#[derive(Debug, Copy, Clone)]
pub struct Interact {
    pub click_animation: bool,
    pub absorbs_mouse_events: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct Layout {
    pub size: Xy<Size>,
    pub padding: Xy<Len>,
    pub position: Xy<Position>,
}


#[derive(Debug, Copy, Clone)]
pub struct Rect {
    pub visible: bool,
    pub filled: bool,
    pub vertex_colors: VertexColors,
    // ... crazy stuff like texture and NinePatchRect
}
impl Rect {
    pub const DEFAULT: Self = Self {
        visible: true,
        filled: true,
        vertex_colors: VertexColors::flat(Color::FLGR_BLUE),
    };
}

// rename
// todo: add greyed text for textinput
#[derive(Debug, Copy, Clone)]
pub struct TextOptions {
    pub editable: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct Image<'data> {
    pub data: &'data [u8],
}

#[derive(Debug, Copy, Clone)]
pub struct NodeParams {
    pub text_params: Option<TextOptions>,
    pub stack: Option<Stack>,
    pub rect: Rect,
    pub interact: Interact,
    pub layout: Layout,
    pub key: NodeKey,
}

pub const RADIUS: f32 = 20.0;

impl NodeParams {
    pub const fn const_default() -> Self {
        return DEFAULT;
    }

    pub const fn key(mut self, key: NodeKey) -> Self {
        self.key = key;
        return self;
    }

    // todo: in a future version of Rust that allows it, change these to take a generic Into<Size>
    pub const fn size_x(mut self, size: Size) -> Self {
        self.layout.size.x = size;
        return self;
    }
    pub const fn size_y(mut self, size: Size) -> Self {
        self.layout.size.y = size;
        return self;
    }
    pub const fn size_symm(mut self, size: Size) -> Self {
        self.layout.size = Xy::new_symm(size);
        return self;
    }

    pub const fn position_x(mut self, position: Position) -> Self {
        self.layout.position.x = position;
        return self;
    }
    pub const fn position_y(mut self, position: Position) -> Self {
        self.layout.position.y = position;
        return self;
    }
    pub const fn position_symm(mut self, position: Position) -> Self {
        self.layout.position = Xy::new_symm(position);
        return self;
    }

    pub const fn visible(mut self) -> Self {
        self.rect.visible = true;
        return self;
    }
    pub const fn invisible(mut self) -> Self {
        self.rect.visible = false;
        self.rect.filled = false;
        self.rect.vertex_colors = VertexColors::flat(Color::FLGR_DEBUG_RED);
        return self;
    }

    pub const fn filled(mut self, filled: bool) -> Self {
        self.rect.filled = filled;
        return self;
    }
    
    pub const fn color(mut self, color: Color) -> Self {
        self.rect.vertex_colors = VertexColors::flat(color);
        return self;
    }

    pub const fn vertex_colors(mut self, colors: VertexColors) -> Self {
        self.rect.vertex_colors = colors;
        return self;
    }

    pub const fn stack(mut self, axis: Axis, arrange: Arrange, spacing: Len) -> Self {
        self.stack = Some(Stack {
            arrange, axis, spacing,
        });
        return self;
    }

    pub const fn stack_arrange(mut self, arrange: Arrange) -> Self {
        let stack = match self.stack {
            Some(stack) => stack,
            None => Stack::DEFAULT,
        };
        self.stack = Some(stack.arrange(arrange));
        return self;
    }
    
    pub const fn stack_spacing(mut self, spacing: Len) -> Self {
        let stack = match self.stack {
            Some(stack) => stack,
            None => Stack::DEFAULT,
        };
        self.stack = Some(stack.spacing(spacing));
        return self;
    }

    // todo: if we don't mind sacrificing symmetry, it could make sense to just remove this one. 
    pub const fn stack_axis(mut self, axis: Axis) -> Self {
        let stack = match self.stack {
            Some(stack) => stack,
            None => Stack::DEFAULT,
        };
        self.stack = Some(stack.axis(axis));
        return self;
    }

    pub const fn padding(mut self, padding: Len) -> Self {
        self.layout.padding = Xy::new_symm(padding);
        return self;
    }

    pub const fn padding_x(mut self, padding: Len) -> Self {
        self.layout.padding.x = padding;
        return self;
    }

    pub const fn padding_y(mut self, padding: Len) -> Self {
        self.layout.padding.x = padding;
        return self;
    }

    pub const fn absorbs_clicks(mut self, absorbs_clicks: bool) -> Self {
        self.interact.absorbs_mouse_events = absorbs_clicks;
        return self;
    }
}

#[derive(Default, Debug, Pod, Copy, Clone, Zeroable)]
#[repr(C)]
// todo: could do some epic SOA stuff to make resolve_mouse_input and friends faster
// Layout has to match the one in the shader.
pub struct RenderRect {
    pub rect: XyRect,
    // this isn't it for images, but i'll keep it for future tiling textures and ninepatchrects
    pub tex_coords: XyRect,

    pub vertex_colors: VertexColors,

    pub last_hover: f32,
    pub last_click: f32,
    pub click_animation: u32,
    pub z: f32,

    pub radius: f32,

    pub filled: u32,
    pub id: Id,
}
impl RenderRect {
    pub fn buffer_desc() -> [VertexAttribute; 15] {
        return vertex_attr_array![
            // xyrect
            0 => Float32x2,
            1 => Float32x2,
            // tex coords
            2 => Float32x2,
            3 => Float32x2,
            // colors
            4 => Uint8x4,
            5 => Uint8x4,
            6 => Uint8x4,
            7 => Uint8x4,
            // last hover
            8 => Float32,
            // last click
            9 => Float32,
            // clickable
            10 => Uint32,
            // z
            11 => Float32,
            12 => Float32,
            // filled
            13 => Uint32,
            // radius
            14 => Uint32,
        ];
    }
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
    pub const FLGR_SOVL_GRAD: Self = VertexColors::diagonal_gradient_backslash(Color::FLGR_BLUE, Color::FLGR_RED);

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
        }
    }

    pub const fn flat(color: Color) -> VertexColors {
        return VertexColors::new(color, color, color, color)
    }

    pub const fn h_gradient(left: Color, right: Color) -> VertexColors {
        return VertexColors::new(left, right, left, right)
    }

    pub const fn v_gradient(top: Color, bottom: Color) -> VertexColors {
        return VertexColors::new(top, top, bottom, bottom)
    }

    // techinically, the blended corners shouldn't be blended with weight 0.5. The weight should depend on the aspect ratio, I think. I don't think that's practical though, and it looks okay like this. 
    pub const fn diagonal_gradient_forward_slash(bottom_left: Color, top_right: Color) -> VertexColors {
        let blended = bottom_left.blend(top_right, 255 / 2);
        return VertexColors {
            top_left: blended,
            top_right,
            bottom_left,
            bottom_right: blended,
        }
    }

    pub const fn diagonal_gradient_backslash(top_left: Color, bottom_right: Color) -> VertexColors {
        let blended = top_left.blend(bottom_right, 255 / 2);
        return VertexColors {
            top_left,
            top_right: blended,
            bottom_left: blended,
            bottom_right,
        }
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
    pub(crate) node: usize,
    pub(crate) ui: &'a mut Ui,
    pub(crate) nodetype_marker: PhantomData<T>,
}

// why can't you just do it separately?
impl<'a,  T: NodeType> UiNode<'a, T> {

    pub fn node_mut(&mut self) -> &mut Node {
        return &mut self.ui.nodes.nodes[self.node];
    }
    pub fn node(&self) -> &Node {
        return &self.ui.nodes.nodes[self.node];
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

    pub fn set_color(&mut self, color: Color)  -> &mut Self {
        self.node_mut().params.rect.vertex_colors = VertexColors::flat(color);
        return self;
    }

    pub fn set_vertex_colors(&mut self, colors: VertexColors)  -> &mut Self {
        self.node_mut().params.rect.vertex_colors = colors;
        return self;
    }

    pub fn set_position_x(&mut self, position: Position)  -> &mut Self {
        self.node_mut().params.layout.position.x = position;
        return self;
    }

    pub fn set_position_y(&mut self, position: Position)  -> &mut Self {
        self.node_mut().params.layout.position.y = position;
        return self;
    }

    pub fn set_size_x(&mut self, size: Size)  -> &mut Self {
        self.node_mut().params.layout.size.x = size;
        return self;
    }

    pub fn set_size_y(&mut self, size: Size)  -> &mut Self {
        self.node_mut().params.layout.size.y = size;
        return self;
    }
}

impl<'a, T: TextTrait> UiNode<'a, T> {

    pub fn text(mut self, text: &str) -> Self {
        
        if let Some(text_id) = self.node_mut().text_id {
            self.ui.sys.text.set_text_hashed(text_id, text);
        } else {
            let text_id = self.ui.sys.text.maybe_new_text_area(Some(text), self.ui.sys.part.current_frame);
            self.node_mut().text_id = text_id;
        }

        return self;
    }

    pub fn set_text_attrs(&mut self, attrs: Attrs)  -> &mut Self {
        if let Some(text_id) = self.node_mut().text_id {
            self.ui.sys.text.set_text_attrs(text_id, attrs);
        } else {
            // todo: log a warning or something
            // or make these things type safe somehow
        }
        return self;
    }

    pub fn set_text_align(&mut self, align: Align)  -> &mut Self {
        if let Some(text_id) = self.node_mut().text_id {
            self.ui.sys.text.set_text_align(text_id, align);
        } else {
            // todo: log a warning or something
            // or make these things type safe somehow
        }
        return self;
    }

    pub fn get_text(&self) -> Option<String> {
        let text_id = self.node().text_id.unwrap();

        let text = self.ui.sys.text.text_areas[text_id].buffer.lines[0]
            .text
            .text()
            .to_string();
        return Some(text);
    }
}

pub struct PartialBorrowStuff {
    pub mouse_pos: PhysicalPosition<f32>,
    pub unifs: Uniforms,
    pub current_frame: u64,
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
    pub text_areas: Vec<TextArea>,
}
const GLOBAL_TEXT_METRICS: Metrics = Metrics::new(24.0, 24.0);
impl TextSystem {
    pub(crate) fn maybe_new_text_area(&mut self, text: Option<&str>, current_frame: u64) -> Option<usize> {
        let text = match text {
            Some(text) => text,
            None => return None,
        };

        let mut buffer = GlyphonBuffer::new(&mut self.font_system, GLOBAL_TEXT_METRICS);
        buffer.set_size(&mut self.font_system, 500., 500.);

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

        let text_area = TextArea {
            buffer,
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
            depth: 0.0,
            last_frame_touched: current_frame,
            last_hash: hash,
        };
        self.text_areas.push(text_area);
        let text_id = self.text_areas.len() - 1;

        return Some(text_id);
    }

    fn refresh_last_frame(&mut self, text_id: Option<usize>, current_frame: u64) {
        if let Some(text_id) = text_id {
            self.text_areas[text_id].last_frame_touched = current_frame;
        }
    }

    fn set_text_hashed(&mut self, text_id: usize, text: &str) {
        let hash = fx_hash(&text);
        let area = &mut self.text_areas[text_id];
        if hash != area.last_hash {
            area.last_hash = hash;
            area.buffer.set_text(
                &mut self.font_system,
                text,
                Attrs::new().family(Family::SansSerif),
                Shaping::Advanced,
            );
        }
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
        }
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
    pub fn build_new_node<T: NodeType>(&mut self, key: &TypedKey<T>, params: &NodeParams, twin_n: Option<u32>) -> Node {
        let parent_i = thread_local_last_parent();

        // add back somewhere

        return Node {
            id: key.id(),
            rect: Xy::new_symm([0.0, 1.0]),
            size: Xy::new_symm(10.0),
            text_id: None,

            imageref: None,
            last_static_image_ptr: None,
            
            parent: parent_i,

            n_children: 0,
            first_child: None,
            next_sibling: None,
            is_twin: twin_n,
            params: *params,
            debug_name: key.debug_name,
            last_frame_status: LastFrameStatus::Nothing,
            last_hover: f32::MIN,
            last_click: f32::MIN,
            z: 0.0,
        }
    
    }
}

// todo: the sys split is no longer needed, lol.
pub struct Ui {
    pub nodes: Nodes,
    pub sys: System,
}

pub struct System {
    pub root_i: usize,
    pub debug_mode: bool,
    pub debug_key_pressed: bool,

    pub mouse_status: MouseInputState,

    pub waiting_for_click_release: bool,

    pub clipboard: ClipboardContext,

    pub key_mods: ModifiersState,

    pub gpu_vertex_buffer: TypedGpuBuffer<RenderRect>,
    pub render_pipeline: RenderPipeline,

    pub base_uniform_buffer: Buffer,
    pub bind_group: BindGroup,

    pub text: TextSystem,
    pub texture_atlas: TextureAtlas,

    pub rects: Vec<RenderRect>,
    
    // stack for traversing
    pub traverse_stack: Vec<usize>,

    pub part: PartialBorrowStuff,

    pub clicked_stack: Vec<(Id, f32)>,
    pub mouse_hit_stack: Vec<(Id, f32)>,
    pub clicked: Vec<Id>,
    pub hovered: Vec<Id>,

    pub focused: Option<Id>,

    pub size_scratch: Vec<f32>,

    // todo: add these back sometime. probably better to have relayout_needed, rerender_needed, etc instead of some vaguely named trash
    // // remember about animations (surely there will be)
    // pub content_changed: bool,
    // pub tree_changed: bool,
    pub frame_t: f32,
}
impl Ui {

    // todo: variadic args lol
    pub fn add_widget<T, S>(&mut self, widget_function: fn(&mut Ui, T) -> S, arg: T) -> S {
        return widget_function(self, arg);
    }

    pub fn new(device: &Device, queue: &Queue, config: &SurfaceConfiguration) -> Self {
        let vertex_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("player bullet pos buffer"),
            contents: bytemuck::cast_slice(&[0.0; 9000]),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let vertex_buffer = TypedGpuBuffer::new(vertex_buffer);
        let vert_buff_layout = VertexBufferLayout {
            array_stride: mem::size_of::<RenderRect>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &RenderRect::buffer_desc(),
        };

        let uniforms = Uniforms {
            size: Xy::new(config.width as f32, config.height as f32),
            t: 0.,
            _padding: 0.,
        };
        let resolution_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: Some("Resolution Uniform Buffer"),
            contents: bytemuck::bytes_of(&uniforms),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let mut texture_atlas = TextureAtlas::new(device);

        let _white_alloc = texture_atlas.allocate_image(include_bytes!("white.png"));

        let texture_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Fulgur texture sampler"),
            min_filter: FilterMode::Nearest,
            mag_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: 0f32,
            lod_max_clamp: 0f32,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("Fulgur Bind Group Layout"),
        });

        // Create the bind group
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: resolution_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(texture_atlas.texture_view()),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&texture_sampler),
                },
            ],
            label: Some("Fulgur Bind Group"),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(include_str!("shaders/box.wgsl").into()),
        });

        let primitive = PrimitiveState::default();

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[vert_buff_layout],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive,
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
        });

        let font_system = FontSystem::new();
        let cache = SwashCache::new();
        let mut atlas = TextAtlas::new(device, queue, config.format);
        let text_renderer =
            TextRenderer::new(&mut atlas, device, MultisampleState::default(), None);

        let text_areas = Vec::with_capacity(50);

        let mut node_hashmap = FxHashMap::with_capacity_and_hasher(100, Default::default());
        
        let mut nodes = Slab::with_capacity(100);
        let root_i = nodes.insert(NODE_ROOT);
        let root_map_entry = NodeMapEntry {
            last_parent: usize::default(),
            last_frame_touched: u64::MAX,
            slab_i: root_i,
            n_twins: 0,
        };
        
        thread_local_push_parent(root_i);

        node_hashmap.insert(NODE_ROOT_ID, root_map_entry);

        let nodes = Nodes {
            node_hashmap,
            nodes,
        };

        Self {

            nodes,

            sys: System {

                root_i,
                waiting_for_click_release: false,
                debug_mode: false,
                debug_key_pressed: false,

                mouse_status: MouseInputState::default(),

                clipboard: ClipboardContext::new().unwrap(),
                key_mods: ModifiersState::default(),

                text: TextSystem {
                    cache,
                    atlas,
                    text_renderer,
                    font_system,
                    text_areas,
                },

                texture_atlas,

                render_pipeline,
                rects: Vec::with_capacity(20),



                gpu_vertex_buffer: vertex_buffer,
                base_uniform_buffer: resolution_buffer,
                bind_group,

                traverse_stack: Vec::with_capacity(50),

                size_scratch: Vec::with_capacity(15),

                part: PartialBorrowStuff {
                    mouse_pos: PhysicalPosition { x: 0., y: 0. },
                    current_frame: 1,
                    unifs: uniforms,
                },

                clicked_stack: Vec::with_capacity(50),
                mouse_hit_stack: Vec::with_capacity(50),
                clicked: Vec::with_capacity(15),
                hovered: Vec::with_capacity(15),
                focused: None,

                frame_t: 0.0,
            }

        }
    }

    // don't expect this to give you twin nodes automatically
    pub fn get_ref<T: NodeType>(&mut self, key: TypedKey<T>) -> UiNode<Any> {
        let node_i = self.nodes.node_hashmap.get(&key.id()).unwrap().slab_i;
        return self.get_ref_unchecked(node_i, &key)
    }

    // only for the macro, use get_ref 
    pub fn get_ref_unchecked<T: NodeType>(&mut self, i: usize, _key: &TypedKey<T>) -> UiNode<Any> {        
        return UiNode {
            node: i,
            ui: self,
            nodetype_marker: PhantomData::<Any>,
        };
    }

    pub fn update_node<T: NodeType>(&mut self, key: TypedKey<T>, params: &NodeParams) -> usize {
        let parent_i = thread_local_last_parent();

        let frame = self.sys.part.current_frame;

        // Check the node corresponding to the key's id.
        // We might find that the key has already been used in this same frame: 
        //      in this case, we take note, and calculate a twin key to use to add a "twin" in the next section.
        // Otherwise, we add or refresh normally, and take note of the final i.
        let twin_check_result = match self.nodes.node_hashmap.entry(key.id()) {
            // Add a normal node (no twins).
            Entry::Vacant(v) => {

                let new_node = self.sys.build_new_node(&key, params, None);
                let final_i = self.nodes.nodes.insert(new_node);
                v.insert(NodeMapEntry::new(parent_i, frame, final_i));

                UpdatedNormal{ final_i }
            },
            Entry::Occupied(o) => {
                let old_map_entry = o.into_mut();
                
                match refresh_or_add_twin(frame, old_map_entry.last_frame_touched) {
                    // Refresh a normal node from the previous frame (no twins).
                    Refresh => {
                        old_map_entry.refresh(parent_i, frame);
                        // todo2: check the map_entry values and maybe skip reaching into the node
                        let final_i = old_map_entry.slab_i;
                        self.refresh_node(params, final_i, parent_i, frame);
                        
                        UpdatedNormal{ final_i }
                    }
                    // do nothing, just calculate the twin key and go to twin part below
                    AddTwin => {
                        old_map_entry.n_twins += 1;
                        let twin_key = key.sibling(old_map_entry.n_twins);
                        NeedToUpdateTwin { twin_key, twin_n: old_map_entry.n_twins }
                    }
                }

            },
        };

        // If twin_check_result is AddedNormal, the node was added in the section before, 
        //      and there's nothing to do regarding twins, so we just confirm final_i.
        // If it's NeedToAddTwin, we repeat the same thing with the new twin_key.
        let real_final_i = match twin_check_result {
            UpdatedNormal { final_i } => final_i,
            NeedToUpdateTwin { twin_key, twin_n } => {
                match self.nodes.node_hashmap.entry(twin_key.id()) {
                    // Add new twin.
                    Entry::Vacant(v) => {
                        let new_twin_node = self.sys.build_new_node(&twin_key, params, Some(twin_n));
                        let real_final_i = self.nodes.nodes.insert(new_twin_node);
                        v.insert(NodeMapEntry::new(parent_i, frame, real_final_i));
                        real_final_i
                    },
                    // Refresh a twin from the previous frame.
                    Entry::Occupied(o) => {
                        let old_twin_map_entry = o.into_mut();
    
                        // todo2: check the map_entry values and maybe skip reaching into the node
                        let real_final_i = old_twin_map_entry.refresh(parent_i, frame);
                        
                        self.refresh_node(params, real_final_i, parent_i, frame);
                        real_final_i
                    },
    
                }
            },
        };

        self.add_child_to_parent(real_final_i, parent_i);

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

    pub fn add_child_to_parent(&mut self, id: usize, parent_id: usize) {
        self.nodes[parent_id].n_children += 1;

        if self.nodes[parent_id].first_child.is_none() {
            self.nodes[parent_id].first_child = Some(id);

            thread_local_push_last_sibling(id);

        } else {
            let prev_sibling = thread_local_cycle_last_sibling(id);
            self.nodes[prev_sibling].next_sibling = Some(id);
        }

    }

    pub fn text(&mut self, text: &str) -> UiNode<Any> {
        self.add(&TEXT).text(text)
    }


    pub fn resize(&mut self, size: &PhysicalSize<u32>, queue: &Queue) {
        self.sys.part.unifs.size[X] = size.width as f32;
        self.sys.part.unifs.size[Y] = size.height as f32;

        queue.write_buffer(
            &self.sys.base_uniform_buffer,
            0,
            &bytemuck::bytes_of(&self.sys.part.unifs)[..16],
        );
    }

    pub fn update_time(&mut self) {
        self.sys.frame_t = ui_time_f32();
    }

    pub fn build_rect(&mut self, node: usize) {
        let current_node = &self.nodes.nodes[node];

        // in debug mode, draw invisible rects as well.
        // usually these have filled = false (just the outline), but this is not enforced.
        if current_node.params.rect.visible || self.sys.debug_mode {
            self.sys.rects.push(RenderRect {
                rect: current_node.rect.to_graphics_space(),
                vertex_colors: current_node.params.rect.vertex_colors,
                last_hover: current_node.last_hover,
                last_click: current_node.last_click,
                click_animation: current_node.params.interact.click_animation.into(),
                id: current_node.id,
                z: 0.0,
                radius: RADIUS,
                filled: current_node.params.rect.filled as u32,

                // magic coords
                // todo: demagic
                tex_coords: Xy { x: [0.9375, 0.9394531], y: [0.00390625 / 2.0, 0.0] },
            });
        }
    }

    pub fn push_cursor_rect(&mut self) -> Option<()> {
        // cursor
        // how to make it appear at the right z? might be impossible if there are overlapping rects at the same z.
        // one epic way could be to increase the z sequentially when rendering, so that all rects have different z's, so the cursor can have the z of its rect plus 0.0001.
        // would definitely be very cringe for anyone doing custom rendering. but not really. nobody will ever want to stick his custom rendered stuff between a rectangle and another. when custom rendering INSIDE a rectangle, the user can get the z every time. might be annoying (very annoying even) but not deal breaking.

        // it's a specific choice by me to keep cursors for every string at all times, but only display (and use) the one on the currently focused ui node.
        // someone might want multi-cursor in the same node, multi-cursor on different nodes, etc.
        let focused_id = &self.sys.focused?;
        let focused_node = self.nodes.get_by_id(focused_id)?;
        let text_id = focused_node.text_id?;
        let focused_text_area = self.sys.text.text_areas.get(text_id)?;

        match focused_text_area.buffer.lines[0].text.cursor() {
            StringCursor::Point(cursor) => {
                let rect_x0 = focused_node.rect[X][0];
                let rect_y1 = focused_node.rect[Y][1];

                let (x, y) = cursor_pos_from_byte_offset(&focused_text_area.buffer, *cursor);

                let cursor_width = focused_text_area.buffer.metrics().font_size / 20.0;
                let cursor_height = focused_text_area.buffer.metrics().font_size;
                // we're counting on this always happening after layout. which should be safe.
                let x0 = ((x - 1.0) / self.sys.part.unifs.size[X]) * 2.0;
                let x1 = ((x + cursor_width) / self.sys.part.unifs.size[X]) * 2.0;
                let x0 = x0 + (rect_x0 * 2. - 1.);
                let x1 = x1 + (rect_x0 * 2. - 1.);

                let y0 = ((-y - cursor_height) / self.sys.part.unifs.size[Y]) * 2.0;
                let y1 = ((-y) / self.sys.part.unifs.size[Y]) * 2.0;
                let y0 = y0 + (rect_y1 * 2. - 1.);
                let y1 = y1 + (rect_y1 * 2. - 1.);

                let cursor_rect = RenderRect {
                    rect: XyRect::new([x0, x1], [y0, y1]),
                    vertex_colors: VertexColors::flat(Color::rgba(128, 77, 128, 230)),
                    last_hover: 0.0,
                    last_click: 0.0,
                    click_animation: 0,
                    z: 0.0,
                    id: Id(0),
                    filled: 1,
                    radius: 0.0,
                    tex_coords: Xy::new([0.0, 0.0], [0.0, 0.0]),

                };

                self.sys.rects.push(cursor_rect);
            }
            StringCursor::Selection(selection) => {
                let rect_x0 = focused_node.rect[X][0];
                let rect_y1 = focused_node.rect[Y][1];

                let (x0, y0) =
                    cursor_pos_from_byte_offset(&focused_text_area.buffer, selection.start);
                let (x1, y1) =
                    cursor_pos_from_byte_offset(&focused_text_area.buffer, selection.end);

                // let cursor_width = focused_text_area.buffer.metrics().font_size / 20.0;
                let cursor_height = focused_text_area.buffer.metrics().font_size;
                let x0 = ((x0 - 1.0) / self.sys.part.unifs.size[X]) * 2.0;
                let x1 = ((x1 + 1.0) / self.sys.part.unifs.size[X]) * 2.0;
                let x0 = x0 + (rect_x0 * 2. - 1.);
                let x1 = x1 + (rect_x0 * 2. - 1.);

                let y0 = ((-y0 - cursor_height) / self.sys.part.unifs.size[Y]) * 2.0;
                let y1 = ((-y1) / self.sys.part.unifs.size[Y]) * 2.0;
                let y0 = y0 + (rect_y1 * 2. - 1.);
                let y1 = y1 + (rect_y1 * 2. - 1.);

                let cursor_rect = RenderRect {
                    rect: XyRect::new([x0, x1], [y0, y1]),
                    vertex_colors: VertexColors::flat(Color::rgba(128, 77, 128, 230)),
                    last_hover: 0.0,
                    last_click: 0.0,
                    click_animation: 0,
                    z: 0.0,
                    id: Id(0),
                    filled: 1,
                    radius: 0.0,

                    tex_coords: Xy::new([0.0, 0.0], [0.0, 0.0]),
                };

                self.sys.rects.push(cursor_rect);
            }
        }

        return Some(());
    }

    // todo: actually call this once in a while
    pub fn prune(&mut self) {
        self.nodes.node_hashmap.retain( |k, v| {
            // the > is to always keep the root node without having to refresh it 
            let should_retain = v.last_frame_touched >= self.sys.part.current_frame;
            if ! should_retain {
                // side effect happens inside this closure... weird
                self.nodes.nodes.remove(v.slab_i);
                // remember to remove text areas and such ...
                println!(" PRUNING {:?} {:?}", k, v);
            }
            should_retain
        });
    }

    fn refresh_node(&mut self, params: &NodeParams, i: usize, parent_id: usize, frame: u64) {
        
        let node = &mut self.nodes[i];

        node.params = *params;

        // add back somewhere
        
        node.refresh(parent_id);
        self.sys.text.refresh_last_frame(node.text_id, frame);
    }
}

#[macro_export]
macro_rules! add {
    ($ui:expr, $params:expr, $code:block) => {
        {
            let i = $ui.add_as_parent_unchecked($params.key, &$params);
            $code;
            $ui.end_parent_unchecked();
            $ui.get_ref_unchecked(i, &$params.key)
        }
    };
    ($ui:expr, $params:expr) => {
        $ui.add($params.key, $params)
    };
}

impl Ui {
    pub fn begin_tree(&mut self) {
        // do cleanup here??
        self.sys.part.current_frame += 1;
    }
    
    pub fn finish_tree(&mut self) {
        clear_thread_local_stacks();
        self.layout_and_build_rects();
        self.resolve_hover();
        
        self.update_time();
        self.nodes[self.sys.root_i].reset_children();
    }
}

#[macro_export]
macro_rules! tree {
    ($ui:expr, $code:block) => {{
        $ui.begin_tree();
        $code;
        $ui.finish_tree();
    }};
}

#[derive(Debug)]
pub struct Node {
    pub id: Id,

    // also for invisible rects, used for layout
    pub rect: XyRect,

    // partial result when layouting?
    pub size: Xy<f32>,

    pub last_frame_status: LastFrameStatus,

    pub text_id: Option<usize>,

    pub imageref: Option<ImageRef>,
    pub last_static_image_ptr: Option<*const u8>,

    pub parent: usize,

    // le epic inline linked list instead of a random Vec somewhere else on the heap
    // todo: Option<usize> is 128 bits, which is ridicolous. Use a NonMaxU32 or something
    pub n_children: u16,
    pub first_child: Option<usize>,
    pub next_sibling: Option<usize>,
    // prev_sibling is never used so far.
    // at some point I was iterating the children in reverse for z ordering purposes, but I don't think that actually makes any difference.  
    // pub prev_sibling: Option<usize>,

    pub params: NodeParams,

    pub debug_name: &'static str,

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
        self.first_child = None;
        self.next_sibling = None;
        // self.prev_sibling = None;
        self.n_children = 0;
    }

    fn refresh(&mut self, parent_id: usize) {
        self.parent = parent_id;
        self.reset_children();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LastFrameStatus {
    Clicked,
    Hovered,
    Nothing,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Size {
    Fixed(Len),
    Fill,
    // "Content" can refer to the children if the node is a Stack or Container, or the inner text if it's a Text node, etc.
    // There will probably be some other size-related settings specific to some node types: for example "strictness" below. I guess those go into the Text enum variation.
    // I still don't like the name either.
    FitContent,
    FitContentOrMinimum(Len),
    // ... something like "strictness":
    //  with the "proposed" thing, a TextContent can either insist to get the minimum size it wants,
    // or be okay with whatever (and clip it, show some "..."'s, etc) 
    // todo: add FitToChildrenInitiallyButNeverResizeAfter 
}

#[derive(Debug, Clone, Copy)]
pub enum Position {
    Center,
    Start,
    End,
    Static(Len),
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct Stack {
    pub arrange: Arrange,
    pub axis: Axis,
    pub spacing: Len,
}
impl Stack {
    pub const DEFAULT: Stack = Stack {
        arrange: Arrange::Center,
        axis: Axis::Y,
        spacing: Len::Pixels(5),
    };
    pub const fn arrange(mut self, arrange: Arrange) -> Self {
        self.arrange = arrange;
        return self;
    }
    pub const fn spacing(mut self, spacing: Len) -> Self {
        self.spacing = spacing;
        return self;
    }
    pub const fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        return self;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Arrange {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[repr(C)]
#[derive(Debug, Pod, Copy, Clone, Zeroable)]
pub struct Uniforms {
    pub size: Xy<f32>,
    pub t: f32,
    pub _padding: f32,
}

pub fn cursor_pos_from_byte_offset(buffer: &GlyphonBuffer, byte_offset: usize) -> (f32, f32) {
    let line = &buffer.lines[0];
    let buffer_line = line.layout_opt().as_ref().unwrap();
    let glyphs = &buffer_line[0].glyphs;

    // todo: binary search? lol. maybe vec has it built in
    for g in glyphs {
        if g.start >= byte_offset {
            return (g.x, g.y);
        }
    }

    if let Some(glyph) = glyphs.last() {
        return (glyph.x + glyph.w, glyph.y);
    }

    // string is empty
    return (0.0, 0.0);
}

use std::hash::Hash;
fn fx_hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = FxHasher::default();
    value.hash(&mut hasher);
    hasher.finish()
}

#[derive(Debug, Default)]
pub struct MouseButtons {
    pub left: bool,
    pub right: bool,
    pub middle: bool,
    pub back: bool,
    pub forward: bool,
    pub other: u16, // 16-bit field for other buttons
}
impl MouseButtons {
    pub fn is_other_button_pressed(&self, id: u16) -> bool {
        if id < 16 {
            return self.other & (1 << id) != 0;
        } else {
            panic!("Mouse button id must be between 0 and 15")
        }
    }
}

#[derive(Debug)]
pub struct MouseInputState {
    pub position: PhysicalPosition<f64>,
    pub buttons: MouseButtons,
    pub scroll_delta: (f32, f32),
    
    // previous for diffs
    pub prev_position: PhysicalPosition<f64>,
}

impl Default for MouseInputState {
    fn default() -> Self {
        return Self {
            position: PhysicalPosition::new(0.0, 0.0),
            buttons: MouseButtons::default(),
            scroll_delta: (0.0, 0.0),

            prev_position: PhysicalPosition::new(0.0, 0.0),
        };
    }
}

impl MouseInputState {

    pub fn update(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.position = *position;
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let pressed = *state == ElementState::Pressed;
                match button {
                    MouseButton::Left => self.buttons.left = pressed,
                    MouseButton::Right => self.buttons.right = pressed,
                    MouseButton::Middle => self.buttons.middle = pressed,
                    MouseButton::Back => self.buttons.back = pressed,
                    MouseButton::Forward => self.buttons.forward = pressed,
                    MouseButton::Other(id) => {
                        if *id < 16 {
                            if pressed {
                                self.buttons.other |= 1 << id;
                            } else {
                                self.buttons.other &= !(1 << id);
                            }
                        }
                    }
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        self.scroll_delta.0 += x;
                        self.scroll_delta.1 += y;
                    }
                    MouseScrollDelta::PixelDelta(pos) => {
                        self.scroll_delta.0 += pos.x as f32;
                        self.scroll_delta.1 += pos.y as f32;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn clear_frame(&mut self) {
        self.prev_position = self.position;
    }

    pub fn cursor_diff(&self) -> (f64, f64) {
        return (
            self.prev_position.x - self.position.x,
            self.prev_position.y - self.position.y, 
        );
    }

    pub fn reset_scroll(&mut self) {
        self.scroll_delta = (0.0, 0.0);
    }
}

#[macro_export]
macro_rules! unwrap_or_return {
    ($expression:expr, $return_value:tt $(,)?) => {{
        match $expression {
            None => return $return_value,
            Some(val) => val,
        }
    }};
}

// todo: possibly split debug_name into debug_name and source_code_location, and maybe put back cfg(debug) for source_code_loc or both
#[derive(Clone, Copy, Debug)]
pub struct TypedKey<T: NodeType> {
    pub id: Id,
    pub debug_name: &'static str,
    pub nodetype_marker: PhantomData<T>,
}
impl<T: NodeType> TypedKey<T> {
    pub(crate) fn id(&self) -> Id {
        return self.id;
    }
    pub(crate) fn sibling<H: Hash>(self, value: H) -> Self {
        let mut hasher = FxHasher::default();
        self.id.0.hash(&mut hasher);
        value.hash(&mut hasher);
        let new_id = hasher.finish();

        return Self {
            id: Id(new_id),
            debug_name: self.debug_name,
            nodetype_marker: PhantomData::<T>,
        };
    }
}
impl<T: NodeType> TypedKey<T> {
    pub const fn new(id: Id, debug_name: &'static str) -> Self {
        return Self { id, debug_name, nodetype_marker: PhantomData::<T> };
    }
}

pub type NodeKey = TypedKey<Any>;

use std::fmt::Debug;
pub trait NodeType: Copy + Debug {}

#[derive(Clone, Copy, Debug)]
pub struct Any {}

impl NodeType for Any {}
impl TextTrait for Any {}
impl ParentTrait for Any {}

pub trait TextTrait: NodeType {}

pub trait ParentTrait: NodeType {}



impl NodeType for Stack {}
impl ParentTrait for Stack {}

#[derive(Clone, Copy, Debug)]
pub struct TextNodeType {}

impl NodeType for TextNodeType {}
impl TextTrait for TextNodeType {}

#[derive(Clone, Copy, Debug)]
pub struct Container {}
impl NodeType for Container {}
impl ParentTrait for Container {}



use RefreshOrClone::*;
pub enum RefreshOrClone {
    Refresh,
    AddTwin,
}
pub fn refresh_or_add_twin(current_frame: u64, old_node_last_frame_touched: u64) -> RefreshOrClone {
    if current_frame == old_node_last_frame_touched {
        // re-adding something that got added in the same frame: making a clone
        return RefreshOrClone::AddTwin;
    } else {
        // re-adding something that was added in an old frame: just a normal refresh
        return RefreshOrClone::Refresh;
    }
}

use TwinCheckResult::*;
enum TwinCheckResult<T: NodeType> {
    UpdatedNormal {
        final_i: usize,
    },
    NeedToUpdateTwin {
        twin_n: u32,
        twin_key: TypedKey<T>,
    }
}

#[macro_export]
// todo: use this macro everywhere else
macro_rules! for_each_child {
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



// impl NodeKey {
//     pub const fn validate(self) -> Self {
//         return self;
//     }
// }

// impl TypedKey<Text> {
//     pub const fn validate(self) -> Self {
//         if self.defaults.stack.is_some() || self.defaults.text.is_none()  {
//             panic!("
//             Blue Gui ran into an error when constructing a `TypedKey<Text>`.
//             Typed keys can only be constructed with NodeParams compatible with their purpose. 
            
//             TypedKey<Text> should have the following content:
//             text: Some
//             stack: None
//             image: None
            
//             If that's not what you want, consider using the general-purpose `NodeKey`, which will give you the maximum flexibility.
//             ");
//         }
        
//         return self;
//     }
// }

// impl TypedKey<Stack> {
//     pub const fn validate(self) -> Self {
//         if self.defaults.stack.is_none() || self.defaults.text.is_some() {
//             panic!("
//             Blue Gui ran into an error when constructing a `TypedKey<Text>`.
//             Typed keys can only be constructed with NodeParams compatible with their purpose. 
            
//             TypedKey<Text> should have the following content:
//             text: Some
//             stack: None
//             image: None
            
//             If that's not what you want, consider using the general-purpose `NodeKey`, which will give you the maximum flexibility.
//             ");
//         }
        
//         return self;
//     }
// }

pub struct Parent {
    node: usize
}
impl Parent {
    pub fn nest(&self, children_block: impl FnOnce()) {
        thread_local_push_parent(self.node);
        
        children_block();
        
        thread_local_pop_parent_and_sibling();
    }
}

impl<'a,  T: NodeType> UiNode<'a, T> {
    pub fn parent(&self) -> Parent {
        return Parent {
            node: self.node
        };
    }    
}

impl Ui {
    pub fn add(&mut self, params: &NodeParams) -> UiNode<Any> {
        let i = self.update_node(params.key, params);
        return self.get_ref_unchecked(i, &params.key)
    }

    pub fn add_parent(&mut self, params: &NodeParams) -> Parent {
        let node = self.update_node(params.key, params);
        return Parent {
            node
        }
    }

    pub fn v_stack2(&mut self) -> Parent {
        let node = self.update_node(ANON_VSTACK, &V_STACK);
        return Parent {
            node
        }
    }

    pub fn h_stack2(&mut self) -> Parent {
        let node = self.update_node(ANON_HSTACK, &H_STACK);
        return Parent {
            node
        }
    }
}

#[derive(Default, Debug, Clone)]
pub(crate) struct Stacks {
    parents: Vec<usize>,
    siblings: Vec<usize>,
}

// Global stacks
thread_local! {
    static THREAD_STACKS: RefCell<Stacks> = RefCell::new(Stacks::default());
}

pub(crate) fn thread_local_push_parent(new_parent: usize) {
    THREAD_STACKS.with(|stack| {
        stack.borrow_mut().parents.push(new_parent);
    });
}

pub(crate) fn thread_local_pop_parent_and_sibling() {
    THREAD_STACKS.with(|stack| {
        let mut stack = stack.borrow_mut();
        stack.parents.pop().unwrap();
        stack.siblings.pop().unwrap();
    })
}

pub(crate) fn thread_local_last_parent() -> usize {
    THREAD_STACKS.with(|stack| {
        *stack.borrow().parents.last().unwrap()
    })
}

pub(crate) fn thread_local_push_last_sibling(new_siblings: usize) {
    THREAD_STACKS.with(|stack| {
        stack.borrow_mut().siblings.push(new_siblings);
    });
}



pub(crate) fn thread_local_cycle_last_sibling(new_sibling: usize) -> usize {
    THREAD_STACKS.with(|stack| {
        let mut stack = stack.borrow_mut();

        let last_ref = stack.siblings.last_mut().unwrap();
        let previous = *last_ref;
        *last_ref = new_sibling;

        return previous;
    })
}

pub(crate) fn clear_thread_local_stacks() {
    THREAD_STACKS.with(|stack| {
        let mut stack = stack.borrow_mut();
        stack.siblings.clear();
        stack.parents.clear();
        // this should be `root_i`, but whatever
        stack.parents.push(0);
    })
}

// pub(crate) fn clone_thread_local_stack() -> Stacks {
//     THREAD_STACKS.with(|stack| {
//         let a = stack.borrow_mut().clone();
//         return a;
//     })
// }
