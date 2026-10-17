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

#![forbid(unsafe_code)]

use super::*;
use crate::change::leader_len_of;
use crate::cstr;
use crate::fileio::Fgets;
use crate::highlight_group::HLF_N;
use crate::mbyte::strnicmp_in;
use crate::types::NUL;
use crate::winlayer::Buf;
use core::ffi::{CStr, c_int};

const FIND_DEFINE: c_int = super::FIND_DEFINE as c_int;
const ACTION_SHOW_ALL: c_int = super::ACTION_SHOW_ALL as c_int;
const LSIZE: usize = super::LSIZE as usize;

/// Whether a match at `startp` is really in code rather than in a
/// comment, for `skip_comments`.
pub(crate) fn match_is_code(line: &CStr, startp: usize) -> bool {
    let bytes = line.to_bytes();
    let at = |i: usize| cstr::byte_at(bytes, i);
    let mut matched = true;
    // A line starting with "# define" is not a comment line.
    let hash_define = at(0) == b'#' && {
        let rest = &bytes[1..];
        rest[skip::white(rest)..].starts_with(b"define")
    };
    if !hash_define && leader_len_of(line, false, true) != 0 {
        matched = false;
    }

    // Also check for a "/*" or "//" before the match, so that a line
    // like "int backwards;  /* normal index */" is skipped when
    // looking for "normal". Note: this does not skip a "/*" that is
    // itself inside a comment.
    let lead = skip::white(bytes);
    if matched || (at(lead) == b'/' && at(lead + 1) == b'*') || at(lead) == b'*' {
        let mut p = 0;
        while at(p) != NUL as u8 && p < startp {
            if matched && at(p) == b'/' {
                let next = at(p + 1);
                if next == b'*' || next == b'/' {
                    matched = false;
                    if next == b'/' {
                        break; // after "//" everything is comment
                    }
                    p += 1;
                }
            } else if !matched && at(p) == b'*' && at(p + 1) == b'/' {
                // A match can be found after "*/".
                matched = true;
                p += 1;
            }
            p += 1;
        }
    }
    matched
}

/// The first `len` bytes of `text`, or fewer when it ends -- or reaches a
/// NUL -- before that: what `strncmp` compares of it.
fn prefix(text: &[u8], len: usize) -> &[u8] {
    cstr::until_nul(&text[..len.min(text.len())])
}

/// Look for a match on `line`, starting the pattern search at offset
/// `from`. `text` is what is being looked for, for the plain comparison.
///
/// Answers the offset where the match starts, or `None`.
pub(crate) fn match_on_line(
    line: &CStr,
    pats: &mut Patterns,
    from: usize,
    text: &[u8],
    whole: bool,
    skip_comments: bool,
) -> Option<usize> {
    let bytes = line.to_bytes();
    let mut p = from;
    let mut define_matched = false;
    if pats.def.regprog.is_some() && vim_regexec(&mut pats.def, line, 0) {
        // The pattern has to be the first identifier after 'define',
        // so skip to it before testing, and don't let the match run
        // past the end of that identifier.
        p = pats.def.ends[0].unwrap_or(0);
        while cstr::byte_at(bytes, p) != NUL as u8
            && !vim_iswordc(c_int::from(cstr::byte_at(bytes, p)))
        {
            p += 1;
        }
        define_matched = true;
    }

    // Don't look for a plain match when a define was asked for and
    // this line did not match the define pattern.
    if !(pats.def.regprog.is_none() || define_matched) {
        return None;
    }

    if define_matched || compl_status_sol() {
        // Compare the first "len" characters with "ptr".
        let rest = bytes.get(p..).unwrap_or_default();
        let startp = p + skip::white(rest);
        let candidate = bytes.get(startp..).unwrap_or_default();
        let len = text.len();
        let matched = if p_ic() {
            strnicmp_in(prefix(candidate, len), prefix(text, len)) == 0
        } else {
            prefix(candidate, len) == prefix(text, len)
        };
        let word_runs_on = vim_iswordc(c_int::from(cstr::byte_at(bytes, startp + len)));
        if matched && !(define_matched && whole && word_runs_on) {
            return Some(startp);
        }
        return None;
    }

    if pats.pat.regprog.is_none() || !vim_regexec(&mut pats.pat, line, p) {
        return None;
    }
    let startp = pats.pat.starts[0].unwrap_or(0);
    // Check that the line is not a comment line, unless a define is
    // what is being looked for.
    if skip_comments && !match_is_code(line, startp) {
        return None;
    }
    Some(startp)
}

/// Print `line`, the line a match was found on, and the lines a definition
/// continues onto -- read from `stream` when the line came from an included
/// file, else from the current buffer. `lnum` is the line's number, and is
/// advanced over the continuation lines.
pub(crate) fn show_pat_in_path(
    line: &[u8],
    kind: c_int,
    did_show: bool,
    action: c_int,
    mut stream: Option<&mut Fgets>,
    lnum: &mut LineNr,
    count: c_int,
) {
    if did_show {
        msg_putchar('\n' as c_int); // cursor below the last one
    } else if msg_silent.get() == 0 {
        gotocmdline(true); // cursor at the status line
    }
    if got_int.get() {
        return; // 'q' typed at the "--more--" message
    }
    let mut line = cstr::until_nul(line).to_vec();
    loop {
        if stream.is_some() {
            // These lines came from fgets(), so strip the newline.
            if line.last() == Some(&b'\n') {
                line.pop();
            }
            if line.last() == Some(&b'\r') {
                line.pop();
            }
        }
        // The last character of the line decides whether a definition
        // continues.
        let last = line.last().copied();
        if action == ACTION_SHOW_ALL {
            msg_str(&cstr::owned(format!("{count:3}: ").as_bytes())); // match nr
            // Highlight the line number.
            msg_str_hl(
                &cstr::owned(format!("{:4}", *lnum).as_bytes()),
                HLF_N,
                false,
            );
            msg_str(c" ");
        }
        cstr::with_terminated(&line, |line| msg_prt_line(line, false));

        // A definition continues until a line that does not end in a
        // backslash.
        if got_int.get() || kind != FIND_DEFINE || last != Some(b'\\') {
            break;
        }

        if let Some(stream) = stream.as_deref_mut() {
            if !stream.line_capped(&mut line, LSIZE) {
                break; // end of file
            }
            // As C reads the line: up to its first NUL.
            let end = cstr::until_nul(&line).len();
            line.truncate(end);
            *lnum += 1;
        } else {
            *lnum += 1;
            if *lnum > Buf::current().b_ml.ml_line_count {
                break;
            }
            line = Lines::current().line(*lnum).to_vec();
        }
        msg_putchar('\n' as c_int);
    }
}
