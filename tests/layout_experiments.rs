use keru::*;
use keru::node_library::*;
use Size::*;

#[node_key] const A: NodeKey;
#[node_key] const B: NodeKey;
#[node_key] const C: NodeKey;
#[node_key] const D: NodeKey;
#[node_key] const E: NodeKey;

const BOX: Node = DEFAULT;

fn leaf(w: Size) -> Node<'static> { BOX.size(w, Pixels(50.0)) }
fn row() -> Node<'static> { H_STACK.stack_spacing(0.0) }
fn col() -> Node<'static> { V_STACK.stack_spacing(0.0) }

fn width(ui: &Ui, key: NodeKey) -> f32 {
    let r = ui.get_node(key).unwrap().rect();
    r.x[1] - r.x[0]
}

fn case(failures: &mut Vec<String>, name: &str, build: impl Fn(&mut Ui), expected: &[(NodeKey, &str, f32)]) {
    let mut ui = Ui::new_headless(1000, 1000);
    for _ in 0..2 {
        ui.begin_frame();
        build(&mut ui);
        ui.finish_frame();
    }
    let mut line = format!("{name:<44}");
    let mut ok = true;
    for &(key, label, w) in expected {
        // Labels that start with "h_" check the height.
        let got = match label.starts_with("h_") {
            true => { let r = ui.get_node(key).unwrap().rect(); r.y[1] - r.y[0] }
            false => width(&ui, key),
        };
        let good = (got - w).abs() < 1.0;
        ok &= good;
        line += &format!(" {label}={got:.0}{}", if good { String::new() } else { format!(" (expected {w:.0})") });
    }
    println!("{} {line}", if ok { "  " } else { "!!" });
    if ! ok {
        failures.push(line);
    }
}

#[test]
fn fit_fill_experiments() {
    let f = &mut Vec::new();
    println!();

    case(f, "toolbar [100, Fill, 100]", |ui| {
        ui.add(row().key(D)).nest(|| {
            ui.add(leaf(Pixels(100.0)));
            ui.add(leaf(Fill).key(B));
            ui.add(leaf(Pixels(100.0)));
        });
    }, &[(D, "row", 1000.0), (B, "fill", 800.0)]);

    case(f, "toolbar, spacer max 50", |ui| {
        ui.add(row().key(D)).nest(|| {
            ui.add(leaf(Pixels(100.0)));
            ui.add(leaf(Fill).max_size_x(Pixels(50.0)).key(B));
            ui.add(leaf(Pixels(100.0)));
        });
    }, &[(D, "row", 250.0), (B, "fill", 50.0)]);

    case(f, "[100, Fill max 50, Fill]", |ui| {
        ui.add(row().key(D)).nest(|| {
            ui.add(leaf(Pixels(100.0)));
            ui.add(leaf(Fill).max_size_x(Pixels(50.0)).key(B));
            ui.add(leaf(Fill).key(C));
        });
    }, &[(D, "row", 1000.0), (B, "capped", 50.0), (C, "free", 850.0)]);

    case(f, "spacing 10 [100, Fill max 50, 100]", |ui| {
        ui.add(row().stack_spacing(10.0).key(D)).nest(|| {
            ui.add(leaf(Pixels(100.0)));
            ui.add(leaf(Fill).max_size_x(Pixels(50.0)).key(B));
            ui.add(leaf(Pixels(100.0)));
        });
    }, &[(D, "row", 270.0), (B, "fill", 50.0)]);

    case(f, "padding 10 [Fill max 50]", |ui| {
        ui.add(row().padding(10.0).key(D)).nest(|| {
            ui.add(leaf(Fill).max_size_x(Pixels(50.0)).key(B));
        });
    }, &[(D, "row", 70.0), (B, "fill", 50.0)]);

    case(f, "col > row [100, Fill]", |ui| {
        ui.add(col().key(D)).nest(|| {
            ui.add(row().key(C)).nest(|| {
                ui.add(leaf(Pixels(100.0)));
                ui.add(leaf(Fill).key(B));
            });
        });
    }, &[(D, "col", 1000.0), (C, "row", 1000.0), (B, "fill", 900.0)]);

    case(f, "col > [row [100, Fill], 300]", |ui| {
        ui.add(col().key(D)).nest(|| {
            ui.add(row().key(C)).nest(|| {
                ui.add(leaf(Pixels(100.0)));
                ui.add(leaf(Fill).key(B));
            });
            ui.add(leaf(Pixels(300.0)));
        });
    }, &[(D, "col", 300.0), (C, "row", 300.0), (B, "fill", 200.0)]);

    case(f, "row max 400 [100, Fill]", |ui| {
        ui.add(row().max_size_x(Pixels(400.0)).key(D)).nest(|| {
            ui.add(leaf(Pixels(100.0)));
            ui.add(leaf(Fill).key(B));
        });
    }, &[(D, "row", 400.0), (B, "fill", 300.0)]);

    // Without the Fill child, the row would be clamped to 50 by its max.
    case(f, "row max 50 [300, Fill]", |ui| {
        ui.add(row().max_size_x(Pixels(50.0)).key(D)).nest(|| {
            ui.add(leaf(Pixels(300.0)));
            ui.add(leaf(Fill).key(B));
        });
    }, &[(D, "row", 50.0), (B, "fill", 0.0)]);

    case(f, "reference: row max 50 [300]", |ui| {
        ui.add(row().max_size_x(Pixels(50.0)).key(D)).nest(|| {
            ui.add(leaf(Pixels(300.0)));
        });
    }, &[(D, "row", 50.0)]);

    case(f, "row min 400 [Fill max 50]", |ui| {
        ui.add(row().min_size_x(Pixels(400.0)).key(D)).nest(|| {
            ui.add(leaf(Fill).max_size_x(Pixels(50.0)).key(B));
        });
    }, &[(D, "row", 400.0), (B, "fill", 50.0)]);

    case(f, "200px row > row min 50 [300, Fill]", |ui| {
        ui.add(row().size_x(Pixels(200.0))).nest(|| {
            ui.add(row().min_size_x(Pixels(50.0)).key(D)).nest(|| {
                ui.add(leaf(Pixels(300.0)));
                ui.add(leaf(Fill).key(B));
            });
        });
    }, &[(D, "row", 300.0), (B, "fill", 0.0)]);

    case(f, "1000px row > [r1 [100, Fill], r2 [600, Fill]]", |ui| {
        ui.add(row().size_x(Pixels(1000.0))).nest(|| {
            ui.add(row().key(C)).nest(|| {
                ui.add(leaf(Pixels(100.0)));
                ui.add(leaf(Fill).key(A));
            });
            ui.add(row().key(D)).nest(|| {
                ui.add(leaf(Pixels(600.0)));
                ui.add(leaf(Fill).key(B));
            });
        });
    }, &[(C, "r1", 400.0), (A, "fill1", 300.0), (D, "r2", 600.0), (B, "fill2", 0.0)]);

    case(f, "1000px row > [r [100, Fill], Fill]", |ui| {
        ui.add(row().size_x(Pixels(1000.0))).nest(|| {
            ui.add(row().key(C)).nest(|| {
                ui.add(leaf(Pixels(100.0)));
                ui.add(leaf(Fill));
            });
            ui.add(leaf(Fill).key(E));
        });
    }, &[(C, "r", 500.0), (E, "fill", 500.0)]);

    case(f, "1000px row > [r [Fill max 50], Fill]", |ui| {
        ui.add(row().size_x(Pixels(1000.0))).nest(|| {
            ui.add(row().key(C)).nest(|| {
                ui.add(leaf(Fill).max_size_x(Pixels(50.0)));
            });
            ui.add(leaf(Fill).key(E));
        });
    }, &[(C, "r", 50.0), (E, "fill", 950.0)]);

    case(f, "row [Frac 0.5, Fill]", |ui| {
        ui.add(row().key(D)).nest(|| {
            ui.add(leaf(Frac(0.5)).key(A));
            ui.add(leaf(Fill).key(B));
        });
    }, &[(D, "row", 1000.0), (A, "frac", 500.0), (B, "fill", 500.0)]);

    case(f, "row [100, Frac 0.5]", |ui| {
        ui.add(row().key(D)).nest(|| {
            ui.add(leaf(Pixels(100.0)));
            ui.add(leaf(Frac(0.5)).key(A));
        });
    }, &[(D, "row", 200.0), (A, "frac", 100.0)]);

    case(f, "row [Fill maxFit [150]]", |ui| {
        ui.add(row().key(D)).nest(|| {
            ui.add(BOX.size(Fill, Pixels(50.0)).max_size_x(FitContent).key(B)).nest(|| {
                ui.add(leaf(Pixels(150.0)));
            });
        });
    }, &[(D, "row", 150.0), (B, "fill", 150.0)]);

    case(f, "row [row [Fill max 50], 100]", |ui| {
        ui.add(row().key(D)).nest(|| {
            ui.add(row().key(C)).nest(|| {
                ui.add(leaf(Fill).max_size_x(Pixels(50.0)));
            });
            ui.add(leaf(Pixels(100.0)));
        });
    }, &[(D, "outer", 150.0), (C, "inner", 50.0)]);

    case(f, "col [Fill max 200, Fill max 300]", |ui| {
        ui.add(col().key(D)).nest(|| {
            ui.add(leaf(Fill).max_size_x(Pixels(200.0)).key(A));
            ui.add(leaf(Fill).max_size_x(Pixels(300.0)).key(B));
        });
    }, &[(D, "col", 300.0), (A, "a", 200.0), (B, "b", 300.0)]);

    case(f, "row [100, Fill max Frac 0.5]", |ui| {
        ui.add(row().key(D)).nest(|| {
            ui.add(leaf(Pixels(100.0)));
            ui.add(leaf(Fill).max_size_x(Frac(0.5)).key(B));
        });
    }, &[(D, "row", 1000.0), (B, "fill", 500.0)]);

    case(f, "row [Fill min Frac 0.5]", |ui| {
        ui.add(row().key(D)).nest(|| {
            ui.add(leaf(Fill).min_size_x(Frac(0.5)).key(B));
        });
    }, &[(D, "row", 1000.0), (B, "fill", 1000.0)]);

    case(f, "4 nested rows [10, ...] with Fill leaf", |ui| {
        ui.add(row().key(D)).nest(|| {
            ui.add(leaf(Pixels(10.0)));
            ui.add(row()).nest(|| {
                ui.add(leaf(Pixels(10.0)));
                ui.add(row()).nest(|| {
                    ui.add(leaf(Pixels(10.0)));
                    ui.add(row()).nest(|| {
                        ui.add(leaf(Pixels(10.0)));
                        ui.add(leaf(Fill).key(B));
                    });
                });
            });
        });
    }, &[(D, "outer", 1000.0), (B, "leaf", 960.0)]);

    case(f, "col min 600 > row [100, Fill]", |ui| {
        ui.add(col().min_size_x(Pixels(600.0)).key(D)).nest(|| {
            ui.add(row().key(C)).nest(|| {
                ui.add(leaf(Pixels(100.0)));
                ui.add(leaf(Fill));
            });
        });
    }, &[(D, "col", 1000.0), (C, "row", 1000.0)]);

    case(f, "free [50, free min Frac .1 [Fill min 200]]", |ui| {
        ui.add(BOX.size_x(FitContent).key(D)).nest(|| {
            ui.add(leaf(Pixels(50.0)));
            ui.add(BOX.size_x(FitContent).min_size_x(Frac(0.1)).key(C)).nest(|| {
                ui.add(leaf(Fill).min_size_x(Pixels(200.0)));
            });
        });
    }, &[(D, "outer", 200.0), (C, "inner", 200.0)]);

    case(f, "row both axes [Fill x Fill]", |ui| {
        ui.add(row().key(D)).nest(|| {
            ui.add(BOX.size(Fill, Fill).key(B));
        });
    }, &[(D, "row", 1000.0), (B, "fill", 1000.0)]);

    case(f, "row [Fill min Fill]", |ui| {
        ui.add(row().key(D)).nest(|| {
            ui.add(leaf(Pixels(100.0)));
            ui.add(leaf(Pixels(100.0)).min_size_x(Fill).key(B));
        });
    }, &[(D, "row", 200.0), (B, "b", 100.0)]);

    case(f, "row [Fit max Fill [300]]", |ui| {
        ui.add(row().key(D)).nest(|| {
            ui.add(BOX.size(FitContent, Pixels(50.0)).max_size_x(Fill).key(B)).nest(|| {
                ui.add(leaf(Pixels(300.0)));
            });
        });
    }, &[(D, "row", 300.0), (B, "b", 300.0)]);

    case(f, "col [Fit max Fill [300], 100]", |ui| {
        ui.add(col().key(D)).nest(|| {
            ui.add(BOX.size(FitContent, Pixels(50.0)).max_size_x(Fill).key(B)).nest(|| {
                ui.add(leaf(Pixels(300.0)));
            });
            ui.add(leaf(Pixels(100.0)));
        });
    }, &[(D, "col", 300.0), (B, "b", 300.0)]);

    println!();
    assert!(f.is_empty(), "{} cases differ from the expected widths:\n{}", f.len(), f.join("\n"));
}

#[test]
fn fit_fill_experiments_weird() {
    let f = &mut Vec::new();
    println!();

    // Grids are never promoted.
    case(f, "fit grid [Fill, Fill]", |ui| {
        ui.add(GRID.key(D)).nest(|| {
            ui.add(leaf(Fill).key(A));
            ui.add(leaf(Fill));
        });
    }, &[(D, "grid", 16.0), (A, "a", 0.0)]);

    case(f, "fit grid [Fill, 100]", |ui| {
        ui.add(GRID.key(D)).nest(|| {
            ui.add(leaf(Fill).key(A));
            ui.add(leaf(Pixels(100.0)));
        });
    }, &[(A, "a", 100.0)]);

    case(f, "col on y [100, Fill]", |ui| {
        ui.add(col().key(D)).nest(|| {
            ui.add(BOX.size(Pixels(50.0), Pixels(100.0)));
            ui.add(BOX.size(Pixels(50.0), Fill).key(B));
        });
    }, &[(D, "h_col", 1000.0), (B, "h_fill", 900.0)]);

    case(f, "row [Fill, h AspectRatio 2]", |ui| {
        ui.add(row().key(D)).nest(|| {
            ui.add(BOX.size(Fill, AspectRatio(2.0)).key(B));
        });
    }, &[(D, "row", 1000.0), (B, "fill", 1000.0), (B, "h_fill", 500.0)]);

    case(f, "row w AspectRatio 2 [h Fill]", |ui| {
        ui.add(row().size_y(Pixels(100.0)).key(D)).nest(|| {
            ui.add(BOX.size(AspectRatio(2.0), Fill).key(B));
        });
    }, &[(D, "row", 200.0), (B, "b", 200.0)]);

    // The row height is FitContent and the child height is Fill: the row becomes Fill on y too.
    case(f, "row [w AspectRatio 1, h Fill]", |ui| {
        ui.add(row().key(D)).nest(|| {
            ui.add(BOX.size(AspectRatio(1.0), Fill).key(B));
        });
    }, &[(D, "h_row", 1000.0), (B, "b", 1000.0)]);

    case(f, "row [free-placement Fill]", |ui| {
        ui.add(row().key(D)).nest(|| {
            ui.add(leaf(Fill).free_placement(true).key(B));
        });
    }, &[(D, "row", 0.0), (B, "fill", 0.0)]);

    case(f, "row [100, free-placement Fill]", |ui| {
        ui.add(row().key(D)).nest(|| {
            ui.add(leaf(Pixels(100.0)));
            ui.add(leaf(Fill).free_placement(true).key(B));
        });
    }, &[(D, "row", 100.0), (B, "fill", 100.0)]);

    case(f, "row [Fill min 300 max 50]", |ui| {
        ui.add(row().key(D)).nest(|| {
            ui.add(leaf(Fill).min_size_x(Pixels(300.0)).max_size_x(Pixels(50.0)).key(B));
        });
    }, &[(D, "row", 300.0), (B, "fill", 300.0)]);

    case(f, "30px row > row [Fill max 50]", |ui| {
        ui.add(row().size_x(Pixels(30.0))).nest(|| {
            ui.add(row().key(D)).nest(|| {
                ui.add(leaf(Fill).max_size_x(Pixels(50.0)).key(B));
            });
        });
    }, &[(D, "row", 30.0), (B, "fill", 30.0)]);

    case(f, "row minFit [Fill min 200]", |ui| {
        ui.add(row().min_size_x(FitContent).key(D)).nest(|| {
            ui.add(leaf(Fill).min_size_x(Pixels(200.0)).key(B));
        });
    }, &[(D, "row", 1000.0), (B, "fill", 1000.0)]);

    case(f, "row maxFit [100, Fill]", |ui| {
        ui.add(row().max_size_x(FitContent).key(D)).nest(|| {
            ui.add(leaf(Pixels(100.0)));
            ui.add(leaf(Fill).key(B));
        });
    }, &[(D, "row", 1000.0), (B, "fill", 900.0)]);

    case(f, "row max Frac .5 [100, Fill]", |ui| {
        ui.add(row().max_size_x(Frac(0.5)).key(D)).nest(|| {
            ui.add(leaf(Pixels(100.0)));
            ui.add(leaf(Fill).key(B));
        });
    }, &[(D, "row", 500.0), (B, "fill", 400.0)]);

    // Should match the reference below: the column can't see the Frac max, and the row gets clamped to 10% of the column.
    case(f, "col [row max Frac .1 [300, Fill], 100]", |ui| {
        ui.add(col().key(D)).nest(|| {
            ui.add(row().max_size_x(Frac(0.1)).key(C)).nest(|| {
                ui.add(leaf(Pixels(300.0)));
                ui.add(leaf(Fill));
            });
            ui.add(leaf(Pixels(100.0)));
        });
    }, &[(D, "col", 300.0), (C, "row", 30.0)]);

    case(f, "reference: col [row max Frac .1 [300], 100]", |ui| {
        ui.add(col().key(D)).nest(|| {
            ui.add(row().max_size_x(Frac(0.1)).key(C)).nest(|| {
                ui.add(leaf(Pixels(300.0)));
            });
            ui.add(leaf(Pixels(100.0)));
        });
    }, &[(D, "col", 300.0), (C, "row", 30.0)]);

    case(f, "row [Fill, Fill] in a 1000px col", |ui| {
        ui.add(col().size_x(Pixels(1000.0))).nest(|| {
            ui.add(row().key(D)).nest(|| {
                ui.add(leaf(Fill).key(A));
                ui.add(leaf(Fill).key(B));
            });
        });
    }, &[(D, "row", 1000.0), (A, "a", 500.0), (B, "b", 500.0)]);

    case(f, "row [Fill [text]]", |ui| {
        ui.add(row().key(D)).nest(|| {
            ui.add(BOX.size(Fill, FitContent).key(B)).nest(|| {
                ui.add(LABEL.text("hello"));
            });
        });
    }, &[(D, "row", 1000.0), (B, "fill", 1000.0)]);

    case(f, "fit panel with text + [Fill]", |ui| {
        ui.add(PANEL.size_x(FitContent).padding(0.0).text("hello world, this is text").key(D)).nest(|| {
            ui.add(leaf(Fill).key(B));
        });
    }, &[(B, "fill", 253.0)]);

    println!();
    assert!(f.is_empty(), "{} cases differ from the expected widths:\n{}", f.len(), f.join("\n"));
}
