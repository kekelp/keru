use crate::*;

use crate::math::Axis::*;

use ahash::{HashMap, HashMapExt};
use glam::DVec2;

use textslabs::{ColorBrush, Text, TextStyle2 as TextStyle};
use winit::dpi::PhysicalSize;
use winit::window::Window;
use winit_key_events::KeyInput;
use winit_mouse_events::MouseInput;

use std::any::Any;
use std::collections::{BinaryHeap, VecDeque};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::mpsc;
use std::sync::Weak;
use std::sync::{Arc, LazyLock};
use std::thread;
use std::time::Duration;
use std::time::Instant;

use vello_common::pixmap::Pixmap;
use vello_common::peniko::color::PremulRgba8;

use bytemuck::{Pod, Zeroable};
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

pub(crate) fn ui_time_f32() -> f32 {
    return T0.elapsed().as_secs_f32();
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
    pub(crate) format_scratch: String,
}

static INSTANCE_COUNTER: AtomicU64 = AtomicU64::new(1);

pub(crate) struct System {
    // in inspect mode, draw invisible rects as well, for example V_STACKs.
    // usually these have filled = false (just the outline), but this is not enforced.
    pub inspect_mode: bool,

    // todo: I didn't mean to keep copies of these handles, but vello's image functions kind of require it.
    pub device: Device,
    pub queue: Queue,

    pub global_animation_speed: f32,

    pub unique_id: u64,
    pub theme: Theme,
    pub debug_key_pressed: bool,

    // todo: new system for this stuff
    pub update_frames_needed: u8,
    pub new_external_events: bool,

    pub text: Text,

    pub svg_storage: Vec<Vec<vello_common::pico_svg::Item>>,

    pub vello_scene: vello_hybrid::Scene,
    pub vello_renderer: vello_hybrid::Renderer,

    pub z_cursor: f32,

    pub click_rects: Vec<ClickRect>,

    // rects that react to mouse wheel scroll
    pub scroll_rects: Vec<ClickRect>,

    pub unifs: Uniforms,
    pub current_frame: u64,
    pub last_frame_end_fake_time: u64,
    pub second_last_frame_end_fake_time: u64,
    pub third_last_frame_end_fake_time: u64,

    pub mouse_hit_stack: Vec<(Id, f32)>,

    // mouse input needs to be Id based, not NodeI based, because you can hold a button for several frames
    pub mouse_input: MouseInput<Id>,
    pub key_input: KeyInput,
    
    pub text_edit_changed_last_frame: Option<Id>,
    pub text_edit_changed_this_frame: Option<Id>,

    #[cfg(debug_assertions)]
    pub inspect_hovered: Option<Id>,

    pub hovered: Vec<Id>,
    pub hovered_scroll_area: Option<Id>,

    pub focused: Option<Id>,

    // this is used exclusively for info messages
    pub partial_relayout_count: u32,

    // Holds the nodes for breadth-first traversal.
    pub breadth_traversal_queue: VecDeque<NodeI>,

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

    pub waker: Option<UiWaker>,
    pub scheduled_wakeup: Option<ScheduledWakeupHandle>,
}

#[derive(Clone)]
pub struct UiWaker {
    pub(crate) needs_update: Arc<AtomicBool>,
    pub(crate) window_ref: Weak<Window>,
}

impl UiWaker {
    pub fn set_update_needed(&self) {
        self.needs_update.store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(window) = self.window_ref.upgrade() {
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

#[repr(C)]
#[derive(Debug, Pod, Copy, Clone, Zeroable)]
pub(crate) struct Uniforms {
    pub size: Xy<f32>,
    pub t: f32,
    pub _padding: f32,
}

impl Ui {
    pub fn new(device: &Device, queue: &Queue, config: &SurfaceConfiguration) -> Self {
        // initialize the static T0
        LazyLock::force(&T0);

        let uniforms = Uniforms {
            size: Xy::new(config.width as f32, config.height as f32),
            t: 0.,
            _padding: 0.,
        };



        let nodes = Nodes::new();

        let third_last_frame_end_fake_time = 3;
        let second_last_frame_end_fake_time = 4;
        let last_frame_end_fake_time = 5;

        Self {
            nodes,
            format_scratch: String::with_capacity(1024),

            sys: System {
                device: device.clone(),
                queue: queue.clone(),

                global_animation_speed: 1.0,
                unique_id: INSTANCE_COUNTER.fetch_add(1, Ordering::Relaxed),
                z_cursor: 0.0,
                theme: KERU_DARK,
                inspect_mode: false,
                debug_key_pressed: false,

                update_frames_needed: 2,
                new_external_events: true,
                
                click_rects: Vec::with_capacity(50),
                scroll_rects: Vec::with_capacity(20),


                partial_relayout_count: 0,

                current_frame: FIRST_FRAME,
                third_last_frame_end_fake_time,
                second_last_frame_end_fake_time,
                last_frame_end_fake_time,

                unifs: uniforms,

                breadth_traversal_queue: VecDeque::with_capacity(64),

                mouse_hit_stack: Vec::with_capacity(50),

                mouse_input: MouseInput::default(),
                key_input: KeyInput::default(),
                text_edit_changed_last_frame: None,
                text_edit_changed_this_frame: None,

                // todo: maybe remove and use mouse_input.current_tag()? There was never a point in having multiple hovereds
                hovered: Vec::with_capacity(15),
                hovered_scroll_area: None,

                #[cfg(debug_assertions)]
                inspect_hovered: None,

                non_fresh_nodes: Vec::with_capacity(10),
                to_cleanup: Vec::with_capacity(30),
                hidden_branch_parents: Vec::with_capacity(30),
                lingering_nodes: Vec::with_capacity(30),

                focused: None,

                anim_render_timer: AnimationRenderTimer::default(),

                changes: Changes::new(),

                text: Text::new(),

                svg_storage: Vec::new(),

                vello_scene: vello_hybrid::Scene::new(config.width as u16, config.height as u16),
                vello_renderer: vello_hybrid::Renderer::new(
                    device,
                    &vello_hybrid::RenderTargetConfig {
                        format: config.format,
                        width: config.width,
                        height: config.height,
                    },
                ),

                user_state: HashMap::with_capacity(7),

                waker: None,
                scheduled_wakeup: None,
            },
        }
    }

    /// Registers the `winit` window so that it can be used for automatic wakeup for cursor blinking, scheduled wakeups, and using the [UiWaker].
    /// 
    /// In applications that don't pause their event loops, like games, there is no need to call this method.
    /// 
    /// You can also handle cursor wakeups manually in your winit event loop with winit's `ControlFlow::WaitUntil` and [`Text::time_until_next_cursor_blink`]. See the `event_loop_smart.rs` example.
    pub fn enable_auto_wakeup(&mut self, window: Arc<Window>) {
        self.sys.text.set_auto_wakeup(window.clone());
        self.sys.waker = Some(UiWaker {
            needs_update: Arc::new(AtomicBool::new(false)),
            window_ref: Arc::downgrade(&window),
        });
    }

    /// Set the global animation speed multiplier.
    pub fn set_global_animation_speed(&mut self, speed: f32) {
        self.sys.global_animation_speed = speed;
    }

    /// Get the global animation speed multiplier.
    pub fn global_animation_speed(&mut self) -> f32 {
        self.sys.global_animation_speed
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

    pub fn push_external_event(&mut self) {
        self.sys.new_external_events = true;
    }

    /// Get a [UiWaker], which can be used to wake up the ui from a different thread.
    ///
    /// Panics if the [Ui::enable_auto_wakeup()] wasn't called on this [Ui] instance.
    pub fn ui_waker(&mut self) -> UiWaker {
        return self.sys.waker.as_ref()
            .expect("Wakeup not enabled. Ui::enable_auto_wakeup() must be called before calling this function.")
            .clone()
    }

    /// Schedule a wakeup after the specified duration.
    ///
    /// The scheduler thread is created lazily on the first call to this method.
    ///
    /// Panics if [Ui::enable_auto_wakeup()] wasn't called on this [Ui] instance.
    pub fn schedule_wakeup(&mut self, duration: Duration) {
        let waker = self.sys.waker.as_ref().expect("Wakeup not enabled. Ui::enable_auto_wakeup() must be called before calling this function.");

        if self.sys.scheduled_wakeup.is_none() {
            self.sys.scheduled_wakeup = Some(ScheduledWakeupHandle::new(waker.clone()));
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
        let real_external_events = if let Some(waker) = &self.sys.waker {
            waker.needs_update.load(std::sync::atomic::Ordering::Relaxed)
        } else { false };
        return self.sys.update_frames_needed > 0 ||
            self.sys.new_external_events || 
            real_external_events;
    }

    /// Returns `true` if the [`Ui`] needs to be updated or rerendered as a consequence of input, animations, or other [`Ui`]-internal events.
    /// 
    /// In a typical `winit` loop for an application that only updates in response to user input, this function is what decides if `winit::Window::request_redraw()` should be called.
    /// 
    /// For an application that updates on every frame regardless of user input, like a game or a simulation, `request_redraw()` should be called on every frame unconditionally, so this function isn't useful.
    pub fn should_request_redraw(&mut self) -> bool {
        return self.should_update() || self.should_rerender();
    }

    pub fn cursor_position(&self) -> DVec2 {
        return self.sys.mouse_input.cursor_position();
    }

    // todo: expose functions directly instead of the inner struct
    pub fn key_input(&self) -> &KeyInput {
        return &self.sys.key_input;
    }

    pub fn scroll_delta(&self) -> Option<glam::DVec2> {
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

        self.sys.unifs.size[X] = size.width as f32;
        self.sys.unifs.size[Y] = size.height as f32;

        self.sys.changes.resize = true;

        // Update vello_scene size (vello_renderer uses RenderSize at render time)
        self.sys.vello_scene = vello_hybrid::Scene::new(size.width as u16, size.height as u16);

        self.set_new_ui_input();
    }

    pub fn default_text_style_mut(&mut self) -> &mut TextStyle {
        self.sys.changes.full_relayout = true;
        self.sys.text.get_default_text_style_mut()
    }

    pub fn default_text_style(&self) -> &TextStyle {
        self.sys.text.get_default_text_style()
    }

    pub fn original_default_style(&self) -> TextStyle {
        self.sys.text.original_default_style()
    }

    pub(crate) fn new_redraw_requested_frame(&mut self) {
        
    }
}

impl Ui {
    pub(crate) fn hit_click_rect(&self, rect: &ClickRect) -> bool {
        let size = self.sys.unifs.size;
        let cursor_pos = (
            self.cursor_position().x as f32 / size[X],
            self.cursor_position().y as f32 / size[Y],
        );

        let aabb_hit = rect.rect[X][0] < cursor_pos.0
            && cursor_pos.0 < rect.rect[X][1]
            && rect.rect[Y][0] < cursor_pos.1
            && cursor_pos.1 < rect.rect[Y][1];

        if aabb_hit == false {
            return false;
        }

        let node_i = rect.i;


        match self.nodes[node_i].params.rect.shape {
            Shape::Rectangle { corner_radius: _ } => {
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
        }

    }

    pub(crate) fn set_static_image(&mut self, i: NodeI, image: &'static [u8]) -> &mut Self {
        let image_pointer: *const u8 = image.as_ptr();

        if let Some(last_pointer) = self.nodes[i].last_static_image_ptr {
            if image_pointer == last_pointer {
                return self;
            }
        }

        // Load and decode the image
        let img = image::load_from_memory(image).unwrap();
        let img = img.to_rgba8();
        let (width, height) = img.dimensions();

        log::info!("Decoded image: {}x{}", width, height);

        // Convert to premultiplied RGBA8
        let pixels: Vec<PremulRgba8> = img.pixels().map(|p| {
            let r = p[0];
            let g = p[1];
            let b = p[2];
            let a = p[3];

            let alpha = a as u16;
            let premul_r = ((r as u16 * alpha) / 255) as u8;
            let premul_g = ((g as u16 * alpha) / 255) as u8;
            let premul_b = ((b as u16 * alpha) / 255) as u8;

            PremulRgba8 { r: premul_r, g: premul_g, b: premul_b, a }
        }).collect();

        let pixmap = Pixmap::from_parts(pixels, width as u16, height as u16);

        // todo: do this without holding handles to the device and the queue and creating a new encoder.
        // I'm trusting that vello will not actually submit the command encoder unless it actually needs to
        let mut encoder = self.sys.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let image_id = self.sys.vello_renderer.upload_image(
            &self.sys.device,
            &self.sys.queue,
            &mut encoder,
            &pixmap
        );

        log::info!("Uploaded image, got ImageId: {:?}", image_id);

        // Store the ImageId in the node
        self.nodes[i].imageref = Some(ImageRef::Raster {
            image_id,
            original_size: Xy::new(width as f32, height as f32),
        });



        self.nodes[i].last_static_image_ptr = Some(image_pointer);

        return self;
    }

    pub(crate) fn set_static_svg(&mut self, i: NodeI, svg_data: &'static [u8]) -> &mut Self {
        let svg_pointer: *const u8 = svg_data.as_ptr();

        if let Some(last_pointer) = self.nodes[i].last_static_image_ptr {
            if svg_pointer == last_pointer {
                return self;
            }
        }

        // Parse SVG using PicoSvg
        let svg_str = std::str::from_utf8(svg_data).expect("Invalid UTF-8 in SVG data");
        let pico_svg = vello_common::pico_svg::PicoSvg::load(svg_str, 1.0)
            .expect("Failed to parse SVG");

        log::info!("Parsed SVG: {}x{}", pico_svg.size.width, pico_svg.size.height);

        // Store the parsed SVG items in central storage
        let svg_index = self.sys.svg_storage.len();
        self.sys.svg_storage.push(pico_svg.items);

        // Store reference in the node
        self.nodes[i].imageref = Some(ImageRef::Svg {
            svg_index,
            original_size: Xy::new(pico_svg.size.width as f32, pico_svg.size.height as f32),
        });

        self.nodes[i].last_static_image_ptr = Some(svg_pointer);

        return self;
    }
}

pub type StateId = Id;
