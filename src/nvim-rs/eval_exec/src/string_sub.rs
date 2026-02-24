//! `do_string_sub` -- regex substitution on a string.
//!
//! Migrated from `eval_shim.c` Phase 7.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    unsafe_op_in_unsafe_fn
)]

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Constants
// =============================================================================

/// RE_MAGIC flag for vim_regcomp
const RE_MAGIC: c_int = 1;
/// RE_STRING flag for vim_regcomp
const RE_STRING: c_int = 2;
/// REGSUB_MAGIC flag for vim_regsub
const REGSUB_MAGIC: c_int = 1;
/// REGSUB_COPY flag for vim_regsub
const REGSUB_COPY: c_int = 2;

/// Maximum number of subexpressions in a regexp
const NSUBEXP: usize = 10;

// =============================================================================
// RegMatch struct
// =============================================================================

/// Structure matching regmatch_T for single-line matching.
/// Must match the C layout exactly (same as in eval/src/indexing.rs).
#[repr(C)]
struct RegMatch {
    regprog: *mut c_void,
    startp: [*mut c_char; NSUBEXP],
    endp: [*mut c_char; NSUBEXP],
    rm_matchcol: c_int,
    rm_ic: bool,
}

impl Default for RegMatch {
    fn default() -> Self {
        Self {
            regprog: std::ptr::null_mut(),
            startp: [std::ptr::null_mut(); NSUBEXP],
            endp: [std::ptr::null_mut(); NSUBEXP],
            rm_matchcol: 0,
            rm_ic: false,
        }
    }
}

// =============================================================================
// GArray struct
// =============================================================================

/// Growing array structure - matches C garray_T layout exactly.
#[repr(C)]
struct GArray {
    ga_len: c_int,
    ga_maxlen: c_int,
    ga_itemsize: c_int,
    ga_growsize: c_int,
    ga_data: *mut c_void,
}

impl Default for GArray {
    fn default() -> Self {
        Self {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 1,
            ga_growsize: 200,
            ga_data: std::ptr::null_mut(),
        }
    }
}

// =============================================================================
// C extern declarations
// =============================================================================

extern "C" {
    // Regex functions
    fn vim_regcomp(pat: *const c_char, flags: c_int) -> *mut c_void;
    fn vim_regexec_nl(rmp: *mut RegMatch, line: *const c_char, col: c_int) -> c_int;
    fn vim_regfree(prog: *mut c_void);
    fn rs_vim_regsub(
        rmp: *mut RegMatch,
        source: *mut c_char,
        expr: *mut c_void,
        dest: *mut c_char,
        destlen: c_int,
        flags: c_int,
    ) -> c_int;

    // garray
    #[link_name = "ga_init"]
    fn ga_init(gap: *mut GArray, itemsize: c_int, growsize: c_int);
    #[link_name = "ga_grow"]
    fn ga_grow(gap: *mut GArray, n: c_int);
    #[link_name = "ga_clear"]
    fn ga_clear(gap: *mut GArray);

    // memory
    fn xstrnsave(s: *const c_char, len: usize) -> *mut c_char;

    // option accessors
    fn p_ic_get() -> c_int;
    fn nvim_p_cpo_get() -> *mut c_char;
    fn nvim_p_cpo_set(val: *mut c_char);
    fn nvim_empty_string_option() -> *mut c_char;

    // p_cpo restoration (complex path: option changed during expr eval)
    fn nvim_do_string_sub_restore_cpo_complex(save_cpo: *mut c_char);

    // multi-byte
    fn utfc_ptr2len(p: *const c_char) -> c_int;
}

// =============================================================================
// Implementation
// =============================================================================

/// Perform regex substitution on a string.
///
/// Equivalent to C `do_string_sub` in eval_shim.c.
///
/// # Safety
///
/// All pointer arguments must be valid for their respective types.
/// `str` must point to a valid string of at least `len` bytes.
/// `pat` must be a valid null-terminated C string.
/// `sub` may be null if `expr` is non-null.
/// `expr` may be null if `sub` is non-null.
/// `flags` must be a valid null-terminated C string.
/// `ret_len` may be null.
#[no_mangle]
pub unsafe extern "C" fn rs_do_string_sub(
    str: *mut c_char,
    len: usize,
    pat: *mut c_char,
    sub: *mut c_char,
    expr: *mut c_void,
    flags: *const c_char,
    ret_len: *mut usize,
) -> *mut c_char {
    // Save and set p_cpo to empty
    let save_cpo = nvim_p_cpo_get();
    let empty = nvim_empty_string_option();
    nvim_p_cpo_set(empty);

    let mut ga = GArray::default();
    ga_init(&mut ga, 1, 200);

    let p_ic = p_ic_get();
    let mut regmatch = RegMatch {
        rm_ic: p_ic != 0,
        regprog: vim_regcomp(pat, RE_MAGIC + RE_STRING),
        ..RegMatch::default()
    };

    if !regmatch.regprog.is_null() {
        let mut tail = str;
        let end = str.add(len);
        let do_all = !flags.is_null() && *flags as u8 == b'g';
        let mut zero_width: *mut c_char = std::ptr::null_mut();

        loop {
            let col = tail.offset_from(str) as c_int;
            if vim_regexec_nl(&mut regmatch, str, col) == 0 {
                break;
            }

            // Skip empty match except for first match
            if regmatch.startp[0] == regmatch.endp[0] {
                if zero_width == regmatch.startp[0] {
                    // Avoid getting stuck on a match with an empty string
                    let i = utfc_ptr2len(tail) as usize;
                    let i = if i < 1 { 1 } else { i };
                    // Copy one char to ga
                    ga_grow(&mut ga, i as c_int);
                    std::ptr::copy_nonoverlapping(
                        tail,
                        (ga.ga_data as *mut c_char).add(ga.ga_len as usize),
                        i,
                    );
                    ga.ga_len += i as c_int;
                    tail = tail.add(i);
                    continue;
                }
                zero_width = regmatch.startp[0];
            }

            // Measure substitution size (pass with destlen=0)
            let sublen = rs_vim_regsub(&mut regmatch, sub, expr, tail, 0, REGSUB_MAGIC);
            if sublen <= 0 {
                ga_clear(&mut ga);
                break;
            }

            let match_len = regmatch.endp[0].offset_from(regmatch.startp[0]) as usize;
            let remaining = end.offset_from(tail) as c_int;
            ga_grow(&mut ga, remaining + sublen - match_len as c_int);

            // Copy text up to where the match starts
            let prefix_len = regmatch.startp[0].offset_from(tail) as usize;
            std::ptr::copy_nonoverlapping(
                tail,
                (ga.ga_data as *mut c_char).add(ga.ga_len as usize),
                prefix_len,
            );

            // Write substituted text
            rs_vim_regsub(
                &mut regmatch,
                sub,
                expr,
                (ga.ga_data as *mut c_char).add(ga.ga_len as usize + prefix_len),
                sublen,
                REGSUB_COPY | REGSUB_MAGIC,
            );

            ga.ga_len += prefix_len as c_int + sublen - 1;
            tail = regmatch.endp[0];

            if *tail == 0 {
                break;
            }
            if !do_all {
                break;
            }
        }

        // Append remainder of string (including NUL terminator via STRCPY)
        if !ga.ga_data.is_null() {
            let remainder = end.offset_from(tail) as usize;
            // Copy tail..end + NUL terminator
            let dest = (ga.ga_data as *mut c_char).add(ga.ga_len as usize);
            std::ptr::copy_nonoverlapping(tail, dest, remainder);
            // Set NUL terminator
            *dest.add(remainder) = 0;
            ga.ga_len += remainder as c_int;
        }

        vim_regfree(regmatch.regprog);
    }

    // Choose source for result
    let (ret_str, ret_sz) = if !ga.ga_data.is_null() {
        (ga.ga_data as *const c_char, ga.ga_len as usize)
    } else {
        (str as *const c_char, len)
    };

    let ret = xstrnsave(ret_str, ret_sz);
    ga_clear(&mut ga);

    // Restore p_cpo
    let current_cpo = nvim_p_cpo_get();
    if current_cpo == empty {
        // p_cpo was not changed during substitution - simple restore
        nvim_p_cpo_set(save_cpo);
    } else {
        // p_cpo was changed by evaluating {sub} expression or {expr}
        // Use the complex restore path
        nvim_do_string_sub_restore_cpo_complex(save_cpo as *mut c_char);
    }

    if !ret_len.is_null() {
        *ret_len = ret_sz;
    }

    ret
}
