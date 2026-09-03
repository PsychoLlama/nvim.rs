//! Fuzzy matching: how well does a pattern describe a string?
//!
//! The scorer is a port of [fzy](https://github.com/jhawthorn/fzy) extended
//! to multibyte characters — [`match_positions`] fills a dynamic-programming
//! table and reads back both the score and where each pattern character
//! landed. Around it sit the three questions the editor asks:
//! [`fuzzy_match`] for a score and the positions behind it,
//! [`fuzzy_match_str`] for a score alone, and [`fuzzy_match_str_with_pos`]
//! for the popup menu's highlighting. `matchfuzzy()`/`matchfuzzypos()` live
//! with the rest of the `match*()` family in `eval::funcs::regexp`, and
//! completion's buffer walk with its caller in `insexpand::sources`.
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
//! # Characters, not bytes
//!
//! Every step of the scoring is over *characters*: the tables are indexed by
//! character, a reported position is a character index, and the whitespace a
//! multi-word pattern splits on is a character too. So each string is decoded
//! once on the way in ([`chars`]) and nothing below ever looks at a byte
//! again — which is also what makes the file safe, since the decode is the
//! only thing here that ever wanted a pointer.
//!
//! Portions of this file are adapted from fzy. Original code: Copyright (c)
//! 2014 John Hawthorn, MIT licensed.

#![forbid(unsafe_code)]

use core::ffi::c_int;
use std::ffi::CStr;

use crate::ascii::ascii_iswhite;
use crate::charset::is_word_char;
use crate::mbyte::{chars as decode, mb_islower, mb_isupper, mb_tolower, mb_toupper};

/// The most characters of a pattern or a candidate that are looked at, and
/// so the most match positions that can be reported.
pub const FUZZY_MATCH_MAX_LEN: usize = 1024;
/// The score of a candidate the pattern does not match at all.
pub const FUZZY_SCORE_NONE: c_int = c_int::MIN;

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

/// The codepoints of `s`, as `MB_PTR_ADV` steps over them — one per base
/// character, with any composing characters folded into the step.
fn chars(s: &CStr) -> Vec<c_int> {
    decode(s).map(|(_, c)| c).collect()
}

/// The whitespace-separated words of a pattern. An empty pattern, and one
/// that is nothing but separators, has none — which is how a candidate ends
/// up with a score of zero rather than a rejection.
fn words(pattern: &[c_int]) -> impl Iterator<Item = &[c_int]> {
    let separator = |&c: &c_int| c == b' ' as c_int || c == b'\t' as c_int;
    pattern.split(separator).filter(|word| !word.is_empty())
}

/// How many of `pattern`'s characters can take part in a match, and so how
/// many positions a successful match reports: all of them under `matchseq`,
/// and everything but the word separators without it.
pub(crate) fn matched_char_count(pattern: &CStr, matchseq: bool) -> usize {
    let counted = decode(pattern).filter(|&(_, c)| matchseq || !ascii_iswhite(c));
    counted.count().min(FUZZY_MATCH_MAX_LEN)
}

/// C's `MAX` on scores: `a` only when strictly greater, so a NaN answers `b`.
fn max_score(a: Score, b: Score) -> Score {
    if a > b { a } else { b }
}

/// Is every character of `needle` somewhere in `haystack`, in order?
///
/// A lowercase pattern character also matches its uppercase form, which is
/// what makes fuzzy matching case-insensitive in one direction only.
fn has_match(needle: &[c_int], haystack: &[c_int]) -> bool {
    if needle.is_empty() {
        return false;
    }
    let mut haystack = haystack.iter();
    needle
        .iter()
        .all(|&n| haystack.any(|&h| n == h || mb_toupper(n) == h))
}

/// What a haystack character earns for the character in front of it: the
/// start of a path component, of a word, of an extension, or the capital
/// that starts the second half of a CamelCase name.
fn compute_bonus(last_c: c_int, c: c_int) -> Score {
    // A codepoint outside ASCII is not alphanumeric, as C's ASCII_ISALNUM
    // classifies them.
    let alnum = u8::try_from(c).is_ok_and(|b| b.is_ascii_alphanumeric());
    if !(alnum || is_word_char(c)) {
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

/// Both strings reduced to what the score depends on: their lowercased
/// codepoints, plus the bonus each haystack character earns from its
/// predecessor. Anything past [`FUZZY_MATCH_MAX_LEN`] characters is ignored.
struct Match {
    lower_needle: Vec<c_int>,
    lower_haystack: Vec<c_int>,
    match_bonus: Vec<Score>,
}

impl Match {
    fn new(needle: &[c_int], haystack: &[c_int]) -> Self {
        let fold = |s: &[c_int]| -> Vec<c_int> {
            s.iter()
                .take(FUZZY_MATCH_MAX_LEN)
                .map(|&c| mb_tolower(c))
                .collect()
        };
        // The first character is treated as if a path separator preceded it.
        let mut last_c = b'/' as c_int;
        let mut match_bonus = Vec::with_capacity(haystack.len().min(FUZZY_MATCH_MAX_LEN));
        for &c in haystack.iter().take(FUZZY_MATCH_MAX_LEN) {
            match_bonus.push(compute_bonus(last_c, c));
            last_c = c;
        }
        Match {
            lower_needle: fold(needle),
            lower_haystack: fold(haystack),
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
fn match_positions(needle: &[c_int], haystack: &[c_int], positions: &mut [u32]) -> Score {
    if needle.is_empty() {
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
pub fn fuzzy_match(
    haystack: &CStr,
    pattern: &CStr,
    matchseq: bool,
    positions: &mut [u32],
) -> (c_int, usize) {
    let rejected = (FUZZY_SCORE_NONE, 0);
    let (haystack, pattern) = (chars(haystack), chars(pattern));
    if matchseq {
        if !has_match(&pattern, &haystack) {
            return rejected;
        }
        let score = scale(match_positions(&pattern, &haystack, positions));
        return (add_score(0, score), pattern.len());
    }

    let (mut total, mut filled) = (0, 0);
    for word in words(&pattern) {
        if !has_match(word, &haystack) {
            return rejected;
        }
        let score = scale(match_positions(word, &haystack, &mut positions[filled..]));
        total = add_score(total, score);
        filled += word.len();
        if filled >= positions.len() {
            break;
        }
    }
    (total, filled)
}

/// Fuzzy match `pat` in `str`, as one sequence. Answers
/// [`FUZZY_SCORE_NONE`] when there is no match.
pub(crate) fn fuzzy_match_str(str: &CStr, pat: &CStr) -> c_int {
    let mut matches = [0u32; FUZZY_MATCH_MAX_LEN];
    fuzzy_match(str, pat, true, &mut matches).0
}

/// Where `pat` matches in `str`, as one character position per pattern
/// character that took part — i.e. everything but the whitespace between the
/// words — or `None` when there is no match.
pub(crate) fn fuzzy_match_str_with_pos(str: &CStr, pat: &CStr) -> Option<Vec<u32>> {
    let mut matches = [0u32; FUZZY_MATCH_MAX_LEN];
    let (score, filled) = fuzzy_match(str, pat, false, &mut matches);
    if filled == 0 || score == FUZZY_SCORE_NONE {
        return None;
    }
    Some(matches[..matched_char_count(pat, false)].to_vec())
}
