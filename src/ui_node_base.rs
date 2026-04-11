use std::ptr::NonNull;

use crate::*;

pub struct UiNode2 {
    unsafe_ui_pointer: NonNull<Ui>,
    pub(crate) i: NodeI,
}
impl UiNode2 {
    pub(crate) fn ui_mut(&mut self) -> &mut Ui {
        unsafe { self.unsafe_ui_pointer.as_mut() }
    }
    pub(crate) fn ui(&self) -> &Ui {
        unsafe { self.unsafe_ui_pointer.as_ref() }
    }

    pub(crate) fn new(i: NodeI, unsafe_ui_pointer: NonNull<Ui>) -> Self {
        Self { unsafe_ui_pointer,  i }
    }
}
