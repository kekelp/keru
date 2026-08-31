//! Accessibility helpers for building AccessKit nodes from text layouts.

#![cfg(feature = "accessibility")]

use std::sync::atomic::{AtomicU64, Ordering};
use accesskit::NodeId;

/// Allocates a fresh AccessKit [`NodeId`] from a reserved high range of the
/// `u64` id space, unlikely to collide with ids assigned by a host GUI library.
pub fn next_node_id() -> NodeId {
    static NEXT: AtomicU64 = AtomicU64::new(16075019835661180680);
    NodeId(NEXT.fetch_add(1, Ordering::Relaxed))
}
