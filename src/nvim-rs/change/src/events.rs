//! Change event notifications for line and byte changes.
//!
//! This module provides functions that notify the system about changes to
//! buffer contents at the line and byte level.

use std::ffi::{c_int, c_void};

use crate::{BufHandle, ColnrT, LinenrT, WinHandle, KEXTMARK_UNDO};

// =============================================================================
// C Accessor Functions (extern declarations)
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Buffer field accessors
    fn nvim_buf_get_b_mod_set(buf: BufHandle) -> bool;
    fn nvim_buf_set_b_mod_set(buf: BufHandle, val: bool);
    fn nvim_buf_get_b_mod_top(buf: BufHandle) -> LinenrT;
    fn nvim_buf_set_b_mod_top(buf: BufHandle, val: LinenrT);
    fn nvim_buf_get_b_mod_bot(buf: BufHandle) -> LinenrT;
    fn nvim_buf_set_b_mod_bot(buf: BufHandle, val: LinenrT);
    fn nvim_buf_get_b_mod_xlines(buf: BufHandle) -> LinenrT;
    fn nvim_buf_set_b_mod_xlines(buf: BufHandle, val: LinenrT);
    fn nvim_buf_get_b_ml_ml_line_count(buf: BufHandle) -> LinenrT;

    // Global state
    fn nvim_get_curbuf() -> BufHandle;
    fn nvim_get_curwin() -> WinHandle;
    fn nvim_get_curbuf_splice_pending() -> c_int;

    // Window accessors
    fn nvim_win_get_buffer(win: WinHandle) -> BufHandle;
    fn nvim_win_get_p_diff(win: WinHandle) -> c_int;

    // Marktree accessors
    fn nvim_buf_marktree_n_keys(buf: BufHandle) -> c_int;
    fn nvim_buf_meta_total(buf: BufHandle, meta_type: c_int) -> c_int;

    // Buffer updates
    fn nvim_buf_updates_send_changes(buf: BufHandle, lnum: LinenrT, added: i64, removed: i64);

    // Extmark operations
    fn nvim_extmark_splice_cols(
        buf: BufHandle,
        lnum: c_int,
        col: ColnrT,
        old_col: c_int,
        new_col: c_int,
        op: c_int,
    );

    // Mark adjustment
    fn nvim_mark_adjust(
        lnum: LinenrT,
        lnume: LinenrT,
        amount: LinenrT,
        amount_after: LinenrT,
        op: c_int,
    );
    fn nvim_extmark_adjust(
        buf: BufHandle,
        lnum: LinenrT,
        lnume: LinenrT,
        amount: LinenrT,
        amount_after: LinenrT,
        op: c_int,
    );

    // Spell checking
    fn nvim_spell_check_window(win: WinHandle) -> bool;
    fn nvim_redrawWinline(win: WinHandle, lnum: LinenrT);

    // Option checking
    fn nvim_vim_strchr_cpo_dollar() -> bool;

    // Diff functions
    fn nvim_diff_internal() -> bool;
    fn nvim_diff_lnum_win(lnum: LinenrT, win: WinHandle) -> LinenrT;

    // Redraw functions
    fn nvim_redraw_later(win: WinHandle, rtype: c_int);

    // Window iteration
    fn nvim_get_curtab() -> *mut c_void;
    fn nvim_curtab_first_win() -> WinHandle;
    fn nvim_win_get_next_in_tab(win: WinHandle) -> WinHandle;

    // Changed function from recording module
    fn rs_changed(buf: BufHandle);

    // Changed common helper (we'll call into C for this complex function)
    fn changed_common(buf: BufHandle, lnum: LinenrT, col: ColnrT, lnume: LinenrT, xtra: LinenrT);
}

/// Update redraw type constant (from drawscreen.h).
const UPD_VALID: c_int = 10;

/// Meta type for virtual lines.
const KMT_META_LINES: c_int = 1;

/// Extmark no-op operation type.
const KEXTMARK_NOOP: c_int = 2;

/// MAXLNUM constant (maximum line number).
const MAXLNUM: LinenrT = 0x7FFF_FFFF;

// =============================================================================
// Redraw Area Tracking
// =============================================================================

/// Marks the area to be redrawn after a change.
///
/// Consider also calling changed_lines_invalidate_buf().
///
/// # Arguments
/// * `buf` - the buffer where lines were changed
/// * `lnum` - first line with change
/// * `lnume` - line below last changed line
/// * `xtra` - number of extra lines (negative when deleting)
fn changed_lines_redraw_buf_impl(buf: BufHandle, lnum: LinenrT, lnume: LinenrT, xtra: LinenrT) {
    // SAFETY: All accessors are safe C functions
    unsafe {
        let mut lnume_adj = lnume;

        // If lines have been deleted and there may be decorations in the buffer, ensure
        // win_update() calculates the height of, and redraws the line to which or whence
        // from its mark may have moved. When lines are deleted, a virt_line mark may
        // have moved be drawn two lines below so increase by one more.
        if xtra != 0 && nvim_buf_marktree_n_keys(buf) > 0 {
            let meta_lines = if xtra < 0 && nvim_buf_meta_total(buf, KMT_META_LINES) != 0 {
                1
            } else {
                0
            };
            lnume_adj += 1 + meta_lines;
        }

        if nvim_buf_get_b_mod_set(buf) {
            // find the maximum area that must be redisplayed
            let cur_mod_top = nvim_buf_get_b_mod_top(buf);
            if lnum < cur_mod_top {
                nvim_buf_set_b_mod_top(buf, lnum);
            }

            let cur_mod_bot = nvim_buf_get_b_mod_bot(buf);
            if lnum < cur_mod_bot {
                // adjust old bot position for xtra lines
                let mut new_bot = cur_mod_bot + xtra;
                if new_bot < lnum {
                    new_bot = lnum;
                }
                nvim_buf_set_b_mod_bot(buf, new_bot);
            }

            let cur_mod_bot = nvim_buf_get_b_mod_bot(buf);
            if lnume_adj + xtra > cur_mod_bot {
                nvim_buf_set_b_mod_bot(buf, lnume_adj + xtra);
            }

            let cur_mod_xlines = nvim_buf_get_b_mod_xlines(buf);
            nvim_buf_set_b_mod_xlines(buf, cur_mod_xlines + xtra);
        } else {
            // set the area that must be redisplayed
            nvim_buf_set_b_mod_set(buf, true);
            nvim_buf_set_b_mod_top(buf, lnum);
            nvim_buf_set_b_mod_bot(buf, lnume_adj + xtra);
            nvim_buf_set_b_mod_xlines(buf, xtra);
        }
    }
}

/// FFI wrapper for `changed_lines_redraw_buf`.
#[no_mangle]
pub extern "C" fn rs_changed_lines_redraw_buf(
    buf: BufHandle,
    lnum: LinenrT,
    lnume: LinenrT,
    xtra: LinenrT,
) {
    changed_lines_redraw_buf_impl(buf, lnum, lnume, xtra);
}

// =============================================================================
// Change Event Functions
// =============================================================================

/// Changed bytes within a single line for the current buffer.
///
/// - marks the windows on this buffer to be redisplayed
/// - marks the buffer changed by calling changed()
/// - invalidates cached values
///
/// Careful: may trigger autocommands that reload the buffer.
fn changed_bytes_impl(lnum: LinenrT, col: ColnrT) {
    // SAFETY: All accessors are safe C functions
    unsafe {
        let curbuf = nvim_get_curbuf();
        let curwin = nvim_get_curwin();

        changed_lines_redraw_buf_impl(curbuf, lnum, lnum + 1, 0);
        changed_common(curbuf, lnum, col, lnum + 1, 0);

        // When text has been changed at the end of the line, possibly the start of
        // the next line may have SpellCap that should be removed or it needs to be
        // displayed.  Schedule the next line for redrawing just in case.
        // Don't do this when displaying '$' at the end of changed text.
        if nvim_spell_check_window(curwin)
            && lnum < nvim_buf_get_b_ml_ml_line_count(curbuf)
            && !nvim_vim_strchr_cpo_dollar()
        {
            nvim_redrawWinline(curwin, lnum + 1);
        }

        // notify any channels that are watching
        nvim_buf_updates_send_changes(curbuf, lnum, 1, 1);

        // Diff highlighting in other diff windows may need to be updated too.
        if nvim_win_get_p_diff(curwin) != 0 {
            let mut wp = nvim_curtab_first_win();
            while !wp.is_null() {
                if nvim_win_get_p_diff(wp) != 0 && wp != curwin {
                    nvim_redraw_later(wp, UPD_VALID);
                    let wlnum = nvim_diff_lnum_win(lnum, wp);
                    if wlnum > 0 {
                        let wp_buf = nvim_win_get_buffer(wp);
                        changed_lines_redraw_buf_impl(wp_buf, wlnum, wlnum + 1, 0);
                    }
                }
                wp = nvim_win_get_next_in_tab(wp);
            }
        }
    }
}

/// FFI wrapper for `changed_bytes`.
#[no_mangle]
pub extern "C" fn rs_changed_bytes(lnum: LinenrT, col: ColnrT) {
    changed_bytes_impl(lnum, col);
}

/// Insert/delete bytes at column.
///
/// Like changed_bytes() but also adjust extmark for "new" bytes.
fn inserted_bytes_impl(lnum: LinenrT, start_col: ColnrT, old_col: c_int, new_col: c_int) {
    // SAFETY: All accessors are safe C functions
    unsafe {
        if nvim_get_curbuf_splice_pending() == 0 {
            let curbuf = nvim_get_curbuf();
            nvim_extmark_splice_cols(
                curbuf,
                lnum as c_int - 1,
                start_col,
                old_col,
                new_col,
                KEXTMARK_UNDO,
            );
        }

        changed_bytes_impl(lnum, start_col);
    }
}

/// FFI wrapper for `inserted_bytes`.
#[no_mangle]
pub extern "C" fn rs_inserted_bytes(
    lnum: LinenrT,
    start_col: ColnrT,
    old_col: c_int,
    new_col: c_int,
) {
    inserted_bytes_impl(lnum, start_col, old_col, new_col);
}

/// Changed lines for a buffer.
///
/// Must be called AFTER the change and after mark_adjust().
/// - mark the buffer changed by calling changed()
/// - mark the windows on this buffer to be redisplayed
/// - invalidate cached values
///
/// "lnum" is the first line that needs displaying, "lnume" the first line
/// below the changed lines (BEFORE the change).
/// When only inserting lines, "lnum" and "lnume" are equal.
/// Takes care of calling changed() and updating b_mod_*.
///
/// Careful: may trigger autocommands that reload the buffer.
fn changed_lines_impl(
    buf: BufHandle,
    lnum: LinenrT,
    col: ColnrT,
    lnume: LinenrT,
    xtra: LinenrT,
    do_buf_event: bool,
) {
    // SAFETY: All accessors are safe C functions
    unsafe {
        changed_lines_redraw_buf_impl(buf, lnum, lnume, xtra);

        let curwin = nvim_get_curwin();
        if xtra == 0
            && nvim_win_get_p_diff(curwin) != 0
            && nvim_win_get_buffer(curwin) == buf
            && !nvim_diff_internal()
        {
            // When the number of lines doesn't change then mark_adjust() isn't
            // called and other diff buffers still need to be marked for
            // displaying.
            let mut wp = nvim_curtab_first_win();
            while !wp.is_null() {
                if nvim_win_get_p_diff(wp) != 0 && wp != curwin {
                    nvim_redraw_later(wp, UPD_VALID);
                    let wlnum = nvim_diff_lnum_win(lnum, wp);
                    if wlnum > 0 {
                        let wp_buf = nvim_win_get_buffer(wp);
                        changed_lines_redraw_buf_impl(wp_buf, wlnum, lnume - lnum + wlnum, 0);
                    }
                }
                wp = nvim_win_get_next_in_tab(wp);
            }
        }

        changed_common(buf, lnum, col, lnume, xtra);

        if do_buf_event {
            let num_added = lnume + xtra - lnum;
            let num_removed = lnume - lnum;
            nvim_buf_updates_send_changes(buf, lnum, num_added.into(), num_removed.into());
        }
    }
}

/// FFI wrapper for `changed_lines`.
#[no_mangle]
pub extern "C" fn rs_changed_lines(
    buf: BufHandle,
    lnum: LinenrT,
    col: ColnrT,
    lnume: LinenrT,
    xtra: LinenrT,
    do_buf_event: bool,
) {
    changed_lines_impl(buf, lnum, col, lnume, xtra, do_buf_event);
}

/// Appended "count" lines below line "lnum" in the given buffer.
///
/// Must be called AFTER the change and after mark_adjust().
/// Takes care of marking the buffer to be redrawn and sets the changed flag.
fn appended_lines_buf_impl(buf: BufHandle, lnum: LinenrT, count: LinenrT) {
    changed_lines_impl(buf, lnum + 1, 0, lnum + 1, count, true);
}

/// FFI wrapper for `appended_lines_buf`.
#[no_mangle]
pub extern "C" fn rs_appended_lines_buf(buf: BufHandle, lnum: LinenrT, count: LinenrT) {
    appended_lines_buf_impl(buf, lnum, count);
}

/// Appended "count" lines below line "lnum" in the current buffer.
///
/// Must be called AFTER the change and after mark_adjust().
/// Takes care of marking the buffer to be redrawn and sets the changed flag.
fn appended_lines_impl(lnum: LinenrT, count: LinenrT) {
    // SAFETY: nvim_get_curbuf is a safe accessor
    unsafe {
        appended_lines_buf_impl(nvim_get_curbuf(), lnum, count);
    }
}

/// FFI wrapper for `appended_lines`.
#[no_mangle]
pub extern "C" fn rs_appended_lines(lnum: LinenrT, count: LinenrT) {
    appended_lines_impl(lnum, count);
}

/// Like appended_lines(), but adjust marks first.
fn appended_lines_mark_impl(lnum: LinenrT, count: c_int) {
    // SAFETY: All accessors are safe C functions
    unsafe {
        let curbuf = nvim_get_curbuf();
        nvim_mark_adjust(lnum + 1, MAXLNUM, count as LinenrT, 0, KEXTMARK_UNDO);
        changed_lines_impl(curbuf, lnum + 1, 0, lnum + 1, count as LinenrT, true);
    }
}

/// FFI wrapper for `appended_lines_mark`.
#[no_mangle]
pub extern "C" fn rs_appended_lines_mark(lnum: LinenrT, count: c_int) {
    appended_lines_mark_impl(lnum, count);
}

/// Deleted "count" lines at line "lnum" in the given buffer.
///
/// Must be called AFTER the change and after mark_adjust().
/// Takes care of marking the buffer to be redrawn and sets the changed flag.
fn deleted_lines_buf_impl(buf: BufHandle, lnum: LinenrT, count: LinenrT) {
    changed_lines_impl(buf, lnum, 0, lnum + count, -count, true);
}

/// FFI wrapper for `deleted_lines_buf`.
#[no_mangle]
pub extern "C" fn rs_deleted_lines_buf(buf: BufHandle, lnum: LinenrT, count: LinenrT) {
    deleted_lines_buf_impl(buf, lnum, count);
}

/// Deleted "count" lines at line "lnum" in the current buffer.
///
/// Must be called AFTER the change and after mark_adjust().
/// Takes care of marking the buffer to be redrawn and sets the changed flag.
fn deleted_lines_impl(lnum: LinenrT, count: LinenrT) {
    // SAFETY: nvim_get_curbuf is a safe accessor
    unsafe {
        deleted_lines_buf_impl(nvim_get_curbuf(), lnum, count);
    }
}

/// FFI wrapper for `deleted_lines`.
#[no_mangle]
pub extern "C" fn rs_deleted_lines(lnum: LinenrT, count: LinenrT) {
    deleted_lines_impl(lnum, count);
}

/// Like deleted_lines(), but adjust marks first.
///
/// Make sure the cursor is on a valid line before calling, a GUI callback may
/// be triggered to display the cursor.
fn deleted_lines_mark_impl(lnum: LinenrT, count: c_int) {
    // SAFETY: All accessors are safe C functions
    unsafe {
        let curbuf = nvim_get_curbuf();
        let ml_empty = nvim_buf_get_ml_empty(curbuf);
        let made_empty = count > 0 && ml_empty;

        nvim_mark_adjust(
            lnum,
            lnum + count as LinenrT - 1,
            MAXLNUM,
            -(count as LinenrT),
            KEXTMARK_NOOP,
        );

        // if we deleted the entire buffer, we need to implicitly add a new empty line
        let xtra_for_extmark = if made_empty { 1 } else { 0 };
        nvim_extmark_adjust(
            curbuf,
            lnum,
            lnum + count as LinenrT - 1,
            MAXLNUM,
            -(count as LinenrT) + xtra_for_extmark,
            KEXTMARK_UNDO,
        );
        changed_lines_impl(
            curbuf,
            lnum,
            0,
            lnum + count as LinenrT,
            -(count as LinenrT),
            true,
        );
    }
}

extern "C" {
    fn nvim_buf_get_ml_empty(buf: BufHandle) -> bool;
}

/// FFI wrapper for `deleted_lines_mark`.
#[no_mangle]
pub extern "C" fn rs_deleted_lines_mark(lnum: LinenrT, count: c_int) {
    deleted_lines_mark_impl(lnum, count);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(UPD_VALID, 10);
        assert_eq!(KMT_META_LINES, 1);
    }
}
