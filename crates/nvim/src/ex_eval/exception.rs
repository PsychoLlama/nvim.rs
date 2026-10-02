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
use crate::cstr;
use crate::debugger::state::debug_break_level;
use crate::drawscreen::state::cmdline_row;
use crate::eval::typval::{ListRef, list_unref};
use crate::eval::userfunc::get_return_cmd;
use crate::eval::vars::{set_vim_var_list, set_vim_var_string};
use crate::ex_docmd::handle_did_throw;
use crate::ex_eval::state::{
    EXCEPTIONS, caught_stack, current_exception, did_throw, force_abort, msg_lists, need_rethrow,
    suppress_errthrow, trylevel,
};
use crate::getchar::state::got_int;
use crate::guard::{Allow, Suppress};
use crate::memory::XString;
use crate::memory::{xfree, xstrdup};
use crate::message::e_interr;
use crate::message::state::{did_emsg, emsg_silent, msg_row, msg_scroll};
use crate::message::{emsg, internal_error, msg_str, verbose_enter, verbose_leave};
use crate::message_fmt::{c_str, report_msg};
use crate::option::vars::p_verbose;
use crate::option::vars::p_vfile;
use crate::os::cshim::gettext_ptr;
use crate::runtime::{estack_sfile, sourcing_lnum, stacktrace_create};
use crate::strings::concat_str;
use crate::tr_plural;
use crate::types::{
    CondStack, ErrorMsg, ErrorMsgs, ExcId, ExceptType, Exception, ExceptionState, Failed, IOSIZE,
    NUL, Pend, Vv, int64_t, ptrdiff_t,
};
use crate::vim_snprintf;
use crate::vim_snprintf_safelen;
use crate::winlayer::Live;
use core::ffi::{CStr, c_char, c_int};
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
    if ::core::ptr::eq(mesg, message(e_interr).as_ptr()) {
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
        if current_type() == Some(ET_INTERRUPT) {
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
    if msg_lists.with(Vec::is_empty) {
        return true;
    }
    // SAFETY: the caller's NUL-terminated message.
    append_msg(unsafe { cstr::bytes_at(mesg) }, multiline, concat, severe);
    true
}

/// Append `mesg` to the innermost message list, the one the error exception
/// will be built from, or concatenate it onto the last entry. There is one.
fn append_msg(mesg: &[u8], multiline: bool, concat: bool, severe: bool) {
    let concatenates =
        concat && msg_lists.with(|lists| lists.last().is_some_and(|l| !l.entries.is_empty()));
    if concatenates {
        msg_lists.with_mut(|lists| {
            let list = lists.last_mut().expect("a message list");
            let last = list.entries.len() - 1;
            list.entries[last].msg.push_bytes(mesg);
            // Upstream points the last entry's `throw_msg` at its joined
            // text; only the head's is ever read.
            if last == 0 {
                list.throw_at = (0, 0);
            }
        });
        return;
    }

    // Take the source name and line number now: they may change before
    // `do_errthrow` runs.
    let sfile = estack_sfile(ESTACK_NONE);
    // SAFETY: `estack_sfile` answers an `xmalloc`'d string or null.
    let sfile = (!sfile.is_null()).then(|| unsafe { XString::from_raw(sfile) });
    let entry = ErrorMsg {
        msg: XString::from_bytes(mesg),
        sfile,
        slnum: sourcing_lnum(),
        multiline,
    };
    msg_lists.with_mut(|lists| {
        let list = lists.last_mut().expect("a message list");
        list.entries.push(entry);
        let at = list.entries.len() - 1;
        if at == 0 || severe {
            // Skip the extra "Vim " prefix, as on message "E458".
            let msg = &list.entries[at].msg[..];
            let vim_prefixed = msg.starts_with(b"Vim E")
                && msg
                    .get(5..8)
                    .is_some_and(|d| d.iter().all(u8::is_ascii_digit))
                && msg.get(8..10) == Some(b": ");
            list.throw_at = (at, if vim_prefixed { 4 } else { 0 });
        }
    });
}

/// Take the innermost message list's messages, leaving it empty.
pub(crate) fn take_msg_list() -> ErrorMsgs {
    msg_lists
        .with_mut(|lists| lists.last_mut().map(core::mem::take))
        .unwrap_or_default()
}

/// Drop the innermost message list's messages.
pub(crate) fn free_global_msglist() {
    drop(take_msg_list());
}

/// Start an empty message list for a command line or an API call to collect
/// its errors in, until [`pop_msg_list`]: upstream pointing `msg_list` at
/// its own frame's list head.
pub(crate) fn push_msg_list() {
    msg_lists.with_mut(|lists| lists.push(ErrorMsgs::default()));
}

/// End the innermost message list, answering what is left in it.
pub(crate) fn pop_msg_list() -> ErrorMsgs {
    msg_lists.with_mut(Vec::pop).expect("a message list to end")
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

    // Nothing to throw, or the conversion belongs to an outer
    // `do_one_cmd`.
    let messages = msg_lists.with_mut(|lists| {
        lists
            .last_mut()
            .filter(|list| !list.entries.is_empty())
            .map(core::mem::take)
    });
    let Some(messages) = messages else {
        return;
    };
    // SAFETY: module contract; `cmdname` is null or NUL-terminated.
    if unsafe { throw_exception(Thrown::Error(messages), cmdname) }.is_err() {
        // The messages went with the failed throw.
    } else if !cstack.is_null() {
        unsafe { super::trycmd::do_throw(cstack) };
    } else {
        need_rethrow.set(true);
    }
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
        if current_type() == Some(ET_INTERRUPT) {
            return false;
        }
        // Otherwise it replaces the user or error exception.
        discard_current_exception();
    }
    if unsafe { throw_exception(Thrown::Interrupt, ptr::null_mut()) }.is_ok() {
        unsafe { super::trycmd::do_throw(cstack) };
    }
    true
}

/// The string an error exception is matched and reported by: the messages
/// it was built from, prefixed with `Vim:` or `Vim(cmdname):`.
pub(crate) fn error_exception_string(messages: &ErrorMsgs, cmdname: &[u8]) -> XString {
    let mut ret = XString::from_bytes(b"Vim");
    if cmdname.is_empty() {
        ret.push_byte(b':');
    } else {
        ret.push_byte(b'(');
        ret.push_bytes(cmdname);
        ret.push_bytes(b"):");
    }
    ret.push_bytes(&exception_message(messages.throw_msg()));
    ret
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

/// What is being thrown.
pub(crate) enum Thrown {
    /// A `:throw`, with its value, which the exception takes over.
    User(*mut c_char),
    /// An error, with the messages that make its value.
    Error(ErrorMsgs),
    /// CTRL-C.
    Interrupt,
}

/// The exception `self` names, as a view: its fields are ordinary reads
/// and writes. It is valid until the exception is discarded.
impl ExcId {
    pub(crate) fn exception(self) -> Live<Exception> {
        // SAFETY: the table owns the exception at a fixed address until it
        // is discarded, which is the view's contract.
        unsafe { Live::new(EXCEPTIONS.with(|table| table.address(self))) }
    }
}

/// The kind of exception being thrown, if one is.
fn current_type() -> Option<ExceptType> {
    current_exception.get().map(|id| id.exception().type_0)
}

/// Build the exception and make it the one being thrown.
///
/// Answers `Err` when a user exception tried to fake a `Vim` one, having
/// freed its value.
///
/// # Safety
/// Module contract; a user value is an owned NUL-terminated string and
/// `cmdname` is null or NUL-terminated.
pub(super) unsafe fn throw_exception(thrown: Thrown, cmdname: *mut c_char) -> Result<(), Failed> {
    let (type_0, value, messages) = match thrown {
        Thrown::User(value) => {
            // Faking an interrupt or error exception as a user one is not
            // allowed: `do_cmdline` treats the two differently when no
            // active try block is found.
            // SAFETY: the caller's NUL-terminated value.
            let text = unsafe { cstr::bytes_at(value) };
            if text.starts_with(b"Vim") && matches!(text.get(3), None | Some(b':' | b'(')) {
                emsg(c"E608: Cannot :throw exceptions with 'Vim' prefix");
                current_exception.set(None);
                unsafe { xfree(value.cast()) };
                return Err(Failed);
            }
            (ET_USER, value, ErrorMsgs::default())
        }
        Thrown::Error(messages) => {
            // SAFETY: the caller's NUL-terminated command name, or null.
            let cmdname = unsafe { cstr::bytes_at_or_empty(cmdname) };
            let value = error_exception_string(&messages, cmdname).into_raw();
            (ET_ERROR, value, messages)
        }
        // A literal: never freed, see `discard_exception`.
        Thrown::Interrupt => (
            ET_INTERRUPT,
            c"Vim:Interrupt".as_ptr().cast_mut(),
            ErrorMsgs::default(),
        ),
    };

    // An error exception throws from where the message was made, which is
    // not where we are now.
    let mut messages = messages;
    let head = messages.entries.first_mut();
    let (throw_name, throw_lnum) = match head {
        Some(head) if head.sfile.is_some() => {
            let name = head.sfile.take().expect("checked");
            (name.into_raw(), head.slnum)
        }
        _ => {
            let name = estack_sfile(ESTACK_NONE);
            let name = if name.is_null() {
                // SAFETY: a literal.
                unsafe { xstrdup(c"".as_ptr()) }
            } else {
                name
            };
            (name, sourcing_lnum())
        }
    };
    let exception = Exception {
        type_0,
        value,
        messages,
        throw_name,
        throw_lnum,
        // The exception owns the stack trace it was thrown with.
        stacktrace: stacktrace_create().map_or(ptr::null_mut(), ListRef::into_raw),
    };
    let (id, _) = EXCEPTIONS.with_mut(|table| table.insert((), exception));

    unsafe { verbose_exception(c"Exception thrown: %s", id.exception().value) };

    current_exception.set(Some(id));
    Ok(())
}

/// Report an exception's fate under 'verbose' >= 13 or while debugging.
///
/// # Safety
/// `mesg` holds one `%s` and `value` is NUL-terminated.
unsafe fn verbose_exception(mesg: &CStr, value: *mut c_char) {
    if p_verbose() < 13 && debug_break_level.get() <= 0 {
        return;
    }
    let debugging = debug_break_level.get() > 0;
    // While debugging the messages have to be displayed.
    let loud = debugging.then(Allow::messages);
    if !debugging {
        verbose_enter();
    }
    let no_prompt = Suppress::wait_return();
    if debug_break_level.get() > 0 || p_vfile(CStr::is_empty) {
        // Always scroll up, don't overwrite.
        msg_scroll.set(1);
    }
    // SAFETY: the caller's NUL-terminated value, and `mesg`, which outlives
    // the message it is translated for.
    let (template, value) = unsafe { (gettext_ptr(mesg.as_ptr()), c_str(value)) };
    let _: bool = report_msg(0, || tr_plural!(template, value));
    // Don't overwrite this either.
    msg_str(c"\n");
    if debug_break_level.get() > 0 || p_vfile(CStr::is_empty) {
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
/// The caller has taken it off the caught stack, if it was there.
pub(super) fn discard_exception(id: ExcId, was_finished: bool) {
    if current_exception.get() == Some(id) {
        current_exception.set(None);
    }

    if p_verbose() >= 13 || debug_break_level.get() > 0 {
        // Upstream saves and restores `IObuff` around this, because the
        // report formatted through it and a caller may have been holding
        // a message there. Nothing shares a buffer here any more.
        let wording = if was_finished {
            c"Exception finished: %s"
        } else {
            c"Exception discarded: %s"
        };
        // SAFETY: the exception's own NUL-terminated value.
        unsafe { verbose_exception(wording, id.exception().value) };
    }
    let exception = EXCEPTIONS.with_mut(|table| table.remove(id));
    if exception.type_0 != ET_INTERRUPT {
        // An interrupt exception's value is a string literal.
        // SAFETY: the exception's own allocation.
        unsafe { xfree(exception.value.cast()) };
    }
    // SAFETY: the exception's own allocation and stack trace reference.
    unsafe { xfree(exception.throw_name.cast()) };
    unsafe { list_unref(exception.stacktrace) };
    // The messages go with the rest of it.
}

/// Discard the exception currently being thrown.
pub(crate) fn discard_current_exception() {
    if let Some(id) = current_exception.get() {
        discard_exception(id, false);
    }
    // Everything reset here is saved and restored by
    // `exception_state_save`/`_restore`.
    did_throw.set(false);
    need_rethrow.set(false);
}

/// Point `v:exception`, `v:throwpoint` and `v:stacktrace` at `excp`, or
/// clear all three when there is none.
fn set_exception_vars(excp: Option<ExcId>) {
    // Where `v:throwpoint` is rendered; upstream shares `IObuff`.
    let mut throwpoint = [0 as c_char; IOSIZE as usize];
    let Some(id) = excp else {
        // SAFETY: a null value clears the variable.
        unsafe { set_vim_var_string(Vv::Exception, ptr::null(), -1) };
        unsafe { set_vim_var_string(Vv::Throwpoint, ptr::null(), -1) };
        set_vim_var_list(Vv::Stacktrace, None);
        return;
    };
    let excp = id.exception();
    // SAFETY: the exception's own NUL-terminated value; the variable takes a
    // copy.
    unsafe { set_vim_var_string(Vv::Exception, excp.value, -1) };
    // SAFETY: the exception's own stack trace; `v:stacktrace` takes a
    // reference of its own.
    unsafe { set_vim_var_list(Vv::Stacktrace, ListRef::retained(excp.stacktrace)) };
    // SAFETY: the exception's own NUL-terminated name.
    if unsafe { *excp.throw_name } == NUL as c_char {
        // `throw_name` is unset for an exception from a typed command.
        unsafe { set_vim_var_string(Vv::Throwpoint, ptr::null(), -1) };
        return;
    }
    let point = throwpoint.as_mut_ptr();
    let len = if excp.throw_lnum == 0 {
        unsafe { vim_snprintf_safelen!(point, IOSIZE as usize, c"%s".as_ptr(), excp.throw_name) }
    } else {
        unsafe {
            vim_snprintf_safelen!(
                point,
                IOSIZE as usize,
                c"%s, line %ld".as_ptr(),
                excp.throw_name,
                excp.throw_lnum as int64_t,
            )
        }
    };
    unsafe { set_vim_var_string(Vv::Throwpoint, point, len as ptrdiff_t) };
}

/// Push an exception onto the caught stack.
pub(super) fn catch_exception(id: ExcId) {
    caught_stack.with_mut(|stack| stack.push(id));
    set_exception_vars(Some(id));
    // SAFETY: the exception's own NUL-terminated value.
    unsafe { verbose_exception(c"Exception caught: %s", id.exception().value) };
}

/// Pop `excp` off the caught stack and free it, restoring `v:exception` and
/// friends to the exception below it.
pub(super) fn finish_exception(excp: Option<ExcId>) {
    let top = caught_stack.with_mut(Vec::pop);
    if excp != top {
        internal_error(c"finish_exception()");
    }
    set_exception_vars(caught_stack.with(|stack| stack.last().copied()));
    // Discard it, but use the "finished" wording for 'verbose'.
    match excp {
        Some(id) => discard_exception(id, true),
        None => internal_error(c"discard_exception()"),
    }
}

/// Save the exception state, for a nested `do_cmdline` that must not see it.
pub(crate) fn exception_state_save() -> ExceptionState {
    ExceptionState {
        estate_current_exception: current_exception.get(),
        estate_did_throw: did_throw.get(),
        estate_need_rethrow: need_rethrow.get(),
        estate_trylevel: trylevel.get(),
        estate_did_emsg: did_emsg.get(),
    }
}

/// Restore what [`exception_state_save`] answered, after handling anything
/// thrown meanwhile.
pub(crate) fn exception_state_restore(estate: &ExceptionState) {
    if did_throw.get() {
        handle_did_throw();
    }
    current_exception.set(estate.estate_current_exception);
    did_throw.set(estate.estate_did_throw);
    need_rethrow.set(estate.estate_need_rethrow);
    trylevel.set(estate.estate_trylevel);
    did_emsg.set(estate.estate_did_emsg);
}

/// Forget the exception state entirely.
pub(crate) fn exception_state_clear() {
    current_exception.set(None);
    did_throw.set(false);
    need_rethrow.set(false);
    trylevel.set(0);
    did_emsg.set(0);
}

/// What [`report_pending`] is saying about the pending thing.
#[derive(Clone, Copy)]
pub(crate) enum PendingAction {
    /// A finally clause made it pending.
    Made,
    /// It is being resumed at the `:endtry`.
    Resumed,
    /// It is being thrown away.
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

/// Report what a finally clause made pending, resumed or discarded, under
/// 'verbose' >= 14 or while debugging. `value` is the return value for a
/// pending `:return` and the exception for a pending throw.
///
/// # Safety
/// Module contract; `value` matches `pending`: a `Pend::Return` value is null
/// or a live typval, and `value` is an exception whenever `pending` carries
/// [`CSTP_THROW`].
pub(crate) unsafe fn report_pending(action: PendingAction, pending: c_int, value: Pend) {
    if p_verbose() < 14 && debug_break_level.get() <= 0 {
        return;
    }
    // Where the "Exception made pending" text is built; upstream shares
    // `IObuff`, which the report it feeds writes again.
    let mut pending_msg = [0 as c_char; IOSIZE as usize];
    debug_assert!(
        matches!(value, Pend::Exception(_)) || pending & CSTP_THROW == 0,
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
        CSTP_RETURN => unsafe {
            get_return_cmd(match value {
                Pend::Return(rettv) => rettv,
                _ => ptr::null_mut(),
            })
        },
        _ if pending & CSTP_THROW != 0 => {
            // "%s made pending" becomes "Exception made pending: %s".
            let out = pending_msg.as_mut_ptr();
            unsafe { vim_snprintf!(out, IOSIZE as usize, mesg, c"Exception".as_ptr()) };
            mesg = unsafe { concat_str(out, c": %s".as_ptr()) };
            match value {
                Pend::Exception(id) => id.exception().value,
                _ => unreachable!("a pending throw reports its exception"),
            }
        }
        _ if pending & CSTP_ERROR != 0 && pending & CSTP_INTERRUPT != 0 => {
            c"Error and interrupt".as_ptr().cast_mut()
        }
        _ if pending & CSTP_ERROR != 0 => c"Error".as_ptr().cast_mut(),
        // Only CSTP_INTERRUPT is left.
        _ => c"Interrupt".as_ptr().cast_mut(),
    };

    let quiet = debug_break_level.get() <= 0;
    if quiet {
        verbose_enter();
    }
    // While debugging the messages have to be displayed.
    let loud = (!quiet).then(Allow::messages);
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
    if quiet {
        verbose_leave();
    }

    if pending == CSTP_RETURN {
        unsafe { xfree(s.cast()) };
    } else if pending & CSTP_THROW != 0 {
        unsafe { xfree(mesg.cast()) };
    }
}
