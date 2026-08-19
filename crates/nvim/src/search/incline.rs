//! One line of an include search: whether it matches, and how it is
//! listed.
//!
//! [`match_on_line`] is the half of
//! [`find_pattern_in_path`](super::find_pattern_in_path) that decides
//! whether a line holds what is being looked for -- which depends on
//! `'define'`, on whether the pattern has to be a whole word, and on
//! whether the match is inside a comment. [`show_pat_in_path`] is how
//! `:ilist` and `:isearch` print it, following a definition onto the
//! lines it is continued over.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::highlight_group::HLF_N;
use crate::types::NUL;
use core::ffi::{c_char, c_int};
use core::ptr;

const FIND_DEFINE: c_int = super::FIND_DEFINE as c_int;
const ACTION_SHOW_ALL: c_int = super::ACTION_SHOW_ALL as c_int;
const LSIZE: usize = super::LSIZE as usize;

/// Whether a match at `startp` is really in code rather than in a
/// comment, for `skip_comments`.
///
/// # Safety
/// `line` and `startp` must address the same NUL-terminated line.
pub(crate) unsafe fn match_is_code(line: *mut c_char, startp: *mut c_char) -> bool {
    unsafe {
        let mut matched = true;
        // A line starting with "# define" is not a comment line.
        if (*line as c_int != '#' as c_int
            || strncmp(skipwhite(line.offset(1)), c"define".as_ptr(), 6) != 0)
            && get_leader_len(line, ptr::null_mut(), false, true) != 0
        {
            matched = false;
        }

        // Also check for a "/*" or "//" before the match, so that a line
        // like "int backwards;  /* normal index */" is skipped when
        // looking for "normal". Note: this does not skip a "/*" that is
        // itself inside a comment.
        let lead = skipwhite(line);
        if matched
            || (*lead as c_int == '/' as c_int && *lead.offset(1) as c_int == '*' as c_int)
            || *lead as c_int == '*' as c_int
        {
            let mut p = line;
            while *p as c_int != NUL && p < startp {
                if matched && *p as c_int == '/' as c_int {
                    let next = *p.offset(1) as c_int;
                    if next == '*' as c_int || next == '/' as c_int {
                        matched = false;
                        if next == '/' as c_int {
                            break; // after "//" everything is comment
                        }
                        p = p.offset(1);
                    }
                } else if !matched
                    && *p as c_int == '*' as c_int
                    && *p.offset(1) as c_int == '/' as c_int
                {
                    // A match can be found after "*/".
                    matched = true;
                    p = p.offset(1);
                }
                p = p.offset(1);
            }
        }
        matched
    }
}

/// Look for a match on the current line, starting the pattern search at
/// `from`.
///
/// Answers where the match starts, or `None`.
///
/// # Safety
/// `ptr` must point at `len` readable bytes; `from` must be inside
/// `walk.line`.
pub(crate) unsafe fn match_on_line(
    line: *mut c_char,
    pats: &mut Patterns,
    from: *mut c_char,
    ptr: *mut c_char,
    len: size_t,
    whole: bool,
    skip_comments: bool,
) -> Option<*mut c_char> {
    unsafe {
        let mut p = from;
        let mut define_matched = false;
        if !pats.def.regprog.is_null() && vim_regexec(&raw mut pats.def, line, 0) {
            // The pattern has to be the first identifier after 'define',
            // so skip to it before testing, and don't let the match run
            // past the end of that identifier.
            p = pats.def.endp[0];
            while *p as c_int != NUL && !vim_iswordc(*p as u8 as c_int) {
                p = p.offset(1);
            }
            define_matched = true;
        }

        // Don't look for a plain match when a define was asked for and
        // this line did not match the define pattern.
        if !(pats.def.regprog.is_null() || define_matched) {
            return None;
        }

        if define_matched || compl_status_sol() {
            // Compare the first "len" characters with "ptr".
            let startp = skipwhite(p);
            let matched = if p_ic.get() != 0 {
                mb_strnicmp(startp, ptr, len) == 0
            } else {
                strncmp(startp, ptr, len) == 0
            };
            if matched && !(define_matched && whole && vim_iswordc(*startp.add(len) as u8 as c_int))
            {
                return Some(startp);
            }
            return None;
        }

        if pats.pat.regprog.is_null()
            || !vim_regexec(&raw mut pats.pat, line, p.offset_from(line) as colnr_T)
        {
            return None;
        }
        let startp = pats.pat.startp[0];
        // Check that the line is not a comment line, unless a define is
        // what is being looked for.
        if skip_comments && !match_is_code(line, startp) {
            return None;
        }
        Some(startp)
    }
}

/// Print the line a match was found on, and the lines a definition
/// continues onto.
///
/// # Safety
/// `line` must be NUL-terminated, `lnum` writable, and `fp` null or an
/// open stream positioned just after `line`.
pub(crate) unsafe fn show_pat_in_path(
    line: *mut c_char,
    kind: c_int,
    did_show: bool,
    action: c_int,
    fp: *mut FILE,
    lnum: *mut linenr_T,
    count: c_int,
) {
    unsafe {
        if did_show {
            msg_putchar('\n' as c_int); // cursor below the last one
        } else if msg_silent.get() == 0 {
            gotocmdline(true); // cursor at the status line
        }
        if got_int.get() {
            return; // 'q' typed at the "--more--" message
        }
        let mut line = line;
        let mut linelen = strlen(line);
        loop {
            // `p` ends up on the last character of the line, which is
            // what decides whether a definition continues.
            let mut p = line.add(linelen).offset(-1);
            if !fp.is_null() {
                // These lines came from fgets(), so strip the newline.
                if p >= line && *p as c_int == '\n' as c_int {
                    p = p.offset(-1);
                }
                if p >= line && *p as c_int == '\r' as c_int {
                    p = p.offset(-1);
                }
                *p.offset(1) = NUL as c_char;
            }
            if action == ACTION_SHOW_ALL {
                let iobuff = IObuff.ptr() as *mut c_char;
                snprintf(iobuff, IOSIZE as size_t, c"%3d: ".as_ptr(), count); // match nr
                msg_puts(iobuff);
                snprintf(iobuff, IOSIZE as size_t, c"%4ld".as_ptr(), *lnum as int64_t);
                msg_puts_hl(iobuff, HLF_N, false); // highlight the line number
                msg_puts(c" ".as_ptr());
            }
            msg_prt_line(line, false);

            // A definition continues until a line that does not end in a
            // backslash.
            if got_int.get() || kind != FIND_DEFINE || p < line || *p as c_int != '\\' as c_int {
                break;
            }

            if !fp.is_null() {
                if vim_fgets(line, LSIZE as c_int, fp) {
                    break; // end of file
                }
                linelen = strlen(line);
                *lnum += 1;
            } else {
                *lnum += 1;
                if *lnum > (*curbuf.get()).b_ml.ml_line_count {
                    break;
                }
                line = ml_get(*lnum);
                linelen = ml_get_len(*lnum) as size_t;
            }
            msg_putchar('\n' as c_int);
        }
    }
}
