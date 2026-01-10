//! Argument list operations
//!
//! This module provides types for argument list modification operations
//! like add, delete, and reorder.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::map_unwrap_or)]

use std::ffi::c_int;

// =============================================================================
// Operation Type
// =============================================================================

/// Type of argument list operation
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgOperation {
    /// Add files to list
    Add = 0,
    /// Delete files from list
    Delete = 1,
    /// Replace entire list
    Replace = 2,
    /// Move file within list
    Move = 3,
    /// Clear entire list
    Clear = 4,
    /// Edit current argument
    Edit = 5,
    /// Rewind to first argument
    Rewind = 6,
    /// Write all arguments
    WriteAll = 7,
}

impl ArgOperation {
    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Add),
            1 => Some(Self::Delete),
            2 => Some(Self::Replace),
            3 => Some(Self::Move),
            4 => Some(Self::Clear),
            5 => Some(Self::Edit),
            6 => Some(Self::Rewind),
            7 => Some(Self::WriteAll),
            _ => None,
        }
    }

    /// Convert to raw value
    pub const fn as_raw(self) -> c_int {
        self as c_int
    }

    /// Check if this modifies the list
    pub const fn modifies_list(self) -> bool {
        matches!(
            self,
            Self::Add | Self::Delete | Self::Replace | Self::Move | Self::Clear
        )
    }

    /// Check if this requires a file argument
    pub const fn requires_file(self) -> bool {
        matches!(self, Self::Add | Self::Delete | Self::Move)
    }

    /// Get command name for this operation
    pub const fn command_name(self) -> &'static str {
        match self {
            Self::Add => "argadd",
            Self::Delete => "argdelete",
            Self::Replace => "args",
            Self::Move => "argmove",
            Self::Clear => "argclear",
            Self::Edit => "argedit",
            Self::Rewind => "rewind",
            Self::WriteAll => "argdo",
        }
    }
}

impl Default for ArgOperation {
    fn default() -> Self {
        Self::Edit
    }
}

// =============================================================================
// Operation Flags
// =============================================================================

/// Flags for argument operations
pub const OP_BANG: u32 = 0x0001;
pub const OP_GLOBAL: u32 = 0x0002;
pub const OP_LOCAL: u32 = 0x0004;
pub const OP_CONFIRM: u32 = 0x0008;
pub const OP_SILENT: u32 = 0x0010;
pub const OP_KEEPALT: u32 = 0x0020;

/// Operation flags wrapper
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ArgOperationFlags {
    flags: u32,
}

impl ArgOperationFlags {
    /// Create with no flags
    pub const fn none() -> Self {
        Self { flags: 0 }
    }

    /// Create from raw value
    pub const fn from_raw(flags: u32) -> Self {
        Self { flags }
    }

    /// Get raw value
    pub const fn as_raw(self) -> u32 {
        self.flags
    }

    /// Check if bang was used
    pub const fn has_bang(self) -> bool {
        (self.flags & OP_BANG) != 0
    }

    /// Check if global
    pub const fn is_global(self) -> bool {
        (self.flags & OP_GLOBAL) != 0
    }

    /// Check if local
    pub const fn is_local(self) -> bool {
        (self.flags & OP_LOCAL) != 0
    }

    /// Check if confirm
    pub const fn is_confirm(self) -> bool {
        (self.flags & OP_CONFIRM) != 0
    }

    /// Check if silent
    pub const fn is_silent(self) -> bool {
        (self.flags & OP_SILENT) != 0
    }
}

// =============================================================================
// Operation Request
// =============================================================================

/// Request for an argument list operation
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ArgOperationRequest {
    /// Operation type
    pub operation: ArgOperation,
    /// Flags
    pub flags: ArgOperationFlags,
    /// Start index (for range operations)
    pub start: c_int,
    /// End index (for range operations)
    pub end: c_int,
    /// Count (for some operations)
    pub count: c_int,
}

impl Default for ArgOperationRequest {
    fn default() -> Self {
        Self {
            operation: ArgOperation::Edit,
            flags: ArgOperationFlags::none(),
            start: 0,
            end: 0,
            count: 1,
        }
    }
}

impl ArgOperationRequest {
    /// Create an add request
    pub const fn add() -> Self {
        Self {
            operation: ArgOperation::Add,
            flags: ArgOperationFlags::none(),
            start: 0,
            end: 0,
            count: 1,
        }
    }

    /// Create a delete request for a range
    pub const fn delete_range(start: c_int, end: c_int) -> Self {
        Self {
            operation: ArgOperation::Delete,
            flags: ArgOperationFlags::none(),
            start,
            end,
            count: 1,
        }
    }

    /// Create a replace request
    pub const fn replace() -> Self {
        Self {
            operation: ArgOperation::Replace,
            flags: ArgOperationFlags::none(),
            start: 0,
            end: 0,
            count: 1,
        }
    }

    /// Create a clear request
    pub const fn clear() -> Self {
        Self {
            operation: ArgOperation::Clear,
            flags: ArgOperationFlags::none(),
            start: 0,
            end: 0,
            count: 0,
        }
    }

    /// Check if this is a range operation
    pub const fn is_range(&self) -> bool {
        self.end > self.start
    }

    /// Get range count
    pub const fn range_count(&self) -> c_int {
        if self.is_range() {
            self.end - self.start + 1
        } else {
            1
        }
    }
}

// =============================================================================
// Operation Result
// =============================================================================

/// Result of an argument list operation
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ArgOperationResult {
    /// Whether operation succeeded
    pub success: bool,
    /// Number of items affected
    pub affected: c_int,
    /// New current index
    pub new_current: c_int,
    /// New total count
    pub new_count: c_int,
    /// Error code (0 = none)
    pub error: c_int,
}

impl Default for ArgOperationResult {
    fn default() -> Self {
        Self {
            success: false,
            affected: 0,
            new_current: -1,
            new_count: 0,
            error: 0,
        }
    }
}

impl ArgOperationResult {
    /// Create a success result
    pub const fn success(affected: c_int, new_current: c_int, new_count: c_int) -> Self {
        Self {
            success: true,
            affected,
            new_current,
            new_count,
            error: 0,
        }
    }

    /// Create a failure result
    pub const fn failure(error: c_int) -> Self {
        Self {
            success: false,
            affected: 0,
            new_current: -1,
            new_count: 0,
            error,
        }
    }

    /// Check if list changed
    pub const fn list_changed(&self) -> bool {
        self.success && self.affected > 0
    }
}

// =============================================================================
// Add Position
// =============================================================================

/// Position for adding to argument list
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArgAddPosition {
    /// Add at end
    #[default]
    End = 0,
    /// Add at beginning
    Beginning = 1,
    /// Add before current
    BeforeCurrent = 2,
    /// Add after current
    AfterCurrent = 3,
    /// Add at specific index
    AtIndex = 4,
}

impl ArgAddPosition {
    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::End),
            1 => Some(Self::Beginning),
            2 => Some(Self::BeforeCurrent),
            3 => Some(Self::AfterCurrent),
            4 => Some(Self::AtIndex),
            _ => None,
        }
    }

    /// Calculate actual index for insertion
    pub fn calculate_index(self, current: c_int, count: c_int, target: c_int) -> c_int {
        match self {
            Self::End => count,
            Self::Beginning => 0,
            Self::BeforeCurrent => current.max(0),
            Self::AfterCurrent => (current + 1).min(count),
            Self::AtIndex => target.clamp(0, count),
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Check if operation is valid
#[no_mangle]
pub extern "C" fn rs_argop_valid(operation: c_int) -> c_int {
    c_int::from(ArgOperation::from_raw(operation).is_some())
}

/// FFI export: Check if operation modifies list
#[no_mangle]
pub extern "C" fn rs_argop_modifies_list(operation: c_int) -> c_int {
    ArgOperation::from_raw(operation).map_or(0, |op| c_int::from(op.modifies_list()))
}

/// FFI export: Check if operation requires file
#[no_mangle]
pub extern "C" fn rs_argop_requires_file(operation: c_int) -> c_int {
    ArgOperation::from_raw(operation).map_or(0, |op| c_int::from(op.requires_file()))
}

/// FFI export: Check if flags has bang
#[no_mangle]
pub extern "C" fn rs_argop_flags_has_bang(flags: u32) -> c_int {
    c_int::from(ArgOperationFlags::from_raw(flags).has_bang())
}

/// FFI export: Create add request
#[no_mangle]
pub extern "C" fn rs_argop_request_add() -> ArgOperationRequest {
    ArgOperationRequest::add()
}

/// FFI export: Create delete range request
#[no_mangle]
pub extern "C" fn rs_argop_request_delete_range(start: c_int, end: c_int) -> ArgOperationRequest {
    ArgOperationRequest::delete_range(start, end)
}

/// FFI export: Create clear request
#[no_mangle]
pub extern "C" fn rs_argop_request_clear() -> ArgOperationRequest {
    ArgOperationRequest::clear()
}

/// FFI export: Create success result
#[no_mangle]
pub extern "C" fn rs_argop_result_success(
    affected: c_int,
    new_current: c_int,
    new_count: c_int,
) -> ArgOperationResult {
    ArgOperationResult::success(affected, new_current, new_count)
}

/// FFI export: Create failure result
#[no_mangle]
pub extern "C" fn rs_argop_result_failure(error: c_int) -> ArgOperationResult {
    ArgOperationResult::failure(error)
}

/// FFI export: Calculate add index
#[no_mangle]
pub extern "C" fn rs_argop_add_index(
    position: c_int,
    current: c_int,
    count: c_int,
    target: c_int,
) -> c_int {
    ArgAddPosition::from_raw(position)
        .map(|pos| pos.calculate_index(current, count, target))
        .unwrap_or(count)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arg_operation() {
        assert_eq!(ArgOperation::from_raw(0), Some(ArgOperation::Add));
        assert_eq!(ArgOperation::from_raw(100), None);

        assert!(ArgOperation::Add.modifies_list());
        assert!(!ArgOperation::Edit.modifies_list());

        assert!(ArgOperation::Add.requires_file());
        assert!(!ArgOperation::Clear.requires_file());
    }

    #[test]
    fn test_operation_flags() {
        let flags = ArgOperationFlags::none();
        assert!(!flags.has_bang());

        let flags = ArgOperationFlags::from_raw(OP_BANG | OP_SILENT);
        assert!(flags.has_bang());
        assert!(flags.is_silent());
        assert!(!flags.is_global());
    }

    #[test]
    fn test_operation_request() {
        let add = ArgOperationRequest::add();
        assert_eq!(add.operation, ArgOperation::Add);

        let delete = ArgOperationRequest::delete_range(2, 5);
        assert!(delete.is_range());
        assert_eq!(delete.range_count(), 4);
    }

    #[test]
    fn test_operation_result() {
        let success = ArgOperationResult::success(3, 2, 10);
        assert!(success.success);
        assert!(success.list_changed());

        let failure = ArgOperationResult::failure(1);
        assert!(!failure.success);
        assert!(!failure.list_changed());
    }

    #[test]
    fn test_add_position() {
        assert_eq!(ArgAddPosition::End.calculate_index(2, 5, 0), 5);
        assert_eq!(ArgAddPosition::Beginning.calculate_index(2, 5, 0), 0);
        assert_eq!(ArgAddPosition::BeforeCurrent.calculate_index(2, 5, 0), 2);
        assert_eq!(ArgAddPosition::AfterCurrent.calculate_index(2, 5, 0), 3);
        assert_eq!(ArgAddPosition::AtIndex.calculate_index(2, 5, 3), 3);
    }

    #[test]
    fn test_ffi_exports() {
        assert_eq!(rs_argop_valid(0), 1);
        assert_eq!(rs_argop_valid(100), 0);

        assert_eq!(rs_argop_modifies_list(0), 1); // Add
        assert_eq!(rs_argop_modifies_list(5), 0); // Edit

        assert_eq!(rs_argop_flags_has_bang(OP_BANG), 1);
        assert_eq!(rs_argop_flags_has_bang(0), 0);
    }
}
