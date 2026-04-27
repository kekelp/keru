use keru::*;
use keru::node_library::{
    V_STACK, H_STACK, TEXT, PANEL, CONTAINER, BUTTON, ICON, V_SCROLL_STACK, H_LINE, DEFAULT,
};
use keru_draw::parley::{FontFamily, FontFamilyName, GenericFamily};

const SIDEBAR_ITEMS: &[&str] = &[
    "Solutions", "Analytics", "Case Studies", "SEO", "Content Management", "Agentic Workflows",
    "Clean Code", "Card", "User Funnel", "Outreach", "API", "Workflows", "AI Policy", "Use Cases", "Responsive Design", "Business Logic", 
    "Big Data", "Pipelines", "Call To Action", "Publishing", "Empowerment", "Test-Driven Development", "Customer Stories", "Marketplace", 
];

struct State {
    selected_item: usize,
}

const GREY_BG: Color = Color::new(0.97, 0.97, 0.97, 1.0);
const SUBTITLE_GREY: Color = Color::new(0.57, 0.57, 0.57, 1.0);
const SELECTED_BG: Color = Color::new(0.93, 0.93, 0.93, 1.0);
const BORDER_COLOR: Color = Color::new(0.90, 0.90, 0.90, 1.0);
const AVATAR_DARK: Color = Color::new(0.1, 0.1, 0.1, 1.0);
const AVATAR_LIGHT_C: Color = Color::new(0.93, 0.93, 0.93, 1.0);
const BADGE_ORANGE: Color = Color::new(0.9, 0.7, 0.4, 1.0);


const CARD: Node = keru::node_library::PANEL
    .shape(Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: 12.0 })
    .color(Color::WHITE)
    .stroke(1.0)
    .stroke_color(BORDER_COLOR)
    .shadow(Shadow { blur: 2.0, offset: Xy::new(0.0, 1.0), color: Some(BORDER_COLOR) })
    .padding(16.0)
    .stack(Axis::Y, Arrange::Start, 12.0)
    .size_symm(Size::Fill);

const DIVIDER: Node = H_LINE.stroke_color(BORDER_COLOR).stroke_width(1.0);

fn card_header(ui: &mut Ui, title: &'static str, subtitle: &'static str) {
    let stack = V_STACK.size_x(Size::Fill).stack_arrange(Arrange::Start).stack_spacing(0.0);
    let title_text = TEXT.static_text(title).text_color(Color::BLACK).text_size(17.0).bold().position_x(Pos::Start);
    let subtitle_text = TEXT.static_text(subtitle).text_color(SUBTITLE_GREY).text_size(14.0).position_x(Pos::Start);

    ui.add(stack).nest(|| {
        ui.add(title_text);
        ui.add(subtitle_text);
    });
}

fn badge_preview(ui: &mut Ui) {
    let container = CONTAINER.size_symm(Size::Pixels(80.0));

    let person = DEFAULT
        .shape(Shape::Circle)
        .color(AVATAR_LIGHT_C)
        .size_symm(Size::Pixels(52.0))
        .stroke(1.5)
        .stroke_color(BORDER_COLOR)
        .anchor_symm(Anchor::Center)
        .position(Pos::Frac(0.43), Pos::Frac(0.43));

    let badge_circle = DEFAULT
        .shape(Shape::Circle)
        .color(BADGE_ORANGE)
        .size_symm(Size::Pixels(22.0))
        .anchor_symm(Anchor::Center)
        .position(Pos::Frac(0.72), Pos::Frac(0.72));

    ui.add(container).nest(|| {
        ui.add(person);
        ui.add(badge_circle);
    });
}

fn button_group_preview(ui: &mut Ui) {
    let container = BUTTON
        .color(Color::WHITE)
        .size_symm(Size::Pixels(40.0))
        .stroke(1.0)
        .stroke_color(BORDER_COLOR)
        .text_color(Color::BLACK)
        .text_size(16.0);

    let bold = container.shape(Shape::Rectangle { rounded_corners: RoundedCorners::LEFT, corner_radius: 6.0 }).text("B").bold();
    let italic = container.shape(Shape::Rectangle { rounded_corners: RoundedCorners::NONE, corner_radius: 6.0 }).text("I").italic();
    let underline = container.shape(Shape::Rectangle { rounded_corners: RoundedCorners::RIGHT, corner_radius: 6.0 }).text("U");

    ui.add(H_STACK.stack_spacing(-1.0)).nest(|| {
        for btn in [bold, italic, underline] {
            ui.add(btn);
        }
    });
}

fn avatar_group_preview(ui: &mut Ui) {
    let letters = ["A", "E", "C", "+1"];

    let circle = BUTTON
        .shape(Shape::Circle)
        .color(AVATAR_LIGHT_C)
        .size_symm(Size::Pixels(40.0))
        .stroke(2.0)
        .stroke_color(Color::WHITE)
        .position_x(Pos::Center)
        .text_color(Color::new(0.4, 0.4, 0.4, 1.0))
        .text_size(12.0);

    let slot = CONTAINER.size_x(Size::Pixels(28.0)).size_y(Size::Pixels(40.0));

    ui.add(H_STACK.stack_spacing(0.0)).nest(|| {
        for letter in &letters {
            ui.add(slot).nest(|| {
                ui.add(circle.text(letter));
            });
        }
    });
}

fn sidebar(state: &mut State, ui: &mut Ui) {
    let sidebar_panel = PANEL
        .color(Color::WHITE)
        .size_x(Size::Pixels(230.0))
        .size_y(Size::Fill)
        .stack(Axis::Y, Arrange::Start, 0.0)
        .padding(0.0);

    let header_row = H_STACK.size_x(Size::Fill).stack_arrange(Arrange::Start).stack_spacing(10.0).padding(16.0);

    let logo = PANEL
        .color(Color::BLACK)
        .size_symm(Size::Pixels(36.0))
        .shape(Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: 8.0 })
        .padding(0.0)
        .static_text("K")
        .text_color(Color::WHITE)
        .text_size(18.0)
        .bold();

    let title_text = TEXT.static_text("Modern Example").text_color(Color::BLACK).text_size(16.0).bold();

    let list = V_SCROLL_STACK.size_x(Size::Fill).stack_spacing(2.0).padding(8.0);

    #[node_key] const ITEM_KEY: NodeKey;

    ui.add(sidebar_panel).nest(|| {
        ui.add(header_row).nest(|| {
            ui.add(logo);
            ui.add(title_text);
        });
        ui.add(DIVIDER);
        ui.add(list).nest(|| {
            for (i, name) in SIDEBAR_ITEMS.iter().enumerate() {
                let key = ITEM_KEY.sibling(i);
                if ui.is_clicked(key) {
                    state.selected_item = i;
                }
                let item_color = if state.selected_item == i { SELECTED_BG } else { Color::TRANSPARENT };
                let item = PANEL
                    .color(item_color)
                    .size_x(Size::Fill)
                    .size_y(Size::FitContent)
                    .shape(Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: 6.0 })
                    .padding(15.0)
                    .sense_click(true)
                    .stack(Axis::X, Arrange::Start, 0.0)
                    .text(name).text_color(Color::BLACK).text_size(14.0)
                    .text_alignment(keru_draw::parley::Alignment::Start)
                    .key(key);

                ui.add(item);
            }
        });
    });
}

fn shapes_preview(ui: &mut Ui) {
    let slot = CONTAINER.size_x(Size::Pixels(36.0)).size_y(Size::Pixels(48.0));

    let red: Color = Color::new(0.88, 0.78, 0.78, 1.0);
    let green: Color = Color::new(0.78, 0.86, 0.78, 1.0);
    let blue: Color = Color::new(0.78, 0.80, 0.88, 1.0);

    let circle = BUTTON
        .shape(Shape::Circle)
        .color(red)
        .size_symm(Size::Pixels(48.0))
        .stroke(2.0)
        .stroke_color(Color::WHITE)
        .position_x(Pos::Center);

    let hexagon = BUTTON
        .shape(Shape::Hexagon { size: 0.85, rotation: 3.14 / 2.0 })
        .color(green)
        .size_symm(Size::Pixels(64.0))
        .stroke(2.0)
        .stroke_color(Color::WHITE)
        .position_x(Pos::Center);

    let hexagon2 = hexagon
        .shape(Shape::Hexagon { size: 0.85, rotation: 0.0 })
        .color(blue);

    ui.add(H_STACK.stack_spacing(0.0)).nest(|| {
        for shape in [circle, hexagon, hexagon2] {
            ui.add(slot).nest(|| {
                ui.add(shape);
            });
        }
    });
}

fn main_content(ui: &mut Ui) {
    let main = CONTAINER
        .size_x(Size::Fill)
        .size_y(Size::Fill)
        .stack(Axis::Y, Arrange::Start, 0.0);

    let controls_bar = H_STACK.size_x(Size::Fill).stack_arrange(Arrange::End).stack_spacing(6.0);
    let gear_icon = ICON
        .static_svg(include_bytes!("../src/svg_icons/settings.svg"))
        .size_symm(Size::Pixels(18.0))
        .color(Color::BLACK);
    let controls_text = TEXT.static_text("Controls").text_color(Color::BLACK).text_size(15.0);

    let grid = CONTAINER
        .size_symm(Size::Fill)
        .clip_children_y(true)
        .grid(MainAxisCellSize::Count(3), 16.0, 16.0, GridFlow::DEFAULT)
        .padding(16.0);

    let avatar = DEFAULT
        .shape(Shape::Circle)
        .color(AVATAR_DARK)
        .size_symm(Size::Pixels(60.0))
        .static_text("K")
        .text_color(Color::WHITE)
        .text_size(22.0)
        .bold();

    let black_button = BUTTON
        .static_text("Button")
        .color(Color::BLACK)
        .text_color(Color::WHITE)
        .text_size(14.0)
        .shape(Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: 8.0 });

    ui.add(main).nest(|| {
        ui.add(controls_bar).nest(|| {
            ui.add(gear_icon);
            ui.add(controls_text);
        });

        quote(ui);

        ui.add(grid).nest(|| {
            ui.add(CARD).nest(|| {
                card_header(ui, "Avatar", "Display");
                ui.add(avatar);
            });

            ui.add(CARD).nest(|| {
                card_header(ui, "Avatar Group", "Display");
                avatar_group_preview(ui);
            });

            ui.add(CARD).nest(|| {
                card_header(ui, "Badge", "Display");
                badge_preview(ui);
            });

            ui.add(CARD).nest(|| {
                card_header(ui, "Button", "Input");
                ui.add(black_button);
            });

            ui.add(CARD).nest(|| {
                card_header(ui, "Button Group", "Input");
                button_group_preview(ui);
            });

            ui.add(CARD).nest(|| {
                card_header(ui, "Shapes", "Display");
                shapes_preview(ui);
            });
        });
    });
}

fn quote(ui: &mut Ui) {
    let parley_serif = &[TextStyleProperty::FontFamily(FontFamily::Single(FontFamilyName::Generic(GenericFamily::Serif)))];

    let open_mark = TEXT.text("“").text_size(72.0).text_color(Color::new(0.75, 0.75, 0.75, 1.0)).text_properties(parley_serif).position_symm(Pos::Start);
    let close_mark = open_mark.text("”").position_symm(Pos::End);
    
    let quote_text = TEXT
        .static_text("We **can't get enough** of web-style interfaces")
        .auto_markdown(true)
        .text_color(Color::new(0.33, 0.33, 0.33, 1.0))
        .text_size(32.0).italic()
        .position_symm(Pos::Center);

    let author = TEXT.static_text("— Everyone").text_color(SUBTITLE_GREY).text_size(16.0).position_symm(Pos::End);

    let wrapper = CONTAINER
        .color(Color::WHITE)
        .size_x(Size::Frac(0.9))
        .size_y(Size::Frac(0.35))
        .shape(Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: 8.0 })
        .stroke(1.0)
        .stroke_color(BORDER_COLOR)
        .padding_x(20.0)
        .padding_y(8.0);

    ui.add(wrapper).nest(|| {
        ui.add(open_mark);
        ui.add(quote_text);
        ui.add(close_mark);
        ui.add(author);
    });
}


fn update_ui(state: &mut State, ui: &mut Ui) {
    let root = PANEL
        .color(GREY_BG)
        .size_symm(Size::Fill)
        .stack(Axis::X, Arrange::Start, 0.0)
        .padding(0.0);

    ui.add(root).nest(|| {
        sidebar(state, ui);
        main_content(ui);
    });
}

fn main() {
    let state = State { selected_item: 1 };
    example_window_loop::run_example_loop(state, update_ui);
}
