use keru::example_window_loop::*;
use keru::*;

const TAB_1: Tab = Tab("Tab 1");
const TAB_2: Tab = Tab("Tab 2");

#[derive(Default)]
pub struct State {
    tabs: Vec<Tab>,
    current_tab: usize,
}

impl State {
    fn update_ui(&mut self, ui: &mut Ui) {
        
        // same state key added to different nodes?
        #[node_key] const BUTTON1: NodeKey;
        #[node_key] const BUTTON2: NodeKey;
        #[state_key] const WIDGET_STATE: StateKey<bool>;

        ui.vertical_tabs(&self.tabs[..], &mut self.current_tab)
            .nest(|| match self.tabs[self.current_tab] {
                TAB_1 => {

                    ui.h_stack().nest(|| {
                    
                        if ui.add(BUTTON.text("Toggle bool").key(BUTTON1)).is_clicked(ui) {
                            *ui.state_mut(WIDGET_STATE) = ! ui.state(WIDGET_STATE)
                        }
                        
                        let text = if *ui.state(WIDGET_STATE) {
                            "Bool on"
                        } else {
                            "Bool off"
                        };
                        ui.label(text);
                    });

                },
                TAB_2 => {                    
                    ui.h_stack().nest(|| {
                        if ui.add(BUTTON.text("Toggle bool").key(BUTTON2)).is_clicked(ui) {
                            *ui.state_mut(WIDGET_STATE) = ! ui.state(WIDGET_STATE)
                        }
            
                        let text = if *ui.state(WIDGET_STATE) {
                            "Bool on"
                        } else {
                            "Bool off"
                        };
                        ui.label(text);
                    });
                },
                _ => {}
            });



    }
}

fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Warn)
        .filter_module("keru::tree", log::LevelFilter::Trace)
        .init();
    let state = State {
        tabs: vec![TAB_1, TAB_2],
        ..Default::default()
    };
    run_example_loop(state, State::update_ui);
}
