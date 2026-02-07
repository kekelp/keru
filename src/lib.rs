//! Keru is an experimental graphical user interface library.
//! 
//! # Code Example
//! 
//! ```no_run
//! # use keru::*;
//! # let mut ui: Ui = unimplemented!();
//! # pub struct State {
//! #     pub count: i32,
//! # }
//! # let mut state = State { count: 0 };
//! // Define a unique identity for the button
//! #[node_key] const INCREASE: NodeKey;
//! 
//! // Create a Node struct describing a button
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
//! ```no_run
//! # use keru::*;
//! # let mut ui: Ui = unimplemented!();
//! ui.begin_frame();
//! ui.v_stack().nest(|| {
//!     ui.label("Hello");
//!     ui.label("World");
//! });
//! ui.finish_frame();
//! ```
//! 
//! The [`Ui`] struct retains the state of the whole GUI, so even if you do this on every frame, it doesn't mean that the GUI is rerendering or doing a full relayout every time. The library can detect differences and apply only the minimal updates or partial relayouts needed.
//! 
//! 
//! * In Keru, everything is a [`Node`]. Whether you want a [button](`BUTTON`), an [image](`IMAGE`), a [text element](`TEXT`), a [stack container](V_STACK), or anything else, the way is always to [`add()`](Ui::add) a node with the right values.
//! 
//! * There are also "components", like [`Slider`]. Components are added with [`Ui::add_component()`] and they are a way to wrap multiple nodes into a reusable structure. You can define custom components with the [`SimpleComponent`] and [`Component`] traits.
//! 
//! * [`Ui`] has some convenience methods like [`Ui::label()`]. These work the same way as components, but with more natural syntax.
//! 
//! * To check interactions on a node, use [`Node::key()`] to associate a [`NodeKey`] to a [`Node`], then call methods like [`Ui::is_clicked()`] with the same [`NodeKey`].
//! 
//! * The [`Ui::reactive()`] function provides an experimental way to improve performance in complex GUIs with many independent components.
//! 

mod tree;
pub use tree::*;

mod ui;
pub use ui::*;

mod math;
pub use math::*;

mod node_library;
pub use node_library::*;

mod node_key;
pub use node_key::*;

mod node;
pub use node::*;

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

mod thread_local_arena;
pub use thread_local_arena::*;

mod component_library;
pub use component_library::*;

mod interact;
pub(crate) use crate::interact::*;

pub mod thread_future;
pub mod thread_future_2;

pub mod example_window_loop;
pub mod basic_window_loop;

pub mod winit_mouse_events;
pub(crate) use crate::winit_mouse_events::SmallVec;

pub mod winit_key_events;

mod thread_local;

pub use bumpalo;

pub use keru_draw::{TextStyle2 as TextStyle, FontWeight, FontStyle, LineHeight, FontStack, ColorBrush, StyleHandle, with_clipboard};

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
mod inner_node;
pub(crate) use crate::inner_node::*;
mod nodes;
pub(crate) use crate::nodes::*;

pub use keru_macros::*;

pub(crate) use Axis::*;

pub use keru_draw::textslabs;