use keru::*;
use keru::example_window_loop::*;

// Define a trait to hold our custom component function, that we will implement for `Ui`.
// This is just for the convenience of using method syntax: `ui.tab_view( ... )`
pub trait CustomWidgets {
    fn tab_view(&mut self, tabs: &[&str], tab_number: &mut usize) -> UiParent;
}

// The function will return an `UiParent`. The user can the call `nest()` on it and add his own stuff inside the tab view's main window.
impl CustomWidgets for Ui {
    fn tab_view(&mut self, tabs: &[&str], tab_number: &mut usize) -> UiParent {
        // Since there can be many tab buttons varying at runtime, we won't use this key directly:
        // we will use it as a base to create dynamic keys for each specific one, using `NodeKey::sibling()`.
        #[node_key] const TAB_BUTTON: NodeKey;

        // Use a subtree to ensure that the component can be reused without key conflicts.
        // (the subtree comes into play when keys are used, not when they are defined. So the TAB_BUTTON line can be outside of the subtree, as well as in another file or anywhere else).
        self.subtree().start(|| {
                
            let max_n = tabs.len() - 1;
            if *tab_number >= max_n {
                *tab_number = max_n;
            }
            
            // Update the state in response to button clicks or keyboard presses
            for (i, _) in tabs.iter().enumerate() {
                if self.is_clicked(TAB_BUTTON.sibling(i)) {
                    *tab_number = i;
                }
            }
            // todo: focused?
            let ilen = tabs.len() as isize;
            if self.key_input().key_pressed_or_repeated(&winit::keyboard::Key::Named(winit::keyboard::NamedKey::Tab)) {
                if self.key_mods().shift_key() {
                    *tab_number = (((*tab_number as isize) - 1 + ilen) % ilen) as usize;
                } else {
                    *tab_number = (*tab_number + 1) % tabs.len();
                }
            }

            let v_stack = V_STACK.stack_spacing(0);
            let tabs_h_stack = H_STACK.size_y(Size::FitContent);
            let inactive_tab = BUTTON.corners(RoundedCorners::TOP).colors(self.theme().muted_background);
            let active_tab = inactive_tab.colors(self.theme().background);
            let content_panel = PANEL.size_symm(Size::Fill).colors(self.theme().background);

            // Add the nodes to the ui.

                self.add(v_stack).nest(|| {
                    self.add(tabs_h_stack).nest(|| {
                        for (i, name) in tabs.iter().enumerate() {
                            let key_i = TAB_BUTTON.sibling(i);
                            let tab = if i == *tab_number { active_tab } else { inactive_tab };
                            let tab_i = tab.text(*name).key(key_i);
                            self.add(tab_i);
                        }
                    });

                    let content_nest = self.add(content_panel);
                    
                    return content_nest;
                })
                // down here, we're using implicit returns to pass the return value through all the closures and out of the function.
                // if this feels wrong, you can also declare `let mut result: Option<UiParent> = None` at the start of the function,
                // then assign `result = content_node`,
                // and `return result.unwrap()` at the end.
        })
    }
}

const TAB_1: &str = "Tab 1";
const TAB_2: &str = "Tab 2";
const TAB_3: &str = "Tab 3";
const TAB_4: &str = "Tab 4";
const TAB_5: &str = "Tab 5";

#[derive(Default)]
pub struct State {
    pub tab_number: usize,
}

impl State {
    fn update_ui(&mut self, ui: &mut Ui) {

        let tabs = [
            TAB_1,
            TAB_2,
            TAB_3,
            TAB_4,
            TAB_5,
        ];

        ui.tab_view(&tabs, &mut self.tab_number).nest(|| {
            match tabs[self.tab_number] {
                TAB_1 => {
                    ui.label("Content 1");
                }
                TAB_2 => {
                    ui.label("Content 2");
                }
                TAB_3 => {
                    ui.v_stack().nest(|| {
                        ui.label("Content 3");
                        ui.label("Content 3");
                        ui.label("Content 3");
                    });
                }
                TAB_4 => {
                    ui.label("Content 4");
                }
                TAB_5 => {
                    ui.v_stack().nest(|| {
                        ui.label("Content 5");
                        ui.label("Content 5");
                        ui.label("Content 5");
                        ui.label("Content 5");
                    });
                }
                _ => {}
            }
        });

    }

}

fn main() {
    basic_env_logger_init();
    let state = State::default();
    run_example_loop(state, State::update_ui);
}
