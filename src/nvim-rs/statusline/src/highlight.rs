//! Highlight tracking for statusline
//!
//! This module provides highlight tracking and management for statusline rendering.
//! It handles `%#HighlightGroup#` parsing, `%*` reset, and highlight span tracking.

use std::ffi::c_int;

use crate::format::StlFlag;

/// Highlight record for statusline rendering.
///
/// This matches the C `stl_hlrec` struct in statusline_defs.h.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct StlHighlightRecord {
    /// Start position in the output buffer (byte offset)
    pub start: usize,
    /// User highlight group (0=none, 1-9=User1-9, <0=syn_id)
    pub userhl: c_int,
    /// Item flag for statuscolumn highlighting
    pub item: StlFlag,
}

impl StlHighlightRecord {
    /// Create a new highlight record.
    pub const fn new(start: usize, userhl: c_int) -> Self {
        Self {
            start,
            userhl,
            item: StlFlag::FilePath, // Default, will be updated for statuscolumn
        }
    }

    /// Create a highlight record for statuscolumn items.
    pub const fn with_item(start: usize, userhl: c_int, item: StlFlag) -> Self {
        Self {
            start,
            userhl,
            item,
        }
    }

    /// Create a record that resets to default highlighting.
    pub const fn reset(start: usize) -> Self {
        Self::new(start, 0)
    }

    /// Check if this record uses default highlighting.
    pub const fn is_default(&self) -> bool {
        self.userhl == 0
    }

    /// Check if this record uses user highlight (1-9).
    pub const fn is_user_hl(&self) -> bool {
        self.userhl > 0 && self.userhl <= 9
    }

    /// Check if this record uses a named highlight group (syn_id).
    pub const fn is_named_hl(&self) -> bool {
        self.userhl < 0
    }

    /// Get the syntax ID for named highlights.
    pub const fn syn_id(&self) -> Option<c_int> {
        if self.userhl < 0 {
            Some(-self.userhl)
        } else {
            None
        }
    }
}

/// Highlight tracker for building statusline.
///
/// This tracks highlight changes during statusline rendering
/// and builds a list of highlight records.
#[derive(Debug)]
pub struct HighlightTracker {
    /// List of highlight records
    records: Vec<StlHighlightRecord>,
    /// Current highlight value
    current_hl: c_int,
}

impl Default for HighlightTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl HighlightTracker {
    /// Create a new highlight tracker.
    pub const fn new() -> Self {
        Self {
            records: Vec::new(),
            current_hl: 0,
        }
    }

    /// Get the current highlight value.
    pub const fn current(&self) -> c_int {
        self.current_hl
    }

    /// Set user highlight (1-9, or 0 for default).
    pub fn set_user_hl(&mut self, start: usize, hl: u8) {
        let userhl = c_int::from(hl.min(9));
        if userhl != self.current_hl {
            self.records.push(StlHighlightRecord::new(start, userhl));
            self.current_hl = userhl;
        }
    }

    /// Set named highlight by syntax ID.
    pub fn set_named_hl(&mut self, start: usize, syn_id: c_int) {
        let userhl = -syn_id;
        if userhl != self.current_hl {
            self.records.push(StlHighlightRecord::new(start, userhl));
            self.current_hl = userhl;
        }
    }

    /// Reset to default highlighting.
    pub fn reset(&mut self, start: usize) {
        if self.current_hl != 0 {
            self.records.push(StlHighlightRecord::reset(start));
            self.current_hl = 0;
        }
    }

    /// Add a statuscolumn highlight record (sign or fold).
    pub fn add_statuscol_hl(&mut self, start: usize, userhl: c_int, item: StlFlag) {
        self.records
            .push(StlHighlightRecord::with_item(start, userhl, item));
        self.current_hl = userhl;
    }

    /// Get all highlight records.
    pub fn records(&self) -> &[StlHighlightRecord] {
        &self.records
    }

    /// Consume the tracker and return the records.
    pub fn into_records(self) -> Vec<StlHighlightRecord> {
        self.records
    }

    /// Get the number of records.
    pub const fn len(&self) -> usize {
        self.records.len()
    }

    /// Check if there are no records.
    pub const fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Clear all records and reset state.
    pub fn clear(&mut self) {
        self.records.clear();
        self.current_hl = 0;
    }
}

/// Parse a highlight group name from a format string.
///
/// Input: position after `%#`, returns the highlight name and new position.
/// Returns None if the format is invalid (no closing `#`).
pub fn parse_highlight_name(input: &str, start: usize) -> Option<(&str, usize)> {
    let bytes = input.as_bytes();
    let mut end = start;

    while end < bytes.len() {
        if bytes[end] == b'#' {
            return Some((&input[start..end], end + 1));
        }
        end += 1;
    }

    None // No closing #
}

/// Common highlight group IDs (matching C's HLF_* constants).
pub mod hlf {
    use std::ffi::c_int;

    /// StatusLine (current window)
    pub const S: c_int = 27;
    /// StatusLineNC (non-current windows)
    pub const SNC: c_int = 28;
    /// TabLine (not selected tab)
    pub const TP: c_int = 34;
    /// TabLineSel (selected tab)
    pub const TPS: c_int = 35;
    /// TabLineFill (fill after last tab)
    pub const TPF: c_int = 36;
    /// WinBar (current window)
    pub const WBR: c_int = 37;
    /// WinBarNC (non-current windows)
    pub const WBRNC: c_int = 38;
    /// CursorLineNr
    pub const CLN: c_int = 11;
    /// CursorLineFold
    pub const CLF: c_int = 12;
    /// FoldColumn
    pub const FC: c_int = 16;
    /// Message area
    pub const MSG: c_int = 21;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_record_new() {
        let rec = StlHighlightRecord::new(10, 5);
        assert_eq!(rec.start, 10);
        assert_eq!(rec.userhl, 5);
        assert!(rec.is_user_hl());
        assert!(!rec.is_default());
        assert!(!rec.is_named_hl());
    }

    #[test]
    fn test_highlight_record_reset() {
        let rec = StlHighlightRecord::reset(20);
        assert_eq!(rec.start, 20);
        assert_eq!(rec.userhl, 0);
        assert!(rec.is_default());
    }

    #[test]
    fn test_highlight_record_named() {
        let rec = StlHighlightRecord::new(30, -42);
        assert!(rec.is_named_hl());
        assert_eq!(rec.syn_id(), Some(42));
    }

    #[test]
    fn test_tracker_user_hl() {
        let mut tracker = HighlightTracker::new();
        assert_eq!(tracker.current(), 0);

        tracker.set_user_hl(10, 1);
        assert_eq!(tracker.current(), 1);
        assert_eq!(tracker.len(), 1);

        // Same highlight, no new record
        tracker.set_user_hl(20, 1);
        assert_eq!(tracker.len(), 1);

        // Different highlight
        tracker.set_user_hl(30, 2);
        assert_eq!(tracker.current(), 2);
        assert_eq!(tracker.len(), 2);
    }

    #[test]
    fn test_tracker_named_hl() {
        let mut tracker = HighlightTracker::new();

        tracker.set_named_hl(10, 42);
        assert_eq!(tracker.current(), -42);
        assert_eq!(tracker.len(), 1);

        let records = tracker.records();
        assert_eq!(records[0].userhl, -42);
    }

    #[test]
    fn test_tracker_reset() {
        let mut tracker = HighlightTracker::new();

        // Reset when already default - no record
        tracker.reset(5);
        assert_eq!(tracker.len(), 0);

        // Set then reset
        tracker.set_user_hl(10, 3);
        tracker.reset(20);
        assert_eq!(tracker.current(), 0);
        assert_eq!(tracker.len(), 2);
    }

    #[test]
    fn test_tracker_clear() {
        let mut tracker = HighlightTracker::new();
        tracker.set_user_hl(10, 1);
        tracker.set_named_hl(20, 42);

        tracker.clear();
        assert!(tracker.is_empty());
        assert_eq!(tracker.current(), 0);
    }

    #[test]
    fn test_parse_highlight_name_valid() {
        let result = parse_highlight_name("StatusLine#rest", 0);
        assert_eq!(result, Some(("StatusLine", 11)));
    }

    #[test]
    fn test_parse_highlight_name_empty() {
        let result = parse_highlight_name("#rest", 0);
        assert_eq!(result, Some(("", 1)));
    }

    #[test]
    fn test_parse_highlight_name_invalid() {
        let result = parse_highlight_name("NoClosing", 0);
        assert_eq!(result, None);
    }

    #[test]
    fn test_hlf_constants() {
        // Verify constants match expected values
        assert_eq!(hlf::S, 27);
        assert_eq!(hlf::SNC, 28);
        assert_eq!(hlf::TP, 34);
        assert_eq!(hlf::TPS, 35);
        assert_eq!(hlf::TPF, 36);
    }
}
