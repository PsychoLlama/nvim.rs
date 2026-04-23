//! Unlet and lockvar functions for VimL.
//!
//! Phase 4: Migrated from `src/nvim/eval/vars.c`.
//! Phase 8b: Migrated ex_unletlock, do_unlet_var, tv_list_unlet_range, do_lock_var.
//!
//! Functions:
//! - `rs_ex_unlet`: :unlet command entry point
//! - `rs_ex_lockvar`: :lockvar/:unlockvar command entry point
//! - `rs_do_unlet`: core variable unlet implementation

#![allow(unsafe_op_in_unsafe_fn)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::borrow_as_ptr)]
#![allow(clippy::manual_c_str_literals)]

use std::ffi::{c_char, c_int, c_void};

// Constants matching C definitions
const GLV_QUIET: c_int = 2; // TFN_QUIET

// TV_CSTRING sentinel for name_len
const TV_CSTRING: usize = usize::MAX - 1;

// OK / FAIL return codes
const OK: c_int = 1;
const FAIL: c_int = 0;

// =============================================================================
// C extern declarations
// =============================================================================

extern "C" {
    // --- eap accessors ---
    fn nvim_eap_get_arg(eap: *const c_void) -> *mut c_char;
    fn nvim_eap_get_forceit_int(eap: *const c_void) -> c_int;
    fn nvim_eap_get_skip(eap: *const c_void) -> c_int;
    fn nvim_eap_get_cmdidx(eap: *const c_void) -> c_int;
    fn nvim_eap_set_nextcmd(eap: *mut c_void, val: *mut c_char);

    // --- char ops ---
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn rs_ascii_isdigit(c: c_int) -> c_int;
    fn rs_ascii_iswhite(c: c_int) -> c_int;
    fn rs_get_env_len(arg: *mut *const c_char) -> c_int;
    fn getdigits_int(pp: *mut *mut c_char, strict: bool, def: c_int) -> c_int;

    // --- ends_excmd / check_nextcmd ---
    fn nvim_ends_excmd_char(c: c_int) -> bool;
    fn nvim_check_nextcmd(arg: *mut c_char) -> *mut c_char;
    fn nvim_emsg_severe_set();
    fn nvim_cmd_lockvar() -> c_int;

    // --- lval_T accessors ---
    fn nvim_vars_alloc_lval() -> *mut c_void;
    fn nvim_vars_clear_lval(lv: *mut c_void);
    fn nvim_lval_get_name(lv: *const c_void) -> *const c_char;
    fn nvim_lval_is_tv_null(lv: *const c_void) -> bool;
    fn nvim_lval_set_name_and_clear_tv(lv: *mut c_void, name: *mut c_char);
    fn nvim_vars_get_lval_unlet(
        name: *mut c_char,
        lv: *mut c_void,
        skip: bool,
        glv_flags: c_int,
    ) -> *mut c_char;
    fn nvim_lval_get_name_len(lv: *const c_void) -> usize;
    fn nvim_lval_get_list(lv: *const c_void) -> *mut c_void;
    fn nvim_lval_get_dict(lv: *const c_void) -> *mut c_void;
    fn nvim_lval_get_di(lv: *const c_void) -> *mut c_void;
    fn nvim_lval_get_li(lv: *const c_void) -> *mut c_void;
    fn nvim_lval_get_range(lv: *const c_void) -> bool;
    fn nvim_lval_get_empty2(lv: *const c_void) -> bool;
    fn nvim_lval_get_n1(lv: *const c_void) -> c_int;
    fn nvim_lval_get_n2(lv: *const c_void) -> c_int;
    fn nvim_lval_inc_n1(lv: *mut c_void);

    // --- var lookup ---
    fn nvim_vars_find_var_ht_dict(
        name: *const c_char,
        name_len: usize,
        varname: *mut *const c_char,
        dict_out: *mut *mut c_void,
    ) -> *mut c_void;
    fn get_current_funccal_dict(ht: *mut c_void) -> *mut c_void;
    fn nvim_get_globvarht() -> *mut c_void;
    fn get_globvar_dict() -> *mut c_void;
    fn nvim_get_compat_hashtab() -> *mut c_void;
    fn get_vimvar_dict() -> *mut c_void;
    fn find_var_in_ht(
        ht: *mut c_void,
        htname: c_int,
        varname: *const c_char,
        varname_len: usize,
        no_autoload: c_int,
    ) -> *mut c_void;
    fn nvim_find_var(name: *const c_char, name_len: usize, no_autoload: bool) -> *mut c_void;
    fn nvim_vars_dictitem_inner_dict(di: *mut c_void) -> *mut c_void;
    fn internal_error(where_: *const c_char);
    fn find_hi_in_scoped_ht(name: *const c_char, pht: *mut *mut c_void) -> *mut c_void;

    // --- hash ops ---
    fn nvim_vars_hash_find(ht: *mut c_void, key: *const c_char) -> *mut c_void;
    fn nvim_hashitem_empty(hi: *mut c_void) -> c_int;
    fn nvim_hi2dictitem(hi: *mut c_void) -> *mut c_void;

    // --- dictitem accessors ---
    fn nvim_vars_dictitem_get_flags(di: *mut c_void) -> c_int;
    fn nvim_vars_dictitem_get_tv_ptr(di: *mut c_void) -> *mut c_void;
    fn nvim_vars_dictitem_get_key(di: *mut c_void) -> *const c_char;
    fn nvim_dictitem_get_tv_type(di: *const c_void) -> c_int;
    fn nvim_dictitem_set_lock_bit(di: *mut c_void, lock: bool);
    fn nvim_di_flags_fix() -> c_int;
    fn nvim_var_dict() -> c_int;
    fn nvim_var_list() -> c_int;

    // --- var checks (in Rust checks.rs but callable via C name) ---
    fn rs_var_check_fixed(flags: c_int, name: *const c_char, name_len: usize) -> bool;
    fn rs_var_check_ro(flags: c_int, name: *const c_char, name_len: usize) -> bool;
    fn nvim_value_check_lock(lock: c_int, name: *const c_char, name_len: usize) -> bool;
    fn nvim_dict_get_lock(d: *mut c_void) -> c_int;

    // --- list/dict operations ---
    fn nvim_tv_list_locked(l: *const c_void) -> c_int;
    fn nvim_tv_list_item_remove(l: *mut c_void, li: *mut c_void);
    fn nvim_tv_list_remove_items(l: *mut c_void, li_first: *mut c_void, li_last: *mut c_void);
    fn nvim_tv_dict_item_remove(dict: *mut c_void, di: *mut c_void);
    fn nvim_tv_item_lock(tv: *mut c_void, deep: c_int, lock: bool, check_refcount: bool);

    // --- dict watch / watcher notify ---
    fn nvim_tv_dict_is_watched(d: *const c_void) -> bool;
    fn nvim_tv_dict_watcher_notify(
        dict: *mut c_void,
        key: *const c_char,
        newtv: *mut c_void,
        oldtv: *mut c_void,
    );

    // --- typval ops ---
    fn tv_copy(from: *mut c_void, to: *mut c_void);
    fn tv_clear(tv: *mut c_void);

    // --- list item iteration ---
    fn rs_listitem_next(item: *mut c_void) -> *mut c_void;
    fn rs_listitem_tv(item: *mut c_void) -> *mut c_void;

    // --- delete_var wrapper ---
    fn nvim_vars_delete_var(ht: *mut c_void, hi: *mut c_void);

    // --- memory ---
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn xfree(ptr: *mut c_void);

    // --- unsetenv ---
    fn nvim_vim_unsetenv_ext(name_with_dollar: *const c_char);

    // --- error messages ---
    fn semsg(fmt: *const c_char, ...) -> c_int;
}

// sizeof(typval_T) = 24 bytes (for allocating oldtv on stack via vec)
const TYPVAL_SIZE: usize = 24;

/// Unlet a range of list items [n1_arg .. optionally n2].
///
/// Matches C `tv_list_unlet_range`.
///
/// # Safety
/// `l` and `li_first` must be valid. `li_first` must be an item of `l`.
unsafe fn rs_tv_list_unlet_range(
    l: *mut c_void,
    li_first: *mut c_void,
    n1_arg: c_int,
    has_n2: bool,
    n2: c_int,
) {
    let mut li_last = li_first;
    let mut n1 = n1_arg;
    loop {
        let li = rs_listitem_next(li_last);
        n1 += 1;
        if li.is_null() || (has_n2 && n2 < n1) {
            break;
        }
        li_last = li;
    }
    nvim_tv_list_remove_items(l, li_first, li_last);
}

/// Unlet a variable indicated by lval_T (Rust version of `do_unlet_var`).
///
/// # Safety
/// `lp` must be a valid heap-allocated lval_T from `nvim_vars_alloc_lval`.
/// `name_end` must be a valid mutable C string.
/// `eap` must be a valid exarg_T*.
unsafe fn rs_do_unlet_var(lp: *mut c_void, name_end: *mut c_char, eap: *mut c_void) -> c_int {
    let forceit = nvim_eap_get_forceit_int(eap);
    let mut ret = OK;

    if nvim_lval_is_tv_null(lp) {
        let cc = *name_end as u8;
        *name_end = 0; // NUL-terminate at name_end

        let lp_name = nvim_lval_get_name(lp);
        if *lp_name == b'$' as c_char {
            // Environment variable
            nvim_vim_unsetenv_ext(lp_name);
        } else {
            let name_len = nvim_lval_get_name_len(lp);
            if rs_do_unlet(lp_name, name_len, forceit) == FAIL {
                ret = FAIL;
            }
        }

        *name_end = cc as c_char; // restore
    } else {
        let lp_list = nvim_lval_get_list(lp);
        let lp_dict = nvim_lval_get_dict(lp);
        let lp_name = nvim_lval_get_name(lp);
        let lp_name_len = nvim_lval_get_name_len(lp);

        // Check if list or dict is locked
        if (!lp_list.is_null()
            && nvim_value_check_lock(nvim_tv_list_locked(lp_list), lp_name, lp_name_len))
            || (!lp_dict.is_null()
                && nvim_value_check_lock(nvim_dict_get_lock(lp_dict), lp_name, lp_name_len))
        {
            return FAIL;
        }

        if nvim_lval_get_range(lp) {
            // Unlet a range of list items
            let li = nvim_lval_get_li(lp);
            let n1 = nvim_lval_get_n1(lp);
            let empty2 = nvim_lval_get_empty2(lp);
            let n2 = nvim_lval_get_n2(lp);
            rs_tv_list_unlet_range(lp_list, li, n1, !empty2, n2);
        } else if !lp_list.is_null() {
            // Unlet a List item
            nvim_tv_list_item_remove(lp_list, nvim_lval_get_li(lp));
        } else {
            // Unlet a Dict item
            let d = lp_dict;
            let di = nvim_lval_get_di(lp);
            let watched = nvim_tv_dict_is_watched(d);

            let mut oldtv_buf = vec![0u8; TYPVAL_SIZE];
            let oldtv = oldtv_buf.as_mut_ptr() as *mut c_void;

            let key = if watched {
                let tv_ptr = nvim_vars_dictitem_get_tv_ptr(di);
                tv_copy(tv_ptr, oldtv);
                // save key because dictitem_remove will free it
                xstrdup(nvim_vars_dictitem_get_key(di))
            } else {
                std::ptr::null_mut()
            };

            nvim_tv_dict_item_remove(d, di);

            if watched {
                nvim_tv_dict_watcher_notify(d, key, std::ptr::null_mut(), oldtv);
                tv_clear(oldtv);
                xfree(key as *mut c_void);
            }
        }
    }

    ret
}

/// Lock or unlock a variable indicated by lval_T (Rust version of `do_lock_var`).
///
/// # Safety
/// `lp` must be a valid heap-allocated lval_T. `eap` must be valid.
unsafe fn rs_do_lock_var(lp: *mut c_void, eap: *mut c_void, deep: c_int) -> c_int {
    let cmd_lockvar = nvim_cmd_lockvar();
    let lock = nvim_eap_get_cmdidx(eap) == cmd_lockvar;
    let mut ret = OK;

    let di_flags_fix = nvim_di_flags_fix();
    let var_dict_type = nvim_var_dict();
    let var_list_type = nvim_var_list();

    if nvim_lval_is_tv_null(lp) {
        let lp_name = nvim_lval_get_name(lp);
        if *lp_name == b'$' as c_char {
            semsg(
                b"E940: Cannot lock or unlock variable %s\0".as_ptr() as *const c_char,
                lp_name,
            );
            ret = FAIL;
        } else {
            let lp_name_len = nvim_lval_get_name_len(lp);
            let di = nvim_find_var(lp_name, lp_name_len, true);
            if di.is_null() {
                ret = FAIL;
            } else {
                let flags = nvim_vars_dictitem_get_flags(di);
                let tv_type = nvim_dictitem_get_tv_type(di);
                if (flags & di_flags_fix) != 0
                    && tv_type != var_dict_type
                    && tv_type != var_list_type
                {
                    // For historical reasons not given for Lists and Dicts.
                    semsg(
                        b"E940: Cannot lock or unlock variable %s\0".as_ptr() as *const c_char,
                        lp_name,
                    );
                    ret = FAIL;
                } else {
                    nvim_dictitem_set_lock_bit(di, lock);
                    if deep != 0 {
                        let tv_ptr = nvim_vars_dictitem_get_tv_ptr(di);
                        nvim_tv_item_lock(tv_ptr, deep, lock, false);
                    }
                }
            }
        }
    } else if deep == 0 {
        // nothing to do
    } else if nvim_lval_get_range(lp) {
        // (un)lock a range of List items
        let mut li = nvim_lval_get_li(lp);
        while !li.is_null()
            && (nvim_lval_get_empty2(lp) || nvim_lval_get_n2(lp) >= nvim_lval_get_n1(lp))
        {
            let tv = rs_listitem_tv(li);
            nvim_tv_item_lock(tv, deep, lock, false);
            li = rs_listitem_next(li);
            nvim_lval_inc_n1(lp);
        }
    } else if !nvim_lval_get_list(lp).is_null() {
        // (un)lock a List item
        let tv = rs_listitem_tv(nvim_lval_get_li(lp));
        nvim_tv_item_lock(tv, deep, lock, false);
    } else {
        // (un)lock a Dict item
        let di = nvim_lval_get_di(lp);
        let tv = nvim_vars_dictitem_get_tv_ptr(di);
        nvim_tv_item_lock(tv, deep, lock, false);
    }

    ret
}

/// Common parsing for :unlet/:lockvar/:unlockvar (Rust version of `ex_unletlock`).
///
/// # Safety
/// `eap` and `argstart` must be valid. `callback` is called with the lval_T.
unsafe fn rs_ex_unletlock_impl<F>(
    eap: *mut c_void,
    argstart: *mut c_char,
    deep: c_int,
    glv_flags: c_int,
    callback: F,
) where
    F: Fn(*mut c_void, *mut c_char, *mut c_void, c_int) -> c_int,
{
    let mut arg = argstart;
    let mut error = false;
    let skip = nvim_eap_get_skip(eap) != 0;

    let lv = nvim_vars_alloc_lval();

    loop {
        let name_end: *mut c_char;

        if *arg == b'$' as c_char {
            // Environment variable case: set ll_name and ll_tv=NULL manually
            nvim_lval_set_name_and_clear_tv(lv, arg);
            arg = arg.add(1);
            if rs_get_env_len(&mut arg.cast_const()) == 0 {
                semsg(
                    b"E475: Invalid argument: %s\0".as_ptr() as *const c_char,
                    arg.sub(1),
                );
                xfree(lv);
                return;
            }
            if !error && !skip && callback(lv, arg, eap, deep) == FAIL {
                error = true;
            }
            name_end = arg;
        } else {
            // Parse the name and find the end
            name_end = nvim_vars_get_lval_unlet(arg, lv, skip || error, glv_flags);
            if nvim_lval_get_name(lv).is_null() {
                error = true; // error but continue parsing
            }
            if name_end.is_null()
                || (rs_ascii_iswhite(*name_end as c_int) == 0
                    && !nvim_ends_excmd_char(*name_end as c_int))
            {
                if !name_end.is_null() {
                    nvim_emsg_severe_set();
                    semsg(
                        b"E488: Trailing characters: %s\0".as_ptr() as *const c_char,
                        name_end,
                    );
                }
                if !skip && !error {
                    nvim_vars_clear_lval(lv);
                }
                break;
            }

            if !error && !skip && callback(lv, name_end, eap, deep) == FAIL {
                error = true;
            }

            if !skip {
                nvim_vars_clear_lval(lv);
            }
        }

        arg = skipwhite(name_end);
        if nvim_ends_excmd_char(*arg as c_int) {
            break;
        }
    }

    let next = nvim_check_nextcmd(arg);
    nvim_eap_set_nextcmd(eap, next);
    xfree(lv);
}

/// :unlet[!] var1 ... command entry point.
///
/// # Safety
/// `eap` must be a valid `exarg_T*`.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_unlet(eap: *mut c_void) {
    let arg = nvim_eap_get_arg(eap);
    let glv_flags = if nvim_eap_get_forceit_int(eap) != 0 {
        GLV_QUIET
    } else {
        0
    };
    rs_ex_unletlock_impl(eap, arg, 0, glv_flags, |lv, name_end, eap_inner, _deep| {
        rs_do_unlet_var(lv, name_end, eap_inner)
    });
}

/// :lockvar / :unlockvar command entry point.
///
/// # Safety
/// `eap` must be a valid `exarg_T*`.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_lockvar(eap: *mut c_void) {
    let mut arg = nvim_eap_get_arg(eap);
    let deep: c_int = if nvim_eap_get_forceit_int(eap) != 0 {
        -1
    } else if rs_ascii_isdigit(*arg as c_int) != 0 {
        let d = getdigits_int(&mut arg, false, -1);
        arg = skipwhite(arg);
        d
    } else {
        2
    };
    rs_ex_unletlock_impl(eap, arg, deep, 0, |lv, _name_end, eap_inner, dp| {
        rs_do_lock_var(lv, eap_inner, dp)
    });
}

/// Core unlet implementation (no lval_T needed).
///
/// Matches C `do_unlet`. Returns OK (1) or FAIL (0).
///
/// # Safety
/// `name` must be a valid C string of length `name_len`.
#[no_mangle]
pub unsafe extern "C" fn rs_do_unlet(
    name: *const c_char,
    name_len: usize,
    forceit: c_int,
) -> c_int {
    let mut varname: *const c_char = std::ptr::null();
    let mut dict_ptr: *mut c_void = std::ptr::null_mut();
    let mut ht = nvim_vars_find_var_ht_dict(name, name_len, &mut varname, &mut dict_ptr);
    let dict = dict_ptr; // the containing dict

    if !ht.is_null() && !varname.is_null() && *varname != 0 {
        let mut d = get_current_funccal_dict(ht);
        if d.is_null() {
            let globvarht = nvim_get_globvarht();
            let compat_ht = nvim_get_compat_hashtab();
            if ht == globvarht {
                d = get_globvar_dict();
            } else if ht == compat_ht {
                d = get_vimvar_dict();
            } else {
                let di = find_var_in_ht(ht, *name as c_int, b"\0".as_ptr() as *const c_char, 0, 0);
                if !di.is_null() {
                    d = nvim_vars_dictitem_inner_dict(di);
                }
            }
            if d.is_null() {
                internal_error(b"do_unlet()\0".as_ptr() as *const c_char);
                return FAIL;
            }
        }

        let mut hi = nvim_vars_hash_find(ht, varname);
        if nvim_hashitem_empty(hi) != 0 {
            // try scoped lookup - pht must be *mut *mut c_void
            hi = find_hi_in_scoped_ht(name, std::ptr::addr_of_mut!(ht));
        }
        if !hi.is_null() && nvim_hashitem_empty(hi) == 0 {
            let di = nvim_hi2dictitem(hi);
            let flags = nvim_vars_dictitem_get_flags(di);
            let d_lock = nvim_dict_get_lock(d);

            if rs_var_check_fixed(flags, name, TV_CSTRING)
                || rs_var_check_ro(flags, name, TV_CSTRING)
                || nvim_value_check_lock(d_lock, name, TV_CSTRING)
            {
                return FAIL;
            }

            if nvim_value_check_lock(d_lock, name, TV_CSTRING) {
                return FAIL;
            }

            // Allocate oldtv on the heap (24 bytes = sizeof(typval_T))
            let mut oldtv_buf = vec![0u8; TYPVAL_SIZE];
            let oldtv = oldtv_buf.as_mut_ptr() as *mut c_void;

            let watched = nvim_tv_dict_is_watched(dict);
            if watched {
                let tv_ptr = nvim_vars_dictitem_get_tv_ptr(di);
                tv_copy(tv_ptr, oldtv);
            }

            nvim_vars_delete_var(ht, hi);

            if watched {
                nvim_tv_dict_watcher_notify(dict, varname, std::ptr::null_mut(), oldtv);
                tv_clear(oldtv);
            }
            return OK;
        }
    }

    if forceit != 0 {
        return OK;
    }
    semsg(
        b"E108: No such variable: \"%s\"\0".as_ptr() as *const c_char,
        name,
    );
    FAIL
}
