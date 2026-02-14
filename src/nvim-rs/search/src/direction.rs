//! Search direction and offset handling
//!
//! This module provides functions for managing search direction state
//! and search offset configuration.

use std::ffi::{c_char, c_int, c_longlong};

use crate::state;

// =============================================================================
// C Functions
// =============================================================================

extern "C" {
    /// Set the search direction in spats[0].off.dir
    fn nvim_set_spat_off_dir(idx: c_int, dir: c_char);

    /// Set the line offset flag in spats array
    fn nvim_set_spat_off_line(idx: c_int, line: c_int);

    /// Set the end offset flag in spats array
    fn nvim_set_spat_off_end(idx: c_int, end: c_int);

    /// Set the offset value in spats array
    fn nvim_set_spat_off_off(idx: c_int, off: c_longlong);

    /// Call set_vv_searchforward() in C
    fn nvim_call_set_vv_searchforward();
}

// =============================================================================
// Direction Constants
// =============================================================================

/// Search direction: forward (/)
pub const DIR_FORWARD: c_char = b'/' as c_char;
/// Search direction: backward (?)
pub const DIR_BACKWARD: c_char = b'?' as c_char;

/// Integer direction: forward
pub const FORWARD: c_int = 1;
/// Integer direction: backward
pub const BACKWARD: c_int = -1;

// =============================================================================
// Direction Functions
// =============================================================================

/// Get the current search direction character ('/' or '?').
#[inline]
pub fn get_search_direction() -> c_char {
    state::get_spat_off_dir(state::RE_SEARCH)
}

/// Set the search direction character.
///
/// # Safety
/// Modifies global state.
#[inline]
pub fn set_search_direction(dir: c_char) {
    // SAFETY: Setting global state
    unsafe {
        nvim_set_spat_off_dir(state::RE_SEARCH, dir);
        nvim_call_set_vv_searchforward();
    }
}

/// Check if the current search direction is forward.
#[inline]
pub fn is_search_forward() -> bool {
    get_search_direction() == DIR_FORWARD
}

/// Check if the current search direction is backward.
#[inline]
pub fn is_search_backward() -> bool {
    get_search_direction() == DIR_BACKWARD
}

/// Reverse the current search direction.
#[inline]
pub fn reverse_search_direction() {
    let current = get_search_direction();
    let new_dir = if current == DIR_FORWARD {
        DIR_BACKWARD
    } else {
        DIR_FORWARD
    };
    set_search_direction(new_dir);
}

/// Convert integer direction to character direction.
#[inline]
pub fn int_to_char_direction(dir: c_int) -> c_char {
    if dir == FORWARD {
        DIR_FORWARD
    } else {
        DIR_BACKWARD
    }
}

/// Convert character direction to integer direction.
#[inline]
pub fn char_to_int_direction(dir: c_char) -> c_int {
    if dir == DIR_FORWARD {
        FORWARD
    } else {
        BACKWARD
    }
}

// =============================================================================
// Search Offset Functions
// =============================================================================

/// Search offset configuration.
#[derive(Debug, Clone, Copy, Default)]
pub struct SearchOffset {
    /// Search direction: '/' for forward, '?' for backward
    pub dir: c_char,
    /// Whether offset is a line offset
    pub line: bool,
    /// Whether to position at end of match
    pub end: bool,
    /// The offset value
    pub off: i64,
}

impl SearchOffset {
    /// Create a new SearchOffset with default values.
    pub fn new() -> Self {
        Self {
            dir: DIR_FORWARD,
            line: false,
            end: false,
            off: 0,
        }
    }

    /// Create a SearchOffset for forward search.
    pub fn forward() -> Self {
        Self {
            dir: DIR_FORWARD,
            ..Self::new()
        }
    }

    /// Create a SearchOffset for backward search.
    pub fn backward() -> Self {
        Self {
            dir: DIR_BACKWARD,
            ..Self::new()
        }
    }

    /// Check if this offset represents forward search.
    #[inline]
    pub fn is_forward(&self) -> bool {
        self.dir == DIR_FORWARD
    }

    /// Check if this offset represents backward search.
    #[inline]
    pub fn is_backward(&self) -> bool {
        self.dir == DIR_BACKWARD
    }

    /// Check if there is any offset applied.
    #[inline]
    pub fn has_offset(&self) -> bool {
        self.line || self.end || self.off != 0
    }
}

/// Get the search offset for the given pattern index.
pub fn get_search_offset(idx: c_int) -> SearchOffset {
    SearchOffset {
        dir: state::get_spat_off_dir(idx),
        line: state::get_spat_off_line(idx),
        end: state::get_spat_off_end(idx),
        off: state::get_spat_off_off(idx),
    }
}

/// Set the search offset for the given pattern index.
///
/// # Safety
/// Modifies global state.
pub fn set_search_offset(idx: c_int, offset: &SearchOffset) {
    // SAFETY: Setting global state through C accessors
    unsafe {
        nvim_set_spat_off_dir(idx, offset.dir);
        nvim_set_spat_off_line(idx, c_int::from(offset.line));
        nvim_set_spat_off_end(idx, c_int::from(offset.end));
        nvim_set_spat_off_off(idx, offset.off);
    }
}

/// Reset the search direction to forward (for "gd" and "gD" commands).
pub fn reset_search_dir() {
    set_search_direction(DIR_FORWARD);
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Get the current search direction from spats[0].
#[no_mangle]
pub extern "C" fn rs_search_get_dir() -> c_char {
    get_search_direction()
}

/// FFI: Set the search direction in spats[0].
#[no_mangle]
pub extern "C" fn rs_search_set_dir(dir: c_int) {
    set_search_direction(dir as c_char);
}

/// FFI: Check if search direction is forward.
#[no_mangle]
pub extern "C" fn rs_is_search_forward() -> c_int {
    c_int::from(is_search_forward())
}

/// FFI: Check if search direction is backward.
#[no_mangle]
pub extern "C" fn rs_is_search_backward() -> c_int {
    c_int::from(is_search_backward())
}

/// FFI: Reverse the search direction.
#[no_mangle]
pub extern "C" fn rs_reverse_search_direction() {
    reverse_search_direction();
}

/// Set the search direction character without updating the vim variable.
///
/// This is the Rust equivalent of `set_search_direction()` in search.c
/// which only sets `spats[0].off.dir` without calling `set_vv_searchforward()`.
#[inline]
pub fn set_search_direction_raw(dir: c_char) {
    // SAFETY: Setting global state
    unsafe {
        nvim_set_spat_off_dir(state::RE_SEARCH, dir);
    }
}

/// FFI: Set the search direction (raw, without updating vim variable).
///
/// This matches the C `set_search_direction()` semantics exactly.
#[no_mangle]
pub extern "C" fn rs_set_search_direction_raw(cdir: c_int) {
    set_search_direction_raw(cdir as c_char);
}

/// FFI: Reset search direction to forward.
#[no_mangle]
pub extern "C" fn rs_reset_search_dir() {
    reset_search_dir();
}

/// FFI: Get search offset line flag.
#[no_mangle]
pub extern "C" fn rs_get_search_offset_line(idx: c_int) -> c_int {
    c_int::from(state::get_spat_off_line(idx))
}

/// FFI: Get search offset end flag.
#[no_mangle]
pub extern "C" fn rs_get_search_offset_end(idx: c_int) -> c_int {
    c_int::from(state::get_spat_off_end(idx))
}

/// FFI: Get search offset value.
#[no_mangle]
pub extern "C" fn rs_get_search_offset_off(idx: c_int) -> c_longlong {
    state::get_spat_off_off(idx)
}

/// FFI: Check if there's any search offset.
#[no_mangle]
pub extern "C" fn rs_has_search_offset(idx: c_int) -> c_int {
    let offset = get_search_offset(idx);
    c_int::from(offset.has_offset())
}

/// FFI: Convert integer direction to char direction.
#[no_mangle]
pub extern "C" fn rs_int_to_char_direction(dir: c_int) -> c_char {
    int_to_char_direction(dir)
}

/// FFI: Convert char direction to integer direction.
#[no_mangle]
pub extern "C" fn rs_char_to_int_direction(dir: c_char) -> c_int {
    char_to_int_direction(dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_constants() {
        assert_eq!(DIR_FORWARD, b'/' as c_char);
        assert_eq!(DIR_BACKWARD, b'?' as c_char);
        assert_eq!(FORWARD, 1);
        assert_eq!(BACKWARD, -1);
    }

    #[test]
    fn test_direction_conversion() {
        assert_eq!(int_to_char_direction(FORWARD), DIR_FORWARD);
        assert_eq!(int_to_char_direction(BACKWARD), DIR_BACKWARD);
        assert_eq!(char_to_int_direction(DIR_FORWARD), FORWARD);
        assert_eq!(char_to_int_direction(DIR_BACKWARD), BACKWARD);
    }

    #[test]
    fn test_search_offset_new() {
        let offset = SearchOffset::new();
        assert_eq!(offset.dir, DIR_FORWARD);
        assert!(!offset.line);
        assert!(!offset.end);
        assert_eq!(offset.off, 0);
    }

    #[test]
    fn test_search_offset_forward_backward() {
        let fwd = SearchOffset::forward();
        assert!(fwd.is_forward());
        assert!(!fwd.is_backward());

        let bwd = SearchOffset::backward();
        assert!(!bwd.is_forward());
        assert!(bwd.is_backward());
    }

    #[test]
    fn test_search_offset_has_offset() {
        let mut offset = SearchOffset::new();
        assert!(!offset.has_offset());

        offset.line = true;
        assert!(offset.has_offset());

        offset.line = false;
        offset.end = true;
        assert!(offset.has_offset());

        offset.end = false;
        offset.off = 5;
        assert!(offset.has_offset());

        offset.off = 0;
        assert!(!offset.has_offset());
    }
}
