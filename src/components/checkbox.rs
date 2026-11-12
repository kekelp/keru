use crate as keru;
use keru::*;
use keru::node_library::*;

/// A checkbox for a `bool` value with an optional text label. Clicking the label also toggles the value.
pub struct Checkbox<'a> {
    pub label: Option<&'a str>,
}

impl<'a> Checkbox<'a> {
    pub fn new() -> Self {
        Self { label: None }
    }

    /// Show a text label next to the checkbox.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }
}

impl<'a> Component for Checkbox<'a> {
    /// `true` if the value changed this frame.
    type AddResult = bool;
    type ComponentOutput = ();
    type State = bool;

    fn add_to_ui(&mut self, ui: &mut Ui, checked: &mut bool) -> bool {
        #[node_key] const CHECKBOX_ROW: NodeKey;
        #[node_key] const CHECKBOX_BOX: NodeKey;
        #[node_key] const CHECKBOX_MARK: NodeKey;
        #[node_key] const CHECKBOX_LABEL: NodeKey;

        let changed = ui.is_clicked(CHECKBOX_ROW);
        if changed {
            *checked = !*checked;
        }

        let theme = ui.theme();
        let box_fill = if *checked { theme.primary } else { theme.surface_alt };

        let row = H_STACK
            .sense_click(true)
            .focusable(true)
            .hover_cursor_icon(CursorIcon::Pointer)
            .accessibility_role(AccessKitRole::CheckBox)
            .accessibility_toggled(*checked)
            .stack_spacing(8.0)
            .padding(2.0)
            .key(CHECKBOX_ROW);

        let check_box = PANEL
            .size_symm(Size::Pixels(24.0))
            .padding(4.0)
            .fill(box_fill)
            .stroke(1.5)
            .stroke_fill(theme.border)
            .shape(Shape::Rectangle { corner_radius: 5.0, rounded_corners: RoundedCorners::ALL })
            .absorbs_clicks(false)
            .key(CHECKBOX_BOX);

        let mark = ICON
            .static_svg(ICON_CHECK)
            .size_symm(Size::Fill)
            .absorbs_clicks(false)
            .key(CHECKBOX_MARK);

        ui.add(row).nest(|| {
            ui.add(check_box).nest(|| {
                if *checked {
                    ui.add(mark);
                }
            });
            if let Some(label) = self.label {
                ui.add(TEXT.text(label).key(CHECKBOX_LABEL));
            }
        });

        changed
    }
}
