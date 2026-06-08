//! Buffer close and free operations.
//!
//! Implements `buf_freeall`.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(dead_code)]

use nvim_window::win_struct::WinStruct;
use std::ffi::{c_char, c_int, c_void};

use crate::{
    buf_struct::{buf_mut, buf_ref},
    BufHandle,
};

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
// Event constants (from auevents_enum.generated.h)
// =============================================================================

const EVENT_BUFWINLEAVE: c_int = 17;
const EVENT_BUFHIDDEN: c_int = 6;

// =============================================================================
// Buffer flags (from buffer_defs.h)
// =============================================================================

const BF_CHECK_RO: c_int = 0x02;
const BF_NEVERLOADED: c_int = 0x04;

// free_buffer_stuff flags (from buffer.h kBff* enum)
// kBffClearWinInfo = 1, kBffInitChangedtick = 2

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    fn nvim_get_curbuf() -> BufHandle;
    fn nvim_get_curwin() -> *mut c_void;
    fn nvim_get_curtab() -> *mut c_void;

    fn set_bufref(bufref: *mut BufRef, buf: BufHandle);
    #[link_name = "rs_bufref_valid"]
    fn nvim_bufref_valid(bufref: *const BufRef) -> c_int;

    fn buf_updates_unload(buf: BufHandle, can_reload: bool);
    fn apply_autocmds(
        event: c_int,
        fname: *const c_char,
        fname_io: *const c_char,
        force: bool,
        buf: BufHandle,
    ) -> bool;

    fn rs_win_valid_any_tab(win: *mut c_void) -> c_int;
    fn rs_one_window_in_tab(win: *mut c_void, tp: *mut c_void) -> c_int;
    fn nvim_win_get_buffer(win: *mut c_void) -> BufHandle;
    fn goto_tabpage_win(tab: *mut c_void, win: *mut c_void);

    #[link_name = "block_autocmds"]
    fn nvim_block_autocmds();
    #[link_name = "unblock_autocmds"]
    fn nvim_unblock_autocmds();
    fn aborting() -> c_int;

    fn rs_diff_buf_delete(buf: BufHandle);
    fn rs_diffopt_hiddenoff() -> c_int;
    fn reset_synblock(win: *mut c_void);
    fn nvim_buf_clearFolding_all_windows(buf: BufHandle);
    fn ml_close(buf: BufHandle, del_file: bool);
    fn u_clearallandblockfree(buf: BufHandle);
    fn nvim_syntax_clear_buf(buf: BufHandle);

    // Phase 4: close_buffer helpers
    fn nvim_emsg_auabort();
    fn nvim_buflist_setfpos_win(buf: BufHandle, win: *mut c_void);
    fn nvim_terminal_close_buf(buf: BufHandle);
    fn nvim_get_VIsual_active() -> c_int;
    fn nvim_get_entered_free_all_mem() -> c_int;
    fn end_visual_mode();
    fn nvim_mark_forget_file_all_tabs(fnum: c_int);
    fn nvim_buf_wipe_free(buf: BufHandle);
    fn nvim_buf_free_stuff_del(buf: BufHandle);
    fn set_last_cursor(win: *mut c_void);
    fn can_unload_buffer(buf: BufHandle) -> bool;

    fn rs_buf_effective_action(buf: BufHandle, action: c_int) -> c_int;
    fn buf_freeall(buf: BufHandle, flags: c_int);
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
    buf_mut(buf).b_locked += 1;
    buf_mut(buf).b_locked_split += 1;

    let mut bufref = BufRef::zeroed();
    set_bufref(&raw mut bufref, buf);

    buf_updates_unload(buf, false);
    if nvim_bufref_valid(&raw const bufref) == 0 {
        // on_detach callback deleted the buffer.
        return;
    }

    if !buf_ref(buf).ml_mfp_is_null()
        && apply_autocmds(
            EVENT_BUFUNLOAD,
            buf_ref(buf).b_fname,
            buf_ref(buf).b_fname,
            false,
            buf,
        )
        && nvim_bufref_valid(&raw const bufref) == 0
    {
        // Autocommands deleted the buffer.
        return;
    }

    if (flags & BFA_DEL) != 0
        && buf_ref(buf).b_p_bl != 0
        && apply_autocmds(
            EVENT_BUFDELETE,
            buf_ref(buf).b_fname,
            buf_ref(buf).b_fname,
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
            buf_ref(buf).b_fname,
            buf_ref(buf).b_fname,
            false,
            buf,
        )
        && nvim_bufref_valid(&raw const bufref) == 0
    {
        // Autocommands may delete the buffer.
        return;
    }

    buf_mut(buf).b_locked -= 1;
    buf_mut(buf).b_locked_split -= 1;

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
    if (flags & BFA_IGNORE_ABORT) == 0 && aborting() != 0 {
        return;
    }

    // It's possible that autocommands change curbuf to the one being deleted.
    // Only return if curbuf changed to the deleted buffer.
    if buf == nvim_get_curbuf() && !is_curbuf {
        return;
    }

    rs_diff_buf_delete(buf); // Can't use 'diff' for unloaded buffer.
    let curwin_for_synblock = nvim_get_curwin();
    if !curwin_for_synblock.is_null() && nvim_win_get_buffer(curwin_for_synblock) == buf {
        reset_synblock(curwin_for_synblock);
    }
    nvim_buf_clearFolding_all_windows(buf);

    ml_close(buf, true); // close and delete the memline/memfile
    buf_mut(buf).ml_line_count = 0; // no lines in buffer

    if (flags & BFA_KEEP_UNDO) == 0 {
        u_clearallandblockfree(buf);
    }

    nvim_syntax_clear_buf(buf); // reset syntax info
    buf_mut(buf).b_flags &= !BF_READERR; // a read error is no longer relevant
}

// =============================================================================
// close_buffer
// =============================================================================

/// Close the link to a buffer.
///
/// - `win`: If not NULL, set `b_last_cursor`.
/// - `action`: What to do when there is no longer a window for the buffer:
///   - 0 (`DOBUF_GOTO`):   buffer becomes hidden
///   - 2 (`DOBUF_UNLOAD`): buffer is unloaded
///   - 3 (`DOBUF_DEL`):    buffer is unloaded and removed from buffer list
///   - 4 (`DOBUF_WIPE`):   buffer is unloaded and really deleted
///     Constants match `buffer.h`; see `_Static_assert(DOBUF_WIPE == 4)` in `buffer.c`.
///
///   The `bufhidden` option can force freeing and deleting.
/// - `abort_if_last`: If true, do not close the buffer if autocommands cause
///   there to be only one window with this buffer.
/// - `ignore_abort`: If true, don't abort even when `aborting()` returns true.
///
/// Returns true if `b_nwindows` was decremented directly by this call.
///
/// # Safety
///
/// Must be called on the main thread with valid Neovim state.
#[allow(clippy::too_many_lines)]
#[unsafe(export_name = "close_buffer")]
pub unsafe extern "C" fn rs_close_buffer(
    win: *mut c_void,
    buf: BufHandle,
    action: c_int,
    abort_if_last: bool,
    ignore_abort: bool,
) -> bool {
    let action = rs_buf_effective_action(buf, action);
    // DOBUF_* constants (from buffer.h): GOTO=0, SPLIT=1, UNLOAD=2, DEL=3, WIPE=4.
    // Verified by _Static_assert(DOBUF_WIPE == 4) in buffer.c.
    let unload_buf = action >= 2; // DOBUF_UNLOAD, DOBUF_DEL, or DOBUF_WIPE
    let del_buf = action == 3 || action == 4; // DOBUF_DEL || DOBUF_WIPE
    let wipe_buf = action == 4; // DOBUF_WIPE

    let the_curwin = nvim_get_curwin();
    let is_curwin = !the_curwin.is_null() && nvim_win_get_buffer(the_curwin) == buf;
    let the_curtab = nvim_get_curtab();

    // Disallow deleting the buffer when it is locked (already being closed or
    // halfway a command that relies on it). Unloading is allowed.
    if (del_buf || wipe_buf) && !can_unload_buffer(buf) {
        return false;
    }

    // Check no autocommands closed the window
    if !win.is_null() && rs_win_valid_any_tab(win) != 0 {
        // Set b_last_cursor when closing the last window for the buffer.
        if buf_ref(buf).b_nwindows == 1 {
            set_last_cursor(win);
        }
        nvim_buflist_setfpos_win(buf, win);
    }

    let mut bufref = BufRef::zeroed();
    set_bufref(&raw mut bufref, buf);

    // When the buffer is no longer in a window, trigger BufWinLeave
    if buf_ref(buf).b_nwindows == 1 {
        buf_mut(buf).b_locked += 1;
        buf_mut(buf).b_locked_split += 1;
        if apply_autocmds(
            EVENT_BUFWINLEAVE,
            buf_ref(buf).b_fname,
            buf_ref(buf).b_fname,
            false,
            buf,
        ) && nvim_bufref_valid(&raw const bufref) == 0
        {
            // Autocommands deleted the buffer.
            nvim_emsg_auabort();
            return false;
        }
        buf_mut(buf).b_locked -= 1;
        buf_mut(buf).b_locked_split -= 1;
        if abort_if_last && !win.is_null() && rs_one_window_in_tab(win, std::ptr::null_mut()) != 0 {
            // Autocommands made this the only window.
            nvim_emsg_auabort();
            return false;
        }

        // When the buffer becomes hidden, but is not unloaded, trigger BufHidden
        if !unload_buf {
            buf_mut(buf).b_locked += 1;
            buf_mut(buf).b_locked_split += 1;
            if apply_autocmds(
                EVENT_BUFHIDDEN,
                buf_ref(buf).b_fname,
                buf_ref(buf).b_fname,
                false,
                buf,
            ) && nvim_bufref_valid(&raw const bufref) == 0
            {
                // Autocommands deleted the buffer.
                nvim_emsg_auabort();
                return false;
            }
            buf_mut(buf).b_locked -= 1;
            buf_mut(buf).b_locked_split -= 1;
            if abort_if_last
                && !win.is_null()
                && rs_one_window_in_tab(win, std::ptr::null_mut()) != 0
            {
                // Autocommands made this the only window.
                nvim_emsg_auabort();
                return false;
            }
        }
        // autocmds may abort script processing
        if !ignore_abort && aborting() != 0 {
            return false;
        }
    }

    // If the buffer was in curwin and the window has changed, go back to that
    // window, if it still exists.
    if is_curwin && nvim_get_curwin() != the_curwin && rs_win_valid_any_tab(the_curwin) != 0 {
        nvim_block_autocmds();
        goto_tabpage_win(the_curtab, the_curwin);
        nvim_unblock_autocmds();
    }

    let nwindows = buf_ref(buf).b_nwindows;

    // Decrease the link count from windows (unless not in any window)
    if buf_ref(buf).b_nwindows > 0 {
        buf_mut(buf).b_nwindows -= 1;
    }

    if rs_diffopt_hiddenoff() != 0 && !unload_buf && buf_ref(buf).b_nwindows == 0 {
        rs_diff_buf_delete(buf); // Clear 'diff' for hidden buffer.
    }

    // Return when a window is displaying the buffer or when it's not unloaded.
    if buf_ref(buf).b_nwindows > 0 || !unload_buf {
        return true;
    }

    nvim_terminal_close_buf(buf);

    // Always remove the buffer when there is no file name.
    let del_buf = del_buf || unsafe { buf_ref(buf).b_ffname.is_null() };

    // Free all things allocated for this buffer.
    let is_curbuf = buf == nvim_get_curbuf();

    // When closing the current buffer stop Visual mode before freeing anything.
    if is_curbuf && nvim_get_VIsual_active() != 0 && nvim_get_entered_free_all_mem() == 0 {
        end_visual_mode();
    }

    buf_mut(buf).b_nwindows = nwindows;

    buf_freeall(
        buf,
        (if del_buf { BFA_DEL } else { 0 })
            + (if wipe_buf { BFA_WIPE } else { 0 })
            + (if ignore_abort { BFA_IGNORE_ABORT } else { 0 }),
    );

    if nvim_bufref_valid(&raw const bufref) == 0 {
        // Autocommands may have deleted the buffer.
        return false;
    }
    // autocmds may abort script processing.
    if !ignore_abort && aborting() != 0 {
        return false;
    }

    // It's possible that autocommands change curbuf to the one being deleted.
    if buf == nvim_get_curbuf() && !is_curbuf {
        return false;
    }

    let clear_w_buf =
        !win.is_null() && rs_win_valid_any_tab(win) != 0 && nvim_win_get_buffer(win) == buf;

    // Autocommands may have opened or closed windows for this buffer.
    // Decrement the count for the close we do here.
    if buf_ref(buf).b_nwindows > 0 {
        buf_mut(buf).b_nwindows -= 1;
    }

    // Remove the buffer from the list.
    if wipe_buf {
        if clear_w_buf {
            (*win.cast::<WinStruct>()).w_buffer = std::ptr::null_mut();
        }
        // Do not wipe out the buffer if it is used in a window.
        if buf_ref(buf).b_nwindows > 0 {
            return true;
        }
        nvim_mark_forget_file_all_tabs(buf_ref(buf).handle);
        nvim_buf_wipe_free(buf);
    } else {
        if del_buf {
            // Free all internal variables and reset option values, to make
            // ":bdel" compatible with Vim 5.7.
            nvim_buf_free_stuff_del(buf);

            // Make it look like a new buffer.
            buf_mut(buf).b_flags = BF_CHECK_RO | BF_NEVERLOADED;

            // Init the options when loaded again.
            buf_mut(buf).b_p_initialized = 0;
        }
        crate::state::rs_buf_clear_file(buf);
        if clear_w_buf {
            (*win.cast::<WinStruct>()).w_buffer = std::ptr::null_mut();
        }
        if del_buf {
            crate::buf_struct::buf_mut(buf).b_p_bl = 0;
        }
    }
    // NOTE: at this point "curbuf" may be invalid!
    true
}

// =============================================================================
// free_buffer cluster (Phase N migration)
// =============================================================================

extern "C" {
    fn nvim_free_buffer_c_parts(buf: BufHandle);
    fn nvim_free_buffer_stuff_c_parts(buf: BufHandle, free_flags: c_int);
    fn nvim_clear_wininfo_c(buf: BufHandle);
    /// Execute all `free_buf_options()` logic in C (batch shim).
    fn nvim_buf_do_free_options(buf: BufHandle, free_p_ff: bool);
    /// Execute the body of `buflist_new()` in C (batch shim).
    fn nvim_buflist_new_impl(
        ffname_arg: *mut c_char,
        sfname_arg: *mut c_char,
        lnum: c_int,
        flags: c_int,
    ) -> BufHandle;
}

// =============================================================================
// buflist_new (Phase 3 migration)
// =============================================================================

/// Add a file name to the buffer list.
///
/// Drop-in replacement for C `buflist_new`. Delegates to `nvim_buflist_new_impl`
/// in `buffer_shim.c`.
///
/// # Safety
/// Must be called on the main thread with valid Neovim state.
#[unsafe(export_name = "buflist_new")]
pub unsafe extern "C" fn rs_buflist_new(
    ffname_arg: *mut c_char,
    sfname_arg: *mut c_char,
    lnum: c_int,
    flags: c_int,
) -> BufHandle {
    nvim_buflist_new_impl(ffname_arg, sfname_arg, lnum, flags)
}

// =============================================================================
// free_buf_options (Phase 2 migration)
// =============================================================================

/// Free the memory for the options of a buffer.
///
/// If `free_p_ff` is true, also frees `fileformat`, `buftype`, and
/// `fileencoding` options.
///
/// Drop-in replacement for C `free_buf_options`.
///
/// # Safety
/// Must be called on the main thread with valid Neovim state.
#[unsafe(export_name = "free_buf_options")]
pub unsafe extern "C" fn rs_free_buf_options(buf: BufHandle, free_p_ff: bool) {
    if buf.is_null() {
        return;
    }
    nvim_buf_do_free_options(buf, free_p_ff);
}

// BufFreeFlags from buffer.h (kBff* enum)
const KBF_CLEAR_WIN_INFO: c_int = 1;
const KBF_INIT_CHANGEDTICK: c_int = 2;

/// Free a buffer structure and the things it contains related to the buffer
/// itself (not the file, that must have been done already).
///
/// Port of C `free_buffer`.
///
/// # Safety
/// Accesses global Neovim state. Must be called on the main thread.
#[unsafe(export_name = "free_buffer")]
pub unsafe extern "C" fn rs_free_buffer(buf: BufHandle) {
    nvim_free_buffer_c_parts(buf);
}

/// Free the `b_wininfo` list for buffer `buf`.
///
/// Port of C `clear_wininfo`.
///
/// # Safety
/// Accesses global Neovim state. Must be called on the main thread.
#[unsafe(export_name = "clear_wininfo")]
pub unsafe extern "C" fn rs_clear_wininfo(buf: BufHandle) {
    nvim_clear_wininfo_c(buf);
}

/// Free stuff in the buffer for `:bdel` and when wiping out the buffer.
///
/// Port of C `free_buffer_stuff`.
///
/// - `buf`: Buffer pointer
/// - `free_flags`: `BufFreeFlags` (`kBffClearWinInfo` | `kBffInitChangedtick`)
///
/// # Safety
/// Accesses global Neovim state. Must be called on the main thread.
#[unsafe(export_name = "free_buffer_stuff")]
pub unsafe extern "C" fn rs_free_buffer_stuff(buf: BufHandle, free_flags: c_int) {
    nvim_free_buffer_stuff_c_parts(buf, free_flags);
}
