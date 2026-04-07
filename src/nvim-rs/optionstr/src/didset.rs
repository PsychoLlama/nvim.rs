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

    /// Get opaque vimoption_T* from index
    fn get_option(idx: OptIndex) -> *mut c_void;

    /// Get null-terminated values array from vimoption_T*
    fn nvim_option_get_values(opt: *mut c_void) -> *const *const c_char;

    /// Get flags bitmask for option at idx
    fn rs_get_option_flags(idx: OptIndex) -> c_uint;

    /// Get opt->var pointer (as void*) from opaque vimoption_T*
    fn nvim_vimoption_get_var(opt: *mut c_void) -> *mut c_void;

    /// Get opt->flags_var pointer (as *mut c_uint) from opaque vimoption_T*;
    /// returns NULL if flags_var is NULL
    fn nvim_vimoption_get_flags_var_ptr(opt: *mut c_void) -> *mut c_uint;

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
    let norm_idx = normalize_opt_idx_for_expand(idx);
    let opt_norm = get_option(norm_idx);
    let values = nvim_option_get_values(opt_norm);

    // Determine if this option is a comma-separated list
    let flags_val = rs_get_option_flags(idx);
    let is_list = (flags_val & (K_OPT_FLAG_COMMA | K_OPT_FLAG_ONE_COMMA)) != 0;

    // Get the actual string to validate
    let val: *const c_char = if varp.is_null() {
        // dereference opt->var (a char**) to get the global string value
        let opt = get_option(idx);
        if opt.is_null() {
            std::ptr::null()
        } else {
            let var_ptr = nvim_vimoption_get_var(opt);
            if var_ptr.is_null() {
                std::ptr::null()
            } else {
                *(var_ptr as *const *const c_char)
            }
        }
    } else {
        *varp
    };

    let result: OptStringsFlagsResult = rs_opt_strings_flags(val, values, is_list);

    // Write flags back (opt->flags_var if non-NULL)
    let opt = get_option(idx);
    if !opt.is_null() {
        let fv_ptr = nvim_vimoption_get_flags_var_ptr(opt);
        if !fv_ptr.is_null() {
            *fv_ptr = result.flags;
        }
    }

    result.ok
}

// =============================================================================
// Option indices for didset_string_options
// (numeric values from build/src/nvim/auto/options_enum.generated.h)
// =============================================================================

const K_OPT_FILEFORMAT: OptIndex = 94;
const K_OPT_FILEFORMATS: OptIndex = 95;
const K_OPT_CASEMAP: OptIndex = 31;
const K_OPT_BACKUPCOPY: OptIndex = 16;
const K_OPT_BELLOFF: OptIndex = 20;
const K_OPT_COMPLETEOPT: OptIndex = 54;
const K_OPT_SESSIONOPTIONS: OptIndex = 253;
const K_OPT_VIEWOPTIONS: OptIndex = 342;

/// Normalize opt_idx for values lookup:
/// viewoptions uses sessionoptions values; fileformats uses fileformat values.
fn normalize_opt_idx_for_expand(idx: OptIndex) -> OptIndex {
    if idx == K_OPT_VIEWOPTIONS {
        K_OPT_SESSIONOPTIONS
    } else if idx == K_OPT_FILEFORMATS {
        K_OPT_FILEFORMAT
    } else {
        idx
    }
}
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

// =============================================================================
// Phase 1 FFI additions
// =============================================================================

extern "C" {
    // optset_T field accessors
    fn nvim_optset_get_errbuf(args: *const c_void) -> *mut c_char;
    fn nvim_optset_get_errbuflen(args: *const c_void) -> usize;

    // String utilities
    fn vim_strchr(string: *const c_char, c: c_int) -> *mut c_char;
    fn vim_snprintf(str_: *mut c_char, str_m: usize, fmt: *const c_char, ...) -> c_int;

    // shada parameter query (already used in C: rs_get_shada_parameter)
    fn rs_get_shada_parameter(param: c_int) -> c_int;

    // transchar_byte: returns static buffer with printable form of char
    fn transchar_byte(c: c_int) -> *const c_char;

    // check_stl_option: validates statusline format string
    fn check_stl_option(s: *mut c_char) -> *const c_char;

    // rs_did_set_title: update window titles
    fn rs_did_set_title();

    // copy_option_part: parse next comma-sep part into buf, advance pp
    fn copy_option_part(
        pp: *mut *mut c_char,
        buf: *mut c_char,
        maxlen: usize,
        sep_chars: *const c_char,
    ) -> usize;

    // strequal: strcmp == 0 helper
    fn strequal(a: *const c_char, b: *const c_char) -> bool;
}

// =============================================================================
// did_set_colorcolumn
// =============================================================================

extern "C" {
    fn nvim_optset_get_win(args: *const c_void) -> *mut c_void;

    /// Validate 'colorcolumn' string; apply to window if wp != NULL.
    fn check_colorcolumn(cc: *mut c_char, wp: *mut c_void) -> *const c_char;
}

/// The 'colorcolumn' option is changed.
///
/// # Safety
/// Must be called only from C option machinery.
#[export_name = "did_set_colorcolumn"]
pub unsafe extern "C" fn did_set_colorcolumn(args: *const c_void) -> *const c_char {
    let varp = nvim_optset_get_varp(args).cast::<*mut c_char>();
    let s = *varp;
    let win = nvim_optset_get_win(args);
    // colorcolumn is a window-only option, so varp is always &win->w_p_cc.
    // Always pass win to check_colorcolumn so it applies the value.
    check_colorcolumn(s, win)
}

// =============================================================================
// C globals for did_set_shada
// =============================================================================

extern "C" {
    #[link_name = "p_shada"]
    static p_shada: *const c_char;
}

/// Error strings reused in shada validation
const E_ILLEGAL_CHAR_PREFIX: *const c_char = c"E539: Illegal character <%s>".as_ptr();
const E_MISSING_NUMBER: *const c_char = c"E526: Missing number after <%s>".as_ptr();
const E_MISSING_COMMA: *const c_char = c"E527: Missing comma".as_ptr();
const E_MUST_SPECIFY_QUOTE: *const c_char = c"E528: Must specify a ' value".as_ptr();

/// Validate 'shada' option value.
///
/// # Safety
/// Must be called only from C option machinery.
#[export_name = "did_set_shada"]
pub unsafe extern "C" fn did_set_shada(args: *const c_void) -> *const c_char {
    let errbuf = nvim_optset_get_errbuf(args);
    let errbuflen = nvim_optset_get_errbuflen(args);

    let mut s = p_shada;
    while *s != 0 {
        // Check it's a valid character
        if vim_strchr(c"!\"%'/:<@cfhnrs".as_ptr(), c_int::from(*s as u8)).is_null() {
            // illegal_char equivalent
            if errbuf.is_null() {
                return c"".as_ptr();
            }
            vim_snprintf(
                errbuf,
                errbuflen,
                E_ILLEGAL_CHAR_PREFIX,
                transchar_byte(c_int::from(*s as u8)),
            );
            return errbuf;
        }

        if *s as u8 == b'n' {
            // name is always last one
            break;
        } else if *s as u8 == b'r' {
            // skip until next ','
            s = s.add(1);
            while *s != 0 && *s as u8 != b',' {
                s = s.add(1);
            }
        } else if *s as u8 == b'%' {
            // optional number
            s = s.add(1);
            while (*s as u8).is_ascii_digit() {
                s = s.add(1);
            }
        } else if *s as u8 == b'!' || *s as u8 == b'h' || *s as u8 == b'c' {
            s = s.add(1); // no extra chars
        } else {
            // must have a number
            s = s.add(1);
            while (*s as u8).is_ascii_digit() {
                s = s.add(1);
            }
            // s-1 must have been a digit
            if !(*s.sub(1) as u8).is_ascii_digit() {
                if !errbuf.is_null() {
                    vim_snprintf(
                        errbuf,
                        errbuflen,
                        E_MISSING_NUMBER,
                        transchar_byte(c_int::from(*s.sub(1) as u8)),
                    );
                    return errbuf;
                }
                return c"".as_ptr();
            }
        }

        if *s as u8 == b',' {
            s = s.add(1);
        } else if *s != 0 {
            if !errbuf.is_null() {
                return E_MISSING_COMMA;
            }
            return c"".as_ptr();
        }
    }

    if *p_shada != 0 && rs_get_shada_parameter(b'\'' as c_int) < 0 {
        return E_MUST_SPECIFY_QUOTE;
    }

    std::ptr::null()
}

// =============================================================================
// C globals for did_set_completeitemalign
// =============================================================================

extern "C" {
    #[link_name = "p_cia"]
    static mut p_cia: *const c_char;

    #[link_name = "cia_flags"]
    static mut cia_flags: c_uint;
}

// CPT enum values (from insexpand.h)
const CPT_ABBR: usize = 0;
const CPT_KIND: usize = 1;
const CPT_MENU: usize = 2;

/// Validate 'completeitemalign' option value.
///
/// # Safety
/// Must be called only from C option machinery.
#[export_name = "did_set_completeitemalign"]
pub unsafe extern "C" fn did_set_completeitemalign(_args: *const c_void) -> *const c_char {
    let mut p = p_cia.cast_mut();
    let mut new_cia_flags: c_uint = 0;
    let mut seen = [false; 3];
    let mut count = 0usize;
    let mut buf = [0u8; 10];

    while *p != 0 {
        copy_option_part(
            &mut p,
            buf.as_mut_ptr().cast::<c_char>(),
            buf.len(),
            c",".as_ptr(),
        );
        if count >= 3 {
            return E_INVARG;
        }

        if strequal(buf.as_ptr().cast::<c_char>(), c"abbr".as_ptr()) {
            if seen[CPT_ABBR] {
                return E_INVARG;
            }
            new_cia_flags = new_cia_flags * 10 + CPT_ABBR as c_uint;
            seen[CPT_ABBR] = true;
            count += 1;
        } else if strequal(buf.as_ptr().cast::<c_char>(), c"kind".as_ptr()) {
            if seen[CPT_KIND] {
                return E_INVARG;
            }
            new_cia_flags = new_cia_flags * 10 + CPT_KIND as c_uint;
            seen[CPT_KIND] = true;
            count += 1;
        } else if strequal(buf.as_ptr().cast::<c_char>(), c"menu".as_ptr()) {
            if seen[CPT_MENU] {
                return E_INVARG;
            }
            new_cia_flags = new_cia_flags * 10 + CPT_MENU as c_uint;
            seen[CPT_MENU] = true;
            count += 1;
        } else {
            return E_INVARG;
        }
    }

    if new_cia_flags == 0 || count != 3 {
        return E_INVARG;
    }

    cia_flags = new_cia_flags;
    std::ptr::null()
}

// =============================================================================
// C globals for did_set_titleiconstring
// =============================================================================

extern "C" {
    #[link_name = "stl_syntax"]
    static mut stl_syntax: c_int;
}

/// The 'titlestring' or the 'iconstring' option is changed.
/// flagval should be STL_IN_ICON or STL_IN_TITLE.
///
/// # Safety
/// Must be called only from C option machinery.
#[export_name = "did_set_titleiconstring"]
pub unsafe extern "C" fn did_set_titleiconstring(
    args: *const c_void,
    flagval: c_int,
) -> *const c_char {
    let varp_void = nvim_optset_get_varp(args);
    let varp = varp_void.cast::<*mut c_char>();
    let s = *varp;

    // NULL => statusline syntax
    if !vim_strchr(s, b'%' as c_int).is_null() && check_stl_option(s).is_null() {
        stl_syntax |= flagval;
    } else {
        stl_syntax &= !flagval;
    }
    rs_did_set_title();

    std::ptr::null()
}

// =============================================================================
// Phase 1: Simple did_set_* Callbacks
// =============================================================================

extern "C" {
    /// optset_T field: args->os_buf
    fn nvim_optset_get_buf(args: *const c_void) -> *mut c_void;

    /// optset_T field: args->os_flags
    fn nvim_optset_get_flags(args: *const c_void) -> c_int;

    /// optset_T field: args->os_oldval.string.data
    fn nvim_optset_get_oldval_str(args: *const c_void) -> *const c_char;

    /// Set os_restore_chartab
    fn nvim_optset_set_restore_chartab(args: *mut c_void, val: c_int);

    /// Set os_value_checked
    #[allow(dead_code)]
    fn nvim_optset_set_value_checked(args: *mut c_void, val: c_int);

    /// buf->b_p_cot (local completeopt)
    fn nvim_buf_get_p_cot(buf: *mut c_void) -> *mut c_char;

    /// &buf->b_cot_flags
    fn nvim_buf_get_cot_flags_ptr(buf: *mut c_void) -> *mut c_uint;

    /// buf->b_p_tc (local tagcase)
    fn nvim_buf_get_p_tc(buf: *mut c_void) -> *mut c_char;

    /// &buf->b_tc_flags
    fn nvim_buf_get_tc_flags_ptr(buf: *mut c_void) -> *mut c_uint;

    /// win->w_p_ve (local virtualedit)
    fn nvim_win_get_p_ve(wp: *mut c_void) -> *mut c_char;

    /// &win->w_ve_flags
    fn nvim_win_get_ve_flags_ptr(wp: *mut c_void) -> *mut c_uint;

    /// &win->w_virtcol (colnr_T = c_int)
    fn nvim_win_get_virtcol(wp: *mut c_void) -> *mut c_int;

    /// &buf->b_p_vsts_array (colnr_T**)
    fn nvim_buf_get_vsts_array_ptr(buf: *mut c_void) -> *mut *mut c_int;

    /// &buf->b_p_vts_array (colnr_T**)
    fn nvim_buf_get_vts_array_ptr(buf: *mut c_void) -> *mut *mut c_int;

    // Cursor utilities
    fn validate_virtcol(wp: *mut c_void);
    fn coladvance(wp: *mut c_void, virtcol: c_int) -> c_int;

    // Chartab utilities
    fn buf_init_chartab(buf: *mut c_void, global: bool) -> c_int;
    fn check_isopt(var: *mut c_char) -> c_int;

    // Memory + markup
    fn ml_setflags(buf: *mut c_void);
    fn redraw_buf_later(buf: *mut c_void, kind: c_int);
    fn redraw_titles();

    // Tabstop utilities
    fn tabstop_set(var: *const c_char, array: *mut *mut c_int) -> bool;
    fn rs_foldmethodIsIndent(wp: *mut c_void) -> c_int;
    fn rs_foldUpdateAll(wp: *mut c_void);

    // Completion
    fn set_cpt_callbacks(args: *const c_void) -> c_int;

    // Fileformat
    fn rs_get_fileformat(buf: *mut c_void) -> c_int;

    // Buffer modifiable check
    fn nvim_buf_get_b_p_ma(buf: *mut c_void) -> c_int;

    // Memory
    fn xfree(ptr: *mut c_void);
}

// OPT_LOCAL / OPT_GLOBAL flags (from nvim/option.h)
const OPT_GLOBAL: c_int = 0x01;
const OPT_LOCAL: c_int = 0x02;
// UPD_NOT_VALID (from nvim/drawscreen.h)
const UPD_NOT_VALID: c_int = 40;
// OK/FAIL (from nvim/vim_defs.h)
const OK: c_int = 1;
// FAIL = 0, not needed explicitly

// C globals accessed via link_name
extern "C" {
    #[link_name = "p_ve"]
    static p_ve: *const c_char;
    #[link_name = "ve_flags"]
    static mut ve_flags: c_uint;
    #[link_name = "p_tc"]
    static p_tc: *const c_char;
    #[link_name = "tc_flags"]
    static mut tc_flags: c_uint;
    #[link_name = "p_cot"]
    static p_cot: *const c_char;
    #[link_name = "cot_flags"]
    static mut cot_flags: c_uint;
    // values arrays (extern pointers to array data)
    #[link_name = "opt_ve_values"]
    static opt_ve_values: *const *const c_char;
    #[link_name = "opt_tc_values"]
    static opt_tc_values: *const *const c_char;
    #[link_name = "opt_cot_values"]
    static opt_cot_values: *const *const c_char;
    #[link_name = "e_invarg"]
    static e_invarg: *const c_char;
    #[link_name = "e_modifiable"]
    static e_modifiable: *const c_char;
}

// =============================================================================
// did_set_completeopt
// =============================================================================

/// The 'completeopt' option is changed.
///
/// # Safety
/// Must be called only from C option machinery.
#[export_name = "did_set_completeopt"]
pub unsafe extern "C" fn did_set_completeopt(args: *const c_void) -> *const c_char {
    let buf = nvim_optset_get_buf(args);
    let opt_flags = nvim_optset_get_flags(args);

    let (cot, flags): (*const c_char, *mut c_uint) = if opt_flags & OPT_LOCAL != 0 {
        (nvim_buf_get_p_cot(buf), nvim_buf_get_cot_flags_ptr(buf))
    } else if opt_flags & OPT_GLOBAL != 0 {
        (p_cot, std::ptr::addr_of_mut!(cot_flags))
    } else {
        // :set clears local flags
        if !buf.is_null() {
            let local_flags = nvim_buf_get_cot_flags_ptr(buf);
            if !local_flags.is_null() {
                *local_flags = 0;
            }
        }
        (p_cot, std::ptr::addr_of_mut!(cot_flags))
    };

    let result = rs_opt_strings_flags(cot, opt_cot_values, true);
    if !result.ok {
        return e_invarg;
    }
    if !flags.is_null() {
        *flags = result.flags;
    }
    std::ptr::null()
}

// =============================================================================
// did_set_tagcase
// =============================================================================

/// The 'tagcase' option is changed.
///
/// # Safety
/// Must be called only from C option machinery.
#[export_name = "did_set_tagcase"]
pub unsafe extern "C" fn did_set_tagcase(args: *const c_void) -> *const c_char {
    let buf = nvim_optset_get_buf(args);
    let opt_flags = nvim_optset_get_flags(args);

    let (p, flags): (*const c_char, *mut c_uint) = if opt_flags & OPT_LOCAL != 0 {
        (nvim_buf_get_p_tc(buf), nvim_buf_get_tc_flags_ptr(buf))
    } else {
        (p_tc, std::ptr::addr_of_mut!(tc_flags))
    };

    // local empty value: use global flags
    if opt_flags & OPT_LOCAL != 0 && !p.is_null() && *p == 0 {
        if !flags.is_null() {
            *flags = 0;
        }
        return std::ptr::null();
    }

    let result = rs_opt_strings_flags(p, opt_tc_values, false);
    if !result.ok {
        return e_invarg;
    }
    if !flags.is_null() {
        *flags = result.flags;
    }
    std::ptr::null()
}

// =============================================================================
// did_set_virtualedit
// =============================================================================

/// The 'virtualedit' option is changed.
///
/// # Safety
/// Must be called only from C option machinery.
#[export_name = "did_set_virtualedit"]
pub unsafe extern "C" fn did_set_virtualedit(args: *const c_void) -> *const c_char {
    let win = nvim_optset_get_win(args);
    let opt_flags = nvim_optset_get_flags(args);
    let oldval = nvim_optset_get_oldval_str(args);

    let (ve, flags): (*const c_char, *mut c_uint) = if opt_flags & OPT_LOCAL != 0 {
        (nvim_win_get_p_ve(win), nvim_win_get_ve_flags_ptr(win))
    } else {
        (p_ve, std::ptr::addr_of_mut!(ve_flags))
    };

    // local empty value: use global flags
    if opt_flags & OPT_LOCAL != 0 && !ve.is_null() && *ve == 0 {
        if !flags.is_null() {
            *flags = 0;
        }
        return std::ptr::null();
    }

    let result = rs_opt_strings_flags(ve, opt_ve_values, true);
    if !result.ok {
        return e_invarg;
    }
    if !flags.is_null() {
        *flags = result.flags;
    }

    // Recompute cursor position if value changed
    if !cstr_eq(ve, oldval) {
        validate_virtcol(win);
        let virtcol_ptr = nvim_win_get_virtcol(win);
        let virtcol = if virtcol_ptr.is_null() {
            0
        } else {
            *virtcol_ptr
        };
        coladvance(win, virtcol);
    }

    std::ptr::null()
}

// =============================================================================
// did_set_signcolumn
// =============================================================================

extern "C" {
    fn check_signcolumn(scl: *mut c_char, wp: *mut c_void) -> c_int;
}

extern "C" {
    fn nvim_win_get_p_scl(wp: *mut c_void) -> *mut c_char;
    fn nvim_win_set_nrwidth_line_count(wp: *mut c_void, val: c_int);
    fn nvim_win_get_minscwidth(wp: *mut c_void) -> c_int;
}

/// The 'signcolumn' option is changed.
///
/// # Safety
/// Must be called only from C option machinery.
#[export_name = "did_set_signcolumn"]
pub unsafe extern "C" fn did_set_signcolumn(args: *const c_void) -> *const c_char {
    let win = nvim_optset_get_win(args);
    let varp = nvim_optset_get_varp(args).cast::<*mut c_char>();
    let oldval = nvim_optset_get_oldval_str(args);

    let scl_ptr = *varp;
    let is_win_local = !win.is_null() && scl_ptr == nvim_win_get_p_scl(win);

    if check_signcolumn(
        scl_ptr,
        if is_win_local {
            win
        } else {
            std::ptr::null_mut()
        },
    ) != OK
    {
        return e_invarg;
    }

    // When changing to/from 'number', recompute the width of the number column
    let oldval_nu =
        !oldval.is_null() && *oldval == b'n' as c_char && *oldval.add(1) == b'u' as c_char;
    let scl_num: c_int = -2; // SCL_NUM from nvim/sign_defs.h
    let minscwidth = if win.is_null() {
        0
    } else {
        nvim_win_get_minscwidth(win)
    };
    if oldval_nu || minscwidth == scl_num {
        nvim_win_set_nrwidth_line_count(win, 0);
    }
    std::ptr::null()
}

// =============================================================================
// did_set_iskeyword
// =============================================================================

/// The 'iskeyword' option is changed.
///
/// # Safety
/// Must be called only from C option machinery.
#[export_name = "did_set_iskeyword"]
pub unsafe extern "C" fn did_set_iskeyword(args: *const c_void) -> *const c_char {
    let varp = nvim_optset_get_varp(args).cast::<*mut c_char>();
    let val = *varp;

    // p_isk is the global 'iskeyword' pointer, obtained via varp comparison
    // We use the C function check_isopt for the global case
    // and did_set_isopt for the local case
    // The plan: varp == &p_isk checks if it's global; we replicate with the existing
    // C accessor. Since we can't compare pointer addresses directly from Rust without
    // the actual global address, we use the pattern: if check_isopt succeeds, it's OK.
    // For the global case (varp == &p_isk), call check_isopt.
    // For the local case (varp != &p_isk), fall through to did_set_isopt.
    // We use a C helper to determine this.

    // Simplified approach: if varp is the global iskeyword, check_isopt is enough
    // Otherwise delegate to did_set_isopt logic inline.
    // To distinguish, we call check_isopt on the value and if it succeeds, do nothing more.
    // If varp is local (not global), the buf_init_chartab path is needed.
    // We call the C check_isopt to validate, then try buf_init_chartab always.
    // The original code: if varp == &p_isk -> check_isopt only; else -> did_set_isopt
    // We replicate this by having a C shim expose whether varp is global.

    // For simplicity, implement the full logic: check_isopt for validation,
    // then conditionally call did_set_isopt if needed.
    // The safest approach: delegate to the C implementations via accessors.
    // Actually, the plan says to implement both did_set_iskeyword and did_set_isopt in Rust.
    // We use nvim_optset_is_global_iskeyword to distinguish.

    // Since we cannot compare `varp` to `&p_isk` without knowing p_isk address,
    // we add a C helper. For now, implement as: always validate with check_isopt,
    // then if it's a local varp, also run buf_init_chartab.
    // We add nvim_optset_varp_is_p_isk as a helper to option_shim.c.
    // But that's extra complexity. Simplest: implement did_set_iskeyword using the
    // C check_isopt for validation and call did_set_isopt_impl for the buffer case.

    if nvim_optset_varp_is_global_isk(args) != 0 {
        // Global case: only check_isopt
        if check_isopt(val) == 0 {
            return e_invarg;
        }
    } else {
        // Local case: delegate to did_set_isopt logic
        return did_set_isopt(args);
    }

    std::ptr::null()
}

extern "C" {
    /// Returns 1 if args->os_varp == &p_isk (global 'iskeyword')
    fn nvim_optset_varp_is_global_isk(args: *const c_void) -> c_int;
}

// =============================================================================
// did_set_isopt
// =============================================================================

/// The 'isident', 'iskeyword', 'isprint', or 'isfname' option is changed.
///
/// # Safety
/// Must be called only from C option machinery.
#[export_name = "did_set_isopt"]
pub unsafe extern "C" fn did_set_isopt(args: *const c_void) -> *const c_char {
    let buf = nvim_optset_get_buf(args);
    if buf_init_chartab(buf, true) == 0 {
        // FAIL = 0: need to restore chartab
        nvim_optset_set_restore_chartab(args.cast_mut(), 1);
        return e_invarg;
    }
    std::ptr::null()
}

// =============================================================================
// did_set_fileformat
// =============================================================================

/// The 'fileformat' option is changed.
///
/// # Safety
/// Must be called only from C option machinery.
#[export_name = "did_set_fileformat"]
pub unsafe extern "C" fn did_set_fileformat(args: *const c_void) -> *const c_char {
    let buf = nvim_optset_get_buf(args);
    let opt_flags = nvim_optset_get_flags(args);
    let oldval = nvim_optset_get_oldval_str(args);

    // Check modifiable when not setting global
    if nvim_buf_get_b_p_ma(buf) == 0 && (opt_flags & OPT_GLOBAL == 0) {
        return e_modifiable;
    }

    let errmsg = did_set_str_generic(args);
    if !errmsg.is_null() {
        return errmsg;
    }

    redraw_titles();
    ml_setflags(buf);

    // Redraw needed when switching to/from "mac"
    let fmt = rs_get_fileformat(buf);
    let old_is_mac = !oldval.is_null() && *oldval == b'm' as c_char;
    // EOL_MAC = 2 (from nvim/fileio.h)
    if fmt == 2 || old_is_mac {
        redraw_buf_later(buf, UPD_NOT_VALID);
    }

    std::ptr::null()
}

// =============================================================================
// did_set_varsofttabstop
// =============================================================================

/// The 'varsofttabstop' option is changed.
///
/// # Safety
/// Must be called only from C option machinery.
#[export_name = "did_set_varsofttabstop"]
pub unsafe extern "C" fn did_set_varsofttabstop(args: *const c_void) -> *const c_char {
    let buf = nvim_optset_get_buf(args);
    let varp = nvim_optset_get_varp(args).cast::<*mut c_char>();
    let val = *varp;

    // Empty or "0": clear the array
    if *val == 0 || (*val == b'0' as c_char && *val.add(1) == 0) {
        let arr_pp = nvim_buf_get_vsts_array_ptr(buf);
        if !arr_pp.is_null() {
            xfree((*arr_pp).cast::<c_void>());
            *arr_pp = std::ptr::null_mut();
        }
        return std::ptr::null();
    }

    // Validate: only digits and commas (no leading/trailing commas)
    let mut cp = val;
    loop {
        let c = *cp as u8;
        if c == 0 {
            break;
        }
        if c.is_ascii_digit() {
            cp = cp.add(1);
            continue;
        }
        if c == b',' && cp > val && *cp.sub(1) as u8 != b',' {
            cp = cp.add(1);
            continue;
        }
        return e_invarg;
    }

    let arr_pp = nvim_buf_get_vsts_array_ptr(buf);
    if arr_pp.is_null() {
        return e_invarg;
    }
    let oldarray = *arr_pp;
    if tabstop_set(val, arr_pp) {
        xfree(oldarray.cast::<c_void>());
    } else {
        return e_invarg;
    }

    std::ptr::null()
}

// =============================================================================
// did_set_vartabstop
// =============================================================================

/// The 'vartabstop' option is changed.
///
/// # Safety
/// Must be called only from C option machinery.
#[export_name = "did_set_vartabstop"]
pub unsafe extern "C" fn did_set_vartabstop(args: *const c_void) -> *const c_char {
    let buf = nvim_optset_get_buf(args);
    let win = nvim_optset_get_win(args);
    let varp = nvim_optset_get_varp(args).cast::<*mut c_char>();
    let val = *varp;

    // Empty or "0": clear the array
    if *val == 0 || (*val == b'0' as c_char && *val.add(1) == 0) {
        let arr_pp = nvim_buf_get_vts_array_ptr(buf);
        if !arr_pp.is_null() {
            xfree((*arr_pp).cast::<c_void>());
            *arr_pp = std::ptr::null_mut();
        }
        return std::ptr::null();
    }

    // Validate: only digits and commas
    let mut cp = val;
    loop {
        let c = *cp as u8;
        if c == 0 {
            break;
        }
        if c.is_ascii_digit() {
            cp = cp.add(1);
            continue;
        }
        if c == b',' && cp > val && *cp.sub(1) as u8 != b',' {
            cp = cp.add(1);
            continue;
        }
        return e_invarg;
    }

    let arr_pp = nvim_buf_get_vts_array_ptr(buf);
    if arr_pp.is_null() {
        return e_invarg;
    }
    let oldarray = *arr_pp;
    if tabstop_set(val, arr_pp) {
        xfree(oldarray.cast::<c_void>());
        if rs_foldmethodIsIndent(win) != 0 {
            rs_foldUpdateAll(win);
        }
    } else {
        return e_invarg;
    }

    std::ptr::null()
}

// =============================================================================
// did_set_complete
// =============================================================================

#[inline]
fn is_ascii_digit_char(c: u8) -> bool {
    c.is_ascii_digit()
}

/// Check if value for 'complete' is valid when 'complete' option is changed.
///
/// # Safety
/// Must be called only from C option machinery.
#[export_name = "did_set_complete"]
pub unsafe extern "C" fn did_set_complete(args: *const c_void) -> *const c_char {
    let varp = nvim_optset_get_varp(args).cast::<*mut c_char>();
    let errbuf = nvim_optset_get_errbuf(args);
    let errbuflen = nvim_optset_get_errbuflen(args);

    let mut p = *varp;
    while *p != 0 {
        // Buffer for current item (LSIZE = 512)
        const LSIZE: usize = 512;
        let mut buffer = [0u8; LSIZE];
        let mut buf_ptr: usize = 0;
        let mut escape = false;

        // Extract substring, handling escaped commas
        while *p != 0 && (*p as u8 != b',' || escape) && buf_ptr < LSIZE - 1 {
            if *p as u8 == b'\\' && *p.add(1) as u8 == b',' {
                escape = true;
                p = p.add(1); // skip '\'
            } else {
                escape = false;
                buffer[buf_ptr] = *p as u8;
                buf_ptr += 1;
            }
            p = p.add(1);
        }
        buffer[buf_ptr] = 0;

        let first = buffer[0];

        // Check first char is valid
        if vim_strchr(c".wbuksid]tUfFo".as_ptr(), c_int::from(first)).is_null() {
            return crate::errors::illegal_char(errbuf, errbuflen, c_int::from(first));
        }

        // Determine which character (if any) caused a validation error.
        // char_before != 0 means an illegal character follows that character.
        #[allow(clippy::useless_let_if_seq)]
        let mut char_before: u8 = 0;
        if vim_strchr(c"ksF".as_ptr(), c_int::from(first)).is_null()
            && buffer[1] != 0
            && buffer[1] != b'^'
        {
            char_before = first;
        } else {
            // Test for a number after '^'
            let caret_pos = buffer[..buf_ptr].iter().position(|&b| b == b'^');
            if let Some(caret_pos) = caret_pos {
                let t = caret_pos + 1;
                if t >= buf_ptr || buffer[t] == 0 {
                    char_before = b'^';
                } else {
                    for &b in &buffer[t..buf_ptr] {
                        if b == 0 {
                            break;
                        }
                        if !is_ascii_digit_char(b) {
                            char_before = b'^';
                            break;
                        }
                    }
                }
            }
        }

        if char_before != 0 {
            if !errbuf.is_null() {
                return crate::errors::illegal_char_after_chr(
                    errbuf,
                    errbuflen,
                    c_int::from(char_before),
                );
            }
            return std::ptr::null();
        }

        // Skip comma and spaces
        while *p as u8 == b',' || *p as u8 == b' ' {
            p = p.add(1);
        }
    }

    if set_cpt_callbacks(args) != OK {
        return crate::errors::illegal_char_after_chr(errbuf, errbuflen, b'F' as c_int);
    }

    std::ptr::null()
}
