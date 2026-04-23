//! Function argument parsing helpers for VimL.
//!
//! Migrated from `src/nvim/eval/userfunc.c` Phase 19.
//! Covers: get_func_arguments

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Constants
// =============================================================================

/// Maximum number of function arguments (matches MAX_FUNC_ARGS in C).
const MAX_FUNC_ARGS: c_int = 20;

/// Size of typval_T in bytes (i32 v_type + i32 v_lock + 8-byte union = 16).
const SIZEOF_TYPVAL: usize = 16;

// Return values
const OK: c_int = 1;
const FAIL: c_int = 0;

// =============================================================================
// C Extern Declarations
// =============================================================================

extern "C" {
    /// Skip whitespace characters.
    fn skipwhite(p: *const c_char) -> *mut c_char;

    /// Evaluate level-1 expression.
    fn eval1(arg: *mut *mut c_char, rettv: *mut c_void, evalarg: *mut c_void) -> c_int;
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
