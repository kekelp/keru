//! Keru is a Graphical User Interface library.
//! 
//! It offers a declarative API similar to immediate mode GUI libraries, but it is not immediate mode.
//! 
//! See the [`about`] page for more information about the API design, the internals, performance considerations, and more.
//! 
//! ## Example
//! 
//! ```rust
//! // Define an unique identity for this button
//! #[node_key] const INCREASE: NodeKey;
//! 
//! // Run code in response to events
//! if ui.is_clicked(INCREASE) {
//!     self.count += 1;
//! }
//! 
//! // Add nodes to the UI and set their parameters
//! ui.add(INCREASE)
//!     .params(BUTTON)
//!     .color(Color::RED)
//!     .text("Increase");
//! 
//! // Place the nodes into the tree and define the layout
//! ui.v_stack().nest(|| {
//!     if self.show {
//!         ui.place(INCREASE);
//!         ui.label(self.count); // This one doesn't need an unique key.
//!     }
//! });
//! ```
//! 
//! ## Window Loop
//! 
//! If you just want to try out some GUI building code, you can use the one-line loop in [`example_window_loop`]. The Counter example uses this method. 
//! If you do this, you can skip the rest of this section, for now.
//! 
//! However, Keru is intended to be used as part of a regular `winit`/`wgpu` window loop managed by the library user. This makes it very simple to combine it with any kind of custom rendering (as long as it uses `wgpu`), spares the library from having to re-expose a ton of window/rendering configuration options, and is generally a simpler and cleaner approach, in my opinion.
//! 
//! When building your own loop, you can still use the helper functions in the [`basic_window_loop`] module to skip most of the `winit` and `wgpu` boilerplate. The Painter example uses this method. 
//! 
//! Once you have a window loop, you can create a [`Ui`] struct and store it in your main program state.
//! The [`Ui`] struct is the central API of the library. All operations start by calling a method of [`Ui`].
//! To integrate it with the window loop, you only need to do two things:
//! 
//! - When you receive a `winit` event, pass it to [`Ui::handle_events`].
//! - When you want to render, call [`Ui::prepare`] to load the GUI data onto the GPU, then call [`Ui::render`].
//! 
//! You can use the [`Ui::needs_rerender()`] to decide whether to render the GUI or skip it.
//! 
//! For a full integration example, see the Painter example. Another simpler integration example will be added in the future.
//! 
//! ## Building the GUI
//! 
//! Then, you can use the [`Ui`] struct to build your UI.
//! 
//! Whenever you want to update your GUI, you have to start a new GUI "tree", rerun all your GUI declaration code, then finish the tree.
//! 
//! ```rust
//! self.ui.begin_tree();
//! // declare the GUI and update state
//! self.ui.finish_tree();
//! ```
//! 
//! Note that even if you do this every frame, it doesn't mean that the GUI is re-rendering every frame, doing a full relayout on every frame, or anything like that. See the [`about`] page for more information on this point.
//! 
//! To see how the GUI declaration code works, you can check the basic example above, the Counter example, or the `paint_ui.rs` file in the painter example.
//! 
//! To summarize, for each element in the GUI, you have to perform some of these conceptual steps:
//! 
//! - optionally, define a [`NodeKey`] for the node
//! - [**add**](`Ui::add`) the node to the `Ui`
//! - set its parameters ([**color**](`UiNode::color`), [**size**](`UiNode::size`), [**text**](`UiNode::text`), ...)
//! - [**place**](Ui::place) it in the tree
//! - optionally, start a [nested](`UiPlacedNode::nest`) block
//! - optionally, [check for input](`Ui::is_clicked`) on the nodes and run code as a consequence
//! 
//! You can do these things by either calling methods directly on the main [`Ui`] struct, or by calling chained methods on the result of a previous method.
//! 
//! Non-chained methods take a [`NodeKey`] argument in order to refer to the specific node.
//! 
//! 
//! ```rust
//! ui.add_anon().params(BUTTON).text("Click Here").
//! ```
//!


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

mod ui_node;
pub use ui_node::*;

mod interact;
pub use interact::*;

pub mod basic_window_loop;
pub mod example_window_loop;

mod changes;
mod twin_nodes;
mod thread_local;
mod render;
mod layout;
mod text;
mod node;
mod render_rect;

mod texture_atlas;

pub use view_derive::node_key;

pub mod about;