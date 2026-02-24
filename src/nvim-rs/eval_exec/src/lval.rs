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
#![allow(dead_code)]

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

    // make_expanded_name (Rust implementation in eval/src/names.rs)
    fn rs_make_expanded_name(
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

}

// =============================================================================
// Phase 2: extern "C" declarations for subscript migration accessors
// =============================================================================

extern "C" {
    // String utilities
    fn skipwhite(p: *const c_char) -> *mut c_char;

    // Typval operations (not already in Phase 3 block)
    fn tv_check_str(tv: TypevalHandle) -> bool;
    fn tv_get_number(tv: TypevalHandle) -> i64;
    fn tv_get_string(tv: TypevalHandle) -> *const c_char;

    // Typval type query (get_blob here)
    fn nvim_tv_get_blob(tv: TypevalHandle) -> *mut c_void;

    // eval1 (recursive subscript parsing)
    fn eval1(arg: *mut *mut c_char, rettv: TypevalHandle, evalarg: *mut c_void) -> c_int;

    // ASCII predicates
    fn rs_ascii_isalnum(c: c_int) -> c_int;

    // lval_T setters (Phase 1) - nvim_lval_set_n2 is already in Phase 3 block
    fn nvim_lval_set_list(lp: LvalHandle, list: *mut c_void);
    fn nvim_lval_set_dict(lp: LvalHandle, dict: *mut c_void);
    fn nvim_lval_set_di(lp: LvalHandle, di: *mut c_void);
    fn nvim_lval_set_n1(lp: LvalHandle, n1: c_int);
    fn nvim_lval_set_range(lp: LvalHandle, range: bool);
    fn nvim_lval_set_empty2(lp: LvalHandle, empty2: bool);
    fn nvim_lval_set_blob(lp: LvalHandle, blob: *mut c_void);
    fn nvim_lval_set_li(lp: LvalHandle, li: *mut c_void);
    fn nvim_lval_set_newkey(lp: LvalHandle, key: *mut c_char);

    // lval_T composite accessors (Phase 1)
    #[allow(dead_code)]
    fn nvim_lval_get_li(lp: LvalHandle) -> *mut c_void;
    fn nvim_lval_dict_is_v_or_a_scope(lp: LvalHandle) -> bool;
    fn nvim_lval_dict_scope(lp: LvalHandle) -> c_int;
    fn nvim_lval_di_check_ro_lock(lp: LvalHandle, name: *const c_char, name_len: usize) -> bool;
    fn nvim_lval_set_tv_to_li_tv(lp: LvalHandle);

    // lval_T composite accessors for subscript loop
    fn nvim_lval_tv_get_type(lp: LvalHandle) -> c_int;
    fn nvim_lval_tv_get_list(lp: LvalHandle) -> *mut c_void;
    fn nvim_lval_tv_get_blob(lp: LvalHandle) -> *mut c_void;
    fn nvim_lval_tv_list_alloc_ret(lp: LvalHandle);
    fn nvim_lval_tv_blob_alloc_ret(lp: LvalHandle);
    fn nvim_lval_tv_blob_len(lp: LvalHandle) -> c_int;
    fn nvim_lval_alloc_dict_if_null(lp: LvalHandle);
    fn nvim_lval_di_is_null(lp: LvalHandle) -> bool;
    fn nvim_lval_di_is_luafunc(lp: LvalHandle) -> bool;
    fn nvim_lval_dict_scope_check(
        lp: LvalHandle,
        key: *mut c_char,
        len: c_int,
        rettv: TypevalHandle,
    ) -> bool;
    fn nvim_lval_set_tv_from_ll_di(lp: LvalHandle);

    // dict/list/blob check helpers
    fn nvim_tv_blob_check_index(bloblen: c_int, n1: c_int, quiet: bool) -> c_int;
    fn nvim_tv_blob_check_range(bloblen: c_int, n1: c_int, n2: c_int, quiet: bool) -> c_int;
    fn nvim_tv_list_check_range_index_one(lp: LvalHandle, quiet: bool) -> *mut c_void;
    fn nvim_tv_list_check_range_index_two(lp: LvalHandle, quiet: bool) -> c_int;

    // EVALARG_EVALUATE accessor
    fn nvim_get_evalarg_evaluate_ptr() -> *mut c_void;

    // xstrdup / xmemdupz
    fn nvim_xstrdup(s: *const c_char) -> *mut c_char;
    fn nvim_xmemdupz(src: *const c_char, len: usize) -> *mut c_char;

    // lval dict helpers
    fn nvim_lval_set_di_from_dict(lp: LvalHandle, key: *const c_char, len: c_int);

    // tv_is_func (int return, existing)
    fn nvim_tv_is_func(tv: TypevalHandle) -> c_int;

    // Error message wrappers (Phase 1)
    fn nvim_emsg_e689();
    fn nvim_emsg_e708();
    fn nvim_emsg_e713();
    fn nvim_emsg_e709();
    fn nvim_semsg_e_dot_dict(name: *const c_char);
    fn nvim_semsg_e_illvar_raw(name: *const c_char);
    fn nvim_semsg_e_illvar(name: *const c_char);
    fn nvim_semsg_e_cannot_slice_dict();
    fn nvim_emsg_missbrac();
}

// VarType constants (must match C enum var_type_T)
const VAR_LIST_TYPE: c_int = 4;
const VAR_DICT_TYPE: c_int = 5;
const VAR_BLOB_TYPE: c_int = 10;

// GLV flag constants (must match C GetLvalFlags enum)
const GLV_QUIET: c_int = 2; // TFN_QUIET = 2
const GLV_READ_ONLY: c_int = 16; // TFN_READ_ONLY = 16

/// GlvStatus mirrors the C `glv_status_T` enum for `get_lval_dict_item`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GlvStatus {
    Fail,
    Ok,
    Stop,
}

// =============================================================================
// get_lval_dict_item_impl - resolve dict subscript for an lvalue
// =============================================================================

/// Resolve a dict-key subscript for an lvalue.
///
/// Mirrors the C `get_lval_dict_item` function.
///
/// # Safety
/// All pointer parameters must be valid.
#[allow(clippy::too_many_arguments)]
unsafe fn get_lval_dict_item_impl(
    lp: LvalHandle,
    name: *mut c_char,
    mut key: *mut c_char,
    len: c_int,
    key_end: *mut *mut c_char, // inout: current position in source
    var1: TypevalHandle,
    flags: c_int,
    unlet: bool,
    rettv: TypevalHandle,
) -> GlvStatus {
    let quiet = (flags & GLV_QUIET) != 0;
    let p = *key_end;

    if len == -1 {
        // "[key]": get key from var1
        key = tv_get_string(var1) as *mut c_char;
    }

    nvim_lval_set_list(lp, std::ptr::null_mut());

    // NULL dict => allocate empty dict
    nvim_lval_alloc_dict_if_null(lp);

    // Find key in dict
    nvim_lval_set_di_from_dict(lp, key, len);

    // Scope check: when assigning to scope dict, validate name
    if !rettv.is_null()
        && nvim_lval_dict_scope(lp) != 0
        && nvim_lval_dict_scope_check(lp, key, len, rettv)
    {
        return GlvStatus::Fail;
    }

    // Check if di is a luafunc (v:['lua'] case)
    if !nvim_lval_di_is_null(lp) && nvim_lval_di_is_luafunc(lp) && len == -1 && rettv.is_null() {
        nvim_semsg_e_illvar_raw(c"v:['lua']".as_ptr());
        return GlvStatus::Fail;
    }

    if nvim_lval_di_is_null(lp) {
        // Key not found -- check if we can add it
        if nvim_lval_dict_is_v_or_a_scope(lp) {
            nvim_semsg_e_illvar(name);
            return GlvStatus::Fail;
        }

        // Key doesn't exist: need to add it
        if *p == b'[' as c_char || *p == b'.' as c_char || unlet {
            if !quiet {
                nvim_semsg_dictkey(key);
            }
            return GlvStatus::Fail;
        }

        // Set newkey (duplicated key string)
        let newkey = if len == -1 {
            nvim_xstrdup(key)
        } else {
            nvim_xmemdupz(key, len as usize)
        };
        nvim_lval_set_newkey(lp, newkey);
        *key_end = p;
        return GlvStatus::Stop;
    }

    // Key exists: check read-only / lock flags unless GLV_READ_ONLY
    if (flags & GLV_READ_ONLY) == 0 {
        let p_minus_name = (p as usize).wrapping_sub(name as usize);
        if nvim_lval_di_check_ro_lock(lp, name, p_minus_name) {
            return GlvStatus::Fail;
        }
    }

    nvim_lval_set_tv_from_ll_di(lp);

    GlvStatus::Ok
}

// =============================================================================
// get_lval_blob_impl - resolve blob subscript for an lvalue
// =============================================================================

/// Resolve a blob-index/range subscript for an lvalue.
///
/// Mirrors the C `get_lval_blob` function.
///
/// # Safety
/// All pointer parameters must be valid.
unsafe fn get_lval_blob_impl(
    lp: LvalHandle,
    var1: TypevalHandle,
    var2: TypevalHandle,
    empty1: bool,
    quiet: bool,
) -> c_int {
    let bloblen = nvim_lval_tv_blob_len(lp);

    if empty1 {
        nvim_lval_set_n1(lp, 0);
    } else {
        nvim_lval_set_n1(lp, tv_get_number(var1) as c_int);
    }

    if nvim_tv_blob_check_index(bloblen, nvim_lval_get_n1(lp), quiet) == FAIL {
        return FAIL;
    }

    if nvim_lval_get_range(lp) && !nvim_lval_get_empty2(lp) {
        nvim_lval_set_n2(lp, tv_get_number(var2) as c_int);
        if nvim_tv_blob_check_range(bloblen, nvim_lval_get_n1(lp), nvim_lval_get_n2(lp), quiet)
            == FAIL
        {
            return FAIL;
        }
    }

    nvim_lval_set_blob(lp, nvim_lval_tv_get_blob(lp));
    nvim_lval_set_tv(lp, TypevalHandle::null());

    OK
}

// =============================================================================
// get_lval_list_impl - resolve list subscript for an lvalue
// =============================================================================

/// Resolve a list-index/range subscript for an lvalue.
///
/// Mirrors the C `get_lval_list` function.
///
/// # Safety
/// All pointer parameters must be valid.
unsafe fn get_lval_list_impl(
    lp: LvalHandle,
    var1: TypevalHandle,
    var2: TypevalHandle,
    empty1: bool,
    quiet: bool,
) -> c_int {
    if empty1 {
        nvim_lval_set_n1(lp, 0);
    } else {
        nvim_lval_set_n1(lp, tv_get_number(var1) as c_int);
    }

    nvim_lval_set_dict(lp, std::ptr::null_mut());
    nvim_lval_set_list(lp, nvim_lval_tv_get_list(lp));

    let li = nvim_tv_list_check_range_index_one(lp, quiet);
    if li.is_null() {
        return FAIL;
    }
    nvim_lval_set_li(lp, li);

    if nvim_lval_get_range(lp) && !nvim_lval_get_empty2(lp) {
        nvim_lval_set_n2(lp, tv_get_number(var2) as c_int);
        if nvim_tv_list_check_range_index_two(lp, quiet) == FAIL {
            return FAIL;
        }
    }

    nvim_lval_set_tv_to_li_tv(lp);

    OK
}

// =============================================================================
// get_lval_subscript_impl - main subscript loop
// =============================================================================

/// Get the lval of a list/dict/blob subitem starting at "p". Loop
/// until no more [idx] or .key is following.
///
/// Mirrors the C `nvim_get_lval_subscript` function.
///
/// Returns pointer to character after subscript on success, or NULL on failure.
///
/// # Safety
/// All pointer parameters must be valid.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::never_loop)]
unsafe fn get_lval_subscript_impl(
    lp: LvalHandle,
    mut p: *mut c_char,
    name: *mut c_char,
    rettv: TypevalHandle,
    _ht: *mut c_void,
    _v: DictitemHandle,
    unlet: bool,
    flags: c_int,
) -> *mut c_char {
    let quiet = (flags & GLV_QUIET) != 0;

    // Allocate var1/var2 on the heap (heap-allocated, cleaned up on all paths)
    let var1 = nvim_tv_alloc_zero();
    let var2 = nvim_tv_alloc_zero();
    let mut rc = FAIL;
    let mut empty1 = false;

    // Macro-like helper: jump to 'done' (implemented as goto-via-loop-break)
    'outer: loop {
        // Main loop: while *p == '[' || (*p == '.' && p[1] != '=' && p[1] != '.')
        while *p == b'[' as c_char
            || (*p == b'.' as c_char && *p.add(1) != b'=' as c_char && *p.add(1) != b'.' as c_char)
        {
            if *p == b'.' as c_char && nvim_lval_tv_get_type(lp) != VAR_DICT_TYPE {
                if !quiet {
                    nvim_semsg_e_dot_dict(name);
                }
                break 'outer;
            }

            let tv_type = nvim_lval_tv_get_type(lp);
            if tv_type != VAR_LIST_TYPE && tv_type != VAR_DICT_TYPE && tv_type != VAR_BLOB_TYPE {
                if !quiet {
                    nvim_emsg_e689();
                }
                break 'outer;
            }

            // Allocate null list/blob if needed
            if tv_type == VAR_LIST_TYPE && nvim_lval_tv_get_list(lp).is_null() {
                nvim_lval_tv_list_alloc_ret(lp);
            } else if tv_type == VAR_BLOB_TYPE && nvim_lval_tv_get_blob(lp).is_null() {
                nvim_lval_tv_blob_alloc_ret(lp);
            }

            if nvim_lval_get_range(lp) {
                if !quiet {
                    nvim_emsg_e708();
                }
                break 'outer; // goto done
            }

            let mut len: c_int = -1;
            let mut key: *mut c_char = std::ptr::null_mut();

            if *p == b'.' as c_char {
                key = p.add(1);
                len = 0;
                // Count alphanumeric + underscore characters
                while rs_ascii_isalnum(*key.add(len as usize) as c_int) != 0
                    || *key.add(len as usize) == b'_' as c_char
                {
                    len += 1;
                }
                if len == 0 {
                    if !quiet {
                        nvim_emsg_e713();
                    }
                    // Return NULL immediately (no goto done, var1/var2 still clean)
                    tv_clear(var1);
                    tv_clear(var2);
                    nvim_tv_free(var1);
                    nvim_tv_free(var2);
                    return std::ptr::null_mut();
                }
                p = key.add(len as usize);
            } else {
                // Get the index [expr] or first index [expr:]
                p = skipwhite(p.add(1));
                if *p == b':' as c_char {
                    empty1 = true;
                } else {
                    empty1 = false;
                    let evalarg = nvim_get_evalarg_evaluate_ptr();
                    if eval1(&mut p, var1, evalarg) == FAIL {
                        break 'outer; // goto done
                    }
                    if !tv_check_str(var1) {
                        break 'outer; // goto done
                    }
                    p = skipwhite(p);
                }

                // Optionally get the second index [:expr]
                if *p == b':' as c_char {
                    if nvim_lval_tv_get_type(lp) == VAR_DICT_TYPE {
                        if !quiet {
                            nvim_semsg_e_cannot_slice_dict();
                        }
                        break 'outer; // goto done
                    }
                    // Check rettv type for range
                    if !rettv.is_null() {
                        let rt = nvim_tv_get_type(rettv);
                        let rv_list = nvim_eval_tv_get_list(rettv);
                        let rv_blob = nvim_tv_get_blob(rettv);
                        let ok = (rt == VAR_LIST_TYPE && !rv_list.is_null())
                            || (rt == VAR_BLOB_TYPE && !rv_blob.is_null());
                        if !ok {
                            if !quiet {
                                nvim_emsg_e709();
                            }
                            break 'outer; // goto done
                        }
                    }
                    p = skipwhite(p.add(1));
                    if *p == b']' as c_char {
                        nvim_lval_set_empty2(lp, true);
                    } else {
                        nvim_lval_set_empty2(lp, false);
                        let evalarg = nvim_get_evalarg_evaluate_ptr();
                        if eval1(&mut p, var2, evalarg) == FAIL {
                            break 'outer; // goto done
                        }
                        if !tv_check_str(var2) {
                            break 'outer; // goto done
                        }
                    }
                    nvim_lval_set_range(lp, true);
                } else {
                    nvim_lval_set_range(lp, false);
                }

                if *p != b']' as c_char {
                    if !quiet {
                        nvim_emsg_missbrac();
                    }
                    break 'outer; // goto done
                }

                // Skip past ']'
                p = p.add(1);
            }

            // Handle each collection type
            if nvim_lval_tv_get_type(lp) == VAR_DICT_TYPE {
                let glv =
                    get_lval_dict_item_impl(lp, name, key, len, &mut p, var1, flags, unlet, rettv);
                match glv {
                    GlvStatus::Fail => {
                        break 'outer; // goto done
                    }
                    GlvStatus::Stop => {
                        break; // break the while loop
                    }
                    GlvStatus::Ok => {}
                }
            } else if nvim_lval_tv_get_type(lp) == VAR_BLOB_TYPE {
                if get_lval_blob_impl(lp, var1, var2, empty1, quiet) == FAIL {
                    break 'outer; // goto done
                }
                break; // break the while loop (blob done)
            } else {
                // List
                if get_lval_list_impl(lp, var1, var2, empty1, quiet) == FAIL {
                    break 'outer; // goto done
                }
            }

            // Clear and reset var1/var2 for next iteration
            tv_clear(var1);
            tv_clear(var2);
            // Re-initialize to VAR_UNKNOWN (already zero from tv_clear)
        } // end while

        rc = OK;
        break 'outer; // normal exit
    }

    // Cleanup
    tv_clear(var1);
    tv_clear(var2);
    nvim_tv_free(var1);
    nvim_tv_free(var2);

    if rc == OK {
        p
    } else {
        std::ptr::null_mut()
    }
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
            rs_make_expanded_name(name, expr_start as *mut c_char, expr_end as *mut c_char, p);
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
    let p2 = get_lval_subscript_impl(lp, p, name, rettv, ht, v, unlet, flags);
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
    fn nvim_eval_tv_get_dict(tv: TypevalHandle) -> *mut c_void; // dict_T*
    fn nvim_eval_tv_get_list(tv: TypevalHandle) -> *mut c_void; // list_T*
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
            nvim_eval_tv_get_list(rettv),
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
            if tv_dict_wrong_func_name(nvim_eval_tv_get_dict(nvim_lval_get_tv(lp)), rettv, newkey)
                != 0
            {
                nvim_tv_free(oldtv);
                return;
            }
            // Add item to dict
            let di = tv_dict_item_alloc(newkey);
            if tv_dict_add(nvim_eval_tv_get_dict(nvim_lval_get_tv(lp)), di) == FAIL {
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

// =============================================================================
// Phase 3 (eval_shim pass 4): var_item_copy
// =============================================================================

use std::cell::Cell;

extern "C" {
    // vimconv_T accessor: returns conv->vc_type (0 = CONV_NONE)
    fn nvim_vimconv_get_type(conv: *const c_void) -> c_int;
    // Get tv->vval.v_string (read-only)
    fn nvim_tv_get_vstring_ro(tv: *const c_void) -> *const c_char;
    // string_convert wrapper; returns allocated string or NULL
    fn nvim_string_convert(conv: *const c_void, str: *const c_char) -> *mut c_char;
    // xstrdup
    fn xstrdup(s: *const c_char) -> *mut c_char;
    // Set tv->vval.v_string (takes ownership)
    fn nvim_tv_set_vstring_owned(tv: TypevalHandle, s: *mut c_char);
    // tv type setter
    fn nvim_tv_set_type(tv: TypevalHandle, vtype: c_int);

    // List operations
    fn nvim_tv_list_copyid(list: *const c_void) -> c_int;
    fn nvim_tv_list_latest_copy(list: *const c_void) -> *mut c_void;
    fn nvim_tv_list_ref(list: *mut c_void);
    fn nvim_tv_list_copy(
        conv: *const c_void,
        list: *mut c_void,
        deep: bool,
        copy_id: c_int,
    ) -> *mut c_void;
    fn nvim_tv_set_list(tv: TypevalHandle, list: *mut c_void);

    // Dict operations
    fn nvim_dict_get_copyid(dict: *const c_void) -> c_int;
    fn nvim_dict_get_copydict(dict: *const c_void) -> *mut c_void;
    fn nvim_dict_refcount_inc(dict: *mut c_void);
    fn nvim_tv_dict_copy(
        conv: *const c_void,
        dict: *mut c_void,
        deep: bool,
        copy_id: c_int,
    ) -> *mut c_void;
    fn nvim_tv_set_dict(tv: TypevalHandle, dict: *mut c_void);

    // Blob operations
    fn nvim_tv_blob_copy(from_blob: *mut c_void, to: TypevalHandle);

    // Error: variable nested too deep
    fn nvim_emsg_nested_too_deep();
    // internal_error
    fn internal_error(where_: *const c_char);
}

/// VAR_UNKNOWN type constant (for var_item_copy)
const VAR_UNKNOWN: c_int = 0;
/// VAR_NUMBER type constant (for var_item_copy)
const VAR_NUMBER: c_int = 1;
/// VAR_STRING type constant (for var_item_copy)
const VAR_STRING: c_int = 2;
/// VAR_FUNC type constant (for var_item_copy)
const VAR_FUNC: c_int = 3;
/// VAR_LIST type constant (for var_item_copy)
const VAR_LIST: c_int = 4;
/// VAR_DICT type constant (for var_item_copy)
const VAR_DICT: c_int = 5;
/// VAR_FLOAT type constant (for var_item_copy)
const VAR_FLOAT: c_int = 6;
/// VAR_BOOL type constant (for var_item_copy)
const VAR_BOOL: c_int = 7;
/// VAR_SPECIAL type constant (for var_item_copy)
const VAR_SPECIAL: c_int = 8;
/// VAR_PARTIAL type constant (for var_item_copy)
const VAR_PARTIAL: c_int = 9;
// Note: VAR_BLOB (10) is already defined above at the module level

/// VAR_UNLOCKED constant (must match C enum)
const VAR_UNLOCKED: c_int = 0;
/// CONV_NONE constant (must match C)
const CONV_NONE: c_int = 0;
/// Maximum nesting depth for lists/dicts in copies
const DICT_MAXNEST: c_int = 100;

thread_local! {
    /// Recursion counter for var_item_copy - prevents infinite recursion
    /// on pathological nested structures.
    static VAR_ITEM_COPY_RECURSE: Cell<c_int> = const { Cell::new(0) };
}

/// Deep copy a VimL value (var_item_copy).
///
/// # Safety
/// - `from` and `to` must be valid non-null typval handles
/// - `conv` may be null (no encoding conversion)
///
/// # C equivalent
/// Replaces the C `var_item_copy` function in eval_shim.c.
unsafe fn var_item_copy_impl(
    conv: *const c_void,
    from: TypevalHandle,
    to: TypevalHandle,
    deep: bool,
    copy_id: c_int,
) -> c_int {
    let recurse = VAR_ITEM_COPY_RECURSE.with(|r| r.get());

    if recurse >= DICT_MAXNEST {
        nvim_emsg_nested_too_deep();
        return FAIL;
    }

    VAR_ITEM_COPY_RECURSE.with(|r| r.set(recurse + 1));

    let vtype = nvim_tv_get_type(from);
    let mut ret = OK;

    match vtype {
        VAR_NUMBER | VAR_FLOAT | VAR_FUNC | VAR_PARTIAL | VAR_BOOL | VAR_SPECIAL => {
            tv_copy(from, to);
        }
        VAR_STRING => {
            let from_str = nvim_tv_get_vstring_ro(from.0.cast_const());
            let conv_type = nvim_vimconv_get_type(conv);
            if conv.is_null() || conv_type == CONV_NONE || from_str.is_null() {
                tv_copy(from, to);
            } else {
                nvim_tv_set_type(to, VAR_STRING);
                nvim_tv_set_v_lock(to, VAR_UNLOCKED);
                let converted = nvim_string_convert(conv, from_str);
                let s = if converted.is_null() {
                    xstrdup(from_str)
                } else {
                    converted
                };
                nvim_tv_set_vstring_owned(to, s);
            }
        }
        VAR_LIST => {
            nvim_tv_set_type(to, VAR_LIST);
            nvim_tv_set_v_lock(to, VAR_UNLOCKED);
            let from_list = nvim_eval_tv_get_list(from);
            if from_list.is_null() {
                nvim_tv_set_list(to, std::ptr::null_mut());
            } else if copy_id != 0 && nvim_tv_list_copyid(from_list) == copy_id {
                // Use the copy made earlier.
                let existing_copy = nvim_tv_list_latest_copy(from_list);
                nvim_tv_list_ref(existing_copy);
                nvim_tv_set_list(to, existing_copy);
            } else {
                let new_list = nvim_tv_list_copy(conv, from_list, deep, copy_id);
                nvim_tv_set_list(to, new_list);
            }
            let to_list = nvim_eval_tv_get_list(to);
            if to_list.is_null() && !nvim_eval_tv_get_list(from).is_null() {
                ret = FAIL;
            }
        }
        VAR_BLOB => {
            let from_blob = nvim_tv_get_blob(from);
            nvim_tv_blob_copy(from_blob, to);
        }
        VAR_DICT => {
            nvim_tv_set_type(to, VAR_DICT);
            nvim_tv_set_v_lock(to, VAR_UNLOCKED);
            let from_dict = nvim_eval_tv_get_dict(from);
            if from_dict.is_null() {
                nvim_tv_set_dict(to, std::ptr::null_mut());
            } else if copy_id != 0 && nvim_dict_get_copyid(from_dict) == copy_id {
                // Use the copy made earlier.
                let existing_copy = nvim_dict_get_copydict(from_dict);
                nvim_dict_refcount_inc(existing_copy);
                nvim_tv_set_dict(to, existing_copy);
            } else {
                let new_dict = nvim_tv_dict_copy(conv, from_dict, deep, copy_id);
                nvim_tv_set_dict(to, new_dict);
            }
            let to_dict = nvim_eval_tv_get_dict(to);
            if to_dict.is_null() && !nvim_eval_tv_get_dict(from).is_null() {
                ret = FAIL;
            }
        }
        _ => {
            static MSG: &[u8] = b"var_item_copy(UNKNOWN)\0";
            internal_error(MSG.as_ptr() as *const c_char);
            ret = FAIL;
        }
    }

    VAR_ITEM_COPY_RECURSE.with(|r| r.set(recurse));
    ret
}

/// FFI export for var_item_copy.
///
/// # Safety
/// - `from` and `to` must be valid non-null typval handles
/// - `conv` may be null
#[no_mangle]
pub unsafe extern "C" fn rs_var_item_copy(
    conv: *const c_void,
    from: TypevalHandle,
    to: TypevalHandle,
    deep: bool,
    copy_id: c_int,
) -> c_int {
    var_item_copy_impl(conv, from, to, deep, copy_id)
}
