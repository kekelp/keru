use crate::*;

use crate::math::Axis::*;

use ahash::{HashMap, HashMapExt};
use glam::Vec2;

use keru_draw::Renderer;
pub use keru_draw::{TextStyle2 as TextStyle, ColorBrush};
use winit::dpi::PhysicalSize;
use winit::window::Window;
use winit_key_events::KeyInput;
use winit_mouse_events::MouseInput;

use std::any::Any;
use std::collections::BinaryHeap;
use std::num::NonZeroUsize;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::sync::Weak;
use std::sync::{Arc, LazyLock};
use std::thread;
use std::time::Duration;
use std::time::Instant;

use wgpu::{
    Device, Queue, SurfaceConfiguration,
};

pub(crate) static T0: LazyLock<Instant> = LazyLock::new(Instant::now);

/// The original default text style that can be restored with Ctrl+0
pub static ORIGINAL_DEFAULT_TEXT_STYLE: LazyLock<TextStyle> = LazyLock::new(|| TextStyle {
    font_size: 24.0,
    brush: ColorBrush([255, 255, 255, 255]),
    ..Default::default()
});

pub(crate) fn slow_accurate_timestamp_for_events_only() -> f32 {
    return T0.elapsed().as_secs_f32();
}

#[derive(Debug, Clone, Copy)]
pub struct KeruElementRange(pub(crate) keru_draw::InstanceRange);

impl KeruElementRange {
    pub(crate) fn new(start: usize, end: usize) -> Self {
        Self(keru_draw::InstanceRange { start, end })
    }
}

/// A single render command in the list provided by [Ui::render_commands()].
/// 
/// A `RenderCommand` can represent 
#[derive(Debug, Clone, Copy)]
pub enum RenderCommand {
    /// A range of regular Keru ui elements, which can be rendered with the [Ui::render_range()] function.
    Keru(KeruElementRange),
    /// A custom rendering region. Corresponds to a Ui element that was marked as [`custom_render(true)`](Node::custom_render).
    CustomRenderingArea { key: NodeKey, rect: XyRect },
}

/// The central struct of the library, representing the whole GUI state.
///
/// To create a new [`Ui`] instance, use [`Ui::new`].
///
/// To build a GUI, add nodes to the [`Ui`] by calling [`Ui::add`].
///
/// To react to mouse clicks and other node events, call [`Ui::is_clicked`] and similar methods.
///
/// To integrate [`Ui`] with your `winit` event loop, pass all your `winit` events to [`Ui::window_event`].
///
/// To render the GUI to the screen, call [`Ui::render`].
pub struct Ui {
    pub(crate) nodes: Nodes,
    pub(crate) sys: System,
    pub(crate) format_scratch: String, // todo use the thread local arena instead?
    pub(crate) custom_render_commands: Vec<RenderCommand>,
}

static INSTANCE_COUNTER: AtomicU64 = AtomicU64::new(1);

pub(crate) struct System {
    // in inspect mode, draw invisible rects as well, for example V_STACKs.
    // usually these have filled = false (just the outline), but this is not enforced.
    pub inspect_mode: bool,

    pub global_animation_speed: f32,
    pub disable_animations_on_resize: bool,

    pub t: f32, // time at the end of the last rendered frame, in seconds since the Ui creation

    pub unique_id: u64,
    pub theme: Theme,
    pub debug_key_pressed: bool,

    // todo: new system for this stuff
    pub update_frames_needed: u8,
    pub new_external_events: bool,

    pub renderer: Renderer,

    pub z_cursor: f32,

    pub click_rects: Vec<ClickRect>,

    pub size: Xy<f32>,

    pub current_frame: u64,
    pub last_frame_end_fake_time: u64,
    pub second_last_frame_end_fake_time: u64,
    pub third_last_frame_end_fake_time: u64,

    // mouse input needs to be Id based, not NodeI based, because you can hold a button for several frames
    pub mouse_input: MouseInput<Id>,
    pub key_input: KeyInput,
    
    pub text_edit_changed_last_frame: Option<Id>,
    pub text_edit_changed_this_frame: Option<Id>,

    #[cfg(debug_assertions)]
    pub inspect_hovered: SmallVec<Id>,

    // ???????
    pub hovered: Vec<Id>,

    pub focused: Option<Id>,

    // this is used exclusively for info messages
    pub partial_relayout_count: u32,

    // Holds the nodes for breadth-first traversal.
    pub depth_traversal_queue: Vec<NodeI>,

    pub non_fresh_nodes: Vec<NodeI>,

    pub to_cleanup: Vec<NodeI>,
    pub hidden_branch_parents: Vec<NodeI>,
    pub lingering_nodes: Vec<NodeWithDepth>,

    pub changes: Changes,

    // move to changes oalgo
    // note that the magic "shader only animations" will probably disappear eventually,
    // so things like this will need to rebuild render data, not just rerender
    pub anim_render_timer: AnimationRenderTimer,

    pub user_state: HashMap<StateId, Box<dyn Any>>,

    // todo: do something else
    pub image_cache: lru::LruCache<ImageSourceId, ImageRef>,

    pub needs_update: Arc<AtomicBool>,
    pub window_ref: Option<Weak<Window>>,
    pub scheduled_wakeup: Option<ScheduledWakeupHandle>,

    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

/// A handle that can be used to wake up the [`Ui`] from another thread.
#[derive(Clone)]
pub struct UiWaker {
    pub(crate) needs_update: Arc<AtomicBool>,
    pub(crate) window_ref: Option<Weak<Window>>,
}

impl UiWaker {
    /// Signal that the [`Ui`] needs to be updated, causing the next call to [`Ui::should_update()`] to return `true`.
    /// 
    /// If [`Ui::enable_auto_wakeup()`] was called on the [`Ui`], this will also wake up the `winit` event loop by calling `request_redraw()` on the window.
    pub fn set_update_needed(&self) {
        self.needs_update.store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(window) = self.window_ref.as_ref().and_then(|w| w.upgrade()) {
            window.request_redraw();
        }
    }
}

use std::cmp::Reverse;
use std::sync::mpsc::{RecvTimeoutError, Sender};

pub(crate) struct ScheduledWakeupHandle {
    sender: Sender<Instant>,
}

impl ScheduledWakeupHandle {
    fn new(waker: UiWaker) -> Self {
        let (sender, receiver) = mpsc::channel::<Instant>();
        
        thread::spawn(move || {
            let mut pending: BinaryHeap<Reverse<Instant>> = BinaryHeap::new();
            
            loop {
                let timeout = pending
                    .peek()
                    .map(|Reverse(wake_at)| wake_at.saturating_duration_since(Instant::now()))
                    .unwrap_or(Duration::MAX);
                
                match receiver.recv_timeout(timeout) {
                    Ok(wake_at) => {
                        pending.push(Reverse(wake_at));
                    }
                    Err(RecvTimeoutError::Timeout) => {
                        // Drain all overdue wakeups
                        let now = Instant::now();
                        while pending.peek().map_or(false, |Reverse(wake_at)| *wake_at <= now) {
                            pending.pop();
                        }
                        waker.set_update_needed();
                    }
                    Err(RecvTimeoutError::Disconnected) => break,
                }
            }
        });
        
        Self { sender }
    }
    
    fn schedule(&self, duration: Duration) {
        let _ = self.sender.send(Instant::now() + duration);
    }
}

pub(crate) struct AnimationRenderTimer(Option<Instant>);

impl AnimationRenderTimer {
    fn default() -> Self {
        Self(None)
    }

    pub(crate) fn push_new(&mut self, duration: Duration) {
        let now = Instant::now();
        let new_end = now + duration;

        if let Some(end) = self.0 {
            if new_end > end {
                *self = AnimationRenderTimer(Some(new_end));
            }
        } else {
            *self = AnimationRenderTimer(Some(new_end));
        }
    }

    pub(crate) fn is_live(&mut self) -> bool {
        if let Some(end) = self.0 {
            let is_live = Instant::now() < end;
            if !is_live {
                *self = AnimationRenderTimer(None);
            }
            return is_live;
        }
        false
    }
}

impl Ui {
    pub fn new(device: &Device, queue: &Queue, config: &SurfaceConfiguration) -> Self {
        // initialize the static T0
        LazyLock::force(&T0);

        let nodes = Nodes::new();

        let third_last_frame_end_fake_time = 3;
        let second_last_frame_end_fake_time = 4;
        let last_frame_end_fake_time = 5;

        let renderer = Renderer::new(device.clone(), queue.clone(), config.format);

        Self {
            nodes,
            format_scratch: String::with_capacity(1024),
            custom_render_commands: Vec::with_capacity(50),

            sys: System {
                t: 0.0,
                global_animation_speed: 1.0,
                disable_animations_on_resize: true,
                unique_id: INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed),
                z_cursor: 0.0,
                theme: KERU_DARK,
                inspect_mode: false,
                debug_key_pressed: false,

                update_frames_needed: 2,
                new_external_events: true,

                click_rects: Vec::with_capacity(50),


                partial_relayout_count: 0,

                current_frame: FIRST_FRAME,
                third_last_frame_end_fake_time,
                second_last_frame_end_fake_time,
                last_frame_end_fake_time,

                size: Xy::new(config.width as f32, config.height as f32),

                depth_traversal_queue: Vec::with_capacity(64),

                mouse_input: MouseInput::default(),
                key_input: KeyInput::default(),
                text_edit_changed_last_frame: None,
                text_edit_changed_this_frame: None,

                // todo: maybe remove and use mouse_input.current_tag()? There was never a point in having multiple hovereds
                hovered: Vec::with_capacity(15),

                #[cfg(debug_assertions)]
                inspect_hovered: smallvec::SmallVec::new(),

                non_fresh_nodes: Vec::with_capacity(10),
                to_cleanup: Vec::with_capacity(30),
                hidden_branch_parents: Vec::with_capacity(30),
                lingering_nodes: Vec::with_capacity(30),

                focused: None,

                anim_render_timer: AnimationRenderTimer::default(),

                changes: Changes::new(),

                renderer,

                user_state: HashMap::with_capacity(7),

                image_cache: lru::LruCache::new(NonZeroUsize::new(128).unwrap()),

                needs_update: Arc::new(AtomicBool::new(false)),
                window_ref: None,
                scheduled_wakeup: None,

                device: device.clone(),
                queue: queue.clone(),
            },
        }
    }

    /// Registers the `winit` window so that it can be used for automatic wakeup for cursor blinking, scheduled wakeups, and using the [UiWaker].
    ///
    /// In applications that don't pause their event loops, like games, there is no need to call this method.
    ///
    /// You can also handle cursor wakeups manually in your winit event loop with winit's `ControlFlow::WaitUntil` and [`Text::time_until_next_cursor_blink`]. See the `event_loop_smart.rs` example.
    pub fn enable_auto_wakeup(&mut self, window: Arc<Window>) {
        self.sys.renderer.text.set_auto_wakeup(window.clone());
        self.sys.window_ref = Some(Arc::downgrade(&window));
    }

    /// Set the global animation speed multiplier.
    pub fn set_global_animation_speed(&mut self, speed: f32) {
        self.sys.global_animation_speed = speed;
    }

    /// Get the global animation speed multiplier.
    pub fn global_animation_speed(&mut self) -> f32 {
        self.sys.global_animation_speed
    }

    /// Set whether animations should be disabled during window resize.
    pub fn set_disable_animations_on_resize(&mut self, disable: bool) {
        self.sys.disable_animations_on_resize = disable;
    }

    /// Set inspect mode. When inspect mode is active, all nodes will be shown, including stacks and containers. 
    pub fn set_inspect_mode(&mut self, inspect_mode: bool) {
        if self.inspect_mode() != inspect_mode {
            self.sys.changes.full_relayout = true;
        }
        self.sys.inspect_mode = inspect_mode;
    }

    /// Get the current inspect mode state.
    /// When inspect mode is active, all nodes will be shown, including stacks and containers.
    pub fn inspect_mode(&self) -> bool {
        return self.sys.inspect_mode;
    }

    /// Get a reference to the active theme.
    pub fn theme(&mut self) -> &mut Theme {
        return &mut self.sys.theme;
    }

    pub fn current_frame(&self) -> u64 {
        return self.sys.current_frame;
    }

    pub fn unique_id(&self) -> u64 {
        return self.sys.unique_id;
    }

    /// Get the current screen size in pixels.
    pub fn screen_size(&self) -> (f32, f32) {
        (self.sys.size.x, self.sys.size.y)
    }

    pub fn push_external_event(&mut self) {
        self.sys.new_external_events = true;
    }

    /// Get a [`UiWaker`] that can be used to wake up the ui from a different thread.
    ///
    /// Panics if [`Ui::enable_auto_wakeup()`] wasn't called on this [`Ui`] instance.
    pub fn ui_waker(&mut self) -> UiWaker {
        if self.sys.window_ref.is_none() {
            panic!("Wakeup not enabled. Ui::enable_auto_wakeup() must be called before calling this function.");
        }
        UiWaker {
            needs_update: Arc::clone(&self.sys.needs_update),
            window_ref: self.sys.window_ref.clone(),
        }
    }

    /// Get a [`UiWaker`] that can be used to wake up the ui from a different thread.
    ///
    /// If [Ui::enable_auto_wakeup()] wasn't called on this [`Ui`] instance, the `UiWaker` won't be able to wake up the `winit` event loop. However, it will still set the [`Ui`]'s state so that the next call to [`Ui::needs_update()`] will return `true`.
    pub fn ui_waker_safe(&mut self) -> UiWaker {
        UiWaker {
            needs_update: Arc::clone(&self.sys.needs_update),
            window_ref: self.sys.window_ref.clone(),
        }
    }

    /// Schedule a wakeup after the specified duration.
    ///
    /// The scheduler thread is created lazily on the first call to this method.
    ///
    /// Panics if [Ui::enable_auto_wakeup()] wasn't called on this [Ui] instance.
    pub fn schedule_wakeup(&mut self, duration: Duration) {
        if self.sys.window_ref.is_none() {
            panic!("Wakeup not enabled. Ui::enable_auto_wakeup() must be called before calling this function.");
        }
        let waker = UiWaker {
            needs_update: Arc::clone(&self.sys.needs_update),
            window_ref: self.sys.window_ref.clone(),
        };

        if self.sys.scheduled_wakeup.is_none() {
            self.sys.scheduled_wakeup = Some(ScheduledWakeupHandle::new(waker));
        }

        self.sys.scheduled_wakeup.as_ref().unwrap().schedule(duration);
    }

    /// Returns `true` if the [`Ui`] needs to be updated.
    ///
    /// This is true when the [`Ui`] received an input that it cares about, such as a click on a clickable element, or when the user explicitly notified it with [`Ui::push_external_event()`].
    ///
    /// In a typical `winit` loop for an application that only updates in response to user input, this function is what decides if the [`Ui`] building code should be rerun.
    ///
    /// In applications that update on every frame regardless of user input, like games or simulations, the [`Ui`] building code should be rerun on every frame unconditionally, so this function isn't useful.
    pub fn should_update(&mut self) -> bool {
        if self.sys.needs_update.swap(false, std::sync::atomic::Ordering::Relaxed) {
            self.sys.update_frames_needed = 2;
        }
        return self.sys.update_frames_needed > 0 ||
            self.sys.new_external_events;
    }

    /// Returns `true` if the [`Ui`] needs to be updated or rerendered as a consequence of input, animations, or other [`Ui`]-internal events.
    /// 
    /// In a typical `winit` loop for an application that only updates in response to user input, this function is what decides if `winit::Window::request_redraw()` should be called.
    /// 
    /// For an application that updates on every frame regardless of user input, like a game or a simulation, `request_redraw()` should be called on every frame unconditionally, so this function isn't useful.
    pub fn should_request_redraw(&mut self) -> bool {
        return self.should_update() || self.should_rerender();
    }

    pub fn cursor_position(&self) -> Vec2 {
        return self.sys.mouse_input.cursor_position();
    }

    /// Returns a reference to the list of render commands for this frame. 
    /// 
    /// See the `custom_rendering.rs` example for an example.
    /// 
    /// If you don't use any custom wgpu rendering or custom shaders, this is not needed: use [Ui::render()] or [Ui::autorender()].
    pub fn render_commands(&self) -> &[RenderCommand] {
        &self.custom_render_commands
    }

    // todo: expose functions directly instead of the inner struct
    pub fn key_input(&self) -> &KeyInput {
        return &self.sys.key_input;
    }

    pub fn scroll_delta(&self) -> Option<glam::Vec2> {
        return self.sys.mouse_input.scrolled(None);
    }

    pub(crate) fn set_new_ui_input(&mut self) {
        // Anti state-tearing: always update two times
        // Or rather, anti get-stuck-in-a-state-teared-frame. The state tearing is still there for one frame.
        self.sys.update_frames_needed = 2;
    }

    /// Resize the `Ui`.
    /// Updates the `Ui`'s internal state, and schedules a full relayout to adapt to the new size.
    /// Called by [`Ui::window_event`].
    pub(crate) fn resize(&mut self, size: &PhysicalSize<u32>) {
        self.sys.changes.full_relayout = true;

        self.sys.size[X] = size.width as f32;
        self.sys.size[Y] = size.height as f32;

        self.sys.changes.resize = true;

        self.sys.renderer.resize(size.width, size.height);

        self.set_new_ui_input();
    }

    pub fn default_text_style_mut(&mut self) -> &mut TextStyle {
        self.sys.changes.full_relayout = true;
        self.sys.renderer.text.get_default_text_style_mut()
    }

    pub fn default_text_style(&self) -> &TextStyle {
        self.sys.renderer.text.get_default_text_style()
    }

    pub fn original_default_style(&self) -> TextStyle {
        self.sys.renderer.text.original_default_style()
    }

    pub(crate) fn new_redraw_requested_frame(&mut self) {
        
    }
}

impl Ui {
    /// Hit test with the current stored cursor position and a click rect
    pub(crate) fn hit_click_rect(&self, rect: &ClickRect) -> bool {
        let size = self.sys.size;

        // Get cursor position and convert to normalized coordinates
        let cursor_pos = (
            self.cursor_position().x as f32 / size[X],
            self.cursor_position().y as f32 / size[Y],
        );

        let node_i = rect.i;

        let aabb_hit = rect.rect[X][0] < cursor_pos.0
            && cursor_pos.0 < rect.rect[X][1]
            && rect.rect[Y][0] < cursor_pos.1
            && cursor_pos.1 < rect.rect[Y][1];

        if aabb_hit == false {
            return false;
        }

        // todo more accurate clicks
        match self.nodes[node_i].params.shape {
            Shape::NoShape => {
                return false; // weird...
            }
            Shape::Rectangle { .. } => {
                return true;
            }
            Shape::Circle => {
                // Calculate the circle center and radius
                let center_x = (rect.rect[X][0] + rect.rect[X][1]) / 2.0;
                let center_y = (rect.rect[Y][0] + rect.rect[Y][1]) / 2.0;
                let radius = (rect.rect[X][1] - rect.rect[X][0]) / 2.0;

                // Check if the mouse is within the circle
                let dx = cursor_pos.0 - center_x;
                let dy = cursor_pos.1 - center_y;
                return dx * dx + dy * dy <= radius * radius;
            }
            Shape::Ring { width } => {
                // scale to correct coordinates
                // width should have been a Len anyway so this will have to change
                let width = width / size[X];

                let aspect = size[X] / size[Y];
                // Calculate the ring's center and radii
                let center_x = (rect.rect[X][0] + rect.rect[X][1]) / 2.0;
                let center_y = (rect.rect[Y][0] + rect.rect[Y][1]) / 2.0;
                let outer_radius = (rect.rect[X][1] - rect.rect[X][0]) / 2.0;
                let inner_radius = outer_radius - width;

                // Check if the mouse is within the ring
                let dx = cursor_pos.0 - center_x;
                let dy = (cursor_pos.1 - center_y) / aspect;
                let distance_squared = dx * dx + dy * dy;
                return distance_squared <= outer_radius * outer_radius
                    && distance_squared >= inner_radius * inner_radius;

            }
            Shape::Arc { .. } => {
                let center_x = (rect.rect[X][0] + rect.rect[X][1]) / 2.0;
                let center_y = (rect.rect[Y][0] + rect.rect[Y][1]) / 2.0;
                let radius = (rect.rect[X][1] - rect.rect[X][0]) / 2.0;

                let dx = cursor_pos.0 - center_x;
                let dy = cursor_pos.1 - center_y;
                return dx * dx + dy * dy <= radius * radius;
            }
            Shape::Pie { .. } => {
                let center_x = (rect.rect[X][0] + rect.rect[X][1]) / 2.0;
                let center_y = (rect.rect[Y][0] + rect.rect[Y][1]) / 2.0;
                let radius = (rect.rect[X][1] - rect.rect[X][0]) / 2.0;

                let dx = cursor_pos.0 - center_x;
                let dy = cursor_pos.1 - center_y;
                return dx * dx + dy * dy <= radius * radius;
            }
            Shape::Segment { .. } | Shape::HorizontalLine | Shape::VerticalLine | Shape::Triangle { .. } | Shape::SquareGrid { .. } | Shape::HexGrid { .. } => {
                // For segments, triangles, and grids, use simple rectangle hit test
                return true;
            }
        }

    }

    fn unload_imageref(&mut self, imageref: &ImageRef) {
        match imageref {
            ImageRef::Raster(loaded) => self.sys.renderer.image_renderer.unload_image(loaded),
            ImageRef::Svg(loaded) => self.sys.renderer.image_renderer.unload_svg(loaded),
        }
    }

    fn cache_image(&mut self, source: ImageSourceId, imageref: ImageRef) {
        if let Some((_evicted_key, evicted_imageref)) = self.sys.image_cache.push(source, imageref) {
            self.unload_imageref(&evicted_imageref);
        }
    }

    pub(crate) fn set_static_image(&mut self, i: NodeI, image: &'static [u8]) {
        let node = &mut self.nodes[i];
        let source = ImageSourceId::StaticPtr(image.as_ptr());

        if node.last_image_source == Some(source) {
            return;
        }

        // Check global cache
        if let Some(cached) = self.sys.image_cache.get(&source) {
            node.imageref = Some(cached.clone());
            node.last_image_source = Some(source);
            self.sys.changes.should_rebuild_render_data = true;
            return;
        }

        if let Some(loaded) = self.sys.renderer.image_renderer.load_encoded_image(image) {
            log::info!("Loaded image: {}x{} on page {}", loaded.width, loaded.height, loaded.page);
            let imageref = ImageRef::Raster(loaded);
            self.cache_image(source, imageref.clone());
            self.nodes[i].imageref = Some(imageref);
            self.nodes[i].last_image_source = Some(source);
            self.sys.changes.should_rebuild_render_data = true;
        } else {
            log::error!("Failed to load image from {} bytes", image.len());
        }
    }

    pub(crate) fn set_static_svg(&mut self, i: NodeI, svg_data: &'static [u8]) {
        let node = &mut self.nodes[i];
        let source = ImageSourceId::StaticPtr(svg_data.as_ptr());

        if node.last_image_source == Some(source) {
            return;
        }

        // Check global cache
        if let Some(cached) = self.sys.image_cache.get(&source) {
            node.imageref = Some(cached.clone());
            node.last_image_source = Some(source);
            self.sys.changes.should_rebuild_render_data = true;
            return;
        }

        let initial_size = 512;
        if let Some(loaded) = self.sys.renderer.image_renderer.load_svg(svg_data, initial_size, initial_size) {
            log::info!("Loaded SVG: {}x{} on page {}", loaded.width, loaded.height, loaded.page);
            let imageref = ImageRef::Svg(loaded);
            self.cache_image(source, imageref.clone());
            self.nodes[i].imageref = Some(imageref);
            self.nodes[i].last_image_source = Some(source);
            self.sys.changes.should_rebuild_render_data = true;
        } else {
            log::error!("Failed to load SVG from {} bytes", svg_data.len());
        }
    }

    pub(crate) fn set_path_image(&mut self, i: NodeI, path: &str) {
        let node = &mut self.nodes[i];
        let source = crate::inner_node::ImageSourceId::PathHash(ahash(&path));

        if node.last_image_source == Some(source) {
            return;
        }

        // Check global cache
        if let Some(cached) = self.sys.image_cache.get(&source) {
            node.imageref = Some(cached.clone());
            node.last_image_source = Some(source);
            self.sys.changes.should_rebuild_render_data = true;
            return;
        }

        match std::fs::read(path) {
            Ok(bytes) => {
                if let Some(loaded) = self.sys.renderer.image_renderer.load_encoded_image(&bytes) {
                    log::info!("Loaded image from path '{}': {}x{} on page {}", path, loaded.width, loaded.height, loaded.page);
                    let imageref = ImageRef::Raster(loaded);
                    self.cache_image(source, imageref.clone());
                    self.nodes[i].imageref = Some(imageref);
                    self.nodes[i].last_image_source = Some(source);
                    self.sys.changes.should_rebuild_render_data = true;
                } else {
                    log::error!("Failed to decode image from path '{}'", path);
                }
            }
            Err(e) => {
                log::error!("Failed to read image file '{}': {}", path, e);
            }
        }
    }

    pub(crate) fn set_path_svg(&mut self, i: NodeI, path: &str) {
        let node = &mut self.nodes[i];
        let source = crate::inner_node::ImageSourceId::PathHash(ahash(&path));

        if node.last_image_source == Some(source) {
            return;
        }

        // Check global cache
        if let Some(cached) = self.sys.image_cache.get(&source) {
            node.imageref = Some(cached.clone());
            node.last_image_source = Some(source);
            self.sys.changes.should_rebuild_render_data = true;
            return;
        }

        match std::fs::read(path) {
            Ok(bytes) => {
                let initial_size = 512;
                if let Some(loaded) = self.sys.renderer.image_renderer.load_svg(&bytes, initial_size, initial_size) {
                    log::info!("Loaded SVG from path '{}': {}x{} on page {}", path, loaded.width, loaded.height, loaded.page);
                    let imageref = ImageRef::Svg(loaded);
                    self.cache_image(source, imageref.clone());
                    self.nodes[i].imageref = Some(imageref);
                    self.nodes[i].last_image_source = Some(source);
                    self.sys.changes.should_rebuild_render_data = true;
                } else {
                    log::error!("Failed to decode SVG from path '{}'", path);
                }
            }
            Err(e) => {
                log::error!("Failed to read SVG file '{}': {}", path, e);
            }
        }
    }
}

pub type StateId = Id;
