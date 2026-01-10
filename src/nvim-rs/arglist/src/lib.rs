//! Argument list management
//!
//! This crate provides Rust implementations for managing the argument list,
//! including file navigation, editing, and global/local argument lists.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

pub mod entry;
pub mod navigation;
pub mod operations;

use std::ffi::c_int;

// Re-export key types
pub use entry::{ArgEntry, ArgEntryFlags};
pub use navigation::{ArgNavigation, ArgPosition};
pub use operations::{ArgOperation, ArgOperationResult};

// =============================================================================
// Constants
// =============================================================================

/// Maximum number of arguments in the list
pub const MAX_ARGS: usize = 10000;

/// Invalid argument index
pub const INVALID_ARG_IDX: c_int = -1;

// =============================================================================
// Argument List Type
// =============================================================================

/// Type of argument list
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArgListType {
    /// Global argument list
    #[default]
    Global = 0,
    /// Window-local argument list
    Local = 1,
}

impl ArgListType {
    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Global),
            1 => Some(Self::Local),
            _ => None,
        }
    }

    /// Convert to raw value
    pub const fn as_raw(self) -> c_int {
        self as c_int
    }

    /// Check if this is global
    pub const fn is_global(self) -> bool {
        matches!(self, Self::Global)
    }

    /// Check if this is local
    pub const fn is_local(self) -> bool {
        matches!(self, Self::Local)
    }
}

// =============================================================================
// Argument List State
// =============================================================================

/// Current state of argument list
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ArgListState {
    /// Number of arguments
    pub count: c_int,
    /// Current argument index (0-based)
    pub current: c_int,
    /// Type of list
    pub list_type: ArgListType,
    /// Whether the list has been modified
    pub modified: bool,
}

impl Default for ArgListState {
    fn default() -> Self {
        Self {
            count: 0,
            current: INVALID_ARG_IDX,
            list_type: ArgListType::Global,
            modified: false,
        }
    }
}

impl ArgListState {
    /// Create a new empty state
    pub const fn empty() -> Self {
        Self {
            count: 0,
            current: INVALID_ARG_IDX,
            list_type: ArgListType::Global,
            modified: false,
        }
    }

    /// Create with count
    pub const fn with_count(count: c_int) -> Self {
        Self {
            count,
            current: if count > 0 { 0 } else { INVALID_ARG_IDX },
            list_type: ArgListType::Global,
            modified: false,
        }
    }

    /// Check if the list is empty
    pub const fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Check if current index is valid
    pub const fn has_current(&self) -> bool {
        self.current >= 0 && self.current < self.count
    }

    /// Check if we're at the first argument
    pub const fn is_at_first(&self) -> bool {
        self.current == 0
    }

    /// Check if we're at the last argument
    pub const fn is_at_last(&self) -> bool {
        self.current == self.count - 1
    }

    /// Get next valid index (or -1)
    pub const fn next_index(&self) -> c_int {
        if self.current < self.count - 1 {
            self.current + 1
        } else {
            INVALID_ARG_IDX
        }
    }

    /// Get previous valid index (or -1)
    pub const fn prev_index(&self) -> c_int {
        if self.current > 0 {
            self.current - 1
        } else {
            INVALID_ARG_IDX
        }
    }
}

// =============================================================================
// Argument List Error
// =============================================================================

/// Error types for argument list operations
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArgListError {
    /// No error
    #[default]
    None = 0,
    /// List is empty
    Empty = 1,
    /// Index out of range
    OutOfRange = 2,
    /// File not found
    FileNotFound = 3,
    /// Cannot add file
    CannotAdd = 4,
    /// Cannot remove file
    CannotRemove = 5,
    /// Already at first
    AtFirst = 6,
    /// Already at last
    AtLast = 7,
    /// Invalid argument
    InvalidArg = 8,
}

impl ArgListError {
    /// Check if this is success
    pub const fn is_success(self) -> bool {
        matches!(self, Self::None)
    }

    /// Check if this is an error
    pub const fn is_error(self) -> bool {
        !self.is_success()
    }

    /// Create from raw value
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::Empty),
            2 => Some(Self::OutOfRange),
            3 => Some(Self::FileNotFound),
            4 => Some(Self::CannotAdd),
            5 => Some(Self::CannotRemove),
            6 => Some(Self::AtFirst),
            7 => Some(Self::AtLast),
            8 => Some(Self::InvalidArg),
            _ => None,
        }
    }
}


// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Check if list type is valid
#[no_mangle]
pub extern "C" fn rs_arglist_type_valid(list_type: c_int) -> c_int {
    c_int::from(ArgListType::from_raw(list_type).is_some())
}

/// FFI export: Check if list type is global
#[no_mangle]
pub extern "C" fn rs_arglist_type_is_global(list_type: c_int) -> c_int {
    ArgListType::from_raw(list_type).map_or(0, |t| c_int::from(t.is_global()))
}

/// FFI export: Create empty state
#[no_mangle]
pub extern "C" fn rs_arglist_state_empty() -> ArgListState {
    ArgListState::empty()
}

/// FFI export: Create state with count
#[no_mangle]
pub extern "C" fn rs_arglist_state_with_count(count: c_int) -> ArgListState {
    ArgListState::with_count(count)
}

/// FFI export: Check if state is empty
#[no_mangle]
pub extern "C" fn rs_arglist_state_is_empty(state: *const ArgListState) -> c_int {
    if state.is_null() {
        return 1;
    }
    c_int::from(unsafe { (*state).is_empty() })
}

/// FFI export: Check if state has current
#[no_mangle]
pub extern "C" fn rs_arglist_state_has_current(state: *const ArgListState) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*state).has_current() })
}

/// FFI export: Get next index from state
#[no_mangle]
pub extern "C" fn rs_arglist_state_next_index(state: *const ArgListState) -> c_int {
    if state.is_null() {
        return INVALID_ARG_IDX;
    }
    unsafe { (*state).next_index() }
}

/// FFI export: Get prev index from state
#[no_mangle]
pub extern "C" fn rs_arglist_state_prev_index(state: *const ArgListState) -> c_int {
    if state.is_null() {
        return INVALID_ARG_IDX;
    }
    unsafe { (*state).prev_index() }
}

/// FFI export: Check if error is success
#[no_mangle]
pub extern "C" fn rs_arglist_error_is_success(error: c_int) -> c_int {
    ArgListError::from_raw(error).map_or(0, |e| c_int::from(e.is_success()))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(clippy::borrow_as_ptr)]
mod tests {
    use super::*;

    #[test]
    fn test_arglist_type() {
        assert_eq!(ArgListType::from_raw(0), Some(ArgListType::Global));
        assert_eq!(ArgListType::from_raw(1), Some(ArgListType::Local));
        assert_eq!(ArgListType::from_raw(100), None);

        assert!(ArgListType::Global.is_global());
        assert!(!ArgListType::Global.is_local());
        assert!(ArgListType::Local.is_local());
    }

    #[test]
    fn test_arglist_state() {
        let empty = ArgListState::empty();
        assert!(empty.is_empty());
        assert!(!empty.has_current());

        let state = ArgListState::with_count(5);
        assert!(!state.is_empty());
        assert!(state.has_current());
        assert!(state.is_at_first());
        assert!(!state.is_at_last());
        assert_eq!(state.next_index(), 1);
        assert_eq!(state.prev_index(), INVALID_ARG_IDX);

        let at_last = ArgListState {
            count: 5,
            current: 4,
            ..Default::default()
        };
        assert!(at_last.is_at_last());
        assert_eq!(at_last.next_index(), INVALID_ARG_IDX);
        assert_eq!(at_last.prev_index(), 3);
    }

    #[test]
    fn test_arglist_error() {
        assert!(ArgListError::None.is_success());
        assert!(!ArgListError::Empty.is_success());
        assert!(ArgListError::OutOfRange.is_error());
    }

    #[test]
    fn test_ffi_exports() {
        assert_eq!(rs_arglist_type_valid(0), 1);
        assert_eq!(rs_arglist_type_valid(100), 0);

        assert_eq!(rs_arglist_type_is_global(0), 1);
        assert_eq!(rs_arglist_type_is_global(1), 0);

        let empty = rs_arglist_state_empty();
        assert_eq!(rs_arglist_state_is_empty(&empty), 1);
    }
}
