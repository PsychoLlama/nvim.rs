use crate::buffer_updates::buf_updates_send_changes;
use crate::change::changed_lines;
use crate::cursor::check_cursor_col;
use crate::diff::diff_lnum_win;
use crate::drawscreen::{UPD_INVERTED, redraw_buf_later, redraw_curbuf_later};
use crate::garray::{ga_grow, ga_init};
use crate::main::{curtab, curwin, firstwin, p_fcl};
use crate::message::emsg;
use crate::r#move::changed_window_setting;
use crate::os::cshim::{gettext, memmove};
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

use super::marker::*;
use super::*;
use crate::pos::MAXLNUM;

/// Close fold for current window at position "pos".
/// Repeat "count" times.
pub unsafe extern "C" fn closeFold(mut pos: pos_T, mut count: c_int) {
    setFoldRepeat(pos, count, 0);
}

/// Close fold for current window at position `pos` recursively.
pub unsafe fn closeFoldRecurse(mut pos: pos_T) {
    setManualFold(pos, false, true, ptr::null_mut());
}

///
/// Open or Close folds for current window in lines "first" to "last".
/// Used for "zo", "zO", "zc" and "zC" in Visual mode.
///
/// `opening` — true to open, false to close
/// `recurse` — true to do it recursively
/// `had_visual` — true when Visual selection used
pub unsafe fn opFoldRange(
    mut firstpos: pos_T,
    mut lastpos: pos_T,
    mut opening: c_int,
    mut recurse: c_int,
    mut had_visual: bool,
) {
    let mut done: c_int = DONE_NOTHING;
    let mut first: linenr_T = firstpos.lnum;
    let mut last: linenr_T = lastpos.lnum;
    let mut lnum_next: linenr_T = 0;
    let mut lnum: linenr_T = first;
    while lnum <= last {
        let mut temp: pos_T = pos_T {
            lnum: lnum,
            col: 0,
            coladd: 0,
        };
        lnum_next = lnum;
        if opening != 0 && recurse == 0 {
            hasFolding(curwin.get(), lnum, ptr::null_mut(), &raw mut lnum_next);
        }
        setManualFold(temp, opening != 0, recurse != 0, &raw mut done);
        if opening == 0 && recurse == 0 {
            hasFolding(curwin.get(), lnum, ptr::null_mut(), &raw mut lnum_next);
        }
        lnum = lnum_next + 1;
    }
    if done == DONE_NOTHING {
        emsg(gettext(e_nofold.get()));
    }
    if had_visual {
        redraw_curbuf_later(UPD_INVERTED);
    }
}

/// Open fold for current window at position "pos".
/// Repeat "count" times.
pub unsafe extern "C" fn openFold(mut pos: pos_T, mut count: c_int) {
    setFoldRepeat(pos, count, 1);
}

/// Open fold for current window at position `pos` recursively.
pub unsafe fn openFoldRecurse(mut pos: pos_T) {
    setManualFold(pos, true, true, ptr::null_mut());
}

/// Open folds until the cursor line is not in a closed fold.
pub unsafe fn foldOpenCursor() {
    checkupdate(curwin.get());
    if hasAnyFolding(curwin.get()) != 0 {
        loop {
            let mut done: c_int = DONE_NOTHING;
            setManualFold((*curwin.get()).w_cursor, true, false, &raw mut done);
            if done & DONE_ACTION == 0 {
                break;
            }
        }
    }
}

/// Set new foldlevel for current window.
pub unsafe fn newFoldLevel() {
    newFoldLevelWin(curwin.get());
    if foldmethodIsDiff(curwin.get()) && (*curwin.get()).w_onebuf_opt.wo_scb != 0 {
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if wp != curwin.get() && foldmethodIsDiff(wp) && (*wp).w_onebuf_opt.wo_scb != 0 {
                (*wp).w_onebuf_opt.wo_fdl = (*curwin.get()).w_onebuf_opt.wo_fdl;
                newFoldLevelWin(wp);
            }
            wp = (*wp).w_next;
        }
    }
}

pub(super) unsafe fn newFoldLevelWin(mut wp: *mut win_T) {
    checkupdate(wp);
    if (*wp).w_fold_manual {
        let mut fp: *mut fold_T = folds(&(*wp).w_folds);
        let mut i: c_int = 0;
        while i < (*wp).w_folds.ga_len {
            (*fp.offset(i as isize)).fd_flags = FD_LEVEL as c_int as c_char;
            i += 1;
        }
        (*wp).w_fold_manual = false;
    }
    changed_window_setting(wp);
}

/// Apply 'foldlevel' to all folds that don't contain the cursor.
pub unsafe fn foldCheckClose() {
    if *p_fcl.get() as c_int == NUL {
        return;
    }
    checkupdate(curwin.get());
    if checkCloseRec(
        &raw mut (*curwin.get()).w_folds,
        (*curwin.get()).w_cursor.lnum,
        (*curwin.get()).w_onebuf_opt.wo_fdl as c_int,
    ) {
        changed_window_setting(curwin.get());
    }
}

pub(super) unsafe fn checkCloseRec(
    mut gap: *mut garray_T,
    mut lnum: linenr_T,
    mut level: c_int,
) -> bool {
    let mut retval: bool = false;
    let mut fp: *mut fold_T = folds(&*gap);
    let mut i: c_int = 0;
    while i < (*gap).ga_len {
        if (*fp.offset(i as isize)).fd_flags as c_int == FD_OPEN as c_int {
            if level <= 0
                && (lnum < (*fp.offset(i as isize)).fd_top
                    || lnum >= (*fp.offset(i as isize)).fd_top + (*fp.offset(i as isize)).fd_len)
            {
                (*fp.offset(i as isize)).fd_flags = FD_LEVEL as c_int as c_char;
                retval = true;
            } else {
                retval = retval as c_int
                    | checkCloseRec(
                        &raw mut (*fp.offset(i as isize)).fd_nested,
                        lnum - (*fp.offset(i as isize)).fd_top,
                        level - 1,
                    ) as c_int
                    != 0;
            }
        }
        i += 1;
    }
    return retval;
}

/// Returns true if it's allowed to manually create or delete a fold or,
///          give an error message and return false if not.
pub unsafe fn foldManualAllowed(mut create: bool) -> c_int {
    if foldmethodIsManual(curwin.get()) || foldmethodIsMarker(curwin.get()) {
        return 1;
    }
    if create {
        emsg(gettext(
            c"E350: Cannot create fold with current 'foldmethod'".as_ptr(),
        ));
    } else {
        emsg(gettext(
            c"E351: Cannot delete fold with current 'foldmethod'".as_ptr(),
        ));
    }
    return 0;
}

/// Create a fold from line "start" to line "end" (inclusive) in the current
/// window.
pub unsafe fn foldCreate(mut wp: *mut win_T, mut start: pos_T, mut end: pos_T) {
    let mut use_level: bool = false;
    let mut closed: bool = false;
    let mut level: c_int = 0;
    let mut start_rel: pos_T = start;
    let mut end_rel: pos_T = end;
    if start.lnum > end.lnum {
        end = start_rel;
        start = end_rel;
        start_rel = start;
        end_rel = end;
    }
    if foldmethodIsMarker(wp) {
        foldCreateMarkers(wp, start, end);
        return;
    }
    checkupdate(wp);
    let mut i: c_int = 0;
    let mut gap: *mut garray_T = &raw mut (*wp).w_folds;
    if (*gap).ga_len == 0 {
        i = 0;
    } else {
        let mut fp: *mut fold_T = ptr::null_mut();
        while foldFind(gap, start_rel.lnum, &raw mut fp) {
            if (*fp).fd_top + (*fp).fd_len <= end_rel.lnum {
                break;
            }
            gap = &raw mut (*fp).fd_nested;
            start_rel.lnum -= (*fp).fd_top;
            end_rel.lnum -= (*fp).fd_top;
            if use_level || (*fp).fd_flags as c_int == FD_LEVEL as c_int {
                use_level = true;
                if level as OptInt >= (*wp).w_onebuf_opt.wo_fdl {
                    closed = true;
                }
            } else if (*fp).fd_flags as c_int == FD_CLOSED as c_int {
                closed = true;
            }
            level += 1;
        }
        if (*gap).ga_len == 0 {
            i = 0;
        } else {
            i = fold_index(&*gap, fp);
        }
    }
    ga_grow(gap, 1);
    let mut fp_0: *mut fold_T = fold_at(&*gap, i);
    let mut fold_ga: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ptr::null_mut(),
    };
    ga_init(&raw mut fold_ga, size_of::<fold_T>() as c_int, 10);
    let mut cont: c_int = 0;
    cont = 0;
    while i + cont < (*gap).ga_len {
        if (*fp_0.offset(cont as isize)).fd_top > end_rel.lnum {
            break;
        }
        cont += 1;
    }
    if cont > 0 {
        ga_grow(&raw mut fold_ga, cont);
        start_rel.lnum = if start_rel.lnum < (*fp_0).fd_top {
            start_rel.lnum
        } else {
            (*fp_0).fd_top
        };
        end_rel.lnum = if end_rel.lnum
            > (*fp_0.offset((cont - 1) as isize)).fd_top
                + (*fp_0.offset((cont - 1) as isize)).fd_len
                - 1
        {
            end_rel.lnum
        } else {
            (*fp_0.offset((cont - 1) as isize)).fd_top + (*fp_0.offset((cont - 1) as isize)).fd_len
                - 1
        };
        memmove(
            fold_ga.ga_data,
            fp_0 as *const c_void,
            size_of::<fold_T>().wrapping_mul(cont as size_t),
        );
        fold_ga.ga_len += cont;
        i += cont;
        let mut j: c_int = 0;
        while j < cont {
            (*fold_at(&fold_ga, j)).fd_top -= start_rel.lnum;
            j += 1;
        }
    }
    if i < (*gap).ga_len {
        memmove(
            fp_0.offset(1) as *mut c_void,
            fold_at(&*gap, i) as *const c_void,
            size_of::<fold_T>().wrapping_mul(((*gap).ga_len - i) as size_t),
        );
    }
    (*gap).ga_len = (*gap).ga_len + 1 - cont;
    (*fp_0).fd_nested = fold_ga;
    (*fp_0).fd_top = start_rel.lnum;
    (*fp_0).fd_len = end_rel.lnum - start_rel.lnum + 1;
    if use_level && !closed && (level as OptInt) < (*wp).w_onebuf_opt.wo_fdl {
        closeFold(start, 1);
    }
    if !use_level {
        (*wp).w_fold_manual = true;
    }
    (*fp_0).fd_flags = FD_CLOSED as c_int as c_char;
    (*fp_0).fd_small = None;
    changed_window_setting(wp);
}

/// `start` — delete all folds from start to end when not 0
/// `end` — delete all folds from start to end when not 0
/// `recursive` — delete recursively if true
/// `had_visual` — true when Visual selection used
pub unsafe fn deleteFold(
    wp: *mut win_T,
    start: linenr_T,
    end: linenr_T,
    recursive: c_int,
    had_visual: bool,
) {
    let mut found_fp: *mut fold_T = ptr::null_mut();
    let mut found_off: linenr_T = 0;
    let mut maybe_small: bool = false;
    let mut level: c_int = 0;
    let mut lnum: linenr_T = start;
    let mut did_one: bool = false;
    let mut first_lnum: linenr_T = MAXLNUM as c_int as linenr_T;
    let mut last_lnum: linenr_T = 0;
    checkupdate(wp);
    while lnum <= end {
        let mut gap: *mut garray_T = &raw mut (*wp).w_folds;
        let mut found_ga: *mut garray_T = ptr::null_mut();
        let mut lnum_off: linenr_T = 0;
        let mut use_level: bool = false;
        loop {
            let mut fp: *mut fold_T = ptr::null_mut();
            if !foldFind(gap, lnum - lnum_off, &raw mut fp) {
                break;
            }
            found_ga = gap;
            found_fp = fp;
            found_off = lnum_off;
            if check_closed(
                wp,
                fp,
                &raw mut use_level,
                level,
                &raw mut maybe_small,
                lnum_off,
            ) {
                break;
            }
            gap = &raw mut (*fp).fd_nested;
            lnum_off += (*fp).fd_top;
            level += 1;
        }
        if found_ga.is_null() {
            lnum += 1;
        } else {
            lnum = (*found_fp).fd_top + (*found_fp).fd_len + found_off;
            if foldmethodIsManual(wp) {
                deleteFoldEntry(found_ga, fold_index(&*found_ga, found_fp), recursive != 0);
            } else {
                first_lnum = if first_lnum < (*found_fp).fd_top + found_off {
                    first_lnum
                } else {
                    (*found_fp).fd_top + found_off
                };
                last_lnum = if last_lnum > lnum { last_lnum } else { lnum };
                if !did_one {
                    parseMarker(wp);
                }
                deleteFoldMarkers(wp, found_fp, recursive != 0, found_off);
            }
            did_one = true;
            changed_window_setting(wp);
        }
    }
    if !did_one {
        emsg(gettext(e_nofold.get()));
        if had_visual {
            redraw_buf_later((*wp).w_buffer, UPD_INVERTED);
        }
    } else {
        check_cursor_col(wp);
    }
    if last_lnum > 0 {
        changed_lines((*wp).w_buffer, first_lnum, 0, last_lnum, 0, false);
        let mut num_changed: int64_t = (last_lnum - first_lnum) as int64_t;
        buf_updates_send_changes((*wp).w_buffer, first_lnum, num_changed, num_changed);
    }
}

/// Open or close fold for current window at position `pos`.
/// Repeat "count" times.
pub(super) unsafe fn setFoldRepeat(mut pos: pos_T, mut count: c_int, mut do_open: c_int) {
    let mut n: c_int = 0;
    while n < count {
        let mut done: c_int = DONE_NOTHING;
        setManualFold(pos, do_open != 0, false, &raw mut done);
        if done & DONE_ACTION == 0 {
            if n == 0 && done & DONE_FOLD == 0 {
                emsg(gettext(e_nofold.get()));
            }
            break;
        } else {
            n += 1;
        }
    }
}

/// Open or close the fold in the current window which contains "lnum".
/// Also does this for other windows in diff mode when needed.
///
/// `opening` — true when opening, false when closing
/// `recurse` — true when closing/opening recursive
pub(super) unsafe fn setManualFold(
    mut pos: pos_T,
    mut opening: bool,
    mut recurse: bool,
    mut donep: *mut c_int,
) -> linenr_T {
    if foldmethodIsDiff(curwin.get()) && (*curwin.get()).w_onebuf_opt.wo_scb != 0 {
        let mut dlnum: linenr_T = 0;
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if wp != curwin.get() && foldmethodIsDiff(wp) && (*wp).w_onebuf_opt.wo_scb != 0 {
                dlnum = diff_lnum_win((*curwin.get()).w_cursor.lnum, wp);
                if dlnum != 0 {
                    setManualFoldWin(wp, dlnum, opening, recurse, ptr::null_mut());
                }
            }
            wp = (*wp).w_next;
        }
    }
    return setManualFoldWin(curwin.get(), pos.lnum, opening, recurse, donep);
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
pub(super) unsafe fn setManualFoldWin(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut opening: bool,
    mut recurse: bool,
    mut donep: *mut c_int,
) -> linenr_T {
    let mut fp: *mut fold_T = ptr::null_mut();
    let mut fp2: *mut fold_T = ptr::null_mut();
    let mut found: *mut fold_T = ptr::null_mut();
    let mut level: c_int = 0;
    let mut use_level: bool = false;
    let mut found_fold: bool = false;
    let mut next: linenr_T = MAXLNUM as c_int as linenr_T;
    let mut off: linenr_T = 0;
    let mut done: c_int = 0;
    checkupdate(wp);
    let mut gap: *mut garray_T = &raw mut (*wp).w_folds;
    loop {
        if !foldFind(gap, lnum, &raw mut fp) {
            if !fp.is_null() && fp < folds_end(&*gap) {
                next = (*fp).fd_top + off;
            }
            break;
        } else {
            found_fold = true;
            if fp.offset(1) < folds_end(&*gap) {
                next = (*fp.offset(1)).fd_top + off;
            }
            if use_level || (*fp).fd_flags as c_int == FD_LEVEL as c_int {
                use_level = true;
                (*fp).fd_flags = (if level as OptInt >= (*wp).w_onebuf_opt.wo_fdl {
                    FD_CLOSED as c_int
                } else {
                    FD_OPEN as c_int
                }) as c_char;
                fp2 = folds(&(*fp).fd_nested);
                let mut j: c_int = 0;
                while j < (*fp).fd_nested.ga_len {
                    (*fp2.offset(j as isize)).fd_flags = FD_LEVEL as c_int as c_char;
                    j += 1;
                }
            }
            if !opening && recurse {
                if (*fp).fd_flags as c_int != FD_CLOSED as c_int {
                    done |= DONE_ACTION;
                    (*fp).fd_flags = FD_CLOSED as c_int as c_char;
                }
            } else if (*fp).fd_flags as c_int == FD_CLOSED as c_int {
                if opening {
                    (*fp).fd_flags = FD_OPEN as c_int as c_char;
                    done |= DONE_ACTION;
                    if recurse {
                        foldOpenNested(fp);
                    }
                }
                break;
            }
            found = fp;
            gap = &raw mut (*fp).fd_nested;
            lnum -= (*fp).fd_top;
            off += (*fp).fd_top;
            level += 1;
        }
    }
    if found_fold {
        if !opening && !found.is_null() {
            (*found).fd_flags = FD_CLOSED as c_int as c_char;
            done |= DONE_ACTION;
        }
        (*wp).w_fold_manual = true;
        if done & DONE_ACTION != 0 {
            changed_window_setting(wp);
        }
        done |= DONE_FOLD;
    } else if donep.is_null() && wp == curwin.get() {
        emsg(gettext(e_nofold.get()));
    }
    if !donep.is_null() {
        *donep |= done;
    }
    return next;
}

/// Open all nested folds in fold "fpr" recursively.
pub(super) unsafe fn foldOpenNested(mut fpr: *mut fold_T) {
    let mut fp: *mut fold_T = folds(&(*fpr).fd_nested);
    let mut i: c_int = 0;
    while i < (*fpr).fd_nested.ga_len {
        foldOpenNested(fp.offset(i as isize));
        (*fp.offset(i as isize)).fd_flags = FD_OPEN as c_int as c_char;
        i += 1;
    }
}

/// Check if a fold is closed and update the info needed to check nested folds.
///
/// `use_levelp` — true: outer fold had FD_LEVEL
/// `fp` — fold to check
/// `level` — folding depth
/// `maybe_smallp` — true: the outer fold had no `fd_small` answer yet
/// `lnum_off` — line number offset for fp->fd_top
/// Returns true if fold is closed
pub(super) unsafe fn check_closed(
    wp: *mut win_T,
    fp: *mut fold_T,
    use_levelp: *mut bool,
    level: c_int,
    maybe_smallp: *mut bool,
    lnum_off: linenr_T,
) -> bool {
    let mut closed: bool = false;
    if *use_levelp || (*fp).fd_flags as c_int == FD_LEVEL as c_int {
        *use_levelp = true;
        if level as OptInt >= (*wp).w_onebuf_opt.wo_fdl {
            closed = true;
        }
    } else if (*fp).fd_flags as c_int == FD_CLOSED as c_int {
        closed = true;
    }
    if (*fp).fd_small.is_none() {
        *maybe_smallp = true;
    }
    if closed {
        if *maybe_smallp {
            (*fp).fd_small = None;
        }
        checkSmall(wp, fp, lnum_off);
        if (*fp).fd_small == Some(true) {
            closed = false;
        }
    }
    return closed;
}
