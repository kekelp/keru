//! Keru is an experimental graphical user interface library.
//! 
//! ## Example
//! 
//! ```rust
//! # use keru::*;
//! # 
//! # pub struct State {
//! #     pub ui: Ui,
//! #     pub count: u32,
//! #     pub show: bool,
//! # }
//! # 
//! # impl State {
//! #   fn declare_ui(&mut self) {
//! # 
//! // Define an unique identity for a Ui node
//! #[node_key] const INCREASE: NodeKey;
//! 
//! // Create a `NodeParams` struct that describes the node
//! let increase_button = BUTTON
//!     .color(Color::RED)
//!     .text("Increase")
//!     .key(INCREASE); // Set its identity
//! 
//! // Add nodes to the tree and define the layout
//! if self.show {
//!     self.ui.v_stack().nest(|| {
//!         self.ui.add(increase_button);
//!         self.ui.label(self.count);
//!     });
//! }
//! 
//! // Update the state if the button is clicked
//! if self.ui.is_clicked(INCREASE) {
//!     self.count += 1;
//! }
//! #   }
//! # }
//! ```
//! 
//! ## Window Loop
//! 
//! If you just want to try out some GUI building code, you can use the one-line loop in [`example_window_loop`]. The Counter example uses this method. 
//! 
//! However, Keru is intended to be used as part of a regular `winit`/`wgpu` window loop managed by the library user. This makes it very simple to combine it with any kind of custom rendering (as long as it uses `wgpu`).
//! 
//! Once you have a window loop, you can create a [`Ui`] struct and store it in your main program state.
//! To integrate it with the window loop, you only need to do two things:
//! 
//! - When you receive a `winit` `WindowEvent`, pass it to [`Ui::window_event()`].
//! - When you receive a `WindowEvent::RedrawRequested`, update your GUI, then call [`Ui::render()`].
//! 
//! ## Building the GUI
//! 
//! Every frame, to update the GUI, start a new GUI frame, rerun all your GUI building code, then finish the frame.
//! 
//! ```rust
//! # use keru::*;
//! # pub struct State {
//! #     pub ui: Ui,
//! # }
//! #
//! # impl State {
//! #   fn declare_ui(&mut self) {
//! #
//! self.ui.begin_frame();
//! self.ui.v_stack().nest(|| {
//!     self.ui.label("Hello");
//!     self.ui.label("World");
//! });
//! self.ui.finish_frame();
//! #
//! #   }
//! # }
//! ```
//! 
//! The [`Ui`] struct retains the state of the whole GUI, so even if you do this every frame, it doesn't mean that the GUI is re-rendering every frame, doing a full relayout on every frame, or anything of that sort.
//! See the ["About"](about) page for more information on this point.
//! 
//! 
//! * In Keru, everything is a node. Whether you want a [button](`BUTTON`), an [image](`IMAGE`), a [text element](`TEXT`), a [stack container](V_STACK), or anything else, the way is always to [`add()`](Ui::add) a node with the right [`NodeParams`].
//! 
//! * [`Ui`] has some convenience methods like [`Ui::label()`]. These are always equivalent to [`adding`](Ui::add) one or more nodes with specific [`NodeParams`].
//! 
//! * To check interactions on a node, use [`NodeParams::key()`] to associate a [`NodeKey`] to a [`NodeParams`], then call methods like [`Ui::is_clicked()`] with the same [`NodeKey`].
//! 
//! * You can use the [`NodeKey::sibling()`] function to create keys dynamically at runtime. This is useful for dynamic GUIs where you can't identify every node with a static [`NodeKey`] in the way the basic examples do it.
//! 
//! * To create reusable "components", you can just wrap the Ui code in a function, like the builtin convenience functions like [`Ui::label()`] do. If the code uses unique [`NodeKeys`](NodeKey), however, you'll need to wrap it in a [`subtree`](Ui::subtree()).
//! 
//!     This allows multiple calls to the same component function to reuse the same key multiple times without conflicts.
//! 
//! * The [`reactive()`] function provides an experimental way to improve performance in complex GUIs with many independent components.
//! 
//! 
//! ## More information
//! 
//! See the ["About"](about) page for more information about how Keru works internally, how it compares to other libraries, and more.


mod tree;
pub use tree::*;

mod ui;
pub use ui::*;

mod math;
pub use math::*;

mod library;
pub use library::*;

mod node_key;
pub use node_key::*;

mod node_params;
pub use node_params::*;

mod color;
pub use color::*;

mod subtree;
pub use subtree::*;

mod ui_node;
pub use ui_node::*;

mod observer;
pub use observer::*;

mod reactive;
pub use reactive::*;

mod theme;
pub use theme::*;

mod components;
pub use components::*;

mod interact;

pub mod basic_window_loop;
pub mod example_window_loop;

pub mod winit_mouse_events;
pub mod winit_key_events;

mod thread_local;

mod changes;
pub(crate) use crate::changes::*;
mod twin_nodes;
pub(crate) use crate::twin_nodes::*;
pub(crate) use crate::twin_nodes::RefreshOrClone::*;
pub(crate) use crate::twin_nodes::TwinCheckResult::*;

mod render;
pub(crate) use crate::render::*;
mod layout;
pub(crate) use crate::layout::*;
mod text;
pub(crate) use crate::text::*;
mod node;
pub(crate) use crate::node::*;
mod render_rect;
pub(crate) use crate::render_rect::*;
mod nodes;
pub(crate) use crate::nodes::*;

mod texture_atlas;
pub(crate) use crate::texture_atlas::*;

pub use keru_macros::node_key;

pub mod about;