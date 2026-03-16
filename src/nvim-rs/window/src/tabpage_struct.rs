//! Repr(C) mirror of `tabpage_T` from `buffer_defs.h`.
//!
//! This struct provides direct field access to `tabpage_T` from Rust, eliminating
//! the need for C accessor functions. Layout validated by `_Static_assert`
//! checks in `src/nvim/window_struct_check.c`.
//!
//! # Safety
//! This struct MUST match the C `tabpage_T` layout exactly. All offsets are
//! validated at compile time via C static assertions.

#![allow(dead_code)]

use std::ffi::{c_int, c_void};

use crate::{win_struct::HandleT, Frame, TabpageHandle, WinHandle};

/// Opaque 24-byte representation of `ScopeDictDictItem`.
///
/// `ScopeDictDictItem` = `typval_T(16) + uint8_t(1) + char[1](1) + padding(6)` = 24 bytes.
/// We never access this from Rust, so opaque padding is sufficient.
#[repr(C)]
#[derive(Clone, Copy)]
struct ScopeDictDictItemOpaque {
    _data: [u8; 24],
}

/// Repr(C) mirror of C `tabpage_T` (`struct tabpage_S`).
///
/// Fields are laid out exactly as in `buffer_defs.h`.
/// Complex nested types that are not accessed directly from Rust use
/// opaque `[u8; N]` padding preserving correct offsets.
///
/// All field offsets are validated by `_Static_assert` in
/// `src/nvim/window_struct_check.c`.
///
/// Layout (verified via static assertions):
/// - offset 0:   handle (i32)
/// - offset 4:   padding
/// - offset 8:   tp_next (*tabpage_T)
/// - offset 16:  tp_topframe (*frame_T)
/// - offset 24:  tp_curwin (*win_T)
/// - offset 32:  tp_prevwin (*win_T)
/// - offset 40:  tp_firstwin (*win_T)
/// - offset 48:  tp_lastwin (*win_T)
/// - offset 56:  tp_old_Rows_avail (i64)
/// - offset 64:  tp_old_Columns (i64)
/// - offset 72:  tp_ch_used (i64)
/// - offset 80:  tp_first_diff (*diff_T, opaque)
/// - offset 88:  tp_diffbuf[8] (8 x *buf_T, opaque, 64 bytes)
/// - offset 152: tp_diff_invalid (i32)
/// - offset 156: tp_diff_update (i32)
/// - offset 160: tp_snapshot[2] (2 x *frame_T, 16 bytes)
/// - offset 176: tp_winvar (ScopeDictDictItem, opaque, 24 bytes)
/// - offset 200: tp_vars (*dict_T, opaque)
/// - offset 208: tp_localdir (*char)
/// - offset 216: tp_prevdir (*char)
/// - total: 224 bytes
#[repr(C)]
pub struct TabpageStruct {
    // offset 0
    pub handle: HandleT,
    _pad0: [u8; 4],

    // offset 8
    pub tp_next: *mut TabpageStruct,

    // offset 16
    pub tp_topframe: *mut Frame,

    // offset 24
    pub tp_curwin: WinHandle,

    // offset 32
    pub tp_prevwin: WinHandle,

    // offset 40
    pub tp_firstwin: WinHandle,

    // offset 48
    pub tp_lastwin: WinHandle,

    // offset 56
    pub tp_old_rows_avail: i64,

    // offset 64
    pub tp_old_columns: i64,

    // offset 72
    pub tp_ch_used: i64,

    // offset 80: tp_first_diff (*diff_T, opaque)
    _tp_first_diff: *mut c_void,

    // offset 88: tp_diffbuf[8] (8 pointers = 64 bytes, opaque)
    _tp_diffbuf: [*mut c_void; 8],

    // offset 152
    _tp_diff_invalid: c_int,
    _tp_diff_update: c_int,

    // offset 160: tp_snapshot[2] (2 frame pointers = 16 bytes)
    pub tp_snapshot: [*mut Frame; 2],

    // offset 176: tp_winvar (ScopeDictDictItem, opaque 24 bytes)
    _tp_winvar: ScopeDictDictItemOpaque,

    // offset 200: tp_vars (*dict_T, opaque)
    pub tp_vars: *mut c_void,

    // offset 208
    pub tp_localdir: *mut i8,

    // offset 216
    pub tp_prevdir: *mut i8,
    // total: 224 bytes
}

impl TabpageHandle {
    /// Dereference this handle as a `TabpageStruct` reference.
    ///
    /// # Safety
    /// The handle must be a valid, non-null `tabpage_T*`.
    #[inline]
    #[must_use]
    pub unsafe fn as_tabpage_ref<'a>(self) -> &'a TabpageStruct {
        &*(self.as_ptr().cast::<TabpageStruct>())
    }

    /// Dereference this handle as a mutable `TabpageStruct` reference.
    ///
    /// # Safety
    /// The handle must be a valid, non-null `tabpage_T*`.
    #[inline]
    #[must_use]
    pub unsafe fn as_tabpage_mut<'a>(self) -> &'a mut TabpageStruct {
        &mut *(self.as_ptr().cast::<TabpageStruct>())
    }
}

// =============================================================================
// Phase 6: Tabpage accessor exports
// =============================================================================

/// Get `tp_firstwin` field from a tabpage.
///
/// # Safety
/// `tp` must be a valid non-null tabpage handle.
#[must_use]
#[export_name = "nvim_tabpage_get_firstwin"]
pub unsafe extern "C" fn tabpage_get_firstwin(tp: TabpageHandle) -> WinHandle {
    tp.as_tabpage_ref().tp_firstwin
}

/// Set `tp_firstwin` field on a tabpage.
///
/// # Safety
/// `tp` must be a valid non-null tabpage handle.
#[export_name = "nvim_tabpage_set_firstwin"]
pub unsafe extern "C" fn tabpage_set_firstwin(tp: TabpageHandle, wp: WinHandle) {
    tp.as_tabpage_mut().tp_firstwin = wp;
}

/// Get `tp_lastwin` field from a tabpage.
///
/// # Safety
/// `tp` must be a valid non-null tabpage handle.
#[must_use]
#[export_name = "nvim_tabpage_get_lastwin"]
pub unsafe extern "C" fn tabpage_get_lastwin(tp: TabpageHandle) -> WinHandle {
    tp.as_tabpage_ref().tp_lastwin
}

/// Set `tp_lastwin` field on a tabpage.
///
/// # Safety
/// `tp` must be a valid non-null tabpage handle.
#[export_name = "nvim_tabpage_set_lastwin"]
pub unsafe extern "C" fn tabpage_set_lastwin(tp: TabpageHandle, wp: WinHandle) {
    tp.as_tabpage_mut().tp_lastwin = wp;
}

/// Get `tp_curwin` field from a tabpage.
///
/// # Safety
/// `tp` must be a valid non-null tabpage handle.
#[must_use]
#[export_name = "nvim_tabpage_get_curwin"]
pub unsafe extern "C" fn tabpage_get_curwin(tp: TabpageHandle) -> WinHandle {
    tp.as_tabpage_ref().tp_curwin
}

/// Set `tp_curwin` field on a tabpage.
///
/// # Safety
/// `tp` must be a valid non-null tabpage handle.
#[export_name = "nvim_tabpage_set_curwin"]
pub unsafe extern "C" fn tabpage_set_curwin(tp: TabpageHandle, wp: WinHandle) {
    tp.as_tabpage_mut().tp_curwin = wp;
}

/// Get `tp_next` field from a tabpage.
///
/// # Safety
/// `tp` must be a valid non-null tabpage handle.
#[must_use]
#[export_name = "nvim_tabpage_get_next"]
pub unsafe extern "C" fn tabpage_get_next(tp: TabpageHandle) -> TabpageHandle {
    let next_ptr = tp.as_tabpage_ref().tp_next;
    TabpageHandle::from_ptr(next_ptr.cast::<c_void>())
}

/// Set `tp_next` field on a tabpage.
///
/// # Safety
/// `tp` must be a valid non-null tabpage handle.
#[export_name = "nvim_tabpage_set_next"]
pub unsafe extern "C" fn tabpage_set_next(tp: TabpageHandle, next: TabpageHandle) {
    tp.as_tabpage_mut().tp_next = next.as_ptr().cast::<TabpageStruct>();
}

/// Get `handle` field from a tabpage as c_int.
///
/// # Safety
/// `tp` must be a valid non-null tabpage handle.
#[must_use]
#[export_name = "nvim_tabpage_get_handle"]
pub unsafe extern "C" fn tabpage_get_handle(tp: TabpageHandle) -> c_int {
    tp.as_tabpage_ref().handle
}

/// Get `tp_topframe` field from a tabpage.
///
/// # Safety
/// `tp` must be a valid non-null tabpage handle.
#[must_use]
#[export_name = "nvim_tabpage_get_topframe"]
pub unsafe extern "C" fn tabpage_get_topframe(tp: TabpageHandle) -> *mut Frame {
    tp.as_tabpage_ref().tp_topframe
}

/// Set `tp_topframe` field on a tabpage. Null-safe.
///
/// # Safety
/// `tp` may be null (null-check is performed).
#[export_name = "nvim_tabpage_set_topframe"]
pub unsafe extern "C" fn tabpage_set_topframe(tp: TabpageHandle, fr: *mut Frame) {
    if !tp.is_null() {
        tp.as_tabpage_mut().tp_topframe = fr;
    }
}

/// Get `tp_prevwin` field from a tabpage. Null-safe (returns null WinHandle if tp is null).
///
/// # Safety
/// `tp` may be null (null-check is performed).
#[must_use]
#[export_name = "nvim_tabpage_get_prevwin"]
pub unsafe extern "C" fn tabpage_get_prevwin(tp: TabpageHandle) -> WinHandle {
    if tp.is_null() {
        WinHandle::null()
    } else {
        tp.as_tabpage_ref().tp_prevwin
    }
}

/// Set `tp_prevwin` field on a tabpage. Null-safe.
///
/// # Safety
/// `tp` may be null (null-check is performed).
#[export_name = "nvim_tabpage_set_prevwin"]
pub unsafe extern "C" fn tabpage_set_prevwin(tp: TabpageHandle, wp: WinHandle) {
    if !tp.is_null() {
        tp.as_tabpage_mut().tp_prevwin = wp;
    }
}

/// Get `tp_old_Rows_avail` field from a tabpage as c_int. Null-safe (returns 0 if null).
///
/// # Safety
/// `tp` may be null (null-check is performed).
#[must_use]
#[export_name = "nvim_tabpage_get_old_rows_avail"]
pub unsafe extern "C" fn tabpage_get_old_rows_avail(tp: TabpageHandle) -> c_int {
    if tp.is_null() {
        0
    } else {
        #[allow(clippy::cast_possible_truncation)]
        {
            tp.as_tabpage_ref().tp_old_rows_avail as c_int
        }
    }
}

/// Set `tp_old_Rows_avail` field on a tabpage. Null-safe.
///
/// # Safety
/// `tp` may be null (null-check is performed).
#[export_name = "nvim_tabpage_set_old_rows_avail"]
pub unsafe extern "C" fn tabpage_set_old_rows_avail(tp: TabpageHandle, val: c_int) {
    if !tp.is_null() {
        tp.as_tabpage_mut().tp_old_rows_avail = i64::from(val);
    }
}

/// Get `tp_old_Columns` field from a tabpage as c_int. Null-safe (returns 0 if null).
///
/// # Safety
/// `tp` may be null (null-check is performed).
#[must_use]
#[export_name = "nvim_tabpage_get_old_columns"]
pub unsafe extern "C" fn tabpage_get_old_columns(tp: TabpageHandle) -> c_int {
    if tp.is_null() {
        0
    } else {
        #[allow(clippy::cast_possible_truncation)]
        {
            tp.as_tabpage_ref().tp_old_columns as c_int
        }
    }
}

/// Set `tp_old_Columns` field on a tabpage. Null-safe.
///
/// # Safety
/// `tp` may be null (null-check is performed).
#[export_name = "nvim_tabpage_set_old_columns"]
pub unsafe extern "C" fn tabpage_set_old_columns(tp: TabpageHandle, val: c_int) {
    if !tp.is_null() {
        tp.as_tabpage_mut().tp_old_columns = i64::from(val);
    }
}

/// Get `tp_ch_used` field from a tabpage as c_int. Null-safe (returns 0 if null).
///
/// # Safety
/// `tp` may be null (null-check is performed).
#[must_use]
#[export_name = "nvim_tabpage_get_ch_used"]
pub unsafe extern "C" fn tabpage_get_ch_used(tp: TabpageHandle) -> c_int {
    if tp.is_null() {
        0
    } else {
        #[allow(clippy::cast_possible_truncation)]
        {
            tp.as_tabpage_ref().tp_ch_used as c_int
        }
    }
}

/// Set `tp_ch_used` field on a tabpage. Null-safe.
///
/// # Safety
/// `tp` may be null (null-check is performed).
#[export_name = "nvim_tabpage_set_ch_used"]
pub unsafe extern "C" fn tabpage_set_ch_used(tp: TabpageHandle, val: i64) {
    if !tp.is_null() {
        tp.as_tabpage_mut().tp_ch_used = val;
    }
}

/// Get `tp_snapshot[idx]` field from a tabpage. Null-safe and bounds-safe.
///
/// # Safety
/// `tp` may be null (null-check is performed).
#[must_use]
#[export_name = "nvim_tabpage_get_snapshot"]
pub unsafe extern "C" fn tabpage_get_snapshot(tp: TabpageHandle, idx: c_int) -> *mut Frame {
    if tp.is_null() || !(0..2).contains(&idx) {
        return std::ptr::null_mut();
    }
    #[allow(clippy::cast_sign_loss)]
    tp.as_tabpage_ref().tp_snapshot[idx as usize]
}

/// Set `tp_snapshot[idx]` field on a tabpage. Null-safe and bounds-safe.
///
/// # Safety
/// `tp` may be null (null-check is performed).
#[export_name = "nvim_tabpage_set_snapshot"]
pub unsafe extern "C" fn tabpage_set_snapshot(tp: TabpageHandle, idx: c_int, val: *mut Frame) {
    if !tp.is_null() && (0..2).contains(&idx) {
        #[allow(clippy::cast_sign_loss)]
        let i = idx as usize;
        tp.as_tabpage_mut().tp_snapshot[i] = val;
    }
}

/// Get `tp_vars` field from a tabpage (opaque dict_T pointer).
///
/// # Safety
/// `tp` must be a valid non-null tabpage handle.
#[must_use]
#[export_name = "nvim_tabpage_get_vars"]
pub unsafe extern "C" fn tabpage_get_vars(tp: TabpageHandle) -> *mut c_void {
    tp.as_tabpage_ref().tp_vars
}

/// Get `w_winrow` of `tp_firstwin` of a tabpage. Null-safe.
///
/// Returns 0 if tp is null or tp_firstwin is null.
///
/// # Safety
/// `tp` may be null (null-check is performed).
#[must_use]
#[export_name = "nvim_tabpage_get_firstwin_winrow"]
pub unsafe extern "C" fn tabpage_get_firstwin_winrow(tp: TabpageHandle) -> c_int {
    if tp.is_null() {
        return 0;
    }
    let firstwin = tp.as_tabpage_ref().tp_firstwin;
    if firstwin.is_null() {
        return 0;
    }
    crate::win_struct::win_ref(firstwin).w_winrow
}
