use std::{fmt::Debug, hash::{Hash, Hasher}, marker::PhantomData};

use crate as keru;
use keru::*;
use keru::Size::*;
use keru::Position::*;

#[derive(Debug)]
pub struct ComponentKey<ComponentType: ?Sized> {
    id: Id,
    debug_name: &'static str,
    phantom: PhantomData<ComponentType>
}
impl<C> ComponentKey<C> {
    /// Create "siblings" of a key dynamically at runtime, based on a hashable value.
    pub fn sibling<H: Hash>(self, value: H) -> Self {
        let mut hasher = ahasher();
        self.id.0.hash(&mut hasher);
        value.hash(&mut hasher);
        let new_id = hasher.finish();

        return Self {
            id: Id(new_id),
            debug_name: self.debug_name,
            phantom: PhantomData::<C>,
        };
    }

    /// Create a key manually.
    /// 
    /// This is usually not needed: use the [`macro@component_key`] macro for static keys, and [`ComponentKey::sibling`] for dynamic keys.
    pub const fn new(id: Id, debug_name: &'static str) -> Self {
        return Self {
            id,
            debug_name,
            phantom: PhantomData::<C>
        };
    }

    pub const fn debug_name(&self) -> &'static str {
        return self.debug_name;
    }

    // Private function that removes the type marker.
    pub(crate) fn as_normal_key(&self) -> NodeKey {
        NodeKey::new(self.id, self.debug_name)
    }
}

// The key should be Copy even if the component params struct (C) isn't. Because of how derive(C) works, this needs to be impl'd manually.
impl<C> Clone for ComponentKey<C> {
    fn clone(&self) -> Self {
        Self { id: self.id, debug_name: self.debug_name, phantom: self.phantom }
    }
}
impl<C> Copy for ComponentKey<C> {}



impl Ui {
    #[track_caller]
    pub fn add_component<T: Component2>(&mut self, component_params: T) -> T::AddResult {
        self.add_stateful_component(component_params)
    }

    pub fn component_output<T: Component2>(&mut self, component_key: ComponentKey<T>) -> Option<T::ComponentOutput> {
        self.stateful_component_output(component_key)
    }
}

pub struct Slider<'a> {
    pub value: &'a mut f32,
    pub min: f32,
    pub max: f32,
    pub clamp: bool, // todo: with clamp = false, still clamp values set WITH the slider
}


impl Component for Slider<'_> {
    fn add_to_ui(self, ui: &mut Ui) {
        with_arena(|arena| {

            #[node_key] const SLIDER_FILL: NodeKey;
            #[node_key] const SLIDER_LABEL: NodeKey;
            #[node_key] const SLIDER_CONTAINER: NodeKey;
                
            let mut new_value = *self.value;
            if let Some(drag) = ui.is_dragged(SLIDER_CONTAINER) {
                new_value += drag.relative_delta.x as f32 * (self.min - self.max);
            }

            if new_value.is_finite() {
                if self.clamp {
                    new_value = new_value.clamp(self.min, self.max);
                }
                *self.value = new_value;
            }

            let filled_frac = (*self.value - self.min) / (self.max - self.min);

            let slider_container = PANEL
                .size_x(Size::Fill)
                .size_y(Size::Pixels(45))
                .sense_drag(true)
                // .shape(Shape::Rectangle { corner_radius: 36.0 })
                .key(SLIDER_CONTAINER);
            
            let slider_fill = PANEL
                .size_y(Fill)
                .size_x(Size::Frac(filled_frac))
                .color(Color::KERU_RED)
                .position_x(Start)
                .padding_x(1)
                .absorbs_clicks(false)
                // .shape(Shape::Rectangle { corner_radius: 16.0 })
                .key(SLIDER_FILL);


            let text = bumpalo::format!(in arena, "{:.2}", self.value);
            let label = TEXT.text(&text).key(SLIDER_LABEL);

            ui.add(slider_container).nest(|| {
                ui.add(slider_fill);
                ui.add(label);
            });

        });
    }
}

impl<'a> Slider<'a> {
    pub fn new(value: &'a mut f32, min: f32, max: f32, clamp: bool) -> Self {
        Self { value, min, max, clamp }
    }
}

#[derive(Default)]
pub struct TransformViewState {
    pub zoom: f32,
    pub pan_x: f32,
    pub pan_y: f32,
    pub zoom_drag_anchor: Option<glam::DVec2>,
}

pub struct TransformView<'a, F> {
    pub state: &'a mut TransformViewState,
    pub content: F,
}

impl<'a, F> TransformView<'a, F>
where
    F: FnOnce(&mut Ui),
{
    pub fn new(state: &'a mut TransformViewState, content: F) -> Self {
        Self { state, content }
    }
}

impl<F> Component for TransformView<'_, F>
where
    F: FnOnce(&mut Ui),
{
    fn add_to_ui(self, ui: &mut Ui) {
        use glam::{DVec2, dvec2};
        use winit::event::MouseButton;
        use winit::keyboard::{Key, NamedKey};

        #[node_key] const PAN_OVERLAY: NodeKey;
        #[node_key] const SPACEBAR_PAN_OVERLAY: NodeKey;
        #[node_key] const TRANSFORMED_AREA: NodeKey;

        let spacebar_pan_overlay = PANEL
            .padding(0)
            .color(Color::TRANSPARENT)
            .sense_drag(true)
            .size(Size::Fill, Size::Fill)
            .key(SPACEBAR_PAN_OVERLAY);

        let pan_overlay = PANEL
            .padding(0)
            .color(Color::TRANSPARENT)
            .sense_drag(true)
            .absorbs_clicks(false)
            .size(Size::Fill, Size::Fill)
            .key(PAN_OVERLAY);

        let transform_area = PANEL
            .size_symm(Size::Pixels(1000000))
            .color(Color::rgba(30, 30, 40, 255))
            .key(TRANSFORMED_AREA)
            .translate(self.state.pan_x, self.state.pan_y)
            .zoom(self.state.zoom)
            .size_symm(Size::Fill)
            .clip_children(true);

        ui.add(transform_area).nest(|| {
            (self.content)(ui);
        });

        if ui.key_input().key_held(&Key::Named(NamedKey::Space)) {
            ui.add(spacebar_pan_overlay);
        }

        ui.add(pan_overlay);

        let size = ui.inner_size(TRANSFORMED_AREA).unwrap_or(Xy::new(600, 600));

        // Handle panning
        if ! ui.key_input().key_held(&Key::Named(NamedKey::Space)) {
            if let Some(drag) = ui.is_mouse_button_dragged(PAN_OVERLAY, MouseButton::Middle) {
                self.state.pan_x -= drag.absolute_delta.x as f32;
                self.state.pan_y -= drag.absolute_delta.y as f32;
            }
        }

        if let Some(drag) = ui.is_dragged(SPACEBAR_PAN_OVERLAY) {
            self.state.pan_x -= drag.absolute_delta.x as f32;
            self.state.pan_y -= drag.absolute_delta.y as f32;
        }

        // Handle zooming
        let mut apply_zoom = |delta_y: f64, mouse_pos: DVec2| {
            let old_zoom = self.state.zoom;
            let curve_factor = ((0.01 + old_zoom).powf(1.1) - 0.01).abs();
            let new_zoom = old_zoom + delta_y as f32 * curve_factor;

            if new_zoom > 0.01 && !new_zoom.is_infinite() && !new_zoom.is_nan() {
                self.state.zoom = new_zoom;
                let zoom_ratio = self.state.zoom / old_zoom;
                let centered_pos = mouse_pos - dvec2(0.5, 0.5);
                self.state.pan_x = self.state.pan_x * zoom_ratio + size.x as f32 * centered_pos.x as f32 * (1.0 - zoom_ratio);
                self.state.pan_y = self.state.pan_y * zoom_ratio + size.y as f32 * centered_pos.y as f32 * (1.0 - zoom_ratio);
            }
        };

        if let Some(drag) = ui.is_mouse_button_dragged(SPACEBAR_PAN_OVERLAY, MouseButton::Middle) {
            if self.state.zoom_drag_anchor.is_none() {
                self.state.zoom_drag_anchor = Some(drag.relative_position);
            }

            apply_zoom(drag.absolute_delta.y * 0.01, self.state.zoom_drag_anchor.unwrap());

        } else {
            self.state.zoom_drag_anchor = None;
        }

        if let Some(scroll_event) = ui.scrolled_at(PAN_OVERLAY) {
            apply_zoom(scroll_event.delta.y, scroll_event.relative_position);
        }
    }
}

use std::cell::RefCell;
use bumpalo::Bump;

thread_local! {
    /// Thread local bump arena for temporary allocations
    static THREAD_ARENA: RefCell<Bump> = RefCell::new(Bump::new());
}

/// Access keru's thread-local bump arena for temporary allocations.
/// Useful for small local allocations without passing an arena around, like formatting strings to show in the gui.
///
/// The arena is reset at the end of each frame, when [`Ui::finish_frame()`] is called.
/// 
/// This function is useful when implementing a reusable component with the [`Component`] traits, since you can't easily access all of your state from within the trait impl. In other cases, it might be more convenient to use your own arena.
///
/// # Panics
/// Panics if [`Ui::finish_frame()`] is called from inside the passes closure.
///
/// # Example
/// ```no_run
/// # use keru::*;
/// # let mut ui: Ui = unimplemented!();
/// # let float_value = 6.7;
/// with_arena(|a| {
///     let text = bumpalo::format!(in a, "{:.2}", float_value);
///     ui.add(LABEL.text(&text)); // Great
///     // ui.finish_frame(); // Don't do this.
/// });
/// 
/// ui.finish_frame(); // Now it's fine.
/// ```
pub fn with_arena<F, R>(f: F) -> R
where
    F: FnOnce(&Bump) -> R,
{
    THREAD_ARENA.with(|arena| {
        f(&arena.borrow())
    })
}

pub(crate) fn reset_arena() {
    THREAD_ARENA.with(|arena| {
        arena.borrow_mut().reset();
    });
}
