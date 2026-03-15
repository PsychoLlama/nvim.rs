//! Expand helper infrastructure for option tab-completion
//!
//! This module provides Rust implementations of the expand_set_opt_* functions
//! previously defined as static helpers in optionstr.c.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Opaque type handles
// =============================================================================

/// Opaque handle for optexpand_T
#[repr(C)]
pub struct OptExpandArgs {
    _opaque: [u8; 0],
}

/// Opaque handle for expand_T
#[repr(C)]
pub struct ExpandT {
    _opaque: [u8; 0],
}

/// Opaque handle for regmatch_T
#[repr(C)]
pub struct RegmatchT {
    _opaque: [u8; 0],
}

// =============================================================================
// C FFI declarations
// =============================================================================

/// Type for CompleteListItemGetter callbacks: `char *(*)(expand_T *, int)`
pub type CompleteListItemGetter = unsafe extern "C" fn(xp: *mut ExpandT, idx: c_int) -> *mut c_char;

extern "C" {
    // Memory management
    fn xmalloc(size: usize) -> *mut c_void;
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn xmemdupz(data: *const c_void, len: usize) -> *mut c_char;
    fn xfree(ptr: *mut c_void);

    // String utilities
    fn vim_strchr(string: *const c_char, c: c_int) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;

    // Regex
    fn vim_regexec(rmp: *mut RegmatchT, line: *const c_char, col: i32) -> bool;

    // Expand generic
    fn ExpandGeneric(
        pat: *const c_char,
        xp: *mut ExpandT,
        regmatch: *mut RegmatchT,
        matches: *mut *mut *mut c_char,
        num_matches: *mut c_int,
        func: CompleteListItemGetter,
        escaped: bool,
    );

    // optexpand_T field accessors
    fn nvim_oe_get_opt_value(args: *const OptExpandArgs) -> *mut c_char;
    fn nvim_oe_get_set_arg(args: *const OptExpandArgs) -> *const c_char;
    fn nvim_oe_get_append(args: *const OptExpandArgs) -> bool;
    fn nvim_oe_get_include_orig_val(args: *const OptExpandArgs) -> bool;
    fn nvim_oe_get_regmatch(args: *const OptExpandArgs) -> *mut RegmatchT;
    fn nvim_oe_get_xp(args: *const OptExpandArgs) -> *mut ExpandT;
    fn nvim_oe_get_varp(args: *const OptExpandArgs) -> *mut c_char;
    fn nvim_oe_get_idx(args: *const OptExpandArgs) -> c_int;

    // Option infrastructure
    fn get_option(idx: c_int) -> *mut c_void;
    fn nvim_option_get_values(opt: *mut c_void) -> *const *const c_char;
    fn nvim_option_get_values_len(opt: *mut c_void) -> usize;

    // OptIndex normalization for expand_set_str_generic
    fn nvim_normalize_opt_idx_for_expand(idx: c_int) -> c_int;

    // Window handle (defined in window_shim)
    fn nvim_get_curwin() -> *const c_void;
}

// Return values matching C OK/FAIL
const OK: c_int = 0;
const FAIL: c_int = -1;

// =============================================================================
// expand_set_opt_string: expands a list of string values
// =============================================================================

/// Expand an option that accepts a list of string values.
/// Equivalent to C's static expand_set_opt_string().
///
/// # Safety
/// All pointer arguments must be valid.
pub unsafe fn expand_set_opt_string_impl(
    args: *const OptExpandArgs,
    values: *const *const c_char,
    num_values: usize,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    let regmatch = nvim_oe_get_regmatch(args);
    let include_orig_val = nvim_oe_get_include_orig_val(args);
    let option_val = nvim_oe_get_opt_value(args);

    // Allocate upfront (numValues is small for fixed enums)
    *matches = xmalloc(std::mem::size_of::<*mut c_char>() * (num_values + 1)).cast();

    let mut count: c_int = 0;

    if include_orig_val && *option_val != 0 {
        *(*matches).add(count as usize) = xstrdup(option_val);
        count += 1;
    }

    let mut val_ptr = values;
    while !(*val_ptr).is_null() {
        let val = *val_ptr;
        if *val == 0 {
            // Ignore empty
            val_ptr = val_ptr.add(1);
            continue;
        }
        if include_orig_val && *option_val != 0 && strcmp(val, option_val) == 0 {
            val_ptr = val_ptr.add(1);
            continue;
        }
        if vim_regexec(regmatch, val, 0) {
            *(*matches).add(count as usize) = xstrdup(val);
            count += 1;
        }
        val_ptr = val_ptr.add(1);
    }

    if count == 0 {
        xfree((*matches).cast());
        *matches = std::ptr::null_mut();
        return FAIL;
    }

    *num_matches = count;
    OK
}

// =============================================================================
// expand_set_opt_listflag: expands a list of single-character flags
// =============================================================================

/// Expand an option which is a list of single-character flags.
/// Equivalent to C's static expand_set_opt_listflag().
///
/// # Safety
/// All pointer arguments must be valid.
pub unsafe fn expand_set_opt_listflag_impl(
    args: *const OptExpandArgs,
    flags: *const c_char,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    let option_val = nvim_oe_get_opt_value(args);
    let cmdline_val = nvim_oe_get_set_arg(args);
    let append = nvim_oe_get_append(args);
    let include_orig_val = nvim_oe_get_include_orig_val(args) && *option_val != 0;

    let num_flags = strlen(flags);

    // Allocate max size
    *matches = xmalloc(std::mem::size_of::<*mut c_char>() * (num_flags + 1)).cast();

    let mut count: c_int = 0;

    if include_orig_val {
        *(*matches).add(count as usize) = xstrdup(option_val);
        count += 1;
    }

    let mut flag = flags;
    while *flag != 0 {
        if append && !vim_strchr(option_val, c_int::from(*flag as u8)).is_null() {
            flag = flag.add(1);
            continue;
        }

        if vim_strchr(cmdline_val, c_int::from(*flag as u8)).is_null() {
            if include_orig_val && *option_val.add(1) == 0 && *flag == *option_val {
                // This flag is already the first choice (existing flag), skip to avoid dup
                flag = flag.add(1);
                continue;
            }
            *(*matches).add(count as usize) = xmemdupz(flag.cast(), 1);
            count += 1;
        }

        flag = flag.add(1);
    }

    if count == 0 {
        xfree((*matches).cast());
        *matches = std::ptr::null_mut();
        return FAIL;
    }

    *num_matches = count;
    OK
}

// =============================================================================
// Static callback state for expand_set_opt_generic
// =============================================================================

// Thread-local state for the generic expand callback
// These correspond to the C statics set_opt_callback_orig_option and set_opt_callback_func
static mut SET_OPT_CALLBACK_ORIG_OPTION: *mut c_char = std::ptr::null_mut();
static mut SET_OPT_CALLBACK_FUNC: Option<CompleteListItemGetter> = None;

/// Callback used by expand_set_opt_generic to also include the original value.
/// Equivalent to C's static expand_set_opt_callback().
unsafe extern "C" fn expand_set_opt_callback(xp: *mut ExpandT, idx: c_int) -> *mut c_char {
    if idx == 0 {
        let orig = SET_OPT_CALLBACK_ORIG_OPTION;
        return if orig.is_null() {
            c"".as_ptr().cast_mut()
        } else {
            orig
        };
    }
    SET_OPT_CALLBACK_FUNC.map_or(std::ptr::null_mut(), |func| func(xp, idx - 1))
}

// =============================================================================
// expand_set_opt_generic: expands options via a callback function
// =============================================================================

/// Expand an option with a callback that iterates through a list of possible names.
/// Equivalent to C's static expand_set_opt_generic().
///
/// # Safety
/// All pointer arguments must be valid.
pub unsafe fn expand_set_opt_generic_impl(
    args: *const OptExpandArgs,
    func: CompleteListItemGetter,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    let include_orig_val = nvim_oe_get_include_orig_val(args);
    let opt_value = nvim_oe_get_opt_value(args);
    let regmatch = nvim_oe_get_regmatch(args);
    let xp = nvim_oe_get_xp(args);

    SET_OPT_CALLBACK_ORIG_OPTION = if include_orig_val {
        opt_value
    } else {
        std::ptr::null_mut()
    };
    SET_OPT_CALLBACK_FUNC = Some(func);

    ExpandGeneric(
        c"".as_ptr(),
        xp,
        regmatch,
        matches,
        num_matches,
        expand_set_opt_callback,
        false,
    );

    SET_OPT_CALLBACK_ORIG_OPTION = std::ptr::null_mut();
    SET_OPT_CALLBACK_FUNC = None;
    OK
}

// =============================================================================
// One-liner expand dispatchers (Phase 3)
// Exported under their original C names via #[export_name]
// =============================================================================

/// Expand 'concealcursor' option
///
/// # Safety
#[export_name = "expand_set_concealcursor"]
pub unsafe extern "C" fn expand_set_concealcursor(
    args: *const OptExpandArgs,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    expand_set_opt_listflag_impl(args, c"nvic".as_ptr(), num_matches, matches)
}

/// Expand 'cpoptions' option
///
/// # Safety
#[export_name = "expand_set_cpoptions"]
pub unsafe extern "C" fn expand_set_cpoptions(
    args: *const OptExpandArgs,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    expand_set_opt_listflag_impl(
        args,
        c"aAbBcCdDeEfFiIJKlLmMnoOpPqrRsStuvWxXyZ$!%+>;~_".as_ptr(),
        num_matches,
        matches,
    )
}

/// Expand 'formatoptions' option
///
/// # Safety
#[export_name = "expand_set_formatoptions"]
pub unsafe extern "C" fn expand_set_formatoptions(
    args: *const OptExpandArgs,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    expand_set_opt_listflag_impl(
        args,
        c"tcro/q2vlb1mMBn,aw]jp".as_ptr(),
        num_matches,
        matches,
    )
}

/// Expand 'mouse' option
///
/// # Safety
#[export_name = "expand_set_mouse"]
pub unsafe extern "C" fn expand_set_mouse(
    args: *const OptExpandArgs,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    expand_set_opt_listflag_impl(args, c"anvichr".as_ptr(), num_matches, matches)
}

/// Expand 'shortmess' option
///
/// # Safety
#[export_name = "expand_set_shortmess"]
pub unsafe extern "C" fn expand_set_shortmess(
    args: *const OptExpandArgs,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    expand_set_opt_listflag_impl(
        args,
        c"rmlwaWtToOsAIcCqFSnfxi".as_ptr(),
        num_matches,
        matches,
    )
}

/// Expand 'whichwrap' option
///
/// # Safety
#[export_name = "expand_set_whichwrap"]
pub unsafe extern "C" fn expand_set_whichwrap(
    args: *const OptExpandArgs,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    expand_set_opt_listflag_impl(args, c"bshl<>[]~".as_ptr(), num_matches, matches)
}

/// Expand options with a string list (expand_set_str_generic)
///
/// # Safety
#[export_name = "expand_set_str_generic"]
pub unsafe extern "C" fn expand_set_str_generic(
    args: *const OptExpandArgs,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    let idx = nvim_oe_get_idx(args);
    let normalized_idx = nvim_normalize_opt_idx_for_expand(idx);
    let opt = get_option(normalized_idx);
    let values = nvim_option_get_values(opt);
    let values_len = nvim_option_get_values_len(opt);
    expand_set_opt_string_impl(args, values, values_len, num_matches, matches)
}

/// Expand 'encoding' option
///
/// # Safety
#[export_name = "expand_set_encoding"]
pub unsafe extern "C" fn expand_set_encoding(
    args: *const OptExpandArgs,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    extern "C" {
        fn get_encoding_name(xp: *mut ExpandT, idx: c_int) -> *mut c_char;
    }
    expand_set_opt_generic_impl(args, get_encoding_name, num_matches, matches)
}

/// Expand 'winhighlight' option
///
/// # Safety
#[export_name = "expand_set_winhighlight"]
pub unsafe extern "C" fn expand_set_winhighlight(
    args: *const OptExpandArgs,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    extern "C" {
        fn get_highlight_name(xp: *mut ExpandT, idx: c_int) -> *mut c_char;
    }
    expand_set_opt_generic_impl(args, get_highlight_name, num_matches, matches)
}

/// Expand 'fillchars' or 'listchars' option
///
/// # Safety
#[export_name = "expand_set_chars_option"]
pub unsafe extern "C" fn expand_set_chars_option(
    args: *const OptExpandArgs,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    extern "C" {
        fn get_listchars_name(xp: *mut ExpandT, idx: c_int) -> *mut c_char;
        fn get_fillchars_name(xp: *mut ExpandT, idx: c_int) -> *mut c_char;

        #[link_name = "p_lcs"]
        static p_lcs: *const c_char;

        fn nvim_win_get_p_lcs(win: *const c_void) -> *const c_char;
    }

    let varp = nvim_oe_get_varp(args);
    let curwin = nvim_get_curwin();
    let curwin_p_lcs = nvim_win_get_p_lcs(curwin);
    let global_p_lcs = p_lcs;

    let is_lcs = varp == global_p_lcs.cast_mut() || varp == curwin_p_lcs.cast_mut();
    let func = if is_lcs {
        get_listchars_name
    } else {
        get_fillchars_name
    };

    expand_set_opt_generic_impl(args, func, num_matches, matches)
}
