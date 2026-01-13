//! Normal mode command dispatch types.
//!
//! This module provides type definitions for normal mode command processing,
//! including handle types and enumerations for motion types and command states.

#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

// =============================================================================
// Handle Types
// =============================================================================

/// Opaque handle to a NormalState (NormalState*).
///
/// This is an opaque pointer type - Rust code should not attempt to
/// dereference or inspect the contents. All field access is done
/// through C accessor functions.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NormalStateHandle(*mut std::ffi::c_void);

impl NormalStateHandle {
    /// Create a null handle.
    #[inline]
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Create a new handle from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be a valid `NormalState*` or null.
    #[inline]
    pub const unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    #[must_use]
    pub const fn as_ptr(self) -> *mut std::ffi::c_void {
        self.0
    }

    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to command arguments (cmdarg_T*).
///
/// This is an opaque pointer type - Rust code should not attempt to
/// dereference or inspect the contents. All field access is done
/// through C accessor functions.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CmdArgHandle(*mut std::ffi::c_void);

impl CmdArgHandle {
    /// Create a null handle.
    #[inline]
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Create a new handle from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be a valid `cmdarg_T*` or null.
    #[inline]
    pub const unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    #[must_use]
    pub const fn as_ptr(self) -> *mut std::ffi::c_void {
        self.0
    }

    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to operator arguments (oparg_T*).
///
/// This is an opaque pointer type - Rust code should not attempt to
/// dereference or inspect the contents. All field access is done
/// through C accessor functions.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpArgHandle(*mut std::ffi::c_void);

impl OpArgHandle {
    /// Create a null handle.
    #[inline]
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Create a new handle from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be a valid `oparg_T*` or null.
    #[inline]
    pub const unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    #[must_use]
    pub const fn as_ptr(self) -> *mut std::ffi::c_void {
        self.0
    }

    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// =============================================================================
// Motion Type Enumeration
// =============================================================================

/// Motion type for operator-pending mode.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionType {
    /// Character-wise motion (inclusive).
    Charwise = 0,
    /// Line-wise motion.
    Linewise = 1,
    /// Block-wise motion.
    Blockwise = 2,
    /// Unknown motion type.
    Unknown = -1,
}

impl MotionType {
    /// Convert from C int value.
    #[inline]
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Charwise,
            1 => Self::Linewise,
            2 => Self::Blockwise,
            _ => Self::Unknown,
        }
    }

    /// Convert to C int value.
    #[inline]
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if the motion type is character-wise.
    #[inline]
    #[must_use]
    pub const fn is_charwise(self) -> bool {
        matches!(self, Self::Charwise)
    }

    /// Check if the motion type is line-wise.
    #[inline]
    #[must_use]
    pub const fn is_linewise(self) -> bool {
        matches!(self, Self::Linewise)
    }

    /// Check if the motion type is block-wise.
    #[inline]
    #[must_use]
    pub const fn is_blockwise(self) -> bool {
        matches!(self, Self::Blockwise)
    }
}

// =============================================================================
// Command State
// =============================================================================

/// State of normal mode command execution.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmdState {
    /// Idle, waiting for command.
    Idle = 0,
    /// Waiting for operator count.
    WaitingCount = 1,
    /// Waiting for motion.
    WaitingMotion = 2,
    /// Waiting for second character.
    WaitingChar = 3,
    /// Executing command.
    Executing = 4,
    /// Command finished.
    Finished = 5,
}

impl CmdState {
    /// Convert from C int value.
    #[inline]
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Idle,
            1 => Self::WaitingCount,
            2 => Self::WaitingMotion,
            3 => Self::WaitingChar,
            4 => Self::Executing,
            _ => Self::Finished,
        }
    }

    /// Convert to C int value.
    #[inline]
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Check if NormalStateHandle is null.
#[unsafe(no_mangle)]
pub extern "C" fn rs_normal_state_is_null(handle: NormalStateHandle) -> c_int {
    c_int::from(handle.is_null())
}

/// FFI: Check if CmdArgHandle is null.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmdarg_is_null(handle: CmdArgHandle) -> c_int {
    c_int::from(handle.is_null())
}

/// FFI: Check if OpArgHandle is null.
#[unsafe(no_mangle)]
pub extern "C" fn rs_oparg_is_null(handle: OpArgHandle) -> c_int {
    c_int::from(handle.is_null())
}

/// FFI: Convert motion type value to charwise check.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_is_charwise(motion_type: c_int) -> c_int {
    c_int::from(MotionType::from_c_int(motion_type).is_charwise())
}

/// FFI: Convert motion type value to linewise check.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_is_linewise(motion_type: c_int) -> c_int {
    c_int::from(MotionType::from_c_int(motion_type).is_linewise())
}

/// FFI: Convert motion type value to blockwise check.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_is_blockwise(motion_type: c_int) -> c_int {
    c_int::from(MotionType::from_c_int(motion_type).is_blockwise())
}

/// FFI: Get charwise motion type constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_type_charwise() -> c_int {
    MotionType::Charwise.to_c_int()
}

/// FFI: Get linewise motion type constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_type_linewise() -> c_int {
    MotionType::Linewise.to_c_int()
}

/// FFI: Get blockwise motion type constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_type_blockwise() -> c_int {
    MotionType::Blockwise.to_c_int()
}

/// FFI: Get unknown motion type constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_motion_type_unknown() -> c_int {
    MotionType::Unknown.to_c_int()
}

/// FFI: Get idle command state constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmd_state_idle() -> c_int {
    CmdState::Idle.to_c_int()
}

/// FFI: Get waiting count command state constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmd_state_waiting_count() -> c_int {
    CmdState::WaitingCount.to_c_int()
}

/// FFI: Get waiting motion command state constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmd_state_waiting_motion() -> c_int {
    CmdState::WaitingMotion.to_c_int()
}

/// FFI: Get waiting char command state constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmd_state_waiting_char() -> c_int {
    CmdState::WaitingChar.to_c_int()
}

/// FFI: Get executing command state constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmd_state_executing() -> c_int {
    CmdState::Executing.to_c_int()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_state_handle_null() {
        let handle = NormalStateHandle::null();
        assert!(handle.is_null());
    }

    #[test]
    fn test_cmdarg_handle_null() {
        let handle = CmdArgHandle::null();
        assert!(handle.is_null());
    }

    #[test]
    fn test_oparg_handle_null() {
        let handle = OpArgHandle::null();
        assert!(handle.is_null());
    }

    #[test]
    fn test_motion_type_conversions() {
        assert_eq!(MotionType::from_c_int(0), MotionType::Charwise);
        assert_eq!(MotionType::from_c_int(1), MotionType::Linewise);
        assert_eq!(MotionType::from_c_int(2), MotionType::Blockwise);
        assert_eq!(MotionType::from_c_int(-1), MotionType::Unknown);
        assert_eq!(MotionType::from_c_int(99), MotionType::Unknown);
    }

    #[test]
    fn test_motion_type_checks() {
        assert!(MotionType::Charwise.is_charwise());
        assert!(!MotionType::Charwise.is_linewise());
        assert!(MotionType::Linewise.is_linewise());
        assert!(MotionType::Blockwise.is_blockwise());
    }

    #[test]
    fn test_cmd_state_conversions() {
        assert_eq!(CmdState::from_c_int(0), CmdState::Idle);
        assert_eq!(CmdState::from_c_int(1), CmdState::WaitingCount);
        assert_eq!(CmdState::from_c_int(4), CmdState::Executing);
    }
}
