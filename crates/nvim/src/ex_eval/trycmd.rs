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
//! So [`ex_finally`] parks it in `cs_pending[]` (the `CSTP_*` values, with
//! the exception itself in `cs_exception[]`), the finally clause runs on a
//! cleared `did_emsg`/`got_int`/`did_throw`, and [`ex_endtry`] resumes
//! whatever was parked -- unless the finally clause produced something new,
//! which replaces it.
//!
//! [`enter_cleanup`] and [`leave_cleanup`] are the same idea for a failing
//! command's cleanup autocommands, where there is no `:try` to hang the
//! pending state on and the error has not become an exception yet.
//!
//! Original: `src/nvim/ex_eval.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use super::exception::{
    catch_exception, discard_current_exception, discard_exception, do_intthrow,
    free_global_msglist, report_discard_pending, report_make_pending, report_resume_pending,
    throw_exception,
};
use super::flag::{
    CSF_ACTIVE, CSF_CAUGHT, CSF_FINALLY, CSF_FOR, CSF_SILENT, CSF_THROWN, CSF_TRUE, CSF_TRY,
    CSF_WHILE, CSL_HAD_FINA, CSTACK_LEN, CSTP_BREAK, CSTP_CONTINUE, CSTP_ERROR, CSTP_FINISH,
    CSTP_INTERRUPT, CSTP_NONE, CSTP_RETURN, CSTP_THROW, ET_USER, THROW_ON_ERROR,
};
use super::{
    aborting, check_skip, cleanup_conditionals, discard_pending_return, ex_break, ex_continue,
    get_end_emsg, message, rewind_conditionals,
};
use crate::charset::skipwhite;
use crate::debugger::dbg_check_skipped;
use crate::eval::eval_to_string_skip;
use crate::eval::userfunc::do_return;
use crate::ex_docmd::{ends_excmd, find_nextcmd};
use crate::guard::Suppress;
use crate::main::{
    current_exception, did_emsg, did_throw, e_argreq, emsg_silent, force_abort, got_int, msg_list,
    need_rethrow, p_cpo,
};
use crate::memory::{xfree, xmalloc};
use crate::message::{emsg_ptr, internal_error};
use crate::message_fmt::c_str;
use crate::optionstr::empty_option;
use crate::regexp::{
    RE_MAGIC, RE_STRING, skip_regexp_err, vim_regcomp, vim_regexec_nl, vim_regfree,
};
use crate::runtime::do_finish;
use crate::semsg;
use crate::types::{FAIL, NUL, cleanup_T, cstack_T, eslist_T, exarg_T, except_T, regmatch_T};
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

/// `:throw {expr}`
///
/// # Safety
/// Module contract.
pub(crate) unsafe fn ex_throw(eap: *mut exarg_T) {
    // SAFETY: module contract.
    let arg = unsafe { (*eap).arg };
    let value = if unsafe { *arg } != NUL as c_char
        && unsafe { *arg } != b'|' as c_char
        && unsafe { *arg } != b'\n' as c_char
    {
        unsafe { eval_to_string_skip(arg, eap, (*eap).skip != 0) }
    } else {
        unsafe { emsg_ptr(message(e_argreq)) };
        ptr::null_mut()
    };

    // Do not throw on an error, or when the argument evaluation threw.
    if unsafe { (*eap).skip } != 0 || value.is_null() {
        return;
    }
    if unsafe { throw_exception(value.cast(), ET_USER, ptr::null_mut()) } == FAIL {
        unsafe { xfree(value.cast()) };
    } else {
        unsafe { do_throw((*eap).cstack) };
    }
}

/// Throw the current exception through `cstack`. Shared by `:throw`, by the
/// error and interrupt exceptions, and by the rethrow at an `:endtry`.
///
/// # Safety
/// Module contract; an exception is current and `cstack` is the running one.
pub(crate) unsafe fn do_throw(cstack: *mut cstack_T) {
    // Clean up and deactivate as far as the next surrounding try conditional
    // that is not in its finally clause. That conditional itself stays
    // active so its ACTIVE flag can be tested below.
    // SAFETY: module contract.
    let idx = unsafe { cleanup_conditionals(cstack, 0, false) };
    if idx >= 0 {
        let flags = unsafe { &raw mut (*cstack).cs_flags[idx as usize] };
        // If this try conditional is active and we are before its first
        // ":catch", set THROWN so the ":catch" commands check whether
        // the exception matches. An exception from a catch clause is
        // instead made pending at the ":finally" and rethrown at the
        // ":endtry" -- which also happens when the conditional is
        // inactive, i.e. when this throw comes from an error or
        // interrupt on the way to a finally or catch clause.
        if unsafe { *flags } & CSF_CAUGHT == 0 {
            if unsafe { *flags } & CSF_ACTIVE != 0 {
                unsafe { *flags |= CSF_THROWN };
            } else {
                // THROWN may be left over from a catchable exception
                // that was discarded; reset it for the new one.
                unsafe { *flags &= !CSF_THROWN };
            }
        }
        unsafe { *flags &= !CSF_ACTIVE };
        unsafe { (*cstack).cs_pend.csp_ex[idx as usize] = current_exception.get().cast() };
    }
    did_throw.set(true);
}

/// `:try`
///
/// # Safety
/// Module contract.
pub(crate) unsafe fn ex_try(eap: *mut exarg_T) {
    // SAFETY: module contract.
    let cstack = unsafe { (*eap).cstack };
    if unsafe { (*cstack).cs_idx } == CSTACK_LEN - 1 {
        unsafe { (*eap).errmsg = Some(c"E601: :try nesting too deep".to_owned()) };
        return;
    }
    unsafe { (*cstack).cs_idx += 1 };
    unsafe { (*cstack).cs_trylevel += 1 };
    let idx = unsafe { (*cstack).cs_idx } as usize;
    unsafe { (*cstack).cs_flags[idx] = CSF_TRY };
    unsafe { (*cstack).cs_pending[idx] = CSTP_NONE as c_char };

    if unsafe { check_skip(cstack) } {
        return;
    }
    // ACTIVE and TRUE: TRUE means the ":catch" commands should look for
    // a match when an exception is thrown, and that the finally clause
    // needs to run.
    unsafe { (*cstack).cs_flags[idx] |= CSF_ACTIVE | CSF_TRUE };

    // ":silent!" disables displaying errors and converting them to
    // exceptions even inside a try conditional. When the silenced
    // commands open a try conditional of their own, save "emsg_silent"
    // and reset it so errors become exceptions again; it is restored
    // when that conditional is left, however it is left. If it is left
    // by an aborting error, an interrupt or an exception, restoring it
    // does not matter -- the effect is then just freeing the memory.
    if emsg_silent.get() != 0 {
        let elem: *mut eslist_T = unsafe { xmalloc(size_of::<eslist_T>()) }.cast();
        unsafe { (*elem).saved_emsg_silent = emsg_silent.get() };
        unsafe { (*elem).next = (*cstack).cs_emsg_silent_list };
        unsafe { (*cstack).cs_emsg_silent_list = elem };
        unsafe { (*cstack).cs_flags[idx] |= CSF_SILENT };
        emsg_silent.set(0);
    }
}

/// `:catch /{pattern}/` and bare `:catch`.
///
/// # Safety
/// Module contract.
pub(crate) unsafe fn ex_catch(eap: *mut exarg_T) {
    // SAFETY: module contract.
    let cstack = unsafe { (*eap).cstack };
    let mut idx: c_int = 0;
    let mut give_up = false;
    let mut skip = false;

    if unsafe { (*cstack).cs_trylevel } <= 0 || unsafe { (*cstack).cs_idx } < 0 {
        unsafe { (*eap).errmsg = Some(c"E603: :catch without :try".to_owned()) };
        give_up = true;
    } else {
        if unsafe { (*cstack).cs_flags[(*cstack).cs_idx as usize] } & CSF_TRY == 0 {
            // Report what is missing if the matching ":try" is not in
            // its finally clause.
            unsafe { (*eap).errmsg = get_end_emsg(cstack) };
            skip = true;
        }
        idx = unsafe { (*cstack).cs_idx };
        while idx > 0 && unsafe { (*cstack).cs_flags[idx as usize] } & CSF_TRY == 0 {
            idx -= 1;
        }
        if unsafe { (*cstack).cs_flags[idx as usize] } & CSF_FINALLY != 0 {
            // Give up on a ":catch" after ":finally" and just parse it.
            unsafe { (*eap).errmsg = Some(c"E604: :catch after :finally".to_owned()) };
            give_up = true;
        } else {
            unsafe {
                rewind_conditionals(
                    cstack,
                    idx,
                    CSF_WHILE | CSF_FOR,
                    &raw mut (*cstack).cs_looplevel,
                )
            };
        }
    }

    let pat;
    let end;
    if ends_excmd(unsafe { *(*eap).arg } as c_int) != 0 {
        // No argument: catch everything.
        pat = c".*".as_ptr().cast_mut();
        end = ptr::null_mut();
        unsafe { (*eap).nextcmd = find_nextcmd((*eap).arg) };
    } else {
        pat = unsafe { (*eap).arg.add(1) };
        end = unsafe { skip_regexp_err(pat, *(*eap).arg as c_int, true as c_int) };
        if end.is_null() {
            give_up = true;
        }
    }

    if !give_up {
        // Nothing to do when no exception has been thrown, or when the
        // try block never got active -- because of an inactive
        // surrounding conditional, or after an error, interrupt or
        // throw.
        if !did_throw.get() || unsafe { (*cstack).cs_flags[idx as usize] } & CSF_TRUE == 0 {
            skip = true;
        }

        // Check for a match only if an exception is being thrown and no
        // earlier ":catch" took it. An exception that replaced a
        // discarded one is not checked -- THROWN is not set then.
        let mut caught = false;
        if !skip
            && unsafe { (*cstack).cs_flags[idx as usize] } & CSF_THROWN != 0
            && unsafe { (*cstack).cs_flags[idx as usize] } & CSF_CAUGHT == 0
        {
            if !end.is_null()
                && unsafe { *end } != NUL as c_char
                && ends_excmd(unsafe { *skipwhite(end.add(1)) } as c_int) == 0
            {
                // SAFETY: a message argument the caller holds as a NUL-terminated string.
                let end = unsafe { c_str(end) };
                semsg!("E488: Trailing characters: {end}");
                return;
            }
            // When debugging, show the prompt before matching: a helpful
            // hint when the pattern does not match. A ">quit" there
            // counts as an interrupt before the ":catch", which replaces
            // the exception and so is not caught by this block.
            if !unsafe { dbg_check_skipped(eap) } || !unsafe { do_intthrow(cstack) } {
                caught = unsafe { pattern_catches(pat, end) };
            }
        }

        if caught {
            // Activate this catch clause, reset did_emsg/got_int/
            // did_throw, and stack the exception.
            unsafe { (*cstack).cs_flags[idx as usize] |= CSF_ACTIVE | CSF_CAUGHT };
            did_emsg.set(0);
            got_int.set(false);
            did_throw.set(false);
            unsafe { catch_exception((*cstack).cs_pend.csp_ex[idx as usize].cast::<except_T>()) };
            // The current exception must be the one in the cstack, so
            // that it can be discarded at the next ":catch", ":finally"
            // or ":endtry", or when the catch clause is left by a
            // ":continue", ":break", ":return", ":finish", error,
            // interrupt or another exception.
            if unsafe { (*cstack).cs_pend.csp_ex[(*cstack).cs_idx as usize] }.cast::<except_T>()
                != current_exception.get()
            {
                unsafe { internal_error(c"ex_catch()".as_ptr()) };
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
            unsafe { cleanup_conditionals(cstack, CSF_TRY, true) };
        }
    }

    if !end.is_null() {
        unsafe { (*eap).nextcmd = find_nextcmd(end) };
    }
}

/// Whether the pattern between `pat` and `end` matches the exception being
/// thrown. `end` is null for the implicit `.*` of a bare `:catch`.
///
/// # Safety
/// Module contract; an exception is current, and `pat`/`end` delimit a
/// pattern inside the command line.
unsafe fn pattern_catches(pat: *mut c_char, end: *mut c_char) -> bool {
    // SAFETY: caller contract.
    // Terminate the pattern, and keep the 'l' flag in 'cpoptions' out of
    // the way while compiling it.
    let mut save_char = 0;
    if !end.is_null() {
        save_char = unsafe { *end };
        unsafe { *end = NUL as c_char };
    }
    let save_cpo = p_cpo.get();
    p_cpo.set(empty_option());
    // Errors here would invalidate the current exception.
    // Disable error messages: one here would invalidate the exception.
    let no_emsg = Suppress::emsg();
    let mut regmatch = regmatch_T {
        regprog: unsafe { vim_regcomp(pat, RE_MAGIC + RE_STRING) },
        ..regmatch_T::default()
    };
    drop(no_emsg);
    if !end.is_null() {
        unsafe { *end = save_char };
    }
    p_cpo.set(save_cpo);
    if regmatch.regprog.is_null() {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let pat = unsafe { c_str(pat) };
        semsg!("E475: Invalid argument: {pat}");
        return false;
    }
    // Save got_int and reset it: an earlier interruption must not cancel
    // the match, only a CTRL-C hit during it.
    let prev_got_int = got_int.get();
    got_int.set(false);
    let caught = unsafe { vim_regexec_nl(&raw mut regmatch, (*current_exception.get()).value, 0) };
    got_int.set(got_int.get() | prev_got_int);
    unsafe { vim_regfree(regmatch.regprog) };
    caught
}

/// `:finally`
///
/// # Safety
/// Module contract.
pub(crate) unsafe fn ex_finally(eap: *mut exarg_T) {
    // SAFETY: module contract.
    let cstack = unsafe { (*eap).cstack };
    let mut pending: c_int = CSTP_NONE;

    let mut idx = unsafe { (*cstack).cs_idx };
    while idx >= 0 && unsafe { (*cstack).cs_flags[idx as usize] } & CSF_TRY == 0 {
        idx -= 1;
    }
    if unsafe { (*cstack).cs_trylevel } <= 0 || idx < 0 {
        unsafe { (*eap).errmsg = Some(c"E606: :finally without :try".to_owned()) };
        return;
    }

    if unsafe { (*cstack).cs_flags[(*cstack).cs_idx as usize] } & CSF_TRY == 0 {
        unsafe { (*eap).errmsg = get_end_emsg(cstack) };
        // Make this error pending so that the following finally clause
        // still runs. It overrules a pending ":continue", ":break",
        // ":return" or ":finish" too.
        pending = CSTP_ERROR;
    }

    if unsafe { (*cstack).cs_flags[idx as usize] } & CSF_FINALLY != 0 {
        // Give up on a second ":finally" and ignore it.
        unsafe { (*eap).errmsg = Some(super::E_MULTIPLE_FINALLY.to_owned()) };
        return;
    }
    unsafe {
        rewind_conditionals(
            cstack,
            idx,
            CSF_WHILE | CSF_FOR,
            &raw mut (*cstack).cs_looplevel,
        )
    };

    // Nothing to do when the try block never got active -- because of an
    // inactive surrounding conditional, or after an error, interrupt or
    // throw -- nor for a ":finally" without ":try" or a second
    // ":finally". After any other error, an interrupt or an exception,
    // the finally clause must run.
    if unsafe { (*cstack).cs_flags[(*cstack).cs_idx as usize] } & CSF_TRUE == 0 {
        return;
    }

    // When debugging, show the prompt so the user knows the finally
    // clause is running. A ">quit" counts as an interrupt before the
    // ":finally", replacing the original exception.
    if unsafe { dbg_check_skipped(eap) } {
        unsafe { do_intthrow(cstack) };
    }

    // A preceding catch clause that caught the exception is finished
    // now. After an error or interrupt this also discards a pending
    // ":continue", ":break", ":finish" or ":return" from the try block
    // or a catch clause.
    unsafe { cleanup_conditionals(cstack, CSF_TRY, false) };

    // Make did_emsg, got_int and did_throw pending; they overrule a
    // pending ":continue"/":break"/":return"/":finish", whose return
    // value must then be discarded. The ":endtry" restores them, unless
    // the finally clause produces something new. A missing ":endwhile",
    // ":endfor" or ":endif" detected above counts as did_emsg and
    // did_throw respectively. did_emsg must not be set here: that would
    // suppress the error message.
    if pending == CSTP_ERROR || did_emsg.get() != 0 || got_int.get() || did_throw.get() {
        let top = unsafe { (*cstack).cs_idx } as usize;
        if unsafe { (*cstack).cs_pending[top] } == CSTP_RETURN as c_char {
            unsafe { report_discard_pending(CSTP_RETURN, (*cstack).cs_pend.csp_rv[top]) };
            unsafe { discard_pending_return((*cstack).cs_pend.csp_rv[top]) };
        }
        if pending == CSTP_ERROR && did_emsg.get() == 0 {
            pending |= if THROW_ON_ERROR { CSTP_THROW } else { 0 };
        } else {
            pending |= if did_throw.get() { CSTP_THROW } else { 0 };
        }
        pending |= if did_emsg.get() != 0 { CSTP_ERROR } else { 0 };
        pending |= if got_int.get() { CSTP_INTERRUPT } else { 0 };
        debug_assert!(
            pending >= c_char::MIN as c_int && pending <= c_char::MAX as c_int,
            "pending >= CHAR_MIN && pending <= CHAR_MAX"
        );
        unsafe { (*cstack).cs_pending[top] = pending as c_char };

        // The current exception must be the one in the cstack, so that
        // it can be rethrown at the ":endtry" or discarded if the
        // finally clause is left by a ":continue", ":break", ":return",
        // ":finish", error, interrupt or another exception. When `emsg`
        // was called for a missing ":endif"/":endwhile"/":endfor"
        // detected here, the exception will be discarded.
        if did_throw.get()
            && unsafe { (*cstack).cs_pend.csp_ex[top] }.cast::<except_T>()
                != current_exception.get()
        {
            unsafe { internal_error(c"ex_finally()".as_ptr()) };
        }
    }

    // CSL_HAD_FINA makes `do_cmdline` reset did_emsg, got_int and
    // did_throw and activate the finally clause. That happens after
    // `emsg` has been called for a missing ":endif" or ":endwhile"
    // detected here, so the finally clause runs even then.
    unsafe { (*cstack).cs_lflags |= CSL_HAD_FINA };
}

/// `:endtry`
///
/// # Safety
/// Module contract.
pub(crate) unsafe fn ex_endtry(eap: *mut exarg_T) {
    // SAFETY: module contract.
    let cstack = unsafe { (*eap).cstack };
    let mut rethrow = false;
    let mut pending: c_char = CSTP_NONE as c_char;
    let mut rettv: *mut c_void = ptr::null_mut();

    let mut idx = unsafe { (*cstack).cs_idx };
    while idx >= 0 && unsafe { (*cstack).cs_flags[idx as usize] } & CSF_TRY == 0 {
        idx -= 1;
    }
    if unsafe { (*cstack).cs_trylevel } <= 0 || idx < 0 {
        unsafe { (*eap).errmsg = Some(c"E602: :endtry without :try".to_owned()) };
        return;
    }

    // Nothing to do after an error, interrupt or throw in the try block,
    // a catch clause or the finally clause before this ":endtry"; after
    // an error or interrupt following a ":continue"/":break"/":return"/
    // ":finish" in one of those; or when the try block never got active.
    // A surrounding conditional made inactive by the finally clause need
    // not be tested: anything pending has already been discarded then.
    let mut skip = did_emsg.get() != 0
        || got_int.get()
        || did_throw.get()
        || unsafe { (*cstack).cs_flags[(*cstack).cs_idx as usize] } & CSF_TRUE == 0;

    if unsafe { (*cstack).cs_flags[(*cstack).cs_idx as usize] } & CSF_TRY == 0 {
        unsafe { (*eap).errmsg = get_end_emsg(cstack) };
        // Find the matching ":try" and report what is missing.
        unsafe {
            rewind_conditionals(
                cstack,
                idx,
                CSF_WHILE | CSF_FOR,
                &raw mut (*cstack).cs_looplevel,
            )
        };
        skip = true;

        // Discard anything being thrown so it is not rethrown at the end
        // of this function; the error message would discard it anyway.
        // Script termination is unaffected, since "trylevel" is
        // decremented only after `emsg` has been called.
        if did_throw.get() {
            unsafe { discard_current_exception() };
        }
        // Report eap->errmsg even if there already was an error.
        did_emsg.set(0);
    } else {
        idx = unsafe { (*cstack).cs_idx };
        // If we stopped here with the exception still being thrown,
        // because we did not yet know this conditional has no finally
        // clause, it has to be rethrown once the conditional is closed.
        if did_throw.get()
            && unsafe { (*cstack).cs_flags[idx as usize] } & CSF_TRUE != 0
            && unsafe { (*cstack).cs_flags[idx as usize] } & CSF_FINALLY == 0
        {
            rethrow = true;
        }
    }

    // With no finally clause, show the user when debugging that the end
    // of the try conditional has been reached. Do that on normal control
    // flow or when an exception was thrown, but not on an interrupt or
    // an error that did not become an exception, and not when a
    // ":break"/":continue"/":return"/":finish" is pending -- those are
    // carried out immediately.
    if (rethrow
        || (!skip
            && unsafe { (*cstack).cs_flags[idx as usize] } & CSF_FINALLY == 0
            && unsafe { (*cstack).cs_pending[idx as usize] } == 0))
        && unsafe { dbg_check_skipped(eap) }
        && got_int.get()
    {
        // A ">quit" counts as an interrupt before the ":endtry".
        skip = true;
        unsafe { do_intthrow(cstack) };
        // `do_intthrow` may have reset did_throw or cs_pending[idx].
        rethrow = did_throw.get() && unsafe { (*cstack).cs_flags[idx as usize] } & CSF_FINALLY == 0;
    }

    // A pending ":return" resumes after the conditional is closed, so
    // remember its value. A finally clause that made an exception
    // pending needs it rethrown, so make it current again.
    if !skip {
        pending = unsafe { (*cstack).cs_pending[idx as usize] };
        unsafe { (*cstack).cs_pending[idx as usize] = CSTP_NONE as c_char };
        if pending == CSTP_RETURN as c_char {
            rettv = unsafe { (*cstack).cs_pend.csp_rv[idx as usize] };
        } else if pending as c_int & CSTP_THROW != 0 {
            current_exception.set(unsafe { (*cstack).cs_pend.csp_ex[idx as usize] }.cast());
        }
    }

    // Discard anything pending on an error, interrupt or throw in the
    // finally clause. With no ":finally", discard a pending
    // ":continue"/":break"/":return"/":finish" if an error or interrupt
    // happened after it but before the ":endtry". If the last catch
    // clause caught an exception and there was no finally clause, finish
    // it now. Restore "emsg_silent" if this conditional reset it.
    unsafe { cleanup_conditionals(cstack, CSF_TRY | CSF_SILENT, true) };

    if unsafe { (*cstack).cs_idx } >= 0
        && unsafe { (*cstack).cs_flags[(*cstack).cs_idx as usize] } & CSF_TRY != 0
    {
        unsafe { (*cstack).cs_idx -= 1 };
    }
    unsafe { (*cstack).cs_trylevel -= 1 };

    if !skip {
        unsafe {
            report_resume_pending(
                pending as c_int,
                if pending == CSTP_RETURN as c_char {
                    rettv
                } else if pending as c_int & CSTP_THROW != 0 {
                    current_exception.get().cast()
                } else {
                    ptr::null_mut()
                },
            )
        };
        // Reactivate a ":continue", ":break", ":return" or ":finish"
        // pending from the try block or a catch clause. Skipped if there
        // was an error in an unskipped conditional command, an interrupt
        // afterwards, or if the finally clause produced something new.
        match pending as c_int {
            CSTP_NONE => {}
            CSTP_CONTINUE => unsafe { ex_continue(eap) },
            CSTP_BREAK => unsafe { ex_break(eap) },
            CSTP_RETURN => {
                unsafe { do_return(eap, false, false, rettv) };
            }
            CSTP_FINISH => unsafe { do_finish(eap, false) },
            // The finally clause was entered because of an error,
            // interrupt or throw rather than a control-flow command:
            // restore those. Skipped if the finally clause produced
            // something new.
            _ => {
                if pending as c_int & CSTP_ERROR != 0 {
                    did_emsg.set(1);
                }
                if pending as c_int & CSTP_INTERRUPT != 0 {
                    got_int.set(true);
                }
                if pending as c_int & CSTP_THROW != 0 {
                    rethrow = true;
                }
            }
        }
    }

    if rethrow {
        // Rethrow within this cstack.
        unsafe { do_throw(cstack) };
    }
}

// enter_cleanup() and leave_cleanup()
//
// Called around a sequence of cleanup autocommands run for a failed command
// -- failure meaning `emsg` was called, an interrupt happened, or a previous
// autocommand execution for the same command left an uncaught exception.
// The `cleanup_T` holds the pending error/interrupt/exception state across
// the pair.

/// Park the current error/interrupt/exception state in `csp` and clear it,
/// so that the cleanup autocommands run on a clean slate.
///
/// A bit like [`ex_finally`], except there was no extra try block around the
/// part that failed, and an error or interrupt has not become an exception
/// yet.
///
/// # Safety
/// Module contract; `csp` is writable and outlives the matching
/// [`leave_cleanup`].
pub(crate) unsafe fn enter_cleanup(csp: *mut cleanup_T) {
    // The pending values are restored by `leave_cleanup`, unless an aborting
    // error, an interrupt or an uncaught exception happens in between.
    if !(did_emsg.get() != 0 || got_int.get() || did_throw.get() || need_rethrow.get()) {
        // SAFETY: caller contract.
        unsafe { (*csp).pending = CSTP_NONE };
        unsafe { (*csp).exception = ptr::null_mut() };
        return;
    }

    // SAFETY: caller contract.
    unsafe {
        (*csp).pending = if did_emsg.get() != 0 { CSTP_ERROR } else { 0 }
            | if got_int.get() { CSTP_INTERRUPT } else { 0 }
            | if did_throw.get() { CSTP_THROW } else { 0 }
            | if need_rethrow.get() { CSTP_THROW } else { 0 }
    };

    // Save the exception being thrown, if there is one. On an error not
    // yet converted, update "force_abort" and reset "cause_abort" as
    // `do_errthrow` would; the `do_cmdline` call about to be made for
    // the autocommands needs that. `*msg_list` need not be saved: every
    // `do_cmdline` has its own.
    if did_throw.get() || need_rethrow.get() {
        unsafe { (*csp).exception = current_exception.get() };
        current_exception.set(ptr::null_mut());
    } else {
        unsafe { (*csp).exception = ptr::null_mut() };
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
    unsafe { report_make_pending(CSTP_NONE, (*csp).exception.cast()) };
}

/// Restore what [`enter_cleanup`] parked -- unless the cleanup autocommands
/// themselves aborted, in which case the parked state is discarded.
///
/// A bit like [`ex_endtry`], except there was no extra try block and the
/// error or interrupt had not become an exception when the autocommands were
/// invoked.
///
/// # Safety
/// Module contract; `csp` was filled by [`enter_cleanup`].
pub(crate) unsafe fn leave_cleanup(csp: *mut cleanup_T) {
    // SAFETY: caller contract.
    let pending = unsafe { (*csp).pending };
    if pending == CSTP_NONE {
        return;
    }

    // An aborting error, an interrupt or an uncaught exception since
    // `enter_cleanup` discards what it made pending.
    if aborting() || need_rethrow.get() {
        if pending & CSTP_THROW != 0 {
            // Cancel the pending exception; this reports it too.
            unsafe { discard_exception((*csp).exception, false) };
        } else {
            unsafe { report_discard_pending(pending, ptr::null_mut()) };
        }
        // If an error was about to become an exception when
        // `enter_cleanup` was called, free the message list.
        if !msg_list.get().is_null() {
            unsafe { free_global_msglist() };
        }
        return;
    }

    // Nothing new happened in between: restore the pending state.
    if pending & CSTP_THROW != 0 {
        // Make the parked exception the one being thrown again.
        current_exception.set(unsafe { (*csp).exception });
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

    unsafe {
        report_resume_pending(
            pending,
            if pending & CSTP_THROW != 0 {
                current_exception.get().cast()
            } else {
                ptr::null_mut()
            },
        )
    };
}
