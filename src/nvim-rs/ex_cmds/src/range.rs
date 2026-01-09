//! Line range types and utilities for Ex commands.
//!
//! This module provides types for representing and manipulating line ranges
//! used by Ex commands. Line numbers are 1-based (as in Vim/Neovim).

use std::ffi::c_int;

/// A line number in a buffer.
///
/// Line numbers are 1-based. A value of 0 typically means "no line" or
/// "before the first line" depending on context.
pub type LineNr = c_int;

/// A range of lines in a buffer.
///
/// Both `start` and `end` are inclusive and 1-based.
/// An empty or invalid range has `start > end`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LineRange {
    /// First line of the range (1-based, inclusive).
    pub start: LineNr,
    /// Last line of the range (1-based, inclusive).
    pub end: LineNr,
}

impl LineRange {
    /// Create a new line range.
    ///
    /// # Arguments
    /// * `start` - First line (1-based, inclusive)
    /// * `end` - Last line (1-based, inclusive)
    #[inline]
    #[must_use]
    pub const fn new(start: LineNr, end: LineNr) -> Self {
        Self { start, end }
    }

    /// Create a range for a single line.
    #[inline]
    #[must_use]
    pub const fn single(line: LineNr) -> Self {
        Self {
            start: line,
            end: line,
        }
    }

    /// Create an empty range (invalid range with start > end).
    #[inline]
    #[must_use]
    pub const fn empty() -> Self {
        Self { start: 1, end: 0 }
    }

    /// Create a range for the entire buffer.
    ///
    /// # Arguments
    /// * `line_count` - Total number of lines in the buffer
    #[inline]
    #[must_use]
    pub const fn whole_buffer(line_count: LineNr) -> Self {
        Self {
            start: 1,
            end: line_count,
        }
    }

    /// Check if this range is empty (invalid).
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.start > self.end
    }

    /// Check if this range is valid (non-empty and positive line numbers).
    #[inline]
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.start >= 1 && self.end >= self.start
    }

    /// Get the number of lines in this range.
    ///
    /// Returns 0 for empty/invalid ranges.
    #[inline]
    #[must_use]
    pub const fn len(&self) -> LineNr {
        if self.is_empty() {
            0
        } else {
            self.end - self.start + 1
        }
    }

    /// Check if a line number is within this range.
    #[inline]
    #[must_use]
    pub const fn contains(&self, line: LineNr) -> bool {
        line >= self.start && line <= self.end
    }

    /// Clamp this range to valid buffer bounds.
    ///
    /// # Arguments
    /// * `line_count` - Total number of lines in the buffer
    ///
    /// Returns a new range clamped to [1, line_count].
    #[inline]
    #[must_use]
    pub fn clamp(&self, line_count: LineNr) -> Self {
        if line_count <= 0 {
            return Self::empty();
        }
        Self {
            start: self.start.clamp(1, line_count),
            end: self.end.clamp(1, line_count),
        }
    }

    /// Extend this range to include another line.
    #[inline]
    pub fn extend_to(&mut self, line: LineNr) {
        if self.is_empty() {
            self.start = line;
            self.end = line;
        } else {
            if line < self.start {
                self.start = line;
            }
            if line > self.end {
                self.end = line;
            }
        }
    }

    /// Merge with another range, creating a range that spans both.
    #[must_use]
    pub fn merge(&self, other: &Self) -> Self {
        if self.is_empty() {
            return *other;
        }
        if other.is_empty() {
            return *self;
        }
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    /// Check if this range overlaps with another.
    #[inline]
    #[must_use]
    pub const fn overlaps(&self, other: &Self) -> bool {
        !self.is_empty() && !other.is_empty() && self.start <= other.end && other.start <= self.end
    }

    /// Create an iterator over the line numbers in this range.
    #[inline]
    pub fn iter(&self) -> LineRangeIter {
        LineRangeIter {
            current: self.start,
            end: self.end,
        }
    }
}

impl Default for LineRange {
    fn default() -> Self {
        Self::empty()
    }
}

impl IntoIterator for LineRange {
    type Item = LineNr;
    type IntoIter = LineRangeIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for &LineRange {
    type Item = LineNr;
    type IntoIter = LineRangeIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Iterator over line numbers in a range.
#[derive(Debug, Clone)]
pub struct LineRangeIter {
    current: LineNr,
    end: LineNr,
}

impl Iterator for LineRangeIter {
    type Item = LineNr;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current <= self.end {
            let line = self.current;
            self.current += 1;
            Some(line)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = if self.current > self.end {
            0
        } else {
            (self.end - self.current + 1) as usize
        };
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for LineRangeIter {}

impl DoubleEndedIterator for LineRangeIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.current <= self.end {
            let line = self.end;
            self.end -= 1;
            Some(line)
        } else {
            None
        }
    }
}

/// Represents the result of parsing an address/range specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressSpec {
    /// A specific line number
    Line(LineNr),
    /// Current line (.)
    Current,
    /// Last line ($)
    Last,
    /// All lines (%)
    All,
    /// Mark ('{a-z})
    Mark(char),
    /// Visual selection ('<, '>)
    Visual,
    /// Search forward (/pattern/)
    SearchForward,
    /// Search backward (?pattern?)
    SearchBackward,
    /// Offset from another address (+N or -N)
    Offset(i32),
}

/// Result of validating/resolving a line range against a buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeValidation {
    /// Range is valid
    Valid(LineRange),
    /// Range is empty (e.g., 10,5)
    Empty,
    /// Line number out of bounds
    OutOfBounds { line: LineNr, max: LineNr },
    /// Invalid negative line number
    NegativeLine(LineNr),
}

impl RangeValidation {
    /// Convert to a Result for easier error handling.
    pub fn into_result(self) -> Result<LineRange, RangeError> {
        match self {
            RangeValidation::Valid(range) => Ok(range),
            RangeValidation::Empty => Err(RangeError::Empty),
            RangeValidation::OutOfBounds { line, max } => {
                Err(RangeError::OutOfBounds { line, max })
            }
            RangeValidation::NegativeLine(line) => Err(RangeError::NegativeLine(line)),
        }
    }
}

/// Error type for range validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeError {
    /// Range is empty (start > end)
    Empty,
    /// Line number is out of buffer bounds
    OutOfBounds { line: LineNr, max: LineNr },
    /// Line number is negative
    NegativeLine(LineNr),
}

impl std::fmt::Display for RangeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RangeError::Empty => write!(f, "empty range"),
            RangeError::OutOfBounds { line, max } => {
                write!(f, "line {line} is beyond end of buffer ({max} lines)")
            }
            RangeError::NegativeLine(line) => write!(f, "invalid line number: {line}"),
        }
    }
}

impl std::error::Error for RangeError {}

/// Validate a line range against buffer bounds.
///
/// # Arguments
/// * `start` - Start line (from exarg line1)
/// * `end` - End line (from exarg line2)
/// * `line_count` - Total lines in buffer
///
/// # Returns
/// A `RangeValidation` indicating whether the range is valid.
#[must_use]
pub fn validate_range(start: LineNr, end: LineNr, line_count: LineNr) -> RangeValidation {
    if start < 0 {
        return RangeValidation::NegativeLine(start);
    }
    if end < 0 {
        return RangeValidation::NegativeLine(end);
    }
    if start > end {
        return RangeValidation::Empty;
    }
    if start > line_count {
        return RangeValidation::OutOfBounds {
            line: start,
            max: line_count,
        };
    }
    if end > line_count {
        return RangeValidation::OutOfBounds {
            line: end,
            max: line_count,
        };
    }

    RangeValidation::Valid(LineRange::new(start, end))
}

/// Create a range from Ex command arguments.
///
/// Handles the various ways ranges can be specified:
/// - No range: use default (usually current line)
/// - One address: single line
/// - Two addresses: range
///
/// # Arguments
/// * `line1` - First line number from exarg
/// * `line2` - Second line number from exarg
/// * `addr_count` - Number of addresses given (0, 1, or 2)
/// * `default_line` - Default line when no range given (usually cursor line)
///
/// # Returns
/// A `LineRange` representing the command's target lines.
#[must_use]
pub fn range_from_exarg(
    line1: LineNr,
    line2: LineNr,
    addr_count: c_int,
    default_line: LineNr,
) -> LineRange {
    match addr_count {
        0 => LineRange::single(default_line),
        1 => LineRange::single(line1),
        _ => LineRange::new(line1, line2),
    }
}

/// Create a range for commands that default to the whole buffer.
///
/// When no range is given, use 1,$ (whole buffer).
///
/// # Arguments
/// * `line1` - First line number from exarg
/// * `line2` - Second line number from exarg
/// * `addr_count` - Number of addresses given
/// * `line_count` - Total lines in buffer
#[must_use]
pub fn range_default_all(
    line1: LineNr,
    line2: LineNr,
    addr_count: c_int,
    line_count: LineNr,
) -> LineRange {
    match addr_count {
        0 => LineRange::whole_buffer(line_count),
        1 => LineRange::single(line1),
        _ => LineRange::new(line1, line2),
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_range_new() {
        let range = LineRange::new(5, 10);
        assert_eq!(range.start, 5);
        assert_eq!(range.end, 10);
        assert!(!range.is_empty());
        assert!(range.is_valid());
        assert_eq!(range.len(), 6);
    }

    #[test]
    fn test_line_range_single() {
        let range = LineRange::single(7);
        assert_eq!(range.start, 7);
        assert_eq!(range.end, 7);
        assert_eq!(range.len(), 1);
        assert!(range.contains(7));
        assert!(!range.contains(6));
    }

    #[test]
    fn test_line_range_empty() {
        let range = LineRange::empty();
        assert!(range.is_empty());
        assert!(!range.is_valid());
        assert_eq!(range.len(), 0);
    }

    #[test]
    fn test_line_range_whole_buffer() {
        let range = LineRange::whole_buffer(100);
        assert_eq!(range.start, 1);
        assert_eq!(range.end, 100);
        assert_eq!(range.len(), 100);
    }

    #[test]
    fn test_line_range_contains() {
        let range = LineRange::new(5, 10);
        assert!(!range.contains(4));
        assert!(range.contains(5));
        assert!(range.contains(7));
        assert!(range.contains(10));
        assert!(!range.contains(11));
    }

    #[test]
    fn test_line_range_clamp() {
        let range = LineRange::new(0, 150);
        let clamped = range.clamp(100);
        assert_eq!(clamped.start, 1);
        assert_eq!(clamped.end, 100);

        let range2 = LineRange::new(50, 200);
        let clamped2 = range2.clamp(100);
        assert_eq!(clamped2.start, 50);
        assert_eq!(clamped2.end, 100);
    }

    #[test]
    fn test_line_range_clamp_empty_buffer() {
        let range = LineRange::new(1, 10);
        let clamped = range.clamp(0);
        assert!(clamped.is_empty());
    }

    #[test]
    fn test_line_range_extend_to() {
        let mut range = LineRange::new(5, 10);
        range.extend_to(3);
        assert_eq!(range.start, 3);
        assert_eq!(range.end, 10);

        range.extend_to(15);
        assert_eq!(range.start, 3);
        assert_eq!(range.end, 15);
    }

    #[test]
    fn test_line_range_extend_to_empty() {
        let mut range = LineRange::empty();
        range.extend_to(5);
        assert_eq!(range.start, 5);
        assert_eq!(range.end, 5);
    }

    #[test]
    fn test_line_range_merge() {
        let range1 = LineRange::new(5, 10);
        let range2 = LineRange::new(8, 15);
        let merged = range1.merge(&range2);
        assert_eq!(merged.start, 5);
        assert_eq!(merged.end, 15);

        let range3 = LineRange::new(1, 3);
        let merged2 = range1.merge(&range3);
        assert_eq!(merged2.start, 1);
        assert_eq!(merged2.end, 10);
    }

    #[test]
    fn test_line_range_merge_with_empty() {
        let range = LineRange::new(5, 10);
        let empty = LineRange::empty();

        let merged1 = range.merge(&empty);
        assert_eq!(merged1, range);

        let merged2 = empty.merge(&range);
        assert_eq!(merged2, range);
    }

    #[test]
    fn test_line_range_overlaps() {
        let range1 = LineRange::new(5, 10);
        let range2 = LineRange::new(8, 15);
        assert!(range1.overlaps(&range2));

        let range3 = LineRange::new(11, 15);
        assert!(!range1.overlaps(&range3));

        let range4 = LineRange::new(1, 4);
        assert!(!range1.overlaps(&range4));

        // Adjacent ranges don't overlap
        let range5 = LineRange::new(1, 5);
        assert!(range1.overlaps(&range5)); // 5 is in both

        let range6 = LineRange::new(10, 15);
        assert!(range1.overlaps(&range6)); // 10 is in both
    }

    #[test]
    fn test_line_range_iterator() {
        let range = LineRange::new(3, 7);
        let lines: Vec<_> = range.iter().collect();
        assert_eq!(lines, vec![3, 4, 5, 6, 7]);
    }

    #[test]
    fn test_line_range_iterator_single() {
        let range = LineRange::single(5);
        let lines: Vec<_> = range.iter().collect();
        assert_eq!(lines, vec![5]);
    }

    #[test]
    fn test_line_range_iterator_empty() {
        let range = LineRange::empty();
        let lines: Vec<_> = range.iter().collect();
        assert!(lines.is_empty());
    }

    #[test]
    fn test_line_range_iterator_reverse() {
        let range = LineRange::new(3, 7);
        let lines: Vec<_> = range.iter().rev().collect();
        assert_eq!(lines, vec![7, 6, 5, 4, 3]);
    }

    #[test]
    fn test_line_range_iterator_size_hint() {
        let range = LineRange::new(3, 7);
        let iter = range.iter();
        assert_eq!(iter.size_hint(), (5, Some(5)));
        assert_eq!(iter.len(), 5);
    }

    #[test]
    fn test_validate_range_valid() {
        let result = validate_range(5, 10, 100);
        assert!(matches!(result, RangeValidation::Valid(_)));
        if let RangeValidation::Valid(range) = result {
            assert_eq!(range.start, 5);
            assert_eq!(range.end, 10);
        }
    }

    #[test]
    fn test_validate_range_empty() {
        let result = validate_range(10, 5, 100);
        assert!(matches!(result, RangeValidation::Empty));
    }

    #[test]
    fn test_validate_range_out_of_bounds() {
        let result = validate_range(5, 150, 100);
        assert!(matches!(result, RangeValidation::OutOfBounds { .. }));
    }

    #[test]
    fn test_validate_range_negative() {
        let result = validate_range(-5, 10, 100);
        assert!(matches!(result, RangeValidation::NegativeLine(-5)));
    }

    #[test]
    fn test_range_from_exarg() {
        // No addresses given - use default
        let range = range_from_exarg(0, 0, 0, 42);
        assert_eq!(range, LineRange::single(42));

        // One address given
        let range = range_from_exarg(10, 0, 1, 42);
        assert_eq!(range, LineRange::single(10));

        // Two addresses given
        let range = range_from_exarg(5, 15, 2, 42);
        assert_eq!(range, LineRange::new(5, 15));
    }

    #[test]
    fn test_range_default_all() {
        // No addresses - whole buffer
        let range = range_default_all(0, 0, 0, 100);
        assert_eq!(range, LineRange::whole_buffer(100));

        // One address - single line
        let range = range_default_all(10, 0, 1, 100);
        assert_eq!(range, LineRange::single(10));

        // Two addresses - given range
        let range = range_default_all(5, 15, 2, 100);
        assert_eq!(range, LineRange::new(5, 15));
    }

    #[test]
    fn test_range_error_display() {
        let err = RangeError::Empty;
        assert_eq!(format!("{err}"), "empty range");

        let err = RangeError::OutOfBounds {
            line: 150,
            max: 100,
        };
        assert_eq!(
            format!("{err}"),
            "line 150 is beyond end of buffer (100 lines)"
        );

        let err = RangeError::NegativeLine(-5);
        assert_eq!(format!("{err}"), "invalid line number: -5");
    }

    #[test]
    fn test_range_validation_into_result() {
        let valid = RangeValidation::Valid(LineRange::new(5, 10));
        assert!(valid.into_result().is_ok());

        let empty = RangeValidation::Empty;
        assert!(matches!(empty.into_result(), Err(RangeError::Empty)));
    }
}
