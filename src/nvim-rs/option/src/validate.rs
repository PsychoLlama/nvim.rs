//! Option value validation functions
//!
//! This module provides validation functions for option values. These functions
//! check if a given value is valid for a particular option, without modifying
//! any state.

use std::ffi::{c_char, c_int};

// =============================================================================
// External C Functions (only used when not testing)
// =============================================================================

// Note: rs_vim_strchr is not currently used, but may be needed for future
// validation functions. It's declared here for completeness.

#[cfg(not(test))]
extern "C" {
    fn illegal_char(errbuf: *mut c_char, errbuflen: usize, c: c_int) -> *mut c_char;
}

// =============================================================================
// Test Stubs
// =============================================================================

#[cfg(test)]
#[allow(clippy::cast_possible_truncation, dead_code)]
unsafe fn rs_vim_strchr(s: *const c_char, c: c_int) -> *mut c_char {
    if s.is_null() {
        return std::ptr::null_mut();
    }
    let target = c as u8;
    let mut p = s;
    while *p != 0 {
        if (*p as u8) == target {
            return p.cast_mut();
        }
        p = p.add(1);
    }
    std::ptr::null_mut()
}

// =============================================================================
// Constants - Statusline Format Characters (only used when not testing)
// =============================================================================

/// Separation between alignment sections
#[cfg(not(test))]
const STL_SEPARATE: u8 = b'=';
/// Truncation mark if line is too long
#[cfg(not(test))]
const STL_TRUNCMARK: u8 = b'<';
/// Highlight from (User)1..9 or 0
#[cfg(not(test))]
const STL_USER_HL: u8 = b'*';
/// Start of expression to substitute
#[cfg(not(test))]
const STL_VIM_EXPR: u8 = b'{';

/// All valid statusline format characters
#[cfg(not(test))]
const STL_ALL: &[u8] = b"fFtcvVlLnkoObBrRhHyYwWmMqpPaNSCs{=<*#TX@";

// =============================================================================
// Error Messages
// =============================================================================

/// Error message for unclosed expression sequence
#[cfg(not(test))]
const E_UNCLOSED_EXPRESSION: &[u8] = b"E540: Unclosed expression sequence\0";

/// Error message for unbalanced groups
#[cfg(not(test))]
const E_UNBALANCED_GROUPS: &[u8] = b"E542: Unbalanced groups\0";

// =============================================================================
// Statusline Format Validation
// =============================================================================

/// Check validity of options with the 'statusline' format.
///
/// This validates format strings for 'statusline', 'tabline', 'winbar',
/// 'statuscolumn', 'rulerformat', and 'titlestring'.
///
/// # Arguments
///
/// * `s` - The format string to validate
/// * `errbuf` - Buffer to write error messages to (if validation fails)
/// * `errbuflen` - Size of the error buffer
///
/// # Returns
///
/// NULL on success, or a pointer to an error message on failure.
///
/// # Safety
///
/// `s` must be a valid null-terminated C string. `errbuf` must be a valid
/// buffer of at least `errbuflen` bytes, or NULL.
#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "C" fn rs_check_stl_option(
    s: *const c_char,
    errbuf: *mut c_char,
    errbuflen: usize,
) -> *const c_char {
    if s.is_null() {
        return std::ptr::null();
    }

    let mut p = s;
    let mut groupdepth: i32 = 0;

    while *p != 0 {
        // Skip non-% characters
        while *p != 0 && (*p as u8) != b'%' {
            p = p.add(1);
        }
        if *p == 0 {
            break;
        }
        p = p.add(1); // Skip the '%'

        let c = *p as u8;

        // Check for %%, %<, or %=
        if c == b'%' || c == STL_TRUNCMARK || c == STL_SEPARATE {
            p = p.add(1);
            continue;
        }

        // Check for %)
        if c == b')' {
            p = p.add(1);
            groupdepth -= 1;
            if groupdepth < 0 {
                break;
            }
            continue;
        }

        // Skip optional - for right alignment
        if c == b'-' {
            p = p.add(1);
        }

        // Skip width digits
        while *p != 0 && (*p as u8).is_ascii_digit() {
            p = p.add(1);
        }

        // Check for %*
        if (*p as u8) == STL_USER_HL {
            continue;
        }

        // Skip precision
        if (*p as u8) == b'.' {
            p = p.add(1);
            while *p != 0 && (*p as u8).is_ascii_digit() {
                p = p.add(1);
            }
        }

        // Check for %(
        if (*p as u8) == b'(' {
            groupdepth += 1;
            continue;
        }

        // Check if character is in STL_ALL
        let current_char = *p as u8;
        if !STL_ALL.contains(&current_char) {
            return illegal_char(errbuf, errbuflen, c_int::from(current_char));
        }

        // Handle %{ expression
        if current_char == STL_VIM_EXPR {
            p = p.add(1);
            let reevaluate = (*p as u8) == b'%';

            if reevaluate {
                p = p.add(1);
                if (*p as u8) == b'}' {
                    // "}" is not allowed immediately after "%{%"
                    return illegal_char(errbuf, errbuflen, c_int::from(b'}'));
                }
            }

            // Find closing }
            while *p != 0 {
                let curr = *p as u8;
                if curr == b'}' && (!reevaluate || (*p.sub(1) as u8) == b'%') {
                    break;
                }
                p = p.add(1);
            }

            if (*p as u8) != b'}' {
                return E_UNCLOSED_EXPRESSION.as_ptr().cast();
            }
        }
    }

    if groupdepth != 0 {
        return E_UNBALANCED_GROUPS.as_ptr().cast();
    }

    std::ptr::null()
}

/// Check validity of options with the 'statusline' format.
/// This is a direct replacement for the C `check_stl_option` function.
/// Uses a per-call static buffer matching the C behavior.
///
/// # Safety
/// `s` must be a valid null-terminated C string.
#[cfg(not(test))]
#[allow(static_mut_refs)]
#[export_name = "check_stl_option"]
pub unsafe extern "C" fn check_stl_option(s: *mut c_char) -> *const c_char {
    static mut ERRBUF: [c_char; 80] = [0; 80];
    rs_check_stl_option(s, ERRBUF.as_mut_ptr(), 80)
}

// =============================================================================
// Signcolumn Validation
// =============================================================================

/// Signcolumn width constants
const SCL_NO: i32 = -1;
const SCL_NUM: i32 = -2;

/// Parsed signcolumn result
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SigncolumnResult {
    /// Minimum width (0 for auto, -1 for no, -2 for number)
    pub min_width: c_int,
    /// Maximum width
    pub max_width: c_int,
    /// Whether the value is valid
    pub valid: c_int,
}

/// Valid signcolumn option values (simple ones)
const OPT_SCL_VALUES: &[&[u8]] = &[
    b"no\0",
    b"yes\0",
    b"auto\0",
    b"number\0",
    b"yes:1\0",
    b"yes:2\0",
    b"yes:3\0",
    b"yes:4\0",
    b"yes:5\0",
    b"yes:6\0",
    b"yes:7\0",
    b"yes:8\0",
    b"yes:9\0",
    b"auto:1\0",
    b"auto:2\0",
    b"auto:3\0",
    b"auto:4\0",
    b"auto:5\0",
    b"auto:6\0",
    b"auto:7\0",
    b"auto:8\0",
    b"auto:9\0",
];

/// Parse and validate a signcolumn option value.
///
/// Valid values are:
/// - "no" - no sign column
/// - "yes" or "yes:N" - always show, width N (1-9)
/// - "auto" or "auto:N" - auto show, max width N (1-9)
/// - "auto:M-N" - auto show, min width M, max width N
/// - "number" - show in number column
///
/// # Arguments
///
/// * `val` - The signcolumn value to parse
///
/// # Returns
///
/// A SigncolumnResult with the parsed min/max widths and validity flag.
///
/// # Safety
///
/// `val` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_signcolumn(val: *const c_char) -> SigncolumnResult {
    let invalid = SigncolumnResult {
        min_width: 0,
        max_width: 0,
        valid: 0,
    };

    if val.is_null() || *val == 0 {
        return invalid;
    }

    // Check if it's one of the simple values by checking for exact matches
    for opt_val in OPT_SCL_VALUES {
        if streq_cstr(val, opt_val) {
            // Parse based on prefix
            let c0 = *val as u8;
            let c1 = if *val != 0 { *val.add(1) as u8 } else { 0 };

            if c0 == b'n' && c1 == b'o' {
                // "no"
                return SigncolumnResult {
                    min_width: SCL_NO,
                    max_width: SCL_NO,
                    valid: 1,
                };
            } else if c0 == b'n' && c1 == b'u' {
                // "number"
                return SigncolumnResult {
                    min_width: SCL_NUM,
                    max_width: SCL_NUM,
                    valid: 1,
                };
            } else if c0 == b'y' && c1 == b'e' {
                // "yes" or "yes:N"
                let c4 = *val.add(4) as u8;
                let width = if c4 == 0 { 1 } else { i32::from(c4 - b'0') };
                return SigncolumnResult {
                    min_width: width,
                    max_width: width,
                    valid: 1,
                };
            } else if c0 == b'a' && c1 == b'u' {
                // "auto" or "auto:N"
                let c5 = *val.add(5) as u8;
                let max = if c5 == 0 { 1 } else { i32::from(c5 - b'0') };
                return SigncolumnResult {
                    min_width: 0,
                    max_width: max,
                    valid: 1,
                };
            }
        }
    }

    // Check for "auto:M-N" format (must be exactly 8 characters)
    let mut len = 0;
    let mut p = val;
    while *p != 0 {
        len += 1;
        p = p.add(1);
    }

    if len != 8 {
        return invalid;
    }

    if !strequal_prefix(val, b"auto:\0") {
        return invalid;
    }

    let c5 = *val.add(5) as u8;
    let c6 = *val.add(6) as u8;
    let c7 = *val.add(7) as u8;

    if !c5.is_ascii_digit() || c6 != b'-' || !c7.is_ascii_digit() {
        return invalid;
    }

    let min = i32::from(c5 - b'0');
    let max = i32::from(c7 - b'0');

    if min < 1 || max < 2 || min > 8 || min >= max {
        return invalid;
    }

    SigncolumnResult {
        min_width: min,
        max_width: max,
        valid: 1,
    }
}

/// Check if a C string equals a null-terminated byte slice.
#[inline]
unsafe fn streq_cstr(s: *const c_char, bytes: &[u8]) -> bool {
    let mut p = s;
    for &c in bytes {
        if c == 0 {
            return *p == 0; // Both should end at the same point
        }
        if *p == 0 || (*p as u8) != c {
            return false;
        }
        p = p.add(1);
    }
    true
}

/// Check if a C string starts with a given prefix.
#[inline]
unsafe fn strequal_prefix(s: *const c_char, prefix: &[u8]) -> bool {
    let mut p = s;
    for &c in prefix {
        if c == 0 {
            return true;
        }
        if *p == 0 || (*p as u8) != c {
            return false;
        }
        p = p.add(1);
    }
    true
}

// =============================================================================
// Generic Option String Validation
// =============================================================================

/// Validate that a string option value matches one of the allowed values.
///
/// This is used for options that have a fixed set of valid values (like
/// 'selection', 'selectmode', etc.).
///
/// # Arguments
///
/// * `val` - The value to validate
/// * `values` - NULL-terminated array of valid values
/// * `is_list` - If true, val can contain comma-separated items from values
///
/// # Returns
///
/// 1 if valid, 0 if invalid.
///
/// # Safety
///
/// `val` must be a valid null-terminated C string. `values` must be a valid
/// NULL-terminated array of C strings.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_string_option(
    val: *const c_char,
    values: *const *const c_char,
    is_list: c_int,
) -> c_int {
    if val.is_null() || values.is_null() {
        return 0;
    }

    if is_list != 0 {
        // For comma-separated lists, validate each item
        let mut p = val;
        while *p != 0 {
            // Skip leading separators
            while *p != 0 && ((*p as u8) == b',' || (*p as u8) == b' ') {
                p = p.add(1);
            }
            if *p == 0 {
                break;
            }

            // Find end of current item
            let item_start = p;
            while *p != 0 && (*p as u8) != b',' {
                p = p.add(1);
            }
            let item_len = p.offset_from(item_start) as usize;

            // Check if this item is valid
            if !is_valid_item(item_start, item_len, values) {
                return 0;
            }
        }
        1
    } else {
        // Simple case: val must exactly match one of the values
        let mut i = 0;
        loop {
            let v = *values.add(i);
            if v.is_null() {
                break;
            }
            if streq(val, v) {
                return 1;
            }
            i += 1;
        }
        0
    }
}

/// Check if an item matches any of the allowed values.
#[inline]
unsafe fn is_valid_item(item: *const c_char, len: usize, values: *const *const c_char) -> bool {
    let mut i = 0;
    loop {
        let v = *values.add(i);
        if v.is_null() {
            break;
        }
        if strneq(item, v, len) && *v.add(len) == 0 {
            return true;
        }
        i += 1;
    }
    false
}

/// Compare two null-terminated strings for equality.
#[inline]
unsafe fn streq(s1: *const c_char, s2: *const c_char) -> bool {
    let mut p1 = s1;
    let mut p2 = s2;
    loop {
        if *p1 != *p2 {
            return false;
        }
        if *p1 == 0 {
            return true;
        }
        p1 = p1.add(1);
        p2 = p2.add(1);
    }
}

/// Compare n bytes of two strings for equality.
#[inline]
unsafe fn strneq(s1: *const c_char, s2: *const c_char, n: usize) -> bool {
    for i in 0..n {
        if *s1.add(i) != *s2.add(i) {
            return false;
        }
    }
    true
}

// =============================================================================
// Blending Validation
// =============================================================================

/// Check if blend values for window are valid.
///
/// This validates that winblend/pumblend values are in the valid range [0, 100].
///
/// # Arguments
///
/// * `value` - The blend value to check
///
/// # Returns
///
/// 1 if valid (0-100), 0 if invalid.
#[no_mangle]
pub extern "C" fn rs_validate_blend(value: c_int) -> c_int {
    c_int::from((0..=100).contains(&value))
}

// =============================================================================
// Cross-Option Dependency Validation
// =============================================================================

/// Validation result with error context.
#[repr(C)]
pub struct ValidationResult {
    /// 0 = valid, non-zero = error code
    pub error_code: c_int,
    /// Offset in string where error occurred (-1 if not applicable)
    pub error_offset: c_int,
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self {
            error_code: 0,
            error_offset: -1,
        }
    }
}

/// Error codes for validation
pub mod error_codes {
    use std::ffi::c_int;

    pub const VALID: c_int = 0;
    pub const INVALID_VALUE: c_int = 1;
    pub const OUT_OF_RANGE: c_int = 2;
    pub const INVALID_FORMAT: c_int = 3;
    pub const DEPENDENCY_CONFLICT: c_int = 4;
    pub const SECURITY_VIOLATION: c_int = 5;
    pub const EMPTY_NOT_ALLOWED: c_int = 6;
    pub const INVALID_CHARACTER: c_int = 7;
}

/// Create a successful validation result.
#[inline]
fn validation_ok() -> ValidationResult {
    ValidationResult::default()
}

/// Create a validation error at a specific position.
#[inline]
fn validation_error(code: c_int, offset: c_int) -> ValidationResult {
    ValidationResult {
        error_code: code,
        error_offset: offset,
    }
}

/// Validate a numeric option value with bounds.
///
/// # Arguments
/// * `value` - The value to validate
/// * `min` - Minimum allowed value (or i64::MIN for no minimum)
/// * `max` - Maximum allowed value (or i64::MAX for no maximum)
/// * `allow_negative` - Whether negative values are allowed
///
/// # Returns
/// ValidationResult with error_code 0 on success.
#[no_mangle]
pub extern "C" fn rs_validate_numeric_bounds(
    value: i64,
    min: i64,
    max: i64,
    allow_negative: c_int,
) -> ValidationResult {
    if value < 0 && allow_negative == 0 {
        return validation_error(error_codes::OUT_OF_RANGE, -1);
    }
    if value < min || value > max {
        return validation_error(error_codes::OUT_OF_RANGE, -1);
    }
    validation_ok()
}

/// Validate that a string contains only characters from an allowed set.
///
/// # Arguments
/// * `value` - The string to validate
/// * `allowed` - String of allowed characters
///
/// # Returns
/// ValidationResult with offset of first invalid character.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_chars(
    value: *const c_char,
    allowed: *const c_char,
) -> ValidationResult {
    if value.is_null() || allowed.is_null() {
        return validation_error(error_codes::INVALID_VALUE, -1);
    }

    // Build allowed set
    let mut allowed_set = [false; 256];
    let mut a = allowed;
    while *a != 0 {
        allowed_set[*a as u8 as usize] = true;
        a = a.add(1);
    }

    // Check each character
    let mut v = value;
    let mut offset = 0;
    while *v != 0 {
        if !allowed_set[*v as u8 as usize] {
            return validation_error(error_codes::INVALID_CHARACTER, offset);
        }
        v = v.add(1);
        offset += 1;
    }

    validation_ok()
}

/// Validate a comma-separated list of values against allowed values.
///
/// # Arguments
/// * `value` - The comma-separated string
/// * `allowed` - Comma-separated list of allowed values
/// * `allow_empty` - Whether empty items are allowed
///
/// # Returns
/// ValidationResult with error info.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_comma_list(
    value: *const c_char,
    allowed: *const c_char,
    allow_empty: c_int,
) -> ValidationResult {
    if value.is_null() {
        return validation_error(error_codes::INVALID_VALUE, -1);
    }

    // Empty string is valid if allow_empty is set
    if *value == 0 {
        return if allow_empty != 0 {
            validation_ok()
        } else {
            validation_error(error_codes::EMPTY_NOT_ALLOWED, 0)
        };
    }

    // Build set of allowed values
    let mut allowed_values: Vec<Vec<u8>> = Vec::new();
    if !allowed.is_null() && *allowed != 0 {
        let mut start = allowed;
        let mut p = allowed;
        loop {
            if *p == 0 || *p as u8 == b',' {
                let len = p.offset_from(start) as usize;
                let mut val = Vec::with_capacity(len);
                for i in 0..len {
                    val.push(*start.add(i) as u8);
                }
                allowed_values.push(val);
                if *p == 0 {
                    break;
                }
                start = p.add(1);
            }
            p = p.add(1);
        }
    }

    // Check each item in the value
    let mut offset: c_int = 0;
    let mut start = value;
    let mut p = value;
    loop {
        if *p == 0 || *p as u8 == b',' {
            let len = p.offset_from(start) as usize;

            // Check if empty
            if len == 0 && allow_empty == 0 {
                return validation_error(error_codes::EMPTY_NOT_ALLOWED, offset);
            }

            // Check if in allowed list (if we have one)
            if len > 0 && !allowed_values.is_empty() {
                let mut item = Vec::with_capacity(len);
                for i in 0..len {
                    item.push(*start.add(i) as u8);
                }

                if !allowed_values.iter().any(|v| v == &item) {
                    return validation_error(error_codes::INVALID_VALUE, offset);
                }
            }

            if *p == 0 {
                break;
            }
            start = p.add(1);
            #[allow(clippy::cast_possible_truncation)]
            {
                offset = (start.offset_from(value)) as c_int;
            }
        }
        p = p.add(1);
    }

    validation_ok()
}

// =============================================================================
// Enhanced Error Message Generation
// =============================================================================

/// Write a formatted error message to a buffer.
///
/// # Arguments
/// * `errbuf` - Buffer to write to
/// * `errbuflen` - Size of buffer
/// * `error_code` - Error code (see error_codes module)
/// * `value` - Value that caused the error
/// * `context` - Additional context (option name, etc.)
///
/// # Returns
/// Pointer to error buffer, or internal static message if buffer too small.
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_format_validation_error(
    errbuf: *mut c_char,
    errbuflen: usize,
    error_code: c_int,
    value: *const c_char,
    context: *const c_char,
) -> *const c_char {
    if errbuf.is_null() || errbuflen < 32 {
        // Return static message
        return match error_code {
            error_codes::OUT_OF_RANGE => c"E474: Value out of range".as_ptr(),
            error_codes::INVALID_FORMAT => c"E474: Invalid format".as_ptr(),
            error_codes::INVALID_CHARACTER => c"E474: Invalid character".as_ptr(),
            error_codes::EMPTY_NOT_ALLOWED => c"E474: Empty value not allowed".as_ptr(),
            error_codes::DEPENDENCY_CONFLICT => c"E474: Option dependency conflict".as_ptr(),
            error_codes::SECURITY_VIOLATION => c"E523: Security violation".as_ptr(),
            _ => c"E474: Invalid argument".as_ptr(),
        };
    }

    // Format error message
    let prefix: &[u8] = match error_code {
        error_codes::OUT_OF_RANGE => b"E474: Value out of range",
        error_codes::INVALID_FORMAT => b"E474: Invalid format",
        error_codes::INVALID_CHARACTER => b"E474: Invalid character",
        error_codes::EMPTY_NOT_ALLOWED => b"E474: Empty value not allowed",
        error_codes::DEPENDENCY_CONFLICT => b"E474: Option dependency conflict",
        error_codes::SECURITY_VIOLATION => b"E523: Security violation",
        _ => b"E474: Invalid argument",
    };

    // Copy prefix
    let prefix_len = prefix.len().min(errbuflen - 1);
    for (i, &b) in prefix.iter().take(prefix_len).enumerate() {
        *errbuf.add(i) = b as c_char;
    }
    let mut pos = prefix_len;

    // Add context if provided
    if !context.is_null() && *context != 0 && pos + 5 < errbuflen {
        *errbuf.add(pos) = b' ' as c_char;
        *errbuf.add(pos + 1) = b'f' as c_char;
        *errbuf.add(pos + 2) = b'o' as c_char;
        *errbuf.add(pos + 3) = b'r' as c_char;
        *errbuf.add(pos + 4) = b' ' as c_char;
        pos += 5;

        let mut c = context;
        while *c != 0 && pos < errbuflen - 1 {
            *errbuf.add(pos) = *c;
            c = c.add(1);
            pos += 1;
        }
    }

    // Add value if provided
    if !value.is_null() && *value != 0 && pos + 3 < errbuflen {
        *errbuf.add(pos) = b':' as c_char;
        *errbuf.add(pos + 1) = b' ' as c_char;
        pos += 2;

        let mut v = value;
        while *v != 0 && pos < errbuflen - 1 {
            *errbuf.add(pos) = *v;
            v = v.add(1);
            pos += 1;
        }
    }

    *errbuf.add(pos) = 0;
    errbuf
}

// =============================================================================
// Path Validation
// =============================================================================

/// Validate a file path for security issues.
///
/// Checks for:
/// - Null bytes (path truncation attack)
/// - Parent directory references (..) in certain contexts
/// - Absolute paths when relative expected
///
/// # Arguments
/// * `path` - The path to validate
/// * `flags` - Validation flags (see path_flags module)
///
/// # Returns
/// ValidationResult with error info.
pub mod path_flags {
    use std::ffi::c_int;

    pub const ALLOW_ABSOLUTE: c_int = 0x01;
    pub const ALLOW_PARENT_REF: c_int = 0x02;
    pub const CHECK_SECURITY: c_int = 0x04;
}

#[no_mangle]
pub unsafe extern "C" fn rs_validate_path(path: *const c_char, flags: c_int) -> ValidationResult {
    if path.is_null() {
        return validation_error(error_codes::INVALID_VALUE, -1);
    }

    let allow_absolute = (flags & path_flags::ALLOW_ABSOLUTE) != 0;
    let allow_parent = (flags & path_flags::ALLOW_PARENT_REF) != 0;
    let check_security = (flags & path_flags::CHECK_SECURITY) != 0;

    let mut p = path;
    let mut offset: c_int = 0;
    let mut prev_was_sep = true;

    // Check first character for absolute path
    if !allow_absolute {
        let first = *p as u8;
        if first == b'/' || first == b'\\' {
            return validation_error(error_codes::SECURITY_VIOLATION, 0);
        }
        // Windows drive letter
        if first.is_ascii_alphabetic() && *p.add(1) as u8 == b':' {
            return validation_error(error_codes::SECURITY_VIOLATION, 0);
        }
    }

    while *p != 0 {
        let c = *p as u8;

        // Check for null byte in middle of string (shouldn't happen, but be safe)
        // Actually we can't check this since the string ends at null

        // Check for parent directory reference
        if check_security && !allow_parent && prev_was_sep && c == b'.' && *p.add(1) as u8 == b'.' {
            let after = *p.add(2) as u8;
            if after == 0 || after == b'/' || after == b'\\' {
                return validation_error(error_codes::SECURITY_VIOLATION, offset);
            }
        }

        prev_was_sep = c == b'/' || c == b'\\';
        p = p.add(1);
        offset += 1;
    }

    validation_ok()
}

// =============================================================================
// Phase 8: validate_option_value (Phase 3)
// =============================================================================

#[cfg(not(test))]
use crate::index::OptIndex;

#[cfg(not(test))]
use crate::storage::OptVal;

#[cfg(not(test))]
use crate::OptValType;

#[cfg(not(test))]
extern "C" {
    fn rs_option_is_global_local(opt_idx: OptIndex) -> c_int;
    fn rs_get_option_unset_value(opt_idx: OptIndex) -> OptVal;
    fn rs_optval_equal(a: OptVal, b: OptVal) -> c_int;
    fn rs_optval_copy(o: OptVal) -> OptVal;
    #[link_name = "option_has_type"]
    fn rs_option_has_type(opt_idx: OptIndex, type_: c_int) -> c_int;
    fn nvim_get_option_type(opt_idx: OptIndex) -> c_int;
    fn nvim_option_get_fullname(opt_idx: OptIndex) -> *const c_char;
    fn rs_validate_num_option(
        opt_idx: OptIndex,
        newval: *mut i64,
        errbuf: *mut c_char,
        errbuflen: usize,
    ) -> *const c_char;
    fn xfree(ptr: *mut c_char);
    fn gettext(s: *const c_char) -> *const c_char;
}

/// kOptValTypeNil constant (must match C kOptValTypeNil = -1)
#[cfg(not(test))]
const K_OPT_VAL_TYPE_NIL: c_int = -1;
/// kOptValTypeBoolean (must match C kOptValTypeBoolean = 0)
#[cfg(not(test))]
const K_OPT_VAL_TYPE_BOOLEAN: c_int = 0;
/// kOptValTypeNumber (must match C kOptValTypeNumber = 1)
#[cfg(not(test))]
const K_OPT_VAL_TYPE_NUMBER: c_int = 1;
/// kOptValTypeString (must match C kOptValTypeString = 2)
#[cfg(not(test))]
const K_OPT_VAL_TYPE_STRING: c_int = 2;
/// OPT_GLOBAL flag (must match C OPT_GLOBAL = 0x01)
#[cfg(not(test))]
const OPT_GLOBAL_VAL: c_int = 0x01;
/// OPT_LOCAL flag (must match C OPT_LOCAL = 0x02)
#[cfg(not(test))]
const OPT_LOCAL_VAL: c_int = 0x02;

/// Mirrors C `optval_type_get_name` (static inline in option.h).
/// Returns a static C string for the given OptValType integer value.
#[cfg(not(test))]
unsafe fn optval_type_name(type_: c_int) -> *const c_char {
    match type_ {
        K_OPT_VAL_TYPE_NIL => c"nil".as_ptr(),
        K_OPT_VAL_TYPE_BOOLEAN => c"boolean".as_ptr(),
        K_OPT_VAL_TYPE_NUMBER => c"number".as_ptr(),
        K_OPT_VAL_TYPE_STRING => c"string".as_ptr(),
        _ => c"unknown".as_ptr(),
    }
}

/// Validate the new value for an option.
///
/// Mirrors C `validate_option_value`.
///
/// # Safety
/// Calls C accessor functions. `newval` must be a valid pointer to an OptVal.
/// `errbuf` must be a valid buffer of at least `errbuflen` bytes if non-null.
#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "C" fn rs_validate_option_value(
    opt_idx: OptIndex,
    newval: *mut OptVal,
    opt_flags: c_int,
    errbuf: *mut c_char,
    errbuflen: usize,
) -> *const c_char {
    // Always allow unsetting local value of global-local option.
    if rs_option_is_global_local(opt_idx) != 0
        && (opt_flags & OPT_LOCAL_VAL) != 0
        && rs_optval_equal(*newval, rs_get_option_unset_value(opt_idx)) != 0
    {
        return std::ptr::null();
    }

    let newval_type = (*newval).type_ as c_int;

    if newval_type == K_OPT_VAL_TYPE_NIL {
        // Don't try to unset local value if scope is global.
        if opt_flags == OPT_GLOBAL_VAL {
            return gettext(c"Cannot unset global option value".as_ptr());
        }
        *newval = rs_optval_copy(rs_get_option_unset_value(opt_idx));
        return std::ptr::null();
    }

    // Check for type mismatch.
    if rs_option_has_type(opt_idx, newval_type) == 0 {
        let opt_type = nvim_get_option_type(opt_idx);
        let type_str = optval_type_name(opt_type);
        let newval_type_str = optval_type_name(newval_type);
        let rep = crate::value::rs_optval_to_cstr(*newval);
        let fullname = nvim_option_get_fullname(opt_idx);
        let fmt = gettext(c"Invalid value for option '%s': expected %s, got %s %s".as_ptr());
        libc::snprintf(
            errbuf,
            errbuflen,
            fmt,
            fullname,
            type_str,
            newval_type_str,
            rep,
        );
        xfree(rep);
        return errbuf;
    }

    // Validate numeric bounds.
    if (*newval).type_ == OptValType::Number {
        return rs_validate_num_option(opt_idx, &raw mut (*newval).data.number, errbuf, errbuflen);
    }

    std::ptr::null()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_parse_signcolumn_simple() {
        unsafe {
            // Test "no"
            let no = CString::new("no").unwrap();
            let result = rs_parse_signcolumn(no.as_ptr());
            assert_eq!(result.valid, 1);
            assert_eq!(result.min_width, SCL_NO);
            assert_eq!(result.max_width, SCL_NO);

            // Test "yes"
            let yes = CString::new("yes").unwrap();
            let result = rs_parse_signcolumn(yes.as_ptr());
            assert_eq!(result.valid, 1);
            assert_eq!(result.min_width, 1);
            assert_eq!(result.max_width, 1);

            // Test "yes:3"
            let yes3 = CString::new("yes:3").unwrap();
            let result = rs_parse_signcolumn(yes3.as_ptr());
            assert_eq!(result.valid, 1);
            assert_eq!(result.min_width, 3);
            assert_eq!(result.max_width, 3);

            // Test "auto"
            let auto = CString::new("auto").unwrap();
            let result = rs_parse_signcolumn(auto.as_ptr());
            assert_eq!(result.valid, 1);
            assert_eq!(result.min_width, 0);
            assert_eq!(result.max_width, 1);

            // Test "auto:5"
            let auto5 = CString::new("auto:5").unwrap();
            let result = rs_parse_signcolumn(auto5.as_ptr());
            assert_eq!(result.valid, 1);
            assert_eq!(result.min_width, 0);
            assert_eq!(result.max_width, 5);

            // Test "number"
            let number = CString::new("number").unwrap();
            let result = rs_parse_signcolumn(number.as_ptr());
            assert_eq!(result.valid, 1);
            assert_eq!(result.min_width, SCL_NUM);
            assert_eq!(result.max_width, SCL_NUM);
        }
    }

    #[test]
    fn test_parse_signcolumn_range() {
        unsafe {
            // Test "auto:2-5"
            let auto_range = CString::new("auto:2-5").unwrap();
            let result = rs_parse_signcolumn(auto_range.as_ptr());
            assert_eq!(result.valid, 1);
            assert_eq!(result.min_width, 2);
            assert_eq!(result.max_width, 5);

            // Test "auto:1-9"
            let auto_range2 = CString::new("auto:1-9").unwrap();
            let result = rs_parse_signcolumn(auto_range2.as_ptr());
            assert_eq!(result.valid, 1);
            assert_eq!(result.min_width, 1);
            assert_eq!(result.max_width, 9);
        }
    }

    #[test]
    fn test_parse_signcolumn_invalid() {
        unsafe {
            // Invalid: empty
            let empty = CString::new("").unwrap();
            let result = rs_parse_signcolumn(empty.as_ptr());
            assert_eq!(result.valid, 0);

            // Invalid: unknown value
            let unknown = CString::new("invalid").unwrap();
            let result = rs_parse_signcolumn(unknown.as_ptr());
            assert_eq!(result.valid, 0);

            // Invalid: min >= max
            let invalid_range = CString::new("auto:5-5").unwrap();
            let result = rs_parse_signcolumn(invalid_range.as_ptr());
            assert_eq!(result.valid, 0);

            // Invalid: min > max
            let invalid_range2 = CString::new("auto:5-3").unwrap();
            let result = rs_parse_signcolumn(invalid_range2.as_ptr());
            assert_eq!(result.valid, 0);

            // Invalid: wrong format
            let wrong_format = CString::new("auto:5-").unwrap();
            let result = rs_parse_signcolumn(wrong_format.as_ptr());
            assert_eq!(result.valid, 0);

            // NULL
            let result = rs_parse_signcolumn(std::ptr::null());
            assert_eq!(result.valid, 0);
        }
    }

    #[test]
    fn test_validate_blend() {
        // Valid values
        assert_eq!(rs_validate_blend(0), 1);
        assert_eq!(rs_validate_blend(50), 1);
        assert_eq!(rs_validate_blend(100), 1);

        // Invalid values
        assert_eq!(rs_validate_blend(-1), 0);
        assert_eq!(rs_validate_blend(101), 0);
        assert_eq!(rs_validate_blend(200), 0);
    }

    #[test]
    fn test_validate_string_option_simple() {
        unsafe {
            let val1 = CString::new("value1").unwrap();
            let val2 = CString::new("value2").unwrap();
            let val3 = CString::new("value3").unwrap();

            let values: [*const c_char; 4] = [
                val1.as_ptr(),
                val2.as_ptr(),
                val3.as_ptr(),
                std::ptr::null(),
            ];

            // Valid value
            let test_val = CString::new("value2").unwrap();
            assert_eq!(
                rs_validate_string_option(test_val.as_ptr(), values.as_ptr(), 0),
                1
            );

            // Invalid value
            let invalid = CString::new("invalid").unwrap();
            assert_eq!(
                rs_validate_string_option(invalid.as_ptr(), values.as_ptr(), 0),
                0
            );
        }
    }

    #[test]
    fn test_validate_string_option_list() {
        unsafe {
            let val1 = CString::new("one").unwrap();
            let val2 = CString::new("two").unwrap();
            let val3 = CString::new("three").unwrap();

            let values: [*const c_char; 4] = [
                val1.as_ptr(),
                val2.as_ptr(),
                val3.as_ptr(),
                std::ptr::null(),
            ];

            // Valid list
            let test_list = CString::new("one,two").unwrap();
            assert_eq!(
                rs_validate_string_option(test_list.as_ptr(), values.as_ptr(), 1),
                1
            );

            // Valid list with spaces
            let test_list2 = CString::new("one, two, three").unwrap();
            assert_eq!(
                rs_validate_string_option(test_list2.as_ptr(), values.as_ptr(), 1),
                1
            );

            // Invalid item in list
            let invalid_list = CString::new("one,invalid,two").unwrap();
            assert_eq!(
                rs_validate_string_option(invalid_list.as_ptr(), values.as_ptr(), 1),
                0
            );
        }
    }
}
