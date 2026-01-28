//! List manipulation functions for VimL.
//!
//! This module implements list functions from `src/nvim/eval/funcs.c`:
//! - `len()` - get length of list/dict/string/blob
//! - `empty()` - check if list/dict/string/blob is empty
//! - `count()` - count occurrences in list/dict/string
//! - `get()` - get item from list/dict/blob with default
//! - `add()` - append item to list
//! - `insert()` - insert item at position
//! - `remove()` - remove item at position
//! - `extend()` - extend list with another list
//! - `copy()` - shallow copy list/dict
//! - `deepcopy()` - deep copy list/dict
//! - `sort()` - sort list
//! - `uniq()` - remove duplicates from sorted list
//! - `reverse()` - reverse list
//! - `filter()` - filter list by predicate
//! - `map()` - transform list items
//! - `reduce()` - reduce list to single value
//! - `index()` - find index of item in list
//! - `indexof()` - find index of item using predicate
//! - `flatten()` - flatten nested lists
//! - `flattennew()` - flatten nested lists (returns new list)

#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::must_use_candidate)]

use std::ffi::{c_int, c_void};

use super::dispatch::{
    argvar_at, dict_len, list_len, rettv_set_bool, rettv_set_number, tv_blob_len, tv_dict_is_null,
    tv_get_dict, tv_get_list, tv_get_string_bytes, tv_get_type, tv_list_is_null, DictPtr, ListPtr,
    TypevalPtrMut, VarType,
};

// =============================================================================
// List Index Operations
// =============================================================================

/// Result of a list index validation.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ListIndexResult {
    /// Whether the index is valid
    pub valid: bool,
    /// The normalized index (0-based, positive)
    pub index: c_int,
}

/// Validate and normalize a VimL list index.
///
/// VimL uses 0-based indexing with negative indices counting from the end.
/// - `0` is first element
/// - `-1` is last element
/// - `-len` is first element
pub const fn validate_list_index(index: i64, len: i64) -> ListIndexResult {
    if len <= 0 {
        return ListIndexResult {
            valid: false,
            index: 0,
        };
    }

    let normalized = if index < 0 { len + index } else { index };

    if normalized >= 0 && normalized < len {
        ListIndexResult {
            valid: true,
            index: normalized as c_int,
        }
    } else {
        ListIndexResult {
            valid: false,
            index: 0,
        }
    }
}

/// Validate list index for insertion (allows index == len for append).
pub const fn validate_insert_index(index: i64, len: i64) -> ListIndexResult {
    if len < 0 {
        return ListIndexResult {
            valid: false,
            index: 0,
        };
    }

    let normalized = if index < 0 { len + index + 1 } else { index };

    if normalized >= 0 && normalized <= len {
        ListIndexResult {
            valid: true,
            index: normalized as c_int,
        }
    } else {
        ListIndexResult {
            valid: false,
            index: 0,
        }
    }
}

// =============================================================================
// List Range Operations
// =============================================================================

/// Result of a list range validation.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ListRangeResult {
    /// Whether the range is valid
    pub valid: bool,
    /// Start index (normalized)
    pub start: c_int,
    /// End index (normalized, exclusive)
    pub end: c_int,
}

/// Validate and normalize a VimL list range (for slicing).
///
/// VimL slice notation: `list[start:end]`
/// - Both indices are inclusive in VimL
/// - Negative indices count from end
/// - Missing start defaults to 0
/// - Missing end defaults to len-1
pub const fn validate_list_range(
    start: i64,
    has_start: bool,
    end: i64,
    has_end: bool,
    len: i64,
) -> ListRangeResult {
    if len <= 0 {
        return ListRangeResult {
            valid: true,
            start: 0,
            end: 0,
        };
    }

    // Normalize start
    let norm_start = if !has_start {
        0
    } else if start < 0 {
        let s = len + start;
        if s < 0 {
            0
        } else {
            s
        }
    } else if start >= len {
        len
    } else {
        start
    };

    // Normalize end (VimL end is inclusive, we convert to exclusive)
    let norm_end = if !has_end {
        len
    } else if end < 0 {
        let e = len + end + 1;
        if e < 0 {
            0
        } else {
            e
        }
    } else if end >= len {
        len
    } else {
        end + 1 // Convert inclusive to exclusive
    };

    // Empty range if start >= end
    if norm_start >= norm_end {
        return ListRangeResult {
            valid: true,
            start: 0,
            end: 0,
        };
    }

    ListRangeResult {
        valid: true,
        start: norm_start as c_int,
        end: norm_end as c_int,
    }
}

// =============================================================================
// List Comparison Operations
// =============================================================================

/// Compare two i64 values for sorting.
#[must_use]
pub const fn compare_i64(a: i64, b: i64) -> i32 {
    if a < b {
        -1
    } else if a > b {
        1
    } else {
        0
    }
}

/// Compare two strings lexicographically.
pub fn compare_strings(a: &[u8], b: &[u8]) -> i32 {
    for (x, y) in a.iter().zip(b.iter()) {
        if x < y {
            return -1;
        }
        if x > y {
            return 1;
        }
    }
    compare_i64(a.len() as i64, b.len() as i64)
}

/// Compare two strings case-insensitively.
pub fn compare_strings_ic(a: &[u8], b: &[u8]) -> i32 {
    for (x, y) in a.iter().zip(b.iter()) {
        let x_lower = x.to_ascii_lowercase();
        let y_lower = y.to_ascii_lowercase();
        if x_lower < y_lower {
            return -1;
        }
        if x_lower > y_lower {
            return 1;
        }
    }
    compare_i64(a.len() as i64, b.len() as i64)
}

// =============================================================================
// List Flatten Operations
// =============================================================================

/// Maximum recursion depth for flatten.
pub const MAX_FLATTEN_DEPTH: i64 = 999;

/// Check if flatten depth is valid.
pub const fn is_valid_flatten_depth(depth: i64) -> bool {
    depth >= 0 && depth <= MAX_FLATTEN_DEPTH
}

// =============================================================================
// List Repeat Operations
// =============================================================================

/// Calculate result size for list repeat.
///
/// VimL: `repeat(list, count)`
/// Returns the total number of elements in the result.
pub const fn repeat_result_len(list_len: i64, count: i64) -> i64 {
    if count <= 0 || list_len <= 0 {
        0
    } else {
        list_len.saturating_mul(count)
    }
}

// =============================================================================
// List Copy Operations
// =============================================================================

/// Copy type for list/dict copy operations.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CopyType {
    /// Shallow copy - only top level is copied
    #[default]
    Shallow = 0,
    /// Deep copy - recursively copy all nested structures
    Deep = 1,
}

impl CopyType {
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::Deep,
            _ => Self::Shallow,
        }
    }

    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    #[must_use]
    pub const fn is_deep(self) -> bool {
        matches!(self, Self::Deep)
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: validate and normalize list index.
#[no_mangle]
pub extern "C" fn rs_f_list_validate_index(index: i64, len: i64) -> ListIndexResult {
    validate_list_index(index, len)
}

/// FFI export: validate list index for insertion.
#[no_mangle]
pub extern "C" fn rs_f_list_validate_insert_index(index: i64, len: i64) -> ListIndexResult {
    validate_insert_index(index, len)
}

/// FFI export: validate and normalize list range.
#[no_mangle]
pub extern "C" fn rs_f_list_validate_range(
    start: i64,
    has_start: c_int,
    end: i64,
    has_end: c_int,
    len: i64,
) -> ListRangeResult {
    validate_list_range(start, has_start != 0, end, has_end != 0, len)
}

/// FFI export: compare i64 values.
#[no_mangle]
pub extern "C" fn rs_f_list_compare_i64(a: i64, b: i64) -> c_int {
    compare_i64(a, b)
}

/// FFI export: compare strings.
///
/// # Safety
/// - Pointers must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_f_list_compare_strings(
    a: *const u8,
    a_len: c_int,
    b: *const u8,
    b_len: c_int,
    ignore_case: c_int,
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

    if ignore_case != 0 {
        compare_strings_ic(a_slice, b_slice)
    } else {
        compare_strings(a_slice, b_slice)
    }
}

/// FFI export: check if flatten depth is valid.
#[no_mangle]
pub extern "C" fn rs_f_list_is_valid_flatten_depth(depth: i64) -> c_int {
    c_int::from(is_valid_flatten_depth(depth))
}

/// FFI export: calculate repeat result length.
#[no_mangle]
pub extern "C" fn rs_f_list_repeat_len(list_len: i64, count: i64) -> i64 {
    repeat_result_len(list_len, count)
}

/// FFI export: get copy type.
#[no_mangle]
pub extern "C" fn rs_f_list_copy_type(deep: c_int) -> c_int {
    CopyType::from_raw(deep).to_raw()
}

/// FFI export: Get max flatten depth constant.
#[no_mangle]
pub extern "C" fn rs_list_max_flatten_depth() -> i64 {
    MAX_FLATTEN_DEPTH
}

/// FFI export: Check if copy type is deep.
#[no_mangle]
pub extern "C" fn rs_list_copy_is_deep(copy_type: c_int) -> c_int {
    c_int::from(CopyType::from_raw(copy_type).is_deep())
}

/// FFI export: Get CopyType::Shallow constant.
#[no_mangle]
pub extern "C" fn rs_list_copy_shallow() -> c_int {
    CopyType::Shallow.to_raw()
}

/// FFI export: Get CopyType::Deep constant.
#[no_mangle]
pub extern "C" fn rs_list_copy_deep() -> c_int {
    CopyType::Deep.to_raw()
}

/// FFI export: Check if list index result is valid.
#[no_mangle]
pub extern "C" fn rs_list_index_valid(result: ListIndexResult) -> c_int {
    c_int::from(result.valid)
}

/// FFI export: Get normalized index from result.
#[no_mangle]
pub extern "C" fn rs_list_index_value(result: ListIndexResult) -> c_int {
    result.index
}

/// FFI export: Check if list range result is valid.
#[no_mangle]
pub extern "C" fn rs_list_range_valid(result: ListRangeResult) -> c_int {
    c_int::from(result.valid)
}

/// FFI export: Get start index from range result.
#[no_mangle]
pub extern "C" fn rs_list_range_start(result: ListRangeResult) -> c_int {
    result.start
}

/// FFI export: Get end index from range result.
#[no_mangle]
pub extern "C" fn rs_list_range_end(result: ListRangeResult) -> c_int {
    result.end
}

/// FFI export: Get range length.
#[no_mangle]
pub extern "C" fn rs_list_range_len(result: ListRangeResult) -> c_int {
    if result.valid {
        result.end - result.start
    } else {
        0
    }
}

// =============================================================================
// VimL Built-in Function Implementations (Typval Dispatch)
// =============================================================================

// --- len() function ---

/// Compute length for various types.
///
/// VimL `len()` returns:
/// - String: byte length (strlen)
/// - Number: byte length of string representation
/// - List: number of items
/// - Dict: number of key-value pairs
/// - Blob: number of bytes
/// - Other types: error (handled by C caller)
fn compute_len(list: ListPtr, dict: DictPtr, blob_len: c_int, string_len: usize) -> i64 {
    if !list.is_null() {
        i64::from(list_len(list))
    } else if !dict.is_null() {
        i64::from(dict_len(dict))
    } else if blob_len > 0 {
        i64::from(blob_len)
    } else {
        string_len as i64
    }
}

/// FFI: len() helper - computes length based on type.
///
/// This is called from C after type checking. The C code passes the appropriate
/// values based on the type.
///
/// # Safety
/// `list` and `dict` must be valid pointers or null.
#[no_mangle]
pub unsafe extern "C" fn rs_f_len_impl(
    list: *const c_void,
    dict: *const c_void,
    blob_len: c_int,
    string_len: usize,
) -> i64 {
    let list = ListPtr::from_raw(list);
    let dict = DictPtr::from_raw(dict);
    compute_len(list, dict, blob_len, string_len)
}

// --- empty() function ---
//
// VimL `empty()` returns true for:
// - String/Func: NULL or empty string
// - Number: 0
// - Float: 0.0
// - List: NULL or empty
// - Dict: NULL or empty
// - Bool: v:false
// - Partial: always false (never empty)
// - Blob: NULL or empty

/// Check if a list is empty.
#[inline]
fn list_is_empty(list: ListPtr) -> bool {
    list.is_null() || list_len(list) == 0
}

/// Check if a dict is empty.
#[inline]
fn dict_is_empty(dict: DictPtr) -> bool {
    dict.is_null() || dict_len(dict) == 0
}

/// FFI: empty() for list type.
///
/// # Safety
/// `list` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn rs_f_empty_list(list: *const c_void) -> c_int {
    let list = ListPtr::from_raw(list);
    c_int::from(list_is_empty(list))
}

/// FFI: empty() for dict type.
///
/// # Safety
/// `dict` must be a valid pointer or null.
#[no_mangle]
pub unsafe extern "C" fn rs_f_empty_dict(dict: *const c_void) -> c_int {
    let dict = DictPtr::from_raw(dict);
    c_int::from(dict_is_empty(dict))
}

/// FFI: Typval dispatch for len() - handles List, Dict, Blob, String, Number types.
///
/// # Safety
/// `argvars` must be a valid pointer to a typval array with at least 1 element.
#[no_mangle]
pub unsafe extern "C" fn rs_f_tv_len(argvars: *const c_void, rettv: *mut c_void) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    let arg0 = argvar_at(argvars, 0);

    let len = match tv_get_type(arg0) {
        VarType::List => {
            let list = tv_get_list(arg0);
            i64::from(list_len(list))
        }
        VarType::Dict => {
            let dict = tv_get_dict(arg0);
            i64::from(dict_len(dict))
        }
        VarType::Blob => i64::from(tv_blob_len(arg0)),
        VarType::String | VarType::Number => {
            let s = tv_get_string_bytes(arg0);
            s.len() as i64
        }
        // Other types are errors - C caller should have validated
        _ => 0,
    };

    rettv_set_number(rettv, len);
}

/// FFI: Typval dispatch for empty() - handles all types.
///
/// # Safety
/// `argvars` must be a valid pointer to a typval array with at least 1 element.
#[no_mangle]
pub unsafe extern "C" fn rs_f_tv_empty(argvars: *const c_void, rettv: *mut c_void) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    let arg0 = argvar_at(argvars, 0);

    let is_empty = match tv_get_type(arg0) {
        VarType::List => tv_list_is_null(arg0) || list_len(tv_get_list(arg0)) == 0,
        VarType::Dict => tv_dict_is_null(arg0) || dict_len(tv_get_dict(arg0)) == 0,
        VarType::Blob => tv_blob_len(arg0) == 0,
        VarType::String | VarType::Func => {
            let s = tv_get_string_bytes(arg0);
            s.is_empty()
        }
        VarType::Number => {
            // Check if number is 0 - use string representation
            let s = tv_get_string_bytes(arg0);
            s == b"0"
        }
        VarType::Float => {
            // Float 0.0 is empty
            let s = tv_get_string_bytes(arg0);
            s.starts_with(b"0.0")
        }
        VarType::Bool => {
            // v:false is empty, v:true is not
            let s = tv_get_string_bytes(arg0);
            s == b"v:false"
        }
        VarType::Special => {
            // v:null and v:none are empty
            let s = tv_get_string_bytes(arg0);
            s == b"v:null" || s == b"v:none"
        }
        VarType::Partial => false, // Partial is never empty
        VarType::Unknown => true,
    };

    rettv_set_bool(rettv, is_empty);
}

// --- count() function helpers ---

/// Count occurrences of a byte in a byte slice.
#[allow(clippy::naive_bytecount)]
pub fn count_byte_in_slice(haystack: &[u8], needle: u8) -> i64 {
    haystack.iter().filter(|&&b| b == needle).count() as i64
}

/// Count non-overlapping occurrences of a substring in a string.
pub fn count_substring(haystack: &[u8], needle: &[u8], ignore_case: bool) -> i64 {
    if needle.is_empty() || haystack.len() < needle.len() {
        return 0;
    }

    let mut count = 0i64;
    let mut pos = 0;

    while pos + needle.len() <= haystack.len() {
        let matches = if ignore_case {
            haystack[pos..pos + needle.len()]
                .iter()
                .zip(needle.iter())
                .all(|(a, b)| a.eq_ignore_ascii_case(b))
        } else {
            &haystack[pos..pos + needle.len()] == needle
        };

        if matches {
            count += 1;
            pos += needle.len(); // Non-overlapping
        } else {
            pos += 1;
        }
    }

    count
}

/// FFI: count() for string - count substring occurrences.
///
/// # Safety
/// Pointers must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_f_count_string(
    haystack: *const u8,
    haystack_len: c_int,
    needle: *const u8,
    needle_len: c_int,
    ignore_case: c_int,
) -> i64 {
    if haystack.is_null() || needle.is_null() {
        return 0;
    }

    let haystack = std::slice::from_raw_parts(haystack, haystack_len.max(0) as usize);
    let needle = std::slice::from_raw_parts(needle, needle_len.max(0) as usize);

    count_substring(haystack, needle, ignore_case != 0)
}

// --- join() function helpers ---

/// Join list items into a string with separator.
///
/// This is a pure Rust implementation for joining strings.
/// The actual list iteration is done in C; this just handles the string joining.
pub fn join_strings(items: &[&[u8]], separator: &[u8]) -> Vec<u8> {
    if items.is_empty() {
        return Vec::new();
    }

    // Calculate total length
    let total_len: usize = items.iter().map(|s| s.len()).sum::<usize>()
        + separator.len() * items.len().saturating_sub(1);

    let mut result = Vec::with_capacity(total_len);

    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            result.extend_from_slice(separator);
        }
        result.extend_from_slice(item);
    }

    result
}

/// FFI: Join two strings with separator.
///
/// This is a building block - C code can call this repeatedly to join list items.
///
/// # Safety
/// All pointers must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_f_join_two(
    first: *const u8,
    first_len: c_int,
    second: *const u8,
    second_len: c_int,
    sep: *const u8,
    sep_len: c_int,
    out_len: *mut c_int,
) -> *mut u8 {
    let first_slice = if first.is_null() {
        &[]
    } else {
        std::slice::from_raw_parts(first, first_len.max(0) as usize)
    };

    let second_slice = if second.is_null() {
        &[]
    } else {
        std::slice::from_raw_parts(second, second_len.max(0) as usize)
    };

    let sep_slice = if sep.is_null() {
        &[]
    } else {
        std::slice::from_raw_parts(sep, sep_len.max(0) as usize)
    };

    let result = join_strings(&[first_slice, second_slice], sep_slice);

    if !out_len.is_null() {
        *out_len = result.len() as c_int;
    }

    // Allocate and copy the result
    // The caller is responsible for freeing this memory
    let ptr = libc::malloc(result.len() + 1).cast::<u8>();
    if ptr.is_null() {
        return std::ptr::null_mut();
    }

    std::ptr::copy_nonoverlapping(result.as_ptr(), ptr, result.len());
    *ptr.add(result.len()) = 0; // NUL terminator

    ptr
}

// --- split() function helpers ---

/// Split a string by separator into parts.
///
/// VimL split() behavior:
/// - If keepempty is false, empty strings are removed from result
/// - If separator is empty, splits on whitespace (runs of whitespace)
pub fn split_string(s: &[u8], sep: &[u8], keepempty: bool) -> Vec<Vec<u8>> {
    if s.is_empty() {
        return if keepempty {
            vec![Vec::new()]
        } else {
            Vec::new()
        };
    }

    if sep.is_empty() {
        // Split on whitespace - runs of whitespace are treated as single separator
        return split_on_whitespace(s, keepempty);
    }

    let mut result = Vec::new();
    let mut start = 0;

    loop {
        // Find next occurrence of separator
        let end = find_substring(&s[start..], sep).map_or(s.len(), |pos| start + pos);

        let part = &s[start..end];
        if keepempty || !part.is_empty() {
            result.push(part.to_vec());
        }

        if end >= s.len() {
            break;
        }

        start = end + sep.len();

        // Handle trailing separator
        if start >= s.len() {
            if keepempty {
                result.push(Vec::new());
            }
            break;
        }
    }

    result
}

/// Split string on whitespace (runs of whitespace are single separator).
fn split_on_whitespace(s: &[u8], keepempty: bool) -> Vec<Vec<u8>> {
    let mut result = Vec::new();
    let mut start = 0;
    let mut in_word = false;

    for (i, &c) in s.iter().enumerate() {
        if c.is_ascii_whitespace() {
            if in_word {
                let part = &s[start..i];
                if keepempty || !part.is_empty() {
                    result.push(part.to_vec());
                }
                in_word = false;
            }
        } else if !in_word {
            start = i;
            in_word = true;
        }
    }

    // Handle last word
    if in_word {
        let part = &s[start..];
        if keepempty || !part.is_empty() {
            result.push(part.to_vec());
        }
    }

    result
}

/// Find first occurrence of needle in haystack.
fn find_substring(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || needle.len() > haystack.len() {
        return None;
    }

    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

// --- flatten() helpers ---

/// Check if flatten depth is valid (0 to MAX_FLATTEN_DEPTH).
pub const fn validate_flatten_depth(depth: i64) -> bool {
    depth >= 0 && depth <= MAX_FLATTEN_DEPTH
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_list_index() {
        // Valid indices
        let r = validate_list_index(0, 5);
        assert!(r.valid);
        assert_eq!(r.index, 0);

        let r = validate_list_index(4, 5);
        assert!(r.valid);
        assert_eq!(r.index, 4);

        // Negative indices
        let r = validate_list_index(-1, 5);
        assert!(r.valid);
        assert_eq!(r.index, 4);

        let r = validate_list_index(-5, 5);
        assert!(r.valid);
        assert_eq!(r.index, 0);

        // Invalid indices
        let r = validate_list_index(5, 5);
        assert!(!r.valid);

        let r = validate_list_index(-6, 5);
        assert!(!r.valid);

        // Empty list
        let r = validate_list_index(0, 0);
        assert!(!r.valid);
    }

    #[test]
    fn test_validate_insert_index() {
        // Valid indices (0 to len inclusive)
        let r = validate_insert_index(0, 5);
        assert!(r.valid);
        assert_eq!(r.index, 0);

        let r = validate_insert_index(5, 5); // Append position
        assert!(r.valid);
        assert_eq!(r.index, 5);

        // Negative indices
        let r = validate_insert_index(-1, 5);
        assert!(r.valid);
        assert_eq!(r.index, 5);

        // Invalid
        let r = validate_insert_index(6, 5);
        assert!(!r.valid);
    }

    #[test]
    fn test_validate_list_range() {
        // Full range
        let r = validate_list_range(0, true, 4, true, 5);
        assert!(r.valid);
        assert_eq!(r.start, 0);
        assert_eq!(r.end, 5);

        // Partial range
        let r = validate_list_range(1, true, 3, true, 5);
        assert!(r.valid);
        assert_eq!(r.start, 1);
        assert_eq!(r.end, 4);

        // Negative indices
        let r = validate_list_range(-3, true, -1, true, 5);
        assert!(r.valid);
        assert_eq!(r.start, 2);
        assert_eq!(r.end, 5);

        // No start
        let r = validate_list_range(0, false, 2, true, 5);
        assert!(r.valid);
        assert_eq!(r.start, 0);
        assert_eq!(r.end, 3);

        // No end
        let r = validate_list_range(2, true, 0, false, 5);
        assert!(r.valid);
        assert_eq!(r.start, 2);
        assert_eq!(r.end, 5);
    }

    #[test]
    fn test_compare_strings() {
        assert_eq!(compare_strings(b"abc", b"abc"), 0);
        assert_eq!(compare_strings(b"abc", b"abd"), -1);
        assert_eq!(compare_strings(b"abd", b"abc"), 1);
        assert_eq!(compare_strings(b"ab", b"abc"), -1);
        assert_eq!(compare_strings(b"abc", b"ab"), 1);
    }

    #[test]
    fn test_compare_strings_ic() {
        assert_eq!(compare_strings_ic(b"ABC", b"abc"), 0);
        assert_eq!(compare_strings_ic(b"ABC", b"abd"), -1);
        assert_eq!(compare_strings_ic(b"Abd", b"ABC"), 1);
    }

    #[test]
    fn test_repeat_result_len() {
        assert_eq!(repeat_result_len(3, 4), 12);
        assert_eq!(repeat_result_len(0, 4), 0);
        assert_eq!(repeat_result_len(3, 0), 0);
        assert_eq!(repeat_result_len(3, -1), 0);
    }

    #[test]
    fn test_copy_type() {
        assert!(!CopyType::Shallow.is_deep());
        assert!(CopyType::Deep.is_deep());
        assert_eq!(CopyType::from_raw(0), CopyType::Shallow);
        assert_eq!(CopyType::from_raw(1), CopyType::Deep);
    }

    #[test]
    fn test_count_byte_in_slice() {
        assert_eq!(count_byte_in_slice(b"hello", b'l'), 2);
        assert_eq!(count_byte_in_slice(b"hello", b'o'), 1);
        assert_eq!(count_byte_in_slice(b"hello", b'x'), 0);
        assert_eq!(count_byte_in_slice(b"", b'x'), 0);
    }

    #[test]
    fn test_count_substring() {
        // Basic cases
        assert_eq!(count_substring(b"hello world", b"o", false), 2);
        assert_eq!(count_substring(b"hello world", b"l", false), 3);
        assert_eq!(count_substring(b"hello world", b"ll", false), 1);
        assert_eq!(count_substring(b"hello world", b"x", false), 0);

        // Case insensitive
        assert_eq!(count_substring(b"Hello World", b"o", true), 2);
        assert_eq!(count_substring(b"Hello World", b"O", true), 2);
        assert_eq!(count_substring(b"HELLO", b"ll", true), 1);

        // Edge cases
        assert_eq!(count_substring(b"", b"x", false), 0);
        assert_eq!(count_substring(b"hello", b"", false), 0);
        assert_eq!(count_substring(b"aaa", b"aa", false), 1); // Non-overlapping

        // Longer needle than haystack
        assert_eq!(count_substring(b"hi", b"hello", false), 0);
    }

    #[test]
    fn test_count_substring_non_overlapping() {
        // "ababa" contains "aba" once non-overlapping (positions 0-2)
        // If we count at position 0, next search starts at position 3
        assert_eq!(count_substring(b"ababa", b"aba", false), 1);

        // "aaaa" contains "aa" twice non-overlapping
        assert_eq!(count_substring(b"aaaa", b"aa", false), 2);
    }

    #[test]
    fn test_join_strings() {
        // Basic join
        let items: Vec<&[u8]> = vec![b"a", b"b", b"c"];
        assert_eq!(join_strings(&items, b","), b"a,b,c");

        // Empty separator
        let items: Vec<&[u8]> = vec![b"hello", b"world"];
        assert_eq!(join_strings(&items, b""), b"helloworld");

        // Single item
        let items: Vec<&[u8]> = vec![b"single"];
        assert_eq!(join_strings(&items, b","), b"single");

        // Empty items
        let items: Vec<&[u8]> = Vec::new();
        assert_eq!(join_strings(&items, b","), b"");

        // Multi-char separator
        let items: Vec<&[u8]> = vec![b"a", b"b"];
        assert_eq!(join_strings(&items, b" - "), b"a - b");
    }

    #[test]
    fn test_split_string() {
        // Basic split
        let result = split_string(b"a,b,c", b",", false);
        assert_eq!(result, vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec()]);

        // Keep empty parts
        let result = split_string(b"a,,c", b",", true);
        assert_eq!(result, vec![b"a".to_vec(), b"".to_vec(), b"c".to_vec()]);

        // Don't keep empty parts
        let result = split_string(b"a,,c", b",", false);
        assert_eq!(result, vec![b"a".to_vec(), b"c".to_vec()]);

        // Trailing separator with keepempty
        let result = split_string(b"a,b,", b",", true);
        assert_eq!(result, vec![b"a".to_vec(), b"b".to_vec(), b"".to_vec()]);

        // Empty string
        let result = split_string(b"", b",", false);
        assert!(result.is_empty());
    }

    #[test]
    fn test_split_on_whitespace() {
        // Basic whitespace split
        let result = split_string(b"hello world", b"", false);
        assert_eq!(result, vec![b"hello".to_vec(), b"world".to_vec()]);

        // Multiple whitespace chars
        let result = split_string(b"hello   world", b"", false);
        assert_eq!(result, vec![b"hello".to_vec(), b"world".to_vec()]);

        // Tabs and spaces
        let result = split_string(b"a\tb\tc", b"", false);
        assert_eq!(result, vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec()]);
    }

    #[test]
    fn test_find_substring() {
        assert_eq!(find_substring(b"hello world", b"world"), Some(6));
        assert_eq!(find_substring(b"hello world", b"hello"), Some(0));
        assert_eq!(find_substring(b"hello world", b"xyz"), None);
        assert_eq!(find_substring(b"hello", b""), None);
        assert_eq!(find_substring(b"hi", b"hello"), None);
    }

    #[test]
    fn test_validate_flatten_depth() {
        assert!(validate_flatten_depth(0));
        assert!(validate_flatten_depth(1));
        assert!(validate_flatten_depth(999));
        assert!(!validate_flatten_depth(-1));
        assert!(!validate_flatten_depth(1000));
    }
}
