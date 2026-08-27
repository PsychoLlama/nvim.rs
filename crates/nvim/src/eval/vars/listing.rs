//! `:let` with no value: printing variables rather than setting them.
//!
//! [`list_arg_vars`] resolves each argument (including a bare scope name)
//! and [`list_one_var_a`] does the printing, padding the name to column 22
//! and prefixing the value with `#`, `*`, `[` or `{` by type.  That layout
//! is a contract: it is what a user sees.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int};
use core::ptr;

use super::*;
use crate::types::{FAIL, IOSIZE, NUL};

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
    for hi in tv_ht_iter(unsafe { &*ht }) {
        // Upstream re-reads `got_int` in the loop condition, so a `:let`
        // listing stops at the interrupt rather than at the end.
        if got_int.get() {
            break;
        }
        let di = unsafe { tv_dict_hi2di(hi) };
        let mut buf = [0 as c_char; IOSIZE as usize];
        unsafe { xstrlcpy(buf.as_mut_ptr(), prefix, IOSIZE as size_t) };
        unsafe { xstrlcat(buf.as_mut_ptr(), tv_dict_item_key(di), IOSIZE as size_t) };
        if unsafe { message_filtered(buf.as_mut_ptr()) } {
            continue;
        }
        if empty
            || unsafe { (*di).di_tv.v_type } != VAR_STRING
            || !unsafe { (*di).di_tv.vval.v_string }.is_null()
        {
            unsafe { list_one_var(di, prefix, first) };
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
            &raw mut (*cur_buf().b_vars).dv_hashtab,
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
            &raw mut (*cur_win().w_vars).dv_hashtab,
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
    unsafe { list_hashtable_vars(get_vimvar_ht(), c"v:".as_ptr(), false, first) }
}

/// The current script's `s:` scope, if there is one.
///
/// # Safety
/// As [`list_glob_vars`].
pub(crate) unsafe fn list_script_vars(first: *mut c_int) {
    let sid = current_sctx.get().sc_sid;
    if script_id_valid(sid) {
        unsafe {
            list_hashtable_vars(
                &raw mut (*script_sv(sid)).sv_dict.dv_hashtab,
                c"s:".as_ptr(),
                false,
                first,
            )
        };
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
    let mut evalarg = EVALARG_EVALUATE;
    let mut error = false;
    while ends_excmd(unsafe { *arg } as c_int) == 0 && !got_int.get() {
        if error || unsafe { (*eap).skip } != 0 {
            // Nothing is being printed any more; just check that what is
            // left parses as names.
            arg = unsafe {
                find_name_end(
                    arg,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    FNE_INCL_BR | FNE_CHECK_START,
                )
            };
            if !ascii_iswhite(unsafe { *arg } as c_int) && ends_excmd(unsafe { *arg } as c_int) == 0
            {
                emsg_severe.set(true);
                semsg_c!(
                    unsafe { gettext(&raw const e_trailing_arg as *const c_char) },
                    arg
                );
                break;
            }
            arg = unsafe { skipwhite(arg) };
            continue;
        }

        let name_start = arg;
        let mut name = arg;
        // A `{curly}` name is expanded into `tofree`.
        let mut tofree: *mut c_char = ptr::null_mut();
        let len = unsafe { get_name_len(&raw mut arg, &raw mut tofree, true, true) };
        'done: {
            if len <= 0 {
                if len < 0 && !aborting() {
                    emsg_severe.set(true);
                    semsg_c!(
                        unsafe { gettext(&raw const e_invarg2 as *const c_char) },
                        arg
                    );
                    unsafe { xfree(tofree.cast()) };
                    return arg;
                }
                error = true;
                break 'done;
            }
            if !tofree.is_null() {
                name = tofree;
            }

            let mut tv = TV_INITIAL_VALUE;
            if unsafe { eval_variable(name, len, &raw mut tv, ptr::null_mut(), true, false) }
                == FAIL
            {
                error = true;
                break 'done;
            }
            let arg_subsc = arg;
            if unsafe { handle_subscript(&raw mut arg, &raw mut tv, &raw mut evalarg, true) }
                == FAIL
            {
                error = true;
                break 'done;
            }

            if arg == arg_subsc && len == 2 && unsafe { *name.add(1) } == b':' as c_char {
                // A bare scope name lists the whole scope.
                match unsafe { *name } as u8 {
                    b'g' => unsafe { list_glob_vars(first) },
                    b'b' => unsafe { list_buf_vars(first) },
                    b'w' => unsafe { list_win_vars(first) },
                    b't' => unsafe { list_tab_vars(first) },
                    b'v' => unsafe { list_vim_vars(first) },
                    b's' => unsafe { list_script_vars(first) },
                    b'l' => unsafe { list_func_vars(first) },
                    _ => {
                        semsg_c!(
                            unsafe { gettext(c"E738: Can't list variables for %s".as_ptr()) },
                            name
                        );
                    }
                }
            } else {
                let s = unsafe { encode_tv2echo(&raw mut tv, ptr::null_mut()) };
                // Without a subscript the expanded name is what was
                // looked up; with one, the command line's own text is
                // what should be shown.
                let used_name = if arg == arg_subsc { name } else { name_start };
                let name_size = if ptr::eq(used_name, tofree) {
                    unsafe { strlen(used_name) as ptrdiff_t }
                } else {
                    unsafe { arg.offset_from(used_name) }
                };
                unsafe {
                    list_one_var_a(
                        c"".as_ptr(),
                        used_name,
                        name_size,
                        tv.v_type,
                        if s.is_null() { c"".as_ptr() } else { s },
                        first,
                    )
                };
                unsafe { xfree(s.cast()) };
            }
            unsafe { tv_clear(&raw mut tv) };
        }
        unsafe { xfree(tofree.cast()) };
        arg = unsafe { skipwhite(arg) };
    }
    arg
}

/// One variable, rendering its value with `encode_tv2echo`.
///
/// # Safety
/// `v` is a live item, `prefix` a NUL-terminated string, `first` writable.
unsafe fn list_one_var(v: *mut dictitem_T, prefix: *const c_char, first: *mut c_int) {
    let key = unsafe { tv_dict_item_key(v) };
    let s = unsafe { encode_tv2echo(&raw mut (*v).di_tv, ptr::null_mut()) };
    unsafe {
        list_one_var_a(
            prefix,
            key,
            strlen(key) as ptrdiff_t,
            (*v).di_tv.v_type,
            if s.is_null() { c"".as_ptr() } else { s },
            first,
        )
    };
    unsafe { xfree(s.cast()) };
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
    if unsafe { *first } != 0 {
        unsafe { msg_ext_set_kind(c"list_cmd".as_ptr()) };
        unsafe { msg_start() };
    } else {
        unsafe { msg_putchar(b'\n' as c_int) };
    }
    // Not `msg()`, which would overwrite "v:statusmsg".
    if unsafe { *prefix } != NUL as c_char {
        unsafe { msg_puts(prefix) };
    }
    if !name.is_null() {
        unsafe { msg_puts_len(name, name_len, 0, false) };
    }
    unsafe { msg_putchar(b' ' as c_int) };
    unsafe { msg_advance(22) };

    // The sigil, and the bracket it stands in for.
    let sigil: u8 = match type_0 {
        VAR_NUMBER => b'#',
        VAR_FUNC | VAR_PARTIAL => b'*',
        VAR_LIST => b'[',
        VAR_DICT => b'{',
        _ => b' ',
    };
    unsafe { msg_putchar(sigil as c_int) };
    if (type_0 == VAR_LIST || type_0 == VAR_DICT) && unsafe { *string } == sigil as c_char {
        string = unsafe { string.add(1) };
    }

    unsafe { msg_outtrans(string, 0, false) };

    if type_0 == VAR_FUNC || type_0 == VAR_PARTIAL {
        unsafe { msg_puts(c"()".as_ptr()) };
    }
    if unsafe { *first } != 0 {
        unsafe { msg_clr_eos() };
        unsafe { *first = 0 };
    }
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
