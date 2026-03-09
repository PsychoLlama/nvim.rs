//! Option get/set core functionality
//!
//! This module provides Rust implementations of core option value manipulation
//! functions used by the `:set` command processing.

#![allow(clippy::missing_safety_doc)] // FFI functions safety is implicit
#![allow(clippy::cast_possible_wrap)] // FFI with C char types

use std::ffi::{c_char, c_int, c_uint};
use std::ptr;

use crate::{OptFlags, SetOp, SetPrefix};

// =============================================================================
// C Function Declarations
// =============================================================================

#[allow(dead_code)] // FFI functions used when linked with C
extern "C" {
    fn xmalloc(size: usize) -> *mut c_char;
    fn xfree(ptr: *mut c_char);
    fn rs_ascii_iswhite(c: c_int) -> c_int;
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn vim_strchr(s: *const c_char, c: c_int) -> *mut c_char;
}

// =============================================================================
// Operator Parsing
// =============================================================================

/// Parse the :set operator from an argument string.
///
/// Recognizes:
/// - `+=` (adding)
/// - `^=` (prepending)
/// - `-=` (removing)
///
/// Returns the operator type.
#[no_mangle]
pub unsafe extern "C" fn rs_get_op(arg: *const c_char) -> SetOp {
    if arg.is_null() || *arg == 0 {
        return SetOp::None;
    }

    let c0 = *arg as u8;
    let c1 = *arg.add(1) as u8;

    if c1 == b'=' {
        match c0 {
            b'+' => SetOp::Adding,
            b'^' => SetOp::Prepending,
            b'-' => SetOp::Removing,
            _ => SetOp::None,
        }
    } else {
        SetOp::None
    }
}

/// Parse the option prefix from an argument string.
///
/// Recognizes:
/// - `no` prefix (e.g., `:set nonumber`)
/// - `inv` prefix (e.g., `:set invnumber`)
///
/// Advances the argument pointer past the prefix if found.
/// Returns the prefix type.
#[no_mangle]
pub unsafe extern "C" fn rs_get_option_prefix(argp: *mut *mut c_char) -> SetPrefix {
    if argp.is_null() || (*argp).is_null() {
        return SetPrefix::None;
    }

    let arg = *argp;

    // Check for "no" prefix
    if *arg as u8 == b'n' && *arg.add(1) as u8 == b'o' {
        *argp = arg.add(2);
        return SetPrefix::No;
    }

    // Check for "inv" prefix
    if *arg as u8 == b'i' && *arg.add(1) as u8 == b'n' && *arg.add(2) as u8 == b'v' {
        *argp = arg.add(3);
        return SetPrefix::Inv;
    }

    SetPrefix::None
}

// =============================================================================
// String Value Copying
// =============================================================================

/// Result of copying a string option value.
#[repr(C)]
pub struct CopyValueResult {
    /// The newly allocated string value (caller must free)
    pub value: *mut c_char,
    /// Updated argument pointer (past the parsed value)
    pub new_arg: *const c_char,
}

/// Copy a string option value from the argument.
///
/// Handles escape sequences and allocates a new string.
/// The caller is responsible for freeing the returned value.
///
/// # Arguments
/// * `origval` - The original option value (for sizing with operators)
/// * `arg` - The argument string to parse
/// * `op` - The operator being used
///
/// # Returns
/// A struct containing the new value and updated argument pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_stropt_copy_value(
    origval: *const c_char,
    arg: *const c_char,
    op: SetOp,
) -> CopyValueResult {
    let mut result = CopyValueResult {
        value: ptr::null_mut(),
        new_arg: arg,
    };

    if arg.is_null() {
        return result;
    }

    // Calculate length needed (get a bit too much)
    let mut arg_len: usize = 0;
    let mut p = arg;
    while *p != 0 {
        arg_len += 1;
        p = p.add(1);
    }
    let mut newlen = arg_len + 1;

    if op != SetOp::None && !origval.is_null() {
        let mut orig_len: usize = 0;
        p = origval;
        while *p != 0 {
            orig_len += 1;
            p = p.add(1);
        }
        newlen += orig_len + 1;
    }

    let newval = xmalloc(newlen);
    if newval.is_null() {
        return result;
    }

    let mut s = newval;
    let mut a = arg;

    // Copy the string, skip over escaped chars
    while *a != 0 && rs_ascii_iswhite(c_int::from(*a)) == 0 {
        if *a as u8 == b'\\' && *a.add(1) != 0 {
            // Skip backslash (remove escape)
            a = a.add(1);
        }

        let char_len = utfc_ptr2len(a);
        if char_len > 1 {
            // Copy multibyte char
            for _ in 0..char_len {
                *s = *a;
                s = s.add(1);
                a = a.add(1);
            }
        } else {
            *s = *a;
            s = s.add(1);
            a = a.add(1);
        }
    }
    *s = 0;

    result.value = newval;
    result.new_arg = a;
    result
}

// =============================================================================
// Comma-Separated List Operations
// =============================================================================

/// Concatenate original and new values with a comma if needed.
///
/// Modifies `newval` in place. Handles both adding (orig + new) and
/// prepending (new + orig) operations.
///
/// # Arguments
/// * `origval` - The original option value
/// * `newval` - The new value (will be modified in place)
/// * `newval_size` - Size of the newval buffer
/// * `op` - The operator being used
/// * `flags` - Option flags (comma handling)
#[no_mangle]
pub unsafe extern "C" fn rs_stropt_concat_with_comma(
    origval: *const c_char,
    newval: *mut c_char,
    op: SetOp,
    flags: c_uint,
) {
    if origval.is_null() || newval.is_null() {
        return;
    }

    let opt_flags = OptFlags(flags);
    let is_comma = opt_flags.contains(OptFlags::COMMA);
    let is_one_comma = opt_flags.contains(OptFlags::ONE_COMMA);

    // Get lengths
    let mut orig_len: usize = 0;
    let mut p = origval;
    while *p != 0 {
        orig_len += 1;
        p = p.add(1);
    }

    let mut new_len: usize = 0;
    p = newval;
    while *p != 0 {
        new_len += 1;
        p = p.add(1);
    }

    // Determine if we need a comma
    let need_comma = is_comma && orig_len > 0 && new_len > 0;

    match op {
        SetOp::Adding => {
            // origval + comma + newval
            let mut len = orig_len;

            // Strip trailing comma to avoid double comma
            if need_comma && is_one_comma && len > 1 {
                let last = *origval.add(len - 1) as u8;
                let prev = *origval.add(len - 2) as u8;
                if last == b',' && prev != b'\\' {
                    len -= 1;
                }
            }

            // Move newval to make room for origval
            let comma_offset = usize::from(need_comma);
            ptr::copy(newval, newval.add(len + comma_offset), new_len + 1);

            // Copy origval to the beginning
            ptr::copy_nonoverlapping(origval, newval, len);

            // Add comma if needed
            if need_comma {
                *newval.add(len) = b',' as c_char;
            }
        }
        SetOp::Prepending => {
            // newval + comma + origval
            let comma_offset = usize::from(need_comma);

            // Add comma after newval
            if need_comma {
                *newval.add(new_len) = b',' as c_char;
            }

            // Copy origval after comma
            ptr::copy_nonoverlapping(origval, newval.add(new_len + comma_offset), orig_len + 1);
        }
        _ => {}
    }
}

/// Remove a value from a comma-separated list.
///
/// Copies origval to newval, then removes the specified substring.
///
/// # Arguments
/// * `origval` - The original option value
/// * `newval` - Buffer to store the result
/// * `flags` - Option flags (comma handling)
/// * `strval` - Pointer into origval where the value to remove starts
/// * `len` - Length of the value to remove
#[no_mangle]
pub unsafe extern "C" fn rs_stropt_remove_val(
    origval: *const c_char,
    newval: *mut c_char,
    flags: c_uint,
    strval: *const c_char,
    len: c_int,
) {
    if origval.is_null() || newval.is_null() || strval.is_null() {
        return;
    }

    let opt_flags = OptFlags(flags);
    let is_comma = opt_flags.contains(OptFlags::COMMA);
    let mut remove_len = len as usize;
    let mut remove_start = strval;

    // Copy origval to newval first
    let mut orig_len: usize = 0;
    let mut p = origval;
    while *p != 0 {
        orig_len += 1;
        p = p.add(1);
    }
    ptr::copy_nonoverlapping(origval, newval, orig_len + 1);

    // If value to remove is empty, we're done
    if *strval == 0 {
        return;
    }

    // Handle comma removal
    if is_comma {
        if strval == origval {
            // Value is at the start - remove trailing comma if present
            if *strval.add(remove_len) as u8 == b',' {
                remove_len += 1;
            }
        } else {
            // Value is in the middle/end - remove leading comma
            remove_start = strval.offset(-1);
            remove_len += 1;
        }
    }

    // Calculate offset in newval and perform removal
    let offset = remove_start.offset_from(origval) as usize;
    let remaining_start = remove_start.add(remove_len);
    let mut remaining_len: usize = 0;
    p = remaining_start;
    while *p != 0 {
        remaining_len += 1;
        p = p.add(1);
    }

    ptr::copy(
        newval.add(offset + remove_len),
        newval.add(offset),
        remaining_len + 1,
    );
}

/// Remove duplicate flags from a flag-list option value.
///
/// For options like 'cpoptions' where each character is a flag,
/// removes any flags that appear more than once.
///
/// # Arguments
/// * `newval` - The value to modify in place
/// * `flags` - Option flags
#[no_mangle]
pub unsafe extern "C" fn rs_stropt_remove_dupflags(newval: *mut c_char, flags: c_uint) {
    if newval.is_null() {
        return;
    }

    let opt_flags = OptFlags(flags);
    let is_one_comma = opt_flags.contains(OptFlags::ONE_COMMA);
    let is_comma = opt_flags.contains(OptFlags::COMMA);

    let mut s = newval;

    while *s != 0 {
        let c = *s as u8;

        if is_one_comma {
            // For options like 'whichwrap': flag,flag,flag
            // Each flag is a single char followed by comma
            if c != b',' && *s.add(1) as u8 == b',' {
                // Check if this flag appears later
                if !vim_strchr(s.add(2), c_int::from(c)).is_null() {
                    // Remove this flag and the following comma
                    let mut remaining_len: usize = 0;
                    let mut p = s.add(2);
                    while *p != 0 {
                        remaining_len += 1;
                        p = p.add(1);
                    }
                    ptr::copy(s.add(2), s, remaining_len + 1);
                    continue;
                }
            }
        } else {
            // For options like 'cpoptions': consecutive flags
            // Skip commas in comma-separated lists
            if !is_comma || c != b',' {
                // Check if this flag appears later
                if !vim_strchr(s.add(1), c_int::from(c)).is_null() {
                    // Remove this flag
                    let mut remaining_len: usize = 0;
                    let mut p = s.add(1);
                    while *p != 0 {
                        remaining_len += 1;
                        p = p.add(1);
                    }
                    ptr::copy(s.add(1), s, remaining_len + 1);
                    continue;
                }
            }
        }

        s = s.add(1);
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_get_op() {
        unsafe {
            let plus_eq = CString::new("+=value").unwrap();
            let caret_eq = CString::new("^=value").unwrap();
            let minus_eq = CString::new("-=value").unwrap();
            let just_eq = CString::new("=value").unwrap();
            let no_op = CString::new("value").unwrap();

            assert_eq!(rs_get_op(plus_eq.as_ptr()), SetOp::Adding);
            assert_eq!(rs_get_op(caret_eq.as_ptr()), SetOp::Prepending);
            assert_eq!(rs_get_op(minus_eq.as_ptr()), SetOp::Removing);
            assert_eq!(rs_get_op(just_eq.as_ptr()), SetOp::None);
            assert_eq!(rs_get_op(no_op.as_ptr()), SetOp::None);
            assert_eq!(rs_get_op(ptr::null()), SetOp::None);
        }
    }

    #[test]
    fn test_get_option_prefix() {
        unsafe {
            // Test "no" prefix
            let no_number = CString::new("nonumber").unwrap();
            let mut arg = no_number.as_ptr().cast_mut();
            let prefix = rs_get_option_prefix(&raw mut arg);
            assert_eq!(prefix, SetPrefix::No);
            assert_eq!(*arg as u8, b'n'); // Should point to "number"

            // Test "inv" prefix
            let inv_number = CString::new("invnumber").unwrap();
            arg = inv_number.as_ptr().cast_mut();
            let prefix = rs_get_option_prefix(&raw mut arg);
            assert_eq!(prefix, SetPrefix::Inv);
            assert_eq!(*arg as u8, b'n'); // Should point to "number"

            // Test no prefix
            let just_number = CString::new("number").unwrap();
            arg = just_number.as_ptr().cast_mut();
            let prefix = rs_get_option_prefix(&raw mut arg);
            assert_eq!(prefix, SetPrefix::None);
            assert_eq!(*arg as u8, b'n'); // Should still point to "number"
        }
    }
}
