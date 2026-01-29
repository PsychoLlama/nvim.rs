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

    /// Check if requesting changedtick field
    pub const fn wants_changedtick(self) -> bool {
        self.wants_all() || (self.flags & QF_WHAT_CHANGEDTICK) != 0
    }

    /// Check if requesting qfbufnr field
    pub const fn wants_qfbufnr(self) -> bool {
        self.wants_all() || (self.flags & QF_WHAT_QFBUFNR) != 0
    }

    /// Check if requesting filewinid field
    pub const fn wants_filewinid(self) -> bool {
        self.wants_all() || (self.flags & QF_WHAT_FILEWINID) != 0
    }

    /// Check if requesting quickfixtextfunc field
    pub const fn wants_quickfixtextfunc(self) -> bool {
        self.wants_all() || (self.flags & QF_WHAT_QUICKFIXTEXTFUNC) != 0
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

/// FFI export: Check what flags wants context
#[no_mangle]
pub extern "C" fn rs_qf_what_wants_context(flags: u32) -> c_int {
    c_int::from(QfWhatFlags::from_raw(flags).wants_context())
}

/// FFI export: Check what flags wants nr (list number)
#[no_mangle]
pub extern "C" fn rs_qf_what_wants_nr(flags: u32) -> c_int {
    c_int::from(QfWhatFlags::from_raw(flags).wants_nr())
}

/// FFI export: Check what flags wants changedtick
#[no_mangle]
pub extern "C" fn rs_qf_what_wants_changedtick(flags: u32) -> c_int {
    c_int::from(QfWhatFlags::from_raw(flags).wants_changedtick())
}

/// FFI export: Check what flags wants qfbufnr
#[no_mangle]
pub extern "C" fn rs_qf_what_wants_qfbufnr(flags: u32) -> c_int {
    c_int::from(QfWhatFlags::from_raw(flags).wants_qfbufnr())
}

/// FFI export: Check what flags wants filewinid
#[no_mangle]
pub extern "C" fn rs_qf_what_wants_filewinid(flags: u32) -> c_int {
    c_int::from(QfWhatFlags::from_raw(flags).wants_filewinid())
}

/// FFI export: Check what flags wants quickfixtextfunc
#[no_mangle]
pub extern "C" fn rs_qf_what_wants_quickfixtextfunc(flags: u32) -> c_int {
    c_int::from(QfWhatFlags::from_raw(flags).wants_quickfixtextfunc())
}

/// FFI export: Parse what flags from individual bool fields
#[no_mangle]
pub extern "C" fn rs_qf_build_what_flags(
    all: bool,
    idx: bool,
    nr: bool,
    items: bool,
    id: bool,
    title: bool,
    context: bool,
    size: bool,
    changedtick: bool,
    qfbufnr: bool,
    filewinid: bool,
    quickfixtextfunc: bool,
) -> u32 {
    let mut flags = 0u32;
    if all {
        flags |= QF_WHAT_ALL;
    }
    if idx {
        flags |= QF_WHAT_IDX;
    }
    if nr {
        flags |= QF_WHAT_NR;
    }
    if items {
        flags |= QF_WHAT_ITEMS;
    }
    if id {
        flags |= QF_WHAT_ID;
    }
    if title {
        flags |= QF_WHAT_TITLE;
    }
    if context {
        flags |= QF_WHAT_CONTEXT;
    }
    if size {
        flags |= QF_WHAT_SIZE;
    }
    if changedtick {
        flags |= QF_WHAT_CHANGEDTICK;
    }
    if qfbufnr {
        flags |= QF_WHAT_QFBUFNR;
    }
    if filewinid {
        flags |= QF_WHAT_FILEWINID;
    }
    if quickfixtextfunc {
        flags |= QF_WHAT_QUICKFIXTEXTFUNC;
    }
    flags
}

// =============================================================================
// Entry Property Struct for getqflist() items
// =============================================================================

/// Properties of a single quickfix entry for getqflist() items
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct QfEntryProps {
    /// Buffer number
    pub bufnr: c_int,
    /// Line number
    pub lnum: c_int,
    /// End line number
    pub end_lnum: c_int,
    /// Column number
    pub col: c_int,
    /// End column number
    pub end_col: c_int,
    /// Column is visual column
    pub vcol: bool,
    /// Entry number
    pub nr: c_int,
    /// Entry type character
    pub entry_type: u8,
    /// Valid entry flag
    pub valid: bool,
}

/// FFI export: Get default entry props
#[no_mangle]
pub extern "C" fn rs_qf_entry_props_default() -> QfEntryProps {
    QfEntryProps::default()
}

// =============================================================================
// List Number Resolution
// =============================================================================

/// Resolve a list number (0 = current, negative = from end, positive = 1-based)
#[no_mangle]
pub extern "C" fn rs_qf_resolve_list_nr(nr: c_int, curlist: c_int, listcount: c_int) -> c_int {
    use std::cmp::Ordering;

    if listcount <= 0 {
        return -1;
    }

    match nr.cmp(&0) {
        Ordering::Equal => {
            // Current list
            curlist
        }
        Ordering::Greater => {
            // 1-based index
            let idx = nr - 1;
            if idx < listcount {
                idx
            } else {
                -1
            }
        }
        Ordering::Less => {
            // Negative: from end
            let idx = listcount + nr;
            if idx >= 0 {
                idx
            } else {
                -1
            }
        }
    }
}

/// FFI export: Check if list number is valid
#[no_mangle]
pub extern "C" fn rs_qf_valid_list_nr(nr: c_int, listcount: c_int) -> bool {
    if listcount <= 0 {
        return false;
    }
    if nr == 0 {
        return true; // Current list always valid if listcount > 0
    }
    if nr > 0 {
        nr <= listcount
    } else {
        nr.abs() <= listcount
    }
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

    #[test]
    fn test_what_flags_extended() {
        let flags = QfWhatFlags::from_raw(QF_WHAT_CHANGEDTICK | QF_WHAT_QFBUFNR);
        assert!(flags.wants_changedtick());
        assert!(flags.wants_qfbufnr());
        assert!(!flags.wants_filewinid());
        assert!(!flags.wants_quickfixtextfunc());

        let all = QfWhatFlags::all();
        assert!(all.wants_changedtick());
        assert!(all.wants_qfbufnr());
        assert!(all.wants_filewinid());
        assert!(all.wants_quickfixtextfunc());
    }

    #[test]
    fn test_build_what_flags() {
        let flags = rs_qf_build_what_flags(
            false, true, false, true, false, false, false, false, false, false, false, false,
        );
        assert_eq!(flags, QF_WHAT_IDX | QF_WHAT_ITEMS);

        let all_flags = rs_qf_build_what_flags(
            true, false, false, false, false, false, false, false, false, false, false, false,
        );
        assert_eq!(all_flags, QF_WHAT_ALL);
    }

    #[test]
    fn test_resolve_list_nr() {
        // Current list (nr=0)
        assert_eq!(rs_qf_resolve_list_nr(0, 3, 5), 3);

        // Positive (1-based)
        assert_eq!(rs_qf_resolve_list_nr(1, 3, 5), 0);
        assert_eq!(rs_qf_resolve_list_nr(5, 3, 5), 4);
        assert_eq!(rs_qf_resolve_list_nr(6, 3, 5), -1); // Out of range

        // Negative (from end)
        assert_eq!(rs_qf_resolve_list_nr(-1, 3, 5), 4);
        assert_eq!(rs_qf_resolve_list_nr(-5, 3, 5), 0);
        assert_eq!(rs_qf_resolve_list_nr(-6, 3, 5), -1); // Out of range

        // Empty list
        assert_eq!(rs_qf_resolve_list_nr(0, 0, 0), -1);
    }

    #[test]
    fn test_valid_list_nr() {
        assert!(rs_qf_valid_list_nr(0, 5)); // Current
        assert!(rs_qf_valid_list_nr(1, 5)); // First
        assert!(rs_qf_valid_list_nr(5, 5)); // Last
        assert!(!rs_qf_valid_list_nr(6, 5)); // Out of range
        assert!(rs_qf_valid_list_nr(-1, 5)); // Last from end
        assert!(rs_qf_valid_list_nr(-5, 5)); // First from end
        assert!(!rs_qf_valid_list_nr(-6, 5)); // Out of range
        assert!(!rs_qf_valid_list_nr(0, 0)); // Empty list
    }

    #[test]
    fn test_entry_props_default() {
        let props = rs_qf_entry_props_default();
        assert_eq!(props.bufnr, 0);
        assert_eq!(props.lnum, 0);
        assert!(!props.valid);
    }
}
