use crate as keru;
use keru::*;
use keru::node_library::*;

/// A tab container that can be laid out horizontally or vertically.
///
/// The tabs can be identified by any `T: PartialEq + Copy`, such as an enum or a `&'static str`.
/// 
/// Adding the component returns a [`TabContainerResult`] containing `[UiParent]s` for the main content and for each tab in the tab bar, in which arbitrary content can be nested.
///
/// ```no_run
/// use keru::example_window_loop::*;
/// use keru::*;
/// use keru::node_library::*;
///
/// #[derive(Clone, Copy, PartialEq, Eq, Default)]
/// enum MyTab { #[default] Home, Settings }
///
/// #[derive(Default)]
/// struct State {
///     current: MyTab,
/// }
///
/// fn update_ui(state: &mut State, ui: &mut Ui) {
///     let tabs = [MyTab::Home, MyTab::Settings];
///     let result = ui.add_component_with_state(TabContainer::new(&tabs).vertical(), &mut state.current);
///     for (tab, button) in result.labels {
///         let label = match tab { MyTab::Home => "Home", MyTab::Settings => "Settings" };
///         button.nest(|| { ui.add(TEXT.static_text(label)); });
///     }
///     result.content.nest(|| {
///         match result.selected {
///             MyTab::Home => ui.add(LABEL.text("Welcome home.")),
///             MyTab::Settings => ui.add(LABEL.text("Settings.")),
///         };
///     });
/// }
///
/// fn main() {
///     run_example_loop(State::default(), update_ui);
/// }
/// ```
pub struct TabContainer<'a, T: PartialEq + Copy> {
    /// The tabs shown in the tab bar.
    pub tabs: &'a [T],
    /// The horizontal or vertical direction of the tab bar.
    pub axis: Axis,
}

/// The result of adding a [`TabContainer`].
pub struct TabContainerResult<T> {
    /// For each tab in the rab bar, the tab value, and the space in which to nest its content. Normally, an icon, the name of the tab, or both.
    pub labels: Vec<(T, UiParent)>,
    /// The content panel in which to nest the parent of the currently active tab.
    pub content: UiParent,
    /// The currently selected tab.
    pub selected: T,
}

impl<'a, T: PartialEq + Copy> TabContainer<'a, T> {
    pub fn new(tabs: &'a [T]) -> Self {
        Self { tabs, axis: Axis::X }
    }

    /// Set the axis the tab buttons run along. `X` is horizontal, `Y` is vertical.
    pub fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }

    /// Make the tabs vertical.
    pub fn vertical(mut self) -> Self {
        self.axis = Axis::Y;
        self
    }
}

impl<'a, T: PartialEq + Copy> Component for TabContainer<'a, T> {
    type State = T;
    type AddResult = TabContainerResult<T>;
    type ComponentOutput = ();

    fn add_to_ui(&mut self, ui: &mut Ui, selected: &mut T) -> Self::AddResult {
        #[node_key] const TAB_BUTTON: NodeKey;
        #[node_key] const CONTENT_PANEL: NodeKey;

        assert!(!self.tabs.is_empty());

        let vertical = self.axis == Axis::Y;

        // Resolve the selected tab to an index, falling back to the first tab if it isn't in the list.
        let mut current = self.tabs.iter().position(|t| *t == *selected).unwrap_or(0);

        for i in 0..self.tabs.len() {
            if ui.is_clicked(TAB_BUTTON.sibling(i)) {
                current = i;
            }
        }

        let ilen = self.tabs.len() as isize;
        if ui.key_input().key_pressed_or_repeated(&winit::keyboard::Key::Named(winit::keyboard::NamedKey::Tab))
            && ui.key_input().key_mods().control_key()
        {
            let step: isize = if ui.key_input().key_mods().shift_key() { -1 } else { 1 };
            let mut new = current;
            // In the technically unsupported case with multiple undistinguishable tabs, we can still try to skip them rather than remaining stuck on the previous one.
            for _ in 0..self.tabs.len() {
                new = (((new as isize) + step + ilen) % ilen) as usize;
                if self.tabs[new] != self.tabs[current] {
                    break;
                }
            }
            current = new;
            
            ui.update_focus_silently(TAB_BUTTON.sibling(current));
        }

        *selected = self.tabs[current];

        // Horizontal tabs stack the tab bar on top of the content; vertical tabs put it beside.
        let outer = if vertical { H_STACK } else { V_STACK }.size_symm(Size::Fill).stack_spacing(0.0);
        let tabs_stack = if vertical {
            V_STACK.size_x(Size::Pixels(250.0)).size_y(Size::Fill)
        } else {
            H_STACK.size_y(Size::FitContent)
        }.accessibility_role(AccessKitRole::TabList);
        let rounded_corners = if vertical { RoundedCorners::LEFT } else { RoundedCorners::TOP };
        let inactive_tab = BUTTON
            .shape(Shape::Rectangle { rounded_corners, corner_radius: 5.0 })
            .size_x(if vertical { Size::Fill } else { Size::FitContent })
            .size_y(Size::FitContent)
            .fill(ui.theme().muted_background)
            .accessibility_role(AccessKitRole::Tab);
        let active_tab = inactive_tab.fill(ui.theme().background);
        let content_panel = PANEL
            .size_symm(Size::Fill)
            .fill(ui.theme().background)
            .children_can_hide(true)
            .accessibility_role(AccessKitRole::TabPanel)
            .key(CONTENT_PANEL);

        let mut labels = Vec::with_capacity(self.tabs.len());

        let content = ui.add(outer).nest(|| {
            ui.add(tabs_stack).nest(|| {
                for i in 0..self.tabs.len() {
                    let active = i == current;
                    let tab = if active { active_tab } else { inactive_tab };
                    let tab = tab.accessibility_selected(active).key(TAB_BUTTON.sibling(i));
                    labels.push((self.tabs[i], ui.add(tab)));
                }
            });

            ui.add(content_panel)
        });

        TabContainerResult { labels, content, selected: *selected }
    }
}
