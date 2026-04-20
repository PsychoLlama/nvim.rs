//! Option index and metadata access
//!
//! This module provides Rust implementations for option index lookup,
//! validation, and metadata access functions.

use std::ffi::{c_char, c_int, c_uint};

use crate::opt_index::{K_OPT_FOLDMETHOD, K_OPT_WRAP};
use crate::{win_ref, SetPrefix, FAIL, OK};

// =============================================================================
// Type Definitions
// =============================================================================

/// Option index type (matches OptIndex in C which is a signed integer enum)
pub type OptIndex = c_int;

/// Invalid option index
pub const OPT_INVALID: OptIndex = -1;

/// Opaque handle to a window (win_T*)
pub type WinHandle = *mut std::ffi::c_void;

// =============================================================================
// Option Set Flags (from option.h)
// =============================================================================

/// Flags for option-setting functions
pub mod opt_flags {
    use std::ffi::c_int;

    /// Use global value
    pub const OPT_GLOBAL: c_int = 0x01;
    /// Use local value
    pub const OPT_LOCAL: c_int = 0x02;
    /// Option in modeline
    pub const OPT_MODELINE: c_int = 0x04;
    /// Only set window-local options
    pub const OPT_WINONLY: c_int = 0x08;
    /// Don't set window-local options
    pub const OPT_NOWIN: c_int = 0x10;
}

/// Option flags from option_defs.h
pub mod option_flags {
    use std::ffi::c_uint;

    /// Cannot change in modeline or secure mode
    pub const SECURE: c_uint = 1 << 14;
    /// Not allowed in modeline
    pub const NO_ML: c_uint = 1 << 20;
    /// Under control of 'modelineexpr'
    pub const MLE: c_uint = 1 << 24;
}

// =============================================================================
// C External Functions
// =============================================================================

extern "C" {
    static mut p_mle: c_int;
}

extern "C" {
    // Option lookup (direct hash — avoids circular delegation through find_option_len/find_option)
    fn nvim_find_option_len_hash(name: *const c_char, len: usize) -> OptIndex;

    // C shim functions (thin wrappers that call Rust, needed for legacy callers)
    fn option_has_type(opt_idx: OptIndex, val_type: c_int) -> c_int;
    #[link_name = "rs_option_is_window_local"]
    fn nvim_option_is_window_local(opt_idx: OptIndex) -> c_int;

    // Field accessors for vimoption_T (Phase 8 metadata)
    fn nvim_get_option_type(opt_idx: OptIndex) -> c_int;
    fn nvim_get_option_scope_flags(opt_idx: OptIndex) -> c_int;
    fn nvim_get_option_scope_idx(opt_idx: OptIndex, scope: c_int) -> c_int;
    fn nvim_get_option_immutable(opt_idx: OptIndex) -> c_int;
    fn nvim_get_option_def_val_data_ptr(opt_idx: OptIndex) -> *const std::ffi::c_void;
    fn nvim_get_option_script_ctx_ptr(opt_idx: OptIndex) -> *mut std::ffi::c_void;
    fn nvim_get_option_var(opt_idx: OptIndex) -> *mut std::ffi::c_void;
    fn nvim_get_option_flags(opt_idx: OptIndex) -> c_uint;
    fn nvim_option_get_flags_ptr(opt_idx: OptIndex) -> *mut c_uint;

    // TTY option handling
    fn rs_find_tty_option_end(arg: *const c_char) -> *const c_char;
    fn rs_is_tty_option(name: *const c_char) -> c_int;

    // sandbox global variable
    fn nvim_get_sandbox() -> c_int;

}

// =============================================================================
// Internal metadata helpers (Phase 8)
// =============================================================================

/// Check if a u32 value is a power of two (returns false for 0).
#[inline]
fn is_power_of_two(v: u32) -> bool {
    v != 0 && (v & v.wrapping_sub(1)) == 0
}

// =============================================================================
// Accessor function declarations (to be added to option.c)
// =============================================================================

// Note: nvim_get_sandbox needs to be added to option.c
// For now we'll use a shim

// =============================================================================
// Option Index Lookup
// =============================================================================

/// Measure the length of a C string without a libc dependency.
unsafe fn libc_strlen(s: *const c_char) -> usize {
    let mut len = 0usize;
    let mut p = s;
    while *p != 0 {
        len += 1;
        p = p.add(1);
    }
    len
}

/// Find option index by name.
///
/// # Safety
///
/// `name` must be a valid null-terminated C string.
#[allow(clippy::must_use_candidate)]
#[export_name = "find_option"]
pub unsafe extern "C" fn rs_find_option(name: *const c_char) -> OptIndex {
    if name.is_null() {
        return OPT_INVALID;
    }
    let len = libc_strlen(name);
    nvim_find_option_len_hash(name, len)
}

/// Find option index by name with specified length.
///
/// # Safety
///
/// `name` must point to valid memory for at least `len` bytes.
#[allow(clippy::must_use_candidate)]
#[export_name = "find_option_len"]
pub unsafe extern "C" fn rs_find_option_len(name: *const c_char, len: usize) -> OptIndex {
    if name.is_null() || len == 0 {
        return OPT_INVALID;
    }
    nvim_find_option_len_hash(name, len)
}

// =============================================================================
// Option End Finding
// =============================================================================

/// Result of finding option name end.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FindOptionEndResult {
    /// Pointer to end of option name (NULL if not found)
    pub end: *const c_char,
    /// Option index (OPT_INVALID if not found or TTY option)
    pub opt_idx: OptIndex,
}

/// Find the end of an option name in a string.
///
/// This handles both regular options and TTY/keycode options.
///
/// # Safety
///
/// `arg` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_find_option_end(arg: *const c_char) -> FindOptionEndResult {
    let invalid = FindOptionEndResult {
        end: std::ptr::null(),
        opt_idx: OPT_INVALID,
    };

    if arg.is_null() {
        return invalid;
    }

    // Check for TTY/keycode options first
    let tty_end = rs_find_tty_option_end(arg);
    if !tty_end.is_null() {
        return FindOptionEndResult {
            end: tty_end,
            opt_idx: OPT_INVALID, // TTY options don't have an index
        };
    }

    // Regular option: must start with ASCII letter
    let first = *arg as u8;
    if !first.is_ascii_alphabetic() {
        return invalid;
    }

    // Find end of option name (alphanumeric characters only)
    let mut p = arg;
    while (*p as u8).is_ascii_alphabetic() {
        p = p.add(1);
    }

    // Calculate length and look up option
    let len = p.offset_from(arg) as usize;
    let opt_idx = nvim_find_option_len_hash(arg, len);

    FindOptionEndResult { end: p, opt_idx }
}

/// Get the length of an option name starting at the given pointer.
///
/// Returns 0 if not a valid option name start.
///
/// # Safety
///
/// `arg` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_get_option_name_len(arg: *const c_char) -> usize {
    if arg.is_null() {
        return 0;
    }

    // Must start with ASCII letter
    let first = *arg as u8;
    if !first.is_ascii_alphabetic() {
        return 0;
    }

    let mut len: usize = 1;
    let mut p = arg.add(1);
    while (*p as u8).is_ascii_alphabetic() {
        len += 1;
        p = p.add(1);
    }

    len
}

// =============================================================================
// Option Type Checking
// =============================================================================

/// Option value type constants (matching kOptValType* in C)
pub mod val_type {
    use std::ffi::c_int;

    pub const NIL: c_int = -1;
    pub const BOOLEAN: c_int = 0;
    pub const NUMBER: c_int = 1;
    pub const STRING: c_int = 2;
}

/// Check if option supports boolean type.
#[no_mangle]
pub unsafe extern "C" fn rs_option_is_boolean(opt_idx: OptIndex) -> c_int {
    if opt_idx == OPT_INVALID {
        return 0;
    }
    c_int::from(nvim_get_option_type(opt_idx) == val_type::BOOLEAN)
}

/// Check if option supports number type.
#[no_mangle]
pub unsafe extern "C" fn rs_option_is_number(opt_idx: OptIndex) -> c_int {
    if opt_idx == OPT_INVALID {
        return 0;
    }
    c_int::from(nvim_get_option_type(opt_idx) == val_type::NUMBER)
}

/// Check if option supports string type.
#[no_mangle]
pub unsafe extern "C" fn rs_option_is_string(opt_idx: OptIndex) -> c_int {
    if opt_idx == OPT_INVALID {
        return 0;
    }
    c_int::from(nvim_get_option_type(opt_idx) == val_type::STRING)
}

/// Check if option has a specific type.
#[allow(clippy::must_use_candidate)]
#[export_name = "option_has_type"]
pub unsafe extern "C" fn rs_option_has_type(opt_idx: OptIndex, val_type: c_int) -> c_int {
    if opt_idx == OPT_INVALID {
        return 0;
    }
    c_int::from(nvim_get_option_type(opt_idx) == val_type)
}

// =============================================================================
// Option Scope Checking
// =============================================================================

/// Scope constants (matching kOptScope* in C)
pub mod scope {
    use std::ffi::c_int;

    pub const GLOBAL: c_int = 0;
    pub const WIN: c_int = 1;
    pub const BUF: c_int = 2;
}

/// Check if option supports a specific scope.
#[allow(clippy::must_use_candidate)]
#[export_name = "option_has_scope"]
pub unsafe extern "C" fn rs_option_has_scope(opt_idx: OptIndex, opt_scope: c_int) -> c_int {
    if opt_idx == OPT_INVALID {
        return 0;
    }
    let scope_flags = nvim_get_option_scope_flags(opt_idx) as u32;
    c_int::from((scope_flags & (1u32 << opt_scope)) != 0)
}

/// Check if option is global-local (has both global and local values).
#[no_mangle]
pub unsafe extern "C" fn rs_option_is_global_local(opt_idx: OptIndex) -> c_int {
    if opt_idx == OPT_INVALID {
        return 0;
    }
    let scope_flags = nvim_get_option_scope_flags(opt_idx) as u32;
    c_int::from(!is_power_of_two(scope_flags))
}

/// Check if option is global-only (no local value).
#[no_mangle]
pub unsafe extern "C" fn rs_option_is_global_only(opt_idx: OptIndex) -> c_int {
    if opt_idx == OPT_INVALID {
        return 0;
    }
    let scope_flags = nvim_get_option_scope_flags(opt_idx) as u32;
    // kOptScopeGlobal == 0, bit 0
    c_int::from(is_power_of_two(scope_flags) && (scope_flags & 1) != 0)
}

/// Check if option is window-local only.
#[no_mangle]
pub unsafe extern "C" fn rs_option_is_window_local(opt_idx: OptIndex) -> c_int {
    if opt_idx == OPT_INVALID {
        return 0;
    }
    let scope_flags = nvim_get_option_scope_flags(opt_idx) as u32;
    // kOptScopeWin == 1, bit 1
    c_int::from(is_power_of_two(scope_flags) && (scope_flags & 2) != 0)
}

/// Check if option is hidden (immutable and var points to def_val.data).
#[allow(clippy::must_use_candidate)]
#[export_name = "is_option_hidden"]
pub unsafe extern "C" fn rs_option_is_hidden(opt_idx: OptIndex) -> c_int {
    if opt_idx == OPT_INVALID {
        return 0;
    }
    let immutable = nvim_get_option_immutable(opt_idx);
    if immutable == 0 {
        return 0;
    }
    let var_ptr = nvim_get_option_var(opt_idx);
    let def_data_ptr = nvim_get_option_def_val_data_ptr(opt_idx);
    c_int::from(var_ptr == def_data_ptr.cast_mut())
}

// =============================================================================
// Option Type, Scope Index, Flags, Was-Set, Script Context (Phase 8)
// =============================================================================

/// kOptFlagWasSet flag value (must match C definition: 1 << 3)
const K_OPT_FLAG_WAS_SET: c_uint = 1 << 3;

/// Get the type of an option.
#[no_mangle]
pub unsafe extern "C" fn rs_option_get_type(opt_idx: OptIndex) -> c_int {
    nvim_get_option_type(opt_idx)
}

/// Get option index for scope.
#[no_mangle]
pub unsafe extern "C" fn rs_option_scope_idx(opt_idx: OptIndex, scope: c_int) -> c_int {
    nvim_get_option_scope_idx(opt_idx, scope)
}

/// Get option flags.
#[no_mangle]
pub unsafe extern "C" fn rs_get_option_flags(opt_idx: OptIndex) -> c_uint {
    if opt_idx == OPT_INVALID {
        return 0;
    }
    nvim_get_option_flags(opt_idx)
}

/// Check if option was set (has kOptFlagWasSet flag).
#[allow(clippy::must_use_candidate)]
#[export_name = "option_was_set"]
pub unsafe extern "C" fn rs_option_was_set(opt_idx: OptIndex) -> c_int {
    if opt_idx == OPT_INVALID {
        return 0;
    }
    c_int::from((nvim_get_option_flags(opt_idx) & K_OPT_FLAG_WAS_SET) != 0)
}

/// Reset the was-set flag for an option.
#[export_name = "reset_option_was_set"]
pub unsafe extern "C" fn rs_reset_option_was_set(opt_idx: OptIndex) {
    if opt_idx != OPT_INVALID {
        *nvim_option_get_flags_ptr(opt_idx) &= !crate::setops::K_OPT_FLAG_WAS_SET;
    }
}

/// Get pointer to option's script context.
#[allow(clippy::must_use_candidate)]
#[export_name = "get_option_sctx"]
pub unsafe extern "C" fn rs_get_option_sctx(opt_idx: OptIndex) -> *mut std::ffi::c_void {
    nvim_get_option_script_ctx_ptr(opt_idx)
}

/// Check if option has secure flag.
#[no_mangle]
pub unsafe extern "C" fn rs_option_is_secure(opt_idx: OptIndex) -> c_int {
    let flags = unsafe { rs_get_option_flags(opt_idx) };
    c_int::from((flags & option_flags::SECURE) != 0)
}

/// Check if option is not allowed in modeline.
#[no_mangle]
pub unsafe extern "C" fn rs_option_no_modeline(opt_idx: OptIndex) -> c_int {
    let flags = unsafe { rs_get_option_flags(opt_idx) };
    c_int::from((flags & option_flags::NO_ML) != 0)
}

/// Check if option is under modelineexpr control.
#[no_mangle]
pub unsafe extern "C" fn rs_option_is_mle(opt_idx: OptIndex) -> c_int {
    let flags = unsafe { rs_get_option_flags(opt_idx) };
    c_int::from((flags & option_flags::MLE) != 0)
}

// =============================================================================
// Option Index Validation
// =============================================================================

/// Validation result for option index.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ValidateOptIdxResult {
    /// OK (1) on success, FAIL (0) on failure
    pub result: c_int,
    /// Error message (NULL if no error)
    pub errmsg: *const c_char,
}

// Error messages (static strings)
static E_INVARG: &[u8] = b"E474: Invalid argument\0";
static E_NOT_ALLOWED_IN_MODELINE: &[u8] = b"E520: Not allowed in a modeline\0";
static E_NOT_ALLOWED_MLE_OFF: &[u8] =
    b"E992: Not allowed in a modeline when 'modelineexpr' is off\0";
static E_SANDBOX: &[u8] = b"E48: Not allowed in sandbox\0";

/// Known option indices for special checks
/// These match the enum values from options_enum.generated.h
pub mod known_opts {
    use super::OptIndex;

    // These values need to match the generated enum
    // We'll use extern declarations to get them from C
    pub const FOLDMETHOD: OptIndex = -2; // Placeholder, actual value from C
    pub const WRAP: OptIndex = -3; // Placeholder, actual value from C
}

/// Validate an option index for the :set command.
///
/// Checks various conditions that would prevent setting an option:
/// - Boolean prefix on non-boolean option
/// - Window-only/no-window filtering
/// - Modeline restrictions
/// - Sandbox restrictions
/// - Diff mode restrictions
///
/// # Safety
///
/// `win` must be a valid window handle or NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_opt_idx(
    win: WinHandle,
    opt_idx: OptIndex,
    opt_flags: c_int,
    flags: c_uint,
    prefix: c_int,
) -> ValidateOptIdxResult {
    let ok = ValidateOptIdxResult {
        result: OK,
        errmsg: std::ptr::null(),
    };

    // Only bools can have a prefix of 'inv' or 'no'
    let is_bool = option_has_type(opt_idx, val_type::BOOLEAN) != 0;
    let has_prefix = prefix != SetPrefix::None as c_int;
    if !is_bool && has_prefix {
        return ValidateOptIdxResult {
            result: FAIL,
            errmsg: E_INVARG.as_ptr().cast(),
        };
    }

    // Skip all options that are not window-local (used when showing
    // an already loaded buffer in a window).
    let is_winonly = (opt_flags & opt_flags::OPT_WINONLY) != 0;
    let is_window_local = nvim_option_is_window_local(opt_idx) != 0;
    if is_winonly && !is_window_local {
        return ValidateOptIdxResult {
            result: FAIL,
            errmsg: std::ptr::null(),
        };
    }

    // Skip all options that are window-local (used for :vimgrep).
    let is_nowin = (opt_flags & opt_flags::OPT_NOWIN) != 0;
    if is_nowin && is_window_local {
        return ValidateOptIdxResult {
            result: FAIL,
            errmsg: std::ptr::null(),
        };
    }

    // Disallow changing some options from modelines.
    let is_modeline = (opt_flags & opt_flags::OPT_MODELINE) != 0;
    if is_modeline {
        let is_secure = (flags & option_flags::SECURE) != 0;
        let is_no_ml = (flags & option_flags::NO_ML) != 0;
        if is_secure || is_no_ml {
            return ValidateOptIdxResult {
                result: FAIL,
                errmsg: E_NOT_ALLOWED_IN_MODELINE.as_ptr().cast(),
            };
        }

        let is_mle = (flags & option_flags::MLE) != 0;
        let mle_on = p_mle != 0;
        if is_mle && !mle_on {
            return ValidateOptIdxResult {
                result: FAIL,
                errmsg: E_NOT_ALLOWED_MLE_OFF.as_ptr().cast(),
            };
        }

        // In diff mode some options are overruled. This avoids that
        // 'foldmethod' becomes "marker" instead of "diff" and that
        // "wrap" gets set.
        if !win.is_null() {
            let win_diff = win_ref(win).w_p_diff() != 0;
            if win_diff && (opt_idx == K_OPT_FOLDMETHOD || opt_idx == K_OPT_WRAP) {
                return ValidateOptIdxResult {
                    result: FAIL,
                    errmsg: std::ptr::null(),
                };
            }
        }
    }

    // Disallow changing some options in the sandbox
    let sandbox = nvim_get_sandbox();
    let is_secure_flag = (flags & option_flags::SECURE) != 0;
    if sandbox != 0 && is_secure_flag {
        return ValidateOptIdxResult {
            result: FAIL,
            errmsg: E_SANDBOX.as_ptr().cast(),
        };
    }

    ok
}

/// Simplified validation without window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_opt_idx_simple(
    opt_idx: OptIndex,
    opt_flags: c_int,
    flags: c_uint,
    prefix: c_int,
) -> ValidateOptIdxResult {
    rs_validate_opt_idx(std::ptr::null_mut(), opt_idx, opt_flags, flags, prefix)
}

// =============================================================================
// TTY Option Checking
// =============================================================================

/// Check if option name is a TTY option.
#[no_mangle]
pub unsafe extern "C" fn rs_is_tty_opt(name: *const c_char) -> c_int {
    if name.is_null() {
        return 0;
    }
    rs_is_tty_option(name)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_get_option_name_len() {
        unsafe {
            let name1 = CString::new("autoindent").unwrap();
            assert_eq!(rs_get_option_name_len(name1.as_ptr()), 10);

            let name2 = CString::new("ai=value").unwrap();
            assert_eq!(rs_get_option_name_len(name2.as_ptr()), 2);

            let name3 = CString::new("123abc").unwrap();
            assert_eq!(rs_get_option_name_len(name3.as_ptr()), 0);

            assert_eq!(rs_get_option_name_len(std::ptr::null()), 0);
        }
    }
}
