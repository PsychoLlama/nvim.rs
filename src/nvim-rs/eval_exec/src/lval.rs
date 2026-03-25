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

use nvim_eval::typval::TypvalT as TypvalTRepr;

use crate::eval::EvalargHandle;

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
// LvalT: Rust-side repr(C) mirror of C's lval_T
// =============================================================================

/// Rust mirror of the C `lval_T` struct.
///
/// Fields must match exactly (order and types) the C definition in `src/nvim/eval.h`.
/// The `_Static_assert` checks in `eval_shim.c` verify layout at compile time.
///
/// # C definition (eval.h lines 52-68):
/// ```c
/// typedef struct {
///   const char *ll_name;     // offset 0
///   size_t ll_name_len;      // offset 8
///   char *ll_exp_name;       // offset 16
///   typval_T *ll_tv;         // offset 24
///   listitem_T *ll_li;       // offset 32
///   list_T *ll_list;         // offset 40
///   bool ll_range;           // offset 48
///   bool ll_empty2;          // offset 49
///   // 2 bytes padding
///   int ll_n1;               // offset 52
///   int ll_n2;               // offset 56
///   // 4 bytes padding
///   dict_T *ll_dict;         // offset 64
///   dictitem_T *ll_di;       // offset 72
///   char *ll_newkey;         // offset 80
///   blob_T *ll_blob;         // offset 88
/// } lval_T;                  // sizeof = 96
/// ```
#[repr(C)]
pub struct LvalT {
    /// Start of variable name (can be NULL).
    pub ll_name: *const c_char,
    /// Length of the ll_name.
    pub ll_name_len: usize,
    /// NULL or expanded name in allocated memory.
    pub ll_exp_name: *mut c_char,
    /// Typeval of item being used. If "newkey" isn't NULL it's the Dict to add the item to.
    pub ll_tv: *mut c_void, // typval_T*
    /// The list item or NULL.
    pub ll_li: *mut c_void, // listitem_T*
    /// The list or NULL.
    pub ll_list: *mut c_void, // list_T*
    /// true when a [i:j] range was used.
    pub ll_range: bool,
    /// Second index is empty: [i:].
    pub ll_empty2: bool,
    /// First index for list.
    pub ll_n1: c_int,
    /// Second index for list range.
    pub ll_n2: c_int,
    /// The Dict or NULL.
    pub ll_dict: *mut c_void, // dict_T*
    /// The dictitem or NULL.
    pub ll_di: *mut c_void, // dictitem_T*
    /// New key for Dict in allocated memory or NULL.
    pub ll_newkey: *mut c_char,
    /// The Blob or NULL.
    pub ll_blob: *mut c_void, // blob_T*
}

// Inline helper: convert raw pointer to TypevalHandle
fn tv(ptr: *mut c_void) -> TypevalHandle {
    TypevalHandle(ptr)
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

    // Error message helpers: trailing_arg and undef_var now in nvim_eval::errors
    // nvim_semsg_invarg2 remains in match.c (not in eval_shim.c)
    fn nvim_semsg_invarg2(name: *const c_char);

    // make_expanded_name (Rust implementation in eval/src/names.rs)
    fn rs_make_expanded_name(
        in_start: *const c_char,
        expr_start: *mut c_char,
        expr_end: *mut c_char,
        in_end: *mut c_char,
    ) -> *mut c_char;

    // find_var wrapper
    #[link_name = "find_var"]
    fn nvim_find_var(
        name: *const c_char,
        name_len: usize,
        htp: *mut *mut c_void,
        no_autoload: bool,
    ) -> DictitemHandle;

    // dictitem_T accessor
    fn nvim_di_get_tv(di: DictitemHandle) -> TypevalHandle;

    // is_luafunc check (from eval crate, via C export)
    fn rs_is_luafunc(pt: *const c_void) -> bool;

    // emsg_severe flag (already exists in message.c with int param)
    static mut emsg_severe: bool;

    // Typval type query
    fn nvim_tv_get_type(tv: TypevalHandle) -> c_int;

    // Typval type query -- eval version (same signature)
    fn nvim_eval_tv_get_type(tv: TypevalHandle) -> c_int;

    // Typval blob accessors
    fn nvim_tv_get_blob(tv: TypevalHandle) -> *mut c_void;
    fn nvim_tv_blob_len(tv: TypevalHandle) -> c_int;
}

// =============================================================================
// Phase 2: extern "C" declarations for subscript migration accessors
// =============================================================================

extern "C" {
    // String utilities
    fn skipwhite(p: *const c_char) -> *mut c_char;

    // Typval operations
    fn tv_check_str(tv: TypevalHandle) -> bool;
    fn tv_get_number(tv: TypevalHandle) -> i64;
    fn tv_get_string(tv: TypevalHandle) -> *const c_char;

    // eval1 (recursive subscript parsing)
    fn eval1(arg: *mut *mut c_char, rettv: TypevalHandle, evalarg: EvalargHandle) -> c_int;

    // ASCII predicates
    fn rs_ascii_isalnum(c: c_int) -> c_int;

    // dict/list/blob check helpers (take raw pointers via void*)
    fn nvim_tv_blob_check_index(bloblen: c_int, n1: c_int, quiet: bool) -> c_int;
    fn nvim_tv_blob_check_range(bloblen: c_int, n1: c_int, n2: c_int, quiet: bool) -> c_int;
    fn nvim_tv_list_check_range_index_one(lp: *mut c_void, quiet: bool) -> *mut c_void;
    fn nvim_tv_list_check_range_index_two(lp: *mut c_void, quiet: bool) -> c_int;

    // EVALARG_EVALUATE accessor
    fn nvim_get_evalarg_evaluate_ptr() -> EvalargHandle;

    // xstrdup / xmemdupz
    fn nvim_xstrdup(s: *const c_char) -> *mut c_char;
    fn nvim_xmemdupz(src: *const c_char, len: usize) -> *mut c_char;

    // tv_is_func (int return, existing)
    fn nvim_tv_is_func(tv: TypevalHandle) -> c_int;

    // Error message wrappers (Phase 1): now in nvim_eval::errors
}

// Composite C accessors needed for operations on nested structs.
// These access ll_tv->vval.* or ll_dict->* etc., which are C struct internals
// not accessible directly from Rust without replicating more struct layouts.
extern "C" {
    // Alloc dict in ll_tv if null; return dict pointer.
    // Takes lval_T* (as *mut c_void for Rust FFI).
    fn nvim_lval_alloc_dict_if_null(lp: *mut c_void);

    // tv_list_alloc_ret on ll_tv; sets ll_tv->v_type = VAR_LIST, allocates list.
    fn nvim_lval_tv_list_alloc_ret(lp: *mut c_void);
    // tv_blob_alloc_ret on ll_tv; sets ll_tv->v_type = VAR_BLOB, allocates blob.
    fn nvim_lval_tv_blob_alloc_ret(lp: *mut c_void);

    // Get ll_tv->vval.v_blob (blob_T*) for a lval_T*.
    fn nvim_lval_tv_get_blob(lp: *const c_void) -> *mut c_void;
    // Get ll_tv->vval.v_list (list_T*) for a lval_T*.
    fn nvim_lval_tv_get_list(lp: *const c_void) -> *mut c_void;
    // Get ll_tv->v_type for a lval_T*.
    fn nvim_lval_tv_get_type(lp: *const c_void) -> c_int;
    // Get tv_blob_len(ll_tv->vval.v_blob) for a lval_T*.
    fn nvim_lval_tv_blob_len(lp: *const c_void) -> c_int;

    // Scope checks on ll_dict:
    fn nvim_lval_dict_is_v_or_a_scope(lp: *const c_void) -> bool;
    fn nvim_lval_dict_scope(lp: *const c_void) -> c_int;
    fn nvim_lval_dict_scope_check(
        lp: *mut c_void,
        key: *mut c_char,
        len: c_int,
        rettv: TypevalHandle,
    ) -> bool;

    // Dict find: lp->ll_di = tv_dict_find(lp->ll_dict, key, len)
    fn nvim_lval_set_di_from_dict(lp: *mut c_void, key: *const c_char, len: c_int);
    // Composite: var_check_ro || var_check_lock on ll_di->di_flags
    fn nvim_lval_di_check_ro_lock(lp: *const c_void, name: *const c_char, name_len: usize) -> bool;

    // Set lp->ll_tv = &lp->ll_di->di_tv
    fn nvim_lval_set_tv_from_ll_di(lp: *mut c_void);
    // Set lp->ll_tv = TV_LIST_ITEM_TV(lp->ll_li)
    fn nvim_lval_set_tv_to_li_tv(lp: *mut c_void);

    // Check ll_di is NULL
    fn nvim_lval_di_is_null(lp: *const c_void) -> bool;

    // Set lp->ll_tv = &di->di_tv  (di is dictitem_T*)
    fn nvim_lval_set_tv_from_di(lp: *mut c_void, di: DictitemHandle);

    // Composite lock check for ll_tv path
    fn nvim_lval_check_tv_lock(lp: *const c_void, name: *const c_char) -> bool;
}

// VarType constants (must match C enum var_type_T)
const VAR_LIST_TYPE: c_int = 4;
const VAR_DICT_TYPE: c_int = 5;
const VAR_BLOB_TYPE: c_int = 10;
const VAR_PARTIAL_TYPE: c_int = 9;

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
    lp: *mut LvalT,
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

    (*lp).ll_list = std::ptr::null_mut();

    // NULL dict => allocate empty dict and set ll_dict
    nvim_lval_alloc_dict_if_null(lp as *mut c_void);

    // Find key in dict: sets lp->ll_di
    nvim_lval_set_di_from_dict(lp as *mut c_void, key, len);

    // Scope check: when assigning to scope dict, validate name
    if !rettv.is_null()
        && nvim_lval_dict_scope(lp as *const c_void) != 0
        && nvim_lval_dict_scope_check(lp as *mut c_void, key, len, rettv)
    {
        return GlvStatus::Fail;
    }

    // Check if di is a luafunc (v:['lua'] case)
    let lval_di_is_luafunc = || -> bool {
        if nvim_lval_di_is_null(lp as *const c_void) {
            return false;
        }
        // Get ll_di's di_tv
        let di = DictitemHandle((*lp).ll_di);
        let di_tv = nvim_di_get_tv(di);
        nvim_tv_get_type(di_tv) == VAR_PARTIAL_TYPE
            && rs_is_luafunc((*di_tv.0.cast::<TypvalTRepr>()).vval.v_partial)
    };

    if !nvim_lval_di_is_null(lp as *const c_void)
        && lval_di_is_luafunc()
        && len == -1
        && rettv.is_null()
    {
        nvim_eval::errors::semsg_e_illvar_raw(c"v:['lua']".as_ptr());
        return GlvStatus::Fail;
    }

    if nvim_lval_di_is_null(lp as *const c_void) {
        // Key not found -- check if we can add it
        if nvim_lval_dict_is_v_or_a_scope(lp as *const c_void) {
            nvim_eval::errors::semsg_e_illvar(name);
            return GlvStatus::Fail;
        }

        // Key doesn't exist: need to add it
        if *p == b'[' as c_char || *p == b'.' as c_char || unlet {
            if !quiet {
                nvim_eval::errors::semsg_dictkey(key);
            }
            return GlvStatus::Fail;
        }

        // Set newkey (duplicated key string)
        let newkey = if len == -1 {
            nvim_xstrdup(key)
        } else {
            nvim_xmemdupz(key, len as usize)
        };
        (*lp).ll_newkey = newkey;
        *key_end = p;
        return GlvStatus::Stop;
    }

    // Key exists: check read-only / lock flags unless GLV_READ_ONLY
    if (flags & GLV_READ_ONLY) == 0 {
        let p_minus_name = (p as usize).wrapping_sub(name as usize);
        if nvim_lval_di_check_ro_lock(lp as *const c_void, name, p_minus_name) {
            return GlvStatus::Fail;
        }
    }

    // lp->ll_tv = &lp->ll_di->di_tv
    nvim_lval_set_tv_from_ll_di(lp as *mut c_void);

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
    lp: *mut LvalT,
    var1: TypevalHandle,
    var2: TypevalHandle,
    empty1: bool,
    quiet: bool,
) -> c_int {
    let bloblen = nvim_lval_tv_blob_len(lp as *const c_void);

    if empty1 {
        (*lp).ll_n1 = 0;
    } else {
        (*lp).ll_n1 = tv_get_number(var1) as c_int;
    }

    if nvim_tv_blob_check_index(bloblen, (*lp).ll_n1, quiet) == FAIL {
        return FAIL;
    }

    if (*lp).ll_range && !(*lp).ll_empty2 {
        (*lp).ll_n2 = tv_get_number(var2) as c_int;
        if nvim_tv_blob_check_range(bloblen, (*lp).ll_n1, (*lp).ll_n2, quiet) == FAIL {
            return FAIL;
        }
    }

    (*lp).ll_blob = nvim_lval_tv_get_blob(lp as *const c_void);
    (*lp).ll_tv = std::ptr::null_mut();

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
    lp: *mut LvalT,
    var1: TypevalHandle,
    var2: TypevalHandle,
    empty1: bool,
    quiet: bool,
) -> c_int {
    if empty1 {
        (*lp).ll_n1 = 0;
    } else {
        (*lp).ll_n1 = tv_get_number(var1) as c_int;
    }

    (*lp).ll_dict = std::ptr::null_mut();
    (*lp).ll_list = nvim_lval_tv_get_list(lp as *const c_void);

    let li = nvim_tv_list_check_range_index_one(lp as *mut c_void, quiet);
    if li.is_null() {
        return FAIL;
    }
    (*lp).ll_li = li;

    if (*lp).ll_range && !(*lp).ll_empty2 {
        (*lp).ll_n2 = tv_get_number(var2) as c_int;
        if nvim_tv_list_check_range_index_two(lp as *mut c_void, quiet) == FAIL {
            return FAIL;
        }
    }

    // lp->ll_tv = TV_LIST_ITEM_TV(lp->ll_li)
    nvim_lval_set_tv_to_li_tv(lp as *mut c_void);

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
    lp: *mut LvalT,
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
            if *p == b'.' as c_char && nvim_lval_tv_get_type(lp as *const c_void) != VAR_DICT_TYPE {
                if !quiet {
                    nvim_eval::errors::semsg_e_dot_dict(name);
                }
                break 'outer;
            }

            let tv_type = nvim_lval_tv_get_type(lp as *const c_void);
            if tv_type != VAR_LIST_TYPE && tv_type != VAR_DICT_TYPE && tv_type != VAR_BLOB_TYPE {
                if !quiet {
                    nvim_eval::errors::emsg_e689();
                }
                break 'outer;
            }

            // Allocate null list/blob if needed
            if tv_type == VAR_LIST_TYPE && nvim_lval_tv_get_list(lp as *const c_void).is_null() {
                nvim_lval_tv_list_alloc_ret(lp as *mut c_void);
            } else if tv_type == VAR_BLOB_TYPE
                && nvim_lval_tv_get_blob(lp as *const c_void).is_null()
            {
                nvim_lval_tv_blob_alloc_ret(lp as *mut c_void);
            }

            if (*lp).ll_range {
                if !quiet {
                    nvim_eval::errors::emsg_e708();
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
                        nvim_eval::errors::emsg_e713();
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
                    if nvim_lval_tv_get_type(lp as *const c_void) == VAR_DICT_TYPE {
                        if !quiet {
                            nvim_eval::errors::emsg_cannot_slice_dict();
                        }
                        break 'outer; // goto done
                    }
                    // Check rettv type for range
                    if !rettv.is_null() {
                        let rt = nvim_eval_tv_get_type(rettv);
                        let rv_list = (*rettv.0.cast::<TypvalTRepr>()).vval.v_list;
                        let rv_blob = nvim_tv_get_blob(rettv);
                        let ok = (rt == VAR_LIST_TYPE && !rv_list.is_null())
                            || (rt == VAR_BLOB_TYPE && !rv_blob.is_null());
                        if !ok {
                            if !quiet {
                                nvim_eval::errors::emsg_e709();
                            }
                            break 'outer; // goto done
                        }
                    }
                    p = skipwhite(p.add(1));
                    if *p == b']' as c_char {
                        (*lp).ll_empty2 = true;
                    } else {
                        (*lp).ll_empty2 = false;
                        let evalarg = nvim_get_evalarg_evaluate_ptr();
                        if eval1(&mut p, var2, evalarg) == FAIL {
                            break 'outer; // goto done
                        }
                        if !tv_check_str(var2) {
                            break 'outer; // goto done
                        }
                    }
                    (*lp).ll_range = true;
                } else {
                    (*lp).ll_range = false;
                }

                if *p != b']' as c_char {
                    if !quiet {
                        nvim_eval::errors::emsg_missbrac();
                    }
                    break 'outer; // goto done
                }

                // Skip past ']'
                p = p.add(1);
            }

            // Handle each collection type
            if nvim_lval_tv_get_type(lp as *const c_void) == VAR_DICT_TYPE {
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
            } else if nvim_lval_tv_get_type(lp as *const c_void) == VAR_BLOB_TYPE {
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
    lp: *mut LvalT,
    unlet: bool,
    skip: bool,
    flags: c_int,
    fne_flags: c_int,
) -> *mut c_char {
    let quiet = (flags & 2) != 0; // GLV_QUIET = TFN_QUIET = 2

    // Clear everything in "lp".
    std::ptr::write_bytes(lp, 0, 1);

    if skip {
        // When skipping just find the end of the name.
        (*lp).ll_name = name;
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
            nvim_eval::errors::semsg_trailing_arg(p);
            return std::ptr::null_mut();
        }

        let exp_name =
            rs_make_expanded_name(name, expr_start as *mut c_char, expr_end as *mut c_char, p);
        (*lp).ll_exp_name = exp_name;
        (*lp).ll_name = exp_name;

        if exp_name.is_null() {
            // Report an invalid expression in braces, unless the expression
            // evaluation has been cancelled due to an aborting error,
            // an interrupt, or an exception.
            if aborting() == 0 && !quiet {
                emsg_severe = (1) != 0;
                nvim_semsg_invarg2(name);
                return std::ptr::null_mut();
            }
            (*lp).ll_name_len = 0;
        } else {
            (*lp).ll_name_len = strlen(exp_name);
        }
    } else {
        (*lp).ll_name = name;
        (*lp).ll_name_len = (p as usize).wrapping_sub(name as usize);
    }

    // Without [idx] or .key we are done.
    if (*lp).ll_name.is_null() || (*p != b'[' as c_char && *p != b'.' as c_char) {
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
    let v = nvim_find_var((*lp).ll_name, (*lp).ll_name_len, ht_ptr, no_autoload);

    if v.is_null() && !quiet {
        nvim_eval::errors::semsg_undef_var((*lp).ll_name_len as c_int, (*lp).ll_name);
    }
    if v.is_null() {
        return std::ptr::null_mut();
    }

    let di_tv = nvim_di_get_tv(v);
    (*lp).ll_tv = di_tv.0;

    if nvim_tv_get_type(tv((*lp).ll_tv)) == VAR_PARTIAL_TYPE
        && rs_is_luafunc((*(*lp).ll_tv.cast::<TypvalTRepr>()).vval.v_partial)
    {
        // For v:lua just return a pointer to the "." after the "v:lua".
        // If the caller is trans_function_name() it will check for a Lua function name.
        return p;
    }

    // If the next character is a "." or a "[", then process the subitem.
    let p2 = get_lval_subscript_impl(lp, p, name, rettv, ht, v, unlet, flags);
    if p2.is_null() {
        return std::ptr::null_mut();
    }

    // lp->ll_name_len = p2 - lp->ll_name
    (*lp).ll_name_len = (p2 as usize).wrapping_sub((*lp).ll_name as usize);
    p2
}

/// FFI export for get_lval.
///
/// # Safety
/// See `get_lval_impl` for safety requirements.
#[export_name = "get_lval"]
pub unsafe extern "C" fn rs_get_lval(
    name: *mut c_char,
    rettv: TypevalHandle,
    lp: *mut LvalT,
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
unsafe fn clear_lval_impl(lp: *mut LvalT) {
    if !(*lp).ll_exp_name.is_null() {
        xfree((*lp).ll_exp_name as *mut c_void);
    }
    if !(*lp).ll_newkey.is_null() {
        xfree((*lp).ll_newkey as *mut c_void);
    }
}

/// FFI export for clear_lval.
///
/// # Safety
/// See `clear_lval_impl` for safety requirements.
#[export_name = "clear_lval"]
pub unsafe extern "C" fn rs_clear_lval(lp: *mut LvalT) {
    clear_lval_impl(lp)
}

// =============================================================================
// Phase 3: set_var_lval migration
// =============================================================================

extern "C" {
    // Blob accessors
    fn nvim_blob_get_bv_lock(blob: *const c_void) -> c_int;

    // typval field accessors

    fn nvim_tv_assign_direct(dst: TypevalHandle, src: TypevalHandle);
    fn nvim_tv_init(tv: TypevalHandle);
    fn nvim_tv_alloc_zero() -> TypevalHandle;
    fn nvim_tv_free(tv: TypevalHandle);

    // dictitem_T accessors
    fn nvim_di_get_key(di: DictitemHandle) -> *const c_char;
    fn nvim_di_check_ro(di: DictitemHandle, name: *const c_char) -> bool;
    fn nvim_di_check_lock(di: DictitemHandle, name: *const c_char) -> bool;

    // VAR_UNLOCKED constant
    fn nvim_var_unlocked() -> c_int;

    // Error message helpers (Phase 3): now in nvim_eval::errors
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
unsafe fn get_tv_type(tv_h: TypevalHandle) -> c_int {
    nvim_tv_get_type(tv_h)
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
    lp: *mut LvalT,
    endp: *mut c_char,
    rettv: TypevalHandle,
    copy: bool,
    is_const: bool,
    op: *const c_char,
) {
    let lp_tv = tv((*lp).ll_tv);

    if lp_tv.is_null() {
        // Plain variable path
        let cc = *endp as u8;
        *endp = 0i8; // NUL

        let blob = (*lp).ll_blob;
        if !blob.is_null() {
            // Blob assignment
            if !op.is_null() && *op != b'=' as c_char {
                nvim_eval::errors::semsg_letwrong(op);
                *endp = cc as c_char;
                return;
            }
            let bv_lock = nvim_blob_get_bv_lock(blob);
            if nvim_value_check_lock(bv_lock, (*lp).ll_name) {
                *endp = cc as c_char;
                return;
            }

            if (*lp).ll_range && get_tv_type(rettv) == VAR_BLOB {
                if (*lp).ll_empty2 {
                    (*lp).ll_n2 = nvim_blob_get_len(blob) - 1;
                }
                if tv_blob_set_range(blob, (*lp).ll_n1, (*lp).ll_n2, rettv) == FAIL {
                    *endp = cc as c_char;
                    return;
                }
            } else {
                let mut error = false;
                let val = tv_get_number_chk(rettv, &mut error) as i8;
                if !error {
                    tv_blob_set_append(blob, (*lp).ll_n1, val as u8);
                }
            }
        } else if !op.is_null() && *op != b'=' as c_char {
            // Operator assignment (+=, -=, etc.)
            if is_const {
                nvim_eval::errors::emsg_cannot_mod();
                *endp = cc as c_char;
                return;
            }

            let tv_tmp = nvim_tv_alloc_zero();
            let mut di_ptr: DictitemHandle = DictitemHandle(std::ptr::null_mut());
            if eval_variable(
                (*lp).ll_name,
                (*lp).ll_name_len as c_int,
                tv_tmp,
                &mut di_ptr,
                true,
                false,
            ) == OK
            {
                let can_modify = di_ptr.is_null()
                    || (!nvim_di_check_ro(di_ptr, (*lp).ll_name)
                        && !nvim_di_check_lock(di_ptr, (*lp).ll_name));
                if can_modify && eexe_mod_op(tv_tmp, rettv, op) == OK {
                    set_var((*lp).ll_name, (*lp).ll_name_len, tv_tmp, false);
                }
                tv_clear(tv_tmp);
            }
            nvim_tv_free(tv_tmp);
        } else {
            // Simple assignment
            set_var_const((*lp).ll_name, (*lp).ll_name_len, rettv, copy, is_const);
        }
        *endp = cc as c_char;
    } else if nvim_lval_check_tv_lock(lp as *const c_void, (*lp).ll_name) {
        // Locked: skip
    } else if (*lp).ll_range {
        // List range assignment
        if is_const {
            nvim_eval::errors::emsg_cannot_lock_range();
            return;
        }
        tv_list_assign_range(
            (*lp).ll_list,
            (*rettv.0.cast::<TypvalTRepr>()).vval.v_list,
            (*lp).ll_n1,
            (*lp).ll_n2,
            (*lp).ll_empty2,
            op,
            (*lp).ll_name,
        );
    } else {
        // Dict/list item assignment
        if is_const {
            nvim_eval::errors::emsg_cannot_lock_list_or_dict();
            return;
        }

        let dict = (*lp).ll_dict;
        let watched = nvim_tv_dict_is_watched(dict);
        let oldtv = nvim_tv_alloc_zero();
        let mut skip_assign = false;
        let mut is_new_key = false;

        let newkey = (*lp).ll_newkey;
        if !newkey.is_null() {
            // New dict key
            is_new_key = true;
            if !op.is_null() && *op != b'=' as c_char {
                nvim_eval::errors::semsg_dictkey(newkey);
                nvim_tv_free(oldtv);
                return;
            }
            let lp_tv_h = tv((*lp).ll_tv);
            if tv_dict_wrong_func_name(
                (*lp_tv_h.0.cast::<TypvalTRepr>()).vval.v_dict,
                rettv,
                newkey,
            ) != 0
            {
                nvim_tv_free(oldtv);
                return;
            }
            // Add item to dict
            let di = tv_dict_item_alloc(newkey);
            let lp_tv_h = tv((*lp).ll_tv);
            if tv_dict_add((*lp_tv_h.0.cast::<TypvalTRepr>()).vval.v_dict, di) == FAIL {
                nvim_tv_dict_item_free(di);
                nvim_tv_free(oldtv);
                return;
            }
            // lp->ll_tv = &di->di_tv
            nvim_lval_set_tv_from_di(lp as *mut c_void, di);
        } else {
            // Existing item
            let lp_tv_h = tv((*lp).ll_tv);
            if watched {
                tv_copy(lp_tv_h, oldtv);
            }
            if !op.is_null() && *op != b'=' as c_char {
                eexe_mod_op(lp_tv_h, rettv, op);
                // Equivalent to goto notify - skip normal assign, go to notify block
                skip_assign = true;
            } else {
                tv_clear(lp_tv_h);
            }
        }

        if !skip_assign {
            // Assign the value
            let lp_tv2 = tv((*lp).ll_tv);
            if copy {
                tv_copy(rettv, lp_tv2);
            } else {
                nvim_tv_assign_direct(lp_tv2, rettv);
                (*lp_tv2.0.cast::<TypvalTRepr>()).v_lock = nvim_var_unlocked();
                nvim_tv_init(rettv);
            }
        }

        // notify watchers
        if watched {
            if is_new_key || get_tv_type(oldtv) == 0 {
                // VAR_UNKNOWN (0): new entry
                tv_dict_watcher_notify(
                    dict,
                    (*lp).ll_newkey,
                    tv((*lp).ll_tv),
                    TypevalHandle::null(),
                );
            } else {
                let di = DictitemHandle((*lp).ll_di);
                tv_dict_watcher_notify(dict, nvim_di_get_key(di), tv((*lp).ll_tv), oldtv);
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
#[export_name = "set_var_lval"]
pub unsafe extern "C" fn rs_set_var_lval(
    lp: *mut LvalT,
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
    // Get tv->vval.v_string (canonical name; cast to *const c_char for read-only use)
    fn nvim_tv_get_vstring(tv: TypevalHandle) -> *mut c_char;
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
    #[link_name = "tv_list_copy"]
    fn nvim_tv_list_copy(
        conv: *const c_void,
        list: *mut c_void,
        deep: bool,
        copy_id: c_int,
    ) -> *mut c_void;

    // Dict operations
    fn nvim_dict_get_copyid(dict: *const c_void) -> c_int;
    fn nvim_dict_get_copydict(dict: *const c_void) -> *mut c_void;
    fn nvim_dict_refcount_inc(dict: *mut c_void);
    #[link_name = "tv_dict_copy"]
    fn nvim_tv_dict_copy(
        conv: *const c_void,
        dict: *mut c_void,
        deep: bool,
        copy_id: c_int,
    ) -> *mut c_void;

    // Blob operations
    #[link_name = "tv_blob_copy"]
    fn nvim_tv_blob_copy(from_blob: *mut c_void, to: TypevalHandle);

    // Error: variable nested too deep -- now in nvim_eval::errors
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
        nvim_eval::errors::emsg_nested_too_deep();
        return FAIL;
    }

    VAR_ITEM_COPY_RECURSE.with(|r| r.set(recurse + 1));

    let vtype = get_tv_type(from);
    let mut ret = OK;

    match vtype {
        VAR_NUMBER | VAR_FLOAT | VAR_FUNC | VAR_PARTIAL | VAR_BOOL | VAR_SPECIAL => {
            tv_copy(from, to);
        }
        VAR_STRING => {
            let from_str = nvim_tv_get_vstring(from) as *const c_char;
            let conv_type = nvim_vimconv_get_type(conv);
            if conv.is_null() || conv_type == CONV_NONE || from_str.is_null() {
                tv_copy(from, to);
            } else {
                nvim_tv_set_type(to, VAR_STRING);
                (*to.0.cast::<TypvalTRepr>()).v_lock = VAR_UNLOCKED;
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
            (*to.0.cast::<TypvalTRepr>()).v_lock = VAR_UNLOCKED;
            let from_list = (*from.0.cast::<TypvalTRepr>()).vval.v_list;
            if from_list.is_null() {
                (*to.0.cast::<TypvalTRepr>()).vval.v_list = std::ptr::null_mut();
            } else if copy_id != 0 && nvim_tv_list_copyid(from_list) == copy_id {
                // Use the copy made earlier.
                let existing_copy = nvim_tv_list_latest_copy(from_list);
                nvim_tv_list_ref(existing_copy);
                (*to.0.cast::<TypvalTRepr>()).vval.v_list = existing_copy;
            } else {
                let new_list = nvim_tv_list_copy(conv, from_list, deep, copy_id);
                (*to.0.cast::<TypvalTRepr>()).vval.v_list = new_list;
            }
            let to_list = (*to.0.cast::<TypvalTRepr>()).vval.v_list;
            if to_list.is_null() && !(*from.0.cast::<TypvalTRepr>()).vval.v_list.is_null() {
                ret = FAIL;
            }
        }
        VAR_BLOB => {
            let from_blob = nvim_tv_get_blob(from);
            nvim_tv_blob_copy(from_blob, to);
        }
        VAR_DICT => {
            nvim_tv_set_type(to, VAR_DICT);
            (*to.0.cast::<TypvalTRepr>()).v_lock = VAR_UNLOCKED;
            let from_dict = (*from.0.cast::<TypvalTRepr>()).vval.v_dict;
            if from_dict.is_null() {
                (*to.0.cast::<TypvalTRepr>()).vval.v_dict = std::ptr::null_mut();
            } else if copy_id != 0 && nvim_dict_get_copyid(from_dict) == copy_id {
                // Use the copy made earlier.
                let existing_copy = nvim_dict_get_copydict(from_dict);
                nvim_dict_refcount_inc(existing_copy);
                (*to.0.cast::<TypvalTRepr>()).vval.v_dict = existing_copy;
            } else {
                let new_dict = nvim_tv_dict_copy(conv, from_dict, deep, copy_id);
                (*to.0.cast::<TypvalTRepr>()).vval.v_dict = new_dict;
            }
            let to_dict = (*to.0.cast::<TypvalTRepr>()).vval.v_dict;
            if to_dict.is_null() && !(*from.0.cast::<TypvalTRepr>()).vval.v_dict.is_null() {
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
#[export_name = "var_item_copy"]
pub unsafe extern "C" fn rs_var_item_copy(
    conv: *const c_void,
    from: TypevalHandle,
    to: TypevalHandle,
    deep: bool,
    copy_id: c_int,
) -> c_int {
    var_item_copy_impl(conv, from, to, deep, copy_id)
}
