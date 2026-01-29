//! Match deletion logic
//!
//! This module provides functions for deleting matches from the match list:
//! - ID validation for deletion
//! - Finding matches by ID
//! - Locating previous item for list unlinking

use std::ffi::c_int;
use std::ptr;

use crate::id::is_valid_id;
use crate::MIN_MATCH_ID;

// =============================================================================
// Opaque Handle Types
// =============================================================================

/// Opaque handle to a C `win_T` structure.
#[repr(C)]
pub struct WinHandle {
    _opaque: [u8; 0],
}

/// Opaque handle to a C `matchitem_T` structure.
#[repr(C)]
pub struct MatchItemHandle {
    _opaque: [u8; 0],
}

// =============================================================================
// FFI Declarations
// =============================================================================

extern "C" {
    fn nvim_match_get_head(wp: *mut WinHandle) -> *mut MatchItemHandle;
    fn nvim_match_item_next(m: *mut MatchItemHandle) -> *mut MatchItemHandle;
    fn nvim_match_item_get_id(m: *mut MatchItemHandle) -> c_int;
}

// =============================================================================
// Error Types
// =============================================================================

/// Errors that can occur when deleting a match.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum DeleteError {
    /// Success (no error)
    Ok = 0,
    /// Invalid ID (must be >= 1)
    InvalidId = -1,
    /// Match with given ID not found
    NotFound = -2,
}

impl DeleteError {
    /// Convert to C integer.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if this is a success.
    #[must_use]
    pub const fn is_ok(self) -> bool {
        matches!(self, Self::Ok)
    }
}

// =============================================================================
// Validation
// =============================================================================

/// Validate a match ID for deletion.
///
/// IDs must be >= 1 for deletion.
#[must_use]
pub const fn validate_delete_id(id: i32) -> DeleteError {
    if id < MIN_MATCH_ID {
        DeleteError::InvalidId
    } else {
        DeleteError::Ok
    }
}

// =============================================================================
// Match Finding
// =============================================================================

/// Result of finding a match for deletion.
#[derive(Debug, Clone, Copy)]
pub struct FindResult {
    /// The found match item (NULL if not found).
    pub item: *mut MatchItemHandle,
    /// The previous item in the list (NULL if item is at head or not found).
    pub prev: *mut MatchItemHandle,
    /// Whether the item is at the head of the list.
    pub at_head: bool,
}

impl FindResult {
    /// Create a not found result.
    #[must_use]
    pub const fn not_found() -> Self {
        Self {
            item: ptr::null_mut(),
            prev: ptr::null_mut(),
            at_head: false,
        }
    }

    /// Create a found at head result.
    #[must_use]
    pub const fn found_at_head(item: *mut MatchItemHandle) -> Self {
        Self {
            item,
            prev: ptr::null_mut(),
            at_head: true,
        }
    }

    /// Create a found result with previous item.
    #[must_use]
    pub const fn found_after(item: *mut MatchItemHandle, prev: *mut MatchItemHandle) -> Self {
        Self {
            item,
            prev,
            at_head: false,
        }
    }

    /// Check if a match was found.
    #[must_use]
    pub fn is_found(&self) -> bool {
        !self.item.is_null()
    }
}

/// Find a match by ID in the window's match list.
///
/// Returns the match item and the previous item (for unlinking).
///
/// # Safety
///
/// `wp` must be a valid pointer to a `win_T`.
#[must_use]
pub unsafe fn find_by_id(wp: *mut WinHandle, id: i32) -> FindResult {
    if wp.is_null() || !is_valid_id(id) {
        return FindResult::not_found();
    }

    let head = nvim_match_get_head(wp);
    if head.is_null() {
        return FindResult::not_found();
    }

    // Check head
    if nvim_match_item_get_id(head) == id {
        return FindResult::found_at_head(head);
    }

    // Search the rest of the list
    let mut prev = head;
    let mut cur = nvim_match_item_next(head);

    while !cur.is_null() {
        if nvim_match_item_get_id(cur) == id {
            return FindResult::found_after(cur, prev);
        }
        prev = cur;
        cur = nvim_match_item_next(cur);
    }

    FindResult::not_found()
}

/// Validate a delete operation.
///
/// # Safety
///
/// `wp` must be a valid pointer to a `win_T`.
#[must_use]
pub unsafe fn validate_delete(wp: *mut WinHandle, id: i32) -> DeleteError {
    // Validate ID
    let id_error = validate_delete_id(id);
    if !id_error.is_ok() {
        return id_error;
    }

    // Check if match exists
    let result = find_by_id(wp, id);
    if !result.is_found() {
        return DeleteError::NotFound;
    }

    DeleteError::Ok
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Validate match ID for deletion.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_validate_delete_id(id: c_int) -> c_int {
    validate_delete_id(id).to_c_int()
}

/// Find a match by ID.
///
/// Returns the match item pointer, or NULL if not found.
///
/// # Safety
///
/// `wp` must be a valid pointer to a `win_T`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_match_find_by_id(
    wp: *mut WinHandle,
    id: c_int,
) -> *mut MatchItemHandle {
    find_by_id(wp, id).item
}

/// Find a match by ID and return the previous item.
///
/// This is useful for unlinking from the list.
///
/// # Safety
///
/// `wp` must be a valid pointer to a `win_T`.
/// `out_prev` must be a valid pointer if not null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_match_find_for_delete(
    wp: *mut WinHandle,
    id: c_int,
    out_prev: *mut *mut MatchItemHandle,
    out_at_head: *mut c_int,
) -> *mut MatchItemHandle {
    let result = find_by_id(wp, id);

    if !out_prev.is_null() {
        *out_prev = result.prev;
    }
    if !out_at_head.is_null() {
        *out_at_head = c_int::from(result.at_head);
    }

    result.item
}

/// Validate a delete operation.
///
/// Returns 0 on success, negative error code on failure.
///
/// # Safety
///
/// `wp` must be a valid pointer to a `win_T`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_match_validate_delete(wp: *mut WinHandle, id: c_int) -> c_int {
    validate_delete(wp, id).to_c_int()
}

/// Get the error code for invalid ID.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_delete_error_invalid_id() -> c_int {
    DeleteError::InvalidId.to_c_int()
}

/// Get the error code for not found.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_delete_error_not_found() -> c_int {
    DeleteError::NotFound.to_c_int()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_delete_id() {
        assert_eq!(validate_delete_id(1), DeleteError::Ok);
        assert_eq!(validate_delete_id(100), DeleteError::Ok);
        assert_eq!(validate_delete_id(0), DeleteError::InvalidId);
        assert_eq!(validate_delete_id(-1), DeleteError::InvalidId);
    }

    #[test]
    fn test_find_result() {
        let not_found = FindResult::not_found();
        assert!(!not_found.is_found());

        // Can't test found results without actual pointers
    }

    #[test]
    fn test_delete_error_codes() {
        assert_eq!(DeleteError::Ok.to_c_int(), 0);
        assert!(DeleteError::InvalidId.to_c_int() < 0);
        assert!(DeleteError::NotFound.to_c_int() < 0);
    }
}
