//! `:redir => var` -- capturing messages into a variable.
//!
//! [`var_redir_start`] resolves the target once and seeds it,
//! [`var_redir_str`] appends every message to a growable buffer, and
//! [`var_redir_stop`] stores the result.  [`assert_error`] is the same trick
//! for `v:errors`.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::cstr;
use crate::message_fmt::c_str;
use crate::semsg;
use core::ffi::{c_char, c_int};
use core::{ptr, slice};

use super::*;
use crate::types::{Failed, NUL};

/// Append the message in `gap` to `v:errors`, which the `assert_*` builtins
/// report through.
///
/// # Safety
/// `gap` is a byte garray holding the message.
pub unsafe fn assert_error(gap: *mut garray_T) {
    // SAFETY: `v:errors` is a table row, and `gap` is the caller's live
    // byte garray.
    let tv = unsafe { Tv::new(get_vim_var_tv(Vv::Errors)) };
    if tv.v_type != VAR_LIST || tv.list_or_null().is_null() {
        // Something replaced it; make sure `v:errors` is a List again.
        unsafe { set_vim_var_list(Vv::Errors, tv_list_alloc(1)) };
    }
    let text = unsafe { (*gap).ga_data } as *const c_char;
    let len = unsafe { (*gap).ga_len } as ssize_t;
    unsafe { tv_list_append_string(get_vim_var_list(Vv::Errors), text, len) };
}

/// The lvalue `:redir =>` is capturing into, its name (kept because the
/// lvalue is re-resolved at the end), where its name ended, and the text
/// collected so far.
///
/// A NULL `redir_lval` means no redirection is running; a NULL `redir_endp`
/// means one is, but failed, so the teardown should only free.
static redir_lval: GlobalCell<*mut lval_T> = GlobalCell::new(ptr::null_mut());
/// The text collected so far, without a terminator: the NUL goes on once, in
/// [`var_redir_stop`], when the buffer has stopped growing.
static redir_ga: GlobalCell<Vec<u8>> = GlobalCell::new(Vec::new());
static redir_endp: GlobalCell<*mut c_char> = GlobalCell::new(ptr::null_mut());
static redir_varname: GlobalCell<*mut c_char> = GlobalCell::new(ptr::null_mut());

/// Resolve the saved `:redir =>` name into the saved lvalue, answering where
/// the name ended.
///
/// Both halves of the redirection parse the same name into the same lvalue,
/// and `var_redir_stop` has to do it again because a Dict or List entry may
/// have moved since the start.
///
/// # Safety
/// `redir_varname` and `redir_lval` are the ones `var_redir_start` set.
unsafe fn resolve_redir_lval() -> *mut c_char {
    let (name, lv) = (redir_varname.get(), redir_lval.get());
    // SAFETY: the caller's obligation.
    unsafe { get_lval(name, ptr::null_mut(), lv, false, false, 0, FNE_CHECK_START) }
}

/// Start capturing messages into the variable `name`, appending to it rather
/// than replacing it when `append`.
///
/// # Safety
/// `name` is a NUL-terminated string.
pub unsafe fn var_redir_start(name: *mut c_char, append: bool) -> Result<(), Failed> {
    // Catch a bad name early.
    if !eval_isnamec1(unsafe { *name } as c_int) {
        emsg_static(e_invarg);
        return Err(Failed);
    }

    // The name is used again in `var_redir_stop`, so it is copied for as
    // long as the redirection runs.
    redir_varname.set(unsafe { xstrdup(name) });
    redir_lval.set(unsafe { xcalloc(1, ::core::mem::size_of::<lval_T>()) } as *mut lval_T);
    // The output is collected here until redirection ends.
    redir_ga.with_mut(|text| {
        text.clear();
        text.reserve(500);
    });

    // Parse the name, which may be a Dict or List entry.
    // SAFETY: the copied name is NUL-terminated and the lvalue is the
    // zeroed one just allocated, which lives until `var_redir_stop`.
    redir_endp.set(unsafe { resolve_redir_lval() });
    let endp = redir_endp.get();
    let trailing = (!endp.is_null()).then(|| unsafe { *endp });
    if trailing.is_none_or(|c| c != NUL as c_char)
        || unsafe { (*redir_lval.get()).ll_name }.is_null()
    {
        unsafe { clear_lval(redir_lval.get()) };
        if trailing.is_some_and(|c| c != NUL as c_char) {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let endp = unsafe { c_str(endp) };
            semsg!("E488: Trailing characters: {endp}");
        } else {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let name = unsafe { c_str(name) };
            semsg!("E475: Invalid argument: {name}");
        }
        // Store no value; only clean up.
        redir_endp.set(ptr::null_mut());
        unsafe { var_redir_stop() };
        return Err(Failed);
    }

    // Check the variable can be written, by setting it to -- or
    // appending to it -- an empty string.
    let called_emsg_before = called_emsg.get();
    did_emsg.set(0);
    let mut tv = typval_T {
        v_type: VAR_STRING,
        v_lock: VarLock::Unlocked,
        vval: typval_vval_union {
            v_string: c"".as_ptr() as *mut c_char,
        },
    };
    let op = if append { c"." } else { c"=" };
    let (lv, endp, tvp) = (redir_lval.get(), redir_endp.get(), &raw mut tv);
    // SAFETY: the lvalue just resolved, and a live local value.
    unsafe { set_var_lval(lv, endp, tvp, true, false, op.as_ptr()) };
    unsafe { clear_lval(redir_lval.get()) };
    if called_emsg.get() > called_emsg_before {
        redir_endp.set(ptr::null_mut());
        unsafe { var_redir_stop() };
        return Err(Failed);
    }
    Ok(())
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
    if redir_lval.get().is_null() {
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
///
/// # Safety
/// Nothing; a call with no redirection running only frees.
pub unsafe fn var_redir_stop() {
    if !redir_lval.get().is_null() {
        // Collecting is over: take the buffer, so that a message emitted
        // from inside `set_var_lval` appends to a fresh one instead of
        // reallocating under the `typval` that borrows this one.
        let mut text = redir_ga.take();
        // Store the text, unless the start failed.
        if !redir_endp.get().is_null() {
            text.push(NUL as u8);
            let mut tv = typval_T {
                v_type: VAR_STRING,
                v_lock: VarLock::Unlocked,
                vval: typval_vval_union {
                    v_string: text.as_mut_ptr().cast::<c_char>(),
                },
            };
            // Resolve the name again: inside a Dict or List it may have
            // moved since.
            // SAFETY: as [`var_redir_start`] -- the saved name and lvalue.
            redir_endp.set(unsafe { resolve_redir_lval() });
            let (lv, endp) = (redir_lval.get(), redir_endp.get());
            if !endp.is_null() && !unsafe { (*lv).ll_name }.is_null() {
                let tvp = &raw mut tv;
                unsafe { set_var_lval(lv, endp, tvp, false, false, c".".as_ptr()) };
            }
            unsafe { clear_lval(redir_lval.get()) };
        }

        unsafe { xfree(redir_lval.get().cast()) };
        redir_lval.set(ptr::null_mut());
    }
    unsafe { xfree(redir_varname.get().cast()) };
    redir_varname.set(ptr::null_mut());
}
