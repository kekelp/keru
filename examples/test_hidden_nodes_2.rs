use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {
    pub current_tab: usize,
    pub remove1: bool,
    pub hide1: bool,
    pub hide2: bool,
}

impl State {
    fn update_ui(&mut self, ui: &mut Ui) {
        #[node_key] const TAB_CONT: NodeKey;
        #[node_key] const CHECK_IF_THIS_NODE_GETS_REMOVED: NodeKey;
        #[node_key] const CHECK_IF_THIS_NODE_GETS_REMOVED2: NodeKey;
        #[node_key] const SHOW: NodeKey;

        let tab_container = PANEL
            .color(Color::KERU_RED)
            .size_symm(Size::Pixels(300))
            .children_can_hide(true)
            .key(TAB_CONT);

        let check_this = LABEL
            .color(Color::KERU_GREEN)
            .size_symm(Size::FitContent)
            .position_x(Position::Start)
            .static_text("Sneed")
            .key(CHECK_IF_THIS_NODE_GETS_REMOVED);

        let check_this2 = LABEL
            .color(Color::KERU_GREEN)
            .position_x(Position::End)
            .size_symm(Size::FitContent)
            .static_text("Sneed2")
            .key(CHECK_IF_THIS_NODE_GETS_REMOVED2);

        let remove = BUTTON
            .color(Color::KERU_RED)
            .size_symm(Size::FitContent)
            .position_y(Position::End)
            .static_text("Remove Outer");

        let hide1 = BUTTON
            .color(Color::KERU_RED)
            .size_symm(Size::FitContent)
            .position_y(Position::End)
            .static_text("Hide Inner 1");

        let hide2 = BUTTON
            .color(Color::KERU_RED)
            .size_symm(Size::FitContent)
            .position_y(Position::End)
            .static_text("Hide Inner 2");

        // the idea is that when show1 is false, the text node should still get removed, even if it's under tab_container.
        // it only stays as hidden when show2 is false but show1 is true.
        if self.remove1 {
            ui.add(tab_container).nest(|| {
                if self.hide1 {
                    ui.add(check_this);
                }
                if self.hide2 {
                    ui.add(check_this2);
                }
            });
        }
    
        ui.add(H_STACK.position_y(Position::End)).nest(|| {
            if ui.add(remove).is_clicked(ui) {
                self.remove1 = !self.remove1;
            };
            if ui.add(hide1).is_clicked(ui) {
                self.hide1 = !self.hide1;
            };
            if ui.add(hide2).is_clicked(ui) {
                self.hide2 = !self.hide2;
            };
        });
    
    }
}

fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Warn)
        // .filter_module("keru", log::LevelFilter::Info)
        .filter_module("keru::tree", log::LevelFilter::Trace)
        .init();
    let mut state = State::default();
    state.remove1 = true;
    state.hide1 = true;
    run_example_loop(state, State::update_ui);
}
