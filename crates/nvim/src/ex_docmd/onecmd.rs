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
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use crate::autocmd::defs::AutoEvent;
use crate::ex_docmd::ex_msg;
use crate::memory::XString;
use crate::types::CmdIdx;
use crate::types::ColNr;
use crate::types::LineNr;
use core::ffi::{CStr, c_int};
use core::ptr;
use std::ffi::CString;

use crate::autocmd::{fire_autocmds_for, has_event};

use crate::debugger::dbg_check_breakpoint;
use crate::edit::{BeginlineOpts, beginline};
use crate::eval::userfunc::current_func_returned;
use crate::ex_docmd::address::{correct_range, find_excmd_after_range, parse_cmd_address};
use crate::ex_docmd::addrtype::set_cmd_dflall_range;

use crate::ex_docmd::api::execute_cmd0;
use crate::ex_docmd::argopt::{getargcmd, getargopt};
use crate::ex_docmd::edit::ex_print;
use crate::ex_docmd::filename::replace_makeprg;
use crate::ex_docmd::lookup::is_user_cmd;
use crate::ex_docmd::modifier::CmdModScope;
use crate::ex_docmd::scan::{get_flags, parse_bang, parse_register, separate_nextcmd, skip_colons};
use crate::ex_docmd::source::{ReadLines, SourceKind, ex_errmsg};

use crate::ex_docmd::verify::verify_command;
use crate::ex_docmd::{
    CMDNAMES, DoCmdOpts, E_AMBIGUOUS_USE_OF_USER_DEFINED_COMMAND, E_NOT_AN_EDITOR_COMMAND, ExFunc,
    PROF_YES, QUITMORE,
};
use crate::ex_eval::CsFlags;

use crate::ex_eval::{aborting, cond_stack_of, do_errthrow, do_intthrow, do_throw};
use crate::ex_getln::{get_text_locked_msg, script_get, text_locked};

use crate::ex_docmd::state::{DID_EMSG_SYNTAX, EX_NESTING_LEVEL, GLOBAL_BUSY};
use crate::ex_eval::state::{CHECK_CSTACK, DID_THROW, NEED_RETHROW};
use crate::getchar::state::{GOT_INT, PENDING_END_REG_EXECUTING, REG_EXECUTING};
use crate::guard::Depth;
use crate::guard::SANDBOX;
use crate::input::ask_yesno;
use crate::mbyte::{cluster_len, head_off};
use crate::message::state::{DID_EMSG, MSG_SILENT};
use crate::message::{
    E_ARGREQ, E_CMDWIN, E_INVARG, E_INVRANGE, E_MODIFIABLE, E_NOBANG, E_NORANGE, E_SANDBOX,
    E_TRAILING_ARG,
};
use crate::profile::DO_PROFILING;
use crate::startup::EXITING;
use crate::state::mode::EXMODE_ACTIVE;
use crate::winlayer::graph::CMDWIN_TYPE;

use crate::ex_eval::defs::CondId;
use crate::message::IOSIZE;
use crate::message::emsg;
use crate::os::cshim::gettext;
use crate::profile::{func_line_exec, script_line_exec};
use crate::runtime::do_finish;
use crate::types::{CmdAddr, CmdLine, ExArg, ExArgt};
use crate::winlayer::{Buf, Win};

/// A zeroed `ExArg` with the empty range the parsers start from.
///
/// `CmdIdx::append` and `CmdAddr::Lines` are both zero, so the only fields the C's
/// `(ExArg){ .line1 = 1, .line2 = 1 }` sets to anything else are the two
/// line numbers.
pub(crate) fn fresh_exarg<'s>() -> ExArg<'s> {
    // Both pairs: until a range is read they agree, so choosing the
    // address kind moves nothing (`ExArg::set_kind_unread`).
    ExArg {
        line1: LineNr::ONE,
        line2: LineNr::ONE,
        number1: 1,
        number2: 1,
        ..ExArg::default()
    }
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
        && (ex_func_is(CMDNAMES[cmdidx.index()].cmd_func, ex_ni)
            || ex_func_is(CMDNAMES[cmdidx.index()].cmd_func, ex_script_ni))
}

/// Drop the first of an API-supplied argument list, and point `eap->arg` at
/// what is left.
///
/// With no arguments left, `eap->arg` answers the end of the *old* first
/// argument rather than null — a command that reads `eap->arg` as a string
/// then sees an empty one.
pub(crate) fn shift_cmd_args(excmd: &mut ExArg) {
    debug_assert!(!excmd.line.args.is_empty());
    let (first, first_len) = excmd.line.args.remove(0);
    excmd.line.arg = match excmd.line.args.first() {
        Some(&(at, _)) => at,
        None => first + first_len,
    };
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

/// A stage of the parse that refused to go on.
///
/// Whatever it has to say is in the caller's `errormsg`; an empty one is a
/// silent refusal, which is what a skipped command and a locked buffer both
/// are. C spelled all of these `goto doend`.
#[derive(Debug)]
struct Refused;

impl ExArg<'_> {
    /// Read the modifiers and the range, and find the command word.
    ///
    /// Answers where the argument starts, which is what the rest of the parse
    /// walks. The command name has to be found *before* the range can be read,
    /// because it is what says whether an address counts lines, windows,
    /// buffers or tab pages -- so the two are one stage.
    ///
    /// The position is left in `self.arg`, which nothing has read yet.
    fn locate_command(
        &mut self,
        in_inactive: bool,
        mods: &CmdModScope,
        flags: DoCmdOpts,
        errormsg: &mut Option<CString>,
    ) -> Result<(), Refused> {
        // "#!anything" is a comment, so that a script can carry a shebang line.
        if self.line.byte_at(0) == b'#' && self.line.byte_at(1) == b'!' {
            return Err(Refused);
        }

        mods.parse(self, errormsg).map_err(|_| Refused)?;
        mods.apply();
        let after_modifier = self.line.cmd;

        let cond = cond_stack_of(self);
        self.skip = DID_EMSG.get() != 0 || GOT_INT.get() || DID_THROW.get() || in_inactive;

        let mut p = find_excmd_after_range(self);
        profile_cmd(self, cond);

        if !EXITING.get() {
            // May go to debug mode. If the `>quit` debug command is used there,
            // an interrupt exception is thrown and this command is skipped.
            dbg_check_breakpoint(self);
        }
        if !self.skip && GOT_INT.get() {
            self.skip = true;
            do_intthrow(cond);
        }

        self.set_addr_type(p.map(|at| self.line.byte_at(self.line.skip_white(at))));
        if let Err(refused) = parse_cmd_address(self, false) {
            refused.into_errormsg(errormsg);
            return Err(Refused);
        }

        self.line.cmd = skip_colons(&self.line, self.line.cmd, true);

        // A range with no command after it. Vi's behaviour, preserved: `:3`
        // jumps to line 3, `:3|…` *prints* line 3, and `:|` prints the current
        // line.
        if self.line.byte_at(self.line.cmd) == 0 || self.line.byte_at(self.line.cmd) == b'"' || {
            self.line.next = self.line.check_next(self.line.cmd);
            self.line.next.is_some()
        } {
            if !self.skip {
                debug_assert!(errormsg.is_none());
                *errormsg = ex_range_without_command(self);
            }
            return Err(Refused);
        }

        // An unknown command spelled like a user command, with a CmdUndefined
        // autocommand waiting to define it.
        if p.is_some()
            && self.cmdidx == CmdIdx::SIZE
            && !self.skip
            && self.line.byte_at(self.line.cmd).is_ascii_uppercase()
            && has_event(AutoEvent::CmdUndefined)
        {
            let mut end = self.line.cmd;
            while self.line.byte_at(end).is_ascii_alphanumeric() {
                end += 1;
            }
            let name = self.line.slice_at(self.line.cmd, end - self.line.cmd);
            let cmdname = XString::from_bytes(name);
            let event = AutoEvent::CmdUndefined;
            let ret = fire_autocmds_for(
                event,
                Some(cmdname.clone()),
                Some(cmdname),
                true,
                None,
                None,
            );
            // Look again only if the autocommands did something and did not
            // fail.
            p = if ret && !aborting() {
                self.find_command(None)
            } else {
                Some(self.line.cmd)
            };
        }

        let Some(p) = p else {
            if !self.skip {
                *errormsg = Some(ex_msg(E_AMBIGUOUS_USE_OF_USER_DEFINED_COMMAND));
            }
            return Err(Refused);
        };

        if self.cmdidx == CmdIdx::SIZE {
            if !self.skip {
                // The modifiers parsed, so the error is in what follows them.
                let msg = ex_msg(E_NOT_AN_EDITOR_COMMAND);
                *errormsg = Some(if flags.has(DoCmdOpts::VERBOSE) {
                    // The whole line is appended by `do_one_cmd` instead.
                    msg
                } else {
                    append_command(&msg, self.line.rest_of(after_modifier))
                });
                DID_EMSG_SYNTAX.set(true);
                verify_command(self.line.rest_of(after_modifier));
            }
            return Err(Refused);
        }
        self.line.arg = p;
        Ok(())
    }

    /// Everything about *where* the command may run: the buffer it would edit,
    /// the range it was given and the `!` it was not allowed.
    ///
    /// `ni` is set for a command this build does not implement, which relaxes
    /// every check here: there is nothing to check them against.
    fn check_may_run(
        &mut self,
        ni: bool,
        flags: DoCmdOpts,
        errormsg: &mut Option<CString>,
    ) -> Result<(), Refused> {
        if !self.skip {
            if let Some(msg) = refuses_here(self) {
                *errormsg = Some(msg);
                return Err(Refused);
            }
            // `CURBUF->b_ro_locked` forbids editing another buffer.
            // `:checktime` is postponed rather than refused, and `:edit` and
            // `:file` are checked again once their argument is known.
            if !self.argt.has(ExArgt::CMDWIN)
                && self.cmdidx != CmdIdx::checktime
                && self.cmdidx != CmdIdx::edit
                && self.cmdidx != CmdIdx::file
                && !is_user_cmd(self.cmdidx)
                && curbuf_locked()
            {
                return Err(Refused);
            }
            if !ni && !self.argt.has(ExArgt::RANGE) && self.addr_count > 0 {
                *errormsg = Some(ex_msg(E_NORANGE));
                return Err(Refused);
            }
        }

        if !ni && !self.argt.has(ExArgt::BANG) && self.forceit {
            *errormsg = Some(ex_msg(E_NOBANG));
            return Err(Refused);
        }

        // A range that is not used is not complained about, which can happen
        // when a line count is accidentally zero.
        if !self.skip && !ni && self.argt.has(ExArgt::RANGE) {
            // A backwards range is offered for swapping. `:global` is busy
            // running a command per line and would fail below anyway, so it is
            // not asked.
            let (first, last) = self.range_numbers();
            if GLOBAL_BUSY.get() == 0 && first > last {
                if MSG_SILENT.get() == 0 {
                    if flags.has(DoCmdOpts::VERBOSE) || EXMODE_ACTIVE.get() {
                        *errormsg = Some(ex_msg(c"E493: Backwards range given"));
                        return Err(Refused);
                    }
                    if ask_yesno(gettext(c"Backwards range given, OK to swap")) != c_int::from(b'y')
                    {
                        return Err(Refused);
                    }
                }
                self.set_range_numbers(last, first);
            }
            *errormsg = invalid_range(self);
            if errormsg.is_some() {
                return Err(Refused);
            }
        }

        // `CmdAddr::Other` counts from 1 rather than from the cursor.
        if self.addr_type == CmdAddr::Other && self.addr_count == 0 {
            self.number2 = 1;
        }
        correct_range(self);

        // Put the first line at the start of a closed fold and the last line at
        // its end.
        if (self.argt.has(ExArgt::WHOLEFOLD) || self.addr_count >= 2)
            && GLOBAL_BUSY.get() == 0
            && self.addr_type == CmdAddr::Lines
        {
            Win::current().has_folding(self.line1, Some(&mut self.line1), None);
            Win::current().has_folding(self.line2, None, Some(&mut self.line2));
        }
        Ok(())
    }

    /// Everything after the command word: the `++opt`s, the `+cmd`, the
    /// redirections `:write` and `:read` spell with `>`/`!`, the register and
    /// the count, and where the *next* command starts.
    ///
    /// On entry `self.arg` is where the command word ended, which is what
    /// [`ExArg::locate_command`] left there.
    fn read_command_args(
        &mut self,
        ni: bool,
        errormsg: &mut Option<CString>,
    ) -> Result<(), Refused> {
        // `:make` and `:grep` splice 'makeprg'/'grepprg' into the line here, so
        // that `%` and friends expand inside it.
        replace_makeprg(self);

        // `:!` keeps the space: `:!! -l` needs it.
        if self.cmdidx != CmdIdx::bang {
            self.line.arg = self.line.skip_white(self.line.arg);
        }

        if self.cmdidx == CmdIdx::file && self.line.byte_at(self.line.arg) != 0 && curbuf_locked() {
            return Err(Refused);
        }

        // `++opt=val` first, so that `:w ++enc=utf8 !cmd` works.
        if self.argt.has(ExArgt::ARGOPT) {
            while self.line.byte_at(self.line.arg) == b'+'
                && self.line.byte_at(self.line.arg + 1) == b'+'
            {
                if getargopt(self).is_err() && !ni {
                    *errormsg = Some(ex_msg(E_INVARG));
                    return Err(Refused);
                }
            }
        }

        if self.cmdidx == CmdIdx::write || self.cmdidx == CmdIdx::update {
            if self.line.byte_at(self.line.arg) == b'>' {
                self.line.arg += 1;
                if self.line.byte_at(self.line.arg) != b'>' {
                    *errormsg = Some(ex_msg(c"E494: Use w or w>>"));
                    return Err(Refused);
                }
                self.line.arg = self.line.skip_white(self.line.arg + 1);
                self.append = true;
            } else if self.line.byte_at(self.line.arg) == b'!' && self.cmdidx == CmdIdx::write {
                // `:w !filter`
                self.line.arg += 1;
                self.usefilter = true;
            }
        } else if self.cmdidx == CmdIdx::read {
            if self.forceit {
                // `:r!filter`
                self.usefilter = true;
                self.forceit = false;
            } else if self.line.byte_at(self.line.arg) == b'!' {
                // `:r !filter`
                self.line.arg += 1;
                self.usefilter = true;
            }
        } else if self.cmdidx == CmdIdx::lshift || self.cmdidx == CmdIdx::rshift {
            // How far to shift is how many `<` or `>` were typed.
            self.amount = 1;
            while self.line.byte_at(self.line.arg) == self.line.byte_at(self.line.cmd) {
                self.line.arg += 1;
                self.amount += 1;
            }
            self.line.arg = self.line.skip_white(self.line.arg);
        }

        // `+command`, before the next command is looked for. Not for
        // `:read !cmd` and `:write !cmd`.
        if self.argt.has(ExArgt::CMDARG) && !self.usefilter {
            // SAFETY: `arg` is the command's own cursor into its line.
            self.do_ecmd_cmd = getargcmd(self);
        }

        if self.argt.has(ExArgt::TRLBAR) && !self.usefilter {
            separate_nextcmd(self);
        } else if self.cmdidx == CmdIdx::bang
            || self.cmdidx == CmdIdx::terminal
            || self.cmdidx == CmdIdx::global
            || self.cmdidx == CmdIdx::vglobal
            || self.usefilter
        {
            separate_at_newline(self);
        }

        if self.argt.has(ExArgt::DFLALL) && self.addr_count == 0 {
            set_cmd_dflall_range(self);
        }

        parse_register(self);
        self.parse_count(errormsg, true).map_err(|_| Refused)?;

        if self.argt.has(ExArgt::FLAGS) {
            get_flags(self);
        }
        if !ni
            && !self.argt.has(ExArgt::EXTRA)
            && self.line.byte_at(self.line.arg) != 0
            && self.line.byte_at(self.line.arg) != b'"'
            && (self.line.byte_at(self.line.arg) != b'|' || !self.argt.has(ExArgt::TRLBAR))
        {
            *errormsg = Some(ex_errmsg(
                E_TRAILING_ARG,
                self.line.cstr_from(self.line.arg),
            ));
            return Err(Refused);
        }
        if !ni && self.argt.has(ExArgt::NEEDARG) && self.line.byte_at(self.line.arg) == 0 {
            *errormsg = Some(ex_msg(E_ARGREQ));
            return Err(Refused);
        }
        Ok(())
    }
}

/// A shell command ends at a newline rather than at a `|`, and one
/// backslash before that newline is removed.
fn separate_at_newline(excmd: &mut ExArg) {
    let mut at = excmd.line.arg;
    while excmd.line.byte_at(at) != 0 {
        if excmd.line.byte_at(at) == b'\\' && excmd.line.byte_at(at + 1) == b'\n' {
            // Drop the backslash, pulling the rest of the string over it.
            excmd.line.drop_byte(at);
        } else if excmd.line.byte_at(at) == b'\n' {
            excmd.line.next = Some(at + 1);
            excmd.line.terminate_at(at);
            break;
        }
        at += 1;
    }
}

/// Re-raise what a nested `do_cmdline` left for the *outer* conditional
/// stack: a throw, a `:return` or a `:finish`.
///
fn rethrow_from_nested(excmd: &mut ExArg) {
    let cond = cond_stack_of(excmd);
    if NEED_RETHROW.get() {
        do_throw(cond);
    } else if CHECK_CSTACK.get() {
        let finished = excmd
            .source
            .as_ref()
            .and_then(|source| source.script())
            .is_some_and(|cookie| cookie.finished);
        if finished {
            do_finish(excmd, true);
        } else if excmd.source_kind() == SourceKind::Function && current_func_returned() != 0 {
            excmd.return_from_function(true, None);
        }
    }
    CHECK_CSTACK.set(false);
    NEED_RETHROW.set(false);
}

/// Parse and execute one Ex command, and answer where the next one starts.
///
/// `source` is where the command may read further lines from (`:append`, a
/// `:function` body, a sourced file), if anywhere. Re-entrant: a command
/// that calls `do_cmdline` lands back here.
///
/// `line` is taken over for the call and handed back: a command may
/// *replace* the buffer under itself (`:make` splices in 'makeprg', an
/// expanded `%` grows it, a continued `:let` swaps in the joined line), and
/// the answer is the line the caller must go on running.
///
/// `in_inactive` says the innermost conditional of `cond` is not active, as
/// the caller last saw it.
pub(crate) fn do_one_cmd(
    line: CmdLine,
    flags: DoCmdOpts,
    cond: CondId,
    in_inactive: bool,
    source: Option<&mut dyn ReadLines>,
) -> CmdLine {
    let mut errormsg: Option<CString> = None;
    let save_reg_executing = REG_EXECUTING.get();
    let save_pending_end_reg_executing = PENDING_END_REG_EXECUTING.get();
    let mut excmd = fresh_exarg();
    let nesting = Depth::of(EX_NESTING_LEVEL);

    // When the last file has not been edited `:q` has to be typed twice.
    // A `'statusline'` function call and an autocommand (QuitPre) both
    // reach here without the user having typed anything, so neither
    // spends the second `:q`.
    let kind = source
        .as_ref()
        .map_or(SourceKind::None, |source| source.kind());
    if QUITMORE.get() != 0 && kind != SourceKind::Function && kind != SourceKind::Autocmd {
        QUITMORE.set(QUITMORE.get() - 1);
    }

    // Modifiers are restored on the way out, for recursive calls. The
    // guard owns the `:filter` pattern and program of the set it took
    // out until it puts them back.
    let mods = CmdModScope::cleared();

    excmd.line = line;
    excmd.source = source;
    excmd.cond_stack = Some(cond);

    // Each stage refuses by answering `Err`, having left whatever it has to
    // say in `errormsg`; the reporting below is shared by all of them.
    let ran = (|| {
        excmd.locate_command(in_inactive, &mods, flags, &mut errormsg)?;

        // Not implemented in this build: the argument checks are relaxed,
        // because there is nothing to check them against.
        let ni = is_cmd_ni(excmd.cmdidx);

        // The bang is read through a cursor of its own: the command is lent
        // to the scan, so its `arg` cannot be lent as well.
        let mut cursor = excmd.line.arg;
        excmd.forceit = parse_bang(&mut excmd, &mut cursor);
        excmd.line.arg = cursor;
        if !is_user_cmd(excmd.cmdidx) {
            excmd.argt = CMDNAMES[excmd.cmdidx.index()].cmd_argt;
        }

        excmd.check_may_run(ni, flags, &mut errormsg)?;
        excmd.read_command_args(ni, &mut errormsg)?;

        if skip_cmd(&mut excmd) {
            return Err(Refused);
        }
        let mut retv: c_int = 0;
        execute_cmd0(&mut retv, &mut excmd, &mut errormsg, false).map_err(|_| Refused)
    })();
    if ran.is_ok() {
        rethrow_from_nested(&mut excmd);
    }

    // Can happen with a zero line number.
    if Win::current().w_cursor.lnum == LineNr::ZERO {
        Win::current().w_cursor.lnum = LineNr::ONE;
        Win::current().w_cursor.col = ColNr::ZERO;
    }

    if let Some(msg) = errormsg
        && !msg.is_empty()
        && DID_EMSG.get() == 0
    {
        let msg = if flags.has(DoCmdOpts::VERBOSE) {
            append_command(&msg, excmd.line.line())
        } else {
            msg
        };
        emsg(&msg);
    }
    let cmdidx = excmd.cmdidx;
    do_errthrow(Some(cond), || {
        (cmdidx != CmdIdx::SIZE && !is_user_cmd(cmdidx))
            .then(|| crate::ex_docmd::builtin_command_cstr(cmdidx))
    });

    drop(mods);
    REG_EXECUTING.set(save_reg_executing);
    PENDING_END_REG_EXECUTING.set(save_pending_end_reg_executing);

    // A trailing bar with nothing after it is not really a next command.
    if excmd
        .line
        .next
        .is_some_and(|at| excmd.line.byte_at(at) == 0)
    {
        excmd.line.next = None;
    }

    drop(nesting);
    excmd.line
}

/// Count this line for `:profile`, if profiling is on and the line is one
/// that will really run.
///
/// The `skip` this recomputes is not `args.skip`: a `:catch` that is about
/// to be entered, an `:else` whose branch is about to be taken and a
/// `:finally` all execute even though the surrounding construct is
/// inactive, and each is worth a profile sample.
pub(crate) fn profile_cmd(excmd: &ExArg, cond: CondId) {
    if DO_PROFILING.get() != PROF_YES {
        return;
    }
    let (idx, top, below) = cond.with(|cs| {
        let top = cs.top().map(|at| cs.flags[at]);
        let below = cs
            .top()
            .and_then(|at| at.checked_sub(1))
            .map(|at| cs.flags[at]);
        (cs.idx, top, below)
    });
    if !(!excmd.skip || idx == 0 || below.is_some_and(|flags| flags.has(CsFlags::ACTIVE))) {
        return;
    }
    let mut skip = DID_EMSG.get() != 0 || GOT_INT.get() || DID_THROW.get();
    match excmd.cmdidx {
        CmdIdx::catch => {
            skip = !skip
                && !top
                    .is_some_and(|flags| flags.has(CsFlags::THROWN) && !flags.has(CsFlags::CAUGHT));
        }
        CmdIdx::r#else | CmdIdx::elseif => {
            skip = skip || !top.is_some_and(|flags| !flags.has(CsFlags::ACTIVE | CsFlags::TRUE));
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
    match excmd.source.as_ref() {
        Some(source) if source.kind() == SourceKind::Function => {
            if let Some(frame) = source.function() {
                func_line_exec(frame);
            }
        }
        Some(source) if source.kind() == SourceKind::Script => script_line_exec(),
        _ => {}
    }
}

/// The three "this command is not allowed here" checks that share an exit.
///
/// Answers the message to report, or `None` when the command may run.
fn refuses_here(excmd: &ExArg) -> Option<CString> {
    if SANDBOX.get() != 0 && !excmd.argt.has(ExArgt::SBOXOK) {
        return Some(ex_msg(E_SANDBOX));
    }
    // `:put` is allowed in a terminal buffer, which is not 'modifiable'.
    if Buf::current().b_p_ma == 0
        && excmd.argt.has(ExArgt::MODIFY)
        && !(!Buf::current().terminal.is_null()
            && (excmd.cmdidx == CmdIdx::put || excmd.cmdidx == CmdIdx::iput))
    {
        return Some(ex_msg(E_MODIFIABLE));
    }
    if !is_user_cmd(excmd.cmdidx) {
        if CMDWIN_TYPE.get() != 0 && !excmd.argt.has(ExArgt::CMDWIN) {
            return Some(ex_msg(E_CMDWIN));
        }
        if text_locked() && !excmd.argt.has(ExArgt::LOCK_OK) {
            return Some(ex_msg(get_text_locked_msg()));
        }
    }
    None
}

/// A range with no command after it: print the lines, or move the cursor to
/// the last of them.
///
/// Which of the two it is depends on how the line ended — a `|` after the
/// range, or Ex mode, means print. The one exception is the `+` Ex mode
/// substitutes for an empty line once its `+` has been consumed, which
/// `CmdLine::substituted` marks: upstream recognises it by the *address* of
/// the static it substituted.
pub(crate) fn ex_range_without_command(excmd: &mut ExArg) -> Option<CString> {
    let mut errormsg: Option<CString> = None;
    let past_substituted_plus = excmd.line.substituted && excmd.line.cmd == 1;
    if excmd.line.byte_at(excmd.line.cmd) == b'|' || (EXMODE_ACTIVE.get() && !past_substituted_plus)
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
        if excmd.line2 < LineNr::ZERO {
            errormsg = Some(ex_msg(E_INVRANGE));
        } else {
            // Line 0 is not a position; the cursor goes to line 1.
            Win::current().w_cursor.lnum = if excmd.line2 == LineNr::ZERO {
                LineNr::ONE
            } else {
                excmd.line2
            };
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
pub(crate) fn append_command(msg: &CStr, cmd: &[u8]) -> CString {
    // Upstream builds this in `IObuff`: `IOSIZE` bytes, the NUL included.
    let room = IOSIZE as usize;
    let mut out = msg.to_bytes().to_vec();
    out.truncate(room - 1);
    if out.len() > room - 100 {
        // Kept with its terminator: `head_off` decodes up to it.
        out.push(0);
        let cut = room - 100 - head_off(&out, room - 100);
        out.truncate(cut);
        out.extend_from_slice(b"...");
    }
    for &b in b": " {
        if out.len() < room - 1 {
            out.push(b);
        }
    }

    // A NUL inside the command ends it, as the terminator did upstream.
    let cmd = &cmd[..cmd.iter().position(|&b| b == 0).unwrap_or(cmd.len())];
    let mut at = 0;
    while at < cmd.len() && out.len() + 5 < room {
        if cmd[at] == 0xc2 && cmd.get(at + 1) == Some(&0xa0) {
            at += 2;
            out.extend_from_slice(b"<a0>");
        } else {
            let len = cluster_len(&cmd[at..]);
            if out.len() + len + 1 >= room {
                break;
            }
            out.extend_from_slice(&cmd[at..at + len]);
            at += len;
        }
    }
    CString::new(out).expect("no NUL is copied in")
}

/// What [`ex_ni`] and [`ex_script_ni`] report.
const E_NOT_IN_THIS_BUILD: &CStr = c"E319: The command is not available in this version";

/// The handler every command this build does not implement runs.
///
/// Keeps the raw signature: it is a `cmd_func` in the command table, and
/// `is_cmd_ni` recognises a command by comparing against its address.
pub fn ex_ni(excmd: &mut ExArg) {
    if !excmd.skip {
        excmd.errmsg = Some(ex_msg(E_NOT_IN_THIS_BUILD));
    }
}

/// The same, for a command whose argument may be a here-document
/// (`:perl <<EOF`) — the body has to be consumed even when the command
/// cannot run, or its lines would be read as commands.
pub(crate) fn ex_script_ni(excmd: &mut ExArg) {
    if !excmd.skip {
        ex_ni(excmd);
    } else {
        drop(script_get(excmd));
    }
}

/// `curbuf_locked()` as checked code.
fn curbuf_locked() -> bool {
    crate::ex_getln::curbuf_locked()
}

/// `invalid_range()` as checked code.
fn invalid_range(excmd: &mut ExArg) -> Option<CString> {
    // SAFETY: the pointers are the command line's own, and live for the call.
    crate::ex_docmd::address::invalid_range(excmd)
}
