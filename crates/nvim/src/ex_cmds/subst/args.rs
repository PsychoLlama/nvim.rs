//! Reading `:s`'s pattern, replacement and flags -- everything that can
//! refuse the command before a single line has been matched.
//!
//! Eight things can end `:substitute` here: a letter used as a delimiter, a
//! `\` that names no previous pattern, a bare `:s` with no previous
//! replacement, the `\n` form that is really a join, a zero or overlarge
//! count, trailing garbage, `eap->skip` (parse only), a 'nomodifiable'
//! buffer, and a pattern the regexp engine will not take.  Each one frees
//! the replacement text, which is what [`Owned`] is for.
//!
//! Original: `src/nvim/ex_cmds.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use super::{
    check_regexp_delim, old_sub, skip_substitute, sub_joining_lines, sub_parse_flags,
    sub_set_replacement, subflags,
};
use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::charset::{getdigits_int, skipwhite};
use crate::cstr;
use crate::ex_cmds::Owned;
use crate::ex_cmds::{INT_MAX, kSubIgnoreCase, kSubMatchCase};
use crate::ex_docmd::check_nextcmd;
use crate::memory::{xfree, xstrdup};
use crate::message::emsg;
use crate::message::{e_backslash, e_invcmd, e_modifiable, e_nopresub, e_zerocount};
use crate::message_fmt::{c_str, c_str_len};
use crate::option::magic_isset;
use crate::os::cshim::gettext;
use crate::os::time::os_time;
use crate::pos::MAXCOL;
use crate::regexp::{RE_LAST, RE_SEARCH, RE_SUBST, regtilde, skip_regexp_ex};
use crate::search::{SEARCH_HIS, search_regcomp};
use crate::semsg;
use crate::strings::has_char;
use crate::types::CmdIdx;
use crate::types::{AdditionalData, ExArg, LineNr, NUL, RegMMatch, SubReplacementString, size_t};
use crate::winlayer::Buf;
use crate::winlayer::Win;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

/// What `do_sub` needs from the command line before it can start matching.
pub(super) struct SubSetup {
    /// The replacement text.  Owned by the caller from here on.
    pub sub: *mut c_char,
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
    pat: *mut c_char,
    patlen: size_t,
    /// None only when `eap->skip` is set and no new pattern was given.
    sub: Option<Owned>,
    which_pat: c_int,
    has_second_delim: bool,
    endcolumn: bool,
    /// Where the flags start.
    cmd: *mut c_char,
}

/// Split `/pattern/replacement/` off the argument, or take the previous
/// pattern and replacement.
fn read_pattern(args: &mut ExArg, cmdpreview_ns: c_int, keeppatterns: bool) -> Option<Parsed> {
    let mut cmd = args.arg_ptr();
    let mut which_pat = if args.cmdidx == CmdIdx::tilde {
        RE_LAST as c_int // use last used regexp
    } else {
        RE_SUBST as c_int // use last substitute regexp
    };

    // A new pattern and substitution?  Not if the argument opens with
    // whitespace, a flag letter or a count -- alphanumerics are not accepted
    // as a separator.
    // SAFETY: the argument is NUL-terminated.
    let fresh = unsafe {
        args.line.byte_at(args.line.cmd) == b's'
            && *cmd as c_int != NUL
            && !ascii_iswhite(*cmd as c_int)
            && !has_char(c"0123456789cegriIp|\"", *cmd as u8 as c_int)
    };
    if !fresh {
        // Use the previous pattern and substitution.
        if args.skip {
            return Some(Parsed {
                pat: ptr::null_mut(),
                patlen: 0 as size_t,
                sub: None,
                which_pat,
                has_second_delim: false,
                endcolumn: false,
                cmd,
            });
        }
        // SAFETY: `old_sub` holds this module's own allocation.
        let previous = old_sub.get().sub;
        if previous.is_null() {
            // There is no previous command.
            emsg(gettext(e_nopresub));
            return None;
        }
        return Some(Parsed {
            // search_regcomp() will use the previous pattern.
            pat: ptr::null_mut(),
            patlen: 0 as size_t,
            // SAFETY: `previous` is a live C string.
            sub: Some(Owned(unsafe { xstrdup(previous) })),
            which_pat,
            has_second_delim: false,
            // SAFETY: the current window is live.
            endcolumn: Win::current().w_curswant == MAXCOL as c_int,
            cmd,
        });
    }

    // SAFETY: as above.
    if unsafe { check_regexp_delim(*cmd as c_int) }.is_err() {
        return None;
    }

    let pat;
    let patlen;
    let delimiter;
    let mut has_second_delim = false;
    // SAFETY: the argument is NUL-terminated and writable.
    if unsafe { *cmd } as c_int == '\\' as c_int {
        // Undocumented vi feature: "\/sub/" and "\?sub?" use the last
        // search pattern (almost like "//sub/r"), "\&sub&" the last
        // substitute pattern (like "//sub/").
        cmd = unsafe { cmd.add(1) };
        if !has_char(c"/?&", unsafe { *cmd } as u8 as c_int) {
            emsg(gettext(e_backslash));
            return None;
        }
        if unsafe { *cmd } as c_int != '&' as c_int {
            which_pat = RE_SEARCH as c_int; // use last '/' pattern
        }
        pat = c"".as_ptr() as *mut c_char; // empty search pattern
        patlen = 0 as size_t;
        delimiter = unsafe { *cmd } as u8 as c_int;
        cmd = unsafe { cmd.add(1) };
        has_second_delim = true;
    } else {
        // Find the end of the regexp.
        which_pat = RE_LAST as c_int; // use last used regexp
        delimiter = unsafe { *cmd } as u8 as c_int;
        cmd = unsafe { cmd.add(1) };
        pat = cmd; // remember the start of the search pattern
        // `newp` is where the skip would hand back a *copy* of the pattern
        // with a `?` delimiter's `\?` unescaped -- but only when the slot it
        // is given is empty. Upstream passes `&eap->arg`, which never is, so
        // the unescape happens in place and the slot's whole job is to say
        // "do not copy". A null here would rewrite a copy nothing reads and
        // leave the `\?` in the pattern.
        let mut no_copy = pat;
        cmd = unsafe {
            skip_regexp_ex(
                cmd,
                delimiter,
                magic_isset() as c_int,
                &raw mut no_copy,
                ptr::null_mut(),
                ptr::null_mut(),
            )
        };
        if unsafe { *cmd } as c_int == delimiter {
            // End delimiter found: replace it with a NUL.
            unsafe { *cmd = NUL as c_char };
            cmd = unsafe { cmd.add(1) };
            has_second_delim = true;
        }
        patlen = unsafe { cstr::bytes_at(pat) }.len();
    }

    // Small incompatibility: vi sees '\n' as end of the command, but we want
    // to use '\n' to find/substitute a NUL.
    // SAFETY: `cmd` is the start of the substitution, inside the argument.
    let sub = unsafe {
        let start = cmd;
        cmd = skip_substitute(cmd, delimiter);
        Owned(xstrdup(start))
    };

    // SAFETY: `sub.0` is a live copy of the replacement.
    if !args.skip && !keeppatterns && cmdpreview_ns <= 0 as c_int {
        unsafe {
            sub_set_replacement(SubReplacementString {
                sub: xstrdup(sub.0),
                timestamp: os_time(),
                additional_data: ptr::null_mut::<AdditionalData>(),
            })
        };
    }

    Some(Parsed {
        pat,
        patlen,
        sub: Some(sub),
        which_pat,
        has_second_delim,
        endcolumn: false,
        cmd,
    })
}

/// Read the trailing count, which turns `:s/pat/sub/ N` into a range of N
/// lines starting at the last one.
///
/// Answers false when the count was refused.
///
/// # Safety
/// Main thread; `cmd` must point into the live argument.
unsafe fn read_count(args: &mut ExArg, cmd: &mut *mut c_char) -> bool {
    // SAFETY: caller's contract.
    if !ascii_isdigit(unsafe { **cmd } as c_int) {
        return true;
    }
    let count_arg: *const c_char = *cmd;
    // SAFETY: as above; `getdigits_int` advances `cmd` past the digits.
    let i = unsafe { getdigits_int(cmd, false, INT_MAX) };
    let skip = args.skip;
    if i <= 0 as c_int && !skip && subflags.with(|flags| flags.do_error) {
        emsg(gettext(e_zerocount));
        return false;
    }
    // Upstream writes `i >= INT_MAX`, which for a `c_int` is `==`.
    if i == INT_MAX {
        // SAFETY: `count_arg` is the digits just read, `cmd` their end.
        let count_arg = unsafe { c_str_len(count_arg, (*cmd).offset_from(count_arg) as usize) };
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
    let parsed = read_pattern(args, cmdpreview_ns, keeppatterns)?;
    let Parsed {
        pat,
        patlen,
        sub,
        mut which_pat,
        has_second_delim,
        endcolumn,
        mut cmd,
    } = parsed;

    if let Some(sub) = sub.as_ref() {
        // SAFETY: three live C strings.
        let joined = unsafe {
            sub_joining_lines(
                args,
                pat,
                patlen,
                sub.0,
                cmd,
                cmdpreview_ns <= 0 as c_int,
                keeppatterns,
            )
        };
        if joined {
            return None;
        }
    }

    // Find the trailing options.  This updates the *static* flags, which is
    // what lets ":&&" and ":s" with no flags reuse them.
    // SAFETY: `cmd` points into the live argument.
    cmd = subflags.with_mut(|flags| unsafe { sub_parse_flags(cmd, flags, &mut which_pat) });
    // Remember the user's "g" and "c" flags for ":&&".
    let (save_do_all, save_do_ask) = subflags.with(|flags| (flags.do_all, flags.do_ask));

    // SAFETY: `cmd` points into the live argument.
    cmd = unsafe { skipwhite(cmd) };
    // SAFETY: as above.
    if !unsafe { read_count(args, &mut cmd) } {
        return None;
    }

    // Check for a trailing command or garbage.
    // SAFETY: as above.
    cmd = unsafe { skipwhite(cmd) };
    if unsafe { *cmd } as c_int != NUL && unsafe { *cmd } as c_int != '"' as c_int {
        // Not end-of-line or comment.
        // SAFETY: as above.
        args.set_nextcmd_ptr(unsafe { check_nextcmd(cmd) });
        if args.line.next.is_none() {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let cmd = unsafe { c_str(cmd) };
            semsg!("E488: Trailing characters: {cmd}");
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
    if !subflags.with(|flags| flags.do_count) && Buf::current().b_p_ma == 0 {
        emsg(gettext(e_modifiable));
        return None;
    }

    let mut regmatch = RegMMatch::default();
    // SAFETY: `pat` is `patlen` bytes or null, and `regmatch` is ours.
    let compiled = unsafe {
        search_regcomp(
            pat,
            patlen,
            ptr::null_mut(),
            RE_SUBST as c_int,
            which_pat,
            if cmdpreview_ns > 0 as c_int {
                0 as c_int
            } else {
                SEARCH_HIS as c_int
            },
            &raw mut regmatch,
        )
    };
    if compiled.is_err() {
        if subflags.with(|flags| flags.do_error) {
            emsg(gettext(e_invcmd));
        }
        return None;
    }

    // The 'i' or 'I' flag overrules 'ignorecase' and 'smartcase'.
    match subflags.with(|flags| flags.do_ic) {
        kSubIgnoreCase => regmatch.rmm_ic = 1,
        kSubMatchCase => regmatch.rmm_ic = 0,
        _ => {}
    }

    // If the substitute pattern starts with "\=" it is an expression: make a
    // copy, since a recursive call may free it.  Otherwise '~' in it stands
    // for the old pattern, which is expanded once here rather than on every
    // match.
    // SAFETY: `sub.0` is a live C string, and `regtilde` either hands back
    // the same pointer or a fresh allocation that replaces it.
    let sub = unsafe {
        let raw = sub.release();
        if *raw as c_int == '\\' as c_int && *raw.add(1) as c_int == '=' as c_int {
            let copy = xstrdup(raw);
            xfree(raw as *mut c_void);
            copy
        } else {
            let expanded = regtilde(raw, magic_isset() as c_int, cmdpreview_ns > 0 as c_int);
            if expanded != raw {
                xfree(raw as *mut c_void);
            }
            expanded
        }
    };

    Some(SubSetup {
        sub,
        regmatch,
        has_second_delim,
        endcolumn,
        pat_given: !pat.is_null(),
        save_do_all,
        save_do_ask,
    })
}
