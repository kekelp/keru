use keru::*;
use keru::node_library::*;
use keru::example_window_loop::*;

struct State {
    qr_image: Option<LoadedImageHandle>,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const TEXT_EDIT: NodeKey;

    let edit = TEXT_EDIT_LINE
        .key(TEXT_EDIT)
        .size_x(Size::Fill)
        .placeholder_text("Text to encode");

    if let Some(node) = ui.get_node(TEXT_EDIT) && let Some(new_text) = node.text_edit_changed() {
        state.qr_image = None;
        if let Ok(code) = qrcode::QrCode::new(new_text.as_bytes()) {
            let img = code.render::<image::Luma<u8>>().build();
            let (width, height) = (img.width(), img.height());
            let rgba = image::DynamicImage::ImageLuma8(img).into_rgba8();
            state.qr_image = ui.load_rgba_image(rgba.as_raw(), width, height);
        }
    }

    let qr = if let Some(qr_image) = &state.qr_image {
        IMAGE.image(qr_image).size(Size::Pixels(300.0), Size::Pixels(300.0))
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
    let state = State { qr_image: None };
    run_example_loop(state, update_ui);
}
