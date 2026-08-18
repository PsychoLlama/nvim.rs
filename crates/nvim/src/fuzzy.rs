//! Fuzzy matching: how well does a pattern describe a string?
//!
//! The scorer is a port of [fzy](https://github.com/jhawthorn/fzy) extended
//! to multibyte characters — [`match_positions`] fills a dynamic-programming
//! table and reads back both the score and where each pattern character
//! landed. Around it sit the entry points the editor uses:
//! [`fuzzy_match_str`] to score a candidate, [`fuzzy_match_str_with_pos`] for
//! the popup menu's highlighting, [`search_for_fuzzy_match`] for
//! `'completeopt'=fuzzy`, and [`f_matchfuzzy`]/[`f_matchfuzzypos`].
//!
//! A pattern is matched word by word (whitespace separated) unless the caller
//! asks for `matchseq`, in which case the whole pattern including its spaces
//! is one unit. Each word is scored against the whole candidate and the
//! scores are added, saturating rather than wrapping; a word that does not
//! match at all rejects the candidate outright.
//!
//! fzy scores in floating point, where a run of consecutive characters, a
//! character after a path separator, and the capital of a CamelCase word all
//! earn a bonus. Everything outside this file compares `int` scores, so the
//! result is scaled by `SCORE_SCALE` and rounded away from zero; the two
//! infinities become `c_int::MAX` and `c_int::MIN + 1`, leaving
//! [`FUZZY_SCORE_NONE`] (`c_int::MIN`) to mean "no match at all".
//!
//! Portions of this file are adapted from fzy. Original code: Copyright (c)
//! 2014 John Hawthorn, MIT licensed.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{c_char, c_int};
use std::ffi::CStr;

use crate::ascii::ascii_iswhite;
use crate::charset::{vim_iswordc, vim_iswordp};
use crate::eval::callback_call;
use crate::eval::typval::{
    callback_free, kCallbackNone, tv_check_for_nonnull_dict_arg, tv_clear, tv_dict_find,
    tv_dict_get_callback, tv_dict_get_string, tv_dict_has_key, tv_dict_unref, tv_get_number_chk,
    tv_get_string, tv_list_alloc, tv_list_alloc_ret, tv_list_append_list, tv_list_append_number,
    tv_list_append_tv, tv_list_find,
};
use crate::garray::{ga_grow, ga_init};
use crate::insexpand::{ctrl_x_mode_whole_line, find_line_end, find_word_end, find_word_start};
use crate::main::{curbuf, e_invarg2, e_invargNval, e_invargval, e_listarg, p_ws};
use crate::mbyte::{mb_islower, mb_isupper, mb_tolower, mb_toupper, utf_ptr2char, utfc_ptr2len};
use crate::memline::{ml_get_buf, ml_get_buf_len};
use crate::memory::{xfree, xmalloc};
use crate::os::libc::gettext;
use crate::pos::equalpos;
use crate::search::FORWARD;
use crate::types::{
    Callback, Callback_data, EvalFuncData, VAR_DICT, VAR_LIST, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN,
    VAR_UNLOCKED, buf_T, dict_T, fuzmatch_str_T, garray_T, kListLenMayKnow, kListLenUnknown,
    linenr_T, list_T, listitem_T, pos_T, typval_T, typval_vval_union, varnumber_T,
};

/// The most characters of a pattern or a candidate that are looked at, and
/// so the most match positions that can be reported.
pub const FUZZY_MATCH_MAX_LEN: usize = 1024;
/// The score of a candidate the pattern does not match at all.
pub const FUZZY_SCORE_NONE: c_int = c_int::MIN;

const FAIL: c_int = 0;

/// An unset typval, as `VAR_UNKNOWN` spells it.
const TV_UNKNOWN: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VAR_UNLOCKED,
    vval: typval_vval_union { v_number: 0 },
};

/// fzy's score, before it is scaled to an `int`.
type Score = f64;

/// No match on this path — a real score, unlike [`FUZZY_SCORE_NONE`].
const SCORE_MIN: Score = Score::NEG_INFINITY;
/// The candidate is the pattern, ignoring case.
const SCORE_MAX: Score = Score::INFINITY;
/// What a fzy score is multiplied by to become an `int` score.
const SCORE_SCALE: Score = 1000.0;

const SCORE_GAP_LEADING: Score = -0.005;
const SCORE_GAP_TRAILING: Score = -0.005;
const SCORE_GAP_INNER: Score = -0.01;
const SCORE_MATCH_CONSECUTIVE: Score = 1.0;
const SCORE_MATCH_SLASH: Score = 0.9;
const SCORE_MATCH_WORD: Score = 0.8;
const SCORE_MATCH_CAPITAL: Score = 0.7;
const SCORE_MATCH_DOT: Score = 0.6;

/// The characters of a string, as `MB_PTR_ADV` steps over them: one item per
/// base character, with any composing characters folded into the step.
struct Chars<'a> {
    str: &'a CStr,
    /// Bytes before the NUL, worked out once.
    len: usize,
    /// Byte offset of the next character.
    at: usize,
}

impl<'a> Chars<'a> {
    fn new(str: &'a CStr) -> Self {
        Chars {
            str,
            len: str.to_bytes().len(),
            at: 0,
        }
    }
}

impl Iterator for Chars<'_> {
    /// The byte offset the character starts at, and its codepoint.
    type Item = (usize, c_int);

    fn next(&mut self) -> Option<(usize, c_int)> {
        if self.at >= self.len {
            return None;
        }
        let at = self.at;
        // SAFETY: `at` is a character boundary before the NUL of a
        // NUL-terminated string, which is what both of these want. Neither
        // reads past the NUL.
        let (c, len) = unsafe {
            let p = self.str.as_ptr().add(at);
            (utf_ptr2char(p), utfc_ptr2len(p) as usize)
        };
        // Zero is the answer only at the NUL, which `at` never points at.
        debug_assert!(len > 0, "fuzzy: utfc_ptr2len stalled mid-string");
        self.at = at + len;
        Some((at, c))
    }
}

/// C's `MAX` on scores: `a` only when strictly greater, so a NaN answers `b`.
fn max_score(a: Score, b: Score) -> Score {
    if a > b { a } else { b }
}

/// Is every character of `needle` somewhere in `haystack`, in order?
///
/// A lowercase pattern character also matches its uppercase form, which is
/// what makes fuzzy matching case-insensitive in one direction only.
fn has_match(needle: &CStr, haystack: &CStr) -> bool {
    if needle.to_bytes().is_empty() {
        return false;
    }
    let mut haystack = Chars::new(haystack);
    Chars::new(needle).all(|(_, n)| haystack.any(|(_, h)| n == h || mb_toupper(n) == h))
}

/// What a haystack character earns for the character in front of it: the
/// start of a path component, of a word, of an extension, or the capital
/// that starts the second half of a CamelCase name.
fn compute_bonus(last_c: c_int, c: c_int) -> Score {
    // A codepoint outside ASCII is not alphanumeric, as C's ASCII_ISALNUM
    // classifies them.
    let alnum = u8::try_from(c).is_ok_and(|b| b.is_ascii_alphanumeric());
    // SAFETY: the other three read only the character-class tables.
    unsafe {
        if !(alnum || vim_iswordc(c)) {
            return 0.0;
        }
        match u8::try_from(last_c) {
            Ok(b'/') => SCORE_MATCH_SLASH,
            Ok(b'-' | b'_' | b' ') => SCORE_MATCH_WORD,
            Ok(b'.') => SCORE_MATCH_DOT,
            _ if mb_isupper(c) && mb_islower(last_c) => SCORE_MATCH_CAPITAL,
            _ => 0.0,
        }
    }
}

/// Both strings reduced to what the score depends on: their lowercased
/// codepoints, plus the bonus each haystack character earns from its
/// predecessor. Anything past [`FUZZY_MATCH_MAX_LEN`] characters is ignored.
struct Match {
    lower_needle: Vec<c_int>,
    lower_haystack: Vec<c_int>,
    match_bonus: Vec<Score>,
}

impl Match {
    fn new(needle: &CStr, haystack: &CStr) -> Self {
        let lower_needle = Chars::new(needle)
            .take(FUZZY_MATCH_MAX_LEN)
            .map(|(_, c)| mb_tolower(c))
            .collect();
        let mut lower_haystack = Vec::new();
        let mut match_bonus = Vec::new();
        // The first character is treated as if a path separator preceded it.
        let mut last_c = b'/' as c_int;
        for (_, c) in Chars::new(haystack).take(FUZZY_MATCH_MAX_LEN) {
            lower_haystack.push(mb_tolower(c));
            match_bonus.push(compute_bonus(last_c, c));
            last_c = c;
        }
        Match {
            lower_needle,
            lower_haystack,
            match_bonus,
        }
    }
}

/// Fill row `i` of the two score matrices: `curr_d[j]` is the best score for
/// a match of the first `i + 1` pattern characters *ending* at haystack
/// character `j`, and `curr_m[j]` the best score reachable by then, match or
/// not. Row 0 reads nothing above it, so `last_d`/`last_m` may be empty there
/// — upstream passes row 0 itself and relies on the reads being dead, which
/// reads uninitialised memory.
fn match_row(
    mtch: &Match,
    i: usize,
    last_d: &[Score],
    last_m: &[Score],
    curr_d: &mut [Score],
    curr_m: &mut [Score],
) {
    let n = mtch.lower_needle.len();
    let m = mtch.lower_haystack.len();
    // A gap after the last pattern character is only the tail of the
    // candidate, which is cheaper than a gap in the middle of the match.
    let gap_score = if i == n - 1 {
        SCORE_GAP_TRAILING
    } else {
        SCORE_GAP_INNER
    };
    let mut prev_score = SCORE_MIN;
    for j in 0..m {
        if mtch.lower_needle[i] == mtch.lower_haystack[j] {
            let score = if i == 0 {
                // Everything before the first match is a leading gap.
                j as Score * SCORE_GAP_LEADING + mtch.match_bonus[j]
            } else if j > 0 {
                max_score(
                    last_m[j - 1] + mtch.match_bonus[j],
                    // A consecutive match does not stack with the bonus.
                    last_d[j - 1] + SCORE_MATCH_CONSECUTIVE,
                )
            } else {
                SCORE_MIN
            };
            curr_d[j] = score;
            prev_score = max_score(score, prev_score + gap_score);
        } else {
            curr_d[j] = SCORE_MIN;
            prev_score += gap_score;
        }
        curr_m[j] = prev_score;
    }
}

/// Score `needle` against `haystack` and record, in `positions`, which
/// haystack character each needle character matched. Only callable when
/// [`has_match`] said yes. `positions` is filled up to its length; upstream
/// writes one entry per needle character with no bound at all, overrunning
/// the caller's array once two pattern words together have more characters
/// than it holds.
fn match_positions(needle: &CStr, haystack: &CStr, positions: &mut [u32]) -> Score {
    if needle.to_bytes().is_empty() {
        return SCORE_MIN;
    }
    let mtch = Match::new(needle, haystack);
    let n = mtch.lower_needle.len();
    let m = mtch.lower_haystack.len();
    if n > m {
        // Cannot be a match; upstream also rejects a candidate longer than
        // FUZZY_MATCH_MAX_LEN here, which `Match` has already truncated to it.
        return SCORE_MIN;
    }
    if n == m {
        // `has_match` only lets equal-length strings through when they are
        // the same string, ignoring case.
        for (i, slot) in positions.iter_mut().take(n).enumerate() {
            *slot = i as u32;
        }
        return SCORE_MAX;
    }

    // Two n×m tables, laid out row by row.
    let mut d = vec![SCORE_MIN; n * m];
    let mut m_best = vec![SCORE_MIN; n * m];
    let (row_d, row_m) = (&mut d[..m], &mut m_best[..m]);
    match_row(&mtch, 0, &[], &[], row_d, row_m);
    for i in 1..n {
        let (above_d, row_d) = d.split_at_mut(i * m);
        let (above_m, row_m) = m_best.split_at_mut(i * m);
        match_row(
            &mtch,
            i,
            &above_d[(i - 1) * m..],
            &above_m[(i - 1) * m..],
            &mut row_d[..m],
            &mut row_m[..m],
        );
    }

    // Walk the tables backwards for the positions that produced the score.
    // Several paths can reach the same weight; take the first one found,
    // which is the latest in the candidate.
    let mut match_required = false;
    let mut j = m as isize - 1;
    for i in (0..n).rev() {
        while j >= 0 {
            let (at, row) = (j as usize, i * m);
            j -= 1;
            if d[row + at] != SCORE_MIN && (match_required || d[row + at] == m_best[row + at]) {
                // A score that came from SCORE_MATCH_CONSECUTIVE says the
                // character before this one has to be a match too.
                match_required = i > 0
                    && at > 0
                    && m_best[row + at] == d[row - m + at - 1] + SCORE_MATCH_CONSECUTIVE;
                if let Some(slot) = positions.get_mut(i) {
                    *slot = at as u32;
                }
                break;
            }
        }
    }
    m_best[(n - 1) * m + m - 1]
}

/// fzy's score as the `int` score the rest of the editor compares.
fn scale(score: Score) -> c_int {
    if score == SCORE_MIN {
        c_int::MIN + 1
    } else if score == SCORE_MAX {
        c_int::MAX
    } else if score < 0.0 {
        (score * SCORE_SCALE - 0.5).ceil() as c_int
    } else {
        (score * SCORE_SCALE + 0.5).floor() as c_int
    }
}

/// Add `score` to `total` the way upstream does: saturating at
/// [`c_int::MAX`] and at `c_int::MIN + 1`, so a summed score never reaches
/// [`FUZZY_SCORE_NONE`].
fn add_score(total: c_int, score: c_int) -> c_int {
    total.saturating_add(score).max(c_int::MIN + 1)
}

/// Match `pattern` against `haystack`, filling `positions` with the haystack
/// character index of each pattern character.
///
/// With `matchseq` the whole pattern is one unit; otherwise its
/// whitespace-separated words are matched independently and their scores
/// added, and any word that fails rejects the candidate.
///
/// Answers the summed score and how many positions were filled — zero, with
/// a score of [`FUZZY_SCORE_NONE`], when the candidate is rejected.
fn fuzzy_match_words(
    haystack: &CStr,
    pattern: &CStr,
    matchseq: bool,
    positions: &mut [u32],
) -> (c_int, usize) {
    let rejected = (FUZZY_SCORE_NONE, 0);
    if matchseq {
        if !has_match(pattern, haystack) {
            return rejected;
        }
        let score = scale(match_positions(pattern, haystack, positions));
        return (add_score(0, score), Chars::new(pattern).count());
    }

    // Upstream terminates each word in a copy of the pattern; the copy is
    // what makes a word a string in its own right.
    let mut buf = pattern.to_bytes_with_nul().to_vec();
    let mut at = 0;
    let mut total = 0;
    let mut filled = 0;
    loop {
        while buf[at] == b' ' || buf[at] == b'\t' {
            at += 1;
        }
        if buf[at] == 0 {
            break;
        }
        let start = at;
        while buf[at] != 0 && buf[at] != b' ' && buf[at] != b'\t' {
            at += 1;
        }
        let complete = buf[at] == 0;
        buf[at] = 0;
        let word = CStr::from_bytes_with_nul(&buf[start..=at]).expect("fuzzy: word is terminated");
        if !has_match(word, haystack) {
            return rejected;
        }
        total = add_score(
            total,
            scale(match_positions(word, haystack, &mut positions[filled..])),
        );
        filled += Chars::new(word).count();
        if complete || filled >= positions.len() {
            break;
        }
        // Step over the NUL that ended the word.
        at += 1;
    }
    (total, filled)
}

/// Fuzzy match `pat_arg` in `str`, reporting the score in `out_score` and the
/// matching character positions in `matches`. With `matchseq` the words of a
/// multi-word pattern have to match in sequence rather than independently.
///
/// # Safety
/// `str` and `pat_arg` must be NUL-terminated strings, and `matches` must
/// point at `max_matches` writable entries.
pub unsafe fn fuzzy_match(
    str: *const c_char,
    pat_arg: *const c_char,
    matchseq: bool,
    out_score: *mut c_int,
    matches: *mut u32,
    max_matches: c_int,
) -> bool {
    unsafe {
        let (score, filled) = fuzzy_match_words(
            CStr::from_ptr(str),
            CStr::from_ptr(pat_arg),
            matchseq,
            core::slice::from_raw_parts_mut(matches, max_matches as usize),
        );
        *out_score = score;
        filled != 0
    }
}

/// Fuzzy match `pat` in `str`, as one sequence. Answers 0 for a missing
/// string, [`FUZZY_SCORE_NONE`] for no match.
///
/// # Safety
/// Both arguments must be NUL-terminated strings or NULL.
pub unsafe fn fuzzy_match_str(str: *const c_char, pat: *const c_char) -> c_int {
    if str.is_null() || pat.is_null() {
        return 0;
    }
    let mut matches = [0u32; FUZZY_MATCH_MAX_LEN];
    // SAFETY: the caller's promise, plus a big enough array.
    unsafe { fuzzy_match_words(CStr::from_ptr(str), CStr::from_ptr(pat), true, &mut matches).0 }
}

/// Where `pat` matches in `str`, as a garray of character positions, or NULL
/// when there is no match. The array is the caller's to free.
///
/// # Safety
/// Both arguments must be NUL-terminated strings or NULL.
pub unsafe fn fuzzy_match_str_with_pos(str: *const c_char, pat: *const c_char) -> *mut garray_T {
    if str.is_null() || pat.is_null() {
        return core::ptr::null_mut();
    }
    unsafe {
        let (str, pat) = (CStr::from_ptr(str), CStr::from_ptr(pat));
        let mut matches = [0u32; FUZZY_MATCH_MAX_LEN];
        let (score, filled) = fuzzy_match_words(str, pat, false, &mut matches);
        if filled == 0 || score == FUZZY_SCORE_NONE {
            return core::ptr::null_mut();
        }

        // One position per pattern character that took part in the match,
        // i.e. everything but the whitespace between the words.
        let placed: Vec<u32> = Chars::new(pat)
            .filter(|&(_, c)| !ascii_iswhite(c))
            .zip(matches)
            .map(|(_, at)| at)
            .collect();
        let positions: *mut garray_T = xmalloc(size_of::<garray_T>()).cast();
        ga_init(positions, size_of::<u32>() as c_int, 10);
        ga_grow(positions, placed.len() as c_int);
        core::ptr::copy_nonoverlapping(
            placed.as_ptr(),
            (*positions).ga_data.cast::<u32>(),
            placed.len(),
        );
        (*positions).ga_len = placed.len() as c_int;
        positions
    }
}

/// Split the line at `*ptr` into words and fuzzy match `pat` against each.
/// On a match `*ptr` points at the matched word, `*len` is its length and
/// `*score` its score; otherwise `*ptr` is left at the end of the line.
///
/// # Safety
/// `*ptr` and `pat` must be NUL-terminated strings or NULL, and the line must
/// be writable — a word is terminated in place while it is scored.
pub unsafe fn fuzzy_match_str_in_line(
    ptr: *mut *mut c_char,
    pat: *const c_char,
    len: *mut c_int,
    current_pos: *mut pos_T,
    score: *mut c_int,
) -> bool {
    unsafe {
        let line = *ptr;
        if line.is_null() || pat.is_null() {
            return false;
        }
        let line_end = find_line_end(line);
        let mut str = line;
        while str < line_end {
            let start = find_word_start(str);
            if *start == 0 {
                break;
            }
            let end = find_word_end(start);
            let save_end = *end;
            *end = 0;
            *score = fuzzy_match_str(start, pat);
            *end = save_end;
            if *score != FUZZY_SCORE_NONE {
                *len = end.offset_from(start) as c_int;
                *ptr = start;
                if !current_pos.is_null() {
                    (*current_pos).col += end.offset_from(line) as c_int;
                }
                return true;
            }

            // Carry on after the word just tried.
            str = end;
            while *str != 0 && !vim_iswordp(str) {
                str = str.offset(utfc_ptr2len(str) as isize);
            }
        }
        *ptr = line_end;
        false
    }
}

/// Where a fuzzy match was found in a buffer line: its start inside the
/// line's own buffer, its length in bytes, and its score — missing for a
/// whole-line match, where upstream leaves the caller's score alone.
pub struct LineMatch {
    pub ptr: *mut c_char,
    pub len: c_int,
    pub score: Option<c_int>,
}

/// Search `buf` for the next fuzzy match of `pattern`, starting at `pos` and
/// going in `dir`, wrapping around to `start_pos` if `'wrapscan'` is set.
/// `pos` is left on the match. In whole-line mode (`CTRL-X CTRL-L`) whole
/// lines are matched rather than words.
///
/// # Safety
/// `pattern` must be a NUL-terminated string, and `pos`/`start_pos` must
/// point at valid positions in `buf`.
pub unsafe fn search_for_fuzzy_match(
    buf: *mut buf_T,
    pos: *mut pos_T,
    pattern: *const c_char,
    dir: c_int,
    start_pos: *const pos_T,
) -> Option<LineMatch> {
    unsafe {
        let whole_line = ctrl_x_mode_whole_line();
        let mut current_pos = *pos;

        // Where the search has come full circle. Another buffer is walked
        // from wherever it is to its end rather than back to the start.
        let circly_end = if buf == curbuf.get() {
            *start_pos
        } else {
            pos_T {
                lnum: (*buf).b_ml.ml_line_count,
                col: 0,
                coladd: 0,
            }
        };
        if whole_line && (*start_pos).lnum != (*pos).lnum {
            current_pos.lnum += dir as linenr_T;
        }
        let mut looped_around = false;
        loop {
            if looped_around
                && (if whole_line {
                    current_pos.lnum == circly_end.lnum
                } else {
                    equalpos(current_pos, circly_end)
                })
            {
                return None;
            }
            if current_pos.lnum >= 1 && current_pos.lnum <= (*buf).b_ml.ml_line_count {
                let line = ml_get_buf(buf, current_pos.lnum);
                let mut ptr = if whole_line {
                    line
                } else {
                    line.offset(current_pos.col as isize)
                };
                if !ptr.is_null() && *ptr != 0 {
                    if whole_line {
                        if fuzzy_match_str(ptr, pattern) != FUZZY_SCORE_NONE {
                            *pos = current_pos;
                            return Some(LineMatch {
                                ptr,
                                len: ml_get_buf_len(buf, current_pos.lnum) as c_int,
                                score: None,
                            });
                        }
                    } else {
                        let (mut len, mut score) = (0, 0);
                        if fuzzy_match_str_in_line(
                            &raw mut ptr,
                            pattern,
                            &raw mut len,
                            &raw mut current_pos,
                            &raw mut score,
                        ) {
                            *pos = current_pos;
                            let score = Some(score);
                            return Some(LineMatch { ptr, len, score });
                        }
                        if looped_around && current_pos.lnum == circly_end.lnum {
                            return None;
                        }
                    }
                }
            }

            // On to the next line, or round to the far end of the buffer
            // if `'wrapscan'` allows it.
            let last = (*buf).b_ml.ml_line_count;
            current_pos.lnum += if dir == FORWARD { 1 } else { -1 };
            if !(1..=last).contains(&current_pos.lnum) {
                if p_ws.get() == 0 {
                    return None;
                }
                current_pos.lnum = if dir == FORWARD { 1 } else { last };
                looped_around = true;
            }
            current_pos.col = 0;
        }
    }
}

/// Sort `fuzmatch` by fuzzy score and hand its strings to `matches`, freeing
/// `fuzmatch` itself. With `funcsort`, `<SNR>` functions sort to the end.
///
/// # Safety
/// `fuzmatch` must be an allocated array of `count` entries naming allocated
/// strings, and `matches` must be writable.
pub unsafe fn fuzzymatches_to_strmatches(
    fuzmatch: *mut fuzmatch_str_T,
    matches: *mut *mut *mut c_char,
    count: c_int,
    funcsort: bool,
) {
    unsafe {
        if count > 0 {
            let count = count as usize;
            let found = core::slice::from_raw_parts_mut(fuzmatch, count);
            // Best score first, `idx` breaking ties — and with `funcsort`,
            // `<SNR>` functions after everything else whatever they scored.
            // Callers number `idx` as they fill the array, so no two entries
            // compare equal and the sort needs no stability of its own.
            let snr = |m: &fuzmatch_str_T| funcsort && *m.str == b'<' as c_char;
            found.sort_by(|a, b| {
                snr(a)
                    .cmp(&snr(b))
                    .then(b.score.cmp(&a.score))
                    .then(a.idx.cmp(&b.idx))
            });
            let strings: *mut *mut c_char = xmalloc(count * size_of::<*mut c_char>()).cast();
            for (i, m) in found.iter().enumerate() {
                *strings.add(i) = m.str;
            }
            *matches = strings;
        }
        xfree(fuzmatch.cast());
    }
}

/// Where the string to match comes from: the list items are strings, and a
/// dict item then contributes nothing — or they are dicts, to look a key up
/// in or to hand to a callback.
enum Source {
    Item,
    Key(*const c_char),
    Callback(*mut Callback),
}

/// What one `matchfuzzy()`/`matchfuzzypos()` call was asked for: the pattern,
/// where each item's string comes from, whether the words of a multi-word
/// pattern have to match in sequence, whether the matching positions are
/// wanted too (that is `matchfuzzypos()`), and how many matches are enough.
struct Request {
    pattern: *const c_char,
    source: Source,
    matchseq: bool,
    retmatchpos: bool,
    limit: c_int,
}

/// One list item that matched.
struct FuzzyItem {
    /// Where it sat in the input list, which is how ties are broken.
    idx: usize,
    /// The item itself, copied to the result list as it is.
    item: *mut listitem_T,
    score: c_int,
    /// Whether the pattern occurs literally at the first matched position.
    exact: bool,
    /// The matching positions, for `matchfuzzypos()`.
    positions: Option<*mut list_T>,
}

/// The item's string, as `Request::source` says to find it. A callback's
/// answer lands in `rettv`, which the caller clears; the string is only
/// borrowed until then.
unsafe fn item_string(
    request: &Request,
    tv: *const typval_T,
    rettv: *mut typval_T,
) -> *const c_char {
    unsafe {
        if (*tv).v_type == VAR_STRING {
            return (*tv).vval.v_string;
        }
        if (*tv).v_type != VAR_DICT {
            return core::ptr::null();
        }
        match request.source {
            Source::Item => core::ptr::null(),
            Source::Key(key) => tv_dict_get_string((*tv).vval.v_dict, key, false),
            Source::Callback(cb) => {
                // The callback is handed the dict, which it must not be able
                // to free out from under this loop.
                (*(*tv).vval.v_dict).dv_refcount += 1;
                let mut argv = [
                    typval_T {
                        v_type: VAR_DICT,
                        v_lock: VAR_UNLOCKED,
                        vval: typval_vval_union {
                            v_dict: (*tv).vval.v_dict,
                        },
                    },
                    TV_UNKNOWN,
                ];
                let called = callback_call(cb, 1, argv.as_mut_ptr(), rettv);
                tv_dict_unref((*tv).vval.v_dict);
                if called && (*rettv).v_type == VAR_STRING {
                    (*rettv).vval.v_string
                } else {
                    core::ptr::null()
                }
            }
        }
    }
}

/// The list held by item `idx` of `list`, which the caller has just built.
unsafe fn nested_list(list: *mut list_T, idx: c_int) -> *mut list_T {
    unsafe {
        let li = tv_list_find(list, idx);
        debug_assert!(!li.is_null(), "fuzzy: result list is short");
        let nested = (*li).li_tv.vval.v_list;
        debug_assert!(!nested.is_null(), "fuzzy: result item is not a list");
        nested
    }
}

/// Fuzzy match `request`'s pattern against the strings of `list`, appending
/// the matches to `fmatchlist` in descending score order. For `matchfuzzy()`
/// that is a list of strings; for `matchfuzzypos()` `fmatchlist` already
/// holds three lists — the matched strings, the matching positions of each,
/// and the scores — which are filled in turn.
unsafe fn fuzzy_match_in_list(list: *mut list_T, request: &Request, fmatchlist: *mut list_T) {
    unsafe {
        let pattern = CStr::from_ptr(request.pattern);
        let mut found: Vec<FuzzyItem> = Vec::new();
        let mut matches = [0u32; FUZZY_MATCH_MAX_LEN];
        let mut li = (*list).lv_first;
        while !li.is_null() {
            if request.limit > 0 && found.len() >= request.limit as usize {
                break;
            }
            let mut rettv = TV_UNKNOWN;
            let itemstr = item_string(request, &raw const (*li).li_tv, &raw mut rettv);
            if !itemstr.is_null() {
                let itemstr = CStr::from_ptr(itemstr);
                let (score, filled) =
                    fuzzy_match_words(itemstr, pattern, request.matchseq, &mut matches);
                if filled != 0 {
                    // Upstream reads the string at the first *character*
                    // position as if it were a byte offset. Preserved: it is
                    // only a tie-break between two equally scored items.
                    let at = matches[0] as usize;
                    let exact = itemstr
                        .to_bytes()
                        .get(at..)
                        .is_some_and(|tail| tail.starts_with(pattern.to_bytes()));
                    let positions = request.retmatchpos.then(|| {
                        let positions = tv_list_alloc(kListLenMayKnow as isize);
                        // One position per pattern character that took part
                        // in the match, i.e. all but the word separators.
                        let placed = Chars::new(pattern)
                            .filter(|&(_, c)| request.matchseq || !ascii_iswhite(c));
                        for (at, _) in placed.enumerate().take(FUZZY_MATCH_MAX_LEN) {
                            tv_list_append_number(positions, matches[at] as varnumber_T);
                        }
                        positions
                    });
                    found.push(FuzzyItem {
                        idx: found.len(),
                        item: li,
                        score,
                        exact,
                        positions,
                    });
                }
            }
            tv_clear(&raw mut rettv);
            li = (*li).li_next;
        }
        if found.is_empty() {
            return;
        }

        // Best score first; an exact match wins a tie, and the input order
        // settles the rest. No two items share an `idx`, so this is a total
        // order and the sort needs no stability of its own.
        found.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then(b.exact.cmp(&a.exact))
                .then(a.idx.cmp(&b.idx))
        });

        // matchfuzzy() answers just the strings; matchfuzzypos() answers
        // them in the first of its three lists.
        let strings = if request.retmatchpos {
            nested_list(fmatchlist, 0)
        } else {
            fmatchlist
        };
        for item in &found {
            tv_list_append_tv(strings, &raw mut (*item.item).li_tv);
        }
        if request.retmatchpos {
            let positions = nested_list(fmatchlist, -2);
            for item in &mut found {
                let list = item.positions.take().expect("fuzzy: positions were kept");
                tv_list_append_list(positions, list);
            }
            let scores = nested_list(fmatchlist, -1);
            for item in &found {
                tv_list_append_number(scores, item.score as varnumber_T);
            }
        }
    }
}

/// The translated text of one of the shared `e_*` message strings.
fn message<const N: usize>(msg: &'static [c_char; N]) -> *const c_char {
    // SAFETY: gettext answers either its argument or a pointer into the
    // loaded message catalog; both outlive the call.
    unsafe { gettext(msg.as_ptr()) }
}

/// The body of `matchfuzzy()` and, with `retmatchpos`, `matchfuzzypos()`.
unsafe fn do_fuzzymatch(argvars: *const typval_T, rettv: *mut typval_T, retmatchpos: bool) {
    unsafe {
        let list = &*argvars;
        if list.v_type != VAR_LIST || list.vval.v_list.is_null() {
            semsg_c!(
                message(&e_listarg),
                if retmatchpos {
                    c"matchfuzzypos()".as_ptr()
                } else {
                    c"matchfuzzy()".as_ptr()
                },
            );
            return;
        }
        let pat = &*argvars.add(1);
        if pat.v_type != VAR_STRING || pat.vval.v_string.is_null() {
            semsg_c!(message(&e_invarg2), tv_get_string(pat));
            return;
        }

        // The optional third argument says where to find the string of a
        // dict item, and how much of the list to bother with.
        let mut cb = Callback {
            data: Callback_data {
                funcref: core::ptr::null_mut(),
            },
            type_0: kCallbackNone,
        };
        let mut key = core::ptr::null();
        let mut matchseq = false;
        let mut limit = 0;
        if (*argvars.add(2)).v_type != VAR_UNKNOWN {
            if tv_check_for_nonnull_dict_arg(argvars, 2) == FAIL {
                return;
            }
            let d: *mut dict_T = (*argvars.add(2)).vval.v_dict;
            let di = tv_dict_find(d, c"key".as_ptr(), -1);
            if !di.is_null() {
                if (*di).di_tv.v_type != VAR_STRING
                    || (*di).di_tv.vval.v_string.is_null()
                    || *(*di).di_tv.vval.v_string == 0
                {
                    semsg_c!(
                        message(&e_invargNval),
                        c"key".as_ptr(),
                        tv_get_string(&raw const (*di).di_tv),
                    );
                    return;
                }
                key = tv_get_string(&raw const (*di).di_tv);
            } else if !tv_dict_get_callback(d, c"text_cb".as_ptr(), -1, &raw mut cb) {
                semsg_c!(message(&e_invargval), c"text_cb".as_ptr());
                return;
            }
            let di = tv_dict_find(d, c"limit".as_ptr(), -1);
            if !di.is_null() {
                if (*di).di_tv.v_type != VAR_NUMBER {
                    semsg_c!(message(&e_invargval), c"limit".as_ptr());
                    return;
                }
                limit = tv_get_number_chk(&raw const (*di).di_tv, core::ptr::null_mut()) as c_int;
            }
            matchseq = tv_dict_has_key(d, c"matchseq".as_ptr());
        }

        // matchfuzzypos() answers three lists: the matching strings, their
        // matching positions, and their scores.
        let len = if retmatchpos {
            3
        } else {
            kListLenUnknown as isize
        };
        let result = tv_list_alloc_ret(rettv, len);
        if retmatchpos {
            for _ in 0..3 {
                tv_list_append_list(result, tv_list_alloc(kListLenUnknown as isize));
            }
        }
        let request = Request {
            pattern: tv_get_string(pat),
            source: if !key.is_null() {
                Source::Key(key)
            } else if cb.type_0 != kCallbackNone {
                Source::Callback(&raw mut cb)
            } else {
                Source::Item
            },
            matchseq,
            retmatchpos,
            limit,
        };
        fuzzy_match_in_list(list.vval.v_list, &request, result);
        callback_free(&raw mut cb);
    }
}

/// `matchfuzzy()`: the items of a list that fuzzy match a pattern.
///
/// # Safety
/// Called with a Vimscript function's arguments and result slot.
pub unsafe fn f_matchfuzzy(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe { do_fuzzymatch(argvars, rettv, false) }
}

/// `matchfuzzypos()`: as [`f_matchfuzzy`], plus where each match landed and
/// what it scored.
///
/// # Safety
/// Called with a Vimscript function's arguments and result slot.
pub unsafe fn f_matchfuzzypos(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe { do_fuzzymatch(argvars, rettv, true) }
}
