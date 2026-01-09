//! Line manipulation command implementations.
//!
//! This module provides implementations for Ex commands that manipulate lines:
//! - `:copy` (`:t`) - Copy lines to another location
//! - `:move` (`:m`) - Move lines to another location
//! - `:delete` (`:d`) - Delete lines
//! - `:yank` (`:y`) - Yank lines to a register
//! - `:put` (`:pu`) - Put register contents
//! - `:join` (`:j`) - Join lines together
//!
//! ## Implementation Notes
//!
//! These commands work with line ranges and optionally registers.
//! The actual buffer modifications are performed by Neovim's core functions.

use std::ffi::c_int;

use crate::range::{LineNr, LineRange};

/// Type of line manipulation operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LineOp {
    /// Copy lines (`:t`, `:copy`)
    Copy,
    /// Move lines (`:m`, `:move`)
    Move,
    /// Delete lines (`:d`, `:delete`)
    Delete,
    /// Yank lines to register (`:y`, `:yank`)
    Yank,
    /// Put from register (`:pu`, `:put`)
    Put,
    /// Join lines (`:j`, `:join`)
    Join,
}

impl LineOp {
    /// Check if this operation modifies the buffer.
    #[inline]
    #[must_use]
    pub const fn modifies_buffer(&self) -> bool {
        matches!(
            self,
            LineOp::Copy | LineOp::Move | LineOp::Delete | LineOp::Put | LineOp::Join
        )
    }

    /// Check if this operation uses a register.
    #[inline]
    #[must_use]
    pub const fn uses_register(&self) -> bool {
        matches!(self, LineOp::Yank | LineOp::Put | LineOp::Delete)
    }

    /// Check if this operation requires a destination.
    #[inline]
    #[must_use]
    pub const fn requires_destination(&self) -> bool {
        matches!(self, LineOp::Copy | LineOp::Move)
    }
}

/// Options for copy and move operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CopyMoveOptions {
    /// Source range of lines.
    pub range: LineRange,
    /// Destination line (insert after this line).
    pub dest: LineNr,
}

impl CopyMoveOptions {
    /// Create options for copying/moving a range to a destination.
    #[must_use]
    pub const fn new(range: LineRange, dest: LineNr) -> Self {
        Self { range, dest }
    }

    /// Check if the destination is valid for copy/move.
    ///
    /// For copy: destination can be anywhere.
    /// For move: destination must not be within the source range.
    #[must_use]
    pub fn is_valid_for_move(&self) -> bool {
        // Destination cannot be within the source range
        self.dest < self.range.start || self.dest > self.range.end
    }

    /// Get the number of lines in the source range.
    #[inline]
    #[must_use]
    pub fn line_count(&self) -> LineNr {
        self.range.len()
    }
}

/// Options for delete operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DeleteOptions {
    /// Range of lines to delete.
    pub range: LineRange,
    /// Register to store deleted text (0 for default register).
    pub register: u8,
    /// Whether to save to register.
    pub use_register: bool,
}

impl DeleteOptions {
    /// Create options for deleting a range.
    #[must_use]
    pub const fn new(range: LineRange) -> Self {
        Self {
            range,
            register: 0,
            use_register: true,
        }
    }

    /// Create options for deleting without saving to register.
    #[must_use]
    pub const fn without_register(range: LineRange) -> Self {
        Self {
            range,
            register: 0,
            use_register: false,
        }
    }

    /// Set the register to use.
    #[must_use]
    pub const fn with_register(mut self, register: u8) -> Self {
        self.register = register;
        self.use_register = true;
        self
    }
}

/// Options for yank operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct YankOptions {
    /// Range of lines to yank.
    pub range: LineRange,
    /// Register to yank to (0 for default register).
    pub register: u8,
}

impl YankOptions {
    /// Create options for yanking a range to the default register.
    #[must_use]
    pub const fn new(range: LineRange) -> Self {
        Self { range, register: 0 }
    }

    /// Create options for yanking to a specific register.
    #[must_use]
    pub const fn to_register(range: LineRange, register: u8) -> Self {
        Self { range, register }
    }
}

/// Options for put operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PutOptions {
    /// Line to put after (0 = before first line).
    pub line: LineNr,
    /// Register to put from (0 for default register).
    pub register: u8,
    /// Put before the line instead of after.
    pub before: bool,
}

impl PutOptions {
    /// Create options for putting after a line.
    #[must_use]
    pub const fn after(line: LineNr) -> Self {
        Self {
            line,
            register: 0,
            before: false,
        }
    }

    /// Create options for putting before a line.
    #[must_use]
    pub const fn before(line: LineNr) -> Self {
        Self {
            line,
            register: 0,
            before: true,
        }
    }

    /// Set the register to put from.
    #[must_use]
    pub const fn from_register(mut self, register: u8) -> Self {
        self.register = register;
        self
    }
}

/// Options for join operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct JoinOptions {
    /// Range of lines to join.
    pub range: LineRange,
    /// Don't insert spaces when joining.
    pub no_space: bool,
}

impl JoinOptions {
    /// Create options for joining a range with spaces.
    #[must_use]
    pub const fn new(range: LineRange) -> Self {
        Self {
            range,
            no_space: false,
        }
    }

    /// Create options for joining without spaces (gJ style).
    #[must_use]
    pub const fn without_space(range: LineRange) -> Self {
        Self {
            range,
            no_space: true,
        }
    }
}

/// Result of a line manipulation operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LineOpResult {
    /// Number of lines affected.
    pub lines: i32,
    /// Whether the buffer was changed.
    pub changed: bool,
}

impl LineOpResult {
    /// Create a new result.
    #[must_use]
    pub const fn new(lines: i32, changed: bool) -> Self {
        Self { lines, changed }
    }

    /// Create a result indicating no change.
    #[must_use]
    pub const fn no_change() -> Self {
        Self {
            lines: 0,
            changed: false,
        }
    }
}

/// Error type for line manipulation operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineError {
    /// Invalid range.
    InvalidRange,
    /// Invalid destination.
    InvalidDestination(LineNr),
    /// Cannot move into own range.
    MoveIntoSelf,
    /// Invalid register.
    InvalidRegister(u8),
    /// Empty register.
    EmptyRegister(u8),
    /// Buffer is readonly.
    Readonly,
}

impl std::fmt::Display for LineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LineError::InvalidRange => write!(f, "invalid range"),
            LineError::InvalidDestination(dest) => write!(f, "invalid destination: {dest}"),
            LineError::MoveIntoSelf => write!(f, "cannot move to same range"),
            LineError::InvalidRegister(r) => write!(f, "invalid register: {}", *r as char),
            LineError::EmptyRegister(r) => write!(f, "register is empty: {}", *r as char),
            LineError::Readonly => write!(f, "buffer is readonly"),
        }
    }
}

impl std::error::Error for LineError {}

/// Validate a copy/move operation.
///
/// # Arguments
/// * `range` - Source range
/// * `dest` - Destination line
/// * `line_count` - Total lines in buffer
/// * `is_move` - True if this is a move operation
///
/// # Returns
/// `Ok(())` if valid, `Err` with the error otherwise.
pub fn validate_copy_move(
    range: LineRange,
    dest: LineNr,
    line_count: LineNr,
    is_move: bool,
) -> Result<(), LineError> {
    // Validate range
    if range.is_empty() {
        return Err(LineError::InvalidRange);
    }
    if range.start < 1 || range.end > line_count {
        return Err(LineError::InvalidRange);
    }

    // Validate destination
    if dest < 0 || dest > line_count {
        return Err(LineError::InvalidDestination(dest));
    }

    // For move, destination can't be within the source range
    if is_move && dest >= range.start && dest < range.end {
        return Err(LineError::MoveIntoSelf);
    }

    Ok(())
}

/// Calculate the new line number after a move operation.
///
/// When lines are moved, line numbers can shift. This function calculates
/// where a given line number ends up after moving a range.
///
/// # Arguments
/// * `lnum` - Line number to track
/// * `range` - Range being moved
/// * `dest` - Destination line
///
/// # Returns
/// The new line number after the move.
#[must_use]
pub fn adjust_line_after_move(lnum: LineNr, range: LineRange, dest: LineNr) -> LineNr {
    let count = range.len();

    if dest < range.start {
        // Moving up
        if lnum >= range.start && lnum <= range.end {
            // Line is in the moved range
            lnum - range.start + dest + 1
        } else if lnum > dest && lnum < range.start {
            // Line is between destination and source - shifts down
            lnum + count
        } else {
            // Line is outside affected area
            lnum
        }
    } else {
        // Moving down (dest > range.end)
        if lnum >= range.start && lnum <= range.end {
            // Line is in the moved range
            lnum + (dest - range.end)
        } else if lnum > range.end && lnum <= dest {
            // Line is between source and destination - shifts up
            lnum - count
        } else {
            // Line is outside affected area
            lnum
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Validate a copy operation.
///
/// Returns 1 if valid, 0 if invalid.
#[no_mangle]
pub extern "C" fn rs_validate_copy(
    start: c_int,
    end: c_int,
    dest: c_int,
    line_count: c_int,
) -> c_int {
    let range = LineRange::new(start, end);
    c_int::from(validate_copy_move(range, dest, line_count, false).is_ok())
}

/// Validate a move operation.
///
/// Returns 1 if valid, 0 if invalid.
#[no_mangle]
pub extern "C" fn rs_validate_move(
    start: c_int,
    end: c_int,
    dest: c_int,
    line_count: c_int,
) -> c_int {
    let range = LineRange::new(start, end);
    c_int::from(validate_copy_move(range, dest, line_count, true).is_ok())
}

/// Adjust a line number after a move operation.
#[no_mangle]
pub extern "C" fn rs_adjust_line_after_move(
    lnum: c_int,
    start: c_int,
    end: c_int,
    dest: c_int,
) -> c_int {
    let range = LineRange::new(start, end);
    adjust_line_after_move(lnum, range, dest)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_op() {
        assert!(LineOp::Copy.modifies_buffer());
        assert!(LineOp::Move.modifies_buffer());
        assert!(LineOp::Delete.modifies_buffer());
        assert!(!LineOp::Yank.modifies_buffer());
        assert!(LineOp::Put.modifies_buffer());
        assert!(LineOp::Join.modifies_buffer());

        assert!(LineOp::Yank.uses_register());
        assert!(LineOp::Put.uses_register());
        assert!(LineOp::Delete.uses_register());
        assert!(!LineOp::Copy.uses_register());

        assert!(LineOp::Copy.requires_destination());
        assert!(LineOp::Move.requires_destination());
        assert!(!LineOp::Delete.requires_destination());
    }

    #[test]
    fn test_copy_move_options() {
        let range = LineRange::new(5, 10);
        let opts = CopyMoveOptions::new(range, 20);
        assert_eq!(opts.range, range);
        assert_eq!(opts.dest, 20);
        assert_eq!(opts.line_count(), 6);

        // Valid for move (dest outside range)
        assert!(opts.is_valid_for_move());

        // Invalid for move (dest inside range)
        let opts = CopyMoveOptions::new(range, 7);
        assert!(!opts.is_valid_for_move());
    }

    #[test]
    fn test_delete_options() {
        let range = LineRange::new(5, 10);
        let opts = DeleteOptions::new(range);
        assert!(opts.use_register);
        assert_eq!(opts.register, 0);

        let opts = DeleteOptions::without_register(range);
        assert!(!opts.use_register);

        let opts = DeleteOptions::new(range).with_register(b'a');
        assert!(opts.use_register);
        assert_eq!(opts.register, b'a');
    }

    #[test]
    fn test_yank_options() {
        let range = LineRange::new(5, 10);
        let opts = YankOptions::new(range);
        assert_eq!(opts.register, 0);

        let opts = YankOptions::to_register(range, b'a');
        assert_eq!(opts.register, b'a');
    }

    #[test]
    fn test_put_options() {
        let opts = PutOptions::after(10);
        assert_eq!(opts.line, 10);
        assert!(!opts.before);

        let opts = PutOptions::before(10);
        assert!(opts.before);

        let opts = PutOptions::after(10).from_register(b'a');
        assert_eq!(opts.register, b'a');
    }

    #[test]
    fn test_join_options() {
        let range = LineRange::new(5, 10);
        let opts = JoinOptions::new(range);
        assert!(!opts.no_space);

        let opts = JoinOptions::without_space(range);
        assert!(opts.no_space);
    }

    #[test]
    fn test_line_op_result() {
        let result = LineOpResult::new(5, true);
        assert_eq!(result.lines, 5);
        assert!(result.changed);

        let result = LineOpResult::no_change();
        assert_eq!(result.lines, 0);
        assert!(!result.changed);
    }

    #[test]
    fn test_line_error_display() {
        let err = LineError::InvalidRange;
        assert_eq!(format!("{err}"), "invalid range");

        let err = LineError::InvalidDestination(150);
        assert_eq!(format!("{err}"), "invalid destination: 150");

        let err = LineError::MoveIntoSelf;
        assert!(format!("{err}").contains("same range"));

        let err = LineError::InvalidRegister(b'@');
        assert!(format!("{err}").contains("@"));
    }

    #[test]
    fn test_validate_copy_move() {
        // Valid copy
        let result = validate_copy_move(LineRange::new(5, 10), 20, 100, false);
        assert!(result.is_ok());

        // Valid copy to position 0 (before first line)
        let result = validate_copy_move(LineRange::new(5, 10), 0, 100, false);
        assert!(result.is_ok());

        // Invalid range
        let result = validate_copy_move(LineRange::empty(), 20, 100, false);
        assert!(matches!(result, Err(LineError::InvalidRange)));

        // Invalid destination
        let result = validate_copy_move(LineRange::new(5, 10), 150, 100, false);
        assert!(matches!(result, Err(LineError::InvalidDestination(150))));

        // Move into self
        let result = validate_copy_move(LineRange::new(5, 10), 7, 100, true);
        assert!(matches!(result, Err(LineError::MoveIntoSelf)));

        // Copy to same position is allowed
        let result = validate_copy_move(LineRange::new(5, 10), 7, 100, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_adjust_line_after_move() {
        // Move lines 5-10 to after line 20
        let range = LineRange::new(5, 10);
        let dest = 20;

        // Line within range - moves to new position
        assert_eq!(adjust_line_after_move(5, range, dest), 15);
        assert_eq!(adjust_line_after_move(10, range, dest), 20);

        // Line between source and dest - shifts up
        assert_eq!(adjust_line_after_move(15, range, dest), 9);

        // Line after dest - unchanged
        assert_eq!(adjust_line_after_move(25, range, dest), 25);

        // Line before source - unchanged
        assert_eq!(adjust_line_after_move(3, range, dest), 3);
    }

    #[test]
    fn test_adjust_line_after_move_up() {
        // Move lines 15-20 to after line 5
        let range = LineRange::new(15, 20);
        let dest = 5;

        // Line within range - moves to new position
        assert_eq!(adjust_line_after_move(15, range, dest), 6);
        assert_eq!(adjust_line_after_move(20, range, dest), 11);

        // Line between dest and source - shifts down
        assert_eq!(adjust_line_after_move(10, range, dest), 16);

        // Line before dest - unchanged
        assert_eq!(adjust_line_after_move(3, range, dest), 3);

        // Line after source - unchanged
        assert_eq!(adjust_line_after_move(25, range, dest), 25);
    }

    #[test]
    fn test_rs_validate_copy() {
        assert_eq!(rs_validate_copy(5, 10, 20, 100), 1);
        assert_eq!(rs_validate_copy(5, 10, 150, 100), 0);
    }

    #[test]
    fn test_rs_validate_move() {
        assert_eq!(rs_validate_move(5, 10, 20, 100), 1);
        assert_eq!(rs_validate_move(5, 10, 7, 100), 0); // Into self
    }

    #[test]
    fn test_rs_adjust_line_after_move() {
        // Move 5-10 to after 20
        assert_eq!(rs_adjust_line_after_move(5, 5, 10, 20), 15);
        assert_eq!(rs_adjust_line_after_move(15, 5, 10, 20), 9);
    }
}
