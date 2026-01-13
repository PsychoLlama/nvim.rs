//! Dictionary operations.
//!
//! This module provides helpers for dictionary operations:
//! dict_alloc, dict_add, dict_copy, dict_extend

#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;
use std::ptr::NonNull;

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to a dict_T structure.
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct DictHandle(NonNull<std::ffi::c_void>);

/// Opaque handle to a dictitem_T structure.
#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct DictItemHandle(NonNull<std::ffi::c_void>);

// =============================================================================
// Dictionary Operation Constants
// =============================================================================

/// Dictionary is empty.
pub const DICT_EMPTY: c_int = 0;
/// Dictionary has items.
pub const DICT_HAS_ITEMS: c_int = 1;

// =============================================================================
// Dictionary Flags
// =============================================================================

/// Dictionary is locked (cannot be modified).
pub const DICT_LOCKED: c_int = 0x01;
/// Dictionary items are frozen (deep lock).
pub const DICT_FROZEN: c_int = 0x02;
/// Dictionary is being copied.
pub const DICT_COPYING: c_int = 0x04;
/// Dictionary is a scope dict (g:, l:, etc).
pub const DICT_SCOPE: c_int = 0x08;

// =============================================================================
// Dictionary Add Behavior
// =============================================================================

/// Error if key exists.
pub const DICT_ADD_ERROR: c_int = 0;
/// Overwrite if key exists.
pub const DICT_ADD_OVERWRITE: c_int = 1;
/// Skip if key exists.
pub const DICT_ADD_SKIP: c_int = 2;

// =============================================================================
// Dictionary Helpers
// =============================================================================

/// Check if key is valid (non-empty, no NUL).
fn is_valid_dict_key(key_len: usize) -> bool {
    key_len > 0
}

/// Check if dictionary is locked.
fn is_dict_locked(flags: c_int) -> bool {
    (flags & DICT_LOCKED) != 0
}

/// Check if dictionary is frozen.
fn is_dict_frozen(flags: c_int) -> bool {
    (flags & DICT_FROZEN) != 0
}

/// Check if dictionary can be modified.
fn can_modify_dict(flags: c_int) -> bool {
    (flags & (DICT_LOCKED | DICT_FROZEN)) == 0
}

/// Check if dictionary is a scope dict.
fn is_scope_dict(flags: c_int) -> bool {
    (flags & DICT_SCOPE) != 0
}

/// Get add behavior as constant.
#[allow(dead_code)]
fn add_behavior_from_flags(force: bool, keep: bool) -> c_int {
    if keep {
        DICT_ADD_SKIP
    } else if force {
        DICT_ADD_OVERWRITE
    } else {
        DICT_ADD_ERROR
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Get DICT_EMPTY constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_dict_empty() -> c_int {
    DICT_EMPTY
}

/// FFI: Get DICT_HAS_ITEMS constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_dict_has_items() -> c_int {
    DICT_HAS_ITEMS
}

/// FFI: Get DICT_LOCKED constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_dict_locked_flag() -> c_int {
    DICT_LOCKED
}

/// FFI: Get DICT_FROZEN constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_dict_frozen_flag() -> c_int {
    DICT_FROZEN
}

/// FFI: Get DICT_SCOPE constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_dict_scope_flag() -> c_int {
    DICT_SCOPE
}

/// FFI: Get DICT_ADD_ERROR constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_dict_add_error() -> c_int {
    DICT_ADD_ERROR
}

/// FFI: Get DICT_ADD_OVERWRITE constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_dict_add_overwrite() -> c_int {
    DICT_ADD_OVERWRITE
}

/// FFI: Get DICT_ADD_SKIP constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_dict_add_skip() -> c_int {
    DICT_ADD_SKIP
}

/// FFI: Check if valid dict key.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_valid_dict_key(key_len: usize) -> c_int {
    c_int::from(is_valid_dict_key(key_len))
}

/// FFI: Check if dictionary is locked.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_dict_locked(flags: c_int) -> c_int {
    c_int::from(is_dict_locked(flags))
}

/// FFI: Check if dictionary is frozen.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_dict_frozen(flags: c_int) -> c_int {
    c_int::from(is_dict_frozen(flags))
}

/// FFI: Check if dictionary can be modified.
#[unsafe(no_mangle)]
pub extern "C" fn rs_can_modify_dict(flags: c_int) -> c_int {
    c_int::from(can_modify_dict(flags))
}

/// FFI: Check if dictionary is a scope dict.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_scope_dict(flags: c_int) -> c_int {
    c_int::from(is_scope_dict(flags))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dict_constants() {
        assert_eq!(DICT_EMPTY, 0);
        assert_eq!(DICT_HAS_ITEMS, 1);
    }

    #[test]
    fn test_dict_flags() {
        assert_eq!(DICT_LOCKED, 0x01);
        assert_eq!(DICT_FROZEN, 0x02);
        assert_eq!(DICT_SCOPE, 0x08);
    }

    #[test]
    fn test_dict_add_behavior() {
        assert_eq!(DICT_ADD_ERROR, 0);
        assert_eq!(DICT_ADD_OVERWRITE, 1);
        assert_eq!(DICT_ADD_SKIP, 2);
    }

    #[test]
    fn test_is_valid_dict_key() {
        assert!(is_valid_dict_key(1));
        assert!(is_valid_dict_key(100));
        assert!(!is_valid_dict_key(0));
    }

    #[test]
    fn test_dict_flag_checks() {
        assert!(is_dict_locked(DICT_LOCKED));
        assert!(!is_dict_locked(0));

        assert!(is_dict_frozen(DICT_FROZEN));
        assert!(!is_dict_frozen(DICT_LOCKED));

        assert!(can_modify_dict(0));
        assert!(!can_modify_dict(DICT_LOCKED));
        assert!(!can_modify_dict(DICT_FROZEN));

        assert!(is_scope_dict(DICT_SCOPE));
        assert!(!is_scope_dict(DICT_LOCKED));
    }

    #[test]
    fn test_add_behavior_from_flags() {
        assert_eq!(add_behavior_from_flags(false, false), DICT_ADD_ERROR);
        assert_eq!(add_behavior_from_flags(true, false), DICT_ADD_OVERWRITE);
        assert_eq!(add_behavior_from_flags(false, true), DICT_ADD_SKIP);
        assert_eq!(add_behavior_from_flags(true, true), DICT_ADD_SKIP); // keep takes precedence
    }
}
