use keru::*;
use keru::example_window_loop::*;

#[derive(Default)]
pub struct State {
    pub tab_number: usize,
}

pub struct UiPlacedTabView {
    pub content: UiPlacedNode,
}
impl UiPlacedTabView {
    fn nest(&self, nest1: impl FnOnce()) -> &Self {
        self.content.nest(|| {
            nest1();
        });
        return self;
    }
}

pub trait CustomWidgets {
    fn tab_view(&mut self, tabs: &[&str], tab_number: &mut usize) -> UiPlacedTabView;
}

impl CustomWidgets for Ui {
    fn tab_view(&mut self, tabs: &[&str], tab_number: &mut usize) -> UiPlacedTabView {
        let mut result: Option<UiPlacedTabView> = None;

        if *tab_number >= 5 {
            *tab_number = 5;
        }

        #[node_key] const ROOT_CONT: NodeKey;
        #[node_key] const CONTENT: NodeKey;
        #[node_key] const TAB_BUTTON: NodeKey;
        
        self.anon_subtree().start(|| {

            self.add(ROOT_CONT)
                .params(CONTAINER)
                .size_symm(Size::Fill);

            self.place(ROOT_CONT).nest(|| {
                
                // todo: focused?
                if self.key_input().key_pressed_or_repeated(&winit::keyboard::Key::Named(winit::keyboard::NamedKey::Tab)) {
                    if self.key_mods().shift_key() {
                        *tab_number = (((*tab_number as isize) - 1 + 5) % 5) as usize;
                    } else {
                        *tab_number = (*tab_number + 1) % 5;
                    }
                }

                self.v_stack().nest(|| {
                    self.h_stack().nest(|| {

                        for (i, name) in tabs.iter().enumerate() {
                            let key = TAB_BUTTON.sibling(i);
                            
                            self.add(key).params(BUTTON).text(name).place();

                            if self.is_clicked(key) {
                                *tab_number = i;
                            }
                        }
                    });

                    let content_node = self.add(CONTENT).params(PANEL).place();
                    
                    result = Some(UiPlacedTabView {
                        content: content_node,
                    });
                });

            })
        });
        
        return result.unwrap();
    }
}

impl ExampleLoop for State {
    fn update_ui(&mut self, ui: &mut Ui) {

        let tabs = [
            "Tab 1",
            "Tab 2",
            "Tab 3",
            "Tab 4",
            "Tab 5",
        ];

        ui.tab_view(&tabs, &mut self.tab_number).nest(|| {
            match tabs[self.tab_number] {
                "Tab 1" => {
                    ui.label("Content 1");
                }
                "Tab 2" => {
                    ui.label("Content 2");
                }
                "Tab 3" => {
                    ui.v_stack().nest(|| {
                        ui.label("Content 3");
                        ui.label("Content 3");
                        ui.label("Content 3");
                    });
                }
                "Tab 4" => {
                    ui.label("Content 4");
                }
                "Tab 5" => {
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
    run_example_loop(state);
}
