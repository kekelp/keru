use keru_layout::example_tree::{ExampleTree, NodeI, ROOT_I};
use keru_layout::*;

fn px(size: f32) -> Size {
    Size::Pixels(size)
}

/// A stack on the X axis that fits its content.
fn row() -> LayoutParams {
    let children_layout = ChildrenLayout::Stack { arrange: Arrange::Start, axis: Axis::X, spacing: 0.0 };
    LayoutParams { size: Xy::new(Size::FitContent, Size::FitContent), children_layout, position: Xy::new(Pos::Start, Pos::Start), ..LayoutParams::DEFAULT }
}

fn leaf(w: Size, h: Size) -> LayoutParams {
    LayoutParams { size: Xy::new(w, h), position: Xy::new(Pos::Start, Pos::Start), ..LayoutParams::DEFAULT }
}

fn size(tree: &ExampleTree, i: NodeI) -> [f32; 2] {
    let r = tree[i].output_rect;
    [r.x[1] - r.x[0], r.y[1] - r.y[0]]
}

fn assert_size(tree: &ExampleTree, i: NodeI, expected: [f32; 2]) {
    let actual = size(tree, i);
    let close = actual.iter().zip(expected).all(|(a, e)| (a - e).abs() < 0.01);
    assert!(close, "node {i:?}: expected {expected:?}, got {actual:?}");
}

fn build(row: LayoutParams, children: &[LayoutParams]) -> (ExampleTree, NodeI, Vec<NodeI>) {
    let mut tree = ExampleTree::with_capacity(children.len() + 2, Xy::new(1000.0, 1000.0));
    let r = tree.add(ROOT_I, row);
    let children = children.iter().map(|&child| tree.add(r, child)).collect();
    tree.solve_layout();
    assert!(tree.cycle_nodes.is_empty());
    (tree, r, children)
}

#[test]
fn fill_gets_no_space_in_a_row_that_fits_its_content() {
    let (tree, r, k) = build(row(), &[leaf(px(100.0), px(10.0)), leaf(Size::Fill, px(10.0))]);
    assert_size(&tree, r, [100.0, 10.0]);
    assert_size(&tree, k[1], [0.0, 10.0]);
}

#[test]
fn fill_gets_the_space_above_the_content_from_a_min_size() {
    let mut params = row();
    params.min_size.x = Some(px(300.0));
    let (tree, r, k) = build(params, &[leaf(px(100.0), px(10.0)), leaf(Size::Fill, px(10.0))]);
    assert_size(&tree, r, [300.0, 10.0]);
    assert_size(&tree, k[0], [100.0, 10.0]);
    assert_size(&tree, k[1], [200.0, 10.0]);
}

#[test]
fn fill_max_keeps_the_content_size() {
    let mut small = leaf(px(50.0), px(10.0));
    small.max_size.x = Some(Size::Fill);
    let mut big = leaf(px(150.0), px(10.0));
    big.max_size.x = Some(Size::Fill);
    let (tree, r, k) = build(row(), &[small, big]);
    assert_size(&tree, r, [200.0, 10.0]);
    assert_size(&tree, k[0], [50.0, 10.0]);
    assert_size(&tree, k[1], [150.0, 10.0]);
}

#[test]
fn fill_min_grows_to_the_share() {
    let mut params = row();
    params.min_size.x = Some(px(300.0));
    let mut small = leaf(px(50.0), px(10.0));
    small.min_size.x = Some(Size::Fill);
    let (tree, _, k) = build(params, &[small, leaf(Size::Fill, px(10.0))]);
    assert_size(&tree, k[0], [150.0, 10.0]);
    assert_size(&tree, k[1], [150.0, 10.0]);
}

#[test]
fn fit_content_max_ignores_fill_children() {
    let mut params = row();
    params.size.x = Size::Fill;
    params.max_size.x = Some(Size::FitContent);
    let (tree, r, k) = build(params, &[leaf(px(100.0), px(10.0)), leaf(Size::Fill, px(10.0))]);
    assert_size(&tree, r, [100.0, 10.0]);
    assert_size(&tree, k[1], [0.0, 10.0]);
}

#[test]
fn fill_on_the_cross_axis_stretches_to_the_tallest_sibling() {
    let (tree, r, k) = build(row(), &[leaf(px(10.0), px(40.0)), leaf(px(10.0), Size::Fill)]);
    assert_size(&tree, r, [20.0, 40.0]);
    assert_size(&tree, k[1], [10.0, 40.0]);
}
