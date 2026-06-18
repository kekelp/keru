use keru::basic_window_loop::basic_env_logger_init;
use keru::*;
use keru::node_library::*;

#[node_key] const CONTENT: NodeKey;
#[node_key] const PLOT: NodeKey;
#[node_key] const LINE_CONTAINER: NodeKey;
#[node_key] const GRADIENT_SOURCE: NodeKey;
#[node_key] const POINT: NodeKey;

const LABEL_GAP: f32 = 8.0;

const YEAR_MIN: f32 = 1880.0;
const YEAR_MAX: f32 = 2017.0;
const VAL_MIN: f32 = -0.8;
const VAL_MAX: f32 = 1.4;

// In this case, it's actually a bit silly to use a shared gradient instead of coloring each point based on its coordinates.
// But this is just an example, so we choose aesthetics over scientific clarity.
const GRAD: LinearGradient = LinearGradient {
    color_start: Color::from_hex_str("#fc5367").with_alpha(0.8), // top
    color_end: Color::from_hex_str("#52cff2").with_alpha(0.8),   // bottom
    angle_deg: 90.0,                             // top -> bottom
};

fn data_x(year: f32) -> f32 {
    (year - YEAR_MIN) / (YEAR_MAX - YEAR_MIN)
}

fn data_y(val: f32) -> f32 {
    (val - VAL_MIN) / (VAL_MAX - VAL_MIN)
}

struct Point {
    year: f32,
    value: f32,
}

fn load_points() -> Vec<Point> {
    let csv = include_str!("assets/temp_anomaly_dataset.csv");
    csv.lines()
        .skip(1) // header
        .filter_map(|line| {
            let mut cols = line.split(',');
            let date = cols.next()?;
            let value: f32 = cols.next()?.trim().parse().ok()?;
            // Date is YYYY-MM-DD; turn it into a fractional year.
            let mut parts = date.split('-');
            let y: f32 = parts.next()?.parse().ok()?;
            let m: f32 = parts.next()?.parse().ok()?;
            Some(Point { year: y + (m - 1.0) / 12.0, value })
        })
        .collect()
}

fn moving_average(points: &[Point], window: usize) -> Vec<[f32; 2]> {
    let half = (window / 2) as isize;
    let n = points.len() as isize;
    (0..points.len())
        .map(|i| {
            let i = i as isize;
            let lo = (i - half).max(0);
            let hi = (i + half).min(n - 1);
            let mut sum = 0.0;
            for j in lo..=hi {
                sum += points[j as usize].value;
            }
            let avg = sum / (hi - lo + 1) as f32;
            [points[i as usize].year, avg]
        })
        .collect()
}

struct Data {
    points: Vec<Point>,
    average: Vec<[f32; 2]>,
}

fn load_data() -> Data {
    let points = load_points();
    let average = moving_average(&points, 12);
    Data { points, average }
}

#[derive(Default)]
struct State {
    data: Option<Data>,
    transform: TransformViewState,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    let State { data, transform } = state;
    let data = data.get_or_insert_with(load_data);
    let points: &[Point] = &data.points;
    ui.add(PANEL.size_symm(Size::Fill).color(Color::WHITE));

    let title = TEXT
        .static_text("Temperature Anomaly")
        .text_size(16.0)
        .text_color(Color::BLACK)
        .z_index(2.0)
        .anchor(Anchor::Start, Anchor::Start)
        .position(Pos::Start, Pos::Start);

    let frame = CONTAINER
        .size_symm(Size::Fill)
        .padding_x(55.0)
        .padding_y(42.0)
        .color(Color::TRANSPARENT);

    let content = CONTAINER
        .key(CONTENT)
        .size_symm(Size::Fill)
        .padding(0.0)
        .color(Color::TRANSPARENT)
        .clip_children(true);

    let plot = CONTAINER
        .key(PLOT)
        .size_symm(Size::Fill)
        .pos_origin(HorizontalOrigin::Left, VerticalOrigin::Bottom)
        .padding(0.0)
        .color(Color::TRANSPARENT);

    let gradient = PANEL
        .key(GRADIENT_SOURCE)
        .invisible()
        .linear_gradient(GRAD)
        .size_symm(Size::Fill);

    let y_label = TEXT
        .text_size(15.0)
        .text_color(Color::rgba_u8(60, 60, 60, 255))
        .anchor(Anchor::End, Anchor::Center);

    let x_label = TEXT
        .text_size(15.0)
        .text_color(Color::rgba_u8(60, 60, 60, 255))
        .anchor(Anchor::Center, Anchor::Start);

    let point = DEFAULT
        .shape(Shape::Circle)
        .color(Color::TRANSPARENT)
        .stroke(1.4)
        .stroke_fill(ColorFill2::SharedGradient(GRADIENT_SOURCE))
        .size_symm(Size::Pixels(8.0))
        .anchor_symm(Anchor::Center)
        .sense_hover_enter_or_exit(true)
        .animate_position(true)
        .absorbs_clicks(true);

    let hovered_point = point
        .size_symm(Size::Pixels(12.0))
        .stroke_fill(ColorFill2::Color(Color::BLACK))
        .z_index(1.0)
        .shared_gradient(GRADIENT_SOURCE);

    let line_container = CONTAINER
        .key(LINE_CONTAINER)
        .size_symm(Size::Fill)
        .padding(0.0)
        .color(Color::TRANSPARENT)
        .absorbs_clicks(false);

    let readout = TEXT.text_size(18.0)
        .text_color(Color::BLACK)
        .z_index(2.0)
        .anchor(Anchor::End, Anchor::Start)
        .position(Pos::End, Pos::Start);

    const Y_TICKS: &[(f32, &str)] = &[
        (-0.6, "-0.6"), (-0.4, "-0.4"), (-0.2, "-0.2"), (0.0, "0.0"),
        (0.2, "0.2"), (0.4, "0.4"), (0.6, "0.6"), (0.8, "0.8"),
        (1.0, "1.0"), (1.2, "1.2"),
    ];

    const X_TICKS: &[(f32, &str)] = &[
        (1880.0, "1880"), (1900.0, "1900"), (1920.0, "1920"),
        (1940.0, "1940"), (1960.0, "1960"), (1980.0, "1980"), (2000.0, "2000"),
    ];

    let mut hovered = None;
    ui.add(frame).nest(|| {
        ui.add(content).nest(|| {
        ui.add_component(TransformView::new(transform)).nest(|| {
            ui.add(plot).nest(|| {
                ui.add(gradient);

                // The transform scales everything inside it, so divide sizes and
                // stroke widths by the zoom to keep them constant on screen.
                let s = transform.scale.max(1e-3);
                let point = point.size_symm(Size::Pixels(8.0 / s)).stroke(1.4 / s);
                let hovered_point = hovered_point.size_symm(Size::Pixels(12.0 / s)).stroke(1.4 / s);

                for (i, p) in points.iter().enumerate() {
                    // We use real nodes for the points so that we can easily make them interactable. It will still run at 165Hz effortlessly.
                    // For the non-interactable line segments though we can use canvas drawing later.
                    let key = POINT.sibling(i);
                    let is_hovered = ui.is_hovered(key);
                    if is_hovered {
                        hovered = Some(i);
                    }
                    let base = if is_hovered { hovered_point } else { point };
                    let dot = base
                        .key(key)
                        .position(Pos::Frac(data_x(p.year)), Pos::Frac(data_y(p.value)));

                    ui.add(dot);
                }

                // If we do canvas_drawing directly on the container, the line will end up behind the point nodes.
                // So we have to add a separate node for it.
                ui.add(line_container);
            });
        });
        });
    });

    // We have to do a bunch of manual math to keep the tick labels consistent with the transform view.
    // Maybe in the future a more advanced Component meant specifically for zoomable plots will do this automatically.
    let rect = ui.get_node(CONTENT).map(|n| n.rect()).unwrap_or_default();
    let center = rect.center();
    let pan = Xy::new(transform.pan_x, transform.pan_y);
    let zoom = Xy::new_symm(transform.scale);
    let to_screen = |fx: f32, fy: f32| {
        let in_rect = Xy::new(rect.x[0], rect.y[0]) + rect.size() * Xy::new(fx, fy);
        center + (in_rect - center) * zoom + pan
    };

    for (v, text) in Y_TICKS {
        let pos = to_screen(0.0, 1.0 - data_y(*v));
        if (rect.y[0]..=rect.y[1]).contains(&pos.y) {
            ui.add(y_label.static_text(text)
                .position(Pos::Pixels(rect.x[0] - LABEL_GAP), Pos::Pixels(pos.y)));
        }
    }
    for (year, text) in X_TICKS {
        let pos = to_screen(data_x(*year), 0.0);
        if (rect.x[0]..=rect.x[1]).contains(&pos.x) {
            ui.add(x_label.static_text(text)
                .position(Pos::Pixels(pos.x), Pos::Pixels(rect.y[1] + LABEL_GAP)));
        }
    }

    ui.add(title);

    if let Some(i) = hovered {
        let p = &points[i];
        const MONTHS: [&str; 12] = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun",
            "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        let year = p.year.floor();
        let month = MONTHS[(((p.year - year) * 12.0).round() as usize).min(11)];
        with_arena(|a| {
            let text = bumpalo::format!(in a, "{} {}   {:+.2} °C", month, year as i32, p.value);
            ui.add(readout.text(text.as_str()));
        });
    }

    let size = ui.get_node(LINE_CONTAINER).map(|n| n.inner_size()).unwrap_or(Xy::new(0.0, 0.0));
    let (w, h) = (size.x, size.y);
    let s = transform.scale.max(1e-3);

    ui.get_node_mut(LINE_CONTAINER).unwrap().canvas_drawing(|canvas| {
        use keru::{Segment, CanvasColorFill};

        let to_canvas = |year: f32, val: f32| -> [f32; 2] {
            [data_x(year) * w, (1.0 - data_y(val)) * h]
        };

        let line = |canvas: &mut Canvas, a: [f32; 2], b: [f32; 2], thickness: f32, color: Color| {
            canvas.draw_segment(Segment {
                start: a,
                end: b,
                thickness: thickness / s,
                stroke_thickness: 0.0,
                fill: CanvasColorFill::Color(color),
                dash_length: None,
                dash_offset: 0.0,
                blur: 0.0,
                texture: None,
                texture_options: None,
            });
        };

        line(canvas,
            to_canvas(YEAR_MIN, 0.0),
            to_canvas(YEAR_MAX, 0.0),
            1.0, Color::rgba_u8(120, 120, 120, 255));

        for win in data.average.windows(2) {
            line(canvas,
                to_canvas(win[0][0], win[0][1]),
                to_canvas(win[1][0], win[1][1]),
                2.0, Color::rgba_u8(40, 40, 40, 255));
        }
    });
}

fn main() {
    basic_env_logger_init();
    let state = State::default();
    example_window_loop::run_example_loop(state, update_ui);
}
