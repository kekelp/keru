//! Some extra information about the library.
//! 
//! # How does it work?
//! 
//! Keru has a declarative API similar to immediate mode GUI libraries. However, it is not immediate mode.
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
//! // Define unique keys/ids for some ui elements.
//! #[node_key] const INCREASE: NodeKey;
//! #[node_key] const SHOW: NodeKey;
//! 
//! self.ui.add(SHOW) // Create an element. This doesn't place it into the active ui tree yet.
//!     .params(BUTTON) // Set style and properties
//!     .static_text("Show Counter"); // Set text
//! 
//! self.ui.add(INCREASE)
//!     .params(BUTTON)
//!     .static_text("Increase");
//! 
//! // Place the nodes in the ui tree.
//! // The nesting of these calls will define the layout.
//! self.ui.v_stack().nest(|| { 
//!     self.ui.place(SHOW);
//!     if self.show {
//!         self.ui.place(INCREASE);
//!         self.ui.label(self.count); // shorthand: add(), text() and place() all in one
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
//! [^1]: In most "slow" UI applications, the UI can "go to sleep" and do nothing until user input or an external event wakes it up. Even true immediate mode GUIs like `egui` can do this. "Cycle" refers to one of these "awake frames".
//! 
//! Every time, you re-declare the whole GUI tree. However, the tree is **fully retained** across frames. If a function like [`place()`](Ui::place) finds that the corresponding node already exists in the tree, it will either update it, or do nothing.
//! 
//! If a function does cause a change in the tree, it will also tell the [`Ui`] to do *partial* relayouts and *partial* updates to the render data, propagating the change to the screen.
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
//! # #[node_key] const SHOW: NodeKey;
//! # 
//! # impl State {
//! #   fn declare_ui(&mut self) {
//! # 
//! // The `add` call will update the node's parameters (size, color, text, etc),
//! self.ui.add(INCREASE)
//!     .params(BUTTON)
//!     .static_text("Increate");
//! 
//! // when a node is placed onto the tree,
//! // the library will compare its new params to the ones it had last frame.
//! self.ui.v_stack().nest(|| {
//!     // For example, if the size of the SHOW node has changed, 
//!     // it will schedule a _partial_ relayout starting 
//!     // at this position in the tree. If the color has changed, 
//!     // it will schedule to update the render data for that _single_ rectangle.
//!     self.ui.place(SHOW);
//!     // Here, depending on the value of `self.show`, some nodes 
//!     // might be included or excluded from the tree.
//!     // If the value of `self.show` is different from last frame, 
//!     // then, at the end of its `nest()` block,
//!     // the parent will notice that its children changed.
//!     // In that case, it will schedule a partial relayout.
//!     if self.show {
//!         self.ui.place(INCREASE);
//!         self.ui.label(self.count);
//!     }
//! });
//! 
//! // At the end, the library will do all the partial relayouts and updates needed.
//! # 
//! #   }
//! # }
//! ```
//! 
//! 
//! While the GUI redeclaration code is rerun every cycle, the functions barely do anything at all, unless something changed in the underlying state.
//! 
//! If something did change, the library knows which nodes need to be updated, and can schedule only the minimal relayouts and updates needed.
//! 
//! Most of the time, all that the library needs to do is to hash some [`NodeParams`] and some text, and conclude that nothing changed. This is usually very light, especially compared to a "true immediate mode" approach.
//! 
//! It's also important to remember that this has nothing to do with the performance of the program when idle: see footnote [^1].
//! 
//! However, since reactivity seems to be the current big thing, I am also experimenting with ways to skip even this work: see the ["Reactivity at Home"](#reactivity-at-home) section. 
//! 
//! # Advantages
//! 
//! This is a list of advantages that I think Keru gives over the most popular alternatives. 
//! 
//! - **Own your window loop and rendering**
//! 
//!     You can use a regular `winit`/`wgpu` render loop and call Keru as a library. It doesn't take any control away from you.
//!     This makes it easy to compose the ui with custom rendering, both "below" and "inside" the GUI. This is demonstrated in the painter example. Both the painting canvas "below" the UI and the OKLAB color picker "inside" an UI element use custom `wgpu` rendering. 
//!     
//!     This also means that Keru is automatically compatible with ecosystem crates for things like SVG rendering, animations, video, etc, without needing to include a particular implementation within Keru. (although including one anyway would still result in a better out-of-the box experience, and probably better ergonomics).
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
//!     - you don't have to write all your code inside of a trait impl or a closure that the runtime executes on its own schedule
//!     - you don't have to write all your logic inside callbacks
//! 
//! -------
//! 
//! - **Flexible code organization** 
//! 
//!     You should have as much freedom as possible when organizing your GUI code. You have the *option* to keep style, layout and effects of an element close to each other, but you aren't forced to do so.
//!     
//!     If you look at the examples in the repos for `gpui`, `floem`, `egui` and others, you'll see that the layout is derived from the order and nesting of the functions that create the elements.
//!     But you also have to specify the style and the effect right after that call by chaining builder functions to it.
//! 
//!     The resulting code is very strange and hard to read, in my opinion. In particular, it's very hard to follow the nesting structure that defines the layout, since it's mixed with so much other stuff. Most of the clarity of the "nested calls -> layout" approach is lost.
//! 
//!     In Keru, you can always refer to a node from anywhere in your code by using the unique [`NodeKey`]. So you can split the layout code ([`place`](Ui::place) and [`nest`](UiPlacedNode::nest)) or the effects ([`is_clicked`](`Ui::is_clicked`), etc.) from the rest. This is the pattern used in the examples.
//! 
//!     If you don't care about any of this, you can still prefer to keep style, layout and effects all together: use anonymous nodes ([`anon`](Ui::add_anon)) and chained builder methods ([`place`](UiNode::place)). Currently, effect functions like [`Ui::is_clicked`] are the only ones that don't have a chained builder method counterpart, but I will be trying to fix this,
//! 
//! -------
//! 
//! - **Own your state**
//! 
//!     Your UI can depend on any variable that you can get a reference to, i.e. anything. You don't have to structure your state in any particular way.
//!     - You don't have to pair the state with its UI display logic (unless you want to!)
//!     - You don't have to wrap your state into observer structs or signal handlers (unless you want to: see the ["Reactivity at Home"](#reactivity-at-home) section)
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
//!     Keru's API and implementation also tries to improve in other areas where Egui is (in my opinion) janky or inconvenient:
//! 
//!     - The API is less fragmented: all operations are methods on the main [`Ui`] struct, as opposed to a mixture of methods and associated functions on `Context`, `Ui`, `Window`, `Frame`, ... in Egui.
//!     - There is no interior mutability or locking hidden inside the [`Ui`], unlike Egui's `Context`.
//!     - Egui's closure pattern is substituted by a much simpler one (see [`UiPlacedNode::nest()`]).
//! 
//!         Because the closure doesn't borrow or capture anything, it's a lot less prone to borrowing compile errors, and gives more flexibility in how user code can be organized.
//!         
//!         To make this pattern possible, Keru keeps track of the nested  [nest()][`UiPlacedNode::nest()`] calls in thread-local variables. The nesting of function calls is an intrinsically thread-local concept, so this feels like a natural step.
//! 
//! --------
//! 
//! I'm not going to spell them out here, but there are disadvantages as well, of course.
//! 
//! ## Reactivity at home
//! 
//! There's still some of room to add "reactivity" (in the Floem/SwiftUI sense) on top of the library as described so far. I am currently experimenting with this.
//! 
//! Since nothing is implemented yet, there's no point in going into too much detail, but the idea is simple:
//! 
//! - The user can optionally choose to wrap *some* of his state in something similar to Floem's `RwSignal`.
//! 
//! - The user can specify explicitly that a block of UI declaration code depends only on a handful of wrapped variables.
//! 
//! - Then, the library can just skip all that code completely if none of the variables changed, or at least turn functions like [`Ui::add`] and [`Ui::place`] into no-ops.
//! 
//! I think the idea is fair, it's just a matter of finding a nice enough API.
//! 
//! Specifying dependencies explicitly might sound annoying, but there's a natural place to do it: at the beginning of any "widget" function.
//!  
//! 
//! ## Open questions
//! 
//! - Less room for mistakes: [`Ui::place`] in particular can panic if used incorrectly (using the same key twice or placing a node that wasn't added). 
//!     There are ways around this, but they make the API worse in other ways. Given that [`UiNode::place`] already offers a less flexible but panic-safe alternative, it might be fine to leave it as it is, but I am still thinking about this often.
//! 
//! - Accessing is_clicked from the builder method chain instead of in a separate block with a key. This is the only operation that can't be done without a key. If I found a good way to do it, keys would become completely optional.
//!    Personally, I like using keys anyway, but it might be worth to think some more about this.
//! 
//! - The current way of doing custom rendered UI elements can result in imperfect alpha blending.
//! 
//! - Problems with `winit`/`wgpu`: At least on my X11 Linux system, both take forever to start up, and resizing the window isn't smooth at all. 
//! 
//! - Problems with `glyphon`: I am extremely grateful for this library and for `cosmic_text`, as a simple way to "just render text on the screen" was somehow still missing until very recent times. (How was this possible if both Chrome and Firefox have had open-source state-of-the-art text renderers since forever? That's just the power of C++, I think). However, as soon as I implemented scrolling, I noticed that glyphon would often take 50 or even 100 milliseconds to run its `prepare()` function, even for pretty small paragraphs.
//! 
//! - Adding the remaining 99% of features.
//! 
//! 
//! ### Inspiration
//! 
//! - [Ryan Fleury's UI series](https://www.rfleury.com/p/ui-series-table-of-contents)
//! - [Egui](https://github.com/emilk/egui) and [Dear Imgui](https://github.com/ocornut/imgui)
//! - [Crochet](https://github.com/raphlinus/crochet)

// This helps with doc links.
#[allow(unused_imports)]
use crate::*;