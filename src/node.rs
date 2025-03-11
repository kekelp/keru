use std::{fmt::Write, panic::Location};

use crate::*;

#[derive(Debug)]
pub struct Node {
    pub id: Id,
    // todo: this surely doesn't need 64 bits?
    pub depth: usize,

    pub last_layout_frame: u64,

    pub scroll: Scroll,


    // also for invisible rects, used for layout
    // Coordinates: who knows???
    pub rect: XyRect,

    // todo: isn't this just the parent's rect?
    pub clip_rect: XyRect,

    // partial result when layouting?
    // in probably in fraction of screen units or some trash 
    pub size: Xy<f32>,
    pub content_bounds: XyRect,

    pub last_proposed_sizes: ProposedSizes,

    pub(crate) relayout_chain_root: Option<NodeI>,

    // todo: get niche optimization here. I don't feel like doing that dumb thing with the zero slot again though
    pub(crate) last_rect_i: Option<usize>,
    pub(crate) last_click_rect_i: Option<usize>,
    pub(crate) last_image_rect_i: Option<usize>,

    pub text_id: Option<usize>,

    pub imageref: Option<ImageRef>,
    pub last_static_image_ptr: Option<*const u8>,

    pub last_text_ptr: usize,

    pub parent: NodeI,

    // le epic inline linked list instead of a random Vec somewhere else on the heap
    pub n_children: u16,

    pub last_child: Option<NodeI>,
    pub prev_sibling: Option<NodeI>,
    
    pub first_child: Option<NodeI>,
    pub next_sibling: Option<NodeI>,

    pub params: NodeParams,

    pub debug_key_name: &'static str,
    pub debug_location: &'static Location<'static>,

    pub children_hash: u64,

    pub is_twin: Option<u32>,

    pub last_click: f32,
    pub hovered: bool,
    pub hover_timestamp: f32,
    pub z: f32,

    pub last_cosmetic_hash: u64,
    pub last_layout_hash: u64,
    pub last_text_hash: Option<u64>,
}

impl Node {
    pub fn new(
        key: &NodeKey,
        twin_n: Option<u32>,
        debug_location: &'static Location<'static>,
    ) -> Node {
        // add back somewhere

        return Node {
            id: key.id_with_subtree(),
            depth: 0,
            rect: Xy::new_symm([0.0, 1.0]),
            clip_rect: Xy::new_symm([0.0, 1.0]),

            size: Xy::new_symm(0.5),
            content_bounds: XyRect::new_symm([0.0, 0.0]),

            last_proposed_sizes: ProposedSizes::container(Xy::new_symm(0.5)),
            text_id: None,

            scroll: Scroll::ZERO,

            imageref: None,
            last_static_image_ptr: None,
            last_text_ptr: 0,

            parent: NodeI::from(12312355), // just a wrong value which will be overwritten. it's even worse here.
            // but it's for symmetry with update_node, where all these values are old and are reset.

            n_children: 0,
            last_child: None,
            first_child: None,
            prev_sibling: None,
            next_sibling: None,

            is_twin: twin_n,
            params: NodeParams::const_default(),
            debug_key_name: key.debug_name(),
            debug_location: debug_location,
            hover_timestamp: f32::MIN,
            hovered: false,
            last_click: f32::MIN,
            z: 0.0,
            last_rect_i: None,
            last_click_rect_i: None,
            last_image_rect_i: None,
            relayout_chain_root: None,
            children_hash: EMPTY_HASH,
            last_layout_frame: 0,

            last_cosmetic_hash: 0,
            last_layout_hash: 0,
            last_text_hash: None,
        };
    }
}

impl Ui {
    pub(crate) fn node_debug_name_fmt_scratch(&mut self, i: NodeI) -> &str {
        self.format_scratch.clear();
        
        if self.nodes[i].debug_key_name != "" {
            let _ = write!(&mut self.format_scratch, "{} ", self.nodes[i].debug_key_name);

            if let Some(twin_n) = self.nodes[i].is_twin {
                let _ = write!(&mut self.format_scratch, "(twin #{})", twin_n );
            }
        }
        let _ = write!(&mut self.format_scratch, "[{}]", self.nodes[i].debug_location );

        return &self.format_scratch;
    }
}
impl Node {
    pub(crate) fn debug_name(&self) -> String {
        let mut result = String::new();
        
        if self.debug_key_name != "" {
            write!(result, "{} ", self.debug_key_name).unwrap();
            
            if let Some(twin_n) = self.is_twin {
                write!(result, "(twin #{})", twin_n).unwrap();
            }
        }
        
        write!(result, "[{}]", self.debug_location).unwrap();
        
        return result;
    }
}


// a dummy node value to fill up the zero slot, so that 
pub const ZERO_NODE_DUMMY: Node = Node {
    id: NODE_ROOT_ID,
    depth: 0,
    rect: Xy::new_symm([0.0, 1.0]),
    clip_rect: Xy::new_symm([0.0, 1.0]),

    size: Xy::new_symm(1.0),
    content_bounds: XyRect::new_symm([0.0, 0.0]),

    last_proposed_sizes: ProposedSizes::container(Xy::new_symm(1.0)),

    scroll: Scroll::ZERO,
    text_id: None,

    imageref: None,
    last_static_image_ptr: None,
    last_text_ptr: 0,


    parent: NodeI::from(91359),

    n_children: 0,
    last_child: None,
    first_child: None,
    prev_sibling: None,
    next_sibling: None,

    is_twin: None,

    params: NODE_ROOT_PARAMS,
    debug_key_name: "ZERO_NODE_DUMMY",
    debug_location: Location::caller(),
    hover_timestamp: f32::MIN,
    hovered: false,

    last_click: f32::MIN,
    z: -10000.0,
    last_rect_i: None,
    last_click_rect_i: None,
    last_image_rect_i: None,
    relayout_chain_root: None,
    children_hash: EMPTY_HASH,
    last_layout_frame: 0,

    last_cosmetic_hash: 0,
    last_layout_hash: 0,
    last_text_hash: None,
};

pub const ROOT_I: NodeI = NodeI::from(1);

pub const NODE_ROOT_ID: Id = Id(0);
pub const NODE_ROOT: Node = Node {
    id: NODE_ROOT_ID,
    depth: 0,
    rect: Xy::new_symm([0.0, 1.0]),
    clip_rect: Xy::new_symm([0.0, 1.0]),

    size: Xy::new_symm(1.0),
    content_bounds: XyRect::new_symm([0.0, 0.0]),

    last_proposed_sizes: ProposedSizes::container(Xy::new_symm(1.0)),

    scroll: Scroll::ZERO,
    text_id: None,

    imageref: None,
    last_static_image_ptr: None,
    last_text_ptr: 0,


    parent: NodeI::from(13354246),

    n_children: 0,
    last_child: None,
    first_child: None,
    prev_sibling: None,
    next_sibling: None,

    is_twin: None,

    params: NODE_ROOT_PARAMS,
    debug_key_name: "Root",
    debug_location: Location::caller(),

    hover_timestamp: f32::MIN,
    hovered: false,

    last_click: f32::MIN,
    z: -10000.0,
    last_rect_i: None,
    last_click_rect_i: None,
    last_image_rect_i: None,
    relayout_chain_root: None,
    children_hash: EMPTY_HASH,
    last_layout_frame: 0,

    last_cosmetic_hash: 0,
    last_layout_hash: 0,
    last_text_hash: None,
};