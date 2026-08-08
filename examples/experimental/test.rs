use keru::*;
use keru::node_library::*;
use keru::example_window_loop::*;

#[derive(Default)]
pub struct State {
    f: f32,
}

#[node_key] const K1: NodeKey;
#[node_key] const K2: NodeKey;
#[node_key] const K3: NodeKey;
#[node_key] const VSTACKKEY: NodeKey;
#[node_key] const WRAP: NodeKey;
#[node_key] const REDFILL: NodeKey;
#[node_key] const K7: NodeKey;
#[node_key] const K8: NodeKey;
#[node_key] const PANELKEY: NodeKey;
#[node_key] const K10: NodeKey;

impl State {
    fn update_ui(&mut self, ui: &mut Ui) {
        // Giraffe
        #[node_key] const HSTACK: NodeKey;
        #[node_key] const GIRAFFE: NodeKey;
        #[node_key] const PARAGRAPH: NodeKey;
        #[node_key] const VSTACK: NodeKey;

        let giraffe = PANEL
            .size_y(Size::Fill)
            .size_x(Size::AspectRatio(0.5))
            .linear_gradient(LinearGradient { color_start: Color::KERU_BLUE, color_end: Color::KERU_RED, angle_deg: 0.0 })
            .key(GIRAFFE);

        let paragraph = TEXT_PARAGRAPH
            .size_x(Size::Fill)
            .text_size(14.0)
            .text_alignment(TextAlignment::Justify)
            .text("Wrapping text with a lot of text that overflows. Wrapping text with a lot of text that overflows. Wrapping text with a lot of text that overflows. Wrapping text with a lot of text that overflows.");

        let h_stack = H_STACK.size_symm(Size::FitContent).key(HSTACK).color(Color::KERU_PINK).padding(10.0);
        
        ui.add(h_stack).nest(|| {
            ui.add(giraffe);
            ui.add(V_STACK.key(VSTACK).size_symm(Size::FitContent)).nest(|| {
                ui.add(TEXT.text("Single line text"));
                ui.add(paragraph);
            })
        });
    }
}

fn main() {
    basic_env_logger_init();
    let state = State::default();
    run_example_loop(state, State::update_ui);
}
