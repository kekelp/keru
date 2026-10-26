use keru_layout::example_tree::{ExampleTree, NodeI, ROOT_I};
use keru_layout::*;

const WINDOW: f32 = 1000.0;

fn px(size: f32) -> Size {
    Size::Pixels(size)
}

fn weighted(mut params: LayoutParams, weight: f32) -> LayoutParams {
    params.fill_weight = weight;
    params
}

/// A 300x100 stack on the X axis.
fn row() -> LayoutParams {
    let children_layout = ChildrenLayout::Stack { arrange: Arrange::Start, axis: Axis::X, spacing: 0.0 };
    LayoutParams { size: Xy::new(px(300.0), px(100.0)), children_layout, position: Xy::new(Pos::Start, Pos::Start), ..LayoutParams::DEFAULT }
}

fn leaf(w: Size, h: Size) -> LayoutParams {
    LayoutParams { size: Xy::new(w, h), position: Xy::new(Pos::Start, Pos::Start), ..LayoutParams::DEFAULT }
}

/// The size of a node in pixels.
fn size(tree: &ExampleTree, i: NodeI) -> [f32; 2] {
    let r = tree[i].output_rect;
    [r.x[1] - r.x[0], r.y[1] - r.y[0]]
}

fn assert_widths(tree: &ExampleTree, nodes: &[NodeI], expected: &[f32]) {
    for (&i, &e) in nodes.iter().zip(expected) {
        let actual = size(tree, i)[0];
        assert!((actual - e).abs() < 0.01, "node {i:?}: expected width {e}, got {actual}");
    }
}

fn build(children: &[LayoutParams]) -> (ExampleTree, Vec<NodeI>) {
    let mut tree = ExampleTree::with_capacity(children.len() + 2, Xy::new(WINDOW, WINDOW));
    let c = tree.add(ROOT_I, row());
    let children = children.iter().map(|&child| tree.add(c, child)).collect();
    tree.solve_layout();
    (tree, children)
}

#[test]
fn space_is_shared_by_weight() {
    let (tree, k) = build(&[leaf(Size::Fill, px(10.0)), weighted(leaf(Size::Fill, px(10.0)), 2.0)]);
    assert_widths(&tree, &k, &[100.0, 200.0]);
}

#[test]
fn weighted_child_frozen_by_max_size() {
    let mut capped = weighted(leaf(Size::Fill, px(10.0)), 2.0);
    capped.max_size.x = Some(px(100.0));
    let (tree, k) = build(&[leaf(Size::Fill, px(10.0)), capped]);
    assert_widths(&tree, &k, &[200.0, 100.0]);
}

#[test]
fn weighted_fill_max_caps_big_content() {
    let mut big = weighted(leaf(px(500.0), px(10.0)), 2.0);
    big.max_size.x = Some(Size::Fill);
    let (tree, k) = build(&[leaf(Size::Fill, px(10.0)), big]);
    assert_widths(&tree, &k, &[100.0, 200.0]);
}

#[test]
fn weighted_fill_max_keeps_small_content() {
    let mut small = weighted(leaf(px(50.0), px(10.0)), 2.0);
    small.max_size.x = Some(Size::Fill);
    let (tree, k) = build(&[leaf(Size::Fill, px(10.0)), small]);
    assert_widths(&tree, &k, &[250.0, 50.0]);
}

#[test]
fn weighted_fill_min_grows_small_content() {
    let mut small = weighted(leaf(px(50.0), px(10.0)), 2.0);
    small.min_size.x = Some(Size::Fill);
    let (tree, k) = build(&[leaf(Size::Fill, px(10.0)), small]);
    assert_widths(&tree, &k, &[100.0, 200.0]);
}

#[test]
fn weight_has_no_effect_on_the_other_axis() {
    let (tree, k) = build(&[weighted(leaf(Size::Fill, Size::Fill), 3.0)]);
    assert_eq!(size(&tree, k[0])[1].round(), 100.0);
}
