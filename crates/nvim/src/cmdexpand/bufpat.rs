//! `:s` and `:g` pattern completion from the buffer's own text.
//!
//! [`expand_pattern_in_buf`] searches the buffer for the pattern being typed
//! and offers what follows each match as a completion, so that `:%s/foo<Tab>`
//! grows into the words that actually occur.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::guard::Suppress;
use crate::types::{FAIL, NUL, OK};
use crate::winlayer::Buf;
use core::ffi::{c_char, c_int, c_uint, c_void};
use core::mem::size_of;
use core::ptr;

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
) -> c_int {
    unsafe {
        let exacttext = exacttext();

        if (*start).lnum > (*end).lnum
            || ((*start).lnum == (*end).lnum && (*start).col >= (*end).col)
        {
            return FAIL; // invalid range
        }

        // A newline, spelled the way `'wildoptions'` wants it: `exacttext`
        // keeps the two-character `\n` a pattern would use.
        let append_newline = |ga: *mut garray_T| {
            if exacttext {
                ga_concat_len(ga, c"\\n".as_ptr(), 2);
            } else {
                ga_append(ga, b'\n');
            }
        };

        // Use a growable string.
        let mut ga = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ptr::null_mut(),
        };
        ga_init(&raw mut ga, 1, 128);

        // Append start line from start->col to end.
        let start_line = ml_get((*start).lnum);
        let start_ptr = start_line.offset((*start).col as isize);
        let is_single_line = (*start).lnum == (*end).lnum;

        let mut segment_len = if is_single_line {
            (*end).col - (*start).col
        } else {
            ml_get_len((*start).lnum) - (*start).col
        };
        ga_grow(&raw mut ga, segment_len + 2);
        ga_concat_len(&raw mut ga, start_ptr, segment_len as size_t);
        if !is_single_line {
            append_newline(&raw mut ga);

            // Append full lines between start and end.
            let mut lnum = (*start).lnum + 1;
            while lnum < (*end).lnum {
                let line = ml_get(lnum);
                let linelen = ml_get_len(lnum);
                ga_grow(&raw mut ga, linelen + 2);
                ga_concat_len(&raw mut ga, line, linelen as size_t);
                append_newline(&raw mut ga);
                lnum += 1;
            }
        }

        // Append partial end line (up to word end).
        let end_line = ml_get((*end).lnum);
        let word_end = find_word_end(end_line.offset((*end).col as isize));
        segment_len = word_end.offset_from(end_line) as c_int;
        ga_grow(&raw mut ga, segment_len);
        let from = if is_single_line { (*end).col } else { 0 };
        ga_concat_len(
            &raw mut ga,
            end_line.offset(from as isize),
            (segment_len - from) as size_t,
        );

        // Null-terminate.
        ga_grow(&raw mut ga, 1);
        ga_append(&raw mut ga, NUL as u8);

        *match_out = ga.ga_data as *mut c_char;
        (*match_end).lnum = (*end).lnum;
        (*match_end).col = segment_len as colnr_T;

        OK
    }
}

/// True if `str` matches the regex pattern `pat`.
///
/// Honours `'ignorecase'` and `'smartcase'` to decide case sensitivity.
pub(crate) unsafe fn is_regex_match(pat: *mut c_char, str: *mut c_char) -> bool {
    unsafe {
        if strcmp(pat, str) == 0 {
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
        regmatch.regprog = vim_regcomp(pat, RE_MAGIC + RE_STRING);
        drop(quiet);

        if regmatch.regprog.is_null() {
            return false;
        }
        regmatch.rm_ic = p_ic.get() != 0;
        if p_ic.get() != 0 && p_scs.get() != 0 {
            regmatch.rm_ic = !pat_has_uppercase(pat);
        }

        let quiet = Suppress::output();
        let result = vim_regexec_nl(&raw mut regmatch, str, 0);
        drop(quiet);

        vim_regfree(regmatch.regprog);
        result
    }
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
    unsafe {
        let line = ml_get((*end_match_pos).lnum);
        let word = line.offset((*end_match_pos).col as isize);
        let word_end = find_word_end(word);
        let match_len = word_end.offset_from(word) as c_int;
        // +1 for NUL.
        let match_out = xmalloc(match_len as size_t + pat_len as size_t + 1) as *mut c_char;

        memmove(
            match_out as *mut c_void,
            pat as *const c_void,
            pat_len as size_t,
        );
        if match_len > 0 {
            if lowercase {
                let mword = xstrnsave(word, match_len as size_t);
                let lower = strcase_save(mword, false);
                xfree(mword as *mut c_void);
                memmove(
                    match_out.offset(pat_len as isize) as *mut c_void,
                    lower as *const c_void,
                    match_len as size_t,
                );
                xfree(lower as *mut c_void);
            } else {
                memmove(
                    match_out.offset(pat_len as isize) as *mut c_void,
                    word as *const c_void,
                    match_len as size_t,
                );
            }
        }
        *match_out.offset((pat_len + match_len) as isize) = NUL as c_char;
        match_out
    }
}

/// Search for strings matching `pat` in the specified range and return them.
///
/// `dir` is `FORWARD` or `BACKWARD`; `matches` and `numMatches` return the
/// answer.  Returns `OK` on success, `FAIL` otherwise.
pub(crate) unsafe fn expand_pattern_in_buf(
    pat: *mut c_char,
    dir: Direction,
    matches: *mut *mut *mut c_char,
    numMatches: *mut c_int,
) -> c_int {
    unsafe {
        let exacttext = exacttext();
        let has_range = search_first_line.get() != 0;

        *matches = ptr::null_mut();
        *numMatches = 0;

        if pat.is_null() || *pat as c_int == NUL {
            return FAIL;
        }

        let pat_len = strlen(pat) as c_int;
        let mut cur_match_pos: pos_T = core::mem::zeroed();
        let mut prev_match_pos: pos_T = core::mem::zeroed();
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

        // A growable array of `char *`.
        let mut ga = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ptr::null_mut(),
        };
        ga_init(&raw mut ga, size_of::<*mut c_char>() as c_int, 10);

        let mut end_match_pos: pos_T = core::mem::zeroed();
        let mut word_end_pos: pos_T = core::mem::zeroed();
        let mut looped_around = false;
        let mut compl_started = false;

        // False is C's `goto cleanup`: the user interrupted, so the matches
        // collected so far are thrown away.
        let completed = 'search: {
            loop {
                let quiet = Suppress::output();
                let found_new_match = searchit(
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
                );
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
                if end_match_pos.lnum > (*curbuf.get()).b_ml.ml_line_count {
                    cur_match_pos.lnum = 1;
                    cur_match_pos.col = 0;
                    cur_match_pos.coladd = 0;
                    continue;
                }

                // Extract the matching text prepended to the completed word.
                let mut full_match = ptr::null_mut();
                if copy_substring_from_pos(
                    &raw mut cur_match_pos,
                    &raw mut end_match_pos,
                    &raw mut full_match,
                    &raw mut word_end_pos,
                ) == FAIL
                {
                    break;
                }

                let mut match_out;
                if exacttext {
                    match_out = full_match;
                } else {
                    // Construct a new match from the completed word appended
                    // to the pattern itself.
                    match_out = concat_pattern_with_buffer_match(
                        pat,
                        pat_len,
                        &raw mut end_match_pos,
                        false,
                    );

                    // The regex pattern may include '\C' or '\c'.  First try
                    // matching the buffer word as-is; if it doesn't match, try
                    // again with the lowercase version of the word to handle
                    // smartcase behaviour.
                    if !is_regex_match(match_out, full_match) {
                        xfree(match_out as *mut c_void);
                        match_out = concat_pattern_with_buffer_match(
                            pat,
                            pat_len,
                            &raw mut end_match_pos,
                            true,
                        );
                        if !is_regex_match(match_out, full_match) {
                            xfree(match_out as *mut c_void);
                            xfree(full_match as *mut c_void);
                            continue;
                        }
                    }
                    xfree(full_match as *mut c_void);
                }

                // Include this match if it is not a duplicate.
                for i in 0..ga.ga_len {
                    if strcmp(
                        match_out,
                        *(ga.ga_data as *mut *mut c_char).offset(i as isize),
                    ) == 0
                    {
                        xfree(match_out as *mut c_void);
                        match_out = ptr::null_mut();
                        break;
                    }
                }
                if !match_out.is_null() {
                    ga_grow(&raw mut ga, 1);
                    (ga.ga_data as *mut *mut c_char)
                        .offset(ga.ga_len as isize)
                        .write(match_out);
                    ga.ga_len += 1;
                    if ga.ga_len > TAG_MANY {
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
            ga_clear_strings(&raw mut ga);
            return FAIL;
        }

        *matches = ga.ga_data as *mut *mut c_char;
        *numMatches = ga.ga_len;
        OK
    }
}
