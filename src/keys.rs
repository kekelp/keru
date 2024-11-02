use std::{fmt::Debug, hash::{Hash, Hasher}, marker::PhantomData};
use rustc_hash::FxHasher;

use crate::{Id, Stack};


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
        return Self {
            id,
            debug_name,
            nodetype_marker: PhantomData::<T>,
        };
    }
}

pub type NodeKey = TypedKey<Any>;

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