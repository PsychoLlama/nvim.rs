//! `do_one_cmd` — parse and run one command from a command line.
//!
//! The order here is the order the C's numbered comments describe, and it
//! is load-bearing: the command *name* has to be found before the range can
//! be parsed, because the name is what says which kind of address the range
//! counts in. So the line is walked twice — `find_excmd_after_range` skips
//! a range it does not yet understand to reach the name, then
//! `parse_cmd_address` goes back and reads the range properly.
//!
//! Every exit runs the same epilogue (the C's `doend:` label), which
//! reports the error, rethrows it as an exception if something is catching,
//! and unwinds the command modifiers. That is what the `'doend` block is.
#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::types::AutoEvent;
use crate::types::CmdIdx;
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;
use std::ffi::CString;

use crate::autocmd::{apply_autocmds, getnextac, has_event};

use crate::cstr;
use crate::debugger::dbg_check_breakpoint;
use crate::edit::{BeginlineOpts, beginline};
use crate::eval::userfunc::{current_func_returned, do_return, get_func_line};
use crate::ex_docmd::address::{correct_range, find_excmd_after_range, parse_cmd_address};
use crate::ex_docmd::addrtype::{set_cmd_addr_type, set_cmd_dflall_range};
use crate::ex_docmd::xfree;

use crate::ex_docmd::api::execute_cmd0;
use crate::ex_docmd::argopt::{getargcmd, getargopt};
use crate::ex_docmd::edit::ex_print;
use crate::ex_docmd::filename::replace_makeprg;
use crate::ex_docmd::lookup::{find_ex_command, is_user_cmd};
use crate::ex_docmd::modifier::CmdModScope;
use crate::ex_docmd::scan::{
    check_nextcmd, get_flags, parse_bang, parse_count, parse_register, separate_nextcmd,
    skip_colon_white,
};
use crate::ex_docmd::source::{ex_errmsg, getline_cookie};

use crate::ex_docmd::verify::verify_command;
use crate::ex_docmd::{
    DoCmdOpts, ExFunc, PROF_YES, cmdnames, e_ambiguous_use_of_user_defined_command,
    e_not_an_editor_command, exmode_plus, quitmore,
};
use crate::ex_eval::CsFlags;

use crate::ex_eval::{aborting, do_errthrow, do_intthrow, do_throw};
use crate::ex_getln::{get_text_locked_msg, script_get, text_locked};

use crate::ex_docmd::state::{did_emsg_syntax, ex_nesting_level, global_busy};
use crate::ex_eval::state::{check_cstack, did_throw, need_rethrow};
use crate::fold::has_folding;
use crate::getchar::state::{got_int, pending_end_reg_executing, reg_executing};
use crate::guard::Depth;
use crate::guard::sandbox;
use crate::input::ask_yesno;
use crate::mbyte::{mb_copy_char, utf_head_off, utfc_ptr2len};
use crate::memory::{xmemdupz, xstrlcat, xstrlcpy};
use crate::message::state::{did_emsg, msg_silent};
use crate::message::{
    e_argreq, e_cmdwin, e_invarg, e_invrange, e_modifiable, e_nobang, e_norange, e_sandbox,
    e_trailing_arg,
};
use crate::profile::do_profiling;
use crate::startup::exiting;
use crate::state::mode::exmode_active;
use crate::winlayer::graph::cmdwin_type;

use crate::message::emsg;
use crate::os::cshim::gettext;
use crate::profile::{func_line_exec, script_line_exec};
use crate::runtime::{do_finish, getsourceline, source_finished};
use crate::types::{CmdAddr, CondStack, ExArg, ExArgt, FAIL, IOSIZE, LineGetter, NUL, size_t};
use crate::winlayer::{Buf, Live, Win};

/// The conditional stack the command is running under, whose caller has
/// promised it outlives the value.
type Cs = Live<CondStack>;
use ::libc::strcpy;

/// A zeroed `ExArg` with the empty range the parsers start from.
///
/// `CmdIdx::append` and `CmdAddr::Lines` are both zero, so the only fields the C's
/// `(ExArg){ .line1 = 1, .line2 = 1 }` sets to anything else are the two
/// line numbers.
pub(crate) fn fresh_exarg() -> ExArg {
    // SAFETY: `ExArg` is a `repr(C)` aggregate of scalars, pointers and
    // `Option<fn>`; all-zero is a valid value of every one of them.
    let mut ea: ExArg = unsafe { core::mem::zeroed() };
    ea.line1 = 1;
    ea.line2 = 1;
    ea
}

/// Is `func` this exact Ex-command handler?
///
/// Ex-command callbacks are identified by address, as the C code did; the
/// comparison is spelled out so the intent survives the
/// `unpredictable_function_pointer_comparisons` lint.
pub(crate) fn ex_func_is(func: ExFunc, f: fn(&mut ExArg)) -> bool {
    func.is_some_and(|g| ptr::fn_addr_eq(g, f))
}

/// Is this a command the build knows the name of but cannot run?
pub fn is_cmd_ni(cmdidx: CmdIdx) -> bool {
    !is_user_cmd(cmdidx)
        && (ex_func_is(cmdnames[cmdidx.index()].cmd_func, ex_ni)
            || ex_func_is(cmdnames[cmdidx.index()].cmd_func, ex_script_ni))
}

/// Drop the first of an API-supplied argument list, and point `eap->arg` at
/// what is left.
///
/// With no arguments left, `eap->arg` answers the end of the *old* first
/// argument rather than null — a command that reads `eap->arg` as a string
/// then sees an empty one.
pub(crate) fn shift_cmd_args(excmd: &mut ExArg) {
    debug_assert!(!excmd.args.is_null() && excmd.argc > 0);
    let oldargs = excmd.args;
    let oldarglens = excmd.arglens;

    excmd.argc -= 1;
    excmd.args = if excmd.argc > 0 {
        xcalloc(excmd.argc, size_of::<*mut c_char>()) as *mut *mut c_char
    } else {
        ptr::null_mut()
    };
    excmd.arglens = if excmd.argc > 0 {
        xcalloc(excmd.argc, size_of::<size_t>()) as *mut size_t
    } else {
        ptr::null_mut()
    };
    for i in 0..excmd.argc {
        unsafe { *excmd.args.add(i) = *oldargs.add(i + 1) };
        unsafe { *excmd.arglens.add(i) = *oldarglens.add(i + 1) };
    }
    excmd.arg = if excmd.argc > 0 {
        unsafe { *excmd.args }
    } else {
        unsafe { (*oldargs).add(*oldarglens) }
    };

    xfree(oldargs as *mut c_void);
    xfree(oldarglens as *mut c_void);
}

/// Should this command be passed over rather than run?
///
/// The alternation is upstream's two lists of commands that still run when
/// `do_one_cmd` is skipping -- inside an inactive `:if` branch, or after an
/// error. The control-flow ones (through `:function`) have to run to find
/// the end of the construct at all; the rest consume the remainder of the
/// line themselves, and upstream's rule for that group is that a command
/// must either carry `ExArgt::TRLBAR`, appear here, or appear in the list at
/// `:help :bar`.
///
/// A `matches!` rather than a table and `.contains`: this runs once per Ex
/// command, and walking 78 enum values is that many calls to the
/// derived `PartialEq` at `-O0`, which is what the test suites build.
#[rustfmt::skip]
pub(crate) fn skip_cmd(excmd: &mut ExArg) -> bool {
    excmd.skip && !matches!(excmd.cmdidx,
        CmdIdx::r#while | CmdIdx::endwhile | CmdIdx::r#for | CmdIdx::endfor |
        CmdIdx::r#if | CmdIdx::elseif | CmdIdx::r#else | CmdIdx::endif | CmdIdx::r#try |
        CmdIdx::catch | CmdIdx::finally | CmdIdx::endtry | CmdIdx::function |
        CmdIdx::aboveleft | CmdIdx::and | CmdIdx::belowright | CmdIdx::botright |
        CmdIdx::browse | CmdIdx::call | CmdIdx::confirm | CmdIdx::r#const |
        CmdIdx::delfunction | CmdIdx::djump | CmdIdx::dlist | CmdIdx::dsearch |
        CmdIdx::dsplit | CmdIdx::echo | CmdIdx::echoerr | CmdIdx::echomsg | CmdIdx::echon |
        CmdIdx::eval | CmdIdx::execute | CmdIdx::filter | CmdIdx::help | CmdIdx::hide |
        CmdIdx::horizontal | CmdIdx::ijump | CmdIdx::ilist | CmdIdx::isearch |
        CmdIdx::isplit | CmdIdx::keepalt | CmdIdx::keepjumps | CmdIdx::keepmarks |
        CmdIdx::keeppatterns | CmdIdx::leftabove | CmdIdx::r#let | CmdIdx::lockmarks |
        CmdIdx::lockvar | CmdIdx::lua | CmdIdx::r#match | CmdIdx::mzscheme |
        CmdIdx::noautocmd | CmdIdx::noswapfile | CmdIdx::perl | CmdIdx::psearch |
        CmdIdx::python | CmdIdx::py3 | CmdIdx::python3 | CmdIdx::pythonx | CmdIdx::pyx |
        CmdIdx::r#return | CmdIdx::rightbelow | CmdIdx::ruby | CmdIdx::silent |
        CmdIdx::smagic | CmdIdx::snomagic | CmdIdx::substitute | CmdIdx::syntax |
        CmdIdx::tab | CmdIdx::tcl | CmdIdx::throw | CmdIdx::tilde | CmdIdx::topleft |
        CmdIdx::unlet | CmdIdx::unlockvar | CmdIdx::verbose | CmdIdx::vertical |
        CmdIdx::wincmd
    )
}

/// Parse and execute one Ex command, and answer where the next one starts.
///
/// `fgetline`/`cookie` are the line source the command may read further
/// lines from (`:append`, a `:function` body, a sourced file); either may be
/// null. Re-entrant: a command that calls `do_cmdline` lands back here.
///
/// # Safety
///
/// `cmdlinep` must point at a writable `*mut c_char` slot the caller owns for
/// the call. `cstack` must point at a live `CondStack`, unaliased for the
/// call. `cookie` must be the payload `fgetline` was registered with, live
/// for the call.
pub(crate) unsafe fn do_one_cmd(
    cmdlinep: *mut *mut c_char,
    flags: DoCmdOpts,
    cstack: *mut CondStack,
    fgetline: LineGetter,
    cookie: *mut c_void,
) -> *mut c_char {
    let mut errormsg: Option<CString> = None;
    let save_reg_executing = reg_executing.get();
    let save_pending_end_reg_executing = pending_end_reg_executing.get();
    let mut ea = fresh_exarg();
    let nesting = Depth::of(&ex_nesting_level);

    // When the last file has not been edited `:q` has to be typed twice.
    // A `'statusline'` function call and an autocommand (QuitPre) both
    // reach here without the user having typed anything, so neither
    // spends the second `:q`.
    if quitmore_is_pending(fgetline, cookie) {
        quitmore.set(quitmore.get() - 1);
    }

    // Modifiers are restored on the way out, for recursive calls. The
    // guard owns the `:filter` pattern and program of the set it took
    // out until it puts them back.
    let mods = CmdModScope::cleared();
    let after_modifier: *mut c_char;

    'doend: {
        // "#!anything" is a comment, so that a script can carry a
        // shebang line.
        // SAFETY: `cmdlinep` is the caller's, and names the command line.
        let line = unsafe { *cmdlinep };
        if byte_at(line, 0) == '#' as c_int && byte_at(line, 1) == '!' as c_int {
            break 'doend;
        }

        ea.cmd = unsafe { *cmdlinep };
        ea.cmdlinep = cmdlinep;
        ea.ea_getline = fgetline;
        ea.cookie = cookie;
        ea.cstack = cstack;

        if mods.parse(&mut ea, &mut errormsg).is_err() {
            break 'doend;
        }
        mods.apply();
        after_modifier = ea.cmd;

        // SAFETY: `cstack` is the caller's conditional stack, live for the
        // whole of this command.
        ea.skip = did_emsg.get() != 0
            || got_int.get()
            || did_throw.get()
            || unsafe {
                (*cstack).cs_idx >= 0
                    && !(*cstack).cs_flags[(*cstack).cs_idx as usize].has(CsFlags::ACTIVE)
            };

        // The command name is needed before the range can be read: it is
        // what says whether an address counts lines, windows, buffers or
        // tab pages.
        let mut p = find_excmd_after_range(&mut ea);
        unsafe { profile_cmd(&ea, cstack, fgetline, cookie) };

        if !exiting.get() {
            // May go to debug mode. If the `>quit` debug command is used
            // there, an interrupt exception is thrown and this command
            // is skipped.
            dbg_check_breakpoint(&mut ea);
        }
        if !ea.skip && got_int.get() {
            ea.skip = true;
            unsafe { do_intthrow(cstack) };
        }

        unsafe { set_cmd_addr_type(&mut ea, p) };
        if parse_cmd_address(&mut ea, &mut errormsg, false) == FAIL {
            break 'doend;
        }

        ea.cmd = unsafe { skip_colon_white(ea.cmd, true) };

        // A range with no command after it. Vi's behaviour, preserved:
        // `:3` jumps to line 3, `:3|…` *prints* line 3, and `:|` prints
        // the current line.
        if byte(ea.cmd) == NUL || byte(ea.cmd) == '"' as c_int || {
            ea.nextcmd = unsafe { check_nextcmd(ea.cmd) };
            !ea.nextcmd.is_null()
        } {
            if !ea.skip {
                debug_assert!(errormsg.is_none());
                errormsg = ex_range_without_command(&mut ea);
            }
            break 'doend;
        }

        // An unknown command spelled like a user command, with a
        // CmdUndefined autocommand waiting to define it.
        if !p.is_null()
            && ea.cmdidx == CmdIdx::SIZE
            && !ea.skip
            && (ubyte(ea.cmd)).is_ascii_uppercase()
            && has_event(AutoEvent::CmdUndefined)
        {
            let mut end = ea.cmd;
            while (ubyte(end)).is_ascii_alphanumeric() {
                end = unsafe { end.add(1) };
            }
            let cmdname =
                unsafe { xmemdupz(ea.cmd as *const c_void, end.offset_from(ea.cmd) as size_t) }
                    as *mut c_char;
            let event = AutoEvent::CmdUndefined;
            let ret = unsafe { apply_autocmds(event, cmdname, cmdname, true, None) };
            xfree(cmdname as *mut c_void);
            // Look again only if the autocommands did something and did
            // not fail.
            p = if ret && !aborting() {
                unsafe { find_ex_command(&mut ea, ptr::null_mut()) }
            } else {
                ea.cmd
            };
        }

        if p.is_null() {
            if !ea.skip {
                errormsg = Some(ex_msg(e_ambiguous_use_of_user_defined_command.as_ptr()));
            }
            break 'doend;
        }

        if ea.cmdidx == CmdIdx::SIZE {
            if !ea.skip {
                // The modifiers parsed, so the error is in what follows
                // them.
                let cmdname = if after_modifier.is_null() {
                    unsafe { *cmdlinep }
                } else {
                    after_modifier
                };
                let msg = ex_msg(e_not_an_editor_command.as_ptr());
                errormsg = Some(if flags.has(DoCmdOpts::VERBOSE) {
                    // The whole line is appended below instead.
                    msg
                } else {
                    unsafe { append_command(&msg, cmdname) }
                });
                did_emsg_syntax.set(true);
                unsafe { verify_command(cmdname) };
            }
            break 'doend;
        }

        // Not implemented in this build: the argument checks below are
        // relaxed, because there is nothing to check them against.
        let ni = is_cmd_ni(ea.cmdidx);

        ea.forceit = unsafe { parse_bang(&mut ea, &raw mut p) };

        if !is_user_cmd(ea.cmdidx) {
            ea.argt = cmdnames[ea.cmdidx.index()].cmd_argt;
        }

        if !ea.skip {
            if let Some(msg) = refuses_here(&ea) {
                errormsg = Some(msg);
                break 'doend;
            }
            // `curbuf->b_ro_locked` forbids editing another buffer.
            // `:checktime` is postponed rather than refused, and `:edit`
            // and `:file` are checked again once their argument is known.
            if !ea.argt.has(ExArgt::CMDWIN)
                && ea.cmdidx != CmdIdx::checktime
                && ea.cmdidx != CmdIdx::edit
                && ea.cmdidx != CmdIdx::file
                && !is_user_cmd(ea.cmdidx)
                && curbuf_locked()
            {
                break 'doend;
            }
            if !ni && !ea.argt.has(ExArgt::RANGE) && ea.addr_count > 0 {
                errormsg = Some(ex_msg(e_norange.as_ptr()));
                break 'doend;
            }
        }

        if !ni && !ea.argt.has(ExArgt::BANG) && ea.forceit {
            errormsg = Some(ex_msg(e_nobang.as_ptr()));
            break 'doend;
        }

        // A range that is not used is not complained about, which can
        // happen when a line count is accidentally zero.
        if !ea.skip && !ni && ea.argt.has(ExArgt::RANGE) {
            // A backwards range is offered for swapping. `:global` is
            // busy running a command per line and would fail below
            // anyway, so it is not asked.
            if global_busy.get() == 0 && ea.line1 > ea.line2 {
                if msg_silent.get() == 0 {
                    if flags.has(DoCmdOpts::VERBOSE) || exmode_active.get() {
                        errormsg = Some(ex_msg(c"E493: Backwards range given".as_ptr()));
                        break 'doend;
                    }
                    if unsafe { ask_yesno(gettext(c"Backwards range given, OK to swap").as_ptr()) }
                        != 'y' as c_int
                    {
                        break 'doend;
                    }
                }
                core::mem::swap(&mut ea.line1, &mut ea.line2);
            }
            errormsg = invalid_range(&mut ea);
            if errormsg.is_some() {
                break 'doend;
            }
        }

        // `CmdAddr::Other` counts from 1 rather than from the cursor.
        if ea.addr_type == CmdAddr::Other && ea.addr_count == 0 {
            ea.line2 = 1;
        }

        correct_range(&mut ea);

        // Put the first line at the start of a closed fold and the last
        // line at its end.
        if (ea.argt.has(ExArgt::WHOLEFOLD) || ea.addr_count >= 2)
            && global_busy.get() == 0
            && ea.addr_type == CmdAddr::Lines
        {
            has_folding(Win::current(), ea.line1, Some(&mut ea.line1), None);
            has_folding(Win::current(), ea.line2, None, Some(&mut ea.line2));
        }

        // `:make` and `:grep` splice 'makeprg'/'grepprg' into the line
        // here, so that `%` and friends expand inside it.
        p = unsafe { replace_makeprg(&mut ea, p, cmdlinep) };
        if p.is_null() {
            break 'doend;
        }

        // `:!` keeps the space: `:!! -l` needs it.
        ea.arg = if ea.cmdidx == CmdIdx::bang {
            p
        } else {
            skipwhite(p)
        };

        if ea.cmdidx == CmdIdx::file && byte(ea.arg) != NUL && curbuf_locked() {
            break 'doend;
        }

        // `++opt=val` first, so that `:w ++enc=utf8 !cmd` works.
        if ea.argt.has(ExArgt::ARGOPT) {
            while byte_at(ea.arg, 0) == '+' as c_int && byte_at(ea.arg, 1) == '+' as c_int {
                if getargopt(&mut ea).is_err() && !ni {
                    errormsg = Some(ex_msg(e_invarg.as_ptr()));
                    break 'doend;
                }
            }
        }

        if ea.cmdidx == CmdIdx::write || ea.cmdidx == CmdIdx::update {
            if byte(ea.arg) == '>' as c_int {
                ea.arg = unsafe { ea.arg.add(1) };
                if byte(ea.arg) != '>' as c_int {
                    errormsg = Some(ex_msg(c"E494: Use w or w>>".as_ptr()));
                    break 'doend;
                }
                ea.arg = unsafe { skipwhite(ea.arg.add(1)) };
                ea.append = true;
            } else if byte(ea.arg) == '!' as c_int && ea.cmdidx == CmdIdx::write {
                // `:w !filter`
                ea.arg = unsafe { ea.arg.add(1) };
                ea.usefilter = true;
            }
        } else if ea.cmdidx == CmdIdx::read {
            if ea.forceit {
                // `:r!filter`
                ea.usefilter = true;
                ea.forceit = false;
            } else if byte(ea.arg) == '!' as c_int {
                // `:r !filter`
                ea.arg = unsafe { ea.arg.add(1) };
                ea.usefilter = true;
            }
        } else if ea.cmdidx == CmdIdx::lshift || ea.cmdidx == CmdIdx::rshift {
            // How far to shift is how many `<` or `>` were typed.
            ea.amount = 1;
            while byte(ea.arg) == byte(ea.cmd) {
                ea.arg = unsafe { ea.arg.add(1) };
                ea.amount += 1;
            }
            ea.arg = skipwhite(ea.arg);
        }

        // `+command`, before the next command is looked for. Not for
        // `:read !cmd` and `:write !cmd`.
        if ea.argt.has(ExArgt::CMDARG) && !ea.usefilter {
            ea.do_ecmd_cmd = unsafe { getargcmd(&raw mut ea.arg) };
        }

        if ea.argt.has(ExArgt::TRLBAR) && !ea.usefilter {
            separate_nextcmd(&mut ea);
        } else if ea.cmdidx == CmdIdx::bang
            || ea.cmdidx == CmdIdx::terminal
            || ea.cmdidx == CmdIdx::global
            || ea.cmdidx == CmdIdx::vglobal
            || ea.usefilter
        {
            // A shell command ends at a newline instead, and one
            // backslash before that newline is removed.
            let mut s = ea.arg;
            while unsafe { *s } != 0 {
                if byte(s) == '\\' as c_int && byte_at(s, 1) == '\n' as c_int {
                    let into = s.cast::<u8>();
                    unsafe { into.copy_from(s.add(1).cast(), len_of(s.add(1)) + 1) };
                } else if byte(s) == '\n' as c_int {
                    ea.nextcmd = unsafe { s.add(1) };
                    unsafe { *s = NUL as c_char };
                    break;
                }
                s = unsafe { s.add(1) };
            }
        }

        if ea.argt.has(ExArgt::DFLALL) && ea.addr_count == 0 {
            set_cmd_dflall_range(&mut ea);
        }

        parse_register(&mut ea);
        if parse_count(&mut ea, &mut errormsg, true).is_err() {
            break 'doend;
        }

        if ea.argt.has(ExArgt::FLAGS) {
            get_flags(&mut ea);
        }
        if !ni
            && !ea.argt.has(ExArgt::EXTRA)
            && byte(ea.arg) != NUL
            && byte(ea.arg) != '"' as c_int
            && (byte(ea.arg) != '|' as c_int || !ea.argt.has(ExArgt::TRLBAR))
        {
            errormsg = Some(unsafe { ex_errmsg(e_trailing_arg.as_ptr(), ea.arg) });
            break 'doend;
        }
        if !ni && ea.argt.has(ExArgt::NEEDARG) && byte(ea.arg) == NUL {
            errormsg = Some(ex_msg(e_argreq.as_ptr()));
            break 'doend;
        }

        if skip_cmd(&mut ea) {
            break 'doend;
        }

        let mut retv: c_int = 0;
        if unsafe { execute_cmd0(&raw mut retv, &mut ea, &mut errormsg, false) }.is_err() {
            break 'doend;
        }

        // A command that called `do_cmdline` may have left a throw, a
        // `:return` or a `:finish` that the *outer* conditional stack
        // still has to see. Re-raise it here.
        if need_rethrow.get() {
            unsafe { do_throw(cstack) };
        } else if check_cstack.get() {
            if unsafe { source_finished(fgetline, cookie) } {
                do_finish(&mut ea, true);
            } else if getline_equal(fgetline, cookie, Some(get_func_line))
                && current_func_returned() != 0
            {
                unsafe { do_return(&mut ea, true, false, ptr::null_mut()) };
            }
        }
        check_cstack.set(false);
        need_rethrow.set(false);
    }

    // Can happen with a zero line number.
    if Win::current().w_cursor.lnum == 0 {
        Win::current().w_cursor.lnum = 1;
        Win::current().w_cursor.col = 0;
    }

    if let Some(msg) = errormsg
        && !msg.is_empty()
        && did_emsg.get() == 0
    {
        let msg = if flags.has(DoCmdOpts::VERBOSE) {
            unsafe { append_command(&msg, *ea.cmdlinep) }
        } else {
            msg
        };
        emsg(&msg);
    }
    unsafe {
        do_errthrow(
            cstack,
            if ea.cmdidx != CmdIdx::SIZE && !is_user_cmd(ea.cmdidx) {
                cmdnames[ea.cmdidx.index()].cmd_name
            } else {
                ptr::null_mut()
            },
        )
    };

    drop(mods);
    reg_executing.set(save_reg_executing);
    pending_end_reg_executing.set(save_pending_end_reg_executing);

    // A trailing bar with nothing after it is not really a next command.
    if !ea.nextcmd.is_null() && byte(ea.nextcmd) == NUL {
        ea.nextcmd = ptr::null_mut();
    }

    drop(nesting);
    xfree(ea.cmdline_tofree as *mut c_void);

    ea.nextcmd
}

/// Does the "type `:q` twice" counter belong to a command the *user* typed?
fn quitmore_is_pending(fgetline: LineGetter, cookie: *mut c_void) -> bool {
    // SAFETY: `getline_equal` only compares `fgetline` against a known line
    // getter, walking `cookie` as a `LoopCookie` chain the caller owns.
    quitmore.get() != 0
        && !getline_equal(fgetline, cookie, Some(get_func_line))
        && !getline_equal(fgetline, cookie, Some(getnextac))
}

/// Count this line for `:profile`, if profiling is on and the line is one
/// that will really run.
///
/// The `skip` this recomputes is not `args.skip`: a `:catch` that is about
/// to be entered, an `:else` whose branch is about to be taken and a
/// `:finally` all execute even though the surrounding construct is
/// inactive, and each is worth a profile sample.
///
/// # Safety
///
/// `excmd` must point at the command's `ExArg`. `cstack` must point at a live
/// `CondStack`, unaliased for the call. `cookie` must be the payload
/// `fgetline` was registered with, live for the call.
pub(crate) unsafe fn profile_cmd(
    excmd: &ExArg,
    cstack: *mut CondStack,
    fgetline: LineGetter,
    cookie: *mut c_void,
) {
    // SAFETY: the caller's conditional stack, live for the command.
    let cs = unsafe { Cs::new(cstack) };
    if do_profiling.get() != PROF_YES
        || !(!excmd.skip
            || cs.cs_idx == 0
            || (cs.cs_idx > 0 && cs.cs_flags[cs.cs_idx as usize - 1].has(CsFlags::ACTIVE)))
    {
        return;
    }
    let mut skip = did_emsg.get() != 0 || got_int.get() || did_throw.get();
    let idx = cs.cs_idx;
    match excmd.cmdidx {
        CmdIdx::catch => {
            skip = !skip
                && !(idx >= 0
                    && cs.cs_flags[idx as usize].has(CsFlags::THROWN)
                    && !cs.cs_flags[idx as usize].has(CsFlags::CAUGHT));
        }
        CmdIdx::r#else | CmdIdx::elseif => {
            skip = skip
                || !(idx >= 0 && !cs.cs_flags[idx as usize].has(CsFlags::ACTIVE | CsFlags::TRUE));
        }
        CmdIdx::finally => skip = false,
        // The four block-enders are the only commands left that keep the
        // caller's `skip`; everything else takes it.
        CmdIdx::endif | CmdIdx::endfor | CmdIdx::endtry | CmdIdx::endwhile => {}
        _ => skip = excmd.skip,
    }
    if skip {
        return;
    }
    if getline_equal(fgetline, cookie, Some(get_func_line)) {
        unsafe { func_line_exec(getline_cookie(fgetline, cookie)) };
    } else if getline_equal(fgetline, cookie, Some(getsourceline)) {
        script_line_exec();
    }
}

/// The three "this command is not allowed here" checks that share an exit.
///
/// Answers the message to report, or `None` when the command may run.
fn refuses_here(ea: &ExArg) -> Option<CString> {
    if sandbox.get() != 0 && !ea.argt.has(ExArgt::SBOXOK) {
        return Some(ex_msg(e_sandbox.as_ptr()));
    }
    // `:put` is allowed in a terminal buffer, which is not 'modifiable'.
    if Buf::current().b_p_ma == 0
        && ea.argt.has(ExArgt::MODIFY)
        && !(!Buf::current().terminal.is_null()
            && (ea.cmdidx == CmdIdx::put || ea.cmdidx == CmdIdx::iput))
    {
        return Some(ex_msg(e_modifiable.as_ptr()));
    }
    if !is_user_cmd(ea.cmdidx) {
        if cmdwin_type.get() != 0 && !ea.argt.has(ExArgt::CMDWIN) {
            return Some(ex_msg(e_cmdwin.as_ptr()));
        }
        if text_locked() && !ea.argt.has(ExArgt::LOCK_OK) {
            return Some(ex_msg(get_text_locked_msg().as_ptr()));
        }
    }
    None
}

/// A range with no command after it: print the lines, or move the cursor to
/// the last of them.
///
/// Which of the two it is depends on how the line ended — a `|` after the
/// range, or Ex mode, means print. `exmode_plus + 1` is the empty string Ex
/// mode substitutes for a bare `+`; it is recognised by *address*, not by
/// content.
pub(crate) fn ex_range_without_command(excmd: &mut ExArg) -> Option<CString> {
    let mut errormsg: Option<CString> = None;
    if byte(excmd.cmd) == '|' as c_int
        || (exmode_active.get() && !ptr::eq(excmd.cmd, unsafe { exmode_plus.as_ptr().add(1) }))
    {
        excmd.cmdidx = CmdIdx::print;
        excmd.argt = ExArgt::RANGE | ExArgt::COUNT | ExArgt::TRLBAR;
        errormsg = invalid_range(excmd);
        if errormsg.is_none() {
            correct_range(excmd);
            ex_print(excmd);
        }
    } else if excmd.addr_count != 0 {
        excmd.line2 = excmd.line2.min(Buf::current().b_ml.ml_line_count);
        if excmd.line2 < 0 {
            errormsg = Some(ex_msg(e_invrange.as_ptr()));
        } else {
            // Line 0 is not a position; the cursor goes to line 1.
            Win::current().w_cursor.lnum = if excmd.line2 == 0 { 1 } else { excmd.line2 };
            beginline(BeginlineOpts::SOL | BeginlineOpts::FIX);
        }
    }
    errormsg
}

/// `msg`, with `cmd` appended after a colon: the "…: :bogus" form a
/// command-line error takes.
///
/// The result is capped at `IOSIZE` bytes, with the message elided to `...`
/// where it alone fills the buffer.
///
/// Truncates to fit, and spells U+00A0 as `<a0>` — it is white space that
/// would otherwise be invisible in the report, and it is a common paste
/// accident.
///
/// # Safety
///
/// `cmd` must point at a NUL-terminated string.
pub(crate) unsafe fn append_command(msg: &CStr, cmd: *const c_char) -> CString {
    let mut buf = [0 as c_char; IOSIZE as usize];
    let iobuff = buf.as_mut_ptr();
    unsafe { xstrlcpy(iobuff, msg.as_ptr(), IOSIZE as size_t) };
    let len = len_of(iobuff);
    if len > (IOSIZE - 100) as size_t {
        let mut d = unsafe { iobuff.add(IOSIZE as usize - 100) };
        d = unsafe { d.sub(utf_head_off(iobuff, d) as usize) };
        unsafe { strcpy(d, c"...".as_ptr() as *mut c_char) };
    }
    unsafe { xstrlcat(iobuff, c": ".as_ptr(), IOSIZE as size_t) };

    let mut s = cmd;
    let mut d = unsafe { iobuff.add(len_of(iobuff)) };
    while byte(s) != NUL && unsafe { d.offset_from(iobuff) } + 5 < IOSIZE as isize {
        if ubyte_at(s, 0) == 0xc2 && ubyte_at(s, 1) == 0xa0 {
            s = unsafe { s.add(2) };
            unsafe { strcpy(d, c"<a0>".as_ptr() as *mut c_char) };
            d = unsafe { d.add(4) };
        } else {
            if unsafe { d.offset_from(iobuff) } + unsafe { utfc_ptr2len(s) } as isize + 1
                >= IOSIZE as isize
            {
                break;
            }
            unsafe { mb_copy_char(&raw mut s, &raw mut d) };
        }
    }
    unsafe { *d = NUL as c_char };
    cstr::in_chars(&buf).to_owned()
}

/// What [`ex_ni`] and [`ex_script_ni`] report.
const E_NOT_IN_THIS_BUILD: &CStr = c"E319: The command is not available in this version";

/// The handler every command this build does not implement runs.
///
/// Keeps the raw signature: it is a `cmd_func` in the command table, and
/// `is_cmd_ni` recognises a command by comparing against its address.
pub fn ex_ni(excmd: &mut ExArg) {
    if !excmd.skip {
        excmd.errmsg = Some(ex_msg(E_NOT_IN_THIS_BUILD.as_ptr()));
    }
}

/// The same, for a command whose argument may be a here-document
/// (`:perl <<EOF`) — the body has to be consumed even when the command
/// cannot run, or its lines would be read as commands.
pub(crate) fn ex_script_ni(excmd: &mut ExArg) {
    if !excmd.skip {
        ex_ni(excmd);
    } else {
        let mut len: size_t = 0;
        unsafe { xfree(script_get(excmd, &raw mut len) as *mut c_void) };
    }
}

/// `curbuf_locked()` as checked code.
fn curbuf_locked() -> bool {
    crate::ex_getln::curbuf_locked()
}

/// `ex_msg()` as checked code.
fn ex_msg(msg: *const c_char) -> CString {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::ex_docmd::ex_msg(msg) }
}

/// `getline_equal()` as checked code.
fn getline_equal(fgetline: LineGetter, cookie: *mut c_void, func: LineGetter) -> bool {
    // SAFETY: the pointers are the command line's own, and live for the call.
    unsafe { crate::ex_docmd::source::getline_equal(fgetline, cookie, func) }
}

/// `invalid_range()` as checked code.
fn invalid_range(excmd: &mut ExArg) -> Option<CString> {
    // SAFETY: the pointers are the command line's own, and live for the call.
    crate::ex_docmd::address::invalid_range(excmd)
}

/// `skipwhite()` as checked code.
fn skipwhite(p: *const c_char) -> *mut c_char {
    // SAFETY: a NUL-terminated string.
    unsafe { crate::charset::skipwhite(p) }
}

/// `xcalloc()` as checked code.
fn xcalloc(count: usize, size: usize) -> *mut c_void {
    // SAFETY: reads the editor's own state, which exists from startup to exit.
    unsafe { crate::memory::xcalloc(count, size) }
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

/// The byte at `p[i]`, unsigned, as the C's `(uint8_t)*(p + i)` reads it.
fn ubyte_at(p: *const c_char, i: isize) -> u8 {
    // SAFETY: an offset within the NUL-terminated string `p` points into.
    unsafe { *p.offset(i) as u8 }
}

/// The length of the string at `s` -- `strlen`, as the slice's own `len()`
/// -- as checked code.
fn len_of(s: *const c_char) -> usize {
    // SAFETY: a NUL-terminated string.
    unsafe { cstr::bytes_at(s) }.len()
}
