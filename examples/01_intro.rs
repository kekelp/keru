use keru::*;
use keru::node_library::*;

// The state of our program is a regular Rust struct. 
pub struct State {
    pub count: i32,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    // This is the function that declaratively builds the GUI every frame.
    // The declarative calls update the retained GUI state in the `Ui` struct. We're not rebuilding the GUI from scratch:
    // this is not an immediate-mode GUI library (at least not in that sense).

    // First, create a NodeKey, which is an unique identity for a UI element.
    #[node_key] const INCREASE: NodeKey;
    
    // Create a Node struct describing a button.
    let increase_button: Node = BUTTON
        .color(Color::RED)
        .text("Increase")
        .key(INCREASE);

    // Create another Node describing the count label.
    // By using the Node's builder methods or by using the basic constants from keru::node_library, 
    // we can create different Nodes that can describe all sorts of UI elements.
    let formatted_count = state.count.to_string();
    let count: Node = LABEL.text(&formatted_count);

    // The vertical stack container is a Node as well!
    // Everything is a Node, and a Node can be many things at once.
    // All the Node's fields are public, so you are free to create all sorts of combinations.
    let v_stack = V_STACK.stack_spacing(15.0);

    // Add the nodes into the tree.
    // The .nest(|| { ... }) calls define the structure of the tree.
    // The `children_layout = ChildrenLayout::Stack` field of the V_STACK means that the children of v_stack are arranged in a stack. 
    // When compiling in debug mode, you can press F1 to see the bounds of the layout rectangles.
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

    // If we really want, we can skip keys completely and use a more traditional immediate mode GUI form:
    let decrease_button: Node = BUTTON
        .color(Color::BLUE)
        .position_y(Pos::Frac(0.75))
        .text("Decrease");

    if ui.add(decrease_button).is_clicked(ui) {
        state.count -= 1;
    }

    // Note that we didn't add this new button as a child of the v_stack,
    // so it's a child of the tree root, and can position itself freely in the space of the whole window.
    // There's no "ZStack", as that's what happens automatically when you add nodes as children of a node that's neither a Stack or a Grid.
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

