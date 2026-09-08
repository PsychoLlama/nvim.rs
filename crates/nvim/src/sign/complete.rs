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
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::cstr;
use crate::narrow::number_as_int;
use crate::strings::vim_strchr;
use crate::types::ExpandContext;

/// What [`get_sign_name`] should enumerate.
#[derive(Copy, Clone, PartialEq, Eq)]
enum ExpandWhat {
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
/// an index: the `Expand` it also passes carries the *other* completions'
/// context, not this one.
static EXPAND_WHAT: GlobalCell<ExpandWhat> = GlobalCell::new(ExpandWhat::Subcmd);

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
/// None; `expand` is unused.
pub(crate) unsafe fn get_sign_name(_expand: *mut Expand, idx: c_int) -> *mut c_char {
    match EXPAND_WHAT.get() {
        ExpandWhat::Subcmd => nth(&CMDS, idx),
        ExpandWhat::Define => nth(
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
        ExpandWhat::Place => nth(
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
        ExpandWhat::List | ExpandWhat::Unplace => nth(&[c"group=", c"file=", c"buffer="], idx),
        ExpandWhat::SignNames => sign_nth_name(at(idx)),
        ExpandWhat::SignGroups => match sign_nth_group(at(idx)).map(number_as_int) {
            Some(ns) => describe_ns(ns, c"".as_ptr()).cast_mut(),
            None => ::core::ptr::null_mut(),
        },
        ExpandWhat::Nothing => ::core::ptr::null_mut(),
    }
}

/// Works out what the word at the end of a `:sign` command line is, and
/// points `expand` at it.
///
/// The line is scanned to its last whitespace-separated word; whether that
/// word contains an `=` decides between completing an argument *name* and
/// completing its *value*, and the subcommand decides which list either one
/// comes from. Values with a completion of their own — highlight groups,
/// files, buffers — are handed off through `xp_context` instead.
///
/// # Safety
/// `expand` must be live and `arg` a writable NUL-terminated string
/// ([`sign_cmd_idx`] terminates the subcommand in place).
pub(crate) unsafe fn set_context_in_sign_cmd(expand: *mut Expand, arg: *mut c_char) {
    // SAFETY: the caller's completion context and command line.
    // Default: expand subcommand names.
    unsafe { (*expand).xp_context = ExpandContext::Sign };
    EXPAND_WHAT.set(ExpandWhat::Subcmd);
    unsafe { (*expand).xp_pattern = arg };

    let end_subcmd = unsafe { skiptowhite(arg) };
    if unsafe { *end_subcmd } == 0 {
        // `:sign {subcmd}<CTRL-D>`, still on the subcommand itself.
        return;
    }

    let cmd_idx = unsafe { sign_cmd_idx(arg, end_subcmd) };
    let begin_subcmd_args = unsafe { skipwhite(end_subcmd) };

    // Walk to the last word of the line.
    let mut last;
    let mut p = begin_subcmd_args;
    loop {
        p = unsafe { skipwhite(p) };
        last = p;
        p = unsafe { skiptowhite(p) };
        if unsafe { *p } == 0 {
            break;
        }
    }

    let eq = unsafe { vim_strchr(last, '=' as c_int) };
    if eq.is_null() {
        // Before the `=`: an argument name, or whatever the subcommand
        // takes instead of one.
        unsafe { (*expand).xp_pattern = last };
        EXPAND_WHAT.set(match cmd_idx {
            SIGNCMD_DEFINE => ExpandWhat::Define,
            // `:sign place {id} ...` places and takes the full argument
            // list; `:sign place ...` lists and takes the short one.
            SIGNCMD_PLACE if ascii_isdigit(c_int::from(unsafe { *begin_subcmd_args })) => {
                ExpandWhat::Place
            }
            SIGNCMD_PLACE => ExpandWhat::List,
            SIGNCMD_LIST | SIGNCMD_UNDEFINE => ExpandWhat::SignNames,
            SIGNCMD_JUMP | SIGNCMD_UNPLACE => ExpandWhat::Unplace,
            _ => {
                unsafe { (*expand).xp_context = ExpandContext::Nothing };
                ExpandWhat::Nothing
            }
        });
        return;
    }

    // After the `=`: the argument's value.
    unsafe { (*expand).xp_pattern = eq.add(1) };
    let starts = |lit: &CStr| unsafe { cstr::prefix_eq(last, lit.as_ptr(), lit.count_bytes()) };
    match cmd_idx {
        SIGNCMD_DEFINE => {
            if starts(c"texthl") || starts(c"linehl") || starts(c"culhl") || starts(c"numhl") {
                unsafe { (*expand).xp_context = ExpandContext::Highlight };
            } else if starts(c"icon") {
                unsafe { (*expand).xp_context = ExpandContext::Files };
            } else {
                unsafe { (*expand).xp_context = ExpandContext::Nothing };
            }
        }
        SIGNCMD_PLACE => {
            if starts(c"name") {
                EXPAND_WHAT.set(ExpandWhat::SignNames);
            } else if starts(c"group") {
                EXPAND_WHAT.set(ExpandWhat::SignGroups);
            } else if starts(c"file") {
                unsafe { (*expand).xp_context = ExpandContext::Buffers };
            } else {
                unsafe { (*expand).xp_context = ExpandContext::Nothing };
            }
        }
        SIGNCMD_UNPLACE | SIGNCMD_JUMP => {
            if starts(c"group") {
                EXPAND_WHAT.set(ExpandWhat::SignGroups);
            } else if starts(c"file") {
                unsafe { (*expand).xp_context = ExpandContext::Buffers };
            } else {
                unsafe { (*expand).xp_context = ExpandContext::Nothing };
            }
        }
        _ => unsafe { (*expand).xp_context = ExpandContext::Nothing },
    }
}
