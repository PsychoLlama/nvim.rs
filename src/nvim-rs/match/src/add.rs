//! Match addition logic
//!
//! This module provides functions for adding matches to the match list:
//! - Input validation (group, pattern, ID, priority)
//! - ID conflict detection
//! - Next ID calculation
//! - Insertion point finding (priority-based ordering)

use std::ffi::{c_char, c_int, CStr};
use std::ptr;

use crate::id::{calc_next_id, is_valid_id, is_valid_id_request};
use crate::item::is_valid_match_args;
use crate::{MATCH_ID_1, MATCH_ID_2, MATCH_ID_3, MIN_MATCH_ID};

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
    fn nvim_match_get_next_id(wp: *mut WinHandle) -> c_int;
    fn nvim_match_item_next(m: *mut MatchItemHandle) -> *mut MatchItemHandle;
    fn nvim_match_item_get_id(m: *mut MatchItemHandle) -> c_int;
    fn nvim_match_item_get_priority(m: *mut MatchItemHandle) -> c_int;
}

// =============================================================================
// Error Types
// =============================================================================

/// Errors that can occur when adding a match.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum AddError {
    /// Success (no error)
    Ok = 0,
    /// Empty group name
    EmptyGroup = -1,
    /// Empty pattern (when pattern is required)
    EmptyPattern = -2,
    /// Invalid ID (not -1 and not >= 1)
    InvalidId = -3,
    /// ID already taken
    IdTaken = -4,
    /// ID reserved for :match command
    IdReserved = -5,
}

impl AddError {
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
// Validation Functions
// =============================================================================

/// Check if a match ID is reserved for the :match commands (1, 2, 3).
#[must_use]
pub const fn is_reserved_id(id: i32) -> bool {
    id == MATCH_ID_1 || id == MATCH_ID_2 || id == MATCH_ID_3
}

/// Check if a match ID is valid for `matchadd()`.
///
/// IDs 1, 2, 3 are reserved for `:match`, `:2match`, `:3match`.
/// Valid IDs are -1 (auto) or >= 4.
#[must_use]
pub const fn is_valid_matchadd_id(id: i32) -> bool {
    id == -1 || id >= 4
}

/// Check if a match ID is valid for `matchaddpos()`.
///
/// ID 3 is allowed (substitutes :3match), but 1 and 2 are reserved.
#[must_use]
pub const fn is_valid_matchaddpos_id(id: i32) -> bool {
    id == -1 || id == MATCH_ID_3 || id >= 4
}

/// Validate match addition inputs (group and pattern only).
///
/// Returns `Ok` if valid, otherwise the specific error.
/// ID validation is handled separately.
#[must_use]
pub fn validate_add_inputs(group: &str, pattern: Option<&str>) -> AddError {
    // Check group and pattern validity
    if !is_valid_match_args(group, pattern) {
        if group.is_empty() {
            return AddError::EmptyGroup;
        }
        return AddError::EmptyPattern;
    }

    AddError::Ok
}

// =============================================================================
// ID Conflict Detection
// =============================================================================

/// Check if a match ID already exists in the window's match list.
///
/// # Safety
///
/// `wp` must be a valid pointer to a `win_T`.
#[must_use]
pub unsafe fn id_exists(wp: *mut WinHandle, id: i32) -> bool {
    if wp.is_null() || id < MIN_MATCH_ID {
        return false;
    }

    let mut cur = nvim_match_get_head(wp);
    while !cur.is_null() {
        if nvim_match_item_get_id(cur) == id {
            return true;
        }
        cur = nvim_match_item_next(cur);
    }

    false
}

/// Find the next available match ID.
///
/// # Safety
///
/// `wp` must be a valid pointer to a `win_T`.
#[must_use]
pub unsafe fn get_next_available_id(wp: *mut WinHandle) -> i32 {
    if wp.is_null() {
        return MIN_MATCH_ID;
    }
    nvim_match_get_next_id(wp)
}

/// Calculate what the next ID should be after using a manual ID.
///
/// Ensures future auto IDs won't conflict with manual IDs.
#[must_use]
pub const fn calc_next_id_after_manual(current_next: i32, manual_id: i32) -> i32 {
    calc_next_id(current_next, manual_id)
}

// =============================================================================
// Insertion Point Finding
// =============================================================================

/// Result of finding an insertion point.
#[derive(Debug, Clone, Copy)]
pub struct InsertionPoint {
    /// The match item to insert after (NULL means insert at head).
    pub insert_after: *mut MatchItemHandle,
    /// Whether to insert at the head of the list.
    pub at_head: bool,
}

impl InsertionPoint {
    /// Create an insertion point at the head of the list.
    #[must_use]
    pub const fn at_head() -> Self {
        Self {
            insert_after: ptr::null_mut(),
            at_head: true,
        }
    }

    /// Create an insertion point after the given match item.
    #[must_use]
    pub const fn after(item: *mut MatchItemHandle) -> Self {
        Self {
            insert_after: item,
            at_head: false,
        }
    }
}

/// Find the insertion point for a new match based on priority.
///
/// Matches are kept in ascending priority order.
///
/// # Safety
///
/// `wp` must be a valid pointer to a `win_T`.
#[must_use]
pub unsafe fn find_insertion_point(wp: *mut WinHandle, priority: i32) -> InsertionPoint {
    if wp.is_null() {
        return InsertionPoint::at_head();
    }

    let head = nvim_match_get_head(wp);
    if head.is_null() {
        return InsertionPoint::at_head();
    }

    // Check if we should insert at head
    if priority < nvim_match_item_get_priority(head) {
        return InsertionPoint::at_head();
    }

    // Find the right position
    let mut cur = head;
    let mut prev = head;
    while !cur.is_null() && priority >= nvim_match_item_get_priority(cur) {
        prev = cur;
        cur = nvim_match_item_next(cur);
    }

    InsertionPoint::after(prev)
}

// =============================================================================
// Comprehensive Add Validation
// =============================================================================

/// Result of match add validation.
#[derive(Debug, Clone, Copy)]
pub struct AddValidation {
    /// Error code (0 = success)
    pub error: AddError,
    /// The ID to use (resolved from auto or manual)
    pub resolved_id: i32,
    /// The new `next_id` value for the window
    pub new_next_id: i32,
}

/// Validate all inputs for adding a match.
///
/// # Safety
///
/// `wp` must be a valid pointer to a `win_T`.
/// `group` and `pattern` must be valid C strings or null.
pub unsafe fn validate_add(
    wp: *mut WinHandle,
    group: *const c_char,
    pattern: *const c_char,
    id: c_int,
    for_matchadd: bool,
) -> AddValidation {
    // Convert C strings
    let group_str = if group.is_null() {
        ""
    } else {
        CStr::from_ptr(group).to_str().unwrap_or("")
    };

    let pattern_opt = if pattern.is_null() {
        None
    } else {
        CStr::from_ptr(pattern).to_str().ok()
    };

    // Basic validation
    let error = validate_add_inputs(group_str, pattern_opt);
    if !error.is_ok() {
        return AddValidation {
            error,
            resolved_id: -1,
            new_next_id: 0,
        };
    }

    // ID validation
    if !is_valid_id_request(id) {
        return AddValidation {
            error: AddError::InvalidId,
            resolved_id: -1,
            new_next_id: 0,
        };
    }

    // Check reserved IDs for matchadd()
    if for_matchadd && is_valid_id(id) && is_reserved_id(id) {
        return AddValidation {
            error: AddError::IdReserved,
            resolved_id: -1,
            new_next_id: 0,
        };
    }

    let current_next_id = nvim_match_get_next_id(wp);

    // Resolve ID
    let (resolved_id, new_next_id) = if id == -1 {
        // Auto-assign
        (current_next_id, current_next_id + 1)
    } else {
        // Manual ID - check for conflicts
        if id_exists(wp, id) {
            return AddValidation {
                error: AddError::IdTaken,
                resolved_id: -1,
                new_next_id: 0,
            };
        }
        // Update next_id to avoid future conflicts
        let new_next = calc_next_id_after_manual(current_next_id, id);
        (id, new_next)
    };

    AddValidation {
        error: AddError::Ok,
        resolved_id,
        new_next_id,
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Check if a match ID is reserved for :match commands.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_is_reserved_id(id: c_int) -> c_int {
    c_int::from(is_reserved_id(id))
}

/// Check if ID is valid for `matchadd()`.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_is_valid_matchadd_id(id: c_int) -> c_int {
    c_int::from(is_valid_matchadd_id(id))
}

/// Check if ID is valid for `matchaddpos()`.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_is_valid_matchaddpos_id(id: c_int) -> c_int {
    c_int::from(is_valid_matchaddpos_id(id))
}

/// Check if match ID already exists in window.
///
/// # Safety
///
/// `wp` must be a valid pointer to a `win_T`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_match_id_exists(wp: *mut WinHandle, id: c_int) -> c_int {
    c_int::from(id_exists(wp, id))
}

/// Find insertion point for a new match.
///
/// Returns pointer to the match item to insert after, or NULL if inserting at head.
///
/// # Safety
///
/// `wp` must be a valid pointer to a `win_T`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_match_find_insert_point(
    wp: *mut WinHandle,
    priority: c_int,
) -> *mut MatchItemHandle {
    let point = find_insertion_point(wp, priority);
    point.insert_after
}

/// Check if we should insert at the head of the match list.
///
/// # Safety
///
/// `wp` must be a valid pointer to a `win_T`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_match_should_insert_at_head(
    wp: *mut WinHandle,
    priority: c_int,
) -> c_int {
    let point = find_insertion_point(wp, priority);
    c_int::from(point.at_head)
}

/// Validate match add operation.
///
/// Returns error code: 0 = success, negative = error.
/// On success, `out_id` is set to the resolved ID and `out_next_id` to the new `next_id`.
///
/// # Safety
///
/// All pointers must be valid or null as documented.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_match_validate_add(
    wp: *mut WinHandle,
    group: *const c_char,
    pattern: *const c_char,
    id: c_int,
    for_matchadd: c_int,
    out_id: *mut c_int,
    out_next_id: *mut c_int,
) -> c_int {
    let result = validate_add(wp, group, pattern, id, for_matchadd != 0);

    if !out_id.is_null() {
        *out_id = result.resolved_id;
    }
    if !out_next_id.is_null() {
        *out_next_id = result.new_next_id;
    }

    result.error.to_c_int()
}

/// Get the error code for empty group.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_add_error_empty_group() -> c_int {
    AddError::EmptyGroup.to_c_int()
}

/// Get the error code for empty pattern.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_add_error_empty_pattern() -> c_int {
    AddError::EmptyPattern.to_c_int()
}

/// Get the error code for invalid ID.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_add_error_invalid_id() -> c_int {
    AddError::InvalidId.to_c_int()
}

/// Get the error code for ID taken.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_add_error_id_taken() -> c_int {
    AddError::IdTaken.to_c_int()
}

/// Get the error code for reserved ID.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_add_error_id_reserved() -> c_int {
    AddError::IdReserved.to_c_int()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_reserved_id() {
        assert!(is_reserved_id(1));
        assert!(is_reserved_id(2));
        assert!(is_reserved_id(3));
        assert!(!is_reserved_id(0));
        assert!(!is_reserved_id(4));
        assert!(!is_reserved_id(-1));
    }

    #[test]
    fn test_is_valid_matchadd_id() {
        assert!(is_valid_matchadd_id(-1)); // Auto
        assert!(is_valid_matchadd_id(4)); // First valid manual
        assert!(is_valid_matchadd_id(100)); // Large manual

        assert!(!is_valid_matchadd_id(1)); // Reserved
        assert!(!is_valid_matchadd_id(2)); // Reserved
        assert!(!is_valid_matchadd_id(3)); // Reserved
        assert!(!is_valid_matchadd_id(0)); // Invalid
    }

    #[test]
    fn test_is_valid_matchaddpos_id() {
        assert!(is_valid_matchaddpos_id(-1)); // Auto
        assert!(is_valid_matchaddpos_id(3)); // :3match substitute
        assert!(is_valid_matchaddpos_id(4)); // First valid manual
        assert!(is_valid_matchaddpos_id(100)); // Large manual

        assert!(!is_valid_matchaddpos_id(1)); // Reserved
        assert!(!is_valid_matchaddpos_id(2)); // Reserved
        assert!(!is_valid_matchaddpos_id(0)); // Invalid
    }

    #[test]
    fn test_validate_add_inputs() {
        assert_eq!(validate_add_inputs("Error", Some("foo")), AddError::Ok);
        assert_eq!(validate_add_inputs("Error", None), AddError::Ok);
        assert_eq!(validate_add_inputs("", Some("foo")), AddError::EmptyGroup);
        assert_eq!(
            validate_add_inputs("Error", Some("")),
            AddError::EmptyPattern
        );
    }

    #[test]
    fn test_calc_next_id_after_manual() {
        // Low current, high manual -> adjust
        assert_eq!(calc_next_id_after_manual(10, 50), 150);
        // High current, low manual -> keep current
        assert_eq!(calc_next_id_after_manual(200, 50), 200);
    }

    #[test]
    fn test_add_error_codes() {
        assert_eq!(AddError::Ok.to_c_int(), 0);
        assert!(AddError::EmptyGroup.to_c_int() < 0);
        assert!(AddError::EmptyPattern.to_c_int() < 0);
        assert!(AddError::InvalidId.to_c_int() < 0);
        assert!(AddError::IdTaken.to_c_int() < 0);
        assert!(AddError::IdReserved.to_c_int() < 0);
    }
}
