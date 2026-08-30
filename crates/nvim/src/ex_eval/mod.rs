//! Vimscript control flow: `:if`, `:while`, `:for`, the try conditional, and
//! the exception machinery underneath all three.
//!
//! Everything here is state on one array, `cstack_T`, which `do_cmdline`
//! owns and passes in through `eap->cstack`. Each `:if`/`:while`/`:for`/
//! `:try` pushes an entry; the matching end command pops it. An entry's
//! `cs_flags` says what it is ([`flag::CSF_WHILE`], [`flag::CSF_TRY`], ...)
//! and how it stands: `CSF_ACTIVE` means its commands are being *executed*
//! rather than merely parsed, and `CSF_TRUE` means the condition was met at
//! least once, which is what tells `:endif` whether to show a debug prompt
//! and `:finally` whether its clause needs running at all.
//!
//! **Skipping is not the same as not executing.** A command inside an
//! inactive conditional is still parsed, because the parser has to find the
//! matching `:endif` -- so almost everything below starts with
//! [`check_skip`], and errors detected while skipping are mostly, but not
//! entirely, ignored.
//!
//! The three parts:
//!
//! - Here: the conditional stack itself, the `:if`/`:while`/`:for` family,
//!   and the two operations everything else needs on that stack --
//!   [`cleanup_conditionals`], which deactivates entries down to the one
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
//! # Safety
//!
//! Every `unsafe fn` here takes editor state by raw pointer -- the `exarg_T`
//! of the command being executed, its `cstack_T`, or an `except_T` from one
//! of the two exception stacks -- and runs on the main thread with that
//! state live. `eap->cstack` is `do_cmdline`'s own stack local and outlives
//! every call made from it. That is the contract these modules share; each
//! states it once by reference and does not restate it.
//!
//! Original: `src/nvim/ex_eval.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

mod exception;
mod trycmd;

use crate::debugger::dbg_check_skipped;
use crate::eval::typval::{tv_clear, tv_free};
use crate::eval::{
    clear_evalarg, eval_for_line, eval_to_bool, eval0, fill_evalarg_from_eap, free_for_info,
    next_for_item,
};
use crate::ex_docmd::{ends_excmd, modifier_len};
use crate::global_cell::GlobalCell;
use crate::main::{
    did_emsg, did_endif, did_throw, e_endfor, e_endif, e_endtry, e_endwhile, e_for, e_while,
    emsg_silent, force_abort, got_int, trylevel,
};
use crate::memory::xfree;
use crate::message_fmt::c_str;
use crate::semsg;
use crate::types::{
    CMD_else, CMD_elseif, CMD_endwhile, CMD_while, FAIL, VAR_UNKNOWN, VarLock, cstack_T, eslist_T,
    evalarg_T, exarg_T, typval_T, typval_vval_union,
};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;
use std::ffi::CString;

use flag::{
    CSF_ACTIVE, CSF_CAUGHT, CSF_ELSE, CSF_FINALLY, CSF_FINISHED, CSF_FOR, CSF_SILENT, CSF_TRUE,
    CSF_TRY, CSF_WHILE, CSL_HAD_CONT, CSL_HAD_ENDLOOP, CSL_HAD_LOOP, CSTACK_LEN, CSTP_BREAK,
    CSTP_CONTINUE, CSTP_FINISH, CSTP_NONE, CSTP_RETURN, CSTP_THROW,
};

pub(crate) use exception::{
    cause_errthrow, discard_current_exception, do_errthrow, do_intthrow, exception_state_clear,
    exception_state_restore, exception_state_save, free_global_msglist, get_exception_string,
    report_make_pending,
};
pub(crate) use trycmd::{
    do_throw, enter_cleanup, ex_catch, ex_endtry, ex_finally, ex_throw, ex_try, leave_cleanup,
};

/// Constants the transpiler copied in from the headers this module includes.
pub(crate) mod flag {
    use super::c_int;
    use crate::types::{estack_arg_T, except_type_T};

    /// How deep `:if`/`:while`/`:for`/`:try` may nest.
    pub(crate) const CSTACK_LEN: c_int = 50;

    /// `cstack_T.cs_flags`: what a conditional stack entry is, and how it
    /// stands. The first two are the state; the rest name the command.
    pub(crate) const CSF_TRUE: c_int = 1;
    pub(crate) const CSF_ACTIVE: c_int = 2;
    pub(crate) const CSF_ELSE: c_int = 4;
    pub(crate) const CSF_WHILE: c_int = 8;
    pub(crate) const CSF_FOR: c_int = 16;
    pub(crate) const CSF_TRY: c_int = 256;
    pub(crate) const CSF_FINALLY: c_int = 512;
    /// An exception was thrown and this `:try` should check its `:catch`es.
    pub(crate) const CSF_THROWN: c_int = 2048;
    /// One of them matched.
    pub(crate) const CSF_CAUGHT: c_int = 4096;
    /// And that catch clause has ended.
    pub(crate) const CSF_FINISHED: c_int = 8192;
    /// This `:try` reset `emsg_silent`; the old value is on
    /// `cs_emsg_silent_list`.
    pub(crate) const CSF_SILENT: c_int = 16384;

    /// `cstack_T.cs_pending`: what a finally clause postponed. The last
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

    /// `cstack_T.cs_lflags`: what `do_cmdline` should do next about the
    /// innermost loop.
    pub(crate) const CSL_HAD_LOOP: c_int = 1;
    pub(crate) const CSL_HAD_ENDLOOP: c_int = 2;
    pub(crate) const CSL_HAD_CONT: c_int = 4;
    pub(crate) const CSL_HAD_FINA: c_int = 8;

    /// `except_T.type_0`.
    pub(crate) const ET_USER: except_type_T = 0;
    pub(crate) const ET_ERROR: except_type_T = 1;
    pub(crate) const ET_INTERRUPT: except_type_T = 2;

    /// Whether an error under an active try conditional becomes a catchable
    /// exception rather than terminating the script after the finally
    /// clauses. True for a Vim release; upstream keeps the switch for its
    /// `THROW_TEST` builds, which this tree does not have.
    pub(crate) const THROW_ON_ERROR: bool = true;

    pub(crate) const ESTACK_NONE: estack_arg_T = 0;
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
fn message(msg: &'static CStr) -> *mut c_char {
    msg.as_ptr().cast_mut()
}

/// A message constant as an owned `eap->errmsg`.
fn err_msg(msg: &'static CStr) -> Option<CString> {
    Some(msg.to_owned())
}

/// Do not do something after an error, an interrupt or a throw, nor when the
/// surrounding conditional was not active. Upstream's `CHECK_SKIP`.
///
/// # Safety
/// Module contract.
unsafe fn check_skip(cstack: *mut cstack_T) -> bool {
    // SAFETY: module contract.
    let idx = unsafe { (*cstack).cs_idx };
    did_emsg.get() != 0
        || got_int.get()
        || did_throw.get()
        || (idx > 0 && unsafe { (*cstack).cs_flags[(idx - 1) as usize] } & CSF_ACTIVE == 0)
}

/// Throw away the value a pending `:return` was carrying.
///
/// # Safety
/// `p` is a `typval_T` a pending `:return` owned.
unsafe fn discard_pending_return(p: *mut c_void) {
    // SAFETY: caller contract.
    unsafe { tv_free(p.cast::<typval_T>()) }
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

/// Whether a function with the "abort" flag should not count as ended on an
/// error -- parsing continues, to find finally clauses to execute, and some
/// errors in skipped commands are still reported.
pub(crate) fn aborted_in_try() -> bool {
    // Only called after an error, where `force_abort` decides whether the
    // search for finally clauses is needed.
    force_abort.get()
}

/// `:eval {expr}`
///
/// # Safety
/// Module contract.
pub(crate) unsafe fn ex_eval(eap: *mut exarg_T) {
    let mut tv = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VarLock::Unlocked,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut evalarg = evalarg_T {
        eval_flags: 0,
        eval_getline: None,
        eval_cookie: ptr::null_mut(),
        eval_tofree: ptr::null_mut(),
    };
    // SAFETY: module contract.
    unsafe { fill_evalarg_from_eap(&raw mut evalarg, eap, (*eap).skip != 0) };
    if unsafe { eval0((*eap).arg, &raw mut tv, eap, &raw mut evalarg) }.is_ok() {
        unsafe { tv_clear(&raw mut tv) };
    }
    unsafe { clear_evalarg(&raw mut evalarg, eap) };
}

/// `:if {expr}`
///
/// # Safety
/// Module contract.
pub(crate) unsafe fn ex_if(eap: *mut exarg_T) {
    // SAFETY: module contract.
    let cstack = unsafe { (*eap).cstack };
    if unsafe { (*cstack).cs_idx } == CSTACK_LEN - 1 {
        unsafe { (*eap).errmsg = Some(c"E579: :if nesting too deep".to_owned()) };
        return;
    }
    unsafe { (*cstack).cs_idx += 1 };
    let idx = unsafe { (*cstack).cs_idx } as usize;
    unsafe { (*cstack).cs_flags[idx] = 0 };

    let skip = unsafe { check_skip(cstack) };
    let mut error = false;
    let result = unsafe { eval_to_bool((*eap).arg, &raw mut error, eap, skip, false) };

    unsafe {
        (*cstack).cs_flags[idx] = if skip || error {
            // Set TRUE, so this conditional never becomes active.
            CSF_TRUE
        } else if result {
            CSF_ACTIVE | CSF_TRUE
        } else {
            0
        }
    };
}

/// `:endif`
///
/// # Safety
/// Module contract.
pub(crate) unsafe fn ex_endif(eap: *mut exarg_T) {
    did_endif.set(true);
    // SAFETY: module contract.
    let cstack = unsafe { (*eap).cstack };
    if unsafe { (*cstack).cs_idx } < 0
        || unsafe { (*cstack).cs_flags[(*cstack).cs_idx as usize] }
            & (CSF_WHILE | CSF_FOR | CSF_TRY)
            != 0
    {
        unsafe { (*eap).errmsg = Some(c"E580: :endif without :if".to_owned()) };
        return;
    }
    // When debugging or at a breakpoint, show the prompt if it has not
    // been shown: this tells the user that an ":endif" runs when the
    // ":if" or a previous ":elseif" was not TRUE. A ">quit" counts as an
    // interrupt before the ":endif", so throw an interrupt exception if
    // appropriate -- doing it here stops the exception for a parsing
    // error being discarded by that interrupt exception later on.
    if unsafe { (*cstack).cs_flags[(*cstack).cs_idx as usize] } & CSF_TRUE == 0
        && unsafe { dbg_check_skipped(eap) }
    {
        unsafe { do_intthrow(cstack) };
    }
    unsafe { (*cstack).cs_idx -= 1 };
}

/// `:else` and `:elseif {expr}`
///
/// # Safety
/// Module contract.
pub(crate) unsafe fn ex_else(eap: *mut exarg_T) {
    // SAFETY: module contract.
    let cstack = unsafe { (*eap).cstack };
    let mut skip = unsafe { check_skip(cstack) };

    if unsafe { (*cstack).cs_idx } < 0
        || unsafe { (*cstack).cs_flags[(*cstack).cs_idx as usize] }
            & (CSF_WHILE | CSF_FOR | CSF_TRY)
            != 0
    {
        if unsafe { (*eap).cmdidx } == CMD_else {
            unsafe { (*eap).errmsg = Some(c"E581: :else without :if".to_owned()) };
            return;
        }
        unsafe { (*eap).errmsg = Some(c"E582: :elseif without :if".to_owned()) };
        skip = true;
    } else if unsafe { (*cstack).cs_flags[(*cstack).cs_idx as usize] } & CSF_ELSE != 0 {
        if unsafe { (*eap).cmdidx } == CMD_else {
            unsafe { (*eap).errmsg = Some(E_MULTIPLE_ELSE.to_owned()) };
            return;
        }
        unsafe { (*eap).errmsg = Some(c"E584: :elseif after :else".to_owned()) };
        skip = true;
    }

    let idx = unsafe { (*cstack).cs_idx } as usize;
    // Skipping, or the ":if" was TRUE: reset ACTIVE. Otherwise set it.
    if skip || unsafe { (*cstack).cs_flags[idx] } & CSF_TRUE != 0 {
        if unsafe { (*eap).errmsg.is_none() } {
            unsafe { (*cstack).cs_flags[idx] = CSF_TRUE };
        }
        // Don't evaluate an ":elseif".
        skip = true;
    } else {
        unsafe { (*cstack).cs_flags[idx] = CSF_ACTIVE };
    }

    // When debugging or at a breakpoint, show the prompt if it has not
    // been shown: this tells the user that an ":else"/":elseif" runs
    // when the ":if" or a previous ":elseif" was not TRUE. A ">quit"
    // counts as an interrupt before it, so set "skip" and throw an
    // interrupt exception -- doing it here stops the exception for a
    // parsing error being discarded by that interrupt exception later.
    if !skip && unsafe { dbg_check_skipped(eap) } && got_int.get() {
        unsafe { do_intthrow(cstack) };
        skip = true;
    }

    if unsafe { (*eap).cmdidx } != CMD_elseif {
        unsafe { (*cstack).cs_flags[idx] |= CSF_ELSE };
        return;
    }

    let mut result = false;
    let mut error = false;
    // While skipping most errors are ignored, but a missing expression
    // is wrong -- perhaps it should have been ":else". A double quote
    // here starts a string, it is not a comment.
    if skip
        && unsafe { *(*eap).arg } != b'"' as c_char
        && ends_excmd(unsafe { *(*eap).arg } as c_int) != 0
    {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let arg = unsafe { c_str((*eap).arg) };
        semsg!("E15: Invalid expression: \"{arg}\"");
    } else {
        result = unsafe { eval_to_bool((*eap).arg, &raw mut error, eap, skip, false) };
    }

    // The first of several errors in a row is the one to throw. That is
    // what happens when a conditional error was found above and parsing
    // the expression then failed too: "skip" is set in that case, so
    // `emsg` ignores the parsing error.
    if !skip && !error {
        unsafe { (*cstack).cs_flags[idx] = if result { CSF_ACTIVE | CSF_TRUE } else { 0 } };
    } else if unsafe { (*eap).errmsg.is_none() } {
        // Set TRUE, so this conditional never becomes active.
        unsafe { (*cstack).cs_flags[idx] = CSF_TRUE };
    }
}

/// `:while {expr}` and `:for {var} in {expr}`
///
/// # Safety
/// Module contract.
pub(crate) unsafe fn ex_while(eap: *mut exarg_T) {
    // SAFETY: module contract.
    let cstack = unsafe { (*eap).cstack };
    if unsafe { (*cstack).cs_idx } == CSTACK_LEN - 1 {
        unsafe { (*eap).errmsg = Some(c"E585: :while/:for nesting too deep".to_owned()) };
        return;
    }

    // The loop flag is set when we jumped back from the matching
    // ":endwhile"/":endfor". When it is not set, this cstack entry needs
    // initialising.
    let jumped_back = unsafe { (*cstack).cs_lflags } & CSL_HAD_LOOP != 0;
    if !jumped_back {
        unsafe { (*cstack).cs_idx += 1 };
        unsafe { (*cstack).cs_looplevel += 1 };
        unsafe { (*cstack).cs_line[(*cstack).cs_idx as usize] = -1 };
    }
    let idx = unsafe { (*cstack).cs_idx } as usize;
    let is_while = unsafe { (*eap).cmdidx } == CMD_while;
    unsafe { (*cstack).cs_flags[idx] = if is_while { CSF_WHILE } else { CSF_FOR } };

    let skip = unsafe { check_skip(cstack) };
    let mut error = false;
    let result = if is_while {
        unsafe { eval_to_bool((*eap).arg, &raw mut error, eap, skip, false) }
    } else {
        unsafe { for_next_item(eap, cstack, idx, jumped_back, skip, &mut error) }
    };

    if !skip && !error && result {
        unsafe { (*cstack).cs_flags[idx] |= CSF_ACTIVE | CSF_TRUE };
        unsafe { (*cstack).cs_lflags ^= CSL_HAD_LOOP };
    } else {
        unsafe { (*cstack).cs_lflags &= !CSL_HAD_LOOP };
        // The ":while" was FALSE or the ":for" ran off the end of the
        // list: show the debug prompt at the ":endwhile"/":endfor" as if
        // there had been a ":break" in a TRUE loop.
        if !skip && !error {
            unsafe { (*cstack).cs_flags[idx] |= CSF_TRUE };
        }
    }
}

/// The `:for` half of [`ex_while`]: evaluate the list on the first pass,
/// then take the next element off it. Answers whether there was one.
///
/// # Safety
/// Module contract; `idx` is `cstack->cs_idx`.
unsafe fn for_next_item(
    eap: *mut exarg_T,
    cstack: *mut cstack_T,
    idx: usize,
    jumped_back: bool,
    skip: bool,
    error: &mut bool,
) -> bool {
    let mut evalarg = evalarg_T {
        eval_flags: 0,
        eval_getline: None,
        eval_cookie: ptr::null_mut(),
        eval_tofree: ptr::null_mut(),
    };
    // SAFETY: module contract.
    unsafe { fill_evalarg_from_eap(&raw mut evalarg, eap, skip) };
    let fi = if jumped_back {
        // Jumped here from a ":continue" or ":endfor": reuse the list
        // that was evaluated then.
        *error = false;
        unsafe { (*cstack).cs_forinfo[idx] }
    } else {
        let fi = unsafe { eval_for_line((*eap).arg, error, eap, &raw mut evalarg) };
        unsafe { (*cstack).cs_forinfo[idx] = fi };
        fi
    };

    // Use the element at the start of the list and advance.
    let result = !*error && !fi.is_null() && !skip && unsafe { next_for_item(fi, (*eap).arg) };
    if !result {
        unsafe { free_for_info(fi) };
        unsafe { (*cstack).cs_forinfo[idx] = ptr::null_mut() };
    }
    unsafe { clear_evalarg(&raw mut evalarg, eap) };
    result
}

/// `:continue`
///
/// # Safety
/// Module contract.
pub(crate) unsafe fn ex_continue(eap: *mut exarg_T) {
    // SAFETY: module contract.
    let cstack = unsafe { (*eap).cstack };
    if unsafe { (*cstack).cs_looplevel } <= 0 || unsafe { (*cstack).cs_idx } < 0 {
        unsafe { (*eap).errmsg = Some(c"E586: :continue without :while or :for".to_owned()) };
        return;
    }
    // Find the matching ":while". This may stop at a try conditional not
    // in its finally clause, which is then what runs next, so deactivate
    // every conditional except the ":while" itself, if it is reached.
    let idx = unsafe { cleanup_conditionals(cstack, CSF_WHILE | CSF_FOR, false) };
    debug_assert!(idx >= 0, "idx >= 0");
    if unsafe { (*cstack).cs_flags[idx as usize] } & (CSF_WHILE | CSF_FOR) != 0 {
        unsafe { rewind_conditionals(cstack, idx, CSF_TRY, &raw mut (*cstack).cs_trylevel) };
        // Let `do_cmdline` jump back to the matching ":while".
        unsafe { (*cstack).cs_lflags |= CSL_HAD_CONT };
    } else {
        // A try conditional not in its finally clause came first: make
        // the ":continue" pending until the ":endtry".
        unsafe { (*cstack).cs_pending[idx as usize] = CSTP_CONTINUE as c_char };
        unsafe { report_make_pending(CSTP_CONTINUE, ptr::null_mut()) };
    }
}

/// `:break`
///
/// # Safety
/// Module contract.
pub(crate) unsafe fn ex_break(eap: *mut exarg_T) {
    // SAFETY: module contract.
    let cstack = unsafe { (*eap).cstack };
    if unsafe { (*cstack).cs_looplevel } <= 0 || unsafe { (*cstack).cs_idx } < 0 {
        unsafe { (*eap).errmsg = Some(c"E587: :break without :while or :for".to_owned()) };
        return;
    }
    // Deactivate conditionals until the matching ":while" or a try
    // conditional not in its finally clause is found. In the latter case
    // the ":break" becomes pending until the ":endtry".
    let idx = unsafe { cleanup_conditionals(cstack, CSF_WHILE | CSF_FOR, true) };
    if idx >= 0 && unsafe { (*cstack).cs_flags[idx as usize] } & (CSF_WHILE | CSF_FOR) == 0 {
        unsafe { (*cstack).cs_pending[idx as usize] = CSTP_BREAK as c_char };
        unsafe { report_make_pending(CSTP_BREAK, ptr::null_mut()) };
    }
}

/// `:endwhile` and `:endfor`
///
/// # Safety
/// Module contract.
pub(crate) unsafe fn ex_endwhile(eap: *mut exarg_T) {
    // SAFETY: module contract.
    let cstack = unsafe { (*eap).cstack };
    let ending_while = unsafe { (*eap).cmdidx } == CMD_endwhile;
    let err = if ending_while {
        err_msg(e_while)
    } else {
        err_msg(e_for)
    };
    let csf = if ending_while { CSF_WHILE } else { CSF_FOR };

    if unsafe { (*cstack).cs_looplevel } <= 0 || unsafe { (*cstack).cs_idx } < 0 {
        unsafe { (*eap).errmsg = err };
        return;
    }

    let mut fl = unsafe { (*cstack).cs_flags[(*cstack).cs_idx as usize] };
    if fl & csf == 0 {
        // In a ":while"/":for" but with the wrong endloop command: do
        // not rewind to the next enclosing one.
        if fl & CSF_WHILE != 0 {
            unsafe { (*eap).errmsg = Some(c"E732: Using :endfor with :while".to_owned()) };
        } else if fl & CSF_FOR != 0 {
            unsafe { (*eap).errmsg = Some(c"E733: Using :endwhile with :for".to_owned()) };
        }
    }
    if fl & (CSF_WHILE | CSF_FOR) == 0 {
        if fl & CSF_TRY == 0 {
            unsafe { (*eap).errmsg = err_msg(e_endif) };
        } else if fl & CSF_FINALLY != 0 {
            unsafe { (*eap).errmsg = err_msg(e_endtry) };
        }
        // Find the matching ":while" and report what is missing.
        let mut idx = unsafe { (*cstack).cs_idx };
        while idx > 0 {
            fl = unsafe { (*cstack).cs_flags[idx as usize] };
            if fl & CSF_TRY != 0 && fl & CSF_FINALLY == 0 {
                // Give up at a try conditional not in its finally
                // clause, and ignore the ":endwhile"/":endfor".
                unsafe { (*eap).errmsg = err };
                return;
            }
            if fl & csf != 0 {
                break;
            }
            idx -= 1;
        }
        // Clean up and rewind every contained, unclosed conditional.
        unsafe { cleanup_conditionals(cstack, CSF_WHILE | CSF_FOR, false) };
        unsafe { rewind_conditionals(cstack, idx, CSF_TRY, &raw mut (*cstack).cs_trylevel) };
    } else if unsafe { (*cstack).cs_flags[(*cstack).cs_idx as usize] } & CSF_TRUE != 0
        && unsafe { (*cstack).cs_flags[(*cstack).cs_idx as usize] } & CSF_ACTIVE == 0
        && unsafe { dbg_check_skipped(eap) }
    {
        // When debugging or at a breakpoint, show the prompt if it has
        // not been shown: an ":endwhile"/":endfor" runs when the
        // ":while" was not TRUE or after a ":break". A ">quit" counts as
        // an interrupt before it, so throw an interrupt exception --
        // doing it here stops the exception for a parsing error being
        // discarded by that interrupt exception later.
        unsafe { do_intthrow(cstack) };
    }

    // Let `do_cmdline` jump back to the matching ":while"/":for".
    unsafe { (*cstack).cs_lflags |= CSL_HAD_ENDLOOP };
}

/// Make conditionals inactive, and discard what their finally clauses had
/// pending, until `searched_cond` or a try conditional not in its finally
/// clause is reached. A caught exception in an active catch clause on the
/// way is finished.
///
/// `searched_cond` is `CSF_WHILE | CSF_FOR`, or `CSF_TRY`, or 0 meaning the
/// innermost try conditional not in its finally clause. `inclusive` says
/// whether the conditional searched for is itself made inactive; a try
/// conditional not in its finally clause found on the way always is.
///
/// With `inclusive` and `searched_cond == CSF_TRY | CSF_SILENT`, the
/// `emsg_silent` a `:try` saved is restored -- [`ex_endtry`] wants that, and
/// normally it only happens when such a conditional is left.
///
/// Answers the cstack index the search stopped at.
///
/// # Safety
/// Module contract.
pub(crate) unsafe fn cleanup_conditionals(
    cstack: *mut cstack_T,
    searched_cond: c_int,
    inclusive: bool,
) -> c_int {
    let mut stop = false;
    // SAFETY: module contract, here and for the walk below.
    let mut idx = unsafe { (*cstack).cs_idx };
    while idx >= 0 {
        if unsafe { (*cstack).cs_flags[idx as usize] } & CSF_TRY != 0 {
            unsafe { discard_finally_pending(cstack, idx) };

            // Stop at a try conditional not in its finally clause. If it
            // is in an active catch clause, finish the caught exception.
            if unsafe { (*cstack).cs_flags[idx as usize] } & CSF_FINALLY == 0 {
                if unsafe { (*cstack).cs_flags[idx as usize] } & CSF_ACTIVE != 0
                    && unsafe { (*cstack).cs_flags[idx as usize] } & CSF_CAUGHT != 0
                    && unsafe { (*cstack).cs_flags[idx as usize] } & CSF_FINISHED == 0
                {
                    unsafe {
                        exception::finish_exception((*cstack).cs_pend.csp_ex[idx as usize].cast())
                    };
                    unsafe { (*cstack).cs_flags[idx as usize] |= CSF_FINISHED };
                }
                // Stop here -- unless the try block never got active,
                // because of an inactive surrounding conditional or
                // because the ":try" came after an error, interrupt or
                // throw.
                if unsafe { (*cstack).cs_flags[idx as usize] } & CSF_TRUE != 0 {
                    if searched_cond == 0 && !inclusive {
                        break;
                    }
                    stop = true;
                }
            }
        }

        // Stop on the searched-for conditional type, even when the
        // surrounding one is inactive or something was made pending.
        if unsafe { (*cstack).cs_flags[idx as usize] } & searched_cond != 0 {
            if !inclusive {
                break;
            }
            stop = true;
        }
        unsafe { (*cstack).cs_flags[idx as usize] &= !CSF_ACTIVE };
        if stop && searched_cond != CSF_TRY | CSF_SILENT {
            break;
        }

        // Leaving a try conditional that reset "emsg_silent" on entry:
        // restore the saved value and free the memory holding it.
        if unsafe { (*cstack).cs_flags[idx as usize] } & CSF_TRY != 0
            && unsafe { (*cstack).cs_flags[idx as usize] } & CSF_SILENT != 0
        {
            let elem: *mut eslist_T = unsafe { (*cstack).cs_emsg_silent_list };
            unsafe { (*cstack).cs_emsg_silent_list = (*elem).next };
            emsg_silent.set(unsafe { (*elem).saved_emsg_silent });
            unsafe { xfree(elem.cast()) };
            unsafe { (*cstack).cs_flags[idx as usize] &= !CSF_SILENT };
        }
        if stop {
            break;
        }
        idx -= 1;
    }
    idx
}

/// Throw away what the finally clause of the try conditional at `idx` had
/// pending. There may also be a `:continue`/`:break`/`:return`/`:finish`
/// from before the finally clause, which must be kept unless an error or
/// interrupt happened after it.
///
/// # Safety
/// Module contract; `idx` names a `CSF_TRY` entry.
unsafe fn discard_finally_pending(cstack: *mut cstack_T, idx: c_int) {
    // SAFETY: module contract.
    if !(did_emsg.get() != 0
        || got_int.get()
        || unsafe { (*cstack).cs_flags[idx as usize] } & CSF_FINALLY != 0)
    {
        return;
    }
    let pending = unsafe { (*cstack).cs_pending[idx as usize] } as c_int;
    match pending {
        CSTP_NONE => {}
        CSTP_CONTINUE | CSTP_BREAK | CSTP_FINISH => {
            unsafe { exception::report_discard_pending(pending, ptr::null_mut()) };
            unsafe { (*cstack).cs_pending[idx as usize] = CSTP_NONE as c_char };
        }
        CSTP_RETURN => {
            unsafe {
                exception::report_discard_pending(
                    CSTP_RETURN,
                    (*cstack).cs_pend.csp_rv[idx as usize],
                )
            };
            unsafe { discard_pending_return((*cstack).cs_pend.csp_rv[idx as usize]) };
            unsafe { (*cstack).cs_pending[idx as usize] = CSTP_NONE as c_char };
        }
        _ => {
            if unsafe { (*cstack).cs_flags[idx as usize] } & CSF_FINALLY == 0 {
                return;
            }
            if pending & CSTP_THROW != 0
                && !unsafe { (*cstack).cs_pend.csp_ex[idx as usize] }.is_null()
            {
                // Cancel the pending exception. This is in the finally
                // clause, so the caught-exception stack is not involved.
                unsafe {
                    exception::discard_exception(
                        (*cstack).cs_pend.csp_ex[idx as usize].cast(),
                        false,
                    )
                };
            } else {
                unsafe { exception::report_discard_pending(pending, ptr::null_mut()) };
            }
            unsafe { (*cstack).cs_pending[idx as usize] = CSTP_NONE as c_char };
        }
    }
}

/// The error for a missing `:endwhile`/`:endfor`/`:endif`.
///
/// # Safety
/// Module contract.
unsafe fn get_end_emsg(cstack: *mut cstack_T) -> Option<CString> {
    // SAFETY: module contract.
    let flags = unsafe { (*cstack).cs_flags[(*cstack).cs_idx as usize] };
    if flags & CSF_WHILE != 0 {
        err_msg(e_endwhile)
    } else if flags & CSF_FOR != 0 {
        err_msg(e_endfor)
    } else {
        err_msg(e_endif)
    }
}

/// Pop conditionals until index `idx` is reached, decrementing `cond_level`
/// for each popped entry of type `cond_type` and freeing any `:for` info.
///
/// # Safety
/// Module contract; `cond_level` points at a live counter, normally one of
/// `cstack`'s own.
pub(crate) unsafe fn rewind_conditionals(
    cstack: *mut cstack_T,
    idx: c_int,
    cond_type: c_int,
    cond_level: *mut c_int,
) {
    // SAFETY: module contract.
    while unsafe { (*cstack).cs_idx } > idx {
        let top = unsafe { (*cstack).cs_idx } as usize;
        if unsafe { (*cstack).cs_flags[top] } & cond_type != 0 {
            unsafe { *cond_level -= 1 };
        }
        if unsafe { (*cstack).cs_flags[top] } & CSF_FOR != 0 {
            unsafe { free_for_info((*cstack).cs_forinfo[top]) };
        }
        unsafe { (*cstack).cs_idx -= 1 };
    }
}

/// `:endfunction` when there was no `:function`.
///
/// # Safety
/// Module contract.
pub(crate) unsafe fn ex_endfunction(_eap: *mut exarg_T) {
    // SAFETY: module contract.
    semsg!("E193: {} not inside a function", ":endfunction");
}

/// Whether `p` looks like a `:while` or `:for` command.
///
/// # Safety
/// `p` is NUL-terminated.
pub(crate) unsafe fn has_loop_cmd(p: *mut c_char) -> bool {
    // SAFETY: caller contract; `modifier_len` stops at the NUL, as does the
    // whitespace skip, so neither walk leaves the string.
    let mut p = p;
    loop {
        while unsafe { *p } == b' ' as c_char
            || unsafe { *p } == b'\t' as c_char
            || unsafe { *p } == b':' as c_char
        {
            p = unsafe { p.add(1) };
        }
        let len = unsafe { modifier_len(p) };
        if len == 0 {
            break;
        }
        p = unsafe { p.offset(len as isize) };
    }
    (unsafe { *p } == b'w' as c_char && unsafe { *p.add(1) } == b'h' as c_char)
        || (unsafe { *p } == b'f' as c_char
            && unsafe { *p.add(1) } == b'o' as c_char
            && unsafe { *p.add(2) } == b'r' as c_char)
}
