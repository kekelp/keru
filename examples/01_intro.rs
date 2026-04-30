use keru::*;
use keru::node_library::*;

// The state of our program is a regular Rust struct. 
pub struct State {
    pub count: i32,
}

// This is the function that declaratively builds the GUI every frame.
// The declarative calls update the retained GUI state in the `Ui` struct.
// We're not rebuilding the GUI from scratch: this is not an immediate-mode library.
// (At least not in that sense).
fn update_ui(state: &mut State, ui: &mut Ui) {
    // First, create a NodeKey, which is a unique identity for a GUI element.
    #[node_key] const INCREASE: NodeKey;
    
    // Create a Node struct describing a button.
    let increase_button: Node = BUTTON
        .color(Color::RED)
        .text("Increase")
        .key(INCREASE);

    // Create another Node describing the count label. This one doesn't need a key.
    // By using the Node's builder methods and the basic constants from keru::node_library, 
    // we can create different Nodes to describe all sorts of UI elements.
    let formatted_count = state.count.to_string();
    let count: Node = LABEL.text(&formatted_count);

    // The vertical stack container is a Node as well!
    // It has `Stack` instead of `Free` in its `children_layout` field,
    // which means that its children will be arranged in a stack.
    // Everything is a Node, and a Node can be many things at once.
    // All the Node's fields are public, so you are free to create all sorts of combinations.
    let v_stack = V_STACK.stack_spacing(15.0);

    // Add the nodes into the tree.
    // The .nest(|| { ... }) calls define the tree structure and the layout.
    // When compiling in debug mode, you can press F1 to see the bounds of the layout rectangles.
    ui.add(v_stack).nest(|| {
        ui.add(increase_button);
        ui.add(count);
    });

    // Use the NodeKey that we assigned to the `button` node to listen to events on it.
    // Using NodeKeys is useful to separate the ui.add() calls and the effects,
    // which helps with making the layout more readable.
    // Of course, we can place this code wherever we want. 
    if ui.is_clicked(INCREASE) {
        state.count += 1;
    }

    // We can also use the key to get a reference to the real retained Node that we added.
    // In this way, we can inspect or modify the tree after building it, 
    // in a way that's usually not possible in purely declarative or immediate-mode libraries.
    // This is usually not needed except in very advanced Components.
    let node_ref = ui.get_node_mut(INCREASE).unwrap();
    if node_ref.is_clicked() {
        let rect = node_ref.rect();
        println!("{:?}", rect);
    }

    // If we want, we can skip keys completely, and use a more traditional immediate-mode GUI form:
    let decrease_button: Node = BUTTON
        .color(Color::BLUE)
        .position_x(Pos::End)
        .text("Decrease");

    if ui.add(decrease_button).is_clicked(ui) {
        state.count -= 1;
    }

    // Note that we didn't add this new button as a child of the v_stack.
    // It's a child of the root, and can position itself freely in the space of the whole window.
    // There's no "ZStack": that's what happens automatically when you add nodes
    // as children of a node that's neither a Stack nor a Grid.
}

fn main() {
    // The examples use the `run_example_loop` helper, which sets up a winit/wgpu loop,
    // and runs our `update_ui` on every frame.
    // This is just meant for the examples and for experimenting: 
    // the "intended" way to use the library is from a user-managed winit/wgpu loop. 
    // It's not that much code, and it gives you full control over what code gets executed and when, 
    // it allows it to access all advanced winit features,
    // and to easily integrate custom wgpu rendering.
    // To see how this works, see the `window_loop` example.
    let state = State { count: 0 };
    example_window_loop::run_example_loop(state, update_ui);
}

// This example already covers most of the concepts of the library.
// 
// To continue:
// - the `02_dynamic.rs` example shows how to create NodeKeys at runtime for dynamic GUI elements.
// - the `03_components.rs` example shows how to use the Component trait.
// 
// Then, the rest of the examples show how these basic concepts 
// can be combined together to build various things.
// In particular:
// - the `showcase.rs` example gives an overview of various components, text,
//       text editing, graphical effects, and vector drawing.
// - the `aesthetics_*.rs` examples show how the Node's styling params 
//       can be used to achieve some different aesthetics.
// - the `drag_and_drop_component.rs` shows an example of using an advanced Component,
//       that can manipulate the children that we add to it.
