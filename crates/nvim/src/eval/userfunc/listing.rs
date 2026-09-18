//! Printing functions back, and `:delfunction`.
//!
//! `list_functions` walks the whole table, `list_functions_matching_pat`
//! the subset a `/pattern/` matches, and `list_one_function` prints one
//! with its numbered body lines.  `ex_delfunction` is here because it is
//! the same argument parse in reverse; `function_exists` and
//! `get_user_func_name` answer `exists('*x')` and completion.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::cstr;
use crate::message_fmt::c_str;
use crate::message_fmt::msg_bytes;
use crate::semsg;
use core::ffi::{c_char, c_int, c_void};
use core::mem::offset_of;
use core::ptr;

use super::*;
use crate::types::{ExpandContext, IOSIZE, NUL};

/// Print the head of every function, or of the ones `regmatch` matches.
///
/// # Safety
/// `regmatch` is null or a compiled pattern.
pub(crate) unsafe fn list_functions(regmatch: *mut RegMatch) {
    let prev_ht_changed = func_table().changed();
    let mut todo = func_table().used();
    let mut idx = 0;

    unsafe { msg_ext_set_kind(c"list_cmd".as_ptr()) };
    while todo > 0 && !got_int.get() {
        let hi = func_table().slot(idx);
        if hi.is_kept() {
            // The key *is* the function's trailing name member, so the
            // function is that many bytes before it.
            let fp = unsafe { hi.hi_key.sub(offset_of!(UserFunc, uf_name)) } as *mut UserFunc;
            todo -= 1;
            // Without a pattern, skip what the user filtered out and the
            // numbered/lambda functions; with one, skip the numbered
            // functions and ask the pattern.
            let show = if regmatch.is_null() {
                !message_filtered(unsafe { cstr::at(uf_name_ptr(fp)) })
                    && !unsafe { func_name_refcount(uf_name_ptr(fp)) }
            } else {
                !(unsafe { *uf_name_ptr(fp) } as u8).is_ascii_digit()
                    && unsafe { vim_regexec(regmatch, uf_name_ptr(fp), 0) }
            };
            if show {
                if unsafe { list_func_head(fp, false, false) }.is_err() {
                    return;
                }
                if function_list_modified(prev_ht_changed) != 0 {
                    return;
                }
            }
        }
        idx += 1;
    }
}

/// `:function /pattern/`: compile the pattern, list what it matches, and
/// answer where it ends.
pub(crate) fn list_functions_matching_pat(excmd: &mut ExArg) -> usize {
    let start = excmd.line.arg + 1;
    let mut at = start + skip_regexp_at(excmd.line.tail(start), b'/' as c_int, 1);
    if !excmd.skip {
        let mut regmatch = REGMATCH_INIT;
        // Terminate the pattern for `vim_regcomp`, then put the byte back.
        // The compiler still takes a `char *`; p32-8 gives it the slice.
        let c = excmd.line.byte_at(at);
        excmd.line.set_byte(at, 0);
        let pat = excmd.line.ptr_at(start);
        // SAFETY: the pattern, terminated in place just above.
        regmatch.regprog = unsafe { vim_regcomp(pat, RE_MAGIC) };
        excmd.line.set_byte(at, c);
        if !regmatch.regprog.is_null() {
            regmatch.rm_ic = p_ic();
            unsafe { list_functions(&raw mut regmatch) };
            unsafe { vim_regfree(regmatch.regprog) };
        }
    }
    if excmd.line.byte_at(at) == b'/' {
        at += 1;
    }
    at
}

/// `:function Name`: print one function with its numbered body lines.
/// Answers the function, so that the caller can go on to redefine it.
///
/// # Safety
/// `excmd` is a live `:function` command, `name` the translated name and `p`
/// the rest of the command line.
pub(crate) unsafe fn list_one_function(
    excmd: &mut ExArg,
    name: *mut c_char,
    p: *mut c_char,
) -> *mut UserFunc {
    // SAFETY: the caller's promise -- `excmd` is the Ex command being run.
    if ends_excmd(unsafe { *skipwhite(p) } as c_int) == 0 {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let p = unsafe { c_str(p) };
        semsg!("E488: Trailing characters: {p}");
        return ptr::null_mut();
    }
    // `p` is the cursor `trans_function_name` left in the line; the walk
    // still answers a pointer, so it comes back to an offset here.
    let at = excmd.line.offset_of(p);
    excmd.line.next = excmd.line.check_next(at);
    if excmd.line.next.is_some() {
        excmd.line.terminate_at(at);
    }
    if excmd.skip || got_int.get() {
        return ptr::null_mut();
    }

    let fp = unsafe { find_func(name) };
    if fp.is_null() {
        unsafe { emsg_funcname(c"E123: Undefined function: %s".as_ptr(), name) };
        return ptr::null_mut();
    }

    // Check no function was added or removed from a callback, and
    // therefore that `fp` is still the function this started on.
    let prev_ht_changed = func_table().changed();
    unsafe { msg_ext_set_kind(c"list_cmd".as_ptr()) };
    if unsafe { list_func_head(fp, !excmd.forceit, excmd.forceit) }.is_err() {
        return fp;
    }
    // SAFETY: `fp` is the live function just listed.
    let f = unsafe { Uf::new(fp) };
    let lines = ga_strings(&f.uf_lines);
    for (j, &line) in lines.iter().enumerate() {
        if got_int.get() {
            break;
        }
        if line.is_null() {
            continue;
        }
        msg_putchar(b'\n' as c_int);
        if !excmd.forceit {
            // The line number, right-aligned in three columns.
            msg_outnum(j as c_int + 1);
            if j < 9 {
                msg_putchar(b' ' as c_int);
            }
            if j < 99 {
                msg_putchar(b' ' as c_int);
            }
            if function_list_modified(prev_ht_changed) != 0 {
                break;
            }
        }
        unsafe { msg_prt_line(line, false) };
        line_breakcheck();
    }
    if !got_int.get() {
        msg_putchar(b'\n' as c_int);
        if function_list_modified(prev_ht_changed) == 0 {
            let end = if excmd.forceit {
                c"endfunction".as_ptr()
            } else {
                c"   endfunction".as_ptr()
            };
            msg_str(unsafe { cstr::at(end) });
        }
    }
    fp
}

/// Whether a function of this *already translated* name exists, builtin or
/// user-defined.
///
/// # Safety
/// `name` is NUL-terminated.
pub unsafe fn translated_function_exists(name: *const c_char) -> bool {
    if unsafe { builtin_function(name, -1) } {
        return !unsafe { find_internal_func(name) }.is_null();
    }
    !unsafe { find_func(name) }.is_null()
}

/// `exists('*name')`: whether `name` names a function, without autoloading
/// one to find out.
///
/// # Safety
/// `name` is NUL-terminated.
pub unsafe fn function_exists(name: *const c_char, no_deref: bool) -> bool {
    let mut nm = name;
    let mut n = false;
    let mut flag = TFN_INT | TFN_QUIET | TFN_NO_AUTOLOAD;
    if no_deref {
        flag |= TFN_NO_DEREF;
    }
    let nmp = (&raw mut nm) as *mut *mut c_char;
    // SAFETY: `nm` is this frame's own cursor into the caller's name.
    let p = unsafe { trans_function_name(nmp, false, flag, ptr::null_mut(), ptr::null_mut()) };
    nm = unsafe { skipwhite(nm) };

    // Only accept "funcname", "funcname ", "funcname (..." and
    // "funcname(...", not "funcname!...".
    if !p.is_null() && (unsafe { *nm } == NUL as c_char || unsafe { *nm } == b'(' as c_char) {
        n = unsafe { translated_function_exists(p) };
    }
    unsafe { xfree(p as *mut c_void) };
    n
}

/// Completion over the user functions: answers the `idx`th name, resuming
/// from where the last call stopped.
///
/// Keeps the raw signature because a completion table holds a pointer to it.
///
/// # Safety
/// Called with `idx` 0 first, then increasing, with no change to the
/// function table in between.
pub unsafe fn get_user_func_name(expand: *mut Expand, idx: c_int) -> *mut c_char {
    static done: GlobalCell<size_t> = GlobalCell::new(0);
    static changed: GlobalCell<c_int> = GlobalCell::new(0);
    // The cursor is a slot *index*: it is parked in a `static` across calls
    // into the editor, and the table's small run lives inside the table, so
    // a pointer would not survive the next mutation of it.
    static slot: GlobalCell<usize> = GlobalCell::new(0);

    if idx == 0 {
        done.set(0);
        slot.set(0);
        changed.set(func_table().changed());
    }
    if changed.get() != func_table().changed() || done.get() >= func_table().used() {
        return ptr::null_mut();
    }

    if done.get() > 0 {
        slot.set(slot.get() + 1);
    }
    done.set(done.get() + 1);
    while !func_table().slot(slot.get()).is_kept() {
        slot.set(slot.get() + 1);
    }
    // The key *is* the function's trailing name member, so the function is
    // that many bytes before it.
    let key = func_table().slot(slot.get()).hi_key;
    let fp = unsafe { key.sub(offset_of!(UserFunc, uf_name)) } as *mut UserFunc;

    if unsafe { (*fp).uf_flags }.has(FuncFlags::DICT)
        || unsafe { cstr::starts_with(uf_name_ptr(fp), b"<lambda>") }
    {
        // Don't show dict and lambda functions.
        return c"".as_ptr() as *mut c_char;
    }
    if unsafe { (*fp).uf_namelen } + 4 >= IOSIZE as size_t {
        // Prevent overflow.
        return uf_name_ptr(fp);
    }

    let buf = unsafe { (*expand).xp_buf.as_mut_ptr() };
    let mut len = unsafe { cat_func_name(buf, IOSIZE as size_t, fp) };
    if unsafe { (*expand).xp_context } != ExpandContext::UserFunc {
        // SAFETY: `buf` is the completion buffer of `IOSIZE` bytes, of
        // which `len` are used, and `fp` is the live function.
        let at = unsafe { buf.offset(len as isize) };
        let left = (IOSIZE as size_t).wrapping_sub(len as size_t);
        unsafe { xstrlcpy(at, c"(".as_ptr(), left) };
        let f = unsafe { Uf::new(fp) };
        if f.uf_varargs == 0 && f.uf_args.ga_len <= 0 {
            len += 1;
            let at = unsafe { buf.offset(len as isize) };
            let left = (IOSIZE as size_t).wrapping_sub(len as size_t);
            unsafe { xstrlcpy(at, c")".as_ptr(), left) };
        }
    }
    buf
}

/// `:delfunction`.
pub fn ex_delfunction(excmd: &mut ExArg) {
    // SAFETY: the caller's promise -- `excmd` is the Ex command being run.
    let mut fudi = FUNCDICT_INIT;
    let mut p = excmd.arg_ptr();
    let name =
        unsafe { trans_function_name(&raw mut p, excmd.skip, 0, &raw mut fudi, ptr::null_mut()) };
    unsafe { xfree(fudi.fd_newkey as *mut c_void) };
    if name.is_null() {
        if !fudi.fd_dict.is_null() && !excmd.skip {
            emsg(gettext(E_FUNCREF));
        }
        return;
    }
    if ends_excmd(unsafe { *skipwhite(p) } as c_int) == 0 {
        unsafe { xfree(name as *mut c_void) };
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let p = unsafe { c_str(p) };
        semsg!("E488: Trailing characters: {p}");
        return;
    }
    // As `list_one_function`: `p` is where the name walk stopped.
    let at = excmd.line.offset_of(p);
    excmd.line.next = excmd.line.check_next(at);
    if excmd.line.next.is_some() {
        excmd.line.terminate_at(at);
    }

    if (unsafe { *name } as u8).is_ascii_digit() && fudi.fd_dict.is_null() {
        // Numbered function.
        if !excmd.skip {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let arg = msg_bytes(excmd.line.arg());
            semsg!("E475: Invalid argument: {arg}");
        }
        unsafe { xfree(name as *mut c_void) };
        return;
    }
    let fp = if !excmd.skip {
        unsafe { find_func(name) }
    } else {
        ptr::null_mut()
    };
    unsafe { xfree(name as *mut c_void) };
    if excmd.skip {
        return;
    }

    if fp.is_null() {
        if !excmd.forceit {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let arg = msg_bytes(excmd.line.arg());
            semsg!("E130: Unknown function: {arg}");
        }
        return;
    }
    if unsafe { (*fp).uf_calls } > 0 {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let arg = msg_bytes(excmd.line.arg());
        semsg!("E131: Cannot delete function {arg}: It is in use");
        return;
    }
    // `> 2` because deleting a function should also drop a reference, and
    // 1 is the initial refcount.  A funccall that outlived its call --
    // one that returned `a:000`, or that a closure captured -- holds one
    // of its own until the garbage collector frees it, which is why this
    // arm is reachable at all (see the docket's O-B14-13).
    if unsafe { (*fp).uf_refcount }.get() > 2 {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let arg = msg_bytes(excmd.line.arg());
        semsg!("Cannot delete function {arg}: It is being used internally");
        return;
    }

    if !fudi.fd_dict.is_null() {
        // Delete the dict item that refers to the function; that invokes
        // `func_unref` and possibly deletes the function.
        unsafe { tv_dict_item_remove(fudi.fd_dict, fudi.fd_di) };
        return;
    }
    // A normal function has a refcount of 1 for its entry in the
    // hashtable; a numbered function or a lambda has none.  Above that,
    // something else still holds it, so unlink it but keep it.
    let held = if unsafe { func_name_refcount(uf_name_ptr(fp)) } {
        0
    } else {
        1
    };
    if unsafe { (*fp).uf_refcount }.get() > held {
        if unsafe { func_remove(fp) } {
            unsafe { (*fp).uf_refcount.release() };
        }
        unsafe { (*fp).uf_flags |= FuncFlags::DELETED };
    } else {
        unsafe { func_clear_free(fp, false) };
    }
}
