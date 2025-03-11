use std::time::{Duration, Instant};

use winit::{dpi::PhysicalPosition, event::{KeyEvent, MouseButton, MouseScrollDelta}, keyboard::{Key, NamedKey}};

use crate::*;
use crate::Axis::{X, Y};

pub(crate) const ANIMATION_RERENDER_TIME: f32 = 0.5;

#[derive(Clone, Copy, Debug)]
pub struct Click {
    pub absolute_position: glam::DVec2,
    pub relative_position: glam::DVec2,
    pub timestamp: Instant,
}

#[derive(Clone, Copy, Debug)]
pub struct Drag {
    /// Position relative to the node (0.0 to 1.0 in each dimension)
    pub relative_position: glam::DVec2,
    /// Absolute screen position in pixels
    pub absolute_position: glam::DVec2,
    /// Delta movement relative to the node's dimensions (as a fraction)
    pub relative_delta: glam::DVec2,
    /// Absolute delta movement in pixels
    pub absolute_delta: glam::DVec2,
    /// Time when the drag event occurred
    pub timestamp: Instant,
}

// todo: remove all of this crap
impl<'a> UiNode<'a> {
    /// Returns `true` if the node was just clicked with the left mouse button.
    /// 
    /// This is "act on press", you might want [is_click_released()](Self::is_click_released()).
    pub fn is_clicked(&mut self) -> bool {
        let id = self.ui.nodes[self.i].id;
        let clicked = self.ui.sys.mouse_input.clicked(Some(MouseButton::Left), Some(id));
        return clicked;
    }

    /// Returns `true` if a left button mouse click was just released on the node.
    pub fn is_click_released(&self) -> bool {
        let id = self.ui.nodes[self.i].id;
        return self.ui.sys.mouse_input.click_released(Some(MouseButton::Left), Some(id));
    }

    /// If the node was being held with the left mouse button in the last frame, returns the duration for which it was held.
    pub fn is_held(&self) -> Option<Duration> {let id 
        = self.ui.nodes[self.i].id;
        return self.ui.sys.mouse_input.held(Some(MouseButton::Left), Some(id));
    }

    /// If the node was dragged, returns the distance dragged. Otherwise, returns `(0.0, 0.0)`.
    pub fn is_dragged(&self) -> (f64, f64) {
        let id = self.ui.nodes[self.i].id;
        return self.ui.sys.mouse_input.dragged(Some(MouseButton::Left), Some(id));
    }

    /// Returns `true` if a node is currently hovered by the cursor.
    pub fn is_hovered(&self) -> bool {
        let id = self.ui.nodes[self.i].id;
        return self.ui.sys.hovered.last() == Some(&id);
    }

    /// Returns `true` if the node was just clicked with the `mouse_button`.
    /// 
    /// This is "act on press", you might want [is_click_released()](Self::is_click_released()).
    pub fn is_mouse_button_clicked(&self, mouse_button: MouseButton) -> bool {
        let id = self.ui.nodes[self.i].id;
        return self.ui.sys.mouse_input.clicked(Some(mouse_button), Some(id));
    }

    /// Returns `true` if a `mouse_button` click was just released on the node.
    pub fn is_mouse_button_click_released(&self, mouse_button: MouseButton) -> bool {
        let id = self.ui.nodes[self.i].id;
        return self.ui.sys.mouse_input.click_released(Some(mouse_button), Some(id));
    }

    /// If the node was being held with `mouse_button` in the last frame, returns the duration for which it was held.
    pub fn is_mouse_button_held(&self, mouse_button: MouseButton) -> Option<Duration> {
        let id = self.ui.nodes[self.i].id;
        return self.ui.sys.mouse_input.held(Some(mouse_button), Some(id));
    }

    /// If the node was dragged, returns the distance dragged. Otherwise, returns `(0.0, 0.0)`.
    pub fn is_mouse_button_dragged(&self, mouse_button: MouseButton) -> (f64, f64) {
        let id = self.ui.nodes[self.i].id;
        return self.ui.sys.mouse_input.dragged(Some(mouse_button), Some(id));
    }

    // todo: the rest of the interact functions
}


impl Ui {
    #[cfg(debug_assertions)]
    fn check_node_sense(&self, id: Id, sense: Sense, fn_name: &'static str) -> bool {
        if let Some((node, _)) = self.nodes.get_by_id(&id) {
            if !node.params.interact.senses.contains(sense) {
                log::error!(
                    "Debug mode check: {} was called on node {}, but the node doesn't have the {:?} sense.",
                    fn_name,
                    node.debug_name(),
                    sense
                );
                return false;
            }
        }
        return true;
    }

    /// Returns `true` if the node corresponding to `key` was just clicked with the left mouse button.
    /// 
    /// This is "act on press". For "act on release", see [is_click_released()](Self::is_click_released()).
    pub fn is_clicked(&mut self, node_key: NodeKey) -> bool {
        let id = node_key.id_with_subtree();
        #[cfg(debug_assertions)] {
            if ! self.check_node_sense(id, Sense::CLICK, "is_clicked") {
                return false;
            }
        }
        let clicked = self.sys.mouse_input.clicked(Some(MouseButton::Left), Some(id));
        return clicked;
    }

    /// If the node corresponding to `key` was clicked in the last frame, returns a struct containing the timestamp and position of the click. Otherwise, returns `None`.
    /// 
    /// If the node was clicked multiple times in the last frame, the result holds the information about the last click only.
    pub fn clicked_at(&mut self, node_key: NodeKey) -> Option<Click> {
        let id = node_key.id_with_subtree();
        #[cfg(debug_assertions)] {
            if !self.check_node_sense(id, Sense::CLICK, "clicked_at") {
                return None;
            }
        }
        let mouse_record = self.sys.mouse_input.clicked_at(Some(MouseButton::Left), Some(id))?;
        let i = self.nodes.node_hashmap.get(&id).unwrap().slab_i;
        let node_rect = self.nodes[i].rect;
        
        let relative_position = glam::DVec2::new(
            ((mouse_record.position.x / self.sys.unifs.size.x as f64) - (node_rect.x[0]) as f64) / node_rect.size().x as f64,
            ((mouse_record.position.y / self.sys.unifs.size.y as f64) - (node_rect.y[0]) as f64) / node_rect.size().y as f64,
        );
        
        return Some(Click {
            relative_position,
            absolute_position: mouse_record.position,
            timestamp: mouse_record.timestamp,
        });
    }

    /// Returns `true` if a left button mouse click was just released on the node corresponding to `key`.
    pub fn is_click_released(&self, node_key: NodeKey) -> bool {
        let id = node_key.id_with_subtree();
        #[cfg(debug_assertions)] {
            if ! self.check_node_sense(id, Sense::CLICK, "is_click_released") {
                return false;
            }
        }
        return self.sys.mouse_input.click_released(Some(MouseButton::Left), Some(id));
    }

    /// If the node corresponding to `key` was being held with the left mouse button in the last frame, returns the duration for which it was held.
    pub fn is_held(&self, node_key: NodeKey) -> Option<Duration> {
        let id = node_key.id_with_subtree();
        #[cfg(debug_assertions)] {
            if ! self.check_node_sense(id, Sense::HOLD, "is_held") {
                return None;
            }
        }
        return self.sys.mouse_input.held(Some(MouseButton::Left), Some(id));
    }

    /// If the node corresponding to `key` was dragged, returns the distance dragged. Otherwise, returns `(0.0, 0.0)`.
    pub fn is_dragged(&mut self, node_key: NodeKey) -> Option<Drag> {
        let id = node_key.id_with_subtree();
        #[cfg(debug_assertions)] {
            if !self.check_node_sense(id, Sense::DRAG, "dragged_at") {
                return None;
            }
        }
        let mouse_record = self.sys.mouse_input.dragged_at(Some(MouseButton::Left), Some(id))?;
        let i = self.nodes.node_hashmap.get(&id).unwrap().slab_i;
        let node_rect = self.nodes[i].rect;
        let relative_position = glam::DVec2::new(
            ((mouse_record.currently_at.position.x / self.sys.unifs.size.x as f64) - (node_rect.x[0]) as f64) / node_rect.size().x as f64,
            ((mouse_record.currently_at.position.y / self.sys.unifs.size.y as f64) - (node_rect.y[0]) as f64) / node_rect.size().y as f64,
        );
        let relative_delta = glam::DVec2::new(
            mouse_record.drag_distance().x / (node_rect.size().x as f64 * self.sys.unifs.size.x as f64),
            mouse_record.drag_distance().y / (node_rect.size().y as f64 * self.sys.unifs.size.y as f64),
        );
        return Some(Drag {
            relative_position,
            absolute_position: mouse_record.currently_at.position,
            relative_delta,
            absolute_delta: mouse_record.drag_distance(),
            timestamp: mouse_record.currently_at.timestamp,
        });
    }

    /// Returns `true` if a node is currently hovered by the cursor.
    pub fn is_hovered(&self, node_key: NodeKey) -> bool {
        let id = node_key.id_with_subtree();
        #[cfg(debug_assertions)] {
            if ! self.check_node_sense(id, Sense::HOVER, "is_hovered") {
                return false;
            }
        }
        return self.sys.hovered.last() == Some(&id);
    }

    /// Returns `true` if the node corresponding to `key` was just clicked with the `mouse_button`.
    /// 
    /// This is "act on press". For "act on release", use [Ui::is_click_released()].
    pub fn is_mouse_button_clicked(&self, node_key: NodeKey, mouse_button: MouseButton) -> bool {
        return self.sys.mouse_input.clicked(Some(mouse_button), Some(node_key.id_with_subtree()));
    }

    /// Returns `true` if a `mouse_button` click was just released on the node corresponding to `key`.
    pub fn is_mouse_button_click_released(&self, node_key: NodeKey, mouse_button: MouseButton) -> bool {
        return self.sys.mouse_input.click_released(Some(mouse_button), Some(node_key.id_with_subtree()));
    }

    /// If the node corresponding to `key` was being held with `mouse_button` in the last frame, returns the duration for which it was held.
    pub fn is_mouse_button_held(&self, node_key: NodeKey, mouse_button: MouseButton) -> Option<Duration> {
        return self.sys.mouse_input.held(Some(mouse_button), Some(node_key.id_with_subtree()));
    }

    /// If the node corresponding to `key` was dragged, returns the distance dragged. Otherwise, returns `(0.0, 0.0)`.
    pub fn is_mouse_button_dragged(&self, node_key: NodeKey, mouse_button: MouseButton) -> (f64, f64) {
        return self.sys.mouse_input.dragged(Some(mouse_button), Some(node_key.id_with_subtree()));
    }

    // todo: think if it's really worth it to do this on every mouse movement.
    // maybe add a global setting to do it just once per frame
    pub(crate) fn resolve_hover(&mut self) {
        // real hover
        let hovered_node_id = self.scan_mouse_hits();
        self.sys.mouse_input.update_current_tag(hovered_node_id);

        if let Some(hovered_id) = hovered_node_id {
            if self.sys.hovered.contains(&hovered_id) {
                let hovered_i = self.nodes.node_hashmap.get(&hovered_id).unwrap().slab_i;
                if self.nodes[hovered_i].params.interact.senses.contains(Sense::HOVER) {
                    self.set_new_ui_input();
                }

                if self.nodes[hovered_i].params.interact.senses.contains(Sense::DRAG)
                    && self.sys.mouse_input.held(Some(MouseButton::Left), Some(hovered_id)).is_some() {
                    self.set_new_ui_input();
                }

            } else {
                // newly entered
                let (_, hovered_node_i) = self.nodes.get_mut_by_id(&hovered_id).unwrap();
                if self.inspect_mode() {
                    log::info!("Inspect mode: hovering {}", self.node_debug_name_fmt_scratch(hovered_node_i))
                }
                self.end_all_hovering();
                self.start_hovering(hovered_id);

                if self.nodes[hovered_node_i].params.interact.senses.contains(Sense::HOVER) {
                    self.set_new_ui_input();
                }
                if self.nodes[hovered_node_i].params.interact.click_animation {
                    // // don't do this
    //                 self.set_new_ui_input();
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
            let hovered_nodemap_entry = self.nodes.node_hashmap.get(&hovered_id);
            
            if let Some(entry) = hovered_nodemap_entry {
                // check that the node is currently part of the tree...
                // this is a bit scary, and it will need to change with `assume_unchanged` and friends
                if entry.last_frame_touched == self.sys.current_frame {

                    let hovered_node_i = entry.slab_i;
                    let hovered_node = &mut self.nodes[hovered_node_i];
                    
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
            // // don't do this
//                 self.set_new_ui_input();
            self.sys.anim_render_timer.push_new(Duration::from_secs_f32(ANIMATION_RERENDER_TIME));
        }

        self.sys.hovered.clear();
    }

    pub(crate) fn begin_frame_resolve_inputs(&mut self) {
        self.sys.mouse_input.begin_new_frame();
        self.sys.key_input.begin_new_frame();
    }

    pub(crate) fn resolve_click_release(&mut self, _button: MouseButton) {
        // todo: there's something wrong here, releasing a click doesn't wake up the event loop somehow (it stays dark)
        self.set_new_ui_input();
    }

    // returns if the ui consumed the mouse press, or if it should be passed down. 
    pub(crate) fn resolve_click_press(&mut self, button: MouseButton) -> bool {
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

        for clk_i in 0..self.sys.click_rects.len() {
            let clk_rect = self.sys.click_rects[clk_i];
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
                self.update_scroll(i, delta[axis], axis);
            };
        }

        if self.nodes[i].params.is_scrollable() {
            self.recursive_place_children(i, true);
            
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
            rect: self.nodes[i].rect,
            i,
        }
    }
}