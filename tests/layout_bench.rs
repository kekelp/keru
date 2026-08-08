use keru::*;
use keru::node_library::*;
use Size::*;
use std::time::Instant;

fn bench(name: &str, node_count: usize, iters: usize, build: impl Fn(&mut Ui)) {
    let mut best = f64::INFINITY;
    for _ in 0..iters {
        let mut ui = Ui::new_headless(1000, 1000);
        ui.begin_frame();
        build(&mut ui);
        let t = Instant::now();
        ui.finish_frame();
        let ns = t.elapsed().as_nanos() as f64;
        best = best.min(ns);
    }
    let per = best / node_count as f64;
    println!("{name:<32} nodes={node_count:>7}  layout={:>9.3} us  {per:>8.2} ns/node", best / 1000.0);
}

#[test]
#[ignore]
fn flex_scaling() {
    println!();
    for n in [1000usize, 2000, 4000, 8000, 16000, 32000] {
        bench(&format!("flex n={n}"), n + 1, 8, |ui| {
            ui.add(H_STACK.size(Pixels(10000.0), Pixels(100.0)).stack_spacing(0.0)).nest(|| {
                for _ in 0..n {
                    ui.add(DEFAULT.size(Fill, Fill));
                }
            });
        });
    }
    println!();
}

#[test]
#[ignore]
fn layout_benchmarks() {
    println!();

    bench("wide_no_wrap_simple_few", 1001, 50, |ui| {
        ui.add(H_STACK.size(Pixels(100.0), Pixels(100.0)).stack_spacing(0.0)).nest(|| {
            for _ in 0..1000 {
                ui.add(DEFAULT.size(Pixels(10.0), Pixels(10.0)));
            }
        });
    });

    bench("wide_no_wrap_simple_many", 100_001, 5, |ui| {
        ui.add(H_STACK.size(Pixels(100.0), Pixels(100.0)).stack_spacing(0.0)).nest(|| {
            for _ in 0..100_000 {
                ui.add(DEFAULT.size(Pixels(10.0), Pixels(10.0)));
            }
        });
    });

    bench("flex_expand_equal_weights", 15_001, 10, |ui| {
        ui.add(H_STACK.size(Pixels(10000.0), Pixels(100.0)).stack_spacing(0.0)).nest(|| {
            for _ in 0..15_000 {
                ui.add(DEFAULT.size(Fill, Fill));
            }
        });
    });

    bench("nested_vertical_stack", 10_001, 20, |ui| {
        ui.add(V_STACK.size_x(Pixels(200.0)).size_y(FitContent).padding(10.0).stack_spacing(5.0)).nest(|| {
            for _ in 0..10_000 {
                ui.add(DEFAULT.size(Fill, Pixels(1.0)));
            }
        });
    });

    bench("percentage_and_ratio", 10_001, 20, |ui| {
        ui.add(V_STACK.size(Pixels(1000.0), Pixels(1000.0)).stack_spacing(0.0)).nest(|| {
            for _ in 0..10_000 {
                ui.add(DEFAULT.size_x(Frac(0.5)).size_y(AspectRatio(2.0)));
            }
        });
    });

    bench("expand_with_max_constraint", 3001, 30, |ui| {
        ui.add(V_STACK.size_x(Pixels(100.0)).size_y(FitContent).stack_spacing(0.0)).nest(|| {
            for _ in 0..1000 {
                ui.add(H_STACK.size_x(Pixels(100.0)).size_y(FitContent).stack_spacing(0.0)).nest(|| {
                    ui.add(DEFAULT.size(Fill, Pixels(10.0)));
                    ui.add(DEFAULT.size(Fill, Pixels(10.0)).max_size_x(Pixels(40.0)));
                });
            }
        });
    });

    bench("fit_nesting", 101_111, 5, |ui| {
        ui.add(V_STACK.size_x(Pixels(1000.0)).size_y(FitContent).padding(5.0).stack_spacing(0.0)).nest(|| {
            for _ in 0..10 {
                ui.add(H_STACK.size_x(Fill).size_y(FitContent).stack_spacing(0.0)).nest(|| {
                    for _ in 0..10 {
                        ui.add(V_STACK.size_x(Fill).size_y(FitContent).stack_spacing(0.0)).nest(|| {
                            for _ in 0..10 {
                                ui.add(H_STACK.size_x(Fill).size_y(FitContent).stack_spacing(0.0)).nest(|| {
                                    for _ in 0..100 {
                                        ui.add(DEFAULT.size(Fill, Pixels(10.0)));
                                    }
                                });
                            }
                        });
                    }
                });
            }
        });
    });

    println!();
}
