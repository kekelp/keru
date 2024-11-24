use crate::*;

use texture_atlas::ImageRef;


#[derive(Debug)]
pub struct Node {
    pub id: Id,
    pub depth: usize,

    pub last_layout_frame: u64,

    // also for invisible rects, used for layout
    // Coordinates: who knows???
    pub rect: XyRect,

    // partial result when layouting?
    // in probably in fraction of screen units or some trash 
    pub size: Xy<f32>,

    pub(crate) relayout_chain_root: Option<usize>,

    pub(crate) last_rect_i: usize,

    pub text_id: Option<usize>,

    pub imageref: Option<ImageRef>,
    pub last_static_image_ptr: Option<*const u8>,
    pub last_static_text_ptr: Option<*const u8>,

    pub parent: usize,

    // le epic inline linked list instead of a random Vec somewhere else on the heap
    // todo: Option<usize> is 128 bits, which is ridicolous. Use a NonMaxU32 or something
    pub n_children: u16,

    pub last_child: Option<usize>,
    pub prev_sibling: Option<usize>,
    // prev_sibling is never used so far.
    // at some point I was iterating the children in reverse for z ordering purposes, but I don't think that makes any difference.
    // pub prev_sibling: Option<usize>,
    pub params: NodeParams,

    pub debug_name: &'static str,

    pub old_children_hash: u64,

    pub is_twin: Option<u32>,

    pub last_hover: f32,
    pub last_click: f32,
    pub z: f32,

    pub needs_cosmetic_update: bool,
    pub needs_partial_relayout: bool,
    pub last_cosmetic_params_hash: u64,
    pub last_layout_params_hash: u64,
}

impl Node {
    pub fn new(
        key: &NodeKey,
        twin_n: Option<u32>,
    ) -> Node {
        // add back somewhere

        return Node {
            id: key.id(),
            depth: 0,
            rect: Xy::new_symm([0.0, 1.0]),
            size: Xy::new_symm(10.0),
            text_id: None,

            imageref: None,
            last_static_image_ptr: None,
            last_static_text_ptr: None,

            parent: 0, // just a wrong value which will be overwritten. it's even worse here.
            // but it's for symmetry with update_node, where all these values are old and are reset.

            n_children: 0,
            last_child: None,
            prev_sibling: None,

            is_twin: twin_n,
            params: NodeParams::const_default(),
            debug_name: key.debug_name,
            last_hover: f32::MIN,
            last_click: f32::MIN,
            z: 0.0,
            last_rect_i: 0,
            relayout_chain_root: None,
            old_children_hash: EMPTY_HASH,
            last_layout_frame: 0,

            last_cosmetic_params_hash: 0,
            last_layout_params_hash: 0,
            needs_cosmetic_update: false,
            needs_partial_relayout: false,        
        };
    }

    pub fn debug_name(&self) -> String {
        let debug_name = match self.is_twin {
            Some(n) => format!("{} (twin #{})", self.debug_name, n),
            None => self.debug_name.to_string(),
        };
        return debug_name;
    }
}


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

    n_children: 0,
    last_child: None,
    prev_sibling: None,

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

    needs_cosmetic_update: false,
    needs_partial_relayout: false,
    last_cosmetic_params_hash: 0,
    last_layout_params_hash: 0,
};