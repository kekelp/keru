use keru::*;
use keru::node_library::*;

pub struct State {
    pub count: f32,
}

// This example shows how to use the Component trait.
// 
// Components are the most robust way of separating GUI code into reusable "components" or "widgets".
// In addition, Components can have their own "implicit" state, without the user having to make space for it in their own State.
// 
// This is an advanced feature: most programs will be fine without it.
//
// In this example, we'll write a counter with a configuration setting that allows adding more than 1 at a time to the count.
//
// In a complex program, it could surely get annoying if the user had to add a field in their State for every configuration variable of 
// every color picker, rich text editor, or every small self-contained widget that they have.


// Using a Component is meant to feel like building a regular Node and adding it.
// A component is a Node-like struct that describes the component's parameters
// We can make it hold references to portions of the outside state.
pub struct Counter<'a> {
    // Reference to the count that the counter's button will modify.
    pub count: &'a mut f32,
    // By including a Layout field and using it as the Layout of the component's root node,
    // we can let the user choose the component's placement and size how they want, like with a regular Node.
    pub layout: Layout,
    // Optional extra customization.
    pub color: Color,
}

// Every `Counter` instance will have an associated `CounterSettings` instance as its implicit state.
// It has to implement `Default`, so that the Ui can initialize the state automatically whenever an instance of the Component is added.
// It will then be cleaned up when the Component is finally removed from the tree.
// If a component is hidden ([Node::children_can_hide]) rather than removed, it will retain its state. See the [StatefulTransformView] component in the showcase example.
pub struct CounterSettings {
    pub increase_step: f32,
}
impl Default for CounterSettings {
    fn default() -> Self {
        Self { increase_step: 1.0 }
    }
}

#[node_key] const INCREASE: NodeKey;
#[node_key] const STEP_UP: NodeKey;

// We finally implement the Component trait and specify what should happen to the Ui when the user adds the component.
impl Component for Counter<'_> {
    // Set the counter to use CounterSettings as its associated state.
    type State = CounterSettings;
    // Set the result of `ui.add_component(counter)`. Not used in this example.
    // The main purpose is to return an `UiParent`, so that the user can add children to it: `ui.add_component(counter_component).nest(|| { ... })`.
    type AddResult = ();
    // Another type that we're not using in this example. See the `drag_component.rs` or `aesthetics_scifi.rs` examples. 
    type ComponentOutput = ();
    // ...hopefully future versions of Rust will let the trait use default values for these types when they are not used.

    fn add_to_ui(&mut self, ui: &mut Ui, state: &mut Self::State) {
        // (Using an arena is not mandatory, but allocating a tiny String on the global heap just to format a value is really not a good thing.)
        // (Keru has an arena that you can use without any setup, and it will make small local allocations virtually free.)
        with_arena(|arena| {

            let panel = PANEL.layout(self.layout);

            let count_text = bumpalo::format!(in arena, "Count: {:.2}", self.count);

            ui.add(panel).nest(|| {
                ui.add(V_STACK).nest(|| {
                    ui.add(LABEL.text(&count_text));
                    ui.add(BUTTON.color(self.color).text("Increase").key(INCREASE));
                    ui.add(TEXT.text("Increase Step:"));
                    ui.add(CONTAINER.size_x(Size::Pixels(200.0))).nest(|| {  
                        ui.add_component(Slider { value: &mut state.increase_step, min: 0.0, max: 10.0, clamp: true });
                    })
                });
            });
                
            if ui.is_clicked(INCREASE) {
                *self.count += state.increase_step;
            }
            if ui.is_clicked(STEP_UP) {
                state.increase_step += 1.0;
            }
            // Note that we're treating these keys as unique identifiers for their nodes, even if the component is meant to be added multiple times!
            // This is fine: each instance of the Component gets its own private "key scope", and can treat its keys as unique.
            // You can do this manually outside of a Component by using `ui.key_scope()`. See the `key_scope.rs` example.
        });
    }
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    // Create and add the component.
    let counter = Counter {
        count: &mut state.count,
        color: Color::KERU_BLUE,
        layout: Layout::default().size_symm(Size::FitContent).position_symm(Pos::Start),
    };
    
    ui.add_component(counter);
    
    // Another Counter instance with different params, but pointing to the same `state.count`.
    // It will get its own separate CounterSettings instance automatically.
    let counter2 = Counter {
        count: &mut state.count,
        color: Color::KERU_GREEN,
        layout: Layout::default().size_symm(Size::FitContent).position_symm(Pos::End),
    };
    
    ui.add_component(counter2);
}

/// Rather than sticking a reference inside of the component struct, we could also return the delta value as the result of ui.add_component, 
/// and leave it to the user to change their state based on the returned result.
/// ```
/// let delta = ui.add_component(counter);
/// state.count += delta;
/// ```
/// 
/// We could also add a [ComponentKey] to the component struct, implement the optional [Component::component_key()] method,
/// and then return the delta from the [Component::run_component()] method.
/// This way, using the component would look very similar to using a regular Node and calling `ui.is_clicked(KEY)` on it:
/// 
/// ```
/// #[component_key] const COUNTER_1;
/// let counter = Counter::default().with_key(COUNTER_1);
/// ui.add_component(counter);
/// if let Some(delta) = ui.run_component(COUNTER_1) {
///     state.count += delta;
/// }
/// ```
/// All these advanced ways to use components are experimental.

fn main() {
    let state = State { count: 0.0 };
    example_window_loop::run_example_loop(state, update_ui);
}
