use keru::*;
use keru::node_library::*;
use keru::example_window_loop::*;

use qrcode::render::svg;
use qrcode::QrCode;

#[derive(Default)]
struct State {
    qr_svg: Option<&'static [u8]>,
}

impl State {
    fn regenerate(&mut self, text: &str) {
        self.qr_svg = QrCode::new(text.as_bytes()).ok().map(|code| {
            let svg = code
                .render::<svg::Color>()
                .min_dimensions(300, 300)
                .dark_color(svg::Color("#000000"))
                .light_color(svg::Color("#ffffff"))
                .build();
            // Leak to obtain a `'static` reference for the image API.
            &*Box::leak(svg.into_bytes().into_boxed_slice())
        });
    }
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const TEXT_EDIT: NodeKey;

    let edit = TEXT_EDIT_LINE
        .key(TEXT_EDIT)
        .size_x(Size::Fill)
        .placeholder_text("Text to encode");

    if let Some(node) = ui.get_node(TEXT_EDIT) && let Some(new_text) = node.text_edit_changed() {
        state.regenerate(new_text);
    }

    let qr = if let Some(svg) = state.qr_svg {
        IMAGE.static_svg(svg).size(Size::Pixels(300.0), Size::Pixels(300.0))
    } else {
        SPACER.size(Size::Pixels(300.0), Size::Pixels(300.0))
    };

    ui.add(V_STACK.size_y(Size::Fill)).nest(|| {
        ui.add(LABEL.text("Enter text to generate QR code:"));
        ui.add(edit);
        ui.add(qr);
    });
}

fn main() {
    let state = State::default();
    run_example_loop(state, update_ui);
}
