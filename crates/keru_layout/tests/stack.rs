use keru_layout::example_tree::{ExampleTree, NodeI, ROOT_I};
use keru_layout::*;

fn px(size: f32) -> Size {
    Size::Pixels(size)
}

/// A 300x100 stack on the X axis, with 10px of padding on the left and 20px on the right.
fn row(arrange: Arrange, flipped: bool) -> LayoutParams {
    let children_layout = ChildrenLayout::Stack { arrange, axis: Axis::X, spacing: 10.0 };
    LayoutParams {
        size: Xy::new(px(300.0), px(100.0)),
        padding: Xy::new([10.0, 20.0], [0.0, 0.0]),
        children_layout,
        children_origin_flipped: Xy::new(flipped, false),
        position: Xy::new(Pos::Start, Pos::Start),
        ..LayoutParams::DEFAULT
    }
}

fn leaf(w: f32) -> LayoutParams {
    LayoutParams { size: Xy::new(px(w), px(10.0)), position: Xy::new(Pos::Start, Pos::Start), ..LayoutParams::DEFAULT }
}

/// The left and right edges of a node.
fn x_range(tree: &ExampleTree, i: NodeI) -> [f32; 2] {
    tree[i].output_rect.x
}

fn build(params: LayoutParams) -> (ExampleTree, Vec<NodeI>) {
    let mut tree = ExampleTree::with_capacity(4, Xy::new(1000.0, 1000.0));
    let r = tree.add(ROOT_I, params);
    let children = vec![tree.add(r, leaf(50.0)), tree.add(r, leaf(100.0))];
    tree.solve_layout();
    (tree, children)
}

#[test]
fn start() {
    let (tree, k) = build(row(Arrange::Start, false));
    assert_eq!(x_range(&tree, k[0]), [10.0, 60.0]);
    assert_eq!(x_range(&tree, k[1]), [70.0, 170.0]);
}

#[test]
fn flipped_start_begins_at_the_end_in_reverse_order() {
    let (tree, k) = build(row(Arrange::Start, true));
    assert_eq!(x_range(&tree, k[0]), [230.0, 280.0]);
    assert_eq!(x_range(&tree, k[1]), [120.0, 220.0]);
}

#[test]
fn flipped_end_packs_against_the_start() {
    let (tree, k) = build(row(Arrange::End, true));
    assert_eq!(x_range(&tree, k[0]), [120.0, 170.0]);
    assert_eq!(x_range(&tree, k[1]), [10.0, 110.0]);
}

#[test]
fn flipped_space_between_reverses_the_order() {
    let (tree, k) = build(row(Arrange::SpaceBetween, true));
    assert_eq!(x_range(&tree, k[0]), [230.0, 280.0]);
    assert_eq!(x_range(&tree, k[1]), [10.0, 110.0]);
}
