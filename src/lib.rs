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
//! // Create a NodeParams struct that describes the node
//! let increase_button = BUTTON
//! .color(Color::RED)
//! .text("Increase")
//! .key(INCREASE); // Set its identity
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
//! - When you receive a `WindowEvent::RedrawRequested`, redeclare your GUI, then call [`Ui::render()`].
//! 
//! ## Building the Ui
//! 
//! Every frame, to update the Ui, you have to start a new Ui frame, rerun all your Ui declaration code, then finish the frame.
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
//! The [`Ui`] struct retains the state of the whole Ui, so even if you do this every frame, it doesn't mean that the GUI is re-rendering every frame, doing a full relayout on every frame, or anything of that sort.
//! See the ["About"](about) page for more information on this point.
//! 
//! To see how the Ui declaration code works, see the basic example above, or the Counter example.
//! 
//! * In general, all you have to do is [`add`](Ui::add) a node with the right [`NodeParams`], and if you want to check interactions on it, call methods like [`Ui::is_clicked()`] with the same [`NodeKey`].
//! 
//! * In dynamic Uis, you can't identify every node with a static [`NodeKey`] in the way the examples do it.
//! 
//!     Instead, you can use the [`NodeKey::sibling()`] function to create keys dynamically at runtime.
//! 
//! * To create reusable "components", you can just wrap the GUI code in a function. If the code uses unique [`NodeKeys'](NodeKey), however, you'll need to wrap the code in [`subtree`](subtree()) to be able to reuse the same key multiple times without conflicts.
//! 
//! These building blocks should be enough to create complex interfaces. But only time will tell.
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

mod param_library;
pub use param_library::*;

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

mod interact;

pub mod basic_window_loop;
pub mod example_window_loop;

pub mod winit_mouse_events;
pub mod winit_key_events;

mod thread_local;

mod changes;
mod twin_nodes;
mod render;
mod layout;
mod text;
mod node;
mod render_rect;
mod nodes;
pub(crate) use crate::nodes::*;

mod texture_atlas;

pub use keru_macros::node_key;

pub mod about;