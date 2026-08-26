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
use crate::types::{IOSIZE, NUL};
use crate::winlayer::Buf;
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
    let mut matched = true;
    // A line starting with "# define" is not a comment line.
    if (unsafe { *line } as c_int != '#' as c_int
        || unsafe { strncmp(skipwhite(line.offset(1)), c"define".as_ptr(), 6) } != 0)
        && unsafe { get_leader_len(line, ptr::null_mut(), false, true) } != 0
    {
        matched = false;
    }

    // Also check for a "/*" or "//" before the match, so that a line
    // like "int backwards;  /* normal index */" is skipped when
    // looking for "normal". Note: this does not skip a "/*" that is
    // itself inside a comment.
    let lead = unsafe { skipwhite(line) };
    if matched
        || (unsafe { *lead } as c_int == '/' as c_int
            && unsafe { *lead.offset(1) } as c_int == '*' as c_int)
        || unsafe { *lead } as c_int == '*' as c_int
    {
        let mut p = line;
        while unsafe { *p } as c_int != NUL && p < startp {
            if matched && unsafe { *p } as c_int == '/' as c_int {
                let next = unsafe { *p.offset(1) } as c_int;
                if next == '*' as c_int || next == '/' as c_int {
                    matched = false;
                    if next == '/' as c_int {
                        break; // after "//" everything is comment
                    }
                    p = unsafe { p.offset(1) };
                }
            } else if !matched
                && unsafe { *p } as c_int == '*' as c_int
                && unsafe { *p.offset(1) } as c_int == '/' as c_int
            {
                // A match can be found after "*/".
                matched = true;
                p = unsafe { p.offset(1) };
            }
            p = unsafe { p.offset(1) };
        }
    }
    matched
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
    let mut p = from;
    let mut define_matched = false;
    if !pats.def.regprog.is_null() && unsafe { vim_regexec(&raw mut pats.def, line, 0) } {
        // The pattern has to be the first identifier after 'define',
        // so skip to it before testing, and don't let the match run
        // past the end of that identifier.
        p = pats.def.endp[0];
        while unsafe { *p } as c_int != NUL && !unsafe { vim_iswordc(*p as u8 as c_int) } {
            p = unsafe { p.offset(1) };
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
        let startp = unsafe { skipwhite(p) };
        let matched = if p_ic.get() != 0 {
            unsafe { mb_strnicmp(startp, ptr, len) == 0 }
        } else {
            unsafe { strncmp(startp, ptr, len) == 0 }
        };
        if matched
            && !(define_matched && whole && unsafe { vim_iswordc(*startp.add(len) as u8 as c_int) })
        {
            return Some(startp);
        }
        return None;
    }

    if pats.pat.regprog.is_null()
        || !unsafe { vim_regexec(&raw mut pats.pat, line, p.offset_from(line) as colnr_T) }
    {
        return None;
    }
    let startp = pats.pat.startp[0];
    // Check that the line is not a comment line, unless a define is
    // what is being looked for.
    if skip_comments && !unsafe { match_is_code(line, startp) } {
        return None;
    }
    Some(startp)
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
    // The match-number prefix; upstream shares `IObuff`, which the message
    // machinery below writes again.
    let mut num = [0 as c_char; IOSIZE as usize];
    if did_show {
        unsafe { msg_putchar('\n' as c_int) }; // cursor below the last one
    } else if msg_silent.get() == 0 {
        unsafe { gotocmdline(true) }; // cursor at the status line
    }
    if got_int.get() {
        return; // 'q' typed at the "--more--" message
    }
    let mut line = line;
    let mut linelen = unsafe { strlen(line) };
    loop {
        // `p` ends up on the last character of the line, which is
        // what decides whether a definition continues.
        let mut p = unsafe { line.add(linelen).offset(-1) };
        if !fp.is_null() {
            // These lines came from fgets(), so strip the newline.
            if p >= line && unsafe { *p } as c_int == '\n' as c_int {
                p = unsafe { p.offset(-1) };
            }
            if p >= line && unsafe { *p } as c_int == '\r' as c_int {
                p = unsafe { p.offset(-1) };
            }
            unsafe { *p.offset(1) = NUL as c_char };
        }
        if action == ACTION_SHOW_ALL {
            let iobuff = num.as_mut_ptr();
            unsafe { snprintf(iobuff, IOSIZE as size_t, c"%3d: ".as_ptr(), count) }; // match nr
            unsafe { msg_puts(iobuff) };
            unsafe { snprintf(iobuff, IOSIZE as size_t, c"%4ld".as_ptr(), *lnum as int64_t) };
            unsafe { msg_puts_hl(iobuff, HLF_N, false) }; // highlight the line number
            unsafe { msg_puts(c" ".as_ptr()) };
        }
        unsafe { msg_prt_line(line, false) };

        // A definition continues until a line that does not end in a
        // backslash.
        if got_int.get()
            || kind != FIND_DEFINE
            || p < line
            || unsafe { *p } as c_int != '\\' as c_int
        {
            break;
        }

        if !fp.is_null() {
            if unsafe { vim_fgets(line, LSIZE as c_int, fp) } {
                break; // end of file
            }
            linelen = unsafe { strlen(line) };
            unsafe { *lnum += 1 };
        } else {
            unsafe { *lnum += 1 };
            if unsafe { *lnum } > cur_buf().b_ml.ml_line_count {
                break;
            }
            line = unsafe { ml_get(*lnum) };
            linelen = unsafe { ml_get_len(*lnum) } as size_t;
        }
        unsafe { msg_putchar('\n' as c_int) };
    }
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}
