// Renders a UI into an image without opening a window.
//
// Doubles as a headless font smoke test: a named serif family and a CJK line exercise system font
// discovery and glyph fallback. On macOS CI this checks that both work without a window.

use keru::*;
use keru::node_library::*;

fn update_ui(_state: &mut (), ui: &mut Ui) {
    let serif = [TextStyleProperty::FontFamily(FontFamily::Single(FontFamilyName::Generic(GenericFamily::Serif)))];

    ui.add(V_STACK).nest(|| {
        ui.add(LABEL.text("Serif font family").text_properties(&serif));
        ui.add(LABEL.text("你好 こんにちは 안녕하세요"));
        ui.add(IMAGE
            .static_image(include_bytes!("../src/textures/clouds.png"))
            .size(Size::Pixels(160.0), Size::Pixels(90.0)));
    });
}

fn main() {
    let width = 400;
    let height = 320;

    let mut ui = Ui::new_headless(width, height);

    ui.begin_frame();
    update_ui(&mut (), &mut ui);
    ui.finish_frame();

    let image = ui.render_to_image(wgpu::Color { r: 0.1, g: 0.1, b: 0.1, a: 1.0 });

    let out_path = std::env::args().nth(1).unwrap_or_else(|| "headless_render.png".to_string());
    image.save(&out_path).expect("failed to save image");

    println!("Wrote {out_path}");
}
