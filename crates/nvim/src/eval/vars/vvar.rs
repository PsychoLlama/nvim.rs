//! Reading and writing `v:` from C.
//!
//! Two families: the `get_vim_var_*` readers, which are how the rest of the
//! editor asks what a `v:` variable holds, and the `set_vim_var_*` writers,
//! which are how it publishes one.  [`before_set_vvar`] is the Vimscript
//! side of the same thing: the type enforcement `:let v:x = …` goes through.
//!
//! Every one of them indexes the `vimvars` table by [`Vv`], so none
//! of them can fail; the table's entries are `dictitem_T`-shaped and are the
//! same items `v:` the dictionary holds.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{c_char, c_int};
use core::ptr;

use super::*;
use crate::eval::typval::NumBuf;
use crate::types::{NUL, OK};

/// Save `v:` variable `idx` into `save_tv` and blank it, adding it to the
/// `v:` dictionary if it is one of the two that are not normally there.
///
/// Pairs with [`restore_vimvar`].
///
/// # Safety
/// `idx` names a `v:` variable and `save_tv` is writable.
pub unsafe fn prepare_vimvar(idx: Vv, save_tv: *mut typval_T) {
    unsafe {
        let vv = vimvar_table().offset(idx as isize);
        *save_tv = (*vv).vv_di.di_tv;
        (*vv).vv_di.di_tv.vval.v_string = ptr::null_mut();
        if (*vv).vv_di.di_tv.v_type == VAR_UNKNOWN {
            // `v:val` and `v:key` have no type until something sets one, and
            // are absent from the dictionary until then.
            hash_add(get_vimvar_ht(), (&raw mut (*vv).vv_di.di_key).cast());
        }
    }
}

/// Put back what [`prepare_vimvar`] saved.
///
/// # Safety
/// As [`prepare_vimvar`], with the `save_tv` it filled.
pub unsafe fn restore_vimvar(idx: Vv, save_tv: *mut typval_T) {
    unsafe {
        let vv = vimvar_table().offset(idx as isize);
        (*vv).vv_di.di_tv = *save_tv;
        if (*vv).vv_di.di_tv.v_type != VAR_UNKNOWN {
            return;
        }
        let hi = hash_find(get_vimvar_ht(), (&raw mut (*vv).vv_di.di_key).cast());
        if (*hi).is_kept() {
            hash_remove(get_vimvar_ht(), hi);
        } else {
            internal_error(c"restore_vimvar()".as_ptr());
        }
    }
}

/// Copy `tv` into `v:` variable `idx`.
///
/// # Safety
/// `idx` names a `v:` variable and `tv` is a live value.
pub unsafe fn set_vim_var_tv(idx: Vv, tv: *mut typval_T) {
    unsafe {
        let tv_out = get_vim_var_tv(idx);
        tv_clear(tv_out);
        tv_copy(tv, tv_out);
    }
}

/// The name of `v:` variable `idx`, without the `v:`.
///
/// # Safety
/// `idx` names a `v:` variable.
pub unsafe fn get_vim_var_name(idx: Vv) -> *mut c_char {
    // SAFETY: `idx` is a `Vv` discriminant, so it is a row of the table.
    unsafe { (*vimvar_table().add(idx as usize)).vv_name }
}

/// The value of `v:` variable `idx`, which the caller may write through.
///
/// # Safety
/// `idx` names a `v:` variable.
pub unsafe fn get_vim_var_tv(idx: Vv) -> *mut typval_T {
    unsafe { &raw mut (*vimvar_table().offset(idx as isize)).vv_di.di_tv }
}

/// `v:` variable `idx` as a Number.  The caller knows its declared type.
///
/// # Safety
/// As [`get_vim_var_tv`].
pub unsafe fn get_vim_var_nr(idx: Vv) -> varnumber_T {
    unsafe { (*get_vim_var_tv(idx)).vval.v_number }
}

/// `v:` variable `idx` as a List.
///
/// # Safety
/// As [`get_vim_var_tv`].
pub unsafe fn get_vim_var_list(idx: Vv) -> *mut list_T {
    unsafe { (*get_vim_var_tv(idx)).vval.v_list }
}

/// `v:` variable `idx` as a Dict.
///
/// # Safety
/// As [`get_vim_var_tv`].
pub unsafe fn get_vim_var_dict(idx: Vv) -> *mut dict_T {
    unsafe { (*get_vim_var_tv(idx)).vval.v_dict }
}

/// `v:` variable `idx` as a string — the variable's own, with an unset one
/// reading as empty.
///
/// Every variable asked for here is declared `VAR_STRING` and `E963` refuses
/// an assignment of another type, so there is nothing to convert and nothing
/// to convert it into: the answer lives as long as the variable does, which
/// is what the callers holding it across a call need.
///
/// # Safety
/// As [`get_vim_var_tv`].
pub unsafe fn get_vim_var_str(idx: Vv) -> *mut c_char {
    // SAFETY: the caller's obligation, and the table slot is initialised.
    let tv = unsafe { &*get_vim_var_tv(idx) };
    debug_assert_eq!(tv.v_type, VAR_STRING, "v: variable {idx:?} is not a String");
    // SAFETY: the type tag says the union holds the string arm.
    let s = unsafe { tv.vval.v_string };
    if s.is_null() {
        c"".as_ptr().cast_mut()
    } else {
        s
    }
}

/// `v:` variable `idx` as a Partial.
///
/// # Safety
/// As [`get_vim_var_tv`].
pub unsafe fn get_vim_var_partial(idx: Vv) -> *mut partial_T {
    unsafe { (*get_vim_var_tv(idx)).vval.v_partial }
}

/// Declare `v:` variable `idx` to be of type `type_0`, without touching its
/// value.
///
/// # Safety
/// As [`get_vim_var_tv`].
pub unsafe fn set_vim_var_type(idx: Vv, type_0: VarType) {
    unsafe { (*get_vim_var_tv(idx)).v_type = type_0 }
}

/// Set `v:` variable `idx` to the Number `val`.
///
/// # Safety
/// As [`get_vim_var_tv`].
pub unsafe fn set_vim_var_nr(idx: Vv, val: varnumber_T) {
    unsafe {
        let tv = get_vim_var_tv(idx);
        tv_clear(tv);
        (*tv).vval.v_number = val;
    }
}

/// Set `v:` variable `idx` to `v:true` or `v:false`.
///
/// # Safety
/// As [`get_vim_var_tv`].
pub unsafe fn set_vim_var_bool(idx: Vv, val: BoolVarValue) {
    unsafe {
        let tv = get_vim_var_tv(idx);
        tv_clear(tv);
        (*tv).v_type = VAR_BOOL;
        (*tv).vval.v_bool = val;
    }
}

/// Set `v:` variable `idx` to `v:null`.
///
/// # Safety
/// As [`get_vim_var_tv`].
pub unsafe fn set_vim_var_special(idx: Vv, val: SpecialVarValue) {
    unsafe {
        let tv = get_vim_var_tv(idx);
        tv_clear(tv);
        (*tv).v_type = VAR_SPECIAL;
        (*tv).vval.v_special = val;
    }
}

/// Set `v:char` to the character `c`.
///
/// # Safety
/// Nothing; `utf_char2bytes` writes at most six bytes, so the NUL lands
/// inside `buf`.
pub unsafe fn set_vim_var_char(c: c_int) {
    let mut buf = [0 as c_char; 7];
    unsafe {
        let buflen = utf_char2bytes(c, buf.as_mut_ptr());
        buf[buflen as usize] = NUL as c_char;
        set_vim_var_string(Vv::Char, buf.as_ptr(), buflen as ptrdiff_t);
    }
}

/// Set `v:` variable `idx` to a copy of `val`, which is `len` bytes long or
/// NUL-terminated when `len` is -1.  A NULL `val` is the null string.
///
/// # Safety
/// As [`get_vim_var_tv`]; `val` is NULL or readable for `len`.
pub unsafe fn set_vim_var_string(idx: Vv, val: *const c_char, len: ptrdiff_t) {
    unsafe {
        let tv = get_vim_var_tv(idx);
        tv_clear(tv);
        (*tv).v_type = VAR_STRING;
        (*tv).vval.v_string = if val.is_null() {
            ptr::null_mut()
        } else if len == -1 {
            xstrdup(val)
        } else {
            xstrndup(val, len as size_t)
        };
    }
}

/// Set `v:` variable `idx` to `val`, taking a reference to it.
///
/// # Safety
/// As [`get_vim_var_tv`]; `val` is NULL or a live list.
pub unsafe fn set_vim_var_list(idx: Vv, val: *mut list_T) {
    unsafe {
        let tv = get_vim_var_tv(idx);
        tv_clear(tv);
        (*tv).v_type = VAR_LIST;
        (*tv).vval.v_list = val;
        if !val.is_null() {
            tv_list_ref(val);
        }
    }
}

/// Set `v:` variable `idx` to `val`, taking a reference to it and making its
/// keys read-only.
///
/// # Safety
/// As [`get_vim_var_tv`]; `val` is NULL or a live dictionary.
pub unsafe fn set_vim_var_dict(idx: Vv, val: *mut dict_T) {
    unsafe {
        let tv = get_vim_var_tv(idx);
        tv_clear(tv);
        (*tv).v_type = VAR_DICT;
        (*tv).vval.v_dict = val;
        if val.is_null() {
            return;
        }
        (*val).dv_refcount += 1;
        tv_dict_set_keys_readonly(val);
    }
}

/// Set `v:lua`'s partial.
///
/// Upstream writes the union member without setting `v_type`, because the
/// table already declares `v:lua` a `VAR_PARTIAL` and nothing ever replaces
/// it; this runs once, from `evalvars_init`.
///
/// # Safety
/// As [`get_vim_var_tv`]; `val` is a live partial whose reference the caller
/// hands over.
pub unsafe fn set_vim_var_partial(idx: Vv, val: *mut partial_T) {
    unsafe { (*get_vim_var_tv(idx)).vval.v_partial = val }
}

/// Set `v:register` to `c`, or to `"` for the unnamed register.
///
/// # Safety
/// Nothing; `c` is a register name or 0.
pub unsafe fn set_reg_var(c: c_int) {
    unsafe {
        let regname = if c == 0 || c == b' ' as c_int {
            b'"' as c_char
        } else {
            c as c_char
        };
        // Only write when it changed, to avoid the reallocation. The test
        // is against `c`, not against the name that would be stored, so
        // `set_reg_var(0)` always rewrites -- upstream's.
        let tv = get_vim_var_tv(Vv::Register);
        if (*tv).vval.v_string.is_null() || *(*tv).vval.v_string != c as c_char {
            let buf = [regname, NUL as c_char];
            set_vim_var_string(Vv::Register, buf.as_ptr(), 1);
        }
    }
}

/// Get or restore `v:exception`: a NULL `oldval` reads it, anything else
/// puts that value back and answers NULL.
///
/// Always called in pairs, and neither half allocates or frees.
///
/// # Safety
/// `oldval` is NULL or a string this took out earlier.
pub unsafe fn v_exception(oldval: *mut c_char) -> *mut c_char {
    unsafe {
        let tv = get_vim_var_tv(Vv::Exception);
        if oldval.is_null() {
            return (*tv).vval.v_string;
        }
        (*tv).vval.v_string = oldval;
        ptr::null_mut()
    }
}

/// [`v_exception`] for `v:throwpoint`.
///
/// # Safety
/// As [`v_exception`].
pub unsafe fn v_throwpoint(oldval: *mut c_char) -> *mut c_char {
    unsafe {
        let tv = get_vim_var_tv(Vv::Throwpoint);
        if oldval.is_null() {
            return (*tv).vval.v_string;
        }
        (*tv).vval.v_string = oldval;
        ptr::null_mut()
    }
}

/// Set `v:cmdarg` to the `++opt` arguments of `eap`, answering the old value
/// for the caller to restore.
///
/// A NULL `eap` is the restore half: `oldarg` goes back and the value that
/// was there is freed.  The same happens if any of the pieces fails to
/// format, which is why the answer is NULL on that path -- there is nothing
/// left for the caller to put back.
///
/// The size is worked out in full first, so the writes below cannot
/// truncate; `xlen` accumulates what each one *would* have written, which is
/// what makes the closing bound check meaningful.
///
/// # Safety
/// `eap` is NULL or a live command; `oldarg` is NULL or an owned string.
pub unsafe fn set_cmdarg(eap: *mut exarg_T, oldarg: *mut c_char) -> *mut c_char {
    unsafe {
        let tv = get_vim_var_tv(Vv::Cmdarg);
        let oldval = (*tv).vval.v_string;

        'error: {
            if eap.is_null() {
                break 'error;
            }
            let mut len: size_t = 0;
            if (*eap).force_bin == FORCE_BIN {
                len += 6; // " ++bin"
            } else if (*eap).force_bin == FORCE_NOBIN {
                len += 8; // " ++nobin"
            }
            if (*eap).read_edit != 0 {
                len += 7; // " ++edit"
            }
            if (*eap).force_ff != 0 {
                len += 10; // " ++ff=unix"
            }
            if (*eap).force_enc != 0 {
                len += strlen((*eap).cmd.offset((*eap).force_enc as isize)) + 7;
            }
            if (*eap).bad_char != 0 {
                len += 7 + 4; // " ++bad=" + "keep" or "drop"
            }
            if (*eap).mkdir_p != 0 {
                len += 4; // " ++p"
            }

            let newval_len = len + 1;
            let newval = xmalloc(newval_len) as *mut c_char;
            let mut xlen: size_t = 0;

            // Append one piece. A macro rather than a closure because
            // `snprintf` is variadic; it bails to `'error` exactly where
            // upstream's `goto error` does, and `mechdiff` cannot see
            // through it, so this file's `snprintf` count reads as 1.
            macro_rules! put {
                ($($arg:tt)*) => {{
                    let rc = snprintf(newval.add(xlen), newval_len - xlen, $($arg)*);
                    if rc < 0 {
                        break 'error;
                    }
                    xlen += rc as size_t;
                }};
            }

            if (*eap).force_bin == FORCE_BIN {
                put!(c" ++bin".as_ptr());
            } else if (*eap).force_bin == FORCE_NOBIN {
                put!(c" ++nobin".as_ptr());
            } else {
                *newval = NUL as c_char;
            }
            if (*eap).read_edit != 0 {
                put!(c" ++edit".as_ptr());
            }
            if (*eap).force_ff != 0 {
                let ff = match (*eap).force_ff as u8 {
                    b'u' => c"unix",
                    b'd' => c"dos",
                    _ => c"mac",
                };
                put!(c" ++ff=%s".as_ptr(), ff.as_ptr());
            }
            if (*eap).force_enc != 0 {
                put!(
                    c" ++enc=%s".as_ptr(),
                    (*eap).cmd.offset((*eap).force_enc as isize)
                );
            }
            if (*eap).bad_char == BAD_KEEP {
                put!(c" ++bad=keep".as_ptr());
            } else if (*eap).bad_char == BAD_DROP {
                put!(c" ++bad=drop".as_ptr());
            } else if (*eap).bad_char != 0 {
                put!(c" ++bad=%c".as_ptr(), (*eap).bad_char);
            }
            if (*eap).mkdir_p != 0 {
                put!(c" ++p".as_ptr());
            }
            debug_assert!(xlen <= newval_len);

            (*tv).vval.v_string = newval;
            return oldval;
        }

        xfree(oldval.cast());
        (*tv).vval.v_string = oldarg;
        ptr::null_mut()
    }
}

/// Set `v:count` and `v:count1`, and `v:prevcount` from the old `v:count`
/// first when asked.
///
/// # Safety
/// Nothing.
pub unsafe fn set_vcount(count: int64_t, count1: int64_t, set_prevcount: bool) {
    unsafe {
        if set_prevcount {
            (*get_vim_var_tv(Vv::Prevcount)).vval.v_number = get_vim_var_nr(Vv::Count);
        }
        (*get_vim_var_tv(Vv::Count)).vval.v_number = count as varnumber_T;
        (*get_vim_var_tv(Vv::Count1)).vval.v_number = count1 as varnumber_T;
    }
}

/// The type enforcement a write to a `v:` variable passes.
///
/// A `v:` variable keeps the type the table declares for it, so a String or
/// a Number one converts what it is given rather than replacing it -- and
/// two of them, `v:searchforward` and `v:hlsearch`, have a side effect on
/// the editor when they change.  Both of those cases do the store
/// themselves, notify the watchers and answer **false**: there is nothing
/// left for the caller to do.  Any other declared type accepts only a value
/// of the same type; a mismatch sets `type_error` (E963) and also answers
/// false.  True means "type checked out, store it the ordinary way".
///
/// # Safety
/// `varname` is the name without the `v:`, `di` its item in the `v:` table,
/// `tv` the value being stored and `type_error` writable.
pub unsafe fn before_set_vvar(
    varname: *const c_char,
    di: *mut dictitem_T,
    tv: *mut typval_T,
    copy: bool,
    watched: bool,
    type_error: *mut bool,
) -> bool {
    let mut numbuf = NumBuf::new();
    unsafe {
        if (*di).di_tv.v_type == VAR_STRING {
            let mut oldtv = TV_INITIAL_VALUE;
            if watched {
                tv_copy(&raw mut (*di).di_tv, &raw mut oldtv);
            }
            xfree((*di).di_tv.vval.v_string.cast());
            (*di).di_tv.vval.v_string = ptr::null_mut();

            if copy || (*tv).v_type != VAR_STRING {
                let val = numbuf.string(tv);
                // Careful: assigning to v:errmsg, `tv_get_string()` may
                // itself raise an error, which sets the variable -- so only
                // store when it is still empty.
                if (*di).di_tv.vval.v_string.is_null() {
                    (*di).di_tv.vval.v_string = xstrdup(val);
                }
            } else {
                // Take the string over, rather than copy and free.
                (*di).di_tv.vval.v_string = (*tv).vval.v_string;
                (*tv).vval.v_string = ptr::null_mut();
            }
            if watched {
                tv_dict_watcher_notify(
                    get_vimvar_dict(),
                    varname,
                    &raw mut (*di).di_tv,
                    &raw mut oldtv,
                );
                tv_clear(&raw mut oldtv);
            }
            return false;
        } else if (*di).di_tv.v_type == VAR_NUMBER {
            let mut oldtv = TV_INITIAL_VALUE;
            if watched {
                tv_copy(&raw mut (*di).di_tv, &raw mut oldtv);
            }
            (*di).di_tv.vval.v_number = tv_get_number(tv);
            let n = (*di).di_tv.vval.v_number;
            if strcmp(varname, c"searchforward".as_ptr()) == 0 {
                set_search_direction(if n != 0 { b'/' as c_int } else { b'?' as c_int });
            } else if strcmp(varname, c"hlsearch".as_ptr()) == 0 {
                no_hlsearch.set(n == 0);
                redraw_all_later(UPD_SOME_VALID);
            }
            if watched {
                tv_dict_watcher_notify(
                    get_vimvar_dict(),
                    varname,
                    &raw mut (*di).di_tv,
                    &raw mut oldtv,
                );
                tv_clear(&raw mut oldtv);
            }
            return false;
        } else if (*di).di_tv.v_type != (*tv).v_type {
            *type_error = true;
            return false;
        }
        true
    }
}

/// A write to a `v:` variable that reached the scope dictionary directly:
/// `let v:['name'] = value`.
///
/// The subscripted spelling makes `get_lval` resolve `v:` to a plain
/// `dict_T` and `set_var_lval` store straight into the `dictitem_T`, so
/// upstream never runs [`before_set_vvar`] for it and the declared type of
/// the variable is simply replaced.  That is a crash and not only a
/// surprise: `get_vim_var_list(Vv::Oldfiles)` reads `vval.v_list` with no
/// type test, so `let v:['oldfiles'] = 1` followed by `:oldfiles`
/// dereferences the address 1.  See docket O-B14-10.
///
/// This does the same work `set_var_const` does for the unsubscripted
/// spelling, including the compound operators -- which `set_whole_var`
/// applies to a *copy* of the current value before handing the result to
/// `set_var_const`, so that `let v:searchforward .= 'x'` converts back to a
/// Number rather than replacing one.
///
/// # Safety
/// `di` is an item of the `v:` scope dictionary, `tv` the value being
/// assigned, and `op` NULL or the assignment's one-character operator.
pub(crate) unsafe fn set_vvar_item(
    di: *mut dictitem_T,
    tv: *mut typval_T,
    copy: bool,
    op: *const c_char,
) {
    unsafe {
        let varname = tv_dict_item_key(di);
        let watched = tv_dict_is_watched(get_vimvar_dict());

        // `+=` and friends act on the current value, so evaluate them into a
        // temporary first and enforce the type on the *result*.
        let mut tmp = TV_INITIAL_VALUE;
        let compound = !op.is_null() && *op != b'=' as c_char;
        let val = if compound {
            tv_copy(&raw mut (*di).di_tv, &raw mut tmp);
            if eexe_mod_op(&raw mut tmp, tv, op) != OK {
                tv_clear(&raw mut tmp);
                return;
            }
            &raw mut tmp
        } else {
            tv
        };

        let mut type_error = false;
        // The temporary is ours to free, so the store must copy out of it
        // rather than take its string.
        if !before_set_vvar(
            varname,
            di,
            val,
            copy || compound,
            watched,
            &raw mut type_error,
        ) {
            if type_error {
                semsg_c!(
                    gettext(e_setting_v_str_to_value_with_wrong_type.as_ptr()),
                    varname,
                );
            }
            tv_clear(&raw mut tmp);
            return;
        }

        // The declared type matched: the ordinary store, as `set_var_const`
        // performs it.
        let mut oldtv = TV_INITIAL_VALUE;
        if watched {
            tv_copy(&raw mut (*di).di_tv, &raw mut oldtv);
        }
        tv_clear(&raw mut (*di).di_tv);
        if !compound && (copy || (*val).v_type == VAR_NUMBER || (*val).v_type == VAR_FLOAT) {
            tv_copy(val, &raw mut (*di).di_tv);
        } else {
            (*di).di_tv = *val;
            (*di).di_tv.v_lock = VAR_UNLOCKED;
            tv_init(val);
        }
        if watched {
            tv_dict_watcher_notify(
                get_vimvar_dict(),
                varname,
                &raw mut (*di).di_tv,
                &raw mut oldtv,
            );
            tv_clear(&raw mut oldtv);
        }
        tv_clear(&raw mut tmp);
    }
}

/// Blank the six `v:option_*` variables the `OptionSet` autocommand reads.
///
/// # Safety
/// Nothing.
pub unsafe fn reset_v_option_vars() {
    unsafe {
        for idx in [
            Vv::OptionNew,
            Vv::OptionOld,
            Vv::OptionOldlocal,
            Vv::OptionOldglobal,
            Vv::OptionCommand,
            Vv::OptionType,
        ] {
            set_vim_var_string(idx, ptr::null(), -1);
        }
    }
}
