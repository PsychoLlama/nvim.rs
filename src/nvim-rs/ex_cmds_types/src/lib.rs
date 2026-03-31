//! Shared FFI types for Ex command argument structs.
//!
//! This crate provides `ExArg` (a `repr(C)` mirror of C's `exarg_T`) so that
//! multiple Rust crates can access Ex command argument fields directly without
//! going through C accessor functions.

#![allow(unsafe_code)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::use_self)]

use std::ffi::{c_char, c_int, c_void};

/// Function pointer type matching C `typedef char *(*LineGetter)(int, void *, int, bool)`.
pub type LineGetterFn = unsafe extern "C" fn(
    c: c_int,
    cookie: *mut c_void,
    indent: c_int,
    do_concat: bool,
) -> *mut c_char;

/// Rust representation of C `exarg_T` (struct exarg).
///
/// Layout must exactly match the C struct definition in `ex_cmds_defs.h`.
/// `#[repr(C)]` ensures the same layout as the C compiler produces.
#[repr(C)]
pub struct ExArg {
    pub arg: *mut c_char,
    pub args: *mut *mut c_char,
    pub arglens: *mut usize,
    pub argc: usize,
    pub nextcmd: *mut c_char,
    pub cmd: *mut c_char,
    pub cmdlinep: *mut *mut c_char,
    pub cmdline_tofree: *mut c_char,
    pub cmdidx: c_int,
    pub argt: u32,
    pub skip: c_int,
    pub forceit: c_int,
    pub addr_count: c_int,
    pub line1: i32,
    pub line2: i32,
    pub addr_type: c_int,
    pub flags: c_int,
    pub do_ecmd_cmd: *mut c_char,
    pub do_ecmd_lnum: i32,
    pub append: c_int,
    pub usefilter: c_int,
    pub amount: c_int,
    pub regname: c_int,
    pub force_bin: c_int,
    pub read_edit: c_int,
    pub mkdir_p: c_int,
    pub force_ff: c_int,
    pub force_enc: c_int,
    pub bad_char: c_int,
    pub useridx: c_int,
    pub errmsg: *mut c_char,
    pub ea_getline: Option<LineGetterFn>,
    pub cookie: *mut c_void,
    pub cstack: *mut c_void,
}

// Safety: ExArg is a C struct passed across FFI boundaries; it contains raw
// pointers and is inherently unsafe. The caller must ensure proper lifetime.
unsafe impl Send for ExArg {}
unsafe impl Sync for ExArg {}

impl ExArg {
    /// Allocate a zeroed `ExArg` on the heap with `line1=1`, `line2=1`.
    ///
    /// Uses C `xcalloc` for allocation so that C code can free it with `xfree`.
    pub fn alloc() -> *mut Self {
        unsafe {
            let eap = xcalloc(1, std::mem::size_of::<ExArg>()) as *mut ExArg;
            (*eap).line1 = 1;
            (*eap).line2 = 1;
            eap
        }
    }
}

extern "C" {
    fn xcalloc(nmemb: usize, size: usize) -> *mut std::ffi::c_void;
}

/// Typed pointer to an `exarg_T` C struct.
pub type ExArgHandle = *mut ExArg;
