//! `:s` and `:g` pattern completion from the buffer's own text.
//!
//! [`expand_pattern_in_buf`] searches the buffer for the pattern being typed
//! and offers what follows each match as a completion, so that `:%s/foo<Tab>`
//! grows into the words that actually occur.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cstr;
use crate::guard::Suppress;
use crate::memory::handoff::{owned_cstr, owned_cstr_array};
use crate::types::{FAIL, Failed, NUL};
use crate::winlayer::Buf;
use core::ffi::{c_char, c_int, c_uint, c_void};
use core::{ptr, slice};
use std::ffi::CString;

/// A run of `len` bytes at `p`.
///
/// # Safety
///
/// `p` is readable for `len` bytes, which every `ml_get`/`ml_get_len` pair
/// promises for as long as the memline is not disturbed.
unsafe fn bytes<'a>(p: *const c_char, len: c_int) -> &'a [u8] {
    // SAFETY: the caller's promise; a line length is never negative.
    unsafe { slice::from_raw_parts(p.cast::<u8>(), usize::try_from(len).unwrap_or(0)) }
}

/// True when `'wildoptions'` carries `exacttext`, which offers the buffer text
/// itself rather than a pattern that would match it.
fn exacttext() -> bool {
    wop_flags.get() & kOptWopFlagExacttext as c_uint != 0
}

/// Copy a substring from the current buffer, spanning from `start` to the word
/// boundary after `end`.
///
/// The copied string is stored in `*match_out`, and the actual end position of
/// the matched text is returned in `*match_end`.
pub(crate) unsafe fn copy_substring_from_pos(
    start: *mut pos_T,
    end: *mut pos_T,
    match_out: *mut *mut c_char,
    match_end: *mut pos_T,
) -> Result<(), Failed> {
    let exacttext = exacttext();

    if unsafe { (*start).lnum } > unsafe { (*end).lnum }
        || (unsafe { (*start).lnum } == unsafe { (*end).lnum }
            && unsafe { (*start).col } >= unsafe { (*end).col })
    {
        return Err(Failed); // invalid range
    }

    // A newline, spelled the way `'wildoptions'` wants it: `exacttext`
    // keeps the two-character `\n` a pattern would use.
    let newline: &[u8] = if exacttext { b"\\n" } else { b"\n" };

    let mut text = Vec::<u8>::new();

    // SAFETY (this body): every `ml_get`/`ml_get_len` pair describes one
    // line of the current buffer, and nothing between the read and the copy
    // touches the memline, so the bytes stay where they are.
    // Append start line from start->col to end.
    let start_line = ml_get(unsafe { (*start).lnum });
    let start_ptr = unsafe { start_line.offset((*start).col as isize) };
    let is_single_line = unsafe { (*start).lnum } == unsafe { (*end).lnum };

    let mut segment_len = if is_single_line {
        unsafe { (*end).col - (*start).col }
    } else {
        ml_get_len(unsafe { (*start).lnum }) - unsafe { (*start).col }
    };
    text.extend_from_slice(unsafe { bytes(start_ptr, segment_len) });
    if !is_single_line {
        text.extend_from_slice(newline);

        // Append full lines between start and end.
        let mut lnum = unsafe { (*start).lnum } + 1;
        while lnum < unsafe { (*end).lnum } {
            let line = ml_get(lnum);
            let linelen = ml_get_len(lnum);
            text.extend_from_slice(unsafe { bytes(line, linelen) });
            text.extend_from_slice(newline);
            lnum += 1;
        }
    }

    // Append partial end line (up to word end).
    let end_line = ml_get(unsafe { (*end).lnum });
    let word_end = unsafe { find_word_end(end_line.offset((*end).col as isize)) };
    segment_len = unsafe { word_end.offset_from(end_line) } as c_int;
    let from = if is_single_line {
        unsafe { (*end).col }
    } else {
        0
    };
    text.extend_from_slice(unsafe { bytes(end_line.offset(from as isize), segment_len - from) });

    unsafe { *match_out = owned_cstr(text) };
    unsafe { (*match_end).lnum = (*end).lnum };
    unsafe { (*match_end).col = segment_len as colnr_T };

    Ok(())
}

/// True if `str` matches the regex pattern `pat`.
///
/// Honours `'ignorecase'` and `'smartcase'` to decide case sensitivity.
pub(crate) unsafe fn is_regex_match(pat: *mut c_char, str: *mut c_char) -> bool {
    if unsafe { cstr::eq(pat, str) } {
        return true;
    }

    let mut regmatch = regmatch_T {
        regprog: ptr::null_mut(),
        startp: [ptr::null_mut(); 10],
        endp: [ptr::null_mut(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };

    let quiet = Suppress::output();
    regmatch.regprog = unsafe { vim_regcomp(pat, RE_MAGIC + RE_STRING) };
    drop(quiet);

    if regmatch.regprog.is_null() {
        return false;
    }
    regmatch.rm_ic = p_ic.get() != 0;
    if p_ic.get() != 0 && p_scs.get() != 0 {
        regmatch.rm_ic = !unsafe { pat_has_uppercase(pat) };
    }

    let quiet = Suppress::output();
    let result = unsafe { vim_regexec_nl(&raw mut regmatch, str, 0) };
    drop(quiet);

    unsafe { vim_regfree(regmatch.regprog) };
    result
}

/// Build a new match string by appending the buffer word that follows
/// `end_match_pos` to the pattern `pat` itself.
///
/// If `lowercase` is true the appended text is folded down first, which is how
/// `'smartcase'` behaviour is reproduced.  The answer is never NULL.
pub(crate) unsafe fn concat_pattern_with_buffer_match(
    pat: *mut c_char,
    pat_len: c_int,
    end_match_pos: *mut pos_T,
    lowercase: bool,
) -> *mut c_char {
    let line = ml_get(unsafe { (*end_match_pos).lnum });
    let word = unsafe { line.offset((*end_match_pos).col as isize) };
    let word_end = unsafe { find_word_end(word) };
    let match_len = unsafe { word_end.offset_from(word) } as c_int;
    // +1 for NUL.
    let match_out = unsafe { xmalloc(match_len as size_t + pat_len as size_t + 1) } as *mut c_char;

    let into = match_out.cast::<u8>();
    unsafe { into.copy_from(pat.cast(), pat_len as size_t) };
    if match_len > 0 {
        if lowercase {
            let mword = unsafe { xstrnsave(word, match_len as size_t) };
            let lower = unsafe { strcase_save(mword, false) };
            unsafe { xfree(mword as *mut c_void) };
            unsafe {
                (match_out.offset(pat_len as isize))
                    .cast::<u8>()
                    .copy_from(lower.cast(), match_len as size_t)
            };
            unsafe { xfree(lower as *mut c_void) };
        } else {
            unsafe {
                (match_out.offset(pat_len as isize))
                    .cast::<u8>()
                    .copy_from(word.cast(), match_len as size_t)
            };
        }
    }
    unsafe { *match_out.offset((pat_len + match_len) as isize) = NUL as c_char };
    match_out
}

/// Search for strings matching `pat` in the specified range and return them.
///
/// `dir` is `FORWARD` or `BACKWARD`; `matches` and `numMatches` return the
/// answer.  Returns `Ok` on success, `Err` otherwise.
pub(crate) unsafe fn expand_pattern_in_buf(
    pat: *mut c_char,
    dir: Direction,
    matches: *mut *mut *mut c_char,
    numMatches: *mut c_int,
) -> Result<(), Failed> {
    let exacttext = exacttext();
    let has_range = search_first_line.get() != 0;

    unsafe { *matches = ptr::null_mut() };
    unsafe { *numMatches = 0 };

    if pat.is_null() || unsafe { *pat } as c_int == NUL {
        return Err(Failed);
    }

    let pat_len = unsafe { cstr::bytes_at(pat) }.len() as c_int;
    let mut cur_match_pos: pos_T = unsafe { core::mem::zeroed() };
    let mut prev_match_pos: pos_T = unsafe { core::mem::zeroed() };
    if has_range {
        cur_match_pos.lnum = search_first_line.get();
    } else {
        cur_match_pos = pre_incsearch_pos.get();
    }

    let search_flags = SEARCH_OPT
        | SEARCH_NOOF
        | SEARCH_PEEK
        | SEARCH_NFMSG
        | if has_range { SEARCH_START } else { 0 };

    // The matches found so far, in the order they were found.
    let mut found = Vec::<CString>::new();

    let mut end_match_pos: pos_T = unsafe { core::mem::zeroed() };
    let mut word_end_pos: pos_T = unsafe { core::mem::zeroed() };
    let mut looped_around = false;
    let mut compl_started = false;

    // False is C's `goto cleanup`: the user interrupted, so the matches
    // collected so far are thrown away.
    let completed = 'search: {
        loop {
            let quiet = Suppress::output();
            let found_new_match = unsafe {
                searchit(
                    None,
                    Buf::current(),
                    &raw mut cur_match_pos,
                    &raw mut end_match_pos,
                    dir,
                    pat,
                    pat_len as size_t,
                    1,
                    search_flags,
                    RE_LAST,
                    ptr::null_mut(),
                )
            };
            drop(quiet);

            if found_new_match == FAIL {
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
                break 'search false;
            }

            // searchit() can return line number +1 past the last line when
            // searching for "foo\n" if "foo" is at end of buffer.
            if end_match_pos.lnum > unsafe { (*curbuf.get()).b_ml.ml_line_count } {
                cur_match_pos.lnum = 1;
                cur_match_pos.col = 0;
                cur_match_pos.coladd = 0;
                continue;
            }

            // Extract the matching text prepended to the completed word.
            let mut full_match = ptr::null_mut();
            if unsafe {
                copy_substring_from_pos(
                    &raw mut cur_match_pos,
                    &raw mut end_match_pos,
                    &raw mut full_match,
                    &raw mut word_end_pos,
                )
            }
            .is_err()
            {
                break;
            }

            let mut match_out;
            if exacttext {
                match_out = full_match;
            } else {
                // Construct a new match from the completed word appended
                // to the pattern itself.
                match_out = unsafe {
                    concat_pattern_with_buffer_match(pat, pat_len, &raw mut end_match_pos, false)
                };

                // The regex pattern may include '\C' or '\c'.  First try
                // matching the buffer word as-is; if it doesn't match, try
                // again with the lowercase version of the word to handle
                // smartcase behaviour.
                if !unsafe { is_regex_match(match_out, full_match) } {
                    unsafe { xfree(match_out as *mut c_void) };
                    match_out = unsafe {
                        concat_pattern_with_buffer_match(pat, pat_len, &raw mut end_match_pos, true)
                    };
                    if !unsafe { is_regex_match(match_out, full_match) } {
                        unsafe { xfree(match_out as *mut c_void) };
                        unsafe { xfree(full_match as *mut c_void) };
                        continue;
                    }
                }
                unsafe { xfree(full_match as *mut c_void) };
            }

            // SAFETY: both producers answer a fresh, owned, NUL-terminated
            // string, and this is the only reference to it.
            let owned = unsafe { CString::from_raw(match_out) };
            // Include this match if it is not a duplicate.
            if !found.contains(&owned) {
                found.push(owned);
                if c_int::try_from(found.len()).is_ok_and(|n| n > TAG_MANY) {
                    break;
                }
            }
            if has_range {
                cur_match_pos = word_end_pos;
            }
        }
        true
    };

    if !completed {
        return Err(Failed);
    }

    unsafe { *numMatches = c_int::try_from(found.len()).expect("a match count fits a c_int") };
    unsafe { *matches = owned_cstr_array(found) };
    Ok(())
}
