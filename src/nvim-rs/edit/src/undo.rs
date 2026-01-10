//! Undo helpers for edit mode
//!
//! This module provides helpers for undo management during insert mode,
//! including undo state tracking, sync control, and change grouping.

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::c_int;

// =============================================================================
// Undo State Constants
// =============================================================================

/// Undo sync state (for controlling when undo breaks happen).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UndoSyncState {
    /// Allow undo sync
    #[default]
    Allow = 0,
    /// Don't sync undo
    DontSync = 1,
    /// Force undo sync
    ForceSync = 2,
}

impl UndoSyncState {
    /// Convert from raw integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            0 => Self::Allow,
            1 => Self::DontSync,
            _ => Self::ForceSync,
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Check if undo sync is allowed.
    #[must_use]
    pub const fn allows_sync(&self) -> bool {
        !matches!(self, Self::DontSync)
    }
}

// =============================================================================
// Undo Change Flags
// =============================================================================

/// Flags for undo change types.
pub mod undo_flags {
    use std::ffi::c_int;

    /// Line was inserted
    pub const UND_INSERT: c_int = 0x01;
    /// Line was deleted
    pub const UND_DELETE: c_int = 0x02;
    /// Line was changed
    pub const UND_CHANGED: c_int = 0x04;
    /// Start of undo block
    pub const UND_BLOCK_START: c_int = 0x08;
    /// End of undo block
    pub const UND_BLOCK_END: c_int = 0x10;
    /// Mark for cursor position
    pub const UND_MARK: c_int = 0x20;
}

/// Check if undo flags have a specific flag set.
#[must_use]
#[inline]
pub const fn has_undo_flag(flags: c_int, flag: c_int) -> bool {
    (flags & flag) != 0
}

/// Set an undo flag.
#[must_use]
#[inline]
pub const fn set_undo_flag(flags: c_int, flag: c_int) -> c_int {
    flags | flag
}

/// Clear an undo flag.
#[must_use]
#[inline]
pub const fn clear_undo_flag(flags: c_int, flag: c_int) -> c_int {
    flags & !flag
}

// =============================================================================
// Undo State Tracking
// =============================================================================

/// State for tracking undo during insert mode.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct UndoInsertState {
    /// Whether undo is needed for next insert
    pub need_undo: bool,
    /// Current sync state
    pub sync_state: c_int,
    /// Undo sequence number at start
    pub start_seq: i64,
    /// Number of changes in current block
    pub change_count: c_int,
}

impl UndoInsertState {
    /// Create a new undo insert state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            need_undo: true,
            sync_state: 0,
            start_seq: 0,
            change_count: 0,
        }
    }

    /// Check if undo is needed.
    #[must_use]
    pub const fn needs_undo(&self) -> bool {
        self.need_undo
    }

    /// Mark that undo was saved.
    pub fn mark_saved(&mut self) {
        self.need_undo = false;
    }

    /// Mark that undo is needed again.
    pub fn mark_needed(&mut self) {
        self.need_undo = true;
    }

    /// Increment change count.
    pub fn add_change(&mut self) {
        self.change_count += 1;
    }

    /// Check if there are pending changes.
    #[must_use]
    pub const fn has_changes(&self) -> bool {
        self.change_count > 0
    }
}

// =============================================================================
// Undo Block Management
// =============================================================================

/// State for managing undo blocks.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct UndoBlockState {
    /// Nesting level of undo blocks
    pub nesting_level: c_int,
    /// Whether currently in an undo block
    pub in_block: bool,
    /// Whether block was opened implicitly
    pub implicit: bool,
}

impl UndoBlockState {
    /// Create a new undo block state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            nesting_level: 0,
            in_block: false,
            implicit: false,
        }
    }

    /// Start an undo block.
    pub fn start_block(&mut self, implicit: bool) {
        if self.nesting_level == 0 {
            self.in_block = true;
            self.implicit = implicit;
        }
        self.nesting_level += 1;
    }

    /// End an undo block.
    pub fn end_block(&mut self) -> bool {
        if self.nesting_level > 0 {
            self.nesting_level -= 1;
            if self.nesting_level == 0 {
                self.in_block = false;
                return true; // Block completed
            }
        }
        false
    }

    /// Check if inside an undo block.
    #[must_use]
    pub const fn is_in_block(&self) -> bool {
        self.in_block
    }

    /// Get nesting level.
    #[must_use]
    pub const fn level(&self) -> c_int {
        self.nesting_level
    }
}

// =============================================================================
// Undo Range
// =============================================================================

/// Range of lines affected by an undo operation.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct UndoRange {
    /// First line (1-based)
    pub first: i32,
    /// Last line (1-based)
    pub last: i32,
    /// Number of lines added (negative for deleted)
    pub lines_added: i32,
}

impl UndoRange {
    /// Create a new undo range.
    #[must_use]
    pub const fn new(first: i32, last: i32, lines_added: i32) -> Self {
        Self {
            first,
            last,
            lines_added,
        }
    }

    /// Create an empty range.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            first: 0,
            last: 0,
            lines_added: 0,
        }
    }

    /// Check if range is valid.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.first > 0 && self.last >= self.first
    }

    /// Get number of lines in range.
    #[must_use]
    pub const fn line_count(&self) -> i32 {
        if self.is_valid() {
            self.last - self.first + 1
        } else {
            0
        }
    }

    /// Merge with another range.
    #[must_use]
    pub const fn merge(&self, other: &Self) -> Self {
        if !self.is_valid() {
            return *other;
        }
        if !other.is_valid() {
            return *self;
        }
        Self {
            first: if self.first < other.first {
                self.first
            } else {
                other.first
            },
            last: if self.last > other.last {
                self.last
            } else {
                other.last
            },
            lines_added: self.lines_added + other.lines_added,
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Get undo sync state from raw value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_undo_sync_state(value: c_int) -> c_int {
    UndoSyncState::from_raw(value).to_raw()
}

/// Check if undo sync is allowed.
#[unsafe(no_mangle)]
pub extern "C" fn rs_undo_allows_sync(value: c_int) -> c_int {
    c_int::from(UndoSyncState::from_raw(value).allows_sync())
}

/// Check if undo flags have a specific flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_undo_has_flag(flags: c_int, flag: c_int) -> c_int {
    c_int::from(has_undo_flag(flags, flag))
}

/// Set an undo flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_undo_set_flag(flags: c_int, flag: c_int) -> c_int {
    set_undo_flag(flags, flag)
}

/// Clear an undo flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_undo_clear_flag(flags: c_int, flag: c_int) -> c_int {
    clear_undo_flag(flags, flag)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undo_sync_state() {
        assert_eq!(UndoSyncState::from_raw(0), UndoSyncState::Allow);
        assert_eq!(UndoSyncState::from_raw(1), UndoSyncState::DontSync);
        assert_eq!(UndoSyncState::from_raw(2), UndoSyncState::ForceSync);

        assert!(UndoSyncState::Allow.allows_sync());
        assert!(!UndoSyncState::DontSync.allows_sync());
        assert!(UndoSyncState::ForceSync.allows_sync());
    }

    #[test]
    fn test_undo_flags() {
        let flags = 0;
        assert!(!has_undo_flag(flags, undo_flags::UND_INSERT));

        let flags = set_undo_flag(flags, undo_flags::UND_INSERT);
        assert!(has_undo_flag(flags, undo_flags::UND_INSERT));

        let flags = set_undo_flag(flags, undo_flags::UND_DELETE);
        assert!(has_undo_flag(flags, undo_flags::UND_INSERT));
        assert!(has_undo_flag(flags, undo_flags::UND_DELETE));

        let flags = clear_undo_flag(flags, undo_flags::UND_INSERT);
        assert!(!has_undo_flag(flags, undo_flags::UND_INSERT));
        assert!(has_undo_flag(flags, undo_flags::UND_DELETE));
    }

    #[test]
    fn test_undo_insert_state() {
        let mut state = UndoInsertState::new();
        assert!(state.needs_undo());
        assert!(!state.has_changes());

        state.mark_saved();
        assert!(!state.needs_undo());

        state.add_change();
        assert!(state.has_changes());
        assert_eq!(state.change_count, 1);

        state.mark_needed();
        assert!(state.needs_undo());
    }

    #[test]
    fn test_undo_block_state() {
        let mut state = UndoBlockState::new();
        assert!(!state.is_in_block());
        assert_eq!(state.level(), 0);

        state.start_block(false);
        assert!(state.is_in_block());
        assert_eq!(state.level(), 1);

        state.start_block(false); // Nested
        assert_eq!(state.level(), 2);

        assert!(!state.end_block()); // Still nested
        assert_eq!(state.level(), 1);

        assert!(state.end_block()); // Block completed
        assert!(!state.is_in_block());
        assert_eq!(state.level(), 0);
    }

    #[test]
    fn test_undo_range() {
        let range = UndoRange::new(5, 10, 3);
        assert!(range.is_valid());
        assert_eq!(range.line_count(), 6);

        let empty = UndoRange::empty();
        assert!(!empty.is_valid());
        assert_eq!(empty.line_count(), 0);

        let range2 = UndoRange::new(8, 15, 2);
        let merged = range.merge(&range2);
        assert_eq!(merged.first, 5);
        assert_eq!(merged.last, 15);
        assert_eq!(merged.lines_added, 5);
    }
}
