//! did_set_* callback implementations for option validation
//!
//! This module provides Rust implementations of option change callbacks
//! that were previously implemented in C's optionstr.c. Each function is
//! exported under the original C symbol name via #[export_name].

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]

use crate::listval::{rs_opt_strings_flags, OptStringsFlagsResult};
use std::ffi::{c_char, c_int, c_uint, c_void};

// =============================================================================
// C FFI: check_str_opt infrastructure
// =============================================================================

/// OptIndex type alias (matches C's OptIndex = int)
type OptIndex = c_int;

/// kOptFlagComma bitmask (1 << 10)
const K_OPT_FLAG_COMMA: c_uint = 1 << 10;
/// kOptFlagOneComma bitmask (1 << 11 | 1 << 10)
const K_OPT_FLAG_ONE_COMMA: c_uint = (1 << 11) | (1 << 10);

extern "C" {
    /// Normalize an option index for values lookup
    /// (kOptViewoptions -> kOptSessionoptions, kOptFileformats -> kOptFileformat)
    fn nvim_normalize_opt_idx_for_expand(idx: OptIndex) -> OptIndex;

    /// Get opaque vimoption_T* from index
    fn get_option(idx: OptIndex) -> *mut c_void;

    /// Get null-terminated values array from vimoption_T*
    fn nvim_option_get_values(opt: *mut c_void) -> *const *const c_char;

    /// Get flags bitmask for option at idx
    fn rs_get_option_flags(idx: OptIndex) -> c_uint;

    /// Get global string value for option at idx (dereferences opt->var)
    fn nvim_option_get_global_str_val(idx: OptIndex) -> *const c_char;

    /// Set *opt->flags_var = val if flags_var is non-NULL
    fn nvim_option_set_flags_var_if_present(idx: OptIndex, val: c_uint);

    /// optset_T field: args->os_idx
    fn nvim_optset_get_idx(args: *const c_void) -> OptIndex;

    /// optset_T field: args->os_varp (the char** itself, as void*)
    fn nvim_optset_get_varp(args: *const c_void) -> *mut c_void;
}

// =============================================================================
// Rust implementation of check_str_opt
// =============================================================================

/// Validate a string option value against its allowed values list.
///
/// Mirrors C's `check_str_opt(idx, varp)`:
/// - normalizes idx for values lookup (viewoptions -> sessionoptions)
/// - calls rs_opt_strings_flags to validate and compute flags
/// - writes flags to opt->flags_var if present
///
/// If varp is null, uses the global option value (opt->var).
///
/// Returns true on success (value is valid), false on failure.
///
/// # Safety
/// idx must be a valid OptIndex; varp (if non-null) must point to a valid string pointer.
pub unsafe fn check_str_opt_impl(idx: OptIndex, varp: *mut *mut c_char) -> bool {
    // Normalize index for values lookup
    let norm_idx = nvim_normalize_opt_idx_for_expand(idx);
    let opt_norm = get_option(norm_idx);
    let values = nvim_option_get_values(opt_norm);

    // Determine if this option is a comma-separated list
    let flags_val = rs_get_option_flags(idx);
    let is_list = (flags_val & (K_OPT_FLAG_COMMA | K_OPT_FLAG_ONE_COMMA)) != 0;

    // Get the actual string to validate
    let val: *const c_char = if varp.is_null() {
        nvim_option_get_global_str_val(idx)
    } else {
        *varp
    };

    let result: OptStringsFlagsResult = rs_opt_strings_flags(val, values, is_list);

    // Write flags back
    nvim_option_set_flags_var_if_present(idx, result.flags);

    result.ok
}

// =============================================================================
// Option indices for didset_string_options
// (numeric values from build/src/nvim/auto/options_enum.generated.h)
// =============================================================================

const K_OPT_CASEMAP: OptIndex = 31;
const K_OPT_BACKUPCOPY: OptIndex = 16;
const K_OPT_BELLOFF: OptIndex = 20;
const K_OPT_COMPLETEOPT: OptIndex = 54;
const K_OPT_SESSIONOPTIONS: OptIndex = 253;
const K_OPT_VIEWOPTIONS: OptIndex = 342;
const K_OPT_FOLDOPEN: OptIndex = 112;
const K_OPT_DISPLAY: OptIndex = 76;
const K_OPT_JUMPOPTIONS: OptIndex = 157;
const K_OPT_REDRAWDEBUG: OptIndex = 231;
const K_OPT_TAGCASE: OptIndex = 306;
const K_OPT_TERMPASTEFILTER: OptIndex = 315;
const K_OPT_VIRTUALEDIT: OptIndex = 343;
const K_OPT_SWITCHBUF: OptIndex = 298;
const K_OPT_TABCLOSE: OptIndex = 301;
const K_OPT_WILDOPTIONS: OptIndex = 353;
const K_OPT_CLIPBOARD: OptIndex = 43;

// =============================================================================
// Error Message Constants
// =============================================================================

/// Error: Invalid argument
const E_INVARG: *const c_char = c"E474: Invalid argument".as_ptr();

/// Error: 'backupext' and 'patchmode' are equal
const E_BEX_EQ_PM: &[u8] = b"E589: 'backupext' and 'patchmode' are equal\0";

// =============================================================================
// C Globals accessed via link_name
// =============================================================================

extern "C" {
    /// 'helplang' global option variable
    #[link_name = "p_hlg"]
    static p_hlg: *const c_char;

    /// 'breakat' global option variable
    #[link_name = "p_breakat"]
    static p_breakat: *const c_char;

    /// 'breakat' flags array (256 bools)
    #[link_name = "breakat_flags"]
    static mut breakat_flags: [u8; 256];

    /// 'backupext' global option variable
    #[link_name = "p_bex"]
    static p_bex: *const c_char;

    /// 'patchmode' global option variable
    #[link_name = "p_pm"]
    static p_pm: *const c_char;

    /// 'mousescroll' global option variable
    #[link_name = "p_mousescroll"]
    static p_mousescroll: *const c_char;

    /// 'mousescroll' vertical default value (p_mousescroll_vert)
    #[link_name = "p_mousescroll_vert"]
    static mut p_mousescroll_vert: i64;

    /// 'mousescroll' horizontal default value (p_mousescroll_hor)
    #[link_name = "p_mousescroll_hor"]
    static mut p_mousescroll_hor: i64;
}

// =============================================================================
// Helper: compare two C strings for equality
// =============================================================================

/// Compare two null-terminated C strings for equality.
/// Returns true if they are equal.
#[inline]
unsafe fn cstr_eq(a: *const c_char, b: *const c_char) -> bool {
    if a.is_null() && b.is_null() {
        return true;
    }
    if a.is_null() || b.is_null() {
        return false;
    }
    let mut pa = a;
    let mut pb = b;
    while *pa != 0 && *pb != 0 {
        if *pa != *pb {
            return false;
        }
        pa = pa.add(1);
        pb = pb.add(1);
    }
    *pa == *pb
}

// =============================================================================
// 'helplang' Callback
// =============================================================================

/// Validate 'helplang' option value.
/// Format: "", "ab", "ab,cd", etc. (two-letter language codes)
///
/// # Safety
/// Must be called only from C option machinery.
#[export_name = "did_set_helplang"]
pub unsafe extern "C" fn did_set_helplang(_args: *const c_void) -> *const c_char {
    let s = p_hlg;
    if s.is_null() || *s == 0 {
        return std::ptr::null();
    }

    // Check for "", "ab", "ab,cd", etc.
    let mut p = s;
    loop {
        // After each two-char code, must have NUL, ',', or end
        // s[0] and s[1] must exist (non-NUL)
        if *p == 0 || *p.add(1) == 0 {
            return E_INVARG;
        }
        let third = *p.add(2) as u8;
        if third == 0 {
            break; // valid end
        }
        if third != b',' {
            return E_INVARG;
        }
        // After comma, must have more content
        if *p.add(3) == 0 {
            return E_INVARG;
        }
        p = p.add(3); // skip "xy,"
    }
    std::ptr::null()
}

// =============================================================================
// 'breakat' Callback
// =============================================================================

/// Update breakat_flags array when 'breakat' option is changed.
///
/// # Safety
/// Must be called only from C option machinery.
#[export_name = "did_set_breakat"]
pub unsafe extern "C" fn did_set_breakat(_args: *const c_void) -> *const c_char {
    // Clear all flags using raw pointer to avoid mutable static ref warning
    let flags_ptr = std::ptr::addr_of_mut!(breakat_flags).cast::<u8>();
    for i in 0..256_usize {
        *flags_ptr.add(i) = 0;
    }

    // Set flags for each character in p_breakat
    let val = p_breakat;
    if !val.is_null() {
        let mut p = val;
        while *p != 0 {
            *flags_ptr.add(*p as u8 as usize) = 1;
            p = p.add(1);
        }
    }

    std::ptr::null()
}

// =============================================================================
// 'backupext' / 'patchmode' Callback
// =============================================================================

/// Validate that 'backupext' and 'patchmode' are not equal.
///
/// # Safety
/// Must be called only from C option machinery.
#[export_name = "did_set_backupext_or_patchmode"]
pub unsafe extern "C" fn did_set_backupext_or_patchmode(_args: *const c_void) -> *const c_char {
    // Skip leading '.' for comparison
    let bex = if !p_bex.is_null() && *p_bex as u8 == b'.' {
        p_bex.add(1)
    } else {
        p_bex
    };
    let pm = if !p_pm.is_null() && *p_pm as u8 == b'.' {
        p_pm.add(1)
    } else {
        p_pm
    };

    if cstr_eq(bex, pm) {
        return E_BEX_EQ_PM.as_ptr().cast();
    }
    std::ptr::null()
}

// =============================================================================
// 'mousescroll' Callback
// =============================================================================

/// Default scroll amounts for 'mousescroll'
const MOUSESCROLL_VERT_DFLT: i64 = 3;
const MOUSESCROLL_HOR_DFLT: i64 = 6;

/// Parse and apply 'mousescroll' option.
/// Format: "ver:N,hor:N" or "hor:N,ver:N" (order doesn't matter)
///
/// # Safety
/// Must be called only from C option machinery.
#[export_name = "did_set_mousescroll"]
pub unsafe extern "C" fn did_set_mousescroll(_args: *const c_void) -> *const c_char {
    let mut vertical: i64 = -1;
    let mut horizontal: i64 = -1;

    let val = p_mousescroll;
    if val.is_null() {
        return E_INVARG;
    }

    let mut string = val;

    loop {
        // Find end of current item (comma or NUL)
        let mut end = string;
        while *end != 0 && *end as u8 != b',' {
            end = end.add(1);
        }
        let length = end.offset_from(string) as usize;

        // Both "ver:" and "hor:" are 4 bytes long + at least one digit
        if length <= 4 {
            return E_INVARG;
        }

        // Determine direction
        let is_ver = *string as u8 == b'v'
            && *string.add(1) as u8 == b'e'
            && *string.add(2) as u8 == b'r'
            && *string.add(3) as u8 == b':';
        let is_hor = *string as u8 == b'h'
            && *string.add(1) as u8 == b'o'
            && *string.add(2) as u8 == b'r'
            && *string.add(3) as u8 == b':';

        if !is_ver && !is_hor {
            return E_INVARG;
        }

        let direction: &mut i64 = if is_ver {
            &mut vertical
        } else {
            &mut horizontal
        };

        // Duplicate direction
        if *direction != -1 {
            return E_INVARG;
        }

        // Parse digits after the colon
        let mut num_ptr = string.add(4);
        let mut value: i64 = 0;
        let mut has_digit = false;

        while num_ptr < end {
            let ch = *num_ptr as u8;
            if !ch.is_ascii_digit() {
                return E_INVARG;
            }
            has_digit = true;
            value = value * 10 + i64::from(ch - b'0');
            num_ptr = num_ptr.add(1);
        }

        if !has_digit || value < 0 {
            return E_INVARG;
        }

        *direction = value;

        if *end == 0 {
            break;
        }
        string = end.add(1);
    }

    // Apply results (fallback to defaults if not set)
    p_mousescroll_vert = if vertical == -1 {
        MOUSESCROLL_VERT_DFLT
    } else {
        vertical
    };
    p_mousescroll_hor = if horizontal == -1 {
        MOUSESCROLL_HOR_DFLT
    } else {
        horizontal
    };

    std::ptr::null()
}

// =============================================================================
// did_set_str_generic and didset_string_options (Phase 4)
// =============================================================================

/// Check a string option by index, using global value if varp is NULL.
/// Returns 1 (OK) if valid, 0 (FAIL) if invalid.
/// Equivalent to C's `check_str_opt(idx, varp)`.
///
/// # Safety
/// idx must be a valid OptIndex; varp (if non-null) must point to a valid string pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_check_str_opt(idx: OptIndex, varp: *mut *mut c_char) -> c_int {
    c_int::from(check_str_opt_impl(idx, varp))
}

/// Validates the current option value against its allowed values list.
/// Equivalent to C's `did_set_str_generic`.
///
/// # Safety
/// args must be a valid optset_T pointer.
#[export_name = "did_set_str_generic"]
pub unsafe extern "C" fn did_set_str_generic(args: *const c_void) -> *const c_char {
    let idx = nvim_optset_get_idx(args);
    // os_varp is char** - get it as a mutable pointer to char*
    let varp_pp = nvim_optset_get_varp(args).cast::<*mut c_char>();
    if check_str_opt_impl(idx, varp_pp) {
        std::ptr::null()
    } else {
        E_INVARG
    }
}

/// Recompute flags for all string options after loading viminfo / shada.
/// Equivalent to C's `didset_string_options`.
///
/// # Safety
/// Must be called only from C option machinery.
#[export_name = "didset_string_options"]
pub unsafe extern "C" fn didset_string_options() {
    let opts: &[OptIndex] = &[
        K_OPT_CASEMAP,
        K_OPT_BACKUPCOPY,
        K_OPT_BELLOFF,
        K_OPT_COMPLETEOPT,
        K_OPT_SESSIONOPTIONS,
        K_OPT_VIEWOPTIONS,
        K_OPT_FOLDOPEN,
        K_OPT_DISPLAY,
        K_OPT_JUMPOPTIONS,
        K_OPT_REDRAWDEBUG,
        K_OPT_TAGCASE,
        K_OPT_TERMPASTEFILTER,
        K_OPT_VIRTUALEDIT,
        K_OPT_SWITCHBUF,
        K_OPT_TABCLOSE,
        K_OPT_WILDOPTIONS,
        K_OPT_CLIPBOARD,
    ];
    for &idx in opts {
        check_str_opt_impl(idx, std::ptr::null_mut());
    }
}

// =============================================================================
// rs_* aliases (Phase 109)
// =============================================================================

/// Alias for 'helplang' callback under the rs_ naming convention.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_helplang(args: *const c_void) -> *const c_char {
    did_set_helplang(args)
}

/// Alias for 'breakat' callback under the rs_ naming convention.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_breakat(args: *const c_void) -> *const c_char {
    did_set_breakat(args)
}

/// Alias for 'backupext'/'patchmode' callback under the rs_ naming convention.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_backupext_or_patchmode(args: *const c_void) -> *const c_char {
    did_set_backupext_or_patchmode(args)
}

/// Alias for 'mousescroll' callback under the rs_ naming convention.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_mousescroll(args: *const c_void) -> *const c_char {
    did_set_mousescroll(args)
}
