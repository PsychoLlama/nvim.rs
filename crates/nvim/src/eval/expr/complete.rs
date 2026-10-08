//! Command-line completion inside an expression.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::ascii::{ascii_iswhite, ascii_iswhite_or_nul};
use crate::charset::skip;
use crate::ex_docmd::cmd_has_expr_args;
use crate::mbyte::head_off;
use crate::types::{CmdIdx, Expand, ExpandContext};
use core::ffi::c_int;

/// The characters that end the plain-name part of an expression: whatever
/// one of them introduces is what completion should look at instead.
const BREAKS: &core::ffi::CStr = c"\"'+-*/%.=!?~|&$([<>,#";

/// Decide what `expand` should complete for the expression starting at
/// `arg` in the completion's line.
pub(crate) fn set_context_for_expression(expand: &mut Expand, arg: usize, cmdidx: CmdIdx) {
    // The line as it stands while the context is worked out: cut at the
    // cursor.
    let text = expand.line.clone();
    let line = text.as_cstr();
    let t = line.to_bytes();
    let at = |i: usize| t.get(i).copied().unwrap_or(0);
    let breaks = BREAKS.to_bytes();
    // Upstream's `strpbrk(arg, BREAKS)`.
    let find_break = |from: usize| {
        t.get(from..)
            .and_then(|rest| rest.iter().position(|c| breaks.contains(c)))
            .map(|i| from + i)
    };
    let skipwhite = |i: usize| i + skip::white(t.get(i..).unwrap_or_default());
    let skiptowhite = |i: usize| i + skip::to_white(t.get(i..).unwrap_or_default());

    let mut arg = arg;
    let mut got_eq = false;

    if cmdidx == CmdIdx::r#let || cmdidx == CmdIdx::r#const {
        expand.context = ExpandContext::UserVars;
        if find_break(arg).is_none() {
            // ":let var1 var2 ...": find the last space.
            let words = &t[arg..];
            let mut p = t.len();
            loop {
                expand.pattern = p;
                // Upstream steps back unconditionally and so reads the
                // byte before `arg` on the last pass; the answer is the
                // same either way, since the loop ends there.
                if p == arg {
                    break;
                }
                p -= head_off(words, p - 1 - arg) + 1;
                if ascii_iswhite(c_int::from(at(p))) {
                    break;
                }
            }
            return;
        }
    } else {
        expand.context = if cmdidx == CmdIdx::call {
            ExpandContext::Functions
        } else {
            ExpandContext::Expression
        };
    }

    while let Some(mut pat) = find_break(arg) {
        let mut c = at(pat);
        if c == b'&' {
            c = at(pat + 1);
            if c == b'&' {
                pat += 1;
                expand.context = if cmdidx != CmdIdx::r#let || got_eq {
                    ExpandContext::Expression
                } else {
                    ExpandContext::Nothing
                };
            } else if c != b' ' {
                expand.context = ExpandContext::Settings;
                if (c == b'l' || c == b'g') && at(pat + 2) == b':' {
                    pat += 2;
                }
            }
        } else if c == b'$' {
            // environment variable
            expand.context = ExpandContext::EnvVars;
        } else if c == b'=' {
            got_eq = true;
            expand.context = ExpandContext::Expression;
        } else if c == b'#' && expand.context == ExpandContext::Expression {
            // An autoload function or variable contains '#'.
            break;
        } else if (c == b'<' || c == b'#')
            && expand.context == ExpandContext::Functions
            && !t[pat..].contains(&b'(')
        {
            // A function name can start with "<SNR>" and contain '#'.
            break;
        } else if cmdidx != CmdIdx::r#let || got_eq {
            if c == b'"' {
                // a string
                loop {
                    pat += 1;
                    c = at(pat);
                    if c == 0 || c == b'"' {
                        break;
                    }
                    if c == b'\\' && at(pat + 1) != 0 {
                        pat += 1;
                    }
                }
                expand.context = ExpandContext::Nothing;
            } else if c == b'\'' {
                // A literal string; `''` is like stopping and starting
                // one, which this walk gets right by accident.
                loop {
                    pat += 1;
                    c = at(pat);
                    if c == 0 || c == b'\'' {
                        break;
                    }
                }
                expand.context = ExpandContext::Nothing;
            } else if c == b'|' {
                if at(pat + 1) == b'|' {
                    pat += 1;
                    expand.context = ExpandContext::Expression;
                } else {
                    expand.context = ExpandContext::Commands;
                }
            } else {
                expand.context = ExpandContext::Expression;
            }
        } else {
            // Nothing that looks valid; expand as an expression anyway.
            expand.context = ExpandContext::Expression;
        }

        arg = pat;
        if at(arg) != 0 {
            loop {
                arg += 1;
                c = at(arg);
                if c == 0 || (c != b' ' && c != b'\t') {
                    break;
                }
            }
        }
    }

    // ":exe one two" completes "two".
    if cmd_has_expr_args(cmdidx) && expand.context == ExpandContext::Expression {
        loop {
            let n = skiptowhite(arg);
            if n == arg || ascii_iswhite_or_nul(c_int::from(at(skipwhite(n)))) {
                break;
            }
            arg = skipwhite(n);
        }
    }
    expand.pattern = arg;
}
