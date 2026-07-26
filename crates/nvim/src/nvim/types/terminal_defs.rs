#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

// Opaque C type: layout unknown here, only ever used behind a pointer.
#[repr(C)]
pub struct terminal {
    _data: [u8; 0],
    _marker: ::core::marker::PhantomData<(*mut u8, ::core::marker::PhantomPinned)>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TerminalOptions {
    pub data: *mut ::core::ffi::c_void,
    pub width: uint16_t,
    pub height: uint16_t,
    pub read_pause_cb: terminal_read_pause_cb,
    pub write_cb: terminal_write_cb,
    pub resize_cb: terminal_resize_cb,
    pub resume_cb: terminal_resume_cb,
    pub close_cb: terminal_close_cb,
    pub force_crlf: bool,
}
pub type terminal_close_cb = Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>;
pub type terminal_read_pause_cb =
    Option<unsafe extern "C" fn(bool, *mut ::core::ffi::c_void) -> ()>;
pub type terminal_resize_cb =
    Option<unsafe extern "C" fn(uint16_t, uint16_t, *mut ::core::ffi::c_void) -> ()>;
pub type terminal_resume_cb = Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>;
pub type terminal_write_cb = Option<
    unsafe extern "C" fn(*const ::core::ffi::c_char, size_t, *mut ::core::ffi::c_void) -> (),
>;
