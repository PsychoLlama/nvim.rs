//! Line range calculations for match highlighting
//!
//! This module provides functions for calculating line ranges that need
//! redrawing when matches are added, modified, or deleted.

use std::ffi::c_int;

// =============================================================================
// Line Range Type
// =============================================================================

/// A range of lines for redraw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineRange {
    /// Top line number (inclusive), 0 means unset
    pub top: i64,
    /// Bottom line number (exclusive), 0 means unset
    pub bot: i64,
}

impl LineRange {
    /// Create an empty/unset range.
    #[must_use]
    pub const fn empty() -> Self {
        Self { top: 0, bot: 0 }
    }

    /// Create a range from top and bottom.
    #[must_use]
    pub const fn new(top: i64, bot: i64) -> Self {
        Self { top, bot }
    }

    /// Check if the range is empty/unset.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.top == 0 && self.bot == 0
    }

    /// Check if the range is valid (top > 0 and bot > top).
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.top > 0 && self.bot > self.top
    }

    /// Expand this range to include a line number.
    #[must_use]
    pub const fn include_line(&self, lnum: i64) -> Self {
        if lnum <= 0 {
            return *self;
        }

        let new_top = if self.top == 0 || lnum < self.top {
            lnum
        } else {
            self.top
        };

        let new_bot = if self.bot == 0 || lnum >= self.bot {
            lnum + 1
        } else {
            self.bot
        };

        Self {
            top: new_top,
            bot: new_bot,
        }
    }

    /// Merge two ranges into one that covers both.
    #[must_use]
    pub const fn merge(&self, other: &Self) -> Self {
        if self.is_empty() {
            return *other;
        }
        if other.is_empty() {
            return *self;
        }

        let top = if self.top < other.top {
            self.top
        } else {
            other.top
        };
        let bot = if self.bot > other.bot {
            self.bot
        } else {
            other.bot
        };

        Self { top, bot }
    }

    /// Check if a line is within this range.
    #[must_use]
    pub const fn contains(&self, lnum: i64) -> bool {
        self.top > 0 && lnum >= self.top && lnum < self.bot
    }
}

impl Default for LineRange {
    fn default() -> Self {
        Self::empty()
    }
}

// =============================================================================
// Range Builder
// =============================================================================

/// Builder for accumulating line ranges from multiple positions.
#[derive(Debug, Clone, Copy, Default)]
pub struct RangeBuilder {
    range: LineRange,
}

impl RangeBuilder {
    /// Create a new empty builder.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            range: LineRange::empty(),
        }
    }

    /// Add a line to the range.
    pub fn add_line(&mut self, lnum: i64) {
        self.range = self.range.include_line(lnum);
    }

    /// Add a position (line, col, len) to the range.
    ///
    /// Only the line number matters for the range calculation.
    pub fn add_position(&mut self, lnum: i64, _col: i32, _len: i32) {
        if lnum > 0 {
            self.range = self.range.include_line(lnum);
        }
    }

    /// Get the accumulated range.
    #[must_use]
    pub const fn get(&self) -> LineRange {
        self.range
    }

    /// Check if any lines have been added.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.range.is_empty()
    }
}

// =============================================================================
// Range Calculations
// =============================================================================

/// Calculate the redraw range for a set of position matches.
///
/// Returns (toplnum, botlnum) for use with `redraw_win_range_later()`.
#[must_use]
pub const fn calc_position_redraw_range(positions: &[(i64, i32, i32)]) -> LineRange {
    let mut top: i64 = 0;
    let mut bot: i64 = 0;

    let mut i = 0;
    while i < positions.len() {
        let lnum = positions[i].0;
        if lnum > 0 {
            if top == 0 || lnum < top {
                top = lnum;
            }
            if bot == 0 || lnum >= bot {
                bot = lnum + 1;
            }
        }
        i += 1;
    }

    LineRange { top, bot }
}

/// Check if two line ranges overlap.
#[must_use]
pub const fn ranges_overlap(r1: &LineRange, r2: &LineRange) -> bool {
    if r1.is_empty() || r2.is_empty() {
        return false;
    }
    // Ranges overlap if neither is entirely before the other
    r1.top < r2.bot && r2.top < r1.bot
}

/// Check if a line range overlaps with a single line.
#[must_use]
pub const fn range_contains_line(range: &LineRange, lnum: i64) -> bool {
    range.contains(lnum)
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Create an empty line range.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_range_empty_top() -> i64 {
    0
}

/// Create an empty line range.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_range_empty_bot() -> i64 {
    0
}

/// Include a line in a range, returning new top.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_range_include_line_top(current_top: i64, lnum: i64) -> i64 {
    if lnum <= 0 {
        return current_top;
    }
    if current_top == 0 || lnum < current_top {
        lnum
    } else {
        current_top
    }
}

/// Include a line in a range, returning new bot.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_range_include_line_bot(current_bot: i64, lnum: i64) -> i64 {
    if lnum <= 0 {
        return current_bot;
    }
    if current_bot == 0 || lnum >= current_bot {
        lnum + 1
    } else {
        current_bot
    }
}

/// Check if a range is valid.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_range_is_valid(top: i64, bot: i64) -> c_int {
    c_int::from(top > 0 && bot > top)
}

/// Check if a line is in a range.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_range_contains(top: i64, bot: i64, lnum: i64) -> c_int {
    c_int::from(top > 0 && lnum >= top && lnum < bot)
}

/// Check if two ranges overlap.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_ranges_overlap(top1: i64, bot1: i64, top2: i64, bot2: i64) -> c_int {
    let r1 = LineRange::new(top1, bot1);
    let r2 = LineRange::new(top2, bot2);
    c_int::from(ranges_overlap(&r1, &r2))
}

/// Merge two ranges, returning new top.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_range_merge_top(top1: i64, bot1: i64, top2: i64, bot2: i64) -> i64 {
    let r1 = LineRange::new(top1, bot1);
    let r2 = LineRange::new(top2, bot2);
    r1.merge(&r2).top
}

/// Merge two ranges, returning new bot.
#[unsafe(no_mangle)]
pub extern "C" fn rs_match_range_merge_bot(top1: i64, bot1: i64, top2: i64, bot2: i64) -> i64 {
    let r1 = LineRange::new(top1, bot1);
    let r2 = LineRange::new(top2, bot2);
    r1.merge(&r2).bot
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_range_empty() {
        let r = LineRange::empty();
        assert!(r.is_empty());
        assert!(!r.is_valid());
    }

    #[test]
    fn test_line_range_new() {
        let r = LineRange::new(5, 10);
        assert!(!r.is_empty());
        assert!(r.is_valid());
        assert_eq!(r.top, 5);
        assert_eq!(r.bot, 10);
    }

    #[test]
    fn test_line_range_include_line() {
        let r = LineRange::empty();

        // Add first line
        let r = r.include_line(5);
        assert_eq!(r.top, 5);
        assert_eq!(r.bot, 6);

        // Add line above
        let r = r.include_line(3);
        assert_eq!(r.top, 3);
        assert_eq!(r.bot, 6);

        // Add line below
        let r = r.include_line(10);
        assert_eq!(r.top, 3);
        assert_eq!(r.bot, 11);

        // Add line in middle (no change)
        let r = r.include_line(7);
        assert_eq!(r.top, 3);
        assert_eq!(r.bot, 11);

        // Invalid line (no change)
        let r = r.include_line(0);
        assert_eq!(r.top, 3);
        assert_eq!(r.bot, 11);
    }

    #[test]
    fn test_line_range_merge() {
        let r1 = LineRange::new(5, 10);
        let r2 = LineRange::new(8, 15);
        let merged = r1.merge(&r2);
        assert_eq!(merged.top, 5);
        assert_eq!(merged.bot, 15);

        // Merge with empty
        let empty = LineRange::empty();
        assert_eq!(r1.merge(&empty), r1);
        assert_eq!(empty.merge(&r1), r1);
    }

    #[test]
    fn test_line_range_contains() {
        let r = LineRange::new(5, 10);
        assert!(!r.contains(4)); // Before
        assert!(r.contains(5)); // At top
        assert!(r.contains(7)); // In middle
        assert!(r.contains(9)); // At bot - 1
        assert!(!r.contains(10)); // At bot (exclusive)
        assert!(!r.contains(11)); // After
    }

    #[test]
    fn test_range_builder() {
        let mut builder = RangeBuilder::new();
        assert!(builder.is_empty());

        builder.add_line(5);
        builder.add_line(10);
        builder.add_line(3);

        let range = builder.get();
        assert_eq!(range.top, 3);
        assert_eq!(range.bot, 11);
    }

    #[test]
    fn test_ranges_overlap() {
        let r1 = LineRange::new(5, 10);
        let r2 = LineRange::new(8, 15);
        assert!(ranges_overlap(&r1, &r2));

        let r3 = LineRange::new(10, 15); // Starts at r1's end
        assert!(!ranges_overlap(&r1, &r3));

        let r4 = LineRange::new(1, 3); // Entirely before r1
        assert!(!ranges_overlap(&r1, &r4));
    }

    #[test]
    fn test_calc_position_redraw_range() {
        let positions = [(5, 1, 3), (10, 5, 2), (3, 0, 0), (7, 10, 5)];
        let range = calc_position_redraw_range(&positions);
        assert_eq!(range.top, 3);
        assert_eq!(range.bot, 11);

        // Empty positions
        let empty: [(i64, i32, i32); 0] = [];
        let range = calc_position_redraw_range(&empty);
        assert!(range.is_empty());

        // With invalid line (0)
        let positions = [(0, 1, 1), (5, 1, 1)];
        let range = calc_position_redraw_range(&positions);
        assert_eq!(range.top, 5);
        assert_eq!(range.bot, 6);
    }
}
