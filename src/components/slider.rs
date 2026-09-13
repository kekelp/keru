use crate as keru;
use keru::*;
use keru::node_library::*;
use keru::Size::*;
use keru::Pos::*;

/// A slider for a `f32` value.
///
/// ```no_run
/// use keru::*;
/// use keru::node_library::*;
/// use keru::example_window_loop::*;
///
/// #[derive(Default)]
/// struct State {
///     volume: f32,
/// }
///
/// fn update_ui(state: &mut State, ui: &mut Ui) {
///     ui.add(V_STACK).nest(|| {
///         // Horizontal by default; call `.vertical()` for a vertical slider.
///         ui.add_component_with_state(Slider::new(0.0, 100.0, true), &mut state.volume);
///     });
/// }
///
/// fn main() {
///     run_example_loop(State::default(), update_ui);
/// }
/// ```
pub struct Slider {
    pub min: f32,
    pub max: f32,
    pub clamp: bool, // todo: with clamp = false, still clamp values set WITH the slider
    pub axis: Axis, // The axis the slider runs along. `X` is horizontal, `Y` is vertical.
}

impl Slider {
    pub fn new(min: f32, max: f32, clamp: bool) -> Self {
        Self { min, max, clamp, axis: Axis::X }
    }

    /// Set the axis the slider runs along
    pub fn axis(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }

    /// Make the slider vertical.
    pub fn vertical(mut self) -> Self {
        self.axis = Axis::Y;
        self
    }
}

impl Component for Slider {
    type AddResult = ();
    type ComponentOutput = ();
    type State = f32;

    fn add_to_ui(&mut self, ui: &mut Ui, value: &mut f32) {
        with_arena(|a| {

            #[node_key] const SLIDER_FILL: NodeKey;
            #[node_key] const SLIDER_LABEL: NodeKey;
            #[node_key] const SLIDER_CONTAINER: NodeKey;

            let vertical = self.axis == Axis::Y;

            let mut new_value = *value;
            if let Some(drag) = ui.is_dragged(SLIDER_CONTAINER) {
                if vertical {
                    new_value -= drag.relative_delta.y as f32 * (self.max - self.min);
                } else {
                    new_value += drag.relative_delta.x as f32 * (self.max - self.min);
                }
            }

            // Arrow keys do a "drag" when the slider is focused.
            if ui.has_visible_keyboard_focus(SLIDER_CONTAINER) {
                let step = (self.max - self.min) * 0.01;
                let (decrement_key, increment_key) = if vertical {
                    (winit::keyboard::NamedKey::ArrowDown, winit::keyboard::NamedKey::ArrowUp)
                } else {
                    (winit::keyboard::NamedKey::ArrowLeft, winit::keyboard::NamedKey::ArrowRight)
                };
                if ui.key_input().key_pressed_or_repeated(&winit::keyboard::Key::Named(decrement_key)) {
                    new_value -= step;
                }
                if ui.key_input().key_pressed_or_repeated(&winit::keyboard::Key::Named(increment_key)) {
                    new_value += step;
                }
            }

            if new_value.is_finite() {
                if self.clamp {
                    new_value = new_value.clamp(self.min, self.max);
                }
                *value = new_value;
            }

            let filled_frac = (*value - self.min) / (self.max - self.min);

            let base_container = PANEL
                .sense_drag(true)
                .focusable(true)
                .accessibility_role(AccessKitRole::Slider)
                .accessibility_numeric_value(*value as f64, self.min as f64, self.max as f64)
                .accessibility_actions(AccessibilityActions::INCREMENT | AccessibilityActions::DECREMENT)
                .shape(Shape::Rectangle { corner_radius: 14.0, rounded_corners: RoundedCorners::ALL })
                .key(SLIDER_CONTAINER);

            let base_fill = PANEL
                .color(Color::KERU_RED)
                .absorbs_clicks(false)
                .shape(Shape::Rectangle { corner_radius: 9.0, rounded_corners: RoundedCorners::ALL })
                .key(SLIDER_FILL);

            let (slider_container, slider_fill);
            if vertical {
                slider_container = base_container
                    .size_y(Size::Fill)
                    .size_x(Size::Pixels(45.0));
                slider_fill = base_fill
                    .size_x(Fill)
                    .size_y(Size::Frac(filled_frac))
                    .position_y(End)
                    .padding_y(1.0);
            } else {
                slider_container = base_container
                    .size_x(Size::Fill)
                    .size_y(Size::Pixels(45.0));
                slider_fill = base_fill
                    .size_y(Fill)
                    .size_x(Size::Frac(filled_frac))
                    .position_x(Start)
                    .padding_x(1.0);
            };

            let text = bumpalo::format!(in a, "{:.2}", value);
            let label = TEXT.text(&text).key(SLIDER_LABEL);

            ui.add(slider_container).nest(|| {
                ui.add(slider_fill);
                ui.add(label);
            });

        });
    }
}

/// A classic track-and-handle slider for a `f32` value. Always horizontal.
///
/// ```no_run
/// use keru::*;
/// use keru::node_library::*;
/// use keru::example_window_loop::*;
///
/// #[derive(Default)]
/// struct State {
///     volume: f32,
/// }
///
/// fn update_ui(state: &mut State, ui: &mut Ui) {
///     ui.add(V_STACK).nest(|| {
///         ui.add_component_with_state(ClassicSlider::new(0.0, 100.0), &mut state.volume);
///     });
/// }
///
/// fn main() {
///     run_example_loop(State::default(), update_ui);
/// }
/// ```
pub struct ClassicSlider {
    pub min: f32,
    pub max: f32,
}

impl ClassicSlider {
    pub fn new(min: f32, max: f32) -> Self {
        Self { min, max }
    }
}

impl Component for ClassicSlider {
    type AddResult = ();
    type ComponentOutput = ();
    type State = f32;

    fn add_to_ui(&mut self, ui: &mut Ui, value: &mut f32) {
        #[node_key] const TRACK: NodeKey;
        #[node_key] const FILLED: NodeKey;
        #[node_key] const HANDLE: NodeKey;
        #[node_key] const SLIDER_CONTAINER: NodeKey;
        #[node_key] const HITBOX: NodeKey;

        // todo: combined with the handle's manual positioning, the handle is drawn at zero on the first frame, and relies on the anti-state-tearing stuff to not stay there. Fix by making it possible to express the " - handle_radius" part when using a Frac.
        let slider_width = match ui.get_node(TRACK) {
            Some(track) => track.inner_size().x,
            None => 1.0, // Only used on the first frame, before the track has a size.
        };

        let handle_radius = 10.0;

        if let Some(click) = ui.clicked_at(HITBOX) {
            *value = self.min + click.relative_position.x as f32 * self.max;
        }
        if let Some(drag) = ui.is_dragged(HITBOX) {
            *value = self.min + drag.relative_position.x as f32 * self.max;
        }

        // Arrow keys do a "drag" when the slider is focused.
        if ui.has_visible_keyboard_focus(HITBOX) {
            let step = (self.max - self.min) * 0.01;
            if ui.key_input().key_pressed_or_repeated(&winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowLeft)) {
                *value -= step;
            }
            if ui.key_input().key_pressed_or_repeated(&winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowRight)) {
                *value += step;
            }
        }

        *value = value.clamp(self.min, self.max);

        let handle_position_frac = (*value - self.min) / (self.max - self.min);

        let slider_track = PANEL
            .size_x(Size::Fill)
            .size_y(Size::Pixels(10.0))
            .padding(0.0)
            .color(Color::GREY)
            .shape(Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: 5.0 })
            .absorbs_clicks(false)
            .key(TRACK);

        let slider_filled = PANEL
            .size_y(Size::Pixels(14.0))
            .shape(Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: 7.0 })
            .size_x(Size::Frac(handle_position_frac))
            .color(Color::KERU_RED)
            .position_x(Start)
            .padding_x(0.0)
            .absorbs_clicks(false)
            .key(FILLED);

        let slider_handle = PANEL
            .size_x(Size::Pixels(handle_radius * 2.0))
            .size_y(Size::Pixels(handle_radius * 2.0))
            .color(Color::WHITE)
            .anchor_x(Anchor::Center)
            .position_x(Pos::Pixels(handle_position_frac * slider_width))
            .position_y(Pos::Center)
            .shape(Shape::Circle)
            .padding_x(0.0)
            .absorbs_clicks(false)
            .key(HANDLE);

        let slider_container = CONTAINER
            .size_x(Size::Fill)
            .size_y(Size::Pixels(45.0))
            .padding_x(0.0)
            .key(SLIDER_CONTAINER);

        let hitbox = CONTAINER
            .size_x(Size::Fill)
            .size_y(Size::Pixels(30.0))
            .sense_click(true)
            .sense_drag(true)
            .focusable(true)
            .accessibility_role(AccessKitRole::Slider)
            .accessibility_numeric_value(*value as f64, self.min as f64, self.max as f64)
            .accessibility_actions(AccessibilityActions::INCREMENT | AccessibilityActions::DECREMENT)
            .padding(0.0)
            .key(HITBOX);

        ui.add(slider_container).nest(|| {
            ui.add(hitbox).nest(|| {
                ui.add(slider_track).nest(|| {
                    ui.add(slider_filled);
                    ui.add(slider_handle);
                });
            });
        });
    }
}
