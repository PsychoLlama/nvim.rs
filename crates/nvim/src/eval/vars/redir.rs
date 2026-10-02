//! `:redir => var` -- capturing messages into a variable.
//!
//! [`var_redir_start`] resolves the target once and seeds it,
//! [`var_redir_str`] appends every message to a growable buffer, and
//! [`var_redir_stop`] stores the result.  [`assert_error`] is the same trick
//! for `v:errors`.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::cstr;
use crate::memory::XString;
use crate::message_fmt::c_str;
use crate::semsg;
use core::ffi::CStr;
use core::ffi::{c_char, c_int};
use core::mem::ManuallyDrop;
use core::{ptr, slice};

use super::*;
use crate::types::{Failed, NUL};

/// Append `message` to `v:errors`, which the `assert_*` builtins report
/// through.
pub fn assert_error(message: &[u8]) {
    // SAFETY: `v:errors` is a table row, and `message` is the caller's own
    // buffer, read for exactly its own length.
    let tv = unsafe { Tv::new(get_vim_var_tv(Vv::Errors)) };
    if tv.v_type() != VAR_LIST || tv.list_or_null().is_null() {
        // Something replaced it; make sure `v:errors` is a List again.
        set_vim_var_list(Vv::Errors, Some(tv_list_alloc(1)));
    }
    // A message that was never appended to used to be a null `ga_data`, and
    // `List::push_string` tells that apart from a zero-length buffer: the
    // first appends `v:null`, the second an empty string.
    let text = if message.is_empty() {
        ptr::null()
    } else {
        message.as_ptr().cast::<c_char>()
    };
    let len = message.len() as ssize_t;
    unsafe { (*get_vim_var_list(Vv::Errors)).push_string(text, len) };
}

/// The name of the variable a running `:redir =>` captures into, `None`
/// while none runs. The name, not the resolved lvalue, is what is kept: a
/// Dict or List entry may move before the end, so [`var_redir_stop`] parses
/// it again.
static REDIR_TARGET: GlobalCell<Option<XString>> = GlobalCell::new(None);
/// The text collected so far, without a terminator: the NUL goes on once, in
/// [`var_redir_stop`], when the buffer has stopped growing.
static redir_ga: GlobalCell<Vec<u8>> = GlobalCell::new(Vec::new());

/// Parse `target` into `lval`, answering where the name ended in `name` — a
/// copy of `target` the caller keeps alive while it uses the answer.
///
/// The copy is per call because `get_lval` writes into the name and can run
/// user code (an index expression), which may start or stop a redirection.
fn resolve_redir_lval(name: &mut XString, lval: &mut LVal) -> *mut c_char {
    // SAFETY: `name` is an owned, writable, NUL-terminated copy and `lval` a
    // whole record; the result points into `name`.
    unsafe {
        get_lval(
            name.as_mut_ptr(),
            None,
            lval,
            false,
            false,
            0,
            FNE_CHECK_START,
        )
    }
}

/// Start capturing messages into the variable `name`, appending to it rather
/// than replacing it when `append`.
///
/// # Safety
/// `name` is a NUL-terminated string.
pub unsafe fn var_redir_start(name: *const c_char, append: bool) -> Result<(), Failed> {
    // SAFETY: the caller's obligation.
    let name = unsafe { CStr::from_ptr(name) };
    // Catch a bad name early.
    if !eval_isnamec1(c_int::from(name.to_bytes().first().copied().unwrap_or(0))) {
        emsg_static(e_invarg);
        return Err(Failed);
    }

    // The name is parsed again in `var_redir_stop`, so it is kept for as
    // long as the redirection runs.
    REDIR_TARGET.set(Some(XString::from_cstr(name)));
    // The output is collected here until redirection ends.
    redir_ga.with_mut(|text| {
        text.clear();
        text.reserve(500);
    });

    // Parse the name, which may be a Dict or List entry.
    let mut target = XString::from_cstr(name);
    let mut lval = LVAL_INITIAL_VALUE;
    let endp = resolve_redir_lval(&mut target, &mut lval);
    // SAFETY: a non-null answer points into `target`, which is terminated.
    let trailing = (!endp.is_null()).then(|| unsafe { *endp });
    if trailing.is_none_or(|c| c != NUL as c_char) || lval.ll_name.is_null() {
        // SAFETY: the record `get_lval` filled.
        unsafe { clear_lval(&mut lval) };
        if trailing.is_some_and(|c| c != NUL as c_char) {
            // SAFETY: `endp` points into `target`, which is terminated.
            let endp = unsafe { c_str(endp) };
            semsg!("E488: Trailing characters: {endp}");
        } else {
            let name = name.to_string_lossy();
            semsg!("E475: Invalid argument: {name}");
        }
        abandon_redir();
        return Err(Failed);
    }

    // Check the variable can be written, by setting it to -- or
    // appending to it -- an empty string.
    let called_emsg_before = called_emsg.get();
    did_emsg.set(0);
    // A literal, so the value must not release it.
    let mut tv = ManuallyDrop::new(TypVal::String(c"".as_ptr() as *mut c_char));
    let op = if append { c"." } else { c"=" };
    // SAFETY: the lvalue just resolved, whose name end points into
    // `target`, and a live local value.
    unsafe { set_var_lval(&mut lval, endp, &mut tv, true, false, op.as_ptr()) };
    // SAFETY: the record `get_lval` filled.
    unsafe { clear_lval(&mut lval) };
    if called_emsg.get() > called_emsg_before {
        abandon_redir();
        return Err(Failed);
    }
    Ok(())
}

/// End a redirection whose start failed: store nothing.
fn abandon_redir() {
    REDIR_TARGET.set(None);
    redir_ga.take();
}

/// Append `value[0..value_len]` to what `:redir =>` is capturing, or the
/// whole NUL-terminated string when `value_len` is -1.
///
/// The store is postponed to [`var_redir_stop`] on purpose: what is being
/// appended may *be* the string being written to, and changing it would then
/// use freed memory --
///
/// ```text
///     :redir => foo
///     :let foo
///     :redir END
/// ```
///
/// # Safety
/// `value` is readable for `value_len` bytes, or NUL-terminated.
pub unsafe fn var_redir_str(value: *const c_char, value_len: c_int) {
    if REDIR_TARGET.with(Option::is_none) {
        return;
    }
    // SAFETY: the caller's `value` is readable for `value_len` bytes, or is
    // NUL-terminated when the length is -1.
    let len = match value_len == -1 {
        true => unsafe { cstr::bytes_at(value) }.len(),
        false => value_len as size_t,
    };
    let bytes = unsafe { slice::from_raw_parts(value.cast::<u8>(), len) };
    redir_ga.with_mut(|text| text.extend_from_slice(bytes));
}

/// Stop capturing and store what was collected.
pub fn var_redir_stop() {
    let Some(mut target) = REDIR_TARGET.with(|target| target.as_deref().map(XString::from_bytes))
    else {
        return;
    };
    // Collecting is over: take the buffer, so that a message emitted from
    // inside `set_var_lval` appends to a fresh one instead of reallocating
    // under the `typval` that borrows this one. The redirection stays on
    // until the store is done, as upstream's does, and what it captures
    // meanwhile is dropped below.
    let mut text = redir_ga.take();
    text.push(NUL as u8);
    // The accumulated bytes stay this frame's; the store copies or appends
    // them, so the value releases nothing.
    let mut tv = ManuallyDrop::new(TypVal::String(text.as_mut_ptr().cast::<c_char>()));
    // Resolve the name again: inside a Dict or List it may have moved since.
    let mut lval = LVAL_INITIAL_VALUE;
    let endp = resolve_redir_lval(&mut target, &mut lval);
    if !endp.is_null() && !lval.ll_name.is_null() {
        // SAFETY: the lvalue just resolved, whose name end points into
        // `target`, and a live local value.
        unsafe { set_var_lval(&mut lval, endp, &mut tv, false, false, c".".as_ptr()) };
    }
    // SAFETY: the record `get_lval` filled.
    unsafe { clear_lval(&mut lval) };
    abandon_redir();
}
