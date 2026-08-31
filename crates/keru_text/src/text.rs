use crate::*;
use slab::Slab;
use std::ops::DerefMut;
use std::ptr::NonNull;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use winit::{event::{KeyEvent, Modifiers, MouseButton, WindowEvent}, keyboard::{Key, NamedKey}, window::Window};
use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;
use std::sync::{Arc, Weak};
use winit::window::WindowId;
use parley::{FontContext, LayoutContext};

const DEFAULT_STYLE_KEY: usize = 0;

const MULTICLICK_DELAY: f64 = 0.4;
const MULTICLICK_TOLERANCE_SQUARED: f64 = 26.0;

/// Direction for cross-box selection extension.
#[derive(Debug, Clone, Copy)]
enum SelectionDirection {
    Forward,
    Backward,
}

#[derive(Debug, Clone)]
pub(crate) struct WindowInfo {
    pub(crate) window_id: WindowId,
    pub(crate) dimensions: (f32, f32),
    pub(crate) prepared: bool,
    pub(crate) scale_factor: f64,
}

#[derive(Debug)]
pub(crate) struct StyleInner {
    pub(crate) text_style: TextStyle2,
    pub(crate) text_edit_style: TextEditStyle,
}


/// Centralized struct that holds collections of [`TextBox`]es, [`TextEdit`]s, [`TextStyle2`]s.
pub struct Text {
    pub(crate) text_boxes: Slab<TextBox>,
    pub(crate) text_edits: Slab<TextEdit>,

    // Box to have a stable address for the backref pointers
    pub(crate) shared: Box<Shared>,

    pub(crate) input_state: TextInputState,

    pub(crate) scroll_animations: Vec<ScrollAnimation>,

    pub(crate) renderer: TextRenderer,

    pub(crate) encoder: Option<wgpu::CommandEncoder>,

    /// Internal buffer for collecting selected text across multiple boxes.
    selected_text_buffer: String,
}

/// Precomputed hit test data for a single text box or text edit, stored in `Shared::hit_tests`.
pub(crate) struct HitTestEntry {
    pub(crate) any_box: AnyBox,
    pub(crate) depth: f32,
    pub(crate) selectable: bool,
    pub(crate) hidden: bool,
    pub(crate) window_id: Option<WindowId>,
    pub(crate) shape: HitTestShape,
}

/// The geometric data needed to do a hit check, as an enum so that non-rotated
/// boxes can store a pre-transformed axis-aligned bounding box instead of the
/// full transform.
pub(crate) enum HitTestShape {
    /// Non-rotated box: screen-space AABB ready for a simple range check.
    Axis {
        min_x: f32,
        max_x: f32,
        min_y: f32,
        max_y: f32,
    },
    /// Rotated box: full transform data for the cursor_to_local calculation.
    Rotated {
        transform: Transform2D,
        group_transform_index: Option<GroupTransformHandle>,
        local_min_x: f32,
        local_max_x: f32,
        local_min_y: f32,
        local_max_y: f32,
    },
}

/// Compute the `HitTestShape` from a box's geometric properties.
/// When `rotation == 0` this pre-transforms all corners and stores a plain AABB.
pub(crate) fn compute_hit_test_shape(
    transform: Transform2D,
    group_transform_index: Option<GroupTransformHandle>,
    width: f32,
    height: f32,
    explicit_hitbox: Option<(f32, f32, f32, f32)>,
    group_transforms: &GpuSlab<GroupTransform>,
) -> HitTestShape {
    let (local_min_x, local_max_x, local_min_y, local_max_y) =
        if let Some((hx0, hy0, hx1, hy1)) = explicit_hitbox {
            (hx0, hx1, hy0, hy1)
        } else {
            (-(X_TOLERANCE as f32), width + X_TOLERANCE as f32, 0.0, height)
        };

    if transform.rotation == 0.0 {
        let s = transform.scale;
        let tx = transform.translation.0;
        let ty = transform.translation.1;

        let mut min_x = local_min_x * s + tx;
        let mut max_x = local_max_x * s + tx;
        let mut min_y = local_min_y * s + ty;
        let mut max_y = local_max_y * s + ty;

        if let Some(handle) = group_transform_index {
            let g = &group_transforms[handle.0];
            if g.scale != 0.0 {
                min_x = g.offset[0] + min_x * g.scale;
                max_x = g.offset[0] + max_x * g.scale;
                min_y = g.offset[1] + min_y * g.scale;
                max_y = g.offset[1] + max_y * g.scale;
            }
        }

        if min_x > max_x { std::mem::swap(&mut min_x, &mut max_x); }
        if min_y > max_y { std::mem::swap(&mut min_y, &mut max_y); }

        HitTestShape::Axis { min_x, max_x, min_y, max_y }
    } else {
        HitTestShape::Rotated {
            transform,
            group_transform_index,
            local_min_x,
            local_max_x,
            local_min_y,
            local_max_y,
        }
    }
}

fn hit_test_shape_check(shape: &HitTestShape, cursor_pos: (f64, f64), group_transforms: &GpuSlab<GroupTransform>) -> bool {
    let cx = cursor_pos.0 as f32;
    let cy = cursor_pos.1 as f32;
    match shape {
        HitTestShape::Axis { min_x, max_x, min_y, max_y } => {
            cx >= *min_x && cx <= *max_x && cy >= *min_y && cy <= *max_y
        }
        HitTestShape::Rotated { transform, group_transform_index, local_min_x, local_max_x, local_min_y, local_max_y } => {
            let mut pos = euclid::Point2D::new(cx, cy);
            if let Some(handle) = group_transform_index {
                let g = &group_transforms[handle.0];
                if g.scale != 0.0 {
                    pos.x = (pos.x - g.offset[0]) / g.scale;
                    pos.y = (pos.y - g.offset[1]) / g.scale;
                }
            }
            let inv = transform.inverse().unwrap_or(Transform2D::identity());
            let local = inv.transform_point(pos);
            local.x >= *local_min_x && local.x <= *local_max_x && local.y >= *local_min_y && local.y <= *local_max_y
        }
    }
}

/// Data that TextBoxMut and similar things need to have a reference to.
pub(crate) struct Shared {
    pub(crate) render_data: RenderData,

    pub styles: Slab<StyleInner>,
    pub(crate) hit_tests: Slab<HitTestEntry>,
    pub scrolled: bool,
    pub focused: Option<AnyBox>,

    /// Text boxes that are part of the current multi-box selection.
    /// When non-empty, selection rects should be drawn for all boxes in this list.
    pub multi_box_selection: Vec<usize>,

    /// The anchor point for cross-box selection: (box_key, local_x, local_y).
    /// Set when clicking on a text box, used when shift-clicking across linked boxes.
    pub cross_box_selection_anchor: Option<usize>,

    /// The box that currently holds the keyboard selection cursor for cross-box keyboard selection.
    /// None means cursor is in the focused/anchor box.
    pub cross_box_cursor_key: Option<usize>,

    pub windows: Vec<WindowInfo>,
    /// A UI scale factor that multiplies with each window's real scale factor. Useful for higher-resolution screenshots.
    pub explicit_scale_factor: f64,
    pub layout_cx: LayoutContext<ColorBrush>,
    pub font_cx: FontContext,

    pub decorations_dirty: bool,
    
    // Throttle paste to once per frame to avoid layout rebuild spam.
    pub pasted_this_frame: bool,
    
    pub current_event_number: u64,

    // Cursor blink state
    pub cursor_blink_start: Option<Instant>,
    pub cursor_blink_animation_currently_visible: bool,
    pub cursor_blink_waker: Option<CursorBlinkWaker>,

    pub window: Option<Weak<Window>>,

    pub(crate) scratch_quads: Vec<GlyphQuad>,

    pub(crate) changed_style_keys: Vec<usize>,
    
    pub(crate) pending_event_response: EventResponse,
}

impl Shared {
    pub(crate) fn refocus(&mut self, new_focus: Option<AnyBox>) {
        let focus_changed = new_focus != self.focused;

        if focus_changed {
            self.pending_event_response.focus_lost = self.focused;
            self.pending_event_response.focus_gained = new_focus;

            // Clear multi-box selection and cross-box state when focus changes
            self.multi_box_selection.clear();
            self.cross_box_selection_anchor = None;
            self.cross_box_cursor_key = None;

            // Enable/disable IME based on whether a text edit is focused
            // Todo: what if the user wants to do his own IME stuff?
            if let Some(weak_window) = &self.window {
                if let Some(window) = weak_window.upgrade() {
                    let ime_allowed = matches!(new_focus, Some(AnyBox::TextEdit(_)));
                    window.set_ime_allowed(ime_allowed);
                }
            }
        }

        self.focused = new_focus;

        if focus_changed {
            self.reset_cursor_blink();
        }
    }

    pub(crate) fn update_blink_timer(&mut self) {
        if let Some(start_time) = self.cursor_blink_start {
            let elapsed = Instant::now().duration_since(start_time);
            let blink_period = Duration::from_millis(CURSOR_BLINK_TIME_MILLIS);
            let blinked_out = (elapsed.as_millis() / blink_period.as_millis()) % 2 == 0;
            let changed = blinked_out != self.cursor_blink_animation_currently_visible;

            self.cursor_blink_animation_currently_visible = blinked_out;

            if changed {
                self.decorations_dirty = true;
            }
        }
    }

    pub(crate) fn reset_cursor_blink(&mut self) {
        if let Some(AnyBox::TextEdit(_)) = self.focused {
            // todo: reorganize some stuff and also check that the selection is collapsed?
            self.cursor_blink_start = Some(Instant::now());
            self.cursor_blink_animation_currently_visible = true;
            self.decorations_dirty = true;

            if let Some(timer) = &self.cursor_blink_waker {
                timer.start();
            }
        } else {
            self.cursor_blink_start = None;
            self.decorations_dirty = true;
            if let Some(waker) = &self.cursor_blink_waker {
                waker.stop();
            }
        }
    }
    
    pub(crate) fn stop_cursor_blink(&mut self) {
        self.cursor_blink_start = None;
        if let Some(waker) = &self.cursor_blink_waker {
            waker.stop();
        }
    }
}

/// Handle for a text edit box.
///
/// Obtained when creating a text edit box with [`Text::add_text_edit()`].
/// 
/// Use with [`Text::get_text_edit()`] to get a reference to the corresponding [`TextEdit`]. 
#[derive(Debug)]
pub struct TextEditHandle {
    pub(crate) key: usize,
}

/// Cloneable handle for a text edit box.
/// 
/// Use with [`Text::try_get_text_edit()`] to get an optional reference to the corresponding [`TextBox`].
/// 
/// Because this handle is not unique, the text box that it refers to can be removed while the handle is still live. This is why [`Text::try_get_text_edit()`] returns an `Option`.
#[derive(Debug, Clone, Copy)]
pub struct ClonedTextEditHandle {
    pub(crate) key: usize,
}

/// Handle for a text box.
/// 
/// Obtained when creating a text box with [`Text::add_text_box()`].
/// 
/// Use with [`Text::get_text_box()`] to get a reference to the corresponding [`TextBox`].
#[derive(Debug)]
pub struct TextBoxHandle {
    pub(crate) key: usize,
}

/// Cloneable handle for a text box.
/// 
/// Use with [`Text::try_get_text_box()`] to get an optional reference to the corresponding [`TextBox`].
/// 
/// Because this handle is not unique, the text box that it refers to can be removed while the handle is still live. This is why [`Text::try_get_text_box()`] returns an `Option`.
#[derive(Debug, Clone, Copy)]
pub struct ClonedTextBoxHandle {
    pub(crate) key: usize,
}

impl TextBoxHandle {
    /// Get a non-unique handle cloned handle from this handle.
    pub fn to_cloned(&self) -> ClonedTextBoxHandle {
        ClonedTextBoxHandle { key: self.key }
    }
}

impl TextEditHandle {
    /// Get a non-unique handle cloned handle from this handle.
    pub fn to_cloned(&self) -> ClonedTextEditHandle {
        ClonedTextEditHandle { key: self.key }
    }
}


#[cfg(feature = "panic_on_handle_drop")]
impl Drop for TextEditHandle {
    fn drop(&mut self) {
        panic!(
            "TextEditHandle was dropped without being consumed! \
            This means that the corresponding text edit wasn't removed. To avoid leaking it, you should call Text::remove_text_edit(handle). \
            If you're intentionally leaking this text edit, you can use \
            std::mem::forget(handle) to skip the handle's drop() call and avoid this panic. \
            You can also disable this check by disabling the \"panic_on_handle_drop\" feature in Cargo.toml."
        );
    }
}

#[cfg(feature = "panic_on_handle_drop")]
impl Drop for TextBoxHandle {
    fn drop(&mut self) {
        panic!(
            "TextBoxHandle was dropped without being consumed! \
            This means that the corresponding text box wasn't removed. To avoid leaking it, you should call Text::remove_text_box(handle). \
            If you're intentionally leaking this text box, you can use \
            std::mem::forget(handle) to skip the handle's drop() call and avoid this panic. \
            You can also disable this check by disabling the \"panic_on_handle_drop\" feature in Cargo.toml."
        );
    }
}


/// Handle for a text style. Use with Text methods to apply styles to text.
#[derive(Debug, Clone, Copy)]
pub struct StyleHandle {
    pub(crate) key: usize,
}
impl StyleHandle {
    pub(crate) fn sneak_clone(&self) -> Self {
        Self { key: self.key }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LastClickInfo {
    pub(crate) time: Instant,
    pub(crate) pos: (f64, f64),
    pub(crate) focused: Option<AnyBox>,
}

#[derive(Debug, Clone)]
pub(crate) struct MouseState {
    pub pointer_down: bool,
    pub cursor_pos: (f64, f64),
    pub last_click_info: Option<LastClickInfo>,
    pub click_count: u32,
}

impl MouseState {
    pub fn new() -> Self {
        Self {
            pointer_down: false,
            cursor_pos: (0.0, 0.0),
            last_click_info: None,
            click_count: 0,
        }
    }
}

/// A non-owning reference to either a `TextBox` or a `TextEditBox`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnyBox {
    /// Text edit box
    TextEdit(usize),
    /// Text box
    TextBox(usize),
}

#[derive(Debug, Clone)]
pub(crate) struct TextInputState {
    pub(crate) mouse: MouseState,
    pub(crate) modifiers: Modifiers,
}

impl TextInputState {
    pub fn new() -> Self {
        Self {
            mouse: MouseState::new(),
            modifiers: Modifiers::default(),
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = *modifiers;
            }
            WindowEvent::CursorMoved { position, .. } => {
                let cursor_pos = (position.x, position.y);
                self.mouse.cursor_pos = cursor_pos;
            },

            WindowEvent::MouseInput { state, .. } => {
                self.mouse.pointer_down = state.is_pressed();
            },
            _ => {}
        }
    }
}

fn apply_shift_nav_op(
    selection: &mut parley::Selection,
    layout: &parley::Layout<ColorBrush>,
    event: &KeyEvent,
    action_mod: bool,
) -> Option<bool> {
    match &event.logical_key {
        Key::Named(NamedKey::ArrowLeft) => {
            if action_mod { selection.select_word_left(layout); } else { selection.select_left(layout); }
            Some(false)
        }
        Key::Named(NamedKey::ArrowRight) => {
            if action_mod { selection.select_word_right(layout); } else { selection.select_right(layout); }
            Some(true)
        }
        Key::Named(NamedKey::ArrowUp) => {
            selection.select_up(layout);
            Some(false)
        }
        Key::Named(NamedKey::ArrowDown) => {
            selection.select_down(layout);
            Some(true)
        }
        Key::Named(NamedKey::Home) => {
            selection.select_to_line_start(layout);
            Some(false)
        }
        Key::Named(NamedKey::End) => {
            selection.select_to_line_end(layout);
            Some(true)
        }
        _ => None,
    }
}


impl Text {
    /// Create a new Text instance with a GPU renderer.
    pub fn new(device: &Device, queue: &Queue, format: TextureFormat) -> Self {
        Self::new_with_params(device, queue, format, None, TextRendererParams::default())
    }

    /// Create a new Text instance with custom renderer parameters.
    pub fn new_with_params(
        device: &Device,
        queue: &Queue,
        format: TextureFormat,
        depth_stencil: Option<DepthStencilState>,
        params: TextRendererParams,
    ) -> Self {
        let mut styles = Slab::with_capacity(10);
        let key = styles.insert(StyleInner {
            text_style: original_default_style(),
            text_edit_style: TextEditStyle::default(),
        });
        debug_assert_eq!(key, DEFAULT_STYLE_KEY);

        let atlas_size = params.atlas_page_size.size(device);
        let mut render_data = RenderData::new(device, queue, atlas_size);
        render_data.set_srgb(format.is_srgb());

        let renderer = TextRenderer::new_with_params(
            device.clone(),
            queue.clone(),
            format,
            depth_stencil,
            atlas_size,
            &render_data.box_data,
            &render_data.group_transforms,
            &render_data.glyph_quads,
        );

        Self {
            text_boxes: Slab::with_capacity(10),
            text_edits: Slab::with_capacity(10),
            input_state: TextInputState::new(),
            scroll_animations: Vec::with_capacity(4),
            encoder: None,
            renderer,

            selected_text_buffer: String::with_capacity(25),

            shared: Box::new(Shared {
                render_data,
                windows: Vec::with_capacity(1),
                explicit_scale_factor: 1.0,
                styles,
                hit_tests: Slab::with_capacity(10),
                scrolled: true,
                focused: None,
                multi_box_selection: Vec::with_capacity(4),
                cross_box_selection_anchor: None,
                cross_box_cursor_key: None,
                layout_cx: LayoutContext::new(),
                font_cx: FontContext::new(),
                decorations_dirty: false,
                pasted_this_frame: false,
                current_event_number: 1,
                cursor_blink_start: None,
                cursor_blink_animation_currently_visible: false,
                cursor_blink_waker: None,
                window: None,
                scratch_quads: Vec::with_capacity(40),
                changed_style_keys: Vec::with_capacity(2),

                pending_event_response: EventResponse::default()
            }),
        }
    }

    /// Setup automatic cursor blink wakeup for applications that pause their event loops.
    ///
    /// `window` is used to wake up the `winit` event loop automatically when it needs to redraw a blinking cursor.
    /// It is also used to enable/disable IME when a text edit box gains or loses focus.
    ///
    /// In applications that don't pause their event loops, like games, there is no need to call this method.
    ///
    /// You can also handle cursor wakeups manually in your winit event loop with winit's `ControlFlow::WaitUntil` and [`Text::time_until_next_cursor_blink`]. See the `event_loop_smart.rs` example.
    pub fn set_auto_wakeup(&mut self, window: Arc<Window>) {
        self.shared.cursor_blink_waker = Some(CursorBlinkWaker::new(Arc::downgrade(&window)));
        self.shared.window = Some(Arc::downgrade(&window));
    }

    /// Load all the renderer data to the gpu.
    ///
    /// Useful only for custom rendering.
    pub fn load_to_gpu(&mut self) {
        let box_data_reallocated = self.shared.render_data.box_data.load_to_gpu();
        let group_transforms_reallocated = self.shared.render_data.group_transforms.load_to_gpu();
        let mut needs_bind_group_recreate = box_data_reallocated || group_transforms_reallocated;

        // Update uniform buffer if needed
        if self.shared.render_data.needs_params_sync {
            let bytes: &[u8] = bytemuck::cast_slice(std::slice::from_ref(&self.shared.render_data.params));
            self.renderer.queue.write_buffer(&self.renderer.params_buffer, 0, bytes);
            self.shared.render_data.needs_params_sync = false;
        }

        // Rebuild texture arrays if needed
        if self.shared.render_data.needs_texture_array_rebuild {
            self.renderer.rebuild_texture_arrays(&mut self.shared.render_data);
            self.shared.render_data.needs_texture_array_rebuild = false;
            needs_bind_group_recreate = true;
        } else {
            self.renderer.update_texture_arrays(&mut self.shared.render_data);
        }

        // Sync quads buffer bind group if the buffer was reallocated during prepare.
        if self.shared.render_data.glyph_quads.take_reallocated() {
            needs_bind_group_recreate = true;
        }
        self.shared.decorations_dirty = false;

        if needs_bind_group_recreate {
            self.renderer.recreate_bind_group(&self.shared.render_data);
        }
    }

    /// Render all prepared text using the provided render pass.
    pub fn render(&mut self, pass: &mut RenderPass) {
        self.load_to_gpu();
        self.renderer.render(pass, &self.shared.render_data);
    }

    /// Get render statistics from the last frame.
    ///
    /// Call this after `prepare_all()` and `load_to_gpu()` (or `render()`) to see
    /// what work was done and whether the optimizations are working.
    ///
    /// Only available in debug builds.
    #[cfg(debug_assertions)]
    pub fn render_stats(&self) -> &RenderStats {
        &self.shared.render_data.stats
    }

    /// Add a text box and return a handle.
    /// 
    /// The handle can be used with [`Text::get_text_box()`] to get a reference to the [`TextBox`] that was added.
    /// 
    /// The [`TextBox`] must be manually removed by calling [`Text::remove_text_box()`].
    /// 
    /// `text` can be a `String`, a `&'static str`, or a `Cow<'static, str>`.
    #[must_use]
    pub fn add_text_box(&mut self, text: impl Into<Cow<'static, str>>, pos: Option<(f64, f64)>, size: (f32, f32), depth: f32) -> TextBoxHandle {
        let pos = pos.unwrap_or((f64::MIN, f64::MIN));
        let shared_backref: NonNull<Shared> = NonNull::new(self.shared.deref_mut()).unwrap();
        let mut text_box = TextBox::new(text, pos, size, depth, DEFAULT_STYLE_KEY, shared_backref);

        let box_data_i = self.shared.render_data.box_data.insert(BoxGpu::zeroed());
        text_box.render_data_info.box_index = box_data_i;

        let key = self.text_boxes.insert(text_box);
        let handle = TextBoxHandle { key };
        // Fill in the local copy of the key.
        self.get_text_box_mut(&handle).key = key;
        let tb = &self.text_boxes[key];
        let shape = compute_hit_test_shape(tb.transform, tb.group_transform_index, tb.width, tb.height, tb.explicit_hitbox, &self.shared.render_data.group_transforms);
        let hit_test_key = self.shared.hit_tests.insert(HitTestEntry {
            any_box: AnyBox::TextBox(key),
            depth: tb.depth,
            selectable: tb.selectable,
            hidden: tb.hidden,
            window_id: tb.window_id,
            shape,
        });
        self.text_boxes[key].hit_test_key = hit_test_key;
        return handle;
    }

    /// Add a text edit and return a handle.
    /// 
    /// The handle can be used with [`Text::get_text_edit()`] to get a reference to the [`TextEdit`] that was added.
    /// 
    /// The [`TextEdit`] must be manually removed by calling [`Text::remove_text_edit()`].
    #[must_use]
    pub fn add_text_edit(&mut self, text: String, pos: Option<(f64, f64)>, size: (f32, f32), depth: f32) -> TextEditHandle {
        let pos = pos.unwrap_or((f64::MIN, f64::MIN));
        let shared_backref: NonNull<Shared> = NonNull::new(self.shared.deref_mut()).unwrap();
        let mut text_edit = TextEdit::new(text, pos, size, depth, DEFAULT_STYLE_KEY, shared_backref);

        let box_data_i = self.shared.render_data.box_data.insert(BoxGpu::zeroed());
        text_edit.text_box.render_data_info.box_index = box_data_i;

        let key = self.text_edits.insert(text_edit);
        let handle = TextEditHandle { key };
        // Fill in the local copy of the key.
        self.get_text_edit_mut(&handle).text_box.key = key;
        let tb = &self.text_edits[key].text_box;
        let shape = compute_hit_test_shape(tb.transform, tb.group_transform_index, tb.width, tb.height, tb.explicit_hitbox, &self.shared.render_data.group_transforms);
        let hit_test_key = self.shared.hit_tests.insert(HitTestEntry {
            any_box: AnyBox::TextEdit(key),
            depth: tb.depth,
            selectable: tb.selectable,
            hidden: tb.hidden,
            window_id: tb.window_id,
            shape,
        });
        self.text_edits[key].text_box.hit_test_key = hit_test_key;
        return handle;
    }

    /// Add a text box for a specific window and return a handle.
    /// 
    /// This is the multi-window version of [`Text::add_text_box()`].
    /// Only use this when you have multiple windows and want to restrict this text box to a specific window.
    #[must_use]
    pub fn add_text_box_for_window(&mut self, text: impl Into<Cow<'static, str>>, pos: (f64, f64), size: (f32, f32), depth: f32, window_id: WindowId) -> TextBoxHandle {
        let shared_backref: NonNull<Shared> = NonNull::new(self.shared.deref_mut()).unwrap();
        let mut text_box = TextBox::new(text, pos, size, depth, DEFAULT_STYLE_KEY, shared_backref);
        text_box.window_id = Some(window_id);
        let key = self.text_boxes.insert(text_box);
        let handle = TextBoxHandle { key };
        // Fill in the local copy of the key.
        self.get_text_box_mut(&handle).key = key;
        let tb = &self.text_boxes[key];
        let shape = compute_hit_test_shape(tb.transform, tb.group_transform_index, tb.width, tb.height, tb.explicit_hitbox, &self.shared.render_data.group_transforms);
        let hit_test_key = self.shared.hit_tests.insert(HitTestEntry {
            any_box: AnyBox::TextBox(key),
            depth: tb.depth,
            selectable: tb.selectable,
            hidden: tb.hidden,
            window_id: tb.window_id,
            shape,
        });
        self.text_boxes[key].hit_test_key = hit_test_key;
        return handle;
    }

    /// Add a text edit for a specific window and return a handle.
    /// 
    /// This is the multi-window version of [`Text::add_text_edit()`].
    /// Only use this when you have multiple windows and want to restrict this text edit to a specific window.
    #[must_use]
    pub fn add_text_edit_for_window(&mut self, text: String, pos: (f64, f64), size: (f32, f32), depth: f32, window_id: WindowId) -> TextEditHandle {
        let shared_backref: NonNull<Shared> = NonNull::new(self.shared.deref_mut()).unwrap();
        let mut text_edit = TextEdit::new(text, pos, size, depth, DEFAULT_STYLE_KEY, shared_backref);
        text_edit.text_box.window_id = Some(window_id);
        let key = self.text_edits.insert(text_edit);
        let handle = TextEditHandle { key };
        // Fill in the local copy of the key.
        self.get_text_edit_mut(&handle).text_box.key = key;
        let tb = &self.text_edits[key].text_box;
        let shape = compute_hit_test_shape(tb.transform, tb.group_transform_index, tb.width, tb.height, tb.explicit_hitbox, &self.shared.render_data.group_transforms);
        let hit_test_key = self.shared.hit_tests.insert(HitTestEntry {
            any_box: AnyBox::TextEdit(key),
            depth: tb.depth,
            selectable: tb.selectable,
            hidden: tb.hidden,
            window_id: tb.window_id,
            shape,
        });
        self.text_edits[key].text_box.hit_test_key = hit_test_key;
        return handle;
    }




    /// Get a mutable reference to a text edit.
    /// 
    /// `handle` is the handle that was returned when first creating the text edit with [`Text::add_text_edit()`] or similar functions.
    ///    
    /// This is a fast lookup operation that does not require any hashing.
    pub fn get_text_edit_mut(&mut self, handle: &TextEditHandle) -> &mut TextEdit {
        return &mut self.text_edits[handle.key];
    }

    /// Get a reference to a text edit.
    /// 
    /// `handle` is the handle that was returned when first creating the text edit with [`Text::add_text_edit()`] or similar functions.
    ///    
    /// This is a fast lookup operation that does not require any hashing.
    pub fn get_text_edit(&self, handle: &TextEditHandle) -> &TextEdit {
        return &self.text_edits[handle.key];
    }

    /// Returns a text edit if it exists, or `None` if it has been removed.
    pub fn try_get_text_edit(&self, handle: &ClonedTextEditHandle) -> Option<&TextEdit> {
        return self.text_edits.get(handle.key);
    }

    /// Returns a mutable text edit if it exists, or `None` if it has been removed.
    pub fn try_get_text_edit_mut(&mut self, handle: &ClonedTextEditHandle) -> Option<&mut TextEdit> {
        return self.text_edits.get_mut(handle.key);
    }

    /// Adds a new text style and returns a handle to it.
    #[must_use]
    pub fn add_style(&mut self, text_style: TextStyle2, text_edit_style: Option<TextEditStyle>) -> StyleHandle {
        let text_edit_style = text_edit_style.unwrap_or_default();
        let key = self.shared.styles.insert(StyleInner {
            text_style,
            text_edit_style,
        });
        StyleHandle { key }
    }

    /// Returns a reference to the text style.
    pub fn get_text_style(&self, handle: &StyleHandle) -> &TextStyle2 {
        &self.shared.styles[handle.key].text_style
    }

    /// Returns a mutable reference to the text style.
    pub fn get_text_style_mut(&mut self, handle: &StyleHandle) -> &mut TextStyle2 {
        self.shared.changed_style_keys.push(handle.key);
        &mut self.shared.styles[handle.key].text_style
    }

    /// Returns a reference to the text edit style.
    pub fn get_text_edit_style(&self, handle: &StyleHandle) -> &TextEditStyle {
        &self.shared.styles[handle.key].text_edit_style
    }

    /// Returns a mutable reference to the text edit style.
    pub fn get_text_edit_style_mut(&mut self, handle: &StyleHandle) -> &mut TextEditStyle {
        self.shared.changed_style_keys.push(handle.key);
        &mut self.shared.styles[handle.key].text_edit_style
    }

    /// Returns a reference to the default text style.
    pub fn get_default_text_style(&self) -> &TextStyle2 {
        &self.shared.styles[DEFAULT_STYLE_KEY].text_style
    }

    /// Returns a mutable reference to the default text style.
    pub fn get_default_text_style_mut(&mut self) -> &mut TextStyle2 {
        let default_style_key = DEFAULT_STYLE_KEY;
        self.shared.changed_style_keys.push(default_style_key);
        &mut self.shared.styles[default_style_key].text_style
    }

    /// Returns a reference to the default text edit style.
    pub fn get_default_text_edit_style(&self) -> &TextEditStyle {
        &self.shared.styles[DEFAULT_STYLE_KEY].text_edit_style
    }

    /// Returns a mutable reference to the default text edit style.
    pub fn get_default_text_edit_style_mut(&mut self) -> &mut TextEditStyle {
        let default_style_key = DEFAULT_STYLE_KEY;
        self.shared.changed_style_keys.push(default_style_key);
        &mut self.shared.styles[default_style_key].text_edit_style
    }

    /// Returns the original default text style.
    pub fn original_default_style(&self) -> TextStyle2 {
        original_default_style()
    }

    /// Remove a text box.
    /// 
    /// `handle` is the handle that was returned when first creating the text box with [`Text::add_text_box()`].
    pub fn remove_text_box(&mut self, handle: TextBoxHandle) {
        if let Some(AnyBox::TextBox(key)) = self.shared.focused {
            if key == handle.key {
                self.shared.focused = None;
            }
        }

        let text_box = self.text_boxes.remove(handle.key);

        let box_data_i = text_box.render_data_info.box_index;
        self.shared.render_data.box_data.remove(box_data_i);
        self.shared.hit_tests.remove(text_box.hit_test_key);

        std::mem::forget(handle);
    }


    /// Remove a text edit.
    ///
    /// `handle` is the handle that was returned when first creating the text edit with [`Text::add_text_edit()`] or similar functions.
    pub fn remove_text_edit(&mut self, handle: TextEditHandle) {
        if let Some(AnyBox::TextEdit(i)) = self.shared.focused {
            if i == handle.key {
                self.shared.focused = None;
            }
        }

        let text_edit = self.text_edits.remove(handle.key);

        let box_data_i = text_edit.text_box.render_data_info.box_index;
        self.shared.render_data.box_data.remove(box_data_i);
        self.shared.hit_tests.remove(text_edit.text_box.hit_test_key);

        std::mem::forget(handle);
    }

    /// Remove a text style.
    ///
    /// If any text boxes are set to this style, they will revert to the default style.
    pub fn remove_style(&mut self, handle: StyleHandle) {
        self.shared.styles.remove(handle.key);
    }

    /// Insert a group transform and return a handle.
    ///
    /// Group transforms can be shared across multiple text boxes and are applied
    /// after the per-box transform.
    #[must_use]
    pub fn insert_group_transform(&mut self, transform: GroupTransform) -> GroupTransformHandle {
        let index = self.shared.render_data.group_transforms.insert(transform);
        GroupTransformHandle(index)
    }

    /// Remove a group transform.
    ///
    /// Text boxes using this transform should have their group_transform_index cleared first.
    pub fn remove_group_transform(&mut self, handle: GroupTransformHandle) {
        self.shared.render_data.group_transforms.remove(handle.0);
    }

    /// Update a group transform.
    ///
    /// All text boxes using this transform will be affected.
    pub fn update_group_transform(&mut self, handle: GroupTransformHandle, transform: GroupTransform) {
        self.shared.render_data.group_transforms[handle.0] = transform;
    }

    /// Get the value of a group transform.
    pub fn get_group_transform(&self, handle: GroupTransformHandle) -> GroupTransform {
        self.shared.render_data.group_transforms[handle.0]
    }


    /// Returns the range of decoration quads (selection rects + cursor) in the shared glyph quad buffer.
    ///
    /// Decoration quads are shared across all text edits. Call this once per frame and push instances
    /// for the returned range alongside the per-box glyph quads.
    pub fn decoration_quad_range(&self) -> (usize, usize) {
        match self.shared.render_data.decoration_quad_handle {
            Some(handle) => {
                let start = handle.vec_index(gpu_heap::CHUNK_SIZE);
                let size = handle.size as usize;
                (start, start + size)
            }
            None => (0, 0),
        }
    }

    /// Layout and rasterize all text belonging to a window, prepare the render data.
    pub fn prepare_all_for_window(&mut self, window: &Window) {
        let window_id = window.id();
        let window_size = window.inner_size();
        let (width, height) = (window_size.width as f32, window_size.height as f32);

        self.prepare_all_impl(window_id, (width, height));
    }

    /// Layout and rasterize all text, prepare the render data.
    ///
    /// This function is for single-window applications only. For multi-window, use [`Text::prepare_all_for_window`].
    ///
    /// [`Text`] keeps track of all changes to text boxes internally. So this function can be called multiple times in same frame without issues, if needed.
    pub fn prepare_all(&mut self) {
        let res = self.shared.windows.first().map(|w| (w.window_id, w.dimensions));

        // This is what we would want to do:
        // let (window_id, window_size) = res.expect("Text::prepare_all didn't register any windows, are you calling Text::handle_events?");
        // However, it seems that winit continues to give us RedrawRequested events even after the CloseRequested event, even if we calling event_loop.exit()?
        // But we unregister windows on CloseRequested.
        // For this reason, it seems that we have to accept that prepare_all might be called when no windows are around and silently do nothing.
        let Some((window_id, window_size)) = res else {
            // Even if there are no windows, we should reset the change flags
            // so they don't stay stuck at true
            return;
        };

        self.prepare_all_impl(window_id, window_size);
    }

    pub(crate) fn prepare_all_impl(&mut self, window_id: WindowId, window_size: (f32, f32)) {
        let mut encoder = self.encoder.take().unwrap_or_else(|| {
            self.renderer.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default())
        });

        #[cfg(debug_assertions)] {
            self.shared.render_data.stats = RenderStats::default();
        }

        // this apparently avoids some sort of cache miss where the contains() keeps loading the empty vec for no reason.
        let styles_changed = ! self.shared.changed_style_keys.is_empty();

        self.shared.pasted_this_frame = false;
        self.shared.render_data.update_resolution(window_size.0, window_size.1);

        // todo: an extra loop just for this?
        for (_key, text_edit) in self.text_edits.iter_mut() {
            text_edit.text_changed = false;
        }

        self.shared.render_data.advance_frame();

        for (_, text_edit) in self.text_edits.iter_mut() {
            if styles_changed && self.shared.changed_style_keys.contains(&text_edit.text_box.style.key) {
                text_edit.text_box.needs_reshape = true;
            }
            if ! text_edit.text_box.hidden() {
                self.shared.render_data.prepare_text_edit_layout(text_edit, &mut self.shared.scratch_quads, &mut encoder);
            }
        }

        for (_, mut text_box) in self.text_boxes.iter_mut() {
            if styles_changed && self.shared.changed_style_keys.contains(&text_box.style.key) {
                text_box.needs_reshape = true;
            }
            if ! text_box.hidden() {
                self.shared.render_data.prepare_text_box_layout(&mut text_box, &mut self.shared.scratch_quads, 4, &mut encoder);
            }
        }

        self.prepare_decoration_quads(&mut encoder);

        let should_clear_flags = {
            if let Some(window_info) = self.shared.windows.iter_mut().find(|info| info.window_id == window_id) {
                window_info.prepared = true;
            }
            self.shared.windows.iter().all(|info| info.prepared)
        };

        if should_clear_flags {
            self.shared.render_data.glyph_quads.finish_belt();
            self.renderer.queue.submit(Some(encoder.finish()));
            self.shared.render_data.glyph_quads.recall_belt();

            // Reset all windows to unprepared for next frame
            for window_info in &mut self.shared.windows {
                window_info.prepared = false;
            }

            self.shared.changed_style_keys.clear();

            self.shared.scrolled = self.get_max_animation_duration().is_some();
        } else {
            self.encoder = Some(encoder);
        }
    }

    fn prepare_decoration_quads(&mut self, encoder: &mut wgpu::CommandEncoder) {
        if !self.shared.decorations_dirty {
            return;
        }
        self.shared.decorations_dirty = false;

        let scratch = &mut self.shared.scratch_quads;
        scratch.clear();
        let selection_color = 0x33_33_ff_aa;

        for &key in &self.shared.multi_box_selection {
            if let Some(text_box) = self.text_boxes.get(key) {
                let box_index = text_box.render_data_info.box_index as u32;
                text_box.selection().geometry_with(&text_box.layout, |rect, _| {
                    scratch.push(make_decoration_quad(rect, selection_color, box_index));
                });
            }
        }

        if let Some(AnyBox::TextEdit(key)) = self.shared.focused {
            if let Some(text_edit) = self.text_edits.get(key) {
                let text_box = &text_edit.text_box;
                let box_index = text_box.render_data_info.box_index as u32;

                text_box.selection().geometry_with(&text_box.layout, |rect, _| {
                    scratch.push(make_decoration_quad(rect, selection_color, box_index));
                });

                if text_box.selection().is_collapsed() {
                    let cursor_color = if self.shared.cursor_blink_animation_currently_visible { CURSOR_COLOR } else { 0x00_00_00_00 };
                    let cursor_rect = text_box.selection().focus().geometry(&text_box.layout, CURSOR_WIDTH);
                    scratch.push(make_decoration_quad(cursor_rect, cursor_color, box_index));
                }
            }
        }

        let scratch = &self.shared.scratch_quads;
        let rd = &mut self.shared.render_data;
        rd.glyph_quads.allocate_or_grow_and_write(&mut rd.decoration_quad_handle, scratch, 20, encoder);
    }

    /// Returns `true` if the event was consumed by a text area.
    pub fn handle_event(&mut self, event: &WindowEvent, window: &Window) -> EventResponse {
        self.shared.pending_event_response = EventResponse::default();

        self.shared.current_event_number += 1;
        
        self.input_state.handle_event(event);

        // Register the window if not already there.
        if let WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } | WindowEvent::RedrawRequested = event {
            if self.shared.windows.iter().find(|w_info| w_info.window_id == window.id()).is_none() {
                self.shared.windows.push(WindowInfo { 
                    window_id: window.id(), 
                    dimensions: (window.inner_size().width as f32, window.inner_size().height as f32), 
                    prepared: false,
                    scale_factor: window.scale_factor(),
                });
            }
        }
        
        if let WindowEvent::Focused(focused) = event {
            if *focused {
                // self.shared.cursor_blink_animation_currently_visible = true;
                self.shared.reset_cursor_blink();
            } else {
                self.shared.cursor_blink_animation_currently_visible = false;
                // Should rerender to hide the selection rectangles
                self.shared.stop_cursor_blink();
                self.shared.decorations_dirty = true;
            }
        }

        if let WindowEvent::CloseRequested | WindowEvent::Destroyed = event {
            self.shared.windows.retain(|info| info.window_id != window.id());
        }

        if let WindowEvent::ScaleFactorChanged { scale_factor, inner_size_writer: _ } = event {
            let window = self.shared.windows.iter_mut().find(|info| info.window_id == window.id()).unwrap();
            window.scale_factor = *scale_factor;
        }

        if let WindowEvent::Resized(new_size) = event {
            let window = self.shared.windows.iter_mut().find(|info| info.window_id == window.id()).unwrap();
            window.dimensions = (new_size.width as f32, new_size.height as f32);
        }

        // update smooth scrolling animations
        if let WindowEvent::RedrawRequested = event {
            self.shared.update_blink_timer();

            let animation_updated = self.update_smooth_scrolling();
            if animation_updated {
                self.shared.scrolled = true;
            }
        }

        let mut handled_shift_click = false;
        if let WindowEvent::MouseInput { state, button, .. } = event {
            if state.is_pressed() && *button == MouseButton::Left {
                let new_focus = self.find_topmost_selectable_at_pos_for_window(self.input_state.mouse.cursor_pos, window.id());
                if new_focus.is_some() {
                    self.shared.pending_event_response.consumed = true;
                }
                handled_shift_click = self.handle_left_click(new_focus);
                self.handle_click_counting();
            }
        }

        if let WindowEvent::MouseWheel { .. } = event {
            let hovered = self.find_topmost_selectable_at_pos_for_window(self.input_state.mouse.cursor_pos, window.id());
            if let Some(hovered_widget) = hovered {
                let consumed = self.handle_scroll_event(hovered_widget, event, window);
                self.shared.pending_event_response.consumed |= consumed;
            }
        }

        if !handled_shift_click {
            if let Some(focused) = self.shared.focused {
                // Only handle the event if the focused element belongs to this window
                let focused_belongs_to_window = match focused {
                    AnyBox::TextEdit(i) => {
                        if let Some(text_edit) = self.text_edits.get(i) {
                            text_edit.text_box.window_id.is_none() || text_edit.text_box.window_id == Some(window.id())
                        } else {
                            false
                        }
                    },
                    AnyBox::TextBox(i) => {
                        if let Some(text_box) = self.text_boxes.get(i) {
                            text_box.window_id.is_none() || text_box.window_id == Some(window.id())
                        } else {
                            false
                        }
                    },
                };

                if focused_belongs_to_window {
                    let consumed = self.handle_focused_event(focused, event, window);
                    self.shared.pending_event_response.consumed |= consumed;
                }
            }
        }

        return self.shared.pending_event_response;
    }

    fn find_topmost_selectable_at_pos_for_window(&self, cursor_pos: (f64, f64), window_id: WindowId) -> Option<AnyBox> {
        let mut topmost = None;
        let mut top_z = f32::MAX;

        for (_, entry) in self.shared.hit_tests.iter() {
            if !entry.selectable { continue; }
            if entry.hidden { continue; }
            if entry.window_id.is_some() && entry.window_id != Some(window_id) { continue; }
            if !hit_test_shape_check(&entry.shape, cursor_pos, &self.shared.render_data.group_transforms) { continue; }
            if entry.depth < top_z {
                top_z = entry.depth;
                topmost = Some(entry.any_box);
            }
        }

        topmost
    }

    /// Find the topmost text box that would receive mouse events, if it wasn't occluded by any non-text-box objects.
    /// 
    /// Returns the handle of the topmost text widget at the event position, or None if no widget is hit.
    /// Use this with [`Text::handle_event_with_topmost()`] for complex z-ordering scenarios.
    pub fn find_topmost_text_box(&mut self, event: &WindowEvent) -> Option<AnyBox> {
        // Only handle mouse events that have a position
        let cursor_pos = match event {
            WindowEvent::MouseInput { .. } => self.input_state.mouse.cursor_pos,
            WindowEvent::CursorMoved { position, .. } => (position.x, position.y),
            _ => return None,
        };

        self.find_topmost_at_pos(cursor_pos)
    }

    /// Get the depth of a text box by its handle.
    /// 
    /// Used for comparing depths when integrating with other objects that might occlude text boxs.
    pub fn get_text_box_depth(&self, text_box_id: &AnyBox) -> f32 {
        match text_box_id {
            AnyBox::TextEdit(i) => self.text_edits.get(*i).map(|te| te.text_box.depth).unwrap_or(f32::MAX),
            AnyBox::TextBox(i) => self.text_boxes.get(*i).map(|tb| tb.depth).unwrap_or(f32::MAX),
        }
    }

    /// Get the custom tag previously set on a box with [`TextBox::set_custom_tag`]
    /// or [`TextEdit::set_custom_tag`].
    pub fn get_custom_tag(&self, any_box: &AnyBox) -> Option<u64> {
        match any_box {
            AnyBox::TextEdit(i) => self.text_edits.get(*i).and_then(|te| te.text_box.custom_tag),
            AnyBox::TextBox(i) => self.text_boxes.get(*i).and_then(|tb| tb.custom_tag),
        }
    }

    /// Handle window events with a pre-determined topmost text box.
    /// 
    /// Use this in cases where text boxes might be occluded by other objects.
    /// Pass `Some(text_box_id)` if a text box should receive the event, or `None` if it's occluded.
    /// 
    /// If the text box is occluded, this function should still be called with `None`, so that other text boxes can defocus.
    pub fn handle_event_with_topmost(&mut self, event: &WindowEvent, window: &Window, topmost_text_box: Option<AnyBox>) -> EventResponse {
        self.shared.pending_event_response = EventResponse::default();

        self.input_state.handle_event(event);

        // update smooth scrolling animations
        if let WindowEvent::RedrawRequested = event {
            let animation_updated = self.update_smooth_scrolling();
            if animation_updated {
                window.request_redraw();
            }
        }

        let mut handled_shift_click = false;
        if let WindowEvent::MouseInput { state, button, .. } = event {
            if state.is_pressed() && *button == MouseButton::Left {
                if topmost_text_box.is_some() {
                    self.shared.pending_event_response.consumed = true;
                }
                handled_shift_click = self.handle_left_click(topmost_text_box);
                self.handle_click_counting();
            }
        }

        if let WindowEvent::MouseWheel { .. } = event {
            if let Some(hovered_widget) = topmost_text_box {
                let consumed = self.handle_scroll_event(hovered_widget, event, window);
                self.shared.pending_event_response.consumed |= consumed;
            }
        }

        if !handled_shift_click {
            if let Some(focused) = self.shared.focused {
                let consumed = self.handle_focused_event(focused, event, window);
                self.shared.pending_event_response.consumed |= consumed;
            }
        }

        return self.shared.pending_event_response;
    }

    fn find_topmost_at_pos(&self, cursor_pos: (f64, f64)) -> Option<AnyBox> {
        let mut topmost = None;
        let mut top_z = f32::MAX;

        for (_, entry) in self.shared.hit_tests.iter() {
            if entry.hidden { continue; }
            if !hit_test_shape_check(&entry.shape, cursor_pos, &self.shared.render_data.group_transforms) { continue; }
            if entry.depth < top_z {
                top_z = entry.depth;
                topmost = Some(entry.any_box);
            }
        }

        topmost
    }

    fn handle_click_counting(&mut self) {
        let now = Instant::now();
        let current_pos = self.input_state.mouse.cursor_pos;
        
        if let Some(last_info) = self.input_state.mouse.last_click_info.take() {
            if now.duration_since(last_info.time).as_secs_f64() < MULTICLICK_DELAY 
                && last_info.focused == self.shared.focused {
                let dx = current_pos.0 - last_info.pos.0;
                let dy = current_pos.1 - last_info.pos.1;
                let distance_squared = dx * dx + dy * dy;
                if distance_squared <= MULTICLICK_TOLERANCE_SQUARED {
                    self.input_state.mouse.click_count = (self.input_state.mouse.click_count + 1) % 4;
                } else {
                    self.input_state.mouse.click_count = 1;
                }
            } else {
                self.input_state.mouse.click_count = 1;
            }
        } else {
            self.input_state.mouse.click_count = 1;
        }
        
        self.input_state.mouse.last_click_info = Some(LastClickInfo {
            time: now,
            pos: current_pos,
            focused: self.shared.focused,
        });
    }
    
    fn handle_scroll_event(&mut self, hovered: AnyBox, event: &WindowEvent, window: &Window) -> bool {
        // scroll wheel event
        if let WindowEvent::MouseWheel { .. } = event {
            match hovered {
                AnyBox::TextEdit(i) => {
                    let handle = TextEditHandle { key: i };
                    let did_scroll = self.handle_text_edit_scroll_event(&handle, event, window);
                    if did_scroll {
                        self.shared.decorations_dirty = true;
                        self.shared.scrolled = true;
                    }
                    return did_scroll;
                },
                AnyBox::TextBox(_) => {}
            }
        }
        false
    }

    fn handle_focused_event(&mut self, focused: AnyBox, event: &WindowEvent, window: &Window) -> bool {
        if let WindowEvent::KeyboardInput { event, .. } = event {
            if event.state.is_pressed() {
                let mods_state = self.input_state.modifiers.state();
                let action_mod = if cfg!(target_os = "macos") {
                    mods_state.super_key()
                } else {
                    mods_state.control_key()
                };
                let shift = mods_state.shift_key();

                if shift {
                    match focused {
                        AnyBox::TextBox(i) => {
                            if self.handle_keyboard_selection(i, event, action_mod) {
                                self.shared.decorations_dirty = true;
                                return true;
                            }
                        }
                        AnyBox::TextEdit(i) => {
                            let can_handle = self.text_edits.get(i).map_or(false, |te| {
                                !te.text_box.hidden() && !te.disabled() && !te.showing_placeholder && !te.is_composing()
                            });
                            if can_handle {
                                let te = self.text_edits.get_mut(i).unwrap();
                                let tb = &mut te.text_box;
                                if apply_shift_nav_op(&mut tb.selection, &tb.layout, event, action_mod).is_some() {
                                    self.shared.reset_cursor_blink();
                                    return true;
                                }
                            }
                        }
                    }
                }

                if action_mod && !shift {
                    if let Key::Character(c) = event.key_without_modifiers() {
                        if c.as_str() == "c" {
                            match focused {
                                AnyBox::TextBox(_) => {
                                    if let Some(text) = self.selected_text() {
                                        with_clipboard(|cb| { cb.set_text(text).ok(); });
                                    }
                                }
                                AnyBox::TextEdit(i) => {
                                    if let Some(te) = self.text_edits.get(i) {
                                        if let Some(text) = te.text_box.selected_text() {
                                            with_clipboard(|cb| { cb.set_text(text).ok(); });
                                        }
                                    }
                                }
                            }
                            return true;
                        }
                    }
                }
            }
        }

        // todo: copying this for now, but maybe it can go into Shared
        let input_state = self.input_state.clone();

        match focused {
            AnyBox::TextEdit(i) => {
                let handle = TextEditHandle { key: i };
                let selection_before = self.text_edits.get(i).map(|te| te.text_box.selection());
                let consumed = self.get_text_edit_mut(&handle).handle_event_editable(event, window, &input_state);
                let selection_after = self.text_edits.get(i).map(|te| te.text_box.selection());

                if selection_before != selection_after {
                    self.shared.reset_cursor_blink();
                }
                consumed
            },
            AnyBox::TextBox(i) => {
                let handle = TextBoxHandle { key: i };
                let consumed = self.get_text_box_mut(&handle).handle_event(event, window, &input_state);

                // Handle cross-box selection extension for linked boxes
                if let WindowEvent::CursorMoved { .. } = event {
                    if input_state.mouse.pointer_down {
                        self.handle_cross_box_selection_extend(i);
                    }
                }

                consumed
            },
        }
    }

    /// Handle extending selection across linked text boxes when dragging.
    fn handle_cross_box_selection_extend(&mut self, focused_key: usize) {
        // Reset all extended selections first, they'll be recreated as needed
        for &key in &self.shared.multi_box_selection {
            if key != focused_key {
                self.text_boxes[key].selection = parley::Selection::default();
            }
        }
        self.shared.multi_box_selection.retain(|&key| key == focused_key);

        let did_extend_forward = self.extend_selection_in_direction(focused_key, SelectionDirection::Forward);
        let did_extend_backward = self.extend_selection_in_direction(focused_key, SelectionDirection::Backward);

        if did_extend_forward || did_extend_backward {
        }
    }

    /// Extend selection in a given direction (forward to next_box, backward to prev_box).
    /// Returns true if any extension happened.
    fn extend_selection_in_direction(&mut self, focused_key: usize, direction: SelectionDirection) -> bool {
        let cursor_pos = self.input_state.mouse.cursor_pos;

        let anchor_point;
        let extend_point;
        match direction {
            SelectionDirection::Forward => {
                anchor_point = (0.0, 0.0);
                extend_point = (f32::MAX, f32::MAX);
            },
            SelectionDirection::Backward => {
                anchor_point = (f32::MAX, f32::MAX);
                extend_point = (0.0, 0.0);
            },
        };

        // let focused_anchor_base = self.text_boxes[focused_key].selection.anchor_base();

        let mut current_key = focused_key;
        let mut did_extend = false;

        loop {
            // Check if cursor is past the boundary of current box
            let is_past_boundary = match direction {
                SelectionDirection::Forward => self.text_boxes[current_key].is_cursor_past_end(cursor_pos),
                SelectionDirection::Backward => self.text_boxes[current_key].is_cursor_before_start(cursor_pos),
            };

            if !is_past_boundary {
                break;
            }

            let linked_key = match direction {
                SelectionDirection::Forward => self.text_boxes[current_key].next_box,
                SelectionDirection::Backward => self.text_boxes[current_key].prev_box,
            };

            // Skip hidden linked boxes
            let linked_key = linked_key.filter(|&k| !self.text_boxes[k].hidden);

            // Extend current box's selection to the boundary
            {
                let current_box = &mut self.text_boxes[current_key];

                // For non-focused boxes (which were reset), set anchor at opposite boundary first
                if current_key != focused_key {
                    current_box.selection.move_to_point(&current_box.layout, anchor_point.0, anchor_point.1);
                }

                current_box.selection.extend_selection_to_point(&current_box.layout, extend_point.0, extend_point.1);
            }
            did_extend = true;

            let Some(linked_key) = linked_key else {
                break;
            };

            // Check if cursor actually hits the linked box
            let cursor_hits_linked = self.text_boxes[linked_key].hit_full_rect(cursor_pos);

            if cursor_hits_linked {
                // Add linked box to multi_box_selection and set partial selection
                if !self.shared.multi_box_selection.contains(&linked_key) {
                    self.shared.multi_box_selection.push(linked_key);
                }

                let linked_box = &mut self.text_boxes[linked_key];
                let local_pos = linked_box.cursor_to_local(cursor_pos);
                let local_cursor = (
                    local_pos.x + linked_box.scroll_offset.0,
                    local_pos.y + linked_box.scroll_offset.1,
                );

                // Parley doesn't let us see the selection granularity, so we can't do anything to preserve it in this case.
                // match focused_anchor_base {
                //     parley::AnchorBase::Word(_, _) => {
                //         linked_box.selection.select_word_at_point(&linked_box.layout, anchor_point.0, anchor_point.1);
                //     }
                //     parley::AnchorBase::Line(_, _) => {
                //         linked_box.selection.select_line_at_point(&linked_box.layout, anchor_point.0, anchor_point.1);
                //     }
                //     _ => {
                //         linked_box.selection.move_to_point(&linked_box.layout, anchor_point.0, anchor_point.1);
                //     }
                // }
                linked_box.selection.move_to_point(&linked_box.layout, anchor_point.0, anchor_point.1);
                linked_box.selection.extend_selection_to_point(&linked_box.layout, local_cursor.0, local_cursor.1);
                break;
            }

            // Cursor doesn't hit linked box, but we're past current box
            // Check if cursor is also past the linked box (it might be in a gap further along)
            let is_past_linked = match direction {
                SelectionDirection::Forward => self.text_boxes[linked_key].is_cursor_past_end(cursor_pos),
                SelectionDirection::Backward => self.text_boxes[linked_key].is_cursor_before_start(cursor_pos),
            };

            if is_past_linked {
                // Add linked box to multi_box_selection since we'll extend it in next iteration
                if !self.shared.multi_box_selection.contains(&linked_key) {
                    self.shared.multi_box_selection.push(linked_key);
                }
                current_key = linked_key;
            } else {
                // Cursor is in the gap but not past linked box
                // Don't extend further
                break;
            }
        }

        did_extend
    }

    fn handle_left_click(&mut self, new_focus: Option<AnyBox>) -> bool {
        let shift = self.input_state.modifiers.state().shift_key();
        let mut handled_shift_click = false;

        if shift {
            if let Some(AnyBox::TextBox(target_key)) = new_focus {
                handled_shift_click = self.handle_shift_click_selection(target_key);
            }
        }

        if ! handled_shift_click {
            // Clear visual selections on all multi-box boxes before refocusing
            if self.shared.multi_box_selection.len() > 1 {
                for &k in &self.shared.multi_box_selection {
                    self.text_boxes[k].selection = parley::Selection::default();
                }
            }
            self.shared.multi_box_selection.clear();

            self.shared.refocus(new_focus);

            // Set cross-box selection anchor for non-shift clicks on TextBox.
            if let Some(AnyBox::TextBox(key)) = new_focus {
                if self.shared.multi_box_selection.is_empty() {
                    self.shared.multi_box_selection.push(key);
                }

                self.shared.cross_box_selection_anchor = Some(key);
            } else {
                self.shared.cross_box_selection_anchor = None;
            }
            self.shared.cross_box_cursor_key = None;
        }

        handled_shift_click
    }

    /// Handle shift-click selection on TextBoxes.
    /// Uses the stored anchor to create selection spanning from anchor box to target box.
    fn handle_shift_click_selection(&mut self, target_key: usize) -> bool {
        let Some(anchor_key) = self.shared.cross_box_selection_anchor else {
            return false;
        };

        // Get click position in target's local coords
        let cursor = self.input_state.mouse.cursor_pos;
        let (click_x, click_y) = {
            let tb = &self.text_boxes[target_key];
            let p = tb.cursor_to_local(cursor);
            (p.x + tb.scroll_offset.0, p.y + tb.scroll_offset.1)
        };

        // Same box: use shift_click_extension which preserves word/line granularity
        if anchor_key == target_key {
            let tb = &mut self.text_boxes[target_key];
            let new_sel = tb.selection.shift_click_extension(&tb.layout, click_x, click_y);
            tb.selection = new_sel;
            self.shared.cross_box_cursor_key = None;
            return true;
        }

        // Determine direction by checking reachability without allocating
        let is_forward = {
            let mut cur = anchor_key;
            let mut found = false;
            while let Some(next) = self.text_boxes.get(cur).and_then(|b| b.next_box) {
                if next == target_key { found = true; break; }
                cur = next;
            }
            found
        };

        if !is_forward {
            // Verify target is reachable backward
            let mut cur = anchor_key;
            let mut found = false;
            while let Some(prev) = self.text_boxes.get(cur).and_then(|b| b.prev_box) {
                if prev == target_key { found = true; break; }
                cur = prev;
            }
            if !found {
                return false; // Not linked
            }
        }

        // Clear old selections (except anchor box which we'll extend)
        for &k in &self.shared.multi_box_selection {
            if k != anchor_key {
                self.text_boxes[k].selection = parley::Selection::default();
            }
        }
        self.shared.multi_box_selection.clear();

        // Boundary points
        let boundary_end = if is_forward { (f32::MAX, f32::MAX) } else { (0.0_f32, 0.0_f32) };
        let boundary_start = if is_forward { (0.0_f32, 0.0_f32) } else { (f32::MAX, f32::MAX) };

        // Walk the chain and apply selections directly
        let mut current = anchor_key;
        loop {
            let is_first = current == anchor_key;
            let is_last = current == target_key;

            {
                let tb = &mut self.text_boxes[current];
                if is_first {
                    // Anchor box: extend existing selection to boundary
                    tb.selection.extend_selection_to_point(&tb.layout, boundary_end.0, boundary_end.1);
                } else if is_last {
                    // Target box: place cursor at boundary edge, extend to exact click point (no snapping)
                    tb.selection.move_to_point(&tb.layout, boundary_start.0, boundary_start.1);
                    tb.selection.extend_selection_to_point(&tb.layout, click_x, click_y);
                } else {
                    // Middle box: select all
                    tb.selection.move_to_point(&tb.layout, boundary_start.0, boundary_start.1);
                    tb.selection.extend_selection_to_point(&tb.layout, boundary_end.0, boundary_end.1);
                }
            }
            self.shared.multi_box_selection.push(current);

            if is_last { break; }

            let next = if is_forward {
                self.text_boxes[current].next_box
            } else {
                self.text_boxes[current].prev_box
            };
            match next {
                Some(k) => current = k,
                None => break,
            }
        }

        self.shared.cross_box_cursor_key = Some(target_key);
        true
    }

    /// Returns true if the event was consumed.
    fn handle_keyboard_selection(
        &mut self,
        focused_key: usize,
        event: &KeyEvent,
        action_mod: bool,
    ) -> bool {
        let cursor_key = self.shared.cross_box_cursor_key.unwrap_or(focused_key);
        let sel_before = self.text_boxes[cursor_key].selection;

        let is_forward = {
            let tb = &mut self.text_boxes[cursor_key];
            match apply_shift_nav_op(&mut tb.selection, &tb.layout, event, action_mod) {
                Some(fwd) => fwd,
                None => return false,
            }
        };

        if self.text_boxes[cursor_key].selection != sel_before {
            return true;
        }

        if cursor_key == focused_key {
            // Try to extend into a linked box in the direction of the operation.
            let next_key = if is_forward {
                self.text_boxes[focused_key].next_box
            } else {
                self.text_boxes[focused_key].prev_box
            };
            let next_key = next_key.filter(|&k| !self.text_boxes[k].hidden);
            if let Some(next_key) = next_key {
                let entry = if is_forward { (0.0_f32, 0.0_f32) } else { (f32::MAX, f32::MAX) };
                {
                    let tb = &mut self.text_boxes[next_key];
                    tb.selection.move_to_point(&tb.layout, entry.0, entry.1);
                    apply_shift_nav_op(&mut tb.selection, &tb.layout, event, action_mod);
                }
                self.shared.cross_box_cursor_key = Some(next_key);
                if !self.shared.multi_box_selection.contains(&next_key) {
                    self.shared.multi_box_selection.push(next_key);
                }
            }
            return true;
        }

        // cursor_key != focused_key: in an extended box.
        // Determine extension direction and whether this op retracts or extends further.
        let is_forward_extension = self.is_cursor_forward_from_focused(focused_key, cursor_key);
        let is_retracting = is_forward_extension != is_forward;

        if is_retracting {
            // Find the box toward focused before modifying state.
            let prev_toward_focused = if is_forward_extension {
                self.text_boxes[cursor_key].prev_box
            } else {
                self.text_boxes[cursor_key].next_box
            };

            // Clear cursor box and remove from multi-box selection.
            self.text_boxes[cursor_key].selection = parley::Selection::default();
            self.shared.multi_box_selection.retain(|&k| k != cursor_key);

            let new_cursor = prev_toward_focused.unwrap_or(focused_key);
            self.shared.cross_box_cursor_key = if new_cursor == focused_key { None } else { Some(new_cursor) };

            {
                let tb = &mut self.text_boxes[new_cursor];
                apply_shift_nav_op(&mut tb.selection, &tb.layout, event, action_mod);
            }
        } else {
            // Extend further in the same direction.
            let next_key = if is_forward {
                self.text_boxes[cursor_key].next_box
            } else {
                self.text_boxes[cursor_key].prev_box
            };
            let next_key = next_key.filter(|&k| !self.text_boxes[k].hidden);
            if let Some(next_key) = next_key {
                let entry = if is_forward { (0.0_f32, 0.0_f32) } else { (f32::MAX, f32::MAX) };
                {
                    let tb = &mut self.text_boxes[next_key];
                    tb.selection.move_to_point(&tb.layout, entry.0, entry.1);
                    apply_shift_nav_op(&mut tb.selection, &tb.layout, event, action_mod);
                }
                self.shared.cross_box_cursor_key = Some(next_key);
                if !self.shared.multi_box_selection.contains(&next_key) {
                    self.shared.multi_box_selection.push(next_key);
                }
            }
        }

        true
    }

    /// Returns true if cursor_key is reachable by following next_box links from focused_key.
    fn is_cursor_forward_from_focused(&self, focused_key: usize, cursor_key: usize) -> bool {
        let mut cur = focused_key;
        while let Some(next) = self.text_boxes.get(cur).and_then(|b| b.next_box) {
            if next == cursor_key { return true; }
            cur = next;
        }
        false
    }

    /// Set the disabled state of a text edit box.
    /// 
    /// When disabled, the text edit will not respond to events and will be rendered with greyed out text.
    pub fn set_text_edit_disabled(&mut self, handle: &TextEditHandle, disabled: bool) {
        let text_edit = &mut self.text_edits[handle.key];
        text_edit.disabled = disabled;
        if disabled {
            if let Some(AnyBox::TextEdit(e)) = self.shared.focused {
                if e == handle.key {
                    self.get_text_edit_mut(&handle).text_box.reset_selection();
                    self.shared.focused = None;
                }
            }
        }

    }

    /// Returns `true` if scrolling occurred in the last frame.
    pub fn scrolled(&self) -> bool {
        self.shared.scrolled
    }


    /// Returns `true` if the text content needs to be redrawn.
    /// 
    /// This function is useful to decide whether to call `winit`'s `Window::request_redraw()` after processing a `winit` event.
    /// 
    /// Games and applications that rerender continuously can call `Window::request_redraw()` unconditionally after every `RedrawRequested` event, without checking this method.
    pub fn needs_rerender(&mut self) -> bool {
        return self.shared.decorations_dirty || self.shared.scrolled;
    }

    /// Get a mutable reference to a text box wrapped with its style.
    /// 
    /// `handle` is the handle that was returned when first creating the text box with [`Text::add_text_box()`].
    /// 
    /// This is a fast lookup operation that does not require any hashing.
    pub fn get_text_box_mut(&mut self, handle: &TextBoxHandle) -> &mut TextBox {
        return &mut self.text_boxes[handle.key];
    }

    /// Set a scale factor for all text.
    /// 
    /// This is an explict scale factor separate from the intrinsic scale factor for high-dpi displays, which is respected automatically. 
    pub fn set_ui_scale_factor(&mut self, ui_scale_factor: f64) {
        if self.shared.explicit_scale_factor == ui_scale_factor {
            return;
        }
        self.shared.explicit_scale_factor = ui_scale_factor;
        for (_, text_box) in self.text_boxes.iter_mut() {
            text_box.needs_reshape = true;
        }
        for (_, text_edit) in self.text_edits.iter_mut() {
            text_edit.text_box.needs_reshape = true;
        }
    }


    /// Get a mutable reference to a text box wrapped with its style.
    /// 
    /// `handle` is the handle that was returned when first creating the text box with [`Text::add_text_box()`].
    /// 
    /// This is a fast lookup operation that does not require any hashing.
    pub fn get_text_box(&self, handle: &TextBoxHandle) -> &TextBox {
        return &self.text_boxes[handle.key];
    }

    /// Returns a text box if it exists, or `None` if it has been removed.
    pub fn try_get_text_box(&self, handle: &ClonedTextBoxHandle) -> Option<&TextBox> {
        return self.text_boxes.get(handle.key);
    }

    /// Returns a mutable text box if it exists, or `None` if it has been removed.
    pub fn try_get_text_box_mut(&mut self, handle: &ClonedTextBoxHandle) -> Option<&mut TextBox> {
        return self.text_boxes.get_mut(handle.key);
    }

    /// Link two text boxes for cross-box selection.
    ///
    /// When selecting past the end of `first`, the selection will continue into `second`.
    /// When selecting before the start of `second`, the selection will continue into `first`.
    /// This only affects non-editable text boxes (TextBox, not TextEdit).
    pub fn link_text_boxes(&mut self, first: &TextBoxHandle, second: &TextBoxHandle) {
        self.text_boxes[first.key].next_box = Some(second.key);
        self.text_boxes[second.key].prev_box = Some(first.key);
    }

    /// Remove all cross-box selection links involving this text box.
    ///
    /// Clears `prev_box` and `next_box` on this box, and also clears the
    /// corresponding back-pointer on each former neighbor.
    pub fn unlink_text_box(&mut self, handle: &TextBoxHandle) {
        let (prev_key, next_key) = match self.text_boxes.get(handle.key) {
            Some(tb) => (tb.prev_box, tb.next_box),
            None => return,
        };
        if let Some(prev_key) = prev_key {
            if let Some(prev_tb) = self.text_boxes.get_mut(prev_key) {
                prev_tb.next_box = None;
            }
        }
        if let Some(next_key) = next_key {
            if let Some(next_tb) = self.text_boxes.get_mut(next_key) {
                next_tb.prev_box = None;
            }
        }
        let tb = &mut self.text_boxes[handle.key];
        tb.prev_box = None;
        tb.next_box = None;
    }

    /// Returns an iterator over selected text from all text boxes in the current multi-box selection.
    pub fn selected_text_iter(&self) -> impl Iterator<Item = &str> {
        self.shared.multi_box_selection.iter().filter_map(|&key| {
            self.text_boxes.get(key).and_then(|tb| tb.selected_text())
        })
    }

    /// Convenience function that returns the selected text from all text boxes in the current cross-box selection as a single contiguous string, inserting a space between each segment, or `None` if nothing is selected.
    ///
    /// If only one box is selected, a reference to the selected text is returned directly without any copying.
    /// Otherwise, the text is copied into an internal buffer.
    /// 
    /// Use [`Text::selected_text_iter()`] to get a zero-cost iterator over the different segments.
    pub fn selected_text(&mut self) -> Option<&str> {
        if self.shared.multi_box_selection.len() == 1 {
            let key = self.shared.multi_box_selection[0];
            return self.text_boxes.get(key).and_then(|tb| tb.selected_text());
        }

        self.selected_text_buffer.clear();
        for &key in &self.shared.multi_box_selection {
            if let Some(tb) = self.text_boxes.get(key) {
                if let Some(text) = tb.selected_text() {
                    if !self.selected_text_buffer.is_empty() && !self.selected_text_buffer.ends_with(' ') {
                        self.selected_text_buffer.push(' ');
                    }
                    self.selected_text_buffer.push_str(text);
                }
            }
        }

        if self.selected_text_buffer.is_empty() {
            None
        } else {
            Some(&self.selected_text_buffer)
        }
    }

    /// Add a scroll animation for a text edit
    pub(crate) fn add_scroll_animation(&mut self, handle: &TextEditHandle, start_offset: f32, target_offset: f32, duration: std::time::Duration, direction: ScrollDirection) {
        // Remove any existing animation for this handle and direction
        self.scroll_animations.retain(|anim| !(anim.handle.key == handle.key && anim.direction == direction));
        self.shared.scrolled = true;
        
        let animation = ScrollAnimation {
            start_offset,
            target_offset,
            start_time: std::time::Instant::now(),
            duration,
            direction,
            handle: handle.to_cloned(),
        };
        
        self.scroll_animations.push(animation);
    }

    /// Get the maximum remaining animation duration, if any animations are running.
    fn get_max_animation_duration(&self) -> Option<Duration> {
        let now = Instant::now();
        let mut max_remaining = Duration::ZERO;
        let mut has_animations = false;
        
        for animation in &self.scroll_animations {
            let elapsed = now.duration_since(animation.start_time);
            if elapsed < animation.duration {
                let remaining = animation.duration - elapsed;
                if remaining > max_remaining {
                    max_remaining = remaining;
                }
                has_animations = true;
            }
        }
        
        if has_animations {
            Some(max_remaining)
        } else {
            None
        }
    }

    /// Update smooth scrolling animations for all text edits automatically.
    /// Returns true if any text edit animations were updated and require redrawing.
    fn update_smooth_scrolling(&mut self) -> bool {
        let mut needs_redraw = false;
        
        // Update all active animations
        let mut i = 0;
        while i < self.scroll_animations.len() {
            let animation = &self.scroll_animations[i];
            if let Some(text_edit) = self.text_edits.get_mut(animation.handle.key) {
                let current_offset = animation.get_current_offset();
                
                match animation.direction {
                    ScrollDirection::Horizontal => {
                        text_edit.text_box.scroll_offset.0 = current_offset;
                    }
                    ScrollDirection::Vertical => {
                        text_edit.text_box.scroll_offset.1 = current_offset;
                    }
                }
                
                if animation.is_finished() {
                    self.scroll_animations.remove(i);
                    // Don't increment i since we removed an element
                } else {
                    i += 1;
                }
                
                needs_redraw = true;
            } else {
                // Text edit doesn't exist anymore, remove the animation
                self.scroll_animations.remove(i);
            }
        }
        
        needs_redraw
    }

    fn handle_text_edit_scroll_event(&mut self, handle: &TextEditHandle, event: &WindowEvent, _window: &Window) -> bool {
        let mut did_scroll = false;

        if let WindowEvent::MouseWheel { delta, .. } = event {
            let shift_held = self.input_state.modifiers.state().shift_key();
            
            if let Some(te) = self.text_edits.get_mut(handle.key) {
                if te.text_box.single_line {
                    // Single-line horizontal scrolling
                    let scroll_amount = match delta {
                        winit::event::MouseScrollDelta::LineDelta(x, y) => {
                            if shift_held {
                                y * 120.0
                            } else {
                                x * 120.0
                            }
                        },
                        winit::event::MouseScrollDelta::PixelDelta(pos) => {
                            if shift_held {
                                pos.y as f32 
                            } else {
                                pos.x as f32
                            }
                        },
                    };
                    
                    if scroll_amount != 0.0 {
                        let current_scroll = te.text_box.scroll_offset.0;
                        let target_scroll = current_scroll - scroll_amount;
                        
                        let total_text_width = te.text_box.layout.full_width();
                        let text_width = te.text_box.width;
                        let max_scroll = (total_text_width - text_width).max(0.0).round() + crate::text_edit::CURSOR_WIDTH;
                        let clamped_target = target_scroll.clamp(0.0, max_scroll).round();
                        
                        if (clamped_target - current_scroll).abs() > 0.1 {
                            if should_use_animation(delta, shift_held) {
                                let animation_duration = std::time::Duration::from_millis(200);
                                self.add_scroll_animation(handle, current_scroll, clamped_target, animation_duration, ScrollDirection::Horizontal);
                            } else {
                                te.text_box.scroll_offset.0 = clamped_target;
                            }
                            did_scroll = true;
                        }
                    }
                } else {
                    // Multi-line vertical scrolling
                    let scroll_amount = match delta {
                        winit::event::MouseScrollDelta::LineDelta(_x, y) => y * 120.0,
                        winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                    };
                    
                    if scroll_amount != 0.0 {
                        let current_scroll = te.text_box.scroll_offset.1;
                        let target_scroll = current_scroll - scroll_amount;
                        
                        let total_text_height = te.text_box.layout.height();
                        let text_height = te.text_box.height;
                        let max_scroll = (total_text_height - text_height).max(0.0).round();
                        let clamped_target = target_scroll.clamp(0.0, max_scroll).round();
                        
                        if (clamped_target - current_scroll).abs() > 0.1 {
                            if should_use_animation(delta, true) {
                                let animation_duration = std::time::Duration::from_millis(200);
                                self.add_scroll_animation(handle, current_scroll, clamped_target, animation_duration, ScrollDirection::Vertical);
                            } else {
                                te.text_box.scroll_offset.1 = clamped_target;
                            }
                            did_scroll = true;
                        }
                    }
                }
            }
        }

        did_scroll
    }

    /// Returns the duration until the next cursor blink state change.
    ///
    /// Returns `None` if cursor blinking should not be blinking.
    pub fn time_until_next_cursor_blink(&self) -> Option<Duration> {
        if let Some(start_time) = self.shared.cursor_blink_start {
            let elapsed = Instant::now().duration_since(start_time);
            let blink_period = Duration::from_millis(CURSOR_BLINK_TIME_MILLIS);
            let elapsed_in_current_cycle = elapsed.as_millis() % blink_period.as_millis();
            let time_until_next_blink = blink_period.as_millis() - elapsed_in_current_cycle;
            Some(Duration::from_millis(time_until_next_blink as u64))
        } else {
            None
        }
    }

    /// Clear focus from any focused text box or text edit.
    pub fn clear_focus(&mut self) {
        self.shared.refocus(None);
    }

    /// Returns the currently focused text widget, if any.
    pub fn focus(&self) -> Option<AnyBox> {
        self.shared.focused
    }

    /// Returns a mutable reference to the FontContext.
    pub fn font_context(&mut self) -> &mut FontContext {
        &mut self.shared.font_cx
    }

    /// Returns a mutable reference to the LayoutContext.
    pub fn layout_context(&mut self) -> &mut LayoutContext<ColorBrush> {
        &mut self.shared.layout_cx
    }

    /// Helper method to load a font from font data and return the family name which can be used in to refer to it in a text style.
    /// 
    /// Returns `None` if the font data is invalid or contains no fonts.
    /// 
    /// For more advanced use cases, use [`Text::font_context()`] to get a mutable reference to the parley `FontContext`. The `collection` field of the `FontContext` is an instance of a `fontique` `Collection`, which offers lower level control.
    /// 
    /// # Example
    /// ```ignored
    /// # use keru_text::*;
    /// # use parley::FontFamily;
    /// # let text = Text::new();
    /// let family_name = text.load_font(include_bytes!("../MyFont.ttf"))
    ///     .expect("Failed to load font");
    /// let style = text.add_style(TextStyle {
    ///     font_stack: FontStack::Single(FontFamily::Named(family_name.into())),
    ///     ..Default::default()
    /// }, None);
    /// 
    /// # let text_box: TextBoxHandle = unimplemented!();
    /// text.get_text_box_mut(&text_box).set_style(&style);
    /// ```
    pub fn load_font(&mut self, font_data: &[u8]) -> Option<String> {
        let families = self.shared.font_cx.collection.register_fonts(font_data.to_vec().into(), None);
        let family_id = families.first()?.0;
        let family = self.shared.font_cx.collection.family(family_id)?;
        Some(family.name().to_string())
    }

}

// todo: get this from system settings.
const CURSOR_BLINK_TIME_MILLIS: u64 = 500;

#[derive(Debug)]
enum WakerCommand {
    Start,
    Stop,
    Exit,
}

pub(crate) struct CursorBlinkWaker {
    command_sender: mpsc::Sender<WakerCommand>,
}

impl Drop for CursorBlinkWaker {
    fn drop(&mut self) {
        // Signal the thread to exit
        let _ = self.command_sender.send(WakerCommand::Exit);
    }
}

impl CursorBlinkWaker {
    fn new(window: Weak<Window>) -> Self {
        let (command_sender, command_receiver) = mpsc::channel();
        
        thread::spawn(move || {
            let mut is_running = false;
            
            loop {
                if is_running {
                    // While running, wait for either a command or timeout
                    match command_receiver.recv_timeout(Duration::from_millis(CURSOR_BLINK_TIME_MILLIS)) {
                        Ok(WakerCommand::Start) => {}
                        Ok(WakerCommand::Stop) => is_running = false,
                        Ok(WakerCommand::Exit) => return,
                        Err(mpsc::RecvTimeoutError::Timeout) => {
                            // Timeout occurred, request redraw directly
                            if let Some(window) = window.upgrade() {
                                window.request_redraw();
                            } else {
                                // Window has been dropped, exit thread
                                return;
                            }
                        }
                        Err(mpsc::RecvTimeoutError::Disconnected) => return,
                    }
                } else {
                    // While stopped, wait indefinitely for a command
                    match command_receiver.recv() {
                        Ok(WakerCommand::Start) => is_running = true,
                        Ok(WakerCommand::Stop) => {}
                        Ok(WakerCommand::Exit) => return,
                        Err(_) => return,
                    }
                }
            }
        });
        
        Self {
            command_sender,
        }
    }
        
    fn start(&self) {
        let _ = self.command_sender.send(WakerCommand::Start);
    }
    
    fn stop(&self) {
        let _ = self.command_sender.send(WakerCommand::Stop);
    }
}

/// The result of handling a `winit::WindowEvent` with [`Text::handle_event()`].
#[derive(Clone, Copy, Default)]
pub struct EventResponse {
    /// Whether the event was consumed by a text box (and should not be handled further by the caller).
    pub consumed: bool,
    /// The box that gained focus as a result of this event, if any.
    pub focus_gained: Option<AnyBox>,
    /// The box that lost focus as a result of this event, if any.
    pub focus_lost: Option<AnyBox>,
}
