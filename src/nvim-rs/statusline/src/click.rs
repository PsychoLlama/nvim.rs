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

// =============================================================================
// FFI Exports for Click Definitions
// =============================================================================

extern "C" {
    fn xfree(ptr: *mut std::ffi::c_void);
    fn vim_strnsize(s: *const c_char, len: c_int) -> c_int;
}

/// Clear status line, window bar or tab page line click definition table.
///
/// # Safety
/// - `click_defs` must be a valid pointer or null.
/// - `click_defs_size` must be the actual size of the array.
#[no_mangle]
pub unsafe extern "C" fn rs_stl_clear_click_defs(
    click_defs: *mut ClickDefinition,
    click_defs_size: usize,
) {
    if click_defs.is_null() {
        return;
    }

    let defs = std::slice::from_raw_parts_mut(click_defs, click_defs_size);

    for i in 0..click_defs_size {
        // Only free if this is the first occurrence of this func pointer
        // (consecutive entries may share the same pointer)
        if (i == 0 || defs[i].func != defs[i - 1].func) && !defs[i].func.is_null() {
            xfree(defs[i].func.cast());
        }
        defs[i] = ClickDefinition::disabled();
    }
}

/// Allocate or resize the click definitions array if needed.
///
/// # Safety
/// - `cdp` must be a valid pointer allocated by xmalloc/xcalloc, or null.
/// - `size` must be a valid pointer to a size_t.
///
/// # Panics
/// May panic if the allocation layout overflows.
#[no_mangle]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_ptr_alignment
)]
pub unsafe extern "C" fn rs_stl_alloc_click_defs(
    cdp: *mut ClickDefinition,
    width: c_int,
    size: *mut usize,
) -> *mut ClickDefinition {
    if size.is_null() {
        return cdp;
    }

    let current_size = *size;
    let needed_size = width as usize;

    if current_size < needed_size {
        if !cdp.is_null() {
            xfree(cdp.cast());
        }
        *size = needed_size;

        // Allocate new array
        let layout = std::alloc::Layout::array::<ClickDefinition>(needed_size).unwrap();
        let ptr = std::alloc::alloc_zeroed(layout).cast::<ClickDefinition>();

        // Initialize all entries to disabled
        if !ptr.is_null() {
            for i in 0..needed_size {
                std::ptr::write(ptr.add(i), ClickDefinition::disabled());
            }
        }
        ptr
    } else {
        cdp
    }
}

// =============================================================================
// Tabline Support
// =============================================================================

/// Tabline item type for rendering.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TablineItemType {
    /// Normal tab label
    Tab = 0,
    /// Tab close button (X)
    Close = 1,
    /// Fill area (after all tabs)
    Fill = 2,
}

/// Tabline item for building tabline.
#[derive(Debug, Clone)]
pub struct TablineItem {
    /// Item type
    pub item_type: TablineItemType,
    /// Tab page number (1-based, 0 for fill)
    pub tabnr: c_int,
    /// Start column in output
    pub start_col: c_int,
    /// End column in output
    pub end_col: c_int,
    /// Whether this tab is selected
    pub is_selected: bool,
}

impl TablineItem {
    /// Create a new tab item.
    pub const fn tab(tabnr: c_int, start_col: c_int, end_col: c_int, is_selected: bool) -> Self {
        Self {
            item_type: TablineItemType::Tab,
            tabnr,
            start_col,
            end_col,
            is_selected,
        }
    }

    /// Create a new close button item.
    pub const fn close(tabnr: c_int, start_col: c_int, end_col: c_int) -> Self {
        Self {
            item_type: TablineItemType::Close,
            tabnr,
            start_col,
            end_col,
            is_selected: false,
        }
    }

    /// Create a new fill item.
    pub const fn fill(start_col: c_int, end_col: c_int) -> Self {
        Self {
            item_type: TablineItemType::Fill,
            tabnr: 0,
            start_col,
            end_col,
            is_selected: false,
        }
    }

    /// Get the width of this item.
    pub const fn width(&self) -> c_int {
        self.end_col - self.start_col
    }
}

/// Tabline builder for creating tabline.
#[derive(Debug, Default)]
pub struct TablineBuilder {
    /// Items in the tabline
    items: Vec<TablineItem>,
    /// Current column position
    current_col: c_int,
    /// Associated click tracker
    clicks: ClickTracker,
}

impl TablineBuilder {
    /// Create a new tabline builder.
    pub const fn new() -> Self {
        Self {
            items: Vec::new(),
            current_col: 0,
            clicks: ClickTracker::new(),
        }
    }

    /// Add a tab to the tabline.
    #[allow(clippy::cast_sign_loss)]
    pub fn add_tab(&mut self, tabnr: c_int, width: c_int, is_selected: bool) {
        let start = self.current_col;
        let end = start + width;

        self.items
            .push(TablineItem::tab(tabnr, start, end, is_selected));
        self.clicks.start_tab_switch(start.max(0) as usize, tabnr);
        self.current_col = end;
    }

    /// Add a close button to the tabline.
    #[allow(clippy::cast_sign_loss)]
    pub fn add_close_button(&mut self, tabnr: c_int, width: c_int) {
        let start = self.current_col;
        let end = start + width;

        self.items.push(TablineItem::close(tabnr, start, end));
        self.clicks.start_tab_close(start.max(0) as usize, tabnr);
        self.current_col = end;
    }

    /// Add fill area to the tabline.
    #[allow(clippy::cast_sign_loss)]
    pub fn add_fill(&mut self, width: c_int) {
        let start = self.current_col;
        let end = start + width;

        self.items.push(TablineItem::fill(start, end));
        self.clicks.end_region(start.max(0) as usize);
        self.current_col = end;
    }

    /// Get the current column position.
    pub const fn current_col(&self) -> c_int {
        self.current_col
    }

    /// Get all items.
    pub fn items(&self) -> &[TablineItem] {
        &self.items
    }

    /// Get the click tracker.
    pub const fn clicks(&self) -> &ClickTracker {
        &self.clicks
    }

    /// Clear the builder.
    pub fn clear(&mut self) {
        self.items.clear();
        self.current_col = 0;
        self.clicks.clear();
    }
}

/// Fill the click definitions array based on click records.
///
/// # Safety
/// - `click_defs` must be a valid pointer with at least `width` elements, or null.
/// - `click_recs` must be a valid null-terminated array of click records.
/// - `buf` must be a valid C string.
///
/// # Panics
/// Panics if `len` exceeds `width` (internal consistency check).
#[no_mangle]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_stl_fill_click_defs(
    click_defs: *mut ClickDefinition,
    click_recs: *const ClickRecord,
    buf: *const c_char,
    width: c_int,
    tabline: bool,
) {
    if click_defs.is_null() || click_recs.is_null() || buf.is_null() {
        return;
    }

    let defs = std::slice::from_raw_parts_mut(click_defs, width as usize);
    let mut col = 0usize;
    let mut len: c_int = 0;
    let mut buf_ptr = buf;
    let mut cur_click_def = ClickDefinition::disabled();

    // Iterate through click records until we find a null start pointer
    let mut i = 0;
    loop {
        let rec = &*click_recs.add(i);
        if rec.start.is_null() {
            break;
        }

        // Calculate width from buf_ptr to rec.start
        let segment_len = rec.start.offset_from(buf_ptr) as c_int;
        len += vim_strnsize(buf_ptr, segment_len);

        assert!(len as usize <= width as usize);

        // Fill columns with current definition
        if col < len as usize {
            while col < len as usize {
                defs[col] = cur_click_def;
                col += 1;
            }
        } else if !cur_click_def.func.is_null() {
            // Free function pointer if we're not using it
            xfree(cur_click_def.func.cast());
        }

        buf_ptr = rec.start;
        cur_click_def = rec.def;

        // For non-tabline, only FuncRun and Disabled are valid
        if !(tabline
            || cur_click_def.click_type == ClickType::Disabled
            || cur_click_def.click_type == ClickType::FuncRun)
        {
            cur_click_def.click_type = ClickType::Disabled;
        }

        i += 1;
    }

    // Fill remaining columns
    if col < width as usize {
        while col < width as usize {
            defs[col] = cur_click_def;
            col += 1;
        }
    } else if !cur_click_def.func.is_null() {
        xfree(cur_click_def.func.cast());
    }
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

    #[test]
    fn test_tabline_item_tab() {
        let item = TablineItem::tab(1, 0, 10, true);
        assert_eq!(item.item_type, TablineItemType::Tab);
        assert_eq!(item.tabnr, 1);
        assert_eq!(item.start_col, 0);
        assert_eq!(item.end_col, 10);
        assert!(item.is_selected);
        assert_eq!(item.width(), 10);
    }

    #[test]
    fn test_tabline_item_close() {
        let item = TablineItem::close(2, 10, 12);
        assert_eq!(item.item_type, TablineItemType::Close);
        assert_eq!(item.tabnr, 2);
        assert_eq!(item.width(), 2);
    }

    #[test]
    fn test_tabline_item_fill() {
        let item = TablineItem::fill(20, 80);
        assert_eq!(item.item_type, TablineItemType::Fill);
        assert_eq!(item.tabnr, 0);
        assert_eq!(item.width(), 60);
    }

    #[test]
    fn test_tabline_builder_basic() {
        let mut builder = TablineBuilder::new();

        builder.add_tab(1, 8, true);
        builder.add_tab(2, 8, false);
        builder.add_close_button(2, 2);
        builder.add_fill(62);

        assert_eq!(builder.items().len(), 4);
        assert_eq!(builder.current_col(), 80);

        let items = builder.items();
        assert_eq!(items[0].tabnr, 1);
        assert!(items[0].is_selected);
        assert_eq!(items[1].tabnr, 2);
        assert!(!items[1].is_selected);
        assert_eq!(items[2].item_type, TablineItemType::Close);
        assert_eq!(items[3].item_type, TablineItemType::Fill);
    }

    #[test]
    fn test_tabline_builder_clicks() {
        let mut builder = TablineBuilder::new();

        builder.add_tab(1, 10, true);
        builder.add_tab(2, 10, false);

        let clicks: Vec<_> = builder.clicks().iter().collect();
        assert_eq!(clicks.len(), 2);
        assert_eq!(clicks[0].1, ClickType::TabSwitch);
        assert_eq!(clicks[0].2, 1); // tabnr
        assert_eq!(clicks[1].2, 2); // tabnr
    }

    #[test]
    fn test_tabline_builder_clear() {
        let mut builder = TablineBuilder::new();
        builder.add_tab(1, 10, true);
        builder.add_fill(70);

        builder.clear();
        assert!(builder.items().is_empty());
        assert_eq!(builder.current_col(), 0);
        assert!(builder.clicks().is_empty());
    }
}
