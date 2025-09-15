//! Keru is an experimental graphical user interface library.
//! 
//! # Code Example
//! 
//! ```no_run
//! # use keru::example_window_loop::*;
//! # use keru::*;
//! # #[derive(Default)]
//! # pub struct State {
//! #     pub count: i32,
//! # }
//! # fn update_ui(state: &mut State, ui: &mut Ui) {
//! // Define a unique identity for the button
//! #[node_key] const INCREASE: NodeKey;
//! 
//! // Create a NodeParams struct describing a button
//! let increase_button = BUTTON
//!     .color(Color::RED)
//!     .text("Increase")
//!     .key(INCREASE);
//! 
//! // Place the nodes into the tree and define the layout
//! ui.v_stack().nest(|| {
//!     ui.add(increase_button);
//!     ui.label(&state.count.to_string());
//! });
//! 
//! // Change the state in response to events
//! if ui.is_clicked(INCREASE) {
//!     state.count += 1;
//! }
//! // `is_clicked()` can be also called as a chained method after `ui.add(increase_button)`.
//! // In that case, using a key wouldn't be necessary.
//! # }
//! ```
//! 
//! See the `counter_small` example in the repository for a full working version of this code. 
//! 
//! 
//! ## Window Loop
//! 
//! Keru is meant to be used as part of a regular `winit`/`wgpu` window loop managed by the library user, as shown in the `window_loop` example in the repository. However, it also includes a [one-line window loop](example_window_loop::run_example_loop) that can be used for quick experimentation. 
//! 
//! Once you have a window loop, you can create a [`Ui`] struct and store it in your main program state.
//! 
//! ## Building the GUI
//! 
//! Every frame, start a new GUI frame, rerun all your GUI building code, then finish the frame.
//! 
//! ```rust
//! # use keru::*;
//! # fn declare_ui(ui: &mut Ui) {
//! #
//! ui.begin_frame();
//! ui.v_stack().nest(|| {
//!     ui.label("Hello");
//!     ui.label("World");
//! });
//! ui.finish_frame();
//! #
//! # }
//! ```
//! 
//! The [`Ui`] struct retains the state of the whole GUI, so even if you do this on every frame, it doesn't mean that the GUI is rerendering or doing a full relayout every time. The library can detect differences and apply only the minimal updates or partial relayouts needed.
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
//! * To create reusable "components", you can just wrap the GUI building code in a function, like the builtin convenience functions like [`Ui::label()`] do. If the code uses unique [`NodeKeys`](NodeKey), however, you'll need to wrap it in a [`subtree`](Ui::subtree()).
//! 
//!     This allows multiple calls to the same component function to reuse the same key multiple times without conflicts.
//! 
//! * The [`Ui::reactive()`] function provides an experimental way to improve performance in complex GUIs with many independent components.
//! 

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

mod component;
pub use component::*;

mod components;
pub use components::*;

mod interact;
pub(crate) use crate::interact::*;

pub mod basic_window_loop;
pub mod example_window_loop;

pub mod winit_mouse_events;
pub mod winit_key_events;

mod thread_local;

pub use textslabs::{TextStyle2 as TextStyle, FontWeight, FontStyle, LineHeight, FontStack, ColorBrush, StyleHandle, with_clipboard};

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

pub use keru_macros::*;

pub(crate) use Axis::*;
