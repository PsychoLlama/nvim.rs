//! Creating, deleting, opening and closing folds by hand — everything behind
//! the `z` commands and `:fold`/`:foldopen`/`:foldclose`.
//!
//! A fold's own `fd_flags` says whether it is drawn open, closed, or takes
//! its state from 'foldlevel' (`FD_LEVEL`); the moment the user opens or
//! closes one by hand the window becomes `w_fold_manual` and stops following
//! the option until 'foldlevel' is set again.

#![forbid(unsafe_code)]

use crate::buffer_updates::buf_updates_send_changes;
use crate::change::changed_lines;
use crate::cursor::check_cursor_col;
use crate::diff::diff_lnum_win;
use crate::drawscreen::{UPD_INVERTED, redraw_buf_later, redraw_curbuf_later};
use crate::message::emsg;
use crate::r#move::changed_window_setting;
use crate::option::vars::p_fcl;
use crate::os::cshim::gettext;
use crate::winlayer::{TabPage, Win, windows_in_tab};
use core::ffi::CStr;
use core::ffi::c_int;

use super::marker::*;
use super::*;

/// Close fold for current window at position "pos".
/// Repeat "count" times.
pub fn close_fold(pos: Pos, count: c_int) {
    set_fold_repeat(pos, count, 0);
}

/// Close fold for current window at position `pos` recursively.
pub fn close_fold_recurse(pos: Pos) {
    set_manual_fold(pos, false, true, None);
}

/// Open or Close folds for current window in lines "first" to "last".
/// Used for "zo", "zO", "zc" and "zC" in Visual mode.
///
/// `opening` — true to open, false to close
/// `recurse` — true to do it recursively
/// `had_visual` — true when Visual selection used
pub fn op_fold_range(
    firstpos: Pos,
    lastpos: Pos,
    opening: c_int,
    recurse: c_int,
    had_visual: bool,
) {
    let mut done: c_int = DONE_NOTHING;
    let last = lastpos.lnum;
    let mut lnum = firstpos.lnum;
    while lnum <= last {
        let at = Pos {
            lnum,
            col: 0,
            coladd: 0,
        };
        let mut lnum_next = lnum;
        if opening != 0 && recurse == 0 {
            has_folding(Win::current(), lnum, None, Some(&mut lnum_next));
        }
        set_manual_fold(at, opening != 0, recurse != 0, Some(&mut done));
        if opening == 0 && recurse == 0 {
            has_folding(Win::current(), lnum, None, Some(&mut lnum_next));
        }
        lnum = lnum_next + 1;
    }
    if done == DONE_NOTHING {
        emsg_nofold();
    }
    if had_visual {
        redraw_curbuf_later(UPD_INVERTED);
    }
}

/// Open fold for current window at position "pos".
/// Repeat "count" times.
pub fn open_fold(pos: Pos, count: c_int) {
    set_fold_repeat(pos, count, 1);
}

/// Open fold for current window at position `pos` recursively.
pub fn open_fold_recurse(pos: Pos) {
    set_manual_fold(pos, true, true, None);
}

/// Open folds until the cursor line is not in a closed fold.
pub fn fold_open_cursor() {
    checkupdate(Win::current());
    if has_any_folding(Win::current()) == 0 {
        return;
    }
    loop {
        let mut done: c_int = DONE_NOTHING;
        set_manual_fold(Win::current().w_cursor, true, false, Some(&mut done));
        // The loop's only exit. Each pass opens the outermost fold still
        // closed over the cursor, so once a pass opens nothing there is
        // nothing left to open.
        if done & DONE_ACTION == 0 {
            break;
        }
    }
}

/// Set new foldlevel for current window.
pub fn new_fold_level() {
    new_fold_level_win(Win::current());
    if !(foldmethod_is_diff(Win::current()) && Win::current().w_onebuf_opt.wo_scb != 0) {
        return;
    }
    // 'scrollbind' in a diff: the other diffed windows follow.
    for mut win in windows_in_tab(TabPage::current()) {
        if !win.is_current() && foldmethod_is_diff(win) && win.w_onebuf_opt.wo_scb != 0 {
            win.w_onebuf_opt.wo_fdl = Win::current().w_onebuf_opt.wo_fdl;
            new_fold_level_win(win);
        }
    }
}

/// Hand `win`'s toplevel folds back to 'foldlevel'.
pub(super) fn new_fold_level_win(mut win: Win) {
    checkupdate(win);
    if win.w_fold_manual {
        win.with_folds(|folds| {
            for fold in folds {
                fold.state = FoldState::Level;
            }
        });
        win.w_fold_manual = false;
    }
    changed_window_setting(win);
}

/// Apply 'foldlevel' to all folds that don't contain the cursor.
pub fn fold_check_close() {
    if p_fcl(CStr::is_empty) {
        return;
    }
    let win = Win::current();
    checkupdate(win);
    let (lnum, level) = (win.w_cursor.lnum, win.w_onebuf_opt.wo_fdl as c_int);
    let changed = win.with_folds(|folds| close_folds_off_cursor(folds, lnum, level));
    if changed {
        changed_window_setting(win);
    }
}

/// Hand every manually opened fold in `folds` that does *not* contain `lnum`
/// back to 'foldlevel' — what `'foldclose'` = "all" asks for.
///
/// Returns whether anything changed.
pub(super) fn close_folds_off_cursor(folds: &mut [Fold], lnum: LineNr, level: c_int) -> bool {
    let mut changed = false;
    for fold in folds {
        if !fold.is(FoldState::Open) {
            continue;
        }
        if level <= 0 && (lnum < fold.top || lnum >= fold.top + fold.len) {
            fold.state = FoldState::Level;
            changed = true;
        } else {
            changed |= close_folds_off_cursor(&mut fold.nested, lnum - fold.top, level - 1);
        }
    }
    changed
}

/// Returns true if it's allowed to manually create or delete a fold or,
///          give an error message and return false if not.
pub fn fold_manual_allowed(create: bool) -> c_int {
    if foldmethod_is_manual(Win::current()) || foldmethod_is_marker(Win::current()) {
        return 1;
    }
    let msg = if create {
        c"E350: Cannot create fold with current 'foldmethod'"
    } else {
        c"E351: Cannot delete fold with current 'foldmethod'"
    };
    emsg(gettext(msg));
    0
}

/// Where [`insert_new_fold`] put the fold, and what the folds above it said.
struct Inserted {
    /// How deep it is; it contains the line it was created from.
    depth: usize,
    /// Whether a fold above it takes its state from 'foldlevel'.
    use_level: bool,
    /// Whether a fold above it is closed.
    closed: bool,
}

/// Insert a fold over `start..=end` into `root`: into the innermost list it
/// fits inside, swallowing the folds it covers whole as its children.
///
/// The new entry's `state` is not yet its own: upstream moves the entries
/// after it up with `memmove` and leaves the slot holding a stale copy of
/// whatever sat there -- the first fold it swallowed, or the fold it was
/// inserted before, or (appended) the zeroed `FD_OPEN` slack -- and the
/// `close_fold` [`fold_create`] may run before setting it reads that state.
/// The copy is reproduced so that walk ends where upstream's does.
fn insert_new_fold(
    root: &mut Vec<Fold>,
    start: LineNr,
    end: LineNr,
    foldlevel: OptInt,
) -> Inserted {
    // Descend to the innermost list the new fold fits inside, tracking
    // whether it will inherit a closed state from the folds above it. `top`
    // and `bot` become relative to that list as we go.
    let mut use_level = false;
    let mut closed = false;
    let mut depth = 0;
    let mut top = start;
    let mut bot = end;
    let mut folds = root;
    let mut i = loop {
        match find(folds, top) {
            Ok(idx) => {
                let fold = &folds[idx];
                if fold.top + fold.len <= bot {
                    // The new fold reaches past this one, so it belongs
                    // beside it rather than inside it.
                    break idx;
                }
                top -= fold.top;
                bot -= fold.top;
                if use_level || fold.is(FoldState::Level) {
                    use_level = true;
                    if OptInt::try_from(depth).unwrap_or(OptInt::MAX) >= foldlevel {
                        closed = true;
                    }
                } else if fold.is(FoldState::Closed) {
                    closed = true;
                }
                depth += 1;
                folds = &mut folds[idx].nested;
            }
            Err(idx) => break idx,
        }
    };
    if folds.is_empty() {
        i = 0;
    }

    // The folds the new one swallows whole become its children.
    let cont = folds[i..].iter().take_while(|fold| fold.top <= bot).count();
    let stale_state = folds.get(i).map_or(FoldState::Open, |fold| fold.state);
    let mut nested: Vec<Fold> = folds.drain(i..i + cont).collect();
    if let (Some(first), Some(last)) = (nested.first(), nested.last()) {
        // A fold cannot start or end inside one of its children.
        top = top.min(first.top);
        bot = bot.max(last.last());
        for child in &mut nested {
            child.top -= top;
        }
    }
    folds.insert(
        i,
        Fold {
            top,
            len: bot - top + 1,
            nested,
            state: stale_state,
            small: None,
        },
    );
    Inserted {
        depth,
        use_level,
        closed,
    }
}

/// Create a fold from line "start" to line "end" (inclusive) in window `window`.
pub fn fold_create(window: Win, start_pos: Pos, end_pos: Pos) {
    let mut win = window;
    let (start, end) = if start_pos.lnum > end_pos.lnum {
        (end_pos, start_pos)
    } else {
        (start_pos, end_pos)
    };
    if foldmethod_is_marker(win) {
        // With 'foldmethod' = "marker" the fold lives in the buffer text.
        fold_create_markers(window, start, end);
        return;
    }
    checkupdate(win);
    let foldlevel = win.w_onebuf_opt.wo_fdl;
    let new = win.with_folds(|root| insert_new_fold(root, start.lnum, end.lnum, foldlevel));
    // We want the new fold to be closed. If it would remain open because of
    // using 'foldlevel', the folds containing it have to be closed by hand.
    let level = OptInt::try_from(new.depth).unwrap_or(OptInt::MAX);
    if new.use_level && !new.closed && level < foldlevel {
        close_fold(start, 1);
    }
    if !new.use_level {
        win.w_fold_manual = true;
    }
    // Closing changed only states, so the new fold is still the one over
    // `start` at its depth.
    win.with_folds(|root| {
        if let Some(fold) = fold_on_line(root, start.lnum, new.depth) {
            fold.state = FoldState::Closed;
            fold.small = None;
        }
    });
    changed_window_setting(win);
}

/// `start` — delete all folds from start to end when not 0
/// `end` — delete all folds from start to end when not 0
/// `recursive` — delete recursively if true
/// `had_visual` — true when Visual selection used
pub fn delete_fold(window: Win, start: LineNr, end: LineNr, recursive: c_int, had_visual: bool) {
    // Upstream's `level` and `maybe_small` live outside the loop below and
    // carry from one deleted fold to the next; so do these.
    let mut maybe_small = false;
    let mut level = 0;
    let mut lnum = start;
    let mut did_one = false;
    let mut first_lnum = MAXLNUM;
    let mut last_lnum: LineNr = 0;
    let win = window;
    checkupdate(win);
    while lnum <= end {
        let mut found: Option<(usize, LineNr)> = None;
        let mut lnum_off: LineNr = 0;
        let mut use_level = false;
        let mut depth = 0;
        // Descend to the innermost fold over `lnum`, stopping at the first
        // closed one — that is the fold the user is pointing at.
        loop {
            let top = win.with_folds(|root| fold_on_line(root, lnum, depth).map(|fold| fold.top));
            let Some(top) = top else { break };
            found = Some((depth, lnum_off));
            let (ul, ms) = (&mut use_level, &mut maybe_small);
            if closed_at(win, lnum, depth, ul, level, ms, lnum_off) {
                break;
            }
            lnum_off += top;
            level += 1;
            depth += 1;
        }
        let Some((found_depth, found_off)) = found else {
            lnum += 1;
            continue;
        };
        let pointed = lnum;
        if foldmethod_is_manual(win) {
            let span = win.with_folds(|root| {
                let (folds, lnum_rel) = list::list_on_line(root, pointed, found_depth)?;
                let i = find(folds, lnum_rel).ok()?;
                let span = (folds[i].top, folds[i].len);
                delete_fold_entry(folds, i, recursive != 0);
                Some(span)
            });
            let (top, len) = span.expect("the walk's own fold");
            lnum = top + len + found_off;
        } else {
            // 'foldmethod' is "marker": the fold only goes away once its
            // markers are gone from the buffer text.
            let (spans, (top, len)) = win
                .with_folds(|root| {
                    fold_on_line(root, pointed, found_depth).map(|fold| {
                        (
                            marker_spans(fold, recursive != 0, found_off),
                            (fold.top, fold.len),
                        )
                    })
                })
                .expect("the walk's own fold");
            lnum = top + len + found_off;
            first_lnum = first_lnum.min(top + found_off);
            last_lnum = last_lnum.max(lnum);
            if !did_one {
                parse_marker(win);
            }
            delete_fold_markers(win, &spans);
        }
        did_one = true;
        changed_window_setting(win);
    }
    if !did_one {
        emsg_nofold();
        if had_visual {
            redraw_buf_later(win.buffer(), UPD_INVERTED);
        }
    } else {
        check_cursor_col(window);
    }
    if last_lnum > 0 {
        let num_changed = (last_lnum - first_lnum) as int64_t;
        let buf = win.buffer();
        changed_lines(buf, first_lnum, 0, last_lnum, 0, false);
        buf_updates_send_changes(buf, first_lnum, num_changed, num_changed);
    }
}

/// Open or close fold for current window at position `pos`.
/// Repeat "count" times.
///
pub(super) fn set_fold_repeat(pos: Pos, count: c_int, do_open: c_int) {
    for n in 0..count {
        let mut done: c_int = DONE_NOTHING;
        set_manual_fold(pos, do_open != 0, false, Some(&mut done));
        if done & DONE_ACTION != 0 {
            continue;
        }
        if n == 0 && done & DONE_FOLD == 0 {
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
    pos: Pos,
    opening: bool,
    recurse: bool,
    donep: Option<&mut c_int>,
) -> LineNr {
    if foldmethod_is_diff(Win::current()) && Win::current().w_onebuf_opt.wo_scb != 0 {
        // 'scrollbind' in a diff: the matching fold in the other windows.
        for win in windows_in_tab(TabPage::current()) {
            if win.is_current() || !foldmethod_is_diff(win) || win.w_onebuf_opt.wo_scb == 0 {
                continue;
            }
            let dlnum = diff_lnum_win(Win::current().w_cursor.lnum, win);
            if dlnum != 0 {
                set_manual_fold_win(win, dlnum, opening, recurse, None);
            }
        }
    }
    set_manual_fold_win(Win::current(), pos.lnum, opening, recurse, donep)
}

/// What [`set_folds_on_line`] found and did.
struct ManualFold {
    found_fold: bool,
    done: c_int,
    next: LineNr,
}

/// The tree half of [`set_manual_fold_win`]: open or close the fold over
/// `lnum` in `root`, and say where the next closable line is.
fn set_folds_on_line(
    root: &mut Vec<Fold>,
    lnum: LineNr,
    opening: bool,
    recurse: bool,
    foldlevel: OptInt,
) -> ManualFold {
    let mut lnum_rel = lnum;
    let mut level: OptInt = 0;
    let mut use_level = false;
    let mut found_fold = false;
    let mut next = MAXLNUM;
    let mut off: LineNr = 0;
    let mut done: c_int = 0;
    // The depth of the innermost fold the walk passed through open: the one
    // a close acts on.
    let mut found: Option<usize> = None;
    let mut depth = 0;
    let mut folds: &mut Vec<Fold> = &mut *root;
    loop {
        let i = match find(folds, lnum_rel) {
            Ok(i) => i,
            Err(i) => {
                if let Some(fold) = folds.get(i) {
                    next = fold.top + off;
                }
                break;
            }
        };
        found_fold = true;
        if let Some(fold) = folds.get(i + 1) {
            next = fold.top + off;
        }
        let fold = &mut folds[i];
        if use_level || fold.is(FoldState::Level) {
            use_level = true;
            fold.state = if level >= foldlevel {
                FoldState::Closed
            } else {
                FoldState::Open
            };
            for child in &mut fold.nested {
                child.state = FoldState::Level;
            }
        }
        if !opening && recurse {
            if !fold.is(FoldState::Closed) {
                done |= DONE_ACTION;
                fold.state = FoldState::Closed;
            }
        } else if fold.is(FoldState::Closed) {
            if opening {
                fold.state = FoldState::Open;
                done |= DONE_ACTION;
                if recurse {
                    fold_open_nested(fold);
                }
            }
            break;
        }
        found = Some(depth);
        lnum_rel -= fold.top;
        off += fold.top;
        level += 1;
        depth += 1;
        folds = &mut fold.nested;
    }
    if found_fold
        && !opening
        && let Some(depth) = found
        && let Some(fold) = fold_on_line(root, lnum, depth)
    {
        // Closing: the innermost fold that was already open closes.
        fold.state = FoldState::Closed;
        done |= DONE_ACTION;
    }
    ManualFold {
        found_fold,
        done,
        next,
    }
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
    lnum: LineNr,
    opening: bool,
    recurse: bool,
    donep: Option<&mut c_int>,
) -> LineNr {
    checkupdate(win);
    let foldlevel = win.w_onebuf_opt.wo_fdl;
    let ManualFold {
        found_fold,
        mut done,
        next,
    } = win.with_folds(|root| set_folds_on_line(root, lnum, opening, recurse, foldlevel));
    if found_fold {
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
pub(super) fn fold_open_nested(fold: &mut Fold) {
    for child in &mut fold.nested {
        fold_open_nested(child);
        child.state = FoldState::Open;
    }
}

/// Check if a fold is closed and update the info needed to check nested folds.
///
/// `use_levelp` — true: outer fold had FD_LEVEL
/// `fold` — fold to check
/// `level` — folding depth
/// `maybe_smallp` — true: the outer fold had no `small` answer yet
///
/// [`Closed::CountRows`] means the fold is closed by its state but whether
/// 'foldminlines' lets it be depends on its screen lines, which the caller
/// counts with [`fits_in_rows`] -- the window cannot be asked while the tree
/// is lent out.
pub(super) fn check_closed(
    fold: &mut Fold,
    use_levelp: &mut bool,
    level: c_int,
    maybe_smallp: &mut bool,
    foldlevel: OptInt,
    foldminlines: OptInt,
) -> Closed {
    let mut closed = false;
    if *use_levelp || fold.is(FoldState::Level) {
        *use_levelp = true;
        if OptInt::from(level) >= foldlevel {
            closed = true;
        }
    } else if fold.is(FoldState::Closed) {
        closed = true;
    }
    if fold.small.is_none() {
        *maybe_smallp = true;
    }
    if !closed {
        return Closed::No;
    }
    if *maybe_smallp {
        fold.small = None;
    }
    if fold.small.is_none() {
        forget_small_flags(&mut fold.nested);
        if OptInt::from(fold.len) > foldminlines {
            fold.small = Some(false);
        } else {
            return Closed::CountRows;
        }
    }
    // 'foldminlines' vetoes a fold too short to be worth drawing closed.
    if fold.small == Some(true) {
        Closed::No
    } else {
        Closed::Yes
    }
}

/// [`check_closed`] for the fold over `lnum` at depth `depth` of `win`'s
/// tree, counting its screen lines when 'foldminlines' needs them.
///
/// `lnum_off` — line number offset for the fold's `top`
pub(super) fn closed_at(
    win: Win,
    lnum: LineNr,
    depth: usize,
    use_levelp: &mut bool,
    level: c_int,
    maybe_smallp: &mut bool,
    lnum_off: LineNr,
) -> bool {
    let (foldlevel, foldminlines) = (win.w_onebuf_opt.wo_fdl, win.w_onebuf_opt.wo_fml);
    let verdict = win.with_folds(|root| {
        let fold = fold_on_line(root, lnum, depth)?;
        let verdict = check_closed(
            fold,
            use_levelp,
            level,
            maybe_smallp,
            foldlevel,
            foldminlines,
        );
        Some((verdict, fold.top, fold.len))
    });
    match verdict {
        None | Some((Closed::No, ..)) => false,
        Some((Closed::Yes, ..)) => true,
        Some((Closed::CountRows, top, len)) => {
            let small = fits_in_rows(win, top + lnum_off, len, foldminlines);
            win.with_folds(|root| {
                if let Some(fold) = fold_on_line(root, lnum, depth) {
                    fold.small = Some(small);
                }
            });
            !small
        }
    }
}

/// C's `emsg(_(e_nofold))`, which every command that found no fold gives.
fn emsg_nofold() {
    emsg(gettext(e_nofold));
}
