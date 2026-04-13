//! Direct access to Neovim C global variables via `extern static`.
//!
//! This module replaces the C accessor functions in runtime_ffi.c that read
//! or write individual global variables. Layout is validated by
//! `_Static_assert(offsetof(...))` in runtime_ffi.c.
//!
//! # Safety
//! All access to `extern static mut` is inherently unsafe. Callers must not
//! hold references across any call that could modify these globals.

use std::ffi::{c_int, c_void};

use crate::ScidT;

// =============================================================================
// SctxT repr(C) mirror
// =============================================================================

/// Mirrors `sctx_T` from typval_defs.h.
///
/// Layout (verified by `_Static_assert(offsetof(...))` in runtime_ffi.c):
/// - offset 0:  sc_sid  (c_int = 4 bytes)
/// - offset 4:  sc_seq  (c_int = 4 bytes)
/// - offset 8:  sc_lnum (i32 = 4 bytes)
/// - offset 12: padding (4 bytes, natural alignment of u64)
/// - offset 16: sc_chan (u64 = 8 bytes)
///
/// Total: 24 bytes
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SctxT {
    pub sc_sid: ScidT,
    pub sc_seq: c_int,
    pub sc_lnum: i32,
    _pad: i32,
    pub sc_chan: u64,
}

// =============================================================================
// Extern static declarations
// =============================================================================

extern "C" {
    pub static mut current_sctx: SctxT;
    pub static mut debug_break_level: c_int;
    pub static mut debug_tick: c_int;
    pub static mut ex_nesting_level: c_int;
    pub static mut do_profiling: c_int;
    pub static mut got_int: bool;
    pub static mut global_busy: c_int;
    pub static mut listcmd_busy: bool;
    pub static mut did_source_packages: bool;
    pub static mut p_lpl: c_int;
    pub static mut p_verbose: i64; // OptInt = int64_t
}

// =============================================================================
// Safe accessors
// =============================================================================

/// Read `current_sctx.sc_sid`.
///
/// # Safety
/// No concurrent modifications to `current_sctx` may occur.
#[inline]
pub unsafe fn get_current_sctx_sid() -> ScidT {
    current_sctx.sc_sid
}

/// Read `current_sctx.sc_seq`.
#[inline]
pub unsafe fn get_current_sctx_seq() -> c_int {
    current_sctx.sc_seq
}

/// Read `current_sctx.sc_lnum`.
#[inline]
pub unsafe fn get_current_sctx_lnum() -> i32 {
    current_sctx.sc_lnum
}

/// Read `current_sctx.sc_chan`.
#[inline]
pub unsafe fn get_current_sctx_chan() -> u64 {
    current_sctx.sc_chan
}

/// Set `current_sctx.sc_sid`.
#[inline]
pub unsafe fn set_current_sctx_sid(sid: ScidT) {
    current_sctx.sc_sid = sid;
}

/// Set `current_sctx.sc_seq`.
#[inline]
pub unsafe fn set_current_sctx_seq(seq: c_int) {
    current_sctx.sc_seq = seq;
}

/// Set `current_sctx.sc_lnum`.
#[inline]
pub unsafe fn set_current_sctx_lnum(lnum: i32) {
    current_sctx.sc_lnum = lnum;
}

/// Save `current_sctx` to a stack-allocated copy; return the saved value.
///
/// Use with `restore_current_sctx` to restore.
#[inline]
pub unsafe fn save_current_sctx() -> SctxT {
    current_sctx
}

/// Restore `current_sctx` from a previously saved copy.
#[inline]
pub unsafe fn restore_current_sctx(saved: SctxT) {
    current_sctx = saved;
}

/// Increment `debug_break_level`.
#[inline]
pub unsafe fn inc_debug_break_level() {
    debug_break_level += 1;
}

/// Read `p_verbose` as c_int (truncating from OptInt = i64).
#[inline]
pub unsafe fn get_p_verbose() -> c_int {
    p_verbose as c_int
}

// =============================================================================
// Helper type alias
// =============================================================================

/// Opaque pointer for curbuf - used only for autocmd calls.
pub type CurbufPtr = *mut c_void;
