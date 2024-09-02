use crate::node_params::{DEFAULT, NODE_ROOT_PARAMS};
use crate::texture_atlas::{ImageRef, TextureAtlas};
use crate::unwrap_or_return;
use copypasta::{ClipboardContext, ClipboardProvider};
use glyphon::cosmic_text::{Align, StringCursor};
use glyphon::{AttrsList, Cursor as GlyphonCursor};
use glyphon::{Affinity, Resolution as GlyphonResolution};
use rustc_hash::{FxHashMap, FxHasher};
use slab::Slab;
use wgpu::*;
use winit::event::{ElementState, MouseScrollDelta};
use winit::keyboard::Key;
use crate::math::{Axis, Xy, XyRect};

use crate::for_each_child;

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
    Attrs, Buffer as GlyphonBuffer, Color as GlyphonColor, Family, FontSystem, Metrics, Shaping,
    SwashCache, TextArea, TextAtlas, TextBounds, TextRenderer,
};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{Event, KeyEvent, MouseButton, WindowEvent},
    keyboard::{ModifiersState, NamedKey},
};
use Axis::{X, Y};
use {
    util::{self, DeviceExt},
    vertex_attr_array, BindGroup, BufferAddress, BufferUsages, ColorTargetState, Device,
    MultisampleState, Queue, RenderPass, RenderPipeline, SurfaceConfiguration, VertexAttribute,
    VertexBufferLayout, VertexStepMode,
};

static T0: LazyLock<Instant> = LazyLock::new(|| Instant::now());
fn time_f32() -> f32 {
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
    rect_id: None,
    text_id: None,
    image: None,

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
pub struct Text<'data> {
    pub text: &'data str,
    pub editable: bool,
}
impl<'text> Text<'text> {
    pub const DEFAULT: Text<'static> = Text {
        text: &"",
        editable: false,
    };

    pub const fn text<'a: 'text>(mut self, text: &'a str) -> Self {
        self.text = text;
        return self;
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Image<'data> {
    pub data: &'data [u8],
}

// todo: rename to NodeDefaults 
#[derive(Debug, Copy, Clone)]
pub struct NodeParams<'text, 'image> {
    pub text: Option<Text<'text>>,
    pub image: Option<Image<'image>>,
    pub stack: Option<Stack>,
    pub rect: Rect,
    pub interact: Interact,
    pub layout: Layout,
}

type StaticParams = NodeParams<'static, 'static>;

impl<'text, 'image> NodeParams<'text, 'image> {
    
    // maybe a separate struct with no Text and no Image would be better.
    fn strip_references(&self) -> StaticParams {
        return StaticParams {
            text: None,
            image: None,
            stack: self.stack.clone(),
            rect: self.rect.clone(),
            interact: self.interact.clone(),
            layout: self.layout.clone(),
        };
    }

    // todo: remove
    fn maybe_text(&self) -> Option<Text> {
        return self.text;
    }

    pub const fn const_default() -> Self {
        return DEFAULT;
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

    pub const fn image(mut self, image_data: &'static [u8]) -> Self {
        self.image = Some(Image { data: image_data });
        return self;
    }

    pub const fn text<'a: 'text>(mut self, text: &'a str) -> Self {
        let textstruct = match self.text {
            Some(textstruct) => textstruct,
            None => Text::DEFAULT,
        };
        self.text = Some(textstruct.text(text));
        return self;
    }
    pub const fn editable(mut self, editable: bool) -> Self {
        let old_default_text = match self.text {
            Some(text) => text.text,
            None => "Insert...",
        };
        self.text = Some(Text {
            text: old_default_text,
            editable,
        });
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

    pub const fn padding_x(mut self, padding: Len) -> Self {
        self.layout.padding.x = padding;
        return self;
    }

    pub const fn padding_y(mut self, padding: Len) -> Self {
        self.layout.padding.x = padding;
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
    pub clickable: u32,
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
        a: 255 as u8,
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

pub struct NodeRef<'a, T: NodeType> {
    pub(crate) node: &'a mut Node,
    pub(crate) nodetype_marker: PhantomData<T>,
    pub(crate) sys: &'a mut System,
}

// why can't you just do it separately?
impl<'a,  T: NodeType> NodeRef<'a, T> {

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
        self.node.params.rect.vertex_colors = VertexColors::flat(color);
        return self;
    }

    pub fn set_vertex_colors(&mut self, colors: VertexColors)  -> &mut Self {
        self.node.params.rect.vertex_colors = colors;
        return self;
    }

    pub fn set_position_x(&mut self, position: Position)  -> &mut Self {
        self.node.params.layout.position.x = position;
        return self;
    }

    pub fn set_position_y(&mut self, position: Position)  -> &mut Self {
        self.node.params.layout.position.y = position;
        return self;
    }

    pub fn set_size_x(&mut self, size: Size)  -> &mut Self {
        self.node.params.layout.size.x = size;
        return self;
    }

    pub fn set_size_y(&mut self, size: Size)  -> &mut Self {
        self.node.params.layout.size.y = size;
        return self;
    }
}

impl<'a, T: TextTrait> NodeRef<'a, T> {

    pub fn set_text(&mut self, text: &str) -> &mut Self {
        if let Some(text_id) = self.node.text_id {
            self.sys.text.set_text(text_id, text);
        } else {
            // todo: log a warning or something
            // or make these things type safe somehow
        }

        return self;
    }

    pub fn set_text_attrs(&mut self, attrs: Attrs)  -> &mut Self {
        if let Some(text_id) = self.node.text_id {
            self.sys.text.set_text_attrs(text_id, attrs);
        } else {
            // todo: log a warning or something
            // or make these things type safe somehow
        }
        return self;
    }

    pub fn set_text_align(&mut self, align: Align)  -> &mut Self {
        if let Some(text_id) = self.node.text_id {
            self.sys.text.set_text_align(text_id, align);
        } else {
            // todo: log a warning or something
            // or make these things type safe somehow
        }
        return self;
    }

    pub fn get_text(&mut self) -> Option<String> {
        let text_id = self.node.text_id.unwrap();

        let text = self.sys.text.text_areas[text_id].buffer.lines[0]
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

#[derive(Debug, Copy, Clone)]
pub struct BlinkyLine {
    pub index: usize,
    pub affinity: Affinity,
}

#[derive(Debug, Copy, Clone)]
pub enum Cursor {
    BlinkyLine(BlinkyLine),
    Selection((GlyphonCursor, GlyphonCursor)),
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
    pub(crate) fn maybe_new_text_area(&mut self, text: Option<Text>, current_frame: u64) -> Option<usize> {
        let text = text?.text;

        let mut buffer = GlyphonBuffer::new(&mut self.font_system, GLOBAL_TEXT_METRICS);
        buffer.set_size(&mut self.font_system, 500., 500.);

        let mut hasher = FxHasher::default();
        text.hash(&mut hasher);
        let hash = hasher.finish();

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

    fn set_text(&mut self, text_id: usize, text: &str) {
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
pub struct NodeFront {
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
impl NodeFront {
    pub fn new(parent_id: usize, frame: u64, new_i: usize) -> Self {
        return Self {
            last_parent: parent_id,
            last_frame_touched: frame,
            n_twins: 0,
            slab_i: new_i,
        }
    }

    pub fn refresh(&mut self, parent_id: usize, frame: u64) {
        self.last_frame_touched = frame;
        self.last_parent = parent_id;
        self.n_twins = 0;
    }


}

pub struct Nodes {
    // todo: make faster o algo
    pub fronts: FxHashMap<Id, NodeFront>,
    pub nodes: Slab<Node>,
}
impl Nodes {
    pub fn get_by_id(&mut self, id: &Id) -> Option<&mut Node> {
        let i = self.fronts.get(&id)?.slab_i;
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
        let frame = self.part.current_frame;
        let parent_i = self.parent_stack.last().unwrap().clone();

        let text = params.maybe_text();
        let text_id = self.text.maybe_new_text_area(text, frame);

        let image = match params.image {
            Some(image) => {
                Some(self.texture_atlas.allocate_image(image.data))
            },
            None => None,
        };
        
        return Node {
            id: key.id(),
            rect_id: None,
            rect: Xy::new_symm([0.0, 1.0]),
            size: Xy::new_symm(10.0),
            text_id,
            image,
            parent: parent_i,

            n_children: 0,
            first_child: None,
            next_sibling: None,
            is_twin: twin_n,
            params: params.strip_references(),
            debug_name: key.debug_name,
            last_frame_status: LastFrameStatus::Nothing,
            last_hover: f32::MIN,
            last_click: f32::MIN,
            z: 0.0,
        }
    
    }
}

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

    // stack for keeping track of parents when adding
    pub parent_stack: Vec<usize>,
    
    // stack for keeping track of siblings when adding
    pub last_child_stack: Vec<usize>,

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


    pub fn to_pixels(&self, len: Len, axis: Axis) -> u32 {
        match len {
            Len::Pixels(pixels) => return pixels,
            Len::Frac(frac) => return (frac * self.sys.part.unifs.size[axis]) as u32,
        }
    }

    pub fn to_pixels2(&self, len: Xy<Len>) -> Xy<u32> {
        return Xy::new(
            self.to_pixels(len.x, X),
            self.to_pixels(len.y, Y),
        );
    }

    pub fn to_frac(&self, len: Len, axis: Axis) -> f32 {
        match len {
            Len::Pixels(pixels) => return (pixels as f32) / self.sys.part.unifs.size[axis],
            Len::Frac(frac) => return frac,
        }
    }

    pub fn pixels_to_frac(&self, pixels: u32, axis: Axis) -> f32 {
        return (pixels as f32) / self.sys.part.unifs.size[axis];
    }
    pub fn f32_pixels_to_frac(&self, pixels: f32, axis: Axis) -> f32 {
        return pixels / self.sys.part.unifs.size[axis];
    }

    pub fn f32_pixels_to_frac2(&self, pixels: Xy<f32>) -> Xy<f32> {
        return Xy::new(
            self.f32_pixels_to_frac(pixels.x, X),
            self.f32_pixels_to_frac(pixels.y, Y),
        );
    }

    pub fn to_frac2(&self, len: Xy<Len>) -> Xy<f32> {
        return Xy::new(
            self.to_frac(len.x, X),
            self.to_frac(len.y, Y),
        );
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

        let mut texture_atlas = TextureAtlas::new(&device);

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
            source: ShaderSource::Wgsl(include_str!("box.wgsl").into()),
        });

        let mut primitive = PrimitiveState::default();
        primitive.cull_mode = None;

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

        let mut node_fronts = FxHashMap::with_capacity_and_hasher(100, Default::default());
        

        
        let mut nodes = Slab::with_capacity(100);
        let root_i = nodes.insert(NODE_ROOT);
        let root_nodefront = NodeFront {
            last_parent: usize::default(),
            last_frame_touched: u64::MAX,
            slab_i: root_i,
            n_twins: 0,
        };
        
        let mut stack = Vec::with_capacity(7);
        stack.push(root_i);
        
        let mut parent_stack = Vec::with_capacity(7);
        parent_stack.push(root_i);

        node_fronts.insert(NODE_ROOT_ID, root_nodefront);

        let nodes = Nodes {
            fronts: node_fronts,
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

                parent_stack,

                last_child_stack: Vec::with_capacity(20),

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

    pub fn add<T: NodeType>(&mut self, key: TypedKey<T>, defaults: &NodeParams) -> NodeRef<T> {
        let i = self.update_node(key, defaults, false);
        return self.get_ref_unchecked(i, &key)
    }

    pub fn add_as_parent_unchecked<T: ParentTrait>(&mut self, key: TypedKey<T>, defaults: &NodeParams) -> usize {
        let i = self.update_node(key, defaults, true);
        return i;
    }

    pub fn end_parent_unchecked(&mut self) {
        self.sys.parent_stack.pop();
        self.sys.last_child_stack.pop();
    }

    // todo: I wanted to add this checked version, but there is a twin-related problem here.
    // if the key got twinned, ended_parent will lead to the node with the twinned id, but the key will have the non-twinned id.
    // sounds stupid to store the non-twinned id just for this stupid check.

    // I think what we want is the still the Latest Twin Id, but should think some more about it.  
    pub fn end_parent<T: NodeType>(&mut self, key: TypedKey<T>) {
        let ended_parent = self.sys.parent_stack.pop();

        #[cfg(debug_assertions)] {
            let ended_parent = ended_parent.expect(&format!("Misplaced end_parent: {}", key.debug_name));
            let ended_parent_id = self.nodes[ended_parent].id;

            let twin_key = self.get_latest_twin_key(key).unwrap();
            debug_assert!(ended_parent_id == twin_key.id(),
            "Misplaced end_parent: tried to end {:?}, but {:?} was the latest parent", self.nodes[ended_parent].debug_name(), twin_key.debug_name
            );
        }


        self.sys.last_child_stack.pop();
    }

    // don't expect this to give you twin nodes automatically
    pub fn get_ref<T: NodeType>(&mut self, key: TypedKey<T>) -> NodeRef<T> {
        let node_i = self.nodes.fronts.get(&key.id()).unwrap().slab_i;
        return self.get_ref_unchecked(node_i, &key)
    }

    // only for the macro, use get_ref 
    pub fn get_ref_unchecked<T: NodeType>(&mut self, i: usize, _key: &TypedKey<T>) -> NodeRef<T> {        
        return NodeRef {
            node: &mut self.nodes[i],
            sys: &mut self.sys,
            nodetype_marker: PhantomData::<T>,
        };
    }

    pub fn update_node<T: NodeType>(&mut self, key: TypedKey<T>, defaults: &NodeParams, make_new_layer: bool) -> usize {
        let parent_i = self.sys.parent_stack.last().unwrap().clone();

        let frame = self.sys.part.current_frame;

        // Check the node corresponding to the key's id.
        // We might find that the key has already been used in this same frame: 
        //      in this case, we take note, and calculate a twin key to use to add a "twin" in the next section.
        // Otherwise, we add or refresh normally, and take note of the final i.
        let twin_check_result = match self.nodes.fronts.entry(key.id()) {
            // Add a normal node (no twins).
            Entry::Vacant(v) => {

                let new_node = self.sys.build_new_node(&key, defaults, None);

                let final_i = self.nodes.nodes.insert(new_node);
                v.insert(NodeFront::new(parent_i, frame, final_i));

                UpdatedNormal{ final_i }
            },
            Entry::Occupied(o) => {
                let old_nodefront = o.into_mut();
                
                match refresh_or_add_twin(frame, old_nodefront.last_frame_touched) {
                    // Refresh a normal node from the previous frame (no twins).
                    Refresh => {
                        old_nodefront.refresh(parent_i, frame);
                        // todo2: check the nodefront values and maybe skip reaching into the node
                        let final_i = old_nodefront.slab_i;
                        self.refresh_node(final_i, parent_i, frame);
                        
                        self.nodes[final_i].params = defaults.strip_references();
                        if let Some(text) = defaults.text {
                            // todo: if there's no text_id, it should be made
                            if let Some(text_id) = self.nodes[final_i].text_id {
                                self.sys.text.set_text(text_id, text.text);
                            }
                        }

                        UpdatedNormal{ final_i }
                    }
                    // do nothing, just calculate the twin key and go to twin part below
                    AddTwin => {
                        old_nodefront.n_twins += 1;
                        let twin_key = key.sibling(old_nodefront.n_twins);
                        NeedToUpdateTwin { twin_key, twin_n: old_nodefront.n_twins }
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
                match self.nodes.fronts.entry(twin_key.id()) {
                    // Add new twin.
                    Entry::Vacant(v) => {
                        let new_twin_node = self.sys.build_new_node(&twin_key, defaults, Some(twin_n));
                        let real_final_i = self.nodes.nodes.insert(new_twin_node);
                        v.insert(NodeFront::new(parent_i, frame, real_final_i));
                        real_final_i
                    },
                    // Refresh a twin from the previous frame.
                    Entry::Occupied(o) => {
                        let old_twin_nodefront = o.into_mut();
    
                        
                        // todo2: check the nodefront values and maybe skip reaching into the node
                        old_twin_nodefront.refresh(parent_i, frame);
                        
                        let real_final_i = old_twin_nodefront.slab_i;

                        self.nodes[real_final_i].params = defaults.strip_references();

                        if let Some(text) = defaults.text {
                            // todo: if there's no text_id, it should be made
                            if let Some(text_id) = self.nodes[real_final_i].text_id {
                                self.sys.text.set_text(text_id, text.text);
                            }
                        }
                        
                        self.refresh_node(real_final_i, parent_i, frame);
                        real_final_i
                    },
    
                }
            },
        };

        self.add_child_to_parent(real_final_i, parent_i);
        if make_new_layer {
            self.sys.parent_stack.push(real_final_i);           
        }

        return real_final_i;
    }

    fn get_latest_twin_key<T: NodeType>(&self, key: TypedKey<T>) -> Option<TypedKey<T>> {

        let nodefront = self.nodes.fronts.get(&key.id())?;

        if nodefront.n_twins == 0 {
            return Some(key);
        }

        // todo: yell a very loud warning here. latest_twin is more like a best-effort way to deal with dumb code. 
        // the proper way is to just use unique keys, or to use the returned noderef, if that becomes a thing.
        let twin_key = key.sibling(nodefront.n_twins);

        return Some(twin_key);
    }

    pub fn add_child_to_parent(&mut self, id: usize, parent_id: usize) {
        self.nodes[parent_id].n_children += 1;

        if self.nodes[parent_id].first_child == None {
            self.nodes[parent_id].first_child = Some(id);

            self.sys.last_child_stack.push(id);

        } else {
            let prev_sibling = *self.sys.last_child_stack.last().unwrap();
            // self.nodes[id].prev_sibling = Some(prev_sibling);
            self.nodes[prev_sibling].next_sibling = Some(id);
            *self.sys.last_child_stack.last_mut().unwrap() = id;
        }

    }

    pub fn handle_keyboard_event(&mut self, event: &KeyEvent) -> bool {
        // todo: remove line.reset(); and do it only once per frame via change watcher guy

        match &event.logical_key {
            Key::Named(named_key) => match named_key {
                NamedKey::F1 => {
                    if event.state.is_pressed() {
                        if self.sys.debug_key_pressed == false {
                            #[cfg(debug_assertions)]
                            {
                                self.sys.debug_mode = !self.sys.debug_mode;
                            }
                        }
                    }

                    self.sys.debug_key_pressed = event.state.is_pressed();
                }
                _ => {}
            },
            _ => {}
        }

        // if there is no focused text node, return consumed: false
        let id = unwrap_or_return!(self.sys.focused, false);
        let node = unwrap_or_return!(self.nodes.get_by_id(&id), false);
        let text_id = unwrap_or_return!(node.text_id, false);

        // return consumed: true in each of these cases. Still don't consume keys that the UI doesn't use.
        if event.state.is_pressed() {
            let buffer = &mut self.sys.text.text_areas[text_id].buffer;
            let line = &mut buffer.lines[0];

            match &event.logical_key {
                // todo: ctrl + Z
                Key::Named(named_key) => match named_key {
                    NamedKey::ArrowLeft => {
                        match (self.sys.key_mods.shift_key(), self.sys.key_mods.control_key()) {
                            (true, true) => line.text.control_shift_left_arrow(),
                            (true, false) => line.text.shift_left_arrow(),
                            (false, true) => line.text.control_left_arrow(),
                            (false, false) => line.text.left_arrow(),
                        }
                        return true;
                    }
                    NamedKey::ArrowRight => {
                        match (self.sys.key_mods.shift_key(), self.sys.key_mods.control_key()) {
                            (true, true) => line.text.control_shift_right_arrow(),
                            (true, false) => line.text.shift_right_arrow(),
                            (false, true) => line.text.control_right_arrow(),
                            (false, false) => line.text.right_arrow(),
                        }
                        return true;
                    }
                    NamedKey::Backspace => {
                        if self.sys.key_mods.control_key() {
                            line.text.ctrl_backspace();
                        } else {
                            line.text.backspace();
                        }
                        line.reset();
                        return true;
                    }
                    NamedKey::End => {
                        match self.sys.key_mods.shift_key() {
                            true => line.text.shift_end(),
                            false => line.text.go_to_end(),
                        }
                        line.reset();
                        return true;
                    }
                    NamedKey::Home => {
                        match self.sys.key_mods.shift_key() {
                            false => line.text.go_to_start(),
                            true => line.text.shift_home(),
                        }
                        line.reset();
                        return true;
                    }
                    NamedKey::Delete => {
                        if self.sys.key_mods.control_key() {
                            line.text.ctrl_delete();
                        } else {
                            line.text.delete();
                        }
                        line.reset();
                        return true;
                    }
                    NamedKey::Space => {
                        line.text.insert_str_at_cursor(" ");
                        line.reset();
                        return true;
                    }
                    _ => {}
                },
                Key::Character(new_char) => {
                    if !self.sys.key_mods.control_key()
                        && !self.sys.key_mods.alt_key()
                        && !self.sys.key_mods.super_key()
                    {
                        line.text.insert_str_at_cursor(new_char);
                        line.reset();
                        return true;
                    } else if self.sys.key_mods.control_key() {
                        match new_char.as_str() {
                            "c" => {
                                let selected_text = line.text.selected_text().to_owned();
                                if let Some(text) = selected_text {
                                    let _ = self.sys.clipboard.set_contents(text.to_string());
                                }
                                return true;
                            }
                            "v" => {
                                if let Ok(pasted_text) = self.sys.clipboard.get_contents() {
                                    line.text.insert_str_at_cursor(&pasted_text);
                                    line.reset();
                                }
                                return true;
                            }
                            _ => {}
                        }
                    }
                }
                Key::Unidentified(_) => {}
                Key::Dead(_) => {}
            };
        }

        return false;
    }

    // returns: is the event consumed?
    pub fn handle_events(&mut self, full_event: &Event<()>, queue: &Queue) -> bool {
        match full_event {
            Event::NewEvents(_) => {
                self.sys.mouse_status.clear_frame();
            },
            _ => {}
        }


        if let Event::WindowEvent { event, .. } = full_event {
            match event {
                WindowEvent::CursorMoved { position, .. } => {
                    self.sys.part.mouse_pos.x = position.x as f32;
                    self.sys.part.mouse_pos.y = position.y as f32;
                    self.resolve_hover();
                    // cursormoved is never consumed
                }
                WindowEvent::MouseInput { button, state, .. } => {
                    if *button == MouseButton::Left {
                        let is_pressed = state.is_pressed();
                        if is_pressed {
                            let consumed = self.resolve_click();
                            return consumed;
                        } else {
                            let waiting_for_click_release = self.sys.waiting_for_click_release;
                            let on_rect = self.resolve_click_release();
                            let consumed = on_rect && waiting_for_click_release;
                            return consumed;
                        }
                    }
                }
                WindowEvent::ModifiersChanged(modifiers) => {
                    self.sys.key_mods = modifiers.state();
                }
                WindowEvent::KeyboardInput {
                    event,
                    is_synthetic,
                    ..
                } => {
                    if !is_synthetic {
                        let consumed = self.handle_keyboard_event(event);
                        return consumed;
                    }
                }
                WindowEvent::Resized(size) => self.resize(size, queue),
                _ => {}
            }

            self.sys.mouse_status.update(event);

        }

        return false;
    }

    pub fn layout_and_build_rects(&mut self) {
        self.sys.rects.clear();
        
        self.determine_size(self.sys.root_i, Xy::new(1.0, 1.0));
        self.build_rect_and_place_children(self.sys.root_i);

        self.push_cursor_rect();
    }

    fn get_proposed_size(&mut self, node: usize, proposed_size: Xy<f32>) -> Xy<f32> {
        let padding = self.to_frac2(self.nodes[node].params.layout.padding);
        let mut proposed_size = proposed_size;

        for axis in [X, Y] {
            // adjust proposed size based on padding
            proposed_size[axis] -= 2.0 * padding[axis];

            // adjust proposed size based on our own size
            match self.nodes[node].params.layout.size[axis] {
                Size::FitContent | Size::FitContentOrMinimum(_) => {}, // propose the whole size. We will shrink our own final size later if they end up using less or more 
                Size::Fill => {}, // keep the whole proposed_size
                Size::Fixed(len) => match len {
                    Len::Pixels(pixels) => {
                        proposed_size[axis] = self.pixels_to_frac(pixels, axis);
                    },
                    Len::Frac(frac) => {
                        proposed_size[axis] *= frac;
                    },
                }
            }
        }

        // just moved this from get_children_proposed_size(), haven't thought about it that hard, but it seems right.
        if let Some(stack) = self.nodes[node].params.stack {
            let main = stack.axis;
            let n_children = self.nodes[node].n_children as f32;
            let spacing = self.to_frac(stack.spacing, stack.axis);

            // adjust proposed size based on spacing
            if n_children > 1.5 {
                proposed_size[main] -= spacing * (n_children - 1.0);
            }
        }

        return proposed_size;
    }

    fn get_children_proposed_size(&mut self, node: usize, proposed_size: Xy<f32>) -> Xy<f32> {
        let mut child_proposed_size = proposed_size;

        if let Some(stack) = self.nodes[node].params.stack {
            let main = stack.axis;
            let n_children = self.nodes[node].n_children as f32;

            // divide between children
            child_proposed_size[main] = child_proposed_size[main] / n_children;
        }
        return child_proposed_size
    }

    fn determine_size(&mut self, node: usize, proposed_size: Xy<f32>) -> Xy<f32> {
        let stack = self.nodes[node].params.stack;
        
        // calculate the total size to propose to children
        let proposed_size = self.get_proposed_size(node, proposed_size);
        // divide it across children (if Stack)
        let child_proposed_size = self.get_children_proposed_size(node, proposed_size);

        // Propose a size to the children and let them decide
        let mut content_size = Xy::new(0.0, 0.0);
        for_each_child!(self, self.nodes[node], child, {
            let child_size = self.determine_size(child, child_proposed_size);
            content_size.update_for_child(child_size, stack);
        });

        // Propose the whole proposed_size (regardless of stack) to the contents, and let them decide.
        if let Some(_) = self.nodes[node].text_id {
            let text_size = self.determine_text_size(node, proposed_size);
            content_size.update_for_content(text_size, stack);
        }
        if let Some(_) = self.nodes[node].image {
            let image_size = self.determine_image_size(node, proposed_size);
            content_size.update_for_content(image_size, stack);
        }

        // Decide our own size. 
        //   We either use the proposed_size that we proposed to the children,
        //   or we change our mind to based on children.
        // todo: is we're not fitcontenting, we can skip the update_for_* calls instead, and then remove this, I guess.
        let mut final_size = proposed_size;
        for axis in [X, Y] {
            match self.nodes[node].params.layout.size[axis] {
                Size::FitContent => {
                    final_size[axis] = content_size[axis];
                }
                Size::FitContentOrMinimum(min_size) => {
                    let min_size = match min_size {
                        Len::Pixels(pixels) => {
                            self.pixels_to_frac(pixels, axis)
                        },
                        Len::Frac(frac) => proposed_size[axis] * frac
                    };

                    final_size[axis] = content_size[axis].max(min_size);
                }
                _ => {},
            }
        }

        // add back padding to get the real final size
        final_size = self.adjust_final_size(node, final_size);


        self.nodes[node].size = final_size;
        return final_size;
    }

    fn determine_image_size(&mut self, node: usize, _proposed_size: Xy<f32>) -> Xy<f32> {
        let image_ref = self.nodes[node].image.unwrap();
        let size = image_ref.original_size;
        return self.f32_pixels_to_frac2(size);
    }

    fn determine_text_size(&mut self, node: usize, _proposed_size: Xy<f32>) -> Xy<f32> {
        let text_id = self.nodes[node].text_id.unwrap();
        let buffer = &mut self.sys.text.text_areas[text_id].buffer;

        // this is for FitContent on both directions, basically.
        // todo: the rest.
        // also, note: the set_align trick might not be good if we expose the ability to set whatever align the user wants.

        // let w = proposed_size.x * self.sys.part.unifs.size[X];
        // let h = proposed_size.y * self.sys.part.unifs.size[Y];
        let w = 999999.0;
        let h = 999999.0;

        for line in &mut buffer.lines {
            line.set_align(Some(glyphon::cosmic_text::Align::Left));
        }

        buffer.set_size(&mut self.sys.text.font_system, w, h);
        buffer.shape_until_scroll(&mut self.sys.text.font_system, false);

        let trimmed_size = buffer.measure_text_pixels();

        // self.sys.text.text_areas[text_id].buffer.set_size(&mut self.sys.text.font_system, trimmed_size.x, trimmed_size.y);
        // self.sys.text.text_areas[text_id]
        //     .buffer
        //     .shape_until_scroll(&mut self.sys.text.font_system, false);

        // for axis in [X, Y] {
        //     trimmed_size[axis] *= 2.0;
        // }

        // return proposed_size;
        return self.f32_pixels_to_frac2(trimmed_size);
    }




    fn adjust_final_size(&mut self, node: usize, final_size: Xy<f32>) -> Xy<f32> {
        // re-add spacing and padding to the final size we calculated
        let mut final_size = final_size;

        let padding = self.to_frac2(self.nodes[node].params.layout.padding);
        for axis in [X, Y] {
            final_size[axis] += 2.0 * padding[axis];
        }

        if let Some(stack) = self.nodes[node].params.stack {
            let spacing = self.to_frac(stack.spacing, stack.axis);
            let n_children = self.nodes[node].n_children as f32;
            let main = stack.axis;

            if n_children > 1.0 {
                final_size[main] += spacing * (n_children - 1.0);
            }
        }

        return final_size;
    }

    fn build_rect_and_place_children(&mut self, node: usize) {
        self.build_rect(node);
        
        if let Some(stack) = self.nodes[node].params.stack {
            self.build_rect_and_place_children_stack(node, stack);
        } else {
            self.build_rect_and_place_children_container(node);
        };

        self.build_and_place_image(node);
        self.place_text(node, self.nodes[node].rect);
    }

    fn build_rect_and_place_children_stack(&mut self, node: usize, stack: Stack) {
        let (main, cross) = (stack.axis, stack.axis.other());
        let parent_rect = self.nodes[node].rect;
        let padding = self.to_frac2(self.nodes[node].params.layout.padding);
        let spacing = self.to_frac(stack.spacing, stack.axis);
        
        // Totally ignore the children's chosen Position's and place them according to our own Stack::Arrange value.

        // collect all the children sizes in a vec
        let n = self.nodes[node].n_children;
        self.sys.size_scratch.clear();
        for_each_child!(self, self.nodes[node], child, {
            self.sys.size_scratch.push(self.nodes[child].size[main]);
        });

        let mut total_size = 0.0;
        for s in &self.sys.size_scratch {
            total_size += s;
        }
        if n > 0 {
            total_size += spacing * (n - 1) as f32;
        }

        let mut main_origin = match stack.arrange {
            Arrange::Start => parent_rect[main][0] + padding[main],
            Arrange::End => parent_rect[main][1] + padding[main] - total_size,
            Arrange::Center => {
                let center = (parent_rect[main][1] + parent_rect[main][0]) / 2.0 - 2.0 * padding[main];
                center - total_size / 2.0
            },
            _ => todo!(),
        };

        for_each_child!(self, self.nodes[node], child, {
            let size = self.nodes[child].size;

            match self.nodes[child].params.layout.position[cross] {
                Position::Center => {
                    let origin = (parent_rect[cross][1] + parent_rect[cross][0]) / 2.0;
                    self.nodes[child].rect[cross] = [
                        origin - size[cross] / 2.0 ,
                        origin + size[cross] / 2.0 ,
                    ];  
                },
                Position::Start => {
                    let origin = parent_rect[cross][0] + padding[cross];
                    self.nodes[child].rect[cross] = [origin, origin + size[cross]];         
                },
                Position::Static(len) => {
                    let static_pos = self.to_frac(len, cross);
                    let origin = parent_rect[cross][0] + padding[cross] + static_pos;
                    self.nodes[child].rect[cross] = [origin, origin + size[cross]];         
                },
                Position::End => {
                    let origin = parent_rect[cross][1] - padding[cross];
                    self.nodes[child].rect[cross] = [origin - size[cross], origin];
                },
            }

            self.nodes[child].rect[main] = [main_origin, main_origin + size[main]];

            self.build_rect_and_place_children(child);

            main_origin += self.nodes[child].size[main] + spacing;
        });
    }

    fn build_rect_and_place_children_container(&mut self, node: usize) {
        let parent_rect = self.nodes[node].rect;
        let padding = self.to_frac2(self.nodes[node].params.layout.padding);

        for_each_child!(self, self.nodes[node], child, {
            let size = self.nodes[child].size;

            // check the children's chosen Position's and place them.
            for ax in [X, Y] {
                match self.nodes[child].params.layout.position[ax] {
                    Position::Start => {
                        let origin = parent_rect[ax][0] + padding[ax];
                        self.nodes[child].rect[ax] = [origin, origin + size[ax]];         
                    },
                    Position::Static(len) => {
                        let static_pos = self.to_frac(len, ax);
                        let origin = parent_rect[ax][0] + padding[ax] + static_pos;
                        self.nodes[child].rect[ax] = [origin, origin + size[ax]];
                    }
                    Position::End => {
                        let origin = parent_rect[ax][1] - padding[ax];
                        self.nodes[child].rect[ax] = [origin - size[ax], origin];
                    },
                    Position::Center => {
                        let origin = (parent_rect[ax][1] + parent_rect[ax][0]) / 2.0;
                        self.nodes[child].rect[ax] = [
                            origin - size[ax] / 2.0 ,
                            origin + size[ax] / 2.0 ,
                        ];           
                    },
                }
            }

            self.build_rect_and_place_children(child);
        });
    }

    pub fn build_and_place_image(&mut self, node: usize) {
        let node = &mut self.nodes.nodes[node];
        
        if let Some(image) = node.image {
            // in debug mode, draw invisible rects as well.
            // usually these have filled = false (just the outline), but this is not enforced.
            if node.params.rect.visible || self.sys.debug_mode {
                self.sys.rects.push(RenderRect {
                    rect: node.rect.to_graphics_space(),
                    vertex_colors: node.params.rect.vertex_colors,
                    last_hover: node.last_hover,
                    last_click: node.last_click,
                    clickable: node.params.interact.click_animation.into(),
                    id: node.id,
                    z: 0.0,
                    radius: 30.0,
                    filled: node.params.rect.filled as u32,

                    tex_coords: image.tex_coords,
                });
            }
        }
    }

    pub fn place_text(&mut self, node: usize, rect: XyRect) {
        let padding = self.to_pixels2(self.nodes[node].params.layout.padding);
        let node = &mut self.nodes[node];
        let text_id = node.text_id;

        if let Some(text_id) = text_id {
            let left = rect[X][0] * self.sys.part.unifs.size[X];
            let top = rect[Y][0] * self.sys.part.unifs.size[Y];

            // let right = rect[X][1] * self.sys.part.unifs.size[X];
            // let bottom =     rect[Y][1] * self.sys.part.unifs.size[Y];

            self.sys.text.text_areas[text_id].left = left + padding[X] as f32;
            self.sys.text.text_areas[text_id].top = top + padding[Y] as f32;
           
            // self.sys.text.text_areas[text_id].bounds.left = left as i32 + padding[X] as i32;
            // self.sys.text.text_areas[text_id].bounds.top = top as i32 + padding[Y] as i32;

            // self.sys.text.text_areas[text_id].bounds.right = right as i32;
            // self.sys.text.text_areas[text_id].bounds.bottom = bottom as i32;
        }
    }

    pub fn is_clicked(&self, node_key: NodeKey) -> bool {
        let real_key = self.get_latest_twin_key(node_key);
        if let Some(real_key) = real_key {
            return self.sys.clicked.contains(&real_key.id);
        } else {
            return false;
        }
        
    }

    pub fn is_dragged(&self, node_key: NodeKey) -> Option<(f64, f64)> {
        if self.is_clicked(node_key) {
            return Some(self.sys.mouse_status.cursor_diff())
        } else {
            return None;
        }
    }

    // todo: is_clicked_advanced

    pub fn is_hovered(&self, node_key: NodeKey) -> bool {
        return self.sys.hovered.last() != Some(&node_key.id);
    }

    // todo: is_hovered_advanced

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
        self.sys.frame_t = time_f32();
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
                clickable: current_node.params.interact.click_animation.into(),
                id: current_node.id,
                z: 0.0,
                radius: 30.0,
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
                    clickable: 0,
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
                    clickable: 0,
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

    pub fn render<'pass>(&'pass self, render_pass: &mut RenderPass<'pass>) {
        let n = self.sys.rects.len() as u32;
        if n > 0 {
            render_pass.set_pipeline(&self.sys.render_pipeline);
            render_pass.set_bind_group(0, &self.sys.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.sys.gpu_vertex_buffer.slice(n));
            render_pass.draw(0..6, 0..n);
        }

        self.sys.text
            .text_renderer
            .render(&self.sys.text.atlas, render_pass)
            .unwrap();
    }

    pub fn prepare(&mut self, device: &Device, queue: &Queue) {       
        
        // self.prune();
        // self.build_buffers();
        self.sys.gpu_vertex_buffer.queue_write(&self.sys.rects[..], queue);
        
        self.sys.texture_atlas.load_to_gpu(&queue);

        // update gpu time
        // magical offset...
        queue.write_buffer(&self.sys.base_uniform_buffer, 8, bytemuck::bytes_of(&self.sys.frame_t));

        self.sys.text
            .text_renderer
            .prepare(
                device,
                queue,
                &mut self.sys.text.font_system,
                &mut self.sys.text.atlas,
                GlyphonResolution {
                    width: self.sys.part.unifs.size[X] as u32,
                    height: self.sys.part.unifs.size[Y] as u32,
                },
                &mut self.sys.text.text_areas,
                &mut self.sys.text.cache,
                self.sys.part.current_frame,
            )
            .unwrap();

        // do cleanup here????
        self.sys.hovered.clear();
        // self.sys.clicked.clear()
    }

    pub fn scan_mouse_hits(&mut self) -> Option<Id> {
        self.sys.mouse_hit_stack.clear();

        for rect in &self.sys.rects {
            if self.sys.part.mouse_hit_rect(rect) {
                self.sys.mouse_hit_stack.push((rect.id, rect.z));
            }
        }

        // only the one with the highest z is actually clicked.
        // in practice, nobody ever sets the Z. it depends on the order.
        let mut topmost_hit = None;

        let mut max_z = f32::MAX;
        for (id, z) in self.sys.mouse_hit_stack.iter().rev() {
            if *z < max_z {
                max_z = *z;
                topmost_hit = Some(*id);
            }
        }

        return topmost_hit;
    }

    // called on every mouse movement AND on every frame.
    // todo: think if it's really worth it to do this on every mouse movement.
    pub fn resolve_hover(&mut self) {
        let topmost_mouse_hit = self.scan_mouse_hits();

        if let Some(hovered_id) = topmost_mouse_hit {
            self.sys.hovered.push(hovered_id);
            let t = time_f32();
            let node = self.nodes.get_by_id(&hovered_id).unwrap();
            node.last_hover = t;
        }
    }

    pub fn resolve_click(&mut self) -> bool {
        let topmost_mouse_hit = self.scan_mouse_hits();

        // defocus when use clicking anywhere outside.
        self.sys.focused = None;

        if let Some(clicked_id) = topmost_mouse_hit {
            self.sys.waiting_for_click_release = true;

            self.sys.clicked.push(clicked_id);
            let t = time_f32();
            let node = self.nodes.get_by_id(&clicked_id).unwrap();
            node.last_click = t;

            if let Some(text) = node.params.maybe_text() {
                if text.editable {
                    self.sys.focused = Some(clicked_id);
                }
            }

            if let Some(id) = node.text_id {
                let text_area = &mut self.sys.text.text_areas[id];
                let (x, y) = (
                    self.sys.part.mouse_pos.x - text_area.left,
                    self.sys.part.mouse_pos.y - text_area.top,
                );

                // todo: with how I'm misusing cosmic-text, this might become "unsafe" soon (as in, might be incorrect or cause panics, not actually unsafe).
                // I think in general, there should be a safe version of hit() that just forces a rerender just to be sure that the offset is safe to use.
                // But in this case, calling this in resolve_mouse_input() and not on every winit mouse event probably means it's safe

                // actually, the enlightened way is that cosmic_text exposes an "unsafe" hit(), but we only ever see the string + cursor + buffer struct, and call that hit(), which doesn't return an offset but just mutates the one inside.
                text_area.buffer.hit(x, y);
            }
        }

        let consumed = topmost_mouse_hit.is_some();
        return consumed;
    }

    pub fn resolve_click_release(&mut self) -> bool {
        self.sys.waiting_for_click_release = false;
        let topmost_mouse_hit = self.scan_mouse_hits();
        let consumed = topmost_mouse_hit.is_some();
        self.sys.clicked.clear();
        return consumed;
    }

    pub fn set_text(&mut self, key: NodeKey, text: &str) {
        if let Some(node) = self.nodes.get_by_id(&key.id()) {
            let text_id = node.text_id.unwrap();
            self.sys.text.set_text(text_id, text);
        }
    }

    // todo: actually call this once in a while
    pub fn prune(&mut self) {
        self.nodes.fronts.retain( |k, v| {
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

    fn refresh_node(&mut self, final_i: usize, parent_id: usize, frame: u64) {
        let old_node = &mut self.nodes[final_i];
                        
        old_node.refresh(parent_id);
        self.sys.text.refresh_last_frame(old_node.text_id, frame);
    }
}

#[macro_export]
macro_rules! add {
    ($ui:expr, $key:expr, $defaults:expr, $code:block) => {
        {
            let i = $ui.add_as_parent_unchecked($key, &$defaults);
            $code;
            $ui.end_parent_unchecked();
            $ui.get_ref_unchecked(i, &$key)
        }
    };
    ($ui:expr, $key:expr, $defaults:expr) => {
        $ui.add($key, $defaults)
    };
}

macro_rules! create_layer_macro {
    ($macro_name:ident, $defaults_name:expr, $debug_name:expr) => {
        #[macro_export]
        macro_rules! $macro_name {
            ($ui:expr, $code:block) => {
                let anonymous_key = view_derive::anon_node_key!(NodeKey, $debug_name);
                $ui.add_as_parent_unchecked(anonymous_key, &$defaults_name);
                $code;
                $ui.end_parent_unchecked();
            };

            // named version. allows writing this: h_stack!(ui, CUSTOM_H_STACK, { ... })
            // it's basically the same as add!, not sure if it's even worth having.
            // especially with no checks that CUSTOM_H_STACK is actually a h_stack.
            ($ui:expr, $node_key:expr, $code:block) => {
                $ui.add_as_parent_unchecked($node_key, &$defaults_name);
                $code;
                $ui.end_parent_unchecked();
            };
        }
    };
}

create_layer_macro!(h_stack, crate::node_params::H_STACK, "HStack");
create_layer_macro!(v_stack, crate::node_params::V_STACK, "HStack");
create_layer_macro!(margin, crate::node_params::MARGIN, "Margin");
create_layer_macro!(panel, crate::node_params::PANEL, "Panel");

#[macro_export]
macro_rules! text {
    ($ui:expr, $text:expr) => {
        let anonymous_key = view_derive::anon_node_key!(TypedKey<Text>, "Text");
        $ui.add(anonymous_key, &TEXT).set_text($text);
    };
}

impl Ui {
    pub fn begin_tree(&mut self) {
        // do cleanup here??
        self.sys.part.current_frame += 1;
    }
    
    pub fn finish_tree(&mut self) {
        self.layout_and_build_rects();
        self.resolve_hover();
        
        // ...maybe it's better to put this stuff in end() rather than begin()?
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
    // visible rect only
    pub rect_id: Option<usize>,
    // also for invisible rects, used for layout
    pub rect: XyRect,

    // partial result when layouting?
    pub size: Xy<f32>,

    pub last_frame_status: LastFrameStatus,

    pub text_id: Option<usize>,

    pub image: Option<ImageRef>,

    pub parent: usize,

    // le epic inline linked list instead of a random Vec somewhere else on the heap
    // todo: Option<usize> is 128 bits, which is ridicolous. Use a NonMaxU32 or something
    pub n_children: u16,
    pub first_child: Option<usize>,
    pub next_sibling: Option<usize>,
    // prev_sibling is never used so far.
    // at some point I was iterating the children in reverse for z ordering purposes, but I don't think that actually makes any difference.  
    // pub prev_sibling: Option<usize>,

    pub params: NodeParams<'static, 'static>,

    pub debug_name: &'static str,

    pub is_twin: Option<u32>,

    pub last_hover: f32,
    pub last_click: f32,
    pub z: f32,
}
impl Node {
    pub fn debug_name(&self) -> String {
        let debug_name = match self.is_twin {
            Some(n) => format!("{} (twin #{}", self.debug_name, n),
            None => format!("{}", self.debug_name),
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Len {
    Pixels(u32),
    Frac(f32),
}
impl Len {
    pub const ZERO: Self = Self::Pixels(0);
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

#[derive(Debug)]
pub struct TypedGpuBuffer<T: Pod> {
    pub buffer: Buffer,
    pub marker: std::marker::PhantomData<T>,
}
impl<T: Pod> TypedGpuBuffer<T> {
    pub fn new(buffer: Buffer) -> Self {
        Self {
            buffer,
            marker: PhantomData::<T>,
        }
    }

    pub fn size() -> u64 {
        mem::size_of::<T>() as u64
    }

    pub fn slice<N: Into<u64>>(&self, n: N) -> BufferSlice {
        let bytes = n.into() * (mem::size_of::<T>()) as u64;
        return self.buffer.slice(..bytes);
    }

    pub fn queue_write(&mut self, data: &[T], queue: &Queue) {
        let data = bytemuck::cast_slice(data);
        queue.write_buffer(&self.buffer, 0, data);
    }
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
    fn id(&self) -> Id {
        return self.id;
    }
    fn sibling<H: Hash>(self, value: H) -> Self {
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

impl<'a> NodeType for Text<'a> {}
impl<'a> TextTrait for Text<'a> {}

impl NodeType for Stack {}
impl ParentTrait for Stack {}

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

impl Xy<f32> {
    fn update_for_child(&mut self, child_size: Xy<f32>, stack: Option<Stack>) {
        match stack {
            None => {
                for axis in [X, Y] {
                    if child_size[axis] > self[axis] {
                        self[axis] = child_size[axis];
                    }
                }
            },
            Some(stack) => {
                let (main, cross) = (stack.axis, stack.axis.other());

                self[main] += child_size[main];
                if child_size[cross] > self[cross] {
                    self[cross] = child_size[cross];
                }
            },
        }
    }
    fn update_for_content(&mut self, child_size: Xy<f32>, _stack: Option<Stack>) {
        for axis in [X, Y] {
            if child_size[axis] > self[axis] {
                self[axis] = child_size[axis];
            }
        }
    }

}

pub trait MeasureText {
    fn measure_text_pixels(&self) -> Xy<f32>;
}
impl MeasureText for GlyphonBuffer {
    fn measure_text_pixels(&self) -> Xy<f32> {
        let layout_runs = self.layout_runs();
        let mut run_width: f32 = 0.;
        let line_height = self.lines.len() as f32 * self.metrics().line_height;
        for run in layout_runs {
            run_width = run_width.max(run.line_w);
        }
        return Xy::new(run_width.ceil(), line_height)
    }
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
