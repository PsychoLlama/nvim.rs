//! Scanning the rest of a command line: the count, the register, the
//! `!`, the `:p`-style flags, and where the next command begins.
//!
//! Every function here is a walk over the command line the caller owns, so
//! each takes one `unsafe` block for its whole body rather than one per
//! dereference: the obligation is the same for all of them — the pointer is
//! into a NUL-terminated buffer that outlives the call — and stating it once
//! is both cheaper and more honest than stating it forty times.
#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_uint, c_void};
use core::ptr;

use crate::ascii::{ascii_isdigit, ascii_isspace, ascii_iswhite};
use crate::charset::{getdigits_int32, skipdigits, skipwhite};
use crate::eval::skip_expr;
use crate::ex_cmds::skip_vimgrep_pat;
use crate::ex_docmd::onecmd::shift_cmd_args;
use crate::ex_docmd::{
    ADDR_LINES, CPO_BAR, EX_BUFNAME, EX_COUNT, EX_CTRLV, EX_NOTRLCOM, EX_REGSTR, EX_XFILE,
    EX_ZEROR, EXFLAG_LIST, EXFLAG_NR, EXFLAG_PRINT, INT32_MAX, e_zerocount,
};
use crate::keycodes::Ctrl_V;
use crate::main::{curbuf, p_cpo};
use crate::mbyte::utfc_ptr2len;
use crate::memory::xstrdup;
use crate::os::cshim::{gettext, memmove};
use crate::quickfix::grep_internal;
use crate::register::{set_expr_line, valid_yank_reg};
use crate::strings::{del_trailing_spaces, vim_strchr};
use crate::types::ex_cmds::exarg_T;
use crate::types::pos::linenr_T;
use crate::types::{
    CMD_append, CMD_at, CMD_change, CMD_insert, CMD_iput, CMD_lvimgrep, CMD_lvimgrepadd, CMD_put,
    CMD_redir, CMD_smagic, CMD_snomagic, CMD_substitute, CMD_vimgrep, CMD_vimgrepadd, FAIL, NUL,
    OK, size_t, uint32_t,
};
use ::libc::strlen;

/// Step over a run of `:`, which is how a mapping's `:cmd<CR>` and a leading
/// `::::print` both reach the command name.
pub(crate) unsafe fn skip_colon_white(p: *const c_char, skipleadingwhite: bool) -> *mut c_char {
    unsafe {
        let mut p = if skipleadingwhite {
            skipwhite(p)
        } else {
            p as *mut c_char
        };
        while *p as c_int == ':' as c_int {
            p = skipwhite(p.add(1));
        }
        p
    }
}

/// Take the register name a command such as `:delete x` may carry.
///
/// Three tests have to pass before the character is read as a register: the
/// command accepts one, a user command (a negative `cmdidx`) does not take
/// `=`, and a digit belongs to the *count* rather than to a register when
/// the command takes both.
pub(crate) unsafe fn parse_register(eap: *mut exarg_T) {
    unsafe {
        let ea = &mut *eap;
        let is_user_command = (ea.cmdidx as c_int) < 0;
        if ea.argt & EX_REGSTR as uint32_t == 0
            || *ea.arg as c_int == NUL
            || (is_user_command && *ea.arg as c_int == '=' as c_int)
            || (ea.argt & EX_COUNT as uint32_t != 0 && ascii_isdigit(*ea.arg as c_int))
        {
            return;
        }
        // `:put` and `:iput` are the two commands that may name a write-only
        // register; every other one is writing to whichever it names.
        let writing = !is_user_command
            && ea.cmdidx as c_int != CMD_put as c_int
            && ea.cmdidx as c_int != CMD_iput as c_int;
        if !valid_yank_reg(*ea.arg as c_int, writing) {
            return;
        }
        ea.regname = *ea.arg as u8 as c_int;
        ea.arg = ea.arg.add(1);
        // The expression register swallows the rest of the line: it *is* the
        // expression, and evaluating it is deferred until the register is read.
        if ea.regname == '=' as c_int && *ea.arg as c_int != NUL {
            if ea.skip == 0 {
                set_expr_line(xstrdup(ea.arg));
            }
            ea.arg = ea.arg.add(strlen(ea.arg));
        }
        ea.arg = skipwhite(ea.arg);
    }
}

/// Turn a count into a range, which is what a count means for every command
/// that takes one: "this many lines, starting where the range ended".
pub unsafe fn set_cmd_count(eap: *mut exarg_T, count: linenr_T, validate: bool) {
    unsafe {
        let ea = &mut *eap;
        if ea.addr_type as c_uint != ADDR_LINES as c_uint {
            ea.line2 = count;
            if ea.addr_count == 0 {
                ea.addr_count = 1;
            }
            return;
        }
        ea.line1 = ea.line2;
        // Upstream's overflow guard is `line2 >= INT32_MAX - (count - 1)`,
        // and for `count == 0` — which only `nvim_cmd` can supply — the
        // right-hand side itself overflows. The C wraps there, so the
        // comparison always succeeds and the answer is `INT32_MAX`. Spelled
        // as a wrapping subtraction so the debug build does not abort.
        if ea.line2 >= (INT32_MAX as linenr_T).wrapping_sub(count.wrapping_sub(1)) {
            ea.line2 = INT32_MAX as linenr_T;
        } else {
            ea.line2 += count - 1;
        }
        ea.addr_count += 1;
        if validate && ea.line2 > (*curbuf.get()).b_ml.ml_line_count {
            ea.line2 = (*curbuf.get()).b_ml.ml_line_count;
        }
    }
}

/// Take the count a command such as `:delete 3` may carry, and fold it into
/// the range.
pub(crate) unsafe fn parse_count(
    eap: *mut exarg_T,
    errormsg: *mut *const c_char,
    validate: bool,
) -> c_int {
    unsafe {
        let ea = &mut *eap;
        if ea.argt & EX_COUNT as uint32_t == 0 || !ascii_isdigit(*ea.arg as c_int) {
            return OK;
        }
        // A command that also takes a buffer name (`:buffer 2x`) only reads
        // the digits as a count when they are the whole word.
        if ea.argt & EX_BUFNAME as uint32_t != 0 {
            let p = skipdigits(ea.arg.add(1));
            if *p as c_int != NUL && !ascii_iswhite(*p as c_int) {
                return OK;
            }
        }

        let n: linenr_T = getdigits_int32(&raw mut ea.arg, false, INT32_MAX);
        ea.arg = skipwhite(ea.arg);
        if !ea.args.is_null() {
            // `nvim_cmd` supplies the arguments already split, so the count
            // that was just consumed has to come off the first of them.
            debug_assert!(ea.argc > 0 && ea.arg >= *ea.args);
            let first = *ea.args;
            let first_len = *ea.arglens;
            if ea.arg < first.add(first_len) {
                *ea.arglens = first_len.wrapping_sub(ea.arg.offset_from(first) as size_t);
                *ea.args = ea.arg;
            } else {
                shift_cmd_args(eap);
            }
        }
        if n <= 0 && ea.argt & EX_ZEROR as uint32_t == 0 {
            if !errormsg.is_null() {
                *errormsg = gettext(&raw const e_zerocount as *const c_char);
            }
            return FAIL;
        }
        set_cmd_count(eap, n, validate);
        OK
    }
}

/// Take the `!` a command may carry. `:substitute` and its two magic
/// spellings are the exception: there a `!` belongs to the pattern.
pub(crate) unsafe fn parse_bang(eap: *const exarg_T, p: *mut *mut c_char) -> bool {
    unsafe {
        let cmdidx = (*eap).cmdidx as c_int;
        if **p as c_int == '!' as c_int
            && cmdidx != CMD_substitute as c_int
            && cmdidx != CMD_smagic as c_int
            && cmdidx != CMD_snomagic as c_int
        {
            *p = (*p).add(1);
            return true;
        }
        false
    }
}

/// Take the trailing `l`, `p` and `#` flags a printing command may carry.
pub(crate) unsafe fn get_flags(eap: *mut exarg_T) {
    unsafe {
        let ea = &mut *eap;
        loop {
            let flag = match *ea.arg as u8 {
                b'l' => EXFLAG_LIST,
                b'p' => EXFLAG_PRINT,
                b'#' => EXFLAG_NR,
                _ => return,
            };
            ea.flags |= flag;
            ea.arg = skipwhite(ea.arg.add(1));
        }
    }
}

/// Step over a `:vimgrep` pattern, whose delimiters are not the ones the
/// rest of the argument scan knows about.
pub(crate) unsafe fn skip_grep_pat(eap: *mut exarg_T) -> *mut c_char {
    unsafe {
        let ea = &mut *eap;
        let cmdidx = ea.cmdidx as c_int;
        let is_grep = cmdidx == CMD_vimgrep as c_int
            || cmdidx == CMD_lvimgrep as c_int
            || cmdidx == CMD_vimgrepadd as c_int
            || cmdidx == CMD_lvimgrepadd as c_int
            || grep_internal(ea.cmdidx);
        if *ea.arg as c_int == NUL || !is_grep {
            return ea.arg;
        }
        let p = skip_vimgrep_pat(ea.arg, ptr::null_mut(), ptr::null_mut());
        if p.is_null() { ea.arg } else { p }
    }
}

/// Cut the command's argument at the `|`, `"` or newline that ends it, and
/// remember where the next command starts.
///
/// Three characters can end an argument and each has exceptions:
///
/// - CTRL-V escapes the next character for a command that asked for it
///   (`EX_CTRLV`/`EX_XFILE`) and is *removed* for every other command.
/// - `"` starts a comment unless the command takes one literally
///   (`EX_NOTRLCOM`); `:@"` and `:redir @"` name a register with it.
/// - `|` separates commands unless the command reads the following lines
///   (`:append`, `:change`, `:insert`).
///
/// A backslash before one of them escapes it — but only while 'cpoptions'
/// does not contain `b`, or the command does not take CTRL-V escapes.
pub unsafe fn separate_nextcmd(eap: *mut exarg_T) {
    unsafe {
        let ea = &mut *eap;
        let mut p = skip_grep_pat(eap);
        while *p != 0 {
            if *p as c_int == Ctrl_V {
                if ea.argt & (EX_CTRLV as uint32_t | EX_XFILE as uint32_t) != 0 {
                    p = p.add(1);
                } else {
                    drop_one_byte(p);
                }
                if *p as c_int == NUL {
                    break;
                }
            } else if *p as c_int == '`' as c_int
                && *p.add(1) as c_int == '=' as c_int
                && ea.argt & EX_XFILE as uint32_t != 0
            {
                // A backtick-equals expression is stepped over by the
                // evaluator, not by this scan: it may contain any of the
                // ending characters.
                p = p.add(2);
                skip_expr(&raw mut p, ptr::null_mut());
                if *p as c_int == NUL {
                    break;
                }
            } else if ends_argument(ea, p) {
                let escaped = (vim_strchr(p_cpo.get(), CPO_BAR).is_null()
                    || ea.argt & EX_CTRLV as uint32_t == 0)
                    && *p.offset(-1) as c_int == '\\' as c_int;
                if escaped {
                    p = p.offset(-1);
                    drop_one_byte(p);
                } else {
                    ea.nextcmd = check_nextcmd(p);
                    *p = NUL as c_char;
                    break;
                }
            }
            p = p.add(utfc_ptr2len(p) as usize);
        }
        if ea.argt & EX_NOTRLCOM as uint32_t == 0 {
            del_trailing_spaces(ea.arg);
        }
    }
}

/// Does the byte at `p` end the argument? See `separate_nextcmd`.
///
/// A named predicate rather than an inline condition, but deliberately
/// *inside* the loop: the `"` half compares `p` against `eap->arg`, so it
/// depends on where the walk has got to and cannot be hoisted.
unsafe fn ends_argument(ea: &exarg_T, p: *mut c_char) -> bool {
    unsafe {
        let c = *p as c_int;
        let cmdidx = ea.cmdidx as c_int;
        let comment = c == '"' as c_int
            && ea.argt & EX_NOTRLCOM as uint32_t == 0
            && (cmdidx != CMD_at as c_int || p != ea.arg)
            && (cmdidx != CMD_redir as c_int
                || p != ea.arg.add(1)
                || *p.offset(-1) as c_int != '@' as c_int);
        let bar = c == '|' as c_int
            && cmdidx != CMD_append as c_int
            && cmdidx != CMD_change as c_int
            && cmdidx != CMD_insert as c_int;
        comment || bar || c == '\n' as c_int
    }
}

/// Delete the byte at `p` by pulling the terminator-inclusive tail over it.
unsafe fn drop_one_byte(p: *mut c_char) {
    unsafe {
        memmove(
            p.cast(),
            p.add(1).cast::<c_void>(),
            strlen(p.add(1)).wrapping_add(1),
        );
    }
}

/// Step to the end of a whitespace-delimited argument, optionally removing
/// the backslashes that escaped whitespace inside it.
pub unsafe fn skip_cmd_arg(p: *mut c_char, rembs: bool) -> *mut c_char {
    unsafe {
        let mut p = p;
        while *p as c_int != 0 && !ascii_isspace(*p as c_int) {
            if *p as c_int == '\\' as c_int && *p.add(1) as c_int != NUL {
                if rembs {
                    drop_one_byte(p);
                } else {
                    p = p.add(1);
                }
            }
            p = p.add(utfc_ptr2len(p) as usize);
        }
        p
    }
}

/// Does this character end an Ex command? Answers a `c_int` rather than a
/// `bool` because a dozen still-transpiled callers compare it against 0.
pub fn ends_excmd(c: c_int) -> c_int {
    (c == NUL || c == '|' as c_int || c == '"' as c_int || c == '\n' as c_int) as c_int
}

/// The command after the next `|` or newline, or null if there is none.
/// Unlike `check_nextcmd` this searches rather than only looking ahead.
pub unsafe fn find_nextcmd(p: *const c_char) -> *mut c_char {
    unsafe {
        let mut p = p;
        while *p as c_int != '|' as c_int && *p as c_int != '\n' as c_int {
            if *p as c_int == NUL {
                return ptr::null_mut();
            }
            p = p.add(1);
        }
        (p as *mut c_char).add(1)
    }
}

/// The command after `p`, if `p` is at the separator that introduces one.
pub unsafe fn check_nextcmd(p: *mut c_char) -> *mut c_char {
    unsafe {
        let s = skipwhite(p);
        if *s as c_int == '|' as c_int || *s as c_int == '\n' as c_int {
            return s.add(1);
        }
        ptr::null_mut()
    }
}
