//! Autowrite functionality
//!
//! This module provides utilities for autowriting buffers.

use std::ffi::c_int;

// =============================================================================
// Autowrite Mode
// =============================================================================

/// Autowrite modes
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutowriteMode {
    /// No autowrite
    Off = 0,
    /// Autowrite on buffer change
    On = 1,
    /// Autowrite all buffers
    All = 2,
}

impl AutowriteMode {
    /// Create from integer value
    pub fn from_int(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Off),
            1 => Some(Self::On),
            2 => Some(Self::All),
            _ => None,
        }
    }

    /// Check if any autowrite is enabled
    pub fn is_enabled(self) -> bool {
        self != Self::Off
    }
}

/// Check if autowrite mode is enabled
#[no_mangle]
pub extern "C" fn rs_autowrite_enabled(mode: c_int) -> bool {
    AutowriteMode::from_int(mode).is_some_and(|m| m.is_enabled())
}

/// Check if autowrite mode is "all"
#[no_mangle]
pub extern "C" fn rs_autowrite_is_all(mode: c_int) -> bool {
    AutowriteMode::from_int(mode) == Some(AutowriteMode::All)
}

// =============================================================================
// Buffer Change Reasons
// =============================================================================

/// Reasons for buffer abandonment
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbandonReason {
    /// Buffer is hidden
    Hidden = 0,
    /// Buffer is not changed
    NotChanged = 1,
    /// Buffer has multiple windows
    MultipleWindows = 2,
    /// Autowrite succeeded
    AutowriteOk = 3,
    /// Force flag was set
    Force = 4,
}

/// Get abandon reason description
#[no_mangle]
pub extern "C" fn rs_abandon_reason_str(reason: c_int) -> *const std::ffi::c_char {
    static HIDDEN: &[u8] = b"hidden\0";
    static NOT_CHANGED: &[u8] = b"not changed\0";
    static MULTI_WIN: &[u8] = b"multiple windows\0";
    static AUTOWRITE_OK: &[u8] = b"autowrite succeeded\0";
    static FORCE: &[u8] = b"force\0";
    static UNKNOWN: &[u8] = b"unknown\0";

    let s = match reason {
        0 => HIDDEN,
        1 => NOT_CHANGED,
        2 => MULTI_WIN,
        3 => AUTOWRITE_OK,
        4 => FORCE,
        _ => UNKNOWN,
    };
    s.as_ptr().cast()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autowrite_mode() {
        assert!(!rs_autowrite_enabled(0));
        assert!(rs_autowrite_enabled(1));
        assert!(rs_autowrite_enabled(2));
        assert!(!rs_autowrite_enabled(99));
    }

    #[test]
    fn test_autowrite_is_all() {
        assert!(!rs_autowrite_is_all(0));
        assert!(!rs_autowrite_is_all(1));
        assert!(rs_autowrite_is_all(2));
    }

    #[test]
    fn test_abandon_reason() {
        let s = rs_abandon_reason_str(0);
        assert!(!s.is_null());
        let s = rs_abandon_reason_str(4);
        assert!(!s.is_null());
        let s = rs_abandon_reason_str(99);
        assert!(!s.is_null());
    }
}
