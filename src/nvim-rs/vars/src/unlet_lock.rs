//! Unlet and lockvar functions for VimL.
//!
//! Phase 4: Migrated from `src/nvim/eval/vars.c`.
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

    // --- ex_unletlock wrappers ---
    fn nvim_ex_unletlock_unlet(eap: *mut c_void, arg: *mut c_char, deep: c_int, glv_flags: c_int);
    fn nvim_ex_unletlock_lock(eap: *mut c_void, arg: *mut c_char, deep: c_int);

    // --- char ops ---
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn rs_ascii_isdigit(c: c_int) -> c_int;
    fn getdigits_int(pp: *mut *mut c_char, strict: bool, def: c_int) -> c_int;

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
    fn nvim_vars_dictitem_inner_dict(di: *mut c_void) -> *mut c_void;
    fn internal_error(where_: *const c_char);
    fn find_hi_in_scoped_ht(name: *const c_char, pht: *mut *mut c_void) -> *mut c_void;

    // --- hash ops ---
    fn nvim_vars_hash_find(ht: *mut c_void, key: *const c_char) -> *mut c_void;
    fn nvim_hashitem_empty(hi: *mut c_void) -> c_int;
    fn nvim_hi2dictitem(hi: *mut c_void) -> *mut c_void;

    // --- dictitem flags ---
    fn nvim_vars_dictitem_get_flags(di: *mut c_void) -> c_int;
    fn nvim_vars_dictitem_get_tv_ptr(di: *mut c_void) -> *mut c_void;

    // --- var checks (in Rust checks.rs but callable via C name) ---
    fn rs_var_check_fixed(flags: c_int, name: *const c_char, name_len: usize) -> bool;
    fn rs_var_check_ro(flags: c_int, name: *const c_char, name_len: usize) -> bool;
    fn nvim_value_check_lock(lock: c_int, name: *const c_char, name_len: usize) -> bool;
    fn nvim_dict_get_lock(d: *mut c_void) -> c_int;

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

    // --- delete_var wrapper ---
    fn nvim_vars_delete_var(ht: *mut c_void, hi: *mut c_void);

    // --- error messages ---
    fn semsg(fmt: *const c_char, ...) -> c_int;
}

// sizeof(typval_T) = 24 bytes (for allocating oldtv on stack via vec)
const TYPVAL_SIZE: usize = 24;

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
    nvim_ex_unletlock_unlet(eap, arg, 0, glv_flags);
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
    nvim_ex_unletlock_lock(eap, arg, deep);
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
