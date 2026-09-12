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
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{c_int, c_void};
use core::mem::offset_of;

use super::*;
use crate::ex_docmd::cmdmod_has;
use crate::memline::MlFlags;
use crate::normal::visual_active;
use crate::option::cpo_has;
use crate::types::CpoFlag;
use crate::winlayer::{Buf, TabPage, Win, tab_windows, windows};

/// Drop the cached display information one window holds about the lines a
/// change covered, and shift what is below it by `xtra`.
fn changed_lines_invalidate_win(
    window: Win,
    lnum: LineNr,
    col: ColNr,
    mut lnume: LineNr,
    xtra: LineNr,
) {
    if window.w_cursor.lnum <= lnum {
        // SAFETY: a live window; the answer is an index into `w_lines` or -1.
        let i = find_wl_entry(window, lnum);
        // SAFETY: as above, and the short circuit is what bounds it.
        let below = i >= 0
            && window.w_cursor.lnum > unsafe { (*window.w_lines.offset(i as isize)).wl_lnum };
        if below {
            changed_line_abv_curs_win(window);
        }
    }
    if window.w_cursor.lnum > lnum {
        changed_line_abv_curs_win(window);
    } else if window.w_cursor.lnum == lnum && window.w_cursor.col >= col {
        changed_cline_bef_curs(window);
    }
    if window.w_botline >= lnum {
        if xtra < 0 {
            invalidate_botline_win(window);
        } else {
            approximate_botline_win(window);
        }
    }

    // Inline virtual text under 'wrap', and virtual lines, make the line
    // after the change part of it as far as the display cache goes.
    // SAFETY: a live window's buffer is live, in both calls; the short
    // circuits are upstream's.
    let widen = xtra < 0
        && window.w_onebuf_opt.wo_wrap != 0
        && buf_meta_total(window.buffer(), kMTMetaInline) != 0
        || xtra != 0 && buf_meta_total(window.buffer(), kMTMetaLines) != 0;
    if widen {
        lnume += 1;
    }

    let lines = window.w_lines;
    for i in 0..window.w_lines_valid {
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

/// [`changed_lines_invalidate_win`] for every window displaying `buffer`.
///
pub fn changed_lines_invalidate_buf(
    buffer: Buf,
    lnum: LineNr,
    col: ColNr,
    lnume: LineNr,
    xtra: LineNr,
) {
    for wp in tab_windows() {
        if wp.w_buffer == buffer.raw() {
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
fn record_change_mark(mut buffer: Buf, lnum: LineNr, col: ColNr) {
    // Only record the view if the changed line is on screen: a change can
    // be made outside the current window's view.
    let mut view = FileMarkView {
        topline_offset: MAXLNUM,
        skipcol: 0,
    };
    let win = Win::current();
    if win.w_buffer == buffer.raw() && lnum >= win.w_topline && lnum <= win.w_botline {
        let at = win.w_cursor;
        view = mark_view_make(win, at);
    }

    // RESET_FMARK: the old mark's additional data is freed first.
    let old = buffer.b_last_change.clone();
    // SAFETY: the additional data is the mark's own, and nothing else holds
    // it once the mark is overwritten below.
    unsafe { free_fmark(old) };
    let handle = buffer.handle;
    let now = os_time();
    buffer.b_last_change.mark = Pos {
        lnum,
        col,
        coladd: 0,
    };
    buffer.b_last_change.fnum = handle;
    buffer.b_last_change.timestamp = now;
    buffer.b_last_change.view = view;
    buffer.b_last_change.additional_data = ::core::ptr::null_mut();

    if buffer.b_new_change || buffer.b_changelistlen == 0 {
        let add = if buffer.b_changelistlen == 0 {
            true
        } else {
            let last = usize::try_from(buffer.b_changelistlen - 1)
                .expect("a non-empty changelist has a last entry");
            let p = buffer.b_changelist[last].mark;
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
            buffer.b_new_change = false;

            if buffer.b_changelistlen == JUMPLISTSIZE {
                // The list is full: drop the oldest entry, and pull every
                // window's index back with it.
                buffer.b_changelistlen = JUMPLISTSIZE - 1;
                // A field's address is the object's plus a constant, which
                // needs no dereference to compute.
                let head = buffer
                    .raw()
                    .wrapping_byte_add(offset_of!(Buffer, b_changelist))
                    .cast::<c_void>();
                let one = ::core::mem::size_of::<FileMark>();
                let bytes = one.wrapping_mul((JUMPLISTSIZE - 1) as size_t);
                // SAFETY: `b_changelist` holds `JUMPLISTSIZE` marks, so its
                // last `JUMPLISTSIZE - 1` fit at its head.
                let into = head.cast::<u8>();
                unsafe { into.copy_from(head.wrapping_byte_add(one).cast_const().cast(), bytes) };
                for mut wp in tab_windows() {
                    if wp.w_buffer == buffer.raw() && wp.w_changelistidx > 0 {
                        wp.w_changelistidx -= 1;
                    }
                }
            }
            // A window sitting at the end of the list stays at the end.
            for mut wp in tab_windows() {
                if wp.w_buffer == buffer.raw() && wp.w_changelistidx == buffer.b_changelistlen {
                    wp.w_changelistidx += 1;
                }
            }
            buffer.b_changelistlen += 1;
        }
    }
    let last = buffer.b_last_change.clone();
    let at = usize::try_from(buffer.b_changelistlen - 1)
        .expect("the changelist has at least the entry just added");
    buffer.b_changelist[at] = last;
    // The current window is always *after* the last change, so that `g,`
    // takes you back to it.
    let len = buffer.b_changelistlen;
    if Win::current().w_buffer == buffer.raw() {
        Win::current().w_changelistidx = len;
    }
}

/// Bring one window's fold, scroll and cursor-line state up to date with a
/// change that covered `lnum`..`lnume` and moved what follows by `xtra`.
fn redraw_win_for_change(
    mut window: Win,
    mut lnum: LineNr,
    col: ColNr,
    lnume: LineNr,
    xtra: LineNr,
) {
    if !redraw_not_allowed.get() && window.w_redr_type < UPD_VALID {
        window.w_redr_type = UPD_VALID;
    }
    // Adding or removing lines invalidates a pending w_redraw_top/bot
    // range, so redraw everything instead.
    if xtra != 0 && window.w_redraw_top != 0 {
        window.redraw_later(UPD_NOT_VALID);
    }

    let mut last = lnume + xtra - 1; // last line after the change

    // Reset 'smoothscroll''s w_skipcol if the topline has become so short
    // that nothing would be visible, allowing for the `<<<` marker.
    let hide_all = window.w_skipcol > 0
        && (last < window.w_topline
            || (window.w_topline >= lnum
                && window.w_topline < lnume
                && linetabsize_eol(window, window.w_topline)
                    <= window.w_skipcol + sms_marker_overlap(window, -1)));
    if hide_all {
        window.w_skipcol = 0;
    }

    // Can't postpone the fold update: a following operator might work on
    // the whole fold, as `>>dd` does.
    // SAFETY: a live window.
    fold_update(window, lnum, last);

    // The change may pull the lines above or below it into a fold, so widen
    // lnum/last to what might now be displayed differently. Setting
    // w_cline_folded here is the cheap way to keep it right when inserting
    // just above a closed fold.
    // Only `firstp` is asked for, and it is a local.
    let mut folded = has_folding_win(window, lnum, Some(&mut lnum), None, false, None);
    if window.w_cursor.lnum == lnum {
        window.w_cline_folded = folded;
    }
    // As above, for `lastp`.
    folded = has_folding_win(window, last, None, Some(&mut last), false, None);
    if window.w_cursor.lnum == last {
        window.w_cline_folded = folded;
    }

    changed_lines_invalidate_win(window, lnum, col, lnume, xtra);

    // Setting w_topline has side effects once the folds have changed --
    // especially when the buffer was changed in another window.
    if window.has_any_folding() {
        let top = window.w_topline;
        // SAFETY: a live window.
        set_topline(window, top);
    }

    // 'relativenumber' always needs a redraw when lines came or went, even
    // if the cursor did not move.
    if window.w_onebuf_opt.wo_rnu != 0 && xtra != 0 {
        window.w_last_cursor_lnum_rnu = 0;
    }

    if window.w_onebuf_opt.wo_cul != 0 && window.w_last_cursorline >= lnum {
        if window.w_last_cursorline < lnume {
            // 'cursorline' was inside the change: the loop above has
            // already invalidated it in w_lines[].
            window.w_last_cursorline = 0;
        } else {
            // Below the change: shift it.
            window.w_last_cursorline += xtra;
        }
    }
}

/// Everything a change does besides marking the redraw area: the modified
/// flag, the diff windows, the `'.` mark and change list, and every window's
/// cached display state.
///
/// See [`changed_lines`] for the arguments.
fn changed_common(buffer: Buf, lnum: LineNr, col: ColNr, lnume: LineNr, xtra: LineNr) {
    // SAFETY: a live buffer.
    unsafe { changed(buffer) };

    for win in windows() {
        // SAFETY: the editor exists; the short circuit is upstream's.
        let diffed = win.w_buffer == buffer.raw()
            && win.w_onebuf_opt.wo_diff != 0
            && unsafe { diff_internal() } != 0;
        if diffed {
            TabPage::current().tp_diff_update = 1;
            diff_update_line(lnum);
        }
    }

    if !cmdmod_has(CmdModFlags::KEEPJUMPS) {
        record_change_mark(buffer, lnum, col);
    }

    if Win::current().w_buffer == buffer.raw() && visual_active() {
        check_visual_pos();
    }

    for wp in tab_windows() {
        if wp.w_buffer == buffer.raw() {
            redraw_win_for_change(wp, lnum, col, lnume, xtra);
        }
        if wp.is_current() && xtra != 0 && search_hl_has_cursor_lnum.get() >= lnum {
            search_hl_has_cursor_lnum.set(search_hl_has_cursor_lnum.get() + xtra);
        }
    }

    // update_screen() works out what to redraw from b_mod_set / b_mod_*.
    set_must_redraw(UPD_VALID);

    // A change on the cursor line always triggers CursorMoved.
    let win = Win::current();
    if last_cursormoved_win.get() == win.raw()
        && win.w_buffer == buffer.raw()
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
pub unsafe fn changed_bytes(lnum: LineNr, col: ColNr) {
    changed_lines_redraw_buf(Buf::current(), lnum, lnum + 1, 0);
    changed_common(Buf::current(), lnum, col, lnum + 1, 0);

    // Changing the end of a line can add or remove SpellCap on the start of
    // the next one, so schedule that line too -- but not when a `$` is
    // being displayed at the end of the changed text.
    let spell_next = spell_check_window(Win::current())
        && lnum < Buf::current().b_ml.ml_line_count
        && !cpo_has(CpoFlag::DOLLAR);
    if spell_next {
        redraw_win_line(Win::current(), lnum + 1);
    }

    // Notify any channels that are watching.
    buf_updates_send_changes(Buf::current(), lnum, 1, 1);

    // Diff highlighting in the other diff windows may need updating too.
    if Win::current().w_onebuf_opt.wo_diff != 0 {
        for wp in windows() {
            if wp.w_onebuf_opt.wo_diff != 0 && !wp.is_current() {
                wp.redraw_later(UPD_VALID);
                let wlnum = diff_lnum_win(lnum, wp);
                if wlnum > 0 {
                    changed_lines_redraw_buf(wp.buffer(), wlnum, wlnum + 1, 0);
                }
            }
        }
    }
}

/// [`changed_bytes`], plus the extmark splice for the bytes that came and went.
///
/// # Safety
/// `lnum` must be a valid line of the current buffer.
pub unsafe fn inserted_bytes(lnum: LineNr, start_col: ColNr, old_col: c_int, new_col: c_int) {
    if curbuf_splice_pending.get() == 0 {
        let cb = Buf::current();
        extmark_splice_cols(cb, lnum - 1, start_col, old_col, new_col, kExtmarkUndo);
    }
    // SAFETY: as above.
    unsafe { changed_bytes(lnum, start_col) };
}

/// `count` lines were appended below line `lnum` of `buffer`.
///
/// Call AFTER the change and after `mark_adjust()`.
pub fn appended_lines_buf(buffer: Buf, lnum: LineNr, count: LineNr) {
    changed_lines(buffer, lnum + 1, 0, lnum + 1, count, true);
}

/// [`appended_lines_buf`] for the current buffer.
pub fn appended_lines(lnum: LineNr, count: LineNr) {
    appended_lines_buf(Buf::current(), lnum, count);
}

/// [`appended_lines`], adjusting the marks first.
///
/// # Safety
/// `lnum` must be a valid line of the current buffer.
pub unsafe fn appended_lines_mark(lnum: LineNr, count: c_int) {
    let max = MAXLNUM;
    let cb = Buf::current();
    // SAFETY: the current buffer is live and `lnum` is a line of it.
    unsafe { mark_adjust(lnum + 1, max, count, 0, kExtmarkUndo) };
    changed_lines(cb, lnum + 1, 0, lnum + 1, count, true);
}

/// `count` lines were deleted at line `lnum` of `buffer`.
///
/// Call AFTER the change and after `mark_adjust()`.
pub fn deleted_lines_buf(buffer: Buf, lnum: LineNr, count: LineNr) {
    changed_lines(buffer, lnum, 0, lnum + count, -count, true);
}

/// [`deleted_lines_buf`] for the current buffer.
pub fn deleted_lines(lnum: LineNr, count: LineNr) {
    deleted_lines_buf(Buf::current(), lnum, count);
}

/// [`deleted_lines`], adjusting the marks first.
///
/// Make sure the cursor is on a valid line before calling: a UI callback may
/// be triggered to display it.
///
/// # Safety
/// `lnum` must be a valid line of the current buffer.
pub unsafe fn deleted_lines_mark(lnum: LineNr, count: c_int) {
    let made_empty = count > 0 && Buf::current().b_ml.ml_flags.has(MlFlags::EMPTY);
    let cb = Buf::current();
    let last = lnum + count - 1;
    let max = MAXLNUM;
    // Deleting the whole buffer implicitly adds one empty line back.
    let back = -count + i32::from(made_empty);
    // SAFETY: the current buffer is live and `lnum` is a line of it.
    unsafe { mark_adjust(lnum, last, max, -count, kExtmarkNOOP) };
    extmark_adjust(cb, lnum, last, max, back, kExtmarkUndo);
    changed_lines(cb, lnum, 0, lnum + count, -count, true);
}

/// Widen `buffer`'s pending redraw area (`b_mod_*`) to cover a change.
///
/// Consider also calling [`changed_lines_invalidate_buf`].
///
/// `lnum` is the first changed line, `lnume` the line below the last changed
/// one *before* the change, and `xtra` the net number of lines added
/// (negative when deleting).
///
pub fn changed_lines_redraw_buf(mut buffer: Buf, lnum: LineNr, mut lnume: LineNr, xtra: LineNr) {
    // A decoration whose mark moved has to be re-measured and redrawn at
    // wherever it moved to, so widen by one line; a virt_line mark may be
    // drawn two lines below, so a deletion widens by one more.
    if xtra != 0 && buffer.b_marktree.n_keys > 0 {
        let lines = buf_meta_total(buffer, kMTMetaLines);
        lnume += 1 + LineNr::from(xtra < 0 && lines != 0);
    }

    if buffer.b_mod_set {
        // Widen to the maximum area that must be redisplayed.
        buffer.b_mod_top = buffer.b_mod_top.min(lnum);
        if lnum < buffer.b_mod_bot {
            // Adjust the old bottom for the lines that came or went.
            buffer.b_mod_bot += xtra;
            buffer.b_mod_bot = buffer.b_mod_bot.max(lnum);
        }
        buffer.b_mod_bot = buffer.b_mod_bot.max(lnume + xtra);
        buffer.b_mod_xlines += xtra;
    } else {
        buffer.b_mod_set = true;
        buffer.b_mod_top = lnum;
        buffer.b_mod_bot = lnume + xtra;
        buffer.b_mod_xlines = xtra;
    }
}

/// Lines of `buffer` changed.
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
/// May trigger autocommands that reload `buffer`, so the caller must not go on
/// using it across this call without re-deriving it from its handle.
pub fn changed_lines(
    buffer: Buf,
    lnum: LineNr,
    col: ColNr,
    lnume: LineNr,
    xtra: LineNr,
    do_buf_event: bool,
) {
    changed_lines_redraw_buf(buffer, lnum, lnume, xtra);

    // SAFETY: the editor exists; the short circuit is upstream's.
    let diff_same_lines = xtra == 0
        && Win::current().w_onebuf_opt.wo_diff != 0
        && Win::current().w_buffer == buffer.raw()
        && unsafe { diff_internal() } == 0;
    if diff_same_lines {
        // With the line count unchanged, mark_adjust() is never called, so
        // the other diff buffers still have to be marked for display.
        for wp in windows() {
            if wp.w_onebuf_opt.wo_diff != 0 && !wp.is_current() {
                wp.redraw_later(UPD_VALID);
                let wlnum = diff_lnum_win(lnum, wp);
                if wlnum > 0 {
                    let bot = lnume - lnum + wlnum;
                    changed_lines_redraw_buf(wp.buffer(), wlnum, bot, 0);
                }
            }
        }
    }

    changed_common(buffer, lnum, col, lnume, xtra);

    if do_buf_event {
        let num_added = int64_t::from(lnume + xtra - lnum);
        let num_removed = int64_t::from(lnume - lnum);
        buf_updates_send_changes(buffer, lnum, num_added, num_removed);
    }
}
