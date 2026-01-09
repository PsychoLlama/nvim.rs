//! Terminal buffer management
//!
//! This module provides Rust implementations for terminal buffer operations,
//! including buffer lifecycle, row/line conversion, and buffer synchronization.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::use_self)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::if_not_else)]

use std::ffi::c_void;
use std::os::raw::c_int;

/// Line number type matching linenr_T (i32).
type LinenrT = i32;

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to Terminal struct.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerminalHandle(*mut c_void);

impl TerminalHandle {
    /// Check if the handle is null.
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Create a null handle.
    #[inline]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Get the raw pointer.
    #[inline]
    pub const fn as_ptr(self) -> *mut c_void {
        self.0
    }
}

/// Opaque handle to a buffer (buf_T).
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BufHandle(*mut c_void);

impl BufHandle {
    /// Check if the handle is null.
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Create a null handle.
    #[inline]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }
}

// =============================================================================
// External C Functions
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Terminal accessors
    fn nvim_terminal_get_buf_handle(term: TerminalHandle) -> c_int;
    fn nvim_terminal_get_closed(term: TerminalHandle) -> c_int;
    fn nvim_terminal_get_sb_current(term: TerminalHandle) -> usize;
    fn nvim_terminal_get_sb_size(term: TerminalHandle) -> usize;
    fn nvim_terminal_get_invalid_start(term: TerminalHandle) -> c_int;
    fn nvim_terminal_get_invalid_end(term: TerminalHandle) -> c_int;

    // VTerm accessors
    fn nvim_terminal_get_vterm(term: TerminalHandle) -> *mut c_void;
    fn nvim_terminal_get_vterm_screen(term: TerminalHandle) -> *mut c_void;

    // Buffer functions
    fn nvim_buf_get_ml_line_count(buf: BufHandle) -> LinenrT;
}

// =============================================================================
// Terminal Buffer State
// =============================================================================

/// Terminal buffer state information.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TerminalBufferState {
    /// Buffer handle ID.
    pub buf_handle: c_int,
    /// Whether the terminal is closed.
    pub closed: bool,
    /// Current scrollback line count.
    pub sb_current: usize,
    /// Maximum scrollback size.
    pub sb_size: usize,
    /// Start of invalid region (-1 if none).
    pub invalid_start: c_int,
    /// End of invalid region (-1 if none).
    pub invalid_end: c_int,
}

impl TerminalBufferState {
    /// Create an empty state.
    pub const fn empty() -> Self {
        Self {
            buf_handle: 0,
            closed: false,
            sb_current: 0,
            sb_size: 0,
            invalid_start: -1,
            invalid_end: -1,
        }
    }

    /// Check if there's an invalid region that needs refresh.
    pub const fn has_invalid_region(&self) -> bool {
        self.invalid_start >= 0 && self.invalid_end >= self.invalid_start
    }

    /// Get the number of invalid rows.
    pub const fn invalid_row_count(&self) -> c_int {
        if self.has_invalid_region() {
            self.invalid_end - self.invalid_start + 1
        } else {
            0
        }
    }
}

/// Get the terminal buffer state.
pub fn get_terminal_buffer_state(term: TerminalHandle) -> TerminalBufferState {
    if term.is_null() {
        return TerminalBufferState::empty();
    }

    unsafe {
        TerminalBufferState {
            buf_handle: nvim_terminal_get_buf_handle(term),
            closed: nvim_terminal_get_closed(term) != 0,
            sb_current: nvim_terminal_get_sb_current(term),
            sb_size: nvim_terminal_get_sb_size(term),
            invalid_start: nvim_terminal_get_invalid_start(term),
            invalid_end: nvim_terminal_get_invalid_end(term),
        }
    }
}

// =============================================================================
// Row/Line Number Conversion
// =============================================================================

/// Convert a terminal row number to a buffer line number.
///
/// Terminal rows are 0-based from the top of the visible screen.
/// Buffer line numbers are 1-based and include scrollback.
///
/// The formula is: linenr = row + sb_current + 1
pub fn row_to_linenr(term: TerminalHandle, row: c_int) -> LinenrT {
    if term.is_null() {
        return 0;
    }

    unsafe {
        let sb_current = nvim_terminal_get_sb_current(term) as LinenrT;
        row + sb_current + 1
    }
}

/// Convert a buffer line number to a terminal row number.
///
/// Buffer line numbers are 1-based and include scrollback.
/// Terminal rows are 0-based from the top of the visible screen.
///
/// The formula is: row = linenr - sb_current - 1
pub fn linenr_to_row(term: TerminalHandle, linenr: LinenrT) -> c_int {
    if term.is_null() {
        return 0;
    }

    unsafe {
        let sb_current = nvim_terminal_get_sb_current(term) as LinenrT;
        linenr - sb_current - 1
    }
}

/// Check if a line number is in the scrollback region.
pub fn is_scrollback_line(term: TerminalHandle, linenr: LinenrT) -> bool {
    if term.is_null() || linenr < 1 {
        return false;
    }

    unsafe {
        let sb_current = nvim_terminal_get_sb_current(term) as LinenrT;
        linenr <= sb_current
    }
}

/// Check if a line number is in the visible screen region.
pub fn is_screen_line(term: TerminalHandle, linenr: LinenrT) -> bool {
    if term.is_null() || linenr < 1 {
        return false;
    }

    unsafe {
        let sb_current = nvim_terminal_get_sb_current(term) as LinenrT;
        linenr > sb_current
    }
}

// =============================================================================
// Buffer Validation
// =============================================================================

/// Terminal buffer validation result.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferValidation {
    /// Buffer is valid and ready.
    Valid = 0,
    /// Terminal handle is null.
    NullTerminal = 1,
    /// Terminal is closed.
    Closed = 2,
    /// Buffer handle is invalid.
    InvalidBuffer = 3,
}

/// Validate that a terminal buffer is ready for operations.
pub fn validate_terminal_buffer(term: TerminalHandle) -> BufferValidation {
    if term.is_null() {
        return BufferValidation::NullTerminal;
    }

    unsafe {
        if nvim_terminal_get_closed(term) != 0 {
            return BufferValidation::Closed;
        }

        if nvim_terminal_get_buf_handle(term) <= 0 {
            return BufferValidation::InvalidBuffer;
        }
    }

    BufferValidation::Valid
}

// =============================================================================
// Invalidation Region
// =============================================================================

/// Region that needs to be refreshed in the terminal buffer.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct InvalidRegion {
    /// Start row (0-based, -1 if no invalid region).
    pub start_row: c_int,
    /// End row (0-based, inclusive, -1 if no invalid region).
    pub end_row: c_int,
}

impl InvalidRegion {
    /// Create an empty (no invalid region) marker.
    pub const fn none() -> Self {
        Self {
            start_row: -1,
            end_row: -1,
        }
    }

    /// Create a region covering specific rows.
    pub const fn new(start: c_int, end: c_int) -> Self {
        Self {
            start_row: start,
            end_row: end,
        }
    }

    /// Check if there's an invalid region.
    pub const fn is_valid(&self) -> bool {
        self.start_row >= 0 && self.end_row >= self.start_row
    }

    /// Expand this region to include another row.
    pub fn expand(&mut self, row: c_int) {
        if !self.is_valid() {
            self.start_row = row;
            self.end_row = row;
        } else {
            if row < self.start_row {
                self.start_row = row;
            }
            if row > self.end_row {
                self.end_row = row;
            }
        }
    }

    /// Merge with another region.
    pub fn merge(&mut self, other: &InvalidRegion) {
        if !other.is_valid() {
            return;
        }
        if !self.is_valid() {
            *self = *other;
            return;
        }
        if other.start_row < self.start_row {
            self.start_row = other.start_row;
        }
        if other.end_row > self.end_row {
            self.end_row = other.end_row;
        }
    }
}

/// Get the current invalid region from a terminal.
pub fn get_invalid_region(term: TerminalHandle) -> InvalidRegion {
    if term.is_null() {
        return InvalidRegion::none();
    }

    unsafe {
        InvalidRegion {
            start_row: nvim_terminal_get_invalid_start(term),
            end_row: nvim_terminal_get_invalid_end(term),
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Get terminal buffer state.
#[no_mangle]
pub extern "C" fn rs_get_terminal_buffer_state(term: TerminalHandle) -> TerminalBufferState {
    get_terminal_buffer_state(term)
}

/// FFI export: Convert row to line number (using terminal handle).
#[no_mangle]
pub extern "C" fn rs_terminal_row_to_linenr_term(term: TerminalHandle, row: c_int) -> LinenrT {
    row_to_linenr(term, row)
}

/// FFI export: Convert line number to row (using terminal handle).
#[no_mangle]
pub extern "C" fn rs_terminal_linenr_to_row_term(term: TerminalHandle, linenr: LinenrT) -> c_int {
    linenr_to_row(term, linenr)
}

/// FFI export: Check if line is in scrollback.
#[no_mangle]
pub extern "C" fn rs_terminal_is_scrollback_line(term: TerminalHandle, linenr: LinenrT) -> c_int {
    c_int::from(is_scrollback_line(term, linenr))
}

/// FFI export: Check if line is in visible screen.
#[no_mangle]
pub extern "C" fn rs_terminal_is_screen_line(term: TerminalHandle, linenr: LinenrT) -> c_int {
    c_int::from(is_screen_line(term, linenr))
}

/// FFI export: Validate terminal buffer.
#[no_mangle]
pub extern "C" fn rs_validate_terminal_buffer(term: TerminalHandle) -> BufferValidation {
    validate_terminal_buffer(term)
}

/// FFI export: Get invalid region.
#[no_mangle]
pub extern "C" fn rs_get_terminal_invalid_region(term: TerminalHandle) -> InvalidRegion {
    get_invalid_region(term)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_buffer_state_empty() {
        let state = TerminalBufferState::empty();
        assert_eq!(state.buf_handle, 0);
        assert!(!state.closed);
        assert_eq!(state.sb_current, 0);
        assert!(!state.has_invalid_region());
    }

    #[test]
    fn test_invalid_region() {
        let region = InvalidRegion::none();
        assert!(!region.is_valid());

        let region = InvalidRegion::new(5, 10);
        assert!(region.is_valid());
        assert_eq!(region.start_row, 5);
        assert_eq!(region.end_row, 10);
    }

    #[test]
    fn test_invalid_region_expand() {
        let mut region = InvalidRegion::none();
        region.expand(5);
        assert!(region.is_valid());
        assert_eq!(region.start_row, 5);
        assert_eq!(region.end_row, 5);

        region.expand(3);
        assert_eq!(region.start_row, 3);
        assert_eq!(region.end_row, 5);

        region.expand(10);
        assert_eq!(region.start_row, 3);
        assert_eq!(region.end_row, 10);
    }

    #[test]
    fn test_invalid_region_merge() {
        let mut region1 = InvalidRegion::new(5, 10);
        let region2 = InvalidRegion::new(2, 7);
        region1.merge(&region2);
        assert_eq!(region1.start_row, 2);
        assert_eq!(region1.end_row, 10);
    }

    #[test]
    fn test_buffer_validation_values() {
        assert_eq!(BufferValidation::Valid as c_int, 0);
        assert_eq!(BufferValidation::NullTerminal as c_int, 1);
        assert_eq!(BufferValidation::Closed as c_int, 2);
        assert_eq!(BufferValidation::InvalidBuffer as c_int, 3);
    }

    #[test]
    fn test_struct_sizes() {
        use std::mem::size_of;
        // TerminalBufferState should be reasonable size
        assert!(size_of::<TerminalBufferState>() <= 48);
        // InvalidRegion: 2 * 4 = 8 bytes
        assert_eq!(size_of::<InvalidRegion>(), 8);
    }
}
