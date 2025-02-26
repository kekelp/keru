Keru is a Graphical User Interface library.

It is in active development and it's not ready to be used. Many features are missing or half-baked.

It offers a declarative API similar to immediate mode GUI libraries, but it is not immediate mode.

See the [docs.rs page](https://docs.rs/keru/latest/keru/) for more information.

![Screenshot of paint example](screenshots/paint.png)

## Code Example

```rust
// Define an unique identity for this button
#[node_key] const INCREASE: NodeKey;

// Add nodes to the UI and set their parameters
ui.add(INCREASE)
    .params(BUTTON)
    .color(Color::RED)
    .text("Increase");

// Place the nodes into the tree and define the layout
ui.v_stack().nest(|| {
    if self.show {
        ui.add(INCREASE);
        ui.label(self.count); // This one doesn't need a key.
    }
});

// Run code in response to events
if ui.is_clicked(INCREASE) {
    self.count += 1;
}
```