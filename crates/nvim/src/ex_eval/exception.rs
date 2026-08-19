//! The exception object: how an error, an interrupt or a `:throw` becomes a
//! catchable value, and what happens to it afterwards.
//!
//! An exception is an [`except_T`] holding the value a `:catch` pattern is
//! matched against, the throw point (`v:throwpoint`), a stack trace, and --
//! for an error exception -- the message list it was built from. Exactly one
//! can be *being thrown* at a time (`current_exception` plus `did_throw`);
//! any number can be *caught*, stacked on `caught_stack` by nesting, since a
//! catch clause may itself contain a try conditional.
//!
//! Three sources feed the same object:
//!
//! - **`:throw`** hands a string straight to [`throw_exception`].
//! - **An error** goes the long way round. `emsg` calls
//!   [`cause_errthrow`] *while the failing command is still running*, which
//!   only appends the message text to `*msg_list` -- the conditional stack
//!   is not reachable from there. [`do_errthrow`] runs after the command
//!   returns and turns that list into the exception. That two-step is why
//!   only the *first* of several errors in a row becomes the exception
//!   value, and why `cause_abort` exists: `force_abort` has to stay off
//!   until the throw point is reached, so that `aborting()` answers the same
//!   thing for every message of one command.
//! - **An interrupt** is [`do_intthrow`], which replaces whatever is being
//!   thrown -- CTRL-C beats a user exception, but not another interrupt.
//!
//! `Vim`-prefixed values are reserved: a user exception may not fake one,
//! because [`super::trycmd::do_throw`] and `do_cmdline` treat an uncaught
//! `Vim:...` differently from an uncaught user value.
//!
//! The three "something is pending in a finally clause" reports at the
//! bottom are the 'verbose' >= 14 half of the same machinery, and
//! [`report_pending`] is where the `CSTP_*` values are turned into words.
//!
//! Original: `src/nvim/ex_eval.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use super::flag::{
    CSTP_BREAK, CSTP_CONTINUE, CSTP_ERROR, CSTP_FINISH, CSTP_INTERRUPT, CSTP_NONE, CSTP_RETURN,
    CSTP_THROW, ESTACK_NONE, ET_ERROR, ET_INTERRUPT, ET_USER, IOSIZE,
};
use super::{cause_abort, iobuff, message};
use crate::ascii::ascii_isdigit;
use crate::eval::typval::{tv_list_ref, tv_list_unref};
use crate::eval::userfunc::get_return_cmd;
use crate::eval::vars::{set_vim_var_list, set_vim_var_string};
use crate::ex_docmd::handle_did_throw;
use crate::main::{
    caught_stack, cmdline_row, current_exception, debug_break_level, did_emsg, did_throw, e_interr,
    e_outofmem, emsg_silent, force_abort, got_int, msg_list, msg_row, msg_scroll, msg_silent,
    need_rethrow, no_wait_return, p_verbose, suppress_errthrow, trylevel,
};
use crate::memory::{xfree, xmalloc, xrealloc, xstrdup, xstrlcpy};
use crate::message::{emsg, internal_error, msg_puts, verbose_enter, verbose_leave};
use crate::option::p_vfile;
use crate::os::cshim::{snprintf, strncmp};
use crate::runtime::{estack_sfile, sourcing_lnum, stacktrace_create};
use crate::smsg_c;
use crate::strings::{concat_str, vim_snprintf, vim_snprintf_safelen, xstrnsave};
use crate::types::{
    FAIL, NUL, OK, VV_EXCEPTION, VV_STACKTRACE, VV_THROWPOINT, cstack_T, except_T, except_type_T,
    exception_state_T, int64_t, list_T, msglist_T, ptrdiff_t,
};
use ::libc::{strcat, strcpy, strlen};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

/// Turn an error message into a pending error exception if one is wanted
/// here, and report whether `emsg` should therefore keep quiet about it.
///
/// `ignore` is set when the `emsg` call should be dropped entirely. `severe`
/// says a later, more specific message should replace the first one;
/// `concat` appends to the previous message instead of starting a new one,
/// for a multi-part message.
///
/// # Safety
/// Module contract; `mesg` is NUL-terminated and `ignore` is writable.
pub unsafe fn cause_errthrow(
    mesg: *const c_char,
    multiline: bool,
    concat: bool,
    severe: bool,
    ignore: *mut bool,
) -> bool {
    // Nothing to do while displaying the interrupt message or reporting an
    // uncaught exception (already discarded by then) at the top level, nor
    // when no exception can be thrown: `emsg` displays the message itself.
    if suppress_errthrow.get() {
        return false;
    }

    // If `emsg` has not been called yet, hold `force_abort` off until the
    // throw point is reached, so `aborting()` gives the same answer for
    // every error of one command. Parsing errors during an expression then
    // all get reported, even from a finally clause entered because of an
    // aborting error.
    if did_emsg.get() == 0 {
        cause_abort.set(force_abort.get());
        force_abort.set(false);
    }

    // No try conditional active, nothing being thrown and no error inside a
    // try conditional so far: do nothing, for the sake of non-EH scripts.
    // Under ":silent!" and outside a throw, likewise -- `emsg` will store
    // the text in v:errmsg without displaying it.
    if (trylevel.get() == 0 && !cause_abort.get() || emsg_silent.get() != 0) && !did_throw.get() {
        return false;
    }

    // Ignore an interrupt message inside a try conditional, so that the
    // interrupt exception stays catchable by the innermost one instead of
    // being replaced by an error exception carrying its text. The identity
    // test is upstream's: only *this* message is meant.
    if mesg == message(&e_interr) {
        // SAFETY: caller contract.
        unsafe { *ignore = true };
        return true;
    }

    // Abort every command in nested calls and sourced files immediately.
    cause_abort.set(true);

    // Some commands (the conditionals) are not skipped while an exception is
    // being thrown, and an error in one of those changes which of the
    // following commands count as catch and finally clauses. Catching the
    // exception would then run commands the user never wrote, without them
    // noticing. So discard what is being thrown, run the finally clauses and
    // terminate.
    if did_throw.get() {
        // SAFETY: `did_throw` implies a live `current_exception`. Resetting
        // `got_int` for an interrupt stops the same interrupt becoming an
        // exception again and discarding the error about to be thrown here.
        if unsafe { (*current_exception.get()).type_0 } == ET_INTERRUPT {
            got_int.set(false);
        }
        // SAFETY: as above.
        unsafe { discard_current_exception() };
    }

    // Prepare the throw: everything but the finally clauses is aborted until
    // the exception is caught, and if it is still uncaught at the top level
    // the message is displayed and the script terminated. There is no access
    // to the conditional stack here, so the actual throw waits until the
    // failing command has returned. Only the first of several errors in a
    // row is thrown, unless a severe one follows.
    if msg_list.get().is_null() {
        return true;
    }
    // SAFETY: `msg_list` points at the current `do_cmdline`'s own list head.
    unsafe { append_msg(mesg, multiline, concat, severe) }
}

/// Append `mesg` to the message list the error exception will be built from,
/// or concatenate it onto the last entry. Answers whether `emsg` should stay
/// quiet, which it always should by this point.
///
/// # Safety
/// `msg_list` is non-null and `mesg` is NUL-terminated.
unsafe fn append_msg(mesg: *const c_char, multiline: bool, concat: bool, severe: bool) -> bool {
    // SAFETY: caller contract; the list is this `do_cmdline`'s own.
    unsafe {
        let head = msg_list.get();
        let mut plist = head;
        while !(*plist).is_null() {
            // Concatenate onto the last entry (a multi-part message).
            if (**plist).next.is_null() && concat {
                let joined = strlen((**plist).msg) + strlen(mesg) + 1;
                (**plist).msg = xrealloc((**plist).msg.cast(), joined).cast();
                (**plist).throw_msg = strcat((**plist).msg, mesg);
                return true;
            }
            plist = &raw mut (**plist).next;
        }

        let elem: *mut msglist_T = xmalloc(core::mem::size_of::<msglist_T>()).cast();
        (*elem).msg = xstrdup(mesg);
        (*elem).multiline = multiline;
        (*elem).next = ptr::null_mut();
        (*elem).throw_msg = ptr::null_mut();
        *plist = elem;

        if plist == head || severe {
            // Skip the extra "Vim " prefix, as on message "E458".
            let tmsg = (*elem).msg;
            let vim_prefixed = strncmp(tmsg, c"Vim E".as_ptr(), 5) == 0
                && ascii_isdigit(*tmsg.add(5) as c_int)
                && ascii_isdigit(*tmsg.add(6) as c_int)
                && ascii_isdigit(*tmsg.add(7) as c_int)
                && *tmsg.add(8) == b':' as c_char
                && *tmsg.add(9) == b' ' as c_char;
            (**head).throw_msg = if vim_prefixed { tmsg.add(4) } else { tmsg };
        }

        // Take the source name and line number now: they may change before
        // `do_errthrow` runs.
        (*elem).sfile = estack_sfile(ESTACK_NONE);
        (*elem).slnum = sourcing_lnum();
        true
    }
}

/// Free a message list and everything in it.
///
/// # Safety
/// `l` heads a message list this owns.
unsafe fn free_msglist(l: *mut msglist_T) {
    // SAFETY: caller contract.
    unsafe {
        let mut messages = l;
        while !messages.is_null() {
            let next = (*messages).next;
            xfree((*messages).msg.cast());
            xfree((*messages).sfile.cast());
            xfree(messages.cast());
            messages = next;
        }
    }
}

/// Free the global `*msg_list` and clear it.
///
/// # Safety
/// Module contract.
pub unsafe fn free_global_msglist() {
    // SAFETY: module contract.
    unsafe {
        free_msglist(*msg_list.get());
        *msg_list.get() = ptr::null_mut();
    }
}

/// Throw what [`cause_errthrow`] collected as an error exception. With a
/// null `cstack` the throw waits until `do_cmdline` returns -- see
/// `do_one_cmd`.
///
/// # Safety
/// Module contract; `cstack`, when non-null, is the running one.
pub unsafe fn do_errthrow(cstack: *mut cstack_T, cmdname: *mut c_char) {
    // Abort every command in nested calls and sourced files immediately.
    if cause_abort.get() {
        cause_abort.set(false);
        force_abort.set(true);
    }

    // SAFETY: module contract.
    unsafe {
        // Nothing to throw, or the conversion belongs to an outer
        // `do_one_cmd`.
        if msg_list.get().is_null() || (*msg_list.get()).is_null() {
            return;
        }
        if throw_exception((*msg_list.get()).cast(), ET_ERROR, cmdname) == FAIL {
            free_msglist(*msg_list.get());
        } else if !cstack.is_null() {
            super::trycmd::do_throw(cstack);
        } else {
            need_rethrow.set(true);
        }
        *msg_list.get() = ptr::null_mut();
    }
}

/// Replace the current exception by an interrupt exception, if an interrupt
/// happened and anyone could catch it. Answers whether the current exception
/// was discarded.
///
/// # Safety
/// Module contract; `cstack` is the running conditional stack.
pub unsafe fn do_intthrow(cstack: *mut cstack_T) -> bool {
    // No interrupt, or no try conditional active and nothing being thrown:
    // do nothing, for the sake of non-EH scripts.
    if !got_int.get() || (trylevel.get() == 0 && !did_throw.get()) {
        return false;
    }

    // SAFETY: module contract; `did_throw` implies a live
    // `current_exception`.
    unsafe {
        if did_throw.get() {
            // An interrupt exception already being thrown stands.
            if (*current_exception.get()).type_0 == ET_INTERRUPT {
                return false;
            }
            // Otherwise it replaces the user or error exception.
            discard_current_exception();
        }
        if throw_exception(
            c"Vim:Interrupt".as_ptr().cast_mut().cast(),
            ET_INTERRUPT,
            ptr::null_mut(),
        ) != FAIL
        {
            super::trycmd::do_throw(cstack);
        }
    }
    true
}

/// The string an exception is matched and reported by.
///
/// For an error exception this is built from the message list and prefixed
/// with `Vim:` or `Vim(cmdname):`, and `should_free` is set. For the other
/// two kinds it is `value` itself, unowned.
///
/// # Safety
/// Module contract. `value` is a message list for [`ET_ERROR`] and a
/// NUL-terminated string otherwise; `should_free` is writable.
pub unsafe fn get_exception_string(
    value: *mut c_void,
    type_0: except_type_T,
    cmdname: *mut c_char,
    should_free: *mut bool,
) -> *mut c_char {
    // SAFETY: caller contract.
    unsafe {
        if type_0 != ET_ERROR {
            *should_free = false;
            return value.cast();
        }
        *should_free = true;

        let mesg = (*value.cast::<msglist_T>()).throw_msg;
        let ret;
        let val;
        if !cmdname.is_null() && *cmdname != NUL as c_char {
            let cmdlen = strlen(cmdname);
            ret = xstrnsave(c"Vim(".as_ptr(), 4 + cmdlen + 2 + strlen(mesg));
            strcpy(ret.add(4), cmdname);
            strcpy(ret.add(4 + cmdlen), c"):".as_ptr());
            val = ret.add(4 + cmdlen + 2);
        } else {
            ret = xstrnsave(c"Vim:".as_ptr(), 4 + strlen(mesg));
            val = ret.add(4);
        }

        // `msg_add_fname` may have prefixed the message with a file name in
        // quotes. In the exception value the file name goes in parentheses
        // at the end instead.
        let mut p = mesg;
        loop {
            if *p == NUL as c_char || error_number_at(p) {
                if *p == NUL as c_char || p == mesg {
                    // "E123" missing, or at the very beginning.
                    strcat(val, mesg);
                    break;
                }
                if *mesg != b'"' as c_char
                    || p.sub(2) < mesg.add(1)
                    || *p.sub(2) != b'"' as c_char
                    || *p.sub(1) != b' ' as c_char
                {
                    // "E123:" is part of the file name after all.
                    p = p.add(1);
                    continue;
                }
                // '"filename" E123: message text'
                strcat(val, p);
                *p.sub(2) = NUL as c_char;
                snprintf(
                    val.add(strlen(p)),
                    c" (%s)".count_bytes(),
                    c" (%s)".as_ptr(),
                    mesg.add(1),
                );
                *p.sub(2) = b'"' as c_char;
                break;
            }
            p = p.add(1);
        }
        ret
    }
}

/// Whether `p` starts an `E123:` message number, with one to three digits.
///
/// # Safety
/// `p` points into a NUL-terminated string.
unsafe fn error_number_at(p: *const c_char) -> bool {
    // SAFETY: caller contract -- each read is guarded by the one before it,
    // and the NUL stops the walk.
    unsafe {
        *p == b'E' as c_char
            && ascii_isdigit(*p.add(1) as c_int)
            && (*p.add(2) == b':' as c_char
                || ascii_isdigit(*p.add(2) as c_int)
                    && (*p.add(3) == b':' as c_char
                        || ascii_isdigit(*p.add(3) as c_int) && *p.add(4) == b':' as c_char))
    }
}

/// Build the exception and make it the one being thrown. `value` is the
/// string for a user or interrupt exception and a message list for an error
/// one.
///
/// Answers `FAIL` when out of memory or when a user exception tried to fake
/// a `Vim` one.
///
/// # Safety
/// Module contract, and `value` matches `type_0` as
/// [`get_exception_string`] describes.
pub(super) unsafe fn throw_exception(
    value: *mut c_void,
    type_0: except_type_T,
    cmdname: *mut c_char,
) -> c_int {
    // SAFETY: caller contract.
    unsafe {
        // Faking an interrupt or error exception as a user one is not
        // allowed: `do_cmdline` treats the two differently when no active
        // try block is found.
        if type_0 == ET_USER {
            let v = value.cast::<c_char>();
            if strncmp(v, c"Vim".as_ptr(), 3) == 0
                && (*v.add(3) == NUL as c_char
                    || *v.add(3) == b':' as c_char
                    || *v.add(3) == b'(' as c_char)
            {
                emsg(c"E608: Cannot :throw exceptions with 'Vim' prefix".as_ptr());
                current_exception.set(ptr::null_mut());
                return FAIL;
            }
        }

        let excp: *mut except_T = xmalloc(core::mem::size_of::<except_T>()).cast();
        if type_0 == ET_ERROR {
            // Keep the original messages; the value is prefixed below.
            (*excp).messages = value.cast();
        }

        let mut should_free = false;
        (*excp).value = get_exception_string(value, type_0, cmdname, &raw mut should_free);
        if (*excp).value.is_null() && should_free {
            xfree(excp.cast());
            suppress_errthrow.set(true);
            emsg(message(&e_outofmem));
            current_exception.set(ptr::null_mut());
            return FAIL;
        }

        (*excp).type_0 = type_0;
        // An error exception throws from where the message was made, which
        // is not where we are now.
        let entry = value.cast::<msglist_T>();
        if type_0 == ET_ERROR && !(*entry).sfile.is_null() {
            (*excp).throw_name = (*entry).sfile;
            (*entry).sfile = ptr::null_mut();
            (*excp).throw_lnum = (*entry).slnum;
        } else {
            (*excp).throw_name = estack_sfile(ESTACK_NONE);
            if (*excp).throw_name.is_null() {
                (*excp).throw_name = xstrdup(c"".as_ptr());
            }
            (*excp).throw_lnum = sourcing_lnum();
        }

        (*excp).stacktrace = stacktrace_create();
        tv_list_ref((*excp).stacktrace);

        verbose_exception(c"Exception thrown: %s", (*excp).value);

        current_exception.set(excp);
        OK
    }
}

/// Report an exception's fate under 'verbose' >= 13 or while debugging.
///
/// # Safety
/// `mesg` holds one `%s` and `value` is NUL-terminated.
unsafe fn verbose_exception(mesg: &CStr, value: *mut c_char) {
    if p_verbose.get() < 13 && debug_break_level.get() <= 0 {
        return;
    }
    // SAFETY: caller contract.
    unsafe {
        let save_msg_silent = msg_silent.get();
        if debug_break_level.get() > 0 {
            // Display messages.
            msg_silent.set(0);
        } else {
            verbose_enter();
        }
        no_wait_return.set(no_wait_return.get() + 1);
        if debug_break_level.get() > 0 || *p_vfile.get() == NUL as c_char {
            // Always scroll up, don't overwrite.
            msg_scroll.set(1);
        }
        smsg_c!(0, mesg.as_ptr(), value);
        // Don't overwrite this either.
        msg_puts(c"\n".as_ptr());
        if debug_break_level.get() > 0 || *p_vfile.get() == NUL as c_char {
            cmdline_row.set(msg_row.get());
        }
        no_wait_return.set(no_wait_return.get() - 1);
        if debug_break_level.get() > 0 {
            msg_silent.set(save_msg_silent);
        } else {
            verbose_leave();
        }
    }
}

/// Free an exception. `was_finished` picks the 'verbose' wording: a caught
/// exception whose catch clause ended normally is *finished*, anything else
/// is *discarded*.
///
/// # Safety
/// Module contract; `excp` is owned and not on the caught stack.
pub(super) unsafe fn discard_exception(excp: *mut except_T, was_finished: bool) {
    if current_exception.get() == excp {
        current_exception.set(ptr::null_mut());
    }
    if excp.is_null() {
        // SAFETY: module contract.
        unsafe { internal_error(c"discard_exception()".as_ptr()) };
        return;
    }

    // SAFETY: caller contract.
    unsafe {
        if p_verbose.get() >= 13 || debug_break_level.get() > 0 {
            // The report formats through IObuff, which a caller may be
            // holding a message in.
            let saved_iobuff = xstrdup(iobuff());
            verbose_exception(
                if was_finished {
                    c"Exception finished: %s"
                } else {
                    c"Exception discarded: %s"
                },
                (*excp).value,
            );
            xstrlcpy(iobuff(), saved_iobuff, IOSIZE);
            xfree(saved_iobuff.cast());
        }
        if (*excp).type_0 != ET_INTERRUPT {
            // An interrupt exception's value is a string literal.
            xfree((*excp).value.cast());
        }
        if (*excp).type_0 == ET_ERROR {
            free_msglist((*excp).messages);
        }
        xfree((*excp).throw_name.cast());
        tv_list_unref((*excp).stacktrace);
        xfree(excp.cast());
    }
}

/// Discard the exception currently being thrown.
///
/// # Safety
/// Module contract.
pub unsafe fn discard_current_exception() {
    if !current_exception.get().is_null() {
        // SAFETY: module contract.
        unsafe { discard_exception(current_exception.get(), false) };
    }
    // Everything reset here is saved and restored by
    // `exception_state_save`/`_restore`.
    did_throw.set(false);
    need_rethrow.set(false);
}

/// Point `v:exception`, `v:throwpoint` and `v:stacktrace` at `excp`, or
/// clear all three when it is null.
///
/// # Safety
/// Module contract; `excp`, when non-null, is a live exception.
unsafe fn set_exception_vars(excp: *mut except_T) {
    // SAFETY: caller contract.
    unsafe {
        if excp.is_null() {
            set_vim_var_string(VV_EXCEPTION, ptr::null(), -1);
            set_vim_var_string(VV_THROWPOINT, ptr::null(), -1);
            set_vim_var_list(VV_STACKTRACE, ptr::null_mut::<list_T>());
            return;
        }
        set_vim_var_string(VV_EXCEPTION, (*excp).value, -1);
        set_vim_var_list(VV_STACKTRACE, (*excp).stacktrace);
        if *(*excp).throw_name == NUL as c_char {
            // `throw_name` is unset for an exception from a typed command.
            set_vim_var_string(VV_THROWPOINT, ptr::null(), -1);
            return;
        }
        let len = if (*excp).throw_lnum == 0 {
            vim_snprintf_safelen(iobuff(), IOSIZE, c"%s".as_ptr(), (*excp).throw_name)
        } else {
            vim_snprintf_safelen(
                iobuff(),
                IOSIZE,
                c"%s, line %ld".as_ptr(),
                (*excp).throw_name,
                (*excp).throw_lnum as int64_t,
            )
        };
        set_vim_var_string(VV_THROWPOINT, iobuff(), len as ptrdiff_t);
    }
}

/// Push an exception onto the caught stack.
///
/// # Safety
/// Module contract; `excp` is the exception just matched.
pub(super) unsafe fn catch_exception(excp: *mut except_T) {
    // SAFETY: caller contract.
    unsafe {
        (*excp).caught = caught_stack.get();
        caught_stack.set(excp);
        set_exception_vars(excp);
        verbose_exception(c"Exception caught: %s", (*excp).value);
    }
}

/// Pop `excp` off the caught stack and free it, restoring `v:exception` and
/// friends to the exception below it.
///
/// # Safety
/// Module contract; `excp` is the top of the caught stack.
pub(super) unsafe fn finish_exception(excp: *mut except_T) {
    // SAFETY: caller contract.
    unsafe {
        if excp != caught_stack.get() {
            internal_error(c"finish_exception()".as_ptr());
        }
        caught_stack.set((*caught_stack.get()).caught);
        set_exception_vars(caught_stack.get());
        // Discard it, but use the "finished" wording for 'verbose'.
        discard_exception(excp, true);
    }
}

/// Save the exception state, for a nested `do_cmdline` that must not see it.
///
/// # Safety
/// Module contract; `estate` is writable.
pub unsafe fn exception_state_save(estate: *mut exception_state_T) {
    // SAFETY: caller contract.
    unsafe {
        (*estate).estate_current_exception = current_exception.get();
        (*estate).estate_did_throw = did_throw.get();
        (*estate).estate_need_rethrow = need_rethrow.get();
        (*estate).estate_trylevel = trylevel.get();
        (*estate).estate_did_emsg = did_emsg.get();
    }
}

/// Restore what [`exception_state_save`] stored, after handling anything
/// thrown meanwhile.
///
/// # Safety
/// Module contract; `estate` was filled by [`exception_state_save`].
pub unsafe fn exception_state_restore(estate: *mut exception_state_T) {
    // SAFETY: caller contract.
    unsafe {
        if did_throw.get() {
            handle_did_throw();
        }
        current_exception.set((*estate).estate_current_exception);
        did_throw.set((*estate).estate_did_throw);
        need_rethrow.set((*estate).estate_need_rethrow);
        trylevel.set((*estate).estate_trylevel);
        did_emsg.set((*estate).estate_did_emsg);
    }
}

/// Forget the exception state entirely.
pub fn exception_state_clear() {
    current_exception.set(ptr::null_mut());
    did_throw.set(false);
    need_rethrow.set(false);
    trylevel.set(0);
    did_emsg.set(0);
}

/// What [`report_pending`] is saying about the pending thing.
#[derive(Clone, Copy)]
enum PendingAction {
    Made,
    Resumed,
    Discarded,
}

impl PendingAction {
    /// The `printf` format, with the `%s` naming what is pending.
    fn message(self) -> &'static CStr {
        match self {
            Self::Made => c"%s made pending",
            Self::Resumed => c"%s resumed",
            Self::Discarded => c"%s discarded",
        }
    }
}

/// Report what a finally clause made pending, resumed or discarded.
/// `value` is the return value for a pending `:return` and the exception for
/// a pending throw.
///
/// # Safety
/// Module contract; `value` matches `pending`, and is non-null whenever
/// `pending` carries [`CSTP_THROW`].
unsafe fn report_pending(action: PendingAction, pending: c_int, value: *mut c_void) {
    debug_assert!(
        !value.is_null() || pending & CSTP_THROW == 0,
        "value || !(pending & CSTP_THROW)"
    );
    let mut mesg = action.message().as_ptr().cast_mut();

    // SAFETY: caller contract.
    unsafe {
        let s = match pending {
            CSTP_NONE => return,
            CSTP_CONTINUE => c":continue".as_ptr().cast_mut(),
            CSTP_BREAK => c":break".as_ptr().cast_mut(),
            CSTP_FINISH => c":finish".as_ptr().cast_mut(),
            // A ":return" producing a value; the text is allocated.
            CSTP_RETURN => get_return_cmd(value),
            _ if pending & CSTP_THROW != 0 => {
                // "%s made pending" becomes "Exception made pending: %s".
                vim_snprintf(iobuff(), IOSIZE, mesg, c"Exception".as_ptr());
                mesg = concat_str(iobuff(), c": %s".as_ptr());
                (*value.cast::<except_T>()).value
            }
            _ if pending & CSTP_ERROR != 0 && pending & CSTP_INTERRUPT != 0 => {
                c"Error and interrupt".as_ptr().cast_mut()
            }
            _ if pending & CSTP_ERROR != 0 => c"Error".as_ptr().cast_mut(),
            // Only CSTP_INTERRUPT is left.
            _ => c"Interrupt".as_ptr().cast_mut(),
        };

        let save_msg_silent = msg_silent.get();
        if debug_break_level.get() > 0 {
            // Display messages.
            msg_silent.set(0);
        }
        no_wait_return.set(no_wait_return.get() + 1);
        // Always scroll up, don't overwrite.
        msg_scroll.set(1);
        smsg_c!(0, mesg, s);
        // Don't overwrite this either.
        msg_puts(c"\n".as_ptr());
        cmdline_row.set(msg_row.get());
        no_wait_return.set(no_wait_return.get() - 1);
        if debug_break_level.get() > 0 {
            msg_silent.set(save_msg_silent);
        }

        if pending == CSTP_RETURN {
            xfree(s.cast());
        } else if pending & CSTP_THROW != 0 {
            xfree(mesg.cast());
        }
    }
}

/// [`report_pending`] under 'verbose' >= 14 or while debugging, which is the
/// only way any of the three wrappers below reaches it.
///
/// # Safety
/// As [`report_pending`].
unsafe fn report_if_verbose(action: PendingAction, pending: c_int, value: *mut c_void) {
    if p_verbose.get() < 14 && debug_break_level.get() <= 0 {
        return;
    }
    let quiet = debug_break_level.get() <= 0;
    // SAFETY: caller contract.
    unsafe {
        if quiet {
            verbose_enter();
        }
        report_pending(action, pending, value);
        if quiet {
            verbose_leave();
        }
    }
}

/// Report something a finally clause made pending.
///
/// # Safety
/// As [`report_pending`].
pub unsafe fn report_make_pending(pending: c_int, value: *mut c_void) {
    // SAFETY: caller contract.
    unsafe { report_if_verbose(PendingAction::Made, pending, value) }
}

/// Report something pending being resumed at the `:endtry`.
///
/// # Safety
/// As [`report_pending`].
pub(super) unsafe fn report_resume_pending(pending: c_int, value: *mut c_void) {
    // SAFETY: caller contract.
    unsafe { report_if_verbose(PendingAction::Resumed, pending, value) }
}

/// Report something pending being thrown away.
///
/// # Safety
/// As [`report_pending`].
pub(super) unsafe fn report_discard_pending(pending: c_int, value: *mut c_void) {
    // SAFETY: caller contract.
    unsafe { report_if_verbose(PendingAction::Discarded, pending, value) }
}
