use keru::*;
use keru::example_window_loop::*;

#[derive(Default)]
pub struct State {
    pub left_strings: Vec<String>,
    pub right_strings: Vec<String>,
}

impl State {
    fn update_ui(&mut self, ui: &mut Ui) {
        #[node_key] const LEFT_STACK: NodeKey;
        #[node_key] const RIGHT_STACK: NodeKey;

        #[node_key] const ITEM: NodeKey;

        let item = BUTTON
            .size_x(Size::Pixels(100.0))
            .anchor_symm(Anchor::Center)
            .sense_drag(true)
            .absorbs_clicks(false)
            .animate_position(true)
            .animation(Animation {
                speed: 1.0,
                enter: EnterAnimation::None,
                exit: ExitAnimation::None,
                state_transition: StateTransition {
                    animate_position: true,
                },
            });
        
        let stack = V_STACK
            .padding(50.0)
            .size_y(Size::Fill)
            .position_y(Pos::Start)
            .sense_drag_drop_target(true)
            .stack_arrange(Arrange::Start)
;
        let left_stack = stack.position_x(Pos::Start).key(LEFT_STACK);

        ui.add(left_stack).nest(|| {
            for string in &self.left_strings {

                let key = ITEM.sibling(string);
                let item = item.text(&string).key(key);
                
                if let Some(drag) = ui.is_dragged(key) {
                    let (x, y) = (Pos::Pixels(drag.absolute_pos.x), Pos::Pixels(drag.absolute_pos.y));
                    
                    ui.jump_to_root().nest(|| {
                        ui.add(item.position(x, y));
                    });

                } else {
                    ui.add(item);
                }
                
            }
        });

        
        let right_stack = stack.position_x(Pos::End).key(RIGHT_STACK);

        ui.add(right_stack).nest(|| {
            for string in &self.right_strings {

                let key = ITEM.sibling(string);
                let item = item.text(&string).key(key);
                
                if let Some(drag) = ui.is_dragged(key) {
                    let (x, y) = (Pos::Pixels(drag.absolute_pos.x), Pos::Pixels(drag.absolute_pos.y));
                    
                    ui.jump_to_root().nest(|| {
                        ui.add(item.position(x, y));
                    });

                } else {
                    ui.add(item);
                }
                
            }
        });

        let mut to_remove: Option<usize> = None;
        for (i, string) in self.left_strings.iter().enumerate() {
            let key = ITEM.sibling(string);
            if ui.is_drag_released_onto(key, RIGHT_STACK) {
                to_remove = Some(i);
            }
        }
        if let Some(to_remove) = to_remove {
            let removed = self.left_strings.remove(to_remove);
            self.right_strings.push(removed);
        }

        let mut to_remove: Option<usize> = None;
        for (i, string) in self.right_strings.iter().enumerate() {
            let key = ITEM.sibling(string);
            if ui.is_drag_released_onto(key, LEFT_STACK) {
                to_remove = Some(i);
            }
        }
        if let Some(to_remove) = to_remove {
            let removed = self.right_strings.remove(to_remove);
            self.left_strings.push(removed);
        }


    }

}

fn main() {
    // basic_env_logger_init();
    let mut state = State::default();
    state.left_strings = vec!["1".into(), "2".into(), "3".into(), "4".into(), "5".into()];
    state.right_strings = vec!["a".into(), "b".into(), "c".into(), "d".into(), "e".into()];
    run_example_loop(state, State::update_ui);
}
