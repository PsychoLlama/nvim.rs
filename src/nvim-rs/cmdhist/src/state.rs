//! History state: static variables and their accessors.
//!
//! Provides `#[no_mangle]` Rust statics that replace the C static variables
//! `history`, `hisidx`, `hisnum`, and `hislen` in `cmdhist.c`.

#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_int, c_void};
use std::os::raw::c_char;

use crate::HIST_COUNT;

/// Matches `histentry_T` in `cmdhist.h` (verified by `_Static_assert(sizeof(histentry_T) == 40)`).
#[repr(C)]
pub struct HistoryEntry {
    pub hisnum: c_int,
    pub _pad: [u8; 4],
    pub hisstr: *mut c_char,
    pub hisstrlen: usize,
    pub timestamp: u64,
    pub additional_data: *mut c_void,
}

// SAFETY: history state is accessed single-threaded in nvim (consistent with
// all other Rust crates in this codebase that use `static mut`).
unsafe impl Send for HistoryEntry {}
unsafe impl Sync for HistoryEntry {}

/// History tables - replaces `static histentry_T *(history[HIST_COUNT])`.
#[no_mangle]
pub static mut history: [*mut HistoryEntry; 5] = [
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
    std::ptr::null_mut(),
];

/// Last used index per history type - replaces `static int hisidx[HIST_COUNT]`.
#[no_mangle]
pub static mut hisidx: [c_int; 5] = [-1, -1, -1, -1, -1];

/// Identifying number of newest entry - replaces `static int hisnum[HIST_COUNT]`.
#[no_mangle]
pub static mut hisnum: [c_int; 5] = [0, 0, 0, 0, 0];

/// Actual length of history tables - replaces `static int hislen`.
#[no_mangle]
pub static mut hislen: c_int = 0;

// =============================================================================
// Accessor functions (replacements for the C functions deleted from cmdhist.c)
// =============================================================================

/// Return the current history length.
///
/// Replaces `int nvim_get_hislen(void)` in cmdhist.c.
#[no_mangle]
pub unsafe extern "C" fn nvim_get_hislen() -> c_int {
    hislen
}

/// Set the history length.
///
/// Replaces `void nvim_cmdhist_set_hislen(int val)` in cmdhist.c.
#[no_mangle]
pub unsafe extern "C" fn nvim_cmdhist_set_hislen(val: c_int) {
    hislen = val;
}

/// Return a pointer to the specified history table.
///
/// Replaces `histentry_T *get_histentry(int hist_type)` in cmdhist.c.
#[no_mangle]
pub unsafe extern "C" fn get_histentry(hist_type: c_int) -> *mut HistoryEntry {
    debug_assert!((0..HIST_COUNT).contains(&hist_type));
    history[hist_type as usize]
}

/// Set the history table pointer for a history type.
///
/// Replaces `void set_histentry(int hist_type, histentry_T *entry)` in cmdhist.c.
#[no_mangle]
pub unsafe extern "C" fn set_histentry(hist_type: c_int, entry: *mut HistoryEntry) {
    debug_assert!((0..HIST_COUNT).contains(&hist_type));
    history[hist_type as usize] = entry;
}

/// Return pointer to the `hisidx` element for the given history type.
///
/// Replaces `int *get_hisidx(int hist_type)` in cmdhist.c.
#[no_mangle]
pub unsafe extern "C" fn get_hisidx(hist_type: c_int) -> *mut c_int {
    debug_assert!((0..HIST_COUNT).contains(&hist_type));
    &raw mut hisidx[hist_type as usize]
}

/// Return pointer to the `hisnum` element for the given history type.
///
/// Replaces `int *get_hisnum(int hist_type)` in cmdhist.c.
#[no_mangle]
pub unsafe extern "C" fn get_hisnum(hist_type: c_int) -> *mut c_int {
    debug_assert!((0..HIST_COUNT).contains(&hist_type));
    &raw mut hisnum[hist_type as usize]
}
