use std::time::Duration;

use glam::Vec2;
use winit::event::MouseButton;

use crate::*;
use crate::inner_node::*;
use crate::mouse_events::{DragEvent, DragReleaseEvent};
use crate::Axis::*;

/// A struct representing a node within the [`Ui`].
pub struct UiNode<'a> {
    pub(crate) i: NodeI,
    pub(crate) ui: &'a Ui,
}

pub struct UiNodeChildrenIter<'a> {
    ui: &'a Ui,
    current: Option<NodeI>,
    remaining: usize,
}

impl<'a> Iterator for UiNodeChildrenIter<'a> {
    type Item = UiNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(child_i) = self.current {
            self.current = self.ui.nodes[child_i].next_sibling;
            if !self.ui.nodes[child_i].exiting {
                self.remaining -= 1;
                return Some(UiNode { ui: self.ui, i: child_i });
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl ExactSizeIterator for UiNodeChildrenIter<'_> {}

impl<'a> UiNode<'a> {
    /// Get an iterator over all the children added to the node so far.
    pub fn children(&self) -> UiNodeChildrenIter<'a> {
        UiNodeChildrenIter {
            ui: self.ui,
            current: self.ui.nodes[self.i].first_child,
            remaining: self.ui.nodes[self.i].n_children as usize,
        }
    }

    /// Get the number of children added to the node so far.
    pub fn children_count(&self) -> usize {
        self.ui.nodes[self.i].n_children as usize
    }

    /// Returns `true` if the node is currently hidden (excluded from the tree but retained in memory).
    pub fn is_hidden(&self) -> bool {
        self.ui.nodes[self.i].currently_hidden
    }

    /// Returns `true` if the node is currently playing its exit animation before being removed.
    pub fn is_exiting(&self) -> bool {
        self.ui.nodes[self.i].exiting
    }

    pub(crate) fn node(&self) -> &InnerNode {
        return &self.ui.nodes[self.i];
    }

    /// Returns the key that can be used to refer to this node.
    ///
    /// Note: This key should be used in the same subtree context as where the node was added.
    pub fn temp_key(&self) -> NodeKey {
        NodeKey::new_temp(self.node().id, "temp_node_key")
    }

    pub(crate) fn last_frame_inner_size(&self) -> Xy<f32> {
        let padding = self.node().params.layout.padding;

        let size = self.node().size;
        let size = self.ui.f32_size_to_pixels2(size);

        return size - padding;
    }

    pub(crate) fn last_frame_center(&self) -> Xy<f32> {
        let rect = self.node().real_rect;
        
        let center = Xy::new(
            (rect[X][1] + rect[X][0]) / 2.0,
            (rect[Y][1] + rect[Y][0]) / 2.0,
        );

        let center = center * self.ui.sys.size;

        return center;
    }

    pub(crate) fn last_frame_bottom_left(&self) -> Xy<f32> {
        let rect = self.node().real_rect;
        
        let center = Xy::new(
            rect[X][0],
            rect[Y][1],
        );

        let center = center * self.ui.sys.size;
        
        return center;
    }

    pub(crate) fn last_frame_rect(&self) -> XyRect {
        return self.node().real_rect * self.ui.sys.size;
    }

    pub(crate) fn render_rect(&self) -> RenderInfo {
        let size = self.ui.sys.size;
        let scale = self.node().accumulated_transform.scale;
        return RenderInfo {
            rect: self.node().real_rect.to_graphics_space_rounded(size, scale),
            z: self.node().z + Z_STEP / 2.0,
        };
    }

    /// Returns the text content if it was changed by user input this frame, otherwise `None`.
    ///
    /// Only works for text edit nodes. Returns `None` for regular text nodes.
    pub fn text_edit_changed(&self) -> Option<&'a str> {
        if let Some(TextI::TextEdit(handle)) = &self.node().text_i {
            let text_edit = self.ui.sys.renderer.text.get_text_edit(&handle);
            if text_edit.text_changed() {
                return Some(text_edit.raw_text());
            }
        }
        None
    }
}

impl Ui {
    /// Get the [`UiNode`] corresponding to the `key`, if such a node is currently part of the visible UI tree.
    /// 
    /// This function will return the node if it exists and it is both visible and interactable. So it will return `None` if the node exists but it is hidden or if it is doing an exiting animation right before disappearing. Use also [`Ui::get_node_unfiltered`] for a version that also returns hidden and exiting nodes.
    ///
    /// If the same key was used to add multiple nodes in the same frame, the key will always return the first one. You can use [`NodeKey::sibling()`] to create different "versions" of the same key dynamically and still be able to point to them.
    /// 
    pub fn get_node(&self, key: NodeKey) -> Option<UiNode<'_>> {
        let i = self.nodes.node_hashmap.get(&key.id_with_subtree())?.slab_i;
        if self.nodes[i].currently_hidden || self.nodes[i].exiting {
            return None;
        }
        return Some(UiNode { i, ui: self });
    }

    /// Like [`Ui::get_node`], but also returns nodes that are currently hidden or exiting.
    ///
    /// You can use [`UiNode::is_hidden`] and [`UiNode::is_exiting`] to filter by state as needed.
    pub fn get_node_unfiltered(&self, key: NodeKey) -> Option<UiNode<'_>> {
        let i = self.nodes.node_hashmap.get(&key.id_with_subtree())?.slab_i;
        return Some(UiNode { i, ui: self });
    }

    pub fn render_rect(&self, key: NodeKey) -> Option<RenderInfo> {
        Some(self.get_node(key)?.render_rect())
    }
    pub fn z_value(&self, key: NodeKey) -> Option<f32> {
        Some(self.get_node(key)?.render_rect().z)
    }
    /// Dimensions of the rect in screen pixels
    pub fn rect(&self, key: NodeKey) -> Option<XyRect> {
        Some(self.get_node(key)?.last_frame_rect())
    }
    pub fn center(&self, key: NodeKey) -> Option<Xy<f32>> {
        Some(self.get_node(key)?.last_frame_center())
    }
    pub fn inner_size(&self, key: NodeKey) -> Option<Xy<f32>> {
        Some(self.get_node(key)?.last_frame_inner_size())
    }
    pub fn bottom_left(&self, key: NodeKey) -> Option<Xy<f32>> {
        Some(self.get_node(key)?.last_frame_bottom_left())
    }
    pub fn get_text(&mut self, key: NodeKey) -> Option<&str> {
        let i = self.nodes.node_hashmap.get(&key.id_with_subtree())?.slab_i;
        let text_i = self.nodes[i].text_i.as_ref()?;
        match text_i {
            TextI::TextBox(handle) => Some(self.sys.renderer.text.get_text_box(&handle).text()),
            TextI::TextEdit(handle) => Some(self.sys.renderer.text.get_text_edit(&handle).raw_text()),
        }
    }
    pub fn set_text(&mut self, key: NodeKey, text: &str) -> Option<()> {
        let i = self.nodes.node_hashmap.get(&key.id_with_subtree())?.slab_i;
        let text_i = self.nodes[i].text_i.as_ref()?;
        match text_i {
            TextI::TextBox(handle) => self.sys.renderer.text.get_text_box_mut(&handle).set_text_hashed(text),
            TextI::TextEdit(handle) => self.sys.renderer.text.get_text_edit_mut(&handle).set_text_hashed(text),
        };
        Some(())
    }

    /// Get the rects (in screen-fraction coords) of all children of a node.
    /// Returns rects from the previous frame's layout.
    pub fn children_rects(&self, key: NodeKey) -> Vec<XyRect> {
        let mut rects = Vec::new();
        let Some(parent_i) = self.nodes.node_hashmap.get(&key.id_with_subtree()).map(|e| e.slab_i) else {
            return rects;
        };
        let mut current = self.nodes[parent_i].first_child;
        while let Some(child_i) = current {
            if !self.nodes[child_i].exiting {
                rects.push(self.nodes[child_i].real_rect);
            }
            current = self.nodes[child_i].next_sibling;
        }
        rects
    }
}

impl UiParent {
    pub(crate) fn get_uinode<'a>(&self, ui: &'a Ui) -> UiNode<'a> {
        return UiNode { i: self.i, ui };
    }
    
    pub fn render_rect(&self, ui: &mut Ui) -> RenderInfo {
        self.get_uinode(ui).render_rect()
    }
    pub fn rect(&self, ui: &mut Ui) -> XyRect {
        self.get_uinode(ui).last_frame_rect()
    }
    pub fn center(&self, ui: &mut Ui) -> Xy<f32> {
        self.get_uinode(ui).last_frame_center()
    }
    pub fn bottom_left(&self, ui: &mut Ui) -> Xy<f32> {
        self.get_uinode(ui).last_frame_bottom_left()
    }
}

/// The data needed for rendering a node with custom code.
/// 
/// Obtained with [`Ui::render_rect`] and a key.
/// 
/// The data is ready to be used in a shader like this:
/// 
/// ```wgsl
/// struct Rect {
///     @location(0) xs: vec2<f32>,
///     @location(1) ys: vec2<f32>,
///     @location(2) z: f32,
/// }
/// ```
/// 
/// With these vertex attributes:
/// 
/// ```rust
/// # use keru::*;
/// wgpu::vertex_attr_array![
///     0 => Float32x2,
///     1 => Float32x2,
///     2 => Float32,
/// ];
/// ```
/// 
/// The format might be changed to something more familiar in the future.
/// 
/// This doesn't include the information about the `Shape`, because it's harder to interpret, and it's usually static.
#[derive(Copy, Clone, Debug)]
pub struct RenderInfo {
    pub rect: XyRect,
    pub z: f32,
}


// pub(crate) struct UiNodeMut<'a> {
//     pub i: NodeI,
//     pub ui: &'a mut Ui,
// }
// impl UiNodeMut<'_> {
//     pub(crate) fn node(&self) -> &mut Node {
//         return &self.ui.nodes[self.i];
//     }
// }



impl UiNode<'_> {
    #[cfg(debug_assertions)]
    fn check_node_sense(&self, sense: Sense, fn_name: &'static str, sense_add_fn_name: &'static str) -> bool {
        let node = self.node();
        if !node.params.interact.senses.contains(sense) {
            eprintln!(
                "Keru: Debug mode check: \"{}\" was called for node {}, but the node doesn't have the {:?} sense. In release mode, this event will be silently ignored! You can add the sense to the node's Node with the \"{}\" function.",
                fn_name,
                node.debug_name(),
                sense,
                sense_add_fn_name,
            );
            return false;
        }
        return true;
    }

    #[cfg(debug_assertions)]
    fn check_dest_node_sense(&self, dest_key: NodeKey, sense: Sense, fn_name: &'static str, sense_add_fn_name: &'static str) -> bool {
        let Some(dest_node) = self.ui.get_node(dest_key) else {
            return true; // Node doesn't exist, let the function return false naturally
        };
        let dest_node = dest_node.node();
        if !dest_node.params.interact.senses.contains(sense) {
            eprintln!(
                "Keru: Debug mode check: \"{}\" was called with destination node {}, but the destination node doesn't have the {:?} sense. In release mode, this event will be silently ignored! You can add the sense to the node's Node with the \"{}\" function.",
                fn_name,
                dest_node.debug_name(),
                sense,
                sense_add_fn_name,
            );
            return false;
        }
        return true;
    }

    /// Returns `true` if the node corresponding to `key` was just clicked with the left mouse button.
    /// 
    /// This is "act on press". For "act on release", see [`Self::is_click_released()`].
    pub fn is_clicked(&self) -> bool {

        #[cfg(debug_assertions)]
        if ! self.check_node_sense(Sense::CLICK, "is_clicked()", "Node::sense_click()") {
            return false;
        }

        self.ui.check_clicked(self.node().id, MouseButton::Left)
    }

    /// Returns `true` if the node corresponding to `key` was just clicked with the right mouse button.
    /// 
    /// This is "act on press". For "act on release", see [`Self::is_right_click_released()`].
    pub fn is_right_clicked(&self) -> bool {

        #[cfg(debug_assertions)]
        if ! self.check_node_sense(Sense::CLICK, "is_clicked()", "Node::sense_click()") {
            return false;
        }

        self.ui.check_clicked(self.node().id, MouseButton::Right)
    }

    /// Returns `true` if the node corresponding to `key` was just clicked with a mouse button.
    /// 
    /// This is "act on press". For "act on release", see [`Self::is_right_click_released()`].
    pub fn is_mouse_button_clicked(&self, button: winit::event::MouseButton) -> bool {

        #[cfg(debug_assertions)]
        if ! self.check_node_sense(Sense::CLICK, "is_clicked()", "Node::sense_click()") {
            return false;
        }

        self.ui.check_clicked(self.node().id, button)
    }

    pub fn is_focused(&self) -> bool {
        return self.ui.sys.focused == Some(self.node().id);
    }

    /// If the node corresponding to `key` was clicked in the last frame, returns a struct containing detailed information of the click. Otherwise, returns `None`.
    ///
    /// If the node was clicked multiple times in the last frame, the result holds the information about the last click only.
    pub fn clicked_at(&self) -> Option<Click> {
        #[cfg(debug_assertions)]
        if ! self.check_node_sense(Sense::CLICK, "clicked_at()", "Node::sense_click()") {
            return None;
        }

        let event = self.ui.check_clicked_at(self.node().id, MouseButton::Left)?;
        let node_rect = self.node().real_rect;

        let relative_position = glam::Vec2::new(
            ((event.position.x / self.ui.sys.size.x) - node_rect.x[0]) / node_rect.size().x,
            ((event.position.y / self.ui.sys.size.y) - node_rect.y[0]) / node_rect.size().y,
        );

        Some(Click {
            relative_position,
            absolute_position: event.position,
            timestamp: event.timestamp,
        })
    }

    /// Returns `true` if a left button mouse click was just released on the node corresponding to `key`.
    pub fn is_click_released(&self) -> bool {
        #[cfg(debug_assertions)]
        if ! self.check_node_sense(Sense::CLICK_RELEASE, "is_click_released()", "Node::sense_click()") {
            return false;
        }

        self.ui.check_click_released(self.node().id, MouseButton::Left)
    }

    /// Returns `true` if a left button mouse drag on the node corresponding to `key` was just released.
    ///
    /// Unlike [`Self::is_click_released()`], this is `true` even if the pointer is not on the node anymore when the button is released.
    pub fn is_drag_released(&self) -> bool {
        #[cfg(debug_assertions)]
        if ! self.check_node_sense(Sense::DRAG, "is_drag_released()", "Node::sense_drag()") {
            return false;
        }

        self.ui.check_drag_released(self.node().id, MouseButton::Left)
    }

    /// If a left button mouse drag on this node was just released onto the node corresponding to the `dest` key, returns the drag info.
    /// The `relative_position` in the returned `Drag` is relative to the destination node.
    pub fn is_drag_released_onto(&self, dest: NodeKey) -> Option<Drag> {
        #[cfg(debug_assertions)]
        if ! self.check_node_sense(Sense::DRAG, "is_drag_released_onto()", "Node::sense_drag()") {
            return None;
        }
        #[cfg(debug_assertions)]
        if ! self.check_dest_node_sense(dest, Sense::DRAG_DROP_TARGET, "is_drag_released_onto()", "Node::sense_drag_drop_target()") {
            return None;
        }

        let event = self.ui.check_drag_released_onto(self.node().id, dest.id_with_subtree(), MouseButton::Left)?;
        let dest_rect = self.ui.get_node(dest)?.node().real_rect;
        self.drag_from_release_event_with_rect(event, dest_rect)
    }

    /// If a left button mouse drag on this node is currently hovering over the node corresponding to the `dest` key, returns the drag info.
    /// The `relative_position` in the returned `Drag` is relative to the destination node.
    pub fn is_drag_hovered_onto(&self, dest: NodeKey) -> Option<Drag> {
        #[cfg(debug_assertions)]
        if ! self.check_node_sense(Sense::DRAG, "is_drag_hovered_onto()", "Node::sense_drag()") {
            return None;
        }
        #[cfg(debug_assertions)]
        if ! self.check_dest_node_sense(dest, Sense::DRAG_DROP_TARGET, "is_drag_hovered_onto()", "Node::sense_drag_drop_target()") {
            return None;
        }

        let event = self.ui.check_drag_hovered_onto(self.node().id, dest.id_with_subtree(), MouseButton::Left)?;
        let dest_rect = self.ui.get_node(dest)?.node().real_rect;
        self.drag_from_event_with_rect(event, dest_rect)
    }

    fn drag_from_event(&self, event: &DragEvent) -> Option<Drag> {
        self.drag_from_event_with_rect(event, self.node().real_rect)
    }

    fn drag_from_event_with_rect(&self, event: &DragEvent, node_rect: XyRect) -> Option<Drag> {
        let relative_position = glam::Vec2::new(
            ((event.current_pos.x / self.ui.sys.size.x) - node_rect.x[0]) / node_rect.size().x,
            ((event.current_pos.y / self.ui.sys.size.y) - node_rect.y[0]) / node_rect.size().y,
        );
        let relative_delta = glam::Vec2::new(
            event.frame_delta.x / (node_rect.size().x * self.ui.sys.size.x),
            event.frame_delta.y / (node_rect.size().y * self.ui.sys.size.y),
        );

        if event.total_delta == Vec2::ZERO {
            return None;
        }

        Some(Drag {
            relative_position,
            absolute_pos: event.current_pos,
            relative_delta,
            absolute_delta: event.frame_delta,
            pressed_timestamp: event.start_time,
            total_drag_distance: event.total_delta,
        })
    }

    fn drag_from_release_event_with_rect(&self, event: &DragReleaseEvent, node_rect: XyRect) -> Option<Drag> {
        let relative_position = glam::Vec2::new(
            ((event.end_pos.x / self.ui.sys.size.x) - node_rect.x[0]) / node_rect.size().x,
            ((event.end_pos.y / self.ui.sys.size.y) - node_rect.y[0]) / node_rect.size().y,
        );

        Some(Drag {
            relative_position,
            absolute_pos: event.end_pos,
            relative_delta: Vec2::ZERO, // No frame delta on release
            absolute_delta: Vec2::ZERO,
            pressed_timestamp: event.start_time,
            total_drag_distance: event.total_delta,
        })
    }

    /// If the node was dragged with a specific mouse button, returns a struct describing the drag event. Otherwise, returns `None`.
    pub fn is_mouse_button_dragged(&self, button: winit::event::MouseButton) -> Option<Drag> {
        #[cfg(debug_assertions)]
        if ! self.check_node_sense(Sense::DRAG, "is_mouse_button_dragged()", "Node::sense_drag()") {
            return None;
        }

        let event = self.ui.check_dragged(self.node().id, button)?;
        self.drag_from_event(event)
    }

    /// If the node corresponding to `key` was dragged, returns a struct describing the drag event. Otherwise, returns `None`.
    pub fn is_dragged(&self) -> Option<Drag> {
        self.is_mouse_button_dragged(MouseButton::Left)
    }

    /// If the node corresponding to `key` was being held with the left mouse button in the last frame, returns the duration for which it was held.
    pub fn is_held(&self) -> Option<Duration> {
        #[cfg(debug_assertions)]
        if ! self.check_node_sense(Sense::HOLD, "is_held()", "Node::sense_hold()") {
            return None;
        }

        self.ui.check_held_duration(self.node().id, MouseButton::Left)
    }

    /// If the node is currently hovered by the cursor, returns hover information including position.
    pub fn is_hovered(&self) -> Option<Hover> {
        #[cfg(debug_assertions)]
        if ! self.check_node_sense(Sense::HOVER, "is_hovered", "Node::sense_hover()") {
            return None;
        }

        if self.ui.check_hovered(self.node().id) {
            Some(Hover {
                absolute_position: self.ui.cursor_position(),
            })
        } else {
            None
        }
    }

    /// If the node was scrolled in the last frame, returns a struct containing detailed information of the scroll event. Otherwise, returns `None`.
    ///
    /// If the node was scrolled multiple times in the last frame, the result holds the information about the last scroll only.
    pub fn scrolled_at(&self) -> Option<ScrollEvent> {
        #[cfg(debug_assertions)]
        if ! self.check_node_sense(Sense::SCROLL, "scrolled_at()", "Node::sense_scroll()") {
            return None;
        }

        let scroll_event = self.ui.check_last_scroll_event(self.node().id)?;
        let node_rect = self.node().real_rect;

        let relative_position = glam::Vec2::new(
            ((scroll_event.position.x / self.ui.sys.size.x) - node_rect.x[0]) / node_rect.size().x,
            ((scroll_event.position.y / self.ui.sys.size.y) - node_rect.y[0]) / node_rect.size().y,
        );

        Some(ScrollEvent {
            relative_position,
            absolute_position: scroll_event.position,
            delta: scroll_event.delta,
            timestamp: scroll_event.timestamp,
        })
    }

    /// Returns the total scroll delta for this node in the last frame, or None if no scroll events occurred.
    pub fn is_scrolled(&self) -> Option<glam::Vec2> {
        #[cfg(debug_assertions)]
        if ! self.check_node_sense(Sense::SCROLL, "is_scrolled()", "Node::sense_scroll()") {
            return None;
        }

        self.ui.check_scrolled(self.node().id)
    }
}



impl Ui {
    /// Returns `true` if the node corresponding to `key` was just clicked with the left mouse button.
    /// 
    /// This is "act on press". For "act on release", see [`Self::is_click_released()`].
    pub fn is_clicked(&self, key: NodeKey) -> bool {
        let Some(uinode) = self.get_node(key) else {
            return false;
        };
        uinode.is_clicked()
    }

    /// Returns `true` if the node corresponding to `key` was just clicked with the right mouse button.
    /// 
    /// This is "act on press". For "act on release", see [`Self::is_click_released()`].
    pub fn is_right_clicked(&self, key: NodeKey) -> bool {
        let Some(uinode) = self.get_node(key) else {
            return false;
        };
        uinode.is_right_clicked()
    }

    /// Returns `true` if the node corresponding to `key` was just clicked with the left mouse button.
    /// 
    /// This is "act on press". For "act on release", see [`Self::is_click_released()`].
    pub fn is_mouse_button_clicked(&self, key: NodeKey, button: winit::event::MouseButton) -> bool {
        let Some(uinode) = self.get_node(key) else {
            return false;
        };
        uinode.is_mouse_button_clicked(button)
    }

    /// Set placeholder text for a text edit node that will be shown when the text edit is empty.
    ///
    /// Does nothing for regular text nodes.
    pub fn set_text_edit_placeholder(&mut self, key: NodeKey, placeholder: &str) {
        let Some(i) = self.nodes.node_hashmap.get(&key.id_with_subtree()) else {
            return;
        };
        let i = i.slab_i;

        if let Some(TextI::TextEdit(handle)) = &self.nodes[i].text_i {
            self.sys.renderer.text.get_text_edit_mut(handle).set_placeholder(placeholder);
        }
    }

    /// Returns the text content if it was changed by user input this frame, otherwise `None`.
    ///
    /// Only works for text edit nodes. Returns `None` for regular text nodes.
    pub fn text_edit_changed(&mut self, key: NodeKey) -> Option<&str> {
        let Some(uinode) = self.get_node(key) else {
            return None;
        };
        uinode.text_edit_changed()
    }

    pub fn is_focused(&self, key: NodeKey) -> bool {
        let Some(uinode) = self.get_node(key) else {
            return false;
        };
        uinode.is_focused()
    }

    /// Returns `true` if a left button mouse click was just released on the node corresponding to `key`.
    pub fn is_click_released(&self, key: NodeKey) -> bool {
        let Some(uinode) = self.get_node(key) else {
            return false;
        };
        uinode.is_click_released()
    }

    /// Returns `true` if a left button mouse drag on the node corresponding to `key` was just released.
    /// 
    /// Unlike [`Self::is_click_released()`], this is `true` even if the cursor is not on the node anymore when the button is released. 
    pub fn is_drag_released(&self, key: NodeKey) -> bool {
        let Some(uinode) = self.get_node(key) else {
            return false;
        };
        uinode.is_drag_released()
    }

    /// If a left button mouse drag on the node corresponding to the `src` key was just released onto the node corresponding to the `dest` key, returns the drag info.
    pub fn is_drag_released_onto(&self, src: NodeKey, dest: NodeKey) -> Option<Drag> {
        self.get_node(src)?.is_drag_released_onto(dest)
    }

    /// If a left button mouse drag on the node corresponding to the `src` key is currently hovering over the node corresponding to the `dest` key, returns the drag info.
    pub fn is_drag_hovered_onto(&self, src: NodeKey, dest: NodeKey) -> Option<Drag> {
        self.get_node(src)?.is_drag_hovered_onto(dest)
    }

    /// If any node is currently being dragged over the node corresponding to `dest`, returns the drag info.
    /// The `relative_position` in the returned `Drag` is relative to the destination node.
    ///
    /// This is useful for drop targets that need to react to any dragged item, without knowing
    /// which specific item is being dragged.
    pub fn is_any_drag_hovered_onto(&self, dest: NodeKey) -> Option<Drag> {
        #[cfg(debug_assertions)]
        {
            let dest_i = self.nodes.node_hashmap.get(&dest.id_with_subtree())?.slab_i;
            if !self.nodes[dest_i].params.interact.senses.contains(Sense::DRAG_DROP_TARGET) {
                log::warn!(
                    "is_any_drag_hovered_onto() was called on node {:?}, but it doesn't have the DRAG_DROP_TARGET sense. Add Node::sense_drag_drop_target() to the node.",
                    dest.debug_name()
                );
                return None;
            }
        }

        let event = self.check_any_drag_hovered_onto(dest.id_with_subtree(), MouseButton::Left)?;
        let dest_node = self.get_node(dest)?;
        let dest_rect = dest_node.node().real_rect;

        let relative_position = glam::Vec2::new(
            ((event.current_pos.x / self.sys.size.x) - dest_rect.x[0]) / dest_rect.size().x,
            ((event.current_pos.y / self.sys.size.y) - dest_rect.y[0]) / dest_rect.size().y,
        );
        let relative_delta = glam::Vec2::new(
            event.frame_delta.x / (dest_rect.size().x * self.sys.size.x),
            event.frame_delta.y / (dest_rect.size().y * self.sys.size.y),
        );

        Some(Drag {
            relative_position,
            absolute_pos: event.current_pos,
            relative_delta,
            absolute_delta: event.frame_delta,
            pressed_timestamp: event.start_time,
            total_drag_distance: event.total_delta,
        })
    }

    /// If any node was just dropped (drag released) onto the node corresponding to `dest`, returns the drag info.
    /// The `relative_position` in the returned `Drag` is relative to the destination node.
    ///
    /// This is useful for drop targets that need to react to any dropped item, without knowing
    /// which specific item was dropped.
    pub fn is_any_drag_released_onto(&self, dest: NodeKey) -> Option<Drag> {
        #[cfg(debug_assertions)]
        {
            let dest_i = self.nodes.node_hashmap.get(&dest.id_with_subtree())?.slab_i;
            if !self.nodes[dest_i].params.interact.senses.contains(Sense::DRAG_DROP_TARGET) {
                log::warn!(
                    "is_any_drag_released_onto() was called on node {:?}, but it doesn't have the DRAG_DROP_TARGET sense. Add Node::sense_drag_drop_target() to the node.",
                    dest.debug_name()
                );
                return None;
            }
        }

        let event = self.check_any_drag_released_onto(dest.id_with_subtree(), MouseButton::Left)?;
        let dest_node = self.get_node(dest)?;
        let dest_rect = dest_node.node().real_rect;

        let relative_position = glam::Vec2::new(
            ((event.end_pos.x / self.sys.size.x) - dest_rect.x[0]) / dest_rect.size().x,
            ((event.end_pos.y / self.sys.size.y) - dest_rect.y[0]) / dest_rect.size().y,
        );

        Some(Drag {
            relative_position,
            absolute_pos: event.end_pos,
            relative_delta: Vec2::ZERO,
            absolute_delta: Vec2::ZERO,
            pressed_timestamp: event.start_time,
            total_drag_distance: event.total_delta,
        })
    }

    /// If the node corresponding to `key` was dragged with a specific mouse button, returns a struct describing the drag event. Otherwise, returns `None`.
    pub fn is_mouse_button_dragged(&self, key: NodeKey, button: winit::event::MouseButton) -> Option<Drag> {
        let Some(uinode) = self.get_node(key) else {
            return None;
        };
        uinode.is_mouse_button_dragged(button)
    }

    /// If the node corresponding to `key` was dragged, returns a struct describing the drag event. Otherwise, returns `None`.
    pub fn is_dragged(&self, key: NodeKey) -> Option<Drag> {
        let Some(uinode) = self.get_node(key) else {
            return None;
        };
        uinode.is_dragged()
    }

    /// If the node corresponding to `key` was clicked in the last frame, returns a struct containing detailed information of the click. Otherwise, returns `None`.
    /// 
    /// If the node was clicked multiple times in the last frame, the result holds the information about the last click only.
    pub fn clicked_at(&self, key: NodeKey) -> Option<Click> {
        let Some(uinode) = self.get_node(key) else {
            return None;
        };
        uinode.clicked_at()
    }

    /// If the node is currently hovered by the cursor, returns hover information including position.
    pub fn is_hovered(&self, key: NodeKey) -> Option<Hover> {
        let Some(uinode) = self.get_node(key) else {
            return None;
        };
        uinode.is_hovered()
    }

   /// If the node corresponding to `key` was being held with the left mouse button in the last frame, returns the duration for which it was held.
   pub fn is_held(&self, key: NodeKey) -> Option<Duration> {
        let Some(uinode) = self.get_node(key) else {
            return None;
        };
        uinode.is_held()
    }

    /// If the node corresponding to `key` was scrolled in the last frame, returns a struct containing detailed information of the scroll event. Otherwise, returns `None`.
    /// 
    /// If the node was scrolled multiple times in the last frame, the result holds the information about the last scroll only.
    pub fn scrolled_at(&self, key: NodeKey) -> Option<ScrollEvent> {
        let Some(uinode) = self.get_node(key) else {
            return None;
        };
        uinode.scrolled_at()
    }

    /// Returns the total scroll delta for the node corresponding to `key` in the last frame, or None if no scroll events occurred.
    pub fn is_scrolled(&self, key: NodeKey) -> Option<glam::Vec2> {
        let Some(uinode) = self.get_node(key) else {
            return None;
        };
        uinode.is_scrolled()
    }
}

impl UiParent {
    /// Returns `true` if the node was just clicked with the left mouse button.
    /// 
    /// This is "act on press". For "act on release", see [`Self::is_click_released()`].
    pub fn is_clicked(&self, ui: &mut Ui) -> bool {
        self.get_uinode(ui).is_clicked()
    }

    /// Returns `true` if a left button mouse click was just released on the node.
    pub fn is_click_released(&self, ui: &mut Ui) -> bool {
        self.get_uinode(ui).is_click_released()
    }

    /// If the node was clicked in the last frame, returns a struct containing detailed information of the click. Otherwise, returns `None`.
    /// 
    /// If the node was clicked multiple times in the last frame, the result holds the information about the last click only.
    pub fn clicked_at(&self, ui: &mut Ui) -> Option<Click> {
        self.get_uinode(ui).clicked_at()
    }

    /// If the node was dragged with a specific mouse button, returns a struct describing the drag event. Otherwise, returns `None`.
    pub fn is_mouse_button_dragged(&self, ui: &mut Ui, button: winit::event::MouseButton) -> Option<Drag> {
        self.get_uinode(ui).is_mouse_button_dragged(button)
    }

    /// If the node was dragged, returns a struct describing the drag event. Otherwise, returns `None`.
    pub fn is_dragged(&self, ui: &mut Ui) -> Option<Drag> {
        self.get_uinode(ui).is_dragged()
    }

    /// If the node is currently hovered by the cursor, returns hover information including position.
    pub fn is_hovered(&self, ui: &mut Ui) -> Option<Hover> {
        self.get_uinode(ui).is_hovered()
    }

   /// If the node corresponding to `key` was being held with the left mouse button in the last frame, returns the duration for which it was held.
   pub fn is_held(&self, ui: &mut Ui) -> Option<Duration> {
        self.get_uinode(ui).is_held()
    }

    /// If the node was scrolled in the last frame, returns a struct containing detailed information of the scroll event. Otherwise, returns `None`.
    /// 
    /// If the node was scrolled multiple times in the last frame, the result holds the information about the last scroll only.
    pub fn scrolled_at(&self, ui: &mut Ui) -> Option<ScrollEvent> {
        self.get_uinode(ui).scrolled_at()
    }

    /// Returns the total scroll delta for the node in the last frame, or None if no scroll events occurred.
    pub fn is_scrolled(&self, ui: &mut Ui) -> Option<glam::Vec2> {
        self.get_uinode(ui).is_scrolled()
    }
}