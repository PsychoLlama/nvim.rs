#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

pub type CmdRedraw = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
pub struct CmdlineColorChunk {
    pub start: ::core::ffi::c_int,
    pub end: ::core::ffi::c_int,
    pub hl_id: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
pub struct CmdlineColors {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut CmdlineColorChunk,
}
pub type CmdlineInfo = cmdline_info;
#[derive(Copy, Clone)]
pub struct ColoredCmdline {
    pub prompt_id: ::core::ffi::c_uint,
    pub cmdbuff: *mut ::core::ffi::c_char,
    pub colors: CmdlineColors,
}
#[derive(Clone)]
pub struct cmdline_info {
    pub cmdbuff: *mut ::core::ffi::c_char,
    pub cmdbufflen: ::core::ffi::c_int,
    pub cmdlen: ::core::ffi::c_int,
    pub cmdpos: ::core::ffi::c_int,
    pub cmdspos: ::core::ffi::c_int,
    pub cmdfirstc: ::core::ffi::c_int,
    pub cmdindent: ::core::ffi::c_int,
    pub cmdprompt: *mut ::core::ffi::c_char,
    pub hl_id: ::core::ffi::c_int,
    pub overstrike: ::core::ffi::c_int,
    pub xpc: *mut expand_T,
    pub xp_context: ExpandContext,
    pub xp_arg: *mut ::core::ffi::c_char,
    pub input_fn: ::core::ffi::c_int,
    pub cmdbuff_replaced: bool,
    pub prompt_id: ::core::ffi::c_uint,
    pub highlight_callback: Callback,
    pub last_colors: ColoredCmdline,
    pub level: ::core::ffi::c_int,
    pub special_char: ::core::ffi::c_char,
    pub special_shift: bool,
    pub redraw_state: CmdRedraw,
    pub one_key: bool,
    pub mouse_used: *mut bool,
}
