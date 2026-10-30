use keru_layout::*;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// The context of each node is the length of a fake text.
type Tree = LayoutTree<u32>;

/// Measures the fake text. Each character is 8 pixels wide and 16 pixels high, and a wrapped line has as many characters as fit.
fn measure(_: NodeId, chars: Option<&mut u32>, wrap_width: Option<f32>) -> Xy<f32> {
    let chars = chars.map_or(0, |chars| *chars);
    let chars_per_line = match wrap_width {
        Some(width) => ((width / 8.0).floor() as u32).max(1),
        None => chars.max(1),
    };
    Xy::new(chars as f32 * 8.0, chars.div_ceil(chars_per_line) as f32 * 16.0)
}

fn random_axis(rng: &mut StdRng) -> Axis {
    if rng.random_bool(0.5) { Axis::X } else { Axis::Y }
}

fn random_size(rng: &mut StdRng) -> Size {
    match rng.random_range(0..20) {
        0..7 => Size::Pixels(rng.random_range(10.0..100.0f32).round()),
        7..12 => Size::FitContent,
        12..16 => Size::Fill,
        16..19 => Size::Frac(rng.random_range(0.1..0.5)),
        _ => Size::AspectRatio(rng.random_range(0.5..2.0)),
    }
}

fn random_min_size(rng: &mut StdRng) -> Size {
    match rng.random_range(0..10) {
        0..2 => Size::Frac(rng.random_range(0.05..0.3)),
        2..4 => Size::FitContent,
        4 => Size::AspectRatio(rng.random_range(0.5..2.0)),
        _ => Size::Pixels(rng.random_range(5.0..40.0f32).round()),
    }
}

fn random_max_size(rng: &mut StdRng) -> Size {
    match rng.random_range(0..10) {
        0..2 => Size::Frac(rng.random_range(0.5..1.0)),
        2..4 => Size::FitContent,
        4 => Size::AspectRatio(rng.random_range(0.5..2.0)),
        _ => Size::Pixels(rng.random_range(40.0..200.0f32).round()),
    }
}

fn random_pos(rng: &mut StdRng) -> Pos {
    match rng.random_range(0..5) {
        0 => Pos::Start,
        1 => Pos::Center,
        2 => Pos::End,
        3 => Pos::Pixels(rng.random_range(0.0..50.0f32).round()),
        _ => Pos::Frac(rng.random_range(0.0..1.0)),
    }
}

fn random_anchor(rng: &mut StdRng) -> Anchor {
    match rng.random_range(0..4) {
        0 => Anchor::Start,
        1 => Anchor::Center,
        2 => Anchor::End,
        _ => Anchor::Frac(rng.random_range(0.0..1.0)),
    }
}

fn random_arrange(rng: &mut StdRng) -> Arrange {
    [Arrange::Start, Arrange::End, Arrange::Center, Arrange::SpaceBetween, Arrange::SpaceAround, Arrange::SpaceEvenly][rng.random_range(0..6)]
}

fn random_window_overflow(rng: &mut StdRng) -> WindowOverflow {
    [WindowOverflow::Ignore, WindowOverflow::Clamp, WindowOverflow::Flip][rng.random_range(0..3)]
}

fn random_grid_flow(rng: &mut StdRng) -> GridFlow {
    let mut direction = || if rng.random_bool(0.5) { Direction::LeftToRight } else { Direction::RightToLeft };
    let (x_fill_direction, y_fill_direction) = (direction(), direction());
    GridFlow { main_axis: random_axis(rng), x_fill_direction, y_fill_direction, backfill: rng.random_bool(0.3) }
}

/// A size of the same kind with a new number, or `None` if the kind has no number.
fn renumber(size: Size, rng: &mut StdRng) -> Option<Size> {
    match size {
        Size::Pixels(_) => Some(Size::Pixels(rng.random_range(5.0..200.0f32).round())),
        Size::Frac(_) => Some(Size::Frac(rng.random_range(0.05..1.0))),
        Size::AspectRatio(_) => Some(Size::AspectRatio(rng.random_range(0.5..2.0))),
        _ => None,
    }
}

fn random_children_layout(rng: &mut StdRng) -> ChildrenLayout {
    let axis = random_axis(rng);
    match rng.random_range(0..10) {
        0..3 => ChildrenLayout::Free,
        3..8 => ChildrenLayout::Stack { arrange: random_arrange(rng), axis, spacing: rng.random_range(0.0..10.0f32).round() },
        8 => ChildrenLayout::Grid { columns: MainAxisCellSize::Count(rng.random_range(1..5)), spacing_x: 4.0, spacing_y: 4.0, flow: random_grid_flow(rng) },
        _ => ChildrenLayout::Grid { columns: MainAxisCellSize::Width(rng.random_range(20.0..60.0f32).round()), spacing_x: 4.0, spacing_y: 4.0, flow: random_grid_flow(rng) },
    }
}

fn random_params(rng: &mut StdRng) -> LayoutParams {
    let mut size = Xy::new(random_size(rng), random_size(rng));
    if let (Size::AspectRatio(_), Size::AspectRatio(_)) = (size.x, size.y) {
        size.y = Size::FitContent;
    }
    let mut params = LayoutParams {
        size,
        fill_weight: rng.random_range(0.5..2.0),
        padding: Xy::new([rng.random_range(0.0..8.0f32).round(), 0.0], [0.0, rng.random_range(0.0..8.0f32).round()]),
        children_layout: random_children_layout(rng),
        free_placement: rng.random_bool(0.08),
        disabled: rng.random_bool(0.03),
        position: Xy::new(random_pos(rng), random_pos(rng)),
        anchor: Xy::new(random_anchor(rng), random_anchor(rng)),
        children_origin_flipped: Xy::new(rng.random_bool(0.2), rng.random_bool(0.2)),
        window_overflow: Xy::new(random_window_overflow(rng), random_window_overflow(rng)),
        text: match rng.random_range(0..20) {
            0..3 => TextContent::Box,
            3 => TextContent::Edit,
            _ => TextContent::None,
        },
        ..LayoutParams::DEFAULT
    };
    for axis in [Axis::X, Axis::Y] {
        if rng.random_bool(0.15) {
            params.min_size[axis] = Some(random_min_size(rng));
        }
        if rng.random_bool(0.1) {
            params.max_size[axis] = Some(random_max_size(rng));
        }
    }
    if rng.random_bool(0.15) {
        params.grid_span = Xy::new(rng.random_range(1..4), rng.random_range(1..4));
    }
    if rng.random_bool(0.2) {
        params.image = Some(Xy::new(rng.random_range(5.0..50.0f32).round(), rng.random_range(5.0..50.0f32).round()));
    }
    params
}

/// Adds a random subtree. The tree disables the children of a disabled node, as `LayoutParams::disabled` requires.
fn add_random_subtree(tree: &mut Tree, rng: &mut StdRng, parent: NodeId, depth: usize, budget: &mut usize) -> NodeId {
    let i = tree.add_node_with_context(parent, random_params(rng), rng.random_range(0..80));
    *budget = budget.saturating_sub(1);
    if depth > 0 {
        let n_children = rng.random_range(0..7);
        for _ in 0..n_children {
            if *budget == 0 {
                break;
            }
            add_random_subtree(tree, rng, i, depth - 1, budget);
        }
    }
    i
}

fn random_tree(rng: &mut StdRng) -> Tree {
    let mut tree = Tree::with_capacity(128);
    tree.set_window_size(Xy::new(800.0, 600.0));
    let root = tree.root();
    let mut params = *tree.params(root);
    params.children_layout = random_children_layout(rng);
    params.padding = Xy::new([rng.random_range(0.0..8.0f32).round(); 2], [rng.random_range(0.0..8.0f32).round(); 2]);
    tree.update_node_params(root, params);
    let mut budget = rng.random_range(5..200);
    let depth = rng.random_range(2..10);
    while budget > 0 {
        add_random_subtree(&mut tree, rng, root, depth, &mut budget);
    }
    tree
}

/// A random tree with its layout solved. Every relayout of a tree with a cycle is a full solve, so it doesn't test the incremental solver.
fn solved_random_tree_without_cycle(rng: &mut StdRng) -> Tree {
    loop {
        let mut tree = random_tree(rng);
        tree.solve_layout_with_measure(measure);
        if ! tree.warnings().iter().any(|warning| matches!(warning, LayoutWarning::Cycle { .. })) {
            return tree;
        }
    }
}

/// The nodes that are reachable from the root. It includes disabled nodes, but not their children.
fn reachable(tree: &Tree) -> Vec<NodeId> {
    let mut nodes = Vec::new();
    let mut stack = vec![tree.root()];
    while let Some(i) = stack.pop() {
        nodes.push(i);
        if ! tree.params(i).disabled {
            stack.extend(tree.children(i));
        }
    }
    nodes
}

fn assert_same_layout(incremental: &Tree, full: &Tree, context: &str) {
    for i in reachable(full) {
        if full.params(i).disabled {
            continue;
        }
        assert_eq!(incremental.layout(i), full.layout(i), "{context}: rect of {i:?}");
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mutation {
    /// A number changes: a size, a min or max size, a padding, a fill weight, a spacing, a grid cell size or an image size. The dependency graph stays the same.
    Number,
    /// The text of a node changes length.
    Content,
    /// The `Size` of a node changes kind on one axis, for example from `Fill` to `FitContent`.
    SizeKind,
    /// The `ChildrenLayout` of a node changes.
    Layout,
    /// The position, the anchor, the children origin, the window overflow or the stack arrangement of a node changes. It affects only the placement.
    Placement,
    /// A subtree is disabled or enabled.
    Toggle,
    /// The window is resized.
    Window,
    /// A node is added.
    Add,
    /// A node and its descendants are removed.
    Remove,
    /// A node moves to a new parent.
    Move,
}

/// The mutations that are never structural.
const VALUE_MUTATIONS: [Mutation; 4] = [Mutation::Number, Mutation::Content, Mutation::Placement, Mutation::Window];

/// The mutations that change the structure.
const STRUCTURAL_MUTATIONS: [Mutation; 6] = [Mutation::SizeKind, Mutation::Layout, Mutation::Toggle, Mutation::Add, Mutation::Remove, Mutation::Move];

/// Changes the tree, which records the change by itself. It usually changes one of the `hot` nodes, if they are enabled, as an animation does.
fn mutate(tree: &mut Tree, rng: &mut StdRng, mutation: Mutation, hot: &[NodeId]) {
    let nodes = reachable(tree);
    let enabled: Vec<NodeId> = nodes.iter().copied().filter(|&i| ! tree.params(i).disabled).collect();
    let non_root: Vec<NodeId> = nodes.iter().copied().filter(|&i| i != tree.root()).collect();
    let hot: Vec<NodeId> = hot.iter().copied().filter(|i| enabled.contains(i)).collect();
    let i = match hot.is_empty() || rng.random_bool(0.3) {
        true => enabled[rng.random_range(0..enabled.len())],
        false => hot[rng.random_range(0..hot.len())],
    };
    let axis = random_axis(rng);
    let old = *tree.params(i);
    let mut params = old;
    match mutation {
        Mutation::Number => {
            let changed = match rng.random_range(0..8) {
                0 => renumber(old.size[axis], rng).map(|size| params.size[axis] = size).is_some(),
                1 => old.min_size[axis].and_then(|size| renumber(size, rng)).map(|size| params.min_size[axis] = Some(size)).is_some(),
                2 => old.max_size[axis].and_then(|size| renumber(size, rng)).map(|size| params.max_size[axis] = Some(size)).is_some(),
                3 if params.image.is_some() => {
                    params.image = Some(Xy::new(rng.random_range(5.0..50.0f32).round(), rng.random_range(5.0..50.0f32).round()));
                    true
                }
                4 => match &mut params.children_layout {
                    ChildrenLayout::Stack { spacing, .. } => {
                        *spacing = rng.random_range(0.0..10.0f32).round();
                        true
                    }
                    ChildrenLayout::Grid { spacing_x, spacing_y, .. } => {
                        *spacing_x = rng.random_range(0.0..10.0f32).round();
                        *spacing_y = rng.random_range(0.0..10.0f32).round();
                        true
                    }
                    ChildrenLayout::Free => false,
                },
                5 => match &mut params.children_layout {
                    ChildrenLayout::Grid { columns: MainAxisCellSize::Width(width), .. } => {
                        *width = rng.random_range(20.0..60.0f32).round();
                        true
                    }
                    _ => false,
                },
                6 => {
                    params.fill_weight = rng.random_range(0.5..2.0);
                    true
                }
                _ => false,
            };
            if ! changed {
                params.padding[axis][rng.random_range(0..2)] = rng.random_range(0.0..8.0f32).round();
            }
        }
        Mutation::Content => {
            let text_nodes: Vec<NodeId> = enabled.iter().copied().filter(|&i| tree.params(i).text != TextContent::None).collect();
            let i = if text_nodes.is_empty() { i } else { text_nodes[rng.random_range(0..text_nodes.len())] };
            tree.update_node_context(i, Some(rng.random_range(0..40)));
            return;
        }
        Mutation::SizeKind => {
            let mut new = random_size(rng);
            while std::mem::discriminant(&new) == std::mem::discriminant(&old.size[axis]) || (matches!(new, Size::AspectRatio(_)) && matches!(old.size[axis.other()], Size::AspectRatio(_))) {
                new = random_size(rng);
            }
            params.size[axis] = new;
        }
        Mutation::Layout => {
            let mut new = random_children_layout(rng);
            while new == old.children_layout {
                new = random_children_layout(rng);
            }
            params.children_layout = new;
        }
        Mutation::Placement => match rng.random_range(0..5) {
            0 => params.position[axis] = random_pos(rng),
            1 => params.anchor[axis] = random_anchor(rng),
            2 => params.children_origin_flipped[axis] = ! params.children_origin_flipped[axis],
            3 => params.window_overflow[axis] = random_window_overflow(rng),
            _ => match &mut params.children_layout {
                ChildrenLayout::Stack { arrange, .. } => *arrange = random_arrange(rng),
                _ => params.position[axis] = random_pos(rng),
            },
        },
        Mutation::Add => {
            tree.add_node_with_context(i, random_params(rng), rng.random_range(0..80));
            return;
        }
        Mutation::Toggle | Mutation::Remove | Mutation::Move if non_root.is_empty() => {}
        Mutation::Remove => {
            tree.remove(non_root[rng.random_range(0..non_root.len())]);
            return;
        }
        Mutation::Move => {
            let moved = non_root[rng.random_range(0..non_root.len())];
            // The new parent can't be in the subtree of the moved node.
            let parents: Vec<NodeId> = enabled.iter().copied().filter(|&p| std::iter::successors(Some(p), |&p| tree.parent(p)).all(|p| p != moved)).collect();
            let siblings: Vec<NodeId> = non_root.iter().copied().filter(|&s| s != moved && parents.contains(&tree.parent(s).unwrap())).collect();
            if siblings.is_empty() || rng.random_bool(0.5) {
                tree.move_node(moved, parents[rng.random_range(0..parents.len())], None);
            } else {
                let sibling = siblings[rng.random_range(0..siblings.len())];
                tree.move_node(moved, tree.parent(sibling).unwrap(), Some(sibling));
            }
            return;
        }
        Mutation::Toggle => {
            // The parent of a reachable node is enabled, so the whole subtree can be enabled or disabled. The tree applies the change to the descendants.
            let i = non_root[rng.random_range(0..non_root.len())];
            let mut params = *tree.params(i);
            params.disabled = ! params.disabled;
            tree.update_node_params(i, params);
            return;
        }
        Mutation::Window => {
            // A small change, as in a resize by dragging, or a jump.
            let window = tree.window_size();
            tree.set_window_size(match rng.random_bool(0.5) {
                true => Xy::new((window.x + rng.random_range(-20.0..20.0f32)).round().max(100.0), (window.y + rng.random_range(-20.0..20.0f32)).round().max(100.0)),
                false => Xy::new(rng.random_range(300.0..1200.0f32).round(), rng.random_range(300.0..1200.0f32).round()),
            });
            return;
        }
    }
    tree.update_node_params(i, params);
}

/// Runs random changes on random trees, and compares the incremental relayout with a full solve.
#[test]
fn incremental_relayout_matches_full_solve() {
    let mut rng = StdRng::seed_from_u64(0x2545_f491_4f6c_dd1d);
    for &mutation in VALUE_MUTATIONS.iter().chain(&STRUCTURAL_MUTATIONS) {
        for _ in 0..1000 {
            // The trees can have cycles, and then the relayout is a full solve.
            let mut tree = random_tree(&mut rng);
            tree.solve_layout_with_measure(measure);
            mutate(&mut tree, &mut rng, mutation, &[]);
            let mut full = tree.clone();
            full.invalidate();
            full.solve_layout_with_measure(measure);

            let outcome = tree.solve_layout_with_measure(measure);
            assert_same_layout(&tree, &full, &format!("{mutation:?}"));
            if VALUE_MUTATIONS.contains(&mutation) {
                assert_ne!(outcome, IncrementalRelayoutOutcome::FallbackStructuralChange, "{mutation:?} is not structural");
            }
        }
    }
}

/// Runs many frames of incremental relayouts on the same tree, with several changes in each frame, and compares each frame with a full solve. Most changes are on a few hot nodes, as in an animation, and some frames have a structural change.
#[test]
fn incremental_relayout_sequences_match_full_solve() {
    let mut rng = StdRng::seed_from_u64(0x9e37_79b9_7f4a_7c15);
    let mut frames = 0;
    let mut incremental_frames = 0;
    for _ in 0..200 {
        let mut tree = solved_random_tree_without_cycle(&mut rng);
        let nodes = reachable(&tree);
        let hot: Vec<NodeId> = (0..3).map(|_| nodes[rng.random_range(0..nodes.len())]).collect();
        for frame in 0..50 {
            let n_mutations = rng.random_range(1..=4);
            let mut mutations: Vec<Mutation> = (0..n_mutations).map(|_| VALUE_MUTATIONS[rng.random_range(0..VALUE_MUTATIONS.len())]).collect();
            if rng.random_bool(0.05) {
                mutations.push(STRUCTURAL_MUTATIONS[rng.random_range(0..STRUCTURAL_MUTATIONS.len())]);
            }
            for &mutation in &mutations {
                mutate(&mut tree, &mut rng, mutation, &hot);
            }
            let mut full = tree.clone();
            full.invalidate();
            full.solve_layout_with_measure(measure);

            let outcome = tree.solve_layout_with_measure(measure);
            assert_same_layout(&tree, &full, &format!("frame {frame}, {mutations:?}"));
            if mutations.iter().all(|mutation| VALUE_MUTATIONS.contains(mutation)) {
                assert_ne!(outcome, IncrementalRelayoutOutcome::FallbackStructuralChange, "{mutations:?} are not structural");
            }
            frames += 1;
            if matches!(outcome, IncrementalRelayoutOutcome::Success { computed } if computed > 0) {
                incremental_frames += 1;
            }
        }
    }
    // Make sure that the test still runs the incremental solver, and doesn't only compare full solves.
    assert!(incremental_frames * 3 > frames, "Only {incremental_frames} of {frames} frames computed slots incrementally");
}
