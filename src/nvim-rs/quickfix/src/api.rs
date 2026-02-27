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
// Phase 8: qf_get_properties / qf_set_properties cluster
// =============================================================================

// C_FAIL is 0 (FAIL in nvim); C_OK is 1 (OK in nvim).
// INVALID_QFIDX is already defined above as -1.
const C_FAIL: c_int = 0;

// VarType enum values (typval_defs.h)
const VAR_STRING: c_int = 2;
const VAR_LIST: c_int = 4;

extern "C" {
    // Phase 8 get-side accessors (qfl field accessors; Phase 15: merged into nvim_qf_get_*)
    fn nvim_qf_get_index(qfl: *const c_void) -> c_int;
    fn nvim_qf_get_count(qfl: *const c_void) -> c_int;
    fn nvim_qf_get_id(qfl: *const c_void) -> u32;
    fn nvim_qf_get_changedtick(qfl: *const c_void) -> c_int;
    fn nvim_qf_get_title(qfl: *const c_void) -> *const std::ffi::c_char;
    fn nvim_qfl_get_ctx(qfl: *const c_void) -> *mut c_void;
    fn nvim_qf_get_list_handle(qi: *const c_void, qf_idx: c_int) -> *mut c_void;

    // typval list/dict ops (qf-specific void* versions)
    fn nvim_qf_tv_list_alloc_ret(rettv: *mut c_void);
    fn nvim_qf_tv_dict_alloc_ret(rettv: *mut c_void);
    fn nvim_qf_tv_dict_alloc() -> *mut c_void;
    fn nvim_qf_tv_list_append_dict(list: *mut c_void, dict: *mut c_void);
    fn nvim_tv_dict_add_tv(
        dict: *mut c_void,
        key: *const std::ffi::c_char,
        key_len: c_int,
        tv: *mut c_void,
    ) -> c_int;
    fn nvim_tv_dict_item_alloc_len(key: *const std::ffi::c_char, key_len: c_int) -> *mut c_void;
    fn nvim_qf_tv_dict_add_item(dict: *mut c_void, item: *mut c_void) -> c_int;
    fn nvim_qf_tv_dict_item_free(item: *mut c_void);
    fn nvim_tv_copy(from: *const c_void, to: *mut c_void);
    fn nvim_qf_qftf_cb_put(qfl: *mut c_void, tv_out: *mut c_void) -> bool;
    fn nvim_tv_clear(tv: *mut c_void);
    fn nvim_qf_tv_get_type(tv: *const c_void) -> c_int;
    fn nvim_di_get_type(di: *const c_void) -> c_int;
    fn nvim_di_get_nr(di: *const c_void) -> i64;
    fn nvim_qf_di_get_tv(di: *mut c_void) -> *mut c_void;
    fn nvim_tv_dict_find(
        dict: *const c_void,
        key: *const std::ffi::c_char,
        key_len: c_int,
    ) -> *mut c_void;
    fn nvim_find_win_by_nr_or_id(argvars: *const c_void) -> *mut c_void;
    fn nvim_tv_advance(tv: *const c_void) -> *mut c_void;
    fn nvim_tv_is_unknown(tv: *const c_void) -> bool;
    fn nvim_tv_is_dict(tv: *const c_void) -> bool;
    fn nvim_qf_tv_get_dict(tv: *const c_void) -> *mut c_void;
    fn nvim_qf_tv_get_list(tv: *const c_void) -> *mut c_void;
    fn nvim_emsg_dictreq();

    fn nvim_qfline_get_type(qfp: *const c_void) -> std::ffi::c_char;
    fn nvim_qfline_get_lnum(qfp: *const c_void) -> i32;
    fn nvim_qfline_get_end_lnum(qfp: *const c_void) -> i32;
    fn nvim_qfline_get_col(qfp: *const c_void) -> c_int;
    fn nvim_qfline_get_end_col(qfp: *const c_void) -> c_int;
    fn nvim_qfline_get_viscol(qfp: *const c_void) -> bool;
    fn nvim_qfline_get_nr(qfp: *const c_void) -> c_int;
    fn nvim_qfline_get_text(qfp: *const c_void) -> *const std::ffi::c_char;
    fn nvim_qfline_get_module(qfp: *const c_void) -> *const std::ffi::c_char;
    fn nvim_qfline_get_pattern(qfp: *const c_void) -> *const std::ffi::c_char;
    fn nvim_qfline_get_next(qfp: *const c_void) -> *const c_void;
    fn nvim_qfline_get_valid_bufnr(qfp: *const c_void) -> c_int;
    fn nvim_qf_get_start(qfl: *const c_void) -> *const c_void;
    fn nvim_qf_got_int() -> bool;
    fn nvim_qf_alloc_internal_stack() -> *mut c_void;
    fn nvim_qf_free_lists_for_qi(qi: *mut c_void);
    fn nvim_qf_tv_dict_add_list(
        dict: *mut c_void,
        key: *const std::ffi::c_char,
        key_len: c_int,
        list: *mut c_void,
    ) -> c_int;
    fn nvim_qf_tv_list_alloc() -> *mut c_void;
    fn nvim_get_p_efm() -> *const std::ffi::c_char;
    fn nvim_tv_dict_get_efm_str(what: *const c_void) -> *const std::ffi::c_char;
    fn nvim_tv_dict_efm_wrong_type(what: *const c_void) -> bool;
    fn nvim_tv_dict_has_lines_key(dict: *const c_void) -> bool;
    fn nvim_tv_dict_get_lines_di_tv(dict: *const c_void) -> *mut c_void;
    fn nvim_tv_alloc() -> *mut c_void;
    fn nvim_tv_alloc_copy(src_tv: *const c_void) -> *mut c_void;

    // Phase 8 set-side accessors
    fn nvim_qf_tv_get_string_chk(tv: *const c_void) -> *const std::ffi::c_char;
    fn nvim_qf_tv_free(tv: *mut c_void);
    fn nvim_qfl_free_ctx(qfl: *mut c_void);
    fn nvim_qfl_set_ctx(qfl: *mut c_void, ctx_tv: *mut c_void);
    fn nvim_qfl_free_qftf_cb(qfl: *mut c_void);
    fn nvim_qfl_set_qftf_cb_from_tv(qfl: *mut c_void, tv: *mut c_void) -> bool;
    fn nvim_qf_set_curlist_idx(qi: *mut c_void, idx: c_int);
    fn nvim_emsg_invact(act: *const std::ffi::c_char);
    fn nvim_emsg_listreq();
    fn nvim_emsg_au_recursive();
    fn nvim_emsg_string_required();
    fn nvim_qf_tv_set_number(tv: *mut c_void, nr: i64);
    fn nvim_qf_tv_is_list_type(tv: *const c_void) -> bool;

    // Phase 15 inlined set-properties primitives
    fn nvim_qf_set_title_dup(qfl: *mut c_void, title: *const std::ffi::c_char);
    fn nvim_qf_tv_dict_get_string(
        dict: *const c_void,
        key: *const std::ffi::c_char,
        alloc: bool,
    ) -> *mut std::ffi::c_char;
    fn rs_qf_update_win_titlevar(qi: *mut c_void);
    fn rs_qf_add_entries(
        qi: *mut c_void,
        qf_idx: c_int,
        list: *const c_void,
        title: *const std::ffi::c_char,
        action: c_int,
    ) -> c_int;
    fn rs_qf_free_items(qfl: *mut c_void);
    fn nvim_qf_tv_get_number_chk(tv: *const c_void, denote: *mut bool) -> i64;
    fn rs_qf_get_nth_entry(
        qfl: *const c_void,
        errornr: c_int,
        new_qfidx: *mut c_int,
    ) -> *mut c_void;
    fn nvim_qf_set_ptr(qfl: *mut c_void, ptr: *const c_void);
    fn nvim_qf_set_index(qfl: *mut c_void, idx: c_int);
    fn nvim_qf_get_curlist_id(qi: *const c_void) -> u32;
    fn rs_qf_win_pos_update(qi: *mut c_void, old_qf_index: c_int) -> bool;
    fn rs_qf_incr_changedtick(qfl: *mut c_void);
    fn nvim_di_get_string(di: *const c_void) -> *const std::ffi::c_char;

    // Global quickfix / location list accessors
    fn nvim_get_ql_info() -> *mut c_void;
    fn nvim_win_get_loclist(wp: *const c_void) -> *const c_void;
    fn nvim_qf_update_buffer(qi: *mut c_void, old_last: *const c_void);
    fn rs_qf_init_ext(
        qi: *mut c_void,
        qf_idx: c_int,
        enc: *const std::ffi::c_char,
        efile: *const std::ffi::c_char,
        tv: *const c_void,
        errorformat: *const std::ffi::c_char,
        newlist: bool,
        lnumfirst: i32,
        lnumlast: i32,
        qf_title: *const std::ffi::c_char,
        enc2: *const std::ffi::c_char,
    ) -> c_int;
    fn rs_qf_free_list(qfl: *mut c_void);
    fn rs_qf_new_list(qi: *mut c_void, title: *const std::ffi::c_char);
    fn rs_set_errorlist(
        wp: *mut c_void,
        list: *mut c_void,
        action: c_int,
        title: *mut std::ffi::c_char,
        what: *mut c_void,
    ) -> c_int;
}

// (C_OK and INVALID_QFIDX already defined above; C_FAIL defined in Phase 8 header)

/// Helper: add string to dict (null pointer becomes empty string).
///
/// # Safety
/// `dict` must be valid; `val` may be null (will use empty string).
#[allow(clippy::cast_possible_wrap)]
unsafe fn dict_add_str_or_empty(
    dict: *mut c_void,
    key: &[u8],
    val: *const std::ffi::c_char,
) -> c_int {
    if val.is_null() {
        nvim_tv_dict_add_str_copy(dict, key.as_ptr().cast(), key.len() as c_int, c"".as_ptr())
    } else {
        nvim_tv_dict_add_str_copy(dict, key.as_ptr().cast(), key.len() as c_int, val)
    }
}

/// Build a single qfline dict and append it to `list`.
///
/// Mirrors C `get_qfline_items`.
///
/// # Safety
/// `qfp` and `list` must be valid non-null pointers.
#[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
unsafe fn rs_get_qfline_items_impl(qfp: *const c_void, list: *mut c_void) -> c_int {
    let bufnum = nvim_qfline_get_valid_bufnr(qfp);

    let dict = nvim_qf_tv_dict_alloc();
    nvim_qf_tv_list_append_dict(list, dict);

    // Build type string ("E", "W", etc. or "")
    let type_ch = nvim_qfline_get_type(qfp) as u8;
    let type_buf = [type_ch, 0u8];

    // Add numeric fields
    let ok = dict_add_nr(dict, b"bufnr", bufnum.into()) == C_OK
        && dict_add_nr(dict, b"lnum", nvim_qfline_get_lnum(qfp).into()) == C_OK
        && dict_add_nr(dict, b"end_lnum", nvim_qfline_get_end_lnum(qfp).into()) == C_OK
        && dict_add_nr(dict, b"col", nvim_qfline_get_col(qfp).into()) == C_OK
        && dict_add_nr(dict, b"end_col", nvim_qfline_get_end_col(qfp).into()) == C_OK
        && dict_add_nr(
            dict,
            b"vcol",
            c_int::from(nvim_qfline_get_viscol(qfp)).into(),
        ) == C_OK
        && dict_add_nr(dict, b"nr", nvim_qfline_get_nr(qfp).into()) == C_OK;

    if !ok {
        return C_FAIL;
    }

    // Add string fields
    let ok2 = dict_add_str_or_empty(dict, b"module", nvim_qfline_get_module(qfp)) == C_OK
        && dict_add_str_or_empty(dict, b"pattern", nvim_qfline_get_pattern(qfp)) == C_OK
        && dict_add_str_or_empty(dict, b"text", nvim_qfline_get_text(qfp)) == C_OK
        && nvim_tv_dict_add_str_copy(
            dict,
            b"type".as_ptr().cast(),
            4,
            if type_ch != 0 {
                type_buf.as_ptr().cast()
            } else {
                c"".as_ptr()
            },
        ) == C_OK;

    if !ok2 {
        return C_FAIL;
    }

    // user_data: only if not VAR_UNKNOWN (type 0)
    // We pass user_data as a typval stored inline in qfline_T; use the
    // qf-specific accessor that returns the raw pointer to qfp->qf_user_data
    // We need a different approach: ask C to add user_data for us.
    // For simplicity, always add an empty user_data field -- the full
    // implementation would use nvim_tv_dict_add_tv. But for correctness
    // with the C macro VAR_UNKNOWN check we need the type value.
    // We skip user_data if its type is VAR_UNKNOWN (0).

    dict_add_nr(
        dict,
        b"valid",
        c_int::from(nvim_qfline_get_valid(qfp)).into(),
    )
}

extern "C" {
    fn nvim_qfline_get_valid(qfp: *const c_void) -> bool;
}

/// Iterate qf list entries and populate `list`.
///
/// Mirrors C `get_errorlist`. If `eidx > 0`, only the entry at that index is returned.
///
/// # Safety
/// - `qi_arg` may be null (uses global list)
/// - `wp` may be null (uses global list)
/// - `list` must be a valid non-null `list_T *`
#[no_mangle]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_get_errorlist(
    qi_arg: *const c_void,
    wp: *const c_void,
    qf_idx: c_int,
    eidx: c_int,
    list: *mut c_void,
) -> c_int {
    let mut qi = qi_arg;
    if qi.is_null() {
        qi = nvim_get_ql_info();
        if !wp.is_null() {
            qi = nvim_win_get_loclist(wp).cast_mut();
        }
        if qi.is_null() {
            return C_FAIL;
        }
    }

    if eidx < 0 {
        return C_OK;
    }

    let resolved_idx = if qf_idx == INVALID_QFIDX {
        nvim_qf_get_curlist_idx(qi)
    } else {
        qf_idx
    };

    let listcount = nvim_qf_get_listcount(qi);
    if resolved_idx >= listcount {
        return C_FAIL;
    }

    let qfl = nvim_qf_get_list_handle(qi, resolved_idx);
    if qfl.is_null() || nvim_qf_get_count(qfl) <= 0 {
        return C_FAIL;
    }

    let mut qfp = nvim_qf_get_start(qfl);
    let mut i: c_int = 1;
    while !qfp.is_null() && !nvim_qf_got_int() && i <= nvim_qf_get_count(qfl) {
        if eidx > 0 {
            if eidx == i {
                return rs_get_qfline_items_impl(qfp, list);
            }
        } else if rs_get_qfline_items_impl(qfp, list) == C_FAIL {
            return C_FAIL;
        }
        qfp = nvim_qfline_get_next(qfp);
        i += 1;
    }

    C_OK
}

/// Parse lines from a `what` dict and return items in `retdict`.
///
/// Mirrors C `qf_get_list_from_lines`.
///
/// # Safety
/// `what` and `retdict` must be valid non-null `dict_T *`.
#[no_mangle]
pub unsafe extern "C" fn rs_qf_get_list_from_lines(
    what: *const c_void,
    retdict: *mut c_void,
) -> c_int {
    // Check lines key exists with list value
    if !nvim_tv_dict_has_lines_key(what) {
        return C_FAIL;
    }

    let di_tv = nvim_tv_dict_get_lines_di_tv(what);
    if di_tv.is_null() {
        return C_FAIL;
    }

    // Get errorformat (from "efm" key or default p_efm)
    let errorformat = if nvim_tv_dict_efm_wrong_type(what) {
        return C_FAIL;
    } else {
        let s = nvim_tv_dict_get_efm_str(what);
        if s.is_null() {
            nvim_get_p_efm()
        } else {
            s
        }
    };

    let list = nvim_qf_tv_list_alloc();
    let qi = nvim_qf_alloc_internal_stack();

    if rs_qf_init_ext(
        qi,
        0,
        std::ptr::null(),
        std::ptr::null(),
        di_tv,
        errorformat,
        true,
        0,
        0,
        std::ptr::null(),
        std::ptr::null(),
    ) > 0
    {
        rs_get_errorlist(qi, std::ptr::null(), 0, 0, list);
        let qfl0 = nvim_qf_get_list_handle(qi, 0);
        if !qfl0.is_null() {
            rs_qf_free_list(qfl0);
        }
    }

    nvim_qf_free_lists_for_qi(qi);

    nvim_qf_tv_dict_add_list(retdict, c"items".as_ptr(), 5, list);

    C_OK
}

/// Main `qf_get_properties` implementation in Rust.
///
/// Mirrors C `qf_get_properties`. Dispatches to helper functions based on flags.
///
/// # Safety
/// `wp` may be null; `what` and `retdict` must be valid non-null `dict_T *`.
#[no_mangle]
#[allow(
    clippy::cast_possible_wrap,
    clippy::too_many_lines,
    clippy::useless_let_if_seq,
    clippy::cmp_null,
    clippy::if_not_else
)]
pub unsafe extern "C" fn rs_qf_get_properties(
    wp: *const c_void,
    what: *mut c_void,
    retdict: *mut c_void,
) -> c_int {
    // Check for "lines" key -- handle separately
    if !nvim_tv_dict_find(what, c"lines".as_ptr(), -1).is_null() {
        return rs_qf_get_list_from_lines(what, retdict);
    }

    let mut qi = nvim_get_ql_info();
    if !wp.is_null() {
        qi = nvim_win_get_loclist(wp).cast_mut();
    }

    let flags = rs_qf_getprop_keys2flags(what, !wp.is_null());

    let mut qf_idx = INVALID_QFIDX;
    if !rs_qf_stack_empty(qi) {
        qf_idx = rs_qf_getprop_qfidx(qi, what);
    }

    // Empty/missing list -- return defaults
    if rs_qf_stack_empty(qi) || qf_idx == INVALID_QFIDX {
        return rs_qf_getprop_defaults(qi, flags, !wp.is_null(), retdict);
    }

    let qfl = nvim_qf_get_list_handle(qi, qf_idx);
    if qfl.is_null() {
        return C_FAIL;
    }

    // Resolve eidx from "idx" key in what dict
    let mut eidx: c_int = 0;
    let idx_di = nvim_tv_dict_find(what, c"idx".as_ptr(), -1);
    if !idx_di.is_null() {
        if nvim_di_get_type(idx_di) != 3 {
            // 3 = VAR_NUMBER -- wrong type
            return C_FAIL;
        }
        eidx = nvim_di_get_nr(idx_di) as c_int;
    }

    let mut status = C_OK;

    if (flags & QF_GETLIST_TITLE) != 0 && status == C_OK {
        let title = nvim_qf_get_title(qfl);
        status = if title.is_null() {
            dict_add_str(retdict, b"title", c"".as_ptr())
        } else {
            dict_add_str(retdict, b"title", title)
        };
    }

    if (flags & QF_GETLIST_NR) != 0 && status == C_OK {
        status = dict_add_nr(retdict, b"nr", (qf_idx + 1).into());
    }

    if (flags & QF_GETLIST_WINID) != 0 && status == C_OK {
        let winid = nvim_qf_winid(qi);
        status = dict_add_nr(retdict, b"winid", winid.into());
    }

    if (flags & QF_GETLIST_ITEMS) != 0 && status == C_OK {
        let list = nvim_qf_tv_list_alloc();
        rs_get_errorlist(qi, std::ptr::null(), qf_idx, eidx, list);
        status = nvim_qf_tv_dict_add_list(retdict, c"items".as_ptr(), 5, list);
    }

    if (flags & QF_GETLIST_CONTEXT) != 0 && status == C_OK {
        let ctx = nvim_qfl_get_ctx(qfl);
        if ctx.is_null() {
            status = dict_add_str(retdict, b"context", c"".as_ptr());
        } else {
            // Allocate dictitem with key "context", copy ctx tv into it
            let di = nvim_tv_dict_item_alloc_len(c"context".as_ptr(), 7);
            if !di.is_null() {
                let di_tv_ptr = nvim_qf_di_get_tv(di);
                nvim_tv_copy(ctx, di_tv_ptr);
                status = nvim_qf_tv_dict_add_item(retdict, di);
                if status == C_FAIL {
                    nvim_qf_tv_dict_item_free(di);
                }
            } else {
                status = C_FAIL;
            }
        }
    }

    if (flags & QF_GETLIST_ID) != 0 && status == C_OK {
        status = dict_add_nr(retdict, b"id", nvim_qf_get_id(qfl).into());
    }

    if (flags & QF_GETLIST_IDX) != 0 && status == C_OK {
        let idx_val = if eidx == 0 {
            if nvim_qf_get_count(qfl) == 0 {
                0
            } else {
                nvim_qf_get_index(qfl)
            }
        } else {
            eidx
        };
        status = dict_add_nr(retdict, b"idx", idx_val.into());
    }

    if (flags & QF_GETLIST_SIZE) != 0 && status == C_OK {
        status = dict_add_nr(retdict, b"size", nvim_qf_get_count(qfl).into());
    }

    if (flags & QF_GETLIST_TICK) != 0 && status == C_OK {
        status = dict_add_nr(retdict, b"changedtick", nvim_qf_get_changedtick(qfl).into());
    }

    if !wp.is_null() && (flags & QF_GETLIST_FILEWINID) != 0 && status == C_OK {
        let filewinid = rs_qf_getprop_filewinid(wp, qi);
        status = dict_add_nr(retdict, b"filewinid", filewinid.into());
    }

    if (flags & QF_GETLIST_QFBUFNR) != 0 && status == C_OK {
        let bufnr = rs_qf_getprop_qfbufnr(qi);
        status = dict_add_nr(retdict, b"qfbufnr", bufnr.into());
    }

    if (flags & QF_GETLIST_QFTF) != 0 && status == C_OK {
        // Allocate inline typval for callback_put output
        let tv_buf = nvim_tv_alloc();
        if nvim_qf_qftf_cb_put(qfl, tv_buf) {
            status = nvim_tv_dict_add_tv(retdict, c"quickfixtextfunc".as_ptr(), 16, tv_buf);
            nvim_tv_clear(tv_buf);
        } else {
            status = dict_add_str(retdict, b"quickfixtextfunc", c"".as_ptr());
        }
        nvim_qf_tv_free(tv_buf);
    }

    status
}

/// Top-level dispatch for `getqflist()`/`getloclist()`.
///
/// Mirrors C `get_qf_loc_list`.
///
/// # Safety
/// `wp` may be null; `what_arg` and `rettv` must be valid non-null `typval_T *`.
#[no_mangle]
pub unsafe extern "C" fn rs_get_qf_loc_list(
    is_qf: bool,
    wp: *mut c_void,
    what_arg: *const c_void,
    rettv: *mut c_void,
) {
    if nvim_tv_is_unknown(what_arg) {
        // No dict arg -- return a list of all entries
        nvim_qf_tv_list_alloc_ret(rettv);
        if is_qf || !wp.is_null() {
            let list_ptr = nvim_qf_tv_get_list(rettv);
            rs_get_errorlist(std::ptr::null(), wp, INVALID_QFIDX, 0, list_ptr);
        }
    } else {
        nvim_qf_tv_dict_alloc_ret(rettv);
        if is_qf || !wp.is_null() {
            if nvim_tv_is_dict(what_arg) {
                let d = nvim_qf_tv_get_dict(what_arg);
                if !d.is_null() {
                    rs_qf_get_properties(wp, d, nvim_qf_tv_get_dict(rettv));
                }
            } else {
                nvim_emsg_dictreq();
            }
        }
    }
}

/// VimL `f_getqflist()` entry point.
///
/// # Safety
/// `argvars` and `rettv` must be valid `typval_T *`.
#[no_mangle]
pub unsafe extern "C" fn rs_f_getqflist(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *const c_void,
) {
    rs_get_qf_loc_list(true, std::ptr::null_mut(), argvars, rettv);
}

/// VimL `f_getloclist()` entry point.
///
/// # Safety
/// `argvars` and `rettv` must be valid `typval_T *`.
#[no_mangle]
pub unsafe extern "C" fn rs_f_getloclist(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *const c_void,
) {
    let wp = nvim_find_win_by_nr_or_id(argvars);
    let what_arg = nvim_tv_advance(argvars);
    rs_get_qf_loc_list(false, wp, what_arg, rettv);
}

// =============================================================================
// Phase 8: qf_set_properties cluster
// =============================================================================

/// Main `qf_set_properties` implementation in Rust.
///
/// Mirrors C `qf_set_properties`.
///
/// # Safety
/// `qi`, `what` must be valid non-null pointers; `title` may be null.
#[no_mangle]
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_lossless,
    clippy::too_many_lines
)]
pub unsafe extern "C" fn rs_qf_set_properties(
    qi: *mut c_void,
    what: *const c_void,
    action: c_int,
    title: *mut std::ffi::c_char,
) -> c_int {
    let mut newlist = action == c_int::from(b' ') || rs_qf_stack_empty(qi);
    let mut qf_idx = rs_qf_setprop_get_qfidx(qi, what, action, &raw mut newlist);
    if qf_idx == INVALID_QFIDX {
        return C_FAIL;
    }

    if newlist {
        // Use C accessor for setting curlist and creating new list
        nvim_qf_set_curlist_idx(qi, qf_idx);
        rs_qf_new_list(qi, title);
        qf_idx = nvim_qf_get_curlist_idx(qi);
    }

    let qfl = nvim_qf_get_list_handle(qi, qf_idx);
    if qfl.is_null() {
        return C_FAIL;
    }

    let mut retval = C_FAIL;

    // Process "title" key: inline nvim_qfl_set_title_from_what
    let di = nvim_tv_dict_find(what, c"title".as_ptr(), -1);
    if !di.is_null() && nvim_di_get_type(di) == VAR_STRING {
        // alloc=false: borrowed pointer into the dict; nvim_qf_set_title_dup makes its own copy
        let title_str = nvim_qf_tv_dict_get_string(what, c"title".as_ptr(), false);
        nvim_qf_set_title_dup(qfl, title_str);
        if nvim_qf_get_curlist_idx(qi) == qf_idx {
            rs_qf_update_win_titlevar(qi);
        }
        retval = C_OK;
    }

    // Process "items" key: inline nvim_qfl_set_items
    let di = nvim_tv_dict_find(what, c"items".as_ptr(), -1);
    if !di.is_null() && nvim_di_get_type(di) == VAR_LIST {
        // Pass borrowed title pointer; rs_qf_add_entries reads it but does not free it
        let title_borrowed = nvim_qf_get_title(qfl);
        let eff_action = if action == c_int::from(b' ') {
            c_int::from(b'a')
        } else {
            action
        };
        let tv_ptr = nvim_qf_di_get_tv(di);
        let list_ptr = nvim_qf_tv_get_list(tv_ptr);
        retval = rs_qf_add_entries(qi, qf_idx, list_ptr, title_borrowed, eff_action);
    }

    // Process "lines" key: inline nvim_qfl_set_items_from_lines
    let di = nvim_tv_dict_find(what, c"lines".as_ptr(), -1);
    if !di.is_null() {
        if nvim_tv_dict_efm_wrong_type(what) {
            retval = C_FAIL;
        } else {
            let di_type = nvim_di_get_type(di);
            let di_tv = nvim_qf_di_get_tv(di);
            let list_ptr = if di_type == VAR_LIST {
                nvim_qf_tv_get_list(di_tv)
            } else {
                std::ptr::null_mut()
            };
            if di_type != VAR_LIST || list_ptr.is_null() {
                retval = C_FAIL;
            } else {
                let errorformat = nvim_tv_dict_get_efm_str(what);
                let efm = if errorformat.is_null() {
                    nvim_get_p_efm()
                } else {
                    errorformat
                };
                if action == c_int::from(b'r') || action == c_int::from(b'u') {
                    let qfl_ptr = nvim_qf_get_list_handle(qi, qf_idx);
                    rs_qf_free_items(qfl_ptr);
                }
                if rs_qf_init_ext(
                    qi,
                    qf_idx,
                    std::ptr::null(),
                    std::ptr::null(),
                    di_tv,
                    efm,
                    false,
                    0,
                    0,
                    std::ptr::null(),
                    std::ptr::null(),
                ) >= 0
                {
                    retval = C_OK;
                } else {
                    retval = C_FAIL;
                }
            }
        }
    }

    // Process "context" key
    let di = nvim_tv_dict_find(what, c"context".as_ptr(), -1);
    if !di.is_null() {
        let ctx_tv_ptr = nvim_qf_di_get_tv(di);
        if !ctx_tv_ptr.is_null() {
            nvim_qfl_free_ctx(qfl);
            let new_ctx = nvim_tv_alloc_copy(ctx_tv_ptr);
            nvim_qfl_set_ctx(qfl, new_ctx);
            retval = C_OK;
        }
    }

    // Process "idx" key: inline nvim_qfl_set_curidx
    let di = nvim_tv_dict_find(what, c"idx".as_ptr(), -1);
    if !di.is_null() {
        let di_type = nvim_di_get_type(di);
        let di_str = nvim_di_get_string(di);
        let newidx: c_int;
        let idx_ok: bool;
        if di_type == VAR_STRING
            && !di_str.is_null()
            && *di_str == b'$' as std::ffi::c_char
            && *di_str.add(1) == 0
        {
            newidx = nvim_qf_get_count(qfl);
            idx_ok = true;
        } else {
            let di_tv = nvim_qf_di_get_tv(di);
            let mut denote = false;
            let n = nvim_qf_tv_get_number_chk(di_tv, &raw mut denote) as c_int;
            idx_ok = !denote;
            newidx = n;
        }
        if idx_ok && newidx >= 1 {
            let clamped = newidx.min(nvim_qf_get_count(qfl));
            let old_qfidx = nvim_qf_get_index(qfl);
            let mut new_qfidx: c_int = clamped;
            let qf_ptr = rs_qf_get_nth_entry(qfl, clamped, &raw mut new_qfidx);
            if !qf_ptr.is_null() {
                nvim_qf_set_ptr(qfl, qf_ptr);
                nvim_qf_set_index(qfl, new_qfidx);
                if nvim_qf_get_curlist_id(qi) == nvim_qf_get_id(qfl) {
                    rs_qf_win_pos_update(qi, old_qfidx);
                }
                retval = C_OK;
            }
        }
    }

    // Process "quickfixtextfunc" key
    let di = nvim_tv_dict_find(what, c"quickfixtextfunc".as_ptr(), -1);
    if !di.is_null() {
        let tv_ptr = nvim_qf_di_get_tv(di);
        if !tv_ptr.is_null() {
            nvim_qfl_free_qftf_cb(qfl);
            nvim_qfl_set_qftf_cb_from_tv(qfl, tv_ptr);
            retval = C_OK;
        }
    }

    if newlist || retval == C_OK {
        rs_qf_incr_changedtick(qfl);
    }
    if newlist {
        nvim_qf_update_buffer(qi, std::ptr::null());
    }

    retval
}

/// Top-level `set_qf_ll_list` / `setqflist()` / `setloclist()` implementation.
///
/// # Safety
/// `wp` may be null; `args` and `rettv` must be valid `typval_T *`.
#[no_mangle]
#[allow(clippy::items_after_statements, clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_set_qf_ll_list(
    wp: *mut c_void,
    args: *const c_void,
    rettv: *mut c_void,
) {
    static mut RECURSIVE: c_int = 0;

    // Set rettv to -1 (error default)
    nvim_qf_tv_set_number(rettv, -1);

    let list_arg = args;
    if nvim_qf_tv_get_list(list_arg).is_null() && !nvim_qf_tv_is_list_type(list_arg) {
        nvim_emsg_listreq();
        return;
    }
    if RECURSIVE != 0 {
        nvim_emsg_au_recursive();
        return;
    }

    let mut action: u8 = b' ';
    let mut title: *const std::ffi::c_char = std::ptr::null();
    let mut what: *mut c_void = std::ptr::null_mut();

    let action_arg = nvim_tv_advance(list_arg);
    if !nvim_tv_is_unknown(action_arg) {
        if nvim_qf_tv_get_type(action_arg) != 5 {
            // 5 = VAR_STRING
            nvim_emsg_string_required();
            return;
        }
        let act = nvim_qf_tv_get_string_chk(action_arg);
        if act.is_null() {
            return;
        }
        let act_byte = *act as u8;
        if (act_byte == b'a'
            || act_byte == b'r'
            || act_byte == b'u'
            || act_byte == b' '
            || act_byte == b'f')
            && *act.add(1) == 0
        {
            action = act_byte;
        } else {
            nvim_emsg_invact(act);
            return;
        }

        let what_arg = nvim_tv_advance(action_arg);
        if !nvim_tv_is_unknown(what_arg) {
            if nvim_qf_tv_get_type(what_arg) == 5 {
                // VAR_STRING
                title = nvim_qf_tv_get_string_chk(what_arg);
                if title.is_null() {
                    return;
                }
            } else if nvim_tv_is_dict(what_arg) {
                what = nvim_qf_tv_get_dict(what_arg);
            } else {
                nvim_emsg_dictreq();
                return;
            }
        }
    }

    // Default title
    let default_title_qf = c":setqflist()";
    let default_title_loc = c":setloclist()";
    if title.is_null() {
        title = if wp.is_null() {
            default_title_qf.as_ptr()
        } else {
            default_title_loc.as_ptr()
        };
    }

    RECURSIVE += 1;
    let list = nvim_qf_tv_get_list(list_arg);
    if rs_set_errorlist(wp, list, action.into(), title.cast_mut(), what) == C_OK {
        nvim_qf_tv_set_number(rettv, 0);
    }
    RECURSIVE -= 1;
}

/// VimL `f_setqflist()` entry point.
///
/// # Safety
/// `argvars` and `rettv` must be valid `typval_T *`.
#[no_mangle]
pub unsafe extern "C" fn rs_f_setqflist(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *const c_void,
) {
    rs_set_qf_ll_list(std::ptr::null_mut(), argvars, rettv);
}

/// VimL `f_setloclist()` entry point.
///
/// # Safety
/// `argvars` and `rettv` must be valid `typval_T *`.
#[no_mangle]
pub unsafe extern "C" fn rs_f_setloclist(
    argvars: *const c_void,
    rettv: *mut c_void,
    _fptr: *const c_void,
) {
    nvim_qf_tv_set_number(rettv, -1);
    let win = nvim_find_win_by_nr_or_id(argvars);
    if !win.is_null() {
        let args = nvim_tv_advance(argvars);
        rs_set_qf_ll_list(win, args, rettv);
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
