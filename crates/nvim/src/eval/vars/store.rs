//! Assigning to a name, and the four checks that can refuse.
//!
//! [`set_var_const`] is the single entry point every assignment reaches; the
//! `var_check_*` trio reads `di_flags` and produces E46 / E795 / E1122, and
//! [`var_wrong_func_name`] and [`valid_varname`] reject the name itself.
//!
//! The three checks each resolve `name_len` the same way before reporting,
//! because it may be one of the two sentinels `TV_TRANSLATE` and
//! `TV_CSTRING` rather than a length: a caller with no length to hand passes
//! one of those so that `strlen` and `gettext` are only paid for on the path
//! that actually reports.  The repetition is upstream's, and keeping it is
//! what keeps each check a single `unsafe` block.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{c_char, c_int};
use core::ptr;

use super::*;
use crate::types::FAIL;

/// Store `tv` in the variable `name`.
///
/// # Safety
/// `name` points at `name_len` readable bytes and is NUL-terminated there;
/// `tv` is a live value.
pub unsafe fn set_var(name: *const c_char, name_len: size_t, tv: *mut typval_T, copy: bool) {
    unsafe { set_var_const(name, name_len, tv, copy, false) }
}

/// Store `tv` in the variable `name`, creating it if it does not exist.
///
/// `copy` asks for a copy of the value; without it `tv` is moved out of and
/// left `VAR_UNKNOWN`, except for the two scalar types, which are copied
/// either way.  `is_const` is `:const`: the variable is created locked, and
/// an *existing* one is refused outright.
///
/// # Safety
/// As [`set_var`].
pub unsafe fn set_var_const(
    name: *const c_char,
    name_len: size_t,
    tv: *mut typval_T,
    copy: bool,
    is_const: bool,
) {
    unsafe {
        let mut varname: *const c_char = ptr::null();
        let mut dict: *mut dict_T = ptr::null_mut();
        let ht = find_var_ht_dict(name, name_len, &raw mut varname, &raw mut dict);
        let watched = tv_dict_is_watched(dict);

        if ht.is_null() || *varname == NUL {
            semsg_c!(gettext(&raw const e_illvar as *const c_char), name);
            return;
        }
        // `varname` is `name` itself or `name + 2`, so this cannot go
        // negative; a name that is nothing but a scope prefix was caught by
        // the NUL test above.
        let varname_len = name_len - varname.offset_from(name) as size_t;

        let mut di = find_var_in_ht(ht, 0, varname, varname_len, true);
        if di.is_null() {
            // Search the parent scope, which a lambda can reference.
            di = find_var_in_scoped_ht(name, name_len, true as c_int);
        }

        if tv_is_func(*tv) && var_wrong_func_name(name, di.is_null()) {
            return;
        }

        let mut oldtv = TV_INITIAL_VALUE;
        if !di.is_null() {
            if is_const {
                emsg(gettext(&raw const e_cannot_mod as *const c_char));
                return;
            }

            // The order is upstream's and is kept for backwards
            // compatibility: read-only first, then the value's lock, then
            // the variable's.
            if var_check_ro((*di).di_flags as c_int, name, name_len)
                || value_check_lock((*di).di_tv.v_lock, name, name_len)
                || var_check_lock((*di).di_flags as c_int, name, name_len)
            {
                return;
            }

            // A `v:` variable keeps its declared type, and two of them have
            // a side effect on assignment; `before_set_vvar` is both, and it
            // answers false when it has already done the store itself.
            let mut type_error = false;
            if ht == &raw mut (*vimvardict.ptr()).dv_hashtab
                && !before_set_vvar(varname, di, tv, copy, watched, &raw mut type_error)
            {
                if type_error {
                    semsg_c!(
                        gettext(e_setting_v_str_to_value_with_wrong_type.as_ptr()),
                        varname,
                    );
                }
                return;
            }

            if watched {
                tv_copy(&raw mut (*di).di_tv, &raw mut oldtv);
            }
            tv_clear(&raw mut (*di).di_tv);
        } else {
            // A new variable. `v:` and `a:` do not take one.
            if ht == &raw mut (*vimvardict.ptr()).dv_hashtab || ht == get_funccal_args_ht() {
                semsg_c!(gettext(&raw const e_illvar as *const c_char), name);
                return;
            }
            if !valid_varname(varname) {
                return;
            }
            debug_assert!(!dict.is_null());

            // Upstream allocates and copies the key by hand here, taking
            // `varname_len + 1` bytes so that the NUL comes along.  That is
            // what `tv_dict_item_alloc_len` does, and it is where every
            // other item in the tree comes from; `valid_varname` has just
            // walked `varname` to its NUL, so the two agree on the length.
            di = tv_dict_item_alloc_len(varname, varname_len);
            if hash_add(ht, tv_dict_item_key(di)) == FAIL {
                xfree(di.cast());
                return;
            }
            (*di).di_flags = DI_FLAGS_ALLOC as uint8_t;
            if is_const {
                (*di).di_flags |= DI_FLAGS_LOCK as uint8_t;
            }
        }

        if copy || (*tv).v_type == VAR_NUMBER || (*tv).v_type == VAR_FLOAT {
            tv_copy(tv, &raw mut (*di).di_tv);
        } else {
            (*di).di_tv = *tv;
            (*di).di_tv.v_lock = VAR_UNLOCKED;
            tv_init(tv);
        }

        if watched {
            tv_dict_watcher_notify(
                dict,
                tv_dict_item_key(di),
                &raw mut (*di).di_tv,
                &raw mut oldtv,
            );
            tv_clear(&raw mut oldtv);
        }

        if is_const {
            // Like `:lockvar! name`: lock the value and what it contains,
            // but only where the reference count is one, so that only
            // literal values are locked.
            tv_item_lock(&raw mut (*di).di_tv, DICT_MAXNEST, true, true);
        }
    }
}

/// Whether `flags` says the variable may not be written, reporting E46 or
/// E794 if so.
///
/// `name_len` may be `TV_TRANSLATE` or `TV_CSTRING` rather than a length.
///
/// # Safety
/// `name` is NUL-terminated, or `name_len` bytes long.
pub unsafe fn var_check_ro(flags: c_int, mut name: *const c_char, mut name_len: size_t) -> bool {
    unsafe {
        let error_message = if flags & DI_FLAGS_RO as c_int != 0 {
            &raw const e_cannot_change_readonly_variable_str as *const c_char
        } else if flags & DI_FLAGS_RO_SBX as c_int != 0 && sandbox.get() != 0 {
            &raw const e_cannot_set_variable_in_sandbox_str as *const c_char
        } else {
            return false;
        };

        if name_len == TV_TRANSLATE as size_t {
            name = gettext(name);
            name_len = strlen(name);
        } else if name_len == TV_CSTRING as size_t {
            name_len = strlen(name);
        }
        semsg_c!(gettext(error_message), name_len as c_int, name);
        true
    }
}

/// Whether `flags` says the variable is locked, reporting E1122 if so.
///
/// # Safety
/// As [`var_check_ro`].
pub unsafe fn var_check_lock(flags: c_int, mut name: *const c_char, mut name_len: size_t) -> bool {
    unsafe {
        if flags & DI_FLAGS_LOCK as c_int == 0 {
            return false;
        }
        if name_len == TV_TRANSLATE as size_t {
            name = gettext(name);
            name_len = strlen(name);
        } else if name_len == TV_CSTRING as size_t {
            name_len = strlen(name);
        }
        semsg_c!(
            gettext(c"E1122: Variable is locked: %.*s".as_ptr()),
            name_len as c_int,
            name,
        );
        true
    }
}

/// Whether `flags` says the variable may not be deleted, reporting E795 if
/// so.
///
/// # Safety
/// As [`var_check_ro`].
pub unsafe fn var_check_fixed(flags: c_int, mut name: *const c_char, mut name_len: size_t) -> bool {
    unsafe {
        if flags & DI_FLAGS_FIX as c_int == 0 {
            return false;
        }
        if name_len == TV_TRANSLATE as size_t {
            name = gettext(name);
            name_len = strlen(name);
        } else if name_len == TV_CSTRING as size_t {
            name_len = strlen(name);
        }
        semsg_c!(
            gettext(&raw const e_cannot_delete_variable_str as *const c_char),
            name_len as c_int,
            name,
        );
        true
    }
}

/// Whether `name` may not hold a Funcref, reporting E704 or E705 if so.
///
/// A Funcref has to look like a function name -- capitalised, or scoped to
/// `w:`/`b:`/`s:`/`t:`, or autoloaded -- and `new_var` additionally forbids
/// shadowing a function that already exists.
///
/// # Safety
/// `name` is a NUL-terminated string.
pub unsafe fn var_wrong_func_name(name: *const c_char, new_var: bool) -> bool {
    unsafe {
        let has_scope = *name != NUL && *name.add(1) == b':' as c_char;
        // The character the capital is wanted at: past a scope prefix, if
        // there is one.
        let first = if has_scope { *name.add(2) } else { *name };
        let func_scope =
            has_scope && !vim_strchr(c"wbst".as_ptr(), *name as uint8_t as c_int).is_null();

        if !func_scope
            && !(first as u8).is_ascii_uppercase()
            && vim_strchr(name, b'#' as c_int).is_null()
        {
            semsg_c!(
                gettext(c"E704: Funcref variable name must start with a capital: %s".as_ptr()),
                name,
            );
            return true;
        }
        // Don't allow hiding a function. With an existing variable this may
        // be assigning another function to the same one, whose type the
        // caller checks.
        if new_var && function_exists(name, false) {
            semsg_c!(
                gettext(c"E705: Variable name conflicts with existing function: %s".as_ptr()),
                name,
            );
            return true;
        }
        false
    }
}

/// Whether `varname` is spellable as a variable name, reporting E461 if not.
///
/// # Safety
/// `varname` is a NUL-terminated string.
pub unsafe fn valid_varname(varname: *const c_char) -> bool {
    unsafe {
        let mut p = varname;
        while *p != NUL {
            if !eval_isnamec1(*p as uint8_t as c_int)
                && (p == varname || !ascii_isdigit(*p as c_int))
                && *p != AUTOLOAD_CHAR as c_char
            {
                semsg_c!(gettext(&raw const e_illvar as *const c_char), varname);
                return false;
            }
            p = p.add(1);
        }
        true
    }
}
