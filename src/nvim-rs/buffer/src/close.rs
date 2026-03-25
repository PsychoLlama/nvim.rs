//! Buffer close and free operations.
//!
//! Implements `buf_freeall`.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(dead_code)]

use std::ffi::{c_char, c_int, c_void};

use crate::BufHandle;

// =============================================================================
// Constants (from buffer.h / buffer_defs.h)
// =============================================================================

const BFA_DEL: c_int = 1;
const BFA_WIPE: c_int = 2;
const BFA_KEEP_UNDO: c_int = 4;
const BFA_IGNORE_ABORT: c_int = 8;

// Event constants from auevents_enum.generated.h
const EVENT_BUFUNLOAD: c_int = 15;
const EVENT_BUFDELETE: c_int = 2;
const EVENT_BUFWIPEOUT: c_int = 18;

/// `BF_READERR` flag (from `buffer_defs.h`): got errors while reading the file
const BF_READERR: c_int = 0x40;

// =============================================================================
// bufref_T layout-compatible struct
// =============================================================================

/// Layout-compatible with C `bufref_T` (`buf_T*`, `int`, `int` = 16 bytes).
#[repr(C)]
#[allow(clippy::struct_field_names)]
pub struct BufRef {
    br_buf: *mut c_void,
    br_fnum: c_int,
    br_buf_free_count: c_int,
}

impl BufRef {
    #[must_use]
    pub const fn zeroed() -> Self {
        Self {
            br_buf: std::ptr::null_mut(),
            br_fnum: 0,
            br_buf_free_count: 0,
        }
    }
}

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    fn nvim_get_curbuf() -> BufHandle;
    fn nvim_get_curwin() -> *mut c_void;
    fn nvim_get_curtab() -> *mut c_void;

    fn nvim_buf_lock(buf: BufHandle);
    fn nvim_buf_unlock(buf: BufHandle);
    fn nvim_buf_set_nwindows(buf: BufHandle, val: c_int);
    fn nvim_buf_get_nwindows(buf: BufHandle) -> c_int;
    fn nvim_buf_get_flags(buf: BufHandle) -> c_int;
    fn nvim_buf_flags_and(buf: BufHandle, mask: c_int);
    fn nvim_buf_set_ml_line_count(buf: BufHandle, val: c_int);
    fn nvim_buf_get_ml_mfp(buf: BufHandle) -> *mut c_void;
    fn nvim_buf_get_b_p_bl(buf: BufHandle) -> c_int;

    fn set_bufref(bufref: *mut BufRef, buf: BufHandle);
    #[link_name = "rs_bufref_valid"]
    fn nvim_bufref_valid(bufref: *const BufRef) -> c_int;

    fn buf_updates_unload(buf: BufHandle, can_reload: bool);
    fn nvim_buf_get_b_fname(buf: BufHandle) -> *const c_char;
    fn apply_autocmds(
        event: c_int,
        fname: *const c_char,
        fname_io: *const c_char,
        force: bool,
        buf: BufHandle,
    ) -> bool;

    fn rs_win_valid_any_tab(win: *mut c_void) -> c_int;
    fn nvim_win_get_buffer(win: *mut c_void) -> BufHandle;
    fn goto_tabpage_win(tab: *mut c_void, win: *mut c_void);

    fn nvim_block_autocmds();
    fn nvim_unblock_autocmds();
    fn nvim_aborting() -> bool;

    fn rs_diff_buf_delete(buf: BufHandle);
    fn nvim_reset_synblock_if_curwin_buf(buf: BufHandle);
    fn nvim_buf_clearFolding_all_windows(buf: BufHandle);
    fn ml_close(buf: BufHandle, del_file: bool);
    fn u_clearallandblockfree(buf: BufHandle);
    fn nvim_syntax_clear_buf(buf: BufHandle);
}

// =============================================================================
// buf_freeall
// =============================================================================

/// Free all things allocated for a buffer that are related to the file.
///
/// Careful: may be called with `curwin` NULL when exiting.
///
/// `flags`:
/// - `BFA_DEL` (1):          buffer is going to be deleted
/// - `BFA_WIPE` (2):         buffer is going to be wiped out
/// - `BFA_KEEP_UNDO` (4):    do not free undo information
/// - `BFA_IGNORE_ABORT` (8): don't abort even when `aborting()` returns true
///
/// Mirrors C `buf_freeall`.
///
/// # Safety
///
/// Must be called on the main thread with valid Neovim state.
#[unsafe(export_name = "buf_freeall")]
pub unsafe extern "C" fn rs_buf_freeall(buf: BufHandle, flags: c_int) {
    let is_curbuf = buf == nvim_get_curbuf();
    let the_curwin = nvim_get_curwin();
    let is_curwin = !the_curwin.is_null() && nvim_win_get_buffer(the_curwin) == buf;
    let the_curtab = nvim_get_curtab();

    // Make sure the buffer isn't closed by autocommands.
    nvim_buf_lock(buf);

    let mut bufref = BufRef::zeroed();
    set_bufref(&raw mut bufref, buf);

    buf_updates_unload(buf, false);
    if nvim_bufref_valid(&raw const bufref) == 0 {
        // on_detach callback deleted the buffer.
        return;
    }

    if !nvim_buf_get_ml_mfp(buf).is_null()
        && apply_autocmds(
            EVENT_BUFUNLOAD,
            nvim_buf_get_b_fname(buf),
            nvim_buf_get_b_fname(buf),
            false,
            buf,
        )
        && nvim_bufref_valid(&raw const bufref) == 0
    {
        // Autocommands deleted the buffer.
        return;
    }

    if (flags & BFA_DEL) != 0
        && nvim_buf_get_b_p_bl(buf) != 0
        && apply_autocmds(
            EVENT_BUFDELETE,
            nvim_buf_get_b_fname(buf),
            nvim_buf_get_b_fname(buf),
            false,
            buf,
        )
        && nvim_bufref_valid(&raw const bufref) == 0
    {
        // Autocommands may delete the buffer.
        return;
    }

    if (flags & BFA_WIPE) != 0
        && apply_autocmds(
            EVENT_BUFWIPEOUT,
            nvim_buf_get_b_fname(buf),
            nvim_buf_get_b_fname(buf),
            false,
            buf,
        )
        && nvim_bufref_valid(&raw const bufref) == 0
    {
        // Autocommands may delete the buffer.
        return;
    }

    nvim_buf_unlock(buf);

    // If the buffer was in curwin and the window has changed, go back to that
    // window, if it still exists.  This avoids that ":edit x" triggering a
    // "tabnext" BufUnload autocmd leaving a window behind without a buffer.
    let new_curwin = nvim_get_curwin();
    if is_curwin && new_curwin != the_curwin && rs_win_valid_any_tab(the_curwin) != 0 {
        nvim_block_autocmds();
        goto_tabpage_win(the_curtab, the_curwin);
        nvim_unblock_autocmds();
    }

    // autocmds may abort script processing
    if (flags & BFA_IGNORE_ABORT) == 0 && nvim_aborting() {
        return;
    }

    // It's possible that autocommands change curbuf to the one being deleted.
    // Only return if curbuf changed to the deleted buffer.
    if buf == nvim_get_curbuf() && !is_curbuf {
        return;
    }

    rs_diff_buf_delete(buf); // Can't use 'diff' for unloaded buffer.
    nvim_reset_synblock_if_curwin_buf(buf);
    nvim_buf_clearFolding_all_windows(buf);

    ml_close(buf, true); // close and delete the memline/memfile
    nvim_buf_set_ml_line_count(buf, 0_i32); // no lines in buffer

    if (flags & BFA_KEEP_UNDO) == 0 {
        u_clearallandblockfree(buf);
    }

    nvim_syntax_clear_buf(buf); // reset syntax info
    nvim_buf_flags_and(buf, !BF_READERR); // a read error is no longer relevant
}
