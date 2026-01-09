//! Search pattern storage and retrieval
//!
//! This module provides functions for managing search patterns,
//! including the main search pattern, substitute pattern, and
//! the last used pattern selection.

use std::ffi::{c_char, c_int};

use crate::state;

// =============================================================================
// C Functions
// =============================================================================

extern "C" {
    /// Get the pattern string from spats array.
    fn nvim_get_spat_pat(idx: c_int) -> *const c_char;
}

// =============================================================================
// Pattern Index Functions
// =============================================================================

/// Get the pattern string for the search pattern (index 0).
///
/// # Safety
/// Returns a pointer to allocated memory that may be freed.
#[inline]
pub unsafe fn get_search_pattern() -> *const c_char {
    nvim_get_spat_pat(state::RE_SEARCH)
}

/// Get the pattern string for the substitute pattern (index 1).
///
/// # Safety
/// Returns a pointer to allocated memory that may be freed.
#[inline]
pub unsafe fn get_subst_pattern() -> *const c_char {
    nvim_get_spat_pat(state::RE_SUBST)
}

/// Get the pattern length for the search pattern.
#[inline]
pub fn get_search_pattern_len() -> usize {
    state::get_spat_patlen(state::RE_SEARCH)
}

/// Get the pattern length for the substitute pattern.
#[inline]
pub fn get_subst_pattern_len() -> usize {
    state::get_spat_patlen(state::RE_SUBST)
}

/// Check if the search pattern is empty or NULL.
#[inline]
pub fn search_pattern_is_empty() -> bool {
    // SAFETY: Just checking for NULL pointer
    unsafe {
        let pat = get_search_pattern();
        pat.is_null() || get_search_pattern_len() == 0
    }
}

/// Check if the substitute pattern is empty or NULL.
#[inline]
pub fn subst_pattern_is_empty() -> bool {
    // SAFETY: Just checking for NULL pointer
    unsafe {
        let pat = get_subst_pattern();
        pat.is_null() || get_subst_pattern_len() == 0
    }
}

/// Get the pattern for the last used index (search or substitute).
///
/// # Safety
/// Returns a pointer to allocated memory that may be freed.
#[inline]
pub unsafe fn get_last_used_pattern() -> *const c_char {
    nvim_get_spat_pat(state::get_last_idx())
}

/// Get the length of the last used pattern.
#[inline]
pub fn get_last_used_pattern_len() -> usize {
    state::get_spat_patlen(state::get_last_idx())
}

/// Check if the last used pattern was the search pattern.
#[inline]
pub fn last_was_search() -> bool {
    state::get_last_idx() == state::RE_SEARCH
}

/// Check if the last used pattern was the substitute pattern.
#[inline]
pub fn last_was_subst() -> bool {
    state::get_last_idx() == state::RE_SUBST
}

// =============================================================================
// Pattern Attributes
// =============================================================================

/// Check if magic mode is enabled for the search pattern.
#[inline]
pub fn search_pattern_magic() -> bool {
    state::get_spat_magic(state::RE_SEARCH)
}

/// Check if magic mode is enabled for the substitute pattern.
#[inline]
pub fn subst_pattern_magic() -> bool {
    state::get_spat_magic(state::RE_SUBST)
}

/// Check if no_smartcase is set for the search pattern.
#[inline]
pub fn search_pattern_no_scs() -> bool {
    state::get_spat_no_scs(state::RE_SEARCH)
}

/// Check if no_smartcase is set for the substitute pattern.
#[inline]
pub fn subst_pattern_no_scs() -> bool {
    state::get_spat_no_scs(state::RE_SUBST)
}

// =============================================================================
// Match Result Pattern (mr_pattern)
// =============================================================================

/// Get the pattern used by search_regcomp().
///
/// This is the pattern that was actually used for the last search,
/// which may have been reversed if 'rl' option is set.
///
/// # Safety
/// Returns a pointer to static memory that may be invalidated.
#[inline]
pub unsafe fn get_mr_pattern() -> *const c_char {
    state::get_mr_pattern()
}

/// Get the length of the mr_pattern.
#[inline]
pub fn get_mr_pattern_len() -> usize {
    state::get_mr_patternlen()
}

/// Check if mr_pattern is empty or NULL.
#[inline]
pub fn mr_pattern_is_empty() -> bool {
    // SAFETY: Just checking for NULL pointer
    unsafe {
        let pat = get_mr_pattern();
        pat.is_null() || get_mr_pattern_len() == 0
    }
}

// =============================================================================
// Pattern String Helpers
// =============================================================================

/// Get the search pattern as a Rust string slice, if available.
///
/// Returns None if the pattern is NULL or invalid UTF-8.
///
/// # Safety
/// The returned string slice is only valid as long as the pattern
/// is not modified in C code.
pub unsafe fn search_pattern_as_str() -> Option<&'static str> {
    let ptr = get_search_pattern();
    if ptr.is_null() {
        return None;
    }
    let len = get_search_pattern_len();
    if len == 0 {
        return Some("");
    }
    let slice = std::slice::from_raw_parts(ptr as *const u8, len);
    std::str::from_utf8(slice).ok()
}

/// Get the substitute pattern as a Rust string slice, if available.
///
/// # Safety
/// The returned string slice is only valid as long as the pattern
/// is not modified in C code.
pub unsafe fn subst_pattern_as_str() -> Option<&'static str> {
    let ptr = get_subst_pattern();
    if ptr.is_null() {
        return None;
    }
    let len = get_subst_pattern_len();
    if len == 0 {
        return Some("");
    }
    let slice = std::slice::from_raw_parts(ptr as *const u8, len);
    std::str::from_utf8(slice).ok()
}

/// Get the mr_pattern as a Rust string slice, if available.
///
/// # Safety
/// The returned string slice is only valid until the next search operation.
pub unsafe fn mr_pattern_as_str() -> Option<&'static str> {
    let ptr = get_mr_pattern();
    if ptr.is_null() {
        return None;
    }
    let len = get_mr_pattern_len();
    if len == 0 {
        return Some("");
    }
    let slice = std::slice::from_raw_parts(ptr as *const u8, len);
    std::str::from_utf8(slice).ok()
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Get the search pattern pointer.
///
/// # Safety
/// Returns a pointer to allocated memory that may be freed by C code.
#[no_mangle]
pub unsafe extern "C" fn rs_get_search_pattern() -> *const c_char {
    get_search_pattern()
}

/// FFI: Get the substitute pattern pointer.
///
/// # Safety
/// Returns a pointer to allocated memory that may be freed by C code.
#[no_mangle]
pub unsafe extern "C" fn rs_get_subst_pattern() -> *const c_char {
    get_subst_pattern()
}

/// FFI: Get the search pattern length.
#[no_mangle]
pub extern "C" fn rs_get_search_pattern_len() -> usize {
    get_search_pattern_len()
}

/// FFI: Get the substitute pattern length.
#[no_mangle]
pub extern "C" fn rs_get_subst_pattern_len() -> usize {
    get_subst_pattern_len()
}

/// FFI: Check if search pattern is empty.
#[no_mangle]
pub extern "C" fn rs_search_pattern_is_empty() -> c_int {
    c_int::from(search_pattern_is_empty())
}

/// FFI: Check if substitute pattern is empty.
#[no_mangle]
pub extern "C" fn rs_subst_pattern_is_empty() -> c_int {
    c_int::from(subst_pattern_is_empty())
}

/// FFI: Get the last used pattern pointer.
///
/// # Safety
/// Returns a pointer to allocated memory that may be freed by C code.
#[no_mangle]
pub unsafe extern "C" fn rs_get_last_used_pattern() -> *const c_char {
    get_last_used_pattern()
}

/// FFI: Get the last used pattern length.
#[no_mangle]
pub extern "C" fn rs_get_last_used_pattern_len() -> usize {
    get_last_used_pattern_len()
}

/// FFI: Check if last pattern was search.
#[no_mangle]
pub extern "C" fn rs_last_was_search() -> c_int {
    c_int::from(last_was_search())
}

/// FFI: Check if last pattern was substitute.
#[no_mangle]
pub extern "C" fn rs_last_was_subst() -> c_int {
    c_int::from(last_was_subst())
}

/// FFI: Check if search pattern has magic.
#[no_mangle]
pub extern "C" fn rs_search_pattern_magic() -> c_int {
    c_int::from(search_pattern_magic())
}

/// FFI: Check if substitute pattern has magic.
#[no_mangle]
pub extern "C" fn rs_subst_pattern_magic() -> c_int {
    c_int::from(subst_pattern_magic())
}

/// FFI: Get mr_pattern pointer.
///
/// # Safety
/// Returns a pointer to static memory that may be invalidated by subsequent searches.
#[no_mangle]
pub unsafe extern "C" fn rs_get_mr_pattern() -> *const c_char {
    get_mr_pattern()
}

/// FFI: Get mr_pattern length.
#[no_mangle]
pub extern "C" fn rs_get_mr_pattern_len() -> usize {
    get_mr_pattern_len()
}

/// FFI: Check if mr_pattern is empty.
#[no_mangle]
pub extern "C" fn rs_mr_pattern_is_empty() -> c_int {
    c_int::from(mr_pattern_is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_indices() {
        assert_eq!(state::RE_SEARCH, 0);
        assert_eq!(state::RE_SUBST, 1);
    }
}
