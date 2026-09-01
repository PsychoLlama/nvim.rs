//! Command-line completion inside an expression.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::types::CmdIdx;
use core::ffi::{c_char, c_int};

use crate::ascii::{ascii_iswhite, ascii_iswhite_or_nul};
use crate::charset::{skiptowhite, skipwhite};
use crate::winlayer::Live;

use crate::ex_docmd::cmd_has_expr_args;
use crate::mbyte::utf_head_off;
use crate::strings::vim_strchr;
use crate::types::{ExpandContext, NUL, expand_T};
use ::libc::strpbrk;

/// The characters that end the plain-name part of an expression: whatever
/// one of them introduces is what completion should look at instead.
const BREAKS: &core::ffi::CStr = c"\"'+-*/%.=!?~|&$([<>,#";

/// Decide what `xp` should complete for the expression `arg` belongs to.
///
/// # Safety
/// `xp` must be valid and `arg` a NUL-terminated string that outlives it —
/// `xp_pattern` is left pointing into it.
pub(crate) unsafe fn set_context_for_expression(
    xp: *mut expand_T,
    mut arg: *mut c_char,
    cmdidx: CmdIdx,
) {
    // SAFETY: the caller's promise -- `xp` is the live completion context
    // and `arg` outlives it.
    let mut xpand = unsafe { Live::new(xp) };
    let mut got_eq = false;

    if cmdidx == CmdIdx::r#let || cmdidx == CmdIdx::r#const {
        xpand.xp_context = ExpandContext::UserVars;
        if unsafe { strpbrk(arg, BREAKS.as_ptr()) }.is_null() {
            // ":let var1 var2 ...": find the last space.
            let mut p = unsafe { arg.add(cstr::bytes_at(arg).len()) };
            loop {
                xpand.xp_pattern = p;
                // Upstream steps back unconditionally and so reads the
                // byte before `arg` on the last pass; the answer is the
                // same either way, since the loop ends there.
                if p == arg {
                    break;
                }
                p = unsafe { p.sub(utf_head_off(arg, p.sub(1)) as usize + 1) };
                if ascii_iswhite(unsafe { *p } as c_int) {
                    break;
                }
            }
            return;
        }
    } else {
        xpand.xp_context = if cmdidx == CmdIdx::call {
            ExpandContext::Functions
        } else {
            ExpandContext::Expression
        };
    }

    loop {
        xpand.xp_pattern = unsafe { strpbrk(arg, BREAKS.as_ptr()) };
        if xpand.xp_pattern.is_null() {
            break;
        }
        let mut c = unsafe { *xpand.xp_pattern } as u8 as c_int;
        if c == '&' as c_int {
            c = unsafe { *xpand.xp_pattern.add(1) } as u8 as c_int;
            if c == '&' as c_int {
                xpand.xp_pattern = unsafe { xpand.xp_pattern.add(1) };
                xpand.xp_context = if cmdidx != CmdIdx::r#let || got_eq {
                    ExpandContext::Expression
                } else {
                    ExpandContext::Nothing
                };
            } else if c != ' ' as c_int {
                xpand.xp_context = ExpandContext::Settings;
                if (c == 'l' as c_int || c == 'g' as c_int)
                    && unsafe { *xpand.xp_pattern.add(2) } == b':' as c_char
                {
                    xpand.xp_pattern = unsafe { xpand.xp_pattern.add(2) };
                }
            }
        } else if c == '$' as c_int {
            // environment variable
            xpand.xp_context = ExpandContext::EnvVars;
        } else if c == '=' as c_int {
            got_eq = true;
            xpand.xp_context = ExpandContext::Expression;
        } else if c == '#' as c_int && xpand.xp_context == ExpandContext::Expression {
            // An autoload function or variable contains '#'.
            break;
        } else if (c == '<' as c_int || c == '#' as c_int)
            && xpand.xp_context == ExpandContext::Functions
            && unsafe { vim_strchr(xpand.xp_pattern, '(' as c_int) }.is_null()
        {
            // A function name can start with "<SNR>" and contain '#'.
            break;
        } else if cmdidx != CmdIdx::r#let || got_eq {
            if c == '"' as c_int {
                // a string
                loop {
                    xpand.xp_pattern = unsafe { xpand.xp_pattern.add(1) };
                    c = unsafe { *xpand.xp_pattern } as u8 as c_int;
                    if c == NUL || c == '"' as c_int {
                        break;
                    }
                    if c == '\\' as c_int && unsafe { *xpand.xp_pattern.add(1) } as c_int != NUL {
                        xpand.xp_pattern = unsafe { xpand.xp_pattern.add(1) };
                    }
                }
                xpand.xp_context = ExpandContext::Nothing;
            } else if c == '\'' as c_int {
                // A literal string; `''` is like stopping and starting
                // one, which this walk gets right by accident.
                loop {
                    xpand.xp_pattern = unsafe { xpand.xp_pattern.add(1) };
                    c = unsafe { *xpand.xp_pattern } as u8 as c_int;
                    if c == NUL || c == '\'' as c_int {
                        break;
                    }
                }
                xpand.xp_context = ExpandContext::Nothing;
            } else if c == '|' as c_int {
                if unsafe { *xpand.xp_pattern.add(1) } == b'|' as c_char {
                    xpand.xp_pattern = unsafe { xpand.xp_pattern.add(1) };
                    xpand.xp_context = ExpandContext::Expression;
                } else {
                    xpand.xp_context = ExpandContext::Commands;
                }
            } else {
                xpand.xp_context = ExpandContext::Expression;
            }
        } else {
            // Nothing that looks valid; expand as an expression anyway.
            xpand.xp_context = ExpandContext::Expression;
        }

        arg = xpand.xp_pattern;
        if unsafe { *arg } as c_int != NUL {
            loop {
                arg = unsafe { arg.add(1) };
                c = unsafe { *arg } as u8 as c_int;
                if c == NUL || (c != ' ' as c_int && c != '\t' as c_int) {
                    break;
                }
            }
        }
    }

    // ":exe one two" completes "two".
    if cmd_has_expr_args(cmdidx) && xpand.xp_context == ExpandContext::Expression {
        loop {
            let n = unsafe { skiptowhite(arg) };
            if n == arg || ascii_iswhite_or_nul(unsafe { *skipwhite(n) } as c_int) {
                break;
            }
            arg = unsafe { skipwhite(n) };
        }
    }
    xpand.xp_pattern = arg;
}
