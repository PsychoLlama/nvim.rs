//! Incremental search support
//!
//! This module provides functions for incremental search (incsearch) functionality,
//! which highlights matches as the user types a search pattern.

use std::ffi::c_int;

use crate::helpers;
use crate::search_state;

// =============================================================================
// C External Functions
// =============================================================================

extern "C" {
    // Incsearch state save/restore
    fn nvim_get_search_match_lines() -> c_int;
    fn nvim_get_search_match_endcol() -> c_int;
    fn nvim_set_search_match_lines(val: c_int);
    fn nvim_set_search_match_endcol(val: c_int);

    // Option accessors
    fn nvim_get_p_is() -> c_int;
    fn nvim_get_p_hls() -> c_int;
}

// =============================================================================
// Incsearch State
// =============================================================================

/// Saved state for incremental search highlighting.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct IncsearchState {
    /// Saved search_match_lines value
    pub saved_match_lines: c_int,
    /// Saved search_match_endcol value
    pub saved_match_endcol: c_int,
    /// Whether state has been saved
    pub saved: bool,
}

impl IncsearchState {
    /// Create a new empty incsearch state.
    pub const fn new() -> Self {
        Self {
            saved_match_lines: 0,
            saved_match_endcol: 0,
            saved: false,
        }
    }

    /// Save the current incsearch state.
    pub fn save(&mut self) {
        // SAFETY: Accessing global variables through accessors
        unsafe {
            self.saved_match_lines = nvim_get_search_match_lines();
            self.saved_match_endcol = nvim_get_search_match_endcol();
        }
        self.saved = true;
    }

    /// Restore the saved incsearch state.
    pub fn restore(&self) {
        if self.saved {
            // SAFETY: Setting global variables through setters
            unsafe {
                nvim_set_search_match_lines(self.saved_match_lines);
                nvim_set_search_match_endcol(self.saved_match_endcol);
            }
        }
    }

    /// Check if state has been saved.
    pub fn is_saved(&self) -> bool {
        self.saved
    }
}

// =============================================================================
// Incsearch Options
// =============================================================================

/// Check if incremental search is enabled ('incsearch' option).
#[inline]
pub fn incsearch_enabled() -> bool {
    // SAFETY: Accessing global option
    unsafe { nvim_get_p_is() != 0 }
}

/// Check if highlight search is enabled ('hlsearch' option).
#[inline]
pub fn hlsearch_enabled() -> bool {
    // SAFETY: Accessing global option
    unsafe { nvim_get_p_hls() != 0 }
}

/// Check if incremental highlighting should be active.
///
/// Returns true if:
/// - 'incsearch' is set, OR
/// - 'hlsearch' is set AND no_hlsearch is false
#[inline]
pub fn should_do_incsearch_highlighting() -> bool {
    if incsearch_enabled() {
        return true;
    }
    hlsearch_enabled() && !helpers::get_no_hlsearch()
}

// =============================================================================
// Match Position Tracking
// =============================================================================

/// Get the current search match lines count.
#[inline]
pub fn get_match_lines() -> c_int {
    // SAFETY: Accessing global variable
    unsafe { nvim_get_search_match_lines() }
}

/// Get the current search match end column.
#[inline]
pub fn get_match_endcol() -> c_int {
    // SAFETY: Accessing global variable
    unsafe { nvim_get_search_match_endcol() }
}

/// Set the search match lines count.
#[inline]
pub fn set_match_lines(lines: c_int) {
    // SAFETY: Setting global variable
    unsafe { nvim_set_search_match_lines(lines) }
}

/// Set the search match end column.
#[inline]
pub fn set_match_endcol(col: c_int) {
    // SAFETY: Setting global variable
    unsafe { nvim_set_search_match_endcol(col) }
}

/// Check if the current match spans multiple lines.
#[inline]
pub fn match_is_multiline() -> bool {
    get_match_lines() > 0
}

/// Check if there's a valid match (match_endcol > 0 or multiline).
#[inline]
pub fn has_valid_match() -> bool {
    get_match_endcol() > 0 || match_is_multiline()
}

// =============================================================================
// Highlight Position Checking
// =============================================================================

/// Check if a line is within the highlighted match range.
///
/// Given the cursor line and a line number, determines if that line
/// is part of the current search match highlight.
#[inline]
pub fn line_in_match_highlight(cursor_lnum: c_int, lnum: c_int) -> bool {
    if !has_valid_match() {
        return false;
    }

    let match_lines = get_match_lines();
    lnum >= cursor_lnum && lnum <= cursor_lnum + match_lines
}

/// Check if we're on the last line of a multiline match.
#[inline]
pub fn is_match_end_line(cursor_lnum: c_int, lnum: c_int) -> bool {
    let match_lines = get_match_lines();
    lnum == cursor_lnum + match_lines
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Create a new incsearch state.
#[no_mangle]
pub extern "C" fn rs_incsearch_state_new() -> IncsearchState {
    IncsearchState::new()
}

/// FFI: Save the current incsearch state.
///
/// # Safety
///
/// If `state` is non-null, it must point to valid, properly aligned memory.
#[no_mangle]
pub unsafe extern "C" fn rs_incsearch_state_save(state: *mut IncsearchState) {
    if !state.is_null() {
        (*state).save();
    }
}

/// FFI: Restore the saved incsearch state.
///
/// # Safety
///
/// If `state` is non-null, it must point to valid, properly aligned memory.
#[no_mangle]
pub unsafe extern "C" fn rs_incsearch_state_restore(state: *const IncsearchState) {
    if !state.is_null() {
        (*state).restore();
    }
}

/// FFI: Check if incsearch is enabled.
#[no_mangle]
pub extern "C" fn rs_incsearch_enabled() -> c_int {
    c_int::from(incsearch_enabled())
}

/// FFI: Check if hlsearch is enabled.
#[no_mangle]
pub extern "C" fn rs_hlsearch_enabled() -> c_int {
    c_int::from(hlsearch_enabled())
}

/// FFI: Check if incsearch highlighting should be done.
#[no_mangle]
pub extern "C" fn rs_should_do_incsearch_highlighting() -> c_int {
    c_int::from(should_do_incsearch_highlighting())
}

/// FFI: Get match lines count.
#[no_mangle]
pub extern "C" fn rs_incsearch_get_match_lines() -> c_int {
    get_match_lines()
}

/// FFI: Get match end column.
#[no_mangle]
pub extern "C" fn rs_incsearch_get_match_endcol() -> c_int {
    get_match_endcol()
}

/// FFI: Set match lines count.
#[no_mangle]
pub extern "C" fn rs_incsearch_set_match_lines(lines: c_int) {
    set_match_lines(lines);
}

/// FFI: Set match end column.
#[no_mangle]
pub extern "C" fn rs_incsearch_set_match_endcol(col: c_int) {
    set_match_endcol(col);
}

/// FFI: Check if match is multiline.
#[no_mangle]
pub extern "C" fn rs_incsearch_match_is_multiline() -> c_int {
    c_int::from(match_is_multiline())
}

/// FFI: Check if there's a valid match.
#[no_mangle]
pub extern "C" fn rs_incsearch_has_valid_match() -> c_int {
    c_int::from(has_valid_match())
}

/// FFI: Check if line is in match highlight range.
#[no_mangle]
pub extern "C" fn rs_incsearch_line_in_match(cursor_lnum: c_int, lnum: c_int) -> c_int {
    c_int::from(line_in_match_highlight(cursor_lnum, lnum))
}

/// FFI: Check if line is the match end line.
#[no_mangle]
pub extern "C" fn rs_incsearch_is_match_end_line(cursor_lnum: c_int, lnum: c_int) -> c_int {
    c_int::from(is_match_end_line(cursor_lnum, lnum))
}

/// FFI: Save incsearch state (search_match_endcol, search_match_lines).
#[no_mangle]
pub extern "C" fn rs_save_incsearch_state() {
    let (endcol, lines) = unsafe {
        (
            nvim_get_search_match_endcol(),
            nvim_get_search_match_lines(),
        )
    };
    search_state::save_incsearch_state_batch(endcol, lines);
}

/// FFI: Restore incsearch state.
#[no_mangle]
pub extern "C" fn rs_restore_incsearch_state() {
    let (endcol, lines) = search_state::restore_incsearch_state_batch();
    unsafe {
        nvim_set_search_match_endcol(endcol);
        nvim_set_search_match_lines(lines);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incsearch_state_new() {
        let state = IncsearchState::new();
        assert_eq!(state.saved_match_lines, 0);
        assert_eq!(state.saved_match_endcol, 0);
        assert!(!state.saved);
    }

    #[test]
    fn test_incsearch_state_default() {
        let state = IncsearchState::default();
        assert!(!state.is_saved());
    }
}
