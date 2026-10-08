//! Vimscript control flow: `:if`, `:while`, `:for`, the try conditional, and
//! the exception machinery underneath all three.
//!
//! Everything here is state on one stack, [`CondStack`], which each running
//! command line opens ([`OwnedCondStack`]) and its commands reach by the id
//! `ExArg::cond_stack` carries. Each `:if`/`:while`/`:for`/`:try` pushes a
//! level; the matching end command pops it. A level's `flags` say what it
//! is ([`CsFlags::WHILE`], [`CsFlags::TRY`], ...) and how it stands:
//! `CsFlags::ACTIVE` means its commands are being *executed* rather than
//! merely parsed, and `CsFlags::TRUE` means the condition was met at least
//! once, which is what tells `:endif` whether to show a debug prompt and
//! `:finally` whether its clause needs running at all.
//!
//! **Skipping is not the same as not executing.** A command inside an
//! inactive conditional is still parsed, because the parser has to find the
//! matching `:endif` -- so almost everything below starts with
//! [`check_skip`], and errors detected while skipping are mostly, but not
//! entirely, ignored.
//!
//! The stack is borrowed a step at a time and never across a call that can
//! run user code (an expression, a debug prompt, a message): that code may
//! run a command line of its own, with a stack of its own beside this one.
//!
//! The three parts:
//!
//! - Here: the conditional stack itself, the `:if`/`:while`/`:for` family,
//!   and the two operations everything else needs on that stack --
//!   [`cleanup_conditionals`], which deactivates levels down to the one
//!   being looked for and discards what their finally clauses had pending,
//!   and [`rewind_conditionals`], which pops them.
//! - [`trycmd`]: `:try`/`:catch`/`:finally`/`:endtry`/`:throw` and the
//!   cleanup pair.
//! - [`exception`]: the exception object -- how an error, an interrupt or a
//!   `:throw` becomes a catchable value.
//!
//! The four predicates at the top ([`aborting`] and friends) are what the
//! rest of the editor asks: "did this fail in a way that should stop the
//! script". They are deliberately delicate -- see [`exception`] for why
//! `force_abort` is held off until the throw point.
//!
//! Original: `src/nvim/ex_eval.c`, Vim/Neovim, Vim license.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

mod exception;
mod stack;
pub(crate) mod state;
#[cfg(test)]
mod tests;
mod trycmd;

use crate::debugger::dbg_check_skipped;
use crate::eval::typval::TV_INITIAL_VALUE;
use crate::eval::typval::tv_clear;
use crate::eval::{eval_cmd_bool, eval_for_line, eval0_in_cmd, next_for_item};
use crate::ex_docmd::{ends_excmd, modifier_len};
use crate::ex_eval::state::{did_endif, did_throw, force_abort, trylevel};
use crate::getchar::state::got_int;
use crate::global_cell::GlobalCell;
use crate::message::state::{did_emsg, emsg_silent};
use crate::message::{e_endfor, e_endif, e_endtry, e_endwhile, e_for, e_while};
use crate::message_fmt::msg_bytes;
use crate::semsg;
use crate::types::CmdIdx;
use crate::types::{CondId, CondStack, ExArg, FAIL, Failed, OK, Pend};
use core::ffi::{CStr, c_int};
use std::ffi::CString;

use flag::{CSTP_BREAK, CSTP_CONTINUE, CSTP_FINISH, CSTP_NONE, CSTP_RETURN, CSTP_THROW};

crate::flag_set! {
    /// `CondStack::flags`: what a conditional stack level is, and how it
    /// stands. The first two are the state; the rest name the command.
    pub(crate) struct CsFlags;

    /// The condition held -- for a `:while`, on the iteration being set up.
    const TRUE = 1;
    /// The level's commands are being *executed* rather than merely parsed.
    const ACTIVE = 2;
    const ELSE = 4;
    const WHILE = 8;
    const FOR = 16;
    const TRY = 256;
    const FINALLY = 512;
    /// An exception was thrown and this `:try` should check its `:catch`es.
    const THROWN = 2048;
    /// One of them matched.
    const CAUGHT = 4096;
    /// And that catch clause has ended.
    const FINISHED = 8192;
    /// This `:try` reset `emsg_silent`; the old value is on
    /// `saved_emsg_silent`.
    const SILENT = 16384;

    /// Either loop command, which is how every caller that cares about
    /// loops asks -- `cleanup_conditionals` and `rewind_conditionals` take
    /// this, or [`Self::TRY`], or nothing.
    const LOOP = Self::WHILE.bits() | Self::FOR.bits();
}

crate::flag_set! {
    /// `CondStack::loop_flags`: what `do_cmdline` should do next about the
    /// innermost loop.
    pub(crate) struct CsLoopFlags;

    const HAD_LOOP = 1;
    const HAD_ENDLOOP = 2;
    const HAD_CONT = 4;
    const HAD_FINA = 8;
}

pub(crate) use exception::{
    PendingAction, PendingValue, cause_errthrow, discard_current_exception, do_errthrow,
    do_intthrow, error_exception_string, exception_state_clear, exception_state_restore,
    exception_state_save, pop_msg_list, push_msg_list, report_pending, take_msg_list,
};
pub(crate) use stack::OwnedCondStack;
pub(crate) use trycmd::{
    CleanupGuard, do_throw, enter_cleanup, ex_catch, ex_endtry, ex_finally, ex_throw, ex_try,
    leave_cleanup,
};

/// Constants the transpiler copied in from the headers this module includes.
pub(crate) mod flag {
    use super::c_int;
    use crate::types::{EStackArg, ExceptType};

    /// `CondStack::pending`: what a finally clause postponed. The last
    /// three are alternatives, not bits -- `CSTP_RETURN` deliberately
    /// overlaps `CSTP_BREAK | CSTP_CONTINUE`, as upstream defines it.
    pub(crate) const CSTP_NONE: c_int = 0;
    pub(crate) const CSTP_ERROR: c_int = 1;
    pub(crate) const CSTP_INTERRUPT: c_int = 2;
    pub(crate) const CSTP_THROW: c_int = 4;
    pub(crate) const CSTP_BREAK: c_int = 8;
    pub(crate) const CSTP_CONTINUE: c_int = 16;
    pub(crate) const CSTP_RETURN: c_int = 24;
    pub(crate) const CSTP_FINISH: c_int = 32;

    /// `Exception.type_0`.
    pub(crate) const ET_USER: ExceptType = 0;
    pub(crate) const ET_ERROR: ExceptType = 1;
    pub(crate) const ET_INTERRUPT: ExceptType = 2;

    /// Whether an error under an active try conditional becomes a catchable
    /// exception rather than terminating the script after the finally
    /// clauses. True for a Vim release; upstream keeps the switch for its
    /// `THROW_TEST` builds, which this tree does not have.
    pub(crate) const THROW_ON_ERROR: bool = true;

    pub(crate) const ESTACK_NONE: EStackArg = 0;
}

const E_MULTIPLE_ELSE: &CStr = c"E583: Multiple :else";
const E_MULTIPLE_FINALLY: &CStr = c"E607: Multiple :finally";

/// Set while several errors appear in a row, delaying `force_abort` until
/// the failing command has returned. Aborting an expression evaluation
/// produces no error messages of its own, but every parsing error inside it
/// is still reported -- even under an active try conditional -- and this is
/// what keeps [`aborting`] answering the same thing throughout.
static cause_abort: GlobalCell<bool> = GlobalCell::new(false);

/// The address of a message constant, for the identity test in
/// [`cause_errthrow`](exception::cause_errthrow).
fn message(msg: &'static CStr) -> &'static CStr {
    msg
}

/// A message constant as an owned `eap->errmsg`.
fn err_msg(msg: &'static CStr) -> Option<CString> {
    Some(msg.to_owned())
}

/// The condition stack of the command line `excmd` runs in.
///
/// # Panics
/// When the command was not run from a command line: only the commands that
/// open or close a conditional ask, and they are only run from one.
pub(crate) fn cond_stack_of(excmd: &ExArg) -> CondId {
    excmd
        .cond_stack
        .expect("a command run from a command line has its condition stack")
}

/// Do not do something after an error, an interrupt or a throw, nor when the
/// surrounding conditional was not active. Upstream's `CHECK_SKIP`.
fn check_skip(cond: CondId) -> bool {
    did_emsg.get() != 0
        || got_int.get()
        || did_throw.get()
        || cond.with(|cs| cs.idx > 0 && !cs.flags[cs.top().expect("open") - 1].has(CsFlags::ACTIVE))
}

/// Whether to abort immediately: an error while aborting, an interrupt, or
/// an exception thrown and not yet caught.
///
/// Used by `:{range}call` to decide whether an aborted function that does
/// not handle a range itself should be called again for the next line, and
/// to cancel expression evaluation after a function call aborted. Note that
/// the first `emsg` call temporarily resets `force_abort` until the throw
/// point is reached, so that during such a cancellation this keeps answering
/// the same thing. `got_int` is also set by `interrupt()`.
pub(crate) fn aborting() -> bool {
    (did_emsg.get() != 0 && force_abort.get()) || got_int.get() || did_throw.get()
}

/// Put `force_abort` back, when it must be restored before the throw point
/// for the error message has been reached. See [`aborting`].
pub(crate) fn update_force_abort() {
    if cause_abort.get() {
        force_abort.set(true);
    }
}

/// Whether a command whose subcommand returned `retcode` should abort the
/// script. Lets an autocommand be suppressed after a failing subcommand, as
/// long as the error message has not been shown and so has not itself caused
/// the abort.
pub(crate) fn should_abort(retcode: c_int) -> bool {
    (retcode == FAIL && trylevel.get() != 0 && emsg_silent.get() == 0) || aborting()
}

/// [`should_abort`] over an answer that has already become a `Result`.
pub(crate) fn should_abort_err<T>(answer: Result<T, Failed>) -> bool {
    should_abort(if answer.is_ok() { OK } else { FAIL })
}

/// Whether a function with the "abort" flag should not count as ended on an
/// error -- parsing continues, to find finally clauses to execute, and some
/// errors in skipped commands are still reported.
pub(crate) fn aborted_in_try() -> bool {
    // Only called after an error, where `force_abort` decides whether the
    // search for finally clauses is needed.
    force_abort.get()
}

/// `:eval {expr}`
pub(crate) fn ex_eval(excmd: &mut ExArg) {
    let mut tv = TV_INITIAL_VALUE;
    let (at, evaluate) = (excmd.line.arg, !excmd.skip);
    if eval0_in_cmd(excmd, at, &mut tv, evaluate).is_ok() {
        tv_clear(&mut tv);
    }
}

/// `:if {expr}`
pub(crate) fn ex_if(excmd: &mut ExArg) {
    let cond = cond_stack_of(excmd);
    let Some(at) = cond.with(|cs| {
        (!cs.is_full()).then(|| {
            let at = cs.push();
            cs.flags[at] = CsFlags::NONE;
            at
        })
    }) else {
        excmd.errmsg = Some(c"E579: :if nesting too deep".to_owned());
        return;
    };

    let skip = check_skip(cond);
    let answer = eval_cmd_bool(excmd, skip);
    let (result, error) = (answer == Ok(true), answer.is_err());

    let flags = if skip || error {
        // Set TRUE, so this conditional never becomes active.
        CsFlags::TRUE
    } else if result {
        CsFlags::ACTIVE | CsFlags::TRUE
    } else {
        CsFlags::NONE
    };
    cond.with(|cs| cs.flags[at] = flags);
}

/// `:endif`
pub(crate) fn ex_endif(excmd: &mut ExArg) {
    did_endif.set(true);
    let cond = cond_stack_of(excmd);
    let Some(flags) = cond.with(|cs| cs.top().map(|at| cs.flags[at])) else {
        excmd.errmsg = Some(c"E580: :endif without :if".to_owned());
        return;
    };
    if flags.has(CsFlags::LOOP | CsFlags::TRY) {
        excmd.errmsg = Some(c"E580: :endif without :if".to_owned());
        return;
    }
    // When debugging or at a breakpoint, show the prompt if it has not
    // been shown: this tells the user that an ":endif" runs when the
    // ":if" or a previous ":elseif" was not TRUE. A ">quit" counts as an
    // interrupt before the ":endif", so throw an interrupt exception if
    // appropriate -- doing it here stops the exception for a parsing
    // error being discarded by that interrupt exception later on.
    if !flags.has(CsFlags::TRUE) && dbg_check_skipped(excmd) {
        do_intthrow(cond);
    }
    cond.with(|cs| cs.idx -= 1);
}

/// `:else` and `:elseif {expr}`
pub(crate) fn ex_else(excmd: &mut ExArg) {
    let cond = cond_stack_of(excmd);
    let mut skip = check_skip(cond);

    let top = cond.with(|cs| cs.top().map(|at| (at, cs.flags[at])));
    let in_if = top.filter(|(_, flags)| !flags.has(CsFlags::LOOP | CsFlags::TRY));
    if in_if.is_none() {
        if excmd.cmdidx == CmdIdx::r#else {
            excmd.errmsg = Some(c"E581: :else without :if".to_owned());
            return;
        }
        excmd.errmsg = Some(c"E582: :elseif without :if".to_owned());
        skip = true;
    } else if in_if.is_some_and(|(_, flags)| flags.has(CsFlags::ELSE)) {
        if excmd.cmdidx == CmdIdx::r#else {
            excmd.errmsg = Some(E_MULTIPLE_ELSE.to_owned());
            return;
        }
        excmd.errmsg = Some(c"E584: :elseif after :else".to_owned());
        skip = true;
    }

    // With nothing open there is no level to write: every write below is
    // behind a test that an error already failed.
    let Some((at, flags)) = top else {
        return else_without_level(excmd, skip);
    };
    // Skipping, or the ":if" was TRUE: reset ACTIVE. Otherwise set it.
    if skip || flags.has(CsFlags::TRUE) {
        if excmd.errmsg.is_none() {
            cond.with(|cs| cs.flags[at] = CsFlags::TRUE);
        }
        // Don't evaluate an ":elseif".
        skip = true;
    } else {
        cond.with(|cs| cs.flags[at] = CsFlags::ACTIVE);
    }

    // When debugging or at a breakpoint, show the prompt if it has not
    // been shown: this tells the user that an ":else"/":elseif" runs
    // when the ":if" or a previous ":elseif" was not TRUE. A ">quit"
    // counts as an interrupt before it, so set "skip" and throw an
    // interrupt exception -- doing it here stops the exception for a
    // parsing error being discarded by that interrupt exception later.
    if !skip && dbg_check_skipped(excmd) && got_int.get() {
        do_intthrow(cond);
        skip = true;
    }

    if excmd.cmdidx != CmdIdx::elseif {
        cond.with(|cs| cs.flags[at] |= CsFlags::ELSE);
        return;
    }

    let (result, error) = elseif_condition(excmd, skip);

    // The first of several errors in a row is the one to throw. That is
    // what happens when a conditional error was found above and parsing
    // the expression then failed too: "skip" is set in that case, so
    // `emsg` ignores the parsing error.
    if !skip && !error {
        let flags = if result {
            CsFlags::ACTIVE | CsFlags::TRUE
        } else {
            CsFlags::NONE
        };
        cond.with(|cs| cs.flags[at] = flags);
    } else if excmd.errmsg.is_none() {
        // Set TRUE, so this conditional never becomes active.
        cond.with(|cs| cs.flags[at] = CsFlags::TRUE);
    }
}

/// The rest of an `:else`/`:elseif` with no level open at all: the error is
/// set and `skip` is, so all that is left is parsing an `:elseif`'s
/// expression.
fn else_without_level(excmd: &mut ExArg, skip: bool) {
    if excmd.cmdidx == CmdIdx::elseif {
        let _ = elseif_condition(excmd, skip);
    }
}

/// Evaluate (or, skipping, parse) an `:elseif`'s expression: whether it held,
/// and whether it failed.
fn elseif_condition(excmd: &mut ExArg, skip: bool) -> (bool, bool) {
    // While skipping most errors are ignored, but a missing expression
    // is wrong -- perhaps it should have been ":else". A double quote
    // here starts a string, it is not a comment.
    if skip
        && excmd.line.byte_at(excmd.line.arg) != b'"'
        && ends_excmd(c_int::from(excmd.line.byte_at(excmd.line.arg))) != 0
    {
        let arg = msg_bytes(excmd.line.arg());
        semsg!("E15: Invalid expression: \"{arg}\"");
        (false, false)
    } else {
        let answer = eval_cmd_bool(excmd, skip);
        (answer == Ok(true), answer.is_err())
    }
}

/// `:while {expr}` and `:for {var} in {expr}`
pub(crate) fn ex_while(excmd: &mut ExArg) {
    let cond = cond_stack_of(excmd);
    let is_while = excmd.cmdidx == CmdIdx::r#while;
    let kind = if is_while {
        CsFlags::WHILE
    } else {
        CsFlags::FOR
    };
    // The loop flag is set when we jumped back from the matching
    // ":endwhile"/":endfor". When it is not set, this level needs
    // initialising. The depth is checked first, as upstream does, so the
    // 50th level is refused when it comes round again too.
    let Some((at, jumped_back)) = cond.with(|cs| {
        if cs.is_full() {
            return None;
        }
        let jumped_back = cs.loop_flags.has(CsLoopFlags::HAD_LOOP);
        if !jumped_back {
            let at = cs.push();
            cs.loop_level += 1;
            cs.line[at] = -1;
        }
        let at = cs.top().expect("a loop level is open");
        cs.flags[at] = kind;
        Some((at, jumped_back))
    }) else {
        excmd.errmsg = Some(c"E585: :while/:for nesting too deep".to_owned());
        return;
    };

    let skip = check_skip(cond);
    let (result, error) = if is_while {
        let answer = eval_cmd_bool(excmd, skip);
        (answer == Ok(true), answer.is_err())
    } else {
        for_next_item(excmd, cond, at, jumped_back, skip)
    };

    cond.with(|cs| {
        if !skip && !error && result {
            cs.flags[at] |= CsFlags::ACTIVE | CsFlags::TRUE;
            cs.loop_flags.toggle(CsLoopFlags::HAD_LOOP);
        } else {
            cs.loop_flags.clear(CsLoopFlags::HAD_LOOP);
            // The ":while" was FALSE or the ":for" ran off the end of the
            // list: show the debug prompt at the ":endwhile"/":endfor" as
            // if there had been a ":break" in a TRUE loop.
            if !skip && !error {
                cs.flags[at] |= CsFlags::TRUE;
            }
        }
    });
}

/// The `:for` half of [`ex_while`]: evaluate the list on the first pass,
/// then take the next element off it. Answers whether there was one, and
/// whether the header failed.
///
/// The iteration is out of the stack while the item is assigned: that runs
/// the targets' index expressions, which are user code.
fn for_next_item(
    excmd: &mut ExArg,
    cond: CondId,
    at: usize,
    jumped_back: bool,
    skip: bool,
) -> (bool, bool) {
    let (info, error) = if jumped_back {
        // Jumped here from a ":continue" or ":endfor": reuse the list
        // that was evaluated then.
        (cond.with(|cs| cs.for_info[at].take()), false)
    } else {
        // A level is opened afresh only after its last `:for` was dropped.
        let (info, error) = eval_for_line(excmd, skip);
        (Some(info), error)
    };

    // Use the element at the start of the list and advance.
    let mut info = info;
    let result = !error
        && !skip
        && info
            .as_deref_mut()
            .is_some_and(|info| next_for_item(info, excmd.line.rest_of(excmd.line.arg)));
    if result {
        cond.with(|cs| cs.for_info[at] = info);
    } else {
        drop(info);
    }
    (result, error)
}

/// `:continue`
pub(crate) fn ex_continue(excmd: &mut ExArg) {
    let cond = cond_stack_of(excmd);
    if cond.with(|cs| cs.loop_level <= 0 || cs.idx < 0) {
        excmd.errmsg = Some(c"E586: :continue without :while or :for".to_owned());
        return;
    }
    // Find the matching ":while". This may stop at a try conditional not
    // in its finally clause, which is then what runs next, so deactivate
    // every conditional except the ":while" itself, if it is reached.
    let at = cleanup_conditionals(cond, CsFlags::LOOP, false).expect("a :continue finds a level");
    if cond.with(|cs| cs.flags[at].has(CsFlags::LOOP)) {
        rewind_conditionals(cond, Some(at), CsFlags::TRY);
        // Let `do_cmdline` jump back to the matching ":while".
        cond.with(|cs| cs.loop_flags |= CsLoopFlags::HAD_CONT);
    } else {
        // A try conditional not in its finally clause came first: make
        // the ":continue" pending until the ":endtry".
        cond.with(|cs| cs.pending[at] = CSTP_CONTINUE);
        report_pending(PendingAction::Made, CSTP_CONTINUE, PendingValue::None);
    }
}

/// `:break`
pub(crate) fn ex_break(excmd: &mut ExArg) {
    let cond = cond_stack_of(excmd);
    if cond.with(|cs| cs.loop_level <= 0 || cs.idx < 0) {
        excmd.errmsg = Some(c"E587: :break without :while or :for".to_owned());
        return;
    }
    // Deactivate conditionals until the matching ":while" or a try
    // conditional not in its finally clause is found. In the latter case
    // the ":break" becomes pending until the ":endtry".
    if let Some(at) = cleanup_conditionals(cond, CsFlags::LOOP, true)
        && !cond.with(|cs| cs.flags[at].has(CsFlags::LOOP))
    {
        cond.with(|cs| cs.pending[at] = CSTP_BREAK);
        report_pending(PendingAction::Made, CSTP_BREAK, PendingValue::None);
    }
}

/// `:endwhile` and `:endfor`
pub(crate) fn ex_endwhile(excmd: &mut ExArg) {
    let cond = cond_stack_of(excmd);
    let ending_while = excmd.cmdidx == CmdIdx::endwhile;
    let err = if ending_while {
        err_msg(e_while)
    } else {
        err_msg(e_for)
    };
    let kind = if ending_while {
        CsFlags::WHILE
    } else {
        CsFlags::FOR
    };

    if cond.with(|cs| cs.loop_level <= 0 || cs.idx < 0) {
        excmd.errmsg = err;
        return;
    }

    let flags = cond.with(|cs| cs.top_flags());
    if !flags.has(kind) {
        // In a ":while"/":for" but with the wrong endloop command: do
        // not rewind to the next enclosing one.
        if flags.has(CsFlags::WHILE) {
            excmd.errmsg = Some(c"E732: Using :endfor with :while".to_owned());
        } else if flags.has(CsFlags::FOR) {
            excmd.errmsg = Some(c"E733: Using :endwhile with :for".to_owned());
        }
    }
    if !flags.has(CsFlags::LOOP) {
        if !flags.has(CsFlags::TRY) {
            excmd.errmsg = err_msg(e_endif);
        } else if flags.has(CsFlags::FINALLY) {
            excmd.errmsg = err_msg(e_endtry);
        }
        // Find the matching ":while" and report what is missing.
        let found = cond.with(|cs| {
            let mut idx = cs.top().expect("a level is open");
            while idx > 0 {
                let flags = cs.flags[idx];
                if flags.has(CsFlags::TRY) && !flags.has(CsFlags::FINALLY) {
                    // Give up at a try conditional not in its finally
                    // clause, and ignore the ":endwhile"/":endfor".
                    return None;
                }
                if flags.has(kind) {
                    break;
                }
                idx -= 1;
            }
            Some(idx)
        });
        let Some(idx) = found else {
            excmd.errmsg = err;
            return;
        };
        // Clean up and rewind every contained, unclosed conditional.
        cleanup_conditionals(cond, CsFlags::LOOP, false);
        rewind_conditionals(cond, Some(idx), CsFlags::TRY);
    } else if flags.has(CsFlags::TRUE) && !flags.has(CsFlags::ACTIVE) && dbg_check_skipped(excmd) {
        // When debugging or at a breakpoint, show the prompt if it has
        // not been shown: an ":endwhile"/":endfor" runs when the
        // ":while" was not TRUE or after a ":break". A ">quit" counts as
        // an interrupt before it, so throw an interrupt exception --
        // doing it here stops the exception for a parsing error being
        // discarded by that interrupt exception later.
        do_intthrow(cond);
    }

    // Let `do_cmdline` jump back to the matching ":while"/":for".
    cond.with(|cs| cs.loop_flags |= CsLoopFlags::HAD_ENDLOOP);
}

/// Make conditionals inactive, and discard what their finally clauses had
/// pending, until `searched_cond` or a try conditional not in its finally
/// clause is reached. A caught exception in an active catch clause on the
/// way is finished.
///
/// `searched_cond` is `CsFlags::LOOP`, or `CsFlags::TRY`, or 0 meaning the
/// innermost try conditional not in its finally clause. `inclusive` says
/// whether the conditional searched for is itself made inactive; a try
/// conditional not in its finally clause found on the way always is.
///
/// With `inclusive` and `searched_cond == CsFlags::TRY | CsFlags::SILENT`, the
/// `emsg_silent` a `:try` saved is restored -- [`ex_endtry`] wants that, and
/// normally it only happens when such a conditional is left.
///
/// Answers the level the search stopped at, `None` when it went through
/// them all.
pub(crate) fn cleanup_conditionals(
    cond: CondId,
    searched_cond: CsFlags,
    inclusive: bool,
) -> Option<usize> {
    let mut stop = false;
    let mut level = cond.with(|cs| cs.top());
    while let Some(at) = level {
        let flags = cond.with(|cs| cs.flags[at]);
        if flags.has(CsFlags::TRY) {
            discard_finally_pending(cond, at);

            // Stop at a try conditional not in its finally clause. If it
            // is in an active catch clause, finish the caught exception.
            if !flags.has(CsFlags::FINALLY) {
                if flags.has(CsFlags::ACTIVE)
                    && flags.has(CsFlags::CAUGHT)
                    && !flags.has(CsFlags::FINISHED)
                {
                    exception::finish_exception(cond.with(|cs| cs.pending_exception(at)));
                    cond.with(|cs| cs.flags[at] |= CsFlags::FINISHED);
                }
                // Stop here -- unless the try block never got active,
                // because of an inactive surrounding conditional or
                // because the ":try" came after an error, interrupt or
                // throw.
                if flags.has(CsFlags::TRUE) {
                    if searched_cond.is_empty() && !inclusive {
                        break;
                    }
                    stop = true;
                }
            }
        }

        // Stop on the searched-for conditional type, even when the
        // surrounding one is inactive or something was made pending.
        if flags.has(searched_cond) {
            if !inclusive {
                break;
            }
            stop = true;
        }
        let leave_silent = cond.with(|cs| {
            cs.flags[at].clear(CsFlags::ACTIVE);
            if stop && searched_cond != CsFlags::TRY | CsFlags::SILENT {
                return None;
            }
            // Leaving a try conditional that reset "emsg_silent" on entry:
            // restore the saved value.
            let flags = cs.flags[at];
            if flags.has(CsFlags::TRY) && flags.has(CsFlags::SILENT) {
                cs.flags[at].clear(CsFlags::SILENT);
                let saved = cs.saved_emsg_silent.pop();
                return Some(Some(saved.expect("a :try that reset it saved emsg_silent")));
            }
            Some(None)
        });
        let Some(restore) = leave_silent else {
            break;
        };
        if let Some(saved) = restore {
            emsg_silent.set(saved);
        }
        if stop {
            break;
        }
        level = at.checked_sub(1);
    }
    level
}

/// Throw away what the finally clause of the try conditional at `at` had
/// pending. There may also be a `:continue`/`:break`/`:return`/`:finish`
/// from before the finally clause, which must be kept unless an error or
/// interrupt happened after it.
fn discard_finally_pending(cond: CondId, at: usize) {
    let (flags, pending) = cond.with(|cs| (cs.flags[at], cs.pending[at]));
    if !(did_emsg.get() != 0 || got_int.get() || flags.has(CsFlags::FINALLY)) {
        return;
    }
    match pending {
        CSTP_NONE => {}
        CSTP_CONTINUE | CSTP_BREAK | CSTP_FINISH => {
            report_pending(PendingAction::Discarded, pending, PendingValue::None);
            cond.with(|cs| cs.pending[at] = CSTP_NONE);
        }
        CSTP_RETURN => {
            let pend = cond.with(|cs| cs.take_pend(at));
            let value = match &pend {
                Pend::Return(value) => value.as_ref(),
                _ => None,
            };
            report_pending(
                PendingAction::Discarded,
                CSTP_RETURN,
                PendingValue::Return(value),
            );
            drop(pend);
            cond.with(|cs| cs.pending[at] = CSTP_NONE);
        }
        _ => {
            if !flags.has(CsFlags::FINALLY) {
                return;
            }
            let exception = cond.with(|cs| cs.pending_exception(at));
            if let Some(exception) = exception.filter(|_| pending & CSTP_THROW != 0) {
                // Cancel the pending exception. This is in the finally
                // clause, so the caught-exception stack is not involved.
                exception::discard_exception(exception, false);
            } else {
                report_pending(PendingAction::Discarded, pending, PendingValue::None);
            }
            cond.with(|cs| cs.pending[at] = CSTP_NONE);
        }
    }
}

/// The error for a missing `:endwhile`/`:endfor`/`:endif`.
fn get_end_emsg(cs: &CondStack) -> Option<CString> {
    let flags = cs.top_flags();
    if flags.has(CsFlags::WHILE) {
        err_msg(e_endwhile)
    } else if flags.has(CsFlags::FOR) {
        err_msg(e_endfor)
    } else {
        err_msg(e_endif)
    }
}

/// Pop conditionals until level `keep` is the innermost (all of them for
/// `None`), counting each popped one of type `cond_type` off its counter
/// (`loop_level` for [`CsFlags::LOOP`], `try_level` for [`CsFlags::TRY`])
/// and dropping any `:for`'s iteration.
pub(crate) fn rewind_conditionals(cond: CondId, keep: Option<usize>, cond_type: CsFlags) {
    let dropped = cond.with(|cs| {
        let mut dropped = Vec::new();
        while let Some(top) = cs.top().filter(|&top| keep.is_none_or(|keep| top > keep)) {
            if cs.flags[top].has(cond_type) {
                if cond_type == CsFlags::TRY {
                    cs.try_level -= 1;
                } else {
                    cs.loop_level -= 1;
                }
            }
            if cs.flags[top].has(CsFlags::FOR)
                && let Some(info) = cs.for_info[top].take()
            {
                dropped.push(info);
            }
            cs.idx -= 1;
        }
        dropped
    });
    drop(dropped);
}

/// `:endfunction` when there was no `:function`.
pub(crate) fn ex_endfunction(_excmd: &mut ExArg) {
    semsg!("E193: {} not inside a function", ":endfunction");
}

/// Whether `line` looks like a `:while` or `:for` command.
///
/// `line` may be a command line's cheap tail: `modifier_len` stops at the
/// NUL, as does the white-space skip, so neither walk leaves the string.
pub(crate) fn has_loop_cmd(line: &[u8]) -> bool {
    let mut at = 0;
    let byte = |at: usize| line.get(at).copied().unwrap_or(0);
    loop {
        while matches!(byte(at), b' ' | b'\t' | b':') {
            at += 1;
        }
        let len = modifier_len(&line[at.min(line.len())..]);
        if len == 0 {
            break;
        }
        at += len;
    }
    (byte(at) == b'w' && byte(at + 1) == b'h')
        || (byte(at) == b'f' && byte(at + 1) == b'o' && byte(at + 2) == b'r')
}
