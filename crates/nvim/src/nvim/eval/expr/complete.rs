//! Command-line completion inside an expression.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};

use crate::src::nvim::ascii::{ascii_iswhite, ascii_iswhite_or_nul};
use crate::src::nvim::charset::{skiptowhite, skipwhite};
use crate::src::nvim::eval::{
    EXPAND_COMMANDS, EXPAND_ENV_VARS, EXPAND_EXPRESSION, EXPAND_FUNCTIONS, EXPAND_NOTHING,
    EXPAND_SETTINGS, EXPAND_USER_VARS, NUL,
};
use crate::src::nvim::ex_docmd::cmd_has_expr_args;
use crate::src::nvim::mbyte::utf_head_off;
use crate::src::nvim::os::libc::{strlen, strpbrk};
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::types::{CMD_call, CMD_const, CMD_let, cmdidx_T, expand_T};

/// The characters that end the plain-name part of an expression: whatever
/// one of them introduces is what completion should look at instead.
const BREAKS: &core::ffi::CStr = c"\"'+-*/%.=!?~|&$([<>,#";

/// Decide what `xp` should complete for the expression `arg` belongs to.
///
/// # Safety
/// `xp` must be valid and `arg` a NUL-terminated string that outlives it —
/// `xp_pattern` is left pointing into it.
pub unsafe fn set_context_for_expression(
    xp: *mut expand_T,
    mut arg: *mut c_char,
    cmdidx: cmdidx_T,
) {
    unsafe {
        let mut got_eq = false;

        if cmdidx == CMD_let || cmdidx == CMD_const {
            (*xp).xp_context = EXPAND_USER_VARS;
            if strpbrk(arg, BREAKS.as_ptr()).is_null() {
                // ":let var1 var2 ...": find the last space.
                let mut p = arg.add(strlen(arg) as usize);
                loop {
                    (*xp).xp_pattern = p;
                    // Upstream steps back unconditionally and so reads the
                    // byte before `arg` on the last pass; the answer is the
                    // same either way, since the loop ends there.
                    if p == arg {
                        break;
                    }
                    p = p.sub(utf_head_off(arg, p.sub(1)) as usize + 1);
                    if ascii_iswhite(*p as c_int) {
                        break;
                    }
                }
                return;
            }
        } else {
            (*xp).xp_context = if cmdidx == CMD_call {
                EXPAND_FUNCTIONS
            } else {
                EXPAND_EXPRESSION
            };
        }

        loop {
            (*xp).xp_pattern = strpbrk(arg, BREAKS.as_ptr());
            if (*xp).xp_pattern.is_null() {
                break;
            }
            let mut c = *(*xp).xp_pattern as u8 as c_int;
            if c == '&' as c_int {
                c = *(*xp).xp_pattern.add(1) as u8 as c_int;
                if c == '&' as c_int {
                    (*xp).xp_pattern = (*xp).xp_pattern.add(1);
                    (*xp).xp_context = if cmdidx != CMD_let || got_eq {
                        EXPAND_EXPRESSION
                    } else {
                        EXPAND_NOTHING
                    };
                } else if c != ' ' as c_int {
                    (*xp).xp_context = EXPAND_SETTINGS;
                    if (c == 'l' as c_int || c == 'g' as c_int)
                        && *(*xp).xp_pattern.add(2) == b':' as c_char
                    {
                        (*xp).xp_pattern = (*xp).xp_pattern.add(2);
                    }
                }
            } else if c == '$' as c_int {
                // environment variable
                (*xp).xp_context = EXPAND_ENV_VARS;
            } else if c == '=' as c_int {
                got_eq = true;
                (*xp).xp_context = EXPAND_EXPRESSION;
            } else if c == '#' as c_int && (*xp).xp_context == EXPAND_EXPRESSION {
                // An autoload function or variable contains '#'.
                break;
            } else if (c == '<' as c_int || c == '#' as c_int)
                && (*xp).xp_context == EXPAND_FUNCTIONS
                && vim_strchr((*xp).xp_pattern, '(' as c_int).is_null()
            {
                // A function name can start with "<SNR>" and contain '#'.
                break;
            } else if cmdidx != CMD_let || got_eq {
                if c == '"' as c_int {
                    // a string
                    loop {
                        (*xp).xp_pattern = (*xp).xp_pattern.add(1);
                        c = *(*xp).xp_pattern as u8 as c_int;
                        if c == NUL || c == '"' as c_int {
                            break;
                        }
                        if c == '\\' as c_int && *(*xp).xp_pattern.add(1) as c_int != NUL {
                            (*xp).xp_pattern = (*xp).xp_pattern.add(1);
                        }
                    }
                    (*xp).xp_context = EXPAND_NOTHING;
                } else if c == '\'' as c_int {
                    // A literal string; `''` is like stopping and starting
                    // one, which this walk gets right by accident.
                    loop {
                        (*xp).xp_pattern = (*xp).xp_pattern.add(1);
                        c = *(*xp).xp_pattern as u8 as c_int;
                        if c == NUL || c == '\'' as c_int {
                            break;
                        }
                    }
                    (*xp).xp_context = EXPAND_NOTHING;
                } else if c == '|' as c_int {
                    if *(*xp).xp_pattern.add(1) == b'|' as c_char {
                        (*xp).xp_pattern = (*xp).xp_pattern.add(1);
                        (*xp).xp_context = EXPAND_EXPRESSION;
                    } else {
                        (*xp).xp_context = EXPAND_COMMANDS;
                    }
                } else {
                    (*xp).xp_context = EXPAND_EXPRESSION;
                }
            } else {
                // Nothing that looks valid; expand as an expression anyway.
                (*xp).xp_context = EXPAND_EXPRESSION;
            }

            arg = (*xp).xp_pattern;
            if *arg as c_int != NUL {
                loop {
                    arg = arg.add(1);
                    c = *arg as u8 as c_int;
                    if c == NUL || (c != ' ' as c_int && c != '\t' as c_int) {
                        break;
                    }
                }
            }
        }

        // ":exe one two" completes "two".
        if cmd_has_expr_args(cmdidx) && (*xp).xp_context == EXPAND_EXPRESSION {
            loop {
                let n = skiptowhite(arg);
                if n == arg || ascii_iswhite_or_nul(*skipwhite(n) as c_int) {
                    break;
                }
                arg = skipwhite(n);
            }
        }
        (*xp).xp_pattern = arg;
    }
}
