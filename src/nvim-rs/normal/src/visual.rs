//! Visual mode state and update helpers
//!
//! This module provides helpers for managing visual mode state,
//! including selection tracking, mode transitions, and visual updates.

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::match_same_arms)]

use std::ffi::c_int;

// =============================================================================
// Visual Mode Types
// =============================================================================

/// Visual mode types.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VisualMode {
    /// Not in visual mode
    #[default]
    None = 0,
    /// Character-wise visual mode (v)
    Char = 1,
    /// Line-wise visual mode (V)
    Line = 2,
    /// Block-wise visual mode (CTRL-V)
    Block = 3,
}

impl VisualMode {
    /// Convert from raw integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Self {
        match value {
            1 => Self::Char,
            2 => Self::Line,
            3 => Self::Block,
            _ => Self::None,
        }
    }

    /// Convert to raw integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Get the visual mode character.
    #[must_use]
    pub const fn char(&self) -> u8 {
        match self {
            Self::None => 0,
            Self::Char => b'v',
            Self::Line => b'V',
            Self::Block => 22, // CTRL-V
        }
    }

    /// Check if in any visual mode.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        !matches!(self, Self::None)
    }

    /// Check if this is line-wise.
    #[must_use]
    pub const fn is_linewise(&self) -> bool {
        matches!(self, Self::Line)
    }

    /// Check if this is block-wise.
    #[must_use]
    pub const fn is_blockwise(&self) -> bool {
        matches!(self, Self::Block)
    }

    /// Get the next mode when cycling (v -> V -> CTRL-V -> v).
    #[must_use]
    pub const fn cycle_next(&self) -> Self {
        match self {
            Self::None => Self::Char,
            Self::Char => Self::Line,
            Self::Line => Self::Block,
            Self::Block => Self::Char,
        }
    }
}

// =============================================================================
// Visual Selection State
// =============================================================================

/// Position in the buffer (1-based line and column).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct VisualPos {
    /// Line number (1-based)
    pub lnum: c_int,
    /// Column number (0-based byte offset)
    pub col: c_int,
    /// Column offset for virtual columns
    pub coladd: c_int,
}

impl VisualPos {
    /// Create a new visual position.
    #[must_use]
    pub const fn new(lnum: c_int, col: c_int) -> Self {
        Self {
            lnum,
            col,
            coladd: 0,
        }
    }

    /// Create a position with virtual column offset.
    #[must_use]
    pub const fn with_coladd(lnum: c_int, col: c_int, coladd: c_int) -> Self {
        Self { lnum, col, coladd }
    }

    /// Check if position is valid.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.lnum > 0 && self.col >= 0
    }

    /// Compare positions (returns -1, 0, or 1).
    #[must_use]
    pub const fn compare(&self, other: &Self) -> c_int {
        if self.lnum < other.lnum {
            -1
        } else if self.lnum > other.lnum {
            1
        } else if self.col < other.col {
            -1
        } else if self.col > other.col {
            1
        } else if self.coladd < other.coladd {
            -1
        } else if self.coladd > other.coladd {
            1
        } else {
            0
        }
    }

    /// Check if this position is before another.
    #[must_use]
    pub const fn is_before(&self, other: &Self) -> bool {
        self.compare(other) < 0
    }

    /// Check if this position is after another.
    #[must_use]
    pub const fn is_after(&self, other: &Self) -> bool {
        self.compare(other) > 0
    }
}

/// Complete visual selection state.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct VisualState {
    /// Visual mode type
    pub mode: c_int,
    /// Start position of selection
    pub start: VisualPos,
    /// End position (cursor)
    pub end: VisualPos,
    /// Whether selection is reselected
    pub reselect: bool,
    /// Column for block mode start
    pub start_vcol: c_int,
    /// Column for block mode end
    pub end_vcol: c_int,
}

impl VisualState {
    /// Create a new visual state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            mode: 0,
            start: VisualPos::new(0, 0),
            end: VisualPos::new(0, 0),
            reselect: false,
            start_vcol: 0,
            end_vcol: 0,
        }
    }

    /// Get the visual mode.
    #[must_use]
    pub const fn get_mode(&self) -> VisualMode {
        VisualMode::from_raw(self.mode)
    }

    /// Check if in visual mode.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.mode != 0
    }

    /// Start visual mode at position.
    pub fn start_at(&mut self, mode: VisualMode, pos: VisualPos) {
        self.mode = mode.to_raw();
        self.start = pos;
        self.end = pos;
        self.reselect = false;
    }

    /// Update end position.
    pub fn update_end(&mut self, pos: VisualPos) {
        self.end = pos;
    }

    /// Get the first position (top-left).
    #[must_use]
    pub const fn first(&self) -> VisualPos {
        if self.start.is_before(&self.end) {
            self.start
        } else {
            self.end
        }
    }

    /// Get the last position (bottom-right).
    #[must_use]
    pub const fn last(&self) -> VisualPos {
        if self.start.is_after(&self.end) {
            self.start
        } else {
            self.end
        }
    }

    /// Get number of lines in selection.
    #[must_use]
    pub const fn line_count(&self) -> c_int {
        let first = self.first();
        let last = self.last();
        if first.lnum > 0 && last.lnum >= first.lnum {
            last.lnum - first.lnum + 1
        } else {
            0
        }
    }

    /// Check if selection spans multiple lines.
    #[must_use]
    pub const fn is_multiline(&self) -> bool {
        self.start.lnum != self.end.lnum
    }

    /// Clear visual state.
    pub fn clear(&mut self) {
        *self = Self::new();
    }

    /// Switch visual mode.
    pub fn switch_mode(&mut self, mode: VisualMode) {
        if self.mode != 0 {
            self.mode = mode.to_raw();
        }
    }
}

// =============================================================================
// Visual Update Flags
// =============================================================================

/// Flags for visual mode updates.
pub mod visual_flags {
    use std::ffi::c_int;

    /// Update required for redraw
    pub const VIS_REDRAW: c_int = 0x01;
    /// Selection changed
    pub const VIS_CHANGED: c_int = 0x02;
    /// Mode changed
    pub const VIS_MODE_CHANGE: c_int = 0x04;
    /// Extended selection (moved forward)
    pub const VIS_EXTENDED: c_int = 0x08;
    /// Shrunk selection (moved backward)
    pub const VIS_SHRUNK: c_int = 0x10;
    /// Selection was swapped (o command)
    pub const VIS_SWAPPED: c_int = 0x20;
}

/// Check if visual flags have a specific flag set.
#[must_use]
#[inline]
pub const fn has_visual_flag(flags: c_int, flag: c_int) -> bool {
    (flags & flag) != 0
}

// =============================================================================
// Visual Update Result
// =============================================================================

/// Result of a visual selection update.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct VisualUpdate {
    /// Update flags
    pub flags: c_int,
    /// First line needing redraw
    pub redraw_first: c_int,
    /// Last line needing redraw
    pub redraw_last: c_int,
}

impl VisualUpdate {
    /// Create a no-change update.
    #[must_use]
    pub const fn no_change() -> Self {
        Self {
            flags: 0,
            redraw_first: 0,
            redraw_last: 0,
        }
    }

    /// Create an update for selection change.
    #[must_use]
    pub const fn selection_changed(first: c_int, last: c_int) -> Self {
        Self {
            flags: visual_flags::VIS_CHANGED | visual_flags::VIS_REDRAW,
            redraw_first: first,
            redraw_last: last,
        }
    }

    /// Create an update for mode change.
    #[must_use]
    pub const fn mode_changed() -> Self {
        Self {
            flags: visual_flags::VIS_MODE_CHANGE | visual_flags::VIS_REDRAW,
            redraw_first: 0,
            redraw_last: 0,
        }
    }

    /// Check if redraw is needed.
    #[must_use]
    pub const fn needs_redraw(&self) -> bool {
        has_visual_flag(self.flags, visual_flags::VIS_REDRAW)
    }

    /// Check if selection changed.
    #[must_use]
    pub const fn selection_changed_flag(&self) -> bool {
        has_visual_flag(self.flags, visual_flags::VIS_CHANGED)
    }
}

// =============================================================================
// Select Mode
// =============================================================================

/// Select mode state (like visual but typing replaces).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SelectState {
    /// Whether select mode is active
    pub active: bool,
    /// The underlying visual mode
    pub visual_mode: c_int,
    /// Whether to keep selection after operation
    pub keep_selection: bool,
}

impl SelectState {
    /// Create a new select state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            active: false,
            visual_mode: 0,
            keep_selection: false,
        }
    }

    /// Enter select mode.
    pub fn enter(&mut self, mode: VisualMode) {
        self.active = true;
        self.visual_mode = mode.to_raw();
    }

    /// Exit select mode.
    pub fn exit(&mut self) {
        self.active = false;
        self.visual_mode = 0;
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Get visual mode from raw value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_visual_mode(value: c_int) -> c_int {
    VisualMode::from_raw(value).to_raw()
}

/// Check if visual mode is active.
#[unsafe(no_mangle)]
pub extern "C" fn rs_visual_is_active(value: c_int) -> c_int {
    c_int::from(VisualMode::from_raw(value).is_active())
}

/// Check if visual mode is line-wise.
#[unsafe(no_mangle)]
pub extern "C" fn rs_visual_is_linewise(value: c_int) -> c_int {
    c_int::from(VisualMode::from_raw(value).is_linewise())
}

/// Check if visual mode is block-wise.
#[unsafe(no_mangle)]
pub extern "C" fn rs_visual_is_blockwise(value: c_int) -> c_int {
    c_int::from(VisualMode::from_raw(value).is_blockwise())
}

/// Get next visual mode in cycle.
#[unsafe(no_mangle)]
pub extern "C" fn rs_visual_cycle_next(value: c_int) -> c_int {
    VisualMode::from_raw(value).cycle_next().to_raw()
}

/// Compare two positions.
#[unsafe(no_mangle)]
pub extern "C" fn rs_visual_pos_compare(
    lnum1: c_int,
    col1: c_int,
    lnum2: c_int,
    col2: c_int,
) -> c_int {
    let pos1 = VisualPos::new(lnum1, col1);
    let pos2 = VisualPos::new(lnum2, col2);
    pos1.compare(&pos2)
}

/// Check if visual flags have a specific flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_has_visual_flag(flags: c_int, flag: c_int) -> c_int {
    c_int::from(has_visual_flag(flags, flag))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visual_mode() {
        assert_eq!(VisualMode::from_raw(0), VisualMode::None);
        assert_eq!(VisualMode::from_raw(1), VisualMode::Char);
        assert_eq!(VisualMode::from_raw(2), VisualMode::Line);
        assert_eq!(VisualMode::from_raw(3), VisualMode::Block);

        assert!(!VisualMode::None.is_active());
        assert!(VisualMode::Char.is_active());
        assert!(VisualMode::Line.is_linewise());
        assert!(VisualMode::Block.is_blockwise());
    }

    #[test]
    fn test_visual_mode_cycle() {
        assert_eq!(VisualMode::Char.cycle_next(), VisualMode::Line);
        assert_eq!(VisualMode::Line.cycle_next(), VisualMode::Block);
        assert_eq!(VisualMode::Block.cycle_next(), VisualMode::Char);
        assert_eq!(VisualMode::None.cycle_next(), VisualMode::Char);
    }

    #[test]
    fn test_visual_pos() {
        let pos1 = VisualPos::new(5, 10);
        let pos2 = VisualPos::new(5, 15);
        let pos3 = VisualPos::new(10, 5);

        assert!(pos1.is_valid());
        assert!(pos1.is_before(&pos2));
        assert!(pos2.is_before(&pos3));
        assert!(pos3.is_after(&pos1));

        assert_eq!(pos1.compare(&pos1), 0);
        assert_eq!(pos1.compare(&pos2), -1);
        assert_eq!(pos2.compare(&pos1), 1);
    }

    #[test]
    fn test_visual_state() {
        let mut state = VisualState::new();
        assert!(!state.is_active());

        let start_pos = VisualPos::new(5, 0);
        state.start_at(VisualMode::Char, start_pos);
        assert!(state.is_active());
        assert_eq!(state.get_mode(), VisualMode::Char);

        let end_pos = VisualPos::new(10, 5);
        state.update_end(end_pos);
        assert!(state.is_multiline());
        assert_eq!(state.line_count(), 6);

        assert_eq!(state.first().lnum, 5);
        assert_eq!(state.last().lnum, 10);

        state.switch_mode(VisualMode::Line);
        assert_eq!(state.get_mode(), VisualMode::Line);

        state.clear();
        assert!(!state.is_active());
    }

    #[test]
    fn test_visual_flags() {
        let flags = visual_flags::VIS_CHANGED | visual_flags::VIS_REDRAW;
        assert!(has_visual_flag(flags, visual_flags::VIS_CHANGED));
        assert!(has_visual_flag(flags, visual_flags::VIS_REDRAW));
        assert!(!has_visual_flag(flags, visual_flags::VIS_EXTENDED));
    }

    #[test]
    fn test_visual_update() {
        let update = VisualUpdate::selection_changed(5, 10);
        assert!(update.needs_redraw());
        assert!(update.selection_changed_flag());
        assert_eq!(update.redraw_first, 5);
        assert_eq!(update.redraw_last, 10);

        let no_change = VisualUpdate::no_change();
        assert!(!no_change.needs_redraw());
    }

    #[test]
    fn test_select_state() {
        let mut state = SelectState::new();
        assert!(!state.active);

        state.enter(VisualMode::Char);
        assert!(state.active);
        assert_eq!(VisualMode::from_raw(state.visual_mode), VisualMode::Char);

        state.exit();
        assert!(!state.active);
    }
}
