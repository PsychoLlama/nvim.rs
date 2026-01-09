//! Quickfix API functions
//!
//! This module provides API function implementations for quickfix and
//! location list operations, matching `nvim_qf_*` function signatures.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::derivable_impls)]

use std::ffi::{c_int, c_void};

// =============================================================================
// API Constants
// =============================================================================

/// Quickfix list types for API
pub const QF_GLOBAL: c_int = 0;
pub const QF_LOCATION: c_int = 1;

/// Action codes for setqflist/setloclist
pub const QF_ACTION_REPLACE: u8 = b'r';
pub const QF_ACTION_APPEND: u8 = b'a';
pub const QF_ACTION_PREPEND: u8 = b'p';
pub const QF_ACTION_FREE: u8 = b'f';

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to quickfix info (`qf_info_T`)
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QfInfoHandle(*mut c_void);

impl QfInfoHandle {
    /// Check if the handle is null
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Create a null handle
    #[inline]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }
}

/// Opaque handle to window (`win_T`)
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WinHandle(*mut c_void);

impl WinHandle {
    /// Check if the handle is null
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Create a null handle
    #[inline]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }
}

// =============================================================================
// API Result Types
// =============================================================================

/// Result of a quickfix API operation
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QfApiResult {
    /// Operation succeeded
    Success = 0,
    /// Invalid list index
    InvalidList = 1,
    /// Invalid entry index
    InvalidEntry = 2,
    /// Invalid action
    InvalidAction = 3,
    /// Invalid what dictionary
    InvalidWhat = 4,
    /// List is empty
    EmptyList = 5,
    /// Window not found
    WindowNotFound = 6,
    /// Operation failed (generic)
    Failed = 7,
}

// =============================================================================
// API Action Parsing
// =============================================================================

/// Parse an action character for setqflist/setloclist
pub const fn parse_action(action: u8) -> Option<QfAction> {
    match action {
        b' ' | 0 | b'r' => Some(QfAction::Replace),
        b'a' => Some(QfAction::Append),
        b'f' => Some(QfAction::Free),
        _ => None,
    }
}

/// Quickfix list modification action
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QfAction {
    /// Replace the list entirely
    Replace = 0,
    /// Append to the list
    Append = 1,
    /// Free/delete the list
    Free = 2,
}

// =============================================================================
// List Information
// =============================================================================

/// Information returned by getqflist()
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct QfListInfo {
    /// Number of items in the list
    pub size: c_int,
    /// Current entry index (1-based)
    pub idx: c_int,
    /// List ID
    pub id: u32,
    /// Whether there are changed items
    pub changed: bool,
    /// Whether this is a location list
    pub is_loclist: bool,
}

impl Default for QfListInfo {
    fn default() -> Self {
        Self {
            size: 0,
            idx: 0,
            id: 0,
            changed: false,
            is_loclist: false,
        }
    }
}

impl QfListInfo {
    /// Check if the list is empty
    pub const fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Check if the current index is valid
    pub const fn has_valid_idx(&self) -> bool {
        self.idx > 0 && self.idx <= self.size
    }
}

// =============================================================================
// Entry What Fields
// =============================================================================

/// Flags for which fields to retrieve in getqflist()
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct QfWhatFlags {
    flags: u32,
}

pub const QF_WHAT_ALL: u32 = 0x0001;
pub const QF_WHAT_IDX: u32 = 0x0002;
pub const QF_WHAT_NR: u32 = 0x0004;
pub const QF_WHAT_ITEMS: u32 = 0x0008;
pub const QF_WHAT_ID: u32 = 0x0010;
pub const QF_WHAT_TITLE: u32 = 0x0020;
pub const QF_WHAT_CONTEXT: u32 = 0x0040;
pub const QF_WHAT_SIZE: u32 = 0x0080;
pub const QF_WHAT_CHANGEDTICK: u32 = 0x0100;
pub const QF_WHAT_QFBUFNR: u32 = 0x0200;
pub const QF_WHAT_FILEWINID: u32 = 0x0400;
pub const QF_WHAT_QUICKFIXTEXTFUNC: u32 = 0x0800;

impl QfWhatFlags {
    /// Create with no flags
    pub const fn none() -> Self {
        Self { flags: 0 }
    }

    /// Create with all flags
    pub const fn all() -> Self {
        Self { flags: QF_WHAT_ALL }
    }

    /// Create from raw value
    pub const fn from_raw(flags: u32) -> Self {
        Self { flags }
    }

    /// Get raw value
    pub const fn as_raw(self) -> u32 {
        self.flags
    }

    /// Check if requesting all fields
    pub const fn wants_all(self) -> bool {
        (self.flags & QF_WHAT_ALL) != 0
    }

    /// Check if requesting idx field
    pub const fn wants_idx(self) -> bool {
        self.wants_all() || (self.flags & QF_WHAT_IDX) != 0
    }

    /// Check if requesting nr field
    pub const fn wants_nr(self) -> bool {
        self.wants_all() || (self.flags & QF_WHAT_NR) != 0
    }

    /// Check if requesting items field
    pub const fn wants_items(self) -> bool {
        self.wants_all() || (self.flags & QF_WHAT_ITEMS) != 0
    }

    /// Check if requesting id field
    pub const fn wants_id(self) -> bool {
        self.wants_all() || (self.flags & QF_WHAT_ID) != 0
    }

    /// Check if requesting title field
    pub const fn wants_title(self) -> bool {
        self.wants_all() || (self.flags & QF_WHAT_TITLE) != 0
    }

    /// Check if requesting context field
    pub const fn wants_context(self) -> bool {
        self.wants_all() || (self.flags & QF_WHAT_CONTEXT) != 0
    }

    /// Check if requesting size field
    pub const fn wants_size(self) -> bool {
        self.wants_all() || (self.flags & QF_WHAT_SIZE) != 0
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Parse action character
#[no_mangle]
pub extern "C" fn rs_qf_parse_action(action: u8) -> c_int {
    match parse_action(action) {
        Some(a) => a as c_int,
        None => -1,
    }
}

/// FFI export: Check if action is valid
#[no_mangle]
pub extern "C" fn rs_qf_is_valid_action(action: u8) -> c_int {
    c_int::from(parse_action(action).is_some())
}

/// FFI export: Check what flags wants idx
#[no_mangle]
pub extern "C" fn rs_qf_what_wants_idx(flags: u32) -> c_int {
    c_int::from(QfWhatFlags::from_raw(flags).wants_idx())
}

/// FFI export: Check what flags wants items
#[no_mangle]
pub extern "C" fn rs_qf_what_wants_items(flags: u32) -> c_int {
    c_int::from(QfWhatFlags::from_raw(flags).wants_items())
}

/// FFI export: Check what flags wants id
#[no_mangle]
pub extern "C" fn rs_qf_what_wants_id(flags: u32) -> c_int {
    c_int::from(QfWhatFlags::from_raw(flags).wants_id())
}

/// FFI export: Check what flags wants title
#[no_mangle]
pub extern "C" fn rs_qf_what_wants_title(flags: u32) -> c_int {
    c_int::from(QfWhatFlags::from_raw(flags).wants_title())
}

/// FFI export: Check what flags wants size
#[no_mangle]
pub extern "C" fn rs_qf_what_wants_size(flags: u32) -> c_int {
    c_int::from(QfWhatFlags::from_raw(flags).wants_size())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_action() {
        assert_eq!(parse_action(b' '), Some(QfAction::Replace));
        assert_eq!(parse_action(0), Some(QfAction::Replace));
        assert_eq!(parse_action(b'r'), Some(QfAction::Replace));
        assert_eq!(parse_action(b'a'), Some(QfAction::Append));
        assert_eq!(parse_action(b'f'), Some(QfAction::Free));
        assert_eq!(parse_action(b'x'), None);
    }

    #[test]
    fn test_qf_action_values() {
        assert_eq!(QfAction::Replace as c_int, 0);
        assert_eq!(QfAction::Append as c_int, 1);
        assert_eq!(QfAction::Free as c_int, 2);
    }

    #[test]
    fn test_qf_api_result_values() {
        assert_eq!(QfApiResult::Success as c_int, 0);
        assert_eq!(QfApiResult::InvalidList as c_int, 1);
        assert_eq!(QfApiResult::Failed as c_int, 7);
    }

    #[test]
    fn test_qf_list_info() {
        let empty = QfListInfo::default();
        assert!(empty.is_empty());
        assert!(!empty.has_valid_idx());

        let with_items = QfListInfo {
            size: 5,
            idx: 3,
            ..Default::default()
        };
        assert!(!with_items.is_empty());
        assert!(with_items.has_valid_idx());

        let invalid_idx = QfListInfo {
            size: 5,
            idx: 10,
            ..Default::default()
        };
        assert!(!invalid_idx.has_valid_idx());
    }

    #[test]
    fn test_qf_what_flags() {
        let none = QfWhatFlags::none();
        assert!(!none.wants_all());
        assert!(!none.wants_idx());
        assert!(!none.wants_items());

        let all = QfWhatFlags::all();
        assert!(all.wants_all());
        assert!(all.wants_idx());
        assert!(all.wants_items());
        assert!(all.wants_title());
        assert!(all.wants_size());

        let specific = QfWhatFlags::from_raw(QF_WHAT_IDX | QF_WHAT_ITEMS);
        assert!(!specific.wants_all());
        assert!(specific.wants_idx());
        assert!(specific.wants_items());
        assert!(!specific.wants_title());
    }

    #[test]
    fn test_handles_null() {
        assert!(QfInfoHandle::null().is_null());
        assert!(WinHandle::null().is_null());
    }
}
