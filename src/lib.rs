//! Keru is an experimental Graphical User Interface library.
//! 
//! It is in active development and it's not ready to be used. Many features are missing or half-baked.
//! 
//! Keru offers a declarative API similar to immediate mode GUI libraries, but it is not immediate mode.
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
//! // Define an unique identity for a node. You can also create keys dynamically.
//! #[node_key] const INCREASE: NodeKey;
//! 
//! // Add the node to the UI and set its parameters
//! self.ui.add(INCREASE)
//!     .params(BUTTON)
//!     .color(Color::RED)
//!     .text("Increase");
//! 
//! // Place nodes into the tree and define the layout
//! self.ui.v_stack().nest(|| {
//!     if self.show {
//!         self.ui.place(INCREASE);
//!         self.ui.label(self.count);
//!     }
//! });
//! 
//! // Run code in response to events
//! if self.ui.is_clicked(INCREASE) {
//!     self.count += 1;
//! }
//! #   }
//! # }
//! ```
//! 
//! Using [`NodeKeys`](NodeKey) gives more flexibility when organizing the code, but they are not required. See the "no_keys" example to see a similar counter written without [`NodeKeys`](NodeKey).
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
//! You can use the [`Ui::needs_rerender()`] to decide whether to render the GUI or skip it.
//! 
//! For a full integration example, see the Painter example. Another simpler integration example will be added in the future.
//! 
//! ## Declaring the GUI
//! 
//! To redeclare your GUI, you have to start a new GUI "tree", rerun all your GUI declaration code, then finish the tree.
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
//! self.ui.begin_tree();
//! self.ui.text("Hello World");
//! self.ui.finish_tree();
//! #
//! #   }
//! # }
//! ```
//! 
//! Note that even if you do this every frame, it doesn't mean that the GUI is re-rendering every frame, doing a full relayout on every frame, or anything of that sort. See the ["About"](about) page for more information on this point.
//! 
//! To see how the GUI declaration code works, you can check the basic example above, the Counter example, or the `paint_ui.rs` file in the painter example.
//! 
//! To summarize, for each element in the GUI, you have to perform some of these conceptual steps:
//! 
//! - optionally, define a [`NodeKey`] for the node
//! - [add](`Ui::add`) the node to the [`Ui`]
//! - set its parameters ([color](`UiNode::color`), [size](`UiNode::size`), [text](`UiNode::text`), ...)
//! - [place](Ui::place) it in the tree
//! - optionally, start a [nested](`UiPlacedNode::nest`) block
//! - optionally, [check for input](`Ui::is_clicked`) on the nodes and run code as a consequence
//! 
//! You can do these things by either calling methods directly on the main [`Ui`] struct, or by calling chained methods on the result of a previous method.
//! 
//! Methods on the [`Ui`] struct usually take a [`NodeKey`] argument to refer to a specific node.
//! 
//! ## Creating complex GUIs
//! 
//! * In dynamic GUIs, you can't identify every node with a static [`NodeKey`] in the way the examples do it.
//! 
//!     Instead, you can use the [`NodeKey::sibling()`] function to create keys dynamically at runtime.
//! 
//! * To create reusable "widgets", you can just wrap the GUI code in a function. However, it's very likely that you'll need to create a [`subtree`](subtree()) for it to make it work correctly.
//! 
//! These building blocks should be enough to create complex GUIs. But only time will tell.
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

mod interact;

pub mod basic_window_loop;
pub mod example_window_loop;

pub mod winit_mouse_events;
pub mod winit_key_events;

mod changes;
mod twin_nodes;
mod thread_local;
mod render;
mod layout;
mod text;
mod node;
mod render_rect;

mod texture_atlas;

pub use node_key_macro::node_key;

pub mod about;