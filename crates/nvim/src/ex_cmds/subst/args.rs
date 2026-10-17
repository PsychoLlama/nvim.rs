//! Reading `:s`'s pattern, replacement and flags -- everything that can
//! refuse the command before a single line has been matched.
//!
//! Eight things can end `:substitute` here: a letter used as a delimiter, a
//! `\` that names no previous pattern, a bare `:s` with no previous
//! replacement, the `\n` form that is really a join, a zero or overlarge
//! count, trailing garbage, `eap->skip` (parse only), a 'nomodifiable'
//! buffer, and a pattern the regexp engine will not take.
//!
//! The argument is read by offset in the command line. Upstream writes two
//! NULs into it -- over the delimiter after the pattern and over the one
//! after the replacement -- and so does this; what is handed on is copied
//! out first, because the replacement may be a `\=` expression whose code
//! can run another `:s`.
//!
//! Original: `src/nvim/ex_cmds.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]

use super::{
    SUB_FLAGS, check_regexp_delim, skip_substitute, sub_joining_lines, sub_parse_flags,
    sub_set_replacement,
};
use crate::ascii::ascii_iswhite;
use crate::charset::getdigits_int_at;
use crate::ex_cmds::{INT_MAX, kSubIgnoreCase, kSubMatchCase};
use crate::memory::XString;
use crate::message::emsg;
use crate::message::{e_backslash, e_invcmd, e_modifiable, e_nopresub, e_zerocount};
use crate::message_fmt::msg_bytes;
use crate::option::magic_isset;
use crate::os::cshim::gettext;
use crate::pos::MAXCOL;
use crate::regexp::{RE_LAST, RE_SEARCH, RE_SUBST, regtilde_owned, skip_regexp_in_place};
use crate::search::{SEARCH_HIS, search_regcomp_owned};
use crate::semsg;
use crate::types::CmdIdx;
use crate::types::RegMMatch;
use crate::types::{ExArg, LineNr, NUL};
use crate::winlayer::Buf;
use crate::winlayer::Win;
use core::ffi::{CStr, c_char, c_int};
use std::ffi::CString;

/// What `do_sub` needs from the command line before it can start matching.
pub(super) struct SubSetup {
    /// The replacement text.
    pub sub: XString,
    /// The compiled pattern.
    pub regmatch: RegMMatch,
    /// Was there a closing delimiter, and so a replacement at all?  A
    /// preview without one only highlights what matched.
    pub has_second_delim: bool,
    /// Vi compatibility quirk: repeating with `:s` keeps the cursor in the
    /// last column after a `$`.
    pub endcolumn: bool,
    /// Whether a pattern was given rather than taken from the last one.  The
    /// preview refuses to draw without it.
    pub pat_given: bool,
    /// The user's `g` and `c` flags, which `:&&` puts back afterwards.
    pub save_do_all: bool,
    pub save_do_ask: bool,
}

/// The pattern half of the command line, before any of the checks that can
/// still refuse it.
struct Parsed {
    /// The pattern, copied out of the line. `None` reuses the previous one
    /// (upstream's null `pat`); the `\/`, `\?` and `\&` forms give an empty
    /// one, which also reuses it but counts as given.
    pat: Option<CString>,
    /// None only when `eap->skip` is set and no new pattern was given.
    sub: Option<XString>,
    which_pat: c_int,
    has_second_delim: bool,
    endcolumn: bool,
    /// Where the flags start, in the command line.
    flags_at: usize,
}

/// Split `/pattern/replacement/` off the argument, or take the previous
/// pattern and replacement.
fn read_pattern(args: &mut ExArg, cmdpreview_ns: c_int, keeppatterns: bool) -> Option<Parsed> {
    let mut at = args.line.arg;
    let mut which_pat = if args.cmdidx == CmdIdx::tilde {
        RE_LAST as c_int // use last used regexp
    } else {
        RE_SUBST as c_int // use last substitute regexp
    };

    // A new pattern and substitution?  Not if the argument opens with
    // whitespace, a flag letter or a count -- alphanumerics are not accepted
    // as a separator.
    let first = args.line.byte_at(at);
    let fresh = args.line.byte_at(args.line.cmd) == b's'
        && c_int::from(first) != NUL
        && !ascii_iswhite(c_int::from(first))
        && !b"0123456789cegriIp|\"".contains(&first);
    if !fresh {
        // Use the previous pattern and substitution.
        if args.skip {
            return Some(Parsed {
                pat: None,
                sub: None,
                which_pat,
                has_second_delim: false,
                endcolumn: false,
                flags_at: at,
            });
        }
        let Some(previous) = super::parse::previous_replacement() else {
            // There is no previous command.
            emsg(gettext(e_nopresub));
            return None;
        };
        return Some(Parsed {
            // search_regcomp() will use the previous pattern.
            pat: None,
            sub: Some(previous),
            which_pat,
            has_second_delim: false,
            endcolumn: Win::current().w_curswant == MAXCOL as c_int,
            flags_at: at,
        });
    }

    // The delimiter is handed on as a `char`, so a high byte arrives
    // sign-extended -- which is what indexes the ctype table upstream.
    if check_regexp_delim(c_int::from(first as c_char)).is_err() {
        return None;
    }

    let pat;
    let delimiter;
    let mut has_second_delim = false;
    if first == b'\\' {
        // Undocumented vi feature: "\/sub/" and "\?sub?" use the last
        // search pattern (almost like "//sub/r"), "\&sub&" the last
        // substitute pattern (like "//sub/").
        at += 1;
        let which = args.line.byte_at(at);
        if which == NUL as u8 || !b"/?&".contains(&which) {
            emsg(gettext(e_backslash));
            return None;
        }
        if which != b'&' {
            which_pat = RE_SEARCH as c_int; // use last '/' pattern
        }
        pat = CString::default(); // empty search pattern
        delimiter = c_int::from(which);
        at += 1;
        has_second_delim = true;
    } else {
        // Find the end of the regexp.
        which_pat = RE_LAST as c_int; // use last used regexp
        delimiter = c_int::from(first);
        at += 1;
        let start = at; // remember the start of the search pattern
        // A `?` delimiter's `\?` is unescaped in the line itself, as
        // upstream's `&eap->arg` slot does.
        at = skip_regexp_in_place(
            args.line.buffer_mut(),
            at,
            delimiter,
            c_int::from(magic_isset()),
        );
        if c_int::from(args.line.byte_at(at)) == delimiter {
            // End delimiter found: replace it with a NUL.
            args.line.terminate_at(at);
            at += 1;
            has_second_delim = true;
        }
        pat = args.line.cstr_from(start).to_owned();
    }

    // Small incompatibility: vi sees '\n' as end of the command, but we want
    // to use '\n' to find/substitute a NUL.
    let start = at;
    let sub_end;
    (sub_end, at) = skip_substitute(&mut args.line, at, delimiter);
    let sub = XString::from_bytes(args.line.slice_at(start, sub_end - start));

    if !args.skip && !keeppatterns && cmdpreview_ns <= 0 as c_int {
        sub_set_replacement(sub.clone());
    }

    Some(Parsed {
        pat: Some(pat),
        sub: Some(sub),
        which_pat,
        has_second_delim,
        endcolumn: false,
        flags_at: at,
    })
}

/// Read the trailing count at `*at`, which turns `:s/pat/sub/ N` into a
/// range of N lines starting at the last one, moving `*at` past it.
///
/// Answers false when the count was refused.
fn read_count(args: &mut ExArg, at: &mut usize) -> bool {
    if !args.line.byte_at(*at).is_ascii_digit() {
        return true;
    }
    let count_at = *at;
    let (i, past) = getdigits_int_at(args.line.buffer(), count_at, false, INT_MAX);
    *at = past;
    if i <= 0 as c_int && !args.skip && SUB_FLAGS.with(|flags| flags.do_error) {
        emsg(gettext(e_zerocount));
        return false;
    }
    // Upstream writes `i >= INT_MAX`, which for a `c_int` is `==`.
    if i == INT_MAX {
        let count_arg = msg_bytes(args.line.slice_at(count_at, past - count_at));
        semsg!("E1510: Value too large: {count_arg}");
        return false;
    }
    args.line1 = args.line2;
    args.line2 += i as LineNr - 1 as LineNr;
    args.line2 = args.line2.min(Buf::current().b_ml.ml_line_count);
    true
}

/// Read the whole `:s` command line, answering None when the command is
/// finished -- because it was refused, because it was only being parsed, or
/// because the `\n` form turned it into a join.
pub(super) fn parse_sub(
    args: &mut ExArg,
    cmdpreview_ns: c_int,
    keeppatterns: bool,
) -> Option<SubSetup> {
    let Parsed {
        pat,
        sub,
        mut which_pat,
        has_second_delim,
        endcolumn,
        mut flags_at,
    } = read_pattern(args, cmdpreview_ns, keeppatterns)?;

    if let Some(sub) = sub.as_ref() {
        let joined = sub_joining_lines(
            args,
            pat.as_deref().map(CStr::to_bytes),
            sub,
            flags_at,
            cmdpreview_ns <= 0 as c_int,
            keeppatterns,
        );
        if joined {
            return None;
        }
    }

    // Find the trailing options.  This updates the *static* flags, which is
    // what lets ":&&" and ":s" with no flags reuse them.
    let flags = args.line.rest_of(flags_at);
    flags_at += SUB_FLAGS.with_mut(|state| sub_parse_flags(flags, state, &mut which_pat));
    // Remember the user's "g" and "c" flags for ":&&".
    let (save_do_all, save_do_ask) = SUB_FLAGS.with(|flags| (flags.do_all, flags.do_ask));

    let mut at = args.line.skip_white(flags_at);
    if !read_count(args, &mut at) {
        return None;
    }

    // Check for a trailing command or garbage.
    at = args.line.skip_white(at);
    let next = args.line.byte_at(at);
    if c_int::from(next) != NUL && next != b'"' {
        // Not end-of-line or comment.
        args.line.next = args.line.check_next(at);
        if args.line.next.is_none() {
            let rest = msg_bytes(args.line.rest_of(at));
            semsg!("E488: Trailing characters: {rest}");
            return None;
        }
    }
    if args.skip {
        // Not executing commands, only parsing.
        return None;
    }
    // Upstream asserts here; the `args.skip` return above is why it holds --
    // that is the only path that leaves the replacement unset.
    let sub = sub?;

    // Substitution is not allowed in a non-'modifiable' buffer.
    if !SUB_FLAGS.with(|flags| flags.do_count) && Buf::current().b_p_ma == 0 {
        emsg(gettext(e_modifiable));
        return None;
    }

    let options = if cmdpreview_ns > 0 as c_int {
        0 as c_int
    } else {
        SEARCH_HIS as c_int
    };
    let compiled = search_regcomp_owned(
        pat.as_deref().unwrap_or_default(),
        RE_SUBST as c_int,
        which_pat,
        options,
        false,
    );
    let Some((mut regmatch, _)) = compiled else {
        if SUB_FLAGS.with(|flags| flags.do_error) {
            emsg(gettext(e_invcmd));
        }
        return None;
    };

    // The 'i' or 'I' flag overrules 'ignorecase' and 'smartcase'.
    match SUB_FLAGS.with(|flags| flags.do_ic) {
        kSubIgnoreCase => regmatch.set_ignore_case(true),
        kSubMatchCase => regmatch.set_ignore_case(false),
        _ => {}
    }

    // If the substitute pattern starts with "\=" it is an expression: it is
    // already this command's own copy, which a recursive call cannot free.
    // Otherwise '~' in it stands for the old pattern, which is expanded once
    // here rather than on every match.
    let sub = if sub.starts_with(b"\\=") {
        sub
    } else {
        regtilde_owned(sub, magic_isset(), cmdpreview_ns > 0 as c_int).0
    };

    Some(SubSetup {
        sub,
        regmatch,
        has_second_delim,
        endcolumn,
        pat_given: pat.is_some(),
        save_do_all,
        save_do_ask,
    })
}
