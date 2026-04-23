//! Funccal management, ex_delfunction, and helper functions for VimL.
//!
//! Migrated from `src/nvim/eval/userfunc.c` Phase 7.
//! Phase 13: Several impl shims inlined directly.
//! Phase 15: callback_call_retnr migrated.

#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_int, c_void};

extern "C" {
    fn nvim_free_funccal_contents_impl(fc: *mut c_void);
    fn nvim_cleanup_function_call_impl(fc: *mut c_void);
    fn nvim_funccal_unref_impl(fc: *mut c_void, fp: *mut c_void, force: c_int);
    fn nvim_ex_delfunction_impl(eap: *mut c_void);
    // Phase 14: For inlining nvim_user_func_error_impl:
    fn nvim_semsg_not_callable(name: *const c_char);

    // current_funccal access (for inlining remove_funccal and create_funccal)
    fn get_current_funccal() -> *mut c_void;
    fn set_current_funccal(fc: *mut c_void);
    fn nvim_set_current_funccal(fc: *mut c_void);
    fn nvim_fc_get_caller(fc: *mut c_void) -> *mut c_void;
    fn nvim_fc_get_func(fc: *mut c_void) -> *mut c_void;

    // Phase 13: For inlining nvim_free_funccal_impl:
    fn nvim_fc_ufuncs_len(fc: *const c_void) -> c_int;
    fn nvim_fc_ufuncs_item(fc: *const c_void, i: c_int) -> *mut c_void;
    fn nvim_fc_ufuncs_ga_clear(fc: *mut c_void);
    fn nvim_ufunc_get_scoped(fp: *mut c_void) -> *mut c_void;
    fn nvim_ufunc_set_scoped(fp: *mut c_void, fc: *mut c_void);
    fn func_ptr_unref(fp: *mut c_void);
    fn xfree(ptr: *mut c_void);

    // Phase 13: For inlining nvim_emsg_funcname_impl:
    fn nvim_emsg_funcname_mk_snr(name: *const c_char) -> *mut c_char;
    fn nvim_semsg_with_name(errmsg: *const c_char, name: *const c_char);

    // Phase 13: For inlining nvim_save_funccal_impl and nvim_restore_funccal_impl:
    fn nvim_funccal_stack_head_mut() -> *mut c_void;
    fn nvim_set_funccal_stack(entry: *mut c_void);
    fn nvim_fc_entry_set_top(fce: *mut c_void, fc: *mut c_void);
    fn nvim_fc_entry_set_next(fce: *mut c_void, next: *mut c_void);
    fn nvim_funccal_entry_top(fce: *mut c_void) -> *mut c_void;
    fn nvim_funccal_entry_next(fce: *mut c_void) -> *mut c_void;
    fn nvim_iemsg(msg: *const c_char);

    // Phase 13: For inlining nvim_create_funccal_impl:
    fn xcalloc(count: usize, size: usize) -> *mut c_void;
    fn nvim_sizeof_funccall() -> usize;
    fn func_ptr_ref(fp: *mut c_void);
    fn nvim_fc_set_func(fc: *mut c_void, fp: *mut c_void);
    fn nvim_fc_set_rettv(fc: *mut c_void, rettv: *mut c_void);
    fn nvim_fc_set_caller(fc: *mut c_void, caller: *mut c_void);

    // Phase 15: For callback_call_retnr
    fn callback_call(
        callback: *mut c_void,
        argcount: c_int,
        argvars: *mut c_void,
        rettv: *mut c_void,
    ) -> bool;
    fn tv_get_number_chk(tv: *const c_void, denote: *mut c_int) -> i64;
    fn tv_clear(tv: *mut c_void);

    // Phase 16: For call_simple_luafunc and call_simple_func
    fn nvim_tv_set_number(tv: *mut c_void, n: i64);
    fn nlua_typval_call(
        funcname: *const c_char,
        len: usize,
        argvars: *mut c_void,
        argcount: c_int,
        rettv: *mut c_void,
    );
    fn find_func(name: *const c_char) -> *mut c_void;
    fn nvim_ufunc_get_flags(fp: *mut c_void) -> c_int;
    fn nvim_call_user_func_check_simple(
        fp: *mut c_void,
        argvars: *mut c_void,
        rettv: *mut c_void,
    ) -> c_int;
    fn rs_fname_trans_sid(
        name: *const c_char,
        fname_buf: *mut c_char,
        tofree: *mut *mut c_char,
        error: *mut c_int,
    ) -> *mut c_char;
    fn xstrnsave(s: *const c_char, len: usize) -> *mut c_char;

    // Phase 21: For call_user_func_check migration
    fn nvim_ufunc_get_luaref(fp: *mut c_void) -> c_int;
    fn typval_exec_lua_callable(
        lua_cb: c_int,
        argcount: c_int,
        argvars: *mut c_void,
        rettv: *mut c_void,
    ) -> c_int;
    fn call_user_func(
        fp: *mut c_void,
        argcount: c_int,
        argvars: *mut c_void,
        rettv: *mut c_void,
        firstline: i32,
        lastline: i32,
        selfdict: *mut c_void,
    );
    fn check_user_func_argcount(fp: *mut c_void, argcount: c_int) -> c_int;
    fn nvim_funcexe_get_doesrange(fe: *mut c_void) -> *mut bool;
    fn nvim_funcexe_get_firstline(fe: *mut c_void) -> i32;
    fn nvim_funcexe_get_lastline(fe: *mut c_void) -> i32;
}

// =============================================================================
// Constants
// =============================================================================

const OK: c_int = 1;
const FAIL: c_int = 0;
const NOTDONE: c_int = 2;
const FCERR_NONE: c_int = 5;
const FCERR_UNKNOWN_OK: c_int = 0; // FCERR_UNKNOWN: "no error" return from check_user_func_argcount
const FC_DELETED_FLAG: c_int = 0x10;
const FC_LUAREF: c_int = 0x800;
const FC_RANGE: c_int = 0x02;
const FC_DICT: c_int = 0x04;
const FLEN_FIXED: usize = 40;

// =============================================================================
// free_funccal
// =============================================================================
//
// Phase 13: inlined from nvim_free_funccal_impl.

#[no_mangle]
pub unsafe extern "C" fn rs_free_funccal(fc: *mut c_void) {
    let len = unsafe { nvim_fc_ufuncs_len(fc) };
    for i in 0..len {
        let fp = unsafe { nvim_fc_ufuncs_item(fc, i) };
        if !fp.is_null() && unsafe { nvim_ufunc_get_scoped(fp) } == fc {
            unsafe { nvim_ufunc_set_scoped(fp, std::ptr::null_mut()) };
        }
    }
    unsafe { nvim_fc_ufuncs_ga_clear(fc) };
    let func = unsafe { nvim_fc_get_func(fc) };
    unsafe { func_ptr_unref(func) };
    unsafe { xfree(fc) };
}

// =============================================================================
// free_funccal_contents
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn rs_free_funccal_contents(fc: *mut c_void) {
    unsafe { nvim_free_funccal_contents_impl(fc) };
}

// =============================================================================
// cleanup_function_call
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn rs_cleanup_function_call(fc: *mut c_void) {
    unsafe { nvim_cleanup_function_call_impl(fc) };
}

// =============================================================================
// funccal_unref
// =============================================================================

#[no_mangle]
pub unsafe extern "C" fn rs_funccal_unref(fc: *mut c_void, fp: *mut c_void, force: c_int) {
    unsafe { nvim_funccal_unref_impl(fc, fp, force) };
}

// =============================================================================
// create_funccal
// =============================================================================
//
// Phase 13: inlined from nvim_create_funccal_impl.

#[unsafe(export_name = "create_funccal")]
pub unsafe extern "C" fn rs_create_funccal(fp: *mut c_void, rettv: *mut c_void) -> *mut c_void {
    let size = unsafe { nvim_sizeof_funccall() };
    let fc = unsafe { xcalloc(1, size) };
    let caller = unsafe { get_current_funccal() };
    unsafe { nvim_fc_set_caller(fc, caller) };
    unsafe { nvim_set_current_funccal(fc) };
    unsafe { nvim_fc_set_func(fc, fp) };
    unsafe { func_ptr_ref(fp) };
    unsafe { nvim_fc_set_rettv(fc, rettv) };
    fc
}

// =============================================================================
// remove_funccal
// =============================================================================

#[unsafe(export_name = "remove_funccal")]
pub unsafe extern "C" fn rs_remove_funccal() {
    let fc = unsafe { get_current_funccal() };
    let caller = unsafe { nvim_fc_get_caller(fc) };
    unsafe { set_current_funccal(caller) };
    unsafe { rs_free_funccal(fc) };
}

// =============================================================================
// save_funccal
// =============================================================================
//
// Phase 13: inlined from nvim_save_funccal_impl.

#[unsafe(export_name = "save_funccal")]
pub unsafe extern "C" fn rs_save_funccal(entry: *mut c_void) {
    let cur = unsafe { get_current_funccal() };
    let stack = unsafe { nvim_funccal_stack_head_mut() };
    unsafe { nvim_fc_entry_set_top(entry, cur) };
    unsafe { nvim_fc_entry_set_next(entry, stack) };
    unsafe { nvim_set_funccal_stack(entry) };
    unsafe { nvim_set_current_funccal(std::ptr::null_mut()) };
}

// =============================================================================
// restore_funccal
// =============================================================================
//
// Phase 13: inlined from nvim_restore_funccal_impl.

#[unsafe(export_name = "restore_funccal")]
pub unsafe extern "C" fn rs_restore_funccal() {
    let stack = unsafe { nvim_funccal_stack_head_mut() };
    if stack.is_null() {
        unsafe { nvim_iemsg(c"INTERNAL: restore_funccal()".as_ptr()) };
    } else {
        let top = unsafe { nvim_funccal_entry_top(stack) };
        let next = unsafe { nvim_funccal_entry_next(stack) };
        unsafe { nvim_set_current_funccal(top) };
        unsafe { nvim_set_funccal_stack(next) };
    }
}

// =============================================================================
// ex_delfunction
// =============================================================================

#[unsafe(export_name = "ex_delfunction")]
pub unsafe extern "C" fn rs_ex_delfunction(eap: *mut c_void) {
    unsafe { nvim_ex_delfunction_impl(eap) };
}

// =============================================================================
// emsg_funcname
// =============================================================================
//
// Phase 13: inlined from nvim_emsg_funcname_impl.

#[unsafe(export_name = "emsg_funcname")]
pub unsafe extern "C" fn rs_emsg_funcname(errmsg: *const c_char, name: *const c_char) {
    let snr = unsafe { nvim_emsg_funcname_mk_snr(name) };
    let display = if snr.is_null() {
        name
    } else {
        snr.cast_const()
    };
    unsafe { nvim_semsg_with_name(errmsg, display) };
    if !snr.is_null() {
        unsafe { xfree(snr.cast::<c_void>()) };
    }
}

// =============================================================================
// user_func_error
// =============================================================================
//
// Phase 14: inlined from nvim_user_func_error_impl.
// FCERR constants must match userfunc.h

const FCERR_UNKNOWN: c_int = 0;
const FCERR_NOTMETHOD: c_int = 8;
const FCERR_DELETED: c_int = 7;
const FCERR_TOOMANY: c_int = 1;
const FCERR_TOOFEW: c_int = 2;
const FCERR_SCRIPT: c_int = 3;
const FCERR_DICT: c_int = 4;

#[no_mangle]
pub unsafe extern "C" fn rs_user_func_error(error: c_int, name: *const c_char, found_var: c_int) {
    match error {
        FCERR_UNKNOWN => {
            if found_var != 0 {
                unsafe { nvim_semsg_not_callable(name) };
            } else {
                unsafe { rs_emsg_funcname(c"E117: Unknown function: %s".as_ptr(), name) };
            }
        }
        FCERR_NOTMETHOD => unsafe {
            rs_emsg_funcname(c"E276: Cannot use function as a method: %s".as_ptr(), name);
        },
        FCERR_DELETED => unsafe {
            rs_emsg_funcname(c"E933: Function was deleted: %s".as_ptr(), name);
        },
        FCERR_TOOMANY => unsafe {
            rs_emsg_funcname(c"E118: Too many arguments for function: %s".as_ptr(), name);
        },
        FCERR_TOOFEW => unsafe {
            rs_emsg_funcname(
                c"E119: Not enough arguments for function: %s".as_ptr(),
                name,
            );
        },
        FCERR_SCRIPT => unsafe {
            rs_emsg_funcname(
                c"E120: Using <SID> not in a script context: %s".as_ptr(),
                name,
            );
        },
        FCERR_DICT => unsafe {
            rs_emsg_funcname(
                c"E725: Calling dict function without Dictionary: %s".as_ptr(),
                name,
            );
        },
        _ => {}
    }
}

// =============================================================================
// callback_call_retnr
// =============================================================================
//
// Phase 15: Migrated from userfunc.c.

/// Call a callback and return the result as a number.
/// Returns -2 when calling the function fails.
///
/// # Safety
/// `callback` must be a valid `Callback *` pointer.
/// `argvars` must be a valid `typval_T *` array of at least `argcount` + 1 elements.
#[unsafe(export_name = "callback_call_retnr")]
pub unsafe extern "C" fn rs_callback_call_retnr(
    callback: *mut c_void,
    argcount: c_int,
    argvars: *mut c_void,
) -> i64 {
    // typval_T is 16 bytes (i32 v_type, i32 v_lock, 8-byte union vval).
    // Zero-initializing gives VAR_UNKNOWN (v_type = 0), which is safe.
    let mut rettv = [0u8; 16];
    let rettv_ptr = rettv.as_mut_ptr().cast::<c_void>();
    if !unsafe { callback_call(callback, argcount, argvars, rettv_ptr) } {
        return -2;
    }
    let retval = unsafe { tv_get_number_chk(rettv_ptr.cast_const(), std::ptr::null_mut()) };
    unsafe { tv_clear(rettv_ptr) };
    retval
}

// =============================================================================
// call_simple_luafunc
// =============================================================================
//
// Phase 16: Migrated from userfunc.c.

/// Call a Lua function by name without arguments.
///
/// # Safety
/// `funcname` must be a valid string pointer of at least `len` bytes.
/// `rettv` must be a valid `typval_T *`.
#[unsafe(export_name = "call_simple_luafunc")]
pub unsafe extern "C" fn rs_call_simple_luafunc(
    funcname: *const c_char,
    len: usize,
    rettv: *mut c_void,
) -> c_int {
    // Set default rettv to number zero.
    unsafe { nvim_tv_set_number(rettv, 0) };
    // typval_T argvars[1]; argvars[0].v_type = VAR_UNKNOWN (0)
    let mut argvars = [0u8; 16];
    unsafe {
        nlua_typval_call(
            funcname,
            len,
            argvars.as_mut_ptr().cast::<c_void>(),
            0,
            rettv,
        );
    };
    OK
}

// =============================================================================
// call_simple_func
// =============================================================================
//
// Phase 16: Migrated from userfunc.c.

/// Call a VimL function by name without arguments.
/// Returns NOTDONE when the function could not be found.
///
/// # Safety
/// `funcname` must be a valid string pointer of at least `len` bytes.
/// `rettv` must be a valid `typval_T *`.
#[unsafe(export_name = "call_simple_func")]
pub unsafe extern "C" fn rs_call_simple_func(
    funcname: *const c_char,
    len: usize,
    rettv: *mut c_void,
) -> c_int {
    let mut ret = FAIL;

    // Set default rettv to number zero.
    unsafe { nvim_tv_set_number(rettv, 0) };

    // Make a copy of the name, an option can be changed in the function.
    let name = unsafe { xstrnsave(funcname, len) };

    let mut error: c_int = FCERR_NONE;
    let mut tofree: *mut c_char = std::ptr::null_mut();
    let mut fname_buf = [0u8; FLEN_FIXED + 1];
    let fname = unsafe {
        rs_fname_trans_sid(
            name,
            fname_buf.as_mut_ptr().cast::<c_char>(),
            std::ptr::addr_of_mut!(tofree),
            std::ptr::addr_of_mut!(error),
        )
    };

    // Skip "g:" before a function name.
    let is_global = unsafe { *fname.cast::<u8>() == b'g' && *fname.add(1).cast::<u8>() == b':' };
    let rfname = if is_global {
        unsafe { fname.add(2) }
    } else {
        fname
    };

    let fp = unsafe { find_func(rfname) };
    if fp.is_null() {
        ret = NOTDONE;
    } else if unsafe { nvim_ufunc_get_flags(fp) } & FC_DELETED_FLAG != 0 {
        error = FCERR_DELETED;
    } else {
        // typval_T argvars[1]; argvars[0].v_type = VAR_UNKNOWN (0)
        let mut argvars = [0u8; 16];
        let argvars_ptr = argvars.as_mut_ptr().cast::<c_void>();
        error = unsafe { nvim_call_user_func_check_simple(fp, argvars_ptr, rettv) };
        if error == FCERR_NONE {
            ret = OK;
        }
    }

    unsafe { rs_user_func_error(error, name, 0) };
    unsafe { xfree(tofree.cast::<c_void>()) };
    unsafe { xfree(name.cast::<c_void>()) };
    ret
}

// =============================================================================
// call_user_func_check
// =============================================================================
//
// Phase 21: migrated from userfunc.c static function.

/// Call a user function after checking the arguments.
///
/// Returns FCERR_NONE (5) on success, or an FCERR_* error code on failure.
///
/// # Safety
/// `fp`, `argvars`, `rettv`, `funcexe` must be valid non-null pointers.
/// `selfdict` may be null.
#[unsafe(export_name = "call_user_func_check")]
pub unsafe extern "C" fn rs_call_user_func_check(
    fp: *mut c_void,
    argcount: c_int,
    argvars: *mut c_void,
    rettv: *mut c_void,
    funcexe: *mut c_void,
    selfdict: *mut c_void,
) -> c_int {
    let flags = unsafe { nvim_ufunc_get_flags(fp) };

    // Lua function: delegate directly to Lua callable
    if flags & FC_LUAREF != 0 {
        let lua_ref = unsafe { nvim_ufunc_get_luaref(fp) };
        return unsafe { typval_exec_lua_callable(lua_ref, argcount, argvars, rettv) };
    }

    // If function takes a range and caller wants to know, mark it
    if flags & FC_RANGE != 0 {
        let doesrange = unsafe { nvim_funcexe_get_doesrange(funcexe) };
        if !doesrange.is_null() {
            unsafe { *doesrange = true };
        }
    }

    // Validate argument count
    let error = unsafe { check_user_func_argcount(fp, argcount) };
    if error != FCERR_UNKNOWN_OK {
        return error;
    }

    // Dict function requires selfdict
    if flags & FC_DICT != 0 && selfdict.is_null() {
        return FCERR_DICT;
    }

    // Call the user function
    let firstline = unsafe { nvim_funcexe_get_firstline(funcexe) };
    let lastline = unsafe { nvim_funcexe_get_lastline(funcexe) };
    let effective_selfdict = if flags & FC_DICT != 0 {
        selfdict
    } else {
        std::ptr::null_mut()
    };
    unsafe {
        call_user_func(
            fp,
            argcount,
            argvars,
            rettv,
            firstline,
            lastline,
            effective_selfdict,
        );
    };

    FCERR_NONE
}
