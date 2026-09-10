// An example of the [`TabContainer`] component.
use keru::*;
use keru::node_library::*;
use keru::example_window_loop::*;

#[derive(Clone, Copy, PartialEq)]
enum MyTab {
    Home,
    Settings,
    About,
}

const TABS: &[MyTab] = &[MyTab::Home, MyTab::Settings, MyTab::About];

struct State {
    selected: MyTab,
    vertical: bool,
}

fn update_ui(state: &mut State, ui: &mut Ui) {
    #[node_key] const TOGGLE: NodeKey;

    if ui.is_clicked(TOGGLE) {
        state.vertical = !state.vertical;
    }

    let toggle_label = if state.vertical { "Switch to horizontal" } else { "Switch to vertical" };
    let toggle = BUTTON.static_text(toggle_label).key(TOGGLE);

    let tabs = TabContainer::new(TABS).axis(if state.vertical { Axis::Y } else { Axis::X });
    let result = ui.add_component_with_state(tabs, &mut state.selected);

    // Nest arbitrary content in the tab labels. Here it's just a string, but could be an icon, a notification counter, etc.
    for (tab, tab_space) in result.labels {
        tab_space.nest(|| {
            match tab {
                MyTab::Home => ui.add(TEXT.static_text("Home")),
                MyTab::Settings => ui.add(TEXT.static_text("Settings")),
                MyTab::About => ui.add(TEXT.static_text("About")),
            }
        });
    }

    // Fill the content panel based on the selected tab.
    result.content.nest(|| {
        match result.selected {
            MyTab::Home => ui.add(V_STACK).nest(|| {
                ui.add(TEXT.text("Home Tab"));
                ui.add(toggle);
            }),
            MyTab::Settings => ui.add(V_STACK).nest(|| {
                ui.add(TEXT.text("Settings Tab"));
                ui.add(toggle);
            }),
            MyTab::About => ui.add(V_STACK).nest(|| {
                ui.add(TEXT.text("About Tab"));
                ui.add(toggle);
            }),
        };
    });
}

fn main() {
    basic_env_logger_init();
    let state = State { selected: MyTab::Home, vertical: false };
    run_example_loop(state, update_ui);
}
