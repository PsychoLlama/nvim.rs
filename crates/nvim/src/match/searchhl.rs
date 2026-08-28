//! 'hlsearch' and match highlighting, one window line at a time.
//!
//! The drawing code calls these in a fixed order: [`init_search_hl`] once
//! per window redraw, [`prepare_search_hl`] once per line to advance every
//! pattern to that line, [`prepare_search_hl_line`] to find what is already
//! highlighted at the left edge, then [`update_search_hl`] per column.
//! `search_hl` (the `'hlsearch'` pattern) and the window's match list are
//! walked together, ordered by priority, with SEARCH_HL_PRIORITY (0) as
//! `'hlsearch'`'s own place in that order.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::option::cpo_has;
use crate::pos::MAXCOL;
use crate::search::SEARCH_HL_PRIORITY;
use crate::types::CpoFlag;
use crate::winlayer::Win;

/// Walks `search_hl` together with a window's match list.
///
/// Four functions here need the same walk and each spelled out the same
/// three-branch pick, `shl_flag` latch and conditional advance. Two orders
/// are wanted: [`Order::SearchFirst`], which takes `'hlsearch'` before any
/// match, and [`Order::ByPriority`], which takes it at its place in the
/// priority order.
struct ShlWalk {
    /// The next match to visit, or null once the list is exhausted.
    cur: *mut matchitem_T,
    /// The `'hlsearch'` state, which is visited exactly once.
    search_hl: *mut match_T,
    /// Whether `search_hl` has been visited.
    taken: bool,
    /// Whether the last item handed out was a match, so `cur` has to advance
    /// before the next one. (Upstream advances at the *bottom* of the loop
    /// body, which is the same thing.)
    advance: bool,
    order: Order,
}

/// Which of `'hlsearch'` and the match list [`ShlWalk`] visits first.
#[derive(Copy, Clone, PartialEq, Eq)]
enum Order {
    /// `'hlsearch'` first, then every match. Used where the body's work is
    /// independent per item and the order cannot matter.
    SearchFirst,
    /// Priority order: `'hlsearch'` once the remaining matches have dropped
    /// to [`SEARCH_HL_PRIORITY`] or below. Used where the last writer wins.
    ByPriority,
}

impl ShlWalk {
    /// # Safety
    /// `wp` and `search_hl` must be live.
    #[inline(always)]
    unsafe fn new(wp: *mut win_T, search_hl: *mut match_T, order: Order) -> Self {
        // SAFETY: the caller's window.
        Self {
            cur: unsafe { (*wp).w_match_head },
            search_hl,
            taken: false,
            advance: false,
            order,
        }
    }

    /// The next `(shl, cur)` pair, where `cur` is null for `'hlsearch'`.
    ///
    /// # Safety
    /// The match list must not be modified during the walk.
    #[inline(always)]
    unsafe fn next(&mut self) -> Option<(*mut match_T, *mut matchitem_T)> {
        // SAFETY: the caller's match list.
        if self.advance {
            self.cur = unsafe { (*self.cur).mit_next };
            self.advance = false;
        }
        if self.cur.is_null() && self.taken {
            return None;
        }
        let take_search = !self.taken
            && (self.cur.is_null()
                || self.order == Order::SearchFirst
                || unsafe { (*self.cur).mit_priority } > SEARCH_HL_PRIORITY);
        if take_search {
            self.taken = true;
            Some((self.search_hl, ::core::ptr::null_mut()))
        } else {
            self.advance = true;
            Some((unsafe { &raw mut (*self.cur).mit_hl }, self.cur))
        }
    }
}

/// Resets every pattern's search state for a fresh window redraw.
///
/// Each match gets a private copy of its compiled regexp (`mit_hl.rm`) so
/// that the per-line search can advance it, and a fresh `'redrawtime'`
/// budget.
///
/// # Safety
/// `wp` and `search_hl` must be live.
pub(crate) unsafe fn init_search_hl(wp: *mut win_T, search_hl: *mut match_T) {
    // SAFETY: the caller's window and search state.
    let mut cur = unsafe { (*wp).w_match_head };
    while !cur.is_null() {
        // The highlight state borrows the item's program; the
        // item keeps owning it, which is why this is a shallow
        // clone and not a compile of its own.
        unsafe { (*cur).mit_hl.rm = (*cur).mit_match.clone() };
        unsafe {
            (*cur).mit_hl.attr = if (*cur).mit_hlg_id == 0 {
                0
            } else {
                syn_id2attr((*cur).mit_hlg_id)
            }
        };
        unsafe { (*cur).mit_hl.buf = (*wp).w_buffer };
        unsafe { (*cur).mit_hl.lnum = 0 };
        unsafe { (*cur).mit_hl.first_lnum = 0 };
        unsafe { (*cur).mit_hl.tm = profile_setlimit(p_rdt.get()) };
        cur = unsafe { (*cur).mit_next };
    }
    unsafe { (*search_hl).buf = (*wp).w_buffer };
    unsafe { (*search_hl).lnum = 0 };
    unsafe { (*search_hl).first_lnum = 0 };
    unsafe { (*search_hl).attr = win_hl_attr(wp, HLF_L) };
    // The time limit is set at the top level, for every window at once.
}

/// Finds the next `matchaddpos()` position on `lnum` at or after `mincol`.
///
/// Answers 1 on a match, having filled `shl` in as if a regexp had matched.
/// The positions are not sorted, so this also swaps the winner to the front
/// of the still-unvisited tail, which is what makes repeated calls walk one
/// line's positions left to right.
///
/// # Safety
/// `shl` and `match_0` must be live.
unsafe fn next_search_hl_pos(
    shl: *mut match_T,
    lnum: linenr_T,
    match_0: *mut matchitem_T,
    mincol: colnr_T,
) -> c_int {
    // SAFETY: the caller's match state.
    let mut found = -1;

    unsafe { (*shl).lnum = 0 };
    let mut i = unsafe { (*match_0).mit_pos_cur };
    while i < unsafe { (*match_0).mit_pos_count } {
        let pos = unsafe { (*match_0).mit_pos_array.offset(i as isize) };
        if unsafe { (*pos).lnum } == 0 {
            break;
        }
        // A whole-line position (len 0) before the column asked for is
        // already behind us; a sized one may still reach past `mincol`.
        if unsafe { (*pos).len } == 0 && unsafe { (*pos).col } < mincol {
            i += 1;
            continue;
        }
        if unsafe { (*pos).lnum } == lnum {
            if found >= 0 {
                let best = unsafe { (*match_0).mit_pos_array.offset(found as isize) };
                if unsafe { (*pos).col } < unsafe { (*best).col } {
                    unsafe { ::core::ptr::swap(pos, best) };
                }
            } else {
                found = i;
            }
        }
        i += 1;
    }

    unsafe { (*match_0).mit_pos_cur = 0 };
    if found < 0 {
        return 0;
    }

    let best = unsafe { (*match_0).mit_pos_array.offset(found as isize) };
    // Column 0 means the whole line; otherwise the position is 1-based
    // and `len` wide.
    let (start, end) = if unsafe { (*best).col } == 0 {
        (0, MAXCOL)
    } else {
        (
            unsafe { (*best).col } - 1,
            unsafe { (*best).col } - 1 + unsafe { (*best).len },
        )
    };

    unsafe { (*shl).lnum = lnum };
    unsafe { (*shl).rm.startpos[0].lnum = 0 };
    unsafe { (*shl).rm.startpos[0].col = start };
    unsafe { (*shl).rm.endpos[0].lnum = 0 };
    unsafe { (*shl).rm.endpos[0].col = end };
    unsafe { (*shl).is_addpos = true };
    unsafe { (*shl).has_cursor = false };
    unsafe { (*match_0).mit_pos_cur = found + 1 };
    1
}

/// Advances `shl` to its next match on `lnum` at or past `mincol`.
///
/// A previous match is assumed to be before `lnum` unless `shl->lnum` is
/// zero. `cur` is the match item when `shl` belongs to one (so that
/// `matchaddpos()` positions can be read), null for `'hlsearch'`.
///
/// Any buffer-line pointer the caller holds is invalidated: a multi-line
/// regexp can force the line to be re-fetched.
///
/// # Safety
/// `win`, `search_hl` and `shl` must be live; `cur` must be null or live.
unsafe fn next_search_hl(
    win: *mut win_T,
    search_hl: *mut match_T,
    shl: *mut match_T,
    lnum: linenr_T,
    mincol: colnr_T,
    cur: *mut matchitem_T,
) {
    // SAFETY: the caller's window and match state.
    let called_emsg_before = called_emsg.get();

    // `:{range}s/pat` only highlights inside the range.
    if (lnum < search_first_line.get() || lnum > search_last_line.get()) && cur.is_null() {
        unsafe { (*shl).lnum = 0 };
        return;
    }

    if unsafe { (*shl).lnum } != 0 {
        // Three cases: `lnum` is below the previous match, so start
        // again; the previous match already includes `mincol`, so keep
        // it; otherwise continue after it.
        let l = unsafe { (*shl).lnum } + unsafe { (*shl).rm.endpos[0].lnum }
            - unsafe { (*shl).rm.startpos[0].lnum };
        if lnum > l {
            unsafe { (*shl).lnum = 0 };
        } else if lnum < l || unsafe { (*shl).rm.endpos[0].col } > mincol {
            return;
        }
    }

    // Search until a match that includes `mincol` turns up, or none does.
    loop {
        if profile_passed_limit(unsafe { (*shl).tm }) {
            unsafe { (*shl).lnum = 0 }; // no match found in time
            break;
        }

        let matchcol: colnr_T = if unsafe { (*shl).lnum } == 0 {
            // No useful previous match: search from the line's start.
            0
        } else if !cpo_has(CpoFlag::SEARCH)
            || (unsafe { (*shl).rm.endpos[0].lnum } == 0
                && unsafe { (*shl).rm.endpos[0].col } <= unsafe { (*shl).rm.startpos[0].col })
        {
            // Not Vi-compatible, or an empty match: continue at the next
            // character, and stop if that is past the end of the line.
            let at = unsafe { (*shl).rm.startpos[0].col };
            let ml = unsafe { ml_get_buf((*shl).buf, lnum).offset(at as isize) };
            if unsafe { *ml } == 0 {
                unsafe { (*shl).lnum = 0 };
                break;
            }
            at + unsafe { utfc_ptr2len(ml) }
        } else {
            // Vi-compatible: continue at the end of the previous match.
            unsafe { (*shl).rm.endpos[0].col }
        };

        unsafe { (*shl).lnum = lnum };
        let mut nmatched = 0;
        if !unsafe { (*shl).rm.regprog }.is_null() {
            // Whether `shl->rm` shares `cur`'s compiled regexp, which
            // `vim_regexec_multi` may free and recompile under us.
            let regprog_is_copy = shl != search_hl
                && !cur.is_null()
                && shl == unsafe { &raw mut (*cur).mit_hl }
                && unsafe { (*cur).mit_match.regprog } == unsafe { (*cur).mit_hl.rm.regprog };
            let mut timed_out: c_int = 0;

            nmatched = unsafe {
                vim_regexec_multi(
                    &raw mut (*shl).rm,
                    win,
                    (*shl).buf,
                    lnum,
                    matchcol,
                    &raw mut (*shl).tm,
                    &raw mut timed_out,
                )
            };
            if regprog_is_copy {
                unsafe { (*cur).mit_match.regprog = (*cur).mit_hl.rm.regprog };
            }
            if called_emsg.get() > called_emsg_before || got_int.get() || timed_out != 0 {
                // Something went wrong in the regexp: stop using it.
                if shl == search_hl {
                    // A match's regprog is a copy and must not be freed.
                    unsafe { vim_regfree((*shl).rm.regprog) };
                    unsafe { set_no_hlsearch(true) };
                }
                unsafe { (*shl).rm.regprog = ::core::ptr::null_mut() };
                unsafe { (*shl).lnum = 0 };
                got_int.set(false); // avoid the "Type :quit to exit Vim" message
                break;
            }
        } else if !cur.is_null() {
            nmatched = unsafe { next_search_hl_pos(shl, lnum, cur, matchcol) };
        }

        if nmatched == 0 {
            unsafe { (*shl).lnum = 0 };
            break;
        }
        if unsafe { (*shl).rm.startpos[0].lnum } > 0
            || unsafe { (*shl).rm.startpos[0].col } >= mincol
            || nmatched > 1
            || unsafe { (*shl).rm.endpos[0].col } > mincol
        {
            unsafe { (*shl).lnum += (*shl).rm.startpos[0].lnum };
            break; // useful match found
        }
    }
}

/// Advances every multi-line pattern to `lnum`, or past it.
///
/// Only a pattern that can span lines needs this: it may have started above
/// the window, so the search restarts from the window top (or from just
/// after the closest closed fold) and steps forward line by line.
///
/// # Safety
/// `wp` and `search_hl` must be live.
pub(crate) unsafe fn prepare_search_hl(wp: *mut win_T, search_hl: *mut match_T, lnum: linenr_T) {
    // SAFETY: the caller's window and search state.
    let mut walk = unsafe { ShlWalk::new(wp, search_hl, Order::SearchFirst) };
    while let Some((shl, cur)) = unsafe { walk.next() } {
        if unsafe { (*shl).rm.regprog }.is_null()
            || unsafe { (*shl).lnum } != 0
            || unsafe { re_multiline((*shl).rm.regprog) } == 0
        {
            continue;
        }

        if unsafe { (*shl).first_lnum } == 0 {
            unsafe { (*shl).first_lnum = lnum };
            while unsafe { (*shl).first_lnum } > unsafe { (*wp).w_topline } {
                if has_folding(
                    unsafe { Win::new(wp) },
                    unsafe { (*shl).first_lnum } - 1,
                    None,
                    None,
                ) {
                    break;
                }
                unsafe { (*shl).first_lnum -= 1 };
            }
        }
        if !cur.is_null() {
            unsafe { (*cur).mit_pos_cur = 0 };
        }

        // A position match is "in progress" while it still has unvisited
        // positions on the line it is on.
        let mut pos_inprogress = true;
        let mut n: colnr_T = 0;
        while unsafe { (*shl).first_lnum } < lnum
            && (!unsafe { (*shl).rm.regprog }.is_null() || (!cur.is_null() && pos_inprogress))
        {
            unsafe { next_search_hl(wp, search_hl, shl, (*shl).first_lnum, n, cur) };
            pos_inprogress = !cur.is_null() && unsafe { (*cur).mit_pos_cur } != 0;
            if unsafe { (*shl).lnum } != 0 {
                unsafe {
                    (*shl).first_lnum =
                        (*shl).lnum + (*shl).rm.endpos[0].lnum - (*shl).rm.startpos[0].lnum
                };
                n = unsafe { (*shl).rm.endpos[0].col };
            } else {
                unsafe { (*shl).first_lnum += 1 };
                n = 0;
            }
        }
    }
}

/// Records whether the cursor is inside `shl`'s match, which is what makes
/// `CurSearch` apply to one match and not the others.
///
/// # Safety
/// `wp` and `shl` must be live.
unsafe fn check_cur_search_hl(wp: *mut win_T, shl: *mut match_T) {
    // SAFETY: the caller's window and match state.
    let linecount = unsafe { (*shl).rm.endpos[0].lnum } - unsafe { (*shl).rm.startpos[0].lnum };
    let cursor = unsafe { (*wp).w_cursor };
    unsafe {
        (*shl).has_cursor = cursor.lnum >= (*shl).lnum
            && cursor.lnum <= (*shl).lnum + linecount
            && (cursor.lnum > (*shl).lnum || cursor.col >= (*shl).rm.startpos[0].col)
            && (cursor.lnum < (*shl).lnum + linecount || cursor.col < (*shl).rm.endpos[0].col)
    };
}

/// Prepares every pattern for one window line, and answers whether any of
/// them highlights part of it.
///
/// A pattern already covering `mincol` — the leftmost column drawn, which is
/// not column 0 under `'nowrap'` or a `'smoothscroll'` skip — also writes
/// `search_attr`, so the drawing loop starts with the right attribute.
///
/// # Safety
/// Every pointer must be live; `line` is re-read, because a multi-line
/// regexp can invalidate it.
pub(crate) unsafe fn prepare_search_hl_line(
    wp: *mut win_T,
    lnum: linenr_T,
    mincol: colnr_T,
    line: *mut *mut c_char,
    search_hl: *mut match_T,
    search_attr: *mut c_int,
    search_attr_from_match: *mut bool,
) -> bool {
    // SAFETY: the caller's window, line and out-parameters.
    let mut area_highlighting = false;
    let mut walk = unsafe { ShlWalk::new(wp, search_hl, Order::SearchFirst) };
    while let Some((shl, cur)) = unsafe { walk.next() } {
        unsafe { (*shl).startcol = MAXCOL };
        unsafe { (*shl).endcol = MAXCOL };
        unsafe { (*shl).attr_cur = 0 };
        unsafe { (*shl).is_addpos = false };
        unsafe { (*shl).has_cursor = false };
        if !cur.is_null() {
            unsafe { (*cur).mit_pos_cur = 0 };
        }
        unsafe { next_search_hl(wp, search_hl, shl, lnum, mincol, cur) };

        // Re-read the line: a multi-line regexp may have invalidated it.
        unsafe { *line = ml_get_buf((*wp).w_buffer, lnum) };

        if unsafe { (*shl).lnum } == 0 || unsafe { (*shl).lnum } > lnum {
            continue;
        }

        // A match that started on an earlier line covers this one from
        // column 0; one that ends on a later line covers it to the end.
        unsafe {
            (*shl).startcol = if (*shl).lnum == lnum {
                (*shl).rm.startpos[0].col
            } else {
                0
            }
        };
        unsafe {
            (*shl).endcol =
                if lnum == (*shl).lnum + (*shl).rm.endpos[0].lnum - (*shl).rm.startpos[0].lnum {
                    (*shl).rm.endpos[0].col
                } else {
                    MAXCOL
                }
        };

        // Before the columns are widened below.
        if shl == search_hl {
            unsafe { check_cur_search_hl(wp, shl) };
        }

        // An empty match still highlights one character.
        if unsafe { (*shl).startcol } == unsafe { (*shl).endcol } {
            if unsafe { *(*line).offset((*shl).endcol as isize) } != 0 {
                unsafe { (*shl).endcol += utfc_ptr2len((*line).offset((*shl).endcol as isize)) };
            } else {
                unsafe { (*shl).endcol += 1 };
            }
        }
        if unsafe { (*shl).startcol } < mincol {
            // Already highlighted at the left edge.
            unsafe { (*shl).attr_cur = (*shl).attr };
            unsafe { *search_attr = (*shl).attr };
            unsafe { *search_attr_from_match = shl != search_hl };
        }
        area_highlighting = true;
    }
    area_highlighting
}

/// Advances every pattern past `col` and answers the attribute that wins
/// there.
///
/// Called once per column. Each pattern is stepped until it either covers
/// `col` or starts after it; then a second walk, in priority order, takes
/// the last non-zero attribute, so the highest-priority pattern wins.
///
/// # Safety
/// Every pointer must be live; `line` is re-read, because a multi-line
/// regexp can invalidate it.
#[allow(clippy::too_many_arguments)]
pub(crate) unsafe fn update_search_hl(
    wp: *mut win_T,
    lnum: linenr_T,
    col: colnr_T,
    line: *mut *mut c_char,
    search_hl: *mut match_T,
    has_match_conc: *mut c_int,
    match_conc: *mut c_int,
    lcs_eol_todo: bool,
    on_last_col: *mut bool,
    search_attr_from_match: *mut bool,
) -> c_int {
    // SAFETY: the caller's window, line and out-parameters.
    let mut walk = unsafe { ShlWalk::new(wp, search_hl, Order::ByPriority) };
    while let Some((shl, cur)) = unsafe { walk.next() } {
        if !cur.is_null() {
            unsafe { (*cur).mit_pos_cur = 0 };
        }
        let mut pos_inprogress = true;
        while !unsafe { (*shl).rm.regprog }.is_null() || (!cur.is_null() && pos_inprogress) {
            if unsafe { (*shl).startcol } != MAXCOL
                && col >= unsafe { (*shl).startcol }
                && col < unsafe { (*shl).endcol }
            {
                // Inside the match. Widen it to a whole character.
                let next_col = col + unsafe { utfc_ptr2len((*line).offset(col as isize)) };
                if unsafe { (*shl).endcol } < next_col {
                    unsafe { (*shl).endcol = next_col };
                }
                // The match holding the cursor uses `CurSearch`.
                if shl == search_hl && unsafe { (*shl).has_cursor } {
                    unsafe { (*shl).attr_cur = win_hl_attr(wp, HLF_LC) };
                    if unsafe { (*shl).attr_cur } != unsafe { (*shl).attr } {
                        search_hl_has_cursor_lnum.set(lnum);
                    }
                } else {
                    unsafe { (*shl).attr_cur = (*shl).attr };
                }
                // A match in the `Conceal` group hides its text; 2 marks
                // the first cell, so the replacement character is drawn
                // once rather than once per cell.
                if !cur.is_null()
                    && shl != search_hl
                    && unsafe { syn_name2id(c"Conceal".as_ptr()) } == unsafe { (*cur).mit_hlg_id }
                {
                    unsafe { *has_match_conc = if col == (*shl).startcol { 2 } else { 1 } };
                    unsafe { *match_conc = (*cur).mit_conceal_char };
                } else {
                    unsafe { *has_match_conc = 0 };
                }
                break;
            }
            if col != unsafe { (*shl).endcol } {
                break;
            }

            // Just past the end: look for the next match on this line.
            unsafe { (*shl).attr_cur = 0 };
            unsafe { next_search_hl(wp, search_hl, shl, lnum, col, cur) };
            pos_inprogress = !cur.is_null() && unsafe { (*cur).mit_pos_cur } != 0;

            // Re-read the line: a multi-line regexp may have invalidated it.
            unsafe { *line = ml_get_buf((*wp).w_buffer, lnum) };

            if unsafe { (*shl).lnum } != lnum {
                break;
            }
            unsafe { (*shl).startcol = (*shl).rm.startpos[0].col };
            unsafe {
                (*shl).endcol = if (*shl).rm.endpos[0].lnum == 0 {
                    (*shl).rm.endpos[0].col
                } else {
                    MAXCOL
                }
            };
            if shl == search_hl {
                unsafe { check_cur_search_hl(wp, shl) };
            }
            if unsafe { (*shl).startcol } == unsafe { (*shl).endcol } {
                // Highlight the empty match, then try again after it.
                let p = unsafe { (*line).offset((*shl).endcol as isize) };
                unsafe { (*shl).endcol += if *p == 0 { 1 } else { utfc_ptr2len(p) } };
            }
            // Round again, in case the new match starts here.
        }
    }

    // The attribute of the highest-priority pattern covering `col`:
    // walking in priority order, the last writer wins.
    unsafe { *search_attr_from_match = false };
    let mut search_attr = unsafe { (*search_hl).attr_cur };
    let mut walk = unsafe { ShlWalk::new(wp, search_hl, Order::ByPriority) };
    while let Some((shl, _)) = unsafe { walk.next() } {
        if unsafe { (*shl).attr_cur } != 0 {
            search_attr = unsafe { (*shl).attr_cur };
            unsafe { *on_last_col = col + 1 >= (*shl).endcol };
            unsafe { *search_attr_from_match = shl != search_hl };
        }
    }

    // Under `'list'` the eol character is drawn from `'listchars'`, so
    // the match must not colour past the last real character.
    if unsafe { *(*line).offset(col as isize) } == 0
        && unsafe { (*wp).w_onebuf_opt.wo_list } != 0
        && !lcs_eol_todo
    {
        search_attr = 0;
    }
    search_attr
}

/// Whether the cell just past the end of the line should be highlighted.
///
/// True when a match started exactly there, or when it continues into the
/// next line — i.e. covers the line break itself. `matchaddpos()` matches
/// are excluded: they name real positions and never the break.
///
/// # Safety
/// `wp` and `search_hl` must be live.
pub(crate) unsafe fn get_prevcol_hl_flag(
    wp: *mut win_T,
    search_hl: *mut match_T,
    curcol: colnr_T,
) -> bool {
    // SAFETY: the caller's window and search state.
    // Not really at that column when text to the left is being skipped.
    let skip = if unsafe { (*wp).w_onebuf_opt.wo_wrap } != 0 {
        unsafe { (*wp).w_skipcol }
    } else {
        unsafe { (*wp).w_leftcol }
    };
    let prevcol = if skip > curcol { curcol + 1 } else { curcol };

    let covers = |m: *const match_T| {
        !unsafe { (*m).is_addpos }
            && (prevcol == unsafe { (*m).startcol }
                || (prevcol > unsafe { (*m).startcol } && unsafe { (*m).endcol } == MAXCOL))
    };

    if covers(search_hl) {
        return true;
    }
    let mut cur = unsafe { (*wp).w_match_head };
    while !cur.is_null() {
        if covers(unsafe { &raw const (*cur).mit_hl }) {
            return true;
        }
        cur = unsafe { (*cur).mit_next };
    }
    false
}

/// The attribute for the character after the text, when a match starts
/// exactly at `col - 1`.
///
/// # Safety
/// `wp`, `search_hl` and `char_attr` must be live.
pub(crate) unsafe fn get_search_match_hl(
    wp: *mut win_T,
    search_hl: *mut match_T,
    col: colnr_T,
    char_attr: *mut c_int,
) {
    // SAFETY: the caller's window and out-parameter.
    let mut walk = unsafe { ShlWalk::new(wp, search_hl, Order::ByPriority) };
    while let Some((shl, _)) = unsafe { walk.next() } {
        if col - 1 == unsafe { (*shl).startcol }
            && (shl == search_hl || !unsafe { (*shl).is_addpos })
        {
            unsafe { *char_attr = (*shl).attr };
        }
    }
}
