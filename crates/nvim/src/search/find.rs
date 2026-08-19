//! Running a pattern over the buffer.
//!
//! [`searchit`] is the one searcher underneath `/`, `?`, `n`, `N`, `*`,
//! `gd`, `:substitute`'s address form and the tag jumps: it walks lines
//! from a starting position in one direction, wrapping at the end of the
//! buffer when `'wrapscan'` is set. [`search_for_exact_line`] is the
//! unrelated plain-text line scanner insert-mode line completion uses.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::pos::MAXCOL;
use crate::regexp::RE_SEARCH;
use crate::search::{
    SEARCH_COL, SEARCH_END, SEARCH_HIS, SEARCH_KEEP, SEARCH_MSG, SEARCH_NOOF, SEARCH_PEEK,
    SEARCH_START,
};
use crate::semsg_c;
use crate::types::{FAIL, NUL, OK};
use core::ffi::{c_char, c_int};
use core::ptr;

// The `SEARCH_*` option flags, retyped for this module: c2rust left the
// family as `c_uint` and `options` is a `c_int`.

/// One `vim_regexec_multi` result: where the match starts, where it ends
/// (both relative to the line the search was run from) and the number of
/// the first sub-pattern that took part.
#[derive(Clone, Copy)]
struct Found {
    start: lpos_T,
    end: lpos_T,
    submatch: c_int,
}

/// Where the search started from, and how close to it a match may sit.
#[derive(Clone, Copy)]
struct StartPos {
    pos: pos_T,
    /// Zero when a match at the start position itself counts; otherwise
    /// the length of the character there, so that a match has to be at
    /// least one character away.
    extra_col: c_int,
}

/// Everything a [`searchit`] line walk needs that does not change while it
/// runs.
struct Searcher {
    win: *mut win_T,
    buf: *mut buf_T,
    /// The compiled pattern. `vim_regexec_multi` may clear `regprog`,
    /// which is how a pattern that turned out to be too expensive stops
    /// the whole search.
    regmatch: regmmatch_T,
    /// Timeout limit, or null for none.
    tm: *mut proftime_T,
    /// Set when the limit was passed, or null.
    timed_out: *mut c_int,
    options: c_int,
    /// `'cpo'` contains `c`: a repeated search continues from the end of
    /// the previous match rather than one character past its start.
    from_match_end: bool,
    /// `called_emsg` on entry — anything above it means an error was
    /// reported and the search must stop.
    called_emsg_before: c_int,
}

impl Searcher {
    /// Whether one of the `SEARCH_*` option bits is set.
    #[inline(always)]
    fn opt(&self, flag: c_int) -> bool {
        self.options & flag != 0
    }

    /// Run the pattern against line `lnum` from column `col`.
    ///
    /// # Safety
    /// `lnum` must be a line of `self.buf`.
    #[inline(always)]
    unsafe fn exec(&mut self, lnum: linenr_T, col: colnr_T) -> c_int {
        unsafe {
            vim_regexec_multi(
                &raw mut self.regmatch,
                self.win,
                self.buf,
                lnum,
                col,
                self.tm,
                self.timed_out,
            )
        }
    }

    /// The match `exec` just reported.
    #[inline(always)]
    fn found(&mut self) -> Found {
        Found {
            start: self.regmatch.startpos[0],
            end: self.regmatch.endpos[0],
            // SAFETY: a pointer to our own field.
            submatch: unsafe { first_submatch(&raw mut self.regmatch) },
        }
    }

    /// Text of line `lnum`.
    ///
    /// # Safety
    /// `lnum` must be a line of `self.buf`.
    #[inline(always)]
    unsafe fn line(&self, lnum: linenr_T) -> *mut c_char {
        unsafe { ml_get_buf(self.buf, lnum) }
    }

    /// Whether an error was reported or the timeout was passed — either
    /// way the search stops where it is.
    #[inline(always)]
    fn aborted(&self) -> bool {
        // SAFETY: `timed_out` is null or points into the caller's
        // `searchit_arg_T`.
        called_emsg.get() > self.called_emsg_before
            || unsafe { !self.timed_out.is_null() && *self.timed_out != 0 }
    }

    /// Whether the timeout has been passed.
    #[inline(always)]
    fn out_of_time(&self) -> bool {
        // SAFETY: `tm` is null or points into the caller's `searchit_arg_T`.
        unsafe { !self.tm.is_null() && profile_passed_limit(*self.tm) }
    }

    /// Advance `col` over the character at it, if there is one.
    ///
    /// # Safety
    /// `line` must be NUL-terminated and `col` within it.
    #[inline(always)]
    unsafe fn step_over(&self, line: *mut c_char, col: colnr_T) -> colnr_T {
        unsafe {
            if *line.offset(col as isize) as c_int != NUL {
                col + utfc_ptr2len(line.offset(col as isize))
            } else {
                col
            }
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
    /// # Safety
    /// `lnum` must be a line of `self.buf` and `line` its text.
    unsafe fn skip_to_start_pos(
        &mut self,
        lnum: linenr_T,
        mut line: *mut c_char,
        found: &mut Found,
        nmatched: &mut c_int,
        start: StartPos,
        first_match: bool,
    ) -> bool {
        unsafe {
            while found.start.lnum == 0
                && if self.opt(SEARCH_END) && first_match {
                    // A match landing on the NUL puts the cursor one back
                    // afterwards; compare against that, or `/$` sticks at
                    // the end of the line.
                    *nmatched == 1 && found.end.col - 1 < start.pos.col + start.extra_col
                } else {
                    let on_nul = c_int::from(*line.offset(found.start.col as isize) == 0);
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
                        self.step_over(line, found.end.col)
                    } else {
                        found.end.col
                    }
                } else {
                    // `rmm_matchcol` is the actual start of the match,
                    // ignoring `\zs`.
                    self.step_over(line, self.regmatch.rmm_matchcol)
                };
                if matchcol == 0 && self.opt(SEARCH_START) {
                    return true;
                }
                if *line.offset(matchcol as isize) as c_int == NUL {
                    return false;
                }
                *nmatched = self.exec(lnum, matchcol);
                if *nmatched == 0 {
                    return false;
                }
                if self.regmatch.regprog.is_null() {
                    return true;
                }
                *found = self.found();
                // The loop only works while the match starts in this line:
                // above that, `line` would not be a buffer line.
                if found.start.lnum != 0 {
                    return true;
                }
                // A multi-line search may have invalidated the pointer.
                line = self.line(lnum);
            }
            true
        }
    }

    /// Backward search: take the last match in the line, or — in the line
    /// the search started in — the last one before the start position.
    ///
    /// Answers false when every match in the line is after the cursor.
    ///
    /// # Safety
    /// `lnum` must be a line of `self.buf` and `line` its text.
    unsafe fn last_match_before(
        &mut self,
        lnum: linenr_T,
        mut line: *mut c_char,
        found: &mut Found,
        nmatched: &mut c_int,
        start: StartPos,
        wrapped: bool,
    ) -> bool {
        unsafe {
            let mut match_ok = false;
            // Remember a position before the start position; it is the
            // answer if it turns out to be the last match in the line.
            // After wrapping around, any position is acceptable.
            while wrapped || self.before_start_pos(lnum, start) {
                match_ok = true;
                *found = self.found();

                // A valid match; now see whether another one follows it.
                let matchcol = if self.from_match_end {
                    if *nmatched > 1 {
                        break;
                    }
                    // For an empty match, advance one character.
                    if found.end.col == found.start.col {
                        self.step_over(line, found.end.col)
                    } else {
                        found.end.col
                    }
                } else {
                    // Stop when the match is in a following line.
                    if found.start.lnum > 0 {
                        break;
                    }
                    self.step_over(line, found.start.col)
                };
                if *line.offset(matchcol as isize) as c_int == NUL || {
                    *nmatched = self.exec(lnum + found.start.lnum, matchcol);
                    *nmatched == 0
                } {
                    // A search that timed out did find a match, but it may
                    // be the wrong one — that is not good enough.
                    if self.out_of_time() {
                        match_ok = false;
                    }
                    break;
                }
                if self.regmatch.regprog.is_null() {
                    break;
                }
                // A multi-line search may have invalidated the pointer.
                line = self.line(lnum + found.start.lnum);
            }
            match_ok
        }
    }

    /// Whether the match `exec` last reported begins (or, with
    /// `SEARCH_END`, ends) before the start position.
    #[inline(always)]
    fn before_start_pos(&self, lnum: linenr_T, start: StartPos) -> bool {
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
    ///
    /// # Safety
    /// `pos` must be writable and `end_pos` writable or null.
    unsafe fn record(&self, lnum: linenr_T, found: Found, pos: *mut pos_T, end_pos: *mut pos_T) {
        unsafe {
            let empty = found.start.lnum == found.end.lnum && found.start.col == found.end.col;
            if self.opt(SEARCH_END) && !self.opt(SEARCH_NOOF) && !empty {
                (*pos).lnum = lnum + found.end.lnum;
                (*pos).col = found.end.col;
                if found.end.col == 0 {
                    // A match in the first column ends on the NUL of the
                    // line before.
                    if (*pos).lnum > 1 {
                        (*pos).lnum -= 1;
                        (*pos).col = ml_get_buf_len(self.buf, (*pos).lnum);
                    }
                } else {
                    (*pos).col -= 1;
                    if (*pos).lnum <= (*self.buf).b_ml.ml_line_count {
                        let line = self.line((*pos).lnum);
                        (*pos).col -= utf_head_off(line, line.offset((*pos).col as isize));
                    }
                }
                if !end_pos.is_null() {
                    (*end_pos).lnum = lnum + found.start.lnum;
                    (*end_pos).col = found.start.col;
                }
            } else {
                (*pos).lnum = lnum + found.start.lnum;
                (*pos).col = found.start.col;
                if !end_pos.is_null() {
                    (*end_pos).lnum = lnum + found.end.lnum;
                    (*end_pos).col = found.end.col;
                }
            }
            (*pos).coladd = 0;
            if !end_pos.is_null() {
                (*end_pos).coladd = 0;
            }
        }
    }
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
/// # Safety
/// `pos` must be writable, `end_pos` writable or null, `pat` a readable
/// string of `patlen` bytes or null, and `extra_arg` writable or null.
///
/// @param win        window to search in; can be NULL for a buffer without a window!
/// @param end_pos    set to end of the match, unless NULL
/// @param pat_use    which pattern to use when `pat` is empty
/// @param extra_arg  optional extra arguments, can be NULL
///
/// @return  FAIL (zero) for failure, otherwise the index of the first
///          matching sub-pattern plus one; one if there was none.
pub unsafe fn searchit(
    win: *mut win_T,
    buf: *mut buf_T,
    pos: *mut pos_T,
    end_pos: *mut pos_T,
    dir: Direction,
    pat: *mut c_char,
    patlen: size_t,
    mut count: c_int,
    options: c_int,
    pat_use: c_int,
    extra_arg: *mut searchit_arg_T,
) -> c_int {
    unsafe {
        let mut regmatch = regmmatch_T::default();
        if search_regcomp(
            pat,
            patlen,
            ptr::null_mut(),
            RE_SEARCH,
            pat_use,
            options & (SEARCH_HIS | SEARCH_KEEP),
            &raw mut regmatch,
        ) == FAIL
        {
            if options & SEARCH_MSG != 0 && !rc_did_emsg.get() {
                semsg_c!(
                    gettext(c"E383: Invalid search string: %s".as_ptr()),
                    get_search_pat(),
                );
            }
            return FAIL;
        }

        // Stop after this line number, when it is not zero.
        let mut stop_lnum: linenr_T = 0;
        let mut s = Searcher {
            win,
            buf,
            regmatch,
            tm: ptr::null_mut(),
            timed_out: ptr::null_mut(),
            options,
            from_match_end: !vim_strchr(p_cpo.get(), CPO_SEARCH).is_null(),
            called_emsg_before: called_emsg.get(),
        };
        if !extra_arg.is_null() {
            stop_lnum = (*extra_arg).sa_stop_lnum;
            s.tm = (*extra_arg).sa_tm;
            s.timed_out = &raw mut (*extra_arg).sa_timed_out;
        }

        let mut found = 0;
        let mut submatch = 0;
        let mut first_match = true;
        let mut break_loop = false;
        // The line the walk stopped on; the "hit TOP" message reads it
        // after every loop has been left.
        let mut lnum: linenr_T = 0;

        loop {
            // When a match at the start position is not acceptable,
            // `extra_col` is non-zero. Not at MAXCOL though, where
            // MAXCOL + 1 is zero.
            let start_char_len = if (*pos).col == MAXCOL as c_int {
                0
            } else if (*pos).lnum >= 1
                && (*pos).lnum <= (*buf).b_ml.ml_line_count
                && (*pos).col < MAXCOL as c_int - 2
            {
                // Watch out for "col" being MAXCOL - 2, used in a closed fold.
                let line = s.line((*pos).lnum);
                if ml_get_buf_len(buf, (*pos).lnum) <= (*pos).col {
                    1
                } else {
                    utfc_ptr2len(line.offset((*pos).col as isize))
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
            if (*pos).lnum == 0 {
                // Correct lnum for when starting in line 0.
                (*pos).lnum = 1;
                (*pos).col = 0;
                at_first_line = false;
            }

            // Start in the current line, unless searching backwards from
            // column 0 without accepting a match there — skipping back a
            // line is then free.
            if dir == BACKWARD && start.pos.col == 0 && !accept_at_start {
                lnum = (*pos).lnum - 1;
                at_first_line = false;
            } else {
                lnum = (*pos).lnum;
            }

            // Loop twice if 'wrapscan' is set.
            for wrapped in [false, true] {
                'lines: while lnum > 0 && lnum <= (*buf).b_ml.ml_line_count {
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
                        (*pos).col
                    } else {
                        0
                    };
                    let mut nmatched = s.exec(lnum, col);
                    // vim_regexec_multi() may clear "regprog".
                    if s.regmatch.regprog.is_null() {
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
                            if s.opt(SEARCH_PEEK)
                                && ((lnum - (*pos).lnum) & 0x3f) == 0
                                && char_avail()
                            {
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
                        // "lnum" may be past the end of the buffer for "\n\zs".
                        let line = if lnum + m.start.lnum > (*buf).b_ml.ml_line_count {
                            c"".as_ptr() as *mut c_char
                        } else {
                            s.line(lnum + m.start.lnum)
                        };

                        if dir == FORWARD
                            && at_first_line
                            && !s.skip_to_start_pos(
                                lnum,
                                line,
                                &mut m,
                                &mut nmatched,
                                start,
                                first_match,
                            )
                        {
                            break 'next_line;
                        }
                        if dir == BACKWARD
                            && !s.last_match_before(
                                lnum,
                                line,
                                &mut m,
                                &mut nmatched,
                                start,
                                wrapped,
                            )
                        {
                            // There is only a match after the cursor.
                            break 'next_line;
                        }

                        s.record(lnum, m, pos, end_pos);
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
                if s.regmatch.regprog.is_null() {
                    break;
                }

                // Stop when 'wrapscan' is off, "stop_lnum" was given,
                // after an interrupt, after a match, and after looping
                // twice.
                if p_ws.get() == 0
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
                    (*buf).b_ml.ml_line_count
                } else {
                    1
                };
                if !shortmess(SHM_SEARCH as c_int)
                    && shortmess(SHM_SEARCHCOUNT as c_int)
                    && s.opt(SEARCH_MSG)
                {
                    let msg = if dir == BACKWARD {
                        top_bot_msg.ptr()
                    } else {
                        bot_top_msg.ptr()
                    };
                    give_warning(gettext(msg.cast()), true, false);
                }
                if !extra_arg.is_null() {
                    (*extra_arg).sa_wrapped = true as c_int;
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

        vim_regfree(s.regmatch.regprog);

        if found == 0 {
            if got_int.get() {
                emsg(gettext(e_interr.as_ptr()));
            } else if options & SEARCH_MSG == SEARCH_MSG {
                let msg = if p_ws.get() != 0 {
                    gettext(e_patnotf2.as_ptr())
                } else if lnum == 0 {
                    gettext(c"E384: Search hit TOP without match for: %s".as_ptr())
                } else {
                    gettext(c"E385: Search hit BOTTOM without match for: %s".as_ptr())
                };
                semsg_c!(msg, get_search_pat());
            }
            return FAIL;
        }

        // A pattern like "\n\zs" may go past the last line.
        if (*pos).lnum > (*buf).b_ml.ml_line_count {
            (*pos).lnum = (*buf).b_ml.ml_line_count;
            (*pos).col = ml_get_buf_len(buf, (*pos).lnum);
            if (*pos).col > 0 {
                (*pos).col -= 1;
            }
        }

        submatch + 1
    }
}

/// The number of the first sub-pattern that matched, or zero if none of
/// them did.
///
/// # Safety
/// `rp` must point at the result of a successful match.
#[inline(always)]
unsafe fn first_submatch(rp: *mut regmmatch_T) -> c_int {
    unsafe {
        let mut submatch = 1;
        while (*rp).startpos[submatch as usize].lnum < 0 {
            if submatch == 9 {
                return 0;
            }
            submatch += 1;
        }
        submatch
    }
}

/// Search for a line starting with `pat`, ignoring leading white space,
/// from `pos` in direction `dir`; `pos` is left on the line found.
///
/// Blank lines match only while insert-mode completion is adding lines.
/// With `'ignorecase'` the pattern must be in lower case.
///
/// # Safety
/// `pos` must be writable and `pat` a NUL-terminated string.
///
/// @return  OK for success, FAIL if no line was found.
pub unsafe fn search_for_exact_line(
    buf: *mut buf_T,
    pos: *mut pos_T,
    dir: Direction,
    pat: *mut c_char,
) -> c_int {
    unsafe {
        let mut start: linenr_T = 0;
        let compl_len = ins_compl_len();
        if (*buf).b_ml.ml_line_count == 0 {
            return FAIL;
        }
        loop {
            (*pos).lnum += dir;
            if (*pos).lnum < 1 {
                if p_ws.get() == 0 {
                    (*pos).lnum = 1;
                    break;
                }
                (*pos).lnum = (*buf).b_ml.ml_line_count;
                if !shortmess(SHM_SEARCH as c_int) {
                    give_warning(gettext(top_bot_msg.ptr().cast()), true, false);
                }
            } else if (*pos).lnum > (*buf).b_ml.ml_line_count {
                (*pos).lnum = 1;
                if p_ws.get() == 0 {
                    break;
                }
                if !shortmess(SHM_SEARCH as c_int) {
                    give_warning(gettext(bot_top_msg.ptr().cast()), true, false);
                }
            }
            if (*pos).lnum == start {
                break;
            }
            if start == 0 {
                start = (*pos).lnum;
            }

            let line = ml_get_buf(buf, (*pos).lnum);
            let text = skipwhite(line);
            (*pos).col = text.offset_from(line) as colnr_T;

            if compl_status_adding() && !compl_status_sol() {
                // When adding lines the matching line may be empty; it is
                // not ignored, because it is the *next* line that is
                // wanted. -- Acevedo
                if mb_strcmp_ic(p_ic.get() != 0, text, pat) == 0 {
                    return OK;
                }
            } else if *text as c_int != NUL {
                // Expanding lines or words; ignore empty lines.
                debug_assert!(compl_len >= 0);
                let same = if p_ic.get() != 0 {
                    mb_strnicmp(text, pat, compl_len as size_t)
                } else {
                    strncmp(text, pat, compl_len as size_t)
                };
                if same == 0 {
                    return OK;
                }
            }
        }
        FAIL
    }
}
