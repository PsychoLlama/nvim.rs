//! Match state tracking for command-line incremental search and wildmenu
//!
//! This module provides state tracking for incremental search matches
//! and wildmenu selection positions.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

// =============================================================================
// Position Types
// =============================================================================

/// A position in the buffer (line number and column).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BufferPos {
    /// Line number (1-indexed)
    pub lnum: i64,
    /// Column number (0-indexed byte offset)
    pub col: i32,
}

impl BufferPos {
    /// Create a new buffer position.
    #[must_use]
    pub const fn new(lnum: i64, col: i32) -> Self {
        Self { lnum, col }
    }

    /// Check if this position is valid (non-zero line).
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.lnum > 0
    }

    /// Clear the position to invalid state.
    pub fn clear(&mut self) {
        self.lnum = 0;
        self.col = 0;
    }

    /// Check if this position equals another.
    #[must_use]
    pub const fn equals(&self, other: &Self) -> bool {
        self.lnum == other.lnum && self.col == other.col
    }

    /// Compare positions for ordering.
    #[must_use]
    pub const fn cmp(&self, other: &Self) -> i32 {
        if self.lnum < other.lnum {
            -1
        } else if self.lnum > other.lnum {
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

// =============================================================================
// Incsearch Match State
// =============================================================================

/// State for tracking incremental search matches.
#[derive(Debug, Clone, Default)]
pub struct IncsearchMatchState {
    /// Start position of the current match
    pub match_start: BufferPos,
    /// End position of the current match
    pub match_end: BufferPos,
    /// Original cursor position when search started
    pub search_start: BufferPos,
    /// Saved cursor position for restoration
    pub save_cursor: BufferPos,
    /// Whether a match was found
    pub match_found: bool,
    /// Whether highlighting is active
    pub highlight_active: bool,
    /// Number of lines the match spans
    pub match_lines: i32,
    /// End column offset for multi-line matches
    pub match_end_col: i32,
    /// Window handle where this state is valid
    pub window_handle: i32,
}

impl IncsearchMatchState {
    /// Create a new incsearch match state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            match_start: BufferPos::new(0, 0),
            match_end: BufferPos::new(0, 0),
            search_start: BufferPos::new(0, 0),
            save_cursor: BufferPos::new(0, 0),
            match_found: false,
            highlight_active: false,
            match_lines: 0,
            match_end_col: 0,
            window_handle: 0,
        }
    }

    /// Initialize state for a new search.
    pub fn init(&mut self, cursor_lnum: i64, cursor_col: i32, window_handle: i32) {
        self.search_start = BufferPos::new(cursor_lnum, cursor_col);
        self.save_cursor = self.search_start;
        self.match_start.clear();
        self.match_end.clear();
        self.match_found = false;
        self.highlight_active = false;
        self.match_lines = 0;
        self.match_end_col = 0;
        self.window_handle = window_handle;
    }

    /// Reset the state.
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Set the match position.
    pub fn set_match(&mut self, start_lnum: i64, start_col: i32, end_lnum: i64, end_col: i32) {
        self.match_start = BufferPos::new(start_lnum, start_col);
        self.match_end = BufferPos::new(end_lnum, end_col);
        self.match_found = true;
        self.match_lines = (end_lnum - start_lnum) as i32;
        self.match_end_col = end_col;
    }

    /// Clear the match (no match found).
    pub fn clear_match(&mut self) {
        self.match_start.clear();
        self.match_end.clear();
        self.match_found = false;
        self.match_lines = 0;
        self.match_end_col = 0;
    }

    /// Check if match is on a single line.
    #[must_use]
    pub const fn is_single_line_match(&self) -> bool {
        self.match_found && self.match_lines == 0
    }

    /// Check if match spans multiple lines.
    #[must_use]
    pub const fn is_multi_line_match(&self) -> bool {
        self.match_found && self.match_lines > 0
    }

    /// Get the match length for single-line matches.
    #[must_use]
    pub const fn single_line_match_len(&self) -> i32 {
        if self.is_single_line_match() {
            self.match_end.col - self.match_start.col
        } else {
            0
        }
    }

    /// Check if the match is at the search start position.
    #[must_use]
    pub const fn match_at_start(&self) -> bool {
        self.match_found && self.match_start.equals(&self.search_start)
    }
}

// =============================================================================
// Wildmenu Selection State
// =============================================================================

/// State for tracking wildmenu selection.
#[derive(Debug, Clone, Default)]
pub struct WildmenuState {
    /// Index of currently selected item (-1 for no selection)
    pub selected_idx: i32,
    /// Total number of items
    pub num_items: i32,
    /// Index of first visible item in menu
    pub first_visible: i32,
    /// Number of visible items in menu
    pub visible_count: i32,
    /// Whether wildmenu is currently showing
    pub showing: bool,
    /// Whether pum (popup menu) mode is active
    pub pum_active: bool,
    /// Original command line position when wildmenu started
    pub orig_cmdpos: i32,
    /// Original command line content length
    pub orig_cmdlen: i32,
}

impl WildmenuState {
    /// Create a new wildmenu state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            selected_idx: -1,
            num_items: 0,
            first_visible: 0,
            visible_count: 0,
            showing: false,
            pum_active: false,
            orig_cmdpos: 0,
            orig_cmdlen: 0,
        }
    }

    /// Initialize for a new completion.
    pub fn init(&mut self, num_items: i32, cmdpos: i32, cmdlen: i32) {
        self.num_items = num_items;
        self.selected_idx = if num_items > 0 { 0 } else { -1 };
        self.first_visible = 0;
        self.showing = num_items > 0;
        self.orig_cmdpos = cmdpos;
        self.orig_cmdlen = cmdlen;
    }

    /// Reset the state.
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Check if any item is selected.
    #[must_use]
    pub const fn has_selection(&self) -> bool {
        self.selected_idx >= 0 && self.selected_idx < self.num_items
    }

    /// Select next item (with wraparound).
    pub fn select_next(&mut self) {
        if self.num_items == 0 {
            return;
        }
        self.selected_idx = if self.selected_idx >= self.num_items - 1 {
            0 // Wrap to first
        } else {
            self.selected_idx + 1
        };
        self.ensure_visible();
    }

    /// Select previous item (with wraparound).
    pub fn select_prev(&mut self) {
        if self.num_items == 0 {
            return;
        }
        self.selected_idx = if self.selected_idx <= 0 {
            self.num_items - 1 // Wrap to last
        } else {
            self.selected_idx - 1
        };
        self.ensure_visible();
    }

    /// Select first item.
    pub fn select_first(&mut self) {
        if self.num_items > 0 {
            self.selected_idx = 0;
            self.first_visible = 0;
        }
    }

    /// Select last item.
    pub fn select_last(&mut self) {
        if self.num_items > 0 {
            self.selected_idx = self.num_items - 1;
            self.ensure_visible();
        }
    }

    /// Clear selection (no item selected).
    pub fn clear_selection(&mut self) {
        self.selected_idx = -1;
    }

    /// Ensure selected item is visible.
    fn ensure_visible(&mut self) {
        if self.visible_count <= 0 || self.selected_idx < 0 {
            return;
        }

        // Scroll up if needed
        if self.selected_idx < self.first_visible {
            self.first_visible = self.selected_idx;
        }
        // Scroll down if needed
        if self.selected_idx >= self.first_visible + self.visible_count {
            self.first_visible = self.selected_idx - self.visible_count + 1;
        }
    }

    /// Get the relative position of selected item in visible area.
    #[must_use]
    pub const fn selected_visible_idx(&self) -> i32 {
        if self.selected_idx < 0 {
            -1
        } else {
            self.selected_idx - self.first_visible
        }
    }

    /// Check if selected item is at start of list.
    #[must_use]
    pub const fn at_start(&self) -> bool {
        self.selected_idx == 0
    }

    /// Check if selected item is at end of list.
    #[must_use]
    pub const fn at_end(&self) -> bool {
        self.selected_idx == self.num_items - 1
    }
}

// =============================================================================
// Combined Command Line Match State
// =============================================================================

/// Combined state for all match tracking in command line mode.
#[derive(Debug, Clone, Default)]
pub struct CmdlineMatchState {
    /// Incremental search state
    pub incsearch: IncsearchMatchState,
    /// Wildmenu state
    pub wildmenu: WildmenuState,
}

impl CmdlineMatchState {
    /// Create a new combined state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            incsearch: IncsearchMatchState::new(),
            wildmenu: WildmenuState::new(),
        }
    }

    /// Reset all state.
    pub fn reset(&mut self) {
        self.incsearch.reset();
        self.wildmenu.reset();
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Create position comparison result.
#[unsafe(no_mangle)]
pub extern "C" fn rs_bufpos_cmp(lnum1: i64, col1: c_int, lnum2: i64, col2: c_int) -> c_int {
    let pos1 = BufferPos::new(lnum1, col1);
    let pos2 = BufferPos::new(lnum2, col2);
    pos1.cmp(&pos2)
}

/// Check if position is before another.
#[unsafe(no_mangle)]
pub extern "C" fn rs_bufpos_is_before(lnum1: i64, col1: c_int, lnum2: i64, col2: c_int) -> c_int {
    let pos1 = BufferPos::new(lnum1, col1);
    let pos2 = BufferPos::new(lnum2, col2);
    c_int::from(pos1.is_before(&pos2))
}

/// Check if position is after another.
#[unsafe(no_mangle)]
pub extern "C" fn rs_bufpos_is_after(lnum1: i64, col1: c_int, lnum2: i64, col2: c_int) -> c_int {
    let pos1 = BufferPos::new(lnum1, col1);
    let pos2 = BufferPos::new(lnum2, col2);
    c_int::from(pos1.is_after(&pos2))
}

/// Calculate wildmenu next selection index with wraparound.
#[unsafe(no_mangle)]
pub extern "C" fn rs_wildmenu_next_idx(current: c_int, num_items: c_int) -> c_int {
    if num_items <= 0 {
        return -1;
    }
    if current >= num_items - 1 {
        0
    } else {
        current + 1
    }
}

/// Calculate wildmenu previous selection index with wraparound.
#[unsafe(no_mangle)]
pub extern "C" fn rs_wildmenu_prev_idx(current: c_int, num_items: c_int) -> c_int {
    if num_items <= 0 {
        return -1;
    }
    if current <= 0 {
        num_items - 1
    } else {
        current - 1
    }
}

/// Calculate single-line match length.
#[unsafe(no_mangle)]
pub extern "C" fn rs_incsearch_match_len(
    start_col: c_int,
    end_col: c_int,
    match_lines: c_int,
) -> c_int {
    if match_lines == 0 {
        end_col - start_col
    } else {
        0
    }
}

/// Check if match is on single line.
#[unsafe(no_mangle)]
pub extern "C" fn rs_incsearch_is_single_line(match_lines: c_int) -> c_int {
    c_int::from(match_lines == 0)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_pos() {
        let pos1 = BufferPos::new(10, 5);
        let pos2 = BufferPos::new(10, 10);
        let pos3 = BufferPos::new(20, 0);

        assert!(pos1.is_valid());
        assert!(pos1.is_before(&pos2));
        assert!(pos2.is_before(&pos3));
        assert!(pos3.is_after(&pos1));
        assert!(pos1.equals(&BufferPos::new(10, 5)));
    }

    #[test]
    fn test_buffer_pos_clear() {
        let mut pos = BufferPos::new(10, 5);
        assert!(pos.is_valid());
        pos.clear();
        assert!(!pos.is_valid());
    }

    #[test]
    fn test_incsearch_match_state() {
        let mut state = IncsearchMatchState::new();
        assert!(!state.match_found);

        state.init(100, 5, 1);
        assert_eq!(state.search_start.lnum, 100);
        assert_eq!(state.search_start.col, 5);
        assert!(!state.match_found);

        state.set_match(100, 10, 100, 15);
        assert!(state.match_found);
        assert!(state.is_single_line_match());
        assert_eq!(state.single_line_match_len(), 5);

        state.set_match(100, 10, 101, 5);
        assert!(state.is_multi_line_match());
    }

    #[test]
    fn test_wildmenu_state() {
        let mut state = WildmenuState::new();
        assert!(!state.showing);
        assert!(!state.has_selection());

        state.init(5, 10, 20);
        assert!(state.showing);
        assert!(state.has_selection());
        assert_eq!(state.selected_idx, 0);
        assert!(state.at_start());

        state.select_next();
        assert_eq!(state.selected_idx, 1);
        assert!(!state.at_start());

        state.select_last();
        assert_eq!(state.selected_idx, 4);
        assert!(state.at_end());

        state.select_next();
        assert_eq!(state.selected_idx, 0); // Wrapped

        state.select_prev();
        assert_eq!(state.selected_idx, 4); // Wrapped back
    }

    #[test]
    fn test_wildmenu_navigation() {
        assert_eq!(rs_wildmenu_next_idx(0, 5), 1);
        assert_eq!(rs_wildmenu_next_idx(4, 5), 0); // Wrap
        assert_eq!(rs_wildmenu_prev_idx(0, 5), 4); // Wrap
        assert_eq!(rs_wildmenu_prev_idx(3, 5), 2);
        assert_eq!(rs_wildmenu_next_idx(0, 0), -1); // Empty list
    }

    #[test]
    fn test_incsearch_match_len() {
        assert_eq!(rs_incsearch_match_len(5, 10, 0), 5);
        assert_eq!(rs_incsearch_match_len(5, 10, 1), 0); // Multi-line
    }
}
