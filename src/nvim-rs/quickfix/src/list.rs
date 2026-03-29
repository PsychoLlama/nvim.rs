//! Quickfix/Location list management operations.
//!
//! This module provides Rust implementations for creating, modifying, and
//! managing quickfix and location lists, including stack navigation.

#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::use_self)]

use crate::ffi_types::QfListPtr;
use std::ffi::{c_char, c_int, c_void};

/// Line number type (matches `linenr_T` in Neovim)
type LinenrT = i32;

/// Opaque handle to `qf_info_T` (quickfix stack)
type QfInfoHandle = *const c_void;
type QfInfoHandleMut = *mut c_void;

/// Opaque handle to `qfline_T` (quickfix entry)
type QfLinePtr = *mut crate::ffi_types::QfLineRaw;

// =============================================================================
// External C accessor functions
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Stack accessors
    fn nvim_qf_get_listcount(qi: QfInfoHandle) -> c_int;
    fn nvim_qf_get_curlist(qi: QfInfoHandle) -> QfListPtr;
    fn nvim_qf_get_list_at(qi: QfInfoHandle, idx: c_int) -> QfListPtr;
    fn nvim_qf_get_curlist_idx(qi: QfInfoHandle) -> c_int;
    fn nvim_qf_set_curlist_idx(qi: QfInfoHandleMut, idx: c_int);

    // List accessors

    // Entry accessors

    // List operations
    // nvim_qf_store_title replaced by nvim_qf_set_title_dup (Phase 14)

    // Stack maxcount
    fn nvim_qf_get_maxcount(qi: QfInfoHandle) -> c_int;

    // Entry modification

    // Phase 2: qf_add_entries accessors
    fn nvim_qf_get_list_at_mut(qi: QfInfoHandleMut, idx: c_int) -> QfListPtr;
    #[link_name = "rs_qf_update_buffer"]
    fn nvim_qf_update_buffer(qi: QfInfoHandleMut, old_last: QfLinePtr);
    fn nvim_tv_list_first(list: *const c_void) -> *const c_void;
    fn nvim_tv_list_item_next(list: *const c_void, li: *const c_void) -> *const c_void;
    fn nvim_tv_list_item_dict(li: *const c_void) -> *mut c_void;
    fn nvim_tv_list_item_is_first(list: *const c_void, li: *const c_void) -> bool;
    fn rs_qf_list_empty(qfl: QfListPtr) -> bool;
    fn rs_qf_entry_is_closer_to_target(
        entry: QfLinePtr,
        other: QfLinePtr,
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
pub unsafe extern "C" fn rs_qf_list_stats(qfl: QfListPtr) -> QfListStats {
    if qfl.is_null() {
        return QfListStats::default();
    }

    let total_entries = (*qfl).qf_count;
    let current_index = (*qfl).qf_index;
    let list_id = (*qfl).qf_id;
    let changedtick = (*qfl).qf_changedtick;
    let title = (*qfl).qf_title;
    let has_title = !title.is_null() && *title != 0;
    let all_invalid = (*qfl).qf_nonevalid;

    // Count valid entries and unique files
    let mut valid_entries = 0;
    let mut file_count = 0;
    let mut last_fnum = -1;

    let mut qfp = (*qfl).qf_start;
    while !qfp.is_null() {
        if (*qfp).qf_valid != 0 {
            valid_entries += 1;
        }
        let fnum = (*qfp).qf_fnum;
        if fnum > 0 && fnum != last_fnum {
            file_count += 1;
            last_fnum = fnum;
        }
        qfp = (*qfp).qf_next;
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

    (*qfl).qf_count
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
    let mut qfp = (*qfl).qf_start;
    while !qfp.is_null() {
        if (*qfp).qf_valid != 0 {
            count += 1;
        }
        qfp = (*qfp).qf_next;
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
pub unsafe extern "C" fn rs_qf_reset_to_first(qfl: QfListPtr) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let start = (*qfl).qf_start;
    if start.is_null() {
        return 0;
    }

    (*qfl).qf_ptr = start;
    (*qfl).qf_index = 1;
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
pub unsafe extern "C" fn rs_qf_reset_to_last(qfl: QfListPtr) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let last = (*qfl).qf_last;
    if last.is_null() {
        return 0;
    }

    let count = (*qfl).qf_count;
    (*qfl).qf_ptr = last;
    (*qfl).qf_index = count;
    count
}

/// Check if the current entry is at the first entry.
///
/// # Safety
///
/// - `qfl` may be null (returns true - empty list is at beginning)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_at_first(qfl: QfListPtr) -> bool {
    if qfl.is_null() {
        return true;
    }

    (*qfl).qf_index <= 1
}

/// Check if the current entry is at the last entry.
///
/// # Safety
///
/// - `qfl` may be null (returns true - empty list is at end)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_at_last(qfl: QfListPtr) -> bool {
    if qfl.is_null() {
        return true;
    }

    let idx = (*qfl).qf_index;
    let count = (*qfl).qf_count;
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
pub unsafe extern "C" fn rs_qf_lists_same(a: QfListPtr, b: QfListPtr) -> bool {
    match (a.is_null(), b.is_null()) {
        (true, true) => true,
        (true, false) | (false, true) => false,
        (false, false) => (*a).qf_id == (*b).qf_id,
    }
}

/// Check if a list has been modified since a given changedtick.
///
/// # Safety
///
/// - `qfl` may be null (returns false)
/// - If non-null, `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_list_modified_since(qfl: QfListPtr, tick: c_int) -> bool {
    if qfl.is_null() {
        return false;
    }

    (*qfl).qf_changedtick > tick
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
pub unsafe extern "C" fn rs_qf_valid_entry_idx(qfl: QfListPtr, idx: c_int) -> bool {
    if qfl.is_null() {
        return false;
    }

    let count = (*qfl).qf_count;
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
pub unsafe extern "C" fn rs_qf_clamp_entry_idx(qfl: QfListPtr, idx: c_int) -> c_int {
    if qfl.is_null() {
        return 0;
    }

    let count = (*qfl).qf_count;
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
pub unsafe extern "C" fn rs_qf_entry_summary(qfp: QfLinePtr) -> QfEntrySummary {
    if qfp.is_null() {
        return QfEntrySummary::empty();
    }

    let type_char = (*qfp).qf_type;
    QfEntrySummary {
        lnum: (*qfp).qf_lnum,
        col: (*qfp).qf_col,
        end_lnum: (*qfp).qf_end_lnum,
        end_col: (*qfp).qf_end_col,
        fnum: (*qfp).qf_fnum,
        entry_type: QfEntryType::from_char(type_char),
        valid: ((*qfp).qf_valid != 0),
        viscol: ((*qfp).qf_viscol != 0),
        nr: (*qfp).qf_nr,
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
pub unsafe extern "C" fn rs_qf_type_counts(qfl: QfListPtr) -> QfTypeCounts {
    if qfl.is_null() {
        return QfTypeCounts::default();
    }

    let mut counts = QfTypeCounts::default();
    let mut qfp = (*qfl).qf_start;
    while !qfp.is_null() {
        let type_char = (*qfp).qf_type;
        match QfEntryType::from_char(type_char) {
            QfEntryType::Error => counts.errors += 1,
            QfEntryType::Warning => counts.warnings += 1,
            QfEntryType::Info => counts.info += 1,
            QfEntryType::Note => counts.notes += 1,
            QfEntryType::Other => counts.other += 1,
        }
        qfp = (*qfp).qf_next;
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
pub unsafe extern "C" fn rs_qf_clear_entries(qfl: QfListPtr) {
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
pub unsafe extern "C" fn rs_qf_mark_entry_cleared(qfp: QfLinePtr) {
    if qfp.is_null() {
        return;
    }
    (*qfp).qf_cleared = 1;
}

/// Check if an entry is cleared.
///
/// # Safety
///
/// - `qfp` must be a valid pointer to a `qfline_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_entry_is_cleared(qfp: QfLinePtr) -> bool {
    if qfp.is_null() {
        return false;
    }
    (*qfp).qf_cleared != 0
}

/// Count entries in a specific file.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T` struct
#[no_mangle]
pub unsafe extern "C" fn rs_qf_file_entry_count(qfl: QfListPtr, fnum: c_int) -> c_int {
    if qfl.is_null() || fnum <= 0 {
        return 0;
    }

    let mut count = 0;
    let mut qfp = (*qfl).qf_start;

    while !qfp.is_null() {
        if (*qfp).qf_fnum == fnum {
            count += 1;
        }
        qfp = (*qfp).qf_next;
    }

    count
}

// =============================================================================
// Phase 2: qf_add_entries
// =============================================================================

const OK: c_int = 1;
const QF_FAIL: c_int = 0;

// =============================================================================
// Phase 11: qf_add_entry_from_dict (migrated from C)
// =============================================================================

extern "C" {
    // Dict field accessors for qf_add_entry_from_dict
    fn tv_dict_get_string(dict: *const c_void, key: *const c_char, save: bool) -> *mut c_char;
    #[link_name = "tv_dict_get_number"]
    fn nvim_tv_dict_get_number(dict: *const c_void, key: *const c_char) -> i64;
    fn tv_dict_find(dict: *const c_void, key: *const c_char, key_len: i64) -> *mut c_void;
    fn nvim_qf_di_get_tv(di: *mut c_void) -> *mut c_void;
    fn tv_copy(from: *const c_void, to: *mut c_void);
    fn tv_clear(tv: *mut c_void);
    fn xcalloc(count: usize, size: usize) -> *mut c_void;
    fn tv_free(tv: *mut c_void);
    fn buflist_findnr(bnr: c_int) -> *mut c_void;
    fn semsg(fmt: *const std::ffi::c_char, ...) -> bool;
    // (nvim_qf_semsg_e92_bufnr deleted: use semsg directly)
    fn nvim_xfree_char(ptr: *mut c_char);
    fn rs_qf_add_entry(
        qfl: QfListPtr,
        dir: *mut c_char,
        fname: *const c_char,
        module: *const c_char,
        bufnum: c_int,
        mesg: *const c_char,
        lnum: LinenrT,
        end_lnum: LinenrT,
        col: c_int,
        end_col: c_int,
        vis_col: c_char,
        pattern: *const c_char,
        nr: c_int,
        type_char: c_char,
        user_data: *const c_void,
        valid: c_char,
    ) -> c_int;
    fn xfree(ptr: *mut ::std::ffi::c_void);
    fn xstrdup(s: *const ::std::ffi::c_char) -> *mut ::std::ffi::c_char;
}

/// Tracks whether we already emitted E92 in the current `add_entries` batch.
/// Reset to false when `first_entry` is true.
static DID_BUFNR_EMSG: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Create a quickfix entry from a `VimL` `dict_T`.
///
/// Mirrors C `qf_add_entry_from_dict`. Reads dict fields via accessors,
/// validates `bufnum`, and calls `rs_qf_add_entry`.
///
/// # Safety
///
/// - `qfl` must be a valid pointer to a `qf_list_T`
/// - `d` must be a valid pointer to a `dict_T`
/// - `valid_entry` must be a valid non-null pointer
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_qf_add_entry_from_dict(
    qfl: QfListPtr,
    d: *mut c_void,
    first_entry: bool,
    valid_entry: *mut bool,
) -> c_int {
    use std::sync::atomic::Ordering;

    if first_entry {
        DID_BUFNR_EMSG.store(false, Ordering::Relaxed);
    }

    let filename = tv_dict_get_string(d, c"filename".as_ptr(), true);
    let module = tv_dict_get_string(d, c"module".as_ptr(), true);
    let mut bufnum = nvim_tv_dict_get_number(d, c"bufnr".as_ptr()) as c_int;
    let lnum = nvim_tv_dict_get_number(d, c"lnum".as_ptr()) as LinenrT;
    let end_lnum = nvim_tv_dict_get_number(d, c"end_lnum".as_ptr()) as LinenrT;
    let col = nvim_tv_dict_get_number(d, c"col".as_ptr()) as c_int;
    let end_col = nvim_tv_dict_get_number(d, c"end_col".as_ptr()) as c_int;
    let vcol = nvim_tv_dict_get_number(d, c"vcol".as_ptr()) as c_char;
    let nr = nvim_tv_dict_get_number(d, c"nr".as_ptr()) as c_int;
    // type: not alloc'd (alloc=false), caller must NOT free
    let type_str = tv_dict_get_string(d, c"type".as_ptr(), false);
    let pattern = tv_dict_get_string(d, c"pattern".as_ptr(), true);

    let text_raw = tv_dict_get_string(d, c"text".as_ptr(), true);
    let text = if text_raw.is_null() {
        // allocate empty string: xcalloc(1, 1) gives a '\0'-terminated buffer
        xcalloc(1, 1).cast()
    } else {
        text_raw
    };

    // Allocate a heap typval_T for user_data (zeroed, VAR_UNKNOWN)
    // sizeof(typval_T) = 16
    let user_data_tv: *mut c_void = xcalloc(1, 16);
    let ud_di = tv_dict_find(d, c"user_data".as_ptr(), -1);
    if !ud_di.is_null() {
        let ud_tv = nvim_qf_di_get_tv(ud_di);
        if !ud_tv.is_null() {
            tv_copy(ud_tv, user_data_tv);
        }
    }

    let mut valid = (filename.is_null() && bufnum == 0) || (lnum == 0 && pattern.is_null());
    valid = !valid; // valid=false if no file/lnum

    // Mark entries with non-existing buffer number as not valid.
    // Emit the error message only once per batch.
    if bufnum != 0 && buflist_findnr(bufnum).is_null() {
        if !DID_BUFNR_EMSG.load(Ordering::Relaxed) {
            DID_BUFNR_EMSG.store(true, Ordering::Relaxed);
            semsg(c"E92: Buffer %lld not found".as_ptr(), i64::from(bufnum));
        }
        valid = false;
        bufnum = 0;
    }

    // If the 'valid' field is present it overrules the detected value.
    let valid_di = tv_dict_find(d, c"valid".as_ptr(), -1);
    if !valid_di.is_null() {
        valid = nvim_tv_dict_get_number(d, c"valid".as_ptr()) != 0;
    }

    let type_char: c_char = if type_str.is_null() || *type_str == 0 {
        0
    } else {
        *type_str
    };

    let status = rs_qf_add_entry(
        qfl,
        std::ptr::null_mut(), // dir
        filename,
        module,
        bufnum,
        text,
        lnum,
        end_lnum,
        col,
        end_col,
        vcol,
        pattern,
        nr,
        type_char,
        user_data_tv,
        c_char::from(valid),
    );

    nvim_xfree_char(filename);
    nvim_xfree_char(module);
    nvim_xfree_char(pattern);
    nvim_xfree_char(text);
    tv_clear(user_data_tv);
    tv_free(user_data_tv);

    if valid {
        *valid_entry = true;
    }

    status
}

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
    let mut old_last: QfLinePtr = std::ptr::null_mut();
    let mut retval: c_int = OK;
    let mut valid_entry = false;

    // Remember current entry's position for 'u' action
    let (prev_fnum, prev_line, prev_col): (c_int, c_int, c_int) = if (*qfl).qf_ptr.is_null() {
        (0, 0, 0)
    } else {
        (
            (*(*qfl).qf_ptr).qf_fnum,
            (*(*qfl).qf_ptr).qf_lnum,
            (*(*qfl).qf_ptr).qf_col,
        )
    };

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
            old_last = (*qfl).qf_last;
        }
    } else if action_char == b'r' {
        select_first_entry = true;
        crate::rs_qf_free_items(qfl);
        {
            xfree((*qfl).qf_title.cast());
            (*qfl).qf_title = if (title).is_null() {
                ::std::ptr::null_mut()
            } else {
                xstrdup(title)
            };
        };
    } else if action_char == b'u' {
        select_nearest_entry = true;
        crate::rs_qf_free_items(qfl);
        {
            xfree((*qfl).qf_title.cast());
            (*qfl).qf_title = if (title).is_null() {
                ::std::ptr::null_mut()
            } else {
                xstrdup(title)
            };
        };
    }

    let mut entry_to_select: QfLinePtr = std::ptr::null_mut();
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
        retval = rs_qf_add_entry_from_dict(qfl, d, is_first, &raw mut valid_entry);
        if retval == QF_FAIL {
            break;
        }

        let entry = (*qfl).qf_last;
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
            entry_to_select_index = (*qfl).qf_count;
        }

        li = nvim_tv_list_item_next(list, li);
    }

    // Check if any valid error entries are added to the list.
    if valid_entry {
        (*qfl).qf_nonevalid = false;
    } else if (*qfl).qf_index == 0 {
        (*qfl).qf_nonevalid = true;
    }

    // Set the current error.
    if !entry_to_select.is_null() {
        (*qfl).qf_ptr = entry_to_select;
        (*qfl).qf_index = entry_to_select_index;
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
            let stats = rs_qf_list_stats(std::ptr::null_mut());
            assert_eq!(stats.total_entries, 0);
            assert_eq!(stats.valid_entries, 0);
        }
    }

    #[test]
    fn test_null_stack_info() {
        unsafe {
            let info = rs_qf_stack_info(std::ptr::null_mut());
            assert!(info.is_empty);
            assert_eq!(info.list_count, 0);
        }
    }

    #[test]
    fn test_null_curlist_entry_count() {
        unsafe {
            assert_eq!(rs_qf_curlist_entry_count(std::ptr::null_mut()), 0);
        }
    }

    #[test]
    fn test_null_curlist_valid_count() {
        unsafe {
            assert_eq!(rs_qf_curlist_valid_count(std::ptr::null_mut()), 0);
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
            assert!(rs_qf_at_first(std::ptr::null_mut()));
        }
    }

    #[test]
    fn test_null_at_last() {
        unsafe {
            assert!(rs_qf_at_last(std::ptr::null_mut()));
        }
    }

    #[test]
    fn test_null_lists_same() {
        unsafe {
            assert!(rs_qf_lists_same(std::ptr::null_mut(), std::ptr::null_mut()));
        }
    }

    #[test]
    fn test_null_list_modified_since() {
        unsafe {
            assert!(!rs_qf_list_modified_since(std::ptr::null_mut(), 0));
        }
    }

    #[test]
    fn test_null_valid_entry_idx() {
        unsafe {
            assert!(!rs_qf_valid_entry_idx(std::ptr::null_mut(), 1));
        }
    }

    #[test]
    fn test_null_clamp_entry_idx() {
        unsafe {
            assert_eq!(rs_qf_clamp_entry_idx(std::ptr::null_mut(), 5), 0);
        }
    }

    #[test]
    fn test_null_stack_has_room() {
        unsafe {
            assert!(!rs_qf_stack_has_room(std::ptr::null_mut()));
        }
    }

    #[test]
    fn test_null_stack_available() {
        unsafe {
            assert_eq!(rs_qf_stack_available(std::ptr::null_mut()), 0);
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
            let summary = rs_qf_entry_summary(std::ptr::null_mut());
            assert_eq!(summary.lnum, 0);
            assert!(!summary.valid);
        }
    }

    #[test]
    fn test_null_type_counts() {
        unsafe {
            let counts = rs_qf_type_counts(std::ptr::null_mut());
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
