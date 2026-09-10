// An example of the [`Accordion`] component.
use keru::*;
use keru::node_library::*;
use keru::example_window_loop::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    Account,
    Notifications,
    Privacy,
}

const SECTIONS: &[Section] = &[Section::Account, Section::Notifications, Section::Privacy];

impl Section {
    fn title(self) -> &'static str {
        match self {
            Section::Account => "Account",
            Section::Notifications => "Notifications",
            Section::Privacy => "Privacy",
        }
    }

    fn body(self) -> &'static str {
        match self {
            Section::Account => "Manage your account details and sign-in options.",
            Section::Notifications => "Choose which events send you a notification.",
            Section::Privacy => "Control who can see your activity.",
        }
    }
}

#[derive(Default)]
struct State {
    expanded: Vec<bool>,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    let panel = V_STACK
        .size_x(Size::Pixels(400.0))
        .size_y(Size::Fill)
        .stack_arrange(Arrange::Start);

    ui.add(panel).nest(|| {
        let result = ui.add_component_with_state(Accordion::new(SECTIONS), &mut state.expanded);

        for section in result.sections {
            // The header can hold arbitrary content. Here it's just a label, but it could be an icon, a badge, etc.
            section.header.nest(|| {
                ui.add(TEXT.static_text(section.value.title()));
            });

            if let Some(body) = section.body {
                body.nest(|| {
                    ui.add(TEXT_PARAGRAPH.static_text(section.value.body()));
                });
            }
        }
    });
}

fn main() {
    basic_env_logger_init();
    run_example_loop(State::default(), update_ui);
}
