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

use super::{
    check_regexp_delim, old_sub, skip_substitute, sub_joining_lines, sub_parse_flags,
    sub_set_replacement, subflags,
};
use crate::ascii::{ascii_isdigit, ascii_iswhite};
use crate::charset::{getdigits_int, skipwhite};
use crate::ex_cmds::{FAIL, INT_MAX, kSubIgnoreCase, kSubMatchCase};
use crate::ex_docmd::check_nextcmd;
use crate::main::{
    curbuf, curwin, e_backslash, e_invcmd, e_modifiable, e_nopresub, e_trailing_arg,
    e_val_too_large_len, e_zerocount,
};
use crate::memory::{xfree, xstrdup};
use crate::message::emsg;
use crate::option::magic_isset;
use crate::os::cshim::gettext;
use crate::os::time::os_time;
use crate::pos::MAXCOL;
use crate::regexp::{RE_LAST, RE_SEARCH, RE_SUBST, regtilde, skip_regexp_ex};
use crate::search::{SEARCH_HIS, search_regcomp};
use crate::semsg_c;
use crate::strings::vim_strchr;
use crate::types::{
    AdditionalData, CMD_tilde, NUL, SubReplacementString, exarg_T, linenr_T, regmmatch_T, size_t,
};
use ::libc::strlen;
use core::ffi::{c_char, c_int, c_void};
use core::mem::ManuallyDrop;
use core::ptr;

/// An `xmalloc`ed C string that frees itself.  Every early exit below is a
/// `goto` that has to release the replacement text, and this is how it
/// cannot be forgotten.
struct Owned(*mut c_char);

impl Drop for Owned {
    fn drop(&mut self) {
        // SAFETY: our own allocation.
        unsafe { xfree(self.0 as *mut c_void) };
    }
}

impl Owned {
    /// Hand the pointer to the caller, who owns it from here on.
    fn release(self) -> *mut c_char {
        ManuallyDrop::new(self).0
    }
}

/// What `do_sub` needs from the command line before it can start matching.
pub(super) struct SubSetup {
    /// The replacement text.  Owned by the caller from here on.
    pub sub: *mut c_char,
    /// The compiled pattern.
    pub regmatch: regmmatch_T,
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
///
/// # Safety
/// Main thread; `eap` must be the live Ex-command argument.
unsafe fn read_pattern(
    eap: *mut exarg_T,
    cmdpreview_ns: c_int,
    keeppatterns: bool,
) -> Option<Parsed> {
    // SAFETY: caller's contract.
    let mut cmd = unsafe { (*eap).arg };
    let mut which_pat = if unsafe { (*eap).cmdidx } as c_int == CMD_tilde as c_int {
        RE_LAST as c_int // use last used regexp
    } else {
        RE_SUBST as c_int // use last substitute regexp
    };

    // A new pattern and substitution?  Not if the argument opens with
    // whitespace, a flag letter or a count -- alphanumerics are not accepted
    // as a separator.
    // SAFETY: the argument is NUL-terminated.
    let fresh = unsafe {
        *(*eap).cmd as u8 == b's'
            && *cmd as c_int != NUL
            && !ascii_iswhite(*cmd as c_int)
            && vim_strchr(c"0123456789cegriIp|\"".as_ptr(), *cmd as u8 as c_int).is_null()
    };
    if !fresh {
        // Use the previous pattern and substitution.
        // SAFETY: caller's contract.
        if unsafe { (*eap).skip } != 0 {
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
            // SAFETY: a live message string.
            unsafe { emsg(gettext(&raw const e_nopresub as *const c_char)) };
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
            endcolumn: unsafe { (*curwin.get()).w_curswant } == MAXCOL as c_int,
            cmd,
        });
    }

    // SAFETY: as above.
    if unsafe { check_regexp_delim(*cmd as c_int) } == FAIL {
        return None;
    }

    let pat;
    let patlen;
    let delimiter;
    let mut has_second_delim = false;
    // SAFETY: the argument is NUL-terminated and writable.
    unsafe {
        if *cmd as c_int == '\\' as c_int {
            // Undocumented vi feature: "\/sub/" and "\?sub?" use the last
            // search pattern (almost like "//sub/r"), "\&sub&" the last
            // substitute pattern (like "//sub/").
            cmd = cmd.add(1);
            if vim_strchr(c"/?&".as_ptr(), *cmd as u8 as c_int).is_null() {
                emsg(gettext(&raw const e_backslash as *const c_char));
                return None;
            }
            if *cmd as c_int != '&' as c_int {
                which_pat = RE_SEARCH as c_int; // use last '/' pattern
            }
            pat = c"".as_ptr() as *mut c_char; // empty search pattern
            patlen = 0 as size_t;
            delimiter = *cmd as u8 as c_int;
            cmd = cmd.add(1);
            has_second_delim = true;
        } else {
            // Find the end of the regexp.
            which_pat = RE_LAST as c_int; // use last used regexp
            delimiter = *cmd as u8 as c_int;
            cmd = cmd.add(1);
            pat = cmd; // remember the start of the search pattern
            cmd = skip_regexp_ex(
                cmd,
                delimiter,
                magic_isset() as c_int,
                &raw mut (*eap).arg,
                ptr::null_mut(),
                ptr::null_mut(),
            );
            if *cmd as c_int == delimiter {
                // End delimiter found: replace it with a NUL.
                *cmd = NUL as c_char;
                cmd = cmd.add(1);
                has_second_delim = true;
            }
            patlen = strlen(pat);
        }
    }

    // Small incompatibility: vi sees '\n' as end of the command, but we want
    // to use '\n' to find/substitute a NUL.
    // SAFETY: `cmd` is the start of the substitution, inside the argument.
    let sub = unsafe {
        let start = cmd;
        cmd = skip_substitute(cmd, delimiter);
        Owned(xstrdup(start))
    };

    // SAFETY: caller's contract; `sub.0` is a live copy of the replacement.
    unsafe {
        if (*eap).skip == 0 && !keeppatterns && cmdpreview_ns <= 0 as c_int {
            sub_set_replacement(SubReplacementString {
                sub: xstrdup(sub.0),
                timestamp: os_time(),
                additional_data: ptr::null_mut::<AdditionalData>(),
            });
        }
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
unsafe fn read_count(eap: *mut exarg_T, cmd: &mut *mut c_char) -> bool {
    // SAFETY: caller's contract.
    if !unsafe { ascii_isdigit(**cmd as c_int) } {
        return true;
    }
    let count_arg: *const c_char = *cmd;
    // SAFETY: as above; `getdigits_int` advances `cmd` past the digits.
    let i = unsafe { getdigits_int(cmd, false, INT_MAX) };
    let skip = unsafe { (*eap).skip } != 0;
    if i <= 0 as c_int && !skip && subflags.with(|flags| flags.do_error) {
        // SAFETY: a live message string.
        unsafe { emsg(gettext(&raw const e_zerocount as *const c_char)) };
        return false;
    }
    // Upstream writes `i >= INT_MAX`, which for a `c_int` is `==`.
    if i == INT_MAX {
        // SAFETY: `count_arg` is the digits just read, `cmd` their end.
        unsafe {
            semsg_c!(
                gettext(&raw const e_val_too_large_len as *const c_char),
                (*cmd).offset_from(count_arg) as c_int,
                count_arg,
            );
        }
        return false;
    }
    // SAFETY: caller's contract; the current buffer is live.
    unsafe {
        (*eap).line1 = (*eap).line2;
        (*eap).line2 += i as linenr_T - 1 as linenr_T;
        (*eap).line2 = (*eap).line2.min((*curbuf.get()).b_ml.ml_line_count);
    }
    true
}

/// Read the whole `:s` command line, answering None when the command is
/// finished -- because it was refused, because it was only being parsed, or
/// because the `\n` form turned it into a join.
///
/// # Safety
/// Main thread; `eap` must be the live Ex-command argument.
pub(super) unsafe fn parse_sub(
    eap: *mut exarg_T,
    cmdpreview_ns: c_int,
    keeppatterns: bool,
) -> Option<SubSetup> {
    // SAFETY: caller's contract.
    let parsed = unsafe { read_pattern(eap, cmdpreview_ns, keeppatterns) }?;
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
                eap,
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
    if !unsafe { read_count(eap, &mut cmd) } {
        return None;
    }

    // Check for a trailing command or garbage.
    // SAFETY: as above.
    unsafe {
        cmd = skipwhite(cmd);
        if *cmd as c_int != NUL && *cmd as c_int != '"' as c_int {
            // Not end-of-line or comment.
            (*eap).nextcmd = check_nextcmd(cmd);
            if (*eap).nextcmd.is_null() {
                semsg_c!(gettext(&raw const e_trailing_arg as *const c_char), cmd);
                return None;
            }
        }
        if (*eap).skip != 0 {
            // Not executing commands, only parsing.
            return None;
        }
    }
    // Upstream asserts here; the `eap->skip` return above is why it holds --
    // that is the only path that leaves the replacement unset.
    let sub = sub?;

    // Substitution is not allowed in a non-'modifiable' buffer.
    // SAFETY: the current buffer is live.
    if !subflags.with(|flags| flags.do_count) && unsafe { (*curbuf.get()).b_p_ma } == 0 {
        // SAFETY: a live message string.
        unsafe { emsg(gettext(&raw const e_modifiable as *const c_char)) };
        return None;
    }

    let mut regmatch = regmmatch_T::default();
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
    if compiled == FAIL {
        if subflags.with(|flags| flags.do_error) {
            // SAFETY: a live message string.
            unsafe { emsg(gettext(&raw const e_invcmd as *const c_char)) };
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
