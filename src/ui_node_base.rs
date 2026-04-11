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
            UiRef::Mut(ui) => return ui,
            UiRef::NonMut(_) => unreachable!(),
        }
    }

    pub(crate) fn ui(&self) -> &Ui {
        match &self.ui_ref {
            UiRef::Mut(ui) => ui,
            UiRef::NonMut(ui) => return ui,
        }
    }
}


// // use crate::*;
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

// This is another way that we could do it safely:

// use crate::*;

// pub struct UiNode2<'a> {
//     pub(crate) i: NodeI,
//     pub(crate) ui_ref: UiRef<'a>,
// }
// pub(crate) enum UiRef<'a> {
//     Mut(&'a mut Ui),
//     NonMut(&'a Ui),
// }

// impl<'a> UiNode2<'a> {
//     pub(crate) fn ui_mut(&mut self) -> &mut Ui {
//         match &mut self.ui_ref {
//             UiRef::Mut(ui) => return ui,
//             UiRef::NonMut(_) => unreachable!(),
//         }
//     }

//     pub(crate) fn ui(&self) -> &Ui {
//         match &self.ui_ref {
//             UiRef::Mut(ui) => ui,
//             UiRef::NonMut(ui) => return ui,
//         }
//     }
// }

// From here, it looks great. The lifetime parameter is a bit of useless noise but not a big deal.
// But the problem is that we'd still have to create the wrapper struct inside the Ui to have reference semantics (which is the whole point).

// fn get_node2_mut(&mut self, key: NodeKey) -> Option<&mut UiNode2> {
//     let i = self.nodes.node_hashmap.get(&key.id_with_subtree())?.slab_i;
//     if self.nodes[i].currently_hidden || self.nodes[i].exiting {
//         return None;
//     }

//     let wrapper = UiNode2 { i, ui_ref: UiRef::Mut(self)  };
//     return Some(self.sys.arena_for_wrapper_structs.alloc(wrapper));
// }

// If we try to do this, we'd get a partial-borrow-style issue, because the arena is another field of `self`.
// So we can't store mutable `&mut self` references in it without it obviously overlapping. 

// The solution would be to add another layer of indirection to the Ui: 

// pub struct Ui {
//     pub(crate) real: UiReal,
//     pub(crate) node_wrapper_arena: Bump,
// }

// Then the UiNode would store a reference to the UiReal.
// At the moment I am not ready to dump so much indirection.
// The NonNull system is complicated as well, but it's scoped to the UiNode: with that kind of indirection, everyone trying to read the code 