//! LValue parsing and manipulation for VimL.
//!
//! This module implements `get_lval`, `clear_lval`, and `set_var_lval`,
//! migrated from `src/nvim/eval_shim.c`.
//!
//! ## LValue Types
//!
//! A VimL lvalue can be:
//! - A plain variable: `name`
//! - A dict item: `dict.key` or `dict['key']`
//! - A list item: `list[expr]`
//! - A list slice: `list[expr:expr]`
//! - A blob index/range: `blob[n]` or `blob[n:m]`

#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(unsafe_op_in_unsafe_fn)]

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Constants
// =============================================================================

/// OK return value from C functions
#[allow(dead_code)]
const OK: c_int = 1;
/// FAIL return value from C functions
#[allow(dead_code)]
const FAIL: c_int = 0;

/// FNE_INCL_BR flag: include brackets in name end search
const FNE_INCL_BR: c_int = 1;

// =============================================================================
// Opaque Handle Types
// =============================================================================

/// Opaque handle to a typval_T
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct TypevalHandle(*mut c_void);

impl TypevalHandle {
    /// Create a null handle
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Check if handle is null
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to a lval_T
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct LvalHandle(*mut c_void);

impl LvalHandle {
    /// Create a null handle
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Check if handle is null
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to a dictitem_T
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct DictitemHandle(*mut c_void);

impl DictitemHandle {
    /// Check if handle is null
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to a hashtab_T
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct HashtabHandle(*mut c_void);

impl HashtabHandle {
    /// Check if handle is null
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// =============================================================================
// C Extern Functions
// =============================================================================

extern "C" {
    // Name end searching (implemented in Rust, called here through C)
    fn rs_find_name_end(
        arg: *const c_char,
        expr_start: *mut *const c_char,
        expr_end: *mut *const c_char,
        flags: c_int,
    ) -> *const c_char;

    // Character predicates
    fn rs_ascii_iswhite(c: c_int) -> c_int;
    fn ends_excmd(c: c_char) -> c_int;

    // Error handling
    fn aborting() -> c_int;

    // Memory
    fn xfree(ptr: *mut c_void);
    fn strlen(s: *const c_char) -> usize;

    // lval_T accessors
    fn nvim_lval_clear(lp: LvalHandle);
    fn nvim_lval_get_name(lp: LvalHandle) -> *const c_char;
    fn nvim_lval_set_name(lp: LvalHandle, name: *const c_char);
    fn nvim_lval_get_name_len(lp: LvalHandle) -> usize;
    fn nvim_lval_set_name_len(lp: LvalHandle, len: usize);
    fn nvim_lval_get_exp_name(lp: LvalHandle) -> *mut c_char;
    fn nvim_lval_set_exp_name(lp: LvalHandle, exp_name: *mut c_char);
    fn nvim_lval_get_tv(lp: LvalHandle) -> TypevalHandle;
    fn nvim_lval_set_tv(lp: LvalHandle, tv: TypevalHandle);
    fn nvim_lval_get_newkey(lp: LvalHandle) -> *mut c_char;
    fn nvim_lval_name_is_null(lp: LvalHandle) -> bool;
    fn nvim_lval_compute_name_len(lp: LvalHandle, p: *const c_char);

    // Error message helpers
    fn nvim_semsg_trailing_arg(p: *const c_char);
    fn nvim_semsg_invarg2(name: *const c_char);
    fn nvim_semsg_undef_var(len: c_int, name: *const c_char);

    // make_expanded_name wrapper
    fn nvim_make_expanded_name(
        in_start: *const c_char,
        expr_start: *mut c_char,
        expr_end: *mut c_char,
        in_end: *mut c_char,
    ) -> *mut c_char;

    // find_var wrapper
    fn nvim_find_var(
        name: *const c_char,
        name_len: usize,
        htp: *mut *mut c_void,
        no_autoload: bool,
    ) -> DictitemHandle;

    // dictitem_T accessor
    fn nvim_di_get_tv(di: DictitemHandle) -> TypevalHandle;

    // tv_is_luafunc wrapper
    fn nvim_tv_is_luafunc_wrapper(tv: TypevalHandle) -> bool;

    // emsg_severe flag (already exists in message.c with int param)
    fn nvim_set_emsg_severe(val: c_int);

    // subscript processing (promoted from static to non-static)
    fn nvim_get_lval_subscript(
        lp: LvalHandle,
        p: *mut c_char,
        name: *mut c_char,
        rettv: TypevalHandle,
        ht: *mut c_void,
        v: DictitemHandle,
        unlet: bool,
        flags: c_int,
    ) -> *mut c_char;
}

// =============================================================================
// rs_get_lval - parse an lvalue expression
// =============================================================================

/// Get an lvalue: parse a variable name (possibly with subscripts).
///
/// Implements the C `get_lval` function from `eval_shim.c`.
///
/// # Safety
/// - `name` must be a valid C string
/// - `rettv` can be null
/// - `lp` must be a valid lval_T pointer
unsafe fn get_lval_impl(
    name: *mut c_char,
    rettv: TypevalHandle,
    lp: LvalHandle,
    unlet: bool,
    skip: bool,
    flags: c_int,
    fne_flags: c_int,
) -> *mut c_char {
    let quiet = (flags & 2) != 0; // GLV_QUIET = TFN_QUIET = 2

    // Clear everything in "lp".
    nvim_lval_clear(lp);

    if skip {
        // When skipping just find the end of the name.
        nvim_lval_set_name(lp, name);
        return rs_find_name_end(
            name,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            FNE_INCL_BR | fne_flags,
        ) as *mut c_char;
    }

    // Find the end of the name.
    let mut expr_start: *const c_char = std::ptr::null();
    let mut expr_end: *const c_char = std::ptr::null();
    let p = rs_find_name_end(name, &mut expr_start, &mut expr_end, fne_flags) as *mut c_char;

    if !expr_start.is_null() {
        // Don't expand the name when we already know there is an error.
        if unlet
            && rs_ascii_iswhite(*p as c_int) == 0
            && ends_excmd(*p) == 0
            && *p != b'[' as c_char
            && *p != b'.' as c_char
        {
            nvim_semsg_trailing_arg(p);
            return std::ptr::null_mut();
        }

        let exp_name =
            nvim_make_expanded_name(name, expr_start as *mut c_char, expr_end as *mut c_char, p);
        nvim_lval_set_exp_name(lp, exp_name);
        nvim_lval_set_name(lp, exp_name);

        if exp_name.is_null() {
            // Report an invalid expression in braces, unless the expression
            // evaluation has been cancelled due to an aborting error,
            // an interrupt, or an exception.
            if aborting() == 0 && !quiet {
                nvim_set_emsg_severe(1);
                nvim_semsg_invarg2(name);
                return std::ptr::null_mut();
            }
            nvim_lval_set_name_len(lp, 0);
        } else {
            nvim_lval_set_name_len(lp, strlen(exp_name));
        }
    } else {
        nvim_lval_set_name(lp, name);
        nvim_lval_set_name_len(lp, (p as usize).wrapping_sub(name as usize));
    }

    // Without [idx] or .key we are done.
    if (*p != b'[' as c_char && *p != b'.' as c_char) || nvim_lval_name_is_null(lp) {
        return p;
    }

    let mut ht: *mut c_void = std::ptr::null_mut();
    let ht_ptr: *mut *mut c_void = if (flags & 16) != 0 {
        // GLV_READ_ONLY = TFN_READ_ONLY = 16: don't pass ht to prevent autoload
        std::ptr::null_mut()
    } else {
        &mut ht
    };

    let no_autoload = (flags & 4) != 0; // GLV_NO_AUTOLOAD = TFN_NO_AUTOLOAD = 4
    let v = nvim_find_var(
        nvim_lval_get_name(lp),
        nvim_lval_get_name_len(lp),
        ht_ptr,
        no_autoload,
    );

    if v.is_null() && !quiet {
        nvim_semsg_undef_var(nvim_lval_get_name_len(lp) as c_int, nvim_lval_get_name(lp));
    }
    if v.is_null() {
        return std::ptr::null_mut();
    }

    nvim_lval_set_tv(lp, nvim_di_get_tv(v));

    if nvim_tv_is_luafunc_wrapper(nvim_lval_get_tv(lp)) {
        // For v:lua just return a pointer to the "." after the "v:lua".
        // If the caller is trans_function_name() it will check for a Lua function name.
        return p;
    }

    // If the next character is a "." or a "[", then process the subitem.
    let p2 = nvim_get_lval_subscript(lp, p, name, rettv, ht, v, unlet, flags);
    if p2.is_null() {
        return std::ptr::null_mut();
    }

    nvim_lval_compute_name_len(lp, p2);
    p2
}

/// FFI export for get_lval.
///
/// # Safety
/// See `get_lval_impl` for safety requirements.
#[no_mangle]
pub unsafe extern "C" fn rs_get_lval(
    name: *mut c_char,
    rettv: TypevalHandle,
    lp: LvalHandle,
    unlet: bool,
    skip: bool,
    flags: c_int,
    fne_flags: c_int,
) -> *mut c_char {
    get_lval_impl(name, rettv, lp, unlet, skip, flags, fne_flags)
}

// =============================================================================
// rs_clear_lval - free lval resources
// =============================================================================

/// Clear lval "lp" that was filled by get_lval().
///
/// Frees the expanded name and newkey if allocated.
///
/// # Safety
/// - `lp` must be a valid lval_T pointer
unsafe fn clear_lval_impl(lp: LvalHandle) {
    let exp_name = nvim_lval_get_exp_name(lp);
    if !exp_name.is_null() {
        xfree(exp_name as *mut c_void);
    }
    let newkey = nvim_lval_get_newkey(lp);
    if !newkey.is_null() {
        xfree(newkey as *mut c_void);
    }
}

/// FFI export for clear_lval.
///
/// # Safety
/// See `clear_lval_impl` for safety requirements.
#[no_mangle]
pub unsafe extern "C" fn rs_clear_lval(lp: LvalHandle) {
    clear_lval_impl(lp)
}

// =============================================================================
// Phase 3: set_var_lval migration
// =============================================================================

extern "C" {
    // lval_T field accessors (Phase 3)
    fn nvim_lval_get_blob(lp: LvalHandle) -> *mut c_void; // blob_T*
    fn nvim_lval_get_range(lp: LvalHandle) -> bool;
    fn nvim_lval_get_empty2(lp: LvalHandle) -> bool;
    fn nvim_lval_get_n1(lp: LvalHandle) -> c_int;
    fn nvim_lval_get_n2(lp: LvalHandle) -> c_int;
    fn nvim_lval_set_n2(lp: LvalHandle, n2: c_int);
    fn nvim_lval_get_list(lp: LvalHandle) -> *mut c_void; // list_T*
    fn nvim_lval_get_dict(lp: LvalHandle) -> *mut c_void; // dict_T*
    fn nvim_lval_get_di(lp: LvalHandle) -> DictitemHandle;
    fn nvim_lval_set_tv_from_di(lp: LvalHandle, di: DictitemHandle);

    // Blob accessors
    fn nvim_blob_get_bv_lock(blob: *const c_void) -> c_int;

    // typval field accessors
    fn nvim_tv_set_v_lock(tv: TypevalHandle, lock: c_int);
    fn nvim_tv_get_v_dict(tv: TypevalHandle) -> *mut c_void; // dict_T*
    fn nvim_tv_get_v_list(tv: TypevalHandle) -> *mut c_void; // list_T*
    fn nvim_tv_assign_direct(dst: TypevalHandle, src: TypevalHandle);
    fn nvim_tv_init(tv: TypevalHandle);
    fn nvim_tv_alloc_zero() -> TypevalHandle;
    fn nvim_tv_free(tv: TypevalHandle);

    // composite lock check for the ll_tv path
    fn nvim_lval_check_tv_lock(lp: LvalHandle, name: *const c_char) -> bool;

    // dictitem_T accessors
    fn nvim_di_get_key(di: DictitemHandle) -> *const c_char;
    fn nvim_di_check_ro(di: DictitemHandle, name: *const c_char) -> bool;
    fn nvim_di_check_lock(di: DictitemHandle, name: *const c_char) -> bool;

    // VAR_UNLOCKED constant
    fn nvim_var_unlocked() -> c_int;

    // Error message helpers (Phase 3)
    fn nvim_emsg_cannot_mod();
    fn nvim_semsg_letwrong(op: *const c_char);
    fn nvim_emsg_cannot_lock_range();
    fn nvim_emsg_cannot_lock_list_or_dict();
    fn nvim_semsg_dictkey(key: *const c_char);
    fn nvim_value_check_lock(lock: c_int, name: *const c_char) -> bool;

    // C functions used by set_var_lval
    fn tv_clear(tv: TypevalHandle);
    fn tv_copy(from: TypevalHandle, to: TypevalHandle);
    fn tv_get_number_chk(tv: TypevalHandle, error: *mut bool) -> i64;
    fn eval_variable(
        name: *const c_char,
        len: c_int,
        rettv: TypevalHandle,
        dip: *mut DictitemHandle,
        verbose: bool,
        no_autoload: bool,
    ) -> c_int;
    fn eexe_mod_op(tv1: TypevalHandle, tv2: TypevalHandle, op: *const c_char) -> c_int;
    fn set_var(name: *const c_char, len: usize, tv: TypevalHandle, copy: bool);
    fn set_var_const(
        name: *const c_char,
        len: usize,
        tv: TypevalHandle,
        copy: bool,
        is_const: bool,
    );
    fn nvim_blob_get_len(blob: *const c_void) -> c_int;
    fn tv_blob_set_range(blob: *mut c_void, n1: c_int, n2: c_int, tv: TypevalHandle) -> c_int;
    fn tv_blob_set_append(blob: *mut c_void, idx: c_int, val: u8);
    fn tv_list_assign_range(
        list: *mut c_void,
        src_list: *mut c_void,
        n1: c_int,
        n2: c_int,
        empty2: bool,
        op: *const c_char,
        name: *const c_char,
    );
    fn nvim_tv_dict_is_watched(dict: *const c_void) -> bool;
    fn tv_dict_wrong_func_name(dict: *mut c_void, tv: TypevalHandle, key: *const c_char) -> c_int;
    fn tv_dict_item_alloc(key: *const c_char) -> DictitemHandle;
    fn tv_dict_add(dict: *mut c_void, di: DictitemHandle) -> c_int;
    fn tv_dict_watcher_notify(
        dict: *mut c_void,
        key: *const c_char,
        newtv: TypevalHandle,
        oldtv: TypevalHandle,
    );
    fn nvim_tv_dict_item_free(di: DictitemHandle);
}

/// VAR_BLOB type constant (must match C enum)
const VAR_BLOB: c_int = 10;

/// Get type from a TypevalHandle using nvim_tv_get_type.
unsafe fn nvim_tv_get_type(tv: TypevalHandle) -> c_int {
    // We declare this locally to avoid conflicts with eval.rs
    extern "C" {
        fn nvim_tv_get_type(tv: TypevalHandle) -> c_int;
    }
    nvim_tv_get_type(tv)
}

/// Implementation of set_var_lval.
///
/// Assigns `rettv` to the lvalue described by `lp`.
///
/// # Safety
/// - `lp` must be a valid lval_T pointer
/// - `endp` must point to just after the parsed name
/// - `rettv` must be a valid typval_T pointer
/// - `op` can be null
unsafe fn set_var_lval_impl(
    lp: LvalHandle,
    endp: *mut c_char,
    rettv: TypevalHandle,
    copy: bool,
    is_const: bool,
    op: *const c_char,
) {
    let lp_tv = nvim_lval_get_tv(lp);

    if lp_tv.is_null() {
        // Plain variable path
        let cc = *endp as u8;
        *endp = 0i8; // NUL

        let blob = nvim_lval_get_blob(lp);
        if !blob.is_null() {
            // Blob assignment
            if !op.is_null() && *op != b'=' as c_char {
                nvim_semsg_letwrong(op);
                *endp = cc as c_char;
                return;
            }
            let bv_lock = nvim_blob_get_bv_lock(blob);
            if nvim_value_check_lock(bv_lock, nvim_lval_get_name(lp)) {
                *endp = cc as c_char;
                return;
            }

            if nvim_lval_get_range(lp) && nvim_tv_get_type(rettv) == VAR_BLOB {
                if nvim_lval_get_empty2(lp) {
                    nvim_lval_set_n2(lp, nvim_blob_get_len(blob) - 1);
                }
                if tv_blob_set_range(blob, nvim_lval_get_n1(lp), nvim_lval_get_n2(lp), rettv)
                    == FAIL
                {
                    *endp = cc as c_char;
                    return;
                }
            } else {
                let mut error = false;
                let val = tv_get_number_chk(rettv, &mut error) as i8;
                if !error {
                    tv_blob_set_append(blob, nvim_lval_get_n1(lp), val as u8);
                }
            }
        } else if !op.is_null() && *op != b'=' as c_char {
            // Operator assignment (+=, -=, etc.)
            if is_const {
                nvim_emsg_cannot_mod();
                *endp = cc as c_char;
                return;
            }

            let tv_tmp = nvim_tv_alloc_zero();
            let mut di_ptr: DictitemHandle = DictitemHandle(std::ptr::null_mut());
            if eval_variable(
                nvim_lval_get_name(lp),
                nvim_lval_get_name_len(lp) as c_int,
                tv_tmp,
                &mut di_ptr,
                true,
                false,
            ) == OK
            {
                let can_modify = di_ptr.is_null()
                    || (!nvim_di_check_ro(di_ptr, nvim_lval_get_name(lp))
                        && !nvim_di_check_lock(di_ptr, nvim_lval_get_name(lp)));
                if can_modify && eexe_mod_op(tv_tmp, rettv, op) == OK {
                    set_var(
                        nvim_lval_get_name(lp),
                        nvim_lval_get_name_len(lp),
                        tv_tmp,
                        false,
                    );
                }
                tv_clear(tv_tmp);
            }
            nvim_tv_free(tv_tmp);
        } else {
            // Simple assignment
            set_var_const(
                nvim_lval_get_name(lp),
                nvim_lval_get_name_len(lp),
                rettv,
                copy,
                is_const,
            );
        }
        *endp = cc as c_char;
    } else if nvim_lval_check_tv_lock(lp, nvim_lval_get_name(lp)) {
        // Locked: skip
    } else if nvim_lval_get_range(lp) {
        // List range assignment
        if is_const {
            nvim_emsg_cannot_lock_range();
            return;
        }
        tv_list_assign_range(
            nvim_lval_get_list(lp),
            nvim_tv_get_v_list(rettv),
            nvim_lval_get_n1(lp),
            nvim_lval_get_n2(lp),
            nvim_lval_get_empty2(lp),
            op,
            nvim_lval_get_name(lp),
        );
    } else {
        // Dict/list item assignment
        if is_const {
            nvim_emsg_cannot_lock_list_or_dict();
            return;
        }

        let dict = nvim_lval_get_dict(lp);
        let watched = nvim_tv_dict_is_watched(dict);
        let oldtv = nvim_tv_alloc_zero();
        let mut skip_assign = false;
        let mut is_new_key = false;

        let newkey = nvim_lval_get_newkey(lp);
        if !newkey.is_null() {
            // New dict key
            is_new_key = true;
            if !op.is_null() && *op != b'=' as c_char {
                nvim_semsg_dictkey(newkey);
                nvim_tv_free(oldtv);
                return;
            }
            if tv_dict_wrong_func_name(nvim_tv_get_v_dict(nvim_lval_get_tv(lp)), rettv, newkey) != 0
            {
                nvim_tv_free(oldtv);
                return;
            }
            // Add item to dict
            let di = tv_dict_item_alloc(newkey);
            if tv_dict_add(nvim_tv_get_v_dict(nvim_lval_get_tv(lp)), di) == FAIL {
                nvim_tv_dict_item_free(di);
                nvim_tv_free(oldtv);
                return;
            }
            nvim_lval_set_tv_from_di(lp, di);
        } else {
            // Existing item
            if watched {
                tv_copy(nvim_lval_get_tv(lp), oldtv);
            }
            if !op.is_null() && *op != b'=' as c_char {
                eexe_mod_op(nvim_lval_get_tv(lp), rettv, op);
                // Equivalent to goto notify - skip normal assign, go to notify block
                skip_assign = true;
            } else {
                tv_clear(nvim_lval_get_tv(lp));
            }
        }

        if !skip_assign {
            // Assign the value
            let lp_tv2 = nvim_lval_get_tv(lp);
            if copy {
                tv_copy(rettv, lp_tv2);
            } else {
                nvim_tv_assign_direct(lp_tv2, rettv);
                nvim_tv_set_v_lock(lp_tv2, nvim_var_unlocked());
                nvim_tv_init(rettv);
            }
        }

        // notify watchers
        if watched {
            if is_new_key || nvim_tv_get_type(oldtv) == 0 {
                // VAR_UNKNOWN (0): new entry
                tv_dict_watcher_notify(
                    dict,
                    nvim_lval_get_newkey(lp),
                    nvim_lval_get_tv(lp),
                    TypevalHandle::null(),
                );
            } else {
                let di = nvim_lval_get_di(lp);
                tv_dict_watcher_notify(dict, nvim_di_get_key(di), nvim_lval_get_tv(lp), oldtv);
                tv_clear(oldtv);
            }
        }
        nvim_tv_free(oldtv);
    }
}

/// FFI export for set_var_lval.
///
/// # Safety
/// See `set_var_lval_impl` for safety requirements.
#[no_mangle]
pub unsafe extern "C" fn rs_set_var_lval(
    lp: LvalHandle,
    endp: *mut c_char,
    rettv: TypevalHandle,
    copy: bool,
    is_const: bool,
    op: *const c_char,
) {
    set_var_lval_impl(lp, endp, rettv, copy, is_const, op)
}
