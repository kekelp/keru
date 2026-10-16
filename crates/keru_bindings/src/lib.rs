use std::ffi::c_void;
use keru::*;
use keru::node_library::*;
use keru::example_window_loop::run_example_loop;

pub type KeruUi = Ui;

pub type KeruKey = u64;

pub type KeruUpdateFn = Option<extern "C" fn(ui: *mut KeruUi, user_data: *mut c_void)>;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct KeruStr {
    pub ptr: *const u8,
    pub len: usize,
}

pub const KERU_DEFAULT: u32 = 0;
pub const KERU_V_STACK: u32 = 1;
pub const KERU_H_STACK: u32 = 2;
pub const KERU_V_SCROLL_STACK: u32 = 3;
pub const KERU_H_SCROLL_STACK: u32 = 4;
pub const KERU_BUTTON: u32 = 5;
pub const KERU_LABEL: u32 = 6;
pub const KERU_TEXT: u32 = 7;
pub const KERU_PANEL: u32 = 8;
pub const KERU_CONTAINER: u32 = 9;
pub const KERU_SPACER: u32 = 10;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct KeruNode {
    pub base: u32,
    pub key: KeruKey,
    pub text: KeruStr,
    pub has_color: bool,
    pub color: Color,
}

struct CState {
    update: KeruUpdateFn,
    user_data: *mut c_void,
}

fn c_update(state: &mut CState, ui: &mut Ui) {
    if let Some(update) = state.update {
        update(ui, state.user_data);
    }
}

fn base_node(base: u32) -> Node<'static> {
    match base {
        KERU_V_STACK => V_STACK,
        KERU_H_STACK => H_STACK,
        KERU_V_SCROLL_STACK => V_SCROLL_STACK,
        KERU_H_SCROLL_STACK => H_SCROLL_STACK,
        KERU_BUTTON => BUTTON,
        KERU_LABEL => LABEL,
        KERU_TEXT => TEXT,
        KERU_PANEL => PANEL,
        KERU_CONTAINER => CONTAINER,
        KERU_SPACER => SPACER,
        _ => DEFAULT,
    }
}

fn to_node_key(key: KeruKey) -> NodeKey {
    NodeKey::new(Id(key), "Bindings key")
}

unsafe fn str_bytes<'a>(s: KeruStr) -> Option<&'a [u8]> {
    if s.ptr.is_null() {
        return None;
    }
    return Some(unsafe { std::slice::from_raw_parts(s.ptr, s.len) });
}

unsafe fn add(ui: &mut Ui, node: &KeruNode) -> UiParent {
    let bytes = unsafe { str_bytes(node.text) };
    let text = bytes.map(String::from_utf8_lossy);

    let mut n = base_node(node.base);
    if node.key != 0 {
        n = n.key(to_node_key(node.key));
    }
    if let Some(text) = &text {
        n = n.text(text);
    }
    if node.has_color {
        n = n.color(node.color);
    }
    return ui.add(n);
}

#[unsafe(no_mangle)]
pub extern "C" fn keru_run(update: KeruUpdateFn, user_data: *mut c_void) {
    run_example_loop(CState { update, user_data }, c_update);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn keru_add(ui: *mut KeruUi, node: KeruNode) {
    let ui = unsafe { &mut *ui };
    unsafe { add(ui, &node) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn keru_add_begin(ui: *mut KeruUi, node: KeruNode) {
    let ui = unsafe { &mut *ui };
    unsafe { add(ui, &node) }.push_parent();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn keru_end(ui: *mut KeruUi) {
    let ui = unsafe { &*ui };
    ui.pop_parent();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn keru_is_clicked(ui: *mut KeruUi, key: KeruKey) -> bool {
    let ui = unsafe { &*ui };
    return ui.is_clicked(to_node_key(key));
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn keru_is_hovered(ui: *mut KeruUi, key: KeruKey) -> bool {
    let ui = unsafe { &*ui };
    return ui.is_hovered(to_node_key(key));
}
