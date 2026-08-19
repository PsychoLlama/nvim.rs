use crate::charset::skipwhite;
use crate::diff::diff_infold;
use crate::drawscreen::redraw_win_range_later;
use crate::eval::eval_foldexpr;
use crate::eval::vars::set_vim_var_nr;
use crate::indent::{get_indent_buf, get_sw_value};
use crate::main::{KeyTyped, curbuf, curwin, diff_context, got_int};
use crate::memline::ml_get_buf;
use crate::r#move::changed_window_setting;
use crate::os::input::line_breakcheck;
use crate::strings::vim_strchr;
use crate::syntax::syn_get_foldlevel;
use core::ffi::{c_char, c_int};
use core::ptr;

use super::adjust::*;
use super::marker::*;
use super::*;
use crate::pos::MAXLNUM;

// The fold-level strategy is dispatched by comparing function addresses, as
// the C code did; the helper spells the address comparison out so the intent
// survives the `unpredictable_function_pointer_comparisons` lint.
pub(super) fn getlevel_is(getlevel: LevelGetter, f: unsafe fn(*mut fline_T)) -> bool {
    getlevel.is_some_and(|g| ::core::ptr::fn_addr_eq(g, f))
}

/// Update the folding for window "wp", at least from lines "top" to "bot".
/// IEMS = "Indent Expr Marker Syntax"
pub(super) unsafe fn foldUpdateIEMS(wp: *mut win_T, mut top: linenr_T, mut bot: linenr_T) {
    if invalid_top.get() != 0 {
        return;
    }
    if (*wp).w_foldinvalid {
        top = 1;
        bot = (*(*wp).w_buffer).b_ml.ml_line_count;
        (*wp).w_foldinvalid = false;
        setSmallMaybe(&raw mut (*wp).w_folds);
    }
    if foldmethodIsDiff(wp) {
        if top > diff_context.get() as linenr_T {
            top = (top as c_int - diff_context.get()) as linenr_T;
        } else {
            top = 1;
        }
        bot = (bot as c_int + diff_context.get()) as linenr_T;
    }
    top = if top < (*(*wp).w_buffer).b_ml.ml_line_count {
        top
    } else {
        (*(*wp).w_buffer).b_ml.ml_line_count
    };
    let mut fline: fline_T = fline_T {
        wp: ptr::null_mut(),
        lnum: 0,
        off: 0,
        lnum_save: 0,
        lvl: 0,
        lvl_next: 0,
        start: 0,
        end: 0,
        had_end: 0,
    };
    fold_changed.set(false);
    fline.wp = wp;
    fline.off = 0;
    fline.lvl = 0;
    fline.lvl_next = -1;
    fline.start = 0;
    fline.end = MAX_LEVEL + 1;
    fline.had_end = MAX_LEVEL + 1;
    invalid_top.set(top);
    invalid_bot.set(bot);
    let mut getlevel: LevelGetter = None;
    if foldmethodIsMarker(wp) {
        getlevel = Some(foldlevelMarker as unsafe fn(*mut fline_T) -> ());
        parseMarker(wp);
        if top > 1 {
            let level: c_int = foldLevelWin(wp, top - 1);
            fline.lnum = top - 1;
            fline.lvl = level;
            getlevel.expect("non-null function pointer")(&raw mut fline);
            if fline.lvl > level {
                fline.lvl = level - (fline.lvl - fline.lvl_next);
            } else {
                fline.lvl = fline.lvl_next;
            }
        }
        fline.lnum = top;
        getlevel.expect("non-null function pointer")(&raw mut fline);
    } else {
        fline.lnum = top;
        if foldmethodIsExpr(wp) {
            getlevel = Some(foldlevelExpr as unsafe fn(*mut fline_T) -> ());
            if top > 1 {
                fline.lnum -= 1;
            }
        } else if foldmethodIsSyntax(wp) {
            getlevel = Some(foldlevelSyntax as unsafe fn(*mut fline_T) -> ());
        } else if foldmethodIsDiff(wp) {
            getlevel = Some(foldlevelDiff as unsafe fn(*mut fline_T) -> ());
        } else {
            getlevel = Some(foldlevelIndent as unsafe fn(*mut fline_T) -> ());
            if top > 1 {
                fline.lnum -= 1;
            }
        }
        fline.lvl = -1;
        while !got_int.get() {
            fline.lvl_next = -1;
            getlevel.expect("non-null function pointer")(&raw mut fline);
            if fline.lvl >= 0 {
                break;
            }
            fline.lnum -= 1;
        }
    }
    if getlevel_is(getlevel, foldlevelSyntax) {
        let mut gap: *mut garray_T = &raw mut (*wp).w_folds;
        let mut fpn: *mut fold_T = ptr::null_mut();
        let mut current_fdl: c_int = 0;
        let mut fold_start_lnum: linenr_T = 0;
        let mut lnum_rel: linenr_T = fline.lnum;
        while current_fdl < fline.lvl {
            if !foldFind(gap, lnum_rel, &raw mut fpn) {
                break;
            }
            current_fdl += 1;
            fold_start_lnum += (*fpn).fd_top;
            gap = &raw mut (*fpn).fd_nested;
            lnum_rel -= (*fpn).fd_top;
        }
        if !fpn.is_null() && current_fdl == fline.lvl {
            let mut fold_end_lnum: linenr_T = fold_start_lnum + (*fpn).fd_len;
            bot = if bot > fold_end_lnum {
                bot
            } else {
                fold_end_lnum
            };
        }
    }
    let mut start: linenr_T = fline.lnum;
    let mut end: linenr_T = bot;
    if start > end && end < (*(*wp).w_buffer).b_ml.ml_line_count {
        end = start;
    }
    let mut fp: *mut fold_T = ptr::null_mut();
    while !got_int.get() {
        if fline.lnum > (*(*wp).w_buffer).b_ml.ml_line_count {
            break;
        }
        if fline.lnum > end {
            if !getlevel_is(getlevel, foldlevelMarker)
                && !getlevel_is(getlevel, foldlevelSyntax)
                && !getlevel_is(getlevel, foldlevelExpr)
            {
                break;
            }
            if start <= end
                && foldFind(&raw mut (*wp).w_folds, end, &raw mut fp)
                && (*fp).fd_top + (*fp).fd_len - 1 > end
                || fline.lvl == 0
                    && foldFind(&raw mut (*wp).w_folds, fline.lnum, &raw mut fp)
                    && (*fp).fd_top < fline.lnum
            {
                end = (*fp).fd_top + (*fp).fd_len - 1;
            } else {
                if !(getlevel_is(getlevel, foldlevelSyntax)
                    && foldLevelWin(wp, fline.lnum) != fline.lvl)
                {
                    break;
                }
                end = fline.lnum;
            }
        }
        if fline.lvl > 0 {
            invalid_top.set(fline.lnum);
            invalid_bot.set(end);
            end = foldUpdateIEMSRecurse(
                &raw mut (*wp).w_folds,
                1,
                start,
                &raw mut fline,
                getlevel,
                end,
                FD_LEVEL as c_int as c_char,
            );
            start = fline.lnum;
        } else {
            if fline.lnum == (*(*wp).w_buffer).b_ml.ml_line_count {
                break;
            }
            fline.lnum += 1;
            fline.lvl = fline.lvl_next;
            getlevel.expect("non-null function pointer")(&raw mut fline);
        }
    }
    foldRemove(wp, &raw mut (*wp).w_folds, start, end);
    if fold_changed.get() && (*wp).w_onebuf_opt.wo_fen != 0 {
        changed_window_setting(wp);
    }
    if end != bot {
        redraw_win_range_later(wp, top, end);
    }
    invalid_top.set(0);
}

/// Update a fold that starts at "flp->lnum".  At this line there is always a
/// valid foldlevel, and its level >= "level".
///
/// "flp" is valid for "flp->lnum" when called and it's valid when returning.
/// "flp->lnum" is set to the lnum just below the fold, if it ends before
/// "bot", it's "bot" plus one if the fold continues and it's bigger when using
/// the marker method and a text change made following folds to change.
/// When returning, "flp->lnum_save" is the line number that was used to get
/// the level when the level at "flp->lnum" is invalid.
/// Remove any folds from "startlnum" up to here at this level.
/// Recursively update nested folds.
/// Below line "bot" there are no changes in the text.
/// "flp->lnum", "flp->lnum_save" and "bot" are relative to the start of the
/// outer fold.
/// "flp->off" is the offset to the real line number in the buffer.
///
/// All this would be a lot simpler if all folds in the range would be deleted
/// and then created again.  But we would lose all information about the
/// folds, even when making changes that don't affect the folding (e.g. "vj~").
///
/// `topflags` — containing fold flags
///
/// Returns bot, which may have been increased for lines that also need to be
/// updated as a result of a detected change in the fold.
pub(super) unsafe fn foldUpdateIEMSRecurse(
    gap: *mut garray_T,
    level: c_int,
    startlnum: linenr_T,
    flp: *mut fline_T,
    mut getlevel: LevelGetter,
    mut bot: linenr_T,
    topflags: c_char,
) -> linenr_T {
    let mut fp: *mut fold_T = ptr::null_mut();
    if getlevel_is(getlevel, foldlevelMarker)
        && (*flp).start <= (*flp).lvl - level
        && (*flp).lvl > 0
    {
        foldFind(gap, startlnum - 1, &raw mut fp);
        if !fp.is_null() && (fp >= folds_end(&*gap) || (*fp).fd_top >= startlnum) {
            fp = ptr::null_mut();
        }
    }
    let mut fp2: *mut fold_T = ptr::null_mut();
    let mut lvl: c_int = level;
    let mut startlnum2: linenr_T = startlnum;
    let firstlnum: linenr_T = (*flp).lnum;
    let mut finish: bool = false;
    let linecount: linenr_T = (*(*(*flp).wp).w_buffer).b_ml.ml_line_count - (*flp).off;
    (*flp).lnum_save = (*flp).lnum;
    while !got_int.get() {
        line_breakcheck();
        lvl = if (*flp).lvl < 20 { (*flp).lvl } else { 20 };
        if (*flp).lnum > firstlnum && (level > lvl - (*flp).start || level >= (*flp).had_end) {
            lvl = 0;
        }
        if (*flp).lnum > bot && !finish && !fp.is_null() {
            if !getlevel_is(getlevel, foldlevelMarker)
                && !getlevel_is(getlevel, foldlevelExpr)
                && !getlevel_is(getlevel, foldlevelSyntax)
            {
                break;
            }
            let mut i: c_int = 0;
            fp2 = fp;
            if lvl >= level {
                let mut ll: c_int = (*flp).lnum as c_int - (*fp).fd_top as c_int;
                while foldFind(&raw mut (*fp2).fd_nested, ll as linenr_T, &raw mut fp2) {
                    i += 1;
                    ll -= (*fp2).fd_top as c_int;
                }
            }
            if lvl < level + i {
                foldFind(
                    &raw mut (*fp).fd_nested,
                    (*flp).lnum - (*fp).fd_top,
                    &raw mut fp2,
                );
                if !fp2.is_null() {
                    bot = (*fp2).fd_top + (*fp2).fd_len - 1 + (*fp).fd_top;
                }
            } else {
                if !((*fp).fd_top + (*fp).fd_len <= (*flp).lnum && lvl >= level) {
                    break;
                }
                finish = true;
            }
        }
        if fp.is_null()
            && (lvl != level
                || (*flp).lnum_save >= bot
                || (*flp).start != 0
                || (*flp).had_end <= MAX_LEVEL
                || (*flp).lnum == linecount)
        {
            while !got_int.get() {
                let mut concat: c_int = if (*flp).start != 0 || (*flp).had_end <= MAX_LEVEL {
                    0
                } else {
                    1
                };
                if (*gap).ga_len > 0
                    && (foldFind(gap, startlnum, &raw mut fp)
                        || fp < folds_end(&*gap) && (*fp).fd_top <= firstlnum
                        || foldFind(gap, firstlnum - concat as linenr_T, &raw mut fp)
                        || fp < folds_end(&*gap)
                            && (lvl < level && (*fp).fd_top < (*flp).lnum
                                || lvl >= level && (*fp).fd_top <= (*flp).lnum_save))
                {
                    if (*fp).fd_top + (*fp).fd_len + concat as linenr_T > firstlnum {
                        if (*fp).fd_top != firstlnum {
                            if (*fp).fd_top >= startlnum {
                                if (*fp).fd_top > firstlnum {
                                    foldMarkAdjustRecurse(
                                        &raw mut (*fp).fd_nested,
                                        0,
                                        MAXLNUM as c_int as linenr_T,
                                        (*fp).fd_top - firstlnum,
                                        0,
                                    );
                                } else {
                                    foldMarkAdjustRecurse(
                                        &raw mut (*fp).fd_nested,
                                        0,
                                        firstlnum - (*fp).fd_top - 1,
                                        MAXLNUM as c_int as linenr_T,
                                        (*fp).fd_top - firstlnum,
                                    );
                                }
                                (*fp).fd_len += (*fp).fd_top - firstlnum;
                                (*fp).fd_top = firstlnum;
                                (*fp).fd_small = None;
                                fold_changed.set(true);
                            } else if (*flp).start != 0 && lvl == level || firstlnum != startlnum {
                                let mut breakstart: linenr_T = 0;
                                let mut breakend: linenr_T = 0;
                                if firstlnum != startlnum {
                                    breakstart = startlnum;
                                    breakend = firstlnum;
                                } else {
                                    breakstart = (*flp).lnum;
                                    breakend = (*flp).lnum;
                                }
                                foldRemove(
                                    (*flp).wp,
                                    &raw mut (*fp).fd_nested,
                                    breakstart - (*fp).fd_top,
                                    breakend - (*fp).fd_top,
                                );
                                let mut i_0: c_int = fold_index(&*gap, fp);
                                foldSplit(
                                    (*(*flp).wp).w_buffer,
                                    gap,
                                    i_0,
                                    breakstart,
                                    breakend - 1,
                                );
                                fp = fold_at(&*gap, i_0).offset(1);
                                if getlevel_is(getlevel, foldlevelMarker)
                                    || getlevel_is(getlevel, foldlevelExpr)
                                    || getlevel_is(getlevel, foldlevelSyntax)
                                {
                                    finish = true;
                                }
                            }
                        }
                        if (*fp).fd_top == startlnum && concat != 0 {
                            let mut i_1: c_int = fold_index(&*gap, fp);
                            if i_1 != 0 {
                                fp2 = fp.offset(-1);
                                if (*fp2).fd_top + (*fp2).fd_len == (*fp).fd_top {
                                    foldMerge(fp2, gap, fp);
                                    fp = fp2;
                                }
                            }
                        }
                        break;
                    } else if (*fp).fd_top >= startlnum {
                        deleteFoldEntry(gap, fold_index(&*gap, fp), true);
                    } else {
                        (*fp).fd_len = startlnum - (*fp).fd_top;
                        foldMarkAdjustRecurse(
                            &raw mut (*fp).fd_nested,
                            (*fp).fd_len,
                            MAXLNUM as c_int as linenr_T,
                            MAXLNUM as c_int as linenr_T,
                            0,
                        );
                        fold_changed.set(true);
                    }
                } else {
                    let mut i_2: c_int = 0;
                    if (*gap).ga_len == 0 {
                        i_2 = 0;
                    } else {
                        i_2 = fold_index(&*gap, fp);
                    }
                    foldInsert(gap, i_2);
                    fp = fold_at(&*gap, i_2);
                    (*fp).fd_top = firstlnum;
                    (*fp).fd_len = bot - firstlnum + 1;
                    if topflags as c_int == FD_OPEN as c_int {
                        (*(*flp).wp).w_fold_manual = true;
                        (*fp).fd_flags = FD_OPEN as c_int as c_char;
                    } else if i_2 <= 0 {
                        (*fp).fd_flags = topflags;
                        if topflags as c_int != FD_LEVEL as c_int {
                            (*(*flp).wp).w_fold_manual = true;
                        }
                    } else {
                        (*fp).fd_flags = (*fp.offset(-1)).fd_flags;
                    }
                    (*fp).fd_small = None;
                    if getlevel_is(getlevel, foldlevelMarker)
                        || getlevel_is(getlevel, foldlevelExpr)
                        || getlevel_is(getlevel, foldlevelSyntax)
                    {
                        finish = true;
                    }
                    fold_changed.set(true);
                    break;
                }
            }
        }
        if lvl < level || (*flp).lnum > linecount {
            break;
        }
        if lvl > level && !fp.is_null() {
            bot = if bot > (*flp).lnum { bot } else { (*flp).lnum };
            (*flp).lnum = (*flp).lnum_save - (*fp).fd_top;
            (*flp).off += (*fp).fd_top;
            let mut i_3: c_int = fold_index(&*gap, fp);
            bot = foldUpdateIEMSRecurse(
                &raw mut (*fp).fd_nested,
                level + 1,
                startlnum2 - (*fp).fd_top,
                flp,
                getlevel,
                bot - (*fp).fd_top,
                (*fp).fd_flags,
            );
            fp = fold_at(&*gap, i_3);
            (*flp).lnum += (*fp).fd_top;
            (*flp).lnum_save += (*fp).fd_top;
            (*flp).off -= (*fp).fd_top;
            bot += (*fp).fd_top;
            startlnum2 = (*flp).lnum;
        } else {
            (*flp).lnum = (*flp).lnum_save;
            let mut ll_0: c_int = (*flp).lnum as c_int + 1;
            while !got_int.get() {
                prev_lnum.set((*flp).lnum);
                prev_lnum_lvl.set((*flp).lvl);
                (*flp).lnum += 1;
                if (*flp).lnum > linecount {
                    break;
                }
                (*flp).lvl = (*flp).lvl_next;
                getlevel.expect("non-null function pointer")(flp);
                if (*flp).lvl >= 0 || (*flp).had_end <= MAX_LEVEL {
                    break;
                }
            }
            prev_lnum.set(0);
            if (*flp).lnum > linecount {
                break;
            }
            (*flp).lnum_save = (*flp).lnum;
            (*flp).lnum = ll_0 as linenr_T;
        }
    }
    if fp.is_null() {
        return bot;
    }
    if (*fp).fd_len < (*flp).lnum - (*fp).fd_top {
        (*fp).fd_len = (*flp).lnum - (*fp).fd_top;
        (*fp).fd_small = None;
        fold_changed.set(true);
    } else if (*fp).fd_top + (*fp).fd_len > linecount {
        (*fp).fd_len = linecount - (*fp).fd_top + 1;
    }
    foldRemove(
        (*flp).wp,
        &raw mut (*fp).fd_nested,
        startlnum2 - (*fp).fd_top,
        (*flp).lnum - 1 - (*fp).fd_top,
    );
    if lvl < level && (*fp).fd_len != (*flp).lnum - (*fp).fd_top {
        if (*fp).fd_top + (*fp).fd_len - 1 > bot {
            if getlevel_is(getlevel, foldlevelMarker)
                || getlevel_is(getlevel, foldlevelExpr)
                || getlevel_is(getlevel, foldlevelSyntax)
            {
                bot = (*fp).fd_top + (*fp).fd_len - 1;
                (*fp).fd_len = (*flp).lnum - (*fp).fd_top;
            } else {
                let mut i_4: c_int = fold_index(&*gap, fp);
                foldSplit((*(*flp).wp).w_buffer, gap, i_4, (*flp).lnum, bot);
                fp = fold_at(&*gap, i_4);
            }
        } else {
            (*fp).fd_len = (*flp).lnum - (*fp).fd_top;
        }
        fold_changed.set(true);
    }
    loop {
        fp2 = fp.offset(1);
        if fp2 >= folds_end(&*gap) || (*fp2).fd_top > (*flp).lnum {
            break;
        }
        if (*fp2).fd_top + (*fp2).fd_len > (*flp).lnum {
            if (*fp2).fd_top < (*flp).lnum {
                foldMarkAdjustRecurse(
                    &raw mut (*fp2).fd_nested,
                    0,
                    (*flp).lnum - (*fp2).fd_top - 1,
                    MAXLNUM as c_int as linenr_T,
                    (*fp2).fd_top - (*flp).lnum,
                );
                (*fp2).fd_len -= (*flp).lnum - (*fp2).fd_top;
                (*fp2).fd_top = (*flp).lnum;
                fold_changed.set(true);
            }
            if lvl >= level {
                foldMerge(fp, gap, fp2);
            }
            break;
        } else {
            fold_changed.set(true);
            deleteFoldEntry(gap, fold_index(&*gap, fp2), true);
        }
    }
    bot = if bot > (*flp).lnum - 1 {
        bot
    } else {
        (*flp).lnum - 1
    };
    bot
}

/// Low level function to get the foldlevel for the "indent" method.
/// Doesn't use any caching.
///
/// Returns a level of -1 if the foldlevel depends on surrounding lines.
pub(super) unsafe fn foldlevelIndent(mut flp: *mut fline_T) {
    let mut lnum: linenr_T = (*flp).lnum + (*flp).off;
    let mut buf: *mut buf_T = (*(*flp).wp).w_buffer;
    let mut s: *mut c_char = skipwhite(ml_get_buf(buf, lnum));
    if *s as c_int == NUL
        || !vim_strchr((*(*flp).wp).w_onebuf_opt.wo_fdi, *s as uint8_t as c_int).is_null()
    {
        (*flp).lvl = if lnum == 1 || lnum == (*buf).b_ml.ml_line_count {
            0
        } else {
            -1
        };
    } else {
        (*flp).lvl = get_indent_buf(buf, lnum) / get_sw_value(buf);
    }
    (*flp).lvl = if (*flp).lvl
        < (if 0 as OptInt > (*(*flp).wp).w_onebuf_opt.wo_fdn {
            0 as OptInt
        } else {
            (*(*flp).wp).w_onebuf_opt.wo_fdn
        }) as c_int
    {
        (*flp).lvl
    } else {
        (if 0 as OptInt > (*(*flp).wp).w_onebuf_opt.wo_fdn {
            0 as OptInt
        } else {
            (*(*flp).wp).w_onebuf_opt.wo_fdn
        }) as c_int
    };
}

/// Low level function to get the foldlevel for the "diff" method.
/// Doesn't use any caching.
pub(super) unsafe fn foldlevelDiff(mut flp: *mut fline_T) {
    (*flp).lvl = if diff_infold((*flp).wp, (*flp).lnum + (*flp).off) {
        1
    } else {
        0
    };
}

/// Low level function to get the foldlevel for the "expr" method.
/// Doesn't use any caching.
///
/// Returns a level of -1 if the foldlevel depends on surrounding lines.
pub(super) unsafe fn foldlevelExpr(mut flp: *mut fline_T) {
    let mut lnum: linenr_T = (*flp).lnum + (*flp).off;
    let mut win: *mut win_T = curwin.get();
    curwin.set((*flp).wp);
    curbuf.set((*(*flp).wp).w_buffer);
    set_vim_var_nr(Vv::Lnum, lnum as varnumber_T);
    (*flp).start = 0;
    (*flp).had_end = (*flp).end;
    (*flp).end = MAX_LEVEL + 1;
    if lnum <= 1 {
        (*flp).lvl = 0;
    }
    let save_keytyped: bool = KeyTyped.get();
    let mut c: c_int = 0;
    let n: c_int = eval_foldexpr((*flp).wp, &raw mut c);
    KeyTyped.set(save_keytyped);
    match c {
        97 => {
            if (*flp).lvl >= 0 {
                (*flp).lvl += n;
                (*flp).lvl_next = (*flp).lvl;
            }
            (*flp).start = n;
        }
        115 => {
            if (*flp).lvl >= 0 {
                if n > (*flp).lvl {
                    (*flp).lvl_next = 0;
                } else {
                    (*flp).lvl_next = (*flp).lvl - n;
                }
                (*flp).end = (*flp).lvl_next + 1;
            }
        }
        62 => {
            (*flp).lvl = n;
            (*flp).lvl_next = n;
            (*flp).start = 1;
        }
        60 => {
            (*flp).lvl_next = if (*flp).lvl < n - 1 {
                (*flp).lvl
            } else {
                n - 1
            };
            (*flp).end = n;
        }
        61 => {
            (*flp).lvl_next = (*flp).lvl;
        }
        _ => {
            if n < 0 {
                (*flp).lvl_next = (*flp).lvl;
            } else {
                (*flp).lvl_next = n;
            }
            (*flp).lvl = n;
        }
    }
    if (*flp).lvl < 0 {
        if lnum <= 1 {
            (*flp).lvl = 0;
            (*flp).lvl_next = 0;
        }
        if lnum == (*curbuf.get()).b_ml.ml_line_count {
            (*flp).lvl_next = 0;
        }
    }
    curwin.set(win);
    curbuf.set((*curwin.get()).w_buffer);
}

/// Low level function to get the foldlevel for the "syntax" method.
/// Doesn't use any caching.
pub(super) unsafe fn foldlevelSyntax(mut flp: *mut fline_T) {
    let mut lnum: linenr_T = (*flp).lnum + (*flp).off;
    (*flp).lvl = syn_get_foldlevel((*flp).wp, lnum);
    (*flp).start = 0;
    if lnum < (*(*(*flp).wp).w_buffer).b_ml.ml_line_count {
        let mut n: c_int = syn_get_foldlevel((*flp).wp, lnum + 1);
        if n > (*flp).lvl {
            (*flp).start = n - (*flp).lvl;
            (*flp).lvl = n;
        }
    }
}
