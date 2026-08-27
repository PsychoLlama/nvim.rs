//! Creating, deleting, opening and closing folds by hand — everything behind
//! the `z` commands and `:fold`/`:foldopen`/`:foldclose`.
//!
//! A fold's own `fd_flags` says whether it is drawn open, closed, or takes
//! its state from 'foldlevel' (`FD_LEVEL`); the moment the user opens or
//! closes one by hand the window becomes `w_fold_manual` and stops following
//! the option until 'foldlevel' is set again.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::buffer_updates::buf_updates_send_changes;
use crate::change::changed_lines;
use crate::cursor::check_cursor_col;
use crate::diff::diff_lnum_win;
use crate::drawscreen::{UPD_INVERTED, redraw_buf_later, redraw_curbuf_later};
use crate::garray::{ga_grow, ga_init};
use crate::main::p_fcl;
use crate::message::emsg;
use crate::r#move::changed_window_setting;
use crate::os::cshim::gettext;
use crate::winlayer::{Buf, TabPage, Win, windows_in_tab};
use core::ffi::c_int;
use core::ptr;

use super::marker::*;
use super::*;

/// Close fold for current window at position "pos".
/// Repeat "count" times.
///
/// # Safety
/// The current window must be live.
pub unsafe extern "C" fn close_fold(pos: pos_T, count: c_int) {
    // SAFETY: the caller's promise.
    set_fold_repeat(pos, count, 0);
}

/// Close fold for current window at position `pos` recursively.
///
/// # Safety
/// The current window must be live.
pub unsafe fn close_fold_recurse(pos: pos_T) {
    // SAFETY: the caller's promise.
    set_manual_fold(pos, false, true, None);
}

/// Open or Close folds for current window in lines "first" to "last".
/// Used for "zo", "zO", "zc" and "zC" in Visual mode.
///
/// `opening` — true to open, false to close
/// `recurse` — true to do it recursively
/// `had_visual` — true when Visual selection used
///
/// # Safety
/// The current window must be live.
pub unsafe fn op_fold_range(
    firstpos: pos_T,
    lastpos: pos_T,
    opening: c_int,
    recurse: c_int,
    had_visual: bool,
) {
    let mut done: c_int = DONE_NOTHING;
    let last = lastpos.lnum;
    let mut lnum = firstpos.lnum;
    while lnum <= last {
        let at = pos_T {
            lnum,
            col: 0,
            coladd: 0,
        };
        let mut lnum_next = lnum;
        // SAFETY: the caller's promise. Which side of `set_manual_fold` the
        // closed range is read on matters: opening a fold makes the closed
        // range shorter and closing one makes it longer, and the walk has to
        // step past whichever range the command leaves behind.
        if opening != 0 && recurse == 0 {
            has_folding(cur_win(), lnum, None, Some(&mut lnum_next));
        }
        set_manual_fold(at, opening != 0, recurse != 0, Some(&mut done));
        if opening == 0 && recurse == 0 {
            has_folding(cur_win(), lnum, None, Some(&mut lnum_next));
        }
        lnum = lnum_next + 1;
    }
    if done == DONE_NOTHING {
        // SAFETY: a static message.
        emsg_nofold();
    }
    if had_visual {
        // SAFETY: the caller's promise.
        redraw_curbuf_later(UPD_INVERTED);
    }
}

/// Open fold for current window at position "pos".
/// Repeat "count" times.
///
/// # Safety
/// The current window must be live.
pub unsafe extern "C" fn open_fold(pos: pos_T, count: c_int) {
    // SAFETY: the caller's promise.
    set_fold_repeat(pos, count, 1);
}

/// Open fold for current window at position `pos` recursively.
///
/// # Safety
/// The current window must be live.
pub unsafe fn open_fold_recurse(pos: pos_T) {
    // SAFETY: the caller's promise.
    set_manual_fold(pos, true, true, None);
}

/// Open folds until the cursor line is not in a closed fold.
///
/// # Safety
/// The current window must be live.
pub unsafe fn fold_open_cursor() {
    checkupdate(cur_win());
    if has_any_folding(cur_win()) == 0 {
        return;
    }
    loop {
        let mut done: c_int = DONE_NOTHING;
        set_manual_fold(cur_win().w_cursor, true, false, Some(&mut done));
        // The loop's only exit. Each pass opens the outermost fold still
        // closed over the cursor, so once a pass opens nothing there is
        // nothing left to open.
        if done & DONE_ACTION == 0 {
            break;
        }
    }
}

/// Set new foldlevel for current window.
///
/// # Safety
/// The current window must be live.
pub unsafe fn new_fold_level() {
    new_fold_level_win(cur_win());
    if !(foldmethod_is_diff(cur_win()) && cur_win().w_onebuf_opt.wo_scb != 0) {
        return;
    }
    // 'scrollbind' in a diff: the other diffed windows follow.
    for mut win in windows_in_tab(cur_tab()) {
        if !win.is_current() && foldmethod_is_diff(win) && win.w_onebuf_opt.wo_scb != 0 {
            win.w_onebuf_opt.wo_fdl = cur_win().w_onebuf_opt.wo_fdl;
            new_fold_level_win(win);
        }
    }
}

/// Hand `win`'s toplevel folds back to 'foldlevel'.
pub(super) fn new_fold_level_win(mut win: Win) {
    checkupdate(win);
    if win.w_fold_manual {
        for fold in window_folds(win).folds() {
            fold.set_flags(FD_LEVEL);
        }
        win.w_fold_manual = false;
    }
    changed_window_setting(win);
}

/// Apply 'foldlevel' to all folds that don't contain the cursor.
///
/// # Safety
/// The current window must be live.
pub unsafe fn fold_check_close() {
    // SAFETY: 'foldclose' is a NUL-terminated option string.
    if unsafe { *p_fcl.get() } as c_int == NUL {
        return;
    }
    // SAFETY: the caller's promise.
    checkupdate(cur_win());
    let changed = close_folds_off_cursor(
        window_folds(cur_win()),
        cur_win().w_cursor.lnum,
        cur_win().w_onebuf_opt.wo_fdl as c_int,
    );
    if changed {
        changed_window_setting(cur_win());
    }
}

/// Hand every manually opened fold in `folds` that does *not* contain `lnum`
/// back to 'foldlevel' — what `'foldclose'` = "all" asks for.
///
/// Returns whether anything changed.
pub(super) fn close_folds_off_cursor(folds: FoldList, lnum: linenr_T, level: c_int) -> bool {
    let mut changed = false;
    for fold in folds.folds() {
        if !fold.is(FD_OPEN) {
            continue;
        }
        if level <= 0 && (lnum < fold.top() || lnum >= fold.top() + fold.len()) {
            fold.set_flags(FD_LEVEL);
            changed = true;
        } else {
            changed |= close_folds_off_cursor(fold.nested(), lnum - fold.top(), level - 1);
        }
    }
    changed
}

/// Returns true if it's allowed to manually create or delete a fold or,
///          give an error message and return false if not.
///
/// # Safety
/// The current window must be live.
pub unsafe fn fold_manual_allowed(create: bool) -> c_int {
    // SAFETY: the caller's promise.
    if foldmethod_is_manual(cur_win()) || foldmethod_is_marker(cur_win()) {
        return 1;
    }
    let msg = if create {
        c"E350: Cannot create fold with current 'foldmethod'"
    } else {
        c"E351: Cannot delete fold with current 'foldmethod'"
    };
    // SAFETY: a static message.
    unsafe { emsg(gettext(msg.as_ptr())) };
    0
}

/// Create a fold from line "start" to line "end" (inclusive) in window `wp`.
///
/// # Safety
/// `wp` must be a live window with a live buffer.
pub unsafe fn fold_create(wp: Win, start_pos: pos_T, end_pos: pos_T) {
    // SAFETY: the caller's promise -- a live window.
    let mut win = wp;
    let (start, end) = if start_pos.lnum > end_pos.lnum {
        (end_pos, start_pos)
    } else {
        (start_pos, end_pos)
    };
    if foldmethod_is_marker(win) {
        // With 'foldmethod' = "marker" the fold lives in the buffer text.
        // SAFETY: the caller's promise.
        unsafe { fold_create_markers(wp, start, end) };
        return;
    }
    checkupdate(win);

    // Descend to the innermost list the new fold fits inside, tracking
    // whether it will inherit a closed state from the folds above it. `top`
    // and `bot` become relative to that list as we go.
    let mut use_level = false;
    let mut closed = false;
    let mut level = 0;
    let mut top = start.lnum;
    let mut bot = end.lnum;
    let mut folds = window_folds(win);
    let mut i = 0;
    if !folds.is_empty() {
        loop {
            match folds.find(top) {
                Ok(idx) => {
                    i = idx;
                    let fold = folds.at(idx);
                    if fold.top() + fold.len() <= bot {
                        // The new fold reaches past this one, so it belongs
                        // beside it rather than inside it.
                        break;
                    }
                    top -= fold.top();
                    bot -= fold.top();
                    if use_level || fold.is(FD_LEVEL) {
                        use_level = true;
                        if level as OptInt >= win.w_onebuf_opt.wo_fdl {
                            closed = true;
                        }
                    } else if fold.is(FD_CLOSED) {
                        closed = true;
                    }
                    level += 1;
                    folds = fold.nested();
                }
                Err(idx) => {
                    i = idx;
                    break;
                }
            }
        }
        if folds.is_empty() {
            i = 0;
        }
    }

    // SAFETY: a live fold list.
    unsafe { ga_grow(folds.gap(), 1) };
    let fold = folds.at(i);
    let mut nested = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ptr::null_mut(),
    };
    // SAFETY: `nested` is a local; this is what makes it a fold list.
    unsafe { ga_init(&raw mut nested, size_of::<fold_T>() as c_int, 10) };

    // The folds the new one swallows whole become its children.
    let mut cont = 0;
    while i + cont < folds.len() && fold.offset(cont).top() <= bot {
        cont += 1;
    }
    if cont > 0 {
        // SAFETY: `nested` is a fold list.
        unsafe { ga_grow(&raw mut nested, cont) };
        // SAFETY: `nested` is a fold list with room for `cont` entries.
        let inner = unsafe { FoldList::new(&raw mut nested) };
        // A fold cannot start or end inside one of its children.
        top = top.min(fold.top());
        bot = bot.max(fold.offset(cont - 1).last());
        // SAFETY: `cont` entries starting at `i`, all inside `folds`, into
        // the room `ga_grow` just made in `nested`.
        unsafe { ptr::copy(fold.entry(), inner.at(0).entry(), cont as usize) };
        inner.set_len(inner.len() + cont);
        i += cont;
        for child in inner.folds() {
            child.set_top(child.top() - top);
        }
    }
    if i < folds.len() {
        // SAFETY: the folds below the new one slide up by one, into the room
        // `ga_grow` made.
        let tail = (folds.len() - i) as usize;
        unsafe { ptr::copy(folds.at(i).entry(), fold.entry().add(1), tail) };
    }
    folds.set_len(folds.len() + 1 - cont);
    // SAFETY: the entry is inside the list; `nested`'s storage passes to it.
    unsafe { (*fold.entry()).fd_nested = nested };
    fold.set_top(top);
    fold.set_len(bot - top + 1);
    if use_level && !closed && (level as OptInt) < win.w_onebuf_opt.wo_fdl {
        // SAFETY: the caller's promise.
        unsafe { close_fold(start, 1) };
    }
    if !use_level {
        win.w_fold_manual = true;
    }
    fold.set_flags(FD_CLOSED);
    fold.set_small(None);
    changed_window_setting(win);
}

/// `start` — delete all folds from start to end when not 0
/// `end` — delete all folds from start to end when not 0
/// `recursive` — delete recursively if true
/// `had_visual` — true when Visual selection used
///
/// # Safety
/// `wp` must be a live window with a live buffer.
pub unsafe fn delete_fold(
    wp: *mut win_T,
    start: linenr_T,
    end: linenr_T,
    recursive: c_int,
    had_visual: bool,
) {
    let mut maybe_small = false;
    let mut level = 0;
    let mut lnum = start;
    let mut did_one = false;
    let mut first_lnum = MAXLNUM as linenr_T;
    let mut last_lnum: linenr_T = 0;
    // SAFETY: the caller's promise -- a live window.
    let win = unsafe { Win::new(wp) };
    checkupdate(win);
    while lnum <= end {
        let mut folds = window_folds(win);
        let mut found: Option<(FoldList, c_int, linenr_T)> = None;
        let mut lnum_off: linenr_T = 0;
        let mut use_level = false;
        // Descend to the innermost fold over `lnum`, stopping at the first
        // closed one — that is the fold the user is pointing at.
        while let Ok(i) = folds.find(lnum - lnum_off) {
            let fold = folds.at(i);
            found = Some((folds, i, lnum_off));
            // SAFETY: a live window, and one of its own folds.
            if unsafe { check_closed(win, fold, &mut use_level, level, &mut maybe_small, lnum_off) }
            {
                break;
            }
            folds = fold.nested();
            lnum_off += fold.top();
            level += 1;
        }
        let Some((found_folds, found_i, found_off)) = found else {
            lnum += 1;
            continue;
        };
        let fold = found_folds.at(found_i);
        lnum = fold.top() + fold.len() + found_off;
        // SAFETY: the caller's promise.
        if foldmethod_is_manual(win) {
            // SAFETY: `found_i` names an entry of `found_folds`.
            unsafe { delete_fold_entry(found_folds, found_i, recursive != 0) };
        } else {
            // 'foldmethod' is "marker": the fold only goes away once its
            // markers are gone from the buffer text.
            first_lnum = first_lnum.min(fold.top() + found_off);
            last_lnum = last_lnum.max(lnum);
            // SAFETY: the caller's promise, and one of `wp`'s own folds.
            if !did_one {
                parse_marker(win);
            }
            unsafe { delete_fold_markers(win, fold, recursive != 0, found_off) };
        }
        did_one = true;
        // SAFETY: the caller's promise.
        changed_window_setting(win);
    }
    if !did_one {
        // SAFETY: a static message, and a live buffer.
        emsg_nofold();
        if had_visual {
            // SAFETY: `win` is live, so its buffer is.
            unsafe { redraw_buf_later(win.w_buffer, UPD_INVERTED) };
        }
    } else {
        // SAFETY: the caller's promise.
        check_cursor_col(unsafe { Win::new(wp) });
    }
    if last_lnum > 0 {
        let num_changed = (last_lnum - first_lnum) as int64_t;
        // SAFETY: the caller's promise; the range is inside the buffer.
        let buf = win.w_buffer;
        unsafe { changed_lines(Buf::new(buf), first_lnum, 0, last_lnum, 0, false) };
        unsafe { buf_updates_send_changes(buf, first_lnum, num_changed, num_changed) };
    }
}

/// Open or close fold for current window at position `pos`.
/// Repeat "count" times.
///
pub(super) fn set_fold_repeat(pos: pos_T, count: c_int, do_open: c_int) {
    for n in 0..count {
        let mut done: c_int = DONE_NOTHING;
        set_manual_fold(pos, do_open != 0, false, Some(&mut done));
        if done & DONE_ACTION != 0 {
            continue;
        }
        if n == 0 && done & DONE_FOLD == 0 {
            // SAFETY: a static message.
            emsg_nofold();
        }
        break;
    }
}

/// Open or close the fold in the current window which contains "lnum".
/// Also does this for other windows in diff mode when needed.
///
/// `opening` — true when opening, false when closing
/// `recurse` — true when closing/opening recursive
///
pub(super) fn set_manual_fold(
    pos: pos_T,
    opening: bool,
    recurse: bool,
    donep: Option<&mut c_int>,
) -> linenr_T {
    if foldmethod_is_diff(cur_win()) && cur_win().w_onebuf_opt.wo_scb != 0 {
        // 'scrollbind' in a diff: the matching fold in the other windows.
        for win in windows_in_tab(cur_tab()) {
            if win.is_current() || !foldmethod_is_diff(win) || win.w_onebuf_opt.wo_scb == 0 {
                continue;
            }
            let dlnum = diff_lnum_win(cur_win().w_cursor.lnum, win);
            if dlnum != 0 {
                set_manual_fold_win(win, dlnum, opening, recurse, None);
            }
        }
    }
    set_manual_fold_win(cur_win(), pos.lnum, opening, recurse, donep)
}

/// Open or close the fold in window "wp" which contains "lnum".
/// "donep", when not NULL, points to flag that is set to DONE_FOLD when some
/// fold was found and to DONE_ACTION when some fold was opened or closed.
/// When "donep" is NULL give an error message when no fold was found for
/// "lnum", but only if "wp" is "curwin".
///
/// `opening` — true when opening, false when closing
/// `recurse` — true when closing/opening recursive
///
/// Returns the line number of the next line that could be closed.
///                 It's only valid when "opening" is true!
///
pub(super) fn set_manual_fold_win(
    mut win: Win,
    mut lnum: linenr_T,
    opening: bool,
    recurse: bool,
    donep: Option<&mut c_int>,
) -> linenr_T {
    let mut level = 0;
    let mut use_level = false;
    let mut found_fold = false;
    let mut found: Option<Fold> = None;
    let mut next = MAXLNUM as linenr_T;
    let mut off: linenr_T = 0;
    let mut done: c_int = 0;
    checkupdate(win);
    let mut folds = window_folds(win);
    loop {
        let i = match folds.find(lnum) {
            Ok(i) => i,
            Err(i) => {
                if i < folds.len() {
                    next = folds.at(i).top() + off;
                }
                break;
            }
        };
        let fold = folds.at(i);
        found_fold = true;
        if i + 1 < folds.len() {
            next = folds.at(i + 1).top() + off;
        }
        if use_level || fold.is(FD_LEVEL) {
            use_level = true;
            // SAFETY: the caller's promise.
            let foldlevel = win.w_onebuf_opt.wo_fdl;
            fold.set_flags(if level as OptInt >= foldlevel {
                FD_CLOSED
            } else {
                FD_OPEN
            });
            for child in fold.nested().folds() {
                child.set_flags(FD_LEVEL);
            }
        }
        if !opening && recurse {
            if !fold.is(FD_CLOSED) {
                done |= DONE_ACTION;
                fold.set_flags(FD_CLOSED);
            }
        } else if fold.is(FD_CLOSED) {
            if opening {
                fold.set_flags(FD_OPEN);
                done |= DONE_ACTION;
                if recurse {
                    fold_open_nested(fold);
                }
            }
            break;
        }
        found = Some(fold);
        folds = fold.nested();
        lnum -= fold.top();
        off += fold.top();
        level += 1;
    }
    if found_fold {
        if !opening && let Some(fold) = found {
            // Closing: the innermost fold that was already open closes.
            fold.set_flags(FD_CLOSED);
            done |= DONE_ACTION;
        }
        win.w_fold_manual = true;
        if done & DONE_ACTION != 0 {
            changed_window_setting(win);
        }
        done |= DONE_FOLD;
    } else if donep.is_none() && win.is_current() {
        emsg_nofold();
    }
    if let Some(out) = donep {
        *out |= done;
    }
    next
}

/// Open all folds nested inside `fold`, recursively.
pub(super) fn fold_open_nested(fold: Fold) {
    for child in fold.nested().folds() {
        fold_open_nested(child);
        child.set_flags(FD_OPEN);
    }
}

/// Check if a fold is closed and update the info needed to check nested folds.
///
/// `use_levelp` — true: outer fold had FD_LEVEL
/// `fold` — fold to check
/// `level` — folding depth
/// `maybe_smallp` — true: the outer fold had no `fd_small` answer yet
/// `lnum_off` — line number offset for fold.top()
/// Returns true if fold is closed
///
/// # Safety
/// `wp` must be a live window, and `fold` one of its folds at `lnum_off`.
pub(super) unsafe fn check_closed(
    win: Win,
    fold: Fold,
    use_levelp: &mut bool,
    level: c_int,
    maybe_smallp: &mut bool,
    lnum_off: linenr_T,
) -> bool {
    let mut closed = false;
    if *use_levelp || fold.is(FD_LEVEL) {
        *use_levelp = true;
        if level as OptInt >= win.w_onebuf_opt.wo_fdl {
            closed = true;
        }
    } else if fold.is(FD_CLOSED) {
        closed = true;
    }
    if fold.small().is_none() {
        *maybe_smallp = true;
    }
    if closed {
        if *maybe_smallp {
            fold.set_small(None);
        }
        // SAFETY: the caller's promise.
        unsafe { check_small(win, fold, lnum_off) };
        if fold.small() == Some(true) {
            // 'foldminlines' vetoes it: too short to be worth drawing closed.
            closed = false;
        }
    }
    closed
}

/// The tab page the editor is working in.
fn cur_tab() -> TabPage {
    // SAFETY: `curtab` is set from startup to exit.
    unsafe { TabPage::current() }
}

/// C's `emsg(_(e_nofold))`, which every command that found no fold gives.
fn emsg_nofold() {
    // SAFETY: a static, translated message.
    unsafe { emsg(gettext(e_nofold.get())) };
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
