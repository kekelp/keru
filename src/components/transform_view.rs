use crate as keru;
use keru::*;
use keru::node_library::*;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TransformViewState {
    pub scale: f32,
    pub pan_x: f32,
    pub pan_y: f32,
    #[cfg_attr(feature = "serde", serde(skip))]
    pub zoom_drag_anchor: Option<glam::Vec2>,
}
impl Default for TransformViewState {
    fn default() -> Self {
        Self {
            scale: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            zoom_drag_anchor: None,
        }
    }
}

pub struct TransformView;

impl Component for TransformView {
    type AddResult = UiParent;
    type ComponentOutput = ();
    type State = TransformViewState;

    fn add_to_ui(&mut self, ui: &mut Ui, state: &mut Self::State) -> Self::AddResult {
        use glam::{Vec2, vec2};
        use winit::event::MouseButton;
        use winit::keyboard::{Key, NamedKey};

        #[node_key] const PAN_OVERLAY: NodeKey;
        #[node_key] const SPACEBAR_PAN_OVERLAY: NodeKey;
        #[node_key] const TRANSFORMED_AREA: NodeKey;

        let spacebar_pan_overlay = PANEL
            .padding(0.0)
            .color(Color::TRANSPARENT)
            .sense_drag(true)
            .size(Size::Fill, Size::Fill)
            .key(SPACEBAR_PAN_OVERLAY);

        let pan_overlay = PANEL
            .padding(0.0)
            .color(Color::TRANSPARENT)
            .sense_drag(true)
            .sense_scroll(true)
            .absorbs_clicks(false)
            .size(Size::Fill, Size::Fill)
            .key(PAN_OVERLAY);

        let transform_area = PANEL
            .padding(0.0)
            .size_symm(Size::Pixels(1000000.0))
            .color(Color::TRANSPARENT)
            .key(TRANSFORMED_AREA)
            .translate(state.pan_x, state.pan_y)
            .scale(state.scale)
            .size_symm(Size::Fill)
            .clip_children_x(true)
            .clip_children_y(true);

        let parent = ui.add(transform_area);

        if ui.key_input().key_held(&Key::Named(NamedKey::Space)) {
            ui.add(spacebar_pan_overlay);
        }

        ui.add(pan_overlay);

        let size = ui.get_node(TRANSFORMED_AREA).map(|x| x.inner_size()).unwrap_or(Xy::new(600.0, 600.0));

        // Handle panning
        if ! ui.key_input().key_held(&Key::Named(NamedKey::Space)) {
            if let Some(drag) = ui.is_mouse_button_dragged(PAN_OVERLAY, MouseButton::Middle) {
                state.pan_x += drag.absolute_delta.x as f32;
                state.pan_y += drag.absolute_delta.y as f32;
            }
        }

        if let Some(drag) = ui.is_dragged(SPACEBAR_PAN_OVERLAY) {
            state.pan_x += drag.absolute_delta.x as f32;
            state.pan_y += drag.absolute_delta.y as f32;
        }

        // Handle zooming
        let mut apply_zoom = |delta_y: f32, mouse_pos: Vec2| {
            let old_zoom = state.scale;
            let curve_factor = ((0.01 + old_zoom).powf(1.1) - 0.01).abs();
            let new_zoom = old_zoom + delta_y as f32 * curve_factor;

            if new_zoom > 0.01 && !new_zoom.is_infinite() && !new_zoom.is_nan() {
                state.scale = new_zoom;
                let zoom_ratio = state.scale / old_zoom;
                let centered_pos = mouse_pos - vec2(0.5, 0.5);
                state.pan_x = state.pan_x * zoom_ratio + size.x as f32 * centered_pos.x as f32 * (1.0 - zoom_ratio);
                state.pan_y = state.pan_y * zoom_ratio + size.y as f32 * centered_pos.y as f32 * (1.0 - zoom_ratio);
            }
        };

        if let Some(drag) = ui.is_mouse_button_dragged(SPACEBAR_PAN_OVERLAY, MouseButton::Middle) {
            if state.zoom_drag_anchor.is_none() {
                state.zoom_drag_anchor = Some(drag.relative_position);
            }

            apply_zoom(drag.absolute_delta.y * 0.01, state.zoom_drag_anchor.unwrap());

        } else {
            state.zoom_drag_anchor = None;
        }

        if let Some(scroll_event) = ui.scrolled_at_animated(PAN_OVERLAY) {
            if ui.key_input().key_mods().control_key() {
                apply_zoom(scroll_event.delta.y, scroll_event.relative_position);
            } else {
                const SCROLL_PAN_SPEED: f32 = 400.0;
                state.pan_x += scroll_event.delta.x * SCROLL_PAN_SPEED;
                state.pan_y += scroll_event.delta.y * SCROLL_PAN_SPEED;
            }
        }

        return parent;
    }
}
