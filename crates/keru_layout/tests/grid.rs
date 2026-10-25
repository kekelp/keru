use keru_layout::example_tree::{ExampleTree, NodeI, ROOT_I};
use keru_layout::*;

const WINDOW: f32 = 1000.0;

fn px(size: f32) -> Size {
    Size::Pixels(size)
}

fn grid(columns: MainAxisCellSize, size: Xy<Size>) -> LayoutParams {
    LayoutParams {
        size,
        children_layout: ChildrenLayout::Grid { columns, spacing_x: 0.0, spacing_y: 0.0, flow: GridFlow::DEFAULT },
        position: Xy::new(Pos::Start, Pos::Start),
        ..LayoutParams::DEFAULT
    }
}

fn leaf(w: f32, h: f32) -> LayoutParams {
    LayoutParams { size: Xy::new(px(w), px(h)), ..LayoutParams::DEFAULT }
}

/// The rect of a node in pixels, as `[x, y, w, h]`.
fn rect(tree: &ExampleTree, i: NodeI) -> [f32; 4] {
    let r = tree[i].output_rect;
    [r.x[0] * WINDOW, r.y[0] * WINDOW, (r.x[1] - r.x[0]) * WINDOW, (r.y[1] - r.y[0]) * WINDOW]
}

fn assert_rect(tree: &ExampleTree, i: NodeI, expected: [f32; 4]) {
    let actual = rect(tree, i);
    let close = actual.iter().zip(expected).all(|(a, e)| (a - e).abs() < 0.01);
    assert!(close, "node {i:?}: expected {expected:?}, got {actual:?}");
}

/// Builds a grid with `n` leaf children under the root, solves the layout, and returns the tree, the grid and the children.
fn build(grid_params: LayoutParams, n: usize, child: LayoutParams) -> (ExampleTree, NodeI, Vec<NodeI>) {
    let mut tree = ExampleTree::with_capacity(n + 2, Xy::new(WINDOW, WINDOW));
    let g = tree.add(ROOT_I, grid_params);
    let children = (0..n).map(|_| tree.add(g, child)).collect();
    tree.solve_layout();
    (tree, g, children)
}

#[test]
fn count_grid_fill_children_split_the_cells() {
    let fill = LayoutParams { size: Xy::new(Size::Fill, Size::Fill), ..LayoutParams::DEFAULT };
    let (tree, g, c) = build(grid(MainAxisCellSize::Count(3), Xy::new(px(300.0), px(200.0))), 6, fill);
    assert_rect(&tree, g, [0.0, 0.0, 300.0, 200.0]);
    for (k, &child) in c.iter().enumerate() {
        assert_rect(&tree, child, [(k % 3) as f32 * 100.0, (k / 3) as f32 * 100.0, 100.0, 100.0]);
    }
}

#[test]
fn count_grid_fits_its_content() {
    let fit = Xy::new(Size::FitContent, Size::FitContent);
    let (tree, g, c) = build(grid(MainAxisCellSize::Count(2), fit), 4, leaf(80.0, 40.0));
    assert_rect(&tree, g, [0.0, 0.0, 160.0, 80.0]);
    assert_rect(&tree, c[3], [80.0, 40.0, 80.0, 40.0]);
}

#[test]
fn width_grid_fixed_size_has_as_many_columns_as_fit() {
    let (tree, _, c) = build(grid(MainAxisCellSize::Width(100.0), Xy::new(px(500.0), px(200.0))), 7, leaf(100.0, 50.0));
    // 5 columns and 2 rows of 100px each.
    assert_rect(&tree, c[4], [400.0, 0.0, 100.0, 50.0]);
    assert_rect(&tree, c[5], [0.0, 100.0, 100.0, 50.0]);
}

#[test]
fn width_grid_fitcontent_width_has_one_column() {
    let (tree, g, c) = build(grid(MainAxisCellSize::Width(100.0), Xy::new(Size::FitContent, Size::FitContent)), 3, leaf(100.0, 50.0));
    assert_rect(&tree, g, [0.0, 0.0, 100.0, 150.0]);
    assert_rect(&tree, c[2], [0.0, 100.0, 100.0, 50.0]);
}

#[test]
fn width_grid_with_fitcontent_min_uses_its_final_width() {
    let mut params = grid(MainAxisCellSize::Width(100.0), Xy::new(px(500.0), Size::FitContent));
    params.min_size = Xy::new(Some(Size::FitContent), None);
    let (tree, g, c) = build(params, 7, leaf(100.0, 50.0));
    // The final width is 500px, so there are 5 columns and 2 rows of 50px.
    assert_rect(&tree, g, [0.0, 0.0, 500.0, 100.0]);
    assert_rect(&tree, c[4], [400.0, 0.0, 100.0, 50.0]);
    assert_rect(&tree, c[5], [0.0, 50.0, 100.0, 50.0]);
}

#[test]
fn width_grid_fill_children_follow_the_final_size() {
    let fill = LayoutParams { size: Xy::new(Size::Fill, Size::Fill), ..LayoutParams::DEFAULT };
    let (tree, _, c) = build(grid(MainAxisCellSize::Width(100.0), Xy::new(px(250.0), px(100.0))), 4, fill);
    // 2 columns of 125px, 2 rows of 50px.
    assert_rect(&tree, c[1], [125.0, 0.0, 125.0, 50.0]);
    assert_rect(&tree, c[3], [125.0, 50.0, 125.0, 50.0]);
}

#[test]
fn width_grid_counts_spacing_and_padding() {
    let mut params = grid(MainAxisCellSize::Width(100.0), Xy::new(px(360.0), Size::FitContent));
    params.children_layout = ChildrenLayout::Grid { columns: MainAxisCellSize::Width(100.0), spacing_x: 10.0, spacing_y: 5.0, flow: GridFlow::DEFAULT };
    params.padding = Xy::new([20.0, 20.0], [20.0, 20.0]);
    let (tree, g, c) = build(params, 4, leaf(100.0, 50.0));
    // The inner width is 320px: 3 columns of 100px with 10px between them. The height is 2 rows with 5px between them, plus padding.
    assert_rect(&tree, g, [0.0, 0.0, 360.0, 145.0]);
    assert_rect(&tree, c[2], [240.0, 20.0, 100.0, 50.0]);
    assert_rect(&tree, c[3], [20.0, 75.0, 100.0, 50.0]);
}

#[test]
fn width_grid_on_the_y_axis() {
    let mut params = grid(MainAxisCellSize::Width(100.0), Xy::new(Size::FitContent, px(300.0)));
    params.children_layout = ChildrenLayout::Grid { columns: MainAxisCellSize::Width(100.0), spacing_x: 0.0, spacing_y: 0.0, flow: GridFlow { main_axis: Axis::Y, ..GridFlow::DEFAULT } };
    let (tree, g, c) = build(params, 5, leaf(60.0, 100.0));
    // 3 cells per column on Y, so 2 columns on X.
    assert_rect(&tree, g, [0.0, 0.0, 120.0, 300.0]);
    assert_rect(&tree, c[2], [0.0, 200.0, 60.0, 100.0]);
    assert_rect(&tree, c[3], [60.0, 0.0, 60.0, 100.0]);
}
