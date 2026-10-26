use keru_layout::example_tree::{ExampleTree, ROOT_I};
use keru_layout::*;

fn leaf(w: Size, h: Size) -> LayoutParams {
    LayoutParams { size: Xy::new(w, h), ..LayoutParams::DEFAULT }
}

#[test]
fn no_cycles_in_a_normal_layout() {
    let mut tree = ExampleTree::with_capacity(3, Xy::new(1000.0, 1000.0));
    let parent = tree.add(ROOT_I, LayoutParams::DEFAULT);
    tree.add(parent, leaf(Size::Pixels(100.0), Size::AspectRatio(2.0)));
    tree.solve_layout();
    assert!(tree.cycle_nodes.is_empty());
}

#[test]
fn aspect_ratio_on_both_axes_is_a_cycle() {
    let mut tree = ExampleTree::with_capacity(3, Xy::new(1000.0, 1000.0));
    let parent = tree.add(ROOT_I, LayoutParams::DEFAULT);
    let node = tree.add(parent, leaf(Size::AspectRatio(1.0), Size::AspectRatio(1.0)));
    tree.solve_layout();
    assert_eq!(tree.cycle_nodes, &[node]);
    // The callback is called again on each layout.
    tree.solve_layout();
    assert_eq!(tree.cycle_nodes, &[node, node]);
}
