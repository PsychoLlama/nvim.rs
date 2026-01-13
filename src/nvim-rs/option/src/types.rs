//! Option type checking helpers.
//!
//! This module provides helpers for option type validation:
//! - Type detection
//! - Type coercion
//! - Type compatibility checks

#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

// =============================================================================
// Option Type Constants
// =============================================================================

/// Option is a boolean (on/off).
pub const OPT_TYPE_BOOL: c_int = 0;
/// Option is a number (integer).
pub const OPT_TYPE_NUMBER: c_int = 1;
/// Option is a string.
pub const OPT_TYPE_STRING: c_int = 2;

// =============================================================================
// Boolean Value Constants
// =============================================================================

/// Boolean option: off.
pub const OPT_BOOL_OFF: c_int = 0;
/// Boolean option: on.
pub const OPT_BOOL_ON: c_int = 1;
/// Boolean option: toggle.
pub const OPT_BOOL_TOGGLE: c_int = 2;

// =============================================================================
// Number Option Flags
// =============================================================================

/// Number option allows negative values.
pub const OPT_NUM_ALLOW_NEGATIVE: c_int = 0x01;
/// Number option has minimum bound.
pub const OPT_NUM_HAS_MIN: c_int = 0x02;
/// Number option has maximum bound.
pub const OPT_NUM_HAS_MAX: c_int = 0x04;
/// Number option is unsigned only.
pub const OPT_NUM_UNSIGNED: c_int = 0x08;

// =============================================================================
// String Option Flags
// =============================================================================

/// String option cannot be empty.
pub const OPT_STR_NONEMPTY: c_int = 0x01;
/// String option is a file path.
pub const OPT_STR_PATH: c_int = 0x02;
/// String option is a comma-separated list.
pub const OPT_STR_COMMA_LIST: c_int = 0x04;
/// String option is a flags string.
pub const OPT_STR_FLAGS: c_int = 0x08;
/// String option expands environment variables.
pub const OPT_STR_EXPAND: c_int = 0x10;

// =============================================================================
// Type Helpers
// =============================================================================

/// Check if option type is boolean.
fn is_bool_option(opt_type: c_int) -> bool {
    opt_type == OPT_TYPE_BOOL
}

/// Check if option type is number.
fn is_number_option(opt_type: c_int) -> bool {
    opt_type == OPT_TYPE_NUMBER
}

/// Check if option type is string.
fn is_string_option(opt_type: c_int) -> bool {
    opt_type == OPT_TYPE_STRING
}

/// Get type name string.
#[allow(dead_code)]
fn type_name(opt_type: c_int) -> &'static str {
    match opt_type {
        OPT_TYPE_BOOL => "boolean",
        OPT_TYPE_NUMBER => "number",
        OPT_TYPE_STRING => "string",
        _ => "unknown",
    }
}

// =============================================================================
// Boolean Helpers
// =============================================================================

/// Parse boolean string value.
#[allow(dead_code)]
fn parse_bool_string(s: &str) -> c_int {
    match s.to_lowercase().as_str() {
        "on" | "yes" | "true" | "1" => OPT_BOOL_ON,
        "off" | "no" | "false" | "0" => OPT_BOOL_OFF,
        "inv" | "invert" | "toggle" => OPT_BOOL_TOGGLE,
        _ => -1, // invalid
    }
}

/// Check if boolean value is valid.
fn is_valid_bool(value: c_int) -> bool {
    matches!(value, OPT_BOOL_OFF | OPT_BOOL_ON | OPT_BOOL_TOGGLE)
}

// =============================================================================
// Number Helpers
// =============================================================================

/// Check if number value is in range.
fn is_in_range(value: i64, min: i64, max: i64) -> bool {
    value >= min && value <= max
}

/// Check if number option allows negative.
fn allows_negative(flags: c_int) -> bool {
    (flags & OPT_NUM_ALLOW_NEGATIVE) != 0
}

/// Check if number is valid for flags.
fn is_valid_number(value: i64, flags: c_int) -> bool {
    if value < 0 && !allows_negative(flags) {
        return false;
    }
    true
}

// =============================================================================
// String Helpers
// =============================================================================

/// Check if string option must be non-empty.
fn must_be_nonempty(flags: c_int) -> bool {
    (flags & OPT_STR_NONEMPTY) != 0
}

/// Check if string option is a path.
fn is_path_option(flags: c_int) -> bool {
    (flags & OPT_STR_PATH) != 0
}

/// Check if string option is comma-separated list.
fn is_comma_list_option(flags: c_int) -> bool {
    (flags & OPT_STR_COMMA_LIST) != 0
}

/// Check if string option is flags-style.
fn is_flags_option(flags: c_int) -> bool {
    (flags & OPT_STR_FLAGS) != 0
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Get OPT_TYPE_BOOL constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_type_bool() -> c_int {
    OPT_TYPE_BOOL
}

/// FFI: Get OPT_TYPE_NUMBER constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_type_number() -> c_int {
    OPT_TYPE_NUMBER
}

/// FFI: Get OPT_TYPE_STRING constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_type_string() -> c_int {
    OPT_TYPE_STRING
}

/// FFI: Get OPT_BOOL_OFF constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_bool_off() -> c_int {
    OPT_BOOL_OFF
}

/// FFI: Get OPT_BOOL_ON constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_bool_on() -> c_int {
    OPT_BOOL_ON
}

/// FFI: Get OPT_BOOL_TOGGLE constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_bool_toggle() -> c_int {
    OPT_BOOL_TOGGLE
}

/// FFI: Check if boolean option.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_bool_option(opt_type: c_int) -> c_int {
    c_int::from(is_bool_option(opt_type))
}

/// FFI: Check if number option.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_number_option(opt_type: c_int) -> c_int {
    c_int::from(is_number_option(opt_type))
}

/// FFI: Check if string option.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_string_option(opt_type: c_int) -> c_int {
    c_int::from(is_string_option(opt_type))
}

/// FFI: Check if valid boolean value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_valid_bool(value: c_int) -> c_int {
    c_int::from(is_valid_bool(value))
}

/// FFI: Check if number in range.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_is_in_range(value: i64, min: i64, max: i64) -> c_int {
    c_int::from(is_in_range(value, min, max))
}

/// FFI: Check if number allows negative.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_allows_negative(flags: c_int) -> c_int {
    c_int::from(allows_negative(flags))
}

/// FFI: Check if valid number for flags.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_is_valid_number(value: i64, flags: c_int) -> c_int {
    c_int::from(is_valid_number(value, flags))
}

/// FFI: Check if string must be non-empty.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_must_be_nonempty(flags: c_int) -> c_int {
    c_int::from(must_be_nonempty(flags))
}

/// FFI: Check if string is path option.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_is_path_option(flags: c_int) -> c_int {
    c_int::from(is_path_option(flags))
}

/// FFI: Check if string is comma list.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_is_comma_list(flags: c_int) -> c_int {
    c_int::from(is_comma_list_option(flags))
}

/// FFI: Check if string is flags-style.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_is_flags_option(flags: c_int) -> c_int {
    c_int::from(is_flags_option(flags))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_constants() {
        assert_eq!(OPT_TYPE_BOOL, 0);
        assert_eq!(OPT_TYPE_NUMBER, 1);
        assert_eq!(OPT_TYPE_STRING, 2);
    }

    #[test]
    fn test_bool_constants() {
        assert_eq!(OPT_BOOL_OFF, 0);
        assert_eq!(OPT_BOOL_ON, 1);
        assert_eq!(OPT_BOOL_TOGGLE, 2);
    }

    #[test]
    fn test_type_checks() {
        assert!(is_bool_option(OPT_TYPE_BOOL));
        assert!(!is_bool_option(OPT_TYPE_NUMBER));

        assert!(is_number_option(OPT_TYPE_NUMBER));
        assert!(!is_number_option(OPT_TYPE_STRING));

        assert!(is_string_option(OPT_TYPE_STRING));
        assert!(!is_string_option(OPT_TYPE_BOOL));
    }

    #[test]
    fn test_type_name() {
        assert_eq!(type_name(OPT_TYPE_BOOL), "boolean");
        assert_eq!(type_name(OPT_TYPE_NUMBER), "number");
        assert_eq!(type_name(OPT_TYPE_STRING), "string");
        assert_eq!(type_name(99), "unknown");
    }

    #[test]
    fn test_parse_bool_string() {
        assert_eq!(parse_bool_string("on"), OPT_BOOL_ON);
        assert_eq!(parse_bool_string("ON"), OPT_BOOL_ON);
        assert_eq!(parse_bool_string("true"), OPT_BOOL_ON);
        assert_eq!(parse_bool_string("1"), OPT_BOOL_ON);

        assert_eq!(parse_bool_string("off"), OPT_BOOL_OFF);
        assert_eq!(parse_bool_string("false"), OPT_BOOL_OFF);
        assert_eq!(parse_bool_string("0"), OPT_BOOL_OFF);

        assert_eq!(parse_bool_string("inv"), OPT_BOOL_TOGGLE);
        assert_eq!(parse_bool_string("toggle"), OPT_BOOL_TOGGLE);

        assert_eq!(parse_bool_string("invalid"), -1);
    }

    #[test]
    fn test_is_valid_bool() {
        assert!(is_valid_bool(OPT_BOOL_OFF));
        assert!(is_valid_bool(OPT_BOOL_ON));
        assert!(is_valid_bool(OPT_BOOL_TOGGLE));
        assert!(!is_valid_bool(-1));
        assert!(!is_valid_bool(99));
    }

    #[test]
    fn test_is_in_range() {
        assert!(is_in_range(5, 0, 10));
        assert!(is_in_range(0, 0, 10));
        assert!(is_in_range(10, 0, 10));
        assert!(!is_in_range(-1, 0, 10));
        assert!(!is_in_range(11, 0, 10));
    }

    #[test]
    fn test_allows_negative() {
        assert!(allows_negative(OPT_NUM_ALLOW_NEGATIVE));
        assert!(!allows_negative(0));
        assert!(!allows_negative(OPT_NUM_HAS_MIN));
    }

    #[test]
    fn test_is_valid_number() {
        assert!(is_valid_number(5, 0));
        assert!(is_valid_number(-5, OPT_NUM_ALLOW_NEGATIVE));
        assert!(!is_valid_number(-5, 0));
    }

    #[test]
    fn test_string_flags() {
        assert!(must_be_nonempty(OPT_STR_NONEMPTY));
        assert!(!must_be_nonempty(0));

        assert!(is_path_option(OPT_STR_PATH));
        assert!(!is_path_option(OPT_STR_FLAGS));

        assert!(is_comma_list_option(OPT_STR_COMMA_LIST));
        assert!(is_flags_option(OPT_STR_FLAGS));
    }
}
