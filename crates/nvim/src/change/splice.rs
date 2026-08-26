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

use core::ffi::{c_int, c_void};
use core::mem::offset_of;

use super::*;
use crate::ex_docmd::cmdmod_has;
use crate::memline::MlFlags;
use crate::normal::visual_active;
use crate::option::cpo_has;
use crate::types::{CpoFlag, foldinfo_T};
use crate::winlayer::{Buf, TabPage, Win, tab_windows, windows};

/// Drop the cached display information one window holds about the lines a
/// change covered, and shift what is below it by `xtra`.
fn changed_lines_invalidate_win(
    mut wp: Win,
    lnum: linenr_T,
    col: colnr_T,
    mut lnume: linenr_T,
    xtra: linenr_T,
) {
    if wp.w_cursor.lnum <= lnum {
        // SAFETY: a live window; the answer is an index into `w_lines` or -1.
        let i = unsafe { find_wl_entry(wp.raw(), lnum) };
        // SAFETY: as above, and the short circuit is what bounds it.
        let below =
            i >= 0 && wp.w_cursor.lnum > unsafe { (*wp.w_lines.offset(i as isize)).wl_lnum };
        if below {
            // SAFETY: a live window.
            unsafe { changed_line_abv_curs_win(wp.raw()) };
        }
    }
    // SAFETY: a live window, in all four calls.
    unsafe {
        if wp.w_cursor.lnum > lnum {
            changed_line_abv_curs_win(wp.raw());
        } else if wp.w_cursor.lnum == lnum && wp.w_cursor.col >= col {
            changed_cline_bef_curs(wp.raw());
        }
        if wp.w_botline >= lnum {
            if xtra < 0 {
                invalidate_botline_win(wp.raw());
            } else {
                approximate_botline_win(wp.raw());
            }
        }
    }

    // Inline virtual text under 'wrap', and virtual lines, make the line
    // after the change part of it as far as the display cache goes.
    // SAFETY: a live window's buffer is live, in both calls; the short
    // circuits are upstream's.
    let widen = xtra < 0
        && wp.w_onebuf_opt.wo_wrap != 0
        && unsafe { buf_meta_total(wp.w_buffer, kMTMetaInline) } != 0
        || xtra != 0 && unsafe { buf_meta_total(wp.w_buffer, kMTMetaLines) } != 0;
    if widen {
        lnume += 1;
    }

    let lines = wp.w_lines;
    for i in 0..wp.w_lines_valid {
        // SAFETY: `w_lines` holds at least `w_lines_valid` entries, and is
        // only null while that count is zero.
        let wl = unsafe { &mut *lines.offset(i as isize) };
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
    for wp in tab_windows() {
        if wp.w_buffer == buf {
            changed_lines_invalidate_win(wp, lnum, col, lnume, xtra);
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
fn record_change_mark(mut buf: Buf, lnum: linenr_T, col: colnr_T) {
    // Only record the view if the changed line is on screen: a change can
    // be made outside the current window's view.
    let mut view = fmarkv_T {
        topline_offset: MAXLNUM as linenr_T,
        skipcol: 0,
    };
    let win = cur_win();
    if win.w_buffer == buf.raw() && lnum >= win.w_topline && lnum <= win.w_botline {
        let at = win.w_cursor;
        // SAFETY: the current window is live.
        view = unsafe { mark_view_make(win.raw(), at) };
    }

    // RESET_FMARK: the old mark's additional data is freed first.
    let old = buf.b_last_change.clone();
    // SAFETY: the additional data is the mark's own, and nothing else holds
    // it once the mark is overwritten below.
    unsafe { free_fmark(old) };
    let handle = buf.handle;
    let now = os_time();
    buf.b_last_change.mark = pos_T {
        lnum,
        col,
        coladd: 0,
    };
    buf.b_last_change.fnum = handle;
    buf.b_last_change.timestamp = now;
    buf.b_last_change.view = view;
    buf.b_last_change.additional_data = ::core::ptr::null_mut();

    if buf.b_new_change || buf.b_changelistlen == 0 {
        let add = if buf.b_changelistlen == 0 {
            true
        } else {
            let p = buf.b_changelist[(buf.b_changelistlen - 1) as usize].mark;
            if p.lnum != lnum {
                true
            } else {
                // SAFETY: the editor exists.
                let mut cols = unsafe { comp_textwidth(false) };
                if cols == 0 {
                    cols = 79;
                }
                p.col + cols < col || col + cols < p.col
            }
        };
        if add {
            // The first of a new sequence of undo-able changes, far enough
            // from the last one to deserve its own entry.
            buf.b_new_change = false;

            if buf.b_changelistlen == JUMPLISTSIZE {
                // The list is full: drop the oldest entry, and pull every
                // window's index back with it.
                buf.b_changelistlen = JUMPLISTSIZE - 1;
                // A field's address is the object's plus a constant, which
                // needs no dereference to compute.
                let head = buf
                    .raw()
                    .wrapping_byte_add(offset_of!(buf_T, b_changelist))
                    .cast::<c_void>();
                let one = ::core::mem::size_of::<fmark_T>();
                let bytes = one.wrapping_mul((JUMPLISTSIZE - 1) as size_t);
                // SAFETY: `b_changelist` holds `JUMPLISTSIZE` marks, so its
                // last `JUMPLISTSIZE - 1` fit at its head.
                unsafe { memmove(head, head.wrapping_byte_add(one).cast_const(), bytes) };
                for mut wp in tab_windows() {
                    if wp.w_buffer == buf.raw() && wp.w_changelistidx > 0 {
                        wp.w_changelistidx -= 1;
                    }
                }
            }
            // A window sitting at the end of the list stays at the end.
            for mut wp in tab_windows() {
                if wp.w_buffer == buf.raw() && wp.w_changelistidx == buf.b_changelistlen {
                    wp.w_changelistidx += 1;
                }
            }
            buf.b_changelistlen += 1;
        }
    }
    let last = buf.b_last_change.clone();
    let at = (buf.b_changelistlen - 1) as usize;
    buf.b_changelist[at] = last;
    // The current window is always *after* the last change, so that `g,`
    // takes you back to it.
    let len = buf.b_changelistlen;
    if cur_win().w_buffer == buf.raw() {
        cur_win().w_changelistidx = len;
    }
}

/// Bring one window's fold, scroll and cursor-line state up to date with a
/// change that covered `lnum`..`lnume` and moved what follows by `xtra`.
fn redraw_win_for_change(
    mut wp: Win,
    mut lnum: linenr_T,
    col: colnr_T,
    lnume: linenr_T,
    xtra: linenr_T,
) {
    if !redraw_not_allowed.get() && wp.w_redr_type < UPD_VALID {
        wp.w_redr_type = UPD_VALID;
    }
    // Adding or removing lines invalidates a pending w_redraw_top/bot
    // range, so redraw everything instead.
    if xtra != 0 && wp.w_redraw_top != 0 {
        wp.redraw_later(UPD_NOT_VALID);
    }

    let mut last = lnume + xtra - 1; // last line after the change

    // Reset 'smoothscroll''s w_skipcol if the topline has become so short
    // that nothing would be visible, allowing for the `<<<` marker.
    // SAFETY: a live window, in both calls; the short circuits are
    // upstream's.
    let hide_all = wp.w_skipcol > 0
        && (last < wp.w_topline
            || (wp.w_topline >= lnum
                && wp.w_topline < lnume
                && unsafe { linetabsize_eol(wp.raw(), wp.w_topline) }
                    <= wp.w_skipcol + unsafe { sms_marker_overlap(wp.raw(), -1) }));
    if hide_all {
        wp.w_skipcol = 0;
    }

    // Can't postpone the fold update: a following operator might work on
    // the whole fold, as `>>dd` does.
    // SAFETY: a live window.
    unsafe { fold_update(wp.raw(), lnum, last) };

    // The change may pull the lines above or below it into a fold, so widen
    // lnum/last to what might now be displayed differently. Setting
    // w_cline_folded here is the cheap way to keep it right when inserting
    // just above a closed fold.
    let noline = ::core::ptr::null_mut::<linenr_T>();
    let nofold = ::core::ptr::null_mut::<foldinfo_T>();
    let at = &raw mut lnum;
    // SAFETY: a live window; only `firstp` is asked for, and it is a local.
    let mut folded = unsafe { has_folding_win(wp.raw(), lnum, at, noline, false, nofold) };
    if wp.w_cursor.lnum == lnum {
        wp.w_cline_folded = folded;
    }
    let to = &raw mut last;
    // SAFETY: as above, for `lastp`.
    folded = unsafe { has_folding_win(wp.raw(), last, noline, to, false, nofold) };
    if wp.w_cursor.lnum == last {
        wp.w_cline_folded = folded;
    }

    changed_lines_invalidate_win(wp, lnum, col, lnume, xtra);

    // Setting w_topline has side effects once the folds have changed --
    // especially when the buffer was changed in another window.
    if wp.has_any_folding() {
        let top = wp.w_topline;
        // SAFETY: a live window.
        unsafe { set_topline(wp.raw(), top) };
    }

    // 'relativenumber' always needs a redraw when lines came or went, even
    // if the cursor did not move.
    if wp.w_onebuf_opt.wo_rnu != 0 && xtra != 0 {
        wp.w_last_cursor_lnum_rnu = 0;
    }

    if wp.w_onebuf_opt.wo_cul != 0 && wp.w_last_cursorline >= lnum {
        if wp.w_last_cursorline < lnume {
            // 'cursorline' was inside the change: the loop above has
            // already invalidated it in w_lines[].
            wp.w_last_cursorline = 0;
        } else {
            // Below the change: shift it.
            wp.w_last_cursorline += xtra;
        }
    }
}

/// Everything a change does besides marking the redraw area: the modified
/// flag, the diff windows, the `'.` mark and change list, and every window's
/// cached display state.
///
/// See [`changed_lines`] for the arguments.
fn changed_common(buf: Buf, lnum: linenr_T, col: colnr_T, lnume: linenr_T, xtra: linenr_T) {
    // SAFETY: a live buffer.
    unsafe { changed(buf.raw()) };

    for win in windows() {
        // SAFETY: the editor exists; the short circuit is upstream's.
        let diffed = win.w_buffer == buf.raw()
            && win.w_onebuf_opt.wo_diff != 0
            && unsafe { diff_internal() } != 0;
        if diffed {
            cur_tab().tp_diff_update = 1;
            // SAFETY: a line of the current buffer.
            unsafe { diff_update_line(lnum) };
        }
    }

    if !cmdmod_has(CmdModFlags::KEEPJUMPS) {
        record_change_mark(buf, lnum, col);
    }

    if cur_win().w_buffer == buf.raw() && visual_active() {
        // SAFETY: the editor exists.
        unsafe { check_visual_pos() };
    }

    for wp in tab_windows() {
        if wp.w_buffer == buf.raw() {
            redraw_win_for_change(wp, lnum, col, lnume, xtra);
        }
        if wp.is_current() && xtra != 0 && search_hl_has_cursor_lnum.get() >= lnum {
            search_hl_has_cursor_lnum.set(search_hl_has_cursor_lnum.get() + xtra);
        }
    }

    // update_screen() works out what to redraw from b_mod_set / b_mod_*.
    set_must_redraw(UPD_VALID);

    // A change on the cursor line always triggers CursorMoved.
    let win = cur_win();
    if last_cursormoved_win.get() == win.raw()
        && win.w_buffer == buf.raw()
        && lnum <= win.w_cursor.lnum
        && lnume + xtra.abs() > win.w_cursor.lnum
    {
        last_cursormoved.set(last_cursormoved.get().with_lnum(0));
    }
}

/// Changed bytes within a single line of the current buffer.
///
/// # Safety
/// `lnum` must be a valid line of the current buffer. May trigger
/// autocommands that reload it.
pub unsafe fn changed_bytes(lnum: linenr_T, col: colnr_T) {
    // SAFETY: the current buffer is live and `lnum` is a line of it.
    unsafe { changed_lines_redraw_buf(curbuf.get(), lnum, lnum + 1, 0) };
    changed_common(cur_buf(), lnum, col, lnum + 1, 0);

    // Changing the end of a line can add or remove SpellCap on the start of
    // the next one, so schedule that line too -- but not when a `$` is
    // being displayed at the end of the changed text.
    // SAFETY: the current window is live; the short circuit is upstream's.
    let spell_next = unsafe { spell_check_window(cur_win().raw()) }
        && lnum < cur_buf().b_ml.ml_line_count
        && !cpo_has(CpoFlag::DOLLAR);
    if spell_next {
        // SAFETY: the current window is live.
        unsafe { redraw_win_line(cur_win().raw(), lnum + 1) };
    }

    // Notify any channels that are watching.
    // SAFETY: the current buffer is live.
    unsafe { buf_updates_send_changes(curbuf.get(), lnum, 1, 1) };

    // Diff highlighting in the other diff windows may need updating too.
    if cur_win().w_onebuf_opt.wo_diff != 0 {
        for wp in windows() {
            if wp.w_onebuf_opt.wo_diff != 0 && !wp.is_current() {
                wp.redraw_later(UPD_VALID);
                // SAFETY: a live window.
                let wlnum = unsafe { diff_lnum_win(lnum, wp.raw()) };
                if wlnum > 0 {
                    // SAFETY: a live window's buffer is live.
                    unsafe { changed_lines_redraw_buf(wp.w_buffer, wlnum, wlnum + 1, 0) };
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
    if curbuf_splice_pending.get() == 0 {
        let cb = curbuf.get();
        // SAFETY: the current buffer is live and `lnum` is a line of it.
        unsafe {
            extmark_splice_cols(cb, lnum - 1, start_col, old_col, new_col, kExtmarkUndo);
        }
    }
    // SAFETY: as above.
    unsafe { changed_bytes(lnum, start_col) };
}

/// `count` lines were appended below line `lnum` of `buf`.
///
/// Call AFTER the change and after `mark_adjust()`.
///
/// # Safety
/// `buf` must be a live buffer.
pub unsafe fn appended_lines_buf(buf: *mut buf_T, lnum: linenr_T, count: linenr_T) {
    // SAFETY: the caller's buffer.
    unsafe { changed_lines(buf, lnum + 1, 0, lnum + 1, count, true) };
}

/// [`appended_lines_buf`] for the current buffer.
///
/// # Safety
/// `lnum` must be a valid line of the current buffer.
pub unsafe fn appended_lines(lnum: linenr_T, count: linenr_T) {
    // SAFETY: the current buffer is live.
    unsafe { appended_lines_buf(curbuf.get(), lnum, count) };
}

/// [`appended_lines`], adjusting the marks first.
///
/// # Safety
/// `lnum` must be a valid line of the current buffer.
pub unsafe fn appended_lines_mark(lnum: linenr_T, count: c_int) {
    let max = MAXLNUM as linenr_T;
    let cb = curbuf.get();
    // SAFETY: the current buffer is live and `lnum` is a line of it.
    unsafe {
        mark_adjust(lnum + 1, max, count, 0, kExtmarkUndo);
        changed_lines(cb, lnum + 1, 0, lnum + 1, count, true);
    }
}

/// `count` lines were deleted at line `lnum` of `buf`.
///
/// Call AFTER the change and after `mark_adjust()`.
///
/// # Safety
/// `buf` must be a live buffer.
pub unsafe fn deleted_lines_buf(buf: *mut buf_T, lnum: linenr_T, count: linenr_T) {
    // SAFETY: the caller's buffer.
    unsafe { changed_lines(buf, lnum, 0, lnum + count, -count, true) };
}

/// [`deleted_lines_buf`] for the current buffer.
///
/// # Safety
/// `lnum` must be a valid line of the current buffer.
pub unsafe fn deleted_lines(lnum: linenr_T, count: linenr_T) {
    // SAFETY: the current buffer is live.
    unsafe { deleted_lines_buf(curbuf.get(), lnum, count) };
}

/// [`deleted_lines`], adjusting the marks first.
///
/// Make sure the cursor is on a valid line before calling: a UI callback may
/// be triggered to display it.
///
/// # Safety
/// `lnum` must be a valid line of the current buffer.
pub unsafe fn deleted_lines_mark(lnum: linenr_T, count: c_int) {
    let made_empty = count > 0 && cur_buf().b_ml.ml_flags.has(MlFlags::EMPTY);
    let cb = curbuf.get();
    let last = lnum + count - 1;
    let max = MAXLNUM as linenr_T;
    // Deleting the whole buffer implicitly adds one empty line back.
    let back = -count + i32::from(made_empty);
    // SAFETY: the current buffer is live and `lnum` is a line of it.
    unsafe {
        mark_adjust(lnum, last, max, -count, kExtmarkNOOP);
        extmark_adjust(cb, lnum, last, max, back, kExtmarkUndo);
        changed_lines(cb, lnum, 0, lnum + count, -count, true);
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
    // SAFETY: the caller's buffer.
    let mut buf = unsafe { Buf::new(buf) };
    // A decoration whose mark moved has to be re-measured and redrawn at
    // wherever it moved to, so widen by one line; a virt_line mark may be
    // drawn two lines below, so a deletion widens by one more.
    if xtra != 0 && buf.b_marktree[0].n_keys > 0 {
        // SAFETY: a live buffer.
        let lines = unsafe { buf_meta_total(buf.raw(), kMTMetaLines) };
        lnume += 1 + linenr_T::from(xtra < 0 && lines != 0);
    }

    if buf.b_mod_set {
        // Widen to the maximum area that must be redisplayed.
        buf.b_mod_top = buf.b_mod_top.min(lnum);
        if lnum < buf.b_mod_bot {
            // Adjust the old bottom for the lines that came or went.
            buf.b_mod_bot += xtra;
            buf.b_mod_bot = buf.b_mod_bot.max(lnum);
        }
        buf.b_mod_bot = buf.b_mod_bot.max(lnume + xtra);
        buf.b_mod_xlines += xtra;
    } else {
        buf.b_mod_set = true;
        buf.b_mod_top = lnum;
        buf.b_mod_bot = lnume + xtra;
        buf.b_mod_xlines = xtra;
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
    // SAFETY: the caller's buffer.
    unsafe { changed_lines_redraw_buf(buf, lnum, lnume, xtra) };

    // SAFETY: the editor exists; the short circuit is upstream's.
    let diff_same_lines = xtra == 0
        && cur_win().w_onebuf_opt.wo_diff != 0
        && cur_win().w_buffer == buf
        && unsafe { diff_internal() } == 0;
    if diff_same_lines {
        // With the line count unchanged, mark_adjust() is never called, so
        // the other diff buffers still have to be marked for display.
        for wp in windows() {
            if wp.w_onebuf_opt.wo_diff != 0 && !wp.is_current() {
                wp.redraw_later(UPD_VALID);
                // SAFETY: a live window.
                let wlnum = unsafe { diff_lnum_win(lnum, wp.raw()) };
                if wlnum > 0 {
                    let bot = lnume - lnum + wlnum;
                    // SAFETY: a live window's buffer is live.
                    unsafe { changed_lines_redraw_buf(wp.w_buffer, wlnum, bot, 0) };
                }
            }
        }
    }

    // SAFETY: the caller's buffer.
    changed_common(unsafe { Buf::new(buf) }, lnum, col, lnume, xtra);

    if do_buf_event {
        let num_added = int64_t::from(lnume + xtra - lnum);
        let num_removed = int64_t::from(lnume - lnum);
        // SAFETY: the caller's buffer.
        unsafe { buf_updates_send_changes(buf, lnum, num_added, num_removed) };
    }
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}

/// The tab page the editor is working in.
fn cur_tab() -> TabPage {
    // SAFETY: `curtab` is set from startup to exit.
    unsafe { TabPage::current() }
}
