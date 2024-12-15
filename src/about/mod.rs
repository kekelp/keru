//! Some extra information about the library.
//! 
//! # What does it do?
//! 
//! Keru has a declarative API similar to immediate mode GUI libraries. However, it is not immediate mode.
//! 
//! This section will try to explain what the library does with an annotated example.
//! 
//! The code to build the GUI looks like this:
//! 
//! ```rust
//! # use keru::*;
//! # pub struct State {
//! #     pub ui: Ui,
//! #     pub count: u32,
//! #     pub show: bool,
//! # }
//! # 
//! # impl State {
//! #   fn declare_ui(&mut self) {
//! # 
//! // Define unique keys/ids for  ui elements
//! #[node_key] const INCREASE: NodeKey;
//! #[node_key] const DECREASE: NodeKey;
//! #[node_key] const SHOW: NodeKey;
//! 
//! self.ui.add(INCREASE) // Create an element
//!     .params(BUTTON) // Set style and properties
//!     .static_text("Increase"); // Set text
//! 
//! self.ui.add(SHOW)
//!     .params(BUTTON)
//!     .static_text("Show Counter");
//! 
//! self.ui.add(DECREASE)
//!     .params(BUTTON)
//!     .static_text("Decrease");
//! 
//! // Place the nodes in the ui tree.
//! // The nesting of these calls will define the layout.
//! self.ui.v_stack().nest(|| { 
//!     self.ui.place(SHOW);
//!     if self.show {
//!         self.ui.place(INCREASE);
//!         self.ui.label(self.count); // This one doesn't need an id, we can use a shorthand
//!         self.ui.place(DECREASE);
//!     }
//! });
//! 
//! if self.ui.is_clicked(SHOW) { // Use the unique key to refer to a specific node
//!     self.show = !self.show; // Update the state
//! }
//! if self.ui.is_clicked(INCREASE) {
//!     self.count += 1;
//! }
//! if self.ui.is_clicked(DECREASE) {
//!     self.count -= 1;
//! }
//! # 
//! #   }
//! # }
//! # 
//! ```
//! 
//! This code is run either on every frame or on every "cycle" (user interaction/external event) [^1].
//! 
//! [^1]: In most "slow" UI applications, the UI can "go to sleep" and do nothing until user input or an external event wakes it up. Even true immediate mode GUIs like `egui` can do this. "Cycle" refers to one of these "awake frames".
//! 
//! Every time, you re-declare the whole GUI tree. However, the tree is **fully retained** across frames. If a call like `add` finds that the corresponding node already exists in the tree, it will either update it, or do nothing.
//! 
//! If a function causes a change in the tree, it will also tell the [`Ui`] to do **partial relayouts** and **partial updates** to the render data, propagating the change to the screen.
//! 
//! To be more precise:
//! 
//! ```rust
//! # use keru::*;
//! # pub struct State {
//! #     pub ui: Ui,
//! #     pub count: u32,
//! #     pub show: bool,
//! # }
//! # 
//! # #[node_key] const INCREASE: NodeKey;
//! # #[node_key] const DECREASE: NodeKey;
//! # #[node_key] const SHOW: NodeKey;
//! # 
//! # impl State {
//! #   fn declare_ui(&mut self) {
//! # 
//! // If they are called with different arguments than in the last frame,
//! //  these calls will change the node's parameters (size, color, text, etc)
//! self.ui.add(DECREASE)
//!     .params(BUTTON)
//!     .static_text("Decrease");
//! 
//! // when a node is placed onto the tree,
//! //  the library will compare the new params to the ones from last frame.
//! self.ui.v_stack().nest(|| {
//!     // For example, if the SHOW node's size has changed, 
//!     //  it will schedule a **partial** relayout
//!     //  starting at this position in the tree.
//!     // If the color has changed, 
//!     //  it will schedule to update the render data for that **single** rectangle.
//!     self.ui.place(SHOW);
//!     // Here, depending on `self.show`, some nodes 
//!     //  might be included or excluded from the tree.
//!     // If the value of `self.show` is different from last frame, 
//!     //  then, at the end of its `nest()` block,
//!     //  the parent will notice that its children changed.
//!     // In that case, it will schedule a partial relayout.
//!     if self.show {
//!         self.ui.place(INCREASE);
//!         self.ui.label(self.count);
//!         self.ui.place(DECREASE);
//!     }
//! });
//! 
//! // At the end, the library will do all the partial relayouts and updates needed.
//! # 
//! #   }
//! # }
//! ```
//! 
//! ### Isn't this still like immediate mode?
//! 
//! A bit, in the sense that there's still some code that gets run on every frame/cycle. This doesn't happen only in immediate mode systems, but also in many "reactive" systems like React [^2].
//! 
//! [^2]: There are also some "true reactive" systems like Floem and SwiftUi, which *don't* run update code on every frame. Instead, they effectively inject "setters" in front of all your state, and then run all app logic and UI updates in callbacks triggered by those setters.
//! 
//! 
//! However, it's definitely not like immediate mode, in the sense that:
//! - the node tree is always retained at all times!
//! - there is no "state tearing" (I think)
//! - you are not forced to write a button's effect immediately after adding it
//! 
//! And most importantly:
//! - there is no need to do a full relayout every frame/cycle
//! - there is no need to recreate the render data from scratch every frame/cycle
//! 
//! What matters is *how much stuff* you're doing every frame/cycle (and how optimized the code is).
//! 
//! In the painter example, the UI redeclaration code takes about 20 μs.
//! 
//! It's also important to remember that this has nothing to do with the performance of the program when idle: see footnote [^1].
//! 
//! ### Reactivity at home
//! 
//! From this starting point, there's still some room for "reactivity". After all, the only difference between what Keru does and a true reactive system (see footnote [^2]) is the redeclaration code that we run every frame/cycle. It doesn't do that much work, but it does have to hash a fair amount of [`NodeParams`] and strings to watch for changes.
//! 
//! The user could still optionally wrap or annotate some of his state with something that keeps track of when it changes, and pass that information to the library.
//! 
//! Then the library could either skip all the hashing/diffing operations, or maybe skip running the redeclaration code completely. I am currently trying out a few different approaches to this.
//! 
//! # Advantages
//! 
//! This is a list of advantages that I think Keru's approach gives over other other commonly seen approaches. 
//! 
//! - **Own your window loop and rendering**
//! 
//!     You can use a regular `winit`/`wgpu` render loop and call Keru as a library. It doesn't take any control away from you.
//!     This makes it easy to compose the ui with custom rendering, both "below" and "inside" the GUI. This is demonstrated in the painter example.
//! 
//!     The library also includes a basic premade render loop if you just want to experiment. See the counter example.
//! 
//!     `egui` and Dear Imgui accomplish this in a more hardcore way by being compatible with most windowing and render libraries on earth. For now, Keru just supports `winit` and `wgpu`.
//! 
//! -------
//! 
//! - **Regular Rust Code**
//! 
//!     You write Regular Rust Code™. It's always clear when your code gets executed and, hopefully, what it does.
//! 
//!     - you don't have to write as your code inside a big proc macro
//!     - you don't have to use a domain specific language
//!     - you don't have to write all your code as part of the impl of some trait that the runtime executes who-knows-when
//!     - you don't have to write all your logic inside callbacks
//! 
//!     The original plan also included "you don't have to write all your code inside closures", but I abandoned it: you do use closures a lot in Keru, when using the [`nest`](`UiPlacedNode::nest`) function. But they're very simple closures with no arguments that get executed immediately. As close as closures can get to "just a normal block of code".
//! 
//! -------
//! 
//! - **Flexible code organization** 
//! 
//!     You should have as much freedom as possible when organizing your GUI code. You have the *option* to keep style, layout and effects of an element close to each other, but you aren't forced to do it.
//!     
//!     If you look at the examples in the repos for `gpui`, `floem` and others, you'll see that the layout is derived from the order and nesting of the functions that create the elements.
//!     But you also have to specify the style and the effect right after that call by chaining builder functions to it.
//! 
//!     The resulting code is very strange and hard to read, in my opinion. In particular, it's very hard to follow the nesting structure that defines the layout, since it's mixed with so much other stuff. Most of the clarity of the "nested calls -> layout" approach is lost.
//! 
//!     In Keru, you can always refer to a node from anywhere in your code by using the unique [`NodeKey`]. So you can split the layout code ([`place`](Ui::place) and [`nest`](UiPlacedNode::nest)) or the effects ([`is_clicked`](`Ui::is_clicked`), etc.) from the rest. This is the pattern used in the examples.
//! 
//! -------
//! 
//! - **Own your state**
//! 
//!     Your UI can depend on any variable that you can get a reference to, i.e. anything. You don't have to structure your state in any particular way.
//!     - You don't have to pair the state with its UI display logic (unless you want to!)
//!     - You don't have to wrap your state into observer structs or signal handlers (unless you want to, see the "[# Reactivity at home]" section)
//!     - You shouldn't get any extra borrowing or lifetime issues (unlike in closure-heavy and callback-heavy systems)
//! 
//! 
//! ## Todo
//! 
//! - Try out the sibling key system for dynamic/runtime keys with an example
//! 
//! - "Subtrees" for reusing UI functions ("widgets") without key collisions
//! 
//! - Bump the `winit` version and hope for no more breaking releases
//! 
//! - Improve the text and image API
//! 
//! - A way to create a [`Ui`] without linking it to a window loop
//! 
//! - Software renderer? Probably not happening for a while
//! 
//! - Other utilities for testing
//! 
//! - Text input, scroll areas, animations, more geometric shapes, more built-in widgets, ...
//! 
//! 
//! ## Open questions and unsolved issues
//! 
//! - "Reactivity": skipping the redeclaration code or part of it when the underlying state is known to be unchanged
//! 
//! - Less room for mistakes: [`Ui::place`] in particular can panic if used incorrectly (using the same key twice or placing a node that wasn't added). 
//!     There are ways around this, but they make the API worse in other ways. Given that [`UiNode::place`] already offers a less flexible but panic-safe alternative, it's probably fine to leave it like this.
//! 
//! - The current way of doing custom rendered UI elements can result in some alpha blending problems.
//! 
//! - Problems with winit/wgpu: takes forever to start up, resizing the window isn't smooth on some platforms ... 
//! 
//! 

// This helps with doc links.
#[allow(unused_imports)]
use crate::*;