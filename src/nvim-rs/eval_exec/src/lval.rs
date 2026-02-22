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
