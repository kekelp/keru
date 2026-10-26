# `keru_layout`

>This file is written by a human.

`keru_layout` is a UI layout library that uses a new graph-based algorithm. It's used in the [Keru GUI library](https://github.com/kekelp/keru/), but it's available as a self-contained reusable crate.

# The Idea

Most layout algorithms will do one or more traversals of the tree of UI elements and calculate the size and position of each elements as they go.

However, the kind of dependencies that layout libraries usually expose can go along the direction of the tree (`Frac`, `Fill`), but also in the opposite direction (`FitContent`), or "laterally" (`AspectRatio`, wrapping text).

In short, the idea of this library is to consider the resulting graph of dependencies and to solve it directly, walking through of this abstract dependency graph rather than the real UI tree.

# Advantages

- Performance: this library is very fast. From the testing that I've done so far, most layouts can fall in either of two categories:

    - Layouts where the graph of dependencies ends up being fairly similar to the real UI tree. For example, a layout composed completely of nested `Frac(0.5)` elements.
    In these cases, `keru_layout` is on par with well-optimized implementations of the traditional tree-based libraries.

    - Layouts where the graph of dependencies ends up being very different from the UI tree. For example, a layout with many `FitContent` or `AspectRatio` nodes.
    In these cases, `keru_layout` is significantly faster.

- Correctness: in some rare cases, such as layouts with many `AspectRatio` dependencies, a traditional tree-based algorithms would have to perform many traversals of the UI tree in order to finally compute the correct results. To avoid spending too much time on layout, many libraries will cut corners and silently produce an inconsistent layout.

    I wrote about one of these cases in this older [blog post](https://kekelp.github.io/keru/blog/layout/).

- Developer Experience: while it's usually not a problem to work around the correctness issues of traditional layout algorithms, it can still be confusing to run nto them during development. Using a more consistent algorithm with less edge cases and surprises helps.

- Debuggability: almost all layout systems can express cyclic dependencies between elements. In a traditional tree-based algorithm, this will result in a silently "broken" layout, but with a graph algorithm we can give the user a loud warning and an accurate list of the nodes that were stuck in cycles.


# Integration

`keru_layout` uses a trait-based API similar to Taffy's "low-level" API. A higher level API might be added in the future.

In the trait-based API, `keru_layout` assumes that you already have a tree of UI elements, and that every UI element roughly corresponds to one layout element. Then, using the library looks like this:

1) Add a `LayoutParams` to each UI node. This is the struct that defines the node's layout role (size, position, stack, grid, etc).
2) Add a `NodeLayoutData` to each UI node. This is a very small opaque struct that the solver uses to store per-node intermediate results and flags.

    >The layout solver could store all the `NodeLayoutData`s internally, but then it would have to maintain a mapping between your `NodeId`s and its own internal `NodeLayoutData` ids. If you help the solver out by storing the `NodeLayoutData`s in your own tree, it can go much faster.

    >Also, for performance, it might be helpful to store the `LayoutParams` and `NodeLayoutData` separately from the rest of the UI node's data. For example, in a `SecondaryMap` if the tree is a `SlotMap`, or in a parallel `Vec` if the tree is a `Slab`.

3) Add a single centralized `TreeLayoutData` anywhere in the tree.

4) Implement the `TreeAdapter` trait for the whole UI tree:

    - Set the `NodeId` type to the Id that can be used to refer to nodes in the UI tree, such as a slotmap key or a slab index.
    - Fill in the `node_layout_params()`, `node_layout_data()`, `node_layout_data_mut()` and `layout_data()` to simply access the data corresponding to that `NodeId`.

    - Implement the `root()`, `parent()`, `first_child()`, `next_sibling()` methods so that the layout solver can read the preexisting tree structures of your UI nodes.

    - Implement the `write_output_rect()` callback, and make it write provided output rects where you need them to go, such as a `rect` field in the UI node.

    - Implement the `measure_text()` method to measure the text contained in a node. If the `width` is `Some`, then the layout solver wants the text to wrap, and you should return the size of the text when wrapping to that width.
    If text measurement is not needed, for example if no node wants to fit to the size of text, it's fine to provide a dummy implementation.

    - Finally, Implement the `write_output_rect()` callback, and make it write provided output rects where you need them to go, such as a `rect` field in the UI node.

    - Optionally, implement `cycle_found()` to notify the user about any cycles in their layout.

Then, you can call the trait's provided methods to calculate the layout and get information about cycles or other issues.

