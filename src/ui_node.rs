use glam::Vec2;
use winit::event::MouseButton;

use crate::*;
use crate::inner_node::*;
use crate::mouse_events::{DragEvent, DragReleaseEvent};
use crate::Axis::*;

pub struct UiNode<'a> {
    pub(crate) i: NodeI,
    pub(crate) sys: UiRef<'a>,
}
pub(crate) enum UiRef<'a> {
    Mut(&'a mut System),
    Shared(&'a System),
}

impl<'a> UiNode<'a> {
    pub(crate) fn sys_mut(&mut self) -> &mut System {
        match &mut self.sys {
            // We only call ui_mut() from functions that take &mut self.
            // [`Ui::get_node_mut()`] ensures that if the caller has access to a `&mut UiNode`, it will have been constructed with `UiRef::Mut`.
            UiRef::Shared(_) => unreachable!(),
            UiRef::Mut(sys) => return sys,
        }
    }

    pub(crate) fn sys(&self) -> &System {
        match &self.sys {
            UiRef::Mut(sys) => sys,
            UiRef::Shared(sys) => return sys,
        }
    }
}


pub struct UiNodeChildrenIter<'a> {
    sys: &'a System,
    current: Option<NodeI>,
    remaining: usize,
}

impl<'a> Iterator for UiNodeChildrenIter<'a> {
    type Item = UiNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(child_i) = self.current {
            self.current = self.sys.nodes[child_i].next_sibling;
            if !self.sys.nodes[child_i].exiting {
                self.remaining -= 1;
                return Some(UiNode { sys: UiRef::Shared(&self.sys), i: child_i });
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
    pub fn children(&'a self) -> impl Iterator<Item = UiNode<'a>> {
        let sys = self.sys();
        UiNodeChildrenIter {
            sys: sys,
            current: sys.nodes[self.i].first_child,
            remaining: sys.nodes[self.i].n_children as usize,
        }
    }

    pub(crate) fn node(&self) -> &InnerNode {
        return &self.sys().nodes[self.i];
    }

    /// Get the number of children added to the node so far.
    pub fn children_count(&self) -> usize {
        self.node().n_children as usize
    }

    /// Returns `true` if the node is currently hidden (excluded from the tree but retained in memory).
    pub fn is_hidden(&self) -> bool {
        self.node().currently_hidden
    }

    /// Returns `true` if the node is currently playing its exit animation before being removed.
    pub fn is_exiting(&self) -> bool {
        self.node().exiting
    }

    /// Returns a temporary key that can be used to refer to this node.
    pub fn temp_key(&self) -> NodeKey {
        NodeKey::new_temp(self.node().id, "temp_node_key")
    }

    /// Returns the node's inner size (without padding), in screen pixels.
    /// 
    /// Since the size and position of nodes is only determined after the layout pass at the end of the frame, 
    /// this function will return the value from last frame.  
    pub fn inner_size(&self) -> Xy<f32> {
        let padding = self.node().params.layout.padding;

        let size = self.node().size;
        let size = self.sys().f32_size_to_pixels2(size);

        return size - padding;
    }

    /// Returns the center of the node's rectangle, in screen pixels.
    /// 
    /// Since the size and position of nodes is only determined after the layout pass at the end of the frame, 
    /// this function will return the value from last frame.
    pub fn center(&self) -> Xy<f32> {
        let rect = self.node().real_rect;
        
        let center = Xy::new(
            (rect[X][1] + rect[X][0]) / 2.0,
            (rect[Y][1] + rect[Y][0]) / 2.0,
        );

        let center = center * self.sys().size;

        return center;
    }

    /// Returns the bottom left point of the node's rectangle, in screen pixels.
    /// 
    /// Since the size and position of nodes is only determined after the layout pass at the end of the frame, 
    /// this function will return the value from last frame.
    pub fn bottom_left(&self) -> Xy<f32> {
        let rect = self.node().real_rect;
        
        let center = Xy::new(
            rect[X][0],
            rect[Y][1],
        );

        let center = center * self.sys().size;
        
        return center;
    }

    /// Returns the node's rectangle in screen pixels.
    /// 
    /// Since the size and position of nodes is only determined after the layout pass at the end of the frame, 
    /// this function will return the value from last frame.
    pub fn rect(&self) -> XyRect {
        return self.node().real_rect * self.sys().size;
    }

    /// Returns the node's rectangle in normalized device coordinates (NDC).
    /// 
    /// Since the size and position of nodes is only determined after the layout pass at the end of the frame, 
    /// this function will return the value from last frame.
    pub fn render_rect(&self) -> RenderInfo {
        let size = self.sys().size;
        let scale = self.node().accumulated_transform.scale;
        return RenderInfo {
            rect: self.node().real_rect.to_graphics_space_rounded(size, scale),
            z: self.node().z + Z_STEP / 2.0,
        };
    }

    /// Returns the text content if it was changed by user input this frame, otherwise `None`.
    ///
    /// Only works for text edit nodes. Returns `None` for regular text nodes.
    pub fn text_edit_changed(&'a self) -> Option<&'a str> {
        if let Some(TextI::TextEdit(handle)) = &self.node().text_i {
            let text_edit = self.sys().renderer.text.get_text_edit(&handle);
            if text_edit.text_changed() {
                return Some(text_edit.raw_text());
            }
        }
        None
    }

    /// Get the text in a node's text edit box or text box.
    /// 
    /// Does nothing and returns `None` if the node doesn't have a text edit box or a text box. 
    pub fn get_text(&self) -> Option<&str> {
        match self.node().text_i.as_ref()? {
            TextI::TextBox(handle) => Some(self.sys().renderer.text.get_text_box(&handle).text()),
            TextI::TextEdit(handle) => Some(self.sys().renderer.text.get_text_edit(&handle).raw_text()),
        }
    }

    /// Set the text in a node's text edit box or text box.
    /// 
    /// Does nothing and returns `None` if the node doesn't have a text edit box or a text box. 
    /// 
    /// This method is meant for programmatically updating text edit content as a response to events. For text that depends on state, it's probably better to stick to the declarative API and set its text conditionally with [`Node::text()`] when rebuilding.
    pub fn set_text(&mut self, text: &str) -> Option<()> {
        let i = self.i;
        let sys = self.sys_mut();
        match sys.nodes[i].text_i.as_ref()? {
            TextI::TextBox(handle) => sys.renderer.text.get_text_box_mut(&handle).set_text_hashed(text),
            TextI::TextEdit(handle) => sys.renderer.text.get_text_edit_mut(&handle).set_text_hashed(text),
        };
        return Some(())
    }

    /// Set placeholder text for a text edit node that will be shown when the text edit is empty.
    ///
    /// Does nothing for non-editable text nodes or for nodes without text.
    // todo: maybe just remove this function?
    pub fn set_text_edit_placeholder(&mut self, placeholder: &str) {
        let i = self.i;
        let sys = self.sys_mut();
        if let Some(TextI::TextEdit(handle)) = &sys.nodes[i].text_i {
            sys.renderer.text.get_text_edit_mut(handle).set_placeholder(placeholder);
        }
    }
}

impl Ui {
    /// Get the [`UiNode`] corresponding to the `key`, if such a node is currently part of the visible UI tree.
    /// 
    /// This function will return the node if it exists and it is both visible and interactable. So it will return `None` if the node exists but it is hidden or if it is doing an exiting animation right before disappearing. Use also [`Ui::get_node_unfiltered`] for a version that also returns hidden and exiting nodes.
    ///
    /// If the same key was used to add multiple nodes in the same frame, the key will always return the first one. You can use [`NodeKey::sibling()`] to create different "versions" of the same key dynamically and still be able to point to them.
    pub fn get_node_mut(&mut self, key: NodeKey) -> Option<&mut UiNode<'_>> {
        let node = self.get_node_unfiltered_mut(key)?;
        if node.node().currently_hidden || node.node().exiting {
            return None;
        } else {
            return Some(node);
        }
    }

    pub fn get_node(&self, key: NodeKey) -> Option<&UiNode<'_>> {
        let node = self.get_node_unfiltered(key)?;
        if node.node().currently_hidden || node.node().exiting {
            return None;
        } else {
            return Some(node);
        }
    }

    /// Like [`Ui::get_node_mut()`], but also returns nodes that are currently hidden or exiting.
    ///
    /// You can check [`UiNode::is_hidden`] and [`UiNode::is_exiting`] on the result to filter as needed.
    pub fn get_node_unfiltered_mut(&mut self, key: NodeKey) -> Option<&mut UiNode<'_>> {
        let i = self.sys.nodes.get_with_subtree(key)?;
        // If you are wondering why are we creating wrapper structs inside an arena in the first place, it's so that the `UiNode` has better ergonomics.
        // That is, so that the interface looks like this: 
        // 
        // ```
        // let node: &UiNode = ui.get_node(key);
        // let node_mut: &mut UiNode = ui.get_node_mut(key);
        // ```
        // 
        // Rather than this: 
        // 
        // ```
        // let node: UiNode = ui.get_node(key);
        // let mut node_mut: UiNodeMut = ui.get_node_mut(key);
        // ```
        // 
        // Where UiNode and UiNodeMut are crappy separate wrapper structs, the caller has to make the node_mut binding itself mutable, etc.

        let wrapper = UiNode { i, sys: UiRef::Mut(&mut self.sys)  };
        let wrapper = self.arena_for_wrapper_structs.alloc(wrapper);

        return Some(wrapper);
    }

    /// Like [`Ui::get_node()`], but also returns nodes that are currently hidden or exiting.
    ///
    /// You can check [`UiNode::is_hidden`] and [`UiNode::is_exiting`] on the result to filter as needed.
    pub fn get_node_unfiltered(&self, key: NodeKey) -> Option<&UiNode<'_>> {
        let i = self.sys.nodes.get_with_subtree(key)?;
        let wrapper = UiNode { i, sys: UiRef::Shared(&self.sys)  };
        return Some(self.arena_for_wrapper_structs.alloc(wrapper));
    }

}

/// The data needed for rendering a node with custom code.
/// 
/// Obtained with [`UiNode::render_rect()`].
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
//         return &self.ui.sys.nodes[self.i];
//     }
// }



impl System {
    #[cfg(debug_assertions)]
    pub(crate) fn check_node_sense(&self, i: NodeI, sense: Sense, fn_name: &'static str, sense_add_fn_name: &'static str) -> bool {
        let node = &self.nodes[i];
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

    pub(crate) fn drag_from_event_with_rect(&self, event: &DragEvent, node_rect: XyRect) -> Option<Drag> {
        let relative_position = glam::Vec2::new(
            ((event.current_pos.x / self.size.x) - node_rect.x[0]) / node_rect.size().x,
            ((event.current_pos.y / self.size.y) - node_rect.y[0]) / node_rect.size().y,
        );
        let relative_delta = glam::Vec2::new(
            event.frame_delta.x / (node_rect.size().x * self.size.x),
            event.frame_delta.y / (node_rect.size().y * self.size.y),
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

    pub(crate) fn drag_from_release_event_with_rect(&self, event: &DragReleaseEvent, node_rect: XyRect) -> Option<Drag> {
        let relative_position = glam::Vec2::new(
            ((event.end_pos.x / self.size.x) - node_rect.x[0]) / node_rect.size().x,
            ((event.end_pos.y / self.size.y) - node_rect.y[0]) / node_rect.size().y,
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
}

impl Ui {

    #[cfg(debug_assertions)]
    fn check_dest_node_sense(&self, dest_key: NodeKey, sense: Sense, fn_name: &'static str, sense_add_fn_name: &'static str) -> bool {
        let Some(i) = self.sys.nodes.get_with_subtree(dest_key) else {
            return true; // Node doesn't exist, let the function return false naturally
        };
        let dest_node = &self.sys.nodes[i];
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

    fn drag_from_event_with_rect(&self, event: &DragEvent, node_rect: XyRect) -> Option<Drag> {
        self.sys.drag_from_event_with_rect(event, node_rect)
    }

    fn drag_from_release_event_with_rect(&self, event: &DragReleaseEvent, node_rect: XyRect) -> Option<Drag> {
        self.sys.drag_from_release_event_with_rect(event, node_rect)
    }

    /// If a left button mouse drag on the node corresponding to the `src` key was just released onto the node corresponding to the `dest` key, returns the drag info.
    pub fn is_drag_released_onto(&self, src_key: NodeKey, dest_key: NodeKey) -> Option<Drag> {
        let src_i = self.sys.nodes.get_with_subtree(src_key)?;
        let src_node = &self.sys.nodes[src_i];
        if src_node.currently_hidden || src_node.exiting {
            return None;
        }

        #[cfg(debug_assertions)]
        if !self.sys.check_node_sense(src_i, Sense::DRAG, "is_drag_released_onto()", "Node::sense_drag()") {
            return None;
        }
        #[cfg(debug_assertions)]
        if !self.check_dest_node_sense(dest_key, Sense::DRAG_DROP_TARGET, "is_drag_released_onto()", "Node::sense_drag_drop_target()") {
            return None;
        }

        let src_id = src_node.id;
        let event = self.sys.check_drag_released_onto(src_id, dest_key.id_with_subtree(), MouseButton::Left)?;
        let dest_i = self.sys.nodes.get_with_subtree(dest_key)?;
        let dest_rect = self.sys.nodes[dest_i].real_rect;
        self.drag_from_release_event_with_rect(event, dest_rect)
    }

    /// If a left button mouse drag on the node corresponding to the `src` key is currently hovering over the node corresponding to the `dest` key, returns the drag info.
    pub fn is_drag_hovered_onto(&self, src_key: NodeKey, dest_key: NodeKey) -> Option<Drag> {
        let src_i = self.sys.nodes.get_with_subtree(src_key)?;
        let src_node = &self.sys.nodes[src_i];
        if src_node.currently_hidden || src_node.exiting {
            return None;
        }

        #[cfg(debug_assertions)]
        if !self.sys.check_node_sense(src_i, Sense::DRAG, "is_drag_hovered_onto()", "Node::sense_drag()") {
            return None;
        }
        #[cfg(debug_assertions)]
        if !self.check_dest_node_sense(dest_key, Sense::DRAG_DROP_TARGET, "is_drag_hovered_onto()", "Node::sense_drag_drop_target()") {
            return None;
        }

        let src_id = src_node.id;
        let event = self.sys.check_drag_hovered_onto(src_id, dest_key.id_with_subtree(), MouseButton::Left)?;
        let dest_i = self.sys.nodes.get_with_subtree(dest_key)?;
        let dest_rect = self.sys.nodes[dest_i].real_rect;
        self.drag_from_event_with_rect(event, dest_rect)
    }

    /// If any node is currently being dragged over the node corresponding to `dest`, returns the drag info.
    /// The `relative_position` in the returned `Drag` is relative to the destination node.
    ///
    /// This is useful for drop targets that need to react to any dragged item, without knowing
    /// which specific item is being dragged.
    pub fn is_any_drag_hovered_onto(&self, dest_key: NodeKey) -> Option<Drag> {
        #[cfg(debug_assertions)]
        {
            let dest_i = self.sys.nodes.get_with_subtree(dest_key)?;
            if !self.sys.nodes[dest_i].params.interact.senses.contains(Sense::DRAG_DROP_TARGET) {
                log::warn!(
                    "is_any_drag_hovered_onto() was called on node {:?}, but it doesn't have the DRAG_DROP_TARGET sense. Add Node::sense_drag_drop_target() to the node.",
                    dest_key.debug_name()
                );
                return None;
            }
        }

        let event = self.sys.check_any_drag_hovered_onto(dest_key.id_with_subtree(), MouseButton::Left)?;
        let dest_rect = self.get_node(dest_key)?.node().real_rect;

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
    pub fn is_any_drag_released_onto(&self, dest_key: NodeKey) -> Option<Drag> {
        #[cfg(debug_assertions)]
        {
            let dest_i = self.sys.nodes.get_with_subtree(dest_key)?;
            if !self.sys.nodes[dest_i].params.interact.senses.contains(Sense::DRAG_DROP_TARGET) {
                log::warn!(
                    "is_any_drag_released_onto() was called on node {:?}, but it doesn't have the DRAG_DROP_TARGET sense. Add Node::sense_drag_drop_target() to the node.",
                    dest_key.debug_name()
                );
                return None;
            }
        }

        let event = self.sys.check_any_drag_released_onto(dest_key.id_with_subtree(), MouseButton::Left)?;
        let dest_node = self.get_node(dest_key)?;
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

}
