use crate::*;

pub struct UiNode2<'a> {
    pub(crate) i: NodeI,
    pub(crate) ui_ref: UiRef<'a>,
}
pub(crate) enum UiRef<'a> {
    Mut(&'a mut Ui),
    NonMut(&'a Ui),
}

impl<'a> UiNode2<'a> {
    pub(crate) fn ui_mut(&mut self) -> &mut Ui {
        match &mut self.ui_ref {
            // We only call ui_mut() from functions that take &mut self.
            // [`Ui::get_node_mut()`] ensures that if the caller has access to a `&mut UiNode`, it will have been constructed with `UiRef::Mut`.
            UiRef::NonMut(_) => unreachable!(),
            UiRef::Mut(ui) => return ui,
        }
    }

    pub(crate) fn ui(&self) -> &Ui {
        match &self.ui_ref {
            UiRef::Mut(ui) => ui,
            UiRef::NonMut(ui) => return ui,
        }
    }
}

// // // This is another way to do it. The unsafe is scarier, but as long as we don't refactor everything to avoid the partial borrow, the other way needs unsafe code as well. It still would have the advantage of not having a lifetime parameter in UiNode2. Although maybe it's more honest to have it.
// use crate::*;
// use std::ptr::NonNull;

// pub struct UiNode2 {
//     /// This is a trick to be able to access the Ui with good "reference semantics".
//     /// That is, the api looks like this: 
//     /// 
//     /// ```
//     /// let node: &UiNode = ui.get_node(key);
//     /// let node_mut: &mut UiNode = ui.get_node_mut(key);
//     /// ```
//     /// 
//     /// Rather than this: 
//     /// 
//     /// ```
//     /// let node: UiNode = ui.get_node_t(key);
//     /// let mut node_mut: UiNodeMut = ui.get_node_mut(key);
//     /// ```
//     /// 
//     /// Where UiNode and UiNodeMut are crappy separate wrapper structs, the caller has to make the node_mut binding mutable, etc.
//     /// 
//     /// You can read it as just a reference to the Ui. The public interface is sound.
//     ui_ref: NonNull<Ui>,
//     pub(crate) i: NodeI,
// }
// impl UiNode2 {
//     pub(crate) fn ui_mut(&mut self) -> &mut Ui {
//         unsafe { self.ui_ref.as_mut() }
//     }
//     pub(crate) fn ui(&self) -> &Ui {
//         unsafe { self.ui_ref.as_ref() }
//     }

//     pub(crate) fn new(i: NodeI, unsafe_ui_pointer: NonNull<Ui>) -> Self {
//         Self { ui_ref: unsafe_ui_pointer,  i }
//     }
// }



