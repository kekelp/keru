#![windows_subsystem = "windows"]

use keru::Size::*;
use keru::example_window_loop::*;
use keru::*;
use keru_draw::FontWeight;
use winit::keyboard::Key;

#[derive(Default)]
struct State {
    tabs: Vec<Tab>,
    current_tab: usize,

    f32_value: f32,
    large_bold_style: Option<StyleHandle>,
}

const INTRO_TAB: Tab = Tab("Intro");
const TEXT_TAB: Tab = Tab("Text");
const GRAPHICS_TAB: Tab = Tab("Graphics");
const NODES_TAB: Tab = Tab("Nodes");

const CHINESE_TEXT: &str = "此后，人民文学出版社和齐鲁书社的做法被诸多出版社效仿，可见文化部出版局1985年的一纸批文并没有打消各地出版社出版此书的念头。所以，1988年新闻出版署发出了《关于整理出版〈金瓶梅〉及其研究资料的通知》。《通知》首先说明《金瓶梅》及其研究资料的需求“日益增大”，“先后有十余家出版社向我署提出报告，分别要求出版《金瓶梅》的各种版本及改编本，包括图录、连环画及影视文学剧本等”，但话锋一转，明确提出“《金瓶梅》一书虽在文学史上占有重要地位，但该书存在大量自然主义的秽亵描写，不宜广泛印行";

const CYRILLIC_TEXT: &str = "Мунди деленит молестиае усу ад, пертинах глориатур диссентиас ет нец. Ессент иудицабит маиестатис яуи ад, про ут дицо лорем легере. Вис те цоммодо сцрипта цорпора, тритани интеллегат аргументум цу еум, меи те яуем феугаит. При дисцере интеллегат ат, аеяуе афферт фуиссет ех вих. Цу хас интегре тхеопхрастус. Диам волуптатибус про еа.

Вис цу сцаевола мнесарчум, ин усу волутпат инцоррупте. При ет стет инвидунт форенсибус. Цонсететур волуптатум омиттантур яуи ет, ут доминг промпта доценди сед. Цетеро пробатус ехпетенда сеа ин, диам витае доминг ад вим.
";

const JAPANESE_TEXT: &str = "ヘッケはこれらのL-函数が全複素平面へ有理型接続を持ち、指標が自明であるときには s = 1 でオーダー 1 である極を持ち、それ以外では解析的であることを証明した。原始ヘッケ指標（原始ディリクレ指標に同じ方法である modulus に相対的に定義された）に対し、ヘッケは、これらのL-函数が指標の L-函数の函数等式を満たし、L-函数の複素共役指標であることを示した。 主イデアル上の座と、無限での座を含む全ての例外有限集合の上で 1 である単円の上への写像を取ることで、イデール類群の指標 ψ を考える。すると、ψ はイデアル群 IS の指標 χ を生成し、イデアル群は S 上に入らない素イデアル上の自由アーベル群となる。";

trait UiExt {
    fn intro_tab(&mut self, state: &mut State);
    fn text_tab(&mut self);
    fn graphics_tab(&mut self, state: &mut State);
    fn nodes_tab(&mut self, state: &mut State);
}

impl UiExt for Ui {
    fn intro_tab(&mut self, state: &mut State) {
        self.add(V_SCROLL_STACK).nest(|| {
            self.static_paragraph("Keru is an experimental GUI library.");
            
            #[node_key] const TEXT_EDIT_1: NodeKey;
            let edit = TEXT_EDIT_LINE
                .key(TEXT_EDIT_1)
                .text("")
                .placeholder_text("Single line text edit box");

            self.add(edit);

            #[node_key] const TEXT_EDIT_2: NodeKey;
            let edit2 = TEXT_EDIT
                .key(TEXT_EDIT_2)
                .size_y(Size::Pixels(200.0))
                .placeholder_text("Multiline text edit box")
                .text("Text edit");

            self.add(edit2);

            self.static_paragraph("Here are some basic GUI elements:");
            self.static_paragraph("Button and label:");

            self.h_stack().nest(|| {
                if self.add(BUTTON.text("Increase")).is_clicked(self) {
                    state.f32_value += 1.0;
                }
                let text = format!("{:.2}", state.f32_value);
                self.label(text.as_str());
            });

            self.static_paragraph("Image:");

            let image = IMAGE.static_image(include_bytes!("../src/textures/clouds.png"));
            self.add(image);

            let icon = ICON.static_svg(include_bytes!("assets/tiger.svg")).size(Size::Pixels(250.0), Size::Pixels(250.0));
            self.add(icon);

            self.static_paragraph("Fat slider:");

            self.add_component(Slider::new(&mut state.f32_value, 0.0, 100.0, true));

            self.static_paragraph("Classic slider:");
            self.classic_slider(&mut state.f32_value, 0.0, 100.0);

            self.static_paragraph("Press F1 for Inspect mode. This lets you see the bounds of the layout rectangles. \n\n\
                In Inspect mode, hovering nodes will also log an Info message with the node's debug name and source code location. \n\n\
                Press Ctrl+Tab and Ctrl+Shift+Tab to switch between tabs. \n\n\
                Press Ctrl+Plus, Ctrl+Minus and Ctrl+0 to control the zoom level of the default text style.\n\n");

            // todo: this is not very nice.
            // real examples without run_example_loop would do this in State::new(), I guess.
            let large_bold_style = state.large_bold_style.get_or_insert_with(|| {
                self.insert_style(TextStyle {
                    font_size: 32.0,
                    brush: ColorBrush([255, 0, 0, 255]),
                    font_weight: FontWeight::BOLD,
                    ..Default::default()
                })
            });
            self.add(TEXT
                .static_text("This text uses a different style.")
                .text_style(large_bold_style.clone())
            );

            self.static_paragraph("The tab viewer uses the \"children_can_hide\" property, that can be set on any node. This means that when switching tabs, all ui state is kept in the background, and we can switch back without recreating the node tree. In addition all implicit \"state\" like the scroll offset, the text in the edit boxes, etc. is retained. \n\n\
            Without \"children_can_hide\", everything would be cleaned up as soon as the tabs change.");

        });
    }

    fn text_tab(&mut self) {
        let v_stack = V_SCROLL_STACK.size_x(Frac(0.8)).size_y(Size::Fill);

        self.add(v_stack).nest(|| {
            self.add(LABEL.auto_markdown(true).static_text(
                "Keru uses `parley` for text through the `keru_text` library. \n\
                The text edit box supports IME, but this hasn't been thoroughly tested on all platforms yet."
            ));
            self.add(H_LINE.color(Color::WHITE));
            self.add(LABEL.auto_markdown(true).static_text(include_str!("showcase.md")));
            self.add(H_LINE.color(Color::WHITE));
            self.add(LABEL.static_text(JAPANESE_TEXT));
            self.add(H_LINE.color(Color::WHITE));
            self.add(LABEL.static_text(CYRILLIC_TEXT));
            self.add(H_LINE.color(Color::WHITE));
            self.add(LABEL.static_text(CHINESE_TEXT));
        });
    }

    fn nodes_tab(&mut self, _state: &mut State) {
        let nested_button_1 = BUTTON
            .size_y(Size::Frac(0.3))
            .static_text("Everything is a node.");
        
        let big_button = BUTTON
            .size_symm(Size::Fill)
            .static_text("This is a button that is also a Stack.")
            .stack(Axis::Y, Arrange::Center, 10.0);
        
        let nested_button_2 = BUTTON
            .size_y(Size::Frac(0.2))
            .size_x(Size::Fill)
            .static_text("Every node can be everything at once.");

        self.add(PANEL).nest(|| {
            self.add(big_button).nest(|| {
                self.spacer();
                self.add(nested_button_1);
                self.spacer();
                self.add(nested_button_2);
                self.spacer();
            });
        });
    }

    fn graphics_tab(&mut self, _state: &mut State) {

        self.add(V_SCROLL_STACK).nest(|| {

            self.static_paragraph("Keru uses its own wgpu-based renderer, with a similar architecture as the ones used in vger-rs and gpui.");

            let button_with_stroke = BUTTON
                .static_text("Button example")
                .color(Color::KERU_BLUE)
                .shape(Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: 20.0 })
                .stroke(5.0)
                .stroke_color(Color::GREEN)
                .stroke_dashes(15.0, 0.0);

            self.add(button_with_stroke);

            let button_with_colored_stroke = BUTTON
                .static_text("Button example 2")
                .color(Color::KERU_PINK)
                .stroke(3.0)
                .blur(5.0)
                .stroke_color(Color::RED)
                .shape(Shape::Rectangle {
                    rounded_corners: RoundedCorners::TOP_LEFT | RoundedCorners::BOTTOM_RIGHT,
                    corner_radius: 15.0
                });

            self.add(button_with_colored_stroke);

            self.static_paragraph("By adding Nodes with different Shape values, we can do some basic vector drawing. In debug mode, you can press F1 to see that every element is a regular Node.");

            #[node_key] const LINE_CONTAINER: NodeKey;
            let line_container = CONTAINER
                .size_x(Size::Fill)
                .size_y(Size::Pixels(100.0))
                .padding(0.0)
                .key(LINE_CONTAINER);

            self.add(line_container).nest(|| {
                let points = [
                    (0.05, 0.5),
                    (0.25, 0.2),
                    (0.45, 0.8),
                    (0.65, 0.3),
                    (0.85, 0.7),
                ];

                // Draw line segments
                for i in 0..points.len() - 1 {
                    let segment_node = Node::segment_frac(points[i], points[i + 1], Some(10.0))
                        .color(Color::KERU_GREEN)
                        .stroke_width(4.0);

                    self.add(segment_node);
                }

                // Draw circles at joins
                for &(x, y) in &points[..points.len().saturating_sub(1)] {
                    let circle_size = 16.0;
                    let circle_node = DEFAULT
                        .shape(Shape::Circle)
                        .color(Color::KERU_BLUE)
                        .anchor_symm(Anchor::Center)
                        .size_symm(Size::Pixels(circle_size))
                        .position_x(Pos::Frac(x))
                        .position_y(Pos::Frac(y));

                    self.add(circle_node);
                }

                // Draw arrow tip
                let p_prev = points[points.len() - 2];
                let p_last = points[points.len() - 1];
                let Xy { x: lx, y: ly } = self.get_node(LINE_CONTAINER).unwrap().inner_size();
                let dx = (p_last.0 - p_prev.0) * lx as f32;
                let dy = (p_last.1 - p_prev.1) * ly as f32;
                let angle = dy.atan2(dx);

                let arrow = DEFAULT
                    .shape(Shape::Triangle {
                        rotation: angle,
                        width: 0.6,
                    })
                    .color(Color::KERU_RED)
                    .anchor_symm(Anchor::Center)
                    .size_symm(Size::Pixels(50.0))
                    .position_x(Pos::Frac(p_last.0))
                    .position_y(Pos::Frac(p_last.1));

                self.add(arrow);

                // Draw hexagons
                let hexagon1 = DEFAULT
                    .shape(Shape::Hexagon {
                        size: 0.85,
                        rotation: 0.0,
                    })
                    .color(Color::KERU_PINK)
                    .anchor_symm(Anchor::Center)
                    .size_symm(Size::Pixels(60.0))
                    .position_x(Pos::Frac(0.15))
                    .position_y(Pos::Frac(0.5));

                self.add(hexagon1);

                let hexagon2 = DEFAULT
                    .shape(Shape::Hexagon {
                        size: 0.8,
                        rotation: std::f32::consts::PI / 6.0,
                    })
                    .color(Color::TRANSPARENT)
                    .stroke_color(Color::KERU_GREEN)
                    .stroke_width(3.0)
                    .anchor_symm(Anchor::Center)
                    .size_symm(Size::Pixels(50.0))
                    .position_x(Pos::Frac(0.35))
                    .position_y(Pos::Frac(0.5));

                self.add(hexagon2);
            });

            self.static_paragraph("There is also an API for using the keru renderer directly to do custom rendering with the keru primitives, such as rectangles, circles, line segments and quadratic Bezier curves.\n\n\
            This allows custom vector drawing without having to create one Node for each tiny graphics element.\n\n\
            This kind of custom drawing will use the same draw call as the keru GUI elements, so it's both faster and simpler to use compared to fully custom wgpu rendering. \n\n\
            Note that the closure is executed immediately, not stored: this means that the code in the closure can freely access all the state it wants without any kind of borrowing restrictions.\n\n\
            It also means that these primitives are created before the keru ones (GUI declaration time vs keru's post-layout rendering step in finish_frame(). However, the renderer is still able to render them in the correct z-order and at the correct post-layout position.)
            ");

            #[node_key] const CANVAS_CONTAINER: NodeKey;
            let canvas_container = CONTAINER
                .size_x(Size::Fill)
                .size_y(Size::Pixels(120.0))
                .padding(0.0)
                .color(Color::rgba_u8(30, 30, 40, 255))
                .key(CANVAS_CONTAINER);

            self.add(canvas_container);

            self.canvas_drawing(CANVAS_CONTAINER, |renderer| {
                use keru_draw::{Segment, ColorFill};

                let num_points = 120;
                let width = 500.0;
                let height = 120.0;
                let margin = 40.0;
                let num_coils = 6.0;
                let coil_radius = 35.0;

                let points: Vec<[f32; 2]> = (0..num_points)
                    .map(|i| {
                        let t = i as f32 / (num_points - 1) as f32;
                        let angle = t * num_coils * 2.0 * std::f32::consts::PI;
                        let x = margin + t * (width - 2.0 * margin) + coil_radius * 0.3 * angle.cos();
                        let y = height / 2.0 + coil_radius * angle.sin();
                        [x, y]
                    })
                    .collect();

                for i in 0..points.len() - 1 {
                    let t = i as f32 / (num_points - 1) as f32;
                    let angle = t * num_coils * 2.0 * std::f32::consts::PI;
                    let depth = angle.cos();
                    let thickness = 3.0 + 2.5 * (depth + 1.0) / 2.0;

                    let color = if depth > 0.0 {
                        Color::KERU_PINK
                    } else {
                        Color::rgba_u8(200, 100, 150, 255)
                    };

                    renderer.draw_segment(Segment {
                        start: points[i],
                        end: points[i + 1],
                        thickness,
                        fill: ColorFill::Color(color),
                        dash_length: None,
                        dash_offset: 0.0,
                        texture: None,
                        blur: 0.0,
                    });
                }
            });

            self.static_paragraph("Nodes also have `scale` and `offset` fields apply to all their children, including the shapes drawn with the canvas API. In this example, we're using the `StatefulTransformView` component, which uses scale and offset internally, and also implements scroll-zooming and panning with space+drag or with the middle mouse button.");

            let bg_panel = PANEL.size_symm(Size::Frac(0.8));
            self.add(bg_panel).nest(|| {
                self.add_component(StatefulTransformView).nest(|| {
                    self.add(V_STACK).nest(|| {
                        self.label("Transformed subtree");
    
                        self.add(BUTTON.text("Button"));
    
                        self.add(H_STACK).nest(|| {
                            self.add(PANEL.color(Color::RED).size_symm(Size::Pixels(50.0)));
                            self.add(PANEL.color(Color::GREEN).size_symm(Size::Pixels(50.0)));
                            self.add(PANEL.color(Color::BLUE).size_symm(Size::Pixels(50.0)));
                        });
    
                        self.static_paragraph("The canvas API can be used inside a zoomable area: these are analytically rendered quadratic Beziers that can be scaled infinitely.");

                        #[node_key] const CANVAS_CONTAINER: NodeKey;
                        let canvas_container = CONTAINER
                            .size_x(Size::Fill)
                            .size_y(Size::Pixels(120.0))
                            .padding(0.0)
                            .color(Color::rgba_u8(30, 30, 40, 255))
                            .key(CANVAS_CONTAINER);

                        self.add(canvas_container);

                        self.canvas_drawing(CANVAS_CONTAINER, |renderer| {
                            use keru_draw::QuadraticBezier;

                            let height = 120.0;
                            let mid_y = height / 2.0;

                            let beziers = [
                                QuadraticBezier {
                                    p0: [40.0, mid_y],
                                    p1: [80.0, mid_y - 40.0],
                                    p2: [120.0, mid_y],
                                    thickness: 4.0,
                                    color: Color::KERU_PINK,
                                    blur: 0.0,
                                },
                                QuadraticBezier {
                                    p0: [120.0, mid_y],
                                    p1: [160.0, mid_y + 50.0],
                                    p2: [200.0, mid_y],
                                    thickness: 4.0,
                                    color: Color::KERU_GREEN,
                                    blur: 0.0,
                                },
                                QuadraticBezier {
                                    p0: [200.0, mid_y],
                                    p1: [260.0, mid_y - 45.0],
                                    p2: [320.0, mid_y],
                                    thickness: 4.0,
                                    color: Color::KERU_BLUE,
                                    blur: 0.0,
                                },
                                QuadraticBezier {
                                    p0: [320.0, mid_y],
                                    p1: [360.0, mid_y + 50.0],
                                    p2: [400.0, mid_y],
                                    thickness: 4.0,
                                    color: Color::KERU_GREEN,
                                    blur: 0.0,
                                },
                            ];

                            for bezier in beziers {
                                renderer.draw_quadratic_bezier(bezier);
                            }
                        });

                        self.label("Unfortunately we can't say the same thing for text. It's rasterized on the CPU using parley's built-in rasterizer and drawn with a traditional atlas renderer. When zooming, we use the same texture and just scale the quads.");

                        self.label("But now that the Slug algorithm isn't patented anymore, maybe this will change.");
                        
                    });
                });
            

            });

            let header = LABEL.static_text("Note that the Transformed view is a stateful component.\n\
            It can remember its own state (the pan and zoom of the transform) without us having to make space for it in our own State struct and passing it by reference.\n\
            The state is initialized to its Default value when the component is first added to the tree, and is stored within the Ui struct in a Box<dyn Any>.\n\
            You can also see how this state is retained when swiching between tabs, thanks to the \"children can hide\" property.\n\n\
            The plan is to provide most components both in stateful and state-borrowing forms.");
            self.add(header);

            self.static_paragraph("For fully custom wgpu rendered content, there is also an experimental system for rendering in-between the Keru ui elements. See the \"custom_rendering\" example. This will necessarily mean that the keru rendering will have to be split between multiple draw calls.");

            self.static_paragraph("Of course if you just want to draw custom wgpu effects below or above the GUI, as when rendering a game, you can do it without any help from keru. Keru doesn't steal control of your winit/wgpu loop. See the \"window_loop\" example.");

        });


    }
}

impl State {
    #[track_caller]
    fn update_ui(&mut self, ui: &mut Ui) {
        self.update_global_text(ui);

        ui.vertical_tabs(&self.tabs[..], &mut self.current_tab)
            .nest(|| match self.tabs[self.current_tab] {
                INTRO_TAB => ui.intro_tab(self),
                NODES_TAB => ui.nodes_tab(self),
                TEXT_TAB => ui.text_tab(),
                GRAPHICS_TAB => ui.graphics_tab(self),
                _ => {}
            });
    }

    fn update_global_text(&mut self,  ui: &mut Ui) {
        if ui.key_input().key_mods().control_key() {
            if ui.key_input().key_pressed(&Key::Character("=".into())) || 
               ui.key_input().key_pressed(&Key::Character("+".into())) {
                ui.default_text_style_mut().font_size = (ui.default_text_style().font_size + 2.0).min(72.0);
            } else if ui.key_input().key_pressed(&Key::Character("-".into())) {
                ui.default_text_style_mut().font_size = (ui.default_text_style().font_size - 2.0).max(8.0);
            } else if ui.key_input().key_pressed(&Key::Character("0".into())) {
                *ui.default_text_style_mut() = ui.original_default_style();
            }
        }

        if ui.key_input().key_mods().control_key() {
            if let Some(scroll_delta) = ui.scroll_delta() {
                if scroll_delta.y > 0.0 {
                    ui.default_text_style_mut().font_size = (ui.default_text_style().font_size + 2.0).min(72.0);
                } else if scroll_delta.y < 0.0 {
                    ui.default_text_style_mut().font_size = (ui.default_text_style().font_size - 2.0).max(8.0);
                }
            }
        }
    }
}

fn main() {
    // basic_env_logger_init();

    let state = State {
        tabs: vec![INTRO_TAB, NODES_TAB, TEXT_TAB, GRAPHICS_TAB],
        current_tab: 0,
        f32_value: 20.0,
        ..Default::default()
    };
    run_example_loop(state, State::update_ui);
}
