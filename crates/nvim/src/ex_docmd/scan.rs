//! Scanning the rest of a command line: the count, the register, the
//! `!`, the `:p`-style flags, and where the next command begins.
//!
//! Every function here is a walk over the command line the caller owns, so
//! each takes one `unsafe` block for its whole body rather than one per
//! dereference: the obligation is the same for all of them — the pointer is
//! into a NUL-terminated buffer that outlives the call — and stating it once
//! is both cheaper and more honest than stating it forty times.
#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::ex_docmd::is_user_cmd;
use crate::types::CmdIdx;
use core::ffi::{c_char, c_int};
use core::ptr;
use std::ffi::CString;

use crate::ascii::{ascii_isdigit, ascii_isspace, ascii_iswhite};

use crate::charset::{getdigits_int32, skipdigits};

use crate::eval::skip_expr;
use crate::ex_cmds::skip_vimgrep_pat;
use crate::ex_docmd::onecmd::shift_cmd_args;
use crate::ex_docmd::{EXFLAG_LIST, EXFLAG_NR, EXFLAG_PRINT, INT32_MAX, e_zerocount, ex_msg};
use crate::keycodes::Ctrl_V;

use crate::memory::xstrdup;
use crate::option::cpo_has;
use crate::quickfix::grep_internal;
use crate::register::{set_expr_line, valid_yank_reg};
use crate::strings::del_trailing_spaces;
use crate::types::ex_cmds::exarg_T;
use crate::types::pos::linenr_T;
use crate::types::{CmdAddr, CpoFlag, ExArgt, Failed, NUL, size_t};
use crate::winlayer::{Buf, Ea};

/// Step over a run of `:`, which is how a mapping's `:cmd<CR>` and a leading
/// `::::print` both reach the command name.
pub(crate) unsafe fn skip_colon_white(p: *const c_char, skipleadingwhite: bool) -> *mut c_char {
    let mut p = if skipleadingwhite {
        skipwhite(p)
    } else {
        p as *mut c_char
    };
    while byte(p) == ':' as c_int {
        p = unsafe { skipwhite(p.add(1)) };
    }
    p
}

/// Take the register name a command such as `:delete x` may carry.
///
/// Three tests have to pass before the character is read as a register: the
/// command accepts one, a user command (a negative `cmdidx`) does not take
/// `=`, and a digit belongs to the *count* rather than to a register when
/// the command takes both.
pub(crate) unsafe fn parse_register(eap: *mut exarg_T) {
    let mut ea = unsafe { Ea::new(eap) };
    let is_user_command = is_user_cmd(ea.cmdidx);
    if !ea.argt.has(ExArgt::REGSTR)
        || byte(ea.arg) == NUL
        || (is_user_command && byte(ea.arg) == '=' as c_int)
        || (ea.argt.has(ExArgt::COUNT) && ascii_isdigit(byte(ea.arg)))
    {
        return;
    }
    // `:put` and `:iput` are the two commands that may name a write-only
    // register; every other one is writing to whichever it names.
    let writing = !is_user_command && ea.cmdidx != CmdIdx::put && ea.cmdidx != CmdIdx::iput;
    if !unsafe { valid_yank_reg(*ea.arg as c_int, writing) } {
        return;
    }
    ea.regname = ubyte(ea.arg) as c_int;
    ea.arg = unsafe { ea.arg.add(1) };
    // The expression register swallows the rest of the line: it *is* the
    // expression, and evaluating it is deferred until the register is read.
    if ea.regname == '=' as c_int && byte(ea.arg) != NUL {
        if ea.skip == 0 {
            unsafe { set_expr_line(xstrdup(ea.arg)) };
        }
        ea.arg = unsafe { ea.arg.add(cstr::bytes_at(ea.arg).len()) };
    }
    ea.arg = skipwhite(ea.arg);
}

/// Turn a count into a range, which is what a count means for every command
/// that takes one: "this many lines, starting where the range ended".
pub unsafe fn set_cmd_count(eap: *mut exarg_T, count: linenr_T, validate: bool) {
    let mut ea = unsafe { Ea::new(eap) };
    if ea.addr_type != CmdAddr::Lines {
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
    if validate && ea.line2 > cur_buf().b_ml.ml_line_count {
        ea.line2 = cur_buf().b_ml.ml_line_count;
    }
}

/// Take the count a command such as `:delete 3` may carry, and fold it into
/// the range.
pub(crate) unsafe fn parse_count(
    eap: *mut exarg_T,
    errormsg: &mut Option<CString>,
    validate: bool,
) -> Result<(), Failed> {
    let mut ea = unsafe { Ea::new(eap) };
    if !ea.argt.has(ExArgt::COUNT) || !ascii_isdigit(byte(ea.arg)) {
        return Ok(());
    }
    // A command that also takes a buffer name (`:buffer 2x`) only reads
    // the digits as a count when they are the whole word.
    if ea.argt.has(ExArgt::BUFNAME) {
        let p = unsafe { skipdigits(ea.arg.add(1)) };
        if byte(p) != NUL && !ascii_iswhite(byte(p)) {
            return Ok(());
        }
    }

    let n: linenr_T = unsafe { getdigits_int32(ea.arg_ptr(), false, INT32_MAX) };
    ea.arg = skipwhite(ea.arg);
    if !ea.args.is_null() {
        // `nvim_cmd` supplies the arguments already split, so the count
        // that was just consumed has to come off the first of them.
        debug_assert!(ea.argc > 0 && ea.arg >= unsafe { *ea.args });
        let first = unsafe { *ea.args };
        let first_len = unsafe { *ea.arglens };
        if ea.arg < unsafe { first.add(first_len) } {
            unsafe { *ea.arglens = first_len.wrapping_sub(ea.arg.offset_from(first) as size_t) };
            unsafe { *ea.args = ea.arg };
        } else {
            shift_cmd_args(ea);
        }
    }
    if n <= 0 && !ea.argt.has(ExArgt::ZEROR) {
        *errormsg = Some(unsafe { ex_msg(e_zerocount.as_ptr()) });
        return Err(Failed);
    }
    unsafe { set_cmd_count(eap, n, validate) };
    Ok(())
}

/// Take the `!` a command may carry. `:substitute` and its two magic
/// spellings are the exception: there a `!` belongs to the pattern.
pub(crate) unsafe fn parse_bang(eap: Ea, p: *mut *mut c_char) -> bool {
    let cmdidx = eap.cmdidx;
    if byte(unsafe { *p }) == '!' as c_int
        && cmdidx != CmdIdx::substitute
        && cmdidx != CmdIdx::smagic
        && cmdidx != CmdIdx::snomagic
    {
        unsafe { *p = (*p).add(1) };
        return true;
    }
    false
}

/// Take the trailing `l`, `p` and `#` flags a printing command may carry.
pub(crate) fn get_flags(mut ea: Ea) {
    loop {
        let flag = match ubyte(ea.arg) {
            b'l' => EXFLAG_LIST,
            b'p' => EXFLAG_PRINT,
            b'#' => EXFLAG_NR,
            _ => return,
        };
        ea.flags |= flag;
        ea.arg = unsafe { skipwhite(ea.arg.add(1)) };
    }
}

/// Step over a `:vimgrep` pattern, whose delimiters are not the ones the
/// rest of the argument scan knows about.
pub(crate) fn skip_grep_pat(mut ea: Ea) -> *mut c_char {
    let cmdidx = ea.cmdidx;
    let is_grep = cmdidx == CmdIdx::vimgrep
        || cmdidx == CmdIdx::lvimgrep
        || cmdidx == CmdIdx::vimgrepadd
        || cmdidx == CmdIdx::lvimgrepadd
        || unsafe { grep_internal(ea.cmdidx) };
    if byte(ea.arg) == NUL || !is_grep {
        return ea.arg;
    }
    let p = unsafe { skip_vimgrep_pat(ea.arg, ptr::null_mut(), ptr::null_mut()) };
    if p.is_null() { ea.arg } else { p }
}

/// Cut the command's argument at the `|`, `"` or newline that ends it, and
/// remember where the next command starts.
///
/// Three characters can end an argument and each has exceptions:
///
/// - CTRL-V escapes the next character for a command that asked for it
///   (`ExArgt::CTRLV`/`ExArgt::XFILE`) and is *removed* for every other command.
/// - `"` starts a comment unless the command takes one literally
///   (`ExArgt::NOTRLCOM`); `:@"` and `:redir @"` name a register with it.
/// - `|` separates commands unless the command reads the following lines
///   (`:append`, `:change`, `:insert`).
///
/// A backslash before one of them escapes it — but only while 'cpoptions'
/// does not contain `b`, or the command does not take CTRL-V escapes.
pub unsafe fn separate_nextcmd(eap: *mut exarg_T) {
    let mut ea = unsafe { Ea::new(eap) };
    let mut p = skip_grep_pat(ea);
    while unsafe { *p } != 0 {
        if byte(p) == Ctrl_V {
            if ea.argt.has(ExArgt::CTRLV | ExArgt::XFILE) {
                p = unsafe { p.add(1) };
            } else {
                drop_one_byte(p);
            }
            if byte(p) == NUL {
                break;
            }
        } else if byte(p) == '`' as c_int
            && byte_at(p, 1) == '=' as c_int
            && ea.argt.has(ExArgt::XFILE)
        {
            // A backtick-equals expression is stepped over by the
            // evaluator, not by this scan: it may contain any of the
            // ending characters.
            p = unsafe { p.add(2) };
            let _ = unsafe { skip_expr(&raw mut p, ptr::null_mut()) };
            if byte(p) == NUL {
                break;
            }
        } else if unsafe { ends_argument(ea, p) } {
            let escaped = (!cpo_has(CpoFlag::BAR) || !ea.argt.has(ExArgt::CTRLV))
                && byte_at(p, -1) == '\\' as c_int;
            if escaped {
                p = unsafe { p.offset(-1) };
                drop_one_byte(p);
            } else {
                ea.nextcmd = unsafe { check_nextcmd(p) };
                unsafe { *p = NUL as c_char };
                break;
            }
        }
        p = unsafe { p.add(utfc_ptr2len(p) as usize) };
    }
    if !ea.argt.has(ExArgt::NOTRLCOM) {
        unsafe { del_trailing_spaces(ea.arg) };
    }
}

/// Does the byte at `p` end the argument? See `separate_nextcmd`.
///
/// A named predicate rather than an inline condition, but deliberately
/// *inside* the loop: the `"` half compares `p` against `eap->arg`, so it
/// depends on where the walk has got to and cannot be hoisted.
unsafe fn ends_argument(ea: Ea, p: *mut c_char) -> bool {
    let c = byte(p);
    let cmdidx = ea.cmdidx;
    let comment = c == '"' as c_int
        && !ea.argt.has(ExArgt::NOTRLCOM)
        && (cmdidx != CmdIdx::at || p != ea.arg)
        && (cmdidx != CmdIdx::redir
            || p != unsafe { ea.arg.add(1) }
            || byte_at(p, -1) != '@' as c_int);
    let bar = c == '|' as c_int
        && cmdidx != CmdIdx::append
        && cmdidx != CmdIdx::change
        && cmdidx != CmdIdx::insert;
    comment || bar || c == '\n' as c_int
}

/// Delete the byte at `p` by pulling the terminator-inclusive tail over it.
fn drop_one_byte(p: *mut c_char) {
    let n_len = unsafe { cstr::bytes_at(p.add(1)) }.len();
    let into = p.cast::<u8>();
    unsafe { into.copy_from(p.add(1).cast(), n_len.wrapping_add(1)) };
}

/// Step to the end of a whitespace-delimited argument, optionally removing
/// the backslashes that escaped whitespace inside it.
pub unsafe fn skip_cmd_arg(p: *mut c_char, rembs: bool) -> *mut c_char {
    let mut p = p;
    while byte(p) != 0 && !ascii_isspace(byte(p)) {
        if byte(p) == '\\' as c_int && byte_at(p, 1) != NUL {
            if rembs {
                drop_one_byte(p);
            } else {
                p = unsafe { p.add(1) };
            }
        }
        p = unsafe { p.add(utfc_ptr2len(p) as usize) };
    }
    p
}

/// Does this character end an Ex command? Answers a `c_int` rather than a
/// `bool` because a dozen still-transpiled callers compare it against 0.
pub fn ends_excmd(c: c_int) -> c_int {
    (c == NUL || c == '|' as c_int || c == '"' as c_int || c == '\n' as c_int) as c_int
}

/// The command after the next `|` or newline, or null if there is none.
/// Unlike `check_nextcmd` this searches rather than only looking ahead.
pub unsafe fn find_nextcmd(p: *const c_char) -> *mut c_char {
    let mut p = p;
    while byte(p) != '|' as c_int && byte(p) != '\n' as c_int {
        if byte(p) == NUL {
            return ptr::null_mut();
        }
        p = unsafe { p.add(1) };
    }
    unsafe { (p as *mut c_char).add(1) }
}

/// The command after `p`, if `p` is at the separator that introduces one.
pub unsafe fn check_nextcmd(p: *mut c_char) -> *mut c_char {
    let s = skipwhite(p);
    if byte(s) == '|' as c_int || byte(s) == '\n' as c_int {
        return unsafe { s.add(1) };
    }
    ptr::null_mut()
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// `skipwhite()` as checked code.
fn skipwhite(p: *const c_char) -> *mut c_char {
    // SAFETY: a NUL-terminated string.
    unsafe { crate::charset::skipwhite(p) }
}

/// `utfc_ptr2len()` as checked code.
fn utfc_ptr2len(p: *const c_char) -> c_int {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::mbyte::utfc_ptr2len(p) }
}

/// The byte `p` points at, as the C's `*p` reads it.
fn byte(p: *const c_char) -> c_int {
    // SAFETY: a NUL-terminated string the command line owns.
    unsafe { *p as c_int }
}

/// The byte `p` points at, unsigned, as the C's `(uint8_t)*p` reads it.
fn ubyte(p: *const c_char) -> u8 {
    // SAFETY: a NUL-terminated string the command line owns.
    unsafe { *p as u8 }
}

/// The byte at `p[i]`, as the C's `*(p + i)` reads it.
fn byte_at(p: *const c_char, i: isize) -> c_int {
    // SAFETY: an offset within the NUL-terminated string `p` points into.
    unsafe { *p.offset(i) as c_int }
}
