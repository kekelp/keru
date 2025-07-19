use keru::Size::*;
use keru::example_window_loop::*;
use keru::*;
use parley2::{TextStyle2 as TextStyle, ColorBrush, FontWeight};

#[derive(Default)]
struct State {
    tabs: Vec<Tab>,
    current_tab: usize,

    f32_value: f32,
    large_bold_style: Option<StyleHandle>,
}

const INTRO_TAB: Tab = Tab("Intro");
const TEXT_TAB: Tab = Tab("Text");
const WEIRD_TAB: Tab = Tab("Other Stuff");

const CHINESE_TEXT: &str = "此后，人民文学出版社和齐鲁书社的做法被诸多出版社效仿，可见文化部出版局1985年的一纸批文并没有打消各地出版社出版此书的念头。所以，1988年新闻出版署发出了《关于整理出版〈金瓶梅〉及其研究资料的通知》。《通知》首先说明《金瓶梅》及其研究资料的需求“日益增大”，“先后有十余家出版社向我署提出报告，分别要求出版《金瓶梅》的各种版本及改编本，包括图录、连环画及影视文学剧本等”，但话锋一转，明确提出“《金瓶梅》一书虽在文学史上占有重要地位，但该书存在大量自然主义的秽亵描写，不宜广泛印行";

const CYRILLIC_TEXT: &str = "Мунди деленит молестиае усу ад, пертинах глориатур диссентиас ет нец. Ессент иудицабит маиестатис яуи ад, про ут дицо лорем легере. Вис те цоммодо сцрипта цорпора, тритани интеллегат аргументум цу еум, меи те яуем феугаит. При дисцере интеллегат ат, аеяуе афферт фуиссет ех вих. Цу хас интегре тхеопхрастус. Диам волуптатибус про еа.

Вис цу сцаевола мнесарчум, ин усу волутпат инцоррупте. При ет стет инвидунт форенсибус. Цонсететур волуптатум омиттантур яуи ет, ут доминг промпта доценди сед. Цетеро пробатус ехпетенда сеа ин, диам витае доминг ад вим.
";

const JAPANESE_TEXT: &str = "ヘッケはこれらのL-函数が全複素平面へ有理型接続を持ち、指標が自明であるときには s = 1 でオーダー 1 である極を持ち、それ以外では解析的であることを証明した。原始ヘッケ指標（原始ディリクレ指標に同じ方法である modulus に相対的に定義された）に対し、ヘッケは、これらのL-函数が指標の L-函数の函数等式を満たし、L-函数の複素共役指標であることを示した。 主イデアル上の座と、無限での座を含む全ての例外有限集合の上で 1 である単円の上への写像を取ることで、イデール類群の指標 ψ を考える。すると、ψ はイデアル群 IS の指標 χ を生成し、イデアル群は S 上に入らない素イデアル上の自由アーベル群となる。";

trait Components {
    fn intro_tab(&mut self, state: &mut State);
    fn text_tab(&mut self);
    fn other_tab(&mut self, state: &mut State);
}

impl Components for Ui {
    fn intro_tab(&mut self, state: &mut State) {
        self.add(V_SCROLL_STACK).nest(|| {
            self.static_paragraph("Keru is an experimental GUI library focused on combining a simple and natural programming model with high performance and flexibility.");
            
            #[node_key] const TEXT_EDIT_1: NodeKey;
            let edit = TEXT_EDIT_LINE
                .key(TEXT_EDIT_1)
                .text("Single line text edit box");

            self.add(edit);

            #[node_key] const TEXT_EDIT_2: NodeKey;
            let edit2 = TEXT_EDIT
                .key(TEXT_EDIT_2)
                .text("Text edit box");

            self.add(edit2);

            self.static_paragraph("Here are some basic GUI elements: \n");
            self.static_paragraph("Button and label:");

            self.h_stack().nest(|| {
                if self.add(BUTTON.text("Increase")).is_clicked(self) {
                    state.f32_value += 1.0;
                }
                let text = format!("{:.2}", state.f32_value);
                self.label(text.as_str());
            });

            let image = IMAGE.static_image(include_bytes!("../src/textures/clouds.png"));
            self.add(image);

            self.static_paragraph("Fat slider:");
            self.slider(&mut state.f32_value, 0.0, 100.0);

            self.static_paragraph("Classic slider:");
            self.classic_slider(&mut state.f32_value, 0.0, 100.0);

            self.static_paragraph("
            Press F1 for Inspect mode. This lets you see the bounds of the layout rectangles. \n\n\
            In Inspect mode, hovering nodes will also log an Info message with the node's debug name and source code location. \n\n\
            Press Ctrl+Tab and Ctrl+Shift+Tab to switch between tabs. \n\n\
            Press Ctrl+Plus, Ctrl+Minus and Ctrl+0 to control the zoom level of the default text style.\n\n
            ");

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
                .text_style(&large_bold_style)
            );
            
            self.static_paragraph("\nNormally, \"state management\" looks the same as in immediate mode libraries: all state is centralized. \n\n\
            But if a widget has a small bit of state that you don't want to centralize, \"StateKey\"s provide a way to handle it. \n\n\
            This bool value isn't declared in the main struct state.");

            #[node_key] const BUTTON3: NodeKey;
            #[state_key] const WIDGET_STATE: StateKey<bool>;
            
            self.h_stack().nest(|| {
                if self.add(BUTTON.text("Toggle bool").key(BUTTON3)).is_clicked(self) {
                    *self.state_mut(WIDGET_STATE) = ! self.state(WIDGET_STATE)
                }
    
                if *self.state(WIDGET_STATE) {
                    self.label("Bool on");
                } else {
                    self.label("Bool off");
                }
            });

            self.static_paragraph("The tab viewer uses the \"children_can_hide\" property on the tab containers. This means that when switching tabs, all ui state is kept in the background: the value of the bool is retained, as well as the scroll state, the text in the edit boxes, etc. \n\n\
            Without \"children_can_hide\", everything would be cleaned up as soon as the tabs change.");

        });
    }

    fn text_tab(&mut self) {
        let v_stack = V_STACK
            .size_x(Frac(0.8))
            .size_y(Size::Frac(0.7))
            .scrollable_y(true);

        self.add(v_stack).nest(|| {
            self.static_label(
                "Keru uses Parley for text. \n\n\
                This means that it's a good 90% of the way there to text that just works in the way that you expect it, including font discovery, full-featured editing and IME support. Accessibility is supported by Parley itself, but it's not integrated into Keru yet.\n\n\
                I think it's reasonable to expect Parley to get to 100% in a couple of years or so.\n\n\
                Keru is curently using the built-in Swash cpu rasterizer and a basic homemade atlas renderer similar to Glyphon. This means that the text rendering performance is not great. This should also be solved soon by switching to the new sparse-strip based Vello renderer once it's ready and stable."
            );
            self.label(&Static(JAPANESE_TEXT));
            self.label(&Static(CYRILLIC_TEXT));
            self.label(&Static(CHINESE_TEXT));
        });
    }

    fn other_tab(&mut self, _state: &mut State) {
        let big_button = BUTTON
            .size_symm(Size::Fill)
            .static_text("Button that is also a Stack")
            .stack(Axis::Y, Arrange::Center, 10);

        let nested_button_1 = BUTTON
            .size_y(Size::Frac(0.3))
            .static_text("Everything is a node");
        let nested_button_2 = BUTTON
            .size_y(Size::Frac(0.2))
            .size_x(Size::Fill)
            .static_text("And every node can be everything at once\n(for now)");

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
}

impl State {
    fn update_ui(&mut self, ui: &mut Ui) {
        self.update_global_text(ui);

        ui.vertical_tabs(&self.tabs[..], &mut self.current_tab)
            .nest(|| match self.tabs[self.current_tab] {
                INTRO_TAB => ui.intro_tab(self),
                TEXT_TAB => ui.text_tab(),
                WEIRD_TAB => ui.other_tab(self),
                _ => {}
            });
    }

    fn update_global_text(&mut self,  ui: &mut Ui) {
        if ui.key_input().key_mods().control_key() {
            if ui.key_input().key_pressed(&winit::keyboard::Key::Character("=".into())) || 
               ui.key_input().key_pressed(&winit::keyboard::Key::Character("+".into())) {
                ui.default_text_style_mut().font_size = (ui.default_text_style().font_size + 2.0).min(72.0);
            } else if ui.key_input().key_pressed(&winit::keyboard::Key::Character("-".into())) {
                ui.default_text_style_mut().font_size = (ui.default_text_style().font_size - 2.0).max(8.0);
            } else if ui.key_input().key_pressed(&winit::keyboard::Key::Character("0".into())) {
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
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Warn)
        .filter_module("keru", log::LevelFilter::Info)
        .filter_module("keru::tree", log::LevelFilter::Trace)
        .init();

    let state = State {
        tabs: vec![INTRO_TAB, TEXT_TAB, WEIRD_TAB],
        current_tab: 0,
        f32_value: 20.0,
        ..Default::default()
    };
    run_example_loop(state, State::update_ui);
}
