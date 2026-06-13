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
    fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    fn snprintf(str_: *mut c_char, size: usize, fmt: *const c_char, ...) -> c_int;

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

    // expand_T field accessors
    fn nvim_xp_get_pattern(xp: *mut ExpandT) -> *mut c_char;

    // Option infrastructure
    fn get_option(idx: c_int) -> *mut c_void;
    fn nvim_option_get_values(opt: *mut c_void) -> *const *const c_char;
    fn nvim_option_get_values_len(opt: *mut c_void) -> usize;

    // Window handle (defined in window_shim)
    fn nvim_get_curwin() -> *mut c_void;

    // autocmd event names
    fn get_event_name_no_group(xp: *mut ExpandT, idx: c_int, win: bool) -> *mut c_char;
}

// OptIndex constants for normalization (must match opt_index.rs)
const K_OPT_FILEFORMAT: c_int = 94;
const K_OPT_FILEFORMATS: c_int = 95;
const K_OPT_SESSIONOPTIONS: c_int = 253;
const K_OPT_VIEWOPTIONS: c_int = 342;

/// Normalize opt_idx for expand_set_str_generic:
/// viewoptions uses sessionoptions values; fileformats uses fileformat values.
fn normalize_opt_idx_for_expand(idx: c_int) -> c_int {
    if idx == K_OPT_VIEWOPTIONS {
        K_OPT_SESSIONOPTIONS
    } else if idx == K_OPT_FILEFORMATS {
        K_OPT_FILEFORMAT
    } else {
        idx
    }
}

// Return values matching C OK/FAIL (vim_defs.h: OK=1, FAIL=0)
const OK: c_int = 1;
const FAIL: c_int = 0;

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
    let normalized_idx = normalize_opt_idx_for_expand(idx);
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

// =============================================================================
// get_fileformat_name
// =============================================================================

extern "C" {
    #[link_name = "opt_ff_values"]
    static opt_ff_values: [*const c_char; 4];
}

/// Return the file format name at index, or NULL if out of range.
/// Equivalent to C's get_fileformat_name().
///
/// # Safety
#[export_name = "get_fileformat_name"]
pub unsafe extern "C" fn get_fileformat_name(_xp: *mut ExpandT, idx: c_int) -> *mut c_char {
    if idx < 0 || idx as usize >= opt_ff_values.len() {
        return std::ptr::null_mut();
    }
    opt_ff_values[idx as usize].cast_mut()
}

/// Check if value is a valid fileformat name.
/// Returns OK (1) if valid, FAIL (0) otherwise.
/// Equivalent to C's check_ff_value().
///
/// # Safety
#[export_name = "check_ff_value"]
pub unsafe extern "C" fn check_ff_value(p: *mut c_char) -> c_int {
    use crate::listval::rs_opt_strings_flags;
    if rs_opt_strings_flags(p, opt_ff_values.as_ptr(), false).ok {
        OK
    } else {
        FAIL
    }
}

// =============================================================================
// expand_set_diffopt
// =============================================================================

extern "C" {
    #[link_name = "opt_dip_algorithm_values"]
    static opt_dip_algorithm_values: [*const c_char; 5];

    #[link_name = "opt_dip_inline_values"]
    static opt_dip_inline_values: [*const c_char; 5];
}

/// Expand 'diffopt' option
///
/// # Safety
#[export_name = "expand_set_diffopt"]
pub unsafe extern "C" fn expand_set_diffopt(
    args: *const OptExpandArgs,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    let xp = nvim_oe_get_xp(args);
    let xp_pattern = nvim_xp_get_pattern(xp);
    let set_arg = nvim_oe_get_set_arg(args);

    if xp_pattern > set_arg.cast_mut() && *xp_pattern.sub(1) as u8 == b':' {
        // Within "algorithm:", we have a subgroup of possible options.
        let algo_len = b"algorithm:".len();
        let offset = xp_pattern.offset_from(set_arg) as usize;
        if offset >= algo_len
            && strncmp(xp_pattern.sub(algo_len), c"algorithm:".as_ptr(), algo_len) == 0
        {
            return expand_set_opt_string_impl(
                args,
                opt_dip_algorithm_values.as_ptr(),
                opt_dip_algorithm_values.len() - 1,
                num_matches,
                matches,
            );
        }
        // Within "inline:", we have a subgroup of possible options.
        let inline_len = b"inline:".len();
        if offset >= inline_len
            && strncmp(xp_pattern.sub(inline_len), c"inline:".as_ptr(), inline_len) == 0
        {
            return expand_set_opt_string_impl(
                args,
                opt_dip_inline_values.as_ptr(),
                opt_dip_inline_values.len() - 1,
                num_matches,
                matches,
            );
        }
        return FAIL;
    }

    expand_set_str_generic(args, num_matches, matches)
}

// =============================================================================
// expand_set_eventignore
// =============================================================================

extern "C" {
    #[link_name = "p_ei"]
    static p_ei: *const c_char;

    // IObuff global buffer
    #[link_name = "IObuff"]
    static mut IOBUFF: [c_char; IOSIZE];
}

const IOSIZE: usize = 1025;

// Thread-local state for expand_set_eventignore
static mut EXPAND_EIW: bool = false;

unsafe extern "C" fn get_eventignore_name(xp: *mut ExpandT, idx: c_int) -> *mut c_char {
    let subtract = *nvim_xp_get_pattern(xp) as u8 == b'-';
    // 'eventignore(win)' allows special keyword "all" in addition to all event names.
    if !subtract && idx == 0 {
        return c"all".as_ptr().cast_mut();
    }

    let name = get_event_name_no_group(xp, idx - 1 + c_int::from(subtract), EXPAND_EIW);
    if name.is_null() {
        return std::ptr::null_mut();
    }

    let iobuf = std::ptr::addr_of_mut!(IOBUFF).cast::<c_char>();
    snprintf(
        iobuf,
        IOSIZE,
        c"%s%s".as_ptr(),
        if subtract {
            c"-".as_ptr()
        } else {
            c"".as_ptr()
        },
        name,
    );
    iobuf
}

/// Expand 'eventignore' option
///
/// # Safety
#[export_name = "expand_set_eventignore"]
pub unsafe extern "C" fn expand_set_eventignore(
    args: *const OptExpandArgs,
    num_matches: *mut c_int,
    matches: *mut *mut *mut c_char,
) -> c_int {
    EXPAND_EIW = nvim_oe_get_varp(args) != p_ei.cast_mut();
    expand_set_opt_generic_impl(args, get_eventignore_name, num_matches, matches)
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

#[cfg(test)]
mod tests {
    use super::{FAIL, OK};

    /// Compile-time guard: OK and FAIL must match vim_defs.h (OK=1, FAIL=0).
    /// If this test fails, the constants in this file are inverted again.
    #[test]
    fn ok_fail_constants_match_vim_defs() {
        assert_eq!(OK, 1, "OK must equal 1 (vim_defs.h)");
        assert_eq!(FAIL, 0, "FAIL must equal 0 (vim_defs.h)");
        // Sanity: they must be distinct
        assert_ne!(OK, FAIL);
    }
}
