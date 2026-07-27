#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

pub type qf_info_T = qf_info_S;
/// A stack of quickfix (or location) lists. Windows and buffers point at one.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct qf_info_S {
    pub qf_refcount: ::core::ffi::c_int,
    pub qf_listcount: ::core::ffi::c_int,
    pub qf_curlist: ::core::ffi::c_int,
    pub qf_maxcount: ::core::ffi::c_int,
    pub qf_lists: *mut qf_list_T,
    pub qfl_type: qfltype_T,
    pub qf_bufnr: ::core::ffi::c_int,
}
pub type qfltype_T = ::core::ffi::c_uint;
pub const QFLT_INTERNAL: qfltype_T = 2;
pub const QFLT_LOCATION: qfltype_T = 1;
pub const QFLT_QUICKFIX: qfltype_T = 0;
/// One quickfix list within a stack.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct qf_list_T {
    pub qf_id: ::core::ffi::c_uint,
    pub qfl_type: qfltype_T,
    pub qf_start: *mut qfline_T,
    pub qf_last: *mut qfline_T,
    pub qf_ptr: *mut qfline_T,
    pub qf_count: ::core::ffi::c_int,
    pub qf_index: ::core::ffi::c_int,
    pub qf_nonevalid: bool,
    pub qf_has_user_data: bool,
    pub qf_title: *mut ::core::ffi::c_char,
    pub qf_ctx: *mut typval_T,
    pub qf_qftf_cb: Callback,
    pub qf_dir_stack: *mut dir_stack_T,
    pub qf_directory: *mut ::core::ffi::c_char,
    pub qf_file_stack: *mut dir_stack_T,
    pub qf_currfile: *mut ::core::ffi::c_char,
    pub qf_multiline: bool,
    pub qf_multiignore: bool,
    pub qf_multiscan: bool,
    pub qf_changedtick: ::core::ffi::c_int,
}
/// A directory name pushed while parsing `make` output.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct dir_stack_T {
    pub next: *mut dir_stack_T,
    pub dirname: *mut ::core::ffi::c_char,
}
pub type qfline_T = qfline_S;
/// One entry in a quickfix list.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct qfline_S {
    pub qf_next: *mut qfline_T,
    pub qf_prev: *mut qfline_T,
    pub qf_lnum: linenr_T,
    pub qf_end_lnum: linenr_T,
    pub qf_fnum: ::core::ffi::c_int,
    pub qf_col: ::core::ffi::c_int,
    pub qf_end_col: ::core::ffi::c_int,
    pub qf_nr: ::core::ffi::c_int,
    pub qf_module: *mut ::core::ffi::c_char,
    pub qf_fname: *mut ::core::ffi::c_char,
    pub qf_pattern: *mut ::core::ffi::c_char,
    pub qf_text: *mut ::core::ffi::c_char,
    pub qf_viscol: ::core::ffi::c_char,
    pub qf_cleared: ::core::ffi::c_char,
    pub qf_type: ::core::ffi::c_char,
    pub qf_user_data: typval_T,
    pub qf_valid: ::core::ffi::c_char,
}
