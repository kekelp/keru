use std::time::Duration;

use winit::{dpi::PhysicalPosition, event::{KeyEvent, MouseButton, MouseScrollDelta}, keyboard::{Key, NamedKey}};
use winit_mouse_events::MouseInput;

use crate::*;
use crate::Axis::{X, Y};

pub(crate) const ANIMATION_RERENDER_TIME: f32 = 0.5;

impl Ui {
    pub fn mouse_input(&self) -> &MouseInput<Id> {
        return &self.sys.mouse_input;
    }

    pub fn is_clicked(&self, node_key: NodeKey) -> bool {
        return self.sys.mouse_input.clicked(Some(MouseButton::Left), Some(node_key.id_with_subtree()));
    }

    pub fn is_held(&self, node_key: NodeKey) -> Option<(Duration, glam::DVec2)> {
        return self.sys.mouse_input.held(Some(MouseButton::Left), Some(node_key.id_with_subtree()));
    }

    pub fn is_dragged(&self, node_key: NodeKey) -> (f64, f64) {
        return self.sys.mouse_input.dragged(Some(MouseButton::Left), Some(node_key.id_with_subtree()));
    }

    /// Returns `true` if a node is currently hovered by the cursor.
    pub fn is_hovered(&self, node_key: NodeKey) -> bool {
        return self.sys.hovered.last() == Some(&node_key.id_with_subtree());
    }

    // todo: think if it's really worth it to do this on every mouse movement.
    // maybe add a global setting to do it just once per frame
    pub(crate) fn resolve_hover(&mut self) {
        // real hover
        let hovered_node_id = self.scan_mouse_hits();
        self.sys.mouse_input.update_current_tag(hovered_node_id);

        if let Some(hovered_id) = hovered_node_id {
            if self.sys.hovered.contains(&hovered_id) {
                // nothing changed, do nothing
            } else {
                // newly entered
                self.end_all_hovering();
                self.start_hovering(hovered_id);
                self.sys.new_ui_input = true;
            }
            
        } else {
            self.end_all_hovering();
        }

        if self.sys.is_anything_dragged {
            self.sys.new_ui_input = true;
        }

        // scroll area hover
        let hovered_scroll_area_id = self.scan_scroll_areas_mouse_hits();

        // since this doesn't cause any rerenders or gpu updates directly, I think we can do it in the dumb way for now
        if let Some(hovered_scroll_area_id) = hovered_scroll_area_id {
            self.sys.hovered_scroll_area = Some(hovered_scroll_area_id);
        } else {
            self.sys.hovered_scroll_area = None;
        }
    }

    pub(crate) fn start_hovering(&mut self, hovered_id: Id) {
        self.sys.hovered.push(hovered_id);
        
        // todo: yuck
        let hovered_node_i = self.nodes.node_hashmap.get(&hovered_id).unwrap().slab_i;
        let hovered_node = &mut self.nodes.nodes[hovered_node_i];

        if hovered_node.params.interact.click_animation {
            hovered_node.hovered = true;
            hovered_node.hover_timestamp = ui_time_f32();
            
            self.sys.changes.cosmetic_rect_updates.push(hovered_node_i);
            self.sys.anim_render_timer.push_new(Duration::from_secs_f32(ANIMATION_RERENDER_TIME));
        }

    }

    pub(crate) fn end_all_hovering(&mut self) {
        if ! self.sys.hovered.is_empty() {
            self.sys.anim_render_timer.push_new(Duration::from_secs_f32(ANIMATION_RERENDER_TIME));
        }

        for hovered_id in &self.sys.hovered {
            let hovered_nodemap_entry = self.nodes.node_hashmap.get(&hovered_id);
            
            if let Some(entry) = hovered_nodemap_entry {
                // check that the node is currently part of the tree...
                // this is a bit scary, and it will need to change with `assume_unchanged` and friends
                if entry.last_frame_touched == self.sys.current_frame {

                    let hovered_node_i = entry.slab_i;
                    let hovered_node = &mut self.nodes.nodes[hovered_node_i];
                    
                    if hovered_node.params.interact.click_animation {
                        hovered_node.hovered = false;
                        hovered_node.hover_timestamp = ui_time_f32();
                        self.sys.changes.cosmetic_rect_updates.push(hovered_node_i);
                    }

                    self.sys.new_ui_input = true;
                }
            }
        }
        self.sys.hovered.clear();
    }

    pub(crate) fn begin_frame_resolve_inputs(&mut self) {
        self.sys.mouse_input.begin_new_frame();

        if self.sys.mouse_input.dragged(Some(MouseButton::Left), None) != (0.0, 0.0) {
            self.sys.is_anything_dragged = true;
        } else {
            self.sys.is_anything_dragged = false;
        }
    }

    pub(crate) fn resolve_click_release(&mut self, _button: MouseButton) {
        self.sys.new_ui_input = true;
    }

    // returns if the ui consumed the mouse press, or if it should be passed down. 
    pub(crate) fn resolve_click_press(&mut self, button: MouseButton) -> bool {
        self.sys.new_ui_input = true;

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
            let clicked_node_i = self.nodes.node_hashmap.get(&clicked_id).unwrap().slab_i;
            let clicked_node = &mut self.nodes.nodes[clicked_node_i];

            if clicked_node.params.interact.click_animation {

                clicked_node.last_click = t.as_secs_f32();
                
                self.sys.changes.cosmetic_rect_updates.push(clicked_node_i);
                
                self.sys.anim_render_timer.push_new(Duration::from_secs_f32(ANIMATION_RERENDER_TIME));
            }
                
            if clicked_node.text_id.is_some() {
                if let Some(text) = clicked_node.params.text_params{
                    if text.editable {
                        self.sys.focused = Some(clicked_id);
                    }
                }
            }

            if let Some(id) = clicked_node.text_id {
                let text_area = &mut self.sys.text.text_areas[id];
                let cursor_pos = self.sys.mouse_input.cursor_position();
                let (x, y) = (
                    cursor_pos.x as f32 - text_area.params.left,
                    cursor_pos.y as f32 - text_area.params.top,
                );

                text_area.buffer.hit(x, y);
            }

        }
   
        let consumed = self.sys.mouse_input.current_tag().is_some();
        return consumed;
    }

    pub(crate) fn scan_mouse_hits(&mut self) -> Option<Id> {
        self.sys.mouse_hit_stack.clear();

        for rect in &self.sys.rects {
            if mouse_hit_rect(rect, &self.sys.unifs.size, self.cursor_position()) {
                self.sys.mouse_hit_stack.push((rect.id, rect.z));
            }
        }

        for rect in &self.sys.invisible_but_clickable_rects {
            if mouse_hit_rect(rect, &self.sys.unifs.size, self.cursor_position()) {
                self.sys.mouse_hit_stack.push((rect.id, rect.z));
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

        for rect in &self.sys.scroll_rects {
            if mouse_hit_rect(rect, &self.sys.unifs.size, self.cursor_position()) {
                self.sys.mouse_hit_stack.push((rect.id, rect.z));
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
                        self.set_debug_mode(!self.debug_mode());
                        self.sys.new_ui_input = true;
                    }
                }

                self.sys.debug_key_pressed = event.state.is_pressed();
            }
        }

        return false;
    }

    pub(crate) fn handle_scroll(&mut self, delta: &MouseScrollDelta) {
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
                self.nodes[i].scroll_offset[axis] += delta[axis];
            };
        }

        if self.nodes[i].params.is_scrollable() {
            self.recursive_place_children(i, true);
            
            self.sys.changes.need_gpu_rect_update = true;
            self.sys.changes.need_rerender = true;
        }
    }
}
