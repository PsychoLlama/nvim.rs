//! Command-line completion for `:sign`.
//!
//! [`set_context_in_sign_cmd`] decides, from how much of the line has been
//! typed, which of seven things the word under the cursor is: a subcommand,
//! an argument name for one of the four subcommands that take them, a
//! defined sign name, a placed sign group, or something with a completion of
//! its own (a highlight group, a file, a buffer). [`get_sign_name`] is the
//! `expand_generic` callback that then enumerates whichever list that answer
//! named.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::narrow::number_as_int;
use crate::types::ExpandContext;

/// What [`get_sign_name`] should enumerate.
#[derive(Copy, Clone, PartialEq, Eq)]
enum Expand {
    /// `:sign {subcmd}`.
    Subcmd,
    /// `:sign define {name} {args}...`.
    Define,
    /// `:sign place {id} {args}...`.
    Place,
    /// `:sign place {args}...` — the listing form, which takes fewer.
    List,
    /// `:sign unplace {args}...` and `:sign jump {args}...`.
    Unplace,
    /// The name of a defined sign.
    SignNames,
    /// The name of a sign group that has had a sign placed in it.
    SignGroups,
    /// Nothing — `xp_context` carries the real answer.
    Nothing,
}

/// What the last [`set_context_in_sign_cmd`] decided.
///
/// A static, because `expand_generic` calls [`get_sign_name`] with nothing but
/// an index: the `expand_T` it also passes carries the *other* completions'
/// context, not this one.
static EXPAND_WHAT: GlobalCell<Expand> = GlobalCell::new(Expand::Subcmd);

/// `expand_generic`'s index as a list position; a negative one is 0, which
/// is what `idx.max(0)` said before the completion lists were slices.
fn at(idx: c_int) -> usize {
    usize::try_from(idx).unwrap_or(0)
}

/// The `idx`'th element of a completion list, or null past its end.
///
/// `expand_generic` walks upwards until it gets a null, which is what the
/// NULL terminator on each of these arrays upstream is for.
fn nth(list: &[&CStr], idx: c_int) -> *mut c_char {
    usize::try_from(idx)
        .ok()
        .and_then(|i| list.get(i))
        .map_or(::core::ptr::null_mut(), |s| s.as_ptr().cast_mut())
}

/// The `expand_generic` callback: the `idx`'th completion of whatever
/// [`set_context_in_sign_cmd`] decided this `:sign` line wants.
///
/// # Safety
/// None; `xp` is unused.
pub(crate) unsafe fn get_sign_name(_xp: *mut expand_T, idx: c_int) -> *mut c_char {
    match EXPAND_WHAT.get() {
        Expand::Subcmd => nth(&CMDS, idx),
        Expand::Define => nth(
            &[
                c"culhl=",
                c"icon=",
                c"linehl=",
                c"numhl=",
                c"text=",
                c"texthl=",
                c"priority=",
            ],
            idx,
        ),
        Expand::Place => nth(
            &[
                c"line=",
                c"name=",
                c"group=",
                c"priority=",
                c"file=",
                c"buffer=",
            ],
            idx,
        ),
        // `:sign place` with no id lists rather than places, so it takes
        // neither `line=` nor `name=`; `:sign unplace` and `:sign jump` take
        // the same three.
        Expand::List | Expand::Unplace => nth(&[c"group=", c"file=", c"buffer="], idx),
        Expand::SignNames => sign_nth_name(at(idx)),
        Expand::SignGroups => match sign_nth_group(at(idx)).map(number_as_int) {
            Some(ns) => describe_ns(ns, c"".as_ptr()).cast_mut(),
            None => ::core::ptr::null_mut(),
        },
        Expand::Nothing => ::core::ptr::null_mut(),
    }
}

/// Works out what the word at the end of a `:sign` command line is, and
/// points `xp` at it.
///
/// The line is scanned to its last whitespace-separated word; whether that
/// word contains an `=` decides between completing an argument *name* and
/// completing its *value*, and the subcommand decides which list either one
/// comes from. Values with a completion of their own — highlight groups,
/// files, buffers — are handed off through `xp_context` instead.
///
/// # Safety
/// `xp` must be live and `arg` a writable NUL-terminated string
/// ([`sign_cmd_idx`] terminates the subcommand in place).
pub(crate) unsafe fn set_context_in_sign_cmd(xp: *mut expand_T, arg: *mut c_char) {
    // SAFETY: the caller's completion context and command line.
    unsafe {
        // Default: expand subcommand names.
        (*xp).xp_context = ExpandContext::Sign;
        EXPAND_WHAT.set(Expand::Subcmd);
        (*xp).xp_pattern = arg;

        let end_subcmd = skiptowhite(arg);
        if *end_subcmd == 0 {
            // `:sign {subcmd}<CTRL-D>`, still on the subcommand itself.
            return;
        }

        let cmd_idx = sign_cmd_idx(arg, end_subcmd);
        let begin_subcmd_args = skipwhite(end_subcmd);

        // Walk to the last word of the line.
        let mut last;
        let mut p = begin_subcmd_args;
        loop {
            p = skipwhite(p);
            last = p;
            p = skiptowhite(p);
            if *p == 0 {
                break;
            }
        }

        let eq = vim_strchr(last, '=' as c_int);
        if eq.is_null() {
            // Before the `=`: an argument name, or whatever the subcommand
            // takes instead of one.
            (*xp).xp_pattern = last;
            EXPAND_WHAT.set(match cmd_idx {
                SIGNCMD_DEFINE => Expand::Define,
                // `:sign place {id} ...` places and takes the full argument
                // list; `:sign place ...` lists and takes the short one.
                SIGNCMD_PLACE if ascii_isdigit(c_int::from(*begin_subcmd_args)) => Expand::Place,
                SIGNCMD_PLACE => Expand::List,
                SIGNCMD_LIST | SIGNCMD_UNDEFINE => Expand::SignNames,
                SIGNCMD_JUMP | SIGNCMD_UNPLACE => Expand::Unplace,
                _ => {
                    (*xp).xp_context = ExpandContext::Nothing;
                    Expand::Nothing
                }
            });
            return;
        }

        // After the `=`: the argument's value.
        (*xp).xp_pattern = eq.add(1);
        let starts = |lit: &CStr| strncmp(last, lit.as_ptr(), lit.count_bytes()) == 0;
        match cmd_idx {
            SIGNCMD_DEFINE => {
                if starts(c"texthl") || starts(c"linehl") || starts(c"culhl") || starts(c"numhl") {
                    (*xp).xp_context = ExpandContext::Highlight;
                } else if starts(c"icon") {
                    (*xp).xp_context = ExpandContext::Files;
                } else {
                    (*xp).xp_context = ExpandContext::Nothing;
                }
            }
            SIGNCMD_PLACE => {
                if starts(c"name") {
                    EXPAND_WHAT.set(Expand::SignNames);
                } else if starts(c"group") {
                    EXPAND_WHAT.set(Expand::SignGroups);
                } else if starts(c"file") {
                    (*xp).xp_context = ExpandContext::Buffers;
                } else {
                    (*xp).xp_context = ExpandContext::Nothing;
                }
            }
            SIGNCMD_UNPLACE | SIGNCMD_JUMP => {
                if starts(c"group") {
                    EXPAND_WHAT.set(Expand::SignGroups);
                } else if starts(c"file") {
                    (*xp).xp_context = ExpandContext::Buffers;
                } else {
                    (*xp).xp_context = ExpandContext::Nothing;
                }
            }
            _ => (*xp).xp_context = ExpandContext::Nothing,
        }
    }
}
