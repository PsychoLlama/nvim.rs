//! List-do command handlers for quickfix/location lists.
//!
//! This module implements the `:cdo`, `:cfdo`, `:ldo`, `:lfdo` commands
//! that execute commands on each quickfix/location list entry.
//!
//! ## Commands
//!
//! - `:cdo {cmd}` - Execute {cmd} on each valid entry in the quickfix list
//! - `:cfdo {cmd}` - Execute {cmd} on each file in the quickfix list
//! - `:ldo {cmd}` - Execute {cmd} on each valid entry in the location list
//! - `:lfdo {cmd}` - Execute {cmd} on each file in the location list
//!
//! ## Range Support
//!
//! All commands support a range: `:1,5cdo {cmd}` operates on entries 1-5.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

// =============================================================================
// List-do Operation Types
// =============================================================================

/// Type of list-do operation.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListDoType {
    /// :cdo - execute on each entry
    Cdo = 0,
    /// :cfdo - execute on each file
    Cfdo = 1,
    /// :ldo - location list version of cdo
    Ldo = 2,
    /// :lfdo - location list version of cfdo
    Lfdo = 3,
}

impl ListDoType {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::Cdo),
            1 => Some(Self::Cfdo),
            2 => Some(Self::Ldo),
            3 => Some(Self::Lfdo),
            _ => None,
        }
    }

    /// Check if this is a location list command.
    pub const fn is_loclist(self) -> bool {
        matches!(self, Self::Ldo | Self::Lfdo)
    }

    /// Check if this operates on files rather than entries.
    pub const fn is_file_based(self) -> bool {
        matches!(self, Self::Cfdo | Self::Lfdo)
    }
}

// =============================================================================
// List-do State
// =============================================================================

/// State for list-do execution.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ListDoState {
    /// Current entry index (1-based)
    pub current: c_int,
    /// First entry to process (1-based)
    pub first: c_int,
    /// Last entry to process (1-based)
    pub last: c_int,
    /// Total number of entries
    pub total: c_int,
    /// Number of entries processed
    pub processed: c_int,
    /// Number of errors encountered
    pub errors: c_int,
    /// Whether to stop on error
    pub stop_on_error: bool,
    /// Whether operation is complete
    pub done: bool,
}

impl ListDoState {
    /// Create a new state for the given range.
    pub const fn new(first: c_int, last: c_int, total: c_int) -> Self {
        Self {
            current: first,
            first,
            last,
            total,
            processed: 0,
            errors: 0,
            stop_on_error: false,
            done: false,
        }
    }

    /// Check if there are more entries to process.
    pub const fn has_more(&self) -> bool {
        !self.done && self.current <= self.last && self.current <= self.total
    }

    /// Advance to the next entry.
    pub fn advance(&mut self) {
        self.processed += 1;
        self.current += 1;
        if self.current > self.last || self.current > self.total {
            self.done = true;
        }
    }

    /// Record an error and check if we should stop.
    pub fn record_error(&mut self) -> bool {
        self.errors += 1;
        if self.stop_on_error {
            self.done = true;
            true
        } else {
            false
        }
    }

    /// Get the progress as a percentage.
    pub fn progress_percent(&self) -> c_int {
        let range = self.last - self.first + 1;
        if range <= 0 {
            return 100;
        }
        ((self.processed * 100) / range).min(100)
    }
}

impl Default for ListDoState {
    fn default() -> Self {
        Self::new(1, 0, 0)
    }
}

/// FFI export: create new list-do state.
#[no_mangle]
pub extern "C" fn rs_listdo_state_new(first: c_int, last: c_int, total: c_int) -> ListDoState {
    ListDoState::new(first, last, total)
}

/// FFI export: check if there are more entries.
#[no_mangle]
pub extern "C" fn rs_listdo_state_has_more(state: *const ListDoState) -> bool {
    if state.is_null() {
        return false;
    }
    unsafe { (*state).has_more() }
}

/// FFI export: advance to next entry.
#[no_mangle]
pub extern "C" fn rs_listdo_state_advance(state: *mut ListDoState) {
    if !state.is_null() {
        unsafe { (*state).advance() }
    }
}

/// FFI export: record an error.
#[no_mangle]
pub extern "C" fn rs_listdo_state_record_error(state: *mut ListDoState) -> bool {
    if state.is_null() {
        return false;
    }
    unsafe { (*state).record_error() }
}

// =============================================================================
// File-based Iteration
// =============================================================================

/// Entry for file-based iteration.
///
/// When using :cfdo/:lfdo, we iterate by unique files rather than
/// individual entries. This tracks the file boundaries.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FileEntry {
    /// Buffer number for this file
    pub bufnr: c_int,
    /// First quickfix entry index for this file (1-based)
    pub first_entry: c_int,
    /// Last quickfix entry index for this file (1-based)
    pub last_entry: c_int,
    /// Number of entries in this file
    pub entry_count: c_int,
}

impl FileEntry {
    /// Create a new file entry.
    pub const fn new(bufnr: c_int, first_entry: c_int) -> Self {
        Self {
            bufnr,
            first_entry,
            last_entry: first_entry,
            entry_count: 1,
        }
    }

    /// Add another entry to this file.
    pub fn add_entry(&mut self, entry_idx: c_int) {
        self.last_entry = entry_idx;
        self.entry_count += 1;
    }
}

impl Default for FileEntry {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

/// State for file-based list-do.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct FileListDoState {
    /// Current file index (0-based into file list)
    pub file_index: c_int,
    /// Total number of unique files
    pub file_count: c_int,
    /// Current file's buffer number
    pub current_bufnr: c_int,
    /// First entry index in current file
    pub current_first: c_int,
    /// Last entry index in current file
    pub current_last: c_int,
    /// Files processed
    pub processed: c_int,
    /// Whether operation is complete
    pub done: bool,
}

impl FileListDoState {
    /// Create new file-based state.
    pub const fn new(file_count: c_int) -> Self {
        Self {
            file_index: 0,
            file_count,
            current_bufnr: 0,
            current_first: 0,
            current_last: 0,
            processed: 0,
            done: file_count == 0,
        }
    }

    /// Check if there are more files.
    pub const fn has_more(&self) -> bool {
        !self.done && self.file_index < self.file_count
    }

    /// Set current file entry.
    pub fn set_current(&mut self, entry: FileEntry) {
        self.current_bufnr = entry.bufnr;
        self.current_first = entry.first_entry;
        self.current_last = entry.last_entry;
    }

    /// Advance to next file.
    pub fn advance(&mut self) {
        self.processed += 1;
        self.file_index += 1;
        if self.file_index >= self.file_count {
            self.done = true;
        }
    }
}

/// FFI export: create new file list-do state.
#[no_mangle]
pub extern "C" fn rs_file_listdo_state_new(file_count: c_int) -> FileListDoState {
    FileListDoState::new(file_count)
}

/// FFI export: check if there are more files.
#[no_mangle]
pub extern "C" fn rs_file_listdo_state_has_more(state: *const FileListDoState) -> bool {
    if state.is_null() {
        return false;
    }
    unsafe { (*state).has_more() }
}

/// FFI export: advance file list-do state.
#[no_mangle]
pub extern "C" fn rs_file_listdo_state_advance(state: *mut FileListDoState) {
    if !state.is_null() {
        unsafe { (*state).advance() }
    }
}

// =============================================================================
// Range Validation
// =============================================================================

/// Validate and normalize a range for list-do.
///
/// Returns the normalized (first, last) pair, or (-1, -1) if invalid.
pub fn validate_range(first: c_int, last: c_int, total: c_int) -> (c_int, c_int) {
    if total <= 0 {
        return (-1, -1);
    }

    // Default range is all entries
    let f = if first <= 0 { 1 } else { first };
    let l = if last <= 0 { total } else { last };

    // Validate range
    if f > total || l < f {
        return (-1, -1);
    }

    // Clamp to valid range
    (f.min(total), l.min(total))
}

/// FFI export: validate list-do range.
#[no_mangle]
pub extern "C" fn rs_listdo_validate_range(
    first: c_int,
    last: c_int,
    total: c_int,
    out_first: *mut c_int,
    out_last: *mut c_int,
) -> bool {
    let (f, l) = validate_range(first, last, total);
    if f < 0 {
        return false;
    }
    if !out_first.is_null() {
        unsafe { *out_first = f }
    }
    if !out_last.is_null() {
        unsafe { *out_last = l }
    }
    true
}

// =============================================================================
// Result Reporting
// =============================================================================

/// Result of a list-do operation.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ListDoResult {
    /// Number of entries/files processed
    pub processed: c_int,
    /// Number of errors encountered
    pub errors: c_int,
    /// Whether the operation completed successfully
    pub success: bool,
    /// Whether the operation was interrupted
    pub interrupted: bool,
}

impl ListDoResult {
    /// Create a success result.
    pub const fn success(processed: c_int) -> Self {
        Self {
            processed,
            errors: 0,
            success: true,
            interrupted: false,
        }
    }

    /// Create an error result.
    pub const fn error(processed: c_int, errors: c_int) -> Self {
        Self {
            processed,
            errors,
            success: errors == 0,
            interrupted: false,
        }
    }

    /// Create an interrupted result.
    pub const fn interrupted(processed: c_int) -> Self {
        Self {
            processed,
            errors: 0,
            success: false,
            interrupted: true,
        }
    }
}

impl Default for ListDoResult {
    fn default() -> Self {
        Self::success(0)
    }
}

/// FFI export: create success result.
#[no_mangle]
pub extern "C" fn rs_listdo_result_success(processed: c_int) -> ListDoResult {
    ListDoResult::success(processed)
}

/// FFI export: create error result.
#[no_mangle]
pub extern "C" fn rs_listdo_result_error(processed: c_int, errors: c_int) -> ListDoResult {
    ListDoResult::error(processed, errors)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_listdo_type() {
        assert_eq!(ListDoType::from_c_int(0), Some(ListDoType::Cdo));
        assert_eq!(ListDoType::from_c_int(2), Some(ListDoType::Ldo));
        assert_eq!(ListDoType::from_c_int(99), None);

        assert!(!ListDoType::Cdo.is_loclist());
        assert!(!ListDoType::Cfdo.is_loclist());
        assert!(ListDoType::Ldo.is_loclist());
        assert!(ListDoType::Lfdo.is_loclist());

        assert!(!ListDoType::Cdo.is_file_based());
        assert!(ListDoType::Cfdo.is_file_based());
        assert!(!ListDoType::Ldo.is_file_based());
        assert!(ListDoType::Lfdo.is_file_based());
    }

    #[test]
    fn test_listdo_state() {
        let mut state = ListDoState::new(1, 5, 10);

        assert!(state.has_more());
        assert_eq!(state.current, 1);
        assert_eq!(state.processed, 0);

        state.advance();
        assert_eq!(state.current, 2);
        assert_eq!(state.processed, 1);

        // Process remaining
        for _ in 0..4 {
            state.advance();
        }
        assert!(!state.has_more());
        assert!(state.done);
        assert_eq!(state.processed, 5);
    }

    #[test]
    fn test_listdo_state_error() {
        let mut state = ListDoState::new(1, 5, 10);
        state.stop_on_error = true;

        assert!(!state.record_error()); // First call sets done
        assert!(state.done);
    }

    #[test]
    fn test_listdo_progress() {
        let mut state = ListDoState::new(1, 10, 10);
        assert_eq!(state.progress_percent(), 0);

        state.advance();
        assert_eq!(state.progress_percent(), 10);

        for _ in 0..9 {
            state.advance();
        }
        assert_eq!(state.progress_percent(), 100);
    }

    #[test]
    fn test_file_entry() {
        let mut entry = FileEntry::new(1, 5);
        assert_eq!(entry.first_entry, 5);
        assert_eq!(entry.last_entry, 5);
        assert_eq!(entry.entry_count, 1);

        entry.add_entry(10);
        assert_eq!(entry.last_entry, 10);
        assert_eq!(entry.entry_count, 2);
    }

    #[test]
    fn test_file_listdo_state() {
        let mut state = FileListDoState::new(3);
        assert!(state.has_more());
        assert_eq!(state.file_index, 0);

        state.advance();
        assert!(state.has_more());
        assert_eq!(state.file_index, 1);

        state.advance();
        state.advance();
        assert!(!state.has_more());
        assert!(state.done);
    }

    #[test]
    fn test_validate_range() {
        // Normal range
        assert_eq!(validate_range(1, 5, 10), (1, 5));

        // Default range
        assert_eq!(validate_range(0, 0, 10), (1, 10));

        // Clamp to total
        assert_eq!(validate_range(1, 100, 10), (1, 10));

        // Invalid: empty list
        assert_eq!(validate_range(1, 5, 0), (-1, -1));

        // Invalid: first > total
        assert_eq!(validate_range(20, 30, 10), (-1, -1));

        // Invalid: last < first
        assert_eq!(validate_range(5, 3, 10), (-1, -1));
    }

    #[test]
    fn test_listdo_result() {
        let success = ListDoResult::success(10);
        assert!(success.success);
        assert_eq!(success.processed, 10);
        assert!(!success.interrupted);

        let error = ListDoResult::error(5, 2);
        assert!(!error.success);
        assert_eq!(error.errors, 2);

        let interrupted = ListDoResult::interrupted(3);
        assert!(!interrupted.success);
        assert!(interrupted.interrupted);
    }
}
