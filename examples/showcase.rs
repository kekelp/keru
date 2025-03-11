use keru::Size::*;
use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
struct State {
    tabs: Vec<Tab>,
    current_tab: usize,

    f32_value: f32,
}

const COMPONENTS_TAB: Tab = Tab("Components");
const TEXT_TAB: Tab = Tab("Cosmic Text");
const WEIRD_TAB: Tab = Tab("Weird Stuff");

const CHINESE_TEXT: &str = "此后，人民文学出版社和齐鲁书社的做法被诸多出版社效仿，可见文化部出版局1985年的一纸批文并没有打消各地出版社出版此书的念头。所以，1988年新闻出版署发出了《关于整理出版〈金瓶梅〉及其研究资料的通知》。《通知》首先说明《金瓶梅》及其研究资料的需求“日益增大”，“先后有十余家出版社向我署提出报告，分别要求出版《金瓶梅》的各种版本及改编本，包括图录、连环画及影视文学剧本等”，但话锋一转，明确提出“《金瓶梅》一书虽在文学史上占有重要地位，但该书存在大量自然主义的秽亵描写，不宜广泛印行";

const JAPANESE_TEXT: &str = "ヘッケはこれらのL-函数が全複素平面へ有理型接続を持ち、指標が自明であるときには s = 1 でオーダー 1 である極を持ち、それ以外では解析的であることを証明した。原始ヘッケ指標（原始ディリクレ指標に同じ方法である modulus に相対的に定義された）に対し、ヘッケは、これらのL-函数が指標の L-函数の函数等式を満たし、L-函数の複素共役指標であることを示した。 主イデアル上の座と、無限での座を含む全ての例外有限集合の上で 1 である単円の上への写像を取ることで、イデール類群の指標 ψ を考える。すると、ψ はイデアル群 IS の指標 χ を生成し、イデアル群は S 上に入らない素イデアル上の自由アーベル群となる。";

trait Components {
    fn components_tab(&mut self, state: &mut State);
    fn text_tab(&mut self);
    fn weird_tab(&mut self, state: &mut State);
}

impl Components for Ui {
    fn components_tab(&mut self, state: &mut State) {
        self.add(V_SCROLL_STACK).nest(|| {
            self.paragraph(
                "Thanks for checking out Keru.\n\n\
                Keru is an experimental Graphical User Interface library, with the goal of being as easy to use as Egui (and even a bit easier), but without the downsides of immediate mode.\n\n\
                I think the experiment has been really successful, and I'm very happy with the user-facing API of the library and the internal architecture. \n\n\
                However, many features are still missing."
            );

            self.text_line("Button and label:");

            self.h_stack().nest(|| {
                if self.add(BUTTON.text("Increase")).is_clicked(self) {
                    state.f32_value += 1.0;
                }
                let text = format!("{:.2}", state.f32_value);
                self.label(&text);
            });

            self.text_line("Fat slider:");
            self.slider(&mut state.f32_value, 0.0, 100.0);

            self.text_line("Classic slider:");
            self.classic_slider(&mut state.f32_value, 0.0, 100.0);

            self.paragraph(
                "Press F1 for Inspect mode. This lets you see the bounds of the layout rectangles. \n\n\
                In Inspect mode, hovering nodes will also log an Info message with the name and source code location of the node."
            );
        });
    }

    fn text_tab(&mut self) {
        let v_stack = V_STACK
            .size_x(Frac(0.8))
            .size_y(Size::Frac(0.7))
            .scrollable_y(true);
        let image = IMAGE.static_image(include_bytes!("../src/textures/clouds.png"));

        self.add(v_stack).nest(|| {
            self.label(&Static(JAPANESE_TEXT));
            self.add(image);
            self.label(&Static(CHINESE_TEXT));
        });
    }

    fn weird_tab(&mut self, _state: &mut State) {
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
            .static_image(include_bytes!("../src/textures/clouds.png"))
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

impl ExampleLoop for State {
    fn update_ui(&mut self, ui: &mut Ui) {
        ui.vertical_tabs(&self.tabs[..], &mut self.current_tab)
            .nest(|| match self.tabs[self.current_tab] {
                COMPONENTS_TAB => ui.components_tab(self),
                TEXT_TAB => ui.text_tab(),
                WEIRD_TAB => ui.weird_tab(self),
                _ => {}
            });
    }
}

fn main() {
    basic_env_logger_init();
    let state = State {
        tabs: vec![COMPONENTS_TAB, TEXT_TAB, WEIRD_TAB],
        current_tab: 0,
        f32_value: 20.0,
        ..Default::default()
    };
    run_example_loop(state);
}
