use keru::*;
use keru::node_library::*;


fn update_ui(_: &mut (), ui: &mut Ui) {
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

    let panel = PANEL.position_y(Pos::End).color(Color::KERU_PINK.with_alpha(0.3));
    let caption = TEXT_PARAGRAPH.text("This is an \"advanced\" layout that showcases Keru's unique dependency-based layout system. Believe it or not, CSS can't do this.").text_size(18.0);

    ui.add(panel).nest(|| {
        ui.add(caption);
    });
}


fn main() {
    example_window_loop::run_example_loop((), update_ui);
}
