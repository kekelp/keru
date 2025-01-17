//! Some extra information about the library.
//! 
//! # What does the code look like?
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
//! // Define unique keys/ids for some ui elements. This is not always required.
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
//!         self.ui.label(self.count);
//!         // For the count label, we used a shorthand to add it, set its text and place it all at once. 
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
//! 
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
//!     // For example, if the SHOW node's size has changed, 
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
//! Not really. It's true that that there's still some code that gets run on every frame/cycle. This doesn't happen only in immediate mode systems, but also in many "reactive" systems like React [^2].
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
//! From this starting point, there's still some room for "reactivity".
//! 
//! The only difference between what Keru does and a true reactive system (see footnote [^2]) is the redeclaration code that we run every frame/cycle. It doesn't do that much work, but it does have to hash a fair amount of [`NodeParams`] and strings to watch for changes.
//! 
//! In reactive systems like Floem, the user has to wrap all the state that the GUI depends on inside a wrapper (`RwSignal` in Floem). The wrapper observes changes in the value and reports it ro a interior-mutable thread_local runtime. The runtime uses the information about all these changes to determine which parts of the UI it has to run. 
//! 
//! In Keru, you can make the GUI read and write any variable that you can get a reference to. But the user could still optionally wrap **some** of his state into a wrapper that works in the same way as Floem's `RwSignal`.
//! 
//! Keru doesn't take control of the main loop, so it can't use that information as effectively and transparently as Floem does. However, with a bit of help from the user, it could still use that information to either skip all the hashing/diffing operations, or maybe to skip running the redeclaration code completely.
//! 
//! None of this is implemented yet, but I am currently trying out a few different approaches.
//! 
//! # Advantages
//! 
//! This is a list of advantages that I think Keru's approach gives over other other commonly seen approaches. 
//! 
//! - **Own your window loop and rendering**
//! 
//!     You can use a regular `winit`/`wgpu` render loop and call Keru as a library. It doesn't take any control away from you.
//!     This makes it easy to compose the ui with custom rendering, both "below" and "inside" the GUI. This is demonstrated in the painter example. Both the painting canvas "below" the UI and the OKLAB color picker "inside" an UI element use custom `wgpu` rendering. 
//!     
//!     In particular, this also means that Keru is trivially compatible with ecosystem crates for things like SVG rendering, animations, video, etc, without needing to include and "bless" a particular implementation within Keru. (Of course, it could also include one anyway for the sake of a more complete out-of-the-box experience).
//! 
//!     Egui and Dear Imgui accomplish this in a much more hardcore way by being compatible with most windowing and render libraries on earth, but this has many disadvantages, in addition to requiring a ton of extra work. For now, Keru just supports `winit` and `wgpu`.
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
//!     - You don't have to wrap your state into observer structs or signal handlers (unless you want to, see the "[Reactivity at Home](#reactivity-at-home)" section)
//!     - You shouldn't get any extra borrowing or lifetime issues (unlike in closure-heavy and callback-heavy systems)
//! 
//! -------
//! 
//! - **But it's still not immediate mode!**
//! 
//!     From the public API, Keru might seem very similar to Egui or other immediate mode libraries, and indeed Egui also offers many of the advantages listed so far.
//! 
//!     However, Keru is **not immediate mode!** The public declarative API might look similar, but inside, there is a fully retained node tree. This is enough to avoid most of the traditional disadvantages of immediate mode GUI:
//!     
//!     - Layout isn't any more difficult than with any traditional retained mode GUI.
//!     - There is no need to do a full relayout on every frame. When few things change, Keru does partial updates and relayouts.
//!     - Integrating accessibility tools **shouldn't** be any more difficult than with any traditional retained mode GUI, but I haven't tried this yet.
//! 
//!     Keru's API and implementation also tries to improve in other areas where Egui is (in my opinion) janky or inconvenient:
//! 
//!     - The API is less fragmented: all operations are methods on the retained [`Ui`] struct, as opposed to a mixture of methods and associated functions on `Context`, `Ui`, `Window`, `Frame`, ... in Egui.
//!     - There is no interior mutability or locking hidden inside the [`Ui`], unlike Egui's `Context`.
//!     - Egui's closure pattern is substituted by a much simpler one (see [`UiPlacedNode::nest()`]). Because the closure doesn't borrow or capture anything, it's a lot less prone to ownership errors, and gives more flexibility in how user code can be organized.
//!     To make this pattern possible, Keru keeps track of the nested  [nest()][`UiPlacedNode::nest()`] calls in thread-local variables. The nesting of function calls is an intrinsically thread-local concept, so this feels like a natural step.
//! 
//! 
//! ## Open questions and unsolved issues
//! 
//! - Reactivity
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
//! - Problems with `glyphon`: I am extremely grateful for this library and for `cosmic_text`, as a simple way to "just render text on the screen" was somehow still missing until very recent times. (How was this possible if both Chrome and Firefox open-source state of the art text renderers since forever? That's just the power of C++, I think). However, as soon as I implemented scrolling, I noticed that it would often take 50 or even 100 milliseconds to run its `prepare()` function, even for pretty small paragraphs.
//! 
//! - Adding the remaining 99% of features.
//! 
//! 

// This helps with doc links.
#[allow(unused_imports)]
use crate::*;