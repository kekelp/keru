use std::time::Duration;

use winit::event::MouseButton;

use crate::*;
use crate::node::*;
use crate::Axis::*;

/// A helper struct that borrows the Ui but "selects" a specific node.
/// Not really revolutionarily different than using self.nodes[i].
pub(crate) struct UiNode<'a> {
    pub(crate) i: NodeI,
    pub(crate) ui: &'a Ui,
}

impl UiNode<'_> {
    pub(crate) fn node(&self) -> &Node {
        return &self.ui.nodes[self.i];
    }

    pub(crate) fn inner_size(&self) -> Xy<u32> {
        let padding = self.node().params.layout.padding;
        
        let size = self.node().size;
        let size = self.ui.f32_size_to_pixels2(size);

        return size - padding;
    }

    pub(crate) fn center(&self) -> Xy<f32> {
        let rect = self.node().real_rect;
        
        let center = Xy::new(
            (rect[X][1] + rect[X][0]) / 2.0,
            (rect[Y][1] + rect[Y][0]) / 2.0,
        );

        let center = center * self.ui.sys.unifs.size;

        return center;
    }

    pub(crate) fn bottom_left(&self) -> Xy<f32> {
        let rect = self.node().real_rect;
        
        let center = Xy::new(
            rect[X][0],
            rect[Y][1],
        );

        let center = center * self.ui.sys.unifs.size;
        
        return center;
    }

    pub(crate) fn rect(&self) -> XyRect {
        return self.node().real_rect * self.ui.sys.unifs.size;
    }

    pub(crate) fn render_rect(&self) -> RenderInfo {
        let size = self.ui.sys.unifs.size;
        return RenderInfo {
            rect: self.node().real_rect.to_graphics_space_rounded(size),
            z: self.node().z + Z_STEP / 2.0,
        };
    }
}

impl Ui {
    /// todo make this public?
    pub(crate) fn get_uinode(&self, key: NodeKey) -> Option<UiNode<'_>> {
        let i = self.nodes.node_hashmap.get(&key.id_with_subtree())?.slab_i;
        return Some(UiNode { i, ui: self });
    }

    pub fn render_rect(&self, key: NodeKey) -> Option<RenderInfo> {
        Some(self.get_uinode(key)?.render_rect())
    }
    pub fn z_value(&self, key: NodeKey) -> Option<f32> {
        Some(self.get_uinode(key)?.render_rect().z)
    }
    pub fn rect(&self, key: NodeKey) -> Option<XyRect> {
        Some(self.get_uinode(key)?.rect())
    }
    pub fn center(&self, key: NodeKey) -> Option<Xy<f32>> {
        Some(self.get_uinode(key)?.center())
    }
    pub fn inner_size(&self, key: NodeKey) -> Option<Xy<u32>> {
        Some(self.get_uinode(key)?.inner_size())
    }
    pub fn bottom_left(&self, key: NodeKey) -> Option<Xy<f32>> {
        Some(self.get_uinode(key)?.bottom_left())
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
            TextI::TextBox(handle) => *self.sys.renderer.text.get_text_box_mut(&handle).text_mut() = std::borrow::Cow::Owned(text.to_string()),
            TextI::TextEdit(handle) => *self.sys.renderer.text.get_text_edit_mut(&handle).raw_text_mut() = text.to_string(),
        };
        Some(())
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
        self.get_uinode(ui).rect()
    }
    pub fn center(&self, ui: &mut Ui) -> Xy<f32> {
        self.get_uinode(ui).center()
    }
    pub fn bottom_left(&self, ui: &mut Ui) -> Xy<f32> {
        self.get_uinode(ui).bottom_left()
    }
    pub fn get_text<'u>(&self, ui: &'u mut Ui) -> Option<&'u str> {
        let text_i = ui.nodes[self.i].text_i.as_ref()?;
        match text_i {
            TextI::TextBox(handle) => Some(ui.sys.renderer.text.get_text_box(&handle).text()),
            TextI::TextEdit(handle) => Some(ui.sys.renderer.text.get_text_edit(&handle).raw_text()),
        }
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
            // todo: 
            eprintln!(
                "Keru: Debug mode check: \"{}\" was called for node {}, but the node doesn't have the {:?} sense. In release mode, this event will be silently ignored! You can add the sense to the node's NodeParams with the \"{}\" function.",
                fn_name,
                node.debug_name(),
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
        if ! self.check_node_sense(Sense::CLICK, "is_clicked()", "NodeParams::sense_click()") {
            return false;
        }

        let clicked = self.ui.sys.mouse_input.clicked(Some(MouseButton::Left), Some(self.node().id));
        return clicked;
    }

    pub fn is_focused(&self) -> bool {
        return self.ui.sys.focused == Some(self.node().id);
    }

    /// If the node corresponding to `key` was clicked in the last frame, returns a struct containing detailed information of the click. Otherwise, returns `None`.
    /// 
    /// If the node was clicked multiple times in the last frame, the result holds the information about the last click only.
    pub fn clicked_at(&self) -> Option<Click> {
         #[cfg(debug_assertions)]
        if !self.check_node_sense(Sense::CLICK, "clicked_at()", "NodeParams::sense_click()") {
            return None;
        }

        let mouse_record = self.ui.sys.mouse_input.clicked_at(Some(MouseButton::Left), Some(self.node().id))?;
        let node_rect = self.node().real_rect;
        
        let relative_position = glam::DVec2::new(
            ((mouse_record.position.x / self.ui.sys.unifs.size.x as f64) - (node_rect.x[0]) as f64) / node_rect.size().x as f64,
            ((mouse_record.position.y / self.ui.sys.unifs.size.y as f64) - (node_rect.y[0]) as f64) / node_rect.size().y as f64,
        );
        
        return Some(Click {
            relative_position,
            absolute_position: mouse_record.position,
            timestamp: mouse_record.timestamp,
        });
    }

    /// Returns `true` if a left button mouse click was just released on the node corresponding to `key`.
    pub fn is_click_released(&self) -> bool {
         #[cfg(debug_assertions)]
        if ! self.check_node_sense(Sense::CLICK, "is_click_released()", "NodeParams::sense_click()") {
            return false;
        }

        return self.ui.sys.mouse_input.click_released(Some(MouseButton::Left), Some(self.node().id));
    }

    /// If the node was dragged with a specific mouse button, returns a struct describing the drag event. Otherwise, returns `None`.
    pub fn is_mouse_button_dragged(&self, button: MouseButton) -> Option<Drag> {
         #[cfg(debug_assertions)]
        if !self.check_node_sense(Sense::DRAG, "is_mouse_button_dragged()", "NodeParams::sense_drag()") {
            return None;
        }

        let mouse_record = self.ui.sys.mouse_input.dragged_at(Some(button), Some(self.node().id))?;
        let node_rect = self.node().real_rect;
        let relative_position = glam::DVec2::new(
            ((mouse_record.currently_at.position.x / self.ui.sys.unifs.size.x as f64) - (node_rect.x[0]) as f64) / node_rect.size().x as f64,
            ((mouse_record.currently_at.position.y / self.ui.sys.unifs.size.y as f64) - (node_rect.y[0]) as f64) / node_rect.size().y as f64,
        );
        let relative_delta = glam::DVec2::new(
            mouse_record.drag_distance().x / (node_rect.size().x as f64 * self.ui.sys.unifs.size.x as f64),
            mouse_record.drag_distance().y / (node_rect.size().y as f64 * self.ui.sys.unifs.size.y as f64),
        );

        return Some(Drag {
            relative_position,
            absolute_position: mouse_record.currently_at.position,
            relative_delta,
            absolute_delta: mouse_record.drag_distance(),
            pressed_timestamp: mouse_record.originally_pressed.timestamp,
        });
    }

    /// If the node corresponding to `key` was dragged, returns a struct describing the drag event. Otherwise, returns `None`.
    pub fn is_dragged(&self) -> Option<Drag> {
        self.is_mouse_button_dragged(MouseButton::Left)
    }

   /// If the node corresponding to `key` was being held with the left mouse button in the last frame, returns the duration for which it was held.
    pub fn is_held(&self) -> Option<Duration> {
        #[cfg(debug_assertions)]
        if ! self.check_node_sense(Sense::HOLD, "is_held()", "NodeParams::sense_hold()") {
            return None;
        }

        return self.ui.sys.mouse_input.held(Some(MouseButton::Left), Some(self.node().id));
    }

    /// Returns `true` if a node is currently hovered by the cursor.
    pub fn is_hovered(&self) -> bool {
         #[cfg(debug_assertions)]
        if ! self.check_node_sense(Sense::HOVER, "is_hovered", "NodeParams::sense_hover()") {
            return false;
        }

        return self.ui.sys.hovered.last() == Some(&self.node().id);
    }

    /// If the node was scrolled in the last frame, returns a struct containing detailed information of the scroll event. Otherwise, returns `None`.
    /// 
    /// If the node was scrolled multiple times in the last frame, the result holds the information about the last scroll only.
    pub fn scrolled_at(&self) -> Option<ScrollEvent> {
        let scroll_event = self.ui.sys.mouse_input.last_scroll_event(Some(self.node().id))?;
        let node_rect = self.node().real_rect;
        
        let relative_position = glam::DVec2::new(
            ((scroll_event.position.x / self.ui.sys.unifs.size.x as f64) - (node_rect.x[0]) as f64) / node_rect.size().x as f64,
            ((scroll_event.position.y / self.ui.sys.unifs.size.y as f64) - (node_rect.y[0]) as f64) / node_rect.size().y as f64,
        );
        
        return Some(ScrollEvent {
            relative_position,
            absolute_position: scroll_event.position,
            delta: scroll_event.delta,
            timestamp: scroll_event.timestamp,
        });
    }

    /// Returns the total scroll delta for this node in the last frame, or None if no scroll events occurred.
    pub fn is_scrolled(&self) -> Option<glam::DVec2> {
        // todo: is there no sense_scroll?
        return self.ui.sys.mouse_input.scrolled(Some(self.node().id));
    }
}



impl Ui {
    /// Returns `true` if the node corresponding to `key` was just clicked with the left mouse button.
    /// 
    /// This is "act on press". For "act on release", see [`Self::is_click_released()`].
    pub fn is_clicked(&self, key: NodeKey) -> bool {
        let Some(uinode) = self.get_uinode(key) else {
            return false;
        };
        uinode.is_clicked()
    }

    /// Returns `true` if the text in the text edit node corresponding to `key` was just changed through user input.
    /// 
    /// This tracks changes from both user events (typing, pasting, etc.) and programmatic changes via [`Self::set_text_edit_text()`].
    /// Only works for text edit nodes - returns `false` for regular text nodes.
    pub fn edit_text_changed(&self, key: NodeKey) -> bool {        
        let Some(uinode) = self.get_uinode(key) else {
            return false;
        };
        let id = uinode.node().id;
        self.sys.text_edit_changed_last_frame == Some(id)
    }

    /// Set the text content of a text edit node.
    /// 
    /// Does nothing for regular text nodes. To change the text, just re-add them with the desired text.
    // todo: think if this really makes sense.
    pub fn set_text_edit_text(&mut self, key: NodeKey, new_text: String) {
        let Some(i) = self.nodes.node_hashmap.get(&key.id_with_subtree()) else {
            return;
        };
        let i = i.slab_i;   

        if let Some(TextI::TextEdit(handle)) = &self.nodes[i].text_i {

            self.sys.renderer.text.get_text_edit_mut(handle).set_text(&new_text);
            // Mark this node as having changed for next frame
            self.sys.text_edit_changed_this_frame = Some(self.nodes[i].id);
        
        } 
    }

    /// Set placeholder text for a text edit node that will be shown when the text edit is empty.
    /// 
    /// Does nothing for regular text nodes.
    pub fn set_text_edit_placeholder(&mut self, key: NodeKey, placeholder: String) {
        let Some(i) = self.nodes.node_hashmap.get(&key.id_with_subtree()) else {
            return;
        };
        let i = i.slab_i;   

        if let Some(TextI::TextEdit(handle)) = &self.nodes[i].text_i {
            self.sys.renderer.text.get_text_edit_mut(handle).set_placeholder(placeholder);
        } 
    }

    pub fn is_focused(&self, key: NodeKey) -> bool {
        let Some(uinode) = self.get_uinode(key) else {
            return false;
        };
        uinode.is_focused()
    }

    /// Returns `true` if a left button mouse click was just released on the node corresponding to `key`.
    pub fn is_click_released(&self, key: NodeKey) -> bool {
        let Some(uinode) = self.get_uinode(key) else {
            return false;
        };
        uinode.is_click_released()
    }

    /// If the node corresponding to `key` was dragged with a specific mouse button, returns a struct describing the drag event. Otherwise, returns `None`.
    pub fn is_mouse_button_dragged(&self, key: NodeKey, button: MouseButton) -> Option<Drag> {
        let Some(uinode) = self.get_uinode(key) else {
            return None;
        };
        uinode.is_mouse_button_dragged(button)
    }

    /// If the node corresponding to `key` was dragged, returns a struct describing the drag event. Otherwise, returns `None`.
    pub fn is_dragged(&self, key: NodeKey) -> Option<Drag> {
        let Some(uinode) = self.get_uinode(key) else {
            return None;
        };
        uinode.is_dragged()
    }

    /// If the node corresponding to `key` was clicked in the last frame, returns a struct containing detailed information of the click. Otherwise, returns `None`.
    /// 
    /// If the node was clicked multiple times in the last frame, the result holds the information about the last click only.
    pub fn clicked_at(&self, key: NodeKey) -> Option<Click> {
        let Some(uinode) = self.get_uinode(key) else {
            return None;
        };
        uinode.clicked_at()
    }

    /// Returns `true` if a node is currently hovered by the cursor.
    pub fn is_hovered(&self, key: NodeKey) -> bool {
        let Some(uinode) = self.get_uinode(key) else {
            return false;
        };
        uinode.is_hovered()
    }

   /// If the node corresponding to `key` was being held with the left mouse button in the last frame, returns the duration for which it was held.
   pub fn is_held(&self, key: NodeKey) -> Option<Duration> {
        let Some(uinode) = self.get_uinode(key) else {
            return None;
        };
        uinode.is_held()
    }

    /// If the node corresponding to `key` was scrolled in the last frame, returns a struct containing detailed information of the scroll event. Otherwise, returns `None`.
    /// 
    /// If the node was scrolled multiple times in the last frame, the result holds the information about the last scroll only.
    pub fn scrolled_at(&self, key: NodeKey) -> Option<ScrollEvent> {
        let Some(uinode) = self.get_uinode(key) else {
            return None;
        };
        uinode.scrolled_at()
    }

    /// Returns the total scroll delta for the node corresponding to `key` in the last frame, or None if no scroll events occurred.
    pub fn is_scrolled(&self, key: NodeKey) -> Option<glam::DVec2> {
        let Some(uinode) = self.get_uinode(key) else {
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
    pub fn is_mouse_button_dragged(&self, ui: &mut Ui, button: MouseButton) -> Option<Drag> {
        self.get_uinode(ui).is_mouse_button_dragged(button)
    }

    /// If the node was dragged, returns a struct describing the drag event. Otherwise, returns `None`.
    pub fn is_dragged(&self, ui: &mut Ui) -> Option<Drag> {
        self.get_uinode(ui).is_dragged()
    }

    /// Returns `true` if the node is currently hovered by the cursor.
    pub fn is_hovered(&self, ui: &mut Ui) -> bool {
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
    pub fn is_scrolled(&self, ui: &mut Ui) -> Option<glam::DVec2> {
        self.get_uinode(ui).is_scrolled()
    }
}