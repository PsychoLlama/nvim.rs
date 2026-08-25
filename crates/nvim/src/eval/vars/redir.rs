//! `:redir => var` -- capturing messages into a variable.
//!
//! [`var_redir_start`] resolves the target once and seeds it,
//! [`var_redir_str`] appends every message to a growable buffer, and
//! [`var_redir_stop`] stores the result.  [`assert_error`] is the same trick
//! for `v:errors`.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{c_char, c_int};
use core::{ptr, slice};

use super::*;
use crate::types::{FAIL, NUL, OK};

/// Append the message in `gap` to `v:errors`, which the `assert_*` builtins
/// report through.
///
/// # Safety
/// `gap` is a byte garray holding the message.
pub unsafe fn assert_error(gap: *mut garray_T) {
    unsafe {
        let tv = get_vim_var_tv(Vv::Errors);
        if (*tv).v_type != VAR_LIST || (*tv).vval.v_list.is_null() {
            // Something replaced it; make sure `v:errors` is a List again.
            set_vim_var_list(Vv::Errors, tv_list_alloc(1));
        }
        tv_list_append_string(
            get_vim_var_list(Vv::Errors),
            (*gap).ga_data as *const c_char,
            (*gap).ga_len as ssize_t,
        );
    }
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

/// Start capturing messages into the variable `name`, appending to it rather
/// than replacing it when `append`.
///
/// # Safety
/// `name` is a NUL-terminated string.
pub unsafe fn var_redir_start(name: *mut c_char, append: bool) -> c_int {
    unsafe {
        // Catch a bad name early.
        if !eval_isnamec1(*name as c_int) {
            emsg(gettext(&raw const e_invarg as *const c_char));
            return FAIL;
        }

        // The name is used again in `var_redir_stop`, so it is copied for as
        // long as the redirection runs.
        redir_varname.set(xstrdup(name));
        redir_lval.set(xcalloc(1, ::core::mem::size_of::<lval_T>()) as *mut lval_T);
        // The output is collected here until redirection ends.
        redir_ga.with_mut(|text| {
            text.clear();
            text.reserve(500);
        });

        // Parse the name, which may be a Dict or List entry.
        redir_endp.set(get_lval(
            redir_varname.get(),
            ptr::null_mut(),
            redir_lval.get(),
            false,
            false,
            0,
            FNE_CHECK_START,
        ));
        let endp = redir_endp.get();
        if endp.is_null() || (*redir_lval.get()).ll_name.is_null() || *endp != NUL as c_char {
            clear_lval(redir_lval.get());
            if !endp.is_null() && *endp != NUL as c_char {
                semsg_c!(gettext(&raw const e_trailing_arg as *const c_char), endp);
            } else {
                semsg_c!(gettext(&raw const e_invarg2 as *const c_char), name);
            }
            // Store no value; only clean up.
            redir_endp.set(ptr::null_mut());
            var_redir_stop();
            return FAIL;
        }

        // Check the variable can be written, by setting it to -- or
        // appending to it -- an empty string.
        let called_emsg_before = called_emsg.get();
        did_emsg.set(0);
        let mut tv = typval_T {
            v_type: VAR_STRING,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union {
                v_string: c"".as_ptr() as *mut c_char,
            },
        };
        let op = if append { c"." } else { c"=" };
        set_var_lval(
            redir_lval.get(),
            redir_endp.get(),
            &raw mut tv,
            true,
            false,
            op.as_ptr(),
        );
        clear_lval(redir_lval.get());
        if called_emsg.get() > called_emsg_before {
            redir_endp.set(ptr::null_mut());
            var_redir_stop();
            return FAIL;
        }
        OK
    }
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
    let bytes = unsafe {
        let len = if value_len == -1 {
            strlen(value)
        } else {
            value_len as size_t
        };
        slice::from_raw_parts(value.cast::<u8>(), len)
    };
    redir_ga.with_mut(|text| text.extend_from_slice(bytes));
}

/// Stop capturing and store what was collected.
///
/// # Safety
/// Nothing; a call with no redirection running only frees.
pub unsafe fn var_redir_stop() {
    unsafe {
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
                    v_lock: VAR_UNLOCKED,
                    vval: typval_vval_union {
                        v_string: text.as_mut_ptr().cast::<c_char>(),
                    },
                };
                // Resolve the name again: inside a Dict or List it may have
                // moved since.
                redir_endp.set(get_lval(
                    redir_varname.get(),
                    ptr::null_mut(),
                    redir_lval.get(),
                    false,
                    false,
                    0,
                    FNE_CHECK_START,
                ));
                if !redir_endp.get().is_null() && !(*redir_lval.get()).ll_name.is_null() {
                    set_var_lval(
                        redir_lval.get(),
                        redir_endp.get(),
                        &raw mut tv,
                        false,
                        false,
                        c".".as_ptr(),
                    );
                }
                clear_lval(redir_lval.get());
            }

            xfree(redir_lval.get().cast());
            redir_lval.set(ptr::null_mut());
        }
        xfree(redir_varname.get().cast());
        redir_varname.set(ptr::null_mut());
    }
}
