//! Replace mode helpers for edit mode
//!
//! This module provides helpers for replace mode operations,
//! including character replacement tracking and virtual replace mode.

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]

use std::ffi::c_int;

// =============================================================================
// Replace Mode Types
// =============================================================================

/// Replace mode types.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReplaceMode {
    /// Normal insert mode (not replace)
    #[default]
    Insert = 0,
    /// Replace mode (R)
    Replace = 1,
    /// Virtual replace mode (gR)
    VirtualReplace = 2,
}

impl ReplaceMode {
    /// Convert from raw integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::Replace,
            2 => Self::VirtualReplace,
            _ => Self::Insert,
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Check if in any replace mode.
    #[must_use]
    pub const fn is_replace(&self) -> bool {
        !matches!(self, Self::Insert)
    }

    /// Check if in virtual replace mode.
    #[must_use]
    pub const fn is_virtual(&self) -> bool {
        matches!(self, Self::VirtualReplace)
    }
}

// =============================================================================
// Replace Stack Entry
// =============================================================================

/// Entry in the replace stack (tracks replaced characters).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ReplaceEntry {
    /// Original character that was replaced
    pub orig_char: i32,
    /// Whether this was a multi-byte character
    pub is_multibyte: bool,
    /// Extra bytes for multi-byte characters
    pub extra_bytes: u8,
}

impl ReplaceEntry {
    /// Create a new replace entry for a single-byte character.
    #[must_use]
    pub const fn single(c: i32) -> Self {
        Self {
            orig_char: c,
            is_multibyte: false,
            extra_bytes: 0,
        }
    }

    /// Create a new replace entry for a multi-byte character.
    #[must_use]
    pub const fn multibyte(c: i32, extra: u8) -> Self {
        Self {
            orig_char: c,
            is_multibyte: true,
            extra_bytes: extra,
        }
    }

    /// Check if this is a NUL entry (no replacement).
    #[must_use]
    pub const fn is_nul(&self) -> bool {
        self.orig_char == 0 && !self.is_multibyte
    }

    /// Get total byte count for this entry.
    #[must_use]
    pub const fn byte_count(&self) -> usize {
        if self.is_multibyte {
            1 + self.extra_bytes as usize
        } else {
            1
        }
    }
}

// =============================================================================
// Replace State
// =============================================================================

/// Maximum size of replace stack.
pub const REPLACE_STACK_MAX: usize = 1024;

/// State for replace mode.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ReplaceState {
    /// Current replace mode
    pub mode: c_int,
    /// Stack pointer (number of entries)
    pub stack_ptr: c_int,
    /// Column where replace started
    pub start_col: c_int,
    /// Virtual column for virtual replace
    pub vcol: c_int,
    /// Whether we've replaced past end of line
    pub past_eol: bool,
}

impl ReplaceState {
    /// Create a new replace state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            mode: 0,
            stack_ptr: 0,
            start_col: 0,
            vcol: 0,
            past_eol: false,
        }
    }

    /// Get the replace mode.
    #[must_use]
    pub const fn get_mode(&self) -> ReplaceMode {
        ReplaceMode::from_raw(self.mode)
    }

    /// Check if in replace mode.
    #[must_use]
    pub const fn is_replace(&self) -> bool {
        self.mode != 0
    }

    /// Check if stack is empty.
    #[must_use]
    pub const fn stack_empty(&self) -> bool {
        self.stack_ptr == 0
    }

    /// Check if stack has room.
    #[must_use]
    pub const fn stack_has_room(&self) -> bool {
        (self.stack_ptr as usize) < REPLACE_STACK_MAX
    }

    /// Push to stack (increment pointer).
    pub fn stack_push(&mut self) {
        if self.stack_has_room() {
            self.stack_ptr += 1;
        }
    }

    /// Pop from stack (decrement pointer).
    pub fn stack_pop(&mut self) -> bool {
        if self.stack_ptr > 0 {
            self.stack_ptr -= 1;
            true
        } else {
            false
        }
    }

    /// Enter replace mode.
    pub fn enter(&mut self, mode: ReplaceMode, col: c_int) {
        self.mode = mode.to_raw();
        self.start_col = col;
        self.stack_ptr = 0;
        self.past_eol = false;
    }

    /// Exit replace mode.
    pub fn exit(&mut self) {
        self.mode = 0;
        self.stack_ptr = 0;
    }
}

// =============================================================================
// Virtual Replace Helpers
// =============================================================================

/// Calculate how many screen columns a character takes.
///
/// This is a simplified version - the full implementation uses
/// the actual character and 'tabstop' setting.
#[must_use]
pub const fn char_cells(c: i32, vcol: c_int, ts: c_int) -> c_int {
    if c == b'\t' as i32 {
        // Tab: expands to next tab stop
        ts - (vcol % ts)
    } else if c < 32 || c == 127 {
        // Control character: ^X format
        2
    } else {
        // Regular character
        1
    }
}

/// State for virtual replace column tracking.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct VirtualReplaceState {
    /// Current virtual column
    pub vcol: c_int,
    /// Virtual column at start of replaced character
    pub vcol_start: c_int,
    /// Number of screen columns being replaced
    pub replace_width: c_int,
}

impl VirtualReplaceState {
    /// Create a new virtual replace state.
    #[must_use]
    pub const fn new(vcol: c_int) -> Self {
        Self {
            vcol,
            vcol_start: vcol,
            replace_width: 0,
        }
    }

    /// Set the width of the character being replaced.
    pub fn set_replace_width(&mut self, width: c_int) {
        self.vcol_start = self.vcol;
        self.replace_width = width;
    }

    /// Advance virtual column by the given width.
    pub fn advance(&mut self, width: c_int) {
        self.vcol += width;
    }

    /// Check if replacement needs padding.
    #[must_use]
    pub const fn needs_padding(&self, new_width: c_int) -> bool {
        new_width < self.replace_width
    }

    /// Get padding needed for replacement.
    #[must_use]
    pub const fn padding_needed(&self, new_width: c_int) -> c_int {
        if new_width < self.replace_width {
            self.replace_width - new_width
        } else {
            0
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Get replace mode from raw value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_replace_mode(value: c_int) -> c_int {
    ReplaceMode::from_raw(value).to_raw()
}

/// Check if value indicates replace mode.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_replace_mode(value: c_int) -> c_int {
    c_int::from(ReplaceMode::from_raw(value).is_replace())
}

/// Check if value indicates virtual replace mode.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_virtual_replace(value: c_int) -> c_int {
    c_int::from(ReplaceMode::from_raw(value).is_virtual())
}

/// Calculate character cell width.
#[unsafe(no_mangle)]
pub extern "C" fn rs_char_cells(c: c_int, vcol: c_int, ts: c_int) -> c_int {
    char_cells(c, vcol, ts)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_mode() {
        assert_eq!(ReplaceMode::from_raw(0), ReplaceMode::Insert);
        assert_eq!(ReplaceMode::from_raw(1), ReplaceMode::Replace);
        assert_eq!(ReplaceMode::from_raw(2), ReplaceMode::VirtualReplace);
        assert_eq!(ReplaceMode::from_raw(99), ReplaceMode::Insert);

        assert!(!ReplaceMode::Insert.is_replace());
        assert!(ReplaceMode::Replace.is_replace());
        assert!(ReplaceMode::VirtualReplace.is_replace());

        assert!(!ReplaceMode::Replace.is_virtual());
        assert!(ReplaceMode::VirtualReplace.is_virtual());
    }

    #[test]
    fn test_replace_entry() {
        let single = ReplaceEntry::single(b'x' as i32);
        assert!(!single.is_multibyte);
        assert_eq!(single.byte_count(), 1);
        assert!(!single.is_nul());

        let multi = ReplaceEntry::multibyte(0xC3, 1);
        assert!(multi.is_multibyte);
        assert_eq!(multi.byte_count(), 2);

        let nul = ReplaceEntry::single(0);
        assert!(nul.is_nul());
    }

    #[test]
    fn test_replace_state() {
        let mut state = ReplaceState::new();
        assert!(!state.is_replace());
        assert!(state.stack_empty());

        state.enter(ReplaceMode::Replace, 10);
        assert!(state.is_replace());
        assert_eq!(state.start_col, 10);
        assert!(state.stack_empty());

        state.stack_push();
        assert!(!state.stack_empty());
        assert_eq!(state.stack_ptr, 1);

        assert!(state.stack_pop());
        assert!(state.stack_empty());

        assert!(!state.stack_pop()); // Can't pop from empty stack

        state.exit();
        assert!(!state.is_replace());
    }

    #[test]
    fn test_char_cells() {
        // Regular character
        assert_eq!(char_cells(b'a' as i32, 0, 8), 1);

        // Tab at column 0 with tabstop 8
        assert_eq!(char_cells(b'\t' as i32, 0, 8), 8);

        // Tab at column 5 with tabstop 8
        assert_eq!(char_cells(b'\t' as i32, 5, 8), 3);

        // Control character (e.g., ^A)
        assert_eq!(char_cells(1, 0, 8), 2);
    }

    #[test]
    fn test_virtual_replace_state() {
        let mut state = VirtualReplaceState::new(10);
        assert_eq!(state.vcol, 10);

        state.set_replace_width(4);
        assert_eq!(state.replace_width, 4);

        assert!(state.needs_padding(2));
        assert_eq!(state.padding_needed(2), 2);

        assert!(!state.needs_padding(5));
        assert_eq!(state.padding_needed(5), 0);

        state.advance(3);
        assert_eq!(state.vcol, 13);
    }
}
