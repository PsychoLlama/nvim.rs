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
    pub const S: c_int = 19;
    /// StatusLineNC (non-current windows)
    pub const SNC: c_int = 20;
    /// TabLine (not selected tab)
    pub const TP: c_int = 52;
    /// TabLineSel (selected tab)
    pub const TPS: c_int = 53;
    /// TabLineFill (fill after last tab)
    pub const TPF: c_int = 54;
    /// WinBar (current window)
    pub const WBR: c_int = 65;
    /// WinBarNC (non-current windows)
    pub const WBRNC: c_int = 66;
    /// CursorLineNr
    pub const CLN: c_int = 15;
    /// CursorLineFold
    pub const CLF: c_int = 17;
    /// FoldColumn
    pub const FC: c_int = 29;
    /// Message area
    pub const MSG: c_int = 63;
    /// LineNr
    pub const N: c_int = 12;
    /// SignColumn
    pub const SC: c_int = 35;
    /// User1-9 start
    pub const USER1: c_int = 59;
}

/// Get the appropriate statusline highlight based on whether window is current.
///
/// Returns `HLF_S` for current window, `HLF_SNC` for non-current.
pub const fn get_statusline_hl(is_curwin: bool) -> c_int {
    if is_curwin {
        hlf::S
    } else {
        hlf::SNC
    }
}

/// Get the appropriate window bar highlight based on whether window is current.
///
/// Returns `HLF_WBR` for current window, `HLF_WBRNC` for non-current.
pub const fn get_winbar_hl(is_curwin: bool) -> c_int {
    if is_curwin {
        hlf::WBR
    } else {
        hlf::WBRNC
    }
}

/// Get the appropriate tabline highlight based on whether tab is selected.
///
/// Returns `HLF_TPS` for selected tab, `HLF_TP` for non-selected.
pub const fn get_tabline_hl(is_selected: bool) -> c_int {
    if is_selected {
        hlf::TPS
    } else {
        hlf::TP
    }
}

/// Convert a user highlight number (0-9) to a userhl value.
///
/// - 0 returns 0 (default)
/// - 1-9 returns 1-9 (User1-User9)
pub const fn user_hl_to_userhl(user_num: u8) -> c_int {
    if user_num == 0 {
        0
    } else if user_num > 9 {
        9
    } else {
        user_num as c_int
    }
}

/// Highlight span for tracking highlight regions.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HighlightSpan {
    /// Start byte offset in output
    pub start: usize,
    /// End byte offset in output
    pub end: usize,
    /// Highlight attribute ID
    pub hl_attr: c_int,
}

impl HighlightSpan {
    /// Create a new highlight span.
    pub const fn new(start: usize, end: usize, hl_attr: c_int) -> Self {
        Self {
            start,
            end,
            hl_attr,
        }
    }

    /// Check if this span is empty.
    pub const fn is_empty(&self) -> bool {
        self.start >= self.end
    }

    /// Get the length of the span.
    pub const fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }
}

/// Builder for highlight spans from highlight records.
pub struct HighlightSpanBuilder {
    spans: Vec<HighlightSpan>,
    current_start: usize,
    current_hl: c_int,
}

impl Default for HighlightSpanBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl HighlightSpanBuilder {
    /// Create a new span builder.
    pub const fn new() -> Self {
        Self {
            spans: Vec::new(),
            current_start: 0,
            current_hl: 0,
        }
    }

    /// Process a highlight record.
    pub fn process_record(&mut self, rec: &StlHighlightRecord) {
        // Close previous span if there was one
        if self.current_hl != 0 && rec.start > self.current_start {
            self.spans.push(HighlightSpan::new(
                self.current_start,
                rec.start,
                self.current_hl,
            ));
        }

        // Start new span
        self.current_start = rec.start;
        self.current_hl = rec.userhl;
    }

    /// Finalize with the total length, closing any open span.
    pub fn finalize(&mut self, total_len: usize) {
        if self.current_hl != 0 && total_len > self.current_start {
            self.spans.push(HighlightSpan::new(
                self.current_start,
                total_len,
                self.current_hl,
            ));
        }
    }

    /// Get the built spans.
    pub fn spans(&self) -> &[HighlightSpan] {
        &self.spans
    }

    /// Consume and return the spans.
    pub fn into_spans(self) -> Vec<HighlightSpan> {
        self.spans
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Get statusline highlight for window.
#[no_mangle]
pub const extern "C" fn rs_stl_get_statusline_hl(is_curwin: c_int) -> c_int {
    get_statusline_hl(is_curwin != 0)
}

/// FFI export: Get winbar highlight for window.
#[no_mangle]
pub const extern "C" fn rs_stl_get_winbar_hl(is_curwin: c_int) -> c_int {
    get_winbar_hl(is_curwin != 0)
}

/// FFI export: Get tabline highlight for tab.
#[no_mangle]
pub const extern "C" fn rs_stl_get_tabline_hl(is_selected: c_int) -> c_int {
    get_tabline_hl(is_selected != 0)
}

/// FFI export: Convert user highlight number to userhl value.
#[no_mangle]
pub const extern "C" fn rs_stl_user_hl_to_userhl(user_num: u8) -> c_int {
    user_hl_to_userhl(user_num)
}

/// FFI export: Parse a highlight name from format string.
///
/// Returns the length of the highlight name, or -1 if invalid.
/// The highlight name starts at `input + start` and ends before the closing '#'.
///
/// # Safety
/// `input` must be null or a valid pointer to at least `input_len` bytes.
/// `name_end` must be null or a valid pointer to a c_int.
#[no_mangle]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_stl_parse_highlight_name(
    input: *const u8,
    input_len: usize,
    start: usize,
    name_end: *mut c_int,
) -> c_int {
    if input.is_null() || start >= input_len {
        return -1;
    }

    let slice = std::slice::from_raw_parts(input, input_len);
    let Ok(s) = std::str::from_utf8(slice) else {
        return -1;
    };

    match parse_highlight_name(s, start) {
        Some((name, end_pos)) => {
            if !name_end.is_null() {
                *name_end = end_pos as c_int;
            }
            name.len() as c_int
        }
        None => -1,
    }
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
        assert_eq!(hlf::S, 19);
        assert_eq!(hlf::SNC, 20);
        assert_eq!(hlf::TP, 52);
        assert_eq!(hlf::TPS, 53);
        assert_eq!(hlf::TPF, 54);
    }

    #[test]
    fn test_get_statusline_hl() {
        assert_eq!(get_statusline_hl(true), hlf::S);
        assert_eq!(get_statusline_hl(false), hlf::SNC);
    }

    #[test]
    fn test_get_winbar_hl() {
        assert_eq!(get_winbar_hl(true), hlf::WBR);
        assert_eq!(get_winbar_hl(false), hlf::WBRNC);
    }

    #[test]
    fn test_get_tabline_hl() {
        assert_eq!(get_tabline_hl(true), hlf::TPS);
        assert_eq!(get_tabline_hl(false), hlf::TP);
    }

    #[test]
    fn test_user_hl_to_userhl() {
        assert_eq!(user_hl_to_userhl(0), 0);
        assert_eq!(user_hl_to_userhl(1), 1);
        assert_eq!(user_hl_to_userhl(9), 9);
        assert_eq!(user_hl_to_userhl(10), 9); // Clamped
    }

    #[test]
    fn test_highlight_span() {
        let span = HighlightSpan::new(10, 20, 5);
        assert_eq!(span.start, 10);
        assert_eq!(span.end, 20);
        assert_eq!(span.len(), 10);
        assert!(!span.is_empty());

        let empty_span = HighlightSpan::new(5, 5, 0);
        assert!(empty_span.is_empty());
        assert_eq!(empty_span.len(), 0);
    }

    #[test]
    fn test_highlight_span_builder() {
        let mut builder = HighlightSpanBuilder::new();

        // Simulate: start with default, then User1 at pos 5, then reset at pos 15
        builder.process_record(&StlHighlightRecord::new(5, 1));
        builder.process_record(&StlHighlightRecord::new(15, 0));
        builder.finalize(20);

        let spans = builder.spans();
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].start, 5);
        assert_eq!(spans[0].end, 15);
        assert_eq!(spans[0].hl_attr, 1);
    }

    #[test]
    fn test_highlight_span_builder_unclosed() {
        let mut builder = HighlightSpanBuilder::new();

        // Start User1 at pos 5, never reset
        builder.process_record(&StlHighlightRecord::new(5, 1));
        builder.finalize(20);

        let spans = builder.spans();
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].start, 5);
        assert_eq!(spans[0].end, 20);
    }
}
