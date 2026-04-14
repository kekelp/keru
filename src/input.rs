use crate::*;

use std::time::Duration;
use glam::Vec2;
use winit::event::MouseButton;

impl<'a> UiNode<'a> {
    /// Returns `true` if this node was just clicked with the left mouse button.
    ///
    /// This is "act on press". For "act on release", see [`UiNode::is_click_released()`].
    pub fn is_clicked(&self) -> bool {
        self.sys().check_clicked(self.node().id, MouseButton::Left)
    }

    /// Returns `true` if this node was just clicked with the right mouse button.
    ///
    /// This is "act on press". For "act on release", see [`UiNode::is_click_released()`].
    pub fn is_right_clicked(&self) -> bool {
        self.sys().check_clicked(self.node().id, MouseButton::Right)
    }

    /// Returns `true` if this node was just clicked with the given mouse button.
    ///
    /// This is "act on press". For "act on release", see [`UiNode::is_click_released()`].
    pub fn is_mouse_button_clicked(&self, button: MouseButton) -> bool {
        self.sys().check_clicked(self.node().id, button)
    }

    /// Returns `true` if a left mouse button click was just released on this node.
    pub fn is_click_released(&self) -> bool {
        self.sys().check_click_released(self.node().id, MouseButton::Left)
    }

    /// Returns details about the click if this node was just clicked, otherwise `None`.
    ///
    /// If the node was clicked multiple times in the last frame, returns only the last click.
    pub fn clicked_at(&self) -> Option<Click> {
        let sys = self.sys();
        let node = self.node();
        let event = sys.check_clicked_at(node.id, MouseButton::Left)?;
        let relative_position = Vec2::new(
            ((event.position.x / sys.size.x) - node.real_rect.x[0]) / node.real_rect.size().x,
            ((event.position.y / sys.size.y) - node.real_rect.y[0]) / node.real_rect.size().y,
        );
        Some(Click {
            relative_position,
            absolute_position: event.position,
            timestamp: event.timestamp,
        })
    }

    /// If this node was dragged with the given mouse button, returns a struct describing the drag event.
    pub fn is_mouse_button_dragged(&self, button: MouseButton) -> Option<Drag> {
        let sys = self.sys();
        let node = self.node();
        let event = sys.check_dragged(node.id, button)?;
        sys.drag_from_event_with_rect(event, node.real_rect)
    }

    /// If this node was dragged with the left mouse button, returns a struct describing the drag event.
    pub fn is_dragged(&self) -> Option<Drag> {
        self.is_mouse_button_dragged(MouseButton::Left)
    }

    /// Returns `true` if a left button mouse drag on this node was just released.
    ///
    /// Unlike [`UiNode::is_click_released()`], this is `true` even if the cursor is not on the node when the button is released.
    pub fn is_drag_released(&self) -> bool {
        self.sys().check_drag_released(self.node().id, MouseButton::Left)
    }

    /// If this node is currently hovered by the cursor, returns hover information.
    pub fn is_hovered(&self) -> Option<Hover> {
        let sys = self.sys();
        if !sys.check_hovered(self.node().id) {
            return None;
        }
        Some(Hover { absolute_position: sys.mouse_input.cursor_position })
    }

    /// Returns `true` if this node currently has keyboard focus.
    pub fn is_focused(&self) -> bool {
        self.sys().check_focused(self.node().id)
    }

    /// If this node is being held with the left mouse button, returns the duration of the hold.
    pub fn is_held(&self) -> Option<Duration> {
        self.sys().check_held_duration(self.node().id, MouseButton::Left)
    }

    /// Returns the total scroll delta for this node in the last frame, or `None` if no scroll events occurred.
    pub fn is_scrolled(&self) -> Option<Vec2> {
        self.sys().check_scrolled(self.node().id)
    }

    /// Returns details about the last scroll event on this node, or `None` if no scroll occurred.
    ///
    /// If the node was scrolled multiple times in the last frame, returns only the last scroll.
    pub fn scrolled_at(&self) -> Option<ScrollEvent> {
        let sys = self.sys();
        let node = self.node();
        let scroll_event = sys.check_last_scroll_event(node.id)?;
        let relative_position = Vec2::new(
            ((scroll_event.position.x / sys.size.x) - node.real_rect.x[0]) / node.real_rect.size().x,
            ((scroll_event.position.y / sys.size.y) - node.real_rect.y[0]) / node.real_rect.size().y,
        );
        Some(ScrollEvent {
            relative_position,
            absolute_position: scroll_event.position,
            delta: scroll_event.delta,
            timestamp: scroll_event.timestamp,
        })
    }
}

impl Ui {
    /// Returns `true` if the node corresponding to `key` was just clicked with the left mouse button.
    ///
    /// This is "act on press". For "act on release", see [`Ui::is_click_released()`].
    pub fn is_clicked(&self, key: NodeKey) -> bool {
        self.sys.check_clicked(key.id_with_subtree(), MouseButton::Left)
    }

    /// Returns `true` if the node corresponding to `key` was just clicked with the right mouse button.
    ///
    /// This is "act on press". For "act on release", see [`Ui::is_click_released()`].
    pub fn is_right_clicked(&self, key: NodeKey) -> bool {
        self.sys.check_clicked(key.id_with_subtree(), MouseButton::Right)
    }

    /// Returns `true` if the node corresponding to `key` was just clicked with the given mouse button.
    ///
    /// This is "act on press". For "act on release", see [`Ui::is_click_released()`].
    pub fn is_mouse_button_clicked(&self, key: NodeKey, button: MouseButton) -> bool {
        self.sys.check_clicked(key.id_with_subtree(), button)
    }

    /// Returns `true` if a left button mouse click was just released on the node corresponding to `key`.
    pub fn is_click_released(&self, key: NodeKey) -> bool {
        self.sys.check_click_released(key.id_with_subtree(), MouseButton::Left)
    }

    /// Returns details about the click if the node corresponding to `key` was just clicked, otherwise `None`.
    ///
    /// If the node was clicked multiple times in the last frame, returns only the last click.
    pub fn clicked_at(&self, key: NodeKey) -> Option<Click> {
        self.get_node(key)?.clicked_at()
    }

    /// If the node corresponding to `key` was dragged with the given mouse button, returns the drag info.
    pub fn is_mouse_button_dragged(&self, key: NodeKey, button: MouseButton) -> Option<Drag> {
        self.get_node(key)?.is_mouse_button_dragged(button)
    }

    /// If the node corresponding to `key` was dragged with the left mouse button, returns the drag info.
    pub fn is_dragged(&self, key: NodeKey) -> Option<Drag> {
        self.is_mouse_button_dragged(key, MouseButton::Left)
    }

    /// Returns `true` if a left button mouse drag on the node corresponding to `key` was just released.
    ///
    /// Unlike [`Ui::is_click_released()`], this is `true` even if the cursor is not on the node when the button is released.
    pub fn is_drag_released(&self, key: NodeKey) -> bool {
        self.sys.check_drag_released(key.id_with_subtree(), MouseButton::Left)
    }

    /// If the node corresponding to `key` is currently hovered by the cursor, returns hover information.
    pub fn is_hovered(&self, key: NodeKey) -> Option<Hover> {
        if !self.sys.check_hovered(key.id_with_subtree()) {
            return None;
        }
        Some(Hover { absolute_position: self.sys.mouse_input.cursor_position })
    }

    /// Returns `true` if the node corresponding to `key` currently has keyboard focus.
    pub fn is_focused(&self, key: NodeKey) -> bool {
        self.sys.check_focused(key.id_with_subtree())
    }

    /// If the node corresponding to `key` is being held with the left mouse button, returns the duration of the hold.
    pub fn is_held(&self, key: NodeKey) -> Option<Duration> {
        self.sys.check_held_duration(key.id_with_subtree(), MouseButton::Left)
    }

    /// Returns the total scroll delta for the node corresponding to `key` in the last frame, or `None` if no scroll events occurred.
    pub fn is_scrolled(&self, key: NodeKey) -> Option<Vec2> {
        self.sys.check_scrolled(key.id_with_subtree())
    }

    /// Returns details about the last scroll event on the node corresponding to `key`, or `None` if no scroll occurred.
    ///
    /// If the node was scrolled multiple times in the last frame, returns only the last scroll.
    pub fn scrolled_at(&self, key: NodeKey) -> Option<ScrollEvent> {
        self.get_node(key)?.scrolled_at()
    }
}


impl UiParent {
    // Shortcut methods to get the real immediate-mode style (at the cost of having to bass back the Ui reference.)
    // The UiParent can't hold a &Ui because it would conflict with use of Ui inside the nest() closure.
    // This would probably not be an issue if Rust had a construct like python's `with` or C#'s `using`. Instead we have to do it with closures.
    fn key(&self, ui: &Ui) -> NodeKey {
        NodeKey::new_temp(ui.sys.nodes[self.i].id, "")
    }

    /// Returns `true` if this node was just clicked with the left mouse button.
    ///
    /// This is "act on press". For "act on release", see [`Ui::is_click_released()`].
    pub fn is_clicked(&self, ui: &Ui) -> bool {
        ui.is_clicked(self.key(ui))
    }

    /// Returns `true` if this node was just clicked with the right mouse button.
    ///
    /// This is "act on press". For "act on release", see [`Ui::is_click_released()`].
    pub fn is_right_clicked(&self, ui: &Ui) -> bool {
        ui.is_right_clicked(self.key(ui))
    }

    /// Returns `true` if this node was just clicked with the given mouse button.
    ///
    /// This is "act on press". For "act on release", see [`Ui::is_click_released()`].
    pub fn is_mouse_button_clicked(&self, ui: &Ui, button: MouseButton) -> bool {
        ui.is_mouse_button_clicked(self.key(ui), button)
    }

    /// Returns `true` if a left mouse button click was just released on this node.
    pub fn is_click_released(&self, ui: &Ui) -> bool {
        ui.is_click_released(self.key(ui))
    }

    /// Returns details about the click if this node was just clicked, otherwise `None`.
    ///
    /// If the node was clicked multiple times in the last frame, returns only the last click.
    pub fn clicked_at(&self, ui: &Ui) -> Option<Click> {
        ui.clicked_at(self.key(ui))
    }

    /// If this node was dragged with the given mouse button, returns a struct describing the drag event.
    pub fn is_mouse_button_dragged(&self, ui: &Ui, button: MouseButton) -> Option<Drag> {
        ui.is_mouse_button_dragged(self.key(ui), button)
    }

    /// If this node was dragged with the left mouse button, returns a struct describing the drag event.
    pub fn is_dragged(&self, ui: &Ui) -> Option<Drag> {
        ui.is_dragged(self.key(ui))
    }

    /// Returns `true` if a left button mouse drag on this node was just released.
    ///
    /// Unlike [`Ui::is_click_released()`], this is `true` even if the cursor is not on the node when the button is released.
    pub fn is_drag_released(&self, ui: &Ui) -> bool {
        ui.is_drag_released(self.key(ui))
    }

    /// If this node is currently hovered by the cursor, returns hover information.
    pub fn is_hovered(&self, ui: &Ui) -> Option<Hover> {
        ui.is_hovered(self.key(ui))
    }

    /// Returns `true` if this node currently has keyboard focus.
    pub fn is_focused(&self, ui: &Ui) -> bool {
        ui.is_focused(self.key(ui))
    }

    /// If this node is being held with the left mouse button, returns the duration of the hold.
    pub fn is_held(&self, ui: &Ui) -> Option<Duration> {
        ui.is_held(self.key(ui))
    }

    /// Returns the total scroll delta for this node in the last frame, or `None` if no scroll events occurred.
    pub fn is_scrolled(&self, ui: &Ui) -> Option<Vec2> {
        ui.is_scrolled(self.key(ui))
    }

    /// Returns details about the last scroll event on this node, or `None` if no scroll occurred.
    ///
    /// If the node was scrolled multiple times in the last frame, returns only the last scroll.
    pub fn scrolled_at(&self, ui: &Ui) -> Option<ScrollEvent> {
        ui.scrolled_at(self.key(ui))
    }
}
