use crate as keru;
use keru::*;
use keru::node_library::*;

/// A text edit box bound to a value of type `T` that can also change on its own.
///
/// While the user is editing the box, the text dictates the value. Otherwise, the value dictates the text.
///
/// Provide two closures: one to format a `T` into text, and one to parse text back into a `T`. The parse closure returns `None` for invalid text, which leaves the value unchanged.
///
/// To bind an optional value, use `Option<T>` as the state and map the empty string to `None`. Combine this with [`LiveEditBox::node()`] and a placeholder to show a fallback when the box is empty.
///
/// ```no_run
/// use keru::*;
/// use keru::example_window_loop::*;
///
/// #[derive(Default)]
/// struct State {
///     value: f32,
/// }
///
/// fn update_ui(state: &mut State, ui: &mut Ui) {
///     let edit_box = LiveEditBox::new(
///         |v: &f32| format!("{:.2}", v),
///         |text: &str| text.trim().parse().ok(),
///     );
///     ui.add_component_with_state(edit_box, &mut state.value);
/// }
///
/// fn main() {
///     run_example_loop(State::default(), update_ui);
/// }
/// ```
pub struct LiveEditBox<'a, ToText, FromText> {
    node: Node<'a>,
    to_text: ToText,
    from_text: FromText,
}

impl<'a, ToText, FromText> LiveEditBox<'a, ToText, FromText> {
    /// Create a new [`LiveEditBox`] from a value-to-text closure and a text-to-value closure.
    pub fn new(to_text: ToText, from_text: FromText) -> Self {
        Self { node: TEXT_EDIT_LINE, to_text, from_text }
    }

    /// Set the base [`Node`] for the text edit box, to customize its size, placeholder text, style, etc.
    pub fn base_node(mut self, node: Node<'a>) -> Self {
        self.node = node;
        self
    }
}

impl<'a, T, Str, ToText, FromText> Component for LiveEditBox<'a, ToText, FromText>
where
    Str: AsRef<str>,
    ToText: Fn(&T) -> Str,
    FromText: Fn(&str) -> Option<T>,
{
    type AddResult = ();
    type ComponentOutput = ();
    type State = T;

    fn add_to_ui(&mut self, ui: &mut Ui, value: &mut T) {
        #[node_key] const EDIT: NodeKey;

        let editing = ui.is_text_edit_focused(EDIT);

        if editing {
            let enter = ui.key_input().key_pressed(&winit::keyboard::Key::Named(winit::keyboard::NamedKey::Enter));
            let new_text = ui.get_node(EDIT).and_then(|n| if enter { n.get_text() } else { n.text_edit_changed() });
            if let Some(new_text) = new_text {
                if let Some(new_value) = (self.from_text)(new_text) {
                    *value = new_value;
                }
            }
        }

        let text = (self.to_text)(value);
        let text = text.as_ref();
        ui.add(self.node.key(EDIT).text(text));

        if ! editing {
            if let Some(node) = ui.get_node_mut(EDIT) {
                node.set_text(text);
            }
        }
    }
}
