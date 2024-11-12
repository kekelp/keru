use texture_atlas::ImageRef;

use crate::*;

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