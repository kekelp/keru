use keru::*;
use keru::example_window_loop::*;

#[derive(Default)]
pub struct State {
    pub count: i32,
    pub show: bool,
}

impl ExampleLoop for State {
    fn update_ui(&mut self, ui: &mut Ui) {
        ui.add_anon().params(PANEL).size_x(Size::Fixed(Len::Pixels(200))).place().nest(|| {
            ui.add_anon().params(V_SCROLL_AREA).place().nest(|| {
                // ui.multiline_text("Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium doloremque laudantium, totam rem aperiam, eaque ipsa quae ab illo inventore veritatis et quasi architecto beatae vitae dicta sunt explicabo. Nemo enim ipsam voluptatem quia voluptas sit aspernatur aut odit aut fugit, sed quia consequuntur magni dolores eos qui ratione voluptatem sequi nesciunt. Neque porro quisquam est, qui dolorem ipsum quia dolor sit amet, consectetur, adipisci velit, sed quia non numquam eius modi tempora incidunt ut labore et dolore magnam aliquam quaerat voluptatem. Ut enim ad minima veniam, quis nostrum exercitationem ullam corporis suscipit laboriosam, nisi ut aliquid ex ea commodi consequatur? Quis autem vel eum iure reprehenderit qui in ea voluptate velit esse quam nihil molestiae consequatur, vel illum qui dolorem eum fugiat quo voluptas nulla pariatur");
                // ui.multiline_text("Он воротился из-за границы и блеснул в виде лектора на кафедре университета уже в самом конце сороковых годов. Успел же прочесть всего только несколько лекций, и, кажется, об аравитянах*; успел тоже защитить блестящую диссертацию о возникавшем было гражданском и ганзеатическом значении немецкого городка Ганау, в эпоху между 1413 и 1428 годами, а вместе с тем и о тех особенных и неясных причинах, почему значение это вовсе не состоялось. Диссертация эта ловко и больно уколола тогдашних славянофилов* и разом доставила ему между ними многочисленных и разъяренных врагов. Потом - впрочем, уже после потери кафедры - он успел напечатать (так сказать, в виде отместки и чтоб указать, кого они потеряли) в ежемесячном и прогрессивном журнале, переводившем из Диккенса и проповедовавшем Жорж Занда*, начало одного глубочайшего исследования - кажется, о причинах необычайного нравственного благородства каких-то рыцарей в какую-то эпоху или что-то в этом роде. По крайней мере проводилась какая-то высшая и необыкновенно благородная мысль. Говорили потом, что продолжение исследования было поспешно запрещено и что даже прогрессивный журнал пострадал за напечатанную первую половину. Очень могло это быть, потому что чего тогда не было? Но в данном случае вероятнее, что ничего не было и что автор сам поленился докончить исследование. Прекратил же он свои лекции об аравитянах потому, что перехвачено было как-то и кем-то (очевидно, из ретроградных врагов его) письмо к кому-то с изложением каких-то \"обстоятельств\", вследствие чего кто-то потребовал от него каких-то объяснений*. Не знаю, верно ли, но утверждали еще, что в Петербурге было отыскано в то же самое время какое-то громадное, противоестественное и противогосударственное общество, человек в тринадцать, и чуть не потрясшее здание. Говорили, что будто бы они собирались переводить самого Фурье*.");
                ui.multiline_text("此后，人民文学出版社和齐鲁书社的做法被诸多出版社效仿，可见文化部出版局1985年的一纸批文并没有打消各地出版社出版此书的念头。所以，1988年新闻出版署发出了《关于整理出版〈金瓶梅〉及其研究资料的通知》。《通知》首先说明《金瓶梅》及其研究资料的需求“日益增大”，“先后有十余家出版社向我署提出报告，分别要求出版《金瓶梅》的各种版本及改编本，包括图录、连环画及影视文学剧本等”，但话锋一转，明确提出“《金瓶梅》一书虽在文学史上占有重要地位，但该书存在大量自然主义的秽亵描写，不宜广泛印行");
                ui.multiline_text("ヘッケはこれらのL-函数が全複素平面へ有理型接続を持ち、指標が自明であるときには s = 1 でオーダー 1 である極を持ち、それ以外では解析的であることを証明した。原始ヘッケ指標（原始ディリクレ指標に同じ方法である modulus に相対的に定義された）に対し、ヘッケは、これらのL-函数が指標の L-函数の函数等式を満たし、L-函数の複素共役指標であることを示した。 主イデアル上の座と、無限での座を含む全ての例外有限集合の上で 1 である単円の上への写像を取ることで、イデール類群の指標 ψ を考える。すると、ψ はイデアル群 IS の指標 χ を生成し、イデアル群は S 上に入らない素イデアル上の自由アーベル群となる。");
            });
        });
    }
}

fn main() {
    basic_env_logger_init();
    let state = State::default();
    run_example_loop(state);
}
