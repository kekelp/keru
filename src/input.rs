use crate::*;

use std::time::Duration;
use glam::Vec2;
use winit::event::MouseButton;

// Returns cursor position as a fraction of the node's inner (post-padding) rect,
// matching the coordinate space used by Pos::Frac for child positioning.
fn inner_relative_position(cursor: Vec2, window_size: Xy<f32>, rect: XyRect, padding: Xy<f32>) -> Vec2 {
    let inner_x0 = rect.x[0] + padding.x / window_size.x;
    let inner_y0 = rect.y[0] + padding.y / window_size.y;
    let inner_w  = rect.size().x - 2.0 * padding.x / window_size.x;
    let inner_h  = rect.size().y - 2.0 * padding.y / window_size.y;
    Vec2::new(
        (cursor.x / window_size.x - inner_x0) / inner_w,
        (cursor.y / window_size.y - inner_y0) / inner_h,
    )
}

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
        let logical_size = sys.logical_size();
        let relative_position = inner_relative_position(event.position, logical_size, node.real_rect, node.params.layout.padding);
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
        let node = self.node();
        if !sys.check_hovered(node.id) {
            return None;
        }
        let cursor = sys.mouse_input.cursor_position;
        let logical_size = sys.logical_size();
        let relative_position = inner_relative_position(cursor, logical_size, node.real_rect, node.params.layout.padding);
        Some(Hover { absolute_position: cursor, relative_position, last_enter_or_exit: node.hover_enter_exit_instant })
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
    pub fn scrolled_at(&self) -> Option<Scroll> {
        let sys = self.sys();
        let node = self.node();
        let scroll_event = sys.check_last_scroll_event(node.id)?;
        let logical_size = sys.logical_size();
        let relative_position = inner_relative_position(scroll_event.position, logical_size, node.real_rect, node.params.layout.padding);
        Some(Scroll {
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
        self.sys.check_clicked(key.id_with_key_scope(), MouseButton::Left)
    }

    /// Returns `true` if the node corresponding to `key` was just clicked with the right mouse button.
    ///
    /// This is "act on press". For "act on release", see [`Ui::is_click_released()`].
    pub fn is_right_clicked(&self, key: NodeKey) -> bool {
        self.sys.check_clicked(key.id_with_key_scope(), MouseButton::Right)
    }

    /// Returns `true` if a screen reader requested the given AccessKit `action`
    /// on the node corresponding to `key` during this frame.
    pub fn accesskit_action(&self, key: NodeKey, action: AccessKitAction) -> bool {
        let id = key.id_with_key_scope();
        self.sys.accesskit_actions.iter().any(|(qid, a)| *qid == id && *a == action)
    }

    /// Returns `true` if the node corresponding to `key` was just clicked with the given mouse button.
    ///
    /// This is "act on press". For "act on release", see [`Ui::is_click_released()`].
    pub fn is_mouse_button_clicked(&self, key: NodeKey, button: MouseButton) -> bool {
        self.sys.check_clicked(key.id_with_key_scope(), button)
    }

    /// Returns `true` if a left button mouse click was just released on the node corresponding to `key`.
    pub fn is_click_released(&self, key: NodeKey) -> bool {
        self.sys.check_click_released(key.id_with_key_scope(), MouseButton::Left)
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
        self.sys.check_drag_released(key.id_with_key_scope(), MouseButton::Left)
    }

    /// Returns `true` the node corresponding to `key` is currently hovered by the cursor.
    pub fn is_hovered(&self, key: NodeKey) -> bool {
        let id = key.id_with_key_scope();
        self.sys.check_hovered(id)
    }

    /// If the node corresponding to `key` is currently hovered by the cursor, returns hover information.
    pub fn is_hovered_info(&self, key: NodeKey) -> Option<Hover> {
        self.get_node(key)?.is_hovered()
    }

    /// Returns `true` if the node corresponding to `key` is currently holding the keyboard navigation focus.
    pub fn is_focused(&self, key: NodeKey) -> bool {
        // Some non-interactive nodes can be "silently" focused in a way that's just useful for future tab navigation. is_focused shouldn't report that, though, so we also check self.sys.show_focus_indicator
        self.sys.show_focus_indicator && self.sys.focused == Some(key.id_with_key_scope())
    }

    /// If the node corresponding to `key` is being held with the left mouse button, returns the duration of the hold.
    pub fn is_held(&self, key: NodeKey) -> Option<Duration> {
        self.sys.check_held_duration(key.id_with_key_scope(), MouseButton::Left)
    }

    /// Returns the total scroll delta for the node corresponding to `key` in the last frame, or `None` if no scroll events occurred.
    pub fn is_scrolled(&self, key: NodeKey) -> Option<Vec2> {
        self.sys.check_scrolled(key.id_with_key_scope())
    }

    /// Returns details about the last scroll event on the node corresponding to `key`, or `None` if no scroll occurred.
    ///
    /// If the node was scrolled multiple times in the last frame, returns only the last scroll.
    pub fn scrolled_at(&self, key: NodeKey) -> Option<Scroll> {
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
    pub fn is_hovered(&self, ui: &Ui) -> bool {
        ui.is_hovered(self.key(ui))
    }

    /// If this node is currently hovered by the cursor, returns hover information.
    pub fn is_hovered_info(&self, ui: &Ui) -> Option<Hover> {
        ui.is_hovered_info(self.key(ui))
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
    pub fn scrolled_at(&self, ui: &Ui) -> Option<Scroll> {
        ui.scrolled_at(self.key(ui))
    }
}
