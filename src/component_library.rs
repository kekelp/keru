use crate as keru;
use keru::*;
use keru::Size::*;
use keru::Pos::*;

/// A tab for [`Ui::vertical_tabs`]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Tab(pub &'static str);

impl Ui {
    /// Add a panel.
    #[track_caller]
    pub fn panel(&mut self) -> UiParent {
        self.add(PANEL)
    }

    /// Add a vertical stack container.
    #[track_caller]
    pub fn v_stack(&mut self) -> UiParent {
        self.add(V_STACK)
    }

    /// Add a spacer.
    #[track_caller]
    pub fn spacer(&mut self) -> UiParent {
        self.add(SPACER)
    }
    
    /// Add a horizontal stack container.
    #[track_caller]
    pub fn h_stack(&mut self) -> UiParent {
        self.add(H_STACK)
    }

    /// Add a multiline text paragraph from a `'static str`.
    #[track_caller]
    pub fn text_edit(&mut self, text: &'static str) -> UiParent {
        let params = TEXT_EDIT.static_text(text);
        self.add(params)
    }

    /// Add a single-line text element.
    #[track_caller]
    pub fn text_line(&mut self, text: &(impl MaybeObservedText + ?Sized)) -> UiParent {
        let params = TEXT.text(text);
        self.add(params)
    }

    /// Add a single-line text element from a `'static str`.
    #[track_caller]
    pub fn static_text_line(&mut self, text: &'static str) -> UiParent {
        let params = TEXT.static_text(text);
        self.add(params)
    }

    /// Add a multiline text paragraph.
    #[track_caller]
    pub fn paragraph(&mut self, text: &(impl MaybeObservedText + ?Sized)) -> UiParent {
        let params = TEXT_PARAGRAPH.text(text);
        self.add(params)
    }

    /// Add a multiline text paragraph from a `'static str`.
    #[track_caller]
    pub fn static_paragraph(&mut self, text: &'static str) -> UiParent {
        let params = TEXT_PARAGRAPH.static_text(text);
        self.add(params)
    }

    /// Add a label.
    #[track_caller]
    pub fn label(&mut self, text: &(impl MaybeObservedText + ?Sized)) -> UiParent {
        let params = LABEL.text(text);
        self.add(params)
    }

    /// Add a label from a `&static str`.
    #[track_caller]
    pub fn static_label(&mut self, text: &'static str) -> UiParent {
        let params = LABEL.static_text(text);
        self.add(params)
    }

    /// Add a vertical tabs container
    #[track_caller]
    pub fn vertical_tabs(&mut self, tabs: &[Tab], current_tab: &mut usize) -> UiParent {
        #[node_key] const VERTICAL_TABS_TAB_BUTTON: NodeKey;
        assert!(tabs.len() != 0);

        self.subtree_old().start(|| {
            let max_n = tabs.len() - 1;
            if *current_tab >= max_n {
                *current_tab = max_n;
            }

            // Update the state in response to button clicks or keyboard presses
            for (i, _) in tabs.iter().enumerate() {
                if self.is_clicked(VERTICAL_TABS_TAB_BUTTON.sibling(i)) {
                    *current_tab = i;
                }
            }
            // todo: focused?
            let ilen = tabs.len() as isize;
            if self
                .key_input()
                .key_pressed_or_repeated(&winit::keyboard::Key::Named(
                    winit::keyboard::NamedKey::Tab,
                ))
                && self.key_input().key_mods().control_key()
            {
                if self.key_input().key_mods().shift_key() {
                    *current_tab = (((*current_tab as isize) - 1 + ilen) % ilen) as usize;
                } else {
                    *current_tab = (*current_tab + 1) % tabs.len();
                }
            }

            let h_stack = H_STACK.stack_spacing(0.0);
            let tabs_v_stack = V_STACK.size_x(Size::Pixels(250.0));
            let inactive_tab = BUTTON
                .shape(Shape::Rectangle { rounded_corners: RoundedCorners::LEFT, corner_radius: DEFAULT_CORNER_RADIUS })
                .size_x(Size::Fill)
                .colors(self.theme().muted_background);
            let active_tab = inactive_tab.colors(self.theme().background);

            #[node_key] const VERTICAL_TABS_CONTENT_PANEL: NodeKey;
            let content_panel = PANEL
                .size_symm(Size::Fill)
                .colors(self.theme().background)
                .children_can_hide(true)
                .key(VERTICAL_TABS_CONTENT_PANEL);

            self.add(h_stack).nest(|| {
                self.add(tabs_v_stack).nest(|| {
                    for (i, tab_name) in tabs.iter().enumerate() {
                        let key_i = VERTICAL_TABS_TAB_BUTTON.sibling(i);
                        let active = i == *current_tab;
                        let tab = if active { active_tab } else { inactive_tab };
                        let tab = tab.static_text(tab_name.0).key(key_i);
                        self.add(tab);
                    }
                });

                // todo: identity for content panel?
                let content_nest = self.add(content_panel);

                content_nest
            })
        })
    }

    /// Add a slider for a `f32` value with a label
    #[track_caller]
    pub fn slider(&mut self, value: &mut f32, min: f32, max: f32) {
        with_arena(|a| {
            self.subtree_old().start(|| {
                let mut new_value = *value;
                if let Some(drag) = self.is_dragged(SLIDER_CONTAINER) {
                    new_value += drag.relative_delta.x as f32 * (max - min);
                }

                if new_value.is_finite() {
                    new_value = new_value.clamp(min, max);
                    *value = new_value;
                }

                let filled_frac = (*value - min) / (max - min);

                #[node_key] const SLIDER_CONTAINER: NodeKey;
                let slider_container = PANEL
                    .size_x(Size::Fill)
                    .size_y(Size::Pixels(45.0))
                    .sense_drag(true)
                    // .shape(Shape::Rectangle { corner_radius: 36.0 })
                    .key(SLIDER_CONTAINER);
                
                #[node_key] const SLIDER_FILL: NodeKey;
                let slider_fill = PANEL
                    .size_y(Fill)
                    .size_x(Size::Frac(filled_frac))
                    .color(Color::KERU_RED)
                    .position_x(Start)
                    .padding_x(1.0)
                    .absorbs_clicks(false)
                    // .shape(Shape::Rectangle { corner_radius: 16.0 })
                    .key(SLIDER_FILL);

                let text = bumpalo::format!(in a, "{:.2}", value);

                self.add(slider_container).nest(|| {
                    self.add(slider_fill);
                    self.text_line(&text);
                });
            });
        });
    }

    /// Add a classic looking slider for a `f32` value
    #[track_caller]
    pub fn classic_slider(&mut self, value: &mut f32, min: f32, max: f32) {
        self.subtree_old().start(|| {
            // todo: combined with the handle's manual positioning, this is pretty awful. it means that the handle is drawn at zero in the first frame.
            // Currently, it relies on the anti-state tearing stuff to not stay at zero.
            // It should be fixed by making it's possible to express the " - handle_radius" part when using a Frac.
            let slider_width = match self.get_node(TRACK) {
                Some(track) => track.last_frame_inner_size().x,
                // this is just for the first frame. awkward.
                // ...or do this calculation after adding it? the result is the same
                None => 1.0,
            };
            
            let handle_radius = 10.0;
            
            if let Some(click) = self.clicked_at(HITBOX) {
                *value = min + click.relative_position.x as f32 * max;
            }
            if let Some(drag) = self.is_dragged(HITBOX) {
                *value = min + drag.relative_position.x as f32 * max;
            }
        
            *value = value.clamp(min, max);
            
            let handle_position_frac = (*value - min) / (max - min);
            
            #[node_key] const TRACK: NodeKey;
            let slider_track = PANEL
                .size_x(Size::Fill)
                .size_y(Size::Pixels(10.0))
                .padding(0.0)
                .color(Color::GREY)
                .shape(Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: 5.0 })
                .absorbs_clicks(false)
                .key(TRACK);
            
            #[node_key] const FILLED: NodeKey;
            let slider_filled = PANEL
                .size_y(Size::Pixels(14.0))
                .shape(Shape::Rectangle { rounded_corners: RoundedCorners::ALL, corner_radius: 7.0 })
                .size_x(Size::Frac(handle_position_frac))
                .color(Color::KERU_RED)
                .position_x(Start)
                .padding_x(0.0)
                .absorbs_clicks(false)
                .key(FILLED);
            
            #[node_key] const HANDLE: NodeKey;
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
            
            #[node_key] const SLIDER_CONTAINER: NodeKey;
            let slider_container = CONTAINER
                .size_x(Size::Fill)
                .size_y(Size::Pixels(45.0))
                .padding_x(0.0)
                .key(SLIDER_CONTAINER);
            
            #[node_key] const HITBOX: NodeKey;
            let hitbox = CONTAINER
                .size_x(Size::Fill)
                .size_y(Size::Pixels(30.0))
                .sense_click(true)
                .sense_drag(true)
                .padding(0.0)
                .key(HITBOX);
            
            self.add(slider_container).nest(|| {
                self.add(hitbox).nest(|| {
                    self.add(slider_track).nest(|| {
                        self.add(slider_filled);
                        self.add(slider_handle);
                    });
                });
            });
            
            self.format_scratch.clear();
        });
    }
}

// Trait components

pub struct Slider<'a> {
    pub value: &'a mut f32,
    pub min: f32,
    pub max: f32,
    pub clamp: bool, // todo: with clamp = false, still clamp values set WITH the slider
}

impl SimpleComponent for Slider<'_> {
    fn add_to_ui(&mut self, ui: &mut Ui) {
        with_arena(|a| {

            #[node_key] const SLIDER_FILL: NodeKey;
            #[node_key] const SLIDER_LABEL: NodeKey;
            #[node_key] const SLIDER_CONTAINER: NodeKey;
                
            let mut new_value = *self.value;
            if let Some(drag) = ui.is_dragged(SLIDER_CONTAINER) {
                new_value += drag.relative_delta.x as f32 * (self.max - self.min);
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
                .size_y(Size::Pixels(45.0))
                .sense_drag(true)
                // .shape(Shape::Rectangle { corner_radius: 36.0 })
                .key(SLIDER_CONTAINER);
            
            let slider_fill = PANEL
                .size_y(Fill)
                .size_x(Size::Frac(filled_frac))
                .color(Color::KERU_RED)
                .position_x(Start)
                .padding_x(1.0)
                .absorbs_clicks(false)
                // .shape(Shape::Rectangle { corner_radius: 16.0 })
                .key(SLIDER_FILL);


            let text = bumpalo::format!(in a, "{:.2}", self.value);
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

pub struct TransformView<'a> {
    pub state: &'a mut TransformViewState,
}

impl<'a> TransformView<'a> {
    pub fn new(state: &'a mut TransformViewState) -> Self {
        Self { state }
    }
}

impl Component for TransformView<'_> {
    type AddResult = UiParent;
    type ComponentOutput = ();
    type State = ();

    fn add_to_ui(&mut self, ui: &mut Ui, _state: &mut Self::State) -> Self::AddResult {
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
            .translate(self.state.pan_x, self.state.pan_y)
            .scale(self.state.scale)
            .size_symm(Size::Fill)
            .clip_children(true);

        let parent = ui.add(transform_area);

        if ui.key_input().key_held(&Key::Named(NamedKey::Space)) {
            ui.add(spacebar_pan_overlay);
        }

        ui.add(pan_overlay);

        let size = ui.inner_size(TRANSFORMED_AREA).unwrap_or(Xy::new(600.0, 600.0));

        // Handle panning
        if ! ui.key_input().key_held(&Key::Named(NamedKey::Space)) {
            if let Some(drag) = ui.is_mouse_button_dragged(PAN_OVERLAY, MouseButton::Middle) {
                self.state.pan_x += drag.absolute_delta.x as f32;
                self.state.pan_y += drag.absolute_delta.y as f32;
            }
        }

        if let Some(drag) = ui.is_dragged(SPACEBAR_PAN_OVERLAY) {
            self.state.pan_x += drag.absolute_delta.x as f32;
            self.state.pan_y += drag.absolute_delta.y as f32;
        }

        // Handle zooming
        let mut apply_zoom = |delta_y: f32, mouse_pos: Vec2| {
            let old_zoom = self.state.scale;
            let curve_factor = ((0.01 + old_zoom).powf(1.1) - 0.01).abs();
            let new_zoom = old_zoom + delta_y as f32 * curve_factor;

            if new_zoom > 0.01 && !new_zoom.is_infinite() && !new_zoom.is_nan() {
                self.state.scale = new_zoom;
                let zoom_ratio = self.state.scale / old_zoom;
                let centered_pos = mouse_pos - vec2(0.5, 0.5);
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

        return parent;
    }
}

pub struct StatefulTransformView;

impl Component for StatefulTransformView {
    type AddResult = UiParent;
    type ComponentOutput = ();
    type State = TransformViewState;

    // todo: right now it's not actually ok to nest them like this.
    fn add_to_ui(&mut self, ui: &mut Ui, state: &mut Self::State) -> Self::AddResult {
        return ui.add_component(TransformView::new(state));
    }
}


#[derive(Default)]
pub struct VerticalTabsState {
    pub i: usize,
}

pub struct StatefulVerticalTabs<'a> {
    pub tabs: &'a [Tab],
    pub key: Option<ComponentKey<Self>>,
}

impl<'a> StatefulVerticalTabs<'a> {
    pub fn new(tabs: &'a [Tab]) -> Self {
        Self { tabs, key: None }
    }

    pub fn key(mut self, key: ComponentKey<Self>) -> Self {
        self.key = Some(key);
        self
    }
}

impl Component for StatefulVerticalTabs<'_> {
    type AddResult = (UiParent, Tab);
    type ComponentOutput = ();
    type State = VerticalTabsState;

    fn add_to_ui(&mut self, ui: &mut Ui, state: &mut Self::State) -> Self::AddResult {
        #[node_key] const VERTICAL_TABS_TAB_BUTTON: NodeKey;
        #[node_key] const VERTICAL_TABS_CONTENT_PANEL: NodeKey;

        assert!(!self.tabs.is_empty());

        let max_n = self.tabs.len() - 1;
        if state.i > max_n {
            state.i = max_n;
        }

        // Update the state in response to button clicks
        for (i, _) in self.tabs.iter().enumerate() {
            if ui.is_clicked(VERTICAL_TABS_TAB_BUTTON.sibling(i)) {
                state.i = i;
            }
        }

        // Handle keyboard navigation with Ctrl+Tab
        let ilen = self.tabs.len() as isize;
        if ui
            .key_input()
            .key_pressed_or_repeated(&winit::keyboard::Key::Named(
                winit::keyboard::NamedKey::Tab,
            ))
            && ui.key_input().key_mods().control_key()
        {
            if ui.key_input().key_mods().shift_key() {
                state.i = (((state.i as isize) - 1 + ilen) % ilen) as usize;
            } else {
                state.i = (state.i + 1) % self.tabs.len();
            }
        }

        let current_tab = self.tabs[state.i];

        let h_stack = H_STACK.stack_spacing(0.0);
        let tabs_v_stack = V_STACK.size_x(Size::Pixels(250.0));
        let inactive_tab = BUTTON
            .shape(Shape::Rectangle { rounded_corners: RoundedCorners::LEFT, corner_radius: 5.0 })
            .size_x(Size::Fill)
            .colors(ui.theme().muted_background);
        let active_tab = inactive_tab.colors(ui.theme().background);

        let content_panel = PANEL
            .size_symm(Size::Fill)
            .colors(ui.theme().background)
            .children_can_hide(true)
            .key(VERTICAL_TABS_CONTENT_PANEL);

        ui.add(h_stack).nest(|| {
            ui.add(tabs_v_stack).nest(|| {
                for (i, tab_name) in self.tabs.iter().enumerate() {
                    let key_i = VERTICAL_TABS_TAB_BUTTON.sibling(i);
                    let active = i == state.i;
                    let tab = if active { active_tab } else { inactive_tab };
                    let tab = tab.static_text(tab_name.0).key(key_i);
                    ui.add(tab);
                }
            });

            (ui.add(content_panel), current_tab)
        })
    }

    fn component_key(&self) -> Option<ComponentKey<Self>> {
        self.key
    }
}

#[derive(Default)]
pub struct HorizontalTabsState {
    pub i: usize,
}

pub struct TabContainer<'a> {
    pub tabs: &'a [Tab],
    pub key: Option<ComponentKey<Self>>,
}

impl<'a> TabContainer<'a> {
    pub fn new(tabs: &'a [Tab]) -> Self {
        Self { tabs, key: None }
    }

    pub fn key(mut self, key: ComponentKey<Self>) -> Self {
        self.key = Some(key);
        self
    }
}

impl Component for TabContainer<'_> {
    type AddResult = (UiParent, Tab);
    type ComponentOutput = ();
    type State = HorizontalTabsState;

    fn add_to_ui(&mut self, ui: &mut Ui, state: &mut Self::State) -> Self::AddResult {
        #[node_key] const HORIZONTAL_TABS_TAB_BUTTON: NodeKey;
        #[node_key] const HORIZONTAL_TABS_CONTENT_PANEL: NodeKey;

        assert!(!self.tabs.is_empty());

        let max_n = self.tabs.len() - 1;
        if state.i > max_n {
            state.i = max_n;
        }

        // Update the state in response to button clicks
        for (i, _) in self.tabs.iter().enumerate() {
            if ui.is_clicked(HORIZONTAL_TABS_TAB_BUTTON.sibling(i)) {
                state.i = i;
            }
        }

        // Handle keyboard navigation with Ctrl+Tab
        let ilen = self.tabs.len() as isize;
        if ui
            .key_input()
            .key_pressed_or_repeated(&winit::keyboard::Key::Named(
                winit::keyboard::NamedKey::Tab,
            ))
            && ui.key_input().key_mods().control_key()
        {
            if ui.key_input().key_mods().shift_key() {
                state.i = (((state.i as isize) - 1 + ilen) % ilen) as usize;
            } else {
                state.i = (state.i + 1) % self.tabs.len();
            }
        }

        let current_tab = self.tabs[state.i];

        let v_stack = V_STACK.stack_spacing(0.0);
        let tabs_h_stack = H_STACK.size_y(Size::FitContent);
        let inactive_tab = BUTTON
            .shape(Shape::Rectangle { rounded_corners: RoundedCorners::TOP, corner_radius: 5.0 })
            .colors(ui.theme().muted_background);
        let active_tab = inactive_tab.colors(ui.theme().background);

        let content_panel = PANEL
            .size_symm(Size::Fill)
            .colors(ui.theme().background)
            .children_can_hide(true)
            .key(HORIZONTAL_TABS_CONTENT_PANEL);

        ui.add(v_stack).nest(|| {
            ui.add(tabs_h_stack).nest(|| {
                for (i, tab_name) in self.tabs.iter().enumerate() {
                    let key_i = HORIZONTAL_TABS_TAB_BUTTON.sibling(i);
                    let active = i == state.i;
                    let tab = if active { active_tab } else { inactive_tab };
                    let tab = tab.static_text(tab_name.0).key(key_i);
                    ui.add(tab);
                }
            });

            (ui.add(content_panel), current_tab)
        })
    }

    fn component_key(&self) -> Option<ComponentKey<Self>> {
        self.key
    }
}


use crate::thread_future_2::{ThreadFuture, run_in_background};
use std::sync::Arc;
use std::task::Poll;

pub struct AsyncButton<T>
where T: Send + 'static {
    function: Arc<dyn Fn() -> T + Send + Sync + 'static>,
    idle_text: &'static str,
    loading_text: &'static str,
    key: Option<ComponentKey<Self>>,
}

impl<T> AsyncButton<T>
where T: Send + 'static {
    pub fn new<F>(function: F, idle_text: &'static str, loading_text: &'static str) -> Self
    where F: Fn() -> T + Send + Sync + 'static {
        Self {
            function: Arc::new(function),
            idle_text,
            loading_text,
            key: None,
        }
    }

    pub fn key(mut self, key: ComponentKey<Self>) -> Self {
        self.key = Some(key);
        self
    }
}

impl<T> Component for AsyncButton<T>
where T: Send + Sync + 'static {
    type AddResult = Poll<T>;
    type ComponentOutput = ();
    type State = Option<ThreadFuture<T>>;

    fn add_to_ui(&mut self, ui: &mut Ui, state: &mut Option<ThreadFuture<T>>) -> Self::AddResult {
        #[node_key] const ASYNC_BUTTON: NodeKey;
    
        let clickable: bool;
        let button_text: &'static str;
        let result: Poll<T>;
    
        match state.as_ref().map(|f| f.poll()) {
            None => {
                button_text = self.idle_text;
                clickable = true;
                result = Poll::Pending;
            }
            Some(Poll::Pending) => {
                button_text = self.loading_text;
                clickable = false;
                result = Poll::Pending;
            }
            Some(Poll::Ready(val)) => {
                button_text = self.idle_text;
                clickable = true;
                result = Poll::Ready(val);
                *state = None; // Reset to idle so we can restart
            }
        };
    
        let button = BUTTON.static_text(button_text).key(ASYNC_BUTTON);

        ui.add(button);
    
        if clickable && ui.is_clicked(ASYNC_BUTTON) {
            let waker = ui.ui_waker_safe();
            let func = Arc::clone(&self.function);
            *state = Some(run_in_background(
                move || func(),
                move || waker.set_update_needed(),
            ));
        }
    
        return result;
    }

    fn component_key(&self) -> Option<ComponentKey<Self>> {
        self.key
    }
}


pub struct ReorderStack {
    pub key: ComponentKey<Self>,
}

impl ReorderStack {
    #[node_key] pub const STACK: NodeKey;
    #[node_key] pub const FLOATING: NodeKey;
    #[node_key] pub const SPACER: NodeKey;
    
    fn calc_insertion_index(ui: &Ui, cursor_y: f32, dragged_index: usize) -> usize {
        let children = ui.get_node(Self::STACK).unwrap().children();

        for (i, child) in children.iter().enumerate() {
            if i == dragged_index {
                continue;
            }

            if cursor_y < child.last_frame_center().y {
                return i;
            }
        }

        return children.len();
    }

}

impl Component for ReorderStack {

    type AddResult = UiParent;
    type ComponentOutput = (usize, usize);
    type State = ();

    fn add_to_ui(&mut self, ui: &mut Ui, _state: &mut Self::State) -> Self::AddResult {
    
        let stack = V_STACK
            .animate_position(true)
            .size(Size::Pixels(100.0), Size::Fill)
            .position_y(Pos::Start)
            .stack_arrange(Arrange::Start)
            .sense_drag_drop_target(true)
            .key(Self::STACK);

        let cursor = ui.cursor_position();
        let floater = CONTAINER
            .anchor(Anchor::Center, Anchor::Center)
            .position(Pos::Pixels(cursor.x), Pos::Pixels(cursor.y))
            .animate_position(true)
            .key(Self::FLOATING);

        ui.jump_to_root().nest(|| {
            ui.add(floater)
        });
        
        return ui.add(stack);
    }

    fn component_key(&self) -> Option<ComponentKey<Self>> {
        return Some(self.key);
    }

    fn run_component(ui: &mut Ui) -> Option<Self::ComponentOutput> {
        // Find the dragged item
        let mut dragged = None;
        if let Some(stack) = ui.get_node(Self::STACK) {
            for (index, child) in stack.children().iter().enumerate() {
                if child.is_dragged().is_some() || child.is_drag_released() {
                    let key = child.temp_key();
                    let height = child.last_frame_rect().size().y;
                    dragged = Some((key, height, index));
                    break;
                }
            }
        }

        if let Some((key, height, index)) = dragged {
            let insertion_index = Self::calc_insertion_index(ui, ui.cursor_position().y, index);

            let hovered = ui.is_any_drag_hovered_onto(Self::STACK).is_some();
            let drag_released = ui.is_any_drag_released_onto(Self::STACK).is_some();
            if hovered || drag_released {

                // Insert spacer at the calculated position
                ui.jump_to_nth_child(Self::STACK, insertion_index).unwrap().nest(|| {
                    let spacer = SPACER
                        .key(Self::SPACER)
                        .size_y(Size::Pixels(height))
                        .absorbs_clicks(false)
                        .animate_position(true);

                    ui.add(spacer);
                });
            }

            ui.jump_to_parent(Self::FLOATING).unwrap().nest(|| {
                ui.remove_and_readd(key);
            });

            // Return swap indices when drag is released
            if drag_released && index != insertion_index {
                return Some((index, insertion_index));
            }
        }

        None
    }
}
