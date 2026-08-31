# keru_draw

This is a renderer for `keru`, an experimental GUI library.

The architecture is heavily inspired by [`vger-rs`](https://github.com/audulus/vger-rs). It uses a single branching megashader and draws everything in one draw call, using an ordered instance buffer for painter's algorithm ordering.

The main upgrade compared to `vger-rs` is that it can render real full-featured text, thanks to `parley`.

The `keru_draw` text boxes are "retained mode" and fully interactive, including selection, copy-paste and IME input.

It uses the `slang` shader language, which allows the megashader to have a degree of modularity/extensibility. This means that the text management is split into a [separate library](https://github.com/kekelp/keru_text) that can be used independently.

The `slang` shaders are compiled into `spir-v` files that are committed in the repository, so the slang compiler is not needed to build and use this crate.

