//! Argument list management
//!
//! This crate provides Rust implementations for managing the argument list,
//! including file navigation, editing, and global/local argument lists.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

pub mod commands;
pub mod core;
pub mod entry;
pub mod ffi;
pub mod manipulation;
pub mod navigation;
pub mod operations;
pub mod parsing;
pub mod query;
pub mod winmgmt;

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

// FFI constants (verified against C headers with _Static_assert)
pub const AL_SET: c_int = 1;
pub const AL_ADD: c_int = 2;
pub const AL_DEL: c_int = 3;

pub const OK: c_int = 1;
pub const FAIL: c_int = 0;
pub const NUL: u8 = 0;

pub const BLN_CURBUF: c_int = 1;
pub const BLN_LISTED: c_int = 2;

pub const EW_DIR: c_int = 0x01;
pub const EW_FILE: c_int = 0x02;
pub const EW_NOTFOUND: c_int = 0x04;
pub const EW_ADDSLASH: c_int = 0x08;
pub const EW_NOERROR: c_int = 0x200;
pub const EW_NOTWILD: c_int = 0x400;

pub const RE_MAGIC: c_int = 1;

pub const K_EQUAL_FILES: c_int = 1;

pub const CCGD_AW: c_int = 1;
pub const CCGD_MULTWIN: c_int = 2;
pub const CCGD_FORCEIT: c_int = 4;
pub const CCGD_EXCMD: c_int = 16;

pub const ECMD_LAST: i32 = -1;
pub const ECMD_HIDE: c_int = 0x01;
pub const ECMD_OLDBUF: c_int = 0x04;
pub const ECMD_FORCEIT: c_int = 0x08;
pub const ECMD_ONE: i32 = 1;

pub const CMD_ARGS: c_int = 7;
pub const CMD_ARGGLOBAL: c_int = 13;
pub const CMD_ARGLOCAL: c_int = 14;
pub const CMD_ARGDO: c_int = 10;
pub const CMD_SNEXT: c_int = 413;
pub const CMD_DROP: c_int = 130;

pub const WSP_ROOM: c_int = 0x01;
pub const WSP_BELOW: c_int = 0x40;

pub const VAR_UNKNOWN: c_int = 0;
pub const VAR_NUMBER: c_int = 1;
pub const VAR_STRING: c_int = 2;

pub const ML_EMPTY: c_int = 0x01;

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
