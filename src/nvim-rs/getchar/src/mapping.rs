//! Key mapping expansion
//!
//! This module provides Rust implementations for key mapping state
//! and related functions.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]

use std::ffi::c_int;

/// Result of mapping lookup in `handle_mapping`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapResult {
    /// Failed, break loop
    Fail = 0,
    /// Get a character from typeahead
    Get = 1,
    /// Try to map again
    Retry = 2,
    /// No matching mapping, get char
    NoMatch = 3,
}

impl From<c_int> for MapResult {
    fn from(value: c_int) -> Self {
        match value {
            0 => Self::Fail,
            1 => Self::Get,
            2 => Self::Retry,
            _ => Self::NoMatch,
        }
    }
}

/// Key length constants
pub mod keylen {
    use std::ffi::c_int;

    /// Need more characters to match mapping
    pub const PART_KEY: c_int = -1;

    /// Part of matching mapping found
    pub const PART_MAP: c_int = -2;
}

/// Mapping timeout state
#[derive(Debug, Clone, Copy, Default)]
pub struct MappingTimeout {
    /// Waited for more than 'timeoutlen' for mapping to complete
    pub mapping_timedout: bool,
    /// Waited for more than 'ttimeoutlen' for key code
    pub keycode_timedout: bool,
}

impl MappingTimeout {
    /// Create a new timeout state
    #[must_use]
    pub const fn new() -> Self {
        Self {
            mapping_timedout: false,
            keycode_timedout: false,
        }
    }

    /// Check if either timeout has occurred
    #[must_use]
    pub const fn is_timedout(&self) -> bool {
        self.mapping_timedout || self.keycode_timedout
    }

    /// Reset both timeout flags
    pub const fn reset(&mut self) {
        self.mapping_timedout = false;
        self.keycode_timedout = false;
    }
}

/// Mapping depth counter for recursive mapping detection
#[derive(Debug, Clone, Copy, Default)]
pub struct MappingDepth {
    /// Current recursion depth
    depth: c_int,
}

impl MappingDepth {
    /// Maximum allowed mapping depth
    pub const MAX_DEPTH: c_int = 1000;

    /// Create a new depth counter
    #[must_use]
    pub const fn new() -> Self {
        Self { depth: 0 }
    }

    /// Increment depth, returns true if exceeded max
    pub const fn increment(&mut self) -> bool {
        self.depth += 1;
        self.depth > Self::MAX_DEPTH
    }

    /// Decrement depth
    pub const fn decrement(&mut self) {
        if self.depth > 0 {
            self.depth -= 1;
        }
    }

    /// Get current depth
    #[must_use]
    pub const fn get(&self) -> c_int {
        self.depth
    }

    /// Reset to zero
    pub const fn reset(&mut self) {
        self.depth = 0;
    }
}

// =============================================================================
// C FFI Accessor Functions
// =============================================================================

#[allow(dead_code)]
extern "C" {
    /// Get the no_mapping global variable
    fn nvim_get_no_mapping() -> c_int;
    /// Set the no_mapping global variable
    fn nvim_set_no_mapping(val: c_int);
    /// Get the allow_keys global variable
    fn nvim_get_allow_keys() -> c_int;
    /// Get the KeyNoremap global variable
    fn nvim_get_keynoremap() -> c_int;
    /// Set the KeyNoremap global variable
    fn nvim_set_keynoremap(val: c_int);
    /// Get the KeyTyped global variable
    fn nvim_get_keytyped() -> c_int;
    /// Set the KeyTyped global variable
    fn nvim_set_keytyped(val: c_int);
    /// Get the KeyStuffed global variable
    fn nvim_get_keystuffed() -> c_int;
    /// Set the KeyStuffed global variable
    fn nvim_set_keystuffed(val: c_int);
    /// Get the vgetc_busy counter
    fn nvim_get_vgetc_busy() -> c_int;
    /// Increment vgetc_busy counter
    fn nvim_inc_vgetc_busy();
    /// Decrement vgetc_busy counter
    fn nvim_dec_vgetc_busy();
    /// Get the ex_normal_busy counter
    fn nvim_get_ex_normal_busy() -> c_int;
    /// Get the maptick counter
    fn nvim_get_maptick() -> c_int;
    /// Increment maptick counter
    fn nvim_inc_maptick();
}

/// Check if key mapping is disabled.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_no_mapping() -> c_int {
    nvim_get_no_mapping()
}

/// Set the no_mapping flag.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_no_mapping(val: c_int) {
    nvim_set_no_mapping(val);
}

/// Check if special keys are allowed.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_allow_keys() -> c_int {
    nvim_get_allow_keys()
}

/// Get the current key noremap value.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_get_keynoremap() -> c_int {
    nvim_get_keynoremap()
}

/// Set the key noremap value.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_keynoremap(val: c_int) {
    nvim_set_keynoremap(val);
}

/// Check if the key was typed by user.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_key_typed() -> c_int {
    nvim_get_keytyped()
}

/// Set the key typed flag.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_key_typed(val: c_int) {
    nvim_set_keytyped(val);
}

/// Check if the key was stuffed (from mapping or script).
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_key_stuffed() -> c_int {
    nvim_get_keystuffed()
}

/// Set the key stuffed flag.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_key_stuffed(val: c_int) {
    nvim_set_keystuffed(val);
}

/// Check if we are busy getting a character (in vgetc).
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_vgetc_busy() -> c_int {
    nvim_get_vgetc_busy()
}

/// Increment the vgetc busy counter.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_inc_vgetc_busy() {
    nvim_inc_vgetc_busy();
}

/// Decrement the vgetc busy counter.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_dec_vgetc_busy() {
    nvim_dec_vgetc_busy();
}

/// Check if :normal command is being executed.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_normal_busy() -> c_int {
    nvim_get_ex_normal_busy()
}

/// Get the mapping tick counter.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_get_maptick() -> c_int {
    nvim_get_maptick()
}

/// Increment the mapping tick counter.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_inc_maptick() {
    nvim_inc_maptick();
}

/// Check if in recursive vgetc call (not safe to get user input).
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_vgetc_recursive() -> c_int {
    c_int::from(nvim_get_vgetc_busy() > 0 && nvim_get_ex_normal_busy() == 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_result_from() {
        assert_eq!(MapResult::from(0), MapResult::Fail);
        assert_eq!(MapResult::from(1), MapResult::Get);
        assert_eq!(MapResult::from(2), MapResult::Retry);
        assert_eq!(MapResult::from(3), MapResult::NoMatch);
        assert_eq!(MapResult::from(99), MapResult::NoMatch);
    }

    #[test]
    fn test_mapping_timeout() {
        let mut timeout = MappingTimeout::new();
        assert!(!timeout.is_timedout());

        timeout.mapping_timedout = true;
        assert!(timeout.is_timedout());

        timeout.reset();
        assert!(!timeout.is_timedout());
    }

    #[test]
    fn test_mapping_depth() {
        let mut depth = MappingDepth::new();
        assert_eq!(depth.get(), 0);

        for _ in 0..1000 {
            assert!(!depth.increment());
        }
        assert!(depth.increment()); // 1001 exceeds max

        depth.reset();
        assert_eq!(depth.get(), 0);
    }
}
