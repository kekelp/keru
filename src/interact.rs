use std::time::{Duration, Instant};

use winit::{dpi::PhysicalPosition, event::{KeyEvent, MouseButton, MouseScrollDelta, WindowEvent}, keyboard::{Key, NamedKey}, window::Window};

use crate::*;
use crate::Axis::{X, Y};

pub(crate) const ANIMATION_RERENDER_TIME: f32 = 0.5;

/// A struct describing a click event on a GUI node.
#[derive(Clone, Copy, Debug)]
pub struct Click {
    /// Absolute screen position in pixels
    pub absolute_position: glam::DVec2,
    /// Position relative to the node (0.0 to 1.0 in each dimension)
    pub relative_position: glam::DVec2,
    /// Timestamp of the click
    pub timestamp: Instant,
}

/// A struct describing a drag event on a GUI node.
#[derive(Clone, Copy, Debug)]
pub struct Drag {
    /// Absolute screen position in pixels
    pub absolute_position: glam::DVec2,
    /// Position relative to the node (0.0 to 1.0 in each dimension)
    pub relative_position: glam::DVec2,
    /// Absolute delta movement in pixels
    pub absolute_delta: glam::DVec2,
    /// Delta movement relative to the node's dimensions (as a fraction)
    pub relative_delta: glam::DVec2,
    /// Time when the drag event started
    pub pressed_timestamp: Instant,
}

/// A struct describing a scroll event on a GUI node.
#[derive(Clone, Copy, Debug)]
pub struct ScrollEvent {
    /// Absolute screen position in pixels where the scroll occurred
    pub absolute_position: glam::DVec2,
    /// Position relative to the node (0.0 to 1.0 in each dimension)
    pub relative_position: glam::DVec2,
    /// Scroll delta (positive Y is scroll up, negative Y is scroll down)
    pub delta: glam::DVec2,
    /// Timestamp of the scroll event
    pub timestamp: Instant,
}

impl Ui {

    pub(crate) fn resolve_hover(&mut self) {
        let hovered_node_ids = self.scan_mouse_hits(false);
        self.sys.mouse_input.update_current_tag(hovered_node_ids.clone());

        // Get the topmost hovered element (first in the list) for hover animations
        if let Some(&hovered_id) = hovered_node_ids.first() {
            if self.sys.hovered.contains(&hovered_id) {
                let hovered_i = self.nodes.node_hashmap.get(&hovered_id).unwrap().slab_i;
                if self.nodes[hovered_i].params.interact.senses.contains(Sense::HOVER) {
                    self.set_new_ui_input();
                }

            } else {
                // newly entered
                let (_, hovered_node_i) = self.nodes.get_mut_by_id(&hovered_id).unwrap();

                self.end_all_hovering();
                self.start_hovering(hovered_id);

                if self.nodes[hovered_node_i].params.interact.senses.contains(Sense::HOVER) {
                    self.set_new_ui_input();
                }
                if self.nodes[hovered_node_i].params.interact.click_animation {
                    self.sys.anim_render_timer.push_new(Duration::from_secs_f32(ANIMATION_RERENDER_TIME));
                }

            }

        } else {
            self.end_all_hovering();
        }

        if self.sys.mouse_input.dragged(Some(MouseButton::Left), None) != (0.0, 0.0) {
            self.set_new_ui_input();
        }

        // scroll area hover
        let hovered_scroll_area_id = self.scan_scroll_areas_mouse_hits();

        // since this doesn't cause any rerenders or gpu updates directly, I think we can do it in the dumb way for now
        if let Some(hovered_scroll_area_id) = hovered_scroll_area_id {
            self.sys.hovered_scroll_area = Some(hovered_scroll_area_id);
        } else {
            self.sys.hovered_scroll_area = None;
        }

        // in debug mode, do a separate scan that sees invisible rects as well
        #[cfg(debug_assertions)] {
            if self.inspect_mode() {
                let inspect_hovered_node_ids = self.scan_mouse_hits(true);
                if let Some(&hovered_id) = inspect_hovered_node_ids.first() {
                    if let Some(&old_id) = self.sys.inspect_hovered.first() {
                        if old_id != hovered_id {
                            // newly entered
                            let (_, hovered_node_i) = self.nodes.get_mut_by_id(&hovered_id).unwrap();
                            if self.inspect_mode() {
                                log::info!("Inspect mode: hovering {}", self.node_debug_name_fmt_scratch(hovered_node_i))
                            }
                        }
                    }
                }
                self.sys.inspect_hovered = inspect_hovered_node_ids;
            }
        }

    }

    pub(crate) fn start_hovering(&mut self, hovered_id: Id) {
        self.sys.hovered.push(hovered_id);
        
        let (hovered_node, _hovered_node_i) = self.nodes.get_mut_by_id(&hovered_id).unwrap();

        if hovered_node.params.interact.click_animation {
            hovered_node.hovered = true;
            hovered_node.hover_timestamp = ui_time_f32();
            
            self.sys.changes.rebuild_render_data = true;
            self.sys.anim_render_timer.push_new(Duration::from_secs_f32(ANIMATION_RERENDER_TIME));
        }

    }

    pub(crate) fn end_all_hovering(&mut self) {
        let mut animation = false;

        for hovered_id in &self.sys.hovered {
            if let Some((hovered_node, _hovered_node_i)) = self.nodes.get_mut_by_id(hovered_id) {
                if hovered_node.last_frame_touched == self.sys.current_frame {
                
                    if hovered_node.params.interact.click_animation {
                        hovered_node.hovered = false;
                        hovered_node.hover_timestamp = ui_time_f32();
                        self.sys.changes.rebuild_render_data = true;

                        animation = true;
                    }
                }
            }
        }

        if animation {
            self.sys.anim_render_timer.push_new(Duration::from_secs_f32(ANIMATION_RERENDER_TIME));
        }

        self.sys.hovered.clear();
    }

    pub(crate) fn begin_frame_resolve_inputs(&mut self) {
        self.sys.mouse_input.begin_new_frame();
        self.sys.key_input.begin_new_frame();
        
        // Double buffer: move this frame's changes to last frame, clear this frame
        self.sys.text_edit_changed_last_frame = self.sys.text_edit_changed_this_frame;
        self.sys.text_edit_changed_this_frame = None;
    }

    pub(crate) fn resolve_click_release(&mut self, _button: MouseButton,  clicked_i: NodeI) {
        if self.nodes[clicked_i].params.interact.senses.contains(Sense::CLICK_RELEASE) {
            self.set_new_ui_input();
        }
    }

    // returns if the ui consumed the mouse press, or if it should be passed down.
    pub(crate) fn resolve_click_press(&mut self, button: MouseButton, _event: &WindowEvent, _window: &Window, clicked_i: NodeI) -> bool {
        // defocus, so that we defocus when clicking anywhere outside.
        // if we're clicking something we'll re-focus below.
        self.sys.focused = None;

        let clicked_id = self.nodes[clicked_i].id;

        let sense_click = self.nodes[clicked_i].params.interact.senses.contains(Sense::CLICK);
        if sense_click {
            self.set_new_ui_input();
        }

        // hardcoded stuff with animations, focusing nodes, spawning cursors, etc
        if button == MouseButton::Left {
            // the default animation and the "focused" flag are hardcoded to work on left click only, I guess.
            let t = T0.elapsed();

            if self.nodes[clicked_i].params.interact.click_animation {
                self.nodes[clicked_i].last_click = t.as_secs_f32();
                self.sys.changes.rebuild_render_data = true;
                self.sys.anim_render_timer.push_new(Duration::from_secs_f32(ANIMATION_RERENDER_TIME));
            }

            if let Some(text_i) = &self.nodes[clicked_i].text_i {
                // todo: isn't this all obsolete now?
                match text_i {
                    TextI::TextEdit(_) => {
                        self.sys.focused = Some(clicked_id);
                    }
                    TextI::TextBox(_) => {}
                }

                // todo: not always...
                // self.push_text_change(clicked_i);
            }
        }

        let consumed = self.nodes[clicked_i].params.interact.absorbs_mouse_events;
        return consumed;
    }

    // _see_invisible_rects needs the _ to avoid the warning in non-debug mode
    pub(crate) fn scan_mouse_hits(&mut self, _see_invisible_rects: bool) -> smallvec::SmallVec<[Id; 6]> {
        let mut result = smallvec::SmallVec::new();
        for clk_i in (0..self.sys.click_rects.len()).rev() {
            let clk_rect = self.sys.click_rects[clk_i];

            // In inspect mode, we see all rects. In normal mode, we only process rects that are interactive
            #[cfg(debug_assertions)] {
                if ! _see_invisible_rects {
                    let has_interaction = self.nodes[clk_rect.i].params.interact.absorbs_mouse_events
                        || self.nodes[clk_rect.i].params.interact.senses != Sense::NONE;
                    if !has_interaction {
                        continue;
                    }
                }
            }

            if self.hit_click_rect(&clk_rect) {
                let node_id = self.nodes[clk_rect.i].id;
                let absorbs = self.nodes[clk_rect.i].params.interact.absorbs_mouse_events;

                result.push(node_id);

                if absorbs {
                    break;
                }
            }
        }

        return result;
    }


    pub(crate) fn scan_scroll_areas_mouse_hits(&mut self) -> Option<Id> {
        for clk_i in (0..self.sys.scroll_rects.len()).rev() {
            let clk_rect = self.sys.scroll_rects[clk_i];

            if self.hit_click_rect(&clk_rect) {
                return Some(self.nodes[clk_rect.i].id);
            }
        }

        return None;
    }

    pub(crate) fn handle_keyboard_event(&mut self, event: &KeyEvent) -> bool {
                

        if let Key::Named(named_key) = &event.logical_key {
            if named_key == &NamedKey::F1 {
                if event.state.is_pressed() && self.sys.debug_key_pressed == false {
                    #[cfg(debug_assertions)]
                    {
                        self.set_inspect_mode(!self.inspect_mode());
                        self.set_new_ui_input();
                    }
                }

                self.sys.debug_key_pressed = event.state.is_pressed();
            }
        }

        return false;
    }

    pub(crate) fn handle_scroll_event(&mut self, delta: &MouseScrollDelta) {
        let Some(hovered_scroll_area_id) = self.sys.hovered_scroll_area else {
            return;
        };

        let Some(i) = self.nodes.node_hashmap.get(&hovered_scroll_area_id) else {
            return;
        };
        let i = i.slab_i;

        let (x, y) = match delta {
            MouseScrollDelta::LineDelta(x, y) => (x * 0.1, y * 0.1),
            MouseScrollDelta::PixelDelta(PhysicalPosition {x, y}) => (*x as f32, *y as f32),
        };
        let delta = Xy::new(x, y);

        for axis in [X, Y] {            
            if self.nodes[i].params.layout.scrollable[axis] {
                self.update_container_scroll(i, delta[axis], axis);
            };
        }
        
        if self.nodes[i].params.is_scrollable() {

            // todo: add quicker functions that just move the rectangles. for text, this requires big changes in textslabs and will probably become impossible if we change renderer
            self.recursive_place_children(i);
            
            self.sys.changes.text_changed = true;
            // self.sys.text.prepare_all(&mut self.sys.text_renderer);

            self.resolve_hover();


            self.sys.changes.need_gpu_rect_update = true;
            self.sys.changes.need_rerender = true;
        }
    }
}


#[derive(Debug, Copy, Clone)]
pub(crate) struct ClickRect {
    pub rect: XyRect,
    pub i: NodeI,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct Sense: u8 {
        const CLICK = 1 << 0;
        const DRAG  = 1 << 1;
        const HOVER = 1 << 2;
        const HOLD  = 1 << 4;
        const CLICK_RELEASE = 1 << 5;
        // todo: HoverEnter could be useful
        
        const NONE = 0;
    }
}

impl Ui {
    pub(crate) fn click_rect(&mut self, i: NodeI) -> ClickRect {
        let real_rect = self.nodes[i].real_rect;
        let transform = self.nodes[i].accumulated_transform;
        let size = self.sys.unifs.size;

        // Apply transform: scale in normalized space, translate in pixel space converted to normalized
        let tx_norm = transform.m31 / size[X];
        let ty_norm = transform.m32 / size[Y];

        let transformed_rect = XyRect::new(
            [real_rect[X][0] * transform.m11 + tx_norm, real_rect[X][1] * transform.m11 + tx_norm],
            [real_rect[Y][0] * transform.m22 + ty_norm, real_rect[Y][1] * transform.m22 + ty_norm],
        );

        // Clip the transformed rect to the node's clip_rect
        let clip_rect = self.nodes[i].clip_rect;
        let clipped_rect = XyRect::new(
            intersect(transformed_rect[X], clip_rect[X]),
            intersect(transformed_rect[Y], clip_rect[Y]),
        );

        return ClickRect {
            rect: clipped_rect,
            i,
        }
    }
}