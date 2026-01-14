//! Terminal output handling
//!
//! This module provides Rust implementations for terminal output operations,
//! including screen updates, bell handling, and title notifications.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::use_self)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_void;
use std::os::raw::c_int;

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
// Output Event Types
// =============================================================================

/// Type of terminal output event.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputEventType {
    /// Screen damage (region needs redraw).
    Damage = 0,
    /// Move rectangle (scroll).
    MoveRect = 1,
    /// Cursor move.
    CursorMove = 2,
    /// Property change.
    PropChange = 3,
    /// Bell notification.
    Bell = 4,
    /// Resize request.
    Resize = 5,
    /// Scrollback push.
    ScrollbackPush = 6,
    /// Scrollback pop.
    ScrollbackPop = 7,
}

/// Terminal output event.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct OutputEvent {
    /// Event type.
    pub event_type: OutputEventType,
    /// Start row (for damage/move).
    pub start_row: c_int,
    /// End row (for damage/move).
    pub end_row: c_int,
    /// Start column (for damage/move).
    pub start_col: c_int,
    /// End column (for damage/move).
    pub end_col: c_int,
    /// Property ID (for prop change).
    pub prop_id: c_int,
}

impl OutputEvent {
    /// Create a damage event.
    pub const fn damage(
        start_row: c_int,
        end_row: c_int,
        start_col: c_int,
        end_col: c_int,
    ) -> Self {
        Self {
            event_type: OutputEventType::Damage,
            start_row,
            end_row,
            start_col,
            end_col,
            prop_id: 0,
        }
    }

    /// Create a bell event.
    pub const fn bell() -> Self {
        Self {
            event_type: OutputEventType::Bell,
            start_row: 0,
            end_row: 0,
            start_col: 0,
            end_col: 0,
            prop_id: 0,
        }
    }

    /// Create a cursor move event.
    pub const fn cursor_move(row: c_int, col: c_int) -> Self {
        Self {
            event_type: OutputEventType::CursorMove,
            start_row: row,
            end_row: row,
            start_col: col,
            end_col: col,
            prop_id: 0,
        }
    }

    /// Create a property change event.
    pub const fn prop_change(prop_id: c_int) -> Self {
        Self {
            event_type: OutputEventType::PropChange,
            start_row: 0,
            end_row: 0,
            start_col: 0,
            end_col: 0,
            prop_id,
        }
    }

    /// Create a resize event.
    pub const fn resize(rows: c_int, cols: c_int) -> Self {
        Self {
            event_type: OutputEventType::Resize,
            start_row: 0,
            end_row: rows,
            start_col: 0,
            end_col: cols,
            prop_id: 0,
        }
    }
}

// =============================================================================
// Bell Types
// =============================================================================

/// Bell configuration flags.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BellFlags {
    flags: c_int,
}

/// Visual bell flag.
pub const BELL_VISUAL: c_int = 1;
/// Audio bell flag.
pub const BELL_AUDIO: c_int = 2;
/// Urgent bell flag (window manager notification).
pub const BELL_URGENT: c_int = 4;

impl BellFlags {
    /// No bell.
    pub const fn none() -> Self {
        Self { flags: 0 }
    }

    /// All bell types enabled.
    pub const fn all() -> Self {
        Self {
            flags: BELL_VISUAL | BELL_AUDIO | BELL_URGENT,
        }
    }

    /// Create from raw flags.
    pub const fn from_raw(flags: c_int) -> Self {
        Self { flags }
    }

    /// Get raw flags.
    pub const fn as_raw(self) -> c_int {
        self.flags
    }

    /// Check if visual bell is enabled.
    pub const fn visual(self) -> bool {
        (self.flags & BELL_VISUAL) != 0
    }

    /// Check if audio bell is enabled.
    pub const fn audio(self) -> bool {
        (self.flags & BELL_AUDIO) != 0
    }

    /// Check if urgent bell is enabled.
    pub const fn urgent(self) -> bool {
        (self.flags & BELL_URGENT) != 0
    }

    /// Enable visual bell.
    pub const fn with_visual(self) -> Self {
        Self {
            flags: self.flags | BELL_VISUAL,
        }
    }

    /// Enable audio bell.
    pub const fn with_audio(self) -> Self {
        Self {
            flags: self.flags | BELL_AUDIO,
        }
    }

    /// Enable urgent bell.
    pub const fn with_urgent(self) -> Self {
        Self {
            flags: self.flags | BELL_URGENT,
        }
    }
}

// =============================================================================
// Screen Update Types
// =============================================================================

/// Screen damage region.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DamageRegion {
    /// Start row (inclusive).
    pub start_row: c_int,
    /// End row (exclusive).
    pub end_row: c_int,
    /// Start column (inclusive).
    pub start_col: c_int,
    /// End column (exclusive).
    pub end_col: c_int,
}

impl DamageRegion {
    /// Create an empty region.
    pub const fn empty() -> Self {
        Self {
            start_row: 0,
            end_row: 0,
            start_col: 0,
            end_col: 0,
        }
    }

    /// Create a region covering specified bounds.
    pub const fn new(start_row: c_int, end_row: c_int, start_col: c_int, end_col: c_int) -> Self {
        Self {
            start_row,
            end_row,
            start_col,
            end_col,
        }
    }

    /// Create a full-width region for the given rows.
    pub const fn full_width(start_row: c_int, end_row: c_int, cols: c_int) -> Self {
        Self {
            start_row,
            end_row,
            start_col: 0,
            end_col: cols,
        }
    }

    /// Check if the region is empty.
    pub const fn is_empty(&self) -> bool {
        self.start_row >= self.end_row || self.start_col >= self.end_col
    }

    /// Get the number of rows in the region.
    pub const fn row_count(&self) -> c_int {
        if self.end_row > self.start_row {
            self.end_row - self.start_row
        } else {
            0
        }
    }

    /// Get the number of columns in the region.
    pub const fn col_count(&self) -> c_int {
        if self.end_col > self.start_col {
            self.end_col - self.start_col
        } else {
            0
        }
    }

    /// Expand this region to include another region.
    pub fn expand(&mut self, other: &DamageRegion) {
        if other.is_empty() {
            return;
        }
        if self.is_empty() {
            *self = *other;
            return;
        }
        if other.start_row < self.start_row {
            self.start_row = other.start_row;
        }
        if other.end_row > self.end_row {
            self.end_row = other.end_row;
        }
        if other.start_col < self.start_col {
            self.start_col = other.start_col;
        }
        if other.end_col > self.end_col {
            self.end_col = other.end_col;
        }
    }

    /// Check if this region intersects another.
    pub const fn intersects(&self, other: &DamageRegion) -> bool {
        !(self.end_row <= other.start_row
            || self.start_row >= other.end_row
            || self.end_col <= other.start_col
            || self.start_col >= other.end_col)
    }
}

// =============================================================================
// Title/Icon Types
// =============================================================================

/// Maximum title length.
pub const MAX_TITLE_LEN: usize = 256;

/// Title update result.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitleUpdateResult {
    /// Title updated successfully.
    Updated = 0,
    /// Title unchanged (same as previous).
    Unchanged = 1,
    /// Title truncated (too long).
    Truncated = 2,
    /// Invalid title (null pointer).
    Invalid = 3,
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Create a damage event.
#[no_mangle]
pub extern "C" fn rs_terminal_damage_event(
    start_row: c_int,
    end_row: c_int,
    start_col: c_int,
    end_col: c_int,
) -> OutputEvent {
    OutputEvent::damage(start_row, end_row, start_col, end_col)
}

/// FFI export: Create a bell event.
#[no_mangle]
pub extern "C" fn rs_terminal_bell_event() -> OutputEvent {
    OutputEvent::bell()
}

/// FFI export: Check if damage region is empty.
#[no_mangle]
pub extern "C" fn rs_terminal_damage_is_empty(region: DamageRegion) -> c_int {
    c_int::from(region.is_empty())
}

/// FFI export: Get damage row count.
#[no_mangle]
pub extern "C" fn rs_terminal_damage_row_count(region: DamageRegion) -> c_int {
    region.row_count()
}

/// FFI export: Check if regions intersect.
#[no_mangle]
pub extern "C" fn rs_terminal_damage_intersects(a: DamageRegion, b: DamageRegion) -> c_int {
    c_int::from(a.intersects(&b))
}

/// FFI export: Get max title length.
#[no_mangle]
pub extern "C" fn rs_terminal_max_title_len() -> usize {
    MAX_TITLE_LEN
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_event_creation() {
        let damage = OutputEvent::damage(0, 10, 0, 80);
        assert_eq!(damage.event_type, OutputEventType::Damage);
        assert_eq!(damage.start_row, 0);
        assert_eq!(damage.end_row, 10);

        let bell = OutputEvent::bell();
        assert_eq!(bell.event_type, OutputEventType::Bell);

        let cursor = OutputEvent::cursor_move(5, 10);
        assert_eq!(cursor.event_type, OutputEventType::CursorMove);
        assert_eq!(cursor.start_row, 5);
        assert_eq!(cursor.start_col, 10);

        let resize = OutputEvent::resize(24, 80);
        assert_eq!(resize.event_type, OutputEventType::Resize);
        assert_eq!(resize.end_row, 24);
        assert_eq!(resize.end_col, 80);
    }

    #[test]
    fn test_bell_flags() {
        let none = BellFlags::none();
        assert!(!none.visual());
        assert!(!none.audio());
        assert!(!none.urgent());

        let all = BellFlags::all();
        assert!(all.visual());
        assert!(all.audio());
        assert!(all.urgent());

        let visual_only = BellFlags::none().with_visual();
        assert!(visual_only.visual());
        assert!(!visual_only.audio());
    }

    #[test]
    fn test_damage_region() {
        let empty = DamageRegion::empty();
        assert!(empty.is_empty());
        assert_eq!(empty.row_count(), 0);

        let region = DamageRegion::new(0, 10, 0, 80);
        assert!(!region.is_empty());
        assert_eq!(region.row_count(), 10);
        assert_eq!(region.col_count(), 80);
    }

    #[test]
    fn test_damage_region_expand() {
        let mut region = DamageRegion::new(5, 10, 20, 40);

        let other = DamageRegion::new(2, 15, 10, 50);
        region.expand(&other);

        assert_eq!(region.start_row, 2);
        assert_eq!(region.end_row, 15);
        assert_eq!(region.start_col, 10);
        assert_eq!(region.end_col, 50);
    }

    #[test]
    fn test_damage_region_expand_from_empty() {
        let mut region = DamageRegion::empty();
        let other = DamageRegion::new(0, 10, 0, 80);
        region.expand(&other);

        assert_eq!(region.start_row, 0);
        assert_eq!(region.end_row, 10);
    }

    #[test]
    fn test_damage_region_intersects() {
        let region1 = DamageRegion::new(0, 10, 0, 80);
        let region2 = DamageRegion::new(5, 15, 40, 120);
        assert!(region1.intersects(&region2));

        let region3 = DamageRegion::new(20, 30, 0, 80);
        assert!(!region1.intersects(&region3));
    }

    #[test]
    fn test_output_event_type_values() {
        assert_eq!(OutputEventType::Damage as c_int, 0);
        assert_eq!(OutputEventType::MoveRect as c_int, 1);
        assert_eq!(OutputEventType::CursorMove as c_int, 2);
        assert_eq!(OutputEventType::PropChange as c_int, 3);
        assert_eq!(OutputEventType::Bell as c_int, 4);
        assert_eq!(OutputEventType::Resize as c_int, 5);
        assert_eq!(OutputEventType::ScrollbackPush as c_int, 6);
        assert_eq!(OutputEventType::ScrollbackPop as c_int, 7);
    }

    #[test]
    fn test_title_update_result_values() {
        assert_eq!(TitleUpdateResult::Updated as c_int, 0);
        assert_eq!(TitleUpdateResult::Unchanged as c_int, 1);
        assert_eq!(TitleUpdateResult::Truncated as c_int, 2);
        assert_eq!(TitleUpdateResult::Invalid as c_int, 3);
    }

    #[test]
    fn test_struct_sizes() {
        use std::mem::size_of;
        // BellFlags: 1 * 4 = 4 bytes
        assert_eq!(size_of::<BellFlags>(), 4);
        // DamageRegion: 4 * 4 = 16 bytes
        assert_eq!(size_of::<DamageRegion>(), 16);
        // OutputEvent: should be reasonable
        assert!(size_of::<OutputEvent>() <= 32);
    }

    #[test]
    fn test_damage_full_width() {
        let region = DamageRegion::full_width(5, 10, 80);
        assert_eq!(region.start_row, 5);
        assert_eq!(region.end_row, 10);
        assert_eq!(region.start_col, 0);
        assert_eq!(region.end_col, 80);
        assert_eq!(region.row_count(), 5);
        assert_eq!(region.col_count(), 80);
    }
}
