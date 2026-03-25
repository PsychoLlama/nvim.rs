//! VimL function implementations for command-line operations
//!
//! This module provides Rust implementations of VimL built-in functions
//! related to the command line: getcmdline(), getcmdpos(), getcmdtype(),
//! getcmdprompt(), getcmdscreenpos(), and the get_list_range() helper.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Type Aliases
// =============================================================================

/// Opaque pointer to typval_T (VimL value)
type TypvalPtr = *mut c_void;

/// EvalFuncData (unused callback data)
type EvalFuncData = *mut c_void;

// =============================================================================
// VarType constants (from typval_defs.h)
// =============================================================================

/// VAR_STRING - string value
const VAR_STRING: c_int = 2;

// =============================================================================
// MODE constant
// =============================================================================

/// MODE_CMDLINE - editing the command line (from state_defs.h)
const MODE_CMDLINE: c_int = 0x08;

// =============================================================================
// Return value constants
// =============================================================================

/// OK return value (FAIL = 0, OK = 1 in C)
const OK: c_int = 1;

/// FAIL return value
const FAIL: c_int = 0;

/// INT_MAX for overflow check
const INT_MAX: i64 = i32::MAX as i64;

// =============================================================================
// C Function Declarations
// =============================================================================

extern "C" {
    static mut State: c_int;
    fn nvim_get_cmdline_star() -> c_int;

    // ccline field accessors
    fn nvim_get_ccline_cmdbuff() -> *mut c_char;
    fn nvim_get_ccline_cmdlen() -> c_int;
    fn nvim_get_ccline_cmdpos() -> c_int;
    fn nvim_get_ccline_cmdspos() -> c_int;
    fn nvim_get_ccline_cmdfirstc() -> c_int;
    fn nvim_get_ccline_input_fn() -> c_int;
    fn nvim_get_ccline_cmdprompt() -> *mut c_char;

    // Memory allocation
    fn xstrnsave(string: *const c_char, len: usize) -> *mut c_char;
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn xmallocz(size: usize) -> *mut c_void;

    // String parsing
    fn skipwhite(s: *const c_char) -> *mut c_char;
    fn vim_str2nr(
        start: *const c_char,
        prep: *mut *mut c_char,
        len: *mut c_int,
        what: c_int,
        nptr: *mut i64,
        unptr: *mut u64,
        maxlen: c_int,
        strict: bool,
        overflow: *mut bool,
    );

    fn nvim_tv_set_number(tv: TypvalPtr, n: i64);
    fn nvim_tv_set_vstring_owned(tv: TypvalPtr, s: *mut c_char);
}

/// Set v_type in a typval_T pointer (v_type is at offset 0).
/// Inlined from nvim_tv_set_type.
#[inline]
unsafe fn tv_set_type(tv: TypvalPtr, vtype: c_int) {
    if !tv.is_null() {
        *tv.cast::<c_int>() = vtype;
    }
}

// =============================================================================
// get_ccline_ptr() equivalent in Rust
// =============================================================================

/// Check if we are in cmdline mode with a valid command buffer.
///
/// This is a Rust approximation of C's `get_ccline_ptr()`.
/// It skips the `prev_ccline` fallback (rare nested-cmdline edge case).
///
/// Returns true if: State has `MODE_CMDLINE` set AND cmdbuff is non-NULL.
///
/// # Safety
///
/// Calls C functions to read global state.
unsafe fn in_active_cmdline() -> bool {
    let state = State;
    if state & MODE_CMDLINE == 0 {
        return false;
    }
    !nvim_get_ccline_cmdbuff().is_null()
}

// =============================================================================
// get_cmdline_type() equivalent
// =============================================================================

/// Get the current command-line type character.
///
/// Returns ':' or '/' or '?' or '@' or '>' or '-', or NUL when not in cmdline.
///
/// # Safety
///
/// Calls C functions to access global state.
unsafe fn get_cmdline_type_char() -> c_int {
    if !in_active_cmdline() {
        return 0; // NUL
    }
    let cmdfirstc = nvim_get_ccline_cmdfirstc();
    if cmdfirstc == 0 {
        // NUL cmdfirstc - check if this is an input function
        if nvim_get_ccline_input_fn() != 0 {
            c_int::from(b'@')
        } else {
            c_int::from(b'-')
        }
    } else {
        cmdfirstc
    }
}

// =============================================================================
// VimL: getcmdline()
// =============================================================================

/// "getcmdline()" function.
///
/// # Safety
///
/// `rettv` must be a valid typval_T pointer.
#[export_name = "f_getcmdline"]
pub unsafe extern "C" fn rs_f_getcmdline(
    _argvars: TypvalPtr,
    rettv: TypvalPtr,
    _fptr: EvalFuncData,
) {
    tv_set_type(rettv, VAR_STRING);
    // Return NULL (VAR_STRING with null = empty result) when:
    // - password mode is active, or
    // - not in active cmdline
    if nvim_get_cmdline_star() > 0 || !in_active_cmdline() {
        nvim_tv_set_vstring_owned(rettv, std::ptr::null_mut());
        return;
    }
    let cmdbuff = nvim_get_ccline_cmdbuff();
    let cmdlen = nvim_get_ccline_cmdlen() as usize;
    let s = xstrnsave(cmdbuff, cmdlen);
    nvim_tv_set_vstring_owned(rettv, s);
}

// =============================================================================
// VimL: getcmdpos()
// =============================================================================

/// "getcmdpos()" function.
///
/// Returns 1-based cursor position on the command line.
///
/// # Safety
///
/// `rettv` must be a valid typval_T pointer.
#[export_name = "f_getcmdpos"]
pub unsafe extern "C" fn rs_f_getcmdpos(
    _argvars: TypvalPtr,
    rettv: TypvalPtr,
    _fptr: EvalFuncData,
) {
    // Default: 0 if not in cmdline
    let pos = if in_active_cmdline() {
        // Convert 0-based internal to 1-based Vim position
        nvim_get_ccline_cmdpos() + 1
    } else {
        0
    };
    nvim_tv_set_number(rettv, i64::from(pos));
}

// =============================================================================
// VimL: getcmdprompt()
// =============================================================================

/// "getcmdprompt()" function.
///
/// Returns the prompt string for the current cmdline, or null string.
///
/// # Safety
///
/// `rettv` must be a valid typval_T pointer.
#[export_name = "f_getcmdprompt"]
pub unsafe extern "C" fn rs_f_getcmdprompt(
    _argvars: TypvalPtr,
    rettv: TypvalPtr,
    _fptr: EvalFuncData,
) {
    tv_set_type(rettv, VAR_STRING);
    if !in_active_cmdline() {
        nvim_tv_set_vstring_owned(rettv, std::ptr::null_mut());
        return;
    }
    let prompt = nvim_get_ccline_cmdprompt();
    let s = if prompt.is_null() {
        std::ptr::null_mut()
    } else {
        xstrdup(prompt)
    };
    nvim_tv_set_vstring_owned(rettv, s);
}

// =============================================================================
// VimL: getcmdscreenpos()
// =============================================================================

/// "getcmdscreenpos()" function.
///
/// Returns 1-based screen column of the cursor on the command line.
///
/// # Safety
///
/// `rettv` must be a valid typval_T pointer.
#[export_name = "f_getcmdscreenpos"]
pub unsafe extern "C" fn rs_f_getcmdscreenpos(
    _argvars: TypvalPtr,
    rettv: TypvalPtr,
    _fptr: EvalFuncData,
) {
    // Default: 0 if not in cmdline
    let pos = if in_active_cmdline() {
        // Convert 0-based internal to 1-based Vim position
        nvim_get_ccline_cmdspos() + 1
    } else {
        0
    };
    nvim_tv_set_number(rettv, i64::from(pos));
}

// =============================================================================
// VimL: getcmdtype()
// =============================================================================

/// "getcmdtype()" function.
///
/// Returns a string containing the cmdline type character.
///
/// # Safety
///
/// `rettv` must be a valid typval_T pointer.
#[export_name = "f_getcmdtype"]
pub unsafe extern "C" fn rs_f_getcmdtype(
    _argvars: TypvalPtr,
    rettv: TypvalPtr,
    _fptr: EvalFuncData,
) {
    tv_set_type(rettv, VAR_STRING);
    // Allocate a 1-byte string (xmallocz gives us 1 byte + NUL)
    let buf = xmallocz(1).cast::<c_char>();
    if buf.is_null() {
        nvim_tv_set_vstring_owned(rettv, std::ptr::null_mut());
        return;
    }
    let c = get_cmdline_type_char();
    // Write the character (safe: buf is valid, allocated with room for 1 byte)
    *buf = c as u8 as c_char;
    nvim_tv_set_vstring_owned(rettv, buf);
}

// =============================================================================
// get_list_range()
// =============================================================================

/// Get indices that specify a range within a list from a string.
///
/// Used for `:history` and `:clist`.
///
/// On success, returns OK and sets `*num1` (from) and `*num2` (to).
/// On failure, returns FAIL.
///
/// # Safety
///
/// - `str` must be a valid pointer to a mutable pointer to a NUL-terminated C string.
/// - `num1` and `num2` must be valid pointers to `c_int`.
#[must_use]
#[export_name = "get_list_range"]
pub unsafe extern "C" fn rs_get_list_range(
    str: *mut *mut c_char,
    num1: *mut c_int,
    num2: *mut c_int,
) -> c_int {
    // Skip leading whitespace
    *str = skipwhite((*str).cast_const());

    let mut first = false;

    // Parse "from" part of range
    let first_char = (**str) as u8;
    if first_char == b'-' || first_char.is_ascii_digit() {
        let mut len: c_int = 0;
        let mut num: i64 = 0;
        vim_str2nr(
            (*str).cast_const(),
            std::ptr::null_mut(),
            &raw mut len,
            0,
            &raw mut num,
            std::ptr::null_mut(),
            0,
            false,
            std::ptr::null_mut(),
        );
        *str = (*str).add(len as usize);
        // overflow check
        if num > INT_MAX {
            return FAIL;
        }
        #[allow(clippy::cast_possible_truncation)]
        {
            *num1 = num as c_int;
        }
        first = true;
    }

    // Skip whitespace before potential comma
    *str = skipwhite((*str).cast_const());

    // Parse "to" part of range
    if (**str) == b',' as c_char {
        *str = skipwhite((*str).add(1).cast_const());

        let mut len: c_int = 0;
        let mut num: i64 = 0;
        vim_str2nr(
            (*str).cast_const(),
            std::ptr::null_mut(),
            &raw mut len,
            0,
            &raw mut num,
            std::ptr::null_mut(),
            0,
            false,
            std::ptr::null_mut(),
        );
        if len > 0 {
            *str = skipwhite((*str).add(len as usize).cast_const());
            // overflow check
            if num > INT_MAX {
                return FAIL;
            }
            #[allow(clippy::cast_possible_truncation)]
            {
                *num2 = num as c_int;
            }
        } else if !first {
            // no number given at all
            return FAIL;
        }
    } else if first {
        // only one number given: set both to same
        *num2 = *num1;
    }

    OK
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_var_string_constant() {
        assert_eq!(VAR_STRING, 2);
    }

    #[test]
    fn test_mode_cmdline() {
        assert_eq!(MODE_CMDLINE, 0x08);
    }

    #[test]
    fn test_int_max() {
        assert_eq!(INT_MAX, i32::MAX as i64);
    }

    #[test]
    fn test_ok_fail_constants() {
        assert_eq!(OK, 1);
        assert_eq!(FAIL, 0);
    }
}
