use std::{fmt::Debug, hash::{Hash, Hasher}};
use rustc_hash::FxHasher;

use crate::*;


// todo: possibly split debug_name into debug_name and source_code_location, and maybe put back cfg(debug) for source_code_loc or both
#[derive(Clone, Copy, Debug)]
pub struct NodeKey {
    pub id: Id,
    pub debug_name: &'static str,
}
impl NodeKey {
    pub(crate) fn id(&self) -> Id {
        return self.id;
    }
    /// Create "siblings" of a key dynamically at runtime, based on a hashable value. See [`NodeKey::sibling`].
    ///
    /// ```rust
    /// #[node_key] const COLOR_NODE: NodeKey;
    /// let strings = ["blue", "green", "violet"];
    /// for s in strings {
    ///     let key = COLOR_NODE.sibling(s);
    /// }
    /// ```
    pub fn sibling<H: Hash>(self, value: H) -> Self {
        let mut hasher = FxHasher::default();
        self.id.0.hash(&mut hasher);
        value.hash(&mut hasher);
        let new_id = hasher.finish();

        return Self {
            id: Id(new_id),
            debug_name: self.debug_name,
        };
    }

    pub const fn new(id: Id, debug_name: &'static str) -> Self {
        return Self {
            id,
            debug_name,
        };
    }
}
