use keru::*;
use keru::node_library::*;

pub struct State {
    pub count: f32,
}

// This example shows how to use the Component trait.
// 
// Components are the most robust way of separating GUI code 
// The most important one is that Components can have their own "implicit" state, without the user having to make space for it in their own State.
//
// In this example, we'll write a counter with a configuration setting that allows adding more than 1 at a time to the count.
// It would surely be annoying if the user had to add a field in their State for every configuration variable of every color picker, rich text editor, 
// or every small self-contained widget that they have.
// 
// The Component is meant to feel like building a regular Node and adding it.
// 
// A component is a Node-like struct that describes the component's parameters.
//
// Then, the Component trait implementation describes what should happen when it is added to the Ui.
// 
// 
pub struct Counter<'a> {
    // The struct can hold references to state, so that the component will use for its effects.
    pub count: &'a mut f32,
    // By including a Layout field and using it as the Layout of the component's root node,
    // we can let the user choose the component's placement and size how they want, like with a regular Node.
    pub layout: Layout,
    // Optional extra customization.
    pub color: Color,
}

// Every `Counter` instance will have an associated `CounterSettings` instance as its implicit state.
// It has to implement `Default`, so that the Ui can initialize the state automatically whenever an instance of the Component is added.
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

impl Component for Counter<'_> {
    type State = CounterSettings;
    type AddResult = ();
    type ComponentOutput = ();

    // Describe what should happen to the Ui when a Counter is added.
    fn add_to_ui(&mut self, ui: &mut Ui, state: &mut Self::State) {

        let panel = PANEL.layout(self.layout);

        let count_text = format!("Count: {:.2}", self.count);

        ui.add(panel).nest(|| {
            ui.add(V_STACK).nest(|| {
                ui.add(LABEL.text(&count_text));
                ui.add(BUTTON.color(self.color).text("Increase").key(INCREASE));
                ui.add(TEXT.text("Increase Step:"));
                ui.add_component(Slider { value: &mut state.increase_step, min: 0.0, max: 10.0, clamp: true });
            });
        });
            
        if ui.is_clicked(INCREASE) {
            *self.count += state.increase_step;
        }
        if ui.is_clicked(STEP_UP) {
            state.increase_step += 1.0;
        }
        // Note that we're treating these keys as unique identifiers for their nodes, 
        // even if the component is meant to be reused!
        // This is fine: each instance of the Component gets its own private "key space" and can treat his keys as unique.

    }
}

fn update_ui(state: &mut State, ui: &mut Ui) {    
    let counter_component = Counter {
        count: &mut state.count,
        color: Color::KERU_BLUE,
        layout: Layout::default().size_symm(Size::FitContent).position_symm(Pos::Start),
    };
    
    ui.add_component(counter_component);
    
    // Another Counter instance with different params, but pointing to the same state.
    // It will also get a separate stance of the CounterSettings.
    let counter_2 = Counter {
        count: &mut state.count,
        color: Color::KERU_GREEN,
        layout: Layout::default().size_symm(Size::FitContent).position_symm(Pos::End),
    };
    
    ui.add_component(counter_2);
    
}

fn main() {
    let state = State { count: 0.0 };
    example_window_loop::run_example_loop(state, update_ui);
}
