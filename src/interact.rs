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

    // todo: think if it's really worth it to do this on every mouse movement.
    // maybe add a global setting to do it just once per frame
    pub(crate) fn resolve_hover(&mut self) {
        // if something draggable is being dragged, stay awake at every mouse movement, regardless of what is hit
        // todo: this probably has false positives
        if self.sys.mouse_input.dragged_at(None, None).is_some() {
            self.set_new_ui_input();
        }

        // real hover
        let hovered_node_id = self.scan_mouse_hits(false);
        self.sys.mouse_input.update_current_tag(hovered_node_id);

        if let Some(hovered_id) = hovered_node_id {
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
                let inspect_hovered_node_id = self.scan_mouse_hits(true);
                if let Some(hovered_id) = inspect_hovered_node_id {
                    if let Some(id) = self.sys.inspect_hovered {
                        if id != hovered_id {
                            // newly entered
                            let (_, hovered_node_i) = self.nodes.get_mut_by_id(&hovered_id).unwrap();
                            if self.inspect_mode() {
                                log::info!("Inspect mode: hovering {}", self.node_debug_name_fmt_scratch(hovered_node_i))
                            }
                        }
                    }
                }
                self.sys.inspect_hovered = inspect_hovered_node_id;
            }
        }
    }

    pub(crate) fn start_hovering(&mut self, hovered_id: Id) {
        self.sys.hovered.push(hovered_id);
        
        let (hovered_node, hovered_node_i) = self.nodes.get_mut_by_id(&hovered_id).unwrap();

        if hovered_node.params.interact.click_animation {
            hovered_node.hovered = true;
            hovered_node.hover_timestamp = ui_time_f32();
            
            self.sys.changes.cosmetic_rect_updates.push(hovered_node_i);
            self.sys.anim_render_timer.push_new(Duration::from_secs_f32(ANIMATION_RERENDER_TIME));
        }

    }

    pub(crate) fn end_all_hovering(&mut self) {
        let mut animation = false;

        for hovered_id in &self.sys.hovered {
            if let Some((hovered_node, hovered_node_i)) = self.nodes.get_mut_by_id(hovered_id) {
                if hovered_node.last_frame_touched == self.sys.current_frame {
                
                    if hovered_node.params.interact.click_animation {
                        hovered_node.hovered = false;
                        hovered_node.hover_timestamp = ui_time_f32();
                        self.sys.changes.cosmetic_rect_updates.push(hovered_node_i);

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

    pub(crate) fn resolve_click_release(&mut self, _button: MouseButton) {
        // todo: there's something wrong here, releasing a click doesn't wake up the event loop somehow (it stays dark)
        self.set_new_ui_input();
    }

    // returns if the ui consumed the mouse press, or if it should be passed down. 
    pub(crate) fn resolve_click_press(&mut self, button: MouseButton, _event: &WindowEvent, _window: &Window) -> bool {
        // todo wtf? don't do this unconditionally, we have senses now
        self.set_new_ui_input();

        // defocus, so that we defocus when clicking anywhere outside.
        // if we're clicking something we'll re-focus below.
        self.sys.focused = None;

        // if nothing is hit, we're done.
        let Some(clicked_id) = self.sys.mouse_input.current_tag() else {
            return false;
        };
        
        // hardcoded stuff with animations, focusing nodes, spawning cursors, etc
        if button == MouseButton::Left {
            // the default animation and the "focused" flag are hardcoded to work on left click only, I guess.
            let t = T0.elapsed();

            // todo: yuck
            let (clicked_node, clicked_node_i) = self.nodes.get_mut_by_id(&clicked_id).unwrap();

            if clicked_node.params.interact.click_animation {

                clicked_node.last_click = t.as_secs_f32();

                self.sys.changes.cosmetic_rect_updates.push(clicked_node_i);
                
                self.sys.anim_render_timer.push_new(Duration::from_secs_f32(ANIMATION_RERENDER_TIME));
            }
          
            if let Some(text_i) = &clicked_node.text_i {
                // todo: isn't this all obsolete now?
                match text_i {
                    TextI::TextEdit(_) => {
                        self.sys.focused = Some(clicked_id);
                    }
                    TextI::TextBox(_) => {}
                }

                // todo: not always...
                self.push_text_change(clicked_node_i);

            }
        }
   
        let consumed = self.sys.mouse_input.current_tag().is_some();
        return consumed;
    }

    pub(crate) fn scan_mouse_hits(&mut self, _see_invisible_rects: bool) -> Option<Id> {
        self.sys.mouse_hit_stack.clear();

        for clk_i in 0..self.sys.click_rects.len() {
            let clk_rect = self.sys.click_rects[clk_i];
            
            // in release mode, if a node is not absorbs_mouse_events it won't have a click_rect in the first place
            #[cfg(debug_assertions)] {
                if ! _see_invisible_rects && ! self.nodes[clk_rect.i].params.interact.absorbs_mouse_events {
                    continue;
                }
            }
            
            if self.hit_click_rect(&clk_rect) {
                let node_i = clk_rect.i;
                let id = self.nodes[node_i].id;
                let z = self.nodes[node_i].z;
                self.sys.mouse_hit_stack.push((id, z));
            }
        }


        // only the one with the top (aka lowest) z is actually clicked.
        // in practice, nobody ever sets the Z. it depends on the order.
        let mut topmost_hit = None;

        let mut top_z = f32::MAX;
        for (id, z) in self.sys.mouse_hit_stack.iter().rev() {

            if *z < top_z {
                top_z = *z;
                topmost_hit = Some(*id);
            }
        }

        return topmost_hit;
    }

    pub(crate) fn scan_scroll_areas_mouse_hits(&mut self) -> Option<Id> {
        self.sys.mouse_hit_stack.clear();

        for clk_i in 0..self.sys.scroll_rects.len() {
            let clk_rect = self.sys.scroll_rects[clk_i];
            if self.hit_click_rect(&clk_rect) {
                let node_i = clk_rect.i;
                let id = self.nodes[node_i].id;
                let z = self.nodes[node_i].z;
                self.sys.mouse_hit_stack.push((id, z));
            }
        }
        
        // only the one with the top (aka lowest) z is actually clicked.
        // in practice, nobody ever sets the Z. it depends on the order.
        let mut topmost_hit = None;

        let mut top_z = f32::MAX;
        for (id, z) in self.sys.mouse_hit_stack.iter().rev() {

            if *z < top_z {
                top_z = *z;
                topmost_hit = Some(*id);
            }
        }

        return topmost_hit;
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
            self.recursive_place_children(i, true);
            
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
        const CLICK    = 1 << 0;
        const DRAG     = 1 << 1;
        const HOVER = 1 << 2;
        const HOLD  = 1 << 4;
        // todo: HoverEnter could be useful
        
        const CLICK_AND_HOVER = Self::CLICK.bits() | Self::HOVER.bits();
        const NONE = 0;
    }
}

impl Ui {
    pub(crate) fn click_rect(&mut self, i: NodeI) -> ClickRect {
        return ClickRect {
            rect: self.nodes[i].animated_rect,
            i,
        }
    }
}