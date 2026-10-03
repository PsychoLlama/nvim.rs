//! `:s` and `:g` pattern completion from the buffer's own text.
//!
//! [`expand_pattern_in_buf`] searches the buffer for the pattern being typed
//! and offers what follows each match as a completion, so that `:%s/foo<Tab>`
//! grows into the words that actually occur.

#![forbid(unsafe_code)]

use super::*;
use crate::getchar::state::got_int;
use crate::getchar::{char_avail, vpeekc};
use crate::guard::Suppress;
use crate::mbyte::class_in;
use crate::mbyte::cluster_len;
use crate::memline::Lines;
use crate::option::vars::{p_ic, p_scs, wop_flags};
use crate::options::kOptWopFlagExacttext;
use crate::pos::ltoreq;
use crate::regexp::{OwnedMatch, RE_LAST, RE_MAGIC, RE_STRING, vim_regexec_nl};
use crate::search::state::{search_first_line, search_last_line};
use crate::search::{
    BACKWARD, FORWARD, SEARCH_NFMSG, SEARCH_NOOF, SEARCH_OPT, SEARCH_PEEK, SEARCH_START,
    pat_has_uppercase_of, searchit_buf,
};
use crate::strings::strcase_copy;
use crate::tag::TAG_MANY;
use crate::types::{ColNr, Direction, Failed, LineNr};
use crate::winlayer::Buf;
use core::ffi::{CStr, c_uint};
use std::ffi::CString;

/// True when `'wildoptions'` carries `exacttext`, which offers the buffer text
/// itself rather than a pattern that would match it.
fn exacttext() -> bool {
    wop_flags.get() & kOptWopFlagExacttext as c_uint != 0
}

/// Line `lnum` of the current buffer.
fn line_of(lnum: LineNr) -> Vec<u8> {
    Lines::current().line(lnum).to_vec()
}

/// Where the word that starts at `start` in `line` ends: upstream's
/// `find_word_end`, which only goes past a character of a word class.
fn word_end(line: &[u8], start: usize) -> usize {
    let buffer = Buf::current();
    let start_class = class_in(line.get(start..).unwrap_or_default(), buffer);
    let mut p = start;
    if start_class > 1 {
        while p < line.len() {
            p += cluster_len(&line[p..]);
            if class_in(&line[p..], buffer) != start_class {
                break;
            }
        }
    }
    p
}

/// A column as an index into its line.
fn col_index(col: ColNr) -> usize {
    usize::try_from(col).unwrap_or(0)
}

/// Copy a substring from the current buffer, spanning from `start` to the word
/// boundary after `end`.
///
/// Answers the copy and where the copied text ends.
pub(crate) fn copy_substring_from_pos(start: Pos, end: Pos) -> Result<(Vec<u8>, Pos), Failed> {
    let exacttext = exacttext();

    if start.lnum > end.lnum || (start.lnum == end.lnum && start.col >= end.col) {
        return Err(Failed); // invalid range
    }

    // A newline, spelled the way `'wildoptions'` wants it: `exacttext`
    // keeps the two-character `\n` a pattern would use.
    let newline: &[u8] = if exacttext { b"\\n" } else { b"\n" };

    let mut text = Vec::<u8>::new();

    // Append start line from start->col to end.
    let start_line = line_of(start.lnum);
    let is_single_line = start.lnum == end.lnum;
    let from = col_index(start.col).min(start_line.len());
    let to = if is_single_line {
        col_index(end.col).min(start_line.len())
    } else {
        start_line.len()
    };
    text.extend_from_slice(&start_line[from..to.max(from)]);
    if !is_single_line {
        text.extend_from_slice(newline);

        // Append full lines between start and end.
        for lnum in start.lnum + 1..end.lnum {
            text.extend_from_slice(Lines::current().line(lnum));
            text.extend_from_slice(newline);
        }
    }

    // Append partial end line (up to word end).
    let end_line = line_of(end.lnum);
    let word_end = word_end(&end_line, col_index(end.col));
    let from = if is_single_line {
        col_index(end.col)
    } else {
        0
    };
    text.extend_from_slice(&end_line[from.min(word_end)..word_end]);

    let match_end = Pos {
        lnum: end.lnum,
        col: ColNr::try_from(word_end).expect("a column fits a ColNr"),
        coladd: 0,
    };
    Ok((text, match_end))
}

/// True if `str` matches the regex pattern `pat`.
///
/// Honours `'ignorecase'` and `'smartcase'` to decide case sensitivity.
pub(crate) fn is_regex_match(pat: &CStr, str: &CStr) -> bool {
    if pat == str {
        return true;
    }

    let quiet = Suppress::output();
    let compiled = OwnedMatch::compile(pat, RE_MAGIC + RE_STRING, false);
    drop(quiet);

    let Some(mut regmatch) = compiled else {
        return false;
    };
    regmatch.rm_ic = p_ic();
    if p_ic() && p_scs() {
        regmatch.rm_ic = !pat_has_uppercase_of(pat);
    }

    let quiet = Suppress::output();
    let result = vim_regexec_nl(&mut regmatch, str, 0);
    drop(quiet);
    result
}

/// Build a new match string by appending the buffer word that follows
/// `end_match_pos` to the pattern `pat` itself.
///
/// If `lowercase` is true the appended text is folded down first, which is how
/// `'smartcase'` behaviour is reproduced.
pub(crate) fn concat_pattern_with_buffer_match(
    pat: &[u8],
    end_match_pos: Pos,
    lowercase: bool,
) -> CString {
    let line = line_of(end_match_pos.lnum);
    let word = col_index(end_match_pos.col).min(line.len());
    let word = &line[word..word_end(&line, word)];
    let mut out = pat.to_vec();
    if !word.is_empty() {
        if lowercase {
            let lower = strcase_copy(&CString::new(word).unwrap_or_default(), false);
            out.extend_from_slice(&lower);
        } else {
            out.extend_from_slice(word);
        }
    }
    CString::new(out).unwrap_or_default()
}

/// Search for strings matching `pat` in the specified range and return them.
///
/// `dir` is `FORWARD` or `BACKWARD`.
pub(crate) fn expand_pattern_in_buf(pat: &CStr, dir: Direction) -> Result<Vec<XString>, Failed> {
    let exacttext = exacttext();
    let has_range = search_first_line.get() != 0;

    if pat.is_empty() {
        return Err(Failed);
    }

    let mut cur_match_pos = if has_range {
        Pos {
            lnum: search_first_line.get(),
            col: 0,
            coladd: 0,
        }
    } else {
        pre_incsearch_pos.get()
    };
    let mut prev_match_pos = Pos::default();

    let search_flags = SEARCH_OPT
        | SEARCH_NOOF
        | SEARCH_PEEK
        | SEARCH_NFMSG
        | if has_range { SEARCH_START } else { 0 };

    // The matches found so far, in the order they were found.
    let mut found = Vec::<CString>::new();

    let mut end_match_pos = Pos::default();
    let mut looped_around = false;
    let mut compl_started = false;

    loop {
        let quiet = Suppress::output();
        let found_new_match = searchit_buf(
            Buf::current(),
            &mut cur_match_pos,
            &mut end_match_pos,
            dir,
            pat,
            search_flags,
            RE_LAST,
        );
        drop(quiet);

        if !found_new_match {
            break;
        }

        // If in range mode, check if match is within the range.
        if has_range
            && (cur_match_pos.lnum < search_first_line.get()
                || cur_match_pos.lnum > search_last_line.get())
        {
            break;
        }

        if compl_started {
            // If we've looped back to an earlier match, stop.
            if (dir == FORWARD && ltoreq(cur_match_pos, prev_match_pos))
                || (dir == BACKWARD && ltoreq(prev_match_pos, cur_match_pos))
            {
                if looped_around {
                    break;
                }
                looped_around = true;
            }
        }

        compl_started = true;
        prev_match_pos = cur_match_pos;

        // Abort if the user typed a character or interrupted.
        if char_avail() || got_int.get() {
            if got_int.get() {
                vpeekc(); // Remove <C-C> from input stream
                got_int.set(false); // Don't abandon the command line
            }
            return Err(Failed);
        }

        // searchit() can return line number +1 past the last line when
        // searching for "foo\n" if "foo" is at end of buffer.
        if end_match_pos.lnum > Buf::current().b_ml.ml_line_count {
            cur_match_pos = Pos {
                lnum: 1,
                col: 0,
                coladd: 0,
            };
            continue;
        }

        // Extract the matching text prepended to the completed word.
        let Ok((full_match, word_end_pos)) = copy_substring_from_pos(cur_match_pos, end_match_pos)
        else {
            break;
        };
        let full_match = CString::new(full_match).unwrap_or_default();

        let match_out = if exacttext {
            full_match
        } else {
            // Construct a new match from the completed word appended
            // to the pattern itself.  The regex pattern may include '\C' or
            // '\c': first try matching the buffer word as-is; if it doesn't
            // match, try again with the lowercase version of the word to
            // handle smartcase behaviour.
            let pattern = pat.to_bytes();
            let as_is = concat_pattern_with_buffer_match(pattern, end_match_pos, false);
            if is_regex_match(&as_is, &full_match) {
                as_is
            } else {
                let lower = concat_pattern_with_buffer_match(pattern, end_match_pos, true);
                if !is_regex_match(&lower, &full_match) {
                    continue;
                }
                lower
            }
        };

        // Include this match if it is not a duplicate.
        if !found.contains(&match_out) {
            found.push(match_out);
            if found.len() > TAG_MANY as usize {
                break;
            }
        }
        if has_range {
            cur_match_pos = word_end_pos;
        }
    }

    Ok(found.iter().map(|m| XString::from_cstr(m)).collect())
}
