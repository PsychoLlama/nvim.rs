//! The four *computed* fold methods — indent, expr, marker and syntax (plus
//! diff, which rides along) — and the incremental update that turns their
//! per-line levels back into a tree.
//!
//! Upstream calls the pair `foldUpdateIEMS`/`foldUpdateIEMSRecurse`, IEMS
//! being its initialism for those four methods; they are
//! [`fold_update_computed`] and [`fold_update_computed_recurse`] here.
//!
//! All of this would be a lot simpler if every fold in the changed range were
//! deleted and built again, but then a change that does not affect the
//! folding (`vj~`, say) would still throw away every open/closed state.

#![deny(unsafe_op_in_unsafe_fn)]

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
use core::ffi::c_int;

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

/// Whether `getlevel` is one of the three methods whose folds can end
/// somewhere the line levels alone do not say — so an update has to keep
/// looking past the changed range.
fn getlevel_is_open_ended(getlevel: LevelGetter) -> bool {
    getlevel_is(getlevel, foldlevel_marker)
        || getlevel_is(getlevel, foldlevel_expr)
        || getlevel_is(getlevel, foldlevel_syntax)
}

/// Update the folding for window "wp", at least from lines "top" to "bot".
///
/// # Safety
/// `wp` must be a live window with a live buffer.
pub(super) unsafe fn fold_update_computed(wp: *mut win_T, mut top: linenr_T, mut bot: linenr_T) {
    if invalid_top.get() != 0 {
        // Already updating this window; the recursion would fight itself.
        return;
    }
    // SAFETY: the caller's promise.
    unsafe {
        if (*wp).w_foldinvalid {
            top = 1;
            bot = (*(*wp).w_buffer).b_ml.ml_line_count;
            (*wp).w_foldinvalid = false;
            forget_small_flags(window_folds(wp));
        }
        if foldmethod_is_diff(wp) {
            // 'diffopt' context lines belong to the same fold as the change.
            top = (top - diff_context.get()).max(1);
            bot += diff_context.get();
        }
        top = top.min((*(*wp).w_buffer).b_ml.ml_line_count);
    }
    let mut fline = fline_T {
        wp,
        lnum: 0,
        off: 0,
        lnum_save: 0,
        lvl: 0,
        lvl_next: -1,
        start: 0,
        end: MAX_LEVEL + 1,
        had_end: MAX_LEVEL + 1,
    };
    fold_changed.set(false);
    invalid_top.set(top);
    invalid_bot.set(bot);

    // Pick the level getter and prime `fline` with a level for `top`.
    let getlevel: LevelGetter;
    // SAFETY: the caller's promise; `fline` is ours.
    unsafe {
        if foldmethod_is_marker(wp) {
            getlevel = Some(foldlevel_marker as unsafe fn(*mut fline_T) -> ());
            parse_marker(wp);
            if top > 1 {
                // The marker method needs the previous line's level, and the
                // markers on that line, before it can read `top`.
                let level = fold_level_win(wp, top - 1);
                fline.lnum = top - 1;
                fline.lvl = level;
                foldlevel_marker(&raw mut fline);
                fline.lvl = if fline.lvl > level {
                    level - (fline.lvl - fline.lvl_next)
                } else {
                    fline.lvl_next
                };
            }
            fline.lnum = top;
            foldlevel_marker(&raw mut fline);
        } else {
            fline.lnum = top;
            if foldmethod_is_expr(wp) {
                getlevel = Some(foldlevel_expr as unsafe fn(*mut fline_T) -> ());
                if top > 1 {
                    fline.lnum -= 1;
                }
            } else if foldmethod_is_syntax(wp) {
                getlevel = Some(foldlevel_syntax as unsafe fn(*mut fline_T) -> ());
            } else if foldmethod_is_diff(wp) {
                getlevel = Some(foldlevel_diff as unsafe fn(*mut fline_T) -> ());
            } else {
                getlevel = Some(foldlevel_indent as unsafe fn(*mut fline_T) -> ());
                if top > 1 {
                    fline.lnum -= 1;
                }
            }
            // Indent and expr can answer "undefined" (-1); walk back until a
            // line has a level of its own.
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
    }

    if getlevel_is(getlevel, foldlevel_syntax) {
        // A syntax fold can reach past `bot`, and the whole of it has to be
        // updated or its end is left dangling.
        // SAFETY: the caller's promise.
        let mut folds = unsafe { window_folds(wp) };
        let mut innermost = None;
        let mut current_fdl = 0;
        let mut fold_start_lnum: linenr_T = 0;
        let mut lnum_rel = fline.lnum;
        while current_fdl < fline.lvl {
            let Ok(i) = folds.find(lnum_rel) else { break };
            let fold = folds.at(i);
            current_fdl += 1;
            fold_start_lnum += fold.top();
            lnum_rel -= fold.top();
            innermost = Some(fold);
            folds = fold.nested();
        }
        if let Some(fold) = innermost
            && current_fdl == fline.lvl
        {
            bot = bot.max(fold_start_lnum + fold.len());
        }
    }

    let mut start = fline.lnum;
    let mut end = bot;
    // Re-read rather than kept: 'foldexpr' is user code, and it can change
    // the buffer under this loop.
    // SAFETY: the caller's promise.
    let line_count = || unsafe { (*(*wp).w_buffer).b_ml.ml_line_count };
    if start > end && end < line_count() {
        end = start;
    }
    while !got_int.get() {
        if fline.lnum > line_count() {
            break;
        }
        if fline.lnum > end {
            if !getlevel_is_open_ended(getlevel) {
                break;
            }
            // SAFETY: the caller's promise.
            let folds = unsafe { window_folds(wp) };
            // A fold that straddles the end of the range drags the range out
            // to cover the whole of it.
            let straddling = if start <= end
                && let Ok(i) = folds.find(end)
                && folds.at(i).last() > end
            {
                Some(folds.at(i))
            } else if fline.lvl == 0
                && let Ok(i) = folds.find(fline.lnum)
                && folds.at(i).top() < fline.lnum
            {
                Some(folds.at(i))
            } else {
                None
            };
            if let Some(fold) = straddling {
                end = fold.last();
            } else {
                // SAFETY: the caller's promise.
                if !(getlevel_is(getlevel, foldlevel_syntax)
                    && unsafe { fold_level_win(wp, fline.lnum) } != fline.lvl)
                {
                    break;
                }
                end = fline.lnum;
            }
        }
        if fline.lvl > 0 {
            invalid_top.set(fline.lnum);
            invalid_bot.set(end);
            // SAFETY: the caller's promise; `fline` is ours.
            end = unsafe {
                fold_update_computed_recurse(
                    window_folds(wp),
                    1,
                    start,
                    &raw mut fline,
                    getlevel,
                    end,
                    FD_LEVEL,
                )
            };
            start = fline.lnum;
        } else {
            if fline.lnum == line_count() {
                break;
            }
            fline.lnum += 1;
            fline.lvl = fline.lvl_next;
            // SAFETY: the caller's promise; `fline` is ours.
            unsafe { getlevel.expect("non-null function pointer")(&raw mut fline) };
        }
    }
    // SAFETY: the caller's promise.
    unsafe {
        fold_remove(wp, window_folds(wp), start, end);
        if fold_changed.get() && (*wp).w_onebuf_opt.wo_fen != 0 {
            changed_window_setting(wp);
        }
        if end != bot {
            redraw_win_range_later(wp, top, end);
        }
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
/// `topflags` — containing fold flags
///
/// Returns bot, which may have been increased for lines that also need to be
/// updated as a result of a detected change in the fold.
///
/// # Safety
/// `folds` must be the fold list at `level`, and `flp` a writable `fline_T`
/// naming a live window.
pub(super) unsafe fn fold_update_computed_recurse(
    folds: FoldList,
    level: c_int,
    startlnum: linenr_T,
    flp: *mut fline_T,
    getlevel: LevelGetter,
    mut bot: linenr_T,
    topflags: c_int,
) -> linenr_T {
    // The fold this call is building, once there is one. `None` means "not
    // started yet", which is what the whole first half of the loop is about.
    let mut fold: Option<Fold> = None;
    // SAFETY: the caller's promise.
    unsafe {
        if getlevel_is(getlevel, foldlevel_marker)
            && (*flp).start <= (*flp).lvl - level
            && (*flp).lvl > 0
        {
            // A marker fold may already start above `startlnum`; continue it
            // rather than opening a second one.
            let i = match folds.find(startlnum - 1) {
                Ok(i) | Err(i) => i,
            };
            if i < folds.len() && folds.at(i).top() < startlnum {
                fold = Some(folds.at(i));
            }
        }
    }
    let mut lvl = level;
    let mut startlnum2 = startlnum;
    // SAFETY: the caller's promise.
    let (firstlnum, linecount) = unsafe {
        (
            (*flp).lnum,
            (*(*(*flp).wp).w_buffer).b_ml.ml_line_count - (*flp).off,
        )
    };
    let mut finish = false;
    // SAFETY: the caller's promise.
    unsafe { (*flp).lnum_save = (*flp).lnum };
    while !got_int.get() {
        line_breakcheck();
        // SAFETY: the caller's promise; `flp` is the caller's.
        unsafe {
            lvl = (*flp).lvl.min(MAX_LEVEL);
            if (*flp).lnum > firstlnum && (level > lvl - (*flp).start || level >= (*flp).had_end) {
                // The fold ended above this line.
                lvl = 0;
            }
            if (*flp).lnum > bot
                && !finish
                && let Some(current) = fold
            {
                // Past the changed text. For the three open-ended methods a
                // fold may still have grown or shrunk, so keep going until
                // the tree agrees with the levels again.
                if !getlevel_is_open_ended(getlevel) {
                    break;
                }
                let mut depth = 0;
                if lvl >= level {
                    // How deep the existing tree says this line is.
                    let mut inner = current.nested();
                    let mut ll = (*flp).lnum - current.top();
                    while let Ok(k) = inner.find(ll) {
                        let nested = inner.at(k);
                        depth += 1;
                        ll -= nested.top();
                        inner = nested.nested();
                    }
                }
                if lvl < level + depth {
                    // The tree is deeper than the levels are: the nested fold
                    // that has to go also has to be redrawn.
                    let inner = current.nested();
                    let k = match inner.find((*flp).lnum - current.top()) {
                        Ok(k) | Err(k) => k,
                    };
                    if inner.has_data() {
                        bot = inner.at(k).last() + current.top();
                    }
                } else {
                    if !(current.top() + current.len() <= (*flp).lnum && lvl >= level) {
                        break;
                    }
                    finish = true;
                }
            }
            if fold.is_none()
                && (lvl != level
                    || (*flp).lnum_save >= bot
                    || (*flp).start != 0
                    || (*flp).had_end <= MAX_LEVEL
                    || (*flp).lnum == linecount)
            {
                // No fold started yet, and this line says one should have.
                // Reuse, truncate, split or delete whatever is in the way,
                // then leave `fold` naming the one to grow.
                while !got_int.get() {
                    // Whether an existing fold that ends just above
                    // `firstlnum` may be joined onto this one.
                    let concat = if (*flp).start != 0 || (*flp).had_end <= MAX_LEVEL {
                        0
                    } else {
                        1
                    };
                    // `fold.c` threads one out-parameter through four tests
                    // here; the first that hits names the entry to work on,
                    // and a miss leaves the index the fold would be inserted
                    // at.
                    let mut i = 0;
                    let mut matched = false;
                    if !folds.is_empty() {
                        i = match folds.find(startlnum) {
                            Ok(k) => {
                                matched = true;
                                k
                            }
                            Err(k) => k,
                        };
                        if !matched && i < folds.len() && folds.at(i).top() <= firstlnum {
                            matched = true;
                        }
                        if !matched {
                            i = match folds.find(firstlnum - concat) {
                                Ok(k) => {
                                    matched = true;
                                    k
                                }
                                Err(k) => k,
                            };
                        }
                        if !matched
                            && i < folds.len()
                            && (lvl < level && folds.at(i).top() < (*flp).lnum
                                || lvl >= level && folds.at(i).top() <= (*flp).lnum_save)
                        {
                            matched = true;
                        }
                    }
                    if !matched {
                        // Nothing usable: make a new fold covering the rest
                        // of the changed range.
                        let at = if folds.is_empty() { 0 } else { i };
                        fold_insert(folds, at);
                        let new = folds.at(at);
                        new.set_top(firstlnum);
                        new.set_len(bot - firstlnum + 1);
                        if topflags == FD_OPEN {
                            (*(*flp).wp).w_fold_manual = true;
                            new.set_flags(FD_OPEN);
                        } else if at <= 0 {
                            new.set_flags(topflags);
                            if topflags != FD_LEVEL {
                                (*(*flp).wp).w_fold_manual = true;
                            }
                        } else {
                            new.set_flags(folds.at(at - 1).flags());
                        }
                        new.set_small(None);
                        if getlevel_is_open_ended(getlevel) {
                            finish = true;
                        }
                        fold_changed.set(true);
                        fold = Some(new);
                        break;
                    }
                    let mut current = folds.at(i);
                    if current.top() + current.len() + concat <= firstlnum {
                        // It ends above the new fold.
                        if current.top() >= startlnum {
                            // Entirely inside the removed range.
                            delete_fold_entry(folds, i, true);
                        } else {
                            current.set_len(startlnum - current.top());
                            fold_mark_adjust_recurse(
                                current.nested().gap(),
                                current.len(),
                                MAXLNUM as linenr_T,
                                LINES_DELETED,
                                0,
                            );
                            fold_changed.set(true);
                        }
                        continue;
                    }
                    if current.top() != firstlnum {
                        if current.top() >= startlnum {
                            // It starts below the new fold: pull its top up,
                            // dragging the nested folds with it.
                            if current.top() > firstlnum {
                                fold_mark_adjust_recurse(
                                    current.nested().gap(),
                                    0,
                                    MAXLNUM as linenr_T,
                                    current.top() - firstlnum,
                                    0,
                                );
                            } else {
                                fold_mark_adjust_recurse(
                                    current.nested().gap(),
                                    0,
                                    firstlnum - current.top() - 1,
                                    LINES_DELETED,
                                    current.top() - firstlnum,
                                );
                            }
                            current.set_len(current.len() + current.top() - firstlnum);
                            current.set_top(firstlnum);
                            current.set_small(None);
                            fold_changed.set(true);
                        } else if (*flp).start != 0 && lvl == level || firstlnum != startlnum {
                            // It starts above: break it in two so the new
                            // fold can start where the levels say it does.
                            let (breakstart, breakend) = if firstlnum != startlnum {
                                (startlnum, firstlnum)
                            } else {
                                ((*flp).lnum, (*flp).lnum)
                            };
                            fold_remove(
                                (*flp).wp,
                                current.nested(),
                                breakstart - current.top(),
                                breakend - current.top(),
                            );
                            fold_split((*(*flp).wp).w_buffer, folds, i, breakstart, breakend - 1);
                            current = folds.at(i + 1);
                            if getlevel_is_open_ended(getlevel) {
                                finish = true;
                            }
                        }
                    }
                    if current.top() == startlnum && concat != 0 {
                        // It now touches the fold above it, so they are one.
                        let k = folds.index_of(current);
                        if k != 0 {
                            let above = folds.at(k - 1);
                            if above.top() + above.len() == current.top() {
                                fold_merge(above, folds, current);
                                current = above;
                            }
                        }
                    }
                    fold = Some(current);
                    break;
                }
            }
            if lvl < level || (*flp).lnum > linecount {
                break;
            }
            if lvl > level
                && let Some(current) = fold
            {
                // Deeper than this level: the nested list takes over.
                bot = bot.max((*flp).lnum);
                (*flp).lnum = (*flp).lnum_save - current.top();
                (*flp).off += current.top();
                let i = folds.index_of(current);
                bot = fold_update_computed_recurse(
                    current.nested(),
                    level + 1,
                    startlnum2 - current.top(),
                    flp,
                    getlevel,
                    bot - current.top(),
                    current.flags(),
                );
                // The recursion may have grown the array under us.
                let current = folds.at(i);
                fold = Some(current);
                (*flp).lnum += current.top();
                (*flp).lnum_save += current.top();
                (*flp).off -= current.top();
                bot += current.top();
                startlnum2 = (*flp).lnum;
            } else {
                // Step to the next line with a level of its own, remembering
                // where that search started so the caller sees both.
                (*flp).lnum = (*flp).lnum_save;
                let next = (*flp).lnum + 1;
                while !got_int.get() {
                    // 'foldexpr' may ask for the level of this line while we
                    // are working out the next one; park the answer.
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
                (*flp).lnum = next;
            }
        }
    }
    let Some(fold) = fold else {
        return bot;
    };
    // SAFETY: the caller's promise.
    unsafe {
        // Give the fold the length the walk ended up at.
        if fold.len() < (*flp).lnum - fold.top() {
            fold.set_len((*flp).lnum - fold.top());
            fold.set_small(None);
            fold_changed.set(true);
        } else if fold.top() + fold.len() > linecount {
            fold.set_len(linecount - fold.top() + 1);
        }
        fold_remove(
            (*flp).wp,
            fold.nested(),
            startlnum2 - fold.top(),
            (*flp).lnum - 1 - fold.top(),
        );
        let mut fold = fold;
        if lvl < level && fold.len() != (*flp).lnum - fold.top() {
            // It used to reach further down than the levels now say.
            if fold.top() + fold.len() - 1 > bot {
                if getlevel_is_open_ended(getlevel) {
                    // The rest of it still has to be looked at.
                    bot = fold.top() + fold.len() - 1;
                    fold.set_len((*flp).lnum - fold.top());
                } else {
                    let i = folds.index_of(fold);
                    fold_split((*(*flp).wp).w_buffer, folds, i, (*flp).lnum, bot);
                    fold = folds.at(i);
                }
            } else {
                fold.set_len((*flp).lnum - fold.top());
            }
            fold_changed.set(true);
        }
        // Absorb or drop the folds the new one now overlaps.
        loop {
            let next = fold.offset(1);
            if !folds.holds(next) || next.top() > (*flp).lnum {
                break;
            }
            if next.top() + next.len() > (*flp).lnum {
                if next.top() < (*flp).lnum {
                    fold_mark_adjust_recurse(
                        next.nested().gap(),
                        0,
                        (*flp).lnum - next.top() - 1,
                        LINES_DELETED,
                        next.top() - (*flp).lnum,
                    );
                    next.set_len(next.len() - ((*flp).lnum - next.top()));
                    next.set_top((*flp).lnum);
                    fold_changed.set(true);
                }
                if lvl >= level {
                    fold_merge(fold, folds, next);
                }
                break;
            }
            fold_changed.set(true);
            delete_fold_entry(folds, folds.index_of(next), true);
        }
        bot.max((*flp).lnum - 1)
    }
}

/// Low level function to get the foldlevel for the "indent" method.
/// Doesn't use any caching.
///
/// Returns a level of -1 if the foldlevel depends on surrounding lines.
///
/// # Safety
/// `flp` must be writable and name a live window and a line inside its
/// buffer.
pub(super) unsafe fn foldlevel_indent(flp: *mut fline_T) {
    // SAFETY: the caller's promise.
    unsafe {
        let lnum = (*flp).lnum + (*flp).off;
        let buf = (*(*flp).wp).w_buffer;
        let s = skipwhite(ml_get_buf(buf, lnum));
        // A blank line, or one starting with a 'foldignore' character, takes
        // its level from its neighbours.
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
        let foldnestmax = (*(*flp).wp).w_onebuf_opt.wo_fdn.max(0) as c_int;
        (*flp).lvl = (*flp).lvl.min(foldnestmax);
    }
}

/// Low level function to get the foldlevel for the "diff" method.
/// Doesn't use any caching.
///
/// # Safety
/// `flp` must be writable and name a live window and a line inside its
/// buffer.
pub(super) unsafe fn foldlevel_diff(flp: *mut fline_T) {
    // SAFETY: the caller's promise.
    unsafe { (*flp).lvl = c_int::from(diff_infold((*flp).wp, (*flp).lnum + (*flp).off)) };
}

/// Low level function to get the foldlevel for the "expr" method.
/// Doesn't use any caching.
///
/// Returns a level of -1 if the foldlevel depends on surrounding lines.
///
/// # Safety
/// `flp` must be writable and name a live window and a line inside its
/// buffer.
pub(super) unsafe fn foldlevel_expr(flp: *mut fline_T) {
    // SAFETY: the caller's promise; the current window is restored below.
    unsafe {
        let lnum = (*flp).lnum + (*flp).off;
        let win = curwin.get();
        curwin.set((*flp).wp);
        curbuf.set((*(*flp).wp).w_buffer);
        set_vim_var_nr(Vv::Lnum, lnum as varnumber_T);
        (*flp).start = 0;
        (*flp).had_end = (*flp).end;
        (*flp).end = MAX_LEVEL + 1;
        if lnum <= 1 {
            (*flp).lvl = 0;
        }
        // 'foldexpr' must not count as typed input.
        let save_keytyped = KeyTyped.get();
        let mut verdict: c_int = 0;
        let n = eval_foldexpr((*flp).wp, &raw mut verdict);
        KeyTyped.set(save_keytyped);
        match verdict as u8 {
            b'a' => {
                // "a1": one level deeper from here on.
                if (*flp).lvl >= 0 {
                    (*flp).lvl += n;
                    (*flp).lvl_next = (*flp).lvl;
                }
                (*flp).start = n;
            }
            b's' => {
                // "s1": one level shallower from the next line.
                if (*flp).lvl >= 0 {
                    (*flp).lvl_next = if n > (*flp).lvl { 0 } else { (*flp).lvl - n };
                    (*flp).end = (*flp).lvl_next + 1;
                }
            }
            b'>' => {
                // ">1": a fold of level 1 starts here.
                (*flp).lvl = n;
                (*flp).lvl_next = n;
                (*flp).start = 1;
            }
            b'<' => {
                // "<1": a fold of level 1 ends here.
                (*flp).lvl_next = (*flp).lvl.min(n - 1);
                (*flp).end = n;
            }
            b'=' => {
                // "=": the same level as the line above.
                (*flp).lvl_next = (*flp).lvl;
            }
            _ => {
                (*flp).lvl_next = if n < 0 { (*flp).lvl } else { n };
                (*flp).lvl = n;
            }
        }
        if (*flp).lvl < 0 {
            // An undefined level at either end of the buffer has nothing to
            // inherit from.
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
}

/// Low level function to get the foldlevel for the "syntax" method.
/// Doesn't use any caching.
///
/// # Safety
/// `flp` must be writable and name a live window and a line inside its
/// buffer.
pub(super) unsafe fn foldlevel_syntax(flp: *mut fline_T) {
    // SAFETY: the caller's promise.
    unsafe {
        let lnum = (*flp).lnum + (*flp).off;
        (*flp).lvl = syn_get_foldlevel((*flp).wp, lnum);
        (*flp).start = 0;
        if lnum < (*(*(*flp).wp).w_buffer).b_ml.ml_line_count {
            // A fold that starts on the next line starts here as far as the
            // tree is concerned, so the syntax item's first line is inside.
            let n = syn_get_foldlevel((*flp).wp, lnum + 1);
            if n > (*flp).lvl {
                (*flp).start = n - (*flp).lvl;
                (*flp).lvl = n;
            }
        }
    }
}
