//! Printing functions back, and `:delfunction`.
//!
//! `list_functions` walks the whole table, `list_functions_matching_pat`
//! the subset a `/pattern/` matches, and `list_one_function` prints one
//! with its numbered body lines.  `ex_delfunction` is here because it is
//! the same argument parse in reverse; `function_exists` and
//! `get_user_func_name` answer `exists('*x')` and completion.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{c_char, c_int, c_void};
use core::mem::offset_of;
use core::ptr;

use super::*;
use crate::types::{ExpandContext, FAIL, IOSIZE, NUL, OK};

/// Print the head of every function, or of the ones `regmatch` matches.
///
/// # Safety
/// `regmatch` is null or a compiled pattern.
pub(crate) unsafe fn list_functions(regmatch: *mut regmatch_T) {
    let prev_ht_changed = func_table().changed();
    let mut todo = func_table().used();
    let mut hi: *const hashitem_T = func_table().array();

    unsafe { msg_ext_set_kind(c"list_cmd".as_ptr()) };
    while todo > 0 && !got_int.get() {
        if unsafe { *hi }.is_kept() {
            // The key *is* the function's trailing name member, so the
            // function is that many bytes before it.
            let fp = unsafe { (*hi).hi_key.sub(offset_of!(ufunc_T, uf_name)) } as *mut ufunc_T;
            todo -= 1;
            // Without a pattern, skip what the user filtered out and the
            // numbered/lambda functions; with one, skip the numbered
            // functions and ask the pattern.
            let show = if regmatch.is_null() {
                !unsafe { message_filtered(uf_name_ptr(fp)) }
                    && !unsafe { func_name_refcount(uf_name_ptr(fp)) }
            } else {
                !(unsafe { *uf_name_ptr(fp) } as u8).is_ascii_digit()
                    && unsafe { vim_regexec(regmatch, uf_name_ptr(fp), 0) }
            };
            if show {
                if unsafe { list_func_head(fp, false, false) } == FAIL {
                    return;
                }
                if unsafe { function_list_modified(prev_ht_changed) } != 0 {
                    return;
                }
            }
        }
        hi = unsafe { hi.add(1) };
    }
}

/// `:function /pattern/`: compile the pattern, list what it matches, and
/// answer the end of it.
///
/// # Safety
/// `eap` is a live `:function` command whose argument starts with `/`.
pub(crate) unsafe fn list_functions_matching_pat(eap: *mut exarg_T) -> *mut c_char {
    // SAFETY: the caller's promise -- `eap` is the Ex command being run.
    let mut ea = unsafe { Ea::new(eap) };
    let mut p = unsafe { skip_regexp(ea.arg.add(1), b'/' as c_int, 1) };
    if ea.skip == 0 {
        let mut regmatch = REGMATCH_INIT;
        // Terminate the pattern for `vim_regcomp`, then put the byte back.
        let c = unsafe { *p };
        unsafe { *p = NUL as c_char };
        regmatch.regprog = unsafe { vim_regcomp(ea.arg.add(1), RE_MAGIC) };
        unsafe { *p = c };
        if !regmatch.regprog.is_null() {
            regmatch.rm_ic = p_ic.get() != 0;
            unsafe { list_functions(&raw mut regmatch) };
            unsafe { vim_regfree(regmatch.regprog) };
        }
    }
    if unsafe { *p } == b'/' as c_char {
        p = unsafe { p.add(1) };
    }
    p
}

/// `:function Name`: print one function with its numbered body lines.
/// Answers the function, so that the caller can go on to redefine it.
///
/// # Safety
/// `eap` is a live `:function` command, `name` the translated name and `p`
/// the rest of the command line.
pub(crate) unsafe fn list_one_function(
    eap: *mut exarg_T,
    name: *mut c_char,
    p: *mut c_char,
) -> *mut ufunc_T {
    // SAFETY: the caller's promise -- `eap` is the Ex command being run.
    let mut ea = unsafe { Ea::new(eap) };
    if ends_excmd(unsafe { *skipwhite(p) } as c_int) == 0 {
        unsafe { semsg_c!(gettext(e_trailing_arg), p) };
        return ptr::null_mut();
    }
    ea.nextcmd = unsafe { check_nextcmd(p) };
    if !ea.nextcmd.is_null() {
        unsafe { *p = NUL as c_char };
    }
    if ea.skip != 0 || got_int.get() {
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
    if unsafe { list_func_head(fp, ea.forceit == 0, ea.forceit != 0) } != OK {
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
        unsafe { msg_putchar(b'\n' as c_int) };
        if ea.forceit == 0 {
            // The line number, right-aligned in three columns.
            unsafe { msg_outnum(j as c_int + 1) };
            if j < 9 {
                unsafe { msg_putchar(b' ' as c_int) };
            }
            if j < 99 {
                unsafe { msg_putchar(b' ' as c_int) };
            }
            if unsafe { function_list_modified(prev_ht_changed) } != 0 {
                break;
            }
        }
        unsafe { msg_prt_line(line, false) };
        line_breakcheck();
    }
    if !got_int.get() {
        unsafe { msg_putchar(b'\n' as c_int) };
        if unsafe { function_list_modified(prev_ht_changed) } == 0 {
            let end = if ea.forceit != 0 {
                c"endfunction".as_ptr()
            } else {
                c"   endfunction".as_ptr()
            };
            unsafe { msg_puts(end) };
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
pub unsafe fn get_user_func_name(xp: *mut expand_T, idx: c_int) -> *mut c_char {
    static done: GlobalCell<size_t> = GlobalCell::new(0);
    static changed: GlobalCell<c_int> = GlobalCell::new(0);
    static hi: GlobalCell<*mut hashitem_T> = GlobalCell::new(ptr::null_mut());

    if idx == 0 {
        done.set(0);
        hi.set(func_table().array());
        changed.set(func_table().changed());
    }
    debug_assert!(!hi.get().is_null());
    if changed.get() != func_table().changed() || done.get() >= func_table().used() {
        return ptr::null_mut();
    }

    if done.get() > 0 {
        hi.set(unsafe { hi.get().add(1) });
    }
    done.set(done.get() + 1);
    while !unsafe { *hi.get() }.is_kept() {
        hi.set(unsafe { hi.get().add(1) });
    }
    // The key *is* the function's trailing name member, so the function is
    // that many bytes before it.
    let fp = unsafe { (*hi.get()).hi_key.sub(offset_of!(ufunc_T, uf_name)) } as *mut ufunc_T;

    if unsafe { (*fp).uf_flags } & FC_DICT != 0
        || unsafe { strncmp(uf_name_ptr(fp), c"<lambda>".as_ptr(), 8) } == 0
    {
        // Don't show dict and lambda functions.
        return c"".as_ptr() as *mut c_char;
    }
    if unsafe { (*fp).uf_namelen } + 4 >= IOSIZE as size_t {
        // Prevent overflow.
        return uf_name_ptr(fp);
    }

    let buf = unsafe { (*xp).xp_buf.as_mut_ptr() };
    let mut len = unsafe { cat_func_name(buf, IOSIZE as size_t, fp) };
    if unsafe { (*xp).xp_context } != ExpandContext::UserFunc {
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
///
/// # Safety
/// `eap` is a live `:delfunction` command.
pub unsafe fn ex_delfunction(eap: *mut exarg_T) {
    // SAFETY: the caller's promise -- `eap` is the Ex command being run.
    let mut ea = unsafe { Ea::new(eap) };
    let mut fudi = FUNCDICT_INIT;
    let mut p = ea.arg;
    let name =
        unsafe { trans_function_name(&raw mut p, ea.skip != 0, 0, &raw mut fudi, ptr::null_mut()) };
    unsafe { xfree(fudi.fd_newkey as *mut c_void) };
    if name.is_null() {
        if !fudi.fd_dict.is_null() && ea.skip == 0 {
            emsg(gettext(E_FUNCREF));
        }
        return;
    }
    if ends_excmd(unsafe { *skipwhite(p) } as c_int) == 0 {
        unsafe { xfree(name as *mut c_void) };
        unsafe { semsg_c!(gettext(e_trailing_arg), p) };
        return;
    }
    ea.nextcmd = unsafe { check_nextcmd(p) };
    if !ea.nextcmd.is_null() {
        unsafe { *p = NUL as c_char };
    }

    if (unsafe { *name } as u8).is_ascii_digit() && fudi.fd_dict.is_null() {
        // Numbered function.
        if ea.skip == 0 {
            unsafe { semsg_c!(gettext(e_invarg2), ea.arg) };
        }
        unsafe { xfree(name as *mut c_void) };
        return;
    }
    let fp = if ea.skip == 0 {
        unsafe { find_func(name) }
    } else {
        ptr::null_mut()
    };
    unsafe { xfree(name as *mut c_void) };
    if ea.skip != 0 {
        return;
    }

    if fp.is_null() {
        if ea.forceit == 0 {
            unsafe { semsg_c!(gettext(E_NOFUNC), ea.arg) };
        }
        return;
    }
    if unsafe { (*fp).uf_calls } > 0 {
        unsafe {
            semsg_c!(
                gettext(c"E131: Cannot delete function %s: It is in use"),
                ea.arg,
            )
        };
        return;
    }
    // `> 2` because deleting a function should also drop a reference, and
    // 1 is the initial refcount.  A funccall that outlived its call --
    // one that returned `a:000`, or that a closure captured -- holds one
    // of its own until the garbage collector frees it, which is why this
    // arm is reachable at all (see the docket's O-B14-13).
    if unsafe { (*fp).uf_refcount }.get() > 2 {
        unsafe {
            semsg_c!(
                gettext(c"Cannot delete function %s: It is being used internally"),
                ea.arg,
            )
        };
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
        unsafe { (*fp).uf_flags |= FC_DELETED };
    } else {
        unsafe { func_clear_free(fp, false) };
    }
}
