#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct AutoCmd {
    pub pat: *mut AutoPat,
    pub id: int64_t,
    pub desc: *mut ::core::ffi::c_char,
    pub handler_cmd: *mut ::core::ffi::c_char,
    pub handler_fn: Callback,
    pub script_ctx: sctx_T,
    pub once: bool,
    pub nested: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct AutoPat {
    pub refcount: size_t,
    pub pat: *mut ::core::ffi::c_char,
    pub reg_prog: *mut regprog_T,
    pub group: ::core::ffi::c_int,
    pub patlen: ::core::ffi::c_int,
    pub buflocal_nr: ::core::ffi::c_int,
    pub allow_dirs: ::core::ffi::c_char,
}
pub type AutoPatCmd = AutoPatCmd_S;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct AutoPatCmd_S {
    pub lastpat: *mut AutoPat,
    pub auidx: size_t,
    pub ausize: size_t,
    pub afile_orig: *mut ::core::ffi::c_char,
    pub fname: *mut ::core::ffi::c_char,
    pub sfname: *mut ::core::ffi::c_char,
    pub tail: *mut ::core::ffi::c_char,
    pub group: ::core::ffi::c_int,
    pub event: event_T,
    pub script_ctx: sctx_T,
    pub arg_bufnr: ::core::ffi::c_int,
    pub data: *mut Object,
    pub next: *mut AutoPatCmd,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct aco_save_T {
    pub use_aucmd_win_idx: ::core::ffi::c_int,
    pub save_curwin_handle: handle_T,
    pub new_curwin_handle: handle_T,
    pub save_prevwin_handle: handle_T,
    pub new_curbuf: bufref_T,
    pub tp_localdir: *mut ::core::ffi::c_char,
    pub globaldir: *mut ::core::ffi::c_char,
    pub save_VIsual_active: bool,
    pub save_prompt_insert: ::core::ffi::c_int,
}

impl Default for aco_save_T {
    /// The zeroed state `aucmd_prepbuf` expects to be handed. Every caller
    /// declares one of these as a local and immediately fills it in.
    fn default() -> Self {
        aco_save_T {
            use_aucmd_win_idx: 0,
            save_curwin_handle: 0,
            new_curwin_handle: 0,
            save_prevwin_handle: 0,
            new_curbuf: bufref_T {
                br_buf: ::core::ptr::null_mut(),
                br_fnum: 0,
                br_buf_free_count: 0,
            },
            tp_localdir: ::core::ptr::null_mut(),
            globaldir: ::core::ptr::null_mut(),
            save_VIsual_active: false,
            save_prompt_insert: 0,
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct aucmdwin_T {
    pub auc_win: *mut win_T,
    pub auc_win_used: bool,
}
