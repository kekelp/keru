//! \[Docs\] Some extra information about the library.
//! 
//! # How does it work?
//! 
//! Keru has a declarative API similar to immediate mode GUI libraries. However, it is not immediate mode in the usual sense of the word.
//! 
//! The code to declare the GUI looks like this:
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
//! // Define unique keys for some ui elements.
//! #[node_key] const INCREASE: NodeKey;
//! #[node_key] const SHOW: NodeKey;
//! 
//! let show_button = BUTTON // Create a NodeParams value on the stack
//!     .color(Color::RED) // Set style and properties
//!     .text("Show") // Set text
//!     .key(SHOW); // Sets its identity with a unique NodeKey
//! 
//! let increase_button = BUTTON
//!     .color(Color::RED)
//!     .text("Increase")
//!     .key(INCREASE);
//! 
//! // Add the nodes to the ui tree.
//! // The nesting of these calls will define the layout.
//! self.ui.v_stack().nest(|| { 
//!     self.ui.add(show_button);
//!     if self.show {
//!         self.ui.add(increase_button);
//!         self.ui.label(self.count); // shorthand (no NodeParams, no key)
//!     }
//! });
//! 
//! if self.ui.is_clicked(SHOW) { // Use the unique key to refer to a specific node
//!     self.show = !self.show; // Update the state
//! }
//! if self.ui.is_clicked(INCREASE) {
//!     self.count += 1;
//! }
//! # 
//! #   }
//! # }
//! # 
//! ```
//! 
//! This code is run either on every frame or on every "cycle" (user interaction/external event) [^1], depending on how the `winit` loop is set up.
//! 
//! [^1]: In most "slow" UI applications, the UI can "go to sleep" and do nothing until user input or an external event wakes it up. Even true immediate mode GUIs like Egui can do this. "Cycle" refers to one of these "awake frames".
//! 
//! Every time, you re-declare the whole GUI tree. However, the tree is **fully retained** across frames.
//! 
//! If [`Ui::add()`] finds that the corresponding node already exists in the tree, and it is called with different [`NodeParams`] from last frame, it will either do a *partial* update, or do nothing.
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
//! # impl State {
//! #   fn declare_ui(&mut self) {
//! # 
//! # #[node_key] const INCREASE: NodeKey;
//! # #[node_key] const SHOW: NodeKey;
//! # 
//! # let show_button = BUTTON // Create a NodeParams value on the stack
//! #     .color(Color::RED) // Set style and properties
//! #     .text("Show") // Se text
//! #     .key(SHOW); // Sets its identity with a unique NodeKey
//! # 
//! # let increase_button = BUTTON
//! #     .color(Color::RED)
//! #     .text("Increase")
//! #     .key(INCREASE);
//! 
//! // when a node is added onto the tree,
//! // Ui will compare its new params to the ones it had last frame.
//! self.ui.v_stack().nest(|| { 
//!     // For example, if the size of the SHOW node has changed, 
//!     // it will schedule a _partial_ relayout starting 
//!     // at this position in the tree. If the color has changed, 
//!     // it will schedule to update the render data for that _single_ rectangle.
//!     self.ui.add(show_button);
//!     // Here, depending on the value of `self.show`, some nodes 
//!     // might be included or excluded from the tree.
//!     // If the value of `self.show` is different from last frame, 
//!     // then, at the end of its `nest()` block,
//!     // the parent will notice that its children changed.
//!     // In that case, it will schedule a partial relayout.
//!     if self.show {
//!         self.ui.add(increase_button);
//!         self.ui.label(self.count); // shorthand (no NodeParams, no key)
//!     }
//! });
//! 
//! // At the end, the Ui will do all the partial relayouts and updates needed.
//! # 
//! #   }
//! # }
//! # 
//! ```
//! 
//! 
//! While the GUI redeclaration code is rerun every cycle, the functions barely do anything at all, unless something changed in the underlying state.
//! 
//! If something did change, the library can check which nodes need to be updated, and can schedule only the minimal relayouts and updates needed.
//! 
//! Most of the time, all that the library needs to do is to hash some [`NodeParams`] and some text, and conclude that nothing changed. This should be very light, especially compared to a "true immediate mode" approach.
//! 
//! It's also important to remember that this has nothing to do with the performance of the program when idle: see footnote [^1].
//! 
//! However, since reactivity seems to be the current big thing, Keru provides an experimental way to skip even this work: see the ["Reactivity"](#reactivity) section.
//! 
//! # Advantages
//! 
//! This is a list of advantages that I think Keru gives over the most popular alternatives. 
//! 
//! - **Own your window loop and rendering**
//! 
//!     You can use a regular `winit`/`wgpu` render loop and call Keru as a library. It doesn't take any control away from you.
//!     This makes it easy to compose the ui with custom rendering, both "below" and "inside" the GUI. This is demonstrated in the painter example. Both the painting canvas ("below" the UI) and the color picker (inside an UI element) use custom `wgpu` rendering. 
//!     
//!     This also means that Keru is automatically compatible with ecosystem crates for things like SVG rendering, animations, video, etc (as long as they use `wgpu`) without needing to include a particular implementation within Keru. (although including one anyway would still result in a better out-of-the box experience, and probably better ergonomics).
//! 
//!     Egui and Dear Imgui accomplish this in a much more hardcore way by being compatible with most windowing and render libraries on earth, but this has many disadvantages, in addition to requiring a ton of extra work. For now, Keru just supports `winit` and `wgpu`.
//! 
//! -------
//! 
//! - **Regular Rust Code**
//! 
//!     You write Regular Rust Code™, and it's always clear when your code gets executed.
//! 
//!     - you don't have to write as your code inside a big proc macro
//!     - you don't have to use a domain specific language
//!     - you don't have to write all your code inside of a trait impl or a long-lived closure that the runtime executes on its own schedule
//!     - you don't have to write all your logic inside callbacks
//! 
//! -------
//! 
//! - **Flexible code organization** 
//! 
//!     You should have as much freedom as possible when organizing your GUI code. You have the *option* to keep style, layout and effects of an element close to each other, but you aren't forced to do so.
//!     
//!     If you look at the examples in the repos for Gpui, Floem, Egui and others, you'll see that like in Keru the layout is derived from the order and nesting of the function calls that create the elements.
//!     But you also have to specify the style and the effect right after that call by chaining builder functions to it.
//! 
//!     The resulting code is very strange and hard to read, in my opinion. In particular, it's very hard to follow the nesting structure that defines the layout, since it's mixed with so much other stuff. Most of the clarity of the approach is lost.
//! 
//!     In Keru, you can always separate the layout code ([`add`](Ui::add) and [`nest`](UiParent::nest)), the styling (creating a [`NodeParams`] struct) and the effects ([`is_clicked`](`Ui::is_clicked`), etc.) from each other.
//! 
//!     But if you do prefer to keep them together, you can still put them all next to each other.
//! 
//! -------
//! 
//! - **Own your state**
//! 
//!     Your UI can depend on any variable that you can get a reference to, i.e. anything. You don't have to structure your state in any particular way.
//!     - You don't have to pair the state with its UI display logic (unless you want to!)
//!     - You don't have to wrap your state into observer structs or signal handlers (unless you want to: see the ["Reactivity"](#reactivity) section)
//!     - You shouldn't get any extra borrowing or lifetime issues (unlike in closure-heavy and callback-heavy approaches)
//! 
//! -------
//! 
//! - **It's not immediate mode!**
//! 
//!     From the public API, Keru might seem very similar to Egui or other immediate mode libraries, and indeed many of the advantages listed so far also apply to Egui.
//! 
//!     However, Keru is **not immediate mode!** The declarative API might look similar, but inside, there is a fully retained node tree. This is enough to avoid most of the traditional disadvantages of immediate mode GUI:
//!     
//!     - Layout isn't any more difficult than with any traditional retained mode GUI.
//!     - There is no need to do a full relayout on every frame. When few things change, Keru does partial updates and relayouts.
//!     - Integrating accessibility tools **shouldn't** be any more difficult than with any traditional retained mode GUI, but I haven't tried this yet.
//! 
//!     Keru also tries to improve in other areas where Egui is (in my opinion) janky or inconvenient:
//! 
//!     - The API is less fragmented: all operations are methods on the main [`Ui`] struct, as opposed to a mixture of methods and associated functions on `Context`, `Ui`, `Window`, `Frame`, ... in Egui.
//!     - There is no interior mutability or locking hidden inside the [`Ui`], unlike Egui's `Context`.
//!     - There's probably a lot less dynamic allocations, though I haven't checked this rigorously. Keru barely does any dynamic allocations at all.
//!     - Egui's closure pattern for nesting is substituted by a much simpler one (see [`UiParent::nest()`]).   
//! 
//!         Because Keru's closure doesn't borrow or capture anything, it's a lot less prone to borrow checker errors, and thus gives more flexibility in how the user can organize their code.
//!         
//!         To make this pattern possible, Keru keeps track of the nested [nest()][`UiParent::nest()`] calls in thread-local variables. The nesting of function calls is an intrinsically thread-local concept, so this feels like a natural step.
//! 
//! 
//! --------
//! 
//! Of course there are some disadvantages as well. I think the main one is having to deal explicitly with keys and subtrees. This might be made mostly optional by adding a way to get interaction results with a chained method directly on the result of [`Ui::add()`]. This would be a bit awkward currently, but doable. 
//! 
//! With this approach, the library has to hash [`NodeParams`] objects and figure out if anything needs to be updated. For this reason, it has a theoretical performance disadvantage compared to "true reactive" approaches like Floem of SwiftUi, but this is not really guaranteed to cause a measurable difference in practice.
//! 
//! ## Reactivity
//! 
//! There's still some room to add "reactivity" (in the Floem/SwiftUI sense) on top of the library as described so far. Currently, this is offered by the [`reactive()`] function. This is the main idea:
//! 
//! - The user can opt in to writing a block of UI code in a "reactive block". For this to work properly, the code has to depend only on a small set of variables: for example: a color picker widget only depends on the value of the color.
//! 
//! - The user does something to track whether or not those variables change or not from one UI frame to another. Keru provides a simple way to do this by through the [`Observed`] wrapper struct, but any method can work.
//! 
//! - Then, the library assumes that the GUI code inside the block won't result in any UI updates or relayouts. Therefore, it can skip all the hashing and diffing operations.
//! 
//! Specifying dependencies explicitly might sound annoying, but there's a natural place to do it: at the beginning of any reusable component function.
//! 
//! 
//! ### Inspiration
//! 
//! - [Ryan Fleury's UI series](https://www.rfleury.com/p/ui-series-table-of-contents)
//! - [Egui](https://github.com/emilk/egui)
//! - [Crochet](https://github.com/raphlinus/crochet)

// This helps with doc links.
#[allow(unused_imports)]
use crate::*;