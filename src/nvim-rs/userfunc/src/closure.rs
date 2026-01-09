//! Closure and captured variable handling.

#![allow(clippy::must_use_candidate)]

use std::ffi::c_int;

// =============================================================================
// Closure Capture Types
// =============================================================================

/// How a variable is captured in a closure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum CaptureMode {
    /// Variable is captured by value (copy)
    Value = 0,
    /// Variable is captured by reference
    Reference = 1,
}

impl CaptureMode {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Reference,
            _ => Self::Value,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Closure Info
// =============================================================================

/// Information about a closure's captured variables.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ClosureInfo {
    /// Number of captured variables
    pub capture_count: i32,
    /// Script ID where closure was defined
    pub script_id: i32,
    /// Whether closure is still valid (outer scope alive)
    pub is_valid: bool,
}

impl ClosureInfo {
    /// Create new closure info.
    pub const fn new(capture_count: i32, script_id: i32) -> Self {
        Self {
            capture_count,
            script_id,
            is_valid: true,
        }
    }

    /// Check if closure has captures.
    pub const fn has_captures(&self) -> bool {
        self.capture_count > 0
    }
}

// =============================================================================
// Captured Variable Info
// =============================================================================

/// Information about a single captured variable.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CapturedVar {
    /// Capture mode (by value or reference)
    pub mode: i32,
    /// Variable slot index in outer scope
    pub outer_idx: i32,
    /// Variable slot index in closure
    pub closure_idx: i32,
}

impl Default for CapturedVar {
    fn default() -> Self {
        Self {
            mode: CaptureMode::Value as i32,
            outer_idx: -1,
            closure_idx: -1,
        }
    }
}

impl CapturedVar {
    /// Create a value-captured variable.
    pub const fn by_value(outer_idx: i32, closure_idx: i32) -> Self {
        Self {
            mode: CaptureMode::Value as i32,
            outer_idx,
            closure_idx,
        }
    }

    /// Create a reference-captured variable.
    pub const fn by_ref(outer_idx: i32, closure_idx: i32) -> Self {
        Self {
            mode: CaptureMode::Reference as i32,
            outer_idx,
            closure_idx,
        }
    }

    /// Check if captured by reference.
    pub const fn is_by_ref(&self) -> bool {
        self.mode == CaptureMode::Reference as i32
    }
}

// =============================================================================
// Scope Chain
// =============================================================================

/// Scope level for closure capture resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ScopeLevel {
    /// Local function scope
    Local = 0,
    /// Enclosing function scope
    Enclosing = 1,
    /// Script scope
    Script = 2,
    /// Global scope
    Global = 3,
}

impl ScopeLevel {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Local,
            1 => Self::Enclosing,
            2 => Self::Script,
            _ => Self::Global,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

/// FFI export: create closure info.
#[no_mangle]
pub extern "C" fn rs_closure_info_new(capture_count: c_int, script_id: c_int) -> ClosureInfo {
    ClosureInfo::new(capture_count, script_id)
}

/// FFI export: check if closure has captures.
#[no_mangle]
pub extern "C" fn rs_closure_has_captures(capture_count: c_int) -> bool {
    capture_count > 0
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_mode() {
        assert_eq!(CaptureMode::from_c_int(0), CaptureMode::Value);
        assert_eq!(CaptureMode::from_c_int(1), CaptureMode::Reference);
    }

    #[test]
    fn test_closure_info() {
        let info = ClosureInfo::new(3, 1);
        assert!(info.has_captures());
        assert!(info.is_valid);

        let empty = ClosureInfo::new(0, 1);
        assert!(!empty.has_captures());
    }

    #[test]
    fn test_captured_var() {
        let by_val = CapturedVar::by_value(0, 0);
        assert!(!by_val.is_by_ref());

        let by_ref = CapturedVar::by_ref(1, 1);
        assert!(by_ref.is_by_ref());
    }

    #[test]
    fn test_scope_level() {
        assert_eq!(ScopeLevel::from_c_int(0), ScopeLevel::Local);
        assert_eq!(ScopeLevel::from_c_int(3), ScopeLevel::Global);
    }
}
