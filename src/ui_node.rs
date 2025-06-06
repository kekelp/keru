use std::time::Duration;

use winit::event::MouseButton;

use crate::*;
use crate::node::*;
use crate::Axis::*;

/// A helper struct that borrows the Ui but "selects" a specific node.
/// Not really revolutionarily different than using self.nodes[i].
pub(crate) struct UiNode<'a> {
    pub i: NodeI,
    pub ui: &'a Ui,
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
        let rect = self.node().rect;
        
        let center = Xy::new(
            (rect[X][1] + rect[X][0]) / 2.0,
            (rect[Y][1] + rect[Y][0]) / 2.0,
        );

        let center = center * self.ui.sys.unifs.size;

        return center;
    }

    pub(crate) fn bottom_left(&self) -> Xy<f32> {
        let rect = self.node().rect;
        
        let center = Xy::new(
            rect[X][0],
            rect[Y][1],
        );

        let center = center * self.ui.sys.unifs.size;
        
        return center;
    }

    pub(crate) fn rect(&self) -> XyRect {
        return self.node().rect * self.ui.sys.unifs.size;
    }

    pub(crate) fn render_rect(&self) -> RenderInfo {
        let size = self.ui.sys.unifs.size;
        return RenderInfo {
            rect: self.node().rect.to_graphics_space_rounded(size),
            z: self.node().z + Z_STEP / 2.0,
        };
    }

    // todo: nasty allocation
    pub fn get_text(&self) -> Option<&str> {
        let text_i = self.node().text_i?;
        return Some(self.ui.sys.text_boxes[text_i.0].raw_text());
    }
}

impl Ui {
    pub(crate) fn get_uinode(&self, key: NodeKey) -> Option<UiNode> {
        let i = self.nodes.node_hashmap.get(&key.id_with_subtree())?.slab_i;
        return Some(UiNode { i, ui: self });
    }

    pub fn render_rect(&self, key: NodeKey) -> Option<RenderInfo> {
        Some(self.get_uinode(key)?.render_rect())
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
    pub fn get_text(&self, key: NodeKey) -> Option<&str> {
        let i = self.nodes.node_hashmap.get(&key.id_with_subtree())?.slab_i;
        let text_i = self.nodes[i].text_i?;
        return Some(self.sys.text_boxes[text_i.0].raw_text());
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
        let text_i = ui.nodes[self.i].text_i?;
        return Some(ui.sys.text_boxes[text_i.0].raw_text());
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
    fn check_node_sense(&self, sense: Sense, fn_name: &'static str) -> bool {
        let node = self.node();
        if !node.params.interact.senses.contains(sense) {
            log::error!(
                "Debug mode check: {} was called on node {}, but the node doesn't have the {:?} sense.",
                fn_name,
                node.debug_name(),
                sense
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
        {
            if ! self.check_node_sense(Sense::CLICK, "is_clicked") {
                return false;
            }
        }
        let id = self.node().id;
        let clicked = self.ui.sys.mouse_input.clicked(Some(MouseButton::Left), Some(id));
        return clicked;
    }

    /// If the node corresponding to `key` was clicked in the last frame, returns a struct containing detailed information of the click. Otherwise, returns `None`.
    /// 
    /// If the node was clicked multiple times in the last frame, the result holds the information about the last click only.
    pub fn clicked_at(&self) -> Option<Click> {
        let id = self.node().id;
        #[cfg(debug_assertions)]
        {
            if !self.check_node_sense(Sense::CLICK, "clicked_at") {
                return None;
            }
        }
        let mouse_record = self.ui.sys.mouse_input.clicked_at(Some(MouseButton::Left), Some(id))?;
        let node_rect = self.node().rect;
        
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
        let id = self.node().id;
        #[cfg(debug_assertions)]
        {
            if ! self.check_node_sense(Sense::CLICK, "is_click_released") {
                return false;
            }
        }
        return self.ui.sys.mouse_input.click_released(Some(MouseButton::Left), Some(id));
    }

    /// If the node corresponding to `key` was dragged, returns a struct describing the drag event. Otherwise, returns `None`.
    pub fn is_dragged(&self) -> Option<Drag> {
        let id = self.node().id;
        #[cfg(debug_assertions)]
        {
            if !self.check_node_sense(Sense::DRAG, "dragged_at") {
                return None;
            }
        }
        let mouse_record = self.ui.sys.mouse_input.dragged_at(Some(MouseButton::Left), Some(id))?;
        let node_rect = self.node().rect;
        let relative_position = glam::DVec2::new(
            ((mouse_record.currently_at.position.x / self.ui.sys.unifs.size.x as f64) - (node_rect.x[0]) as f64) / node_rect.size().x as f64,
            ((mouse_record.currently_at.position.y / self.ui.sys.unifs.size.y as f64) - (node_rect.y[0]) as f64) / node_rect.size().y as f64,
        );
        let relative_delta = glam::DVec2::new(
            mouse_record.drag_distance().x / (node_rect.size().x as f64 * self.ui.sys.unifs.size.x as f64),
            mouse_record.drag_distance().y / (node_rect.size().y as f64 * self.ui.sys.unifs.size.y as f64),
        );

        if mouse_record.drag_distance() == glam::dvec2(0.0, 0.0) {
            return None;
        }

        return Some(Drag {
            relative_position,
            absolute_position: mouse_record.currently_at.position,
            relative_delta,
            absolute_delta: mouse_record.drag_distance(),
            pressed_timestamp: mouse_record.originally_pressed.timestamp,
        });
    }

   /// If the node corresponding to `key` was being held with the left mouse button in the last frame, returns the duration for which it was held.
    pub fn is_held(&self) -> Option<Duration> {
        let id = self.node().id;
        #[cfg(debug_assertions)]
        {
            if ! self.check_node_sense(Sense::HOLD, "is_held") {
                return None;
            }
        }
        return self.ui.sys.mouse_input.held(Some(MouseButton::Left), Some(id));
    }

    /// Returns `true` if a node is currently hovered by the cursor.
    pub fn is_hovered(&self) -> bool {
        let id = self.node().id;
        #[cfg(debug_assertions)]
        {
            if ! self.check_node_sense(Sense::HOVER, "is_hovered") {
                return false;
            }
        }
        return self.ui.sys.hovered.last() == Some(&id);
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

    /// Returns `true` if a left button mouse click was just released on the node corresponding to `key`.
    pub fn is_click_released(&self, key: NodeKey) -> bool {
        let Some(uinode) = self.get_uinode(key) else {
            return false;
        };
        uinode.is_click_released()
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
}