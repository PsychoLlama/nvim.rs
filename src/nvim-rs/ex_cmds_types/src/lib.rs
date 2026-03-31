//! Shared FFI types for Ex command argument structs.
//!
//! This crate provides `ExArg` (a `repr(C)` mirror of C's `exarg_T`) and
//! `CmdMod` (a `repr(C)` mirror of C's `cmdmod_T`) so that multiple Rust
//! crates can access Ex command argument fields directly without going through
//! C accessor functions.

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

/// Opaque blob matching `regmatch_T` layout (176 bytes, 8-byte aligned).
///
/// C definition (`regexp_defs.h`):
/// - `regprog_T *regprog`: 8 bytes
/// - `char *startp[NSUBEXP]`: 10 * 8 = 80 bytes
/// - `char *endp[NSUBEXP]`: 10 * 8 = 80 bytes
/// - `colnr_T rm_matchcol`: 4 bytes (int)
/// - `bool rm_ic`: 1 byte + 3 padding
///
/// Total = 176 bytes
#[repr(C, align(8))]
#[derive(Default)]
pub struct RegMatchBlob {
    /// Raw storage matching `regmatch_T` layout; element 0 holds the `regprog_T *` pointer.
    pub data: [u64; 22],
}

/// Rust representation of C `cmdmod_T` (command modifiers struct).
///
/// Layout must exactly match the C struct definition in `ex_cmds_defs.h`.
/// `cmod_filter_regmatch` is represented as an opaque blob since Rust does
/// not need to access `regprog_T` fields directly.
#[repr(C)]
pub struct CmdMod {
    pub cmod_flags: c_int,
    pub cmod_split: c_int,
    pub cmod_tab: c_int,
    // 4 bytes padding before pointer
    pub cmod_filter_pat: *mut c_char,
    pub cmod_filter_regmatch: RegMatchBlob,
    pub cmod_filter_force: bool,
    // 3 bytes padding before c_int
    pub cmod_verbose: c_int,
    // 4 bytes padding before pointer (on 64-bit: pointer follows int+padding)
    pub cmod_save_ei: *mut c_char,
    pub cmod_did_sandbox: c_int,
    // 4 bytes padding before i64
    pub cmod_verbose_save: i64,
    pub cmod_save_msg_silent: c_int,
    pub cmod_save_msg_scroll: c_int,
    pub cmod_did_esilent: c_int,
}

unsafe impl Send for CmdMod {}
unsafe impl Sync for CmdMod {}

/// Magic flags for `CmdParseInfo`.
#[repr(C)]
pub struct CmdParseInfoMagic {
    pub file: bool,
    pub bar: bool,
}

/// Rust representation of C `CmdParseInfo`.
#[repr(C)]
pub struct CmdParseInfo {
    pub cmdmod: CmdMod,
    pub magic: CmdParseInfoMagic,
}

unsafe impl Send for CmdParseInfo {}
unsafe impl Sync for CmdParseInfo {}

/// Typed pointer to a `cmdmod_T` C struct.
pub type CmdModHandle = *mut CmdMod;

/// Typed pointer to a `CmdParseInfo` C struct.
pub type CmdParseInfoHandle = *mut CmdParseInfo;

/// Opaque blob matching `tasave_T` layout (192 bytes, 8-byte aligned).
#[repr(C, align(8))]
#[derive(Default)]
pub struct TasaveBlob {
    /// Raw storage matching `tasave_T` layout (192 bytes).
    pub data: [u64; 24],
}

/// Rust representation of C `save_state_T`.
///
/// Layout must exactly match the C struct definition in `ex_docmd.h`.
/// The `tabuf` field (`tasave_T`) is represented as an opaque blob since
/// Rust only needs to pass it through `save_typeahead`/`restore_typeahead`.
#[repr(C)]
pub struct SaveState {
    pub save_msg_scroll: c_int,
    pub save_restart_edit: c_int,
    pub save_msg_didout: bool,
    pub save_state: c_int,
    pub save_finish_op: bool,
    pub save_opcount: c_int,
    pub save_reg_executing: c_int,
    pub save_pending_end_reg_executing: bool,
    // 3 bytes padding then tasave_T at offset 32
    pub tabuf: TasaveBlob,
}

unsafe impl Send for SaveState {}
unsafe impl Sync for SaveState {}

/// Typed pointer to a `save_state_T` C struct.
pub type SaveStateHandle = *mut SaveState;
