//! Variable validation and check functions for VimL.
//!
//! Phase 1: Migrated from `src/nvim/eval/vars.c`.
//!
//! Functions:
//! - `rs_var_check_ro`: check if variable is read-only, emit error
//! - `rs_var_check_lock`: check if variable is locked, emit error
//! - `rs_var_check_fixed`: check if variable is fixed (cannot delete), emit error
//! - `rs_var_wrong_func_name`: validate funcref variable names
//! - `rs_valid_varname`: validate variable name characters

#![allow(unsafe_op_in_unsafe_fn)]

use std::ffi::{c_char, c_int};

// =============================================================================
// DI_FLAGS constants (matching C's DI_FLAGS_* in typval_defs.h)
// =============================================================================

/// DI_FLAGS_RO: read-only value
const DI_FLAGS_RO: c_int = 1;
/// DI_FLAGS_RO_SBX: read-only in sandbox
const DI_FLAGS_RO_SBX: c_int = 2;
/// DI_FLAGS_FIX: fixed value (cannot be :unlet or remove()d)
const DI_FLAGS_FIX: c_int = 4;
/// DI_FLAGS_LOCK: locked value
const DI_FLAGS_LOCK: c_int = 8;

/// TV_TRANSLATE sentinel: use gettext on the name
const TV_TRANSLATE: usize = usize::MAX;
/// TV_CSTRING sentinel: compute length with strlen
const TV_CSTRING: usize = usize::MAX - 1;

// AUTOLOAD_CHAR: '#' separates autoload function components
const AUTOLOAD_CHAR: u8 = b'#';

// =============================================================================
// C extern declarations
// =============================================================================

extern "C" {
    /// semsg: print formatted error message
    fn semsg(fmt: *const c_char, ...) -> c_int;

    /// gettext: translate string (C's `_()` macro)
    fn gettext(s: *const c_char) -> *const c_char;

    /// strlen: compute string length
    fn strlen(s: *const c_char) -> usize;

    /// vim_strchr: find character in string (returns NULL or pointer)
    fn vim_strchr(string: *const c_char, c: c_int) -> *mut c_char;

    /// function_exists: check if a function exists
    fn function_exists(name: *const c_char, no_deref: bool) -> bool;

    /// rs_eval_isnamec1: check if char can start a variable name (Rust export)
    fn rs_eval_isnamec1(c: c_int) -> bool;

    /// sandbox: global sandbox level (non-zero when in sandbox)
    static sandbox: c_int;

    /// e_readonlyvar: "E46: Cannot change read-only variable \"%.*s\""
    static e_readonlyvar: c_char;

    /// e_illvar: "E461: Illegal variable name: %s"
    static e_illvar: c_char;
}

// =============================================================================
// Error message strings (copied byte-for-byte for gettext key identity)
// =============================================================================

// "E794: Cannot set variable in the sandbox: \"%.*s\""
const E_SANDBOX: &std::ffi::CStr = c"E794: Cannot set variable in the sandbox: \"%.*s\"";

// "E1122: Variable is locked: %*s"
const E_LOCKED: &std::ffi::CStr = c"E1122: Variable is locked: %*s";

// "E795: Cannot delete variable %.*s"
const E_FIXED: &std::ffi::CStr = c"E795: Cannot delete variable %.*s";

// "E704: Funcref variable name must start with a capital: %s"
const E_FUNCREF_NAME: &std::ffi::CStr =
    c"E704: Funcref variable name must start with a capital: %s";

// "E705: Variable name conflicts with existing function: %s"
const E_FUNC_CONFLICT: &std::ffi::CStr =
    c"E705: Variable name conflicts with existing function: %s";

// =============================================================================
// Implementation
// =============================================================================

/// Resolve name and name_len, handling TV_TRANSLATE and TV_CSTRING sentinels.
///
/// Returns (resolved_name_ptr, resolved_len).
///
/// # Safety
/// `name` must be a valid C string.
unsafe fn resolve_name(name: *const c_char, name_len: usize) -> (*const c_char, usize) {
    if name_len == TV_TRANSLATE {
        let translated = gettext(name);
        let len = strlen(translated);
        (translated, len)
    } else if name_len == TV_CSTRING {
        let len = strlen(name);
        (name, len)
    } else {
        (name, name_len)
    }
}

/// Check if variable is read-only; emit error and return true if so.
///
/// Matches C `var_check_ro`.
///
/// # Safety
/// `name` must be a valid C string (null-terminated). `name_len` may be
/// `TV_TRANSLATE`, `TV_CSTRING`, or an actual byte length.
#[no_mangle]
pub unsafe extern "C" fn rs_var_check_ro(
    flags: c_int,
    name: *const c_char,
    name_len: usize,
) -> bool {
    let error_message: *const c_char;

    if flags & DI_FLAGS_RO != 0 {
        error_message = gettext(&raw const e_readonlyvar);
    } else if (flags & DI_FLAGS_RO_SBX != 0) && sandbox != 0 {
        error_message = E_SANDBOX.as_ptr();
    } else {
        return false;
    }

    let (resolved_name, resolved_len) = resolve_name(name, name_len);
    semsg(error_message, resolved_len as c_int, resolved_name);
    true
}

/// Check if variable is locked; emit error and return true if so.
///
/// Matches C `var_check_lock`.
///
/// # Safety
/// `name` must be a valid C string. `name_len` may be `TV_TRANSLATE`,
/// `TV_CSTRING`, or an actual length.
#[no_mangle]
pub unsafe extern "C" fn rs_var_check_lock(
    flags: c_int,
    name: *const c_char,
    name_len: usize,
) -> bool {
    if flags & DI_FLAGS_LOCK == 0 {
        return false;
    }

    let (resolved_name, resolved_len) = resolve_name(name, name_len);
    semsg(E_LOCKED.as_ptr(), resolved_len as c_int, resolved_name);
    true
}

/// Check if variable is fixed (cannot be deleted); emit error and return true if so.
///
/// Matches C `var_check_fixed`.
///
/// # Safety
/// `name` must be a valid C string. `name_len` may be `TV_TRANSLATE`,
/// `TV_CSTRING`, or an actual length.
#[no_mangle]
pub unsafe extern "C" fn rs_var_check_fixed(
    flags: c_int,
    name: *const c_char,
    name_len: usize,
) -> bool {
    if flags & DI_FLAGS_FIX == 0 {
        return false;
    }

    let (resolved_name, resolved_len) = resolve_name(name, name_len);
    semsg(E_FIXED.as_ptr(), resolved_len as c_int, resolved_name);
    true
}

/// Check if name is valid for assigning a funcref.
///
/// Matches C `var_wrong_func_name`. Returns true if the name is *wrong*
/// (invalid), false if it is valid.
///
/// # Safety
/// `name` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_var_wrong_func_name(name: *const c_char, new_var: bool) -> bool {
    let first = *name as u8;
    let second = *name.add(1) as u8;

    // Allow w:, b:, s:, t: prefixes
    let has_scope_prefix = matches!(first, b'w' | b'b' | b's' | b't') && second == b':';

    // The char to check for uppercase: skip over the "x:" prefix if present
    let check_char: u8 = if has_scope_prefix {
        *name.add(2) as u8
    } else {
        first
    };

    // Check for autoload '#' character anywhere in the name
    let has_autoload = !vim_strchr(name, c_int::from(AUTOLOAD_CHAR)).is_null();

    if !has_scope_prefix && !check_char.is_ascii_uppercase() && !has_autoload {
        semsg(E_FUNCREF_NAME.as_ptr(), name);
        return true;
    }

    // Don't allow hiding a function.
    if new_var && function_exists(name, false) {
        semsg(E_FUNC_CONFLICT.as_ptr(), name);
        return true;
    }

    false
}

/// Check if a variable name is valid.
///
/// Matches C `valid_varname`. Returns true if valid, false otherwise (also
/// emits an error).
///
/// # Safety
/// `varname` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_valid_varname(varname: *const c_char) -> bool {
    let mut p = varname;
    loop {
        let c = *p as u8;
        if c == 0 {
            break;
        }
        let c_int_val = c_int::from(c);
        let is_digit = c.is_ascii_digit();
        if !rs_eval_isnamec1(c_int_val) && (p == varname || !is_digit) && c != AUTOLOAD_CHAR {
            semsg(&raw const e_illvar, varname);
            return false;
        }
        p = p.add(1);
    }
    true
}
