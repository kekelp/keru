use std::{fmt::Debug, hash::{Hash, Hasher}};

use crate::*;


// todo: possibly split debug_name into debug_name and source_code_location, and maybe put back cfg(debug) for source_code_loc or both

/// An unique key that identifies a GUI node.
/// 
/// Usually created with the [`macro@node_key`] macro or with [`NodeKey::sibling`]:
/// 
/// ```rust
/// # use keru::*;
/// #[node_key] const UNIQUE_KEY: NodeKey;
/// ```
/// 
/// Used in many [`Ui`] methods to refer to specific nodes: for example, [`Ui::is_clicked`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NodeKey {
    id: Id,
    debug_name: &'static str,
}
impl NodeKey {
    pub(crate) fn id_with_subtree(&self) -> Id {
        
        if let Some(subtree_id) = thread_local::last_subtree() {
            let mut hasher = ahasher();
            subtree_id.hash(&mut hasher);
            self.id.hash(&mut hasher);
            return Id(hasher.finish());
        } else {
            return self.id;
        } 
    }

    /// Create "siblings" of a key dynamically at runtime, based on a hashable value.
    ///
    /// ```rust
    /// # use keru::*;
    /// #[node_key] const ROOT_COLOR_KEY: NodeKey;
    /// let colors = ["blue", "green", "violet"];
    /// for color in colors {
    ///     let color_key = ROOT_COLOR_KEY.sibling(color);
    /// }
    /// ```
    pub fn sibling<H: Hash>(self, value: H) -> Self {
        let mut hasher = ahasher();
        self.id.0.hash(&mut hasher);
        value.hash(&mut hasher);
        let new_id = hasher.finish();

        return Self {
            id: Id(new_id),
            debug_name: self.debug_name,
        };
    }

    pub fn s<H: Hash>(self, value: H) -> Self {
        self.sibling(value)
    }

    /// Create a key manually.
    /// 
    /// This is usually not needed: use the [`macro@node_key`] macro for static keys, and [`NodeKey::sibling`] for dynamic keys.
    pub const fn new(id: Id, debug_name: &'static str) -> Self {
        return Self {
            id,
            debug_name,
        };
    }

    pub const fn debug_name(&self) -> &'static str {
        return self.debug_name;
    }
}

pub type SubtreeKey = NodeKey;

