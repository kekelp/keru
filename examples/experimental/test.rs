use keru::*;
use keru::node_library::*;
use keru::example_window_loop::*;

#[derive(Default)]
pub struct State {}

const FADE: LinearGradient = LinearGradient::new(Color::KERU_BLUE, Color::KERU_RED, 90.0);

const BODY: &str = "Long wrapping text that makes this v_stack's height dependent on its width... Long wrapping text that makes this v_stack's height dependent on its width... .";

const MORE: &str = "The \"image\" to the left uses AspectRatio to have its width explicitly dependent on its height.\nThen, the outer panel needs to know that width to fit its own width.\nThis is too much for Clay's algorithm, as far as I can tell.";

impl State {
    fn update_ui(&mut self, ui: &mut Ui) {
        ui.add(PANEL.size(Size::Pixels(560.0), Size::FitContent)).nest(|| {

            ui.add(H_STACK.size(Size::Fill, Size::FitContent)).nest(|| {

                ui.add(PANEL.size(Size::AspectRatio(0.75), Size::Fill).linear_gradient(FADE));
                // ui.add(PANEL.size(Size::Frac(0.3), Size::Fill).linear_gradient(FADE));

                let paragraph = TEXT_PARAGRAPH.text_size(16.0).text_alignment(keru_draw::parley::Alignment::Start);

                ui.add(V_STACK.size(Size::Fill, Size::FitContent)).nest(|| {
                    ui.add(paragraph.size(Size::Fill, Size::FitContent).text(BODY));
                    ui.add(paragraph.size(Size::Fill, Size::FitContent).text(MORE));
                });
            });
        });
    }
}

fn main() {
    basic_env_logger_init();
    let state = State::default();
    run_example_loop(state, State::update_ui);
}
