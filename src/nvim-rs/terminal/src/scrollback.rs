//! Scrollback buffer management
//!
//! This module provides Rust implementations for terminal scrollback buffer
//! operations, including line storage, retrieval, and capacity management.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::use_self)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::implicit_saturating_sub)]
#![allow(clippy::bool_comparison)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::comparison_chain)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::c_void;
use std::os::raw::c_int;

// =============================================================================
// Constants
// =============================================================================

/// Maximum scrollback buffer size.
pub const TERMINAL_SB_MAX: usize = 100_000;

/// Default scrollback buffer size.
pub const TERMINAL_SB_DEFAULT: usize = 10_000;

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
}

// =============================================================================
// Internal Helpers
// =============================================================================

/// Helper: get shared reference to CTerminal from a handle.
/// # Safety: handle must be non-null and valid.
#[inline]
unsafe fn term_ref(term: TerminalHandle) -> &'static crate::CTerminal {
    unsafe { &*(term.0 as *const crate::CTerminal) }
}

// =============================================================================
// Scrollback State
// =============================================================================

/// Scrollback buffer state.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ScrollbackState {
    /// Current number of lines in scrollback.
    pub current: usize,
    /// Maximum capacity of scrollback buffer.
    pub size: usize,
    /// Number of lines pending to be added to scrollback.
    pub pending: c_int,
    /// Number of lines that have been deleted from scrollback.
    pub deleted: usize,
}

impl ScrollbackState {
    /// Create an empty state.
    pub const fn empty() -> Self {
        Self {
            current: 0,
            size: 0,
            pending: 0,
            deleted: 0,
        }
    }

    /// Check if the scrollback buffer is empty.
    pub const fn is_empty(&self) -> bool {
        self.current == 0
    }

    /// Check if the scrollback buffer is full.
    pub const fn is_full(&self) -> bool {
        self.current >= self.size
    }

    /// Get the available capacity.
    pub const fn available(&self) -> usize {
        if self.size > self.current {
            self.size - self.current
        } else {
            0
        }
    }

    /// Get the fill percentage (0-100).
    pub const fn fill_percent(&self) -> u8 {
        if self.size == 0 {
            return 0;
        }
        let percent = (self.current * 100) / self.size;
        if percent > 100 {
            100
        } else {
            percent as u8
        }
    }

    /// Check if there are pending lines to process.
    pub const fn has_pending(&self) -> bool {
        self.pending > 0
    }
}

/// Get the scrollback state from a terminal.
pub fn get_scrollback_state(term: TerminalHandle) -> ScrollbackState {
    if term.is_null() {
        return ScrollbackState::empty();
    }

    let t = unsafe { term_ref(term) };
    ScrollbackState {
        current: t.sb_current,
        size: t.sb_size,
        pending: t.sb_pending,
        deleted: t.sb_deleted,
    }
}

// =============================================================================
// Scrollback Line Info
// =============================================================================

/// Information about a scrollback line.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ScrollbackLine {
    /// Index in the scrollback buffer (0 = oldest).
    pub index: usize,
    /// Column count for this line.
    pub cols: c_int,
    /// Whether this line wraps to the next.
    pub wrapped: bool,
}

impl ScrollbackLine {
    /// Create an empty line info.
    pub const fn empty() -> Self {
        Self {
            index: 0,
            cols: 0,
            wrapped: false,
        }
    }
}

// =============================================================================
// Scrollback Operations
// =============================================================================

/// Result of scrollback operations.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollbackResult {
    /// Operation successful.
    Success = 0,
    /// Terminal is null.
    NullTerminal = 1,
    /// Index out of bounds.
    OutOfBounds = 2,
    /// Buffer is full.
    BufferFull = 3,
    /// Buffer is empty.
    BufferEmpty = 4,
}

/// Check if a scrollback index is valid.
pub fn is_valid_scrollback_index(term: TerminalHandle, index: usize) -> bool {
    if term.is_null() {
        return false;
    }

    let current = unsafe { term_ref(term).sb_current };
    index < current
}

/// Calculate the new scrollback size based on an option change.
///
/// This handles the 'scrollback' option being changed and determines
/// what the new size should be.
pub fn calculate_scrollback_size(requested: isize) -> usize {
    if requested < 0 {
        // Negative means unlimited (use max)
        TERMINAL_SB_MAX
    } else if requested == 0 {
        // Zero means default
        TERMINAL_SB_DEFAULT
    } else {
        // Positive value, clamp to max
        let requested = requested as usize;
        if requested > TERMINAL_SB_MAX {
            TERMINAL_SB_MAX
        } else {
            requested
        }
    }
}

/// Calculate how many lines need to be trimmed when scrollback shrinks.
pub fn calculate_trim_count(state: &ScrollbackState, new_size: usize) -> usize {
    if new_size >= state.current {
        0
    } else {
        state.current - new_size
    }
}

// =============================================================================
// Scrollback Adjustment
// =============================================================================

/// Result of a scrollback adjustment (resize/trim).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ScrollbackAdjustment {
    /// New scrollback size.
    pub new_size: usize,
    /// Number of lines to trim.
    pub lines_to_trim: usize,
    /// Whether the buffer needs to be resized.
    pub needs_resize: bool,
}

impl ScrollbackAdjustment {
    /// Create a no-op adjustment.
    pub const fn none() -> Self {
        Self {
            new_size: 0,
            lines_to_trim: 0,
            needs_resize: false,
        }
    }
}

/// Calculate what adjustment is needed for a new scrollback size.
pub fn calculate_adjustment(term: TerminalHandle, new_size: usize) -> ScrollbackAdjustment {
    if term.is_null() {
        return ScrollbackAdjustment::none();
    }

    let state = get_scrollback_state(term);

    if new_size == state.size {
        return ScrollbackAdjustment::none();
    }

    let lines_to_trim = calculate_trim_count(&state, new_size);

    ScrollbackAdjustment {
        new_size,
        lines_to_trim,
        needs_resize: true,
    }
}

// =============================================================================
// Scrollback Push/Pop
// =============================================================================

/// Information for pushing a line to scrollback.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ScrollbackPush {
    /// Number of columns in the line.
    pub cols: c_int,
    /// Whether we need to evict oldest line.
    pub evict_oldest: bool,
    /// Index where the line will be stored.
    pub target_index: usize,
}

impl ScrollbackPush {
    /// Create a failed push (no room).
    pub const fn failed() -> Self {
        Self {
            cols: 0,
            evict_oldest: false,
            target_index: 0,
        }
    }
}

/// Calculate where a new scrollback line should go.
pub fn calculate_push(term: TerminalHandle, cols: c_int) -> ScrollbackPush {
    if term.is_null() {
        return ScrollbackPush::failed();
    }

    let state = get_scrollback_state(term);

    if state.size == 0 {
        return ScrollbackPush::failed();
    }

    let evict_oldest = state.current >= state.size;
    let target_index = if evict_oldest {
        // Will replace oldest, which is at index 0 after rotation
        state.size - 1
    } else {
        state.current
    };

    ScrollbackPush {
        cols,
        evict_oldest,
        target_index,
    }
}

/// Information for popping a line from scrollback.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ScrollbackPop {
    /// Whether pop is possible.
    pub can_pop: bool,
    /// Index of line to pop (newest).
    pub pop_index: usize,
}

impl ScrollbackPop {
    /// Create a failed pop (empty buffer).
    pub const fn failed() -> Self {
        Self {
            can_pop: false,
            pop_index: 0,
        }
    }
}

/// Calculate which line to pop from scrollback.
pub fn calculate_pop(term: TerminalHandle) -> ScrollbackPop {
    if term.is_null() {
        return ScrollbackPop::failed();
    }

    let state = get_scrollback_state(term);

    if state.is_empty() {
        return ScrollbackPop::failed();
    }

    ScrollbackPop {
        can_pop: true,
        pop_index: state.current - 1,
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Get scrollback state.
#[no_mangle]
pub extern "C" fn rs_get_scrollback_state(term: TerminalHandle) -> ScrollbackState {
    get_scrollback_state(term)
}

/// FFI export: Check if scrollback index is valid.
#[no_mangle]
pub extern "C" fn rs_is_valid_scrollback_index(term: TerminalHandle, index: usize) -> c_int {
    c_int::from(is_valid_scrollback_index(term, index))
}

/// FFI export: Calculate scrollback size from option.
#[no_mangle]
pub extern "C" fn rs_calculate_scrollback_size(requested: isize) -> usize {
    calculate_scrollback_size(requested)
}

/// FFI export: Calculate scrollback adjustment.
#[no_mangle]
pub extern "C" fn rs_calculate_scrollback_adjustment(
    term: TerminalHandle,
    new_size: usize,
) -> ScrollbackAdjustment {
    calculate_adjustment(term, new_size)
}

/// FFI export: Calculate scrollback push.
#[no_mangle]
pub extern "C" fn rs_calculate_scrollback_push(
    term: TerminalHandle,
    cols: c_int,
) -> ScrollbackPush {
    calculate_push(term, cols)
}

/// FFI export: Calculate scrollback pop.
#[no_mangle]
pub extern "C" fn rs_calculate_scrollback_pop(term: TerminalHandle) -> ScrollbackPop {
    calculate_pop(term)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrollback_constants() {
        assert_eq!(TERMINAL_SB_MAX, 100_000);
        assert_eq!(TERMINAL_SB_DEFAULT, 10_000);
    }

    #[test]
    fn test_scrollback_state_empty() {
        let state = ScrollbackState::empty();
        assert!(state.is_empty());
        assert!(!state.is_full());
        assert_eq!(state.available(), 0);
        assert_eq!(state.fill_percent(), 0);
        assert!(!state.has_pending());
    }

    #[test]
    fn test_scrollback_state_full() {
        let state = ScrollbackState {
            current: 100,
            size: 100,
            pending: 0,
            deleted: 0,
        };
        assert!(!state.is_empty());
        assert!(state.is_full());
        assert_eq!(state.available(), 0);
        assert_eq!(state.fill_percent(), 100);
    }

    #[test]
    fn test_scrollback_state_partial() {
        let state = ScrollbackState {
            current: 50,
            size: 100,
            pending: 5,
            deleted: 0,
        };
        assert!(!state.is_empty());
        assert!(!state.is_full());
        assert_eq!(state.available(), 50);
        assert_eq!(state.fill_percent(), 50);
        assert!(state.has_pending());
    }

    #[test]
    fn test_calculate_scrollback_size() {
        // Negative = unlimited
        assert_eq!(calculate_scrollback_size(-1), TERMINAL_SB_MAX);
        assert_eq!(calculate_scrollback_size(-100), TERMINAL_SB_MAX);

        // Zero = default
        assert_eq!(calculate_scrollback_size(0), TERMINAL_SB_DEFAULT);

        // Positive = clamped
        assert_eq!(calculate_scrollback_size(1000), 1000);
        assert_eq!(calculate_scrollback_size(500_000), TERMINAL_SB_MAX);
    }

    #[test]
    fn test_calculate_trim_count() {
        let state = ScrollbackState {
            current: 100,
            size: 100,
            pending: 0,
            deleted: 0,
        };

        // No trim needed if new size >= current
        assert_eq!(calculate_trim_count(&state, 100), 0);
        assert_eq!(calculate_trim_count(&state, 150), 0);

        // Trim needed if new size < current
        assert_eq!(calculate_trim_count(&state, 80), 20);
        assert_eq!(calculate_trim_count(&state, 0), 100);
    }

    #[test]
    fn test_scrollback_result_values() {
        assert_eq!(ScrollbackResult::Success as c_int, 0);
        assert_eq!(ScrollbackResult::NullTerminal as c_int, 1);
        assert_eq!(ScrollbackResult::OutOfBounds as c_int, 2);
        assert_eq!(ScrollbackResult::BufferFull as c_int, 3);
        assert_eq!(ScrollbackResult::BufferEmpty as c_int, 4);
    }

    #[test]
    fn test_scrollback_adjustment_none() {
        let adj = ScrollbackAdjustment::none();
        assert_eq!(adj.new_size, 0);
        assert_eq!(adj.lines_to_trim, 0);
        assert!(!adj.needs_resize);
    }

    #[test]
    fn test_scrollback_push_failed() {
        let push = ScrollbackPush::failed();
        assert_eq!(push.cols, 0);
        assert!(!push.evict_oldest);
    }

    #[test]
    fn test_scrollback_pop_failed() {
        let pop = ScrollbackPop::failed();
        assert!(!pop.can_pop);
    }

    #[test]
    fn test_scrollback_line_empty() {
        let line = ScrollbackLine::empty();
        assert_eq!(line.index, 0);
        assert_eq!(line.cols, 0);
        assert!(!line.wrapped);
    }
}
