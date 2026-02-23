//! :set command implementation
//!
//! This module provides Rust implementations for the :set command
//! processing including argument parsing, value assignment, and
//! special handling for different option types.

use std::ffi::{c_char, c_int};

use crate::{OptInt, OptScope, SetOp, SetPrefix};

// =============================================================================
// Set Command Context
// =============================================================================

/// Context for :set command execution.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SetContext {
    /// Option scope (global, local, etc.)
    pub scope: c_int,
    /// Set operation type
    pub op: c_int,
    /// Prefix (no, inv)
    pub prefix: c_int,
    /// Whether this is a terminal option
    pub is_tty_opt: bool,
    /// Whether modeline processing
    pub modeline: bool,
    /// Whether to show the option after setting
    pub do_show: bool,
    /// Whether to skip error messages
    pub silent: bool,
    /// Line number for modeline errors
    pub line_number: c_int,
}

impl SetContext {
    /// Create a new set context.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            scope: OptScope::Global as c_int,
            op: SetOp::None as c_int,
            prefix: SetPrefix::None as c_int,
            is_tty_opt: false,
            modeline: false,
            do_show: false,
            silent: false,
            line_number: 0,
        }
    }

    /// Check if context has no prefix.
    #[must_use]
    pub const fn has_no_prefix(&self) -> bool {
        self.prefix == SetPrefix::No as c_int
    }

    /// Check if context has inv prefix.
    #[must_use]
    pub const fn has_inv_prefix(&self) -> bool {
        self.prefix == SetPrefix::Inv as c_int
    }

    /// Check if this is an addition operation.
    #[must_use]
    pub const fn is_adding(&self) -> bool {
        self.op == SetOp::Adding as c_int
    }

    /// Check if this is a prepending operation.
    #[must_use]
    pub const fn is_prepending(&self) -> bool {
        self.op == SetOp::Prepending as c_int
    }

    /// Check if this is a removal operation.
    #[must_use]
    pub const fn is_removing(&self) -> bool {
        self.op == SetOp::Removing as c_int
    }
}

// =============================================================================
// Set Result Types
// =============================================================================

/// Result of a :set operation.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetResult {
    /// Operation succeeded
    Ok = 0,
    /// Unknown option
    UnknownOption = 1,
    /// Invalid argument
    InvalidArg = 2,
    /// Invalid value
    InvalidValue = 3,
    /// Option is read-only
    ReadOnly = 4,
    /// Not allowed in modeline
    NotInModeline = 5,
    /// Not allowed in sandbox
    NotInSandbox = 6,
    /// Requires GUI
    RequiresGui = 7,
    /// Generic failure
    Fail = 99,
}

impl SetResult {
    /// Convert to C integer.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if result indicates success.
    #[must_use]
    pub const fn is_ok(self) -> bool {
        matches!(self, Self::Ok)
    }
}

// =============================================================================
// Boolean Option Handling
// =============================================================================

/// Determine the new value for a boolean option.
///
/// # Arguments
/// * `current` - Current option value
/// * `prefix` - Prefix used (None, No, Inv)
///
/// # Returns
/// The new boolean value
#[must_use]
pub const fn compute_bool_value(current: bool, prefix: SetPrefix) -> bool {
    match prefix {
        SetPrefix::No => false,
        SetPrefix::Inv => !current,
        SetPrefix::None => true,
    }
}

/// FFI: Compute boolean value.
#[no_mangle]
pub extern "C" fn rs_compute_bool_value(current: c_int, prefix: c_int) -> c_int {
    let pref = match prefix {
        1 => SetPrefix::No,
        2 => SetPrefix::Inv,
        _ => SetPrefix::None,
    };
    c_int::from(compute_bool_value(current != 0, pref))
}

// =============================================================================
// Number Option Handling
// =============================================================================

/// Result of parsing a numeric option value.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NumParseResult {
    /// Parsed value
    pub value: OptInt,
    /// Whether parsing succeeded
    pub ok: bool,
    /// Number of characters consumed
    pub len: c_int,
}

/// Parse a numeric value from string.
///
/// Handles:
/// - Decimal: 123
/// - Hex: 0x7F
/// - Octal: 0777
/// - Negative: -123
#[no_mangle]
pub unsafe extern "C" fn rs_parse_num_value(arg: *const c_char) -> NumParseResult {
    if arg.is_null() {
        return NumParseResult {
            value: 0,
            ok: false,
            len: 0,
        };
    }

    let mut p = arg;
    let mut negative = false;
    let mut value: OptInt = 0;
    let mut len: c_int = 0;

    // Check for negative
    if *p as u8 == b'-' {
        negative = true;
        p = p.add(1);
        len += 1;
    } else if *p as u8 == b'+' {
        p = p.add(1);
        len += 1;
    }

    // Check for hex
    if *p as u8 == b'0' && (*p.add(1) as u8 == b'x' || *p.add(1) as u8 == b'X') {
        p = p.add(2);
        len += 2;

        while is_hex_digit(*p as u8) {
            value = value * 16 + OptInt::from(hex_digit_value(*p as u8));
            p = p.add(1);
            len += 1;
        }
    }
    // Check for octal
    else if *p as u8 == b'0' && is_octal_digit(*p.add(1) as u8) {
        p = p.add(1);
        len += 1;

        while is_octal_digit(*p as u8) {
            value = value * 8 + OptInt::from(*p as u8 - b'0');
            p = p.add(1);
            len += 1;
        }
    }
    // Decimal
    else {
        while (*p as u8).is_ascii_digit() {
            value = value * 10 + OptInt::from(*p as u8 - b'0');
            p = p.add(1);
            len += 1;
        }
    }

    if negative {
        value = -value;
    }

    NumParseResult {
        value,
        ok: len > c_int::from(negative || *arg as u8 == b'+'),
        len,
    }
}

/// Check if byte is a hex digit.
#[must_use]
const fn is_hex_digit(c: u8) -> bool {
    c.is_ascii_digit() || matches!(c, b'a'..=b'f' | b'A'..=b'F')
}

/// Get numeric value of hex digit.
#[must_use]
const fn hex_digit_value(c: u8) -> u8 {
    if c.is_ascii_digit() {
        c - b'0'
    } else if matches!(c, b'a'..=b'f') {
        c - b'a' + 10
    } else if matches!(c, b'A'..=b'F') {
        c - b'A' + 10
    } else {
        0
    }
}

/// Check if byte is an octal digit.
#[must_use]
const fn is_octal_digit(c: u8) -> bool {
    matches!(c, b'0'..=b'7')
}

// =============================================================================
// Apply Operations
// =============================================================================

/// Apply a numeric operation to a value.
///
/// # Arguments
/// * `current` - Current value
/// * `operand` - Value to apply
/// * `op` - Operation type
///
/// # Returns
/// New value after operation
#[must_use]
pub const fn apply_num_op(current: OptInt, operand: OptInt, op: SetOp) -> OptInt {
    match op {
        SetOp::Adding => current + operand,
        SetOp::Removing => current - operand,
        SetOp::Prepending => current * operand, // For numbers, prepend means multiply
        SetOp::None => operand,
    }
}

/// FFI: Apply numeric operation.
#[no_mangle]
pub extern "C" fn rs_apply_num_op(current: OptInt, operand: OptInt, op: c_int) -> OptInt {
    let operation = match op {
        1 => SetOp::Adding,
        2 => SetOp::Prepending,
        3 => SetOp::Removing,
        _ => SetOp::None,
    };
    apply_num_op(current, operand, operation)
}

// =============================================================================
// Argument Skipping
// =============================================================================

/// Skip whitespace in argument string.
///
/// # Safety
/// `arg` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_set_skip_whitespace(arg: *const c_char) -> *const c_char {
    if arg.is_null() {
        return arg;
    }

    let mut p = arg;
    while is_ascii_whitespace(*p as u8) {
        p = p.add(1);
    }
    p
}

/// Check if byte is ASCII whitespace.
#[must_use]
const fn is_ascii_whitespace(c: u8) -> bool {
    matches!(c, b' ' | b'\t')
}

/// Skip to next option in argument string.
/// Stops at whitespace or end of string.
///
/// # Safety
/// `arg` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_set_skip_to_whitespace(arg: *const c_char) -> *const c_char {
    if arg.is_null() {
        return arg;
    }

    let mut p = arg;
    while *p != 0 && !is_ascii_whitespace(*p as u8) {
        p = p.add(1);
    }
    p
}

// =============================================================================
// Validation Helpers
// =============================================================================

/// Check if option name character is valid.
/// Option names can contain alphanumeric and underscore.
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub extern "C" fn rs_is_option_name_char(c: c_int) -> c_int {
    let ch = c as u8;
    c_int::from(ch.is_ascii_alphanumeric() || ch == b'_')
}

/// Check if a set argument looks like an option name.
/// Returns 1 if it starts with letter or underscore.
///
/// # Safety
/// `arg` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_is_option_arg(arg: *const c_char) -> c_int {
    if arg.is_null() {
        return 0;
    }
    let c = *arg as u8;
    c_int::from(c.is_ascii_alphabetic() || c == b'_')
}

// =============================================================================
// Set Context FFI
// =============================================================================

/// Create a new set context.
#[no_mangle]
pub extern "C" fn rs_set_context_new() -> SetContext {
    SetContext::new()
}

/// Check if set context indicates success.
#[no_mangle]
pub extern "C" fn rs_set_result_is_ok(result: c_int) -> c_int {
    c_int::from(result == SetResult::Ok as c_int)
}

/// Get error message for set result.
///
/// # Safety
/// Returned pointer is static, do not free.
#[no_mangle]
pub extern "C" fn rs_set_result_errmsg(result: c_int) -> *const c_char {
    static UNKNOWN_OPTION: &[u8] = b"Unknown option\0";
    static INVALID_ARG: &[u8] = b"Invalid argument\0";
    static INVALID_VALUE: &[u8] = b"Invalid value\0";
    static READ_ONLY: &[u8] = b"Option is read-only\0";
    static NOT_IN_MODELINE: &[u8] = b"Not allowed in modeline\0";
    static NOT_IN_SANDBOX: &[u8] = b"Not allowed in sandbox\0";
    static REQUIRES_GUI: &[u8] = b"Option requires GUI\0";
    static EMPTY: &[u8] = b"\0";

    let ptr = match result {
        1 => UNKNOWN_OPTION.as_ptr(),
        2 => INVALID_ARG.as_ptr(),
        3 => INVALID_VALUE.as_ptr(),
        4 => READ_ONLY.as_ptr(),
        5 => NOT_IN_MODELINE.as_ptr(),
        6 => NOT_IN_SANDBOX.as_ptr(),
        7 => REQUIRES_GUI.as_ptr(),
        _ => EMPTY.as_ptr(),
    };
    ptr.cast()
}

// =============================================================================
// String Option New Value (stropt_get_newval migration)
// =============================================================================

#[cfg(not(test))]
extern "C" {
    /// Expand environment variables and ~ in an option value.
    /// Returns a static buffer (NameBuff) if expansion occurred, or NULL.
    fn nvim_option_expand(opt_idx: c_int, val: *const c_char) -> *mut c_char;

    /// Returns the address of the p_kp (keywordprg) global as void*.
    fn nvim_option_get_p_kp_ptr() -> *mut c_void;

    /// xmalloc: allocate memory (C allocator, caller must xfree)
    fn xmalloc(size: usize) -> *mut c_char;

    /// xfree: free memory allocated with xmalloc
    fn xfree(ptr: *mut c_char);

    /// strlen of a C string
    fn strlen(s: *const c_char) -> usize;
}

#[cfg(not(test))]
use crate::getset::{
    rs_stropt_concat_with_comma, rs_stropt_copy_value, rs_stropt_remove_dupflags,
    rs_stropt_remove_val,
};
#[cfg(not(test))]
use crate::parsing::rs_find_dup_item;
#[cfg(not(test))]
use crate::OptFlags;
#[cfg(not(test))]
use std::ffi::c_void;
#[cfg(not(test))]
use std::ptr;

/// Get the string value specified for a `:set` command.
///
/// Full port of C's `stropt_get_newval`. Handles:
/// - `set opt=val` (OP_NONE)
/// - `set opt+=val` (OP_ADDING)
/// - `set opt^=val` (OP_PREPENDING)
/// - `set opt-=val` (OP_REMOVING)
/// - Backslash escaping in values
/// - Environment variable expansion
/// - Duplicate prevention and flag deduplication
/// - 'keywordprg' empty-value special case
///
/// # Returns
/// Newly xmalloc'd string (caller must xfree).
///
/// # Safety
/// All pointer arguments must be valid.
#[cfg(not(test))]
#[no_mangle]
#[allow(clippy::items_after_statements)]
pub unsafe extern "C" fn rs_stropt_get_newval(
    _nextchar: c_int,
    opt_idx: c_int,
    argp: *mut *mut c_char,
    varp: *mut c_void,
    origval: *const c_char,
    op_arg: *mut c_int,
    flags: u32,
) -> *mut c_char {
    static HELP: &[u8] = b":help\0";

    if argp.is_null() || (*argp).is_null() {
        return ptr::null_mut();
    }

    let mut arg = *argp;
    let mut op = *op_arg;
    let mut save_arg: *mut c_char = ptr::null_mut();

    // Jump past the '=' or ':'
    arg = arg.add(1);

    // Set 'keywordprg' to ":help" if an empty value was passed
    let p_kp_ptr = nvim_option_get_p_kp_ptr();
    if varp == p_kp_ptr && (*arg == 0 || (*arg as u8) == b' ') {
        save_arg = arg;
        // Temporarily point arg to the literal ":help" string
        // (safe: we restore it before returning)
        arg = HELP.as_ptr().cast_mut().cast();
    }

    // Convert op integer to SetOp enum for Rust functions
    let set_op = match op {
        1 => SetOp::Adding,
        2 => SetOp::Prepending,
        3 => SetOp::Removing,
        _ => SetOp::None,
    };

    // Copy the new string into allocated memory (C's stropt_copy_value)
    let copy_result = rs_stropt_copy_value(origval, arg, set_op);
    let mut newval = copy_result.value;
    // Update arg to past the parsed value
    arg = copy_result.new_arg.cast_mut();

    // Expand environment variables and ~
    // Don't expand when adding without inserting a comma
    let opt_flags = OptFlags(flags);
    if set_op == SetOp::None || opt_flags.contains(OptFlags::COMMA) {
        // stropt_expand_envvar inline
        let s = nvim_option_expand(opt_idx, newval);
        if !s.is_null() {
            xfree(newval);
            let s_len = strlen(s);
            let mut newlen = s_len + 1;
            if set_op != SetOp::None && !origval.is_null() {
                newlen += strlen(origval) + 1;
            }
            newval = xmalloc(newlen);
            // STRCPY(newval, s)
            ptr::copy_nonoverlapping(s, newval, s_len + 1);
        }
    }

    // Track current op as mutable (may change to None if duplicate found)
    let mut cur_op = set_op;

    // Find duplicate when removing or when NoDup flag is set
    let mut len: usize = 0;
    let mut s: *const c_char = ptr::null();
    if cur_op == SetOp::Removing || opt_flags.contains(OptFlags::NO_DUP) {
        len = strlen(newval);
        s = rs_find_dup_item(origval, newval, len, flags);

        // Do not add if already there
        if (cur_op == SetOp::Adding || cur_op == SetOp::Prepending) && !s.is_null() {
            cur_op = SetOp::None;
            op = SetOp::None as c_int;
            // STRCPY(newval, origval)
            let orig_len = if origval.is_null() {
                0
            } else {
                strlen(origval)
            };
            ptr::copy_nonoverlapping(origval, newval, orig_len + 1);
        }

        // If no duplicate, move pointer to end of original value
        if s.is_null() {
            s = if origval.is_null() {
                newval.cast() // fallback, shouldn't happen
            } else {
                origval.add(strlen(origval))
            };
        }
    }

    // Concatenate the two strings; add a comma if needed
    if cur_op == SetOp::Adding || cur_op == SetOp::Prepending {
        rs_stropt_concat_with_comma(origval, newval, cur_op, flags);
    } else if cur_op == SetOp::Removing {
        #[allow(clippy::cast_possible_truncation)]
        rs_stropt_remove_val(origval, newval, flags, s, len as c_int);
    }

    // Remove duplicate flags for flag-list options
    if opt_flags.contains(OptFlags::FLAG_LIST) {
        rs_stropt_remove_dupflags(newval, flags);
    }

    // Restore arg if it was temporarily changed for keywordprg
    if !save_arg.is_null() {
        arg = save_arg;
    }

    *argp = arg;
    *op_arg = op;

    newval
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_context() {
        let ctx = SetContext::new();
        assert!(!ctx.has_no_prefix());
        assert!(!ctx.has_inv_prefix());
        assert!(!ctx.is_adding());
    }

    #[test]
    fn test_compute_bool_value() {
        assert!(compute_bool_value(false, SetPrefix::None));
        assert!(!compute_bool_value(true, SetPrefix::No));
        assert!(compute_bool_value(false, SetPrefix::Inv));
        assert!(!compute_bool_value(true, SetPrefix::Inv));
    }

    #[test]
    fn test_parse_num_value() {
        unsafe {
            // Decimal
            let result = rs_parse_num_value(c"123".as_ptr());
            assert!(result.ok);
            assert_eq!(result.value, 123);

            // Negative
            let result = rs_parse_num_value(c"-42".as_ptr());
            assert!(result.ok);
            assert_eq!(result.value, -42);

            // Hex
            let result = rs_parse_num_value(c"0xFF".as_ptr());
            assert!(result.ok);
            assert_eq!(result.value, 255);

            // Octal
            let result = rs_parse_num_value(c"0777".as_ptr());
            assert!(result.ok);
            assert_eq!(result.value, 511);
        }
    }

    #[test]
    fn test_apply_num_op() {
        assert_eq!(apply_num_op(10, 5, SetOp::Adding), 15);
        assert_eq!(apply_num_op(10, 3, SetOp::Removing), 7);
        assert_eq!(apply_num_op(10, 2, SetOp::Prepending), 20);
        assert_eq!(apply_num_op(10, 5, SetOp::None), 5);
    }

    #[test]
    fn test_is_option_name_char() {
        assert_eq!(rs_is_option_name_char(c_int::from(b'a')), 1);
        assert_eq!(rs_is_option_name_char(c_int::from(b'Z')), 1);
        assert_eq!(rs_is_option_name_char(c_int::from(b'5')), 1);
        assert_eq!(rs_is_option_name_char(c_int::from(b'_')), 1);
        assert_eq!(rs_is_option_name_char(c_int::from(b'-')), 0);
        assert_eq!(rs_is_option_name_char(c_int::from(b' ')), 0);
    }

    #[test]
    fn test_skip_whitespace() {
        unsafe {
            let arg = c"  hello".as_ptr();
            let result = rs_set_skip_whitespace(arg);
            assert_eq!(*result as u8, b'h');
        }
    }

    #[test]
    fn test_skip_to_whitespace() {
        unsafe {
            let arg = c"hello world".as_ptr();
            let result = rs_set_skip_to_whitespace(arg);
            assert_eq!(*result as u8, b' ');
        }
    }

    #[test]
    fn test_set_result() {
        assert!(SetResult::Ok.is_ok());
        assert!(!SetResult::InvalidArg.is_ok());
    }
}
