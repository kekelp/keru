pub use keru_draw::{StyleHandle, TextBoxHandle, TextEditHandle};

use crate::*;

#[derive(Debug)]
pub enum TextI {
    TextBox(TextBoxHandle),
    TextEdit(TextEditHandle),
}


impl Ui {
    /// Insert a style, and get a [`StyleHandle`] that can be used to access and mutate it with the [`Self::get_style_mut`] functions.
    ///
    /// This function **should not be called on every frame**, as that would insert a new copy of the style every time.
    ///
    // todo: figure out a better way to do this.
    pub fn insert_style(&mut self, style: TextStyle) -> StyleHandle {
        self.sys.renderer.text.add_style(style, None)
    }

    pub fn get_style(&self, style: &StyleHandle) -> &TextStyle {
        self.sys.renderer.text.get_text_style(style)
    }

    pub fn get_style_mut(&mut self, style: &StyleHandle) -> &mut TextStyle {
        self.sys.renderer.text.get_text_style_mut(style)
    }
}
