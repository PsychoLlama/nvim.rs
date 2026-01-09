//! Tag match collection and prioritization for Neovim C-to-Rust migration
//!
//! This module provides Rust implementations for collecting tag matches,
//! managing match storage, and prioritizing matches for display.

#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::ptr_as_ptr)]

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

// =============================================================================
// Re-export match type constants from lib.rs
// =============================================================================

use crate::match_type;

// =============================================================================
// Opaque handle types
// =============================================================================

/// Opaque handle to hashtab_T
type HashtabHandle = *mut c_void;

// =============================================================================
// External C functions
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Hashtab functions
    fn hash_find(ht: HashtabHandle, key: *const c_char) -> *mut c_void;

    // String functions
    fn strlen(s: *const c_char) -> usize;
}

// =============================================================================
// Match storage structure
// =============================================================================

/// Number of match type buckets
pub const MT_COUNT: usize = 16;

/// Structure to manage match collection across all priority buckets.
///
/// This provides a Rust interface to the match storage arrays that exist
/// in `findtags_state_T`.
#[repr(C)]
pub struct MatchStorage {
    /// Total number of matches across all buckets
    pub total_count: c_int,
    /// Whether matches have been collected
    pub has_matches: bool,
    /// Best (lowest) match type seen
    pub best_match_type: c_int,
}

impl Default for MatchStorage {
    fn default() -> Self {
        Self {
            total_count: 0,
            has_matches: false,
            best_match_type: match_type::MT_COUNT,
        }
    }
}

// =============================================================================
// Match priority functions
// =============================================================================

/// Determine the match type bucket for a tag match.
///
/// The match type determines priority and is based on:
/// - Whether the match is static (file-local) or global
/// - Whether the match is in the current file or other files
/// - Whether the match is case-sensitive or case-insensitive
/// - Whether the match was via regexp
///
/// # Arguments
///
/// * `is_static` - True if the tag is file-local (static)
/// * `is_current_file` - True if tag is in the current buffer's file
/// * `is_icase` - True if match was case-insensitive
/// * `is_regexp` - True if match was via regexp
#[no_mangle]
pub extern "C" fn rs_get_match_type(
    is_static: bool,
    is_current_file: bool,
    is_icase: bool,
    is_regexp: bool,
) -> c_int {
    let mut mt: c_int = if is_current_file {
        if is_static {
            match_type::MT_ST_CUR
        } else {
            match_type::MT_GL_CUR
        }
    } else if is_static {
        match_type::MT_ST_OTH
    } else {
        match_type::MT_GL_OTH
    };

    if is_icase {
        mt += match_type::MT_IC_OFF;
    }
    if is_regexp {
        mt += match_type::MT_RE_OFF;
    }

    mt
}

/// Get the display priority for a match type.
///
/// Lower values = higher priority for display.
/// Priority is based on the masked match type (ignoring regexp flag).
#[no_mangle]
pub extern "C" fn rs_match_display_priority(mt: c_int) -> c_int {
    mt & match_type::MT_MASK
}

/// Compare two match types for sorting.
///
/// Returns negative if mt1 should come before mt2 (higher priority).
#[no_mangle]
pub extern "C" fn rs_compare_match_types(mt1: c_int, mt2: c_int) -> c_int {
    let p1 = rs_match_display_priority(mt1);
    let p2 = rs_match_display_priority(mt2);
    p1 - p2
}

// =============================================================================
// Match storage management
// =============================================================================

/// Create a new MatchStorage structure.
#[no_mangle]
pub extern "C" fn rs_match_storage_new() -> *mut MatchStorage {
    Box::into_raw(Box::new(MatchStorage::default()))
}

/// Free a MatchStorage structure.
#[no_mangle]
pub unsafe extern "C" fn rs_match_storage_free(storage: *mut MatchStorage) {
    if !storage.is_null() {
        drop(Box::from_raw(storage));
    }
}

/// Initialize match storage to empty state.
#[no_mangle]
pub unsafe extern "C" fn rs_match_storage_init(storage: *mut MatchStorage) {
    if storage.is_null() {
        return;
    }
    *storage = MatchStorage::default();
}

/// Record a match in the storage.
#[no_mangle]
pub unsafe extern "C" fn rs_match_storage_add(storage: *mut MatchStorage, match_type: c_int) {
    if storage.is_null() {
        return;
    }
    let storage = &mut *storage;
    storage.total_count += 1;
    storage.has_matches = true;
    if match_type < storage.best_match_type {
        storage.best_match_type = match_type;
    }
}

/// Get the total match count.
#[no_mangle]
pub unsafe extern "C" fn rs_match_storage_count(storage: *const MatchStorage) -> c_int {
    if storage.is_null() {
        return 0;
    }
    (*storage).total_count
}

/// Check if any matches have been collected.
#[no_mangle]
pub unsafe extern "C" fn rs_match_storage_has_matches(storage: *const MatchStorage) -> bool {
    if storage.is_null() {
        return false;
    }
    (*storage).has_matches
}

/// Get the best (highest priority, lowest value) match type.
#[no_mangle]
pub unsafe extern "C" fn rs_match_storage_best_type(storage: *const MatchStorage) -> c_int {
    if storage.is_null() {
        return match_type::MT_COUNT;
    }
    (*storage).best_match_type
}

// =============================================================================
// Match deduplication helpers
// =============================================================================

/// Generate a deduplication key for a tag match.
///
/// The key includes tag name, filename, and command to identify unique matches.
/// Returns the length of the key written to the buffer, or 0 on error.
///
/// # Safety
///
/// - All pointers must be valid or null
/// - `buf` must have at least `buf_size` bytes available
#[no_mangle]
pub unsafe extern "C" fn rs_make_match_key(
    buf: *mut c_char,
    buf_size: usize,
    tagname: *const c_char,
    fname: *const c_char,
    command: *const c_char,
) -> usize {
    if buf.is_null() || buf_size == 0 {
        return 0;
    }

    let mut pos = 0usize;

    // Write tagname
    if !tagname.is_null() {
        let len = strlen(tagname);
        let copy_len = len.min(buf_size - pos - 1);
        ptr::copy_nonoverlapping(tagname, buf.add(pos), copy_len);
        pos += copy_len;
    }

    // Separator
    if pos < buf_size - 1 {
        *buf.add(pos) = 0x01; // Use SOH as separator
        pos += 1;
    }

    // Write fname
    if !fname.is_null() && pos < buf_size - 1 {
        let len = strlen(fname);
        let copy_len = len.min(buf_size - pos - 1);
        ptr::copy_nonoverlapping(fname, buf.add(pos), copy_len);
        pos += copy_len;
    }

    // Separator
    if pos < buf_size - 1 {
        *buf.add(pos) = 0x01;
        pos += 1;
    }

    // Write command (first part only for efficiency)
    if !command.is_null() && pos < buf_size - 1 {
        let len = strlen(command).min(50); // Limit command length
        let copy_len = len.min(buf_size - pos - 1);
        ptr::copy_nonoverlapping(command, buf.add(pos), copy_len);
        pos += copy_len;
    }

    // Null terminate
    *buf.add(pos) = 0;

    pos
}

/// Check if a key already exists in the match set.
///
/// This is used for deduplication.
#[no_mangle]
pub unsafe extern "C" fn rs_match_exists(ht: HashtabHandle, key: *const c_char) -> bool {
    if ht.is_null() || key.is_null() {
        return false;
    }
    !hash_find(ht, key).is_null()
}

// =============================================================================
// Match formatting for display
// =============================================================================

/// Format match information for tselect display.
///
/// Returns a string with format: " [idx] kind pri file"
/// or similar depending on display settings.
#[no_mangle]
pub unsafe extern "C" fn rs_format_match_info(
    buf: *mut c_char,
    buf_size: usize,
    idx: c_int,
    match_type: c_int,
    kind: *const c_char,
    fname: *const c_char,
) -> usize {
    if buf.is_null() || buf_size == 0 {
        return 0;
    }

    let mut pos = 0usize;

    // Write index
    let idx_str = format!("{:>3} ", idx + 1);
    let idx_bytes = idx_str.as_bytes();
    let copy_len = idx_bytes.len().min(buf_size - pos - 1);
    for (i, &byte) in idx_bytes.iter().take(copy_len).enumerate() {
        *buf.add(pos + i) = byte as c_char;
    }
    pos += copy_len;

    // Write kind character if available
    if !kind.is_null() && pos < buf_size - 2 {
        let kind_char = *kind;
        if kind_char != 0 {
            *buf.add(pos) = kind_char;
            pos += 1;
            *buf.add(pos) = b' ' as c_char;
            pos += 1;
        }
    }

    // Write match type indicator
    let mt_name = crate::rs_tag_match_type_name(match_type);
    if !mt_name.is_null() && pos < buf_size - 4 {
        for i in 0..3 {
            let c = *mt_name.add(i);
            if c == 0 {
                break;
            }
            *buf.add(pos) = c;
            pos += 1;
        }
        *buf.add(pos) = b' ' as c_char;
        pos += 1;
    }

    // Write filename (truncated if needed)
    if !fname.is_null() && pos < buf_size - 1 {
        let fname_len = strlen(fname);
        let max_fname = buf_size - pos - 1;
        let copy_len = fname_len.min(max_fname);
        ptr::copy_nonoverlapping(fname, buf.add(pos), copy_len);
        pos += copy_len;
    }

    // Null terminate
    *buf.add(pos) = 0;

    pos
}

// =============================================================================
// Match sorting helpers
// =============================================================================

/// Comparison function signature for qsort-compatible sorting.
pub type MatchCompareFn = unsafe extern "C" fn(*const c_void, *const c_void) -> c_int;

/// Compare two match entries by priority.
///
/// This extracts the match type from the stored format and compares priorities.
#[no_mangle]
pub unsafe extern "C" fn rs_compare_matches(a: *const c_void, b: *const c_void) -> c_int {
    if a.is_null() || b.is_null() {
        return 0;
    }

    // Match entries are stored as char* pointers
    // First byte is the match type
    let ma = *(a as *const *const c_char);
    let mb = *(b as *const *const c_char);

    if ma.is_null() || mb.is_null() {
        return 0;
    }

    let mt_a = *ma as c_int;
    let mt_b = *mb as c_int;

    rs_compare_match_types(mt_a, mt_b)
}

// =============================================================================
// Match iteration helpers
// =============================================================================

/// Calculate the bucket index for iterating matches in priority order.
///
/// Given a linear index, returns the bucket (match type) that should be
/// accessed at that position.
#[no_mangle]
pub unsafe extern "C" fn rs_match_bucket_for_index(
    bucket_counts: *const c_int,
    num_buckets: c_int,
    index: c_int,
) -> c_int {
    if bucket_counts.is_null() || num_buckets <= 0 || index < 0 {
        return -1;
    }

    let mut remaining = index;
    for bucket in 0..num_buckets {
        let count = *bucket_counts.add(bucket as usize);
        if remaining < count {
            return bucket;
        }
        remaining -= count;
    }

    -1 // Index out of range
}

/// Calculate the offset within a bucket for a given linear index.
#[no_mangle]
pub unsafe extern "C" fn rs_match_offset_in_bucket(
    bucket_counts: *const c_int,
    num_buckets: c_int,
    index: c_int,
) -> c_int {
    if bucket_counts.is_null() || num_buckets <= 0 || index < 0 {
        return -1;
    }

    let mut remaining = index;
    for bucket in 0..num_buckets {
        let count = *bucket_counts.add(bucket as usize);
        if remaining < count {
            return remaining;
        }
        remaining -= count;
    }

    -1 // Index out of range
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_match_type() {
        // Static in current file
        assert_eq!(
            rs_get_match_type(true, true, false, false),
            match_type::MT_ST_CUR
        );

        // Global in current file
        assert_eq!(
            rs_get_match_type(false, true, false, false),
            match_type::MT_GL_CUR
        );

        // Global in other file
        assert_eq!(
            rs_get_match_type(false, false, false, false),
            match_type::MT_GL_OTH
        );

        // Static in other file
        assert_eq!(
            rs_get_match_type(true, false, false, false),
            match_type::MT_ST_OTH
        );

        // With icase
        assert_eq!(
            rs_get_match_type(true, true, true, false),
            match_type::MT_ST_CUR + match_type::MT_IC_OFF
        );

        // With regexp
        assert_eq!(
            rs_get_match_type(true, true, false, true),
            match_type::MT_ST_CUR + match_type::MT_RE_OFF
        );

        // With both
        assert_eq!(
            rs_get_match_type(true, true, true, true),
            match_type::MT_ST_CUR + match_type::MT_IC_OFF + match_type::MT_RE_OFF
        );
    }

    #[test]
    fn test_match_display_priority() {
        assert_eq!(
            rs_match_display_priority(match_type::MT_ST_CUR),
            match_type::MT_ST_CUR
        );
        assert_eq!(
            rs_match_display_priority(match_type::MT_ST_CUR + match_type::MT_IC_OFF),
            match_type::MT_ST_CUR
        );
        assert_eq!(
            rs_match_display_priority(match_type::MT_GL_OTH + match_type::MT_RE_OFF),
            match_type::MT_GL_OTH
        );
    }

    #[test]
    fn test_compare_match_types() {
        // ST_CUR < GL_CUR
        assert!(rs_compare_match_types(match_type::MT_ST_CUR, match_type::MT_GL_CUR) < 0);

        // GL_OTH > GL_CUR
        assert!(rs_compare_match_types(match_type::MT_GL_OTH, match_type::MT_GL_CUR) > 0);

        // Same type
        assert_eq!(
            rs_compare_match_types(match_type::MT_ST_CUR, match_type::MT_ST_CUR),
            0
        );

        // IC flag doesn't affect comparison (same base type)
        assert_eq!(
            rs_compare_match_types(
                match_type::MT_ST_CUR,
                match_type::MT_ST_CUR + match_type::MT_IC_OFF
            ),
            0
        );
    }

    #[test]
    fn test_match_storage() {
        unsafe {
            let storage = rs_match_storage_new();
            assert!(!storage.is_null());

            assert_eq!(rs_match_storage_count(storage), 0);
            assert!(!rs_match_storage_has_matches(storage));
            assert_eq!(rs_match_storage_best_type(storage), match_type::MT_COUNT);

            rs_match_storage_add(storage, match_type::MT_GL_OTH);
            assert_eq!(rs_match_storage_count(storage), 1);
            assert!(rs_match_storage_has_matches(storage));
            assert_eq!(rs_match_storage_best_type(storage), match_type::MT_GL_OTH);

            rs_match_storage_add(storage, match_type::MT_ST_CUR);
            assert_eq!(rs_match_storage_count(storage), 2);
            assert_eq!(rs_match_storage_best_type(storage), match_type::MT_ST_CUR);

            rs_match_storage_free(storage);
        }
    }

    #[test]
    fn test_match_bucket_for_index() {
        let counts: [c_int; 4] = [2, 3, 0, 1]; // Total: 6 matches

        // First bucket (indices 0, 1)
        assert_eq!(
            unsafe { rs_match_bucket_for_index(counts.as_ptr(), 4, 0) },
            0
        );
        assert_eq!(
            unsafe { rs_match_bucket_for_index(counts.as_ptr(), 4, 1) },
            0
        );

        // Second bucket (indices 2, 3, 4)
        assert_eq!(
            unsafe { rs_match_bucket_for_index(counts.as_ptr(), 4, 2) },
            1
        );
        assert_eq!(
            unsafe { rs_match_bucket_for_index(counts.as_ptr(), 4, 4) },
            1
        );

        // Third bucket is empty, skip to fourth (index 5)
        assert_eq!(
            unsafe { rs_match_bucket_for_index(counts.as_ptr(), 4, 5) },
            3
        );

        // Out of range
        assert_eq!(
            unsafe { rs_match_bucket_for_index(counts.as_ptr(), 4, 6) },
            -1
        );
    }

    #[test]
    fn test_match_offset_in_bucket() {
        let counts: [c_int; 4] = [2, 3, 0, 1];

        // Offsets in first bucket
        assert_eq!(
            unsafe { rs_match_offset_in_bucket(counts.as_ptr(), 4, 0) },
            0
        );
        assert_eq!(
            unsafe { rs_match_offset_in_bucket(counts.as_ptr(), 4, 1) },
            1
        );

        // Offsets in second bucket
        assert_eq!(
            unsafe { rs_match_offset_in_bucket(counts.as_ptr(), 4, 2) },
            0
        );
        assert_eq!(
            unsafe { rs_match_offset_in_bucket(counts.as_ptr(), 4, 3) },
            1
        );
        assert_eq!(
            unsafe { rs_match_offset_in_bucket(counts.as_ptr(), 4, 4) },
            2
        );

        // Offset in fourth bucket
        assert_eq!(
            unsafe { rs_match_offset_in_bucket(counts.as_ptr(), 4, 5) },
            0
        );
    }

    #[test]
    fn test_null_safety() {
        unsafe {
            assert_eq!(rs_match_storage_count(ptr::null()), 0);
            assert!(!rs_match_storage_has_matches(ptr::null()));
            assert_eq!(rs_match_bucket_for_index(ptr::null(), 4, 0), -1);
            assert_eq!(rs_match_offset_in_bucket(ptr::null(), 4, 0), -1);
            assert!(!rs_match_exists(ptr::null_mut(), ptr::null()));
        }
    }
}
