//! Direct access to Neovim C global variables via `extern static`.
//!
//! This module replaces the C accessor functions in runtime_ffi.c that read
//! or write individual global variables. Layout is validated by
//! `_Static_assert(offsetof(...))` in runtime_ffi.c.
//!
//! # Safety
//! All access to `extern static mut` is inherently unsafe. Callers must not
//! hold references across any call that could modify these globals.

use std::ffi::{c_char, c_int, c_void};

use crate::{LinenrT, ScidT};

// =============================================================================
// GarrayT repr(C) mirror
// =============================================================================

/// Mirrors `garray_T` from garray_defs.h.
///
/// Layout (trivial - all int/ptr fields):
/// - offset 0:  ga_len      (c_int = 4 bytes)
/// - offset 4:  ga_maxlen   (c_int = 4 bytes)
/// - offset 8:  ga_itemsize (c_int = 4 bytes)
/// - offset 12: ga_growsize (c_int = 4 bytes)
/// - offset 16: ga_data     (*mut c_void = 8 bytes)
///
/// Total: 24 bytes
#[repr(C)]
#[allow(clippy::struct_field_names)]
pub struct GarrayT {
    pub ga_len: c_int,
    pub ga_maxlen: c_int,
    pub ga_itemsize: c_int,
    pub ga_growsize: c_int,
    pub ga_data: *mut c_void,
}

impl GarrayT {
    pub const fn zeroed() -> Self {
        Self {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: std::ptr::null_mut(),
        }
    }
}

// SAFETY: GarrayT is only accessed from a single thread in practice (guarded by Neovim's
// single-threaded event loop).
unsafe impl Send for GarrayT {}
unsafe impl Sync for GarrayT {}

// =============================================================================
// EstackT repr(C) mirror
// =============================================================================

/// Mirrors `estack_T` from runtime_defs.h.
///
/// Layout (verified by `_Static_assert(offsetof(...))` in runtime_ffi.c):
/// - offset 0:  es_lnum  (i32 = 4 bytes)
/// - offset 4:  padding  (4 bytes)
/// - offset 8:  es_name  (*mut c_char = 8 bytes)
/// - offset 16: es_type  (c_int = 4 bytes)
/// - offset 20: padding  (4 bytes)
/// - offset 24: es_info  (union of pointers = 8 bytes)
///
/// Total: 32 bytes
#[repr(C)]
pub struct EstackT {
    pub es_lnum: LinenrT,
    _pad1: i32,
    pub es_name: *mut c_char,
    pub es_type: c_int,
    _pad2: i32,
    /// Union of pointers: sctx_T*, ufunc_T*, AutoPatCmd*, except_T*
    /// Reinterpret according to es_type.
    pub es_info: *mut c_void,
}

impl EstackT {
    pub const fn null_info() -> *mut c_void {
        std::ptr::null_mut()
    }
}

// SAFETY: EstackT is only accessed from a single thread in practice.
unsafe impl Send for EstackT {}
unsafe impl Sync for EstackT {}

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

    /// The global execution stack garray (`exestack` in C).
    pub static mut exestack: GarrayT;

    /// `ga_grow` for growing a garray.
    fn ga_grow(gap: *mut GarrayT, n: c_int);
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
// Exestack helpers
// =============================================================================

/// Grow the exestack garray by `n` entries.
#[inline]
pub unsafe fn exestack_ga_grow(n: c_int) {
    ga_grow(&raw mut exestack, n);
}

/// Get a pointer to the execution stack entry at `idx`.
///
/// # Safety
/// Caller must ensure `idx` is in bounds (0..ga_len).
#[inline]
pub unsafe fn exestack_get_entry(idx: c_int) -> *mut EstackT {
    debug_assert!(idx >= 0 && idx < exestack.ga_len);
    exestack.ga_data.cast::<EstackT>().add(idx as usize)
}

/// Get a pointer to the next-available slot (ga_data[ga_len]).
#[inline]
pub unsafe fn exestack_get_next_slot() -> *mut EstackT {
    exestack
        .ga_data
        .cast::<EstackT>()
        .add(exestack.ga_len as usize)
}

/// Increment ga_len (after filling next slot).
#[inline]
pub unsafe fn exestack_inc_len() {
    exestack.ga_len += 1;
}

/// Decrement ga_len if it is > 1.
#[inline]
pub unsafe fn exestack_dec_len() {
    if exestack.ga_len > 1 {
        exestack.ga_len -= 1;
    }
}

/// Check if exestack has any entries.
#[inline]
pub unsafe fn exestack_has_data() -> bool {
    !exestack.ga_data.is_null() && exestack.ga_len > 0
}

/// Read SOURCING_LNUM (= exestack[ga_len-1].es_lnum).
///
/// # Panics
/// (debug only) if the stack is empty.
#[inline]
pub unsafe fn get_sourcing_lnum_direct() -> LinenrT {
    debug_assert!(exestack.ga_len > 0);
    (*exestack_get_entry(exestack.ga_len - 1)).es_lnum
}

// =============================================================================
// Helper type alias
// =============================================================================

/// Opaque pointer for curbuf - used only for autocmd calls.
pub type CurbufPtr = *mut c_void;
