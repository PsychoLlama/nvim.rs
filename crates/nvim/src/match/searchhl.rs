//! 'hlsearch' and match highlighting, one window line at a time.
//!
//! The drawing code calls these in a fixed order: [`init_search_hl`] once
//! per window redraw, [`prepare_search_hl`] once per line to advance every
//! pattern to that line, [`prepare_search_hl_line`] to find what is already
//! highlighted at the left edge, then [`Win::update_search_hl`] per column.
//! The `'hlsearch'` state (the redraw's `screen_search_hl`) and the window's
//! match list are walked together, ordered by priority, with
//! SEARCH_HL_PRIORITY (0) as `'hlsearch'`'s own place in that order.
//!
//! A pattern's state is reached through [`Win::with_search_hl`], one short borrow per
//! access. The matcher reads the window while it runs; a match's regmatch
//! lives in the list's own storage, outside the window, so the borrow of it
//! the matcher advances does not overlap anything the matcher reads.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::drawscreen::state::screen_search_hl;
use crate::mbyte::c_cluster_len;
use crate::option::cpo_has;
use crate::option::defs::CpoFlag;
use crate::search::SEARCH_HL_PRIORITY;
use crate::winlayer::{Buf, Win};

/// Which pattern's highlight state: `'hlsearch'`'s, or the window's match
/// at this index.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Which {
    Search,
    Item(usize),
}

impl Win {
    /// Run `f` on the highlight state `which` names.
    #[inline(always)]
    fn with_search_hl<R>(mut self, which: Which, f: impl FnOnce(&mut MatchState) -> R) -> R {
        match which {
            Which::Search => screen_search_hl.with_mut(f),
            Which::Item(i) => f(&mut self.w_matches[i].mit_hl),
        }
    }

    /// Run `f` on the window's match at index `i`.
    #[inline(always)]
    fn with_match_item<R>(mut self, i: usize, f: impl FnOnce(&mut MatchItem) -> R) -> R {
        f(&mut self.w_matches[i])
    }

    /// Resets every pattern's search state for a fresh window redraw: each
    /// match gets its attribute, a fresh `'redrawtime'` budget, and its
    /// program back if the last redraw suspended it.
    pub(crate) fn init_search_hl(self) {
        let mut window = self;
        let conceal = if window.w_matches.is_empty() {
            0
        } else {
            syn_name2id(c"Conceal")
        };
        for item in &mut window.w_matches {
            let shl = &mut item.mit_hl;
            shl.rm.rmm_ic = item.mit_match.rmm_ic;
            shl.rm.rmm_maxcol = item.mit_match.rmm_maxcol;
            shl.attr = if item.mit_hlg_id == 0 {
                0
            } else {
                syn_id2attr(item.mit_hlg_id)
            };
            shl.conceals = item.mit_hlg_id != 0 && item.mit_hlg_id == conceal;
            shl.lnum = LineNr::ZERO;
            shl.first_lnum = LineNr::ZERO;
            shl.tm = profile_setlimit(p_rdt());
            item.mit_suspended = false;
        }
        let attr = win_hl_attr(self, HLF_L);
        screen_search_hl.with_mut(|shl| {
            shl.lnum = LineNr::ZERO;
            shl.first_lnum = LineNr::ZERO;
            shl.attr = attr;
        });
        // The time limit is set at the top level, for every window at once.
    }

    /// Advances every pattern past `col` and answers the attribute that wins
    /// there.
    ///
    /// Called once per column. Each pattern is stepped until it either covers
    /// `col` or starts after it; then a second walk, in priority order, takes
    /// the last non-zero attribute, so the highest-priority pattern wins.
    ///
    /// `char_len_here` answers the length of the character at `col` (0 at the
    /// end of the line) from the caller's own copy of the line. Upstream re-read
    /// the line after searching a pattern again, because a multi-line regexp
    /// reads other lines and its `ml_get` pointer could move; a copy cannot, so
    /// the answer stands for the whole column.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn update_search_hl(
        self,
        lnum: LineNr,
        col: ColNr,
        char_len_here: impl Fn() -> usize,
        has_match_conc: &mut c_int,
        match_conc: &mut c_int,
        lcs_eol_todo: bool,
        on_last_col: &mut bool,
        search_attr_from_match: &mut bool,
    ) -> c_int {
        // The attribute of the highest-priority pattern covering `col`: the
        // walk is in priority order and each pattern's state is final once the
        // walk leaves it, so the last non-zero attribute wins.
        *search_attr_from_match = false;
        let mut search_attr = 0;
        let mut walk = ShlWalk::new(Order::ByPriority);
        while let Some(which) = walk.next(self) {
            let is_item = matches!(which, Which::Item(_));
            let mut pos_inprogress = true;
            let mut first = true;
            let (attr_cur, endcol) = loop {
                // The common case -- `col` neither inside the match nor at its
                // end -- is settled by this one look at the state.
                let peek = peek(self, which, first, is_item && pos_inprogress);
                first = false;
                if !peek.live {
                    break (peek.attr_cur, peek.endcol);
                }
                let Peek {
                    startcol, endcol, ..
                } = peek;
                if startcol != ColNr::MAX && col >= startcol && col < endcol {
                    // Inside the match. Widen it to a whole character.
                    let next_col = col + col_len(char_len_here()).get();
                    let cursor_attr = match which {
                        Which::Search if screen_search_hl.with(|shl| shl.has_cursor) => {
                            Some(win_hl_attr(self, HLF_LC))
                        }
                        _ => None,
                    };
                    let (attr_cur, endcol) = self.with_search_hl(which, |shl| {
                        if shl.endcol < next_col {
                            shl.endcol = next_col;
                        }
                        // The match holding the cursor uses `CurSearch`.
                        if let Some(attr) = cursor_attr {
                            shl.attr_cur = attr;
                            if shl.attr_cur != shl.attr {
                                search_hl_has_cursor_lnum.set(lnum);
                            }
                        } else {
                            shl.attr_cur = shl.attr;
                        }
                        (shl.attr_cur, shl.endcol)
                    });
                    // A match in the `Conceal` group hides its text; 2 marks
                    // the first cell, so the replacement character is drawn
                    // once rather than once per cell.
                    match which {
                        Which::Item(i) if peek.conceals => {
                            *has_match_conc = if col == startcol { 2 } else { 1 };
                            *match_conc = self.w_matches[i].mit_conceal_char;
                        }
                        _ => *has_match_conc = 0,
                    }
                    break (attr_cur, endcol);
                }
                if col != endcol {
                    break (peek.attr_cur, endcol);
                }

                // Just past the end: look for the next match on this line.
                self.with_search_hl(which, |shl| shl.attr_cur = 0);
                next_search_hl(self, which, lnum, col);
                pos_inprogress = pos_in_progress(self, which);

                let found = self.with_search_hl(which, |shl| {
                    if shl.lnum != lnum {
                        return false;
                    }
                    shl.startcol = shl.rm.startpos[0].col;
                    shl.endcol = if shl.rm.endpos[0].lnum == LineNr::ZERO {
                        shl.rm.endpos[0].col
                    } else {
                        ColNr::MAX
                    };
                    true
                });
                if !found {
                    break self.with_search_hl(which, |shl| (shl.attr_cur, shl.endcol));
                }
                if which == Which::Search {
                    check_cur_search_hl(self);
                }
                let (startcol, endcol) =
                    self.with_search_hl(which, |shl| (shl.startcol, shl.endcol));
                if startcol == endcol {
                    // Highlight the empty match, then try again after it.
                    let width = empty_match_width(self, lnum, endcol);
                    self.with_search_hl(which, |shl| shl.endcol += width.get());
                }
                // Round again, in case the new match starts here.
            };
            if attr_cur != 0 {
                search_attr = attr_cur;
                *on_last_col = col + 1 >= endcol;
                *search_attr_from_match = is_item;
            }
        }

        // Under `'list'` the eol character is drawn from `'listchars'`, so
        // the match must not colour past the last real character.
        if self.w_onebuf_opt.wo_list != 0 && !lcs_eol_todo && char_len_here() == 0 {
            search_attr = 0;
        }
        search_attr
    }
}

/// Whether `which` still has a compiled pattern to search with this
/// redraw. A match that went wrong keeps its program but is suspended until
/// the next redraw; `'hlsearch'` drops its program, which the next redraw
/// compiles again.
#[inline(always)]
fn has_prog(window: Win, which: Which) -> bool {
    match which {
        Which::Search => screen_search_hl.with(|shl| shl.rm.regprog.is_some()),
        Which::Item(i) => window.w_matches[i].active(),
    }
}

/// Whether `which`'s pattern can match a line break.
fn is_multiline(window: Win, which: Which) -> bool {
    match which {
        Which::Search => screen_search_hl.with(|shl| shl.rm.is_multiline()),
        Which::Item(i) => {
            window.w_matches[i].active() && window.w_matches[i].mit_match.is_multiline()
        }
    }
}

impl MatchItem {
    /// Whether the match's program is in use this redraw.
    #[inline(always)]
    fn active(&self) -> bool {
        !self.mit_suspended && self.mit_match.regprog.is_some()
    }
}

/// Walks `'hlsearch'` together with a window's match list.
///
/// Four functions here need the same walk. Two orders are wanted:
/// [`Order::SearchFirst`], which takes `'hlsearch'` before any match, and
/// [`Order::ByPriority`], which takes it at its place in the priority order
/// (the list is kept in ascending priority).
#[derive(Debug)]
struct ShlWalk {
    /// The next match to visit.
    next: usize,
    /// Whether `'hlsearch'` has been visited.
    taken: bool,
    order: Order,
}

/// Which of `'hlsearch'` and the match list [`ShlWalk`] visits first.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Order {
    /// `'hlsearch'` first, then every match. Used where the body's work is
    /// independent per item and the order cannot matter.
    SearchFirst,
    /// Priority order: `'hlsearch'` once the remaining matches are above
    /// [`SEARCH_HL_PRIORITY`]. Used where the last writer wins.
    ByPriority,
}

impl ShlWalk {
    #[inline(always)]
    fn new(order: Order) -> Self {
        Self {
            next: 0,
            taken: false,
            order,
        }
    }

    /// The next pattern to visit.
    #[inline(always)]
    fn next(&mut self, window: Win) -> Option<Which> {
        let more = self.next < window.w_matches.len();
        if !more && self.taken {
            return None;
        }
        let take_search = !self.taken
            && (!more
                || self.order == Order::SearchFirst
                || window.w_matches[self.next].mit_priority > SEARCH_HL_PRIORITY);
        if take_search {
            self.taken = true;
            Some(Which::Search)
        } else {
            self.next += 1;
            Some(Which::Item(self.next - 1))
        }
    }
}

/// Finds the next `matchaddpos()` position on `lnum` at or after `mincol`.
///
/// Answers 1 (the lines it spans) on a match, having filled the item's
/// highlight state in as if a regexp had matched; 0 for none. The positions are not sorted, so this also swaps the
/// winner to the front of the still-unvisited tail, which is what makes
/// repeated calls walk one line's positions left to right.
fn next_search_hl_pos(item: &mut MatchItem, lnum: LineNr, mincol: ColNr) -> usize {
    item.mit_hl.lnum = LineNr::ZERO;
    let mut found: Option<usize> = None;
    let start = usize::try_from(item.mit_pos_cur).unwrap_or(0);
    for i in start..item.mit_pos_array.len() {
        let pos = &item.mit_pos_array[i];
        if pos.lnum == LineNr::ZERO {
            break;
        }
        // A whole-line position (len 0) before the column asked for is
        // already behind us; a sized one may still reach past `mincol`.
        if pos.len == 0 && pos.col < mincol {
            continue;
        }
        if pos.lnum == lnum {
            match found {
                Some(best) => {
                    if pos.col < item.mit_pos_array[best].col {
                        item.mit_pos_array.swap(i, best);
                    }
                }
                None => found = Some(i),
            }
        }
    }

    item.mit_pos_cur = 0;
    let Some(found) = found else {
        return 0;
    };

    let best = &item.mit_pos_array[found];
    // Column 0 means the whole line; otherwise the position is 1-based
    // and `len` wide.
    let (start, end) = if best.col == ColNr::ZERO {
        (ColNr::ZERO, ColNr::MAX)
    } else {
        (best.col - 1, best.col - 1 + best.len)
    };

    let shl = &mut item.mit_hl;
    shl.lnum = lnum;
    shl.rm.startpos[0].lnum = LineNr::ZERO;
    shl.rm.startpos[0].col = start;
    shl.rm.endpos[0].lnum = LineNr::ZERO;
    shl.rm.endpos[0].col = end;
    shl.is_addpos = true;
    shl.has_cursor = false;
    item.mit_pos_cur = c_int::try_from(found + 1).unwrap_or(c_int::MAX);
    1
}

/// Run `which`'s pattern against line `lnum` of `window` from `col`:
/// how many lines the match spans, 0 for none.
///
/// A match lends its program to its highlight regmatch for the call, and
/// takes back what the matcher hands back -- a program recompiled for the
/// other engine, say -- for later redraws, as upstream's shared program did.
fn exec_shl(window: Win, which: Which, lnum: LineNr, col: ColNr, timed_out: &mut bool) -> usize {
    let buffer = window.buffer();
    let nmatched = match which {
        // The matcher does not reach the redraw's matcher cell.
        Which::Search => screen_search_hl.with_mut(|shl| {
            let tm = shl.tm;
            let rm = &mut shl.rm;
            vim_regexec_multi(
                rm,
                Some(window),
                buffer,
                lnum,
                col,
                Some(&tm),
                Some(timed_out),
            )
        }),
        Which::Item(i) => window.with_match_item(i, |item| {
            let tm = item.mit_hl.tm;
            let rm = &mut item.mit_hl.rm;
            rm.regprog = item.mit_match.regprog.take();
            let nmatched = vim_regexec_multi(
                rm,
                Some(window),
                buffer,
                lnum,
                col,
                Some(&tm),
                Some(timed_out),
            );
            item.mit_match.regprog = rm.regprog.take();
            nmatched
        }),
    };
    usize::try_from(nmatched).unwrap_or(0)
}

/// Advances `which` to its next match on `lnum` at or past `mincol`.
///
/// A previous match is assumed to be before `lnum` unless its `lnum` is
/// zero.
fn next_search_hl(window: Win, which: Which, lnum: LineNr, mincol: ColNr) {
    let called_emsg_before = called_emsg.get();
    let set_lnum = |lnum: LineNr| window.with_search_hl(which, |shl| shl.lnum = lnum);

    // `:{range}s/pat` only highlights inside the range.
    if (lnum < search_first_line.get() || lnum > search_last_line.get()) && which == Which::Search {
        set_lnum(LineNr::ZERO);
        return;
    }

    let (prev_lnum, start, end) = window.with_search_hl(which, |shl| {
        (shl.lnum, shl.rm.startpos[0], shl.rm.endpos[0])
    });
    if prev_lnum != LineNr::ZERO {
        // Three cases: `lnum` is below the previous match, so start
        // again; the previous match already includes `mincol`, so keep
        // it; otherwise continue after it.
        let l = prev_lnum + end.lnum.get() - start.lnum;
        if lnum > LineNr::new(l) {
            set_lnum(LineNr::ZERO);
        } else if lnum < LineNr::new(l) || end.col > mincol {
            return;
        }
    }

    // Search until a match that includes `mincol` turns up, or none does.
    loop {
        let (tm, prev_lnum, start, end) = window.with_search_hl(which, |shl| {
            (shl.tm, shl.lnum, shl.rm.startpos[0], shl.rm.endpos[0])
        });
        if profile_passed_limit(tm) {
            set_lnum(LineNr::ZERO); // no match found in time
            break;
        }

        let matchcol: ColNr = if prev_lnum == LineNr::ZERO {
            // No useful previous match: search from the line's start.
            ColNr::ZERO
        } else if !cpo_has(CpoFlag::SEARCH) || (end.lnum == LineNr::ZERO && end.col <= start.col) {
            // Not Vi-compatible, or an empty match: continue at the next
            // character, and stop if that is past the end of the line.
            let at = usize::try_from(start.col).unwrap_or(0);
            let Some(len) = cluster_len_in_line(window.buffer(), lnum, at) else {
                set_lnum(LineNr::ZERO);
                break;
            };
            start.col + col_len(len).get()
        } else {
            // Vi-compatible: continue at the end of the previous match.
            end.col
        };

        set_lnum(lnum);
        let mut nmatched = 0;
        if has_prog(window, which) {
            let mut timed_out = false;
            nmatched = exec_shl(window, which, lnum, matchcol, &mut timed_out);
            if called_emsg.get() > called_emsg_before || got_int.get() || timed_out {
                // Something went wrong in the regexp: stop using it --
                // `'hlsearch'` for good, a match for the rest of this redraw.
                match which {
                    Which::Search => {
                        set_no_hlsearch(true);
                        screen_search_hl.with_mut(|shl| shl.rm.regprog = None);
                    }
                    Which::Item(i) => window.with_match_item(i, |item| item.mit_suspended = true),
                }
                set_lnum(LineNr::ZERO);
                got_int.set(false); // avoid the "Type :quit to exit Vim" message
                break;
            }
        } else if let Which::Item(i) = which {
            nmatched = window.with_match_item(i, |item| next_search_hl_pos(item, lnum, matchcol));
        }

        if nmatched == 0 {
            set_lnum(LineNr::ZERO);
            break;
        }
        let useful = window.with_search_hl(which, |shl| {
            let useful = shl.rm.startpos[0].lnum > LineNr::ZERO
                || shl.rm.startpos[0].col >= mincol
                || nmatched > 1
                || shl.rm.endpos[0].col > mincol;
            if useful {
                shl.lnum += shl.rm.startpos[0].lnum.get();
            }
            useful
        });
        if useful {
            break; // useful match found
        }
    }
}

/// Reset a match's position walk; nothing for `'hlsearch'`.
#[inline(always)]
fn reset_pos_cur(window: Win, which: Which) {
    if let Which::Item(i) = which {
        window.with_match_item(i, |item| item.mit_pos_cur = 0);
    }
}

/// Whether a position match still has unvisited positions on the line it is
/// on; never for `'hlsearch'`.
#[inline(always)]
fn pos_in_progress(window: Win, which: Which) -> bool {
    match which {
        Which::Search => false,
        Which::Item(i) => window.w_matches[i].mit_pos_cur != 0,
    }
}

/// Advances every multi-line pattern to `lnum`, or past it.
///
/// Only a pattern that can span lines needs this: it may have started above
/// the window, so the search restarts from the window top (or from just
/// after the closest closed fold) and steps forward line by line.
pub(crate) fn prepare_search_hl(window: Win, lnum: LineNr) {
    let mut walk = ShlWalk::new(Order::SearchFirst);
    while let Some(which) = walk.next(window) {
        if !is_multiline(window, which)
            || window.with_search_hl(which, |shl| shl.lnum) != LineNr::ZERO
        {
            continue;
        }

        if window.with_search_hl(which, |shl| shl.first_lnum) == LineNr::ZERO {
            let mut first = lnum;
            while first > window.w_topline {
                if window.has_folding(first - 1, None, None) {
                    break;
                }
                first -= 1;
            }
            window.with_search_hl(which, |shl| shl.first_lnum = first);
        }
        reset_pos_cur(window, which);

        // A position match is "in progress" while it still has unvisited
        // positions on the line it is on.
        let is_item = matches!(which, Which::Item(_));
        let mut pos_inprogress = true;
        let mut n: ColNr = ColNr::ZERO;
        while window.with_search_hl(which, |shl| shl.first_lnum) < lnum
            && (has_prog(window, which) || (is_item && pos_inprogress))
        {
            let first = window.with_search_hl(which, |shl| shl.first_lnum);
            next_search_hl(window, which, first, n);
            pos_inprogress = pos_in_progress(window, which);
            n = window.with_search_hl(which, |shl| {
                if shl.lnum != LineNr::ZERO {
                    shl.first_lnum = LineNr::new(
                        shl.lnum + shl.rm.endpos[0].lnum.get() - shl.rm.startpos[0].lnum,
                    );
                    shl.rm.endpos[0].col
                } else {
                    shl.first_lnum += 1;
                    ColNr::ZERO
                }
            });
        }
    }
}

/// Records whether the cursor is inside the `'hlsearch'` match, which is
/// what makes `CurSearch` apply to one match and not the others.
fn check_cur_search_hl(window: Win) {
    let cursor = window.w_cursor;
    screen_search_hl.with_mut(|shl| {
        let linecount = shl.rm.endpos[0].lnum - shl.rm.startpos[0].lnum;
        shl.has_cursor = cursor.lnum >= shl.lnum
            && cursor.lnum <= shl.lnum + linecount
            && (cursor.lnum > shl.lnum || cursor.col >= shl.rm.startpos[0].col)
            && (cursor.lnum < shl.lnum + linecount || cursor.col < shl.rm.endpos[0].col);
    });
}

/// A byte count as a column count.
#[inline(always)]
fn col_len(n: usize) -> ColNr {
    ColNr::try_from(n).unwrap_or(ColNr::MAX)
}

/// The length of the character at `col` of line `lnum`; 0 at the end of
/// the line.
#[inline(always)]
fn char_len_at(window: Win, lnum: LineNr, col: ColNr) -> ColNr {
    let at = usize::try_from(col).unwrap_or(usize::MAX);
    col_len(cluster_len_in_line(window.buffer(), lnum, at).unwrap_or(0))
}

/// The length of the character, with its combining characters, at byte
/// `at` of line `lnum` of `buffer`; `None` at or past the end of the line.
///
/// Out of line on purpose: the line read inlines the memline's cache
/// lookup, and inlined here it lands in `win_line`, whose drawing loop then
/// loses registers to a branch it almost never takes (scrbench +0.4 % Ir).
#[inline(never)]
fn cluster_len_in_line(buffer: Buf, lnum: LineNr, at: usize) -> Option<usize> {
    let mut lines = buffer.lines();
    let line = lines.line(lnum);
    (cstr::byte_at(line, at) != 0).then(|| c_cluster_len(&line[at..]))
}

/// How far an empty match at `col` of line `lnum` highlights: the
/// character there, or one cell past the end of the line.
#[inline(always)]
fn empty_match_width(window: Win, lnum: LineNr, col: ColNr) -> ColNr {
    char_len_at(window, lnum, col).max(ColNr::new(1))
}

/// Prepares every pattern for one window line, and answers whether any of
/// them highlights part of it.
///
/// A pattern already covering `mincol` — the leftmost column drawn, which is
/// not column 0 under `'nowrap'` or a `'smoothscroll'` skip — also writes
/// `search_attr`, so the drawing loop starts with the right attribute.
///
/// A multi-line regexp can make the line move: the caller reads it again.
pub(crate) fn prepare_search_hl_line(
    window: Win,
    lnum: LineNr,
    mincol: ColNr,
    search_attr: &mut c_int,
    search_attr_from_match: &mut bool,
) -> bool {
    let mut area_highlighting = false;
    let mut walk = ShlWalk::new(Order::SearchFirst);
    while let Some(which) = walk.next(window) {
        window.with_search_hl(which, |shl| {
            shl.startcol = ColNr::MAX;
            shl.endcol = ColNr::MAX;
            shl.attr_cur = 0;
            shl.is_addpos = false;
            shl.has_cursor = false;
        });
        reset_pos_cur(window, which);
        next_search_hl(window, which, lnum, mincol);

        let found = window.with_search_hl(which, |shl| {
            if shl.lnum == LineNr::ZERO || shl.lnum > lnum {
                return false;
            }
            // A match that started on an earlier line covers this one from
            // column 0; one that ends on a later line covers it to the end.
            shl.startcol = if shl.lnum == lnum {
                shl.rm.startpos[0].col
            } else {
                ColNr::ZERO
            };
            shl.endcol = if lnum
                == LineNr::new(shl.lnum + shl.rm.endpos[0].lnum.get() - shl.rm.startpos[0].lnum)
            {
                shl.rm.endpos[0].col
            } else {
                ColNr::MAX
            };
            true
        });
        if !found {
            continue;
        }

        // Before the columns are widened below.
        if which == Which::Search {
            check_cur_search_hl(window);
        }

        // An empty match still highlights one character.
        let (startcol, endcol) = window.with_search_hl(which, |shl| (shl.startcol, shl.endcol));
        if startcol == endcol {
            let width = empty_match_width(window, lnum, endcol);
            window.with_search_hl(which, |shl| shl.endcol += width.get());
        }
        if startcol < mincol {
            // Already highlighted at the left edge.
            let attr = window.with_search_hl(which, |shl| {
                shl.attr_cur = shl.attr;
                shl.attr
            });
            *search_attr = attr;
            *search_attr_from_match = which != Which::Search;
        }
        area_highlighting = true;
    }
    area_highlighting
}

/// What [`Win::update_search_hl`] reads of a pattern's state at each column.
#[derive(Debug)]
struct Peek {
    /// The pattern can still search, or (a position match) still has
    /// positions to visit on this line.
    live: bool,
    startcol: ColNr,
    endcol: ColNr,
    attr_cur: c_int,
    conceals: bool,
}

/// One borrow of `which`'s state for [`Win::update_search_hl`]: `reset` starts a
/// match's position walk over (each column does, once), and `in_progress`
/// is whether a position match still had positions when last searched.
#[inline(always)]
fn peek(window: Win, which: Which, reset: bool, in_progress: bool) -> Peek {
    let read = |shl: &MatchState, live: bool| Peek {
        live,
        startcol: shl.startcol,
        endcol: shl.endcol,
        attr_cur: shl.attr_cur,
        conceals: shl.conceals,
    };
    match which {
        Which::Search => screen_search_hl.with(|shl| read(shl, shl.rm.regprog.is_some())),
        Which::Item(i) => window.with_match_item(i, |item| {
            if reset {
                item.mit_pos_cur = 0;
            }
            read(&item.mit_hl, item.active() || in_progress)
        }),
    }
}

/// Whether the cell just past the end of the line should be highlighted.
///
/// True when a match started exactly there, or when it continues into the
/// next line — i.e. covers the line break itself. `matchaddpos()` matches
/// are excluded: they name real positions and never the break.
pub(crate) fn get_prevcol_hl_flag(window: Win, curcol: ColNr) -> bool {
    // Not really at that column when text to the left is being skipped.
    let skip = if window.w_onebuf_opt.wo_wrap != 0 {
        window.w_skipcol
    } else {
        window.w_leftcol
    };
    let prevcol = if skip > curcol { curcol + 1 } else { curcol };

    let covers = |m: &MatchState| {
        !m.is_addpos && (prevcol == m.startcol || (prevcol > m.startcol && m.endcol == ColNr::MAX))
    };

    screen_search_hl.with(covers) || window.w_matches.iter().any(|item| covers(&item.mit_hl))
}

/// The attribute for the character after the text, when a match starts
/// exactly at `col - 1`.
pub(crate) fn get_search_match_hl(window: Win, col: ColNr, char_attr: &mut c_int) {
    let mut walk = ShlWalk::new(Order::ByPriority);
    while let Some(which) = walk.next(window) {
        let (startcol, is_addpos, attr) =
            window.with_search_hl(which, |shl| (shl.startcol, shl.is_addpos, shl.attr));
        if col - 1 == startcol && (which == Which::Search || !is_addpos) {
            *char_attr = attr;
        }
    }
}
