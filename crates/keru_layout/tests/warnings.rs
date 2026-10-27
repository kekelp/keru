use keru_layout::example_tree::{ExampleTree, NodeI, ROOT_I};
use keru_layout::*;

fn px(size: f32) -> Size {
    Size::Pixels(size)
}

fn leaf(w: Size, h: Size) -> LayoutParams {
    LayoutParams { size: Xy::new(w, h), position: Xy::new(Pos::Start, Pos::Start), ..LayoutParams::DEFAULT }
}

/// A stack on the X axis that fits its content.
fn row() -> LayoutParams {
    let children_layout = ChildrenLayout::Stack { arrange: Arrange::Start, axis: Axis::X, spacing: 0.0 };
    LayoutParams { size: Xy::new(Size::FitContent, Size::FitContent), children_layout, position: Xy::new(Pos::Start, Pos::Start), ..LayoutParams::DEFAULT }
}

/// Solves a node with the given children under the root, and returns the tree and the node.
fn build(params: LayoutParams, children: &[LayoutParams]) -> (ExampleTree, NodeI) {
    let mut tree = ExampleTree::with_capacity(children.len() + 2, Xy::new(1000.0, 1000.0));
    let node = tree.add(ROOT_I, params);
    for &child in children {
        tree.add(node, child);
    }
    tree.solve_layout();
    (tree, node)
}

#[test]
fn no_warnings_in_normal_layouts() {
    let (tree, _) = build(row(), &[leaf(px(100.0), px(10.0)), leaf(Size::Fill, px(10.0))]);
    assert_eq!(tree.warnings, &[]);
    // An empty node that fits its content is zero on purpose.
    let (tree, _) = build(row(), &[]);
    assert_eq!(tree.warnings, &[]);
    // A min size gives the `Fill` child some space.
    let mut params = row();
    params.min_size.x = Some(px(300.0));
    let (tree, _) = build(params, &[leaf(Size::Fill, px(10.0))]);
    assert_eq!(tree.warnings, &[]);
}

#[test]
fn aspect_ratio_on_both_axes() {
    let (mut tree, node) = build(leaf(Size::AspectRatio(1.0), Size::AspectRatio(1.0)), &[]);
    assert!(tree.warnings.contains(&LayoutWarning::AspectRatioOnBothAxes { node }));
    assert!(tree.warnings.iter().any(|warning| matches!(warning, LayoutWarning::Cycle { node: n, .. } if *n == node)));
    // Each warning is reported once for each node, in each run.
    let n = tree.warnings.len();
    tree.solve_layout();
    assert_eq!(tree.warnings.len(), 2 * n);
}

#[test]
fn fit_content_with_only_fill_children_collapses() {
    let (tree, node) = build(row(), &[leaf(Size::Fill, px(10.0)), leaf(Size::Frac(0.5), px(10.0))]);
    assert_eq!(tree.warnings, &[LayoutWarning::FitContentCollapsed { node, axis: Axis::X }]);
}

#[test]
fn min_above_max() {
    let mut params = leaf(px(150.0), px(10.0));
    params.min_size.x = Some(px(200.0));
    params.max_size.x = Some(px(100.0));
    let (tree, node) = build(params, &[]);
    assert_eq!(tree.warnings, &[LayoutWarning::MinAboveMax { node, axis: Axis::X }]);
}

#[test]
fn frac_children_add_up_to_more_than_the_parent() {
    let children = [leaf(px(100.0), px(10.0)), leaf(Size::Frac(0.6), px(10.0)), leaf(Size::Frac(0.6), px(10.0))];
    let (tree, node) = build(row(), &children);
    assert_eq!(tree.warnings, &[LayoutWarning::ImpossibleFracChildren { node, axis: Axis::X }]);
}

#[test]
fn infinite_size() {
    let (tree, node) = build(leaf(px(100.0), Size::AspectRatio(0.0)), &[]);
    assert_eq!(tree.warnings, &[LayoutWarning::NotFinite { node, axis: Axis::Y }]);
}
