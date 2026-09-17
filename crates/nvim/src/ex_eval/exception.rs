//! The exception object: how an error, an interrupt or a `:throw` becomes a
//! catchable value, and what happens to it afterwards.
//!
//! An exception is an [`Exception`] holding the value a `:catch` pattern is
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
#![allow(unsafe_code)]

use super::flag::{
    CSTP_BREAK, CSTP_CONTINUE, CSTP_ERROR, CSTP_FINISH, CSTP_INTERRUPT, CSTP_NONE, CSTP_RETURN,
    CSTP_THROW, ESTACK_NONE, ET_ERROR, ET_INTERRUPT, ET_USER,
};
use super::{cause_abort, message};
use crate::ascii::ascii_isdigit;
use crate::cstr;
use crate::debugger::state::debug_break_level;
use crate::drawscreen::state::cmdline_row;
use crate::eval::typval::{ListRef, list_unref};
use crate::eval::userfunc::get_return_cmd;
use crate::eval::vars::{set_vim_var_list, set_vim_var_string};
use crate::ex_docmd::handle_did_throw;
use crate::ex_eval::state::{
    caught_stack, current_exception, did_throw, force_abort, msg_list, need_rethrow,
    suppress_errthrow, trylevel,
};
use crate::getchar::state::got_int;
use crate::guard::{Allow, Suppress};
use crate::memory::XString;
use crate::memory::{xfree, xmalloc, xrealloc, xstrdup};
use crate::message::state::{did_emsg, emsg_silent, msg_row, msg_scroll};
use crate::message::{e_interr, e_outofmem};
use crate::message::{emsg, emsg_ptr, internal_error, msg_str, verbose_enter, verbose_leave};
use crate::message_fmt::{c_str, report_msg};
use crate::option::p_vfile;
use crate::option::vars::p_verbose;
use crate::os::cshim::gettext_ptr;
use crate::runtime::{estack_sfile, sourcing_lnum, stacktrace_create};
use crate::strings::{concat_str, vim_snprintf, vim_snprintf_safelen};
use crate::tr_plural;
use crate::types::{
    CondStack, ExceptType, Exception, ExceptionState, Failed, IOSIZE, MsgList, NUL, Vv, int64_t,
    ptrdiff_t,
};
use ::libc::strcat;
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
pub(crate) unsafe fn cause_errthrow(
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
    if mesg == message(e_interr) {
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
        discard_current_exception();
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
    let head = msg_list.get();
    let mut plist = head;
    while !unsafe { *plist }.is_null() {
        // Concatenate onto the last entry (a multi-part message).
        if unsafe { (**plist).next }.is_null() && concat {
            let (old, add) = unsafe { (cstr::bytes_at((**plist).msg), cstr::bytes_at(mesg)) };
            let joined = old.len() + add.len() + 1;
            unsafe { (**plist).msg = xrealloc((**plist).msg.cast(), joined).cast() };
            unsafe { (**plist).throw_msg = strcat((**plist).msg, mesg) };
            return true;
        }
        plist = unsafe { &raw mut (**plist).next };
    }

    let elem: *mut MsgList = unsafe { xmalloc(size_of::<MsgList>()) }.cast();
    unsafe { (*elem).msg = xstrdup(mesg) };
    unsafe { (*elem).multiline = multiline };
    unsafe { (*elem).next = ptr::null_mut() };
    unsafe { (*elem).throw_msg = ptr::null_mut() };
    unsafe { *plist = elem };

    if plist == head || severe {
        // Skip the extra "Vim " prefix, as on message "E458".
        let tmsg = unsafe { (*elem).msg };
        let vim_prefixed = unsafe { cstr::starts_with(tmsg, b"Vim E") }
            && ascii_isdigit(unsafe { *tmsg.add(5) } as c_int)
            && ascii_isdigit(unsafe { *tmsg.add(6) } as c_int)
            && ascii_isdigit(unsafe { *tmsg.add(7) } as c_int)
            && unsafe { *tmsg.add(8) } == b':' as c_char
            && unsafe { *tmsg.add(9) } == b' ' as c_char;
        unsafe { (**head).throw_msg = if vim_prefixed { tmsg.add(4) } else { tmsg } };
    }

    // Take the source name and line number now: they may change before
    // `do_errthrow` runs.
    unsafe { (*elem).sfile = estack_sfile(ESTACK_NONE) };
    unsafe { (*elem).slnum = sourcing_lnum() };
    true
}

/// Free a message list and everything in it.
///
/// # Safety
/// `l` heads a message list this owns.
unsafe fn free_msglist(l: *mut MsgList) {
    let mut messages = l;
    while !messages.is_null() {
        let next = unsafe { (*messages).next };
        unsafe { xfree((*messages).msg.cast()) };
        unsafe { xfree((*messages).sfile.cast()) };
        unsafe { xfree(messages.cast()) };
        messages = next;
    }
}

/// Free the global `*msg_list` and clear it.
pub(crate) fn free_global_msglist() {
    // SAFETY: module contract.
    unsafe { free_msglist(*msg_list.get()) };
    unsafe { *msg_list.get() = ptr::null_mut() };
}

/// Throw what [`cause_errthrow`] collected as an error exception. With a
/// null `cstack` the throw waits until `do_cmdline` returns -- see
/// `do_one_cmd`.
///
/// # Safety
/// Module contract; `cstack`, when non-null, is the running one.
pub(crate) unsafe fn do_errthrow(cstack: *mut CondStack, cmdname: *mut c_char) {
    // Abort every command in nested calls and sourced files immediately.
    if cause_abort.get() {
        cause_abort.set(false);
        force_abort.set(true);
    }

    // SAFETY: module contract.
    // Nothing to throw, or the conversion belongs to an outer
    // `do_one_cmd`.
    if msg_list.get().is_null() || unsafe { *msg_list.get() }.is_null() {
        return;
    }
    if unsafe { throw_exception((*msg_list.get()).cast(), ET_ERROR, cmdname) }.is_err() {
        unsafe { free_msglist(*msg_list.get()) };
    } else if !cstack.is_null() {
        unsafe { super::trycmd::do_throw(cstack) };
    } else {
        need_rethrow.set(true);
    }
    unsafe { *msg_list.get() = ptr::null_mut() };
}

/// Replace the current exception by an interrupt exception, if an interrupt
/// happened and anyone could catch it. Answers whether the current exception
/// was discarded.
///
/// # Safety
/// Module contract; `cstack` is the running conditional stack.
pub(crate) unsafe fn do_intthrow(cstack: *mut CondStack) -> bool {
    // No interrupt, or no try conditional active and nothing being thrown:
    // do nothing, for the sake of non-EH scripts.
    if !got_int.get() || (trylevel.get() == 0 && !did_throw.get()) {
        return false;
    }

    // SAFETY: module contract; `did_throw` implies a live
    // `current_exception`.
    if did_throw.get() {
        // An interrupt exception already being thrown stands.
        if unsafe { (*current_exception.get()).type_0 } == ET_INTERRUPT {
            return false;
        }
        // Otherwise it replaces the user or error exception.
        discard_current_exception();
    }
    if unsafe {
        throw_exception(
            c"Vim:Interrupt".as_ptr().cast_mut().cast(),
            ET_INTERRUPT,
            ptr::null_mut(),
        )
    }
    .is_ok()
    {
        unsafe { super::trycmd::do_throw(cstack) };
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
pub(crate) unsafe fn get_exception_string(
    value: *mut c_void,
    type_0: ExceptType,
    cmdname: *mut c_char,
    should_free: *mut bool,
) -> *mut c_char {
    // SAFETY: caller contract.
    if type_0 != ET_ERROR {
        unsafe { *should_free = false };
        return value.cast();
    }
    unsafe { *should_free = true };

    // SAFETY: caller contract -- an error's value is a message list whose
    // `throw_msg` is NUL-terminated, and `cmdname` is null or a string.
    let (message, command) = unsafe {
        (
            cstr::bytes_at((*value.cast::<MsgList>()).throw_msg),
            cstr::bytes_at_or_empty(cmdname),
        )
    };
    let mut ret = XString::from_bytes(b"Vim");
    if command.is_empty() {
        ret.push_byte(b':');
    } else {
        ret.push_byte(b'(');
        ret.push_bytes(command);
        ret.push_bytes(b"):");
    }
    ret.push_bytes(&exception_message(message));
    ret.into_raw()
}

/// The message as an exception value carries it: `message` itself, unless
/// [`msg_add_fname`](crate::message::msg_add_fname) prefixed it with a file
/// name in quotes, in which case the name moves to the end in parentheses.
///
/// **The move truncates the name to two bytes.** Upstream passes the format
/// string's own length (`strlen(" (%s)")`, 5) as the destination size, so
/// `snprintf` writes `" ("` and two more bytes whatever the name is. It is a
/// truncation, not an overrun -- the block is far larger than the size given
/// -- and it is what every `v:exception` has said since Vim 7, so it is
/// reproduced here rather than diverged from.
fn exception_message(message: &[u8]) -> Vec<u8> {
    let mut at = 0;
    loop {
        if at == message.len() || error_number_at(&message[at..]) {
            if at == message.len() || at == 0 {
                // "E123" missing, or at the very beginning.
                return message.to_vec();
            }
            if message[0] != b'"' || at < 3 || message[at - 2] != b'"' || message[at - 1] != b' ' {
                // "E123:" is part of the file name after all.
                at += 1;
                continue;
            }
            // '"filename" E123: message text'
            let mut moved = message[at..].to_vec();
            let mut parenthesised = b" (".to_vec();
            parenthesised.extend_from_slice(&message[1..at - 2]);
            parenthesised.push(b')');
            parenthesised.truncate(c" (%s)".count_bytes() - 1);
            moved.extend_from_slice(&parenthesised);
            return moved;
        }
        at += 1;
    }
}

/// Whether `rest` opens with an `E123:` message number, one to three digits.
fn error_number_at(rest: &[u8]) -> bool {
    let digit = |i: usize| rest.get(i).is_some_and(u8::is_ascii_digit);
    let colon = |i: usize| rest.get(i) == Some(&b':');
    rest.first() == Some(&b'E')
        && digit(1)
        && (colon(2) || digit(2) && (colon(3) || digit(3) && colon(4)))
}

/// Build the exception and make it the one being thrown. `value` is the
/// string for a user or interrupt exception and a message list for an error
/// one.
///
/// Answers `Err` when out of memory or when a user exception tried to fake
/// a `Vim` one.
///
/// # Safety
/// Module contract, and `value` matches `type_0` as
/// [`get_exception_string`] describes.
pub(super) unsafe fn throw_exception(
    value: *mut c_void,
    type_0: ExceptType,
    cmdname: *mut c_char,
) -> Result<(), Failed> {
    // SAFETY: caller contract.
    // Faking an interrupt or error exception as a user one is not
    // allowed: `do_cmdline` treats the two differently when no active
    // try block is found.
    if type_0 == ET_USER {
        let v = value.cast::<c_char>();
        if unsafe { cstr::starts_with(v, b"Vim") }
            && (unsafe { *v.add(3) } == NUL as c_char
                || unsafe { *v.add(3) } == b':' as c_char
                || unsafe { *v.add(3) } == b'(' as c_char)
        {
            emsg(c"E608: Cannot :throw exceptions with 'Vim' prefix");
            current_exception.set(ptr::null_mut());
            return Err(Failed);
        }
    }

    let excp: *mut Exception = unsafe { xmalloc(size_of::<Exception>()) }.cast();
    if type_0 == ET_ERROR {
        // Keep the original messages; the value is prefixed below.
        unsafe { (*excp).messages = value.cast() };
    }

    let mut should_free = false;
    unsafe { (*excp).value = get_exception_string(value, type_0, cmdname, &raw mut should_free) };
    if unsafe { (*excp).value }.is_null() && should_free {
        unsafe { xfree(excp.cast()) };
        suppress_errthrow.set(true);
        unsafe { emsg_ptr(message(e_outofmem)) };
        current_exception.set(ptr::null_mut());
        return Err(Failed);
    }

    unsafe { (*excp).type_0 = type_0 };
    // An error exception throws from where the message was made, which
    // is not where we are now.
    let entry = value.cast::<MsgList>();
    if type_0 == ET_ERROR && !unsafe { (*entry).sfile }.is_null() {
        unsafe { (*excp).throw_name = (*entry).sfile };
        unsafe { (*entry).sfile = ptr::null_mut() };
        unsafe { (*excp).throw_lnum = (*entry).slnum };
    } else {
        unsafe { (*excp).throw_name = estack_sfile(ESTACK_NONE) };
        if unsafe { (*excp).throw_name }.is_null() {
            unsafe { (*excp).throw_name = xstrdup(c"".as_ptr()) };
        }
        unsafe { (*excp).throw_lnum = sourcing_lnum() };
    }

    // The exception owns the stack trace it was thrown with.
    unsafe { (*excp).stacktrace = stacktrace_create().map_or(ptr::null_mut(), ListRef::into_raw) };

    unsafe { verbose_exception(c"Exception thrown: %s", (*excp).value) };

    current_exception.set(excp);
    Ok(())
}

/// Report an exception's fate under 'verbose' >= 13 or while debugging.
///
/// # Safety
/// `mesg` holds one `%s` and `value` is NUL-terminated.
unsafe fn verbose_exception(mesg: &CStr, value: *mut c_char) {
    if p_verbose.get() < 13 && debug_break_level.get() <= 0 {
        return;
    }
    let debugging = debug_break_level.get() > 0;
    // While debugging the messages have to be displayed.
    let loud = debugging.then(Allow::messages);
    if !debugging {
        verbose_enter();
    }
    let no_prompt = Suppress::wait_return();
    if debug_break_level.get() > 0 || unsafe { *p_vfile.get() } == NUL as c_char {
        // Always scroll up, don't overwrite.
        msg_scroll.set(1);
    }
    // SAFETY: the caller's NUL-terminated value, and `mesg`, which outlives
    // the message it is translated for.
    let (template, value) = unsafe { (gettext_ptr(mesg.as_ptr()), c_str(value)) };
    let _: bool = report_msg(0, || tr_plural!(template, value));
    // Don't overwrite this either.
    msg_str(c"\n");
    if debug_break_level.get() > 0 || unsafe { *p_vfile.get() } == NUL as c_char {
        cmdline_row.set(msg_row.get());
    }
    drop(no_prompt);
    drop(loud);
    if !debugging {
        verbose_leave();
    }
}

/// Free an exception. `was_finished` picks the 'verbose' wording: a caught
/// exception whose catch clause ended normally is *finished*, anything else
/// is *discarded*.
///
/// # Safety
/// Module contract; `excp` is owned and not on the caught stack.
pub(super) unsafe fn discard_exception(excp: *mut Exception, was_finished: bool) {
    if current_exception.get() == excp {
        current_exception.set(ptr::null_mut());
    }
    if excp.is_null() {
        // SAFETY: module contract.
        unsafe { internal_error(c"discard_exception()".as_ptr()) };
        return;
    }

    // SAFETY: caller contract.
    if p_verbose.get() >= 13 || debug_break_level.get() > 0 {
        // Upstream saves and restores `IObuff` around this, because the
        // report formatted through it and a caller may have been holding
        // a message there. Nothing shares a buffer here any more.
        unsafe {
            verbose_exception(
                if was_finished {
                    c"Exception finished: %s"
                } else {
                    c"Exception discarded: %s"
                },
                (*excp).value,
            )
        };
    }
    if unsafe { (*excp).type_0 } != ET_INTERRUPT {
        // An interrupt exception's value is a string literal.
        unsafe { xfree((*excp).value.cast()) };
    }
    if unsafe { (*excp).type_0 } == ET_ERROR {
        unsafe { free_msglist((*excp).messages) };
    }
    unsafe { xfree((*excp).throw_name.cast()) };
    unsafe { list_unref((*excp).stacktrace) };
    unsafe { xfree(excp.cast()) };
}

/// Discard the exception currently being thrown.
pub(crate) fn discard_current_exception() {
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
unsafe fn set_exception_vars(excp: *mut Exception) {
    // Where `v:throwpoint` is rendered; upstream shares `IObuff`.
    let mut throwpoint = [0 as c_char; IOSIZE as usize];
    // SAFETY: caller contract.
    if excp.is_null() {
        unsafe { set_vim_var_string(Vv::Exception, ptr::null(), -1) };
        unsafe { set_vim_var_string(Vv::Throwpoint, ptr::null(), -1) };
        set_vim_var_list(Vv::Stacktrace, None);
        return;
    }
    unsafe { set_vim_var_string(Vv::Exception, (*excp).value, -1) };
    // SAFETY: the exception's own stack trace; `v:stacktrace` takes a
    // reference of its own.
    unsafe { set_vim_var_list(Vv::Stacktrace, ListRef::retained((*excp).stacktrace)) };
    if unsafe { *(*excp).throw_name } == NUL as c_char {
        // `throw_name` is unset for an exception from a typed command.
        unsafe { set_vim_var_string(Vv::Throwpoint, ptr::null(), -1) };
        return;
    }
    let point = throwpoint.as_mut_ptr();
    let len = if unsafe { (*excp).throw_lnum } == 0 {
        unsafe { vim_snprintf_safelen(point, IOSIZE as usize, c"%s".as_ptr(), (*excp).throw_name) }
    } else {
        unsafe {
            vim_snprintf_safelen(
                point,
                IOSIZE as usize,
                c"%s, line %ld".as_ptr(),
                (*excp).throw_name,
                (*excp).throw_lnum as int64_t,
            )
        }
    };
    unsafe { set_vim_var_string(Vv::Throwpoint, point, len as ptrdiff_t) };
}

/// Push an exception onto the caught stack.
///
/// # Safety
/// Module contract; `excp` is the exception just matched.
pub(super) unsafe fn catch_exception(excp: *mut Exception) {
    // SAFETY: caller contract.
    unsafe { (*excp).caught = caught_stack.get() };
    caught_stack.set(excp);
    unsafe { set_exception_vars(excp) };
    unsafe { verbose_exception(c"Exception caught: %s", (*excp).value) };
}

/// Pop `excp` off the caught stack and free it, restoring `v:exception` and
/// friends to the exception below it.
///
/// # Safety
/// Module contract; `excp` is the top of the caught stack.
pub(super) unsafe fn finish_exception(excp: *mut Exception) {
    // SAFETY: caller contract.
    if excp != caught_stack.get() {
        unsafe { internal_error(c"finish_exception()".as_ptr()) };
    }
    caught_stack.set(unsafe { (*caught_stack.get()).caught });
    unsafe { set_exception_vars(caught_stack.get()) };
    // Discard it, but use the "finished" wording for 'verbose'.
    unsafe { discard_exception(excp, true) };
}

/// Save the exception state, for a nested `do_cmdline` that must not see it.
///
/// # Safety
/// Module contract; `estate` is writable.
pub(crate) unsafe fn exception_state_save(estate: *mut ExceptionState) {
    // SAFETY: caller contract.
    unsafe { (*estate).estate_current_exception = current_exception.get() };
    unsafe { (*estate).estate_did_throw = did_throw.get() };
    unsafe { (*estate).estate_need_rethrow = need_rethrow.get() };
    unsafe { (*estate).estate_trylevel = trylevel.get() };
    unsafe { (*estate).estate_did_emsg = did_emsg.get() };
}

/// Restore what [`exception_state_save`] stored, after handling anything
/// thrown meanwhile.
///
/// # Safety
/// Module contract; `estate` was filled by [`exception_state_save`].
pub(crate) unsafe fn exception_state_restore(estate: *mut ExceptionState) {
    if did_throw.get() {
        handle_did_throw();
    }
    current_exception.set(unsafe { (*estate).estate_current_exception });
    did_throw.set(unsafe { (*estate).estate_did_throw });
    need_rethrow.set(unsafe { (*estate).estate_need_rethrow });
    trylevel.set(unsafe { (*estate).estate_trylevel });
    did_emsg.set(unsafe { (*estate).estate_did_emsg });
}

/// Forget the exception state entirely.
pub(crate) fn exception_state_clear() {
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
    // Where the "Exception made pending" text is built; upstream shares
    // `IObuff`, which the report it feeds writes again.
    let mut pending_msg = [0 as c_char; IOSIZE as usize];
    debug_assert!(
        !value.is_null() || pending & CSTP_THROW == 0,
        "value || !(pending & CSTP_THROW)"
    );
    let mut mesg = action.message().as_ptr().cast_mut();

    // SAFETY: caller contract.
    let s = match pending {
        CSTP_NONE => return,
        CSTP_CONTINUE => c":continue".as_ptr().cast_mut(),
        CSTP_BREAK => c":break".as_ptr().cast_mut(),
        CSTP_FINISH => c":finish".as_ptr().cast_mut(),
        // A ":return" producing a value; the text is allocated.
        CSTP_RETURN => unsafe { get_return_cmd(value) },
        _ if pending & CSTP_THROW != 0 => {
            // "%s made pending" becomes "Exception made pending: %s".
            let out = pending_msg.as_mut_ptr();
            unsafe { vim_snprintf(out, IOSIZE as usize, mesg, c"Exception".as_ptr()) };
            mesg = unsafe { concat_str(out, c": %s".as_ptr()) };
            unsafe { (*value.cast::<Exception>()).value }
        }
        _ if pending & CSTP_ERROR != 0 && pending & CSTP_INTERRUPT != 0 => {
            c"Error and interrupt".as_ptr().cast_mut()
        }
        _ if pending & CSTP_ERROR != 0 => c"Error".as_ptr().cast_mut(),
        // Only CSTP_INTERRUPT is left.
        _ => c"Interrupt".as_ptr().cast_mut(),
    };

    // While debugging the messages have to be displayed.
    let loud = (debug_break_level.get() > 0).then(Allow::messages);
    let no_prompt = Suppress::wait_return();
    // Always scroll up, don't overwrite.
    msg_scroll.set(1);
    // SAFETY: `mesg` is a NUL-terminated format -- a literal, or the one
    // this frame just built -- and `s` the NUL-terminated text it names.
    let (template, text) = unsafe { (gettext_ptr(mesg), c_str(s)) };
    let _: bool = report_msg(0, || tr_plural!(template, text));
    // Don't overwrite this either.
    msg_str(c"\n");
    cmdline_row.set(msg_row.get());
    drop(no_prompt);
    drop(loud);

    if pending == CSTP_RETURN {
        unsafe { xfree(s.cast()) };
    } else if pending & CSTP_THROW != 0 {
        unsafe { xfree(mesg.cast()) };
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
    if quiet {
        verbose_enter();
    }
    unsafe { report_pending(action, pending, value) };
    if quiet {
        verbose_leave();
    }
}

/// Report something a finally clause made pending.
///
/// # Safety
/// As [`report_pending`].
pub(crate) unsafe fn report_make_pending(pending: c_int, value: *mut c_void) {
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
