//! Quickfix/Location list management operations.
//!
//! This module provides Rust implementations for creating, modifying, and
//! managing quickfix and location lists, including stack navigation.

#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::use_self)]

use std::ffi::{c_char, c_int, c_void};

/// Line number type (matches `linenr_T` in Neovim)
type LinenrT = i32;

/// Opaque handle to `qf_info_T` (quickfix stack)
type QfInfoHandle = *const c_void;
type QfInfoHandleMut = *mut c_void;

/// Opaque handle to `qf_list_T` (quickfix list)
type QfListHandle = *const c_void;
type QfListHandleMut = *mut c_void;

/// Opaque handle to `qfline_T` (quickfix entry)
type QfLineHandle = *const c_void;

// =============================================================================
// External C accessor functions
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Stack accessors
    fn nvim_qf_get_listcount(qi: QfInfoHandle) -> c_int;
    fn nvim_qf_get_curlist(qi: QfInfoHandle) -> QfListHandle;
    fn nvim_qf_get_list_at(qi: QfInfoHandle, idx: c_int) -> QfListHandle;
    fn nvim_qf_get_curlist_idx(qi: QfInfoHandle) -> c_int;
    fn nvim_qf_set_curlist_idx(qi: QfInfoHandleMut, idx: c_int);

    // List accessors
    fn nvim_qf_get_count(qfl: QfListHandle) -> c_int;
    fn nvim_qf_get_index(qfl: QfListHandle) -> c_int;
    fn nvim_qf_get_start(qfl: QfListHandle) -> QfLineHandle;
    fn nvim_qf_get_ptr(qfl: QfListHandle) -> QfLineHandle;
    fn nvim_qf_get_last(qfl: QfListHandle) -> QfLineHandle;
    fn nvim_qf_get_id(qfl: QfListHandle) -> u32;
    fn nvim_qf_get_changedtick(qfl: QfListHandle) -> c_int;
    fn nvim_qf_get_title(qfl: QfListHandle) -> *const c_char;
    fn nvim_qf_get_nonevalid(qfl: QfListHandle) -> bool;

    fn nvim_qf_set_index(qfl: QfListHandleMut, idx: c_int);
    fn nvim_qf_set_ptr(qfl: QfListHandleMut, ptr: QfLineHandle);

    // Entry accessors
    fn nvim_qfline_get_next(qfp: QfLineHandle) -> QfLineHandle;
    fn nvim_qfline_get_prev(qfp: QfLineHandle) -> QfLineHandle;
    fn nvim_qfline_get_fnum(qfp: QfLineHandle) -> c_int;
    fn nvim_qfline_get_lnum(qfp: QfLineHandle) -> LinenrT;
    fn nvim_qfline_get_col(qfp: QfLineHandle) -> c_int;
    fn nvim_qfline_get_end_lnum(qfp: QfLineHandle) -> LinenrT;
    fn nvim_qfline_get_end_col(qfp: QfLineHandle) -> c_int;
    fn nvim_qfline_get_nr(qfp: QfLineHandle) -> c_int;
    fn nvim_qfline_get_valid(qfp: QfLineHandle) -> bool;
    fn nvim_qfline_get_type(qfp: QfLineHandle) -> c_char;
    fn nvim_qfline_get_viscol(qfp: QfLineHandle) -> bool;

    // List operations
    fn nvim_qf_store_title(qfl: QfListHandleMut, title: *const c_char);

    // Stack maxcount
    fn nvim_qf_get_maxcount(qi: QfInfoHandle) -> c_int;

    // Entry modification
    fn nvim_qfline_get_cleared(qfp: QfLineHandle) -> bool;
    fn nvim_qfline_set_cleared(qfp: *mut c_void, cleared: c_char);

    // Phase 2: qf_add_entries accessors
    fn nvim_qf_get_list_at_mut(qi: QfInfoHandleMut, idx: c_int) -> QfListHandleMut;
    fn nvim_qf_set_nonevalid(qfl: QfListHandleMut, nonevalid: bool);
    fn nvim_qf_update_buffer(qi: QfInfoHandleMut, old_last: QfLineHandle);
    fn nvim_qf_get_ptr_position(
        qfl: QfListHandle,
        fnum: *mut c_int,
        lnum: *mut c_int,
        col: *mut c_int,
    );
    fn nvim_tv_list_first(list: *const c_void) -> *const c_void;
    fn nvim_tv_list_item_next(list: *const c_void, li: *const c_void) -> *const c_void;
    fn nvim_tv_list_item_dict(li: *const c_void) -> *mut c_void;
    fn nvim_tv_list_item_is_first(list: *const c_void, li: *const c_void) -> bool;
    fn nvim_qf_add_entry_from_dict(
        qfl: QfListHandleMut,
        d: *mut c_void,
        first_entry: bool,
        valid_entry: *mut bool,
    ) -> c_int;
    fn rs_qf_list_empty(qfl: QfListHandle) -> bool;
    fn rs_qf_entry_is_closer_to_target(
        entry: QfLineHandle,
        other: QfLineHandle,
        fnum: c_int,
        lnum: c_int,
        col: c_int,
    ) -> bool;
}

// =============================================================================
// List Statistics
// =============================================================================

/// Statistics about a quickfix list.
#[repr(C)]
#[derive(Default)]
pub struct QfListStats {
    /// Total number of entries in the list
    pub total_entries: c_int,
    /// Number of valid entries
    pub valid_entries: c_int,
    /// Number of unique files referenced
    pub file_count: c_int,
    /// Current entry index (1-based)
    pub current_index: c_int,
    /// List ID
    pub list_id: u32,
    /// Changed tick counter
    pub changedtick: c_int,
    /// Whether the list has a title
    pub has_title: bool,
    /// Whether all entries are invalid
    pub all_invalid: bool,
}

/// Get comprehensive statistics about a quickfix list.
///
/// # Safety
///
/// - `qfl` may be null (returns default statistics)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_list_stats(qfl: QfListHandle) -> QfListStats {
    if qfl.is_null() {
        return QfListStats::default();
    }

    let total_entries = nvim_qf_get_count(qfl);
    let current_index = nvim_qf_get_index(qfl);
    let list_id = nvim_qf_get_id(qfl);
    let changedtick = nvim_qf_get_changedtick(qfl);
    let title = nvim_qf_get_title(qfl);
    let has_title = !title.is_null() && *title != 0;
    let all_invalid = nvim_qf_get_nonevalid(qfl);

    // Count valid entries and unique files
    let mut valid_entries = 0;
    let mut file_count = 0;
    let mut last_fnum = -1;

    let mut qfp = nvim_qf_get_start(qfl);
    while !qfp.is_null() {
        if nvim_qfline_get_valid(qfp) {
            valid_entries += 1;
        }
        let fnum = nvim_qfline_get_fnum(qfp);
        if fnum > 0 && fnum != last_fnum {
            file_count += 1;
            last_fnum = fnum;
        }
        qfp = nvim_qfline_get_next(qfp);
    }

    QfListStats {
        total_entries,
        valid_entries,
        file_count,
        current_index,
        list_id,
        changedtick,
        has_title,
        all_invalid,
    }
}

// =============================================================================
// Stack Operations
// =============================================================================

/// Information about a quickfix stack.
#[repr(C)]
#[derive(Default)]
pub struct QfStackInfo {
    /// Number of lists in the stack
    pub list_count: c_int,
    /// Current list index (0-based)
    pub current_index: c_int,
    /// Maximum number of lists the stack can hold
    pub max_count: c_int,
    /// Whether the stack is at capacity
    pub at_capacity: bool,
    /// Whether the stack is empty
    pub is_empty: bool,
    /// Whether we can go to older list
    pub can_go_older: bool,
    /// Whether we can go to newer list
    pub can_go_newer: bool,
}

/// Get information about a quickfix stack.
///
/// # Safety
///
/// - `qi` may be null (returns default info with `is_empty=true`)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_stack_info(qi: QfInfoHandle) -> QfStackInfo {
    if qi.is_null() {
        return QfStackInfo {
            is_empty: true,
            ..Default::default()
        };
    }

    let list_count = nvim_qf_get_listcount(qi);
    let current_index = nvim_qf_get_curlist_idx(qi);
    let max_count = nvim_qf_get_maxcount(qi);

    let is_empty = list_count <= 0;
    let at_capacity = list_count >= max_count;
    let can_go_older = current_index > 0;
    let can_go_newer = current_index < list_count - 1;

    QfStackInfo {
        list_count,
        current_index,
        max_count,
        at_capacity,
        is_empty,
        can_go_older,
        can_go_newer,
    }
}

/// Get the number of entries in the current list.
///
/// # Safety
///
/// - `qi` may be null (returns 0)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_curlist_entry_count(qi: QfInfoHandle) -> c_int {
    if qi.is_null() {
        return 0;
    }

    let qfl = nvim_qf_get_curlist(qi);
    if qfl.is_null() {
        return 0;
    }

    nvim_qf_get_count(qfl)
}

/// Get the number of valid entries in the current list.
///
/// # Safety
///
/// - `qi` may be null (returns 0)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_curlist_valid_count(qi: QfInfoHandle) -> c_int {
    if qi.is_null() {
        return 0;
    }

    let qfl = nvim_qf_get_curlist(qi);
    if qfl.is_null() {
        return 0;
    }

    // Count valid entries
    let mut count = 0;
    let mut qfp = nvim_qf_get_start(qfl);
    while !qfp.is_null() {
        if nvim_qfline_get_valid(qfp) {
            count += 1;
        }
        qfp = nvim_qfline_get_next(qfp);
    }

    count
}

// =============================================================================
// List Navigation
// =============================================================================

/// Move to the next older list in the stack.
///
/// Returns the new list index (0-based) or -1 if at the oldest list.
///
/// # Safety
///
/// - `qi` may be null (returns -1)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_older_list(qi: QfInfoHandleMut) -> c_int {
    if qi.is_null() {
        return -1;
    }

    let current = nvim_qf_get_curlist_idx(qi);
    if current <= 0 {
        return -1;
    }

    let new_idx = current - 1;
    nvim_qf_set_curlist_idx(qi, new_idx);
    new_idx
}

/// Move to the next newer list in the stack.
///
/// Returns the new list index (0-based) or -1 if at the newest list.
///
/// # Safety
///
/// - `qi` may be null (returns -1)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_newer_list(qi: QfInfoHandleMut) -> c_int {
    if qi.is_null() {
        return -1;
    }

    let current = nvim_qf_get_curlist_idx(qi);
    let count = nvim_qf_get_listcount(qi);
    if current >= count - 1 {
        return -1;
    }

    let new_idx = current + 1;
    nvim_qf_set_curlist_idx(qi, new_idx);
    new_idx
}

/// Move to a specific list by index (0-based).
///
/// Returns true if successful, false if the index is invalid.
///
/// # Safety
///
/// - `qi` may be null (returns false)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_goto_list_idx(qi: QfInfoHandleMut, idx: c_int) -> bool {
    if qi.is_null() {
        return false;
    }

    let count = nvim_qf_get_listcount(qi);
    if idx < 0 || idx >= count {
        return false;
    }

    nvim_qf_set_curlist_idx(qi, idx);
    true
}

/// Move to the oldest list in the stack.
///
/// Returns the list index (0) or -1 if the stack is empty.
///
/// # Safety
///
/// - `qi` may be null (returns -1)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_oldest_list(qi: QfInfoHandleMut) -> c_int {
    if qi.is_null() {
        return -1;
    }

    let count = nvim_qf_get_listcount(qi);
    if count <= 0 {
        return -1;
    }

    nvim_qf_set_curlist_idx(qi, 0);
    0
}

/// Move to the newest list in the stack.
///
/// Returns the list index or -1 if the stack is empty.
///
/// # Safety
///
/// - `qi` may be null (returns -1)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_newest_list(qi: QfInfoHandleMut) -> c_int {
    if qi.is_null() {
        return -1;
    }

    let count = nvim_qf_get_listcount(qi);
    if count <= 0 {
        return -1;
    }

    let new_idx = count - 1;
    nvim_qf_set_curlist_idx(qi, new_idx);
    new_idx
}

// =============================================================================
// Entry Position Management
// =============================================================================

/// Reset the current entry to the first entry in the list.
///
/// Returns the new index (1) or 0 if the list is empty.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_reset_to_first(qfl: QfListHandleMut) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let start = nvim_qf_get_start(qfl);
    if start.is_null() {
        return 0;
    }

    nvim_qf_set_ptr(qfl, start);
    nvim_qf_set_index(qfl, 1);
    1
}

/// Reset the current entry to the last entry in the list.
///
/// Returns the new index or 0 if the list is empty.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_reset_to_last(qfl: QfListHandleMut) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let last = nvim_qf_get_last(qfl);
    if last.is_null() {
        return 0;
    }

    let count = nvim_qf_get_count(qfl);
    nvim_qf_set_ptr(qfl, last);
    nvim_qf_set_index(qfl, count);
    count
}

/// Check if the current entry is at the first entry.
///
/// # Safety
///
/// - `qfl` may be null (returns true - empty list is at beginning)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_at_first(qfl: QfListHandle) -> bool {
    if qfl.is_null() {
        return true;
    }

    nvim_qf_get_index(qfl) <= 1
}

/// Check if the current entry is at the last entry.
///
/// # Safety
///
/// - `qfl` may be null (returns true - empty list is at end)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_at_last(qfl: QfListHandle) -> bool {
    if qfl.is_null() {
        return true;
    }

    let idx = nvim_qf_get_index(qfl);
    let count = nvim_qf_get_count(qfl);
    idx >= count
}

// =============================================================================
// List Comparison
// =============================================================================

/// Compare two lists by their ID.
///
/// Returns true if both lists have the same ID (or both are null).
///
/// # Safety
///
/// - Either or both handles may be null
#[no_mangle]
pub unsafe extern "C" fn rs_qf_lists_same(a: QfListHandle, b: QfListHandle) -> bool {
    match (a.is_null(), b.is_null()) {
        (true, true) => true,
        (true, false) | (false, true) => false,
        (false, false) => nvim_qf_get_id(a) == nvim_qf_get_id(b),
    }
}

/// Check if a list has been modified since a given changedtick.
///
/// # Safety
///
/// - `qfl` may be null (returns false)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_list_modified_since(qfl: QfListHandle, tick: c_int) -> bool {
    if qfl.is_null() {
        return false;
    }

    nvim_qf_get_changedtick(qfl) > tick
}

// =============================================================================
// Entry Bounds Checking
// =============================================================================

/// Check if an entry index is valid for the list.
///
/// Entry indices are 1-based.
///
/// # Safety
///
/// - `qfl` may be null (returns false)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_valid_entry_idx(qfl: QfListHandle, idx: c_int) -> bool {
    if qfl.is_null() {
        return false;
    }

    let count = nvim_qf_get_count(qfl);
    idx >= 1 && idx <= count
}

/// Clamp an entry index to valid bounds.
///
/// Returns the clamped index (1 to count), or 0 if the list is empty.
///
/// # Safety
///
/// - `qfl` may be null (returns 0)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_clamp_entry_idx(qfl: QfListHandle, idx: c_int) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let count = nvim_qf_get_count(qfl);
    if count == 0 {
        return 0;
    }

    idx.clamp(1, count)
}

// =============================================================================
// List Capacity
// =============================================================================

/// Check if there's room for more lists in the stack.
///
/// # Safety
///
/// - `qi` may be null (returns false)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_stack_has_room(qi: QfInfoHandle) -> bool {
    if qi.is_null() {
        return false;
    }

    let count = nvim_qf_get_listcount(qi);
    let max = nvim_qf_get_maxcount(qi);
    count < max
}

/// Get the number of available slots in the stack.
///
/// # Safety
///
/// - `qi` may be null (returns 0)
/// - If non-null, `qi` must be a valid pointer to a `qf_info_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_stack_available(qi: QfInfoHandle) -> c_int {
    if qi.is_null() {
        return 0;
    }

    let count = nvim_qf_get_listcount(qi);
    let max = nvim_qf_get_maxcount(qi);
    if max > count {
        max - count
    } else {
        0
    }
}

// =============================================================================
// Entry Types and Categories
// =============================================================================

/// Quickfix entry type classification.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QfEntryType {
    /// Error entry.
    Error = 0,
    /// Warning entry.
    Warning = 1,
    /// Informational entry.
    Info = 2,
    /// Note entry.
    Note = 3,
    /// Other/unknown type.
    Other = 4,
}

impl QfEntryType {
    /// Convert from type character.
    pub const fn from_char(c: c_char) -> Self {
        match c as u8 {
            b'E' | b'e' => QfEntryType::Error,
            b'W' | b'w' => QfEntryType::Warning,
            b'I' | b'i' => QfEntryType::Info,
            b'N' | b'n' => QfEntryType::Note,
            _ => QfEntryType::Other,
        }
    }

    /// Get the type character.
    pub const fn to_char(self) -> c_char {
        match self {
            QfEntryType::Error => b'E' as c_char,
            QfEntryType::Warning => b'W' as c_char,
            QfEntryType::Info => b'I' as c_char,
            QfEntryType::Note => b'N' as c_char,
            QfEntryType::Other => b' ' as c_char,
        }
    }

    /// Check if this is an error type.
    pub const fn is_error(self) -> bool {
        matches!(self, QfEntryType::Error)
    }

    /// Check if this is a warning type.
    pub const fn is_warning(self) -> bool {
        matches!(self, QfEntryType::Warning)
    }
}

/// Quickfix entry summary.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct QfEntrySummary {
    /// Line number (1-based).
    pub lnum: LinenrT,
    /// Column number (1-based, 0 if not specified).
    pub col: c_int,
    /// End line number (0 if not specified).
    pub end_lnum: LinenrT,
    /// End column (0 if not specified).
    pub end_col: c_int,
    /// Buffer number (0 if not associated).
    pub fnum: c_int,
    /// Entry type.
    pub entry_type: QfEntryType,
    /// Whether the entry is valid.
    pub valid: bool,
    /// Whether column is visual column.
    pub viscol: bool,
    /// Entry number/error number.
    pub nr: c_int,
}

impl QfEntrySummary {
    /// Create an empty summary.
    pub const fn empty() -> Self {
        Self {
            lnum: 0,
            col: 0,
            end_lnum: 0,
            end_col: 0,
            fnum: 0,
            entry_type: QfEntryType::Other,
            valid: false,
            viscol: false,
            nr: 0,
        }
    }

    /// Check if this entry has a range (multi-line/column).
    pub const fn has_range(&self) -> bool {
        self.end_lnum > 0 || self.end_col > 0
    }

    /// Check if this entry references a file.
    pub const fn has_file(&self) -> bool {
        self.fnum > 0
    }

    /// Check if this entry has a position.
    pub const fn has_position(&self) -> bool {
        self.lnum > 0
    }
}

impl Default for QfEntrySummary {
    fn default() -> Self {
        Self::empty()
    }
}

/// Get entry summary from a quickfix entry handle.
///
/// # Safety
///
/// - `qfp` may be null (returns empty summary)
/// - If non-null, `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_summary(qfp: QfLineHandle) -> QfEntrySummary {
    if qfp.is_null() {
        return QfEntrySummary::empty();
    }

    let type_char = nvim_qfline_get_type(qfp);
    QfEntrySummary {
        lnum: nvim_qfline_get_lnum(qfp),
        col: nvim_qfline_get_col(qfp),
        end_lnum: nvim_qfline_get_end_lnum(qfp),
        end_col: nvim_qfline_get_end_col(qfp),
        fnum: nvim_qfline_get_fnum(qfp),
        entry_type: QfEntryType::from_char(type_char),
        valid: nvim_qfline_get_valid(qfp),
        viscol: nvim_qfline_get_viscol(qfp),
        nr: nvim_qfline_get_nr(qfp),
    }
}

// =============================================================================
// Entry Categorization
// =============================================================================

/// Entry count by type.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct QfTypeCounts {
    /// Number of error entries.
    pub errors: c_int,
    /// Number of warning entries.
    pub warnings: c_int,
    /// Number of info entries.
    pub info: c_int,
    /// Number of note entries.
    pub notes: c_int,
    /// Number of other entries.
    pub other: c_int,
}

impl QfTypeCounts {
    /// Get total count.
    pub const fn total(&self) -> c_int {
        self.errors + self.warnings + self.info + self.notes + self.other
    }

    /// Check if there are any errors.
    pub const fn has_errors(&self) -> bool {
        self.errors > 0
    }

    /// Check if there are any warnings.
    pub const fn has_warnings(&self) -> bool {
        self.warnings > 0
    }
}

/// Count entries by type in a quickfix list.
///
/// # Safety
///
/// - `qfl` may be null (returns zero counts)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_type_counts(qfl: QfListHandle) -> QfTypeCounts {
    if qfl.is_null() {
        return QfTypeCounts::default();
    }

    let mut counts = QfTypeCounts::default();
    let mut qfp = nvim_qf_get_start(qfl);
    while !qfp.is_null() {
        let type_char = nvim_qfline_get_type(qfp);
        match QfEntryType::from_char(type_char) {
            QfEntryType::Error => counts.errors += 1,
            QfEntryType::Warning => counts.warnings += 1,
            QfEntryType::Info => counts.info += 1,
            QfEntryType::Note => counts.notes += 1,
            QfEntryType::Other => counts.other += 1,
        }
        qfp = nvim_qfline_get_next(qfp);
    }

    counts
}

// =============================================================================
// File Association Helpers
// =============================================================================

/// File reference in quickfix entry.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct QfFileRef {
    /// Buffer number (0 if none).
    pub fnum: c_int,
    /// Number of entries for this file.
    pub entry_count: c_int,
    /// First entry index for this file (1-based).
    pub first_entry: c_int,
}

impl QfFileRef {
    /// Create an empty file reference.
    pub const fn empty() -> Self {
        Self {
            fnum: 0,
            entry_count: 0,
            first_entry: 0,
        }
    }

    /// Check if this references a valid file.
    pub const fn has_file(&self) -> bool {
        self.fnum > 0
    }
}

impl Default for QfFileRef {
    fn default() -> Self {
        Self::empty()
    }
}

/// FFI export: Get entry type from character.
#[no_mangle]
pub extern "C" fn rs_qf_entry_type_from_char(c: c_char) -> QfEntryType {
    QfEntryType::from_char(c)
}

/// FFI export: Get character from entry type.
#[no_mangle]
pub extern "C" fn rs_qf_entry_type_to_char(entry_type: QfEntryType) -> c_char {
    entry_type.to_char()
}

/// FFI export: Check if entry has file reference.
#[no_mangle]
pub extern "C" fn rs_qf_entry_has_file(summary: QfEntrySummary) -> c_int {
    c_int::from(summary.has_file())
}

/// FFI export: Check if entry has range.
#[no_mangle]
pub extern "C" fn rs_qf_entry_has_range(summary: QfEntrySummary) -> c_int {
    c_int::from(summary.has_range())
}

// =============================================================================
// Phase Q2: Additional List Operations
// =============================================================================

/// Clear all entries from a list while preserving list metadata.
///
/// This resets entry pointers and count but keeps ID, title, and type.
///
/// # Safety
///
/// - `qfl` may be null (does nothing)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_clear_entries(qfl: QfListHandleMut) {
    if qfl.is_null() {
        return;
    }
    crate::rs_qf_free_items(qfl);
}

/// Mark an entry as cleared (deleted but still in list).
///
/// Cleared entries are skipped during navigation but remain in the list.
///
/// # Safety
///
/// - `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_mark_entry_cleared(qfp: QfLineHandle) {
    if qfp.is_null() {
        return;
    }
    nvim_qfline_set_cleared(qfp.cast_mut(), 1);
}

/// Check if an entry is cleared.
///
/// # Safety
///
/// - `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_is_cleared(qfp: QfLineHandle) -> bool {
    if qfp.is_null() {
        return false;
    }
    nvim_qfline_get_cleared(qfp)
}

/// Count entries in a specific file.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_file_entry_count(qfl: QfListHandle, fnum: c_int) -> c_int {
    if qfl.is_null() || fnum <= 0 {
        return 0;
    }

    let mut count = 0;
    let mut qfp = nvim_qf_get_start(qfl);

    while !qfp.is_null() {
        if nvim_qfline_get_fnum(qfp) == fnum {
            count += 1;
        }
        qfp = nvim_qfline_get_next(qfp);
    }

    count
}

// =============================================================================
// Phase 2: qf_add_entries
// =============================================================================

const OK: c_int = 1;
const QF_FAIL: c_int = 0;

/// Add list of entries to quickfix/location list. Each list entry is
/// a dictionary with item information.
///
/// # Safety
///
/// - `qi` must be a valid pointer to a `qf_info_T`
/// - `list` must be a valid pointer to a `list_T`
/// - `title` must be a valid C string or NULL
#[no_mangle]
#[allow(clippy::too_many_lines, clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_qf_add_entries(
    qi: QfInfoHandleMut,
    mut qf_idx: c_int,
    list: *const c_void,
    title: *const c_char,
    action: c_int,
) -> c_int {
    let mut qfl = nvim_qf_get_list_at_mut(qi, qf_idx);
    let mut old_last: QfLineHandle = std::ptr::null();
    let mut retval: c_int = OK;
    let mut valid_entry = false;

    // Remember current entry's position for 'u' action
    let mut prev_fnum: c_int = 0;
    let mut prev_line: c_int = 0;
    let mut prev_col: c_int = 0;
    nvim_qf_get_ptr_position(
        qfl,
        &raw mut prev_fnum,
        &raw mut prev_line,
        &raw mut prev_col,
    );

    let mut select_first_entry = false;
    let mut select_nearest_entry = false;

    let action_char = action as u8;
    if action_char == b' ' || qf_idx == nvim_qf_get_listcount(qi) {
        select_first_entry = true;
        crate::rs_qf_new_list(qi, title);
        qf_idx = nvim_qf_get_curlist_idx(qi);
        qfl = nvim_qf_get_list_at_mut(qi, qf_idx);
    } else if action_char == b'a' {
        if rs_qf_list_empty(qfl) {
            select_first_entry = true;
        } else {
            old_last = nvim_qf_get_last(qfl);
        }
    } else if action_char == b'r' {
        select_first_entry = true;
        crate::rs_qf_free_items(qfl);
        nvim_qf_store_title(qfl, title);
    } else if action_char == b'u' {
        select_nearest_entry = true;
        crate::rs_qf_free_items(qfl);
        nvim_qf_store_title(qfl, title);
    }

    let mut entry_to_select: QfLineHandle = std::ptr::null();
    let mut entry_to_select_index: c_int = 0;

    // Iterate over the VimL list items
    let mut li = nvim_tv_list_first(list);
    while !li.is_null() {
        let d = nvim_tv_list_item_dict(li);
        if d.is_null() {
            li = nvim_tv_list_item_next(list, li);
            continue;
        }

        let is_first = nvim_tv_list_item_is_first(list, li);
        retval = nvim_qf_add_entry_from_dict(qfl, d, is_first, &raw mut valid_entry);
        if retval == QF_FAIL {
            break;
        }

        let entry = nvim_qf_get_last(qfl);
        if (select_first_entry && entry_to_select.is_null())
            || (select_nearest_entry
                && (entry_to_select.is_null()
                    || rs_qf_entry_is_closer_to_target(
                        entry,
                        entry_to_select,
                        prev_fnum,
                        prev_line,
                        prev_col,
                    )))
        {
            entry_to_select = entry;
            entry_to_select_index = nvim_qf_get_count(qfl);
        }

        li = nvim_tv_list_item_next(list, li);
    }

    // Check if any valid error entries are added to the list.
    if valid_entry {
        nvim_qf_set_nonevalid(qfl, false);
    } else if nvim_qf_get_index(qfl) == 0 {
        nvim_qf_set_nonevalid(qfl, true);
    }

    // Set the current error.
    if !entry_to_select.is_null() {
        nvim_qf_set_ptr(qfl, entry_to_select);
        nvim_qf_set_index(qfl, entry_to_select_index);
    }

    // Don't update the cursor in quickfix window when appending entries
    nvim_qf_update_buffer(qi, old_last);

    retval
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_list_stats() {
        unsafe {
            let stats = rs_qf_list_stats(std::ptr::null());
            assert_eq!(stats.total_entries, 0);
            assert_eq!(stats.valid_entries, 0);
        }
    }

    #[test]
    fn test_null_stack_info() {
        unsafe {
            let info = rs_qf_stack_info(std::ptr::null());
            assert!(info.is_empty);
            assert_eq!(info.list_count, 0);
        }
    }

    #[test]
    fn test_null_curlist_entry_count() {
        unsafe {
            assert_eq!(rs_qf_curlist_entry_count(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_null_curlist_valid_count() {
        unsafe {
            assert_eq!(rs_qf_curlist_valid_count(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_null_older_list() {
        unsafe {
            assert_eq!(rs_qf_older_list(std::ptr::null_mut()), -1);
        }
    }

    #[test]
    fn test_null_newer_list() {
        unsafe {
            assert_eq!(rs_qf_newer_list(std::ptr::null_mut()), -1);
        }
    }

    #[test]
    fn test_null_goto_list_idx() {
        unsafe {
            assert!(!rs_qf_goto_list_idx(std::ptr::null_mut(), 0));
        }
    }

    #[test]
    fn test_null_oldest_list() {
        unsafe {
            assert_eq!(rs_qf_oldest_list(std::ptr::null_mut()), -1);
        }
    }

    #[test]
    fn test_null_newest_list() {
        unsafe {
            assert_eq!(rs_qf_newest_list(std::ptr::null_mut()), -1);
        }
    }

    #[test]
    fn test_null_reset_to_first() {
        unsafe {
            assert_eq!(rs_qf_reset_to_first(std::ptr::null_mut()), 0);
        }
    }

    #[test]
    fn test_null_reset_to_last() {
        unsafe {
            assert_eq!(rs_qf_reset_to_last(std::ptr::null_mut()), 0);
        }
    }

    #[test]
    fn test_null_at_first() {
        unsafe {
            assert!(rs_qf_at_first(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_at_last() {
        unsafe {
            assert!(rs_qf_at_last(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_lists_same() {
        unsafe {
            assert!(rs_qf_lists_same(std::ptr::null(), std::ptr::null()));
        }
    }

    #[test]
    fn test_null_list_modified_since() {
        unsafe {
            assert!(!rs_qf_list_modified_since(std::ptr::null(), 0));
        }
    }

    #[test]
    fn test_null_valid_entry_idx() {
        unsafe {
            assert!(!rs_qf_valid_entry_idx(std::ptr::null(), 1));
        }
    }

    #[test]
    fn test_null_clamp_entry_idx() {
        unsafe {
            assert_eq!(rs_qf_clamp_entry_idx(std::ptr::null(), 5), 0);
        }
    }

    #[test]
    fn test_null_stack_has_room() {
        unsafe {
            assert!(!rs_qf_stack_has_room(std::ptr::null()));
        }
    }

    #[test]
    fn test_null_stack_available() {
        unsafe {
            assert_eq!(rs_qf_stack_available(std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_entry_type_values() {
        assert_eq!(QfEntryType::Error as c_int, 0);
        assert_eq!(QfEntryType::Warning as c_int, 1);
        assert_eq!(QfEntryType::Info as c_int, 2);
        assert_eq!(QfEntryType::Note as c_int, 3);
        assert_eq!(QfEntryType::Other as c_int, 4);
    }

    #[test]
    fn test_entry_type_from_char() {
        assert_eq!(QfEntryType::from_char(b'E' as c_char), QfEntryType::Error);
        assert_eq!(QfEntryType::from_char(b'e' as c_char), QfEntryType::Error);
        assert_eq!(QfEntryType::from_char(b'W' as c_char), QfEntryType::Warning);
        assert_eq!(QfEntryType::from_char(b'w' as c_char), QfEntryType::Warning);
        assert_eq!(QfEntryType::from_char(b'I' as c_char), QfEntryType::Info);
        assert_eq!(QfEntryType::from_char(b'N' as c_char), QfEntryType::Note);
        assert_eq!(QfEntryType::from_char(b'X' as c_char), QfEntryType::Other);
    }

    #[test]
    fn test_entry_type_to_char() {
        assert_eq!(QfEntryType::Error.to_char(), b'E' as c_char);
        assert_eq!(QfEntryType::Warning.to_char(), b'W' as c_char);
        assert_eq!(QfEntryType::Info.to_char(), b'I' as c_char);
        assert_eq!(QfEntryType::Note.to_char(), b'N' as c_char);
        assert_eq!(QfEntryType::Other.to_char(), b' ' as c_char);
    }

    #[test]
    fn test_entry_type_checks() {
        assert!(QfEntryType::Error.is_error());
        assert!(!QfEntryType::Warning.is_error());

        assert!(QfEntryType::Warning.is_warning());
        assert!(!QfEntryType::Error.is_warning());
    }

    #[test]
    fn test_entry_summary_empty() {
        let summary = QfEntrySummary::empty();
        assert_eq!(summary.lnum, 0);
        assert!(!summary.valid);
        assert!(!summary.has_file());
        assert!(!summary.has_position());
        assert!(!summary.has_range());
    }

    #[test]
    fn test_entry_summary_with_values() {
        let summary = QfEntrySummary {
            lnum: 10,
            col: 5,
            end_lnum: 12,
            end_col: 10,
            fnum: 1,
            entry_type: QfEntryType::Error,
            valid: true,
            viscol: false,
            nr: 1,
        };
        assert!(summary.has_file());
        assert!(summary.has_position());
        assert!(summary.has_range());
    }

    #[test]
    fn test_type_counts_default() {
        let counts = QfTypeCounts::default();
        assert_eq!(counts.total(), 0);
        assert!(!counts.has_errors());
        assert!(!counts.has_warnings());
    }

    #[test]
    fn test_type_counts_with_values() {
        let counts = QfTypeCounts {
            errors: 5,
            warnings: 3,
            info: 2,
            notes: 1,
            other: 0,
        };
        assert_eq!(counts.total(), 11);
        assert!(counts.has_errors());
        assert!(counts.has_warnings());
    }

    #[test]
    fn test_file_ref_empty() {
        let file_ref = QfFileRef::empty();
        assert_eq!(file_ref.fnum, 0);
        assert!(!file_ref.has_file());
    }

    #[test]
    fn test_file_ref_with_file() {
        let file_ref = QfFileRef {
            fnum: 5,
            entry_count: 10,
            first_entry: 1,
        };
        assert!(file_ref.has_file());
    }

    #[test]
    fn test_null_entry_summary() {
        unsafe {
            let summary = rs_qf_entry_summary(std::ptr::null());
            assert_eq!(summary.lnum, 0);
            assert!(!summary.valid);
        }
    }

    #[test]
    fn test_null_type_counts() {
        unsafe {
            let counts = rs_qf_type_counts(std::ptr::null());
            assert_eq!(counts.total(), 0);
        }
    }

    #[test]
    fn test_struct_sizes() {
        use std::mem::size_of;
        // QfEntrySummary should be reasonable
        assert!(size_of::<QfEntrySummary>() <= 40);
        // QfTypeCounts: 5 * 4 = 20 bytes
        assert_eq!(size_of::<QfTypeCounts>(), 20);
        // QfFileRef: 3 * 4 = 12 bytes
        assert_eq!(size_of::<QfFileRef>(), 12);
    }
}
