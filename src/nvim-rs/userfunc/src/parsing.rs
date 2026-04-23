//! Function argument parsing helpers for VimL.
//!
//! Migrated from `src/nvim/eval/userfunc.c`.
//! Phase 19: get_func_arguments
//! Phase 20: one_function_arg, get_function_args

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::cast_possible_wrap)]

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Constants
// =============================================================================

/// Maximum number of function arguments (matches MAX_FUNC_ARGS in C).
const MAX_FUNC_ARGS: c_int = 20;

/// Size of typval_T in bytes (i32 v_type + i32 v_lock + 8-byte union = 16).
const SIZEOF_TYPVAL: usize = 16;

/// Size of a char* pointer (needed for ga_init of char* arrays).
const SIZEOF_CHARP: c_int = std::mem::size_of::<*mut c_char>() as c_int;

// Return values
const OK: c_int = 1;
const FAIL: c_int = 0;

// =============================================================================
// GarrayT -- must match C's garray_T exactly
// =============================================================================

/// C garray_T: growing array.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct GarrayT {
    pub ga_len: c_int,
    pub ga_maxlen: c_int,
    pub ga_itemsize: c_int,
    pub ga_growsize: c_int,
    pub ga_data: *mut c_void,
}

// =============================================================================
// C Extern Declarations
// =============================================================================

extern "C" {
    /// Skip whitespace characters.
    fn skipwhite(p: *const c_char) -> *mut c_char;

    /// Evaluate level-1 expression.
    fn eval1(arg: *mut *mut c_char, rettv: *mut c_void, evalarg: *mut c_void) -> c_int;

    /// Grow a garray by at least `n` items.
    fn ga_grow(gap: *mut GarrayT, n: c_int);

    /// Initialize a garray.
    fn ga_init(gap: *mut GarrayT, itemsize: c_int, growsize: c_int);

    /// Clear a garray of strings (freeing each entry).
    fn nvim_ga_clear_strings_wrapper(ga: *mut c_void);

    /// Duplicate a string.
    fn xstrdup(s: *const c_char) -> *mut c_char;

    /// Free memory.
    fn xfree(ptr: *mut c_void);

    /// strcmp
    fn strcmp(a: *const c_char, b: *const c_char) -> c_int;

    /// strncmp
    fn strncmp(a: *const c_char, b: *const c_char, n: usize) -> c_int;

    // Error message shims (variadic wrapping is unsafe; use dedicated wrappers)
    fn nvim_semsg_e125_illegal_arg(arg: *const c_char);
    fn nvim_semsg_e853_duplicate_arg(arg: *const c_char);
    fn nvim_emsg_e989_nondefault_follows();
    fn nvim_semsg_no_white_before_comma(p: *const c_char);
    fn nvim_semsg_invarg2(arg: *const c_char);
}

// =============================================================================
// ascii_iswhite -- inline equivalent of C macro
// =============================================================================

/// Returns true if `c` is a whitespace character (space or tab).
#[inline]
fn ascii_iswhite(c: u8) -> bool {
    c == b' ' || c == b'\t'
}

// =============================================================================
// one_function_arg
// =============================================================================

/// Parse one function argument name from `arg`.
///
/// Returns a pointer to after the argument name on success, or `arg` on error.
/// If `newargs` is non-null and not skipping, appends the arg name to the array.
///
/// # Safety
/// - `arg` must be a valid C string
/// - `newargs` may be null or a valid `garray_T *`
unsafe fn rs_one_function_arg_inner(
    arg: *mut c_char,
    newargs: *mut GarrayT,
    skip: bool,
) -> *mut c_char {
    let mut p = arg;

    // Advance p past ASCII alphanumerics and underscores (matches ASCII_ISALNUM(*p) || *p == '_')
    while {
        let c = unsafe { *p as u8 };
        c.is_ascii_alphanumeric() || c == b'_'
    } {
        p = unsafe { p.add(1) };
    }

    let len = (p as usize).wrapping_sub(arg as usize);

    // Error if: no progress, starts with digit, or is "firstline"/"lastline"
    let first_byte = unsafe { *arg as u8 };
    if arg == p
        || first_byte.is_ascii_digit()
        || (len == 9 && unsafe { strncmp(arg, c"firstline".as_ptr(), 9) } == 0)
        || (len == 8 && unsafe { strncmp(arg, c"lastline".as_ptr(), 8) } == 0)
    {
        if !skip {
            unsafe { nvim_semsg_e125_illegal_arg(arg) };
        }
        return arg;
    }

    if !newargs.is_null() {
        unsafe { ga_grow(newargs, 1) };
        let c = unsafe { *p as u8 };
        unsafe { *p = 0i8 }; // NUL-terminate arg name
        let arg_copy = unsafe { xstrdup(arg) };

        // Check for duplicate argument name
        let ga = unsafe { &*newargs };
        for i in 0..ga.ga_len {
            let existing = unsafe { *(ga.ga_data.cast::<*mut c_char>().add(i as usize)) };
            if unsafe { strcmp(existing, arg_copy) } == 0 {
                unsafe { nvim_semsg_e853_duplicate_arg(arg_copy) };
                unsafe { xfree(arg_copy.cast::<c_void>()) };
                unsafe { *p = i8::from_ne_bytes([c]) }; // restore
                return arg;
            }
        }

        // Append the new argument name
        let slot = unsafe { ga.ga_data.cast::<*mut c_char>().add(ga.ga_len as usize) };
        unsafe { *slot = arg_copy };
        unsafe { (*newargs).ga_len += 1 };

        unsafe { *p = i8::from_ne_bytes([c]) }; // restore
    }

    p
}

// =============================================================================
// get_function_args
// =============================================================================

/// Parse function definition argument list "(arg1, arg2, ...)" or "{arg1, arg2 ->".
///
/// - `argp`: on entry points to `(` or `{`; on return points past the closing char.
/// - `endchar`: the closing character (`)` or `-`).
/// - `newargs`: if non-null, receives argument names (initialized here).
/// - `varargs`: if non-null, set to true if `...` was found.
/// - `default_args`: if non-null, receives default expression strings.
/// - `skip`: if true, only find the end without reporting errors.
///
/// Returns OK on success, FAIL on error.
///
/// # Safety
/// All pointers must be valid; evalarg is passed as NULL internally for defaults.
#[unsafe(export_name = "get_function_args")]
pub unsafe extern "C" fn rs_get_function_args(
    argp: *mut *mut c_char,
    endchar: c_char,
    newargs: *mut GarrayT,
    varargs: *mut c_int,
    default_args: *mut GarrayT,
    skip: bool,
) -> c_int {
    let end = endchar as u8;
    let mut mustend = false;
    let mut arg = unsafe { *argp };
    let mut p = arg;

    if !newargs.is_null() {
        unsafe { ga_init(newargs, SIZEOF_CHARP, 3) };
    }
    if !default_args.is_null() {
        unsafe { ga_init(default_args, SIZEOF_CHARP, 3) };
    }
    if !varargs.is_null() {
        unsafe { *varargs = 0 }; // false
    }

    let mut any_default = false;

    while unsafe { *p as u8 } != end {
        // Handle variadic "..."
        if unsafe { *p as u8 } == b'.'
            && unsafe { *p.add(1) as u8 } == b'.'
            && unsafe { *p.add(2) as u8 } == b'.'
        {
            if !varargs.is_null() {
                unsafe { *varargs = 1 }; // true
            }
            p = unsafe { p.add(3) };
            mustend = true;
        } else {
            arg = p;
            p = unsafe { rs_one_function_arg_inner(p, newargs, skip) };
            if p == arg {
                break;
            }

            // Check for '=' (default value)
            let sw_p = unsafe { skipwhite(p) };
            if unsafe { *sw_p as u8 } == b'=' && !default_args.is_null() {
                any_default = true;
                p = unsafe { sw_p.add(1) }; // skip '='
                p = unsafe { skipwhite(p) };
                let expr_start = p;

                // Evaluate the default expression (evalarg = NULL)
                let mut rettv = [0u8; SIZEOF_TYPVAL];
                let eval_result = unsafe {
                    eval1(
                        &raw mut p,
                        rettv.as_mut_ptr().cast::<c_void>(),
                        std::ptr::null_mut(),
                    )
                };

                if eval_result == OK {
                    unsafe { ga_grow(default_args, 1) };

                    // trim trailing whitespace
                    while p > expr_start && ascii_iswhite(unsafe { *p.sub(1) as u8 }) {
                        p = unsafe { p.sub(1) };
                    }
                    let c = unsafe { *p as u8 };
                    unsafe { *p = 0i8 }; // NUL-terminate
                    let expr_copy = unsafe { xstrdup(expr_start) };
                    let slot = unsafe {
                        let ga = &*default_args;
                        ga.ga_data.cast::<*mut c_char>().add(ga.ga_len as usize)
                    };
                    unsafe { *slot = expr_copy };
                    unsafe { (*default_args).ga_len += 1 };
                    unsafe { *p = i8::from_ne_bytes([c]) }; // restore
                } else {
                    mustend = true;
                }
            } else if any_default {
                unsafe { nvim_emsg_e989_nondefault_follows() };
                mustend = true;
            }

            // Check for whitespace before comma (not tolerated unless skipping)
            if ascii_iswhite(unsafe { *p as u8 }) {
                let sw2 = unsafe { skipwhite(p) };
                if unsafe { *sw2 as u8 } == b',' {
                    if !skip {
                        unsafe { nvim_semsg_no_white_before_comma(p) };
                        return rs_get_function_args_err(newargs, default_args);
                    }
                    p = sw2;
                }
            }
            if unsafe { *p as u8 } == b',' {
                p = unsafe { p.add(1) };
            } else {
                mustend = true;
            }
        }

        p = unsafe { skipwhite(p) };
        if mustend && unsafe { *p as u8 } != end {
            if !skip {
                unsafe { nvim_semsg_invarg2(*argp) };
            }
            break;
        }
    }

    if unsafe { *p as u8 } != end {
        return rs_get_function_args_err(newargs, default_args);
    }
    p = unsafe { p.add(1) }; // skip endchar

    unsafe { *argp = p };
    OK
}

/// Helper: clear arrays and return FAIL (replaces `goto err_ret`).
unsafe fn rs_get_function_args_err(newargs: *mut GarrayT, default_args: *mut GarrayT) -> c_int {
    if !newargs.is_null() {
        unsafe { nvim_ga_clear_strings_wrapper(newargs.cast::<c_void>()) };
    }
    if !default_args.is_null() {
        unsafe { nvim_ga_clear_strings_wrapper(default_args.cast::<c_void>()) };
    }
    FAIL
}

// =============================================================================
// get_func_arguments
// =============================================================================

/// Get the arguments for a function call from an argument string.
///
/// Parses "arg1, arg2, ...)" from `*arg`, evaluating each expression and storing
/// the result in `argvars[*argcount .. MAX_FUNC_ARGS - partial_argc]`.
///
/// On return, `*arg` points past the closing `)` (or at the failure point).
/// Returns OK on success, FAIL on parse error.
///
/// # Safety
/// - `*arg` must be a valid C string pointing to a `(` or `,` character
/// - `evalarg` may be NULL
/// - `argvars` must have room for at least `MAX_FUNC_ARGS + 1` typval_T values
/// - `argcount` must be a valid pointer
#[unsafe(export_name = "get_func_arguments")]
pub unsafe extern "C" fn rs_get_func_arguments(
    arg: *mut *mut c_char,
    evalarg: *mut c_void,
    partial_argc: c_int,
    argvars: *mut c_void,
    argcount: *mut c_int,
) -> c_int {
    let mut argp = unsafe { *arg };
    let mut ret = OK;

    // Get the arguments.
    while unsafe { *argcount } < MAX_FUNC_ARGS - partial_argc {
        // skip the '(' or ','
        argp = unsafe { skipwhite(argp.add(1)) };

        let ch = unsafe { *argp as u8 };
        if ch == b')' || ch == b',' || ch == b'\0' {
            break;
        }

        // Compute pointer to argvars[*argcount]
        let argcount_idx = unsafe { *argcount } as usize;
        let rettv_slot =
            unsafe { argvars.cast::<u8>().add(argcount_idx * SIZEOF_TYPVAL) }.cast::<c_void>();

        if unsafe { eval1(&raw mut argp, rettv_slot, evalarg) } == FAIL {
            ret = FAIL;
            break;
        }
        unsafe { *argcount += 1 };
        if unsafe { *argp as u8 } != b',' {
            break;
        }
    }

    argp = unsafe { skipwhite(argp) };
    if unsafe { *argp as u8 } == b')' {
        argp = unsafe { argp.add(1) };
    } else {
        ret = FAIL;
    }
    unsafe { *arg = argp };
    ret
}
