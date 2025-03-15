use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {
    pub current_tab: usize,
    pub show: bool,
}

impl ExampleLoop for State {
    fn update_ui(&mut self, ui: &mut Ui) {
        #[node_key] const C1: NodeKey;
        #[node_key] const SHOULD_BE_HIDDEN_NOT_REMOVED: NodeKey;
        #[node_key] const C3: NodeKey;
        #[node_key] const C4: NodeKey;
        #[node_key] const C5: NodeKey;
        #[node_key] const SHOW: NodeKey;

        let c1 = PANEL
            .color(Color::KERU_DEBUG_RED)
            .size_symm(Size::FitContent)
            .children_can_hide(true)
            .key(C1);
        let c2 = PANEL
            .color(Color::KERU_GREEN)
            .size_symm(Size::FitContent)
            .key(SHOULD_BE_HIDDEN_NOT_REMOVED);
        let c3 = PANEL
            .color(Color::KERU_BLUE)
            .size_symm(Size::FitContent)
            .key(C3);
        let c4 = PANEL
            .color(Color::WHITE)
            .size_symm(Size::FitContent)
            .key(C4);
        let c5 = PANEL
            .color(Color::KERU_RED)
            .size_symm(Size::FitContent)
            .key(C5);

        let show = BUTTON
            .color(Color::KERU_RED)
            .size_symm(Size::FitContent)
            .position_y(Position::End)
            .static_text(&"Show")
            .key(SHOW);


        ui.add(c1).nest(|| {
            if self.show {
                ui.add(c2).nest(|| {
                    ui.add(c3).nest(|| {
                        ui.add(c4).nest(|| {
                            ui.add(c5).nest(|| {
                                ui.static_text_line(&"Suh")
                            });
                        });
                    });
                });
            }
        });

        ui.add(show);
        
        if ui.is_clicked(SHOW) {
            self.show = !self.show;
        }
    
    }
}

fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Warn)
        .filter_module("keru", log::LevelFilter::Info)
        .filter_module("keru::tree", log::LevelFilter::Trace)
        .init();
    let mut state = State::default();
    state.show = true;
    run_example_loop(state);
}
