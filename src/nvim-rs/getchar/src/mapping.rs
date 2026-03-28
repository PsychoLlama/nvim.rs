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

extern "C" {
    /// no_mapping: currently no mapping allowed
    static mut no_mapping: c_int;
    /// allow_keys: allow key codes when no_mapping is set
    static mut allow_keys: c_int;
    /// KeyNoremap: remapping flags (non-static in C after Phase 3)
    static KeyNoremap: c_int;
    /// Set the KeyNoremap global variable
    fn nvim_set_keynoremap(val: c_int);
    /// KeyTyped: true if user typed current char
    static mut KeyTyped: bool;
    /// KeyStuffed: true if current char from stuffbuf
    static mut KeyStuffed: c_int;
    /// vgetc_busy: counter for vgetc recursion
    static mut vgetc_busy: c_int;
    /// ex_normal_busy: recursiveness of ex_normal()
    static mut ex_normal_busy: c_int;
    /// maptick: tick for each non-mapped char
    static mut maptick: c_int;
}

/// Check if key mapping is disabled.
///
/// # Safety
/// Reads C global directly.
#[no_mangle]
pub unsafe extern "C" fn rs_no_mapping() -> c_int {
    no_mapping
}

/// Set the no_mapping flag.
///
/// # Safety
/// Writes C global directly.
#[no_mangle]
pub unsafe extern "C" fn rs_set_no_mapping(val: c_int) {
    no_mapping = val;
}

/// Check if special keys are allowed.
///
/// # Safety
/// Reads C global directly.
#[no_mangle]
pub unsafe extern "C" fn rs_allow_keys() -> c_int {
    allow_keys
}

/// Get the current key noremap value.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_get_keynoremap() -> c_int {
    KeyNoremap
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
/// Reads C global directly.
#[no_mangle]
pub unsafe extern "C" fn rs_key_typed() -> c_int {
    c_int::from(KeyTyped)
}

/// Set the key typed flag.
///
/// # Safety
/// Writes C global directly.
#[no_mangle]
pub unsafe extern "C" fn rs_set_key_typed(val: c_int) {
    KeyTyped = val != 0;
}

/// Check if the key was stuffed (from mapping or script).
///
/// # Safety
/// Reads C global directly.
#[no_mangle]
pub unsafe extern "C" fn rs_key_stuffed() -> c_int {
    KeyStuffed
}

/// Set the key stuffed flag.
///
/// # Safety
/// Writes C global directly.
#[no_mangle]
pub unsafe extern "C" fn rs_set_key_stuffed(val: c_int) {
    KeyStuffed = val;
}

/// Check if we are busy getting a character (in vgetc).
///
/// # Safety
/// Reads C global directly.
#[no_mangle]
pub unsafe extern "C" fn rs_vgetc_busy() -> c_int {
    vgetc_busy
}

/// Increment the vgetc busy counter.
///
/// # Safety
/// Writes C global directly.
#[no_mangle]
pub unsafe extern "C" fn rs_inc_vgetc_busy() {
    vgetc_busy += 1;
}

/// Decrement the vgetc busy counter.
///
/// # Safety
/// Writes C global directly.
#[no_mangle]
pub unsafe extern "C" fn rs_dec_vgetc_busy() {
    if vgetc_busy > 0 {
        vgetc_busy -= 1;
    }
}

/// Check if :normal command is being executed.
///
/// # Safety
/// Reads C global directly.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_normal_busy() -> c_int {
    ex_normal_busy
}

/// Get the mapping tick counter.
///
/// # Safety
/// Reads C global directly.
#[no_mangle]
pub unsafe extern "C" fn rs_get_maptick() -> c_int {
    maptick
}

/// Increment the mapping tick counter.
///
/// # Safety
/// Writes C global directly.
#[no_mangle]
pub unsafe extern "C" fn rs_inc_maptick() {
    maptick += 1;
}

/// Check if in recursive vgetc call (not safe to get user input).
///
/// # Safety
/// Reads C globals directly.
#[no_mangle]
pub unsafe extern "C" fn rs_vgetc_recursive() -> c_int {
    c_int::from(vgetc_busy > 0 && ex_normal_busy == 0)
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
