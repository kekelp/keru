![Keru is experimental](https://img.shields.io/badge/status-alpha-orange)
[![Documentation for the `master` branch](https://img.shields.io/badge/docs-master-informational)](https://kekelp.github.io/keru/keru/index.html)

Keru is an experimental Graphical User Interface library.

The goal of the library is to be as easy to use as immediate mode libraries (and even a bit easier), but without the downsides of immediate mode.

See the [docs.rs page](https://docs.rs/keru/latest/keru/) for more information.

![Screenshot of paint example](screenshots/paint.png)

## Code Example

```rust
// Define an unique identity for this button
#[node_key] const INCREASE: NodeKey;

// Create a NodeParams struct describing a button
let increase_button = BUTTON
    .color(Color::RED)
    .text("Increase")
    .key(INCREASE);

// Place the nodes into the tree and define the layout
ui.v_stack().nest(|| {
    if self.show {
        ui.add(increase_button);
        ui.label(&self.count); // This one doesn't need a key.
    }
});

// Run code in response to events
if ui.is_clicked(INCREASE) {
    self.count += 1;
}
// This can also be done with a chained method after ui.add(increase_button).
// In that case, the key isn't needed.
```