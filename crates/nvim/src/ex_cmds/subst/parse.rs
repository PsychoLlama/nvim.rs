//! Reading a `:s` command line, and remembering it for the next one.
//!
//! [`skip_substitute`] walks the replacement off the command line without
//! interpreting it (`:s` is its own little language, and the delimiter may
//! be almost any character -- [`check_regexp_delim`] rejects the ones that
//! would be ambiguous), [`sub_parse_flags`] turns the trailing letters into
//! `SubFlags`, and the previous replacement is the `~` text carried from
//! the last `:s` (and through ShaDa). [`sub_joining_lines`] is the
//! `\n`-in-the-pattern case, which joins rather than substitutes.
//!
//! Original: `src/nvim/ex_cmds.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]

use super::do_sub_msg;
use crate::cmdhist::add_to_history;
use crate::ex_cmds::{
    _ISalpha, EXFLAG_LIST, EXFLAG_NR, EXFLAG_PRINT, HIST_SEARCH, SubFlags, kSubHonorOptions,
    kSubIgnoreCase, kSubMatchCase,
};
use crate::ex_cmds::{SUB_NLINES, SUB_NSUBS};
use crate::ex_docmd::ex_may_print;
use crate::global_cell::GlobalCell;
use crate::mbyte::cluster_len;
use crate::memory::XString;
use crate::message::emsg;
use crate::ops::do_join;
use crate::option::magic_isset;
use crate::option::vars::p_gd;
use crate::os::cshim::{ctype_bits, gettext};
use crate::os::time::os_time;
use crate::regexp::{RE_LAST, RE_SUBST, regtilde_owned};
use crate::search::save_re_pat_text;
use crate::shada::ShadaExtra;
use crate::types::{
    AdditionalData, CmdLine, ExArg, Failed, LineNr, NUL, SubReplacementString, Timestamp, size_t,
};
use crate::winlayer::Buf;
use crate::winlayer::Win;
use core::ffi::c_int;
use core::ptr;

/// The previous `:s` replacement: what a bare `:s` repeats and what `~`
/// stands for, and the one ShaDa saves and restores with whatever extra
/// data the file carried for it.
struct LastReplacement {
    text: Option<XString>,
    timestamp: Timestamp,
    extra: Option<ShadaExtra>,
}

static OLD_SUB: GlobalCell<LastReplacement> = GlobalCell::new(LastReplacement {
    text: None,
    timestamp: 0,
    extra: None,
});

/// The `:s` flags in force.  Process-wide, because `:&&` and a bare `:s`
/// reuse the previous command's flags, and because a `\=` replacement can
/// run another `:s` that must not leave its own flags behind (`do_sub` saves
/// and restores this around the expression).
pub(crate) static SUB_FLAGS: GlobalCell<SubFlags> = GlobalCell::new(SubFlags {
    do_all: false,
    do_ask: false,
    do_count: false,
    do_error: true,
    do_print: false,
    do_list: false,
    do_number: false,
    do_ic: kSubHonorOptions,
});

/// Set by `do_sub` when a `:global` is running, so that `global_exe` puts the
/// cursor on the first non-blank once the whole command is done.
pub(crate) static GLOBAL_NEED_BEGINLINE: GlobalCell<bool> = GlobalCell::new(false);

/// The previous replacement as ShaDa writes it out: the text, its
/// timestamp and its extra data, borrowed from the store. The two pointers
/// are null when there is nothing, and live until the next
/// [`sub_set_replacement`] or [`sub_restore_replacement`].
pub fn sub_get_replacement() -> SubReplacementString {
    OLD_SUB.with(|last| SubReplacementString {
        sub: last
            .text
            .as_ref()
            .map_or(ptr::null_mut(), |text| text.as_ptr().cast_mut()),
        timestamp: last.timestamp,
        additional_data: last
            .extra
            .as_ref()
            .map_or(ptr::null_mut::<AdditionalData>(), ShadaExtra::as_ptr),
    })
}

/// The previous replacement's text, copied, for a bare `:s`.
pub(super) fn previous_replacement() -> Option<XString> {
    OLD_SUB.with(|last| last.text.clone())
}

/// Remember `text` as the previous replacement, stamped now.
pub(crate) fn sub_set_replacement(text: XString) {
    let displaced = OLD_SUB.with_mut(|last| {
        core::mem::replace(
            last,
            LastReplacement {
                text: Some(text),
                timestamp: os_time(),
                extra: None,
            },
        )
    });
    drop(displaced);
}

/// Put back the replacement ShaDa read, with its own timestamp and extra
/// data, and expand its `~` once as upstream does on load -- without that,
/// `s//~` does not reach it until a `:s` has run. A `~` with nothing to
/// stand for is dropped from the stored text; an expansion is not stored.
pub(crate) fn sub_restore_replacement(
    text: XString,
    timestamp: Timestamp,
    extra: Option<ShadaExtra>,
) {
    let (expanded, fresh) = regtilde_owned(text.clone(), magic_isset(), false);
    let stored = if fresh { text } else { expanded };
    let displaced = OLD_SUB.with_mut(|last| {
        core::mem::replace(
            last,
            LastReplacement {
                text: Some(stored),
                timestamp,
                extra,
            },
        )
    });
    drop(displaced);
}

/// Recognise `:%s/\n//` and turn it into a join command, which is much more
/// efficient.
///
/// The pattern must be exactly `\n`, the replacement empty, and the flags
/// (everything from `flags_at` on) either absent or one of `g`, `l`, `p`,
/// `#` -- anything else is a real substitution.  `save` says whether to
/// remember the pattern (a preview must not).
///
/// Returns true when `:substitute` was handled as a join, including under
/// `eap->skip`, where nothing is done at all.
pub(crate) fn sub_joining_lines(
    args: &mut ExArg,
    pat: Option<&[u8]>,
    sub: &[u8],
    flags_at: usize,
    save: bool,
    keeppatterns: bool,
) -> bool {
    let flags = args.line.rest_of(flags_at);
    let joins = pat == Some(b"\\n".as_slice())
        && sub.is_empty()
        && matches!(flags, [] | [b'g' | b'l' | b'p' | b'#']);
    let Some(pat) = pat.filter(|_| joins) else {
        return false;
    };
    if args.skip {
        return true;
    }

    Win::current().w_cursor.lnum = args.line1;
    args.flags = match flags.first() {
        Some(b'l') => EXFLAG_LIST,
        Some(b'#') => EXFLAG_NR,
        Some(b'p') => EXFLAG_PRINT,
        _ => args.flags,
    };
    // The number of lines joined is the number of lines in the range, plus
    // one more if this is not the end of the file.
    let joined_lines_count = args.line2 - args.line1
        + 1 as LineNr
        + LineNr::from(args.line2 < Buf::current().b_ml.ml_line_count);
    if joined_lines_count > 1 as LineNr {
        let _ = do_join(joined_lines_count as size_t, false, true, false, true);
        SUB_NSUBS.set(joined_lines_count - 1 as LineNr);
        SUB_NLINES.set(1 as LineNr);
        do_sub_msg(false);
        ex_may_print(args);
    }

    if save {
        if !keeppatterns {
            save_re_pat_text(RE_SUBST as c_int, pat, magic_isset());
        }
        // Put the pattern in the search history.
        add_to_history(HIST_SEARCH as c_int, pat, true, NUL as u8);
    }
    true
}

/// Read `:substitute`'s trailing `{flags}` from `text` into `flags`,
/// answering how many bytes they took.
///
/// A leading `&` keeps the previous flags; otherwise they are reset from
/// 'gdefault' and the defaults first.  `g` and `c` toggle, `r` is never a
/// toggle but redirects `which_pat`, and the first unknown letter ends the
/// run.
pub(crate) fn sub_parse_flags(text: &[u8], flags: &mut SubFlags, which_pat: &mut c_int) -> usize {
    // Find the trailing options.  When '&' is used, keep the old ones.
    let mut i = 0;
    if text.first() == Some(&b'&') {
        i = 1;
    } else {
        flags.do_all = p_gd();
        flags.do_ask = false;
        flags.do_error = true;
        flags.do_print = false;
        flags.do_list = false;
        flags.do_count = false;
        flags.do_number = false;
        flags.do_ic = kSubHonorOptions;
    }
    while i < text.len() {
        // Note that 'g' and 'c' are always inverted, and 'r' never is.
        match text[i] {
            b'g' => flags.do_all = !flags.do_all,
            b'c' => flags.do_ask = !flags.do_ask,
            b'n' => flags.do_count = true,
            b'e' => flags.do_error = !flags.do_error,
            b'r' => *which_pat = RE_LAST as c_int, // use last used regexp
            b'p' => flags.do_print = true,
            b'#' => {
                flags.do_print = true;
                flags.do_number = true;
            }
            b'l' => {
                flags.do_print = true;
                flags.do_list = true;
            }
            b'i' => flags.do_ic = kSubIgnoreCase, // ignore case
            b'I' => flags.do_ic = kSubMatchCase,  // don't ignore case
            _ => break,
        }
        i += 1;
    }
    if flags.do_count {
        flags.do_ask = false;
    }
    i
}

/// Skip the `sub` part of `:s/pat/sub/` that starts at `start` in `line`,
/// where `delimiter` separates the parts, and answer where what follows it
/// starts.
///
/// The closing delimiter is replaced by a NUL in place, so the replacement
/// ends where it should; an unterminated one simply runs to the end.
pub(crate) fn skip_substitute(line: &mut CmdLine, start: usize, delimiter: c_int) -> usize {
    let end = line.end_of(start);
    let mut i = start;
    while i < end {
        let byte = line.byte_at(i);
        if c_int::from(byte) == delimiter {
            // End delimiter found: replace it with a NUL.
            line.terminate_at(i);
            return i + 1;
        }
        if byte == b'\\' && i + 1 < end {
            i += 1; // skip escaped characters
        }
        // A non-NUL byte of the string, so the character is at least one
        // byte long and stops at the terminator.
        i += cluster_len(&line.buffer()[i..end]);
    }
    end
}

/// Reject a delimiter that would make the command ambiguous.
///
/// `c` is a `char` value (`-128..=255`), sign-extended from the command
/// line's byte, which is how the ctype table upstream indexes is read.
pub(crate) fn check_regexp_delim(c: c_int) -> Result<(), Failed> {
    let isalpha = c_int::from(ctype_bits(c as u8)) & _ISalpha as c_int;
    if isalpha != 0 {
        emsg(gettext(
            c"E146: Regular expressions can't be delimited by letters",
        ));
        return Err(Failed);
    }
    Ok(())
}
