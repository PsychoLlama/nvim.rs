//! Tag selection interface
//!
//! This module provides Rust implementations for tag selection UI,
//! including tag list display and user selection handling.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::{c_int, c_void};

// =============================================================================
// Constants
// =============================================================================

/// Maximum tag display width
pub const TAG_DISPLAY_MAX_WIDTH: usize = 80;

/// Selection prompt types
pub const TAG_SELECT_NORMAL: c_int = 0;
pub const TAG_SELECT_TSELECT: c_int = 1;
pub const TAG_SELECT_TJUMP: c_int = 2;

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to tag match entry
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TagMatchHandle(*mut c_void);

impl TagMatchHandle {
    /// Check if the handle is null
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Create a null handle
    #[inline]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }
}

// =============================================================================
// Selection State
// =============================================================================

/// State of the tag selection UI
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TagSelectionState {
    /// Total number of matches
    pub num_matches: c_int,
    /// Currently highlighted match (0-based)
    pub current_idx: c_int,
    /// First visible match (for scrolling)
    pub first_visible: c_int,
    /// Number of visible lines
    pub visible_lines: c_int,
    /// Selection type (normal, tselect, tjump)
    pub select_type: c_int,
    /// Whether selection is active
    pub is_active: bool,
}

impl Default for TagSelectionState {
    fn default() -> Self {
        Self {
            num_matches: 0,
            current_idx: 0,
            first_visible: 0,
            visible_lines: 0,
            select_type: TAG_SELECT_NORMAL,
            is_active: false,
        }
    }
}

impl TagSelectionState {
    /// Create a new selection state
    pub fn new(num_matches: c_int, select_type: c_int) -> Self {
        Self {
            num_matches,
            current_idx: 0,
            first_visible: 0,
            visible_lines: 10, // Default
            select_type,
            is_active: num_matches > 1,
        }
    }

    /// Check if there are multiple matches requiring selection
    pub const fn needs_selection(&self) -> bool {
        self.num_matches > 1
    }

    /// Check if there's only one match
    pub const fn single_match(&self) -> bool {
        self.num_matches == 1
    }

    /// Check if there are no matches
    pub const fn no_matches(&self) -> bool {
        self.num_matches == 0
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        if self.current_idx > 0 {
            self.current_idx -= 1;
            if self.current_idx < self.first_visible {
                self.first_visible = self.current_idx;
            }
        }
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        if self.current_idx < self.num_matches - 1 {
            self.current_idx += 1;
            let last_visible = self.first_visible + self.visible_lines - 1;
            if self.current_idx > last_visible {
                self.first_visible = self.current_idx - self.visible_lines + 1;
            }
        }
    }

    /// Move to first match
    pub fn move_first(&mut self) {
        self.current_idx = 0;
        self.first_visible = 0;
    }

    /// Move to last match
    pub fn move_last(&mut self) {
        self.current_idx = self.num_matches - 1;
        let last_visible = self.num_matches.saturating_sub(self.visible_lines);
        self.first_visible = last_visible.max(0);
    }

    /// Page up
    pub fn page_up(&mut self) {
        self.current_idx = (self.current_idx - self.visible_lines).max(0);
        self.first_visible = (self.first_visible - self.visible_lines).max(0);
    }

    /// Page down
    pub fn page_down(&mut self) {
        let max_idx = self.num_matches - 1;
        self.current_idx = (self.current_idx + self.visible_lines).min(max_idx);
        let max_first = (self.num_matches - self.visible_lines).max(0);
        self.first_visible = (self.first_visible + self.visible_lines).min(max_first);
    }

    /// Select by number (1-based input)
    pub fn select_by_number(&mut self, num: c_int) -> bool {
        if num >= 1 && num <= self.num_matches {
            self.current_idx = num - 1;
            true
        } else {
            false
        }
    }

    /// Get selected index (1-based for display)
    pub const fn selected_display_idx(&self) -> c_int {
        self.current_idx + 1
    }

    /// Check if scroll up indicator should show
    pub const fn has_more_above(&self) -> bool {
        self.first_visible > 0
    }

    /// Check if scroll down indicator should show
    pub const fn has_more_below(&self) -> bool {
        self.first_visible + self.visible_lines < self.num_matches
    }
}

// =============================================================================
// Selection Input
// =============================================================================

/// Input action for tag selection
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionAction {
    /// No action
    None = 0,
    /// Confirm selection
    Confirm = 1,
    /// Cancel selection
    Cancel = 2,
    /// Move up
    Up = 3,
    /// Move down
    Down = 4,
    /// Page up
    PageUp = 5,
    /// Page down
    PageDown = 6,
    /// Go to first
    First = 7,
    /// Go to last
    Last = 8,
    /// Enter number
    Number = 9,
}

/// Parse a key input into a selection action
pub const fn parse_selection_key(key: c_int) -> SelectionAction {
    match key {
        // Enter confirms
        13 => SelectionAction::Confirm,
        // Escape cancels
        27 => SelectionAction::Cancel,
        // Arrow keys
        0x100 | 107 => SelectionAction::Up,   // K_UP or 'k'
        0x101 | 106 => SelectionAction::Down, // K_DOWN or 'j'
        // Page keys
        0x102 | 2 => SelectionAction::PageUp,   // K_PAGEUP or Ctrl-B
        0x103 | 6 => SelectionAction::PageDown, // K_PAGEDOWN or Ctrl-F
        // Home/End
        0x104 | 103 => SelectionAction::First, // K_HOME or 'g'
        0x105 | 71 => SelectionAction::Last,   // K_END or 'G'
        // Numbers 1-9
        48..=57 => SelectionAction::Number,
        _ => SelectionAction::None,
    }
}

/// Check if a key is a digit
pub const fn is_digit_key(key: c_int) -> bool {
    key >= 48 && key <= 57
}

/// Get digit value from key
pub const fn key_to_digit(key: c_int) -> c_int {
    if is_digit_key(key) {
        key - 48
    } else {
        -1
    }
}

// =============================================================================
// Selection Result
// =============================================================================

/// Result of tag selection
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SelectionResult {
    /// Selected match index (0-based), -1 if cancelled
    pub selected_idx: c_int,
    /// Whether selection was cancelled
    pub cancelled: bool,
    /// Whether user jumped directly to a number
    pub direct_jump: bool,
}

impl SelectionResult {
    /// Create a cancelled result
    pub const fn cancelled() -> Self {
        Self {
            selected_idx: -1,
            cancelled: true,
            direct_jump: false,
        }
    }

    /// Create a selected result
    pub const fn selected(idx: c_int, direct: bool) -> Self {
        Self {
            selected_idx: idx,
            cancelled: false,
            direct_jump: direct,
        }
    }

    /// Create a single-match result (auto-selected)
    pub const fn single() -> Self {
        Self {
            selected_idx: 0,
            cancelled: false,
            direct_jump: true,
        }
    }
}

// =============================================================================
// Tag Match Display Info
// =============================================================================

/// Information for displaying a tag match
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TagMatchDisplayInfo {
    /// Match index (0-based)
    pub index: c_int,
    /// Priority indicator ('>' for current, ' ' otherwise)
    pub priority_char: u8,
    /// Whether this is the current/highlighted match
    pub is_current: bool,
    /// Whether this match is from a static tag
    pub is_static: bool,
    /// Kind character if available
    pub kind_char: u8,
}

impl Default for TagMatchDisplayInfo {
    fn default() -> Self {
        Self {
            index: 0,
            priority_char: b' ',
            is_current: false,
            is_static: false,
            kind_char: 0,
        }
    }
}

impl TagMatchDisplayInfo {
    /// Create for a specific index
    pub fn new(index: c_int, is_current: bool) -> Self {
        Self {
            index,
            priority_char: if is_current { b'>' } else { b' ' },
            is_current,
            is_static: false,
            kind_char: 0,
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Create selection state
#[no_mangle]
pub extern "C" fn rs_tag_select_init(num_matches: c_int, select_type: c_int) -> TagSelectionState {
    TagSelectionState::new(num_matches, select_type)
}

/// FFI export: Check if selection is needed
#[no_mangle]
pub extern "C" fn rs_tag_select_needs_selection(state: *const TagSelectionState) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*state).needs_selection() })
}

/// FFI export: Move selection up
#[no_mangle]
pub extern "C" fn rs_tag_select_move_up(state: *mut TagSelectionState) {
    if !state.is_null() {
        unsafe { (*state).move_up() };
    }
}

/// FFI export: Move selection down
#[no_mangle]
pub extern "C" fn rs_tag_select_move_down(state: *mut TagSelectionState) {
    if !state.is_null() {
        unsafe { (*state).move_down() };
    }
}

/// FFI export: Get current selection index
#[no_mangle]
pub extern "C" fn rs_tag_select_current_idx(state: *const TagSelectionState) -> c_int {
    if state.is_null() {
        return -1;
    }
    unsafe { (*state).current_idx }
}

/// FFI export: Parse selection key
#[no_mangle]
pub extern "C" fn rs_tag_parse_selection_key(key: c_int) -> SelectionAction {
    parse_selection_key(key)
}

/// FFI export: Check if key is digit
#[no_mangle]
pub extern "C" fn rs_tag_is_digit_key(key: c_int) -> c_int {
    c_int::from(is_digit_key(key))
}

/// FFI export: Get digit from key
#[no_mangle]
pub extern "C" fn rs_tag_key_to_digit(key: c_int) -> c_int {
    key_to_digit(key)
}

/// FFI export: Select by number
#[no_mangle]
pub extern "C" fn rs_tag_select_by_number(state: *mut TagSelectionState, num: c_int) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*state).select_by_number(num) })
}

/// FFI export: Check has more above
#[no_mangle]
pub extern "C" fn rs_tag_select_has_more_above(state: *const TagSelectionState) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*state).has_more_above() })
}

/// FFI export: Check has more below
#[no_mangle]
pub extern "C" fn rs_tag_select_has_more_below(state: *const TagSelectionState) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*state).has_more_below() })
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_state_default() {
        let state = TagSelectionState::default();
        assert_eq!(state.num_matches, 0);
        assert!(!state.is_active);
        assert!(state.no_matches());
    }

    #[test]
    fn test_selection_state_new() {
        let state = TagSelectionState::new(5, TAG_SELECT_TSELECT);
        assert_eq!(state.num_matches, 5);
        assert!(state.needs_selection());
        assert!(!state.single_match());
        assert!(!state.no_matches());
    }

    #[test]
    fn test_selection_navigation() {
        let mut state = TagSelectionState::new(10, TAG_SELECT_NORMAL);
        state.visible_lines = 5;

        // Move down
        state.move_down();
        assert_eq!(state.current_idx, 1);

        // Move up
        state.move_up();
        assert_eq!(state.current_idx, 0);

        // Can't go above 0
        state.move_up();
        assert_eq!(state.current_idx, 0);

        // Move to last
        state.move_last();
        assert_eq!(state.current_idx, 9);

        // Can't go below max
        state.move_down();
        assert_eq!(state.current_idx, 9);

        // Move to first
        state.move_first();
        assert_eq!(state.current_idx, 0);
    }

    #[test]
    fn test_select_by_number() {
        let mut state = TagSelectionState::new(10, TAG_SELECT_NORMAL);

        assert!(state.select_by_number(5));
        assert_eq!(state.current_idx, 4);

        assert!(state.select_by_number(1));
        assert_eq!(state.current_idx, 0);

        assert!(!state.select_by_number(0));
        assert!(!state.select_by_number(11));
    }

    #[test]
    fn test_scroll_indicators() {
        let mut state = TagSelectionState::new(20, TAG_SELECT_NORMAL);
        state.visible_lines = 5;
        state.first_visible = 0;

        assert!(!state.has_more_above());
        assert!(state.has_more_below());

        state.first_visible = 5;
        assert!(state.has_more_above());
        assert!(state.has_more_below());

        state.first_visible = 15;
        assert!(state.has_more_above());
        assert!(!state.has_more_below());
    }

    #[test]
    fn test_parse_selection_key() {
        assert_eq!(parse_selection_key(13), SelectionAction::Confirm);
        assert_eq!(parse_selection_key(27), SelectionAction::Cancel);
        assert_eq!(parse_selection_key(107), SelectionAction::Up);
        assert_eq!(parse_selection_key(106), SelectionAction::Down);
    }

    #[test]
    fn test_digit_keys() {
        assert!(is_digit_key(48)); // '0'
        assert!(is_digit_key(57)); // '9'
        assert!(!is_digit_key(65)); // 'A'

        assert_eq!(key_to_digit(48), 0);
        assert_eq!(key_to_digit(53), 5);
        assert_eq!(key_to_digit(65), -1);
    }

    #[test]
    fn test_selection_result() {
        let cancelled = SelectionResult::cancelled();
        assert!(cancelled.cancelled);
        assert_eq!(cancelled.selected_idx, -1);

        let selected = SelectionResult::selected(3, true);
        assert!(!selected.cancelled);
        assert_eq!(selected.selected_idx, 3);
        assert!(selected.direct_jump);
    }

    #[test]
    fn test_tag_match_display_info() {
        let info = TagMatchDisplayInfo::new(5, true);
        assert_eq!(info.index, 5);
        assert!(info.is_current);
        assert_eq!(info.priority_char, b'>');

        let info = TagMatchDisplayInfo::new(3, false);
        assert!(!info.is_current);
        assert_eq!(info.priority_char, b' ');
    }
}
