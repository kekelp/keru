use crate::*;
use winit::event::MouseButton;

impl<'a> UiNode<'a> {
    /// Returns `true` if this node was just clicked with the left mouse button.
    ///
    /// This is "act on press". For "act on release", see [`UiNode::is_click_released()`].
    pub fn is_clicked(&self) -> bool {
        self.sys().check_clicked(self.node().id, MouseButton::Left)
    }
}
impl Ui {
    /// Returns `true` if the node corresponding to `key` was just clicked with the left mouse button.
    ///
    /// This is "act on press". For "act on release", see [`Ui::is_click_released()`].
    pub fn is_clicked(&self, key: NodeKey) -> bool {
        self.sys.check_clicked(key.id_with_subtree(), MouseButton::Left)
    }
}
impl UiParent {
    /// Returns `true` if the node corresponding to `key` was just clicked with the left mouse button.
    ///
    /// This is "act on press". For "act on release", see [`Ui::is_click_released()`].
    pub fn is_clicked(&self, ui: &Ui) -> bool {
        ui.is_clicked(self.key(ui))
    }
}