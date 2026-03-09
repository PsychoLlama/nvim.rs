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

/// Helper: get shared reference to CTerminal from a handle.
/// # Safety: handle must be non-null and valid.
#[inline]
unsafe fn term_ref(term: TerminalHandle) -> &'static crate::CTerminal {
    unsafe { &*(term.as_ptr() as *const crate::CTerminal) }
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
    let t = unsafe { term_ref(term) };
    TerminalBufferState {
        buf_handle: t.buf_handle,
        closed: t.closed,
        sb_current: t.sb_current,
        sb_size: t.sb_size,
        invalid_start: t.invalid_start,
        invalid_end: t.invalid_end,
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
    let sb_current = unsafe { term_ref(term).sb_current } as LinenrT;
    row + sb_current + 1
}

/// Convert a buffer line number to a terminal row number.
pub fn linenr_to_row(term: TerminalHandle, linenr: LinenrT) -> c_int {
    if term.is_null() {
        return 0;
    }
    let sb_current = unsafe { term_ref(term).sb_current } as LinenrT;
    linenr - sb_current - 1
}

/// Check if a line number is in the scrollback region.
pub fn is_scrollback_line(term: TerminalHandle, linenr: LinenrT) -> bool {
    if term.is_null() || linenr < 1 {
        return false;
    }
    let sb_current = unsafe { term_ref(term).sb_current } as LinenrT;
    linenr <= sb_current
}

/// Check if a line number is in the visible screen region.
pub fn is_screen_line(term: TerminalHandle, linenr: LinenrT) -> bool {
    if term.is_null() || linenr < 1 {
        return false;
    }
    let sb_current = unsafe { term_ref(term).sb_current } as LinenrT;
    linenr > sb_current
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
    let t = unsafe { term_ref(term) };
    if t.closed {
        return BufferValidation::Closed;
    }
    if t.buf_handle <= 0 {
        return BufferValidation::InvalidBuffer;
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
    let t = unsafe { term_ref(term) };
    InvalidRegion {
        start_row: t.invalid_start,
        end_row: t.invalid_end,
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

// =============================================================================
// Cursor Position
// =============================================================================

/// Terminal cursor position (0-based row, 0-based col).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalCursor {
    /// Row position (0-based).
    pub row: c_int,
    /// Column position (0-based).
    pub col: c_int,
}

impl TerminalCursor {
    /// Create a cursor at the origin.
    pub const fn origin() -> Self {
        Self { row: 0, col: 0 }
    }

    /// Create a cursor at a specific position.
    pub const fn new(row: c_int, col: c_int) -> Self {
        Self { row, col }
    }

    /// Convert to buffer position (1-based line number).
    pub fn to_buffer_pos(self, sb_current: usize) -> (LinenrT, c_int) {
        let linenr = self.row + sb_current as LinenrT + 1;
        (linenr, self.col)
    }

    /// Check if cursor is at origin.
    pub const fn is_origin(&self) -> bool {
        self.row == 0 && self.col == 0
    }

    /// Check if cursor is valid (non-negative).
    pub const fn is_valid(&self) -> bool {
        self.row >= 0 && self.col >= 0
    }
}

// =============================================================================
// Buffer Line Iterator (logical structure for range iteration)
// =============================================================================

/// Iterator over terminal buffer line numbers.
///
/// This provides a logical structure for iterating over buffer lines
/// within a range. The actual line data access requires FFI calls.
pub struct BufferLineRange {
    /// Current line number (1-based).
    current: LinenrT,
    /// End line number (1-based, inclusive).
    end: LinenrT,
}

impl BufferLineRange {
    /// Create a new range iterator.
    pub const fn new(start: LinenrT, end: LinenrT) -> Self {
        Self {
            current: start,
            end,
        }
    }

    /// Create a range for scrollback lines only.
    pub fn scrollback_range(sb_current: usize) -> Self {
        if sb_current == 0 {
            Self { current: 1, end: 0 } // Empty range
        } else {
            Self {
                current: 1,
                end: sb_current as LinenrT,
            }
        }
    }

    /// Create a range for screen lines only.
    pub fn screen_range(sb_current: usize, screen_rows: c_int) -> Self {
        let start = sb_current as LinenrT + 1;
        let end = start + screen_rows - 1;
        Self {
            current: start,
            end,
        }
    }

    /// Check if the range is empty.
    pub const fn is_empty(&self) -> bool {
        self.current > self.end
    }

    /// Get the number of lines in the range.
    #[allow(clippy::cast_sign_loss)]
    pub const fn len(&self) -> usize {
        if self.current > self.end {
            0
        } else {
            // Safe: we've already checked current <= end, so result is non-negative
            (self.end - self.current + 1) as usize
        }
    }
}

impl Iterator for BufferLineRange {
    type Item = LinenrT;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current > self.end {
            return None;
        }
        let result = self.current;
        self.current += 1;
        Some(result)
    }
}

impl ExactSizeIterator for BufferLineRange {
    fn len(&self) -> usize {
        BufferLineRange::len(self)
    }
}

// =============================================================================
// Buffer Region Type
// =============================================================================

/// Type of buffer region a line belongs to.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferRegion {
    /// Line is in scrollback history.
    Scrollback = 0,
    /// Line is in the visible screen area.
    Screen = 1,
    /// Line number is invalid.
    Invalid = 2,
}

/// Determine which region a line number belongs to.
#[allow(clippy::cast_sign_loss)]
pub const fn classify_linenr(linenr: LinenrT, sb_current: usize) -> BufferRegion {
    if linenr < 1 {
        return BufferRegion::Invalid;
    }
    // Safe: we've already checked linenr >= 1, so it's non-negative
    if (linenr as usize) <= sb_current {
        BufferRegion::Scrollback
    } else {
        BufferRegion::Screen
    }
}

/// FFI export: Classify a line number.
#[no_mangle]
pub extern "C" fn rs_terminal_classify_linenr(linenr: LinenrT, sb_current: usize) -> BufferRegion {
    classify_linenr(linenr, sb_current)
}

/// FFI export: Get cursor buffer position.
#[no_mangle]
pub extern "C" fn rs_terminal_cursor_to_linenr(cursor_row: c_int, sb_current: usize) -> LinenrT {
    cursor_row + sb_current as LinenrT + 1
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
    fn test_terminal_buffer_state_invalid_region() {
        let state = TerminalBufferState {
            buf_handle: 1,
            closed: false,
            sb_current: 100,
            sb_size: 1000,
            invalid_start: 5,
            invalid_end: 10,
        };
        assert!(state.has_invalid_region());
        assert_eq!(state.invalid_row_count(), 6); // 5, 6, 7, 8, 9, 10
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
    fn test_invalid_region_merge_empty() {
        let mut region1 = InvalidRegion::none();
        let region2 = InvalidRegion::new(5, 10);
        region1.merge(&region2);
        assert_eq!(region1.start_row, 5);
        assert_eq!(region1.end_row, 10);

        let mut region3 = InvalidRegion::new(1, 3);
        let region4 = InvalidRegion::none();
        region3.merge(&region4);
        assert_eq!(region3.start_row, 1);
        assert_eq!(region3.end_row, 3);
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
        // TerminalCursor: 2 * 4 = 8 bytes
        assert_eq!(size_of::<TerminalCursor>(), 8);
    }

    #[test]
    fn test_terminal_cursor() {
        let cursor = TerminalCursor::origin();
        assert!(cursor.is_origin());
        assert!(cursor.is_valid());

        let cursor = TerminalCursor::new(5, 10);
        assert!(!cursor.is_origin());
        assert!(cursor.is_valid());
        assert_eq!(cursor.row, 5);
        assert_eq!(cursor.col, 10);

        // Convert to buffer position with 100 scrollback lines
        let (linenr, col) = cursor.to_buffer_pos(100);
        assert_eq!(linenr, 106); // 5 + 100 + 1
        assert_eq!(col, 10);
    }

    #[test]
    fn test_terminal_cursor_invalid() {
        let cursor = TerminalCursor::new(-1, 5);
        assert!(!cursor.is_valid());

        let cursor = TerminalCursor::new(5, -1);
        assert!(!cursor.is_valid());
    }

    #[test]
    fn test_buffer_line_range() {
        let range = BufferLineRange::new(1, 5);
        assert!(!range.is_empty());
        assert_eq!(range.len(), 5);

        let collected: Vec<_> = BufferLineRange::new(1, 5).collect();
        assert_eq!(collected, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_buffer_line_range_empty() {
        let range = BufferLineRange::new(5, 4); // start > end
        assert!(range.is_empty());
        assert_eq!(range.len(), 0);

        // Verify iterator produces no items
        let count = BufferLineRange::new(5, 4).count();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_buffer_line_range_scrollback() {
        // With 100 scrollback lines
        let range = BufferLineRange::scrollback_range(100);
        assert_eq!(range.len(), 100);

        let first = BufferLineRange::scrollback_range(100).next();
        assert_eq!(first, Some(1));

        // With no scrollback
        let range = BufferLineRange::scrollback_range(0);
        assert!(range.is_empty());
    }

    #[test]
    fn test_buffer_line_range_screen() {
        // 100 scrollback lines, 24 screen rows
        let range = BufferLineRange::screen_range(100, 24);
        assert_eq!(range.len(), 24);

        let first = BufferLineRange::screen_range(100, 24).next();
        assert_eq!(first, Some(101)); // First screen line
    }

    #[test]
    fn test_classify_linenr() {
        // With 100 scrollback lines
        assert_eq!(classify_linenr(1, 100), BufferRegion::Scrollback);
        assert_eq!(classify_linenr(50, 100), BufferRegion::Scrollback);
        assert_eq!(classify_linenr(100, 100), BufferRegion::Scrollback);
        assert_eq!(classify_linenr(101, 100), BufferRegion::Screen);
        assert_eq!(classify_linenr(150, 100), BufferRegion::Screen);

        // Invalid line numbers
        assert_eq!(classify_linenr(0, 100), BufferRegion::Invalid);
        assert_eq!(classify_linenr(-1, 100), BufferRegion::Invalid);

        // No scrollback
        assert_eq!(classify_linenr(1, 0), BufferRegion::Screen);
    }

    #[test]
    fn test_buffer_region_values() {
        assert_eq!(BufferRegion::Scrollback as c_int, 0);
        assert_eq!(BufferRegion::Screen as c_int, 1);
        assert_eq!(BufferRegion::Invalid as c_int, 2);
    }

    #[test]
    fn test_null_handle_behavior() {
        let null_term = TerminalHandle::null();
        assert!(null_term.is_null());

        let null_buf = BufHandle::null();
        assert!(null_buf.is_null());

        // Test validate_terminal_buffer with null
        assert_eq!(
            validate_terminal_buffer(TerminalHandle::null()),
            BufferValidation::NullTerminal
        );
    }
}
