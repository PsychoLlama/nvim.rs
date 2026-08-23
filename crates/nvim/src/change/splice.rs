//! Who has to be told that lines changed, and what they are told.
//!
//! One splice -- lines `lnum`..`lnume` replaced, `xtra` lines net -- reaches
//! five audiences, and this file is the fan-out:
//!
//! | audience | reached by |
//! | --- | --- |
//! | the redraw area (`b_mod_*`) | [`changed_lines_redraw_buf`] |
//! | the modified flag and `b:changedtick` | [`changed`] |
//! | the `'.` mark, the change list, folds, 'cursorline' | [`changed_common`] |
//! | every window's `w_lines` display cache | [`changed_lines_invalidate_win`] |
//! | the extmark tree, the buffer-update RPC and Lua callbacks | the callers |
//!
//! The last row is why this family has no cheap test:
//! `lua/buffer_updates_spec` is its real gate, not any key-sequence sweep.
//! Every `buf_updates_send_changes` argument here is an *event payload*, and
//! getting one wrong is invisible to the buffer text.
//!
//! [`changed_bytes`] is the one-line case, [`changed_lines`] the general one,
//! and `appended_lines`/`deleted_lines` the two that also move marks. The
//! `_buf` suffix means "a buffer that may not be the current one" and the
//! `_mark` suffix "adjust the marks first".

#![deny(unsafe_op_in_unsafe_fn)]

use crate::memline::MlFlags;
use core::ffi::{c_int, c_void};

use super::*;
use crate::ex_docmd::cmdmod_has;
use crate::option::cpo_has;
use crate::types::CpoFlag;

/// Every window in every tabpage.
///
/// The current tabpage keeps its window list in `firstwin` rather than in the
/// tabpage struct, which is why this is not a plain walk of `tp_firstwin`.
/// (`terminal.rs` and `shada.rs` carry their own copies; a shared one belongs
/// in `window.rs`, which no B15 slice owns.)
///
/// # Safety
/// The caller must not restructure the window lists while iterating.
unsafe fn all_windows() -> impl Iterator<Item = *mut win_T> {
    let mut tp = first_tabpage.get() as *mut tabpage_T;
    let mut wp: *mut win_T = ::core::ptr::null_mut();
    ::core::iter::from_fn(move || unsafe {
        while wp.is_null() {
            if tp.is_null() {
                return None;
            }
            wp = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            tp = (*tp).tp_next as *mut tabpage_T;
        }
        let found = wp;
        wp = (*found).w_next;
        Some(found)
    })
}

/// Every window of the *current* tabpage.
///
/// # Safety
/// The caller must not restructure the window list while iterating.
unsafe fn windows_in_curtab() -> impl Iterator<Item = *mut win_T> {
    let mut wp = firstwin.get();
    ::core::iter::from_fn(move || unsafe {
        let found = wp;
        if found.is_null() {
            return None;
        }
        wp = (*found).w_next;
        Some(found)
    })
}

/// Drop the cached display information one window holds about the lines a
/// change covered, and shift what is below it by `xtra`.
///
/// # Safety
/// `wp` must be a live window.
unsafe fn changed_lines_invalidate_win(
    wp: *mut win_T,
    lnum: linenr_T,
    col: colnr_T,
    mut lnume: linenr_T,
    xtra: linenr_T,
) {
    unsafe {
        if (*wp).w_cursor.lnum <= lnum {
            let i = find_wl_entry(wp, lnum);
            if i >= 0 && (*wp).w_cursor.lnum > (*(*wp).w_lines.offset(i as isize)).wl_lnum {
                changed_line_abv_curs_win(wp);
            }
        }
        if (*wp).w_cursor.lnum > lnum {
            changed_line_abv_curs_win(wp);
        } else if (*wp).w_cursor.lnum == lnum && (*wp).w_cursor.col >= col {
            changed_cline_bef_curs(wp);
        }
        if (*wp).w_botline >= lnum {
            if xtra < 0 {
                invalidate_botline_win(wp);
            } else {
                approximate_botline_win(wp);
            }
        }

        // Inline virtual text under 'wrap', and virtual lines, make the line
        // after the change part of it as far as the display cache goes.
        if xtra < 0
            && (*wp).w_onebuf_opt.wo_wrap != 0
            && buf_meta_total((*wp).w_buffer, kMTMetaInline) != 0
            || xtra != 0 && buf_meta_total((*wp).w_buffer, kMTMetaLines) != 0
        {
            lnume += 1;
        }

        for i in 0..(*wp).w_lines_valid {
            let wl = &mut *(*wp).w_lines.offset(i as isize);
            if !wl.wl_valid {
                continue;
            }
            if wl.wl_lnum >= lnum {
                // Index zero's wl_lnum is compared against w_topline, so it is
                // invalidated rather than shifted.
                if i == 0 || wl.wl_lnum < lnume {
                    wl.wl_valid = false; // inside the change
                } else if xtra != 0 {
                    wl.wl_lnum += xtra; // below the change
                    wl.wl_foldend += xtra;
                    wl.wl_lastlnum += xtra;
                }
            } else if wl.wl_lastlnum >= lnum {
                // The change is inside this run of folded or concealed lines.
                wl.wl_valid = false;
            }
        }
    }
}

/// [`changed_lines_invalidate_win`] for every window displaying `buf`.
///
/// # Safety
/// `buf` must be a live buffer.
pub unsafe fn changed_lines_invalidate_buf(
    buf: *mut buf_T,
    lnum: linenr_T,
    col: colnr_T,
    lnume: linenr_T,
    xtra: linenr_T,
) {
    unsafe {
        for wp in all_windows() {
            if (*wp).w_buffer == buf {
                changed_lines_invalidate_win(wp, lnum, col, lnume, xtra);
            }
        }
    }
}

/// Set the `'.` mark to the change, and add it to the change list unless it
/// continues the last entry.
///
/// A new entry is only made for a new undo-able change (`b_new_change`), and
/// then only if it is far enough from the last one -- otherwise typing
/// `xxxxx` would fill the list. "Far enough" is a 'textwidth' away, or 79
/// columns when 'textwidth' is 0.
///
/// # Safety
/// `buf` must be a live buffer.
unsafe fn record_change_mark(buf: *mut buf_T, lnum: linenr_T, col: colnr_T) {
    unsafe {
        // Only record the view if the changed line is on screen: a change can
        // be made outside the current window's view.
        let mut view = fmarkv_T {
            topline_offset: MAXLNUM as linenr_T,
            skipcol: 0,
        };
        if (*curwin.get()).w_buffer == buf
            && lnum >= (*curwin.get()).w_topline
            && lnum <= (*curwin.get()).w_botline
        {
            view = mark_view_make(curwin.get(), (*curwin.get()).w_cursor);
        }

        // RESET_FMARK: the old mark's additional data is freed first.
        let last_change = &raw mut (*buf).b_last_change;
        free_fmark(*last_change);
        (*last_change).mark = pos_T {
            lnum,
            col,
            coladd: 0,
        };
        (*last_change).fnum = (*buf).handle;
        (*last_change).timestamp = os_time();
        (*last_change).view = view;
        (*last_change).additional_data = ::core::ptr::null_mut();

        let changelist = &raw mut (*buf).b_changelist as *mut fmark_T;
        if (*buf).b_new_change || (*buf).b_changelistlen == 0 {
            let add = if (*buf).b_changelistlen == 0 {
                true
            } else {
                let p = &(*changelist.offset(((*buf).b_changelistlen - 1) as isize)).mark;
                if p.lnum != lnum {
                    true
                } else {
                    let mut cols = comp_textwidth(false);
                    if cols == 0 {
                        cols = 79;
                    }
                    p.col + cols < col || col + cols < p.col
                }
            };
            if add {
                // The first of a new sequence of undo-able changes, far enough
                // from the last one to deserve its own entry.
                (*buf).b_new_change = false;

                if (*buf).b_changelistlen == JUMPLISTSIZE {
                    // The list is full: drop the oldest entry, and pull every
                    // window's index back with it.
                    (*buf).b_changelistlen = JUMPLISTSIZE - 1;
                    memmove(
                        changelist as *mut c_void,
                        changelist.offset(1) as *const c_void,
                        ::core::mem::size_of::<fmark_T>()
                            .wrapping_mul((JUMPLISTSIZE - 1) as size_t),
                    );
                    for wp in all_windows() {
                        if (*wp).w_buffer == buf && (*wp).w_changelistidx > 0 {
                            (*wp).w_changelistidx -= 1;
                        }
                    }
                }
                // A window sitting at the end of the list stays at the end.
                for wp in all_windows() {
                    if (*wp).w_buffer == buf && (*wp).w_changelistidx == (*buf).b_changelistlen {
                        (*wp).w_changelistidx += 1;
                    }
                }
                (*buf).b_changelistlen += 1;
            }
        }
        *changelist.offset(((*buf).b_changelistlen - 1) as isize) = (*buf).b_last_change;
        // The current window is always *after* the last change, so that `g,`
        // takes you back to it.
        if (*curwin.get()).w_buffer == buf {
            (*curwin.get()).w_changelistidx = (*buf).b_changelistlen;
        }
    }
}

/// Bring one window's fold, scroll and cursor-line state up to date with a
/// change that covered `lnum`..`lnume` and moved what follows by `xtra`.
///
/// # Safety
/// `wp` must be a live window displaying the buffer the change was made in.
unsafe fn redraw_win_for_change(
    wp: *mut win_T,
    mut lnum: linenr_T,
    col: colnr_T,
    lnume: linenr_T,
    xtra: linenr_T,
) {
    unsafe {
        if !redraw_not_allowed.get() && (*wp).w_redr_type < UPD_VALID {
            (*wp).w_redr_type = UPD_VALID;
        }
        // Adding or removing lines invalidates a pending w_redraw_top/bot
        // range, so redraw everything instead.
        if xtra != 0 && (*wp).w_redraw_top != 0 {
            redraw_later(wp, UPD_NOT_VALID);
        }

        let mut last = lnume + xtra - 1; // last line after the change

        // Reset 'smoothscroll''s w_skipcol if the topline has become so short
        // that nothing would be visible, allowing for the `<<<` marker.
        if (*wp).w_skipcol > 0
            && (last < (*wp).w_topline
                || ((*wp).w_topline >= lnum
                    && (*wp).w_topline < lnume
                    && linetabsize_eol(wp, (*wp).w_topline)
                        <= (*wp).w_skipcol + sms_marker_overlap(wp, -1)))
        {
            (*wp).w_skipcol = 0;
        }

        // Can't postpone the fold update: a following operator might work on
        // the whole fold, as `>>dd` does.
        fold_update(wp, lnum, last);

        // The change may pull the lines above or below it into a fold, so widen
        // lnum/last to what might now be displayed differently. Setting
        // w_cline_folded here is the cheap way to keep it right when inserting
        // just above a closed fold.
        let mut folded = has_folding_win(
            wp,
            lnum,
            &raw mut lnum,
            ::core::ptr::null_mut(),
            false,
            ::core::ptr::null_mut(),
        );
        if (*wp).w_cursor.lnum == lnum {
            (*wp).w_cline_folded = folded;
        }
        folded = has_folding_win(
            wp,
            last,
            ::core::ptr::null_mut(),
            &raw mut last,
            false,
            ::core::ptr::null_mut(),
        );
        if (*wp).w_cursor.lnum == last {
            (*wp).w_cline_folded = folded;
        }

        changed_lines_invalidate_win(wp, lnum, col, lnume, xtra);

        // Setting w_topline has side effects once the folds have changed --
        // especially when the buffer was changed in another window.
        if has_any_folding(wp) != 0 {
            set_topline(wp, (*wp).w_topline);
        }

        // 'relativenumber' always needs a redraw when lines came or went, even
        // if the cursor did not move.
        if (*wp).w_onebuf_opt.wo_rnu != 0 && xtra != 0 {
            (*wp).w_last_cursor_lnum_rnu = 0;
        }

        if (*wp).w_onebuf_opt.wo_cul != 0 && (*wp).w_last_cursorline >= lnum {
            if (*wp).w_last_cursorline < lnume {
                // 'cursorline' was inside the change: the loop above has
                // already invalidated it in w_lines[].
                (*wp).w_last_cursorline = 0;
            } else {
                // Below the change: shift it.
                (*wp).w_last_cursorline += xtra;
            }
        }
    }
}

/// Everything a change does besides marking the redraw area: the modified
/// flag, the diff windows, the `'.` mark and change list, and every window's
/// cached display state.
///
/// See [`changed_lines`] for the arguments.
///
/// # Safety
/// `buf` must be a live buffer. May trigger autocommands that reload it.
unsafe fn changed_common(
    buf: *mut buf_T,
    lnum: linenr_T,
    col: colnr_T,
    lnume: linenr_T,
    xtra: linenr_T,
) {
    unsafe {
        changed(buf);

        for win in windows_in_curtab() {
            if (*win).w_buffer == buf && (*win).w_onebuf_opt.wo_diff != 0 && diff_internal() != 0 {
                (*curtab.get()).tp_diff_update = true as c_int;
                diff_update_line(lnum);
            }
        }

        if !cmdmod_has(CmdModFlags::KEEPJUMPS) {
            record_change_mark(buf, lnum, col);
        }

        if (*curwin.get()).w_buffer == buf && VIsual_active.get() {
            check_visual_pos();
        }

        for wp in all_windows() {
            if (*wp).w_buffer == buf {
                redraw_win_for_change(wp, lnum, col, lnume, xtra);
            }
            if wp == curwin.get() && xtra != 0 && search_hl_has_cursor_lnum.get() >= lnum {
                *search_hl_has_cursor_lnum.ptr() += xtra;
            }
        }

        // update_screen() works out what to redraw from b_mod_set / b_mod_*.
        set_must_redraw(UPD_VALID);

        // A change on the cursor line always triggers CursorMoved.
        if last_cursormoved_win.get() == curwin.get()
            && (*curwin.get()).w_buffer == buf
            && lnum <= (*curwin.get()).w_cursor.lnum
            && lnume + xtra.abs() > (*curwin.get()).w_cursor.lnum
        {
            (*last_cursormoved.ptr()).lnum = 0;
        }
    }
}

/// Changed bytes within a single line of the current buffer.
///
/// # Safety
/// `lnum` must be a valid line of the current buffer. May trigger
/// autocommands that reload it.
pub unsafe fn changed_bytes(lnum: linenr_T, col: colnr_T) {
    unsafe {
        changed_lines_redraw_buf(curbuf.get(), lnum, lnum + 1, 0);
        changed_common(curbuf.get(), lnum, col, lnum + 1, 0);

        // Changing the end of a line can add or remove SpellCap on the start of
        // the next one, so schedule that line too -- but not when a `$` is
        // being displayed at the end of the changed text.
        if spell_check_window(curwin.get())
            && lnum < (*curbuf.get()).b_ml.ml_line_count
            && !cpo_has(CpoFlag::DOLLAR)
        {
            redraw_win_line(curwin.get(), lnum + 1);
        }

        // Notify any channels that are watching.
        buf_updates_send_changes(curbuf.get(), lnum, 1, 1);

        // Diff highlighting in the other diff windows may need updating too.
        if (*curwin.get()).w_onebuf_opt.wo_diff != 0 {
            for wp in windows_in_curtab() {
                if (*wp).w_onebuf_opt.wo_diff != 0 && wp != curwin.get() {
                    redraw_later(wp, UPD_VALID);
                    let wlnum = diff_lnum_win(lnum, wp);
                    if wlnum > 0 {
                        changed_lines_redraw_buf((*wp).w_buffer, wlnum, wlnum + 1, 0);
                    }
                }
            }
        }
    }
}

/// [`changed_bytes`], plus the extmark splice for the bytes that came and went.
///
/// # Safety
/// `lnum` must be a valid line of the current buffer.
pub unsafe fn inserted_bytes(lnum: linenr_T, start_col: colnr_T, old_col: c_int, new_col: c_int) {
    unsafe {
        if curbuf_splice_pending.get() == 0 {
            extmark_splice_cols(
                curbuf.get(),
                lnum - 1,
                start_col,
                old_col,
                new_col,
                kExtmarkUndo,
            );
        }
        changed_bytes(lnum, start_col);
    }
}

/// `count` lines were appended below line `lnum` of `buf`.
///
/// Call AFTER the change and after `mark_adjust()`.
///
/// # Safety
/// `buf` must be a live buffer.
pub unsafe fn appended_lines_buf(buf: *mut buf_T, lnum: linenr_T, count: linenr_T) {
    unsafe {
        changed_lines(buf, lnum + 1, 0, lnum + 1, count, true);
    }
}

/// [`appended_lines_buf`] for the current buffer.
///
/// # Safety
/// `lnum` must be a valid line of the current buffer.
pub unsafe fn appended_lines(lnum: linenr_T, count: linenr_T) {
    unsafe {
        appended_lines_buf(curbuf.get(), lnum, count);
    }
}

/// [`appended_lines`], adjusting the marks first.
///
/// # Safety
/// `lnum` must be a valid line of the current buffer.
pub unsafe fn appended_lines_mark(lnum: linenr_T, count: c_int) {
    unsafe {
        mark_adjust(lnum + 1, MAXLNUM as linenr_T, count, 0, kExtmarkUndo);
        changed_lines(curbuf.get(), lnum + 1, 0, lnum + 1, count, true);
    }
}

/// `count` lines were deleted at line `lnum` of `buf`.
///
/// Call AFTER the change and after `mark_adjust()`.
///
/// # Safety
/// `buf` must be a live buffer.
pub unsafe fn deleted_lines_buf(buf: *mut buf_T, lnum: linenr_T, count: linenr_T) {
    unsafe {
        changed_lines(buf, lnum, 0, lnum + count, -count, true);
    }
}

/// [`deleted_lines_buf`] for the current buffer.
///
/// # Safety
/// `lnum` must be a valid line of the current buffer.
pub unsafe fn deleted_lines(lnum: linenr_T, count: linenr_T) {
    unsafe {
        deleted_lines_buf(curbuf.get(), lnum, count);
    }
}

/// [`deleted_lines`], adjusting the marks first.
///
/// Make sure the cursor is on a valid line before calling: a UI callback may
/// be triggered to display it.
///
/// # Safety
/// `lnum` must be a valid line of the current buffer.
pub unsafe fn deleted_lines_mark(lnum: linenr_T, count: c_int) {
    unsafe {
        let made_empty = count > 0 && (*curbuf.get()).b_ml.ml_flags.has(MlFlags::EMPTY);

        mark_adjust(
            lnum,
            lnum + count - 1,
            MAXLNUM as linenr_T,
            -count,
            kExtmarkNOOP,
        );
        // Deleting the whole buffer implicitly adds one empty line back.
        extmark_adjust(
            curbuf.get(),
            lnum,
            lnum + count - 1,
            MAXLNUM as linenr_T,
            -count + i32::from(made_empty),
            kExtmarkUndo,
        );
        changed_lines(curbuf.get(), lnum, 0, lnum + count, -count, true);
    }
}

/// Widen `buf`'s pending redraw area (`b_mod_*`) to cover a change.
///
/// Consider also calling [`changed_lines_invalidate_buf`].
///
/// `lnum` is the first changed line, `lnume` the line below the last changed
/// one *before* the change, and `xtra` the net number of lines added
/// (negative when deleting).
///
/// # Safety
/// `buf` must be a live buffer.
pub unsafe fn changed_lines_redraw_buf(
    buf: *mut buf_T,
    lnum: linenr_T,
    mut lnume: linenr_T,
    xtra: linenr_T,
) {
    unsafe {
        // A decoration whose mark moved has to be re-measured and redrawn at
        // wherever it moved to, so widen by one line; a virt_line mark may be
        // drawn two lines below, so a deletion widens by one more.
        if xtra != 0 && (*(&raw mut (*buf).b_marktree as *mut MarkTree)).n_keys > 0 {
            lnume += 1 + linenr_T::from(xtra < 0 && buf_meta_total(buf, kMTMetaLines) != 0);
        }

        if (*buf).b_mod_set {
            // Widen to the maximum area that must be redisplayed.
            (*buf).b_mod_top = (*buf).b_mod_top.min(lnum);
            if lnum < (*buf).b_mod_bot {
                // Adjust the old bottom for the lines that came or went.
                (*buf).b_mod_bot += xtra;
                (*buf).b_mod_bot = (*buf).b_mod_bot.max(lnum);
            }
            (*buf).b_mod_bot = (*buf).b_mod_bot.max(lnume + xtra);
            (*buf).b_mod_xlines += xtra;
        } else {
            (*buf).b_mod_set = true;
            (*buf).b_mod_top = lnum;
            (*buf).b_mod_bot = lnume + xtra;
            (*buf).b_mod_xlines = xtra;
        }
    }
}

/// Lines of `buf` changed.
///
/// Call AFTER the change and after `mark_adjust()`. `lnum` is the first line
/// that needs displaying, `lnume` the first line below the changed ones
/// *before* the change (so the two are equal when only inserting), and `xtra`
/// the net number of lines added.
///
/// `do_buf_event` exists for undo/redo, which call this and then bump
/// `b:changedtick` *again*; those callers send the `nvim_buf_lines_event`
/// themselves once they are done.
///
/// # Safety
/// `buf` must be a live buffer. May trigger autocommands that reload it.
pub unsafe fn changed_lines(
    buf: *mut buf_T,
    lnum: linenr_T,
    col: colnr_T,
    lnume: linenr_T,
    xtra: linenr_T,
    do_buf_event: bool,
) {
    unsafe {
        changed_lines_redraw_buf(buf, lnum, lnume, xtra);

        if xtra == 0
            && (*curwin.get()).w_onebuf_opt.wo_diff != 0
            && (*curwin.get()).w_buffer == buf
            && diff_internal() == 0
        {
            // With the line count unchanged, mark_adjust() is never called, so
            // the other diff buffers still have to be marked for display.
            for wp in windows_in_curtab() {
                if (*wp).w_onebuf_opt.wo_diff != 0 && wp != curwin.get() {
                    redraw_later(wp, UPD_VALID);
                    let wlnum = diff_lnum_win(lnum, wp);
                    if wlnum > 0 {
                        changed_lines_redraw_buf((*wp).w_buffer, wlnum, lnume - lnum + wlnum, 0);
                    }
                }
            }
        }

        changed_common(buf, lnum, col, lnume, xtra);

        if do_buf_event {
            let num_added = int64_t::from(lnume + xtra - lnum);
            let num_removed = int64_t::from(lnume - lnum);
            buf_updates_send_changes(buf, lnum, num_added, num_removed);
        }
    }
}
