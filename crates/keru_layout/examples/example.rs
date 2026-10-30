use keru_layout::*;

/// Measures text with a fake monospace font. Each character is 7 pixels wide and 14 pixels high.
fn measure(_: NodeId, text: Option<&mut &str>, wrap_width: Option<f32>) -> Xy<f32> {
    let chars = text.map_or(0, |text| text.len()) as f32;
    let chars_per_line = wrap_width.map_or(chars, |width| (width / 7.0).floor().max(1.0));
    Xy::new(chars * 7.0, (chars / chars_per_line).ceil() * 14.0)
}

fn stack(axis: Axis, spacing: f32, padding: f32) -> LayoutParams {
    LayoutParams {
        padding: Xy::new([padding, padding], [padding, padding]),
        children_layout: ChildrenLayout::Stack { arrange: Arrange::Start, axis, spacing },
        ..LayoutParams::DEFAULT
    }
}

fn text(size: Xy<Size>) -> LayoutParams {
    LayoutParams { size, text: TextContent::Box, ..LayoutParams::DEFAULT }
}

fn main() {
    let mut tree: LayoutTree<&str> = LayoutTree::new();
    let root = tree.root();

    let hstack = tree.add_node(root, stack(Axis::X, 10.0, 10.0));
    // The height fills the hstack, and the width follows the height.
    tree.add_node(hstack, LayoutParams { size: Xy::new(Size::AspectRatio(0.75), Size::Fill), ..LayoutParams::DEFAULT });

    let vstack = tree.add_node(hstack, stack(Axis::Y, 0.0, 8.0));
    tree.add_node_with_context(vstack, text(Xy::new(Size::FitContent, Size::FitContent)), "Single line text");
    // A `Fill` width doesn't count in the width of the vstack, so the paragraph wraps to the width of the single line text.
    tree.add_node_with_context(vstack, text(Xy::new(Size::Fill, Size::FitContent)), "Wrapping text with a lot of text that overflows. Wrapping text with a lot of text that overflows.");

    tree.set_window_size(Xy::new(400.0, 300.0));
    tree.solve_layout_with_measure(measure);
    for warning in tree.warnings() {
        println!("Warning: {warning}");
    }
    std::fs::write("example.svg", tree.to_svg()).unwrap();
    println!("Wrote example.svg");
}
