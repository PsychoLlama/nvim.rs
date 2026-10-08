//! The try conditional: `:try`, `:catch`, `:finally`, `:endtry` and
//! `:throw`, plus the [`enter_cleanup`]/[`leave_cleanup`] pair that gives
//! cleanup autocommands the same treatment without an actual `:try`.
//!
//! ```text
//! :try                    -+
//!     ...  try block       |
//! :catch /RE/              |
//!     ...  catch clause    +- try conditional
//! :finally                 |
//!     ...  finally clause  |
//! :endtry                 -+
//! ```
//!
//! Any number of catch clauses, at most one finally clause, nesting
//! allowed. A `:throw` may sit in the try block, a catch clause, the finally
//! clause, a function called from any of them, or entirely outside.
//!
//! **What makes this hard is that the finally clause must run anyway.** When
//! something interrupts the try block -- an error, a CTRL-C, a `:throw`, or
//! a `:continue`/`:break`/`:return`/`:finish` trying to leave -- that
//! outcome cannot simply happen: the finally clause has to execute first.
//! So [`ex_finally`] parks it in the level's `pending` (the `CSTP_*` values,
//! with the value or exception beside it), the finally clause runs on a
//! cleared `did_emsg`/`got_int`/`did_throw`, and [`ex_endtry`] resumes
//! whatever was parked -- unless the finally clause produced something new,
//! which replaces it.
//!
//! [`enter_cleanup`] and [`leave_cleanup`] are the same idea for a failing
//! command's cleanup autocommands, where there is no `:try` to hang the
//! pending state on and the error has not become an exception yet.
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

use super::exception::Thrown;
use super::exception::{
    PendingAction, PendingValue, catch_exception, discard_current_exception, discard_exception,
    do_intthrow, free_global_msglist, report_pending, throw_exception,
};
use super::flag::{
    CSTP_BREAK, CSTP_CONTINUE, CSTP_ERROR, CSTP_FINISH, CSTP_INTERRUPT, CSTP_NONE, CSTP_RETURN,
    CSTP_THROW, THROW_ON_ERROR,
};
use super::{CsFlags, CsLoopFlags};
use super::{
    aborting, check_skip, cleanup_conditionals, cond_stack_of, ex_break, ex_continue, get_end_emsg,
    message, rewind_conditionals,
};
use crate::cstr;
use crate::debugger::dbg_check_skipped;
use crate::eval::eval_cmd_string;
use crate::eval::userfunc::do_return;
use crate::ex_docmd::ends_excmd;
use crate::ex_eval::state::{current_exception, did_throw, force_abort, need_rethrow};
use crate::getchar::state::got_int;
use crate::guard::Suppress;
use crate::message::e_argreq;
use crate::message::state::{did_emsg, emsg_silent};
use crate::message::{emsg, internal_error};
use crate::message_fmt::msg_bytes;
use crate::option::SavedCpo;
use crate::regexp::{OwnedProg, RE_MAGIC, RE_STRING, skip_regexp_err_at};
use crate::runtime::do_finish;
use crate::semsg;
use crate::types::{Cleanup, CondId, ExArg, Pend, TypVal};
use core::ffi::c_int;

/// `:throw {expr}`
pub(crate) fn ex_throw(excmd: &mut ExArg) {
    let value = if !matches!(excmd.line.byte_at(excmd.line.arg), 0 | b'|' | b'\n') {
        let skip = excmd.skip;
        eval_cmd_string(excmd, skip)
    } else {
        emsg(message(e_argreq));
        None
    };

    // Do not throw on an error, or when the argument evaluation threw.
    if excmd.skip {
        return;
    }
    let Some(value) = value else {
        return;
    };
    // A refused throw has reported itself and dropped the value.
    if throw_exception(Thrown::User(value), None).is_ok() {
        do_throw(cond_stack_of(excmd));
    }
}

/// Throw the current exception through `cond`. Shared by `:throw`, by the
/// error and interrupt exceptions, and by the rethrow at an `:endtry`.
pub(crate) fn do_throw(cond: CondId) {
    // Clean up and deactivate as far as the next surrounding try conditional
    // that is not in its finally clause. That conditional itself stays
    // active so its ACTIVE flag can be tested below.
    if let Some(at) = cleanup_conditionals(cond, CsFlags::NONE, false) {
        cond.with(|cs| {
            let flags = &mut cs.flags[at];
            // If this try conditional is active and we are before its first
            // ":catch", set THROWN so the ":catch" commands check whether
            // the exception matches. An exception from a catch clause is
            // instead made pending at the ":finally" and rethrown at the
            // ":endtry" -- which also happens when the conditional is
            // inactive, i.e. when this throw comes from an error or
            // interrupt on the way to a finally or catch clause.
            if !flags.has(CsFlags::CAUGHT) {
                if flags.has(CsFlags::ACTIVE) {
                    *flags |= CsFlags::THROWN;
                } else {
                    // THROWN may be left over from a catchable exception
                    // that was discarded; reset it for the new one.
                    flags.clear(CsFlags::THROWN);
                }
            }
            flags.clear(CsFlags::ACTIVE);
            cs.set_pending_exception(at, current_exception.get());
        });
    }
    did_throw.set(true);
}

/// `:try`
pub(crate) fn ex_try(excmd: &mut ExArg) {
    let cond = cond_stack_of(excmd);
    let Some(at) = cond.with(|cs| {
        (!cs.is_full()).then(|| {
            let at = cs.push();
            cs.try_level += 1;
            cs.flags[at] = CsFlags::TRY;
            cs.pending[at] = CSTP_NONE;
            at
        })
    }) else {
        excmd.errmsg = Some(c"E601: :try nesting too deep".to_owned());
        return;
    };

    if check_skip(cond) {
        return;
    }
    // ":silent!" disables displaying errors and converting them to
    // exceptions even inside a try conditional. When the silenced
    // commands open a try conditional of their own, save "emsg_silent"
    // and reset it so errors become exceptions again; it is restored
    // when that conditional is left, however it is left. If it is left
    // by an aborting error, an interrupt or an exception, restoring it
    // does not matter -- the effect is then just forgetting the value.
    let silent = emsg_silent.get();
    cond.with(|cs| {
        // ACTIVE and TRUE: TRUE means the ":catch" commands should look for
        // a match when an exception is thrown, and that the finally clause
        // needs to run.
        cs.flags[at] |= CsFlags::ACTIVE | CsFlags::TRUE;
        if silent != 0 {
            cs.saved_emsg_silent.push(silent);
            cs.flags[at] |= CsFlags::SILENT;
        }
    });
    if silent != 0 {
        emsg_silent.set(0);
    }
}

/// `:catch /{pattern}/` and bare `:catch`.
pub(crate) fn ex_catch(excmd: &mut ExArg) {
    let cond = cond_stack_of(excmd);
    let mut at = 0;
    let mut give_up = false;
    let mut skip = false;

    if cond.with(|cs| cs.try_level <= 0 || cs.idx < 0) {
        excmd.errmsg = Some(c"E603: :catch without :try".to_owned());
        give_up = true;
    } else {
        let (missing, try_at, after_finally) = cond.with(|cs| {
            // Report what is missing if the matching ":try" is not in its
            // finally clause.
            let missing = (!cs.top_flags().has(CsFlags::TRY)).then(|| get_end_emsg(cs));
            let mut at = cs.top().expect("a level is open");
            while at > 0 && !cs.flags[at].has(CsFlags::TRY) {
                at -= 1;
            }
            (missing, at, cs.flags[at].has(CsFlags::FINALLY))
        });
        if let Some(missing) = missing {
            excmd.errmsg = missing;
            skip = true;
        }
        at = try_at;
        if after_finally {
            // Give up on a ":catch" after ":finally" and just parse it.
            excmd.errmsg = Some(c"E604: :catch after :finally".to_owned());
            give_up = true;
        } else {
            rewind_conditionals(cond, Some(at), CsFlags::LOOP);
        }
    }

    // Where the pattern starts and ends in the line. `None` is the
    // implicit `.*` of a bare `:catch`, which is not in the line at all.
    let mut span = None;
    if ends_excmd(c_int::from(excmd.line.byte_at(excmd.line.arg))) != 0 {
        // No argument: catch everything.
        excmd.line.next = excmd.line.find_next(excmd.line.arg);
    } else {
        let delim = c_int::from(excmd.line.byte_at(excmd.line.arg));
        let start = excmd.line.arg + 1;
        match skip_regexp_err_at(excmd.line.tail(start), delim, c_int::from(true)) {
            Some(len) => span = Some((start, start + len)),
            None => give_up = true,
        }
    }

    if !give_up {
        let flags = cond.with(|cs| cs.flags[at]);
        // Nothing to do when no exception has been thrown, or when the
        // try block never got active -- because of an inactive
        // surrounding conditional, or after an error, interrupt or
        // throw.
        if !did_throw.get() || !flags.has(CsFlags::TRUE) {
            skip = true;
        }

        // Check for a match only if an exception is being thrown and no
        // earlier ":catch" took it. An exception that replaced a
        // discarded one is not checked -- THROWN is not set then.
        let mut caught = false;
        if !skip && flags.has(CsFlags::THROWN) && !flags.has(CsFlags::CAUGHT) {
            if let Some((_, end)) = span
                && excmd.line.byte_at(end) != 0
                && ends_excmd(c_int::from(
                    excmd.line.byte_at(excmd.line.skip_white(end + 1)),
                )) == 0
            {
                let trailing = msg_bytes(excmd.line.rest_of(end));
                semsg!("E488: Trailing characters: {trailing}");
                return;
            }
            // When debugging, show the prompt before matching: a helpful
            // hint when the pattern does not match. A ">quit" there
            // counts as an interrupt before the ":catch", which replaces
            // the exception and so is not caught by this block.
            if !dbg_check_skipped(excmd) || !do_intthrow(cond) {
                let pat = match span {
                    Some((start, end)) => excmd.line.slice_at(start, end - start),
                    None => b".*",
                };
                caught = pattern_catches(pat);
            }
        }

        if caught {
            // Activate this catch clause, reset did_emsg/got_int/
            // did_throw, and stack the exception.
            let caught = cond.with(|cs| {
                cs.flags[at] |= CsFlags::ACTIVE | CsFlags::CAUGHT;
                cs.pending_exception(at)
            });
            did_emsg.set(0);
            got_int.set(false);
            did_throw.set(false);
            catch_exception(caught.expect("a thrown exception at the level that catches it"));
            // The current exception must be the one in the stack, so that
            // it can be discarded at the next ":catch", ":finally" or
            // ":endtry", or when the catch clause is left by a
            // ":continue", ":break", ":return", ":finish", error,
            // interrupt or another exception.
            let top = cond.with(|cs| cs.pending_exception(cs.top().expect("a level is open")));
            if top != current_exception.get() {
                internal_error(c"ex_catch()");
            }
        } else {
            // A preceding catch clause that caught the exception is
            // finished now; this happens after errors too, except when
            // this ":catch" came after the ":finally" or outside a
            // ":try". Making the conditional inactive skips the
            // following catch clauses. After an error or interrupt
            // following a ":continue"/":break"/":return"/":finish" out
            // of the try block or a catch clause, the pending action is
            // discarded.
            cleanup_conditionals(cond, CsFlags::TRY, true);
        }
    }

    if let Some((_, end)) = span {
        excmd.line.next = excmd.line.find_next(end);
    }
}

/// Whether `pat` matches the exception being thrown. There is one: only
/// `ex_catch` calls this, and only inside its `THROWN` test.
fn pattern_catches(pat: &[u8]) -> bool {
    // Keep the 'l' flag in 'cpoptions' out of the way while compiling. The
    // compiler takes a NUL-terminated pattern, so it is copied.
    let owned = cstr::owned(pat);
    let cpo = SavedCpo::empty();
    // Disable error messages: one here would invalidate the exception.
    let no_emsg = Suppress::emsg();
    let prog = OwnedProg::compile(&owned, RE_MAGIC + RE_STRING);
    drop(no_emsg);
    drop(cpo);
    let Some(mut prog) = prog else {
        let pat = msg_bytes(pat);
        semsg!("E475: Invalid argument: {pat}");
        return false;
    };
    // Save got_int and reset it: an earlier interruption must not cancel
    // the match, only a CTRL-C hit during it.
    let prev_got_int = got_int.get();
    got_int.set(false);
    let thrown = current_exception.get().expect("an exception being thrown");
    // A copy: the match cannot run user code, but the exception table is
    // not borrowed across a call into another module.
    let value = thrown.with(|exception| exception.value.clone());
    let caught = prog.exec_nl(value.as_cstr(), 0, false).is_some();
    got_int.set(got_int.get() | prev_got_int);
    caught
}

/// `:finally`
pub(crate) fn ex_finally(excmd: &mut ExArg) {
    let cond = cond_stack_of(excmd);
    let mut pending = CSTP_NONE;

    let found = cond.with(|cs| {
        let mut idx = cs.idx;
        while let Ok(at) = usize::try_from(idx) {
            if cs.flags[at].has(CsFlags::TRY) {
                break;
            }
            idx -= 1;
        }
        (cs.try_level > 0 && idx >= 0).then_some(idx)
    });
    let Some(idx) = found else {
        excmd.errmsg = Some(c"E606: :finally without :try".to_owned());
        return;
    };
    let at = usize::try_from(idx).expect("checked");

    let (missing, has_finally) = cond.with(|cs| {
        let missing = (!cs.top_flags().has(CsFlags::TRY)).then(|| get_end_emsg(cs));
        (missing, cs.flags[at].has(CsFlags::FINALLY))
    });
    if let Some(missing) = missing {
        excmd.errmsg = missing;
        // Make this error pending so that the following finally clause
        // still runs. It overrules a pending ":continue", ":break",
        // ":return" or ":finish" too.
        pending = CSTP_ERROR;
    }

    if has_finally {
        // Give up on a second ":finally" and ignore it.
        excmd.errmsg = Some(super::E_MULTIPLE_FINALLY.to_owned());
        return;
    }
    rewind_conditionals(cond, Some(at), CsFlags::LOOP);

    // Nothing to do when the try block never got active -- because of an
    // inactive surrounding conditional, or after an error, interrupt or
    // throw -- nor for a ":finally" without ":try" or a second
    // ":finally". After any other error, an interrupt or an exception,
    // the finally clause must run.
    if !cond.with(|cs| cs.top_flags().has(CsFlags::TRUE)) {
        return;
    }

    // When debugging, show the prompt so the user knows the finally
    // clause is running. A ">quit" counts as an interrupt before the
    // ":finally", replacing the original exception.
    if dbg_check_skipped(excmd) {
        do_intthrow(cond);
    }

    // A preceding catch clause that caught the exception is finished
    // now. After an error or interrupt this also discards a pending
    // ":continue", ":break", ":finish" or ":return" from the try block
    // or a catch clause.
    cleanup_conditionals(cond, CsFlags::TRY, false);

    // Make did_emsg, got_int and did_throw pending; they overrule a
    // pending ":continue"/":break"/":return"/":finish", whose return
    // value must then be discarded. The ":endtry" restores them, unless
    // the finally clause produces something new. A missing ":endwhile",
    // ":endfor" or ":endif" detected above counts as did_emsg and
    // did_throw respectively. did_emsg must not be set here: that would
    // suppress the error message.
    if pending == CSTP_ERROR || did_emsg.get() != 0 || got_int.get() || did_throw.get() {
        let top = cond.with(|cs| cs.top().expect("the :try is open"));
        let returning = cond.with(|cs| (cs.pending[top] == CSTP_RETURN).then(|| cs.take_pend(top)));
        if let Some(pend) = returning {
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
        }
        if pending == CSTP_ERROR && did_emsg.get() == 0 {
            pending |= if THROW_ON_ERROR { CSTP_THROW } else { 0 };
        } else {
            pending |= if did_throw.get() { CSTP_THROW } else { 0 };
        }
        pending |= if did_emsg.get() != 0 { CSTP_ERROR } else { 0 };
        pending |= if got_int.get() { CSTP_INTERRUPT } else { 0 };
        let parked = cond.with(|cs| {
            cs.pending[top] = pending;
            cs.pending_exception(top)
        });

        // The current exception must be the one in the stack, so that it
        // can be rethrown at the ":endtry" or discarded if the finally
        // clause is left by a ":continue", ":break", ":return", ":finish",
        // error, interrupt or another exception. When `emsg` was called
        // for a missing ":endif"/":endwhile"/":endfor" detected here, the
        // exception will be discarded.
        if did_throw.get() && parked != current_exception.get() {
            internal_error(c"ex_finally()");
        }
    }

    // CsLoopFlags::HAD_FINA makes `do_cmdline` reset did_emsg, got_int and
    // did_throw and activate the finally clause. That happens after
    // `emsg` has been called for a missing ":endif" or ":endwhile"
    // detected here, so the finally clause runs even then.
    cond.with(|cs| cs.loop_flags |= CsLoopFlags::HAD_FINA);
}

/// `:endtry`
pub(crate) fn ex_endtry(excmd: &mut ExArg) {
    let cond = cond_stack_of(excmd);
    let mut rethrow = false;
    let mut pending = CSTP_NONE;
    let mut returned: Option<TypVal> = None;

    let found = cond.with(|cs| {
        let mut idx = cs.idx;
        while let Ok(at) = usize::try_from(idx) {
            if cs.flags[at].has(CsFlags::TRY) {
                break;
            }
            idx -= 1;
        }
        (cs.try_level > 0 && idx >= 0).then_some(idx)
    });
    let Some(mut idx) = found else {
        excmd.errmsg = Some(c"E602: :endtry without :try".to_owned());
        return;
    };

    // Nothing to do after an error, interrupt or throw in the try block,
    // a catch clause or the finally clause before this ":endtry"; after
    // an error or interrupt following a ":continue"/":break"/":return"/
    // ":finish" in one of those; or when the try block never got active.
    // A surrounding conditional made inactive by the finally clause need
    // not be tested: anything pending has already been discarded then.
    let top_flags = cond.with(|cs| cs.top_flags());
    let mut skip =
        did_emsg.get() != 0 || got_int.get() || did_throw.get() || !top_flags.has(CsFlags::TRUE);

    if top_flags.has(CsFlags::TRY) {
        idx = cond.with(|cs| cs.idx);
        // If we stopped here with the exception still being thrown,
        // because we did not yet know this conditional has no finally
        // clause, it has to be rethrown once the conditional is closed.
        if did_throw.get() && top_flags.has(CsFlags::TRUE) && !top_flags.has(CsFlags::FINALLY) {
            rethrow = true;
        }
    } else {
        excmd.errmsg = cond.with(|cs| get_end_emsg(cs));
        // Find the matching ":try" and report what is missing.
        rewind_conditionals(cond, usize::try_from(idx).ok(), CsFlags::LOOP);
        skip = true;

        // Discard anything being thrown so it is not rethrown at the end
        // of this function; the error message would discard it anyway.
        // Script termination is unaffected, since "trylevel" is
        // decremented only after `emsg` has been called.
        if did_throw.get() {
            discard_current_exception();
        }
        // Report eap->errmsg even if there already was an error.
        did_emsg.set(0);
    }
    let at = usize::try_from(idx).expect("checked");

    // With no finally clause, show the user when debugging that the end
    // of the try conditional has been reached. Do that on normal control
    // flow or when an exception was thrown, but not on an interrupt or
    // an error that did not become an exception, and not when a
    // ":break"/":continue"/":return"/":finish" is pending -- those are
    // carried out immediately.
    let (flags, pending_here) = cond.with(|cs| (cs.flags[at], cs.pending[at]));
    if (rethrow || (!skip && !flags.has(CsFlags::FINALLY) && pending_here == 0))
        && dbg_check_skipped(excmd)
        && got_int.get()
    {
        // A ">quit" counts as an interrupt before the ":endtry".
        skip = true;
        do_intthrow(cond);
        // `do_intthrow` may have reset did_throw or the level's pending.
        rethrow = did_throw.get() && !cond.with(|cs| cs.flags[at].has(CsFlags::FINALLY));
    }

    // A pending ":return" resumes after the conditional is closed, so
    // remember its value. A finally clause that made an exception
    // pending needs it rethrown, so make it current again.
    if !skip {
        let (was, pend, exception) = cond.with(|cs| {
            let was = core::mem::replace(&mut cs.pending[at], CSTP_NONE);
            let pend = (was == CSTP_RETURN).then(|| cs.take_pend(at));
            (was, pend, cs.pending_exception(at))
        });
        pending = was;
        if let Some(Pend::Return(value)) = pend {
            returned = value;
        } else if pending & CSTP_THROW != 0 {
            current_exception.set(exception);
        }
    }

    // Discard anything pending on an error, interrupt or throw in the
    // finally clause. With no ":finally", discard a pending
    // ":continue"/":break"/":return"/":finish" if an error or interrupt
    // happened after it but before the ":endtry". If the last catch
    // clause caught an exception and there was no finally clause, finish
    // it now. Restore "emsg_silent" if this conditional reset it.
    cleanup_conditionals(cond, CsFlags::TRY | CsFlags::SILENT, true);

    cond.with(|cs| {
        if cs.top_flags().has(CsFlags::TRY) {
            cs.idx -= 1;
        }
        cs.try_level -= 1;
    });

    if !skip {
        let value = if pending == CSTP_RETURN {
            PendingValue::Return(returned.as_ref())
        } else if pending & CSTP_THROW != 0 {
            current_exception
                .get()
                .map_or(PendingValue::None, PendingValue::Exception)
        } else {
            PendingValue::None
        };
        report_pending(PendingAction::Resumed, pending, value);
        // Reactivate a ":continue", ":break", ":return" or ":finish"
        // pending from the try block or a catch clause. Skipped if there
        // was an error in an unskipped conditional command, an interrupt
        // afterwards, or if the finally clause produced something new.
        match pending {
            CSTP_NONE => {}
            CSTP_CONTINUE => ex_continue(excmd),
            CSTP_BREAK => ex_break(excmd),
            CSTP_RETURN => {
                do_return(excmd, false, returned.take());
            }
            CSTP_FINISH => do_finish(excmd, false),
            // The finally clause was entered because of an error,
            // interrupt or throw rather than a control-flow command:
            // restore those. Skipped if the finally clause produced
            // something new.
            _ => {
                if pending & CSTP_ERROR != 0 {
                    did_emsg.set(1);
                }
                if pending & CSTP_INTERRUPT != 0 {
                    got_int.set(true);
                }
                if pending & CSTP_THROW != 0 {
                    rethrow = true;
                }
            }
        }
    }

    if rethrow {
        // Rethrow within this stack.
        do_throw(cond);
    }
}

// enter_cleanup() and leave_cleanup()
//
// Called around a sequence of cleanup autocommands run for a failed command
// -- failure meaning `emsg` was called, an interrupt happened, or a previous
// autocommand execution for the same command left an uncaught exception.
// The `Cleanup` holds the pending error/interrupt/exception state across
// the pair.

/// Park the current error/interrupt/exception state in `parked` and clear
/// it, so that the cleanup autocommands run on a clean slate.
///
/// A bit like [`ex_finally`], except there was no extra try block around the
/// part that failed, and an error or interrupt has not become an exception
/// yet.
pub(crate) fn enter_cleanup(parked: &mut Cleanup) {
    // The pending values are restored by `leave_cleanup`, unless an aborting
    // error, an interrupt or an uncaught exception happens in between.
    if !(did_emsg.get() != 0 || got_int.get() || did_throw.get() || need_rethrow.get()) {
        parked.pending = CSTP_NONE;
        parked.exception = None;
        return;
    }

    parked.pending = if did_emsg.get() != 0 { CSTP_ERROR } else { 0 }
        | if got_int.get() { CSTP_INTERRUPT } else { 0 }
        | if did_throw.get() { CSTP_THROW } else { 0 }
        | if need_rethrow.get() { CSTP_THROW } else { 0 };

    // Save the exception being thrown, if there is one. On an error not
    // yet converted, update "force_abort" and reset "cause_abort" as
    // `do_errthrow` would; the `do_cmdline` call about to be made for
    // the autocommands needs that. `*msg_list` need not be saved: every
    // `do_cmdline` has its own.
    if did_throw.get() || need_rethrow.get() {
        parked.exception = current_exception.take();
    } else {
        parked.exception = None;
        if did_emsg.get() != 0 {
            force_abort.set(force_abort.get() | super::cause_abort.get());
            super::cause_abort.set(false);
        }
    }
    did_emsg.set(0);
    got_int.set(false);
    did_throw.set(false);
    need_rethrow.set(false);

    // Upstream passes its own uninitialised-by-intent `pending` local
    // here, which is still `CSTP_NONE` -- so this report never fires.
    // Kept as it is: `report_pending` returns immediately on CSTP_NONE,
    // and changing it would add 'verbose' output nothing expects.
    let value = parked
        .exception
        .map_or(PendingValue::None, PendingValue::Exception);
    report_pending(PendingAction::Made, CSTP_NONE, value);
}

/// Restore what [`enter_cleanup`] parked -- unless the cleanup autocommands
/// themselves aborted, in which case the parked state is discarded.
///
/// A bit like [`ex_endtry`], except there was no extra try block and the
/// error or interrupt had not become an exception when the autocommands were
/// invoked.
pub(crate) fn leave_cleanup(parked: &mut Cleanup) {
    let pending = parked.pending;
    if pending == CSTP_NONE {
        return;
    }

    // An aborting error, an interrupt or an uncaught exception since
    // `enter_cleanup` discards what it made pending.
    if aborting() || need_rethrow.get() {
        if pending & CSTP_THROW != 0 {
            // Cancel the pending exception; this reports it too.
            match parked.exception {
                Some(exception) => discard_exception(exception, false),
                None => internal_error(c"discard_exception()"),
            }
        } else {
            report_pending(PendingAction::Discarded, pending, PendingValue::None);
        }
        // If an error was about to become an exception when
        // `enter_cleanup` was called, free the message list.
        free_global_msglist();
        return;
    }

    // Nothing new happened in between: restore the pending state.
    if pending & CSTP_THROW != 0 {
        // Make the parked exception the one being thrown again.
        current_exception.set(parked.exception);
    } else if pending & CSTP_ERROR != 0 {
        // An error was about to become an exception: let "cause_abort"
        // take the part of "force_abort", as `cause_errthrow` does.
        super::cause_abort.set(force_abort.get());
        force_abort.set(false);
    }

    if pending & CSTP_ERROR != 0 {
        did_emsg.set(1);
    }
    if pending & CSTP_INTERRUPT != 0 {
        got_int.set(true);
    }
    if pending & CSTP_THROW != 0 {
        // `do_one_cmd` will set did_throw.
        need_rethrow.set(true);
    }

    let value = match current_exception.get() {
        Some(id) if pending & CSTP_THROW != 0 => PendingValue::Exception(id),
        _ => PendingValue::None,
    };
    report_pending(PendingAction::Resumed, pending, value);
}

/// A pending exception, error or `:return` parked for the length of a
/// value: [`enter_cleanup`] when it is made, [`leave_cleanup`] when it is
/// dropped. For code that has to run autocommands while one is pending.
pub(crate) struct CleanupGuard(Cleanup);

impl CleanupGuard {
    pub(crate) fn enter() -> CleanupGuard {
        let mut parked = Cleanup {
            pending: 0,
            exception: None,
        };
        enter_cleanup(&mut parked);
        CleanupGuard(parked)
    }
}

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        leave_cleanup(&mut self.0);
    }
}
