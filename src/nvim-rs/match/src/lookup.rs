//! Match lookup and query operations
//!
//! This module provides functions for querying match information:
//! - Get match by ID (for `matcharg()`)
//! - Count matches in window
//! - Iterate over matches

use std::ffi::c_int;

use crate::id::is_valid_id;

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
    fn nvim_match_item_get_priority(m: *mut MatchItemHandle) -> c_int;
    fn nvim_match_item_get_hlg_id(m: *mut MatchItemHandle) -> c_int;
    fn nvim_match_item_has_pattern(m: *mut MatchItemHandle) -> bool;
    fn nvim_match_item_has_positions(m: *mut MatchItemHandle) -> bool;
    fn nvim_match_item_get_pos_count(m: *mut MatchItemHandle) -> c_int;
}

// =============================================================================
// Match Information
// =============================================================================

/// Information about a match item.
#[derive(Debug, Clone, Copy)]
pub struct MatchInfo {
    /// Match ID
    pub id: i32,
    /// Match priority
    pub priority: i32,
    /// Highlight group ID
    pub hlg_id: i32,
    /// Whether the match uses a pattern
    pub has_pattern: bool,
    /// Whether the match uses positions
    pub has_positions: bool,
    /// Number of position entries (if `has_positions`)
    pub pos_count: i32,
}

impl MatchInfo {
    /// Create an invalid/empty match info.
    #[must_use]
    pub const fn invalid() -> Self {
        Self {
            id: 0,
            priority: 0,
            hlg_id: 0,
            has_pattern: false,
            has_positions: false,
            pos_count: 0,
        }
    }

    /// Check if this is a valid match info.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.id > 0
    }
}

// =============================================================================
// Lookup Functions
// =============================================================================

/// Get a match by ID from the window's match list.
///
/// # Safety
///
/// `wp` must be a valid pointer to a `win_T`.
#[must_use]
pub unsafe fn get_by_id(wp: *mut WinHandle, id: i32) -> *mut MatchItemHandle {
    if wp.is_null() || !is_valid_id(id) {
        return std::ptr::null_mut();
    }

    let mut cur = nvim_match_get_head(wp);
    while !cur.is_null() {
        if nvim_match_item_get_id(cur) == id {
            return cur;
        }
        cur = nvim_match_item_next(cur);
    }

    std::ptr::null_mut()
}

/// Get information about a match by ID.
///
/// # Safety
///
/// `wp` must be a valid pointer to a `win_T`.
#[must_use]
pub unsafe fn get_info_by_id(wp: *mut WinHandle, id: i32) -> MatchInfo {
    let item = get_by_id(wp, id);
    if item.is_null() {
        return MatchInfo::invalid();
    }

    MatchInfo {
        id: nvim_match_item_get_id(item),
        priority: nvim_match_item_get_priority(item),
        hlg_id: nvim_match_item_get_hlg_id(item),
        has_pattern: nvim_match_item_has_pattern(item),
        has_positions: nvim_match_item_has_positions(item),
        pos_count: nvim_match_item_get_pos_count(item),
    }
}

/// Count the number of matches in the window.
///
/// # Safety
///
/// `wp` must be a valid pointer to a `win_T`.
#[must_use]
pub unsafe fn count_matches(wp: *mut WinHandle) -> i32 {
    if wp.is_null() {
        return 0;
    }

    let mut count = 0;
    let mut cur = nvim_match_get_head(wp);
    while !cur.is_null() {
        count += 1;
        cur = nvim_match_item_next(cur);
    }

    count
}

/// Check if a window has any matches.
///
/// # Safety
///
/// `wp` must be a valid pointer to a `win_T`.
#[must_use]
pub unsafe fn has_matches(wp: *mut WinHandle) -> bool {
    if wp.is_null() {
        return false;
    }
    !nvim_match_get_head(wp).is_null()
}

/// Get the first match in the window.
///
/// # Safety
///
/// `wp` must be a valid pointer to a `win_T`.
#[must_use]
pub unsafe fn get_first(wp: *mut WinHandle) -> *mut MatchItemHandle {
    if wp.is_null() {
        return std::ptr::null_mut();
    }
    nvim_match_get_head(wp)
}

/// Get the next match after the given one.
///
/// # Safety
///
/// `m` must be a valid pointer to a `matchitem_T`.
#[must_use]
pub unsafe fn get_next(m: *mut MatchItemHandle) -> *mut MatchItemHandle {
    if m.is_null() {
        return std::ptr::null_mut();
    }
    nvim_match_item_next(m)
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Get a match by ID.
///
/// # Safety
///
/// `wp` must be a valid pointer to a `win_T`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_match_get_by_id(wp: *mut WinHandle, id: c_int) -> *mut MatchItemHandle {
    get_by_id(wp, id)
}

/// Get match info by ID.
///
/// Populates the out parameters with match information.
/// Returns true if match was found.
///
/// # Safety
///
/// `wp` must be a valid pointer to a `win_T`.
/// Out pointers must be valid if not null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_match_get_info_by_id(
    wp: *mut WinHandle,
    id: c_int,
    out_priority: *mut c_int,
    out_hlg_id: *mut c_int,
    out_has_pattern: *mut c_int,
    out_has_positions: *mut c_int,
    out_pos_count: *mut c_int,
) -> c_int {
    let info = get_info_by_id(wp, id);

    if !info.is_valid() {
        return 0;
    }

    if !out_priority.is_null() {
        *out_priority = info.priority;
    }
    if !out_hlg_id.is_null() {
        *out_hlg_id = info.hlg_id;
    }
    if !out_has_pattern.is_null() {
        *out_has_pattern = c_int::from(info.has_pattern);
    }
    if !out_has_positions.is_null() {
        *out_has_positions = c_int::from(info.has_positions);
    }
    if !out_pos_count.is_null() {
        *out_pos_count = info.pos_count;
    }

    1
}

/// Count matches in window.
///
/// # Safety
///
/// `wp` must be a valid pointer to a `win_T`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_match_count(wp: *mut WinHandle) -> c_int {
    count_matches(wp)
}

/// Check if window has any matches.
///
/// # Safety
///
/// `wp` must be a valid pointer to a `win_T`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_match_has_matches(wp: *mut WinHandle) -> c_int {
    c_int::from(has_matches(wp))
}

/// Get first match in window.
///
/// # Safety
///
/// `wp` must be a valid pointer to a `win_T`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_match_get_first(wp: *mut WinHandle) -> *mut MatchItemHandle {
    get_first(wp)
}

/// Get next match after the given one.
///
/// # Safety
///
/// `m` must be a valid pointer to a `matchitem_T`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_match_get_next(m: *mut MatchItemHandle) -> *mut MatchItemHandle {
    get_next(m)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_info_invalid() {
        let info = MatchInfo::invalid();
        assert!(!info.is_valid());
        assert_eq!(info.id, 0);
        assert_eq!(info.priority, 0);
    }

    #[test]
    fn test_match_info_valid() {
        let info = MatchInfo {
            id: 5,
            priority: 10,
            hlg_id: 1,
            has_pattern: true,
            has_positions: false,
            pos_count: 0,
        };
        assert!(info.is_valid());
    }
}
