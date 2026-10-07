//! `:let` with no value: printing variables rather than setting them.
//!
//! [`list_arg_vars`] resolves each argument (including a bare scope name)
//! and [`list_one_var_a`] does the printing, padding the name to column 22
//! and prefixing the value with `#`, `*`, `[` or `{` by type.  That layout
//! is a contract: it is what a user sees.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::cstr;
use crate::semsg;
use crate::winlayer::TabPage;
use crate::winlayer::{Buf, Win};
use core::ffi::{CStr, c_char, c_int};
use core::mem::offset_of;
use core::ptr;

use super::*;
use crate::eval::typval::{DictRef, DictTab};
use crate::types::{IOSIZE, NUL};

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
    ht: *mut DictTab,
    prefix: *const c_char,
    empty: bool,
    first: *mut c_int,
) {
    for hi in unsafe { tv_ht_iter(ht) } {
        // Upstream re-reads `got_int` in the loop condition, so a `:let`
        // listing stops at the interrupt rather than at the end.
        if got_int.get() {
            break;
        }
        let di = tv_dict_hi2di(hi);
        let mut buf = [0 as c_char; IOSIZE as usize];
        unsafe { xstrlcpy(buf.as_mut_ptr(), prefix, IOSIZE as size_t) };
        unsafe { xstrlcat(buf.as_mut_ptr(), (*di).di_key.as_ptr(), IOSIZE as size_t) };
        if message_filtered(unsafe { cstr::at(buf.as_mut_ptr()) }) {
            continue;
        }
        if empty
            || unsafe { (*di).di_tv.v_type() } != VAR_STRING
            || unsafe { (*di).di_tv.string_ref() }.is_some()
        {
            unsafe { list_one_var(di, prefix, first) };
        }
    }
}

/// The variables in `dict`, each shown with `prefix` in front of its name.
pub(crate) fn list_dict_vars(dict: &DictRef, prefix: &CStr, empty: bool, first: &mut c_int) {
    // SAFETY: the dictionary's own table, live for as long as the caller
    // holds it, a NUL-terminated prefix, and the caller's `first`.
    unsafe {
        list_hashtable_vars(
            ptr::from_mut(&mut dict.edit().dv_hashtab),
            prefix.as_ptr(),
            empty,
            first,
        )
    }
}

/// The `g:` scope.
pub(crate) fn list_glob_vars(first: &mut c_int) {
    list_dict_vars(&globvar_dict(), c"", true, first);
}

/// The current buffer's `b:` scope.
pub(crate) fn list_buf_vars(first: &mut c_int) {
    // SAFETY: the current buffer's own `b:` dictionary.
    let ht = unsafe { &raw mut (*Buf::current().b_vars).dv_hashtab };
    // SAFETY: a live scope table, and the caller's `first`.
    unsafe { list_hashtable_vars(ht, c"b:".as_ptr(), true, first) }
}

/// The current window's `w:` scope.
pub(crate) fn list_win_vars(first: &mut c_int) {
    // SAFETY: the current window's own `w:` dictionary.
    let ht = unsafe { &raw mut (*Win::current().w_vars).dv_hashtab };
    // SAFETY: a live scope table, and the caller's `first`.
    unsafe { list_hashtable_vars(ht, c"w:".as_ptr(), true, first) }
}

/// The current tab page's `t:` scope.
pub(crate) fn list_tab_vars(first: &mut c_int) {
    // SAFETY: `curtab` is set from startup to exit, and the tab page's own
    // `t:` dictionary is live with it.
    let ht = unsafe { &raw mut (*TabPage::current().tp_vars).dv_hashtab };
    // SAFETY: a live scope table, and the caller's `first`.
    unsafe { list_hashtable_vars(ht, c"t:".as_ptr(), true, first) }
}

/// The `v:` scope.  `empty` is false: the `v:` variables that hold no string
/// are not listed.
pub(crate) fn list_vim_vars(first: &mut c_int) {
    list_dict_vars(&vimvar_dict(), c"v:", false, first);
}

/// The current script's `s:` scope, if there is one.
pub(crate) fn list_script_vars(first: &mut c_int) {
    let sid = current_sctx.get().sc_sid;
    if let Some(dict) = script_scope_dict(sid) {
        list_dict_vars(&dict, c"s:", false, first);
    }
}

/// `:let name …`: print each named variable in `text`, or the whole of a
/// scope named on its own. `skip` only checks that the names parse. Answers
/// where in `text` it stopped.
pub(crate) fn list_arg_vars(text: &[u8], skip: bool, first: &mut c_int) -> usize {
    let mut error = false;
    let mut arg = 0;
    let rest = |at: usize| text.get(at..).unwrap_or_default();
    while ends_excmd(c_int::from(cstr::byte_at(text, arg))) == 0 && !got_int.get() {
        if error || skip {
            // Nothing is being printed any more; just check that what is
            // left parses as names.
            let flags = FNE_INCL_BR | FNE_CHECK_START;
            arg += name_end(rest(arg), flags).end;
            let c = c_int::from(cstr::byte_at(text, arg));
            if !ascii_iswhite(c) && ends_excmd(c) == 0 {
                emsg_severe.set(true);
                let shown = crate::message_fmt::msg_bytes(rest(arg));
                semsg!("E488: Trailing characters: {shown}");
                break;
            }
            arg += skip::white(rest(arg));
            continue;
        }

        let name_start = arg;
        // A `{curly}` name is expanded into `tofree`.
        let mut cursor = Cursor::new(rest(arg));
        let (len, tofree) = get_name_len(&mut cursor, true, true);
        arg += cursor.offset();
        'done: {
            if len <= 0 {
                if len < 0 && !aborting() {
                    emsg_severe.set(true);
                    let shown = crate::message_fmt::msg_bytes(rest(arg));
                    semsg!("E475: Invalid argument: {shown}");
                    return arg;
                }
                error = true;
                break 'done;
            }
            let len = usize::try_from(len).expect("a positive length");
            let name: &[u8] = match &tofree {
                Some(expanded) => {
                    let expanded: &[u8] = expanded;
                    expanded.get(..len).unwrap_or(expanded)
                }
                None => &text[name_start..name_start + len],
            };

            let mut tv = TV_INITIAL_VALUE;
            if eval_variable(name, Some(&mut tv), true, false).is_err() {
                error = true;
                break 'done;
            }
            // The subscript is read from the same text as the name.
            let arg_subsc = arg;
            let subscripted = handle_subscript(&mut cursor, &mut tv, true, true);
            arg = name_start + cursor.offset();
            if subscripted.is_err() {
                error = true;
                break 'done;
            }

            if arg == arg_subsc && len == 2 && name[1] == b':' {
                // A bare scope name lists the whole scope.
                let lister: Option<ScopeLister> = match name[0] {
                    b'g' => Some(list_glob_vars),
                    b'b' => Some(list_buf_vars),
                    b'w' => Some(list_win_vars),
                    b't' => Some(list_tab_vars),
                    b'v' => Some(list_vim_vars),
                    b's' => Some(list_script_vars),
                    b'l' => Some(list_func_vars),
                    _ => None,
                };
                match lister {
                    Some(lister) => lister(first),
                    None => {
                        let name = crate::message_fmt::msg_bytes(name);
                        semsg!("E738: Can't list variables for {name}");
                    }
                }
            } else {
                // SAFETY: a live value, and no state to thread.
                let s = encode_tv2echo(&tv).into_raw();
                // Without a subscript the expanded name is what was
                // looked up; with one, the command line's own text is
                // what should be shown.
                let shown: &[u8] = match &tofree {
                    Some(expanded) if arg == arg_subsc => expanded,
                    _ => &text[name_start..arg],
                };
                let text = if s.is_null() { c"".as_ptr() } else { s };
                let ty = tv.v_type();
                let name_size = ptrdiff_t::try_from(shown.len()).expect("a name fits");
                // SAFETY: a NUL-terminated rendering, a name of `name_size`
                // bytes, and the caller's `first`.
                unsafe {
                    list_one_var_a(
                        c"".as_ptr(),
                        shown.as_ptr().cast(),
                        name_size,
                        ty,
                        text,
                        first,
                    )
                };
                // SAFETY: the rendering is this frame's own allocation.
                unsafe { xfree(s.cast()) };
            }
            clear_local(&mut tv);
        }
        drop(tofree);
        arg += skip::white(rest(arg));
    }
    arg
}

/// One variable, rendering its value with `encode_tv2echo`.
///
/// # Safety
/// `v` is a live item, `prefix` a NUL-terminated string, `first` writable.
unsafe fn list_one_var(v: *mut DictItem, prefix: *const c_char, first: *mut c_int) {
    // SAFETY: the caller's obligation -- a live item, whose key and value
    // are its own.
    let item = unsafe { Di::new(v) };
    let key = unsafe { (*v).di_key.as_ptr() };
    let len = unsafe { (*v).di_key.len() } as ptrdiff_t;
    let tv = item.field_ptr::<TypVal>(offset_of!(DictItem, di_tv));
    let s = unsafe { encode_tv2echo(&*tv).into_raw() };
    let text = if s.is_null() { c"".as_ptr() } else { s };
    let ty = item.di_tv.v_type();
    unsafe { list_one_var_a(prefix, key, len, ty, text, first) };
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
    // SAFETY: the caller's obligation throughout -- `first` is writable,
    // `prefix` and `string` are NUL-terminated, and `name` is `name_len`
    // bytes or NULL. Every callee below writes to the message area.
    let is_first = unsafe { *first } != 0;
    if is_first {
        msg_ext_set_kind(c"list_cmd");
        msg_start();
    } else {
        msg_putchar(b'\n' as c_int);
    }
    // Not `msg()`, which would overwrite "v:statusmsg".
    if unsafe { *prefix } != NUL as c_char {
        msg_str(unsafe { cstr::at(prefix) });
    }
    if !name.is_null() {
        // SAFETY: `name_len` bytes follow `name`.
        msg_bytes(unsafe { cstr::slice_at(name, name_len as usize) }, 0, false);
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
    if (type_0 == VAR_LIST || type_0 == VAR_DICT) && unsafe { *string } == sigil as c_char {
        string = unsafe { string.add(1) };
    }

    msg_display(unsafe { cstr::at(string) }, 0, false);

    if type_0 == VAR_FUNC || type_0 == VAR_PARTIAL {
        msg_str(c"()");
    }
    if is_first {
        msg_clr_eos();
        unsafe { *first = 0 };
    }
}
