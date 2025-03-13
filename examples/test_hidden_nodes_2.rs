use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {
    pub current_tab: usize,
    pub show1: bool,
    pub show2: bool,
}

impl ExampleLoop for State {
    fn update_ui(&mut self, ui: &mut Ui) {
        #[node_key] const TAB_CONT: NodeKey;
        #[node_key] const CHECK_IF_THIS_NODE_GETS_REMOVED: NodeKey;
        #[node_key] const SHOW: NodeKey;

        let tab_container = PANEL
            .color(Color::KERU_RED)
            .size_symm(Size::Pixels(300))
            .children_can_hide(true)
            .key(TAB_CONT);

        let check_this = LABEL
            .color(Color::KERU_GREEN)
            .size_symm(Size::FitContent)
            .static_text("Sneed")
            .key(CHECK_IF_THIS_NODE_GETS_REMOVED);

        let show = BUTTON
            .color(Color::KERU_RED)
            .size_symm(Size::FitContent)
            .position_y(Position::End)
            .static_text("Show");

        let show2 = BUTTON
            .color(Color::KERU_RED)
            .size_symm(Size::FitContent)
            .position_y(Position::End)
            .static_text("Show2");

        // the idea is that when show1 is false, the text node should still get removed, even if it's under tab_container.
        // it only stays as hidden when show2 is false but show1 is true.
        if self.show1 {
            ui.add(tab_container).nest(|| {
                if self.show2 {
                    ui.add(check_this);
                }
            });
        }
    
        ui.add(H_STACK.position_y(Position::End)).nest(|| {
            if ui.add(show).is_clicked(ui) {
                self.show1 = !self.show1;
            };
            if ui.add(show2).is_clicked(ui) {
                self.show2 = !self.show2;
            };
        });
    
    }
}

fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Warn)
        .filter_module("keru", log::LevelFilter::Info)
        .filter_module("keru::tree", log::LevelFilter::Trace)
        .init();
    let mut state = State::default();
    state.show1 = true;
    state.show2 = true;
    run_example_loop(state);
}
