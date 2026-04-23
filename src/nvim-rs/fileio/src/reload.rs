//! Buffer reload implementation.
//!
//! Provides `rs_buf_reload` (replaces the C `buf_reload` function) and
//! the internal `move_lines` helper.

use crate::{bref_void, buf_mut_void};
use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// External C dependencies
// =============================================================================

extern "C" {
    // --- aucmd save/restore ---
    /// Allocates aco_save_T, calls aucmd_prepbuf(aco, buf).
    fn nvim_aucmd_prepbuf_alloc(buf: *mut c_void) -> *mut c_void;
    /// Calls aucmd_restbuf(aco) then frees aco.
    fn nvim_aucmd_restbuf_free(aco: *mut c_void);

    // --- exarg_T management ---
    /// Allocates exarg_T and zero-initialises it (CLEAR_FIELD).
    fn nvim_exarg_alloc_clear() -> *mut c_void;
    /// Frees ea->cmd then frees ea itself.
    fn nvim_exarg_free(ea: *mut c_void);

    // --- prep_exarg (fileio/src/operations.rs, exported as "prep_exarg") ---
    fn prep_exarg(eap: *mut c_void, buf: *const c_void);

    // --- undo ---
    fn nvim_u_savecommon_reload_ok(buf: *mut c_void) -> c_int;
    #[link_name = "u_clearallandblockfree"]
    fn nvim_u_clearallandblockfree(buf: *mut c_void);
    #[link_name = "u_unchanged"]
    fn nvim_u_unchanged(buf: *mut c_void);

    // --- buffer list ---
    fn nvim_buflist_new(
        ffname: *const c_char,
        sfname: *const c_char,
        lnum: c_int,
        flags: c_int,
    ) -> *mut c_void;
    fn nvim_wipe_buffer(buf: *mut c_void);

    // --- bufref ---
    #[link_name = "set_bufref"]
    fn nvim_set_bufref(br: *mut c_void, buf: *mut c_void);
    fn nvim_bufref_valid(br: *mut c_void) -> c_int;

    // --- curbuf / curwin manipulation ---
    fn nvim_get_curbuf() -> *mut c_void;
    fn nvim_set_curbuf(buf: *mut c_void);
    fn nvim_curwin_set_buffer2(buf: *mut c_void);

    // --- p_ur ---
    fn nvim_get_p_ur() -> c_int;

    // --- memline ---
    fn nvim_buf_is_empty(buf: *mut c_void) -> c_int;
    fn nvim_ml_open_buf(buf: *mut c_void) -> c_int;
    #[link_name = "ml_get_buf"]
    fn nvim_ml_get_buf(buf: *mut c_void, lnum: c_int) -> *mut c_char;
    fn nvim_ml_append_curbuf(lnum: c_int, line: *const c_char) -> c_int;
    fn nvim_ml_delete_in_buf(buf: *mut c_void, lnum: c_int) -> c_int;
    fn nvim_get_curbuf_ml_line_count() -> c_int;

    // --- buffer flags ---
    fn nvim_curbuf_set_b_flags_or(flags: c_int);
    fn nvim_curbuf_set_b_keep_filetype(val: c_int);
    fn nvim_curbuf_set_b_mod_set(val: c_int);

    // --- readfile ---
    fn nvim_readfile_reload(
        buf: *mut c_void,
        ea: *mut c_void,
        flags: c_int,
        silent: c_int,
    ) -> c_int;
    fn nvim_aborting() -> c_int;
    fn nvim_shortmess_fileinfo() -> c_int;

    // --- unchanged / buf_updates ---
    fn nvim_unchanged(buf: *mut c_void);
    fn nvim_buf_updates_unload(buf: *mut c_void);

    // --- cursor & view ---
    fn nvim_curwin_set_topline_clamped(topline: c_int);
    fn nvim_curwin_get_cursor(lnum: *mut c_int, col: *mut c_int);
    fn nvim_curwin_set_cursor(lnum: c_int, col: c_int);
    fn nvim_check_cursor_curwin();
    fn nvim_update_topline_curwin();

    // --- fold helpers (from fold crate) ---
    fn rs_diff_invalidate(buf: *mut c_void);
    fn rs_foldmethodIsManual(wp: *mut c_void) -> c_int;
    fn rs_foldUpdateAll(wp: *mut c_void);

    // --- tab/window iteration (from change_ffi.c) ---
    fn nvim_for_all_tab_windows_start() -> *mut c_void;
    fn nvim_for_all_tab_windows_next(handle: *mut c_void) -> *mut c_void;
    #[link_name = "xfree"]
    fn nvim_for_all_tab_windows_end(handle: *mut c_void);

    // --- win / buf accessor ---
    fn nvim_win_get_buffer(wp: *mut c_void) -> *mut c_void;
    fn nvim_get_curwin() -> *mut c_void;

    // --- do_modelines ---
    fn nvim_do_modelines();

    // --- error messages ---
    fn nvim_semsg_reload_fail(fname: *const c_char);
    fn nvim_semsg_prep_reload_fail(fname: *const c_char);

    // --- xstrdup / xfree ---
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn xfree(ptr: *mut c_void);
}

// Read flags (from fileio.h)
const READ_NEW: c_int = 0x01;
const READ_KEEP_UNDO: c_int = 0x08;

// Buffer list flags (from buffer.h)
const BLN_DUMMY: c_int = 4;

// Buffer flags (from buffer_defs.h)
const BF_CHECK_RO: c_int = 0x02;

// Return values
const OK: c_int = 1;
const FAIL: c_int = 0;

/// Move all the lines from buffer `frombuf` to buffer `tobuf`.
///
/// Returns OK or FAIL. When FAIL, `tobuf` is incomplete and/or `frombuf`
/// is not empty.
///
/// Replaces the C static `move_lines` function.
///
/// # Safety
/// Both `frombuf` and `tobuf` must be valid non-null buffer pointers.
unsafe fn move_lines(frombuf: *mut c_void, tobuf: *mut c_void) -> c_int {
    let original_curbuf = nvim_get_curbuf();
    let mut retval = OK;
    let line_count = bref_void(frombuf as *const c_void).ml_line_count;

    // Copy lines from frombuf to tobuf.
    nvim_set_curbuf(tobuf);
    for lnum in 1..=line_count {
        let p = nvim_ml_get_buf(frombuf, lnum);
        if p.is_null() {
            retval = FAIL;
            break;
        }
        let copy = xstrdup(p);
        if nvim_ml_append_curbuf(lnum - 1, copy) == FAIL {
            xfree(copy as *mut c_void);
            retval = FAIL;
            break;
        }
        xfree(copy as *mut c_void);
    }

    // Delete all lines in frombuf if copy succeeded.
    if retval != FAIL {
        nvim_set_curbuf(frombuf);
        let frombuf_count = nvim_get_curbuf_ml_line_count();
        let mut lnum = frombuf_count;
        while lnum > 0 {
            if nvim_ml_delete_in_buf(frombuf, lnum) == FAIL {
                retval = FAIL;
                break;
            }
            lnum -= 1;
        }
    }

    nvim_set_curbuf(original_curbuf);
    retval
}

/// Reload a buffer that is already loaded.
///
/// Used when the file was changed outside of Vim.
/// `orig_mode` is `buf->b_orig_mode` before the need for reloading was detected.
///
/// Replaces the C `buf_reload` function.
///
/// # Safety
/// `buf` must be a valid non-null buffer pointer.
#[export_name = "buf_reload"]
pub unsafe extern "C" fn rs_buf_reload(buf: *mut c_void, orig_mode: c_int, reload_options: c_int) {
    let old_ro = bref_void(buf as *const c_void).b_p_ro;

    // Set curwin/curbuf for "buf" and save some things.
    let aco = nvim_aucmd_prepbuf_alloc(buf);

    // Allocate exarg_T.
    let ea = nvim_exarg_alloc_clear();

    if reload_options == 0 {
        // Force the fileformat and encoding to be the same.
        prep_exarg(ea, buf);
    }
    // If reload_options != 0, ea is already zero-initialised (CLEAR_FIELD equivalent).

    // Save cursor and topline.
    let mut old_cursor_lnum: c_int = 0;
    let mut old_cursor_col: c_int = 0;
    nvim_curwin_get_cursor(&mut old_cursor_lnum, &mut old_cursor_col);
    let old_topline =
        nvim_window::win_struct::win_ref(nvim_window::WinHandle::from_ptr(nvim_get_curwin()))
            .w_topline;

    let curbuf = nvim_get_curbuf();
    let ml_line_count = bref_void(curbuf as *const c_void).ml_line_count;

    let mut flags = READ_NEW;
    let mut saved = OK;

    if nvim_get_p_ur() < 0 || ml_line_count <= nvim_get_p_ur() {
        // Save all the text so the reload can be undone.
        saved = nvim_u_savecommon_reload_ok(curbuf);
        if saved == OK {
            flags |= READ_KEEP_UNDO;
        }
    }

    // Use a hidden buffer to save the old contents in case readfile fails.
    let mut savebuf: *mut c_void = std::ptr::null_mut();
    let mut bufref: [u64; 2] = [0; 2];
    let bufref_ptr = bufref.as_mut_ptr() as *mut c_void;

    if nvim_buf_is_empty(curbuf) == 0 && saved != FAIL {
        // Allocate a buffer without putting it in the buffer list.
        savebuf = nvim_buflist_new(std::ptr::null(), std::ptr::null(), 1, BLN_DUMMY);
        if !savebuf.is_null() {
            nvim_set_bufref(bufref_ptr, savebuf);
        }

        if !savebuf.is_null() && buf == curbuf {
            // Open the memline for savebuf.
            let prev_curbuf = nvim_get_curbuf();
            nvim_set_curbuf(savebuf);
            nvim_curwin_set_buffer2(savebuf);
            let ml_ok = nvim_ml_open_buf(savebuf);
            nvim_set_curbuf(prev_curbuf);
            nvim_curwin_set_buffer2(prev_curbuf);
            if ml_ok == FAIL {
                saved = FAIL;
            }
        }

        if savebuf.is_null() || saved == FAIL || buf != curbuf || move_lines(buf, savebuf) == FAIL {
            let b_fname = bref_void(buf as *const c_void).b_fname;
            nvim_semsg_prep_reload_fail(b_fname);
            saved = FAIL;
        }
    }

    let curbuf = nvim_get_curbuf();
    if saved == OK {
        // BF_CHECK_RO: check for RO again.
        nvim_curbuf_set_b_flags_or(BF_CHECK_RO);
        nvim_curbuf_set_b_keep_filetype(1);

        let silent = nvim_shortmess_fileinfo();
        let readfile_ok = nvim_readfile_reload(buf, ea, flags, silent);

        if readfile_ok != OK {
            if nvim_aborting() == 0 {
                let b_fname = bref_void(buf as *const c_void).b_fname;
                nvim_semsg_reload_fail(b_fname);
            }
            if !savebuf.is_null() && nvim_bufref_valid(bufref_ptr) != 0 && buf == curbuf {
                // Restore old contents: delete what readfile added, then move lines back.
                loop {
                    let lnum = nvim_get_curbuf_ml_line_count();
                    if nvim_buf_is_empty(curbuf) != 0 {
                        break;
                    }
                    if nvim_ml_delete_in_buf(curbuf, lnum) == FAIL {
                        break;
                    }
                }
                move_lines(savebuf, buf);
            }
        } else if buf == curbuf {
            // Mark as unmodified and manage undo info.
            nvim_unchanged(buf);
            if (flags & READ_KEEP_UNDO) == 0 {
                nvim_u_clearallandblockfree(buf);
            } else {
                nvim_u_unchanged(curbuf);
            }
            nvim_buf_updates_unload(curbuf);
            nvim_curbuf_set_b_mod_set(1);
        }
    }

    nvim_exarg_free(ea); // frees ea->cmd (from rs_prep_exarg) then ea itself

    if !savebuf.is_null() && nvim_bufref_valid(bufref_ptr) != 0 {
        nvim_wipe_buffer(savebuf);
    }

    // Invalidate diff info.
    let curbuf = nvim_get_curbuf();
    rs_diff_invalidate(curbuf);

    // Restore topline and cursor.
    nvim_curwin_set_topline_clamped(old_topline);
    nvim_curwin_set_cursor(old_cursor_lnum, old_cursor_col);
    nvim_check_cursor_curwin();
    nvim_update_topline_curwin();
    nvim_curbuf_set_b_keep_filetype(0);

    // Update folds for all tab windows.
    let curwin = nvim_get_curwin();
    let iter = nvim_for_all_tab_windows_start();
    loop {
        let wp = nvim_for_all_tab_windows_next(iter);
        if wp.is_null() {
            break;
        }
        if nvim_win_get_buffer(wp) == nvim_win_get_buffer(curwin) && rs_foldmethodIsManual(wp) == 0
        {
            rs_foldUpdateAll(wp);
        }
    }
    nvim_for_all_tab_windows_end(iter);

    // If mode didn't change and 'readonly' was set, keep old value.
    if orig_mode == bref_void(curbuf as *const c_void).b_orig_mode {
        buf_mut_void(curbuf).b_p_ro |= old_ro;
    }

    // Modelines must override settings done by autocommands.
    nvim_do_modelines();

    // Restore curwin/curbuf.
    nvim_aucmd_restbuf_free(aco);
    // Careful: autocommands may have made "buf" invalid!
    // (undo file writing is done by the caller, rs_buf_check_timestamp)
}
