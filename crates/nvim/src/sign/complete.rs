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
use crate::charset::skip;
use crate::narrow::number_as_int;
use crate::types::{Candidate, ExpandContext};

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

/// The `idx`'th element of a completion list, or `None` past its end.
fn nth(list: &[&'static CStr], idx: usize) -> Option<Candidate> {
    list.get(idx).map(|&name| Candidate::Borrowed(name))
}

/// The `expand_generic` callback: the `idx`'th completion of whatever
/// [`set_context_in_sign_cmd`] decided this `:sign` line wants.
pub(crate) fn get_sign_name(_expand: &Expand, idx: usize) -> Option<Candidate> {
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
        ExpandWhat::SignNames => sign_nth_name(idx).map(Candidate::Owned),
        ExpandWhat::SignGroups => {
            let ns = number_as_int(sign_nth_group(idx)?);
            // SAFETY: a namespace's name, or the empty literal; NUL-terminated.
            let name = unsafe { CStr::from_ptr(describe_ns(ns, c"".as_ptr())) };
            Some(Candidate::Owned(name.to_owned()))
        }
        ExpandWhat::Nothing => None,
    }
}

/// Works out what the word at the end of a `:sign` command line is, and
/// points `expand` at it.
///
/// The line is scanned from `arg` to its last whitespace-separated word;
/// whether that word contains an `=` decides between completing an argument
/// *name* and completing its *value*, and the subcommand decides which list
/// either one comes from. Values with a completion of their own — highlight
/// groups, files, buffers — are handed off through `expand.context` instead.
pub(crate) fn set_context_in_sign_cmd(expand: &mut Expand, arg: usize) {
    // Default: expand subcommand names.
    expand.context = ExpandContext::Sign;
    EXPAND_WHAT.set(ExpandWhat::Subcmd);
    expand.pattern = arg;

    let line = expand.line_cstr().to_bytes().to_vec();
    let at = |i: usize| line.get(i).copied().unwrap_or(0);
    let skipwhite = |i: usize| i + skip::white(line.get(i..).unwrap_or_default());
    let skiptowhite = |i: usize| i + skip::to_white(line.get(i..).unwrap_or_default());

    let end_subcmd = skiptowhite(arg);
    if at(end_subcmd) == 0 {
        // `:sign {subcmd}<CTRL-D>`, still on the subcommand itself.
        return;
    }

    let subcmd = &line[arg..end_subcmd];
    let cmd_idx = CMDS
        .iter()
        .position(|cmd| cmd.to_bytes() == subcmd)
        .map_or(SIGNCMD_LAST, |i| {
            c_int::try_from(i).expect("six subcommands")
        });
    let begin_subcmd_args = skipwhite(end_subcmd);

    // Walk to the last word of the line.
    let mut last;
    let mut p = begin_subcmd_args;
    loop {
        p = skipwhite(p);
        last = p;
        p = skiptowhite(p);
        if at(p) == 0 {
            break;
        }
    }

    let Some(eq) = line[last..].iter().position(|&c| c == b'=') else {
        // Before the `=`: an argument name, or whatever the subcommand
        // takes instead of one.
        expand.pattern = last;
        EXPAND_WHAT.set(match cmd_idx {
            SIGNCMD_DEFINE => ExpandWhat::Define,
            // `:sign place {id} ...` places and takes the full argument
            // list; `:sign place ...` lists and takes the short one.
            SIGNCMD_PLACE if ascii_isdigit(c_int::from(at(begin_subcmd_args))) => ExpandWhat::Place,
            SIGNCMD_PLACE => ExpandWhat::List,
            SIGNCMD_LIST | SIGNCMD_UNDEFINE => ExpandWhat::SignNames,
            SIGNCMD_JUMP | SIGNCMD_UNPLACE => ExpandWhat::Unplace,
            _ => {
                expand.context = ExpandContext::Nothing;
                ExpandWhat::Nothing
            }
        });
        return;
    };

    // After the `=`: the argument's value.
    expand.pattern = last + eq + 1;
    let starts = |lit: &CStr| line[last..].starts_with(lit.to_bytes());
    match cmd_idx {
        SIGNCMD_DEFINE => {
            if starts(c"texthl") || starts(c"linehl") || starts(c"culhl") || starts(c"numhl") {
                expand.context = ExpandContext::Highlight;
            } else if starts(c"icon") {
                expand.context = ExpandContext::Files;
            } else {
                expand.context = ExpandContext::Nothing;
            }
        }
        SIGNCMD_PLACE => {
            if starts(c"name") {
                EXPAND_WHAT.set(ExpandWhat::SignNames);
            } else if starts(c"group") {
                EXPAND_WHAT.set(ExpandWhat::SignGroups);
            } else if starts(c"file") {
                expand.context = ExpandContext::Buffers;
            } else {
                expand.context = ExpandContext::Nothing;
            }
        }
        SIGNCMD_UNPLACE | SIGNCMD_JUMP => {
            if starts(c"group") {
                EXPAND_WHAT.set(ExpandWhat::SignGroups);
            } else if starts(c"file") {
                expand.context = ExpandContext::Buffers;
            } else {
                expand.context = ExpandContext::Nothing;
            }
        }
        _ => expand.context = ExpandContext::Nothing,
    }
}
