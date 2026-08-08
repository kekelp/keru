// Some basic ai-generated layout tests. Not exhaustive
use keru::*;
use keru::node_library::*;
use Size::*;

const W: u32 = 1000;
const H: u32 = 1000;

#[node_key] const A: NodeKey;
#[node_key] const B: NodeKey;
#[node_key] const C: NodeKey;
#[node_key] const D: NodeKey;

fn width(ui: &Ui, key: NodeKey) -> f32 {
    let r = ui.get_node(key).unwrap().rect();
    r.x[1] - r.x[0]
}
fn height(ui: &Ui, key: NodeKey) -> f32 {
    let r = ui.get_node(key).unwrap().rect();
    r.y[1] - r.y[0]
}

fn run(build: impl Fn(&mut Ui)) -> Ui {
    let mut ui = Ui::new_headless(W, H);
    for _ in 0..2 {
        ui.begin_frame();
        build(&mut ui);
        ui.finish_frame();
    }
    ui
}

fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() < 1.0
}

fn x0(ui: &Ui, key: NodeKey) -> f32 { ui.get_node(key).unwrap().rect().x[0] }
fn y0(ui: &Ui, key: NodeKey) -> f32 { ui.get_node(key).unwrap().rect().y[0] }

#[allow(unused_macros)]
macro_rules! dump {
    ($ui:expr, $($k:expr),+) => {{
        $( println!("  {:>2}: pos=({:6.1},{:6.1}) size=({:6.1} x {:6.1})",
            stringify!($k), x0(&$ui,$k), y0(&$ui,$k), width(&$ui,$k), height(&$ui,$k)); )+
    }};
}

// Asserts a node's size (width, height). Uses `approx` (1px tolerance) to absorb text/subpixel rounding.
fn assert_size(ui: &Ui, key: NodeKey, w: f32, h: f32) {
    let (aw, ah) = (width(ui, key), height(ui, key));
    assert!(approx(aw, w) && approx(ah, h), "size expected ({w} x {h}), got ({aw} x {ah})");
}

// Asserts a node's top-left position.
fn assert_pos(ui: &Ui, key: NodeKey, x: f32, y: f32) {
    let (ax, ay) = (x0(ui, key), y0(ui, key));
    assert!(approx(ax, x) && approx(ay, y), "pos expected ({x},{y}), got ({ax},{ay})");
}

// Asserts both position and size.
fn assert_rect(ui: &Ui, key: NodeKey, x: f32, y: f32, w: f32, h: f32) {
    assert_pos(ui, key, x, y);
    assert_size(ui, key, w, h);
}

// A minimal building block: Free layout, Frac(1.0) size, no padding, no default max_size, centered in its parent. Good for isolating a single Size behavior.
const BOX: Node = DEFAULT;

// --- Single node, no parent influence (placed in the 1000x1000 root, centered) ---

#[test]
fn fixed_pixels() {
    // A Pixels size is taken literally, and the node is centered in the root.
    let ui = run(|ui| { ui.add(BOX.key(A).size(Pixels(200.0), Pixels(100.0))); });
    assert_rect(&ui, A, 400.0, 450.0, 200.0, 100.0);
}

#[test]
fn fill_fills_root() {
    // Fill on both axes takes the whole root.
    let ui = run(|ui| { ui.add(BOX.key(A).size(Fill, Fill)); });
    assert_rect(&ui, A, 0.0, 0.0, 1000.0, 1000.0);
}

#[test]
fn frac_of_root() {
    // Frac is a fraction of the parent's inner size (the root, 1000x1000).
    let ui = run(|ui| { ui.add(BOX.key(A).size(Frac(0.5), Frac(0.25))); });
    assert_rect(&ui, A, 250.0, 375.0, 500.0, 250.0);
}

#[test]
fn min_size_clamps_up() {
    // A min_size larger than the size raises the node to the min.
    let ui = run(|ui| { ui.add(BOX.key(A).size_symm(Pixels(100.0)).min_size_symm(Pixels(300.0))); });
    assert_rect(&ui, A, 350.0, 350.0, 300.0, 300.0);
}

#[test]
fn max_size_clamps_down() {
    // A max_size smaller than the size lowers the node to the max.
    let ui = run(|ui| { ui.add(BOX.key(A).size_symm(Pixels(800.0)).max_size_symm(Pixels(400.0))); });
    assert_rect(&ui, A, 300.0, 300.0, 400.0, 400.0);
}

#[test]
fn min_size_wins_over_max_size() {
    // When min > max, the min bound wins (clamped last), so 400, not 300.
    let ui = run(|ui| { ui.add(BOX.key(A).size_symm(Pixels(200.0)).min_size_symm(Pixels(400.0)).max_size_symm(Pixels(300.0))); });
    assert_size(&ui, A, 400.0, 400.0);
}

#[test]
fn aspect_ratio_x_follows_y() {
    // AspectRatio(2.0) on x, with a 1:1 window, makes width = 2 * height.
    let ui = run(|ui| { ui.add(BOX.key(A).size_y(Pixels(200.0)).size_x(AspectRatio(2.0))); });
    assert_rect(&ui, A, 300.0, 400.0, 400.0, 200.0);
}

#[test]
fn aspect_ratio_y_follows_x() {
    // AspectRatio(3.0) on y makes height = width / 3.
    let ui = run(|ui| { ui.add(BOX.key(A).size_x(Pixels(300.0)).size_y(AspectRatio(3.0))); });
    assert_rect(&ui, A, 350.0, 450.0, 300.0, 100.0);
}

// --- Stacks ---

#[test]
fn vstack_fits_fixed_children() {
    // A FitContent V_STACK is as tall as its children plus the 8px spacing, and as wide as the widest child. Children are centered on the cross axis.
    let ui = run(|ui| {
        ui.add(V_STACK.key(C)).nest(|| {
            ui.add(BOX.key(A).size(Pixels(200.0), Pixels(100.0)));
            ui.add(BOX.key(B).size(Pixels(150.0), Pixels(50.0)));
        });
    });
    assert_rect(&ui, C, 400.0, 421.0, 200.0, 158.0);
    assert_rect(&ui, A, 400.0, 421.0, 200.0, 100.0);
    assert_rect(&ui, B, 425.0, 529.0, 150.0, 50.0);
}

#[test]
fn hstack_fits_fixed_children() {
    // A FitContent H_STACK is as wide as its children plus spacing, as tall as the tallest.
    let ui = run(|ui| {
        ui.add(H_STACK.key(C)).nest(|| {
            ui.add(BOX.key(A).size(Pixels(200.0), Pixels(100.0)));
            ui.add(BOX.key(B).size(Pixels(150.0), Pixels(50.0)));
        });
    });
    assert_rect(&ui, C, 321.0, 450.0, 358.0, 100.0);
    assert_rect(&ui, A, 321.0, 450.0, 200.0, 100.0);
    assert_rect(&ui, B, 529.0, 475.0, 150.0, 50.0);
}

#[test]
fn two_fill_children_split_stack() {
    // Two Fill children share the stack's main axis evenly, minus the spacing: (600 - 8) / 2 = 296 each.
    let ui = run(|ui| {
        ui.add(V_STACK.key(C).size(Pixels(300.0), Pixels(600.0))).nest(|| {
            ui.add(BOX.key(A).size(Fill, Fill));
            ui.add(BOX.key(B).size(Fill, Fill));
        });
    });
    assert_rect(&ui, C, 350.0, 200.0, 300.0, 600.0);
    assert_rect(&ui, A, 350.0, 200.0, 300.0, 296.0);
    assert_rect(&ui, B, 350.0, 504.0, 300.0, 296.0);
}

#[test]
fn frac_children_on_stack_main_axis() {
    // Frac on the stack main axis is a fraction of the available (post-spacing) main size, 592: 0.25 -> 148, 0.5 -> 296.
    let ui = run(|ui| {
        ui.add(V_STACK.key(C).size(Pixels(300.0), Pixels(600.0))).nest(|| {
            ui.add(BOX.key(A).size(Fill, Frac(0.25)));
            ui.add(BOX.key(B).size(Fill, Frac(0.5)));
        });
    });
    assert_size(&ui, A, 300.0, 148.0);
    assert_size(&ui, B, 300.0, 296.0);
}

#[test]
fn fill_cross_axis_matches_stack_width() {
    // A Fill on the cross axis grows to the stack's cross size, which itself fits the fixed 400px width.
    let ui = run(|ui| {
        ui.add(V_STACK.key(C).size_x(Pixels(400.0))).nest(|| {
            ui.add(BOX.key(A).size(Fill, Pixels(60.0)));
            ui.add(BOX.key(B).size(Pixels(120.0), Pixels(40.0)));
        });
    });
    assert_rect(&ui, C, 300.0, 446.0, 400.0, 108.0);
    assert_rect(&ui, A, 300.0, 446.0, 400.0, 60.0);
    assert_rect(&ui, B, 440.0, 514.0, 120.0, 40.0);
}

// --- Parent / child sizing ---

#[test]
fn fill_child_fills_fixed_parent() {
    // A Fill child of a fixed (padding-less) parent takes the whole parent.
    let ui = run(|ui| {
        ui.add(BOX.key(C).size_symm(Pixels(400.0))).nest(|| {
            ui.add(BOX.key(A).size(Fill, Fill));
        });
    });
    assert_rect(&ui, C, 300.0, 300.0, 400.0, 400.0);
    assert_rect(&ui, A, 300.0, 300.0, 400.0, 400.0);
}

#[test]
fn frac_child_of_fixed_parent() {
    // A Frac child is a fraction of the parent's inner size (400): 0.5 -> 200, 0.25 -> 100, centered.
    let ui = run(|ui| {
        ui.add(BOX.key(C).size_symm(Pixels(400.0))).nest(|| {
            ui.add(BOX.key(A).size(Frac(0.5), Frac(0.25)));
        });
    });
    assert_rect(&ui, A, 400.0, 450.0, 200.0, 100.0);
}

#[test]
fn fitcontent_parent_wraps_child_with_padding() {
    // A FitContent CONTAINER (10px padding) hugs its child: 120+20 by 80+20, and the child sits inset by the padding.
    let ui = run(|ui| {
        ui.add(CONTAINER.key(C).size_symm(FitContent)).nest(|| {
            ui.add(BOX.key(A).size(Pixels(120.0), Pixels(80.0)));
        });
    });
    assert_rect(&ui, C, 430.0, 450.0, 140.0, 100.0);
    assert_rect(&ui, A, 440.0, 460.0, 120.0, 80.0);
}

#[test]
fn max_size_fill_caps_fixed_child_at_share() {
    // max_size_x(Fill) on an otherwise fixed child is a cap at the parent's available space; since 100 < 500 it stays 100.
    let ui = run(|ui| {
        ui.add(BOX.key(C).size_symm(Pixels(500.0))).nest(|| {
            ui.add(BOX.key(A).size(Pixels(100.0), Pixels(100.0)).max_size_x(Fill));
        });
    });
    assert_rect(&ui, A, 450.0, 450.0, 100.0, 100.0);
}

// --- Cycles ---

// A genuine cross-axis constraint cycle, built from two AspectRatio nodes:
//   N (C): x = AspectRatio (width follows height), y = FitContent (height fits children).
//   The FitContent-only cycle would be broken by the "fill punches through FitContent"
//   collapse rule, so N is given a hard sibling (A, fixed Pixels) to keep it a real
//   FitContent parent, and the aspect child (B) to close the loop:
//   B: x = Fill (fills N), y = AspectRatio (height follows width).
// This closes: B.y -aspect-> B.x -fill-> N.x -aspect-> N.y -fit-> B.y.
// AspectRatio children are not excluded from the FitContent sum (only Fill/Frac are),
// so N.y really does wait on B.y, and the loop is real and unsolvable. The point of the
// test is that the solver still terminates and produces finite sizes via the deferred
// fallback, instead of hanging or emitting NaN/inf.
#[test]
fn aspect_ratio_cycle_terminates_with_finite_sizes() {
    let ui = run(|ui| {
        ui.add(CONTAINER.key(D).size_x(AspectRatio(1.0)).size_y(FitContent)).nest(|| {
            ui.add(BOX.key(A).size(Pixels(50.0), Pixels(50.0)));
            ui.add(BOX.key(B).size_x(Fill).size_y(AspectRatio(1.0)));
        });
    });
    for key in [D, A, B] {
        let (w, h) = (width(&ui, key), height(&ui, key));
        assert!(w.is_finite() && h.is_finite(), "expected finite size, got ({w} x {h})");
    }
}
