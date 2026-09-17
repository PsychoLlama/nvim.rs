//! Scanning the rest of a command line: the count, the register, the
//! `!`, the `:p`-style flags, and where the next command begins.
//!
//! Every function here is a walk over the command line the caller owns, so
//! each takes one `unsafe` block for its whole body rather than one per
//! dereference: the obligation is the same for all of them — the pointer is
//! into a NUL-terminated buffer that outlives the call — and stating it once
//! is both cheaper and more honest than stating it forty times.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

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
use crate::types::ex_cmds::ExArg;
use crate::types::pos::LineNr;
use crate::types::{CmdAddr, CmdLine, CpoFlag, ExArgt, Failed, NUL};
use crate::winlayer::Buf;

/// Step over a run of `:`, which is how a mapping's `:cmd<CR>` and a leading
/// `::::print` both reach the command name.
///
/// # Safety
///
/// `p` must point at a NUL-terminated string.
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

/// [`skip_colon_white`] over a command line, answering an offset.
pub(crate) fn skip_colons(line: &CmdLine, at: usize, skipleadingwhite: bool) -> usize {
    let mut at = if skipleadingwhite {
        line.skip_white(at)
    } else {
        at
    };
    while line.byte_at(at) == b':' {
        at = line.skip_white(at + 1);
    }
    at
}

/// Take the register name a command such as `:delete x` may carry.
///
/// Three tests have to pass before the character is read as a register: the
/// command accepts one, a user command (a negative `cmdidx`) does not take
/// `=`, and a digit belongs to the *count* rather than to a register when
/// the command takes both.
pub(crate) fn parse_register(excmd: &mut ExArg) {
    let is_user_command = is_user_cmd(excmd.cmdidx);
    if !excmd.argt.has(ExArgt::REGSTR)
        || byte(excmd.arg_ptr()) == NUL
        || (is_user_command && byte(excmd.arg_ptr()) == '=' as c_int)
        || (excmd.argt.has(ExArgt::COUNT) && ascii_isdigit(byte(excmd.arg_ptr())))
    {
        return;
    }
    // `:put` and `:iput` are the two commands that may name a write-only
    // register; every other one is writing to whichever it names.
    let writing = !is_user_command && excmd.cmdidx != CmdIdx::put && excmd.cmdidx != CmdIdx::iput;
    if !unsafe { valid_yank_reg(*excmd.arg_ptr() as c_int, writing) } {
        return;
    }
    excmd.regname = ubyte(excmd.arg_ptr()) as c_int;
    let arg_start = excmd.arg_ptr();
    excmd.set_arg_ptr(unsafe { arg_start.add(1) });
    // The expression register swallows the rest of the line: it *is* the
    // expression, and evaluating it is deferred until the register is read.
    if excmd.regname == '=' as c_int && byte(excmd.arg_ptr()) != NUL {
        if !excmd.skip {
            unsafe { set_expr_line(xstrdup(excmd.arg_ptr())) };
        }
        let arg_start = excmd.arg_ptr();
        excmd.set_arg_ptr(unsafe { arg_start.add(cstr::bytes_at(arg_start).len()) });
    }
    let arg_start = excmd.arg_ptr();
    excmd.set_arg_ptr(skipwhite(arg_start));
}

/// Turn a count into a range, which is what a count means for every command
/// that takes one: "this many lines, starting where the range ended".
pub fn set_cmd_count(excmd: &mut ExArg, count: LineNr, validate: bool) {
    if excmd.addr_type != CmdAddr::Lines {
        excmd.line2 = count;
        if excmd.addr_count == 0 {
            excmd.addr_count = 1;
        }
        return;
    }
    excmd.line1 = excmd.line2;
    // Upstream's overflow guard is `line2 >= INT32_MAX - (count - 1)`,
    // and for `count == 0` — which only `nvim_cmd` can supply — the
    // right-hand side itself overflows. The C wraps there, so the
    // comparison always succeeds and the answer is `INT32_MAX`. Spelled
    // as a wrapping subtraction so the debug build does not abort.
    if excmd.line2 >= (INT32_MAX as LineNr).wrapping_sub(count.wrapping_sub(1)) {
        excmd.line2 = INT32_MAX as LineNr;
    } else {
        excmd.line2 += count - 1;
    }
    excmd.addr_count += 1;
    if validate && excmd.line2 > Buf::current().b_ml.ml_line_count {
        excmd.line2 = Buf::current().b_ml.ml_line_count;
    }
}

/// Take the count a command such as `:delete 3` may carry, and fold it into
/// the range.
pub(crate) fn parse_count(
    excmd: &mut ExArg,
    errormsg: &mut Option<CString>,
    validate: bool,
) -> Result<(), Failed> {
    if !excmd.argt.has(ExArgt::COUNT) || !ascii_isdigit(byte(excmd.arg_ptr())) {
        return Ok(());
    }
    // A command that also takes a buffer name (`:buffer 2x`) only reads
    // the digits as a count when they are the whole word.
    if excmd.argt.has(ExArgt::BUFNAME) {
        let p = unsafe { skipdigits(excmd.arg_ptr().add(1)) };
        if byte(p) != NUL && !ascii_iswhite(byte(p)) {
            return Ok(());
        }
    }

    let n: LineNr =
        unsafe { excmd.with_arg_cursor(|cursor| getdigits_int32(cursor, false, INT32_MAX)) };
    let arg_start = excmd.arg_ptr();
    excmd.set_arg_ptr(skipwhite(arg_start));
    if let Some(&(first, first_len)) = excmd.line.args.first() {
        // `nvim_cmd` supplies the arguments already split, so the count
        // that was just consumed has to come off the first of them.
        let arg = excmd.line.arg;
        debug_assert!(arg >= first);
        if arg < first + first_len {
            excmd.line.args[0] = (arg, first_len - (arg - first));
        } else {
            shift_cmd_args(excmd);
        }
    }
    if n <= 0 && !excmd.argt.has(ExArgt::ZEROR) {
        *errormsg = Some(unsafe { ex_msg(e_zerocount.as_ptr()) });
        return Err(Failed);
    }
    set_cmd_count(excmd, n, validate);
    Ok(())
}

/// Take the `!` a command may carry. `:substitute` and its two magic
/// spellings are the exception: there a `!` belongs to the pattern.
///
/// # Safety
///
/// `p` must point at a writable `*mut c_char` slot the caller owns for the
/// call.
pub(crate) unsafe fn parse_bang(excmd: &mut ExArg, p: *mut *mut c_char) -> bool {
    let cmdidx = excmd.cmdidx;
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
pub(crate) fn get_flags(excmd: &mut ExArg) {
    loop {
        let flag = match ubyte(excmd.arg_ptr()) {
            b'l' => EXFLAG_LIST,
            b'p' => EXFLAG_PRINT,
            b'#' => EXFLAG_NR,
            _ => return,
        };
        excmd.flags |= flag;
        let arg_start = excmd.arg_ptr();
        excmd.set_arg_ptr(unsafe { skipwhite(arg_start.add(1)) });
    }
}

/// Step over a `:vimgrep` pattern, whose delimiters are not the ones the
/// rest of the argument scan knows about.
pub(crate) fn skip_grep_pat(excmd: &mut ExArg) -> usize {
    let cmdidx = excmd.cmdidx;
    let is_grep = cmdidx == CmdIdx::vimgrep
        || cmdidx == CmdIdx::lvimgrep
        || cmdidx == CmdIdx::vimgrepadd
        || cmdidx == CmdIdx::lvimgrepadd
        || grep_internal(excmd.cmdidx);
    if excmd.line.byte_at(excmd.line.arg) == 0 || !is_grep {
        return excmd.line.arg;
    }
    let arg = excmd.line.ptr_at(excmd.line.arg);
    // SAFETY: the command's own argument, NUL-terminated; neither
    // out-parameter is wanted here.
    let p = unsafe { skip_vimgrep_pat(arg, ptr::null_mut(), ptr::null_mut()) };
    if p.is_null() {
        excmd.line.arg
    } else {
        excmd.line.offset_of(p)
    }
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
pub fn separate_nextcmd(excmd: &mut ExArg) {
    let mut at = skip_grep_pat(excmd);
    while excmd.line.byte_at(at) != 0 {
        let c = excmd.line.byte_at(at);
        if c_int::from(c) == Ctrl_V {
            if excmd.argt.has(ExArgt::CTRLV | ExArgt::XFILE) {
                at += 1;
            } else {
                excmd.line.drop_byte(at);
            }
            if excmd.line.byte_at(at) == 0 {
                break;
            }
        } else if c == b'`' && excmd.line.byte_at(at + 1) == b'=' && excmd.argt.has(ExArgt::XFILE) {
            // A backtick-equals expression is stepped over by the
            // evaluator, not by this scan: it may contain any of the
            // ending characters.
            let mut cursor = excmd.line.ptr_at(at + 2);
            // SAFETY: `cursor` is this frame's own, over the command's line.
            let _ = unsafe { skip_expr(&raw mut cursor, ptr::null_mut()) };
            at = excmd.line.offset_of(cursor);
            if excmd.line.byte_at(at) == 0 {
                break;
            }
        } else if ends_argument(excmd, at) {
            let escaped = (!cpo_has(CpoFlag::BAR) || !excmd.argt.has(ExArgt::CTRLV))
                && at > 0
                && excmd.line.byte_at(at - 1) == b'\\';
            if escaped {
                at -= 1;
                excmd.line.drop_byte(at);
            } else {
                excmd.line.next = excmd.line.check_next(at);
                excmd.line.terminate_at(at);
                break;
            }
        }
        at += utfc_len_at(&excmd.line, at);
    }
    if !excmd.argt.has(ExArgt::NOTRLCOM) {
        let arg = excmd.line.ptr_at(excmd.line.arg);
        // SAFETY: the command's own argument, NUL-terminated.
        unsafe { del_trailing_spaces(arg) };
    }
}

/// Does the byte at `at` end the argument? See [`separate_nextcmd`].
///
/// A named predicate rather than an inline condition, but deliberately
/// *inside* the loop: the `"` half compares `at` against the argument, so it
/// depends on where the walk has got to and cannot be hoisted.
fn ends_argument(excmd: &ExArg, at: usize) -> bool {
    let c = excmd.line.byte_at(at);
    let cmdidx = excmd.cmdidx;
    let comment = c == b'"'
        && !excmd.argt.has(ExArgt::NOTRLCOM)
        && (cmdidx != CmdIdx::at || at != excmd.line.arg)
        && (cmdidx != CmdIdx::redir
            || at != excmd.line.arg + 1
            || at == 0
            || excmd.line.byte_at(at - 1) != b'@');
    let bar = c == b'|'
        && cmdidx != CmdIdx::append
        && cmdidx != CmdIdx::change
        && cmdidx != CmdIdx::insert;
    comment || bar || c == b'\n'
}

/// How many bytes the character at `at` occupies.
fn utfc_len_at(line: &CmdLine, at: usize) -> usize {
    // SAFETY: a cursor into the NUL-terminated command line.
    let len = unsafe { crate::mbyte::utfc_ptr2len(line.ptr_from(at)) };
    len.max(1).cast_unsigned() as usize
}

/// Delete the byte at `p` by pulling the terminator-inclusive tail over it.
fn drop_one_byte(p: *mut c_char) {
    let n_len = unsafe { cstr::bytes_at(p.add(1)) }.len();
    let into = p.cast::<u8>();
    unsafe { into.copy_from(p.add(1).cast(), n_len.wrapping_add(1)) };
}

/// Step to the end of a whitespace-delimited argument, optionally removing
/// the backslashes that escaped whitespace inside it.
///
/// # Safety
///
/// `p` must point at a NUL-terminated string, unaliased for the call.
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
///
/// # Safety
///
/// `p` must point at a NUL-terminated string.
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
///
/// # Safety
///
/// `p` must point at a NUL-terminated string, unaliased for the call.
pub unsafe fn check_nextcmd(p: *mut c_char) -> *mut c_char {
    let s = skipwhite(p);
    if byte(s) == '|' as c_int || byte(s) == '\n' as c_int {
        return unsafe { s.add(1) };
    }
    ptr::null_mut()
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
