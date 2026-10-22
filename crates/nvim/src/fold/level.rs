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
//!
//! # Holding a place in the tree
//!
//! Asking for a level is user code: 'foldexpr' runs Vimscript, and the
//! syntax and diff getters can run 'diffexpr' or reach the tree through
//! `foldlevel()`. So the update never holds a reference into the tree across
//! one. Each level of the recursion names its list by a *path* — the indices
//! of the folds above it — and the fold it is growing by an index, and takes
//! the list again from the window after every call out. A 'foldexpr' that
//! switched to "manual" and ran `zE` leaves a path that no longer resolves;
//! the update then stops ([`TreeLost`]) rather than write into folds that are
//! gone, which is what upstream's `fold_T *` did.

#![forbid(unsafe_code)]

use crate::charset::skip;
use crate::cstr::byte_at;
use crate::diff::diff_infold;
use crate::diff::state::diff_context;
use crate::drawscreen::redraw_win_range_later;
use crate::eval::eval_foldexpr;
use crate::eval::vars::set_vim_var_nr;
use crate::getchar::state::{KeyTyped, got_int};
use crate::indent::{get_indent_buf, get_sw_value};
use crate::r#move::changed_window_setting;
use crate::os::input::line_breakcheck;
use crate::strings::has_char;
use crate::syntax::syn_get_foldlevel;
use crate::winlayer::graph::switch_to;
use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

use super::adjust::*;
use super::list::list_at;
use super::marker::*;
use super::*;
use crate::optionstr::OptString;
use crate::pos::MAXLNUM;

/// Which computed 'foldmethod' an update asks for levels.
///
/// The marker method carries 'foldmarker''s two halves, cut once per update:
/// it runs no user code, so the option cannot change under it.
#[derive(Copy, Clone, PartialEq, Eq)]
enum Method<'m> {
    Indent,
    Expr,
    Marker(&'m [u8], &'m [u8]),
    Syntax,
    Diff,
}

impl Method<'_> {
    /// Ask for the level at `line`.
    fn level(self, line: &mut FoldLine) {
        match self {
            Self::Indent => foldlevel_indent(line),
            Self::Expr => foldlevel_expr(line),
            Self::Marker(start, end) => foldlevel_marker(line, start, end),
            Self::Syntax => foldlevel_syntax(line),
            Self::Diff => foldlevel_diff(line),
        }
    }

    /// Whether this is one of the three methods whose folds can end
    /// somewhere the line levels alone do not say — so an update has to keep
    /// looking past the changed range.
    fn is_open_ended(self) -> bool {
        matches!(self, Self::Marker(..) | Self::Expr | Self::Syntax)
    }
}

/// The tree changed shape while a level getter ran, and the update's
/// positions no longer name anything.
struct TreeLost;

/// Lend the list `path` names in `win`'s tree to `f`; [`TreeLost`] when the
/// path no longer resolves.
fn with_list<R>(
    win: Win,
    path: &[usize],
    f: impl FnOnce(&mut Vec<Fold>) -> R,
) -> Result<R, TreeLost> {
    win.with_folds(|root| list_at(root, path).map(f))
        .ok_or(TreeLost)
}

/// Update the folding for window "wp", at least from lines "top" to "bot".
pub(super) fn fold_update_computed(mut win: Win, mut top: LineNr, mut bot: LineNr) {
    if invalid_top.get() != 0 {
        // Already updating this window; the recursion would fight itself.
        return;
    }
    if win.w_foldinvalid {
        top = 1;
        bot = win.buffer().b_ml.ml_line_count;
        win.w_foldinvalid = false;
        win.with_folds(|folds| forget_small_flags(folds));
    }
    if foldmethod_is_diff(win) {
        // 'diffopt' context lines belong to the same fold as the change.
        top = (top - diff_context.get()).max(1);
        bot += diff_context.get();
    }
    top = top.min(win.buffer().b_ml.ml_line_count);
    let mut line = FoldLine {
        win,
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

    // Pick the level getter and prime `line` with a level for `top`.
    let method;
    let foldmarker;
    if foldmethod_is_marker(win) {
        parse_marker(win);
        foldmarker = win.w_onebuf_opt.wo_fmr.get();
        let (start, end) = markers(&foldmarker);
        method = Method::Marker(start, end);
        if top > 1 {
            // The marker method needs the previous line's level, and the
            // markers on that line, before it can read `top`.
            let level = fold_level_win(win, top - 1);
            line.lnum = top - 1;
            line.lvl = level;
            method.level(&mut line);
            line.lvl = if line.lvl > level {
                level - (line.lvl - line.lvl_next)
            } else {
                line.lvl_next
            };
        }
        line.lnum = top;
        method.level(&mut line);
    } else {
        line.lnum = top;
        if foldmethod_is_expr(win) {
            method = Method::Expr;
            if top > 1 {
                line.lnum -= 1;
            }
        } else if foldmethod_is_syntax(win) {
            method = Method::Syntax;
        } else if foldmethod_is_diff(win) {
            method = Method::Diff;
        } else {
            method = Method::Indent;
            if top > 1 {
                line.lnum -= 1;
            }
        }
        // Indent and expr can answer "undefined" (-1); walk back until a
        // line has a level of its own.
        line.lvl = -1;
        while !got_int.get() {
            line.lvl_next = -1;
            method.level(&mut line);
            if line.lvl >= 0 {
                break;
            }
            line.lnum -= 1;
        }
    }

    if method == Method::Syntax {
        // A syntax fold can reach past `bot`, and the whole of it has to be
        // updated or its end is left dangling.
        let mut folds: &[Fold] = &win.w_folds;
        let mut innermost = None;
        let mut current_fdl = 0;
        let mut fold_start_lnum: LineNr = 0;
        let mut lnum_rel = line.lnum;
        while current_fdl < line.lvl {
            let Ok(i) = find(folds, lnum_rel) else { break };
            let fold = &folds[i];
            current_fdl += 1;
            fold_start_lnum += fold.top;
            lnum_rel -= fold.top;
            innermost = Some(fold.len);
            folds = &fold.nested;
        }
        if let Some(len) = innermost
            && current_fdl == line.lvl
        {
            bot = bot.max(fold_start_lnum + len);
        }
    }

    let mut start = line.lnum;
    let mut end = bot;
    // Re-read rather than kept: 'foldexpr' is user code, and it can change
    // the buffer under this loop.
    let line_count = || win.buffer().b_ml.ml_line_count;
    if start > end && end < line_count() {
        end = start;
    }
    let mut path = Vec::new();
    let mut lost = false;
    while !got_int.get() {
        if line.lnum > line_count() {
            break;
        }
        if line.lnum > end {
            if !method.is_open_ended() {
                break;
            }
            // A fold that straddles the end of the range drags the range out
            // to cover the whole of it.
            let (lvl, lnum) = (line.lvl, line.lnum);
            let straddling = win.with_folds(|folds| {
                if start <= end
                    && let Ok(i) = find(folds, end)
                    && folds[i].last() > end
                {
                    Some(folds[i].last())
                } else if lvl == 0
                    && let Ok(i) = find(folds, lnum)
                    && folds[i].top < lnum
                {
                    Some(folds[i].last())
                } else {
                    None
                }
            });
            if let Some(last) = straddling {
                end = last;
            } else {
                if !(method == Method::Syntax && fold_level_win(win, line.lnum) != line.lvl) {
                    break;
                }
                end = line.lnum;
            }
        }
        if line.lvl > 0 {
            invalid_top.set(line.lnum);
            invalid_bot.set(end);
            let update = fold_update_computed_recurse(
                win,
                &mut path,
                1,
                start,
                &mut line,
                method,
                end,
                FoldState::Level,
            );
            let Ok(new_end) = update else {
                lost = true;
                break;
            };
            end = new_end;
            start = line.lnum;
        } else {
            if line.lnum == line_count() {
                break;
            }
            line.lnum += 1;
            line.lvl = line.lvl_next;
            method.level(&mut line);
        }
    }
    if !lost {
        win.with_folds(|folds| fold_remove(folds, start, end));
    }
    if fold_changed.get() && win.w_onebuf_opt.wo_fen != 0 {
        changed_window_setting(win);
    }
    if end != bot && !lost {
        redraw_win_range_later(win, top, end);
    }
    invalid_top.set(0);
}

/// What one pass of [`fold_update_computed_recurse`]'s loop does once the
/// tree has had its say, if it goes on.
enum Next {
    /// The levels go deeper here: recurse into the nested list of fold
    /// `index`, whose `top` and `state` are given.
    Recurse {
        index: usize,
        top: LineNr,
        state: FoldState,
    },
    /// Step on to the next line with a level.
    Step,
}

/// Update a fold that starts at "line->lnum".  At this line there is always a
/// valid foldlevel, and its level >= "level".
///
/// "line" is valid for "line->lnum" when called and it's valid when returning.
/// "line->lnum" is set to the lnum just below the fold, if it ends before
/// "bot", it's "bot" plus one if the fold continues and it's bigger when using
/// the marker method and a text change made following folds to change.
/// When returning, "line->lnum_save" is the line number that was used to get
/// the level when the level at "line->lnum" is invalid.
/// Remove any folds from "startlnum" up to here at this level.
/// Recursively update nested folds.
/// Below line "bot" there are no changes in the text.
/// "line->lnum", "line->lnum_save" and "bot" are relative to the start of the
/// outer fold.
/// "line->off" is the offset to the real line number in the buffer.
///
/// `path` names the list being updated (see the module docs); it is pushed
/// and popped around each recursion and comes back as it was handed in.
///
/// `topflags` — containing fold flags
///
/// Returns bot, which may have been increased for lines that also need to be
/// updated as a result of a detected change in the fold.
#[allow(clippy::too_many_arguments)]
fn fold_update_computed_recurse(
    win: Win,
    path: &mut Vec<usize>,
    level: c_int,
    startlnum: LineNr,
    line: &mut FoldLine,
    method: Method<'_>,
    mut bot: LineNr,
    topflags: FoldState,
) -> Result<LineNr, TreeLost> {
    let open_ended = method.is_open_ended();
    // The fold this call is building, once there is one, as its index in the
    // list. `None` means "not started yet", which is what the whole first
    // half of the loop is about.
    let mut fold: Option<usize> = None;
    if matches!(method, Method::Marker(..)) && line.start <= line.lvl - level && line.lvl > 0 {
        // A marker fold may already start above `startlnum`; continue it
        // rather than opening a second one.
        fold = with_list(win, path, |folds| {
            let i = find_at_or_below(folds, startlnum - 1);
            folds.get(i).filter(|f| f.top < startlnum).map(|_| i)
        })?;
    }
    let mut lvl = level;
    let mut startlnum2 = startlnum;
    let firstlnum = line.lnum;
    let linecount = line.win.buffer().b_ml.ml_line_count - line.off;
    let mut finish = false;
    line.lnum_save = line.lnum;
    while !got_int.get() {
        line_breakcheck();
        lvl = line.lvl.min(MAX_LEVEL);
        if line.lnum > firstlnum && (level > lvl - line.start || level >= line.had_end) {
            // The fold ended above this line.
            lvl = 0;
        }
        let mut manual = false;
        if line.lnum > bot
            && !finish
            && let Some(ci) = fold
        {
            // Past the changed text. For the three open-ended methods a fold
            // may still have grown or shrunk, so keep going until the tree
            // agrees with the levels again.
            if !open_ended {
                break;
            }
            let stop = with_list(win, path, |folds| {
                let current = folds.get(ci).ok_or(TreeLost)?;
                let mut depth = 0;
                if lvl >= level {
                    // How deep the existing tree says this line is.
                    let mut inner: &[Fold] = &current.nested;
                    let mut ll = line.lnum - current.top;
                    while let Ok(k) = find(inner, ll) {
                        depth += 1;
                        ll -= inner[k].top;
                        inner = &inner[k].nested;
                    }
                }
                if lvl < level + depth {
                    // The tree is deeper than the levels are: the nested fold
                    // that has to go also has to be redrawn.
                    let inner = &current.nested;
                    let k = find_at_or_below(inner, line.lnum - current.top);
                    // UPSTREAM BUG: `k` may be `inner.len()`, and `fold.c`
                    // reads that entry anyway (guarding only against a list
                    // that never had storage). Upstream reads whatever bytes
                    // sit there -- usually the zeroed slack `ga_grow` left,
                    // sometimes a stale copy of a deleted fold, sometimes past
                    // the block. The zeroed case is reproduced: a fold at 0 of
                    // length 0 puts `bot` one above this fold. See
                    // ~/agents/context/1786212071-upstream-neovim-bugs/
                    // fold-iems-recurse-fp2-past-end.md.
                    match inner.get(k) {
                        Some(nested) => bot = nested.last() + current.top,
                        None if inner.capacity() > 0 => bot = current.top - 1,
                        None => {}
                    }
                } else {
                    if !(current.top + current.len <= line.lnum && lvl >= level) {
                        return Ok(true);
                    }
                    finish = true;
                }
                Ok(false)
            })??;
            if stop {
                break;
            }
        }
        if fold.is_none()
            && (lvl != level
                || line.lnum_save >= bot
                || line.start != 0
                || line.had_end <= MAX_LEVEL
                || line.lnum == linecount)
        {
            // No fold started yet, and this line says one should have. Reuse,
            // truncate, split or delete whatever is in the way, then leave
            // `fold` naming the one to grow.
            fold = with_list(win, path, |folds| {
                Reuse {
                    folds,
                    line,
                    level,
                    lvl,
                    startlnum,
                    firstlnum,
                    bot,
                    topflags,
                    open_ended,
                    finish: &mut finish,
                    manual: &mut manual,
                }
                .find_or_make()
            })?;
        }
        if manual {
            line.win.w_fold_manual = true;
        }
        if lvl < level || line.lnum > linecount {
            break;
        }
        let next = match fold {
            Some(index) if lvl > level => {
                let (top, state) = with_list(win, path, |folds| {
                    folds.get(index).map(|f| (f.top, f.state))
                })?
                .ok_or(TreeLost)?;
                Next::Recurse { index, top, state }
            }
            _ => Next::Step,
        };
        match next {
            Next::Recurse { index, top, state } => {
                // Deeper than this level: the nested list takes over.
                bot = bot.max(line.lnum);
                line.lnum = line.lnum_save - top;
                line.off += top;
                let (from, to) = (startlnum2 - top, bot - top);
                path.push(index);
                let inner = fold_update_computed_recurse(
                    win,
                    path,
                    level + 1,
                    from,
                    line,
                    method,
                    to,
                    state,
                );
                path.pop();
                bot = inner?;
                // The recursion changed only the fold's nested list, so its
                // `top` is the one read above; whether the getters it ran
                // left the fold there at all, the next borrow checks.
                fold = Some(index);
                line.lnum += top;
                line.lnum_save += top;
                line.off -= top;
                bot += top;
                startlnum2 = line.lnum;
            }
            Next::Step => {
                // Step to the next line with a level of its own, remembering
                // where that search started so the caller sees both.
                line.lnum = line.lnum_save;
                let next = line.lnum + 1;
                while !got_int.get() {
                    // 'foldexpr' may ask for the level of this line while we
                    // are working out the next one; park the answer.
                    prev_lnum.set(line.lnum);
                    prev_lnum_lvl.set(line.lvl);
                    line.lnum += 1;
                    if line.lnum > linecount {
                        break;
                    }
                    line.lvl = line.lvl_next;
                    method.level(line);
                    if line.lvl >= 0 || line.had_end <= MAX_LEVEL {
                        break;
                    }
                }
                prev_lnum.set(0);
                if line.lnum > linecount {
                    break;
                }
                line.lnum_save = line.lnum;
                line.lnum = next;
            }
        }
    }
    let Some(index) = fold else {
        return Ok(bot);
    };
    with_list(win, path, |folds| {
        if index >= folds.len() {
            return Err(TreeLost);
        }
        Ok(finish_fold(
            folds, index, line.lnum, lvl, level, bot, startlnum2, linecount, open_ended,
        ))
    })?
}

/// The tail of [`fold_update_computed_recurse`]: give fold `index` of `folds`
/// the length the walk ended up at (`lnum` is the line just below it), drop
/// what it no longer holds, and absorb or drop the folds it now overlaps.
/// Answers the new `bot`.
#[allow(clippy::too_many_arguments)]
fn finish_fold(
    folds: &mut Vec<Fold>,
    index: usize,
    lnum: LineNr,
    lvl: c_int,
    level: c_int,
    mut bot: LineNr,
    startlnum2: LineNr,
    linecount: LineNr,
    open_ended: bool,
) -> LineNr {
    let fold = &mut folds[index];
    if fold.len < lnum - fold.top {
        fold.len = lnum - fold.top;
        fold.small = None;
        fold_changed.set(true);
    } else if fold.top + fold.len > linecount {
        fold.len = linecount - fold.top + 1;
    }
    let (from, to) = (startlnum2 - fold.top, lnum - 1 - fold.top);
    fold_remove(&mut fold.nested, from, to);
    if lvl < level && fold.len != lnum - fold.top {
        // It used to reach further down than the levels now say.
        if fold.top + fold.len - 1 > bot {
            if open_ended {
                // The rest of it still has to be looked at.
                bot = fold.top + fold.len - 1;
                fold.len = lnum - fold.top;
            } else {
                fold_split(folds, index, lnum, bot);
            }
        } else {
            fold.len = lnum - fold.top;
        }
        fold_changed.set(true);
    }
    // Absorb or drop the folds the new one now overlaps.
    while let Some(next) = folds.get_mut(index + 1) {
        if next.top > lnum {
            break;
        }
        if next.top + next.len > lnum {
            if next.top < lnum {
                let nt = next.top;
                adjust_fold_list(&mut next.nested, 0, lnum - nt - 1, LINES_DELETED, nt - lnum);
                next.len -= lnum - nt;
                next.top = lnum;
                fold_changed.set(true);
            }
            if lvl >= level {
                fold_merge(folds, index);
            }
            break;
        }
        fold_changed.set(true);
        delete_fold_entry(folds, index + 1, true);
    }
    bot.max(lnum - 1)
}

/// The state [`Reuse::find_or_make`] works with: one level of
/// [`fold_update_computed_recurse`], at the line where a fold should start.
struct Reuse<'a> {
    folds: &'a mut Vec<Fold>,
    line: &'a mut FoldLine,
    level: c_int,
    lvl: c_int,
    startlnum: LineNr,
    firstlnum: LineNr,
    bot: LineNr,
    topflags: FoldState,
    open_ended: bool,
    finish: &'a mut bool,
    /// Set when the new fold makes the window's folds manual.
    manual: &'a mut bool,
}

impl Reuse<'_> {
    /// Find the existing fold the new one should be (reusing, truncating,
    /// splitting or deleting whatever is in the way), or insert one, and
    /// answer its index. `None` only if CTRL-C cut the search short.
    fn find_or_make(&mut self) -> Option<usize> {
        let (line, firstlnum, startlnum) = (&*self.line, self.firstlnum, self.startlnum);
        let (level, lvl) = (self.level, self.lvl);
        while !got_int.get() {
            let folds = &mut *self.folds;
            // Whether an existing fold that ends just above `firstlnum` may
            // be joined onto this one.
            let concat = if line.start != 0 || line.had_end <= MAX_LEVEL {
                0
            } else {
                1
            };
            // `fold.c` threads one out-parameter through four tests here; the
            // first that hits names the entry to work on, and a miss leaves
            // the index the fold would be inserted at.
            let mut i = 0;
            let mut matched = false;
            if !folds.is_empty() {
                i = match find(folds, startlnum) {
                    Ok(k) => {
                        matched = true;
                        k
                    }
                    Err(k) => k,
                };
                if !matched && folds.get(i).is_some_and(|f| f.top <= firstlnum) {
                    matched = true;
                }
                if !matched {
                    i = match find(folds, firstlnum - concat) {
                        Ok(k) => {
                            matched = true;
                            k
                        }
                        Err(k) => k,
                    };
                }
                if !matched
                    && folds.get(i).is_some_and(|f| {
                        lvl < level && f.top < line.lnum || lvl >= level && f.top <= line.lnum_save
                    })
                {
                    matched = true;
                }
            }
            if !matched {
                // Nothing usable: make a new fold covering the rest of the
                // changed range.
                let at = if folds.is_empty() { 0 } else { i };
                let state = if self.topflags == FoldState::Open {
                    *self.manual = true;
                    FoldState::Open
                } else if at == 0 {
                    if self.topflags != FoldState::Level {
                        *self.manual = true;
                    }
                    self.topflags
                } else {
                    folds[at - 1].state
                };
                let mut new = Fold::new(firstlnum, self.bot - firstlnum + 1);
                new.state = state;
                folds.insert(at, new);
                if self.open_ended {
                    *self.finish = true;
                }
                fold_changed.set(true);
                return Some(at);
            }
            let mut ci = i;
            let current = &mut folds[ci];
            if current.top + current.len + concat <= firstlnum {
                // It ends above the new fold.
                if current.top >= startlnum {
                    // Entirely inside the removed range.
                    delete_fold_entry(folds, ci, true);
                } else {
                    current.len = startlnum - current.top;
                    let len = current.len;
                    adjust_fold_list(&mut current.nested, len, MAXLNUM, LINES_DELETED, 0);
                    fold_changed.set(true);
                }
                continue;
            }
            if current.top != firstlnum {
                if current.top >= startlnum {
                    // It starts below the new fold: pull its top up,
                    // dragging the nested folds with it.
                    let ct = current.top;
                    if ct > firstlnum {
                        adjust_fold_list(&mut current.nested, 0, MAXLNUM, ct - firstlnum, 0);
                    } else {
                        adjust_fold_list(
                            &mut current.nested,
                            0,
                            firstlnum - ct - 1,
                            LINES_DELETED,
                            ct - firstlnum,
                        );
                    }
                    current.len += ct - firstlnum;
                    current.top = firstlnum;
                    current.small = None;
                    fold_changed.set(true);
                } else if line.start != 0 && lvl == level || firstlnum != startlnum {
                    // It starts above: break it in two so the new fold can
                    // start where the levels say it does.
                    let (breakstart, breakend) = if firstlnum != startlnum {
                        (startlnum, firstlnum)
                    } else {
                        (line.lnum, line.lnum)
                    };
                    let ct = current.top;
                    fold_remove(&mut current.nested, breakstart - ct, breakend - ct);
                    fold_split(folds, ci, breakstart, breakend - 1);
                    ci += 1;
                    if self.open_ended {
                        *self.finish = true;
                    }
                }
            }
            if folds[ci].top == startlnum && concat != 0 && ci != 0 {
                // It now touches the fold above it, so they are one.
                let above = &folds[ci - 1];
                if above.top + above.len == folds[ci].top {
                    fold_merge(folds, ci - 1);
                    ci -= 1;
                }
            }
            return Some(ci);
        }
        None
    }
}

/// Low level function to get the foldlevel for the "indent" method.
/// Doesn't use any caching.
///
/// Returns a level of -1 if the foldlevel depends on surrounding lines.
pub(super) fn foldlevel_indent(line: &mut FoldLine) {
    let lnum = line.lnum + line.off;
    let win = line.win;
    let buf = win.buffer();
    let mut lines = buf.lines();
    let text = lines.line(lnum);
    let first = byte_at(text, skip::white(text));
    // A blank line, or one starting with a 'foldignore' character, takes
    // its level from its neighbours.
    if c_int::from(first) == NUL || has_char(win.w_onebuf_opt.wo_fdi.cstr(), c_int::from(first)) {
        line.lvl = if lnum == 1 || lnum == buf.b_ml.ml_line_count {
            0
        } else {
            -1
        };
    } else {
        line.lvl = get_indent_buf(buf, lnum) / get_sw_value(buf);
    }
    let foldnestmax = win.w_onebuf_opt.wo_fdn.max(0) as c_int;
    line.lvl = line.lvl.min(foldnestmax);
}

/// Low level function to get the foldlevel for the "diff" method.
/// Doesn't use any caching.
pub(super) fn foldlevel_diff(line: &mut FoldLine) {
    let infold = diff_infold(line.win, line.lnum + line.off);
    line.lvl = c_int::from(infold);
}

/// Low level function to get the foldlevel for the "expr" method.
/// Doesn't use any caching.
///
/// Returns a level of -1 if the foldlevel depends on surrounding lines.
pub(super) fn foldlevel_expr(line: &mut FoldLine) {
    let lnum = line.lnum + line.off;
    // The current window is restored below.
    let saved = switch_to(line.win);
    set_vim_var_nr(Vv::Lnum, lnum as VarNumber);
    line.start = 0;
    line.had_end = line.end;
    line.end = MAX_LEVEL + 1;
    if lnum <= 1 {
        line.lvl = 0;
    }
    // 'foldexpr' must not count as typed input.
    let save_keytyped = KeyTyped.get();
    let (n, verdict) = eval_foldexpr(line.win);
    KeyTyped.set(save_keytyped);
    // `eval_foldexpr` writes one byte of the expression's answer, so the
    // truncation below is exact.
    match verdict as u8 {
        b'a' => {
            // "a1": one level deeper from here on.
            if line.lvl >= 0 {
                line.lvl += n;
                line.lvl_next = line.lvl;
            }
            line.start = n;
        }
        b's' => {
            // "s1": one level shallower from the next line.
            if line.lvl >= 0 {
                line.lvl_next = if n > line.lvl { 0 } else { line.lvl - n };
                line.end = line.lvl_next + 1;
            }
        }
        b'>' => {
            // ">1": a fold of level 1 starts here.
            line.lvl = n;
            line.lvl_next = n;
            line.start = 1;
        }
        b'<' => {
            // "<1": a fold of level 1 ends here.
            line.lvl_next = line.lvl.min(n - 1);
            line.end = n;
        }
        b'=' => {
            // "=": the same level as the line above.
            line.lvl_next = line.lvl;
        }
        _ => {
            line.lvl_next = if n < 0 { line.lvl } else { n };
            line.lvl = n;
        }
    }
    if line.lvl < 0 {
        // An undefined level at either end of the buffer has nothing to
        // inherit from.
        if lnum <= 1 {
            line.lvl = 0;
            line.lvl_next = 0;
        }
        if lnum == Buf::current().b_ml.ml_line_count {
            line.lvl_next = 0;
        }
    }
    saved.restore();
}

/// Low level function to get the foldlevel for the "syntax" method.
/// Doesn't use any caching.
pub(super) fn foldlevel_syntax(line: &mut FoldLine) {
    let lnum = line.lnum + line.off;
    line.lvl = syn_get_foldlevel(line.win, lnum);
    line.start = 0;
    if lnum < line.win.buffer().b_ml.ml_line_count {
        // A fold that starts on the next line starts here as far as the
        // tree is concerned, so the syntax item's first line is inside.
        let n = syn_get_foldlevel(line.win, lnum + 1);
        if n > line.lvl {
            line.start = n - line.lvl;
            line.lvl = n;
        }
    }
}
