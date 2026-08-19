//! `:let` with no value: printing variables rather than setting them.
//!
//! [`list_arg_vars`] resolves each argument (including a bare scope name)
//! and [`list_one_var_a`] does the printing, padding the name to column 22
//! and prefixing the value with `#`, `*`, `[` or `{` by type.  That layout
//! is a contract: it is what a user sees.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{c_char, c_int};
use core::ptr;

use super::*;
use crate::types::FAIL;

/// Every variable of `ht`, one per line, each name prefixed with `prefix`.
///
/// `empty` includes the variables holding the null string, which only the
/// scopes that can hold one want.  `:filter` is applied to the prefixed
/// name.
///
/// # Safety
/// `ht` is a live variable hashtab, `prefix` a NUL-terminated string and
/// `first` writable.
pub unsafe fn list_hashtable_vars(
    ht: *mut hashtab_T,
    prefix: *const c_char,
    empty: bool,
    first: *mut c_int,
) {
    unsafe {
        for hi in tv_ht_iter(&*ht) {
            // Upstream re-reads `got_int` in the loop condition, so a `:let`
            // listing stops at the interrupt rather than at the end.
            if got_int.get() {
                break;
            }
            let di = tv_dict_hi2di(hi);
            let mut buf = [0 as c_char; IOSIZE as usize];
            xstrlcpy(buf.as_mut_ptr(), prefix, IOSIZE as size_t);
            xstrlcat(buf.as_mut_ptr(), tv_dict_item_key(di), IOSIZE as size_t);
            if message_filtered(buf.as_mut_ptr()) {
                continue;
            }
            if empty || (*di).di_tv.v_type != VAR_STRING || !(*di).di_tv.vval.v_string.is_null() {
                list_one_var(di, prefix, first);
            }
        }
    }
}

/// The `g:` scope.
///
/// # Safety
/// `first` is writable.
pub(crate) unsafe fn list_glob_vars(first: *mut c_int) {
    unsafe { list_hashtable_vars(get_globvar_ht(), c"".as_ptr(), true, first) }
}

/// The current buffer's `b:` scope.
///
/// # Safety
/// As [`list_glob_vars`].
pub(crate) unsafe fn list_buf_vars(first: *mut c_int) {
    unsafe {
        list_hashtable_vars(
            &raw mut (*(*curbuf.get()).b_vars).dv_hashtab,
            c"b:".as_ptr(),
            true,
            first,
        )
    }
}

/// The current window's `w:` scope.
///
/// # Safety
/// As [`list_glob_vars`].
pub(crate) unsafe fn list_win_vars(first: *mut c_int) {
    unsafe {
        list_hashtable_vars(
            &raw mut (*(*curwin.get()).w_vars).dv_hashtab,
            c"w:".as_ptr(),
            true,
            first,
        )
    }
}

/// The current tab page's `t:` scope.
///
/// # Safety
/// As [`list_glob_vars`].
pub(crate) unsafe fn list_tab_vars(first: *mut c_int) {
    unsafe {
        list_hashtable_vars(
            &raw mut (*(*curtab.get()).tp_vars).dv_hashtab,
            c"t:".as_ptr(),
            true,
            first,
        )
    }
}

/// The `v:` scope.  `empty` is false: the `v:` variables that hold no string
/// are not listed.
///
/// # Safety
/// As [`list_glob_vars`].
pub(crate) unsafe fn list_vim_vars(first: *mut c_int) {
    unsafe {
        list_hashtable_vars(
            &raw mut (*vimvardict.ptr()).dv_hashtab,
            c"v:".as_ptr(),
            false,
            first,
        )
    }
}

/// The current script's `s:` scope, if there is one.
///
/// # Safety
/// As [`list_glob_vars`].
pub(crate) unsafe fn list_script_vars(first: *mut c_int) {
    unsafe {
        let sid = (*current_sctx.ptr()).sc_sid;
        if sid > 0 && sid <= (*script_items.ptr()).ga_len {
            list_hashtable_vars(
                &raw mut (*script_sv(sid)).sv_dict.dv_hashtab,
                c"s:".as_ptr(),
                false,
                first,
            );
        }
    }
}

/// `:let name …`: print each named variable, or the whole of a scope named
/// on its own.  Answers where it stopped.
///
/// # Safety
/// `eap` is live, `arg` a NUL-terminated string and `first` writable.
pub(crate) unsafe fn list_arg_vars(
    eap: *mut exarg_T,
    mut arg: *const c_char,
    first: *mut c_int,
) -> *const c_char {
    unsafe {
        let mut error = false;
        while ends_excmd(*arg as c_int) == 0 && !got_int.get() {
            if error || (*eap).skip != 0 {
                // Nothing is being printed any more; just check that what is
                // left parses as names.
                arg = find_name_end(
                    arg,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    FNE_INCL_BR | FNE_CHECK_START,
                );
                if !ascii_iswhite(*arg as c_int) && ends_excmd(*arg as c_int) == 0 {
                    emsg_severe.set(true);
                    semsg_c!(gettext(&raw const e_trailing_arg as *const c_char), arg);
                    break;
                }
                arg = skipwhite(arg);
                continue;
            }

            let name_start = arg;
            let mut name = arg;
            // A `{curly}` name is expanded into `tofree`.
            let mut tofree: *mut c_char = ptr::null_mut();
            let len = get_name_len(&raw mut arg, &raw mut tofree, true, true);
            'done: {
                if len <= 0 {
                    if len < 0 && !aborting() {
                        emsg_severe.set(true);
                        semsg_c!(gettext(&raw const e_invarg2 as *const c_char), arg);
                        xfree(tofree.cast());
                        return arg;
                    }
                    error = true;
                    break 'done;
                }
                if !tofree.is_null() {
                    name = tofree;
                }

                let mut tv = TV_INITIAL_VALUE;
                if eval_variable(name, len, &raw mut tv, ptr::null_mut(), true, false) == FAIL {
                    error = true;
                    break 'done;
                }
                let arg_subsc = arg;
                if handle_subscript(&raw mut arg, &raw mut tv, EVALARG_EVALUATE.ptr(), true) == FAIL
                {
                    error = true;
                    break 'done;
                }

                if arg == arg_subsc && len == 2 && *name.add(1) == b':' as c_char {
                    // A bare scope name lists the whole scope.
                    match *name as u8 {
                        b'g' => list_glob_vars(first),
                        b'b' => list_buf_vars(first),
                        b'w' => list_win_vars(first),
                        b't' => list_tab_vars(first),
                        b'v' => list_vim_vars(first),
                        b's' => list_script_vars(first),
                        b'l' => list_func_vars(first),
                        _ => {
                            semsg_c!(gettext(c"E738: Can't list variables for %s".as_ptr()), name);
                        }
                    }
                } else {
                    let s = encode_tv2echo(&raw mut tv, ptr::null_mut());
                    // Without a subscript the expanded name is what was
                    // looked up; with one, the command line's own text is
                    // what should be shown.
                    let used_name = if arg == arg_subsc { name } else { name_start };
                    let name_size = if used_name == tofree as *const c_char {
                        strlen(used_name) as ptrdiff_t
                    } else {
                        arg.offset_from(used_name)
                    };
                    list_one_var_a(
                        c"".as_ptr(),
                        used_name,
                        name_size,
                        tv.v_type,
                        if s.is_null() { c"".as_ptr() } else { s },
                        first,
                    );
                    xfree(s.cast());
                }
                tv_clear(&raw mut tv);
            }
            xfree(tofree.cast());
            arg = skipwhite(arg);
        }
        arg
    }
}

/// One variable, rendering its value with `encode_tv2echo`.
///
/// # Safety
/// `v` is a live item, `prefix` a NUL-terminated string, `first` writable.
unsafe fn list_one_var(v: *mut dictitem_T, prefix: *const c_char, first: *mut c_int) {
    unsafe {
        let key = tv_dict_item_key(v);
        let s = encode_tv2echo(&raw mut (*v).di_tv, ptr::null_mut());
        list_one_var_a(
            prefix,
            key,
            strlen(key) as ptrdiff_t,
            (*v).di_tv.v_type,
            if s.is_null() { c"".as_ptr() } else { s },
            first,
        );
        xfree(s.cast());
    }
}

/// Print one `name  <sigil><value>` line.
///
/// The name is padded to column 22 and the sigil says what the type is:
/// `#` a Number, `*` a Funcref, `[` a List, `{` a Dict, a space anything
/// else.  For a List or a Dict the sigil replaces the bracket the rendered
/// value already starts with.
///
/// `first` clears the rest of the screen on the first line and is set false;
/// a NULL `name` is an `a:` variable, which stores none.
///
/// # Safety
/// `prefix` and `string` are NUL-terminated; `name` is NULL or `name_len`
/// bytes; `first` is writable.
unsafe fn list_one_var_a(
    prefix: *const c_char,
    name: *const c_char,
    name_len: ptrdiff_t,
    type_0: VarType,
    mut string: *const c_char,
    first: *mut c_int,
) {
    unsafe {
        if *first != 0 {
            msg_ext_set_kind(c"list_cmd".as_ptr());
            msg_start();
        } else {
            msg_putchar(b'\n' as c_int);
        }
        // Not `msg()`, which would overwrite "v:statusmsg".
        if *prefix != NUL {
            msg_puts(prefix);
        }
        if !name.is_null() {
            msg_puts_len(name, name_len, 0, false);
        }
        msg_putchar(b' ' as c_int);
        msg_advance(22);

        // The sigil, and the bracket it stands in for.
        let sigil: u8 = match type_0 {
            VAR_NUMBER => b'#',
            VAR_FUNC | VAR_PARTIAL => b'*',
            VAR_LIST => b'[',
            VAR_DICT => b'{',
            _ => b' ',
        };
        msg_putchar(sigil as c_int);
        if (type_0 == VAR_LIST || type_0 == VAR_DICT) && *string == sigil as c_char {
            string = string.add(1);
        }

        msg_outtrans(string, 0, false);

        if type_0 == VAR_FUNC || type_0 == VAR_PARTIAL {
            msg_puts(c"()".as_ptr());
        }
        if *first != 0 {
            msg_clr_eos();
            *first = false_0;
        }
    }
}
