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
// Phase Q7: Additional VimL API Helpers
// =============================================================================

/// Parameters for setqflist()/setloclist()
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct QfSetListParams {
    /// Action to perform ('r', 'a', 'f', or ' ')
    pub action: u8,
    /// Target list number (0 = current)
    pub nr: c_int,
    /// ID to operate on (0 = use nr)
    pub id: u32,
    /// New index to set (0 = don't change)
    pub idx: c_int,
    /// Whether we have title
    pub has_title: bool,
    /// Whether we have context
    pub has_context: bool,
    /// Whether to create new list
    pub create_new: bool,
}

/// FFI export: Parse setqflist action and validate
#[no_mangle]
pub extern "C" fn rs_qf_parse_setlist_action(action: u8) -> c_int {
    match action {
        b' ' | 0 | b'r' => 0, // Replace
        b'a' => 1,            // Append
        b'f' => 2,            // Free
        _ => -1,              // Invalid
    }
}

/// Validation result for setqflist parameters
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct QfSetListValidation {
    /// Whether parameters are valid
    pub valid: bool,
    /// Error code (0 = no error)
    pub error_code: c_int,
    /// Resolved list index (0-based)
    pub resolved_list_idx: c_int,
    /// Whether to create a new list
    pub create_new: bool,
}

/// FFI export: Validate setqflist parameters
#[no_mangle]
pub extern "C" fn rs_qf_validate_setlist(
    action: u8,
    nr: c_int,
    id: u32,
    curlist: c_int,
    listcount: c_int,
) -> QfSetListValidation {
    let mut result = QfSetListValidation::default();

    // Validate action
    let parsed_action = rs_qf_parse_setlist_action(action);
    if parsed_action < 0 {
        result.error_code = 1; // Invalid action
        return result;
    }

    // For 'f' (free) action, must have valid list
    if parsed_action == 2 && listcount == 0 {
        result.error_code = 2; // No list to free
        return result;
    }

    // Resolve list number
    if id != 0 {
        // ID takes precedence - caller needs to resolve
        result.valid = true;
        result.resolved_list_idx = -1; // Signal to use ID
    } else if nr == 0 {
        // Current list
        if listcount == 0 && parsed_action != 0 {
            // No current list and not replacing
            result.create_new = true;
        }
        result.resolved_list_idx = curlist;
        result.valid = true;
    } else {
        let resolved = rs_qf_resolve_list_nr(nr, curlist, listcount);
        if resolved < 0 && parsed_action != 0 {
            result.error_code = 3; // Invalid list number
            return result;
        }
        result.resolved_list_idx = resolved;
        result.valid = true;
    }

    result
}

/// Result of getqflist() operation
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct QfGetListResult {
    /// Operation succeeded
    pub success: bool,
    /// Number of entries
    pub size: c_int,
    /// Current index (1-based)
    pub idx: c_int,
    /// List ID
    pub id: u32,
    /// Changed tick
    pub changedtick: c_int,
    /// Quickfix buffer number
    pub qfbufnr: c_int,
    /// File window ID (for location lists)
    pub filewinid: c_int,
}

/// FFI export: Create empty getqflist result
#[no_mangle]
pub extern "C" fn rs_qf_getlist_result_empty() -> QfGetListResult {
    QfGetListResult::default()
}

/// FFI export: Create success getqflist result with basic info
#[no_mangle]
pub extern "C" fn rs_qf_getlist_result_success(
    size: c_int,
    idx: c_int,
    id: u32,
) -> QfGetListResult {
    QfGetListResult {
        success: true,
        size,
        idx,
        id,
        ..Default::default()
    }
}

/// Index validation for setqflist 'idx' field
#[no_mangle]
pub extern "C" fn rs_qf_validate_setlist_idx(idx: c_int, count: c_int) -> c_int {
    if idx <= 0 {
        // 0 or negative means "don't change" or "last"
        if idx == 0 {
            0 // Don't change
        } else {
            // Negative: from end
            let resolved = count + idx + 1;
            if resolved < 1 {
                1 // Clamp to first
            } else if resolved > count {
                count // Clamp to last
            } else {
                resolved
            }
        }
    } else if idx > count {
        count // Clamp to last
    } else {
        idx
    }
}

/// Entry index validation for API
#[no_mangle]
pub extern "C" fn rs_qf_api_valid_entry_idx(idx: c_int, count: c_int) -> bool {
    idx >= 1 && idx <= count
}

/// Get the "what" constant value
#[no_mangle]
pub extern "C" fn rs_qf_what_all() -> u32 {
    QF_WHAT_ALL
}

/// Get the "what" idx constant value
#[no_mangle]
pub extern "C" fn rs_qf_what_idx() -> u32 {
    QF_WHAT_IDX
}

/// Get the "what" items constant value
#[no_mangle]
pub extern "C" fn rs_qf_what_items() -> u32 {
    QF_WHAT_ITEMS
}

/// Get the "what" title constant value
#[no_mangle]
pub extern "C" fn rs_qf_what_title() -> u32 {
    QF_WHAT_TITLE
}

/// Get the "what" size constant value
#[no_mangle]
pub extern "C" fn rs_qf_what_size() -> u32 {
    QF_WHAT_SIZE
}

// =============================================================================
// Phase 3: QF_GETLIST_* constants and property flag / index resolution
// =============================================================================

/// QF_GETLIST_* flag constants (must match C enum in quickfix_shim.c)
const QF_GETLIST_NONE: c_int = 0x0;
const QF_GETLIST_TITLE: c_int = 0x1;
const QF_GETLIST_ITEMS: c_int = 0x2;
const QF_GETLIST_NR: c_int = 0x4;
const QF_GETLIST_WINID: c_int = 0x8;
const QF_GETLIST_CONTEXT: c_int = 0x10;
const QF_GETLIST_ID: c_int = 0x20;
const QF_GETLIST_IDX: c_int = 0x40;
const QF_GETLIST_SIZE: c_int = 0x80;
const QF_GETLIST_TICK: c_int = 0x100;
const QF_GETLIST_FILEWINID: c_int = 0x200;
const QF_GETLIST_QFBUFNR: c_int = 0x400;
const QF_GETLIST_QFTF: c_int = 0x800;
const QF_GETLIST_ALL: c_int = 0xFFF;

/// INVALID_QFIDX sentinel (same as C define)
const INVALID_QFIDX: c_int = -1;

/// C return value OK
const C_OK: c_int = 1;

extern "C" {
    // qi / qfl accessors already in lib.rs; we re-declare the subset used here.
    fn nvim_qf_get_curlist_idx(qi: *const c_void) -> c_int;
    fn nvim_qf_get_listcount(qi: *const c_void) -> c_int;

    // dict key check
    fn nvim_tv_dict_find_has_key(
        dict: *const c_void,
        key: *const std::ffi::c_char,
        key_len: c_int,
    ) -> bool;
    // dict number value (returns false if key absent or wrong type)
    fn nvim_tv_dict_find_nr(
        dict: *const c_void,
        key: *const std::ffi::c_char,
        key_len: c_int,
        out: *mut i64,
    ) -> bool;
    // dict string value is exactly "$"
    fn nvim_tv_dict_find_str_is_dollar(
        dict: *const c_void,
        key: *const std::ffi::c_char,
        key_len: c_int,
    ) -> bool;
    // dict add functions
    fn nvim_tv_dict_add_nr_ret(
        dict: *mut c_void,
        key: *const std::ffi::c_char,
        key_len: c_int,
        nr: i64,
    ) -> c_int;
    fn nvim_tv_dict_add_str_copy(
        dict: *mut c_void,
        key: *const std::ffi::c_char,
        key_len: c_int,
        val: *const std::ffi::c_char,
    ) -> c_int;
    fn nvim_tv_dict_add_list_empty(
        dict: *mut c_void,
        key: *const std::ffi::c_char,
        key_len: c_int,
    ) -> c_int;
    // qf window id
    fn nvim_qf_winid(qi: *const c_void) -> c_int;
    // qf valid buf number
    fn nvim_qf_get_valid_bufnr(qi: *const c_void) -> c_int;
    // id-to-index lookup (already in lib.rs but needed here)
    fn rs_qf_id2nr(qi: *const c_void, qf_id: u32) -> c_int;
    // stack empty check (already in lib.rs)
    fn rs_qf_stack_empty(qi: *const c_void) -> bool;
}

/// Check if the given dict has a key, using a byte-slice key (no null terminator needed).
///
/// # Safety
/// `dict` must be a valid non-null `dict_T *`.
#[inline]
#[allow(clippy::cast_possible_wrap)]
unsafe fn dict_has_key(dict: *const c_void, key: &[u8]) -> bool {
    nvim_tv_dict_find_has_key(dict, key.as_ptr().cast(), key.len() as c_int)
}

/// Attempt to read a VAR_NUMBER value from a dict key.
///
/// Returns `Some(value)` if the key exists and has type VAR_NUMBER.
///
/// # Safety
/// `dict` must be a valid non-null `dict_T *`.
#[inline]
#[allow(clippy::cast_possible_wrap)]
unsafe fn dict_find_nr(dict: *const c_void, key: &[u8]) -> Option<i64> {
    let mut val: i64 = 0;
    if nvim_tv_dict_find_nr(dict, key.as_ptr().cast(), key.len() as c_int, &raw mut val) {
        Some(val)
    } else {
        None
    }
}

/// Returns true if the dict has the given key with a VAR_STRING value equal to "$".
///
/// # Safety
/// `dict` must be a valid non-null `dict_T *`.
#[inline]
#[allow(clippy::cast_possible_wrap)]
unsafe fn dict_find_str_is_dollar(dict: *const c_void, key: &[u8]) -> bool {
    nvim_tv_dict_find_str_is_dollar(dict, key.as_ptr().cast(), key.len() as c_int)
}

/// Add a number to a dict; returns C_OK or C_FAIL.
///
/// # Safety
/// `dict` must be a valid non-null `dict_T *`.
#[inline]
#[allow(clippy::cast_possible_wrap)]
unsafe fn dict_add_nr(dict: *mut c_void, key: &[u8], nr: i64) -> c_int {
    nvim_tv_dict_add_nr_ret(dict, key.as_ptr().cast(), key.len() as c_int, nr)
}

/// Add a string copy to a dict; returns C_OK or C_FAIL.
///
/// # Safety
/// `dict` must be a valid non-null `dict_T *`.
#[inline]
#[allow(clippy::cast_possible_wrap)]
unsafe fn dict_add_str(dict: *mut c_void, key: &[u8], val: *const std::ffi::c_char) -> c_int {
    nvim_tv_dict_add_str_copy(dict, key.as_ptr().cast(), key.len() as c_int, val)
}

/// Add an empty list to a dict; returns C_OK or C_FAIL.
///
/// # Safety
/// `dict` must be a valid non-null `dict_T *`.
#[inline]
#[allow(clippy::cast_possible_wrap)]
unsafe fn dict_add_list_empty(dict: *mut c_void, key: &[u8]) -> c_int {
    nvim_tv_dict_add_list_empty(dict, key.as_ptr().cast(), key.len() as c_int)
}

/// Convert the keys in `what` dict to `QF_GETLIST_*` flag bitmask.
///
/// Mirrors C `qf_getprop_keys2flags`.
///
/// # Safety
/// `what` must be a valid non-null `dict_T *`.
#[no_mangle]
pub unsafe extern "C" fn rs_qf_getprop_keys2flags(what: *const c_void, loclist: bool) -> c_int {
    let mut flags = QF_GETLIST_NONE;

    if dict_has_key(what, b"all") {
        flags |= QF_GETLIST_ALL;
        if !loclist {
            // File window ID is applicable only to location list windows
            flags &= !QF_GETLIST_FILEWINID;
        }
    }
    if dict_has_key(what, b"title") {
        flags |= QF_GETLIST_TITLE;
    }
    if dict_has_key(what, b"nr") {
        flags |= QF_GETLIST_NR;
    }
    if dict_has_key(what, b"winid") {
        flags |= QF_GETLIST_WINID;
    }
    if dict_has_key(what, b"context") {
        flags |= QF_GETLIST_CONTEXT;
    }
    if dict_has_key(what, b"id") {
        flags |= QF_GETLIST_ID;
    }
    if dict_has_key(what, b"items") {
        flags |= QF_GETLIST_ITEMS;
    }
    if dict_has_key(what, b"idx") {
        flags |= QF_GETLIST_IDX;
    }
    if dict_has_key(what, b"size") {
        flags |= QF_GETLIST_SIZE;
    }
    if dict_has_key(what, b"changedtick") {
        flags |= QF_GETLIST_TICK;
    }
    if loclist && dict_has_key(what, b"filewinid") {
        flags |= QF_GETLIST_FILEWINID;
    }
    if dict_has_key(what, b"qfbufnr") {
        flags |= QF_GETLIST_QFBUFNR;
    }
    if dict_has_key(what, b"quickfixtextfunc") {
        flags |= QF_GETLIST_QFTF;
    }

    flags
}

/// Resolve the quickfix list index from "nr" and "id" keys in `what` dict.
///
/// Returns the list index or `INVALID_QFIDX` (-1).
/// Mirrors C `qf_getprop_qfidx`.
///
/// # Safety
/// `qi` and `what` must be valid non-null pointers.
#[no_mangle]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_qf_getprop_qfidx(qi: *const c_void, what: *const c_void) -> c_int {
    let listcount = nvim_qf_get_listcount(qi);
    let mut qf_idx = nvim_qf_get_curlist_idx(qi); // default is current list

    // Check "nr" key
    if let Some(nr) = dict_find_nr(what, b"nr") {
        if nr != 0 {
            qf_idx = nr as c_int - 1;
            if qf_idx < 0 || qf_idx >= listcount {
                return INVALID_QFIDX;
            }
        }
        // nr == 0 means "use the current list" -- keep qf_idx as is
    } else if dict_find_str_is_dollar(what, b"nr") {
        // "$" means last list
        qf_idx = listcount - 1;
    } else if dict_has_key(what, b"nr") {
        // key exists but wrong type
        return INVALID_QFIDX;
    }

    // Check "id" key
    if let Some(id_val) = dict_find_nr(what, b"id") {
        // For zero, use the current list or the list specified by 'nr'
        if id_val != 0 {
            #[allow(clippy::cast_sign_loss)]
            let id = id_val as u32;
            qf_idx = rs_qf_id2nr(qi, id);
        }
    } else if dict_has_key(what, b"id") {
        // id key exists but wrong type
        return INVALID_QFIDX;
    }

    qf_idx
}

/// Return default values for quickfix list properties in `retdict`.
///
/// Mirrors C `qf_getprop_defaults`.
///
/// # Safety
/// `qi` and `retdict` must be valid pointers (`qi` may be null for global quickfix).
#[no_mangle]
pub unsafe extern "C" fn rs_qf_getprop_defaults(
    qi: *const c_void,
    flags: c_int,
    locstack: bool,
    retdict: *mut c_void,
) -> c_int {
    let mut status = C_OK;

    macro_rules! check {
        ($expr:expr) => {
            if status == C_OK {
                status = $expr;
            }
        };
    }

    if (flags & QF_GETLIST_TITLE) != 0 {
        status = dict_add_str(retdict, b"title", c"".as_ptr());
    }
    check!({
        if (flags & QF_GETLIST_ITEMS) != 0 {
            dict_add_list_empty(retdict, b"items")
        } else {
            C_OK
        }
    });
    check!({
        if (flags & QF_GETLIST_NR) != 0 {
            dict_add_nr(retdict, b"nr", 0)
        } else {
            C_OK
        }
    });
    check!({
        if (flags & QF_GETLIST_WINID) != 0 {
            let winid = nvim_qf_winid(qi);
            dict_add_nr(retdict, b"winid", winid.into())
        } else {
            C_OK
        }
    });
    check!({
        if (flags & QF_GETLIST_CONTEXT) != 0 {
            dict_add_str(retdict, b"context", c"".as_ptr())
        } else {
            C_OK
        }
    });
    check!({
        if (flags & QF_GETLIST_ID) != 0 {
            dict_add_nr(retdict, b"id", 0)
        } else {
            C_OK
        }
    });
    check!({
        if (flags & QF_GETLIST_IDX) != 0 {
            dict_add_nr(retdict, b"idx", 0)
        } else {
            C_OK
        }
    });
    check!({
        if (flags & QF_GETLIST_SIZE) != 0 {
            dict_add_nr(retdict, b"size", 0)
        } else {
            C_OK
        }
    });
    check!({
        if (flags & QF_GETLIST_TICK) != 0 {
            dict_add_nr(retdict, b"changedtick", 0)
        } else {
            C_OK
        }
    });
    check!({
        if locstack && (flags & QF_GETLIST_FILEWINID) != 0 {
            dict_add_nr(retdict, b"filewinid", 0)
        } else {
            C_OK
        }
    });
    check!({
        if (flags & QF_GETLIST_QFBUFNR) != 0 {
            let bufnum = nvim_qf_get_valid_bufnr(qi);
            dict_add_nr(retdict, b"qfbufnr", bufnum.into())
        } else {
            C_OK
        }
    });
    check!({
        if (flags & QF_GETLIST_QFTF) != 0 {
            dict_add_str(retdict, b"quickfixtextfunc", c"".as_ptr())
        } else {
            C_OK
        }
    });

    status
}

/// Resolve the quickfix list index from "nr" and "id" for setqflist.
///
/// Mirrors C `qf_setprop_get_qfidx`.
///
/// # Safety
/// `qi`, `what`, and `newlist` must be valid non-null pointers.
#[no_mangle]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_qf_setprop_get_qfidx(
    qi: *const c_void,
    what: *const c_void,
    action: c_int,
    newlist: *mut bool,
) -> c_int {
    const ACTION_SPACE: c_int = b' ' as c_int;
    const ACTION_A: c_int = b'a' as c_int;

    let listcount = nvim_qf_get_listcount(qi);
    let mut qf_idx = nvim_qf_get_curlist_idx(qi); // default is current list

    if dict_has_key(what, b"nr") {
        if let Some(nr) = dict_find_nr(what, b"nr") {
            // for zero use the current list
            if nr != 0 {
                qf_idx = nr as c_int - 1;
            }

            if (action == ACTION_SPACE || action == ACTION_A) && qf_idx == listcount {
                // When creating a new list, accept qf_idx pointing to the next
                // non-available list and add the new list at the end of the stack.
                *newlist = true;
                qf_idx = if rs_qf_stack_empty(qi) {
                    0
                } else {
                    listcount - 1
                };
            } else if qf_idx < 0 || qf_idx >= listcount {
                return INVALID_QFIDX;
            } else if action != ACTION_SPACE {
                *newlist = false; // use the specified list
            }
        } else if dict_find_str_is_dollar(what, b"nr") {
            if !rs_qf_stack_empty(qi) {
                qf_idx = listcount - 1;
            } else if *newlist {
                qf_idx = 0;
            } else {
                return INVALID_QFIDX;
            }
        } else {
            return INVALID_QFIDX;
        }
    }

    if !(*newlist) && dict_has_key(what, b"id") {
        // Use the quickfix/location list with the specified id
        if let Some(id_val) = dict_find_nr(what, b"id") {
            #[allow(clippy::cast_sign_loss)]
            let id = id_val as u32;
            return rs_qf_id2nr(qi, id);
        }
        return INVALID_QFIDX;
    }

    qf_idx
}

// =============================================================================
// Phase 2: qf_getprop_filewinid, qf_getprop_qfbufnr helpers
// =============================================================================

extern "C" {
    fn nvim_qf_is_ll_window(wp: *const c_void) -> bool;
    fn nvim_qf_find_win_with_loclist(ll: *const c_void) -> *mut c_void;
    fn nvim_qf_win_get_handle(wp: *const c_void) -> c_int;
}

/// Get the window ID for the file-display window associated with a location list.
///
/// Returns 0 if `wp` is not a location list window, or if no associated window
/// is found. Mirrors the logic of C `qf_getprop_filewinid`.
///
/// # Safety
///
/// - `wp` may be null (returns 0)
/// - `qi` may be null (returns 0)
#[no_mangle]
pub unsafe extern "C" fn rs_qf_getprop_filewinid(wp: *const c_void, qi: *const c_void) -> c_int {
    if wp.is_null() || qi.is_null() {
        return 0;
    }
    if !nvim_qf_is_ll_window(wp) {
        return 0;
    }
    let ll_wp = nvim_qf_find_win_with_loclist(qi);
    if ll_wp.is_null() {
        return 0;
    }
    nvim_qf_win_get_handle(ll_wp.cast_const())
}

/// Get the quickfix buffer number for the given quickfix stack, or 0 if the
/// buffer is not valid. Mirrors the logic of C `qf_getprop_qfbufnr`.
///
/// # Safety
///
/// - `qi` may be null (returns 0)
#[no_mangle]
pub unsafe extern "C" fn rs_qf_getprop_qfbufnr(qi: *const c_void) -> c_int {
    nvim_qf_get_valid_bufnr(qi)
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

    // Phase Q7 tests
    #[test]
    fn test_parse_setlist_action() {
        assert_eq!(rs_qf_parse_setlist_action(b' '), 0);
        assert_eq!(rs_qf_parse_setlist_action(b'r'), 0);
        assert_eq!(rs_qf_parse_setlist_action(b'a'), 1);
        assert_eq!(rs_qf_parse_setlist_action(b'f'), 2);
        assert_eq!(rs_qf_parse_setlist_action(b'x'), -1);
    }

    #[test]
    fn test_validate_setlist_valid() {
        let result = rs_qf_validate_setlist(b' ', 0, 0, 2, 5);
        assert!(result.valid);
        assert_eq!(result.resolved_list_idx, 2);
    }

    #[test]
    fn test_validate_setlist_invalid_action() {
        let result = rs_qf_validate_setlist(b'x', 0, 0, 0, 5);
        assert!(!result.valid);
        assert_eq!(result.error_code, 1);
    }

    #[test]
    fn test_validate_setlist_with_id() {
        let result = rs_qf_validate_setlist(b' ', 0, 123, 0, 5);
        assert!(result.valid);
        assert_eq!(result.resolved_list_idx, -1); // Signal to use ID
    }

    #[test]
    fn test_getlist_result_empty() {
        let result = rs_qf_getlist_result_empty();
        assert!(!result.success);
        assert_eq!(result.size, 0);
    }

    #[test]
    fn test_getlist_result_success() {
        let result = rs_qf_getlist_result_success(10, 3, 42);
        assert!(result.success);
        assert_eq!(result.size, 10);
        assert_eq!(result.idx, 3);
        assert_eq!(result.id, 42);
    }

    #[test]
    fn test_validate_setlist_idx() {
        // Normal positive
        assert_eq!(rs_qf_validate_setlist_idx(3, 10), 3);

        // Zero means don't change
        assert_eq!(rs_qf_validate_setlist_idx(0, 10), 0);

        // Clamp to max
        assert_eq!(rs_qf_validate_setlist_idx(15, 10), 10);

        // Negative from end
        assert_eq!(rs_qf_validate_setlist_idx(-1, 10), 10);
        assert_eq!(rs_qf_validate_setlist_idx(-5, 10), 6);

        // Clamp negative
        assert_eq!(rs_qf_validate_setlist_idx(-20, 10), 1);
    }

    #[test]
    fn test_valid_entry_idx() {
        assert!(rs_qf_api_valid_entry_idx(1, 10));
        assert!(rs_qf_api_valid_entry_idx(10, 10));
        assert!(!rs_qf_api_valid_entry_idx(0, 10));
        assert!(!rs_qf_api_valid_entry_idx(11, 10));
        assert!(!rs_qf_api_valid_entry_idx(-1, 10));
    }

    #[test]
    fn test_what_constants() {
        assert_eq!(rs_qf_what_all(), QF_WHAT_ALL);
        assert_eq!(rs_qf_what_idx(), QF_WHAT_IDX);
        assert_eq!(rs_qf_what_items(), QF_WHAT_ITEMS);
        assert_eq!(rs_qf_what_title(), QF_WHAT_TITLE);
        assert_eq!(rs_qf_what_size(), QF_WHAT_SIZE);
    }

    #[test]
    fn test_setlist_params_default() {
        let params = QfSetListParams::default();
        assert_eq!(params.action, 0);
        assert_eq!(params.nr, 0);
        assert_eq!(params.id, 0);
        assert!(!params.has_title);
    }

    #[test]
    fn test_setlist_validation_default() {
        let validation = QfSetListValidation::default();
        assert!(!validation.valid);
        assert_eq!(validation.error_code, 0);
    }
}
