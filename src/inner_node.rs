use std::{fmt::Write, hash::Hash, panic::Location};
use glam::Vec2;

use crate::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ImageSourceId {
    StaticPtr(*const u8),
    PathHash(u64),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Transform {
    pub offset: Vec2,
    pub scale: f32,
}
impl Transform {
    pub const IDENTITY: Transform = Transform {
        offset: Vec2::ZERO,
        scale: 1.0,
    };
}

#[derive(Debug)]
pub struct InnerNode {
    pub id: Id,
    pub original_key: NodeKey, // Without subtree
    pub depth: usize,

    pub last_layout_frame: u64,
    pub frame_added: u64,
    pub last_frame_touched: u64,

    pub scroll: Scroll,

    // Accumulated transform from all parents, used for rendering and hit testing
    pub accumulated_transform: Transform,

    pub real_rect: XyRect,
    pub expected_final_rect: XyRect,

    // todo: should try to get rid of some of these stored rects that are basically just partial results in the layout process.
    // todo: or at least make a struct to hide them.
    pub local_layout_rect: XyRect,
    pub local_animated_rect: XyRect,
    pub content_bounds: XyRect,
    // could maybe be passed down while traversing instead of stored.
    pub clip_rect: XyRect,
    // sort of a partial result compared to expected_final_rect.
    pub layout_rect: XyRect,    
    // this is sort of a partial result, but might be necessary because of the two-pass size, position layout.
    pub size: Xy<f32>,
    // partial result, but used for partial relayouts.
    pub last_proposed_sizes: ProposedSizes,

    // Enter or exit animation can be a fuzzy concept, because what if the node gets relayouted to a different position/state before the animation is over? The animation would be "extended" and only end what the node settles in the new final position. Even if at that point it's a mix between an enter/exit animation and a regular interpolation one.
    pub enter_animation_still_going: bool,
    pub exit_animation_still_going: bool,

    pub relayout_chain_root: Option<NodeI>,

    pub text_i: Option<TextI>,

    pub imageref: Option<ImageRef>,
    pub last_image_source: Option<ImageSourceId>,

    pub last_text_ptr: usize,

    pub parent: NodeI,

    // doesn't include lingering children.
    pub n_children: u16,

    pub last_child: Option<NodeI>,
    pub prev_sibling: Option<NodeI>,

    pub old_first_child: Option<NodeI>,
    pub old_next_sibling: Option<NodeI>,

    pub first_child: Option<NodeI>,
    pub next_sibling: Option<NodeI>,

    pub first_hidden_child: Option<NodeI>,
    pub next_hidden_sibling: Option<NodeI>,

    pub params: Node,

    pub debug_location: &'static Location<'static>,

    pub is_twin: Option<u32>,

    pub last_click: f32,
    pub hovered: bool,
    pub hover_timestamp: f32,
    pub z: f32,

    pub last_cosmetic_hash: u64,
    pub last_layout_hash: u64,
    pub last_text_hash: Option<u64>,

    pub can_hide: bool,
    pub currently_hidden: bool,

    // only kept around until the exit animation is done.
    pub exiting: bool,
}

impl InnerNode {
    /// Get the current animated rect position
    pub fn get_animated_rect(&self) -> XyRect {
        // let mut final_rect = self.rect;
        
        // // todo: move and get self.sys.global_animation_speed
        // let speed = 0.3 * self.params.animation.speed;

        // let elapsed = current_time - self.animation_start_time;
        // let duration = 0.1 / speed;
        
        // if elapsed < duration {
        //     let t = elapsed / duration;
        //     // Quadratic ease-out
        //     let ease_t = 1.0 - (1.0 - t) * (1.0 - t);
            
        //     // Interpolate the offset from its starting value to zero
        //     let current_offset_x = ease_t * self.target_offset.x + self.animation_offset.x * (1.0 - ease_t);
        //     let current_offset_y = ease_t * self.target_offset.y + self.animation_offset.y * (1.0 - ease_t);
            
        //     // Apply the interpolated offset to the base rect
        //     final_rect.x[0] += current_offset_x;
        //     final_rect.x[1] += current_offset_x;
        //     final_rect.y[0] += current_offset_y;
        //     final_rect.y[1] += current_offset_y;
        // }
        
        // // Add the cumulative parent animation offset
        // final_rect.x[0] += self.cumulative_parent_animation_offset_delta.x;
        // final_rect.x[1] += self.cumulative_parent_animation_offset_delta.x;
        // final_rect.y[0] += self.cumulative_parent_animation_offset_delta.y;
        // final_rect.y[1] += self.cumulative_parent_animation_offset_delta.y;
        
        self.real_rect
    }

    pub fn new(
        key: &NodeKey,
        twin_n: Option<u32>,
        debug_location: &'static Location<'static>,
        current_frame: u64,
    ) -> InnerNode {
        // add back somewhere

        return InnerNode {
            expected_final_rect: Xy::new_symm([0.0, 1.0]),
            exit_animation_still_going: false,
            enter_animation_still_going: false,
            id: key.id_with_subtree(),
            original_key: *key,
            depth: 0,
            layout_rect: Xy::new_symm([0.0, 1.0]),
            real_rect: Xy::new_symm([0.0, 1.0]),
            local_layout_rect: Xy::new_symm([0.0, 0.0]),
            local_animated_rect: Xy::new_symm([0.0, 0.0]),
            clip_rect: Xy::new_symm([0.0, 1.0]),

            size: Xy::new_symm(0.5),
            content_bounds: XyRect::new_symm([0.0, 0.0]),

            last_proposed_sizes: ProposedSizes::container(Xy::new_symm(0.5)),
            text_i: None,

            scroll: Scroll::ZERO,

            accumulated_transform: Transform::IDENTITY,

            imageref: None,
            last_image_source: None,
            last_text_ptr: 0,

            parent: NodeI::from(12312355), // just a wrong value which will be overwritten. it's even worse here.
            // but it's for symmetry with update_node, where all these values are old and are reset.

            n_children: 0,
            last_child: None,
            first_child: None,

            old_first_child: None,
            old_next_sibling: None,        

            prev_sibling: None,
            next_sibling: None,

            first_hidden_child: None,
            next_hidden_sibling: None,
        
            is_twin: twin_n,
            params: Node::const_default(),
            debug_location,
            hover_timestamp: f32::MIN,
            hovered: false,
            last_click: f32::MIN,
            z: 0.0,

            relayout_chain_root: None,
            last_layout_frame: 0,
            frame_added: current_frame,
            last_frame_touched: current_frame,

            last_cosmetic_hash: 0,
            last_layout_hash: 0,
            last_text_hash: None,

            can_hide: false,
            exiting: false,
            currently_hidden: false,
        };
    }
}

impl Ui {
    pub(crate) fn node_debug_name_fmt_scratch(&mut self, i: NodeI) -> &str {
        self.format_scratch.clear();
        
        if !self.nodes[i].original_key.debug_name().is_empty() {
            let _ = write!(&mut self.format_scratch, "{} ", self.nodes[i].original_key.debug_name());

            if let Some(twin_n) = self.nodes[i].is_twin {
                let _ = write!(&mut self.format_scratch, "(twin #{})", twin_n );
            }
        }
        let _ = write!(&mut self.format_scratch, "[{}]", self.nodes[i].debug_location );

        return &self.format_scratch;
    }
}
impl InnerNode {
    pub(crate) fn debug_name(&self) -> String {
        let mut result = String::new();
        
        if !self.original_key.debug_name().is_empty() {
            write!(result, "{} ", self.original_key.debug_name()).unwrap();
            
            if let Some(twin_n) = self.is_twin {
                write!(result, "(twin #{})", twin_n).unwrap();
            }
        }
        
        write!(result, "[{}]", self.debug_location).unwrap();
        
        return result;
    }
}


// A dummy node value to fill up the zero slot, so that the arena can be indexed by NonZero values. 
pub const ZERO_NODE_DUMMY: InnerNode = const {
    let mut node = NODE_ROOT;
    node.original_key = NodeKey::new(NODE_ROOT_ID, "Zero node dummy");
    node.debug_location = Location::caller();
    node
};

pub const ROOT_I: NodeI = NodeI::from(1);

pub const NODE_ROOT_ID: Id = Id(0);
pub const NODE_ROOT: InnerNode = InnerNode {
    expected_final_rect: Xy::new_symm([0.0, 1.0]),
    exit_animation_still_going: false,
    enter_animation_still_going: false,
    id: NODE_ROOT_ID,
    original_key: NodeKey::new(NODE_ROOT_ID, "Root"),
    depth: 0,
    layout_rect: Xy::new_symm([0.0, 1.0]),
    real_rect: Xy::new_symm([0.0, 1.0]),
    local_layout_rect: Xy::new_symm([0.0, 1.0]),
    local_animated_rect: Xy::new_symm([0.0, 1.0]),
    clip_rect: Xy::new_symm([0.0, 1.0]),

    size: Xy::new_symm(1.0),
    content_bounds: XyRect::new_symm([0.0, 0.0]),

    last_proposed_sizes: ProposedSizes::container(Xy::new_symm(1.0)),

    scroll: Scroll::ZERO,

    accumulated_transform: Transform::IDENTITY,

    text_i: None,

    imageref: None,
    last_image_source: None,
    last_text_ptr: 0,

    // The root node is his own parent. This can be nice sometimes but it would probably be better to not use it.
    parent: ROOT_I,

    n_children: 0,
    last_child: None,
    first_child: None,
    prev_sibling: None,
    next_sibling: None,

    old_first_child: None,
    old_next_sibling: None,

    first_hidden_child: None,
    next_hidden_sibling: None,

    is_twin: None,

    params: NODE_ROOT_PARAMS,
    debug_location: Location::caller(),

    hover_timestamp: f32::MIN,
    hovered: false,

    last_click: f32::MIN,
    z: -10000.0,

    relayout_chain_root: None,
    last_layout_frame: 0,
    frame_added: 0,
    last_frame_touched: u64::MAX,

    last_cosmetic_hash: 0,
    last_layout_hash: 0,
    last_text_hash: None,

    can_hide: false,

    exiting: false,
    currently_hidden: false,

};