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
    unsafe {
        let prev_ht_changed = func_table().changed();
        let mut todo = func_table().used();
        let mut hi: *const hashitem_T = func_table().array();

        msg_ext_set_kind(c"list_cmd".as_ptr());
        while todo > 0 && !got_int.get() {
            if (*hi).is_kept() {
                let fp = (*hi).hi_key.sub(offset_of!(ufunc_T, uf_name)) as *mut ufunc_T;
                todo -= 1;
                // Without a pattern, skip what the user filtered out and the
                // numbered/lambda functions; with one, skip the numbered
                // functions and ask the pattern.
                let show = if regmatch.is_null() {
                    !message_filtered(uf_name_ptr(fp)) && !func_name_refcount(uf_name_ptr(fp))
                } else {
                    !(*uf_name_ptr(fp) as u8).is_ascii_digit()
                        && vim_regexec(regmatch, uf_name_ptr(fp), 0)
                };
                if show {
                    if list_func_head(fp, false, false) == FAIL {
                        return;
                    }
                    if function_list_modified(prev_ht_changed) != 0 {
                        return;
                    }
                }
            }
            hi = hi.add(1);
        }
    }
}

/// `:function /pattern/`: compile the pattern, list what it matches, and
/// answer the end of it.
///
/// # Safety
/// `eap` is a live `:function` command whose argument starts with `/`.
pub(crate) unsafe fn list_functions_matching_pat(eap: *mut exarg_T) -> *mut c_char {
    unsafe {
        let mut p = skip_regexp((*eap).arg.add(1), b'/' as c_int, 1);
        if (*eap).skip == 0 {
            let mut regmatch = REGMATCH_INIT;
            // Terminate the pattern for `vim_regcomp`, then put the byte back.
            let c = *p;
            *p = NUL as c_char;
            regmatch.regprog = vim_regcomp((*eap).arg.add(1), RE_MAGIC);
            *p = c;
            if !regmatch.regprog.is_null() {
                regmatch.rm_ic = p_ic.get() != 0;
                list_functions(&raw mut regmatch);
                vim_regfree(regmatch.regprog);
            }
        }
        if *p == b'/' as c_char {
            p = p.add(1);
        }
        p
    }
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
    unsafe {
        if ends_excmd(*skipwhite(p) as c_int) == 0 {
            semsg_c!(gettext(&raw const e_trailing_arg as *const c_char), p);
            return ptr::null_mut();
        }
        (*eap).nextcmd = check_nextcmd(p);
        if !(*eap).nextcmd.is_null() {
            *p = NUL as c_char;
        }
        if (*eap).skip != 0 || got_int.get() {
            return ptr::null_mut();
        }

        let fp = find_func(name);
        if fp.is_null() {
            emsg_funcname(c"E123: Undefined function: %s".as_ptr(), name);
            return ptr::null_mut();
        }

        // Check no function was added or removed from a callback, and
        // therefore that `fp` is still the function this started on.
        let prev_ht_changed = func_table().changed();
        msg_ext_set_kind(c"list_cmd".as_ptr());
        if list_func_head(fp, (*eap).forceit == 0, (*eap).forceit != 0) != OK {
            return fp;
        }
        let lines = ga_strings(&(*fp).uf_lines);
        for (j, &line) in lines.iter().enumerate() {
            if got_int.get() {
                break;
            }
            if line.is_null() {
                continue;
            }
            msg_putchar(b'\n' as c_int);
            if (*eap).forceit == 0 {
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
            msg_prt_line(line, false);
            line_breakcheck();
        }
        if !got_int.get() {
            msg_putchar(b'\n' as c_int);
            if function_list_modified(prev_ht_changed) == 0 {
                msg_puts(if (*eap).forceit != 0 {
                    c"endfunction".as_ptr()
                } else {
                    c"   endfunction".as_ptr()
                });
            }
        }
        fp
    }
}

/// Whether a function of this *already translated* name exists, builtin or
/// user-defined.
///
/// # Safety
/// `name` is NUL-terminated.
pub unsafe fn translated_function_exists(name: *const c_char) -> bool {
    unsafe {
        if builtin_function(name, -1) {
            return !find_internal_func(name).is_null();
        }
        !find_func(name).is_null()
    }
}

/// `exists('*name')`: whether `name` names a function, without autoloading
/// one to find out.
///
/// # Safety
/// `name` is NUL-terminated.
pub unsafe fn function_exists(name: *const c_char, no_deref: bool) -> bool {
    unsafe {
        let mut nm = name;
        let mut n = false;
        let mut flag = TFN_INT | TFN_QUIET | TFN_NO_AUTOLOAD;
        if no_deref {
            flag |= TFN_NO_DEREF;
        }
        let p = trans_function_name(
            &raw mut nm as *mut *mut c_char,
            false,
            flag,
            ptr::null_mut(),
            ptr::null_mut(),
        );
        nm = skipwhite(nm);

        // Only accept "funcname", "funcname ", "funcname (..." and
        // "funcname(...", not "funcname!...".
        if !p.is_null() && (*nm == NUL as c_char || *nm == b'(' as c_char) {
            n = translated_function_exists(p);
        }
        xfree(p as *mut c_void);
        n
    }
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
    unsafe {
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
            hi.set(hi.get().add(1));
        }
        done.set(done.get() + 1);
        while !(*hi.get()).is_kept() {
            hi.set(hi.get().add(1));
        }
        let fp = (*hi.get()).hi_key.sub(offset_of!(ufunc_T, uf_name)) as *mut ufunc_T;

        if (*fp).uf_flags & FC_DICT != 0 || strncmp(uf_name_ptr(fp), c"<lambda>".as_ptr(), 8) == 0 {
            // Don't show dict and lambda functions.
            return c"".as_ptr() as *mut c_char;
        }
        if (*fp).uf_namelen + 4 >= IOSIZE as size_t {
            // Prevent overflow.
            return uf_name_ptr(fp);
        }

        let buf = (*xp).xp_buf.as_mut_ptr();
        let mut len = cat_func_name(buf, IOSIZE as size_t, fp);
        if (*xp).xp_context != ExpandContext::UserFunc {
            xstrlcpy(
                buf.offset(len as isize),
                c"(".as_ptr(),
                (IOSIZE as size_t).wrapping_sub(len as size_t),
            );
            if (*fp).uf_varargs == 0 && (*fp).uf_args.ga_len <= 0 {
                len += 1;
                xstrlcpy(
                    buf.offset(len as isize),
                    c")".as_ptr(),
                    (IOSIZE as size_t).wrapping_sub(len as size_t),
                );
            }
        }
        buf
    }
}

/// `:delfunction`.
///
/// # Safety
/// `eap` is a live `:delfunction` command.
pub unsafe fn ex_delfunction(eap: *mut exarg_T) {
    unsafe {
        let mut fudi = FUNCDICT_INIT;
        let mut p = (*eap).arg;
        let name = trans_function_name(
            &raw mut p,
            (*eap).skip != 0,
            0,
            &raw mut fudi,
            ptr::null_mut(),
        );
        xfree(fudi.fd_newkey as *mut c_void);
        if name.is_null() {
            if !fudi.fd_dict.is_null() && (*eap).skip == 0 {
                emsg(gettext(E_FUNCREF.as_ptr()));
            }
            return;
        }
        if ends_excmd(*skipwhite(p) as c_int) == 0 {
            xfree(name as *mut c_void);
            semsg_c!(gettext(&raw const e_trailing_arg as *const c_char), p);
            return;
        }
        (*eap).nextcmd = check_nextcmd(p);
        if !(*eap).nextcmd.is_null() {
            *p = NUL as c_char;
        }

        if (*name as u8).is_ascii_digit() && fudi.fd_dict.is_null() {
            // Numbered function.
            if (*eap).skip == 0 {
                semsg_c!(gettext(&raw const e_invarg2 as *const c_char), (*eap).arg);
            }
            xfree(name as *mut c_void);
            return;
        }
        let fp = if (*eap).skip == 0 {
            find_func(name)
        } else {
            ptr::null_mut()
        };
        xfree(name as *mut c_void);
        if (*eap).skip != 0 {
            return;
        }

        if fp.is_null() {
            if (*eap).forceit == 0 {
                semsg_c!(gettext(E_NOFUNC.as_ptr()), (*eap).arg);
            }
            return;
        }
        if (*fp).uf_calls > 0 {
            semsg_c!(
                gettext(c"E131: Cannot delete function %s: It is in use".as_ptr()),
                (*eap).arg,
            );
            return;
        }
        // `> 2` because deleting a function should also drop a reference, and
        // 1 is the initial refcount.  A funccall that outlived its call --
        // one that returned `a:000`, or that a closure captured -- holds one
        // of its own until the garbage collector frees it, which is why this
        // arm is reachable at all (see the docket's O-B14-13).
        if (*fp).uf_refcount.get() > 2 {
            semsg_c!(
                gettext(c"Cannot delete function %s: It is being used internally".as_ptr()),
                (*eap).arg,
            );
            return;
        }

        if !fudi.fd_dict.is_null() {
            // Delete the dict item that refers to the function; that invokes
            // `func_unref` and possibly deletes the function.
            tv_dict_item_remove(fudi.fd_dict, fudi.fd_di);
            return;
        }
        // A normal function has a refcount of 1 for its entry in the
        // hashtable; a numbered function or a lambda has none.  Above that,
        // something else still holds it, so unlink it but keep it.
        let held = if func_name_refcount(uf_name_ptr(fp)) {
            0
        } else {
            1
        };
        if (*fp).uf_refcount.get() > held {
            if func_remove(fp) {
                (*fp).uf_refcount.release();
            }
            (*fp).uf_flags |= FC_DELETED;
        } else {
            func_clear_free(fp, false);
        }
    }
}
