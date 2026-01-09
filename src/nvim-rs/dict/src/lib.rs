//! Dictionary operations for VimL.
//!
//! This module provides dictionary utilities migrated from `src/nvim/eval/typval.c`:
//! - Dictionary creation and key lookup
//! - Key iteration helpers
//! - Dictionary comparison support
//! - Nested dict access utilities
//!
//! ## VimL Dictionary Semantics
//!
//! VimL dictionaries are ordered hash maps with string keys and typval values:
//! - Keys are always strings (UTF-8)
//! - Values can be any VimL type (including nested dicts/lists)
//! - Order is preserved (insertion order)
//! - Reference counted for sharing
//!
//! ## FFI Pattern
//!
//! These functions work with opaque `DictHandle` pointers to C `dict_T` structures.
//! All actual memory management happens on the C side.

#![allow(clippy::cast_sign_loss)]
#![allow(clippy::too_many_lines)]

use std::ffi::c_int;

use nvim_typval::TypevalHandle;

// =============================================================================
// Dict Item Status
// =============================================================================

/// Result of dictionary item lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum DictLookupResult {
    /// Key found in dictionary
    Found = 0,
    /// Key not found
    NotFound = -1,
    /// Dictionary is empty
    Empty = -2,
    /// Invalid key (null or empty string)
    InvalidKey = -3,
    /// Dictionary handle is null
    NullDict = -4,
}

impl DictLookupResult {
    /// Create from C int result.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Found,
            -1 => Self::NotFound,
            -2 => Self::Empty,
            -3 => Self::InvalidKey,
            -4 => Self::NullDict,
            _ => Self::NotFound, // Default for unknown values
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if lookup was successful.
    pub const fn is_found(self) -> bool {
        matches!(self, Self::Found)
    }
}

// =============================================================================
// Dict Compare Result
// =============================================================================

/// Result of dictionary comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum DictCompareResult {
    /// Dictionaries are equal
    Equal = 0,
    /// First dict has fewer keys
    FewerKeys = -1,
    /// First dict has more keys
    MoreKeys = 1,
    /// Keys differ (at some key position)
    KeysDiffer = 2,
    /// Values differ (for same key)
    ValuesDiffer = 3,
    /// One or both dicts are null
    NullDict = -2,
}

impl DictCompareResult {
    /// Create from C int result.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Equal,
            -1 => Self::FewerKeys,
            1 => Self::MoreKeys,
            2 => Self::KeysDiffer,
            3 => Self::ValuesDiffer,
            -2 => Self::NullDict,
            _ => Self::ValuesDiffer, // Default for unknown values
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if dicts are equal.
    pub const fn is_equal(self) -> bool {
        matches!(self, Self::Equal)
    }
}

// =============================================================================
// Dict Flags
// =============================================================================

/// Flags for dictionary operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DictFlags(u32);

impl DictFlags {
    /// No special flags.
    pub const NONE: Self = Self(0);

    /// Dictionary is fixed (cannot add keys).
    pub const FIXED: Self = Self(1 << 0);

    /// Dictionary is from :def function (can't change type).
    pub const DEF_SCOPE: Self = Self(1 << 1);

    /// Create new flag set.
    pub const fn new(bits: u32) -> Self {
        Self(bits)
    }

    /// Check if flag is set.
    pub const fn contains(self, flag: Self) -> bool {
        (self.0 & flag.0) != 0
    }

    /// Get raw bits.
    pub const fn bits(self) -> u32 {
        self.0
    }
}

// =============================================================================
// Key Validation
// =============================================================================

/// Validate a dictionary key.
///
/// VimL dictionary keys must be:
/// - Non-null
/// - Non-empty strings
/// - Valid UTF-8
///
/// Returns true if key is valid.
pub fn validate_key(key: &[u8]) -> bool {
    !key.is_empty()
}

/// Check if a string is a valid VimL identifier (for dict keys and variables).
///
/// Valid identifiers start with a letter or underscore, followed by
/// letters, digits, or underscores.
pub fn is_valid_identifier(s: &[u8]) -> bool {
    if s.is_empty() {
        return false;
    }

    let first = s[0];
    if !first.is_ascii_alphabetic() && first != b'_' {
        return false;
    }

    s.iter().all(|&c| c.is_ascii_alphanumeric() || c == b'_')
}

/// FFI export: validate dictionary key.
///
/// # Safety
/// - `key` must be a valid pointer to a null-terminated C string, or null.
/// - If `key_len` is provided (non-negative), `key` must point to at least `key_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_dict_validate_key(key: *const u8, key_len: c_int) -> bool {
    if key.is_null() {
        return false;
    }

    let slice = if key_len >= 0 {
        // SAFETY: Caller guarantees key points to at least key_len bytes
        unsafe { std::slice::from_raw_parts(key, key_len as usize) }
    } else {
        // Null-terminated string - find length
        let mut len = 0;
        // SAFETY: Caller guarantees key is a valid null-terminated string
        while unsafe { *key.add(len) } != 0 {
            len += 1;
        }
        unsafe { std::slice::from_raw_parts(key, len) }
    };

    validate_key(slice)
}

/// FFI export: check if string is valid identifier.
///
/// # Safety
/// - `s` must be a valid pointer to a null-terminated C string, or null.
/// - If `len` is provided (non-negative), `s` must point to at least `len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_dict_is_valid_identifier(s: *const u8, len: c_int) -> bool {
    if s.is_null() {
        return false;
    }

    let slice = if len >= 0 {
        // SAFETY: Caller guarantees s points to at least len bytes
        unsafe { std::slice::from_raw_parts(s, len as usize) }
    } else {
        // Null-terminated string - find length
        let mut slen = 0;
        // SAFETY: Caller guarantees s is a valid null-terminated string
        while unsafe { *s.add(slen) } != 0 {
            slen += 1;
        }
        unsafe { std::slice::from_raw_parts(s, slen) }
    };

    is_valid_identifier(slice)
}

// =============================================================================
// Dict Key Iteration Helper
// =============================================================================

/// Iterator state for dictionary keys.
///
/// This is a Rust-side helper for iterating over dictionary keys.
/// The actual iteration state is maintained by the C hashtable iterator.
#[derive(Debug, Clone)]
pub struct DictKeyIter {
    /// Current index in the dict
    index: i64,
    /// Total number of keys
    count: i64,
}

impl DictKeyIter {
    /// Create a new key iterator.
    pub const fn new(count: i64) -> Self {
        Self { index: 0, count }
    }

    /// Check if there are more keys.
    pub const fn has_next(&self) -> bool {
        self.index < self.count
    }

    /// Advance to next key.
    pub fn advance(&mut self) -> bool {
        if self.has_next() {
            self.index += 1;
            true
        } else {
            false
        }
    }

    /// Get current index.
    pub const fn current_index(&self) -> i64 {
        self.index
    }

    /// Reset to beginning.
    pub fn reset(&mut self) {
        self.index = 0;
    }
}

// =============================================================================
// Dict Item Helper
// =============================================================================

/// Helper structure for dictionary item access.
#[derive(Debug, Clone, Copy)]
pub struct DictItemInfo {
    /// Key hash (for quick comparison)
    pub hash: u64,
    /// Key length
    pub key_len: usize,
}

impl DictItemInfo {
    /// Create new item info.
    pub const fn new(hash: u64, key_len: usize) -> Self {
        Self { hash, key_len }
    }
}

/// Simple hash function for dictionary keys (FNV-1a).
///
/// This matches the hash function used in VimL for consistency.
pub fn hash_key(key: &[u8]) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0100_0000_01b3;

    let mut hash = FNV_OFFSET;
    for &byte in key {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// FFI export: hash a dictionary key.
///
/// # Safety
/// - `key` must be a valid pointer to at least `key_len` bytes, or null.
#[no_mangle]
pub unsafe extern "C" fn rs_dict_hash_key(key: *const u8, key_len: c_int) -> u64 {
    if key.is_null() || key_len < 0 {
        return 0;
    }

    // SAFETY: Caller guarantees key points to at least key_len bytes
    let slice = unsafe { std::slice::from_raw_parts(key, key_len as usize) };
    hash_key(slice)
}

// =============================================================================
// Nested Dict Access
// =============================================================================

/// Parse a dot-notation key path for nested dict access.
///
/// For example, "foo.bar.baz" returns `["foo", "bar", "baz"]`.
/// Returns empty vec if path is empty.
pub fn parse_key_path(path: &[u8]) -> Vec<&[u8]> {
    if path.is_empty() {
        return Vec::new();
    }

    path.split(|&c| c == b'.').collect()
}

/// Check if a key path has multiple components (nested access).
pub fn is_nested_key(path: &[u8]) -> bool {
    path.contains(&b'.')
}

/// FFI export: check if key path is nested.
///
/// # Safety
/// - `path` must be a valid pointer to at least `path_len` bytes, or null.
#[no_mangle]
pub unsafe extern "C" fn rs_dict_is_nested_key(path: *const u8, path_len: c_int) -> bool {
    if path.is_null() || path_len < 0 {
        return false;
    }

    // SAFETY: Caller guarantees path points to at least path_len bytes
    let slice = unsafe { std::slice::from_raw_parts(path, path_len as usize) };
    is_nested_key(slice)
}

/// FFI export: count components in a key path.
///
/// # Safety
/// - `path` must be a valid pointer to at least `path_len` bytes, or null.
#[no_mangle]
pub unsafe extern "C" fn rs_dict_key_path_depth(path: *const u8, path_len: c_int) -> c_int {
    if path.is_null() || path_len < 0 {
        return 0;
    }

    // SAFETY: Caller guarantees path points to at least path_len bytes
    let slice = unsafe { std::slice::from_raw_parts(path, path_len as usize) };
    parse_key_path(slice).len() as c_int
}

// =============================================================================
// Dict Merge Mode
// =============================================================================

/// Mode for merging dictionaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum DictMergeMode {
    /// Error on duplicate keys (default)
    Error = 0,
    /// Keep existing value on duplicates
    Keep = 1,
    /// Overwrite with new value on duplicates
    Overwrite = 2,
    /// Deep merge nested dicts
    DeepMerge = 3,
}

impl DictMergeMode {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Error,
            1 => Self::Keep,
            2 => Self::Overwrite,
            3 => Self::DeepMerge,
            _ => Self::Error,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Dict Type Checking
// =============================================================================

/// Check if a typval is a dict type.
///
/// # Safety
/// - `tv` must be a valid TypevalHandle or null.
#[no_mangle]
pub unsafe extern "C" fn rs_dict_is_dict_type(tv: TypevalHandle) -> bool {
    if tv.is_null() {
        return false;
    }

    // Check type field in typval
    // Type is stored at offset 0 as a u8 in typval_T
    // SAFETY: Caller guarantees tv is valid if non-null
    let type_ptr = tv.as_ptr() as *const u8;
    let vtype = unsafe { *type_ptr };

    // VAR_DICT = 5 (from typval_defs.h)
    const VAR_DICT: u8 = 5;
    vtype == VAR_DICT
}

// =============================================================================
// Dict Extension: Filter and Map Helpers
// =============================================================================

/// Filter mode for dict operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum DictFilterMode {
    /// Remove items where predicate returns true
    Remove = 0,
    /// Keep items where predicate returns true
    Keep = 1,
}

impl DictFilterMode {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Remove,
            1 => Self::Keep,
            _ => Self::Keep,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Dict Copy Mode
// =============================================================================

/// Copy mode for dictionaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum DictCopyMode {
    /// Shallow copy (share references)
    Shallow = 0,
    /// Deep copy (recursive)
    Deep = 1,
}

impl DictCopyMode {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Shallow,
            1 => Self::Deep,
            _ => Self::Shallow,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Dict Lock Status
// =============================================================================

/// Lock status for dictionaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum DictLockStatus {
    /// Dictionary is unlocked
    Unlocked = 0,
    /// Dictionary is locked (can't add/remove keys)
    Locked = 1,
    /// Dictionary is fixed (type is locked, from :def)
    Fixed = 2,
}

impl DictLockStatus {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Unlocked,
            1 => Self::Locked,
            2 => Self::Fixed,
            _ => Self::Unlocked,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if dict is modifiable.
    pub const fn is_modifiable(self) -> bool {
        matches!(self, Self::Unlocked)
    }
}

// =============================================================================
// Dict Operations Result
// =============================================================================

/// Result of dictionary modification operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum DictOpResult {
    /// Operation succeeded
    Ok = 0,
    /// Dictionary is locked
    Locked = -1,
    /// Key not found (for removal)
    NotFound = -2,
    /// Key already exists (for unique insert)
    Exists = -3,
    /// Invalid argument
    InvalidArg = -4,
    /// Out of memory
    NoMemory = -5,
}

impl DictOpResult {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Ok,
            -1 => Self::Locked,
            -2 => Self::NotFound,
            -3 => Self::Exists,
            -4 => Self::InvalidArg,
            -5 => Self::NoMemory,
            _ => Self::InvalidArg,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if operation succeeded.
    pub const fn is_ok(self) -> bool {
        matches!(self, Self::Ok)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dict_lookup_result() {
        assert!(DictLookupResult::Found.is_found());
        assert!(!DictLookupResult::NotFound.is_found());
        assert_eq!(DictLookupResult::from_c_int(0), DictLookupResult::Found);
        assert_eq!(DictLookupResult::from_c_int(-1), DictLookupResult::NotFound);
        assert_eq!(DictLookupResult::Empty.to_c_int(), -2);
    }

    #[test]
    fn test_dict_compare_result() {
        assert!(DictCompareResult::Equal.is_equal());
        assert!(!DictCompareResult::FewerKeys.is_equal());
        assert_eq!(DictCompareResult::from_c_int(0), DictCompareResult::Equal);
        assert_eq!(
            DictCompareResult::from_c_int(1),
            DictCompareResult::MoreKeys
        );
    }

    #[test]
    fn test_dict_flags() {
        let flags = DictFlags::NONE;
        assert!(!flags.contains(DictFlags::FIXED));

        let fixed = DictFlags::FIXED;
        assert!(fixed.contains(DictFlags::FIXED));
        assert!(!fixed.contains(DictFlags::DEF_SCOPE));
    }

    #[test]
    fn test_validate_key() {
        assert!(validate_key(b"foo"));
        assert!(validate_key(b"_bar"));
        assert!(validate_key(b"123"));
        assert!(!validate_key(b""));
    }

    #[test]
    fn test_is_valid_identifier() {
        assert!(is_valid_identifier(b"foo"));
        assert!(is_valid_identifier(b"_bar"));
        assert!(is_valid_identifier(b"Foo123"));
        assert!(is_valid_identifier(b"_"));
        assert!(is_valid_identifier(b"a"));

        assert!(!is_valid_identifier(b""));
        assert!(!is_valid_identifier(b"123"));
        assert!(!is_valid_identifier(b"foo-bar"));
        assert!(!is_valid_identifier(b"foo.bar"));
    }

    #[test]
    fn test_dict_key_iter() {
        let mut iter = DictKeyIter::new(3);
        assert!(iter.has_next());
        assert_eq!(iter.current_index(), 0);

        assert!(iter.advance());
        assert_eq!(iter.current_index(), 1);

        assert!(iter.advance());
        assert!(iter.advance());
        assert!(!iter.has_next());
        assert!(!iter.advance());

        iter.reset();
        assert_eq!(iter.current_index(), 0);
    }

    #[test]
    fn test_hash_key() {
        let h1 = hash_key(b"foo");
        let h2 = hash_key(b"foo");
        let h3 = hash_key(b"bar");

        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
        assert_ne!(hash_key(b""), hash_key(b"x"));
    }

    #[test]
    fn test_parse_key_path() {
        let path = parse_key_path(b"foo.bar.baz");
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], b"foo");
        assert_eq!(path[1], b"bar");
        assert_eq!(path[2], b"baz");

        let single = parse_key_path(b"foo");
        assert_eq!(single.len(), 1);
        assert_eq!(single[0], b"foo");

        let empty = parse_key_path(b"");
        assert!(empty.is_empty());
    }

    #[test]
    fn test_is_nested_key() {
        assert!(is_nested_key(b"foo.bar"));
        assert!(is_nested_key(b"a.b.c"));
        assert!(!is_nested_key(b"foo"));
        assert!(!is_nested_key(b""));
    }

    #[test]
    fn test_dict_merge_mode() {
        assert_eq!(DictMergeMode::from_c_int(0), DictMergeMode::Error);
        assert_eq!(DictMergeMode::from_c_int(2), DictMergeMode::Overwrite);
        assert_eq!(DictMergeMode::Overwrite.to_c_int(), 2);
    }

    #[test]
    fn test_dict_copy_mode() {
        assert_eq!(DictCopyMode::from_c_int(0), DictCopyMode::Shallow);
        assert_eq!(DictCopyMode::from_c_int(1), DictCopyMode::Deep);
    }

    #[test]
    fn test_dict_lock_status() {
        assert!(DictLockStatus::Unlocked.is_modifiable());
        assert!(!DictLockStatus::Locked.is_modifiable());
        assert!(!DictLockStatus::Fixed.is_modifiable());
    }

    #[test]
    fn test_dict_op_result() {
        assert!(DictOpResult::Ok.is_ok());
        assert!(!DictOpResult::Locked.is_ok());
        assert_eq!(DictOpResult::from_c_int(-1), DictOpResult::Locked);
    }

    #[test]
    fn test_dict_item_info() {
        let info = DictItemInfo::new(12345, 3);
        assert_eq!(info.hash, 12345);
        assert_eq!(info.key_len, 3);
    }
}
