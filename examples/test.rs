use keru::Size::*;
use keru::example_window_loop::*;
use keru::*;

#[derive(Default)]
pub struct State {
    pub current_tab: usize,
}

const SEED_TAB: &str = "Seed";
const FEED_TAB: &str = "Feed";
const CHUCK_TAB: &str = "Chuck";

const CHINESE_TEXT: &str = "此后，人民文学出版社和齐鲁书社的做法被诸多出版社效仿，可见文化部出版局1985年的一纸批文并没有打消各地出版社出版此书的念头。所以，1988年新闻出版署发出了《关于整理出版〈金瓶梅〉及其研究资料的通知》。《通知》首先说明《金瓶梅》及其研究资料的需求“日益增大”，“先后有十余家出版社向我署提出报告，分别要求出版《金瓶梅》的各种版本及改编本，包括图录、连环画及影视文学剧本等”，但话锋一转，明确提出“《金瓶梅》一书虽在文学史上占有重要地位，但该书存在大量自然主义的秽亵描写，不宜广泛印行";

const JAPANESE_TEXT: &str = "ヘッケはこれらのL-函数が全複素平面へ有理型接続を持ち、指標が自明であるときには s = 1 でオーダー 1 である極を持ち、それ以外では解析的であることを証明した。原始ヘッケ指標（原始ディリクレ指標に同じ方法である modulus に相対的に定義された）に対し、ヘッケは、これらのL-函数が指標の L-函数の函数等式を満たし、L-函数の複素共役指標であることを示した。 主イデアル上の座と、無限での座を含む全ての例外有限集合の上で 1 である単円の上への写像を取ることで、イデール類群の指標 ψ を考える。すると、ψ はイデアル群 IS の指標 χ を生成し、イデアル群は S 上に入らない素イデアル上の自由アーベル群となる。";

impl ExampleLoop for State {
    fn update_ui(&mut self, ui: &mut Ui) {

        let image1 = IMAGE.static_image(include_bytes!("../src/textures/E.png"));
        let image2 = IMAGE.static_image(include_bytes!("../src/textures/clouds_450.png"));

        ui.v_stack().nest(|| {
            ui.add(image2);
            // ui.add(image1);
        });

    }
}

fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Warn)
        .filter_module("keru::node_params", log::LevelFilter::Trace)
        .init();
    let state = State::default();
    run_example_loop(state);
}
