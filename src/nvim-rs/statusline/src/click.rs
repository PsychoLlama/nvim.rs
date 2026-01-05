//! Click region tracking for statusline
//!
//! This module provides click region tracking and management for statusline/tabline.
//! It handles `%@FuncName@` click handler parsing and region tracking.

use std::ffi::{c_char, c_int};

/// Click handler type for statusline regions.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickType {
    /// Clicks to this area are ignored
    Disabled = 0,
    /// Switch to the given tab
    TabSwitch = 1,
    /// Close given tab
    TabClose = 2,
    /// Run user function
    FuncRun = 3,
}

impl ClickType {
    /// Check if this click type is disabled.
    pub const fn is_disabled(self) -> bool {
        matches!(self, Self::Disabled)
    }

    /// Check if this click type triggers a function call.
    pub const fn is_func_run(self) -> bool {
        matches!(self, Self::FuncRun)
    }

    /// Check if this click type handles tab actions.
    pub const fn is_tab_action(self) -> bool {
        matches!(self, Self::TabSwitch | Self::TabClose)
    }
}

/// Click definition for a region.
///
/// This matches the C `StlClickDefinition` struct in statusline_defs.h.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ClickDefinition {
    /// Type of the click
    pub click_type: ClickType,
    /// Tab page number (for tab actions) or user data
    pub tabnr: c_int,
    /// Function to run (for FuncRun type, may be null)
    pub func: *mut c_char,
}

impl ClickDefinition {
    /// Create a disabled click definition.
    pub const fn disabled() -> Self {
        Self {
            click_type: ClickType::Disabled,
            tabnr: 0,
            func: std::ptr::null_mut(),
        }
    }

    /// Create a tab switch click definition.
    pub const fn tab_switch(tabnr: c_int) -> Self {
        Self {
            click_type: ClickType::TabSwitch,
            tabnr,
            func: std::ptr::null_mut(),
        }
    }

    /// Create a tab close click definition.
    pub const fn tab_close(tabnr: c_int) -> Self {
        Self {
            click_type: ClickType::TabClose,
            tabnr,
            func: std::ptr::null_mut(),
        }
    }

    /// Create a function run click definition.
    ///
    /// # Safety
    /// `func` must be a valid C string pointer or null.
    pub const fn func_run(func: *mut c_char, minwid: c_int) -> Self {
        Self {
            click_type: ClickType::FuncRun,
            tabnr: minwid,
            func,
        }
    }

    /// Check if this click definition is disabled.
    pub const fn is_disabled(&self) -> bool {
        self.click_type.is_disabled()
    }
}

impl Default for ClickDefinition {
    fn default() -> Self {
        Self::disabled()
    }
}

/// Click record for tracking click regions.
///
/// This matches the C `StlClickRecord` struct in statusline_defs.h.
#[repr(C)]
pub struct ClickRecord {
    /// Click definition for this region
    pub def: ClickDefinition,
    /// Start position in the output buffer (pointer for C compat)
    pub start: *const c_char,
}

impl ClickRecord {
    /// Create a new click record.
    pub const fn new(def: ClickDefinition, start: *const c_char) -> Self {
        Self { def, start }
    }

    /// Create a disabled click record at the given position.
    pub const fn disabled(start: *const c_char) -> Self {
        Self {
            def: ClickDefinition::disabled(),
            start,
        }
    }

    /// Create a terminator record (marks end of click records).
    pub const fn terminator() -> Self {
        Self {
            def: ClickDefinition::disabled(),
            start: std::ptr::null(),
        }
    }

    /// Check if this is a terminator record.
    pub const fn is_terminator(&self) -> bool {
        self.start.is_null()
    }
}

impl Default for ClickRecord {
    fn default() -> Self {
        Self::terminator()
    }
}

/// Click region tracker for building statusline/tabline.
///
/// This tracks click regions during statusline rendering
/// and builds a list of click records.
#[derive(Debug)]
pub struct ClickTracker {
    /// List of click records (start position + definition)
    records: Vec<ClickRecordInternal>,
    /// Current click definition
    current_def: ClickType,
    /// Current tab number
    current_tabnr: c_int,
    /// Current function name (owned string for safety)
    current_func: Option<String>,
}

/// Internal click record with owned function name.
#[derive(Debug, Clone)]
struct ClickRecordInternal {
    /// Start position in the output buffer (byte offset)
    start: usize,
    /// Click type
    click_type: ClickType,
    /// Tab number
    tabnr: c_int,
    /// Function name (owned)
    func: Option<String>,
}

impl Default for ClickTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl ClickTracker {
    /// Create a new click tracker.
    pub const fn new() -> Self {
        Self {
            records: Vec::new(),
            current_def: ClickType::Disabled,
            current_tabnr: 0,
            current_func: None,
        }
    }

    /// Get the current click type.
    pub const fn current_type(&self) -> ClickType {
        self.current_def
    }

    /// Start a tab switch region.
    pub fn start_tab_switch(&mut self, start: usize, tabnr: c_int) {
        self.current_def = ClickType::TabSwitch;
        self.current_tabnr = tabnr;
        self.current_func = None;
        self.push_record(start);
    }

    /// Start a tab close region.
    pub fn start_tab_close(&mut self, start: usize, tabnr: c_int) {
        self.current_def = ClickType::TabClose;
        self.current_tabnr = tabnr.abs();
        self.current_func = None;
        self.push_record(start);
    }

    /// Start a function run region.
    pub fn start_func_run(&mut self, start: usize, func: &str, minwid: c_int) {
        self.current_def = ClickType::FuncRun;
        self.current_tabnr = minwid;
        self.current_func = Some(func.to_string());
        self.push_record(start);
    }

    /// End the current click region (return to disabled).
    pub fn end_region(&mut self, start: usize) {
        if self.current_def != ClickType::Disabled {
            self.current_def = ClickType::Disabled;
            self.current_tabnr = 0;
            self.current_func = None;
            self.push_record(start);
        }
    }

    /// Push the current state as a record.
    fn push_record(&mut self, start: usize) {
        self.records.push(ClickRecordInternal {
            start,
            click_type: self.current_def,
            tabnr: self.current_tabnr,
            func: self.current_func.clone(),
        });
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
        self.current_def = ClickType::Disabled;
        self.current_tabnr = 0;
        self.current_func = None;
    }

    /// Iterate over records as (start, click_type, tabnr, func_name).
    pub fn iter(&self) -> impl Iterator<Item = (usize, ClickType, c_int, Option<&str>)> {
        self.records
            .iter()
            .map(|r| (r.start, r.click_type, r.tabnr, r.func.as_deref()))
    }
}

/// Parse a click function name from a format string.
///
/// Input: position after `%@`, returns the function name and new position.
/// Returns None if the format is invalid (no closing `@`).
///
/// The format is `%@FuncName@` where FuncName is the function to call.
pub fn parse_click_func(input: &str, start: usize) -> Option<(&str, usize)> {
    let bytes = input.as_bytes();
    let mut end = start;

    while end < bytes.len() {
        if bytes[end] == b'@' {
            return Some((&input[start..end], end + 1));
        }
        end += 1;
    }

    None // No closing @
}

/// Determine click type from minwid value in tab items.
///
/// - `minwid > 0`: TabSwitch to tab number `minwid`
/// - `minwid < 0`: TabClose tab number `-minwid`
/// - `minwid == 0`: Disabled
pub const fn click_type_from_minwid(minwid: c_int) -> (ClickType, c_int) {
    if minwid > 0 {
        (ClickType::TabSwitch, minwid)
    } else if minwid < 0 {
        (ClickType::TabClose, -minwid)
    } else {
        (ClickType::Disabled, 0)
    }
}

/// Check if a click type is valid for non-tabline use.
///
/// Window bar and status line only support click functions (FuncRun)
/// and disabled state. Tab switch/close are tabline-only.
pub const fn is_valid_for_statusline(click_type: ClickType) -> bool {
    matches!(click_type, ClickType::Disabled | ClickType::FuncRun)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_click_type_values() {
        assert_eq!(ClickType::Disabled as c_int, 0);
        assert_eq!(ClickType::TabSwitch as c_int, 1);
        assert_eq!(ClickType::TabClose as c_int, 2);
        assert_eq!(ClickType::FuncRun as c_int, 3);
    }

    #[test]
    fn test_click_type_predicates() {
        assert!(ClickType::Disabled.is_disabled());
        assert!(!ClickType::TabSwitch.is_disabled());
        assert!(ClickType::FuncRun.is_func_run());
        assert!(ClickType::TabSwitch.is_tab_action());
        assert!(ClickType::TabClose.is_tab_action());
        assert!(!ClickType::FuncRun.is_tab_action());
    }

    #[test]
    fn test_click_definition_disabled() {
        let def = ClickDefinition::disabled();
        assert!(def.is_disabled());
        assert_eq!(def.tabnr, 0);
        assert!(def.func.is_null());
    }

    #[test]
    fn test_click_definition_tab_switch() {
        let def = ClickDefinition::tab_switch(5);
        assert_eq!(def.click_type, ClickType::TabSwitch);
        assert_eq!(def.tabnr, 5);
        assert!(def.func.is_null());
    }

    #[test]
    fn test_click_definition_tab_close() {
        let def = ClickDefinition::tab_close(3);
        assert_eq!(def.click_type, ClickType::TabClose);
        assert_eq!(def.tabnr, 3);
        assert!(def.func.is_null());
    }

    #[test]
    fn test_click_record_terminator() {
        let rec = ClickRecord::terminator();
        assert!(rec.is_terminator());
        assert!(rec.def.is_disabled());
    }

    #[test]
    fn test_click_tracker_new() {
        let tracker = ClickTracker::new();
        assert!(tracker.is_empty());
        assert_eq!(tracker.current_type(), ClickType::Disabled);
    }

    #[test]
    fn test_click_tracker_tab_switch() {
        let mut tracker = ClickTracker::new();

        tracker.start_tab_switch(10, 2);
        assert_eq!(tracker.current_type(), ClickType::TabSwitch);
        assert_eq!(tracker.len(), 1);

        tracker.end_region(20);
        assert_eq!(tracker.current_type(), ClickType::Disabled);
        assert_eq!(tracker.len(), 2);
    }

    #[test]
    fn test_click_tracker_tab_close() {
        let mut tracker = ClickTracker::new();

        // Tab close uses absolute value of tabnr
        tracker.start_tab_close(5, -3);
        assert_eq!(tracker.current_type(), ClickType::TabClose);

        let records: Vec<_> = tracker.iter().collect();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].2, 3); // tabnr should be positive
    }

    #[test]
    fn test_click_tracker_func_run() {
        let mut tracker = ClickTracker::new();

        tracker.start_func_run(0, "MyClickHandler", 42);
        assert_eq!(tracker.current_type(), ClickType::FuncRun);

        let records: Vec<_> = tracker.iter().collect();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].0, 0); // start
        assert_eq!(records[0].1, ClickType::FuncRun); // click_type
        assert_eq!(records[0].2, 42); // minwid
        assert_eq!(records[0].3, Some("MyClickHandler")); // func
    }

    #[test]
    fn test_click_tracker_multiple_regions() {
        let mut tracker = ClickTracker::new();

        tracker.start_tab_switch(0, 1);
        tracker.start_tab_switch(10, 2);
        tracker.start_tab_close(20, 3);
        tracker.end_region(30);

        assert_eq!(tracker.len(), 4);

        let records: Vec<_> = tracker.iter().collect();
        // Records store the new state at each position
        assert_eq!(records[0].1, ClickType::TabSwitch);
        assert_eq!(records[0].2, 1); // tabnr
        assert_eq!(records[1].1, ClickType::TabSwitch);
        assert_eq!(records[1].2, 2); // tabnr
        assert_eq!(records[2].1, ClickType::TabClose);
        assert_eq!(records[2].2, 3); // tabnr
        assert_eq!(records[3].1, ClickType::Disabled);
    }

    #[test]
    fn test_click_tracker_clear() {
        let mut tracker = ClickTracker::new();
        tracker.start_tab_switch(10, 1);
        tracker.start_func_run(20, "Test", 0);

        tracker.clear();
        assert!(tracker.is_empty());
        assert_eq!(tracker.current_type(), ClickType::Disabled);
    }

    #[test]
    fn test_parse_click_func_valid() {
        let result = parse_click_func("MyFunc@rest", 0);
        assert_eq!(result, Some(("MyFunc", 7)));
    }

    #[test]
    fn test_parse_click_func_empty() {
        let result = parse_click_func("@rest", 0);
        assert_eq!(result, Some(("", 1)));
    }

    #[test]
    fn test_parse_click_func_invalid() {
        let result = parse_click_func("NoClosing", 0);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_click_func_offset() {
        let result = parse_click_func("prefix@Func@suffix", 7);
        assert_eq!(result, Some(("Func", 12)));
    }

    #[test]
    fn test_click_type_from_minwid_positive() {
        let (click_type, tabnr) = click_type_from_minwid(5);
        assert_eq!(click_type, ClickType::TabSwitch);
        assert_eq!(tabnr, 5);
    }

    #[test]
    fn test_click_type_from_minwid_negative() {
        let (click_type, tabnr) = click_type_from_minwid(-3);
        assert_eq!(click_type, ClickType::TabClose);
        assert_eq!(tabnr, 3);
    }

    #[test]
    fn test_click_type_from_minwid_zero() {
        let (click_type, tabnr) = click_type_from_minwid(0);
        assert_eq!(click_type, ClickType::Disabled);
        assert_eq!(tabnr, 0);
    }

    #[test]
    fn test_is_valid_for_statusline() {
        assert!(is_valid_for_statusline(ClickType::Disabled));
        assert!(is_valid_for_statusline(ClickType::FuncRun));
        assert!(!is_valid_for_statusline(ClickType::TabSwitch));
        assert!(!is_valid_for_statusline(ClickType::TabClose));
    }
}
