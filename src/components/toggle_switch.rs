use crate as keru;
use keru::*;
use keru::node_library::*;

/// A toggle switch for a `bool` value with an optional text label. Clicking the label also toggles the value.
pub struct ToggleSwitch<'a> {
    pub label: Option<&'a str>,
    /// Track height in pixels. The width, corner radius and knob size follow from it.
    pub height: f32,
}

impl<'a> ToggleSwitch<'a> {
    pub fn new() -> Self {
        Self { label: None, height: 26.0 }
    }

    /// Show a text label next to the switch.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }

    /// Set the track height in pixels. The width, corner radius and knob size follow from it.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}

impl<'a> Component for ToggleSwitch<'a> {
    /// `true` if the value changed this frame.
    type AddResult = bool;
    type ComponentOutput = ();
    type State = bool;

    fn add_to_ui(&mut self, ui: &mut Ui, on: &mut bool) -> bool {
        #[node_key] const SWITCH_ROW: NodeKey;
        #[node_key] const SWITCH_TRACK: NodeKey;
        #[node_key] const SWITCH_KNOB: NodeKey;
        #[node_key] const SWITCH_LABEL: NodeKey;

        let track_height = self.height;
        let track_width = track_height * 1.75;
        let knob_padding = track_height * (5.0 / 26.0);
        let knob_size = track_height - 2.0 * knob_padding;

        let changed = ui.is_clicked(SWITCH_ROW);
        if changed {
            *on = !*on;
        }

        let theme = ui.theme();
        let track_fill = if *on { theme.primary } else { theme.surface_alt };
        let track_stroke = theme.border;

        let row = H_STACK
            .sense_click(true)
            .focusable(true)
            .hover_cursor_icon(CursorIcon::Pointer)
            .accessibility_role(AccessKitRole::Switch)
            .accessibility_toggled(*on)
            .stack_spacing(8.0)
            .padding(2.0)
            .key(SWITCH_ROW);

        let track = PANEL
            .size_x(Size::Pixels(track_width))
            .size_y(Size::Pixels(track_height))
            .padding(knob_padding)
            .fill(track_fill)
            .stroke(1.5)
            .stroke_fill(track_stroke)
            .shape(Shape::Rectangle { corner_radius: track_height / 2.0, rounded_corners: RoundedCorners::ALL })
            .animate_properties(true)
            .absorbs_clicks(false)
            .key(SWITCH_TRACK);

        let knob = PANEL
            .size_symm(Size::Pixels(knob_size))
            .padding(0.0)
            .color(Color::WHITE)
            .shape(Shape::Circle)
            .position_x(if *on { Pos::End } else { Pos::Start })
            .position_y(Pos::Center)
            .animate_layout(true)
            .absorbs_clicks(false)
            .key(SWITCH_KNOB);

        ui.add(row).nest(|| {
            ui.add(track).nest(|| {
                ui.add(knob);
            });
            if let Some(label) = self.label {
                ui.add(TEXT.text(label).key(SWITCH_LABEL));
            }
        });

        changed
    }
}
