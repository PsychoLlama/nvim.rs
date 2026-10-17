//! Running a pattern over the buffer.
//!
//! [`searchit`] is the one searcher underneath `/`, `?`, `n`, `N`, `*`,
//! `gd`, `:substitute`'s address form and the tag jumps: it walks lines
//! from a starting position in one direction, wrapping at the end of the
//! buffer when `'wrapscan'` is set. [`search_for_exact_line`] is the
//! unrelated plain-text line scanner insert-mode line completion uses.

#![forbid(unsafe_code)]

use super::*;
use crate::cstr;
use crate::mbyte::{strcmp_ic_in, strnicmp_in};
use crate::message_fmt::{emsg_text, msg_bytes};
use crate::option::cpo_has;
use crate::pos::MAXCOL;
use crate::regexp::RE_SEARCH;
use crate::search::{
    SEARCH_COL, SEARCH_END, SEARCH_HIS, SEARCH_KEEP, SEARCH_MSG, SEARCH_NOOF, SEARCH_PEEK,
    SEARCH_START,
};
use crate::semsg;
use crate::tr_plural;
use crate::types::{CpoFlag, FAIL, Failed, NUL, ShmFlag};
use crate::winlayer::{Buf, Win};
use core::ffi::CStr;
use core::ffi::c_int;

// The `SEARCH_*` option flags, retyped for this module: c2rust left the
// family as `c_uint` and `options` is a `c_int`.

/// One `vim_regexec_multi` result: where the match starts, where it ends
/// (both relative to the line the search was run from) and the number of
/// the first sub-pattern that took part.
#[derive(Clone, Copy)]
struct Found {
    start: LPos,
    end: LPos,
    submatch: c_int,
}

/// Where the search started from, and how close to it a match may sit.
#[derive(Clone, Copy)]
struct StartPos {
    pos: Pos,
    /// Zero when a match at the start position itself counts; otherwise
    /// the length of the character there, so that a match has to be at
    /// least one character away.
    extra_col: c_int,
}

/// Everything a [`searchit`] line walk needs that does not change while it
/// runs.
struct Searcher<'a> {
    /// The window the search runs in, `None` for a buffer with no window.
    win: Option<Win>,
    buf: Buf,
    /// The compiled pattern. `vim_regexec_multi` may clear `regprog`,
    /// which is how a pattern that turned out to be too expensive stops
    /// the whole search.
    regmatch: RegMMatch,
    /// Timeout limit, if there is one.
    tm: Option<&'a ProfTime>,
    /// Set when the limit was passed.
    timed_out: bool,
    options: c_int,
    /// `'cpo'` contains `c`: a repeated search continues from the end of
    /// the previous match rather than one character past its start.
    from_match_end: bool,
    /// `called_emsg` on entry — anything above it means an error was
    /// reported and the search must stop.
    called_emsg_before: c_int,
}

impl Searcher<'_> {
    /// Whether one of the `SEARCH_*` option bits is set.
    #[inline(always)]
    fn opt(&self, flag: c_int) -> bool {
        self.options & flag != 0
    }

    /// Run the pattern against line `lnum` from column `col`.
    #[inline(always)]
    fn exec(&mut self, lnum: LineNr, col: ColNr) -> c_int {
        let mut timed_out = false;
        let lines = vim_regexec_multi(
            &mut self.regmatch,
            self.win,
            self.buf,
            lnum,
            col,
            self.tm,
            Some(&mut timed_out),
        );
        self.timed_out |= timed_out;
        lines
    }

    /// The match `exec` just reported.
    #[inline(always)]
    fn found(&mut self) -> Found {
        Found {
            start: self.regmatch.startpos[0],
            end: self.regmatch.endpos[0],
            submatch: first_submatch(&self.regmatch),
        }
    }

    /// The buffer's line cache.
    ///
    /// Read afresh at every use rather than held: `exec` hands the line to
    /// `vim_regexec_multi`, which reads other lines of this same buffer for
    /// a multi-line pattern, so a slice taken before it is stale after it.
    /// That is the pointer upstream re-fetched by hand ("a multi-line search
    /// may have invalidated the pointer"), with the re-fetch made
    /// unnecessary instead of remembered.
    #[inline(always)]
    fn lines(&self) -> Lines {
        self.buf.lines()
    }

    /// The byte at column `col` of line `lnum`.
    ///
    /// NUL at the end of the line, and for a line past the end of the
    /// buffer -- which `\n\zs` can name, and which upstream answered with a
    /// static empty string.
    #[inline(always)]
    fn byte_at(&self, lnum: LineNr, col: ColNr) -> u8 {
        if lnum > self.buf.b_ml.ml_line_count {
            return NUL as u8;
        }
        cstr::byte_at(self.lines().line(lnum), usize::try_from(col).unwrap_or(0))
    }

    /// Whether an error was reported or the timeout was passed — either
    /// way the search stops where it is.
    #[inline(always)]
    fn aborted(&self) -> bool {
        called_emsg.get() > self.called_emsg_before || self.timed_out
    }

    /// Whether the timeout has been passed.
    #[inline(always)]
    fn out_of_time(&self) -> bool {
        self.tm.is_some_and(|tm| profile_passed_limit(*tm))
    }

    /// Advance `col` over the character at it, if there is one.
    #[inline(always)]
    fn step_over(&self, lnum: LineNr, col: ColNr) -> ColNr {
        if lnum > self.buf.b_ml.ml_line_count {
            return col;
        }
        let at = usize::try_from(col).unwrap_or(0);
        let mut lines = self.lines();
        let line = lines.line(lnum);
        if at < line.len() {
            col + ColNr::try_from(cluster_len(&line[at..])).unwrap_or(0)
        } else {
            col
        }
    }

    /// Forward search in the line the search started in: the match has to
    /// lie after the start position. When it does not, look again — from
    /// the end of the match with `'cpo'`'s `c` (vi compatible), otherwise
    /// from one character on — until a match is past the start or the line
    /// runs out.
    ///
    /// Answers false when this line holds nothing usable.
    ///
    /// The loop only runs while the match starts in `lnum` itself, which is
    /// why every read below names that one line.
    fn skip_to_start_pos(
        &mut self,
        lnum: LineNr,
        found: &mut Found,
        nmatched: &mut c_int,
        start: StartPos,
        first_match: bool,
    ) -> bool {
        while found.start.lnum == 0
            && if self.opt(SEARCH_END) && first_match {
                // A match landing on the NUL puts the cursor one back
                // afterwards; compare against that, or `/$` sticks at
                // the end of the line.
                *nmatched == 1 && found.end.col - 1 < start.pos.col + start.extra_col
            } else {
                let on_nul = c_int::from(self.byte_at(lnum, found.start.col) == 0);
                found.start.col - on_nul < start.pos.col + start.extra_col
            }
        {
            let matchcol = if self.from_match_end {
                if *nmatched > 1 {
                    // The end is in the next line, so there is no
                    // match in this one.
                    return false;
                }
                // For an empty match, advance one character.
                if found.end.col == found.start.col {
                    self.step_over(lnum, found.end.col)
                } else {
                    found.end.col
                }
            } else {
                // `rmm_matchcol` is the actual start of the match,
                // ignoring `\zs`.
                self.step_over(lnum, self.regmatch.rmm_matchcol)
            };
            if matchcol == 0 && self.opt(SEARCH_START) {
                return true;
            }
            if self.byte_at(lnum, matchcol) == NUL as u8 {
                return false;
            }
            *nmatched = self.exec(lnum, matchcol);
            if *nmatched == 0 {
                return false;
            }
            if self.regmatch.regprog.is_none() {
                return true;
            }
            *found = self.found();
            // The loop only works while the match starts in this line:
            // above that, `line` would not be a buffer line.
            if found.start.lnum != 0 {
                return true;
            }
        }
        true
    }

    /// Backward search: take the last match in the line, or — in the line
    /// the search started in — the last one before the start position.
    ///
    /// Answers false when every match in the line is after the cursor.
    fn last_match_before(
        &mut self,
        lnum: LineNr,
        found: &mut Found,
        nmatched: &mut c_int,
        start: StartPos,
        wrapped: bool,
    ) -> bool {
        let mut match_ok = false;
        // Remember a position before the start position; it is the
        // answer if it turns out to be the last match in the line.
        // After wrapping around, any position is acceptable.
        while wrapped || self.before_start_pos(lnum, start) {
            match_ok = true;
            *found = self.found();

            // A valid match; now see whether another one follows it.
            // Every read below is of the line the match starts in.
            let at = lnum + found.start.lnum;
            let matchcol = if self.from_match_end {
                if *nmatched > 1 {
                    break;
                }
                // For an empty match, advance one character.
                if found.end.col == found.start.col {
                    self.step_over(at, found.end.col)
                } else {
                    found.end.col
                }
            } else {
                // Stop when the match is in a following line.
                if found.start.lnum > 0 {
                    break;
                }
                self.step_over(at, found.start.col)
            };
            if self.byte_at(at, matchcol) == NUL as u8 || {
                *nmatched = self.exec(at, matchcol);
                *nmatched == 0
            } {
                // A search that timed out did find a match, but it may
                // be the wrong one — that is not good enough.
                if self.out_of_time() {
                    match_ok = false;
                }
                break;
            }
            if self.regmatch.regprog.is_none() {
                break;
            }
        }
        match_ok
    }

    /// Whether the match `exec` last reported begins (or, with
    /// `SEARCH_END`, ends) before the start position.
    #[inline(always)]
    fn before_start_pos(&self, lnum: LineNr, start: StartPos) -> bool {
        if self.opt(SEARCH_END) {
            let end = self.regmatch.endpos[0];
            lnum + end.lnum < start.pos.lnum
                || (lnum + end.lnum == start.pos.lnum
                    && end.col - 1 < start.pos.col + start.extra_col)
        } else {
            let begin = self.regmatch.startpos[0];
            lnum + begin.lnum < start.pos.lnum
                || (lnum + begin.lnum == start.pos.lnum
                    && begin.col < start.pos.col + start.extra_col)
        }
    }

    /// Write a match out. With `SEARCH_END` the position is the last
    /// character of the match and `end_pos` gets its start; otherwise the
    /// other way round. An empty match has no last character, so it is
    /// reported as its start either way.
    fn record(&self, lnum: LineNr, found: Found, pos: &mut Pos, end_pos: Option<&mut Pos>) {
        let empty = found.start.lnum == found.end.lnum && found.start.col == found.end.col;
        let (at, other) = if self.opt(SEARCH_END) && !self.opt(SEARCH_NOOF) && !empty {
            pos.lnum = lnum + found.end.lnum;
            pos.col = found.end.col;
            if found.end.col == 0 {
                // A match in the first column ends on the NUL of the
                // line before.
                if pos.lnum > 1 {
                    pos.lnum -= 1;
                    pos.col = self.lines().line_len(pos.lnum);
                }
            } else {
                pos.col -= 1;
                if pos.lnum <= self.buf.b_ml.ml_line_count {
                    // Step back off a trail byte onto the character's own
                    // first byte.
                    let col = usize::try_from(pos.col).unwrap_or(0);
                    let back = head_off(self.lines().line(pos.lnum), col);
                    pos.col -= back as ColNr;
                }
            }
            (pos, found.start)
        } else {
            pos.lnum = lnum + found.start.lnum;
            pos.col = found.start.col;
            (pos, found.end)
        };
        at.coladd = 0;
        if let Some(end_pos) = end_pos {
            end_pos.lnum = lnum + other.lnum;
            end_pos.col = other.col;
            end_pos.coladd = 0;
        }
    }
}

/// [`searchit`] once in the current window for the last pattern `pat_use`
/// names, from `pos`, answering the match in `pos`.
pub(crate) fn search_last_pattern(
    pos: &mut Pos,
    dir: Direction,
    options: c_int,
    pat_use: c_int,
) -> c_int {
    let (win, buf) = (Some(Win::current()), Buf::current());
    searchit(win, buf, pos, None, dir, b"", 1, options, pat_use, None)
}

/// Search for the `count`th occurrence of `pat` in direction `dir`,
/// starting at `pos` and answering the position found in `pos`.
///
/// - `options & SEARCH_MSG == 0`: no messages at all; `== SEARCH_MSG`:
///   every message, including "not found".
/// - `options & SEARCH_HIS`: put the pattern in the search history.
/// - `options & SEARCH_END`: answer the end of the match.
/// - `options & SEARCH_START`: accept a match at `pos` itself.
/// - `options & SEARCH_KEEP`: do not remember the pattern.
/// - `options & SEARCH_PEEK`: give up when a character is typed.
/// - `options & SEARCH_COL`: start at `pos->col` rather than at zero.
///
/// `win` is the window to search in, `None` for a buffer without one;
/// `end_pos` gets the end of the match; `pat_use` is which remembered
/// pattern an empty `pat` means; `extra_arg` adds a line to stop after and
/// a deadline, and reports a timeout and a wrap.
///
/// Answers `FAIL` (zero) for failure, otherwise the index of the first
/// matching sub-pattern plus one; one if there was none.
#[allow(clippy::too_many_arguments)]
pub fn searchit(
    win: Option<Win>,
    buffer: Buf,
    pos: &mut Pos,
    mut end_pos: Option<&mut Pos>,
    dir: Direction,
    pat: &[u8],
    mut count: c_int,
    options: c_int,
    pat_use: c_int,
    mut extra_arg: Option<&mut SearchItArg>,
) -> c_int {
    let mut regmatch = RegMMatch::default();
    let keep = options & (SEARCH_HIS | SEARCH_KEEP);
    if search_regcomp(pat, RE_SEARCH, pat_use, keep, &mut regmatch).is_err() {
        if options & SEARCH_MSG != 0 && !rc_did_emsg.get() {
            let pattern = get_search_pat();
            let arg0 = msg_bytes(&pattern);
            semsg!("E383: Invalid search string: {arg0}");
        }
        return FAIL;
    }

    // Stop after this line number, when it is not zero.
    let mut stop_lnum: LineNr = 0;
    let mut s = Searcher {
        win,
        buf: buffer,
        regmatch,
        tm: None,
        timed_out: false,
        options,
        from_match_end: cpo_has(CpoFlag::SEARCH),
        called_emsg_before: called_emsg.get(),
    };
    if let Some(extra) = &extra_arg {
        stop_lnum = extra.sa_stop_lnum;
        s.tm = extra.sa_tm;
        s.timed_out = extra.sa_timed_out;
    }

    let mut found;
    let mut submatch = 0;
    let mut first_match = true;
    let mut break_loop = false;
    // The line the walk stopped on; the "hit TOP" message reads it
    // after every loop has been left.
    let mut lnum: LineNr;

    loop {
        // When a match at the start position is not acceptable,
        // `extra_col` is non-zero. Not at MAXCOL though, where
        // MAXCOL + 1 is zero.
        let start_char_len = if pos.col == MAXCOL as c_int {
            0
        } else if pos.lnum >= 1 && pos.lnum <= buffer.b_ml.ml_line_count && pos.col < MAXCOL - 2 {
            // Watch out for "col" being MAXCOL - 2, used in a closed fold.
            let (lnum, col) = (pos.lnum, usize::try_from(pos.col).unwrap_or(0));
            let mut lines = buffer.lines();
            let line = lines.line(lnum);
            if line.len() <= col {
                1
            } else {
                cluster_len(&line[col..]) as c_int
            }
        } else {
            1
        };
        let accept_at_start = s.opt(SEARCH_START);
        let start = StartPos {
            // Remember the start position, for detecting "no match".
            pos: *pos,
            extra_col: if dir == FORWARD {
                if accept_at_start { 0 } else { start_char_len }
            } else if accept_at_start {
                start_char_len
            } else {
                0
            },
        };

        found = 0;
        let mut at_first_line = true;
        if pos.lnum == 0 {
            // Correct lnum for when starting in line 0.
            pos.lnum = 1;
            pos.col = 0;
            at_first_line = false;
        }

        // Start in the current line, unless searching backwards from
        // column 0 without accepting a match there — skipping back a
        // line is then free.
        if dir == BACKWARD && start.pos.col == 0 && !accept_at_start {
            lnum = pos.lnum - 1;
            at_first_line = false;
        } else {
            lnum = pos.lnum;
        }

        // Loop twice if 'wrapscan' is set.
        for wrapped in [false, true] {
            'lines: while lnum > 0 && lnum <= buffer.b_ml.ml_line_count {
                // Stop after checking "stop_lnum", if it is set.
                if stop_lnum != 0
                    && if dir == FORWARD {
                        lnum > stop_lnum
                    } else {
                        lnum < stop_lnum
                    }
                {
                    break 'lines;
                }
                // Stop after passing the time limit.
                if s.out_of_time() {
                    break 'lines;
                }

                // Look for a match somewhere in line "lnum".
                let col = if at_first_line && s.opt(SEARCH_COL) {
                    pos.col
                } else {
                    0
                };
                let mut nmatched = s.exec(lnum, col);
                // vim_regexec_multi() may clear "regprog".
                if s.regmatch.regprog.is_none() {
                    break 'lines;
                }
                // Abort searching on an error (e.g., out of stack).
                if s.aborted() {
                    break 'lines;
                }

                'next_line: {
                    if nmatched <= 0 {
                        line_breakcheck(); // stop if ctrl-C typed
                        if got_int.get() {
                            break 'lines;
                        }
                        // Cancel the search if a character was typed,
                        // for 'incsearch'. Checking too often would
                        // slow searching down too much.
                        if s.opt(SEARCH_PEEK) && ((lnum - pos.lnum) & 0x3f) == 0 && char_avail() {
                            break_loop = true;
                            break 'lines;
                        }
                        if wrapped && lnum == start.pos.lnum {
                            // Second time round: stop where we started.
                            break 'lines;
                        }
                        break 'next_line;
                    }

                    // The match may be in another line, with `\zs`.
                    let mut m = s.found();

                    if dir == FORWARD
                        && at_first_line
                        && !s.skip_to_start_pos(lnum, &mut m, &mut nmatched, start, first_match)
                    {
                        break 'next_line;
                    }
                    if dir == BACKWARD
                        && !s.last_match_before(lnum, &mut m, &mut nmatched, start, wrapped)
                    {
                        // There is only a match after the cursor.
                        break 'next_line;
                    }

                    s.record(lnum, m, pos, end_pos.as_deref_mut());
                    found = 1;
                    first_match = false;
                    submatch = m.submatch;

                    // Variables used for 'incsearch' highlighting.
                    search_match_lines.set(m.end.lnum - m.start.lnum);
                    search_match_endcol.set(m.end.col);
                    break 'lines;
                }
                lnum += dir;
                at_first_line = false;
            }
            at_first_line = false;

            // vim_regexec_multi() may clear "regprog".
            if s.regmatch.regprog.is_none() {
                break;
            }

            // Stop when 'wrapscan' is off, "stop_lnum" was given,
            // after an interrupt, after a match, and after looping
            // twice.
            if !p_ws()
                || stop_lnum != 0
                || got_int.get()
                || s.aborted()
                || break_loop
                || found != 0
                || wrapped
            {
                break;
            }

            // Continue at the other end of the file. Say so unless
            // 'shortmess' asks us not to, or the search stat is going
            // to be shown anyway (so SEARCH_COUNT must be absent).
            // The message is remembered in keep_msg for a redraw.
            lnum = if dir == BACKWARD {
                buffer.b_ml.ml_line_count
            } else {
                1
            };
            if !shortmess(ShmFlag::SEARCH) && shortmess(ShmFlag::SEARCHCOUNT) && s.opt(SEARCH_MSG) {
                let msg = if dir == BACKWARD {
                    top_bot_msg
                } else {
                    bot_top_msg
                };
                give_warning(gettext(msg), true, false);
            }
            if let Some(extra) = extra_arg.as_deref_mut() {
                extra.sa_wrapped = true;
            }
        }

        if got_int.get() || s.aborted() || break_loop {
            break;
        }
        // Stop after "count" matches, or as soon as one is missed.
        count -= 1;
        if count <= 0 || found == 0 {
            break;
        }
    }

    s.regmatch.regprog = None;
    if s.timed_out
        && let Some(extra) = extra_arg
    {
        extra.sa_timed_out = true;
    }

    if found == 0 {
        if got_int.get() {
            emsg(gettext(e_interr));
        } else if options & SEARCH_MSG == SEARCH_MSG {
            let msg = if p_ws() {
                gettext(e_patnotf2)
            } else if lnum == 0 {
                gettext(c"E384: Search hit TOP without match for: %s")
            } else {
                gettext(c"E385: Search hit BOTTOM without match for: %s")
            };
            let pattern = get_search_pat();
            let pat = msg_bytes(&pattern);
            emsg_text(tr_plural!(msg, pat));
        }
        return FAIL;
    }

    // A pattern like "\n\zs" may go past the last line.
    if pos.lnum > buffer.b_ml.ml_line_count {
        pos.lnum = buffer.b_ml.ml_line_count;
        pos.col = buffer.lines().line_len(pos.lnum);
        if pos.col > 0 {
            pos.col -= 1;
        }
    }

    submatch + 1
}

/// [`searchit`] once, for `pat` in `buffer` from `pos`, with no window and
/// no extra arguments. Answers whether it found one, with the match at `pos`
/// and its end at `end_pos`.
pub(crate) fn searchit_buf(
    buffer: Buf,
    pos: &mut Pos,
    end_pos: &mut Pos,
    dir: Direction,
    pat: &CStr,
    options: c_int,
    pat_use: c_int,
) -> bool {
    let pat = pat.to_bytes();
    searchit(
        None,
        buffer,
        pos,
        Some(end_pos),
        dir,
        pat,
        1,
        options,
        pat_use,
        None,
    ) != FAIL
}

/// [`searchit`] once in the current window, for the search builtins: for
/// `pat` from `pos`, which answers the match, giving up past line
/// `stop_lnum` (when it is not zero) or at the deadline `deadline`.
///
/// Answers [`searchit`]'s number: [`FAIL`], or the first matching
/// sub-pattern plus one.
pub(crate) fn search_in_current(
    pos: &mut Pos,
    dir: Direction,
    pat: &[u8],
    options: c_int,
    stop_lnum: LineNr,
    deadline: &ProfTime,
) -> c_int {
    let mut extra = SearchItArg {
        sa_stop_lnum: stop_lnum,
        sa_tm: Some(deadline),
        ..SearchItArg::default()
    };
    let (win, buf) = (Some(Win::current()), Buf::current());
    searchit(
        win,
        buf,
        pos,
        None,
        dir,
        pat,
        1,
        options,
        RE_SEARCH,
        Some(&mut extra),
    )
}

/// The number of the first sub-pattern that matched, or zero if none of
/// them did.
#[inline(always)]
fn first_submatch(rp: &RegMMatch) -> c_int {
    (1..10)
        .find(|&i| rp.startpos[i].lnum >= 0)
        .map_or(0, |i| i as c_int)
}

/// Search for a line starting with `pat`, ignoring leading white space,
/// from `pos` in direction `dir`; `pos` is left on the line found.
///
/// Blank lines match only while insert-mode completion is adding lines.
/// With `'ignorecase'` the pattern must be in lower case. The pattern stops
/// at a NUL.
///
/// Answers `Err` if no line was found.
pub fn search_for_exact_line(
    buffer: Buf,
    pos: &mut Pos,
    dir: Direction,
    pat: &[u8],
) -> Result<(), Failed> {
    let pat = cstr::until_nul(pat);
    let mut start: LineNr = 0;
    let compl_len = ins_compl_len();
    if buffer.b_ml.ml_line_count == 0 {
        return Err(Failed);
    }
    loop {
        pos.lnum += dir;
        if pos.lnum < 1 {
            if !p_ws() {
                pos.lnum = 1;
                break;
            }
            pos.lnum = buffer.b_ml.ml_line_count;
            if !shortmess(ShmFlag::SEARCH) {
                give_warning(gettext(top_bot_msg), true, false);
            }
        } else if pos.lnum > buffer.b_ml.ml_line_count {
            pos.lnum = 1;
            if !p_ws() {
                break;
            }
            if !shortmess(ShmFlag::SEARCH) {
                give_warning(gettext(bot_top_msg), true, false);
            }
        }
        if pos.lnum == start {
            break;
        }
        if start == 0 {
            start = pos.lnum;
        }

        let mut lines = buffer.lines();
        let line = lines.line(pos.lnum);
        let at = skip::white(line);
        pos.col = at as ColNr;
        let text = cstr::until_nul(&line[at..]);

        if compl_status_adding() && !compl_status_sol() {
            // When adding lines the matching line may be empty; it is
            // not ignored, because it is the *next* line that is
            // wanted. -- Acevedo
            if strcmp_ic_in(p_ic(), text, pat) == 0 {
                return Ok(());
            }
        } else if at < line.len() {
            // Expanding lines or words; ignore empty lines.
            debug_assert!(compl_len >= 0);
            let n = usize::try_from(compl_len).unwrap_or(0);
            let (text, pat) = (&text[..n.min(text.len())], &pat[..n.min(pat.len())]);
            let same = if p_ic() {
                strnicmp_in(text, pat) == 0
            } else {
                text == pat
            };
            if same {
                return Ok(());
            }
        }
    }
    Err(Failed)
}
