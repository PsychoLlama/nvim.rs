//! Cursor state management
//!
//! This module provides cursor state infrastructure for Neovim,
//! including position tracking, movement, and selection.

use std::ffi::c_int;

// =============================================================================
// Cursor Position
// =============================================================================

/// Cursor position in a buffer.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CursorPos {
    /// Line number (1-based)
    pub line: i64,
    /// Column number (0-based byte offset)
    pub col: c_int,
    /// Column offset for virtual columns
    pub coladd: c_int,
}

impl CursorPos {
    /// Create new cursor position.
    #[must_use]
    pub const fn new(line: i64, col: c_int) -> Self {
        Self {
            line,
            col,
            coladd: 0,
        }
    }

    /// Create position with virtual column.
    #[must_use]
    pub const fn with_coladd(line: i64, col: c_int, coladd: c_int) -> Self {
        Self { line, col, coladd }
    }

    /// Check if position is valid.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.line > 0 && self.col >= 0
    }

    /// Check if position is at start of line.
    #[must_use]
    pub const fn at_line_start(&self) -> bool {
        self.col == 0 && self.coladd == 0
    }

    /// Compare positions.
    #[must_use]
    pub const fn cmp(&self, other: &Self) -> i32 {
        if self.line < other.line {
            -1
        } else if self.line > other.line {
            1
        } else if self.col < other.col {
            -1
        } else if self.col > other.col {
            1
        } else {
            0
        }
    }

    /// Check if this position is before another.
    #[must_use]
    pub const fn is_before(&self, other: &Self) -> bool {
        self.cmp(other) < 0
    }

    /// Check if this position is after another.
    #[must_use]
    pub const fn is_after(&self, other: &Self) -> bool {
        self.cmp(other) > 0
    }
}

/// FFI: Create cursor position.
#[no_mangle]
pub extern "C" fn rs_state_cursor_pos_new(line: i64, col: c_int) -> CursorPos {
    CursorPos::new(line, col)
}

/// FFI: Check if position is valid.
///
/// # Safety
/// `pos` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_state_cursor_pos_is_valid(pos: *const CursorPos) -> c_int {
    if pos.is_null() {
        return 0;
    }
    c_int::from((*pos).is_valid())
}

/// FFI: Compare positions.
///
/// # Safety
/// Both `a` and `b` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_state_cursor_pos_cmp(
    a: *const CursorPos,
    b: *const CursorPos,
) -> c_int {
    if a.is_null() || b.is_null() {
        return 0;
    }
    (*a).cmp(&*b)
}

// =============================================================================
// Cursor State
// =============================================================================

/// Complete cursor state.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct CursorState {
    /// Current position
    pub pos: CursorPos,
    /// Wanted column (for up/down movement)
    pub wanted_col: c_int,
    /// Whether wanted column is at end of line
    pub wanted_col_eol: bool,
    /// Cursor is visible
    pub visible: bool,
    /// Cursor shape
    pub shape: c_int,
    /// Blink state
    pub blink_on: bool,
}

impl CursorState {
    /// Create new cursor state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            pos: CursorPos::new(1, 0),
            wanted_col: 0,
            wanted_col_eol: false,
            visible: true,
            shape: 0,
            blink_on: true,
        }
    }

    /// Move cursor to position.
    pub fn move_to(&mut self, line: i64, col: c_int) {
        self.pos = CursorPos::new(line, col);
    }

    /// Move cursor by delta.
    pub fn move_by(&mut self, line_delta: i64, col_delta: c_int) {
        self.pos.line += line_delta;
        self.pos.col += col_delta;
    }

    /// Update wanted column.
    pub fn update_wanted_col(&mut self) {
        self.wanted_col = self.pos.col;
        self.wanted_col_eol = false;
    }

    /// Set wanted column to end of line.
    pub fn set_wanted_col_eol(&mut self) {
        self.wanted_col_eol = true;
    }
}

/// FFI: Create cursor state.
#[no_mangle]
pub extern "C" fn rs_cursor_state_new() -> CursorState {
    CursorState::new()
}

/// FFI: Move cursor to position.
///
/// # Safety
/// `state` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_move_to(state: *mut CursorState, line: i64, col: c_int) {
    if !state.is_null() {
        (*state).move_to(line, col);
    }
}

/// FFI: Move cursor by delta.
///
/// # Safety
/// `state` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_cursor_move_by(
    state: *mut CursorState,
    line_delta: i64,
    col_delta: c_int,
) {
    if !state.is_null() {
        (*state).move_by(line_delta, col_delta);
    }
}

// =============================================================================
// Selection
// =============================================================================

/// Selection type.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectionType {
    /// No selection
    #[default]
    None = 0,
    /// Character-wise selection
    Char = 1,
    /// Line-wise selection
    Line = 2,
    /// Block selection
    Block = 3,
}

impl SelectionType {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Char,
            2 => Self::Line,
            3 => Self::Block,
            _ => Self::None,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

/// Selection state.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SelectionState {
    /// Selection type
    pub sel_type: c_int,
    /// Start position
    pub start: CursorPos,
    /// End position
    pub end: CursorPos,
    /// Selection is active
    pub active: bool,
    /// Selection is inclusive
    pub inclusive: bool,
}

impl SelectionState {
    /// Create new selection state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            sel_type: SelectionType::None as c_int,
            start: CursorPos::new(1, 0),
            end: CursorPos::new(1, 0),
            active: false,
            inclusive: false,
        }
    }

    /// Get selection type.
    #[must_use]
    pub const fn get_type(&self) -> SelectionType {
        SelectionType::from_c_int(self.sel_type)
    }

    /// Start a new selection.
    pub fn start_selection(&mut self, sel_type: SelectionType, pos: CursorPos) {
        self.sel_type = sel_type as c_int;
        self.start = pos;
        self.end = pos;
        self.active = true;
    }

    /// Extend selection to new position.
    pub fn extend_to(&mut self, pos: CursorPos) {
        self.end = pos;
    }

    /// Clear the selection.
    pub fn clear(&mut self) {
        self.sel_type = SelectionType::None as c_int;
        self.active = false;
    }

    /// Get the ordered start and end positions.
    #[must_use]
    pub const fn ordered(&self) -> (CursorPos, CursorPos) {
        if self.start.is_before(&self.end) {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        }
    }

    /// Check if a position is within the selection.
    #[must_use]
    pub const fn contains(&self, pos: &CursorPos) -> bool {
        if !self.active {
            return false;
        }

        let (start, end) = self.ordered();
        !pos.is_before(&start) && !pos.is_after(&end)
    }
}

/// FFI: Create selection state.
#[no_mangle]
pub extern "C" fn rs_selection_state_new() -> SelectionState {
    SelectionState::new()
}

/// FFI: Start selection.
///
/// # Safety
/// `state` and `pos` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_selection_start(
    state: *mut SelectionState,
    sel_type: c_int,
    pos: *const CursorPos,
) {
    if state.is_null() || pos.is_null() {
        return;
    }
    (*state).start_selection(SelectionType::from_c_int(sel_type), *pos);
}

/// FFI: Extend selection.
///
/// # Safety
/// `state` and `pos` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_selection_extend_to(state: *mut SelectionState, pos: *const CursorPos) {
    if state.is_null() || pos.is_null() {
        return;
    }
    (*state).extend_to(*pos);
}

/// FFI: Clear selection.
///
/// # Safety
/// `state` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_selection_clear(state: *mut SelectionState) {
    if !state.is_null() {
        (*state).clear();
    }
}

/// FFI: Check if position in selection.
///
/// # Safety
/// `state` and `pos` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_selection_contains(
    state: *const SelectionState,
    pos: *const CursorPos,
) -> c_int {
    if state.is_null() || pos.is_null() {
        return 0;
    }
    c_int::from((*state).contains(&*pos))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_pos() {
        let pos = CursorPos::new(1, 0);
        assert!(pos.is_valid());
        assert!(pos.at_line_start());

        let pos2 = CursorPos::new(2, 5);
        assert!(pos.is_before(&pos2));
        assert!(pos2.is_after(&pos));
    }

    #[test]
    fn test_cursor_state() {
        let mut state = CursorState::new();
        assert_eq!(state.pos.line, 1);
        assert_eq!(state.pos.col, 0);

        state.move_to(5, 10);
        assert_eq!(state.pos.line, 5);
        assert_eq!(state.pos.col, 10);

        state.move_by(-1, 2);
        assert_eq!(state.pos.line, 4);
        assert_eq!(state.pos.col, 12);
    }

    #[test]
    fn test_selection_state() {
        let mut sel = SelectionState::new();
        assert!(!sel.active);

        let start_pos = CursorPos::new(1, 0);
        sel.start_selection(SelectionType::Char, start_pos);
        assert!(sel.active);
        assert_eq!(sel.get_type(), SelectionType::Char);

        let end_pos = CursorPos::new(1, 10);
        sel.extend_to(end_pos);

        let mid_pos = CursorPos::new(1, 5);
        assert!(sel.contains(&mid_pos));

        let outside_pos = CursorPos::new(2, 0);
        assert!(!sel.contains(&outside_pos));

        sel.clear();
        assert!(!sel.active);
    }
}
