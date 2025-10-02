use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {
    pub show_container: bool,
    pub show_elem_1: bool,
    pub show_elem_2: bool,
}

impl State {
    fn update_ui(&mut self, ui: &mut Ui) {
        #[node_key] const CONTAINER: NodeKey;
        #[node_key] const ELEM_1: NodeKey;
        #[node_key] const ELEM_2: NodeKey;

        #[node_key] const SHOW_CONTAINER: NodeKey;
        #[node_key] const SHOW_ELEM_1: NodeKey;
        #[node_key] const SHOW_ELEM_2: NodeKey;

        let container = PANEL
            .color(Color::KERU_RED)
            .size_symm(Size::Pixels(300))
            .children_can_hide(true)
            .key(CONTAINER);

        let elem_1 = TEXT_EDIT_LINE
            .color(Color::KERU_GREEN)
            .size_symm(Size::FitContent)
            .position_y(Position::Start)
            .static_text("Edit text")
            .key(ELEM_1);

        let elem_2 = TEXT_EDIT_LINE
            .color(Color::KERU_GREEN)
            .position_y(Position::End)
            .size_symm(Size::FitContent)
            .static_text("Write here")
            .key(ELEM_2);

        let show_container = BUTTON.static_text("Remove Container").key(SHOW_CONTAINER);
        let show_elem_1 = BUTTON.static_text("Remove Element 1").key(SHOW_ELEM_1);
        let show_elem_2 = BUTTON.static_text("Hide Element 2").key(SHOW_ELEM_2);

        let description = LABEL.static_text("The red container has children_can_hide = true. So, when the elements are removed, they remain in memory in the background, and their edited text is retained. \n When the container itself is removed, however, the elements that are kept alive by the container's \"children_can_hide\" should be removed as well. So bringing it back should reset the edited text.").position_y(Position::Start);

        let v_stack = V_STACK.size_y(Size::Fill).stack_arrange(Arrange::Start).padding(5);

        if ui.is_clicked(SHOW_CONTAINER) {
            self.show_container = !self.show_container;
        };
        if ui.is_clicked(SHOW_ELEM_1) {
            self.show_elem_1 = !self.show_elem_1;
        };
        if ui.is_clicked(SHOW_ELEM_2) {
            self.show_elem_2 = !self.show_elem_2;
        };

        ui.add(v_stack).nest(|| {
            ui.add(description);

            ui.add(H_STACK).nest(|| {
                ui.add(show_container);
                ui.add(show_elem_1);
                ui.add(show_elem_2);
            });

            if self.show_container {
                ui.add(container).nest(|| {
                    if self.show_elem_1 {
                        ui.add(elem_1);
                    }
                    if self.show_elem_2 {
                        ui.add(elem_2);
                    }
                });
            }
        });

        ui.debug_print_tree();
    }
}

fn main() {
    let state = State {
        show_container: true,
        show_elem_1: true,
        show_elem_2: true,
    };
    run_example_loop(state, State::update_ui);
}
