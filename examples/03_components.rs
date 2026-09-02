use keru::*;
use keru::node_library::*;

// This example shows how to use the 'Component` trait.
// 
// Components are the most robust way of separating GUI code into reusable "components" or "widgets".
// In addition, Components can optionally hold their own "local" state.
//
// In a complex program, it could surely get annoying if the user had to add a field in their State
// for every state variable of every color picker, rich text editor,
// or every small self-contained widget that they have.


// Using a Component is meant to feel like building a regular Node and adding it.
// A component is a Node-like struct that describes its parameters.
pub struct StatefulCounter {
    pub color: Color,
}

#[node_key] const INCREASE: NodeKey;

impl Component for StatefulCounter {
    // Define the type of the component's local state.
    // It must implement Default, so that the Ui can initialize it when the component is added.
    type State = i32;
    // Other types that we're not using in this example.
    // Hopefully future versions of Rust will allow setting default values for associated types.
    type AddResult = ();
    type ComponentOutput = ();

    fn add_to_ui(&mut self, ui: &mut Ui, state: &mut i32) {
        // (Using an arena is not mandatory, but it makes formatting values a lot faster.)
        // (Keru has a thread local arena that you can use without any setup.)
        with_arena(|arena| {

            let v_stack = V_STACK.padding(10.0).color(self.color);
            let count_text = bumpalo::format!(in arena, "Count: {}", state);

            ui.add(v_stack).nest(|| {
                ui.add(TEXT_PARAGRAPH.text(&count_text));
                ui.add(BUTTON.text("Increase").key(INCREASE));
            });
                
            if ui.is_clicked(INCREASE) {
                *state += 1;
            }

            // Note that we can treat the INCREASE key as unique within the container,
            // even if the component is meant to be added multiple times.
            // This works because each instance of the component gets its own private "key scope".
            // You can do this manually outside of a Component by using `ui.key_scope()`.
        });
    }
}

// The same component can use both an implicit state or an explicit state passed from outside.
// To show this, let's still add a centralized `count` in the state. 
pub struct State {
    count: i32,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    let counter = StatefulCounter {
        color: Color::KERU_RED,
    };
    let counter2 = StatefulCounter {
        color: Color::KERU_GREEN,
    };
    let counter3 = StatefulCounter {
        color: Color::KERU_BLUE,
    };
    let counter4 = StatefulCounter {
        color: Color::KERU_BLUE
    };

    ui.add(H_STACK.stack_spacing(20.0).size_x(Size::Fill)).nest(|| {
        // The first two components hold their own state locally.
        ui.add_component(counter);
        ui.add_component(counter2);
        // The other two get a reference to the centralized state passed from outside.
        ui.add_component_with_state(counter3, &mut state.count);
        ui.add_component_with_state(counter4, &mut state.count);
    });
}

fn main() {
    let state = State { count: 0 };
    example_window_loop::run_example_loop(state, update_ui);
}


// Components are still experimental, and there's lots of different ways to use them.
// 
// - We can use the AddResult type so that ui.add_component_*() returns a value.
//   The returned value can be an output, such as the selected color of a color picker. 
//   It can also be an `UiParent`, so that the caller can add nodes as children of the component:
//       
//       ui.add_component(container_component).nest(|| { ... })`.
// 
// - We can also add a `ComponentKey` to the component struct,
//   implement the optional `component_key()` method to let the trait know where to find it,
//   and then implement the `Component::run_component()` method and return the output from there.
//   Then, the user can call `ui.run_component(COMPONENT_KEY)` to run that logic and get the output.
//   
//   This system is currently a little clunky, but it's the way that will feel most similar 
//   to adding a node with a key calling `ui.is_clicked(key)` on it:
// 
//     #[component_key] const COMPONENT_KEY;
//     let counter = Counter::default().with_key(COMPONENT_KEY);
//
//     ui.add_component(counter);
//
//     if let Some(output) = ui.run_component(COUNTER_1) { ... }
//
