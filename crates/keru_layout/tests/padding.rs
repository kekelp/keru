use keru_layout::example_tree::{ExampleTree, NodeI, ROOT_I};
use keru_layout::*;

const WINDOW: f32 = 1000.0;

/// Padding of 10px on the left, 30px on the right, 5px on the top and 15px on the bottom.
const PADDING: Xy<[f32; 2]> = Xy::new([10.0, 30.0], [5.0, 15.0]);

fn px(size: f32) -> Size {
    Size::Pixels(size)
}

fn container(size: Xy<Size>, children_layout: ChildrenLayout) -> LayoutParams {
    LayoutParams { size, padding: PADDING, children_layout, position: Xy::new(Pos::Start, Pos::Start), ..LayoutParams::DEFAULT }
}

fn stack(axis: Axis, arrange: Arrange) -> ChildrenLayout {
    ChildrenLayout::Stack { arrange, axis, spacing: 0.0 }
}

fn leaf(w: Size, h: Size) -> LayoutParams {
    LayoutParams { size: Xy::new(w, h), position: Xy::new(Pos::Start, Pos::Start), ..LayoutParams::DEFAULT }
}

/// The rect of a node in pixels, as `[x, y, w, h]`.
fn rect(tree: &ExampleTree, i: NodeI) -> [f32; 4] {
    let r = tree[i].output_rect;
    [r.x[0], r.y[0], r.x[1] - r.x[0], r.y[1] - r.y[0]]
}

fn assert_rect(tree: &ExampleTree, i: NodeI, expected: [f32; 4]) {
    let actual = rect(tree, i);
    let close = actual.iter().zip(expected).all(|(a, e)| (a - e).abs() < 0.01);
    assert!(close, "node {i:?}: expected {expected:?}, got {actual:?}");
}

/// Builds a container under the root with the given children, solves the layout, and returns the tree, the container and the children.
fn build(params: LayoutParams, children: &[LayoutParams]) -> (ExampleTree, NodeI, Vec<NodeI>) {
    let mut tree = ExampleTree::with_capacity(children.len() + 2, Xy::new(WINDOW, WINDOW));
    let c = tree.add(ROOT_I, params);
    let children = children.iter().map(|&child| tree.add(c, child)).collect();
    tree.solve_layout();
    (tree, c, children)
}

#[test]
fn fitcontent_stack_adds_both_sides() {
    let fit = Xy::new(Size::FitContent, Size::FitContent);
    let (tree, c, k) = build(container(fit, stack(Axis::Y, Arrange::Start)), &[leaf(px(50.0), px(20.0)); 2]);
    assert_rect(&tree, c, [0.0, 0.0, 90.0, 60.0]);
    assert_rect(&tree, k[0], [10.0, 5.0, 50.0, 20.0]);
    assert_rect(&tree, k[1], [10.0, 25.0, 50.0, 20.0]);
}

#[test]
fn fill_child_takes_the_inner_area() {
    let fixed = Xy::new(px(200.0), px(100.0));
    let (tree, _, k) = build(container(fixed, stack(Axis::X, Arrange::Start)), &[leaf(Size::Fill, Size::Fill)]);
    assert_rect(&tree, k[0], [10.0, 5.0, 160.0, 80.0]);
}

#[test]
fn stack_arrange_uses_the_inner_area() {
    let fixed = Xy::new(px(200.0), px(100.0));
    let child = leaf(px(40.0), px(20.0));
    let (tree, _, k) = build(container(fixed, stack(Axis::X, Arrange::End)), &[child]);
    assert_rect(&tree, k[0], [130.0, 5.0, 40.0, 20.0]);
    let (tree, _, k) = build(container(fixed, stack(Axis::X, Arrange::Center)), &[child]);
    // The inner area goes from 10 to 170.
    assert_rect(&tree, k[0], [70.0, 5.0, 40.0, 20.0]);
}

#[test]
fn free_children_use_the_inner_area() {
    let fixed = Xy::new(px(200.0), px(100.0));
    let mut end = leaf(px(40.0), px(20.0));
    end.position = Xy::new(Pos::End, Pos::End);
    let mut center = end;
    center.position = Xy::new(Pos::Center, Pos::Center);
    let (tree, _, k) = build(container(fixed, ChildrenLayout::Free), &[end, center]);
    assert_rect(&tree, k[0], [130.0, 65.0, 40.0, 20.0]);
    // The inner area goes from 10 to 170 on X and from 5 to 85 on Y.
    assert_rect(&tree, k[1], [70.0, 35.0, 40.0, 20.0]);
}

#[test]
fn grid_cells_start_after_the_start_padding() {
    let fixed = Xy::new(px(240.0), px(100.0));
    let grid = ChildrenLayout::Grid { columns: MainAxisCellSize::Count(2), spacing_x: 0.0, spacing_y: 0.0, flow: GridFlow::DEFAULT };
    let (tree, _, k) = build(container(fixed, grid), &[leaf(Size::Fill, Size::Fill); 2]);
    // The inner area is 200px wide and 80px high, so each cell is 100px by 80px.
    assert_rect(&tree, k[0], [10.0, 5.0, 100.0, 80.0]);
    assert_rect(&tree, k[1], [110.0, 5.0, 100.0, 80.0]);
}
