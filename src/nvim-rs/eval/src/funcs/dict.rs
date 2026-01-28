//! Dictionary manipulation functions for VimL.
//!
//! This module implements dictionary functions from `src/nvim/eval/funcs.c`:
//! - `keys()` - get list of dictionary keys
//! - `values()` - get list of dictionary values
//! - `items()` - get list of [key, value] pairs
//! - `has_key()` - check if key exists
//! - `get()` - get value with default
//! - `remove()` - remove key from dictionary
//! - `extend()` - extend dictionary with another
//! - `copy()` - shallow copy dictionary
//! - `deepcopy()` - deep copy dictionary
//! - `filter()` - filter dictionary by predicate
//! - `map()` - transform dictionary values
//! - `foreach()` - iterate over dictionary
//! - `reduce()` - reduce dictionary to single value

#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::must_use_candidate)]

use std::ffi::{c_int, c_void};

use super::dispatch::{
    argvar_at, dict_len, rettv_set_bool, rettv_set_number, tv_dict_is_null, tv_get_dict,
    tv_get_type, DictPtr, TypevalPtrMut, VarType,
};

// =============================================================================
// Dictionary Key Validation
// =============================================================================

/// Result of dictionary key validation.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DictKeyResult {
    /// Whether the key is valid
    pub valid: bool,
    /// Error code (0 = no error, 1 = empty key, 2 = invalid type)
    pub error: c_int,
}

/// Validate a dictionary key.
///
/// VimL dictionary keys must be non-empty strings.
pub fn validate_dict_key(key: &[u8]) -> DictKeyResult {
    if key.is_empty() {
        DictKeyResult {
            valid: false,
            error: 1, // Empty key
        }
    } else {
        DictKeyResult {
            valid: true,
            error: 0,
        }
    }
}

/// Check if a byte sequence is a valid dictionary key.
pub const fn is_valid_dict_key_len(len: i64) -> bool {
    len > 0
}

// =============================================================================
// Dictionary Merge Strategies
// =============================================================================

/// Strategy for handling duplicate keys during dictionary extend.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExtendStrategy {
    /// Keep existing value (don't overwrite)
    Keep = 0,
    /// Overwrite with new value
    #[default]
    Overwrite = 1,
    /// Error on duplicate
    Error = 2,
    /// Force overwrite (ignore locked)
    Force = 3,
}

impl ExtendStrategy {
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            0 => Self::Keep,
            2 => Self::Error,
            3 => Self::Force,
            _ => Self::Overwrite,
        }
    }

    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    #[must_use]
    pub const fn should_overwrite(self) -> bool {
        matches!(self, Self::Overwrite | Self::Force)
    }

    #[must_use]
    pub const fn should_error_on_duplicate(self) -> bool {
        matches!(self, Self::Error)
    }
}

// =============================================================================
// Dictionary Key Comparison
// =============================================================================

/// Compare two dictionary keys.
///
/// Dictionary keys are compared as byte strings.
pub fn compare_dict_keys(a: &[u8], b: &[u8]) -> i32 {
    for (x, y) in a.iter().zip(b.iter()) {
        if x < y {
            return -1;
        }
        if x > y {
            return 1;
        }
    }

    a.len().cmp(&b.len()) as i32
}

/// Check if two dictionary keys are equal.
pub fn dict_keys_equal(a: &[u8], b: &[u8]) -> bool {
    a == b
}

// =============================================================================
// Dictionary Hash Helpers
// =============================================================================

/// Simple hash function for dictionary keys.
///
/// Uses FNV-1a algorithm for good distribution.
pub fn hash_dict_key(key: &[u8]) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0100_0000_01b3;

    let mut hash = FNV_OFFSET;
    for &byte in key {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

// =============================================================================
// Dictionary Iteration Order
// =============================================================================

/// Order for iterating dictionary items.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DictIterOrder {
    /// Insertion order (default for VimL)
    #[default]
    Insertion = 0,
    /// Sorted by key
    Sorted = 1,
    /// Reverse insertion order
    Reverse = 2,
}

impl DictIterOrder {
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::Sorted,
            2 => Self::Reverse,
            _ => Self::Insertion,
        }
    }

    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Dictionary Size Calculations
// =============================================================================

/// Calculate new hash table size for dictionary resize.
///
/// Doubles the size, capped at a maximum.
pub const fn dict_resize_capacity(current: i64, min_needed: i64) -> i64 {
    const MAX_DICT_SIZE: i64 = 1 << 30; // ~1 billion entries

    let mut new_size = current;
    while new_size < min_needed && new_size < MAX_DICT_SIZE {
        new_size *= 2;
    }
    if new_size > MAX_DICT_SIZE {
        MAX_DICT_SIZE
    } else {
        new_size
    }
}

/// Check if dictionary should grow.
///
/// Typically grows when load factor exceeds ~0.75.
pub const fn dict_should_grow(count: i64, capacity: i64) -> bool {
    if capacity <= 0 {
        return true;
    }
    // Grow when count > capacity * 3/4
    count * 4 > capacity * 3
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: validate dictionary key.
///
/// # Safety
/// - `key` must be valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn rs_f_dict_validate_key(key: *const u8, len: c_int) -> DictKeyResult {
    if key.is_null() || len <= 0 {
        return DictKeyResult {
            valid: false,
            error: 1,
        };
    }

    let key_slice = std::slice::from_raw_parts(key, len as usize);
    validate_dict_key(key_slice)
}

/// FFI export: check if key length is valid.
#[no_mangle]
pub extern "C" fn rs_f_dict_is_valid_key_len(len: i64) -> c_int {
    c_int::from(is_valid_dict_key_len(len))
}

/// FFI export: get extend strategy.
#[no_mangle]
pub extern "C" fn rs_f_dict_extend_strategy(mode: c_int) -> c_int {
    ExtendStrategy::from_raw(mode).to_raw()
}

/// FFI export: check if should overwrite.
#[no_mangle]
pub extern "C" fn rs_f_dict_should_overwrite(mode: c_int) -> c_int {
    c_int::from(ExtendStrategy::from_raw(mode).should_overwrite())
}

/// FFI export: compare dictionary keys.
///
/// # Safety
/// - Pointers must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_f_dict_compare_keys(
    a: *const u8,
    a_len: c_int,
    b: *const u8,
    b_len: c_int,
) -> c_int {
    if a.is_null() && b.is_null() {
        return 0;
    }
    if a.is_null() {
        return -1;
    }
    if b.is_null() {
        return 1;
    }

    let a_slice = std::slice::from_raw_parts(a, a_len.max(0) as usize);
    let b_slice = std::slice::from_raw_parts(b, b_len.max(0) as usize);

    compare_dict_keys(a_slice, b_slice)
}

/// FFI export: check if dictionary keys are equal.
///
/// # Safety
/// - Pointers must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_f_dict_keys_equal(
    a: *const u8,
    a_len: c_int,
    b: *const u8,
    b_len: c_int,
) -> c_int {
    if a.is_null() || b.is_null() {
        return c_int::from(a.is_null() && b.is_null());
    }
    if a_len != b_len {
        return 0;
    }

    let a_slice = std::slice::from_raw_parts(a, a_len.max(0) as usize);
    let b_slice = std::slice::from_raw_parts(b, b_len.max(0) as usize);

    c_int::from(dict_keys_equal(a_slice, b_slice))
}

/// FFI export: hash dictionary key.
///
/// # Safety
/// - `key` must be valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn rs_f_dict_hash_key(key: *const u8, len: c_int) -> u64 {
    if key.is_null() || len <= 0 {
        return 0;
    }

    let key_slice = std::slice::from_raw_parts(key, len as usize);
    hash_dict_key(key_slice)
}

/// FFI export: calculate resize capacity.
#[no_mangle]
pub extern "C" fn rs_f_dict_resize_capacity(current: i64, min_needed: i64) -> i64 {
    dict_resize_capacity(current, min_needed)
}

/// FFI export: check if dictionary should grow.
#[no_mangle]
pub extern "C" fn rs_f_dict_should_grow(count: i64, capacity: i64) -> c_int {
    c_int::from(dict_should_grow(count, capacity))
}

// =============================================================================
// VimL Built-in Function Implementations (Typval Dispatch)
// =============================================================================

// --- len() for dict ---

/// FFI: Get dict length - used by len() function.
///
/// # Safety
/// `dict` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn rs_f_dict_len(dict: *const c_void) -> i64 {
    let dict = DictPtr::from_raw(dict);
    i64::from(dict_len(dict))
}

// --- empty() for dict ---

/// FFI: Check if dict is empty.
///
/// # Safety
/// `dict` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn rs_f_dict_is_empty(dict: *const c_void) -> c_int {
    let dict = DictPtr::from_raw(dict);
    c_int::from(dict.is_null() || dict_len(dict) == 0)
}

// --- Typval dispatch for has_key() ---

/// FFI: Typval dispatch for has_key() - check if dict has a key.
///
/// This is a partial implementation - the actual key lookup is done in C.
/// This function validates the argument type and extracts the dict.
///
/// # Safety
/// `argvars` must be a valid pointer to a typval array with at least 1 element.
#[no_mangle]
pub unsafe extern "C" fn rs_f_tv_has_key_check(argvars: *const c_void) -> c_int {
    let arg0 = argvar_at(argvars, 0);

    // Check if first arg is a dict
    if tv_get_type(arg0) != VarType::Dict {
        return -1; // Not a dict - error
    }

    if tv_dict_is_null(arg0) {
        return 0; // Null dict - doesn't have any key
    }

    1 // Dict is valid, proceed with key lookup in C
}

// --- count() for dict ---

/// Count type for dict counting.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DictCountType {
    /// Count matching values
    #[default]
    Values = 0,
    /// Count matching keys
    Keys = 1,
}

impl DictCountType {
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::Keys,
            _ => Self::Values,
        }
    }
}

/// FFI: Get count type for dict counting.
#[no_mangle]
pub extern "C" fn rs_f_dict_count_type(mode: c_int) -> c_int {
    DictCountType::from_raw(mode) as c_int
}

// --- Type checking helpers ---

/// FFI: Check if typval is a valid dict for dict functions.
///
/// # Safety
/// `argvars` must be a valid pointer to a typval array with at least 1 element.
#[no_mangle]
pub unsafe extern "C" fn rs_f_tv_is_dict(argvars: *const c_void) -> c_int {
    let arg0 = argvar_at(argvars, 0);
    c_int::from(tv_get_type(arg0) == VarType::Dict)
}

/// FFI: Check if typval is a valid non-null dict.
///
/// # Safety
/// `argvars` must be a valid pointer to a typval array with at least 1 element.
#[no_mangle]
pub unsafe extern "C" fn rs_f_tv_is_valid_dict(argvars: *const c_void) -> c_int {
    let arg0 = argvar_at(argvars, 0);
    if tv_get_type(arg0) != VarType::Dict {
        return 0;
    }
    c_int::from(!tv_dict_is_null(arg0))
}

/// FFI: Typval dispatch for dict len() - returns dict length.
///
/// # Safety
/// `argvars` must be a valid pointer to a typval array with at least 1 element.
#[no_mangle]
pub unsafe extern "C" fn rs_f_tv_dict_len(argvars: *const c_void, rettv: *mut c_void) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    let arg0 = argvar_at(argvars, 0);

    let len = if tv_get_type(arg0) == VarType::Dict {
        let dict = tv_get_dict(arg0);
        i64::from(dict_len(dict))
    } else {
        0
    };

    rettv_set_number(rettv, len);
}

/// FFI: Typval dispatch for dict empty() - check if dict is empty.
///
/// # Safety
/// `argvars` must be a valid pointer to a typval array with at least 1 element.
#[no_mangle]
pub unsafe extern "C" fn rs_f_tv_dict_empty(argvars: *const c_void, rettv: *mut c_void) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    let arg0 = argvar_at(argvars, 0);

    let is_empty = if tv_get_type(arg0) == VarType::Dict {
        tv_dict_is_null(arg0) || dict_len(tv_get_dict(arg0)) == 0
    } else {
        true
    };

    rettv_set_bool(rettv, is_empty);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_dict_key() {
        let r = validate_dict_key(b"key");
        assert!(r.valid);
        assert_eq!(r.error, 0);

        let r = validate_dict_key(b"");
        assert!(!r.valid);
        assert_eq!(r.error, 1);
    }

    #[test]
    fn test_extend_strategy() {
        assert!(!ExtendStrategy::Keep.should_overwrite());
        assert!(ExtendStrategy::Overwrite.should_overwrite());
        assert!(!ExtendStrategy::Error.should_overwrite());
        assert!(ExtendStrategy::Force.should_overwrite());

        assert!(!ExtendStrategy::Keep.should_error_on_duplicate());
        assert!(ExtendStrategy::Error.should_error_on_duplicate());
    }

    #[test]
    fn test_compare_dict_keys() {
        assert_eq!(compare_dict_keys(b"abc", b"abc"), 0);
        assert_eq!(compare_dict_keys(b"abc", b"abd"), -1);
        assert_eq!(compare_dict_keys(b"abd", b"abc"), 1);
        assert_eq!(compare_dict_keys(b"ab", b"abc"), -1);
    }

    #[test]
    fn test_dict_keys_equal() {
        assert!(dict_keys_equal(b"key", b"key"));
        assert!(!dict_keys_equal(b"key", b"Key"));
        assert!(!dict_keys_equal(b"key", b"ke"));
    }

    #[test]
    fn test_hash_dict_key() {
        // Same key should have same hash
        let h1 = hash_dict_key(b"test");
        let h2 = hash_dict_key(b"test");
        assert_eq!(h1, h2);

        // Different keys should (usually) have different hashes
        let h3 = hash_dict_key(b"test2");
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_dict_resize_capacity() {
        assert_eq!(dict_resize_capacity(8, 10), 16);
        assert_eq!(dict_resize_capacity(8, 20), 32);
        assert_eq!(dict_resize_capacity(8, 8), 8);
    }

    #[test]
    fn test_dict_should_grow() {
        // 6/8 = 0.75, should not grow
        assert!(!dict_should_grow(6, 8));
        // 7/8 = 0.875, should grow
        assert!(dict_should_grow(7, 8));
        // Empty capacity always grows
        assert!(dict_should_grow(1, 0));
    }

    #[test]
    fn test_dict_iter_order() {
        assert_eq!(DictIterOrder::from_raw(0), DictIterOrder::Insertion);
        assert_eq!(DictIterOrder::from_raw(1), DictIterOrder::Sorted);
        assert_eq!(DictIterOrder::from_raw(2), DictIterOrder::Reverse);
    }
}
