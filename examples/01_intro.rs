use keru::*;
use keru::node_library::*;

// The state of our program is a regular Rust struct. 
pub struct State {
    pub count: i32,
}

// This is the function that declaratively builds the GUI every frame.
// The declarative calls update the retained GUI state in the `Ui` struct.
// We're not rebuilding the GUI from scratch: this is not an immediate-mode library (at least not in that sense).
fn update_ui(state: &mut State, ui: &mut Ui) {
    // First, create a NodeKey, which is an unique identity for a GUI element.
    #[node_key] const INCREASE: NodeKey;
    
    // Create a Node struct describing a button.
    let increase_button: Node = BUTTON
        .color(Color::RED)
        .text("Increase")
        .key(INCREASE);

    // Create another Node describing the count label.
    // By using the Node's builder methods and the basic constants from keru::node_library, 
    // we can create different Nodes to describe all sorts of UI elements.
    let formatted_count = state.count.to_string();
    let count: Node = LABEL.text(&formatted_count);

    // The vertical stack container is a Node as well!
    // It just has `ChildrenLayout::Stack` instead of `ChildrenLayout::Free` as its `children_layout` field,
    // which means that its children will be arranged in a stack.
    // Everything is a Node, and a Node can be many things at once.
    // All the Node's fields are public, so you are free to create all sorts of combinations.
    let v_stack = V_STACK.stack_spacing(15.0);

    // Add the nodes into the tree.
    // The .nest(|| { ... }) calls define the structure of the tree.
    // If the program is compiled in debug mode, you can press F1 to see the bounds of the layout rectangles.
    ui.add(v_stack).nest(|| {
        ui.add(increase_button);
        ui.add(count);
    });

    // Use the NodeKey that we assigned to the `button` node to listen to events on it.
    // Using NodeKeys is useful to separate the ui.add() calls and the effects, which helps with making the layout more readable.
    // Of course, we can place this code wherever we want. 
    if ui.is_clicked(INCREASE) {
        state.count += 1;
    }

    // We can also use the key to get a reference to the real retained Node that we added.
    // In this way, we can inspect or modify the tree after building it, 
    // in a way that's usually not possible in purely declarative or immediate-mode libraries.
    // This is usually not needed except in very advanced Components. See the component used in `drag_and_drop_component.rs` for an example.
    let node_ref = ui.get_node_mut(INCREASE).unwrap();
    if node_ref.is_clicked() {
        let rect = node_ref.rect();
        println!("{:?}", rect);
    }

    // If we really want, we can skip keys completely and use a more traditional immediate mode GUI form:
    let decrease_button: Node = BUTTON
        .color(Color::BLUE)
        .position_x(Pos::End)
        .text("Decrease");

    if ui.add(decrease_button).is_clicked(ui) {
        state.count -= 1;
    }

    // Note that we didn't add this new button as a child of the v_stack,
    // so it's a child of the tree root, and can position itself freely in the space of the whole window.
    // There's no "ZStack": that's what happens automatically when you add nodes as children of a node that's neither a Stack or a Grid.
}

fn main() {
    // The examples use the `run_example_loop` helper, which sets up a winit/wgpu loop automatically, and runs the `update_ui` function that we pass.
    // This is just meant for the examples and for experimenting: the "intended" way to use the library is from a user-managed winit/wgpu loop. 
    // It's really not that much code, and it allows your program to have full control over what code gets executed and when, 
    // to access all advanced winit features, and to easily integrate custom wgpu rendering.
    // To see how this works, you can see the `window_loop` example.
    let state = State { count: 0 };
    example_window_loop::run_example_loop(state, update_ui);
}

// This example already covers most of the concepts of the library.
// 
// To continue:
// - the `02_dynamic.rs` example shows how to create NodeKeys at runtime for dynamic GUI elements.
// - the `03_component.rs` example shows how to use the Component trait to create reusable components that can also manage their own state.
// 
// Then, the rest of the examples show how 
