//! Search state accessors
//!
//! This module provides Rust wrappers for accessing search state variables.
//! Character search state lives in `char_search_state.rs` (Rust-owned).
//! Pattern state (spats, mr_pattern, etc.) lives in `search_state.rs` (Rust-owned).

use std::ffi::{c_char, c_int, c_longlong, c_uchar};

use crate::char_search_state;
use crate::search_state;

// =============================================================================
// Direction Constants
// =============================================================================

/// Forward search direction.
pub const FORWARD: c_int = 1;
/// Backward search direction.
pub const BACKWARD: c_int = -1;

// =============================================================================
// Character Search State
// =============================================================================

/// Get the last character search direction.
/// Returns FORWARD (1) or BACKWARD (-1).
#[inline]
pub fn get_lastcdir() -> c_int {
    char_search_state::get_lastcdir()
}

/// Set the last character search direction.
#[inline]
pub fn set_lastcdir(dir: c_int) {
    char_search_state::set_lastcdir(dir);
}

/// Check if the last character search direction was forward.
#[inline]
pub fn last_csearch_forward() -> bool {
    get_lastcdir() == FORWARD
}

/// Get whether the last character search was a 't' (until) command.
#[inline]
pub fn get_last_t_cmd() -> c_int {
    c_int::from(char_search_state::get_last_t_cmd())
}

/// Set whether the last character search was a 't' (until) command.
#[inline]
pub fn set_last_t_cmd(t_cmd: bool) {
    char_search_state::set_last_t_cmd(t_cmd);
}

/// Check if the last character search was an 'until' command (t or T).
#[inline]
pub fn last_csearch_until() -> bool {
    get_last_t_cmd() != 0
}

/// Get the last character search bytes pointer.
///
/// # Safety
/// Returns a pointer to static memory.
#[inline]
pub unsafe fn get_lastc_bytes() -> *const c_char {
    char_search_state::get_lastc_bytes_ptr()
}

/// Get the length of the last character search bytes.
#[inline]
pub fn get_lastc_bytelen() -> c_int {
    char_search_state::get_lastc_bytelen()
}

/// Set the length of the last character search bytes.
#[inline]
pub fn set_lastc_bytelen(len: c_int) {
    char_search_state::set_lastc_bytelen(len);
}

/// Get a character from the lastc array.
#[inline]
pub fn get_lastc(idx: c_int) -> c_uchar {
    char_search_state::get_lastc(idx)
}

/// Set a character in the lastc array.
#[inline]
pub fn set_lastc(idx: c_int, val: c_uchar) {
    char_search_state::set_lastc(idx, val);
}

// =============================================================================
// Pattern Index State
// =============================================================================

/// Pattern index for search.
pub const RE_SEARCH: c_int = 0;
/// Pattern index for substitute.
pub const RE_SUBST: c_int = 1;
/// Pattern index for last used.
pub const RE_LAST: c_int = 2;
/// Pattern index for both search and substitute.
pub const RE_BOTH: c_int = 3;

/// Get the last used pattern index (search or substitute).
#[inline]
pub fn get_last_idx() -> c_int {
    search_state::get_last_idx()
}

/// Set the last used pattern index.
#[inline]
pub fn set_last_idx(idx: c_int) {
    search_state::set_last_idx(idx);
}

/// Check if the search pattern was the last used one (not substitute).
#[inline]
pub fn search_was_last_used() -> bool {
    get_last_idx() == RE_SEARCH
}

// =============================================================================
// Match Result Pattern
// =============================================================================

/// Get the pattern used by search_regcomp().
///
/// # Safety
/// Returns a pointer to static memory that may be invalidated by subsequent searches.
#[inline]
pub unsafe fn get_mr_pattern() -> *const c_char {
    search_state::get_mr_pattern()
}

/// Get the length of the pattern used by search_regcomp().
#[inline]
pub fn get_mr_patternlen() -> usize {
    search_state::get_mr_patternlen()
}

// =============================================================================
// Search Pattern Array State (spats)
// =============================================================================

/// Get the pattern string for the given index.
///
/// # Safety
/// Returns a pointer to allocated memory that may be freed.
#[inline]
pub unsafe fn get_spat_pat(idx: c_int) -> *const c_char {
    search_state::get_spat_pat(idx)
}

/// Get the pattern length for the given index.
#[inline]
pub fn get_spat_patlen(idx: c_int) -> usize {
    search_state::get_spat_patlen(idx)
}

/// Check if magic is enabled for the pattern at the given index.
#[inline]
pub fn get_spat_magic(idx: c_int) -> bool {
    search_state::get_spat_magic(idx)
}

/// Check if no_scs (no smartcase) is set for the pattern at the given index.
#[inline]
pub fn get_spat_no_scs(idx: c_int) -> bool {
    search_state::get_spat_no_scs(idx)
}

/// Get the search direction character ('/' or '?') for the pattern at the given index.
#[inline]
pub fn get_spat_off_dir(idx: c_int) -> c_char {
    search_state::get_spat_off_dir(idx) as c_char
}

/// Check if line offset is enabled for the pattern at the given index.
#[inline]
pub fn get_spat_off_line(idx: c_int) -> bool {
    search_state::get_spat_off_line(idx)
}

/// Check if end-of-match offset is enabled for the pattern at the given index.
#[inline]
pub fn get_spat_off_end(idx: c_int) -> bool {
    search_state::get_spat_off_end(idx)
}

/// Get the offset value for the pattern at the given index.
#[inline]
pub fn get_spat_off_off(idx: c_int) -> i64 {
    search_state::get_spat_off_off(idx)
}

// =============================================================================
// Save/Restore State
// =============================================================================

/// Get the current save level for search patterns.
#[inline]
pub fn get_save_level() -> c_int {
    search_state::get_save_level()
}

/// Check if patterns are currently saved (save_level > 0).
#[inline]
pub fn patterns_are_saved() -> bool {
    get_save_level() > 0
}

/// Get the did_save_last_search_spat counter.
#[inline]
pub fn get_did_save_last_search_spat() -> c_int {
    search_state::get_did_save_last_search_spat()
}

/// Check if incremental search pattern is currently saved.
#[inline]
pub fn incsearch_pattern_is_saved() -> bool {
    get_did_save_last_search_spat() > 0
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI wrapper: Check if last character search was forward.
#[no_mangle]
pub extern "C" fn rs_state_last_csearch_forward() -> c_int {
    c_int::from(last_csearch_forward())
}

/// FFI wrapper: Check if last character search was until.
#[no_mangle]
pub extern "C" fn rs_state_last_csearch_until() -> c_int {
    c_int::from(last_csearch_until())
}

/// FFI wrapper: Get last character search bytes.
///
/// # Safety
/// Returns pointer to static memory.
#[no_mangle]
pub unsafe extern "C" fn rs_state_get_lastc_bytes() -> *const c_char {
    get_lastc_bytes()
}

/// FFI wrapper: Get last character search byte length.
#[no_mangle]
pub extern "C" fn rs_state_get_lastc_bytelen() -> c_int {
    get_lastc_bytelen()
}

/// FFI wrapper: Check if search pattern was last used.
#[no_mangle]
pub extern "C" fn rs_state_search_was_last_used() -> c_int {
    c_int::from(search_was_last_used())
}

/// FFI wrapper: Get last pattern index.
#[no_mangle]
pub extern "C" fn rs_state_get_last_idx() -> c_int {
    get_last_idx()
}

/// FFI wrapper: Check if patterns are saved.
#[no_mangle]
pub extern "C" fn rs_state_patterns_are_saved() -> c_int {
    c_int::from(patterns_are_saved())
}

/// FFI wrapper: Check if incsearch pattern is saved.
#[no_mangle]
pub extern "C" fn rs_state_incsearch_is_saved() -> c_int {
    c_int::from(incsearch_pattern_is_saved())
}

/// FFI wrapper: Get pattern length for given index.
#[no_mangle]
pub extern "C" fn rs_state_get_spat_patlen(idx: c_int) -> usize {
    get_spat_patlen(idx)
}

/// FFI wrapper: Check if magic is set for pattern at index.
#[no_mangle]
pub extern "C" fn rs_state_get_spat_magic(idx: c_int) -> c_int {
    c_int::from(get_spat_magic(idx))
}

/// FFI wrapper: Get search direction for pattern at index.
#[no_mangle]
pub extern "C" fn rs_state_get_spat_off_dir(idx: c_int) -> c_char {
    get_spat_off_dir(idx)
}

/// FFI wrapper: Check if line offset is set for pattern at index.
#[no_mangle]
pub extern "C" fn rs_state_get_spat_off_line(idx: c_int) -> c_int {
    c_int::from(get_spat_off_line(idx))
}

/// FFI wrapper: Check if end offset is set for pattern at index.
#[no_mangle]
pub extern "C" fn rs_state_get_spat_off_end(idx: c_int) -> c_int {
    c_int::from(get_spat_off_end(idx))
}

/// FFI wrapper: Get offset value for pattern at index.
#[no_mangle]
pub extern "C" fn rs_state_get_spat_off_off(idx: c_int) -> c_longlong {
    get_spat_off_off(idx)
}

/// FFI wrapper: Get mr_pattern length.
#[no_mangle]
pub extern "C" fn rs_state_get_mr_patternlen() -> usize {
    get_mr_patternlen()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_constants() {
        assert_eq!(FORWARD, 1);
        assert_eq!(BACKWARD, -1);
        assert_eq!(FORWARD, -BACKWARD);
    }

    #[test]
    fn test_re_constants() {
        assert_eq!(RE_SEARCH, 0);
        assert_eq!(RE_SUBST, 1);
        assert_eq!(RE_LAST, 2);
        assert_eq!(RE_BOTH, 3);
    }
}
