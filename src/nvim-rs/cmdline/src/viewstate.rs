//! View state management for command-line mode
//!
//! This module provides utilities for saving and restoring window view state,
//! which is essential for incremental search (incsearch) and command preview.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

// =============================================================================
// C Type Definitions
// =============================================================================

/// Opaque handle to a window (win_T*)
pub type WinHandle = *mut std::ffi::c_void;

// =============================================================================
// C Function Declarations
// =============================================================================

extern "C" {
    // Window field accessors
    fn nvim_win_get_curswant(wp: WinHandle) -> c_int;
    fn nvim_win_get_leftcol(wp: WinHandle) -> c_int;
    fn nvim_win_get_skipcol(wp: WinHandle) -> c_int;
    fn nvim_win_get_topline(wp: WinHandle) -> c_int;
    fn nvim_win_get_topfill(wp: WinHandle) -> c_int;
    fn nvim_win_get_botline(wp: WinHandle) -> c_int;
    fn nvim_win_get_empty_rows(wp: WinHandle) -> c_int;

    fn nvim_win_set_curswant(wp: WinHandle, val: c_int);
    fn nvim_win_set_leftcol(wp: WinHandle, val: c_int);
    fn nvim_win_set_skipcol(wp: WinHandle, val: c_int);
    fn nvim_win_set_topline(wp: WinHandle, val: c_int);
    fn nvim_win_set_topfill(wp: WinHandle, val: c_int);
    fn nvim_win_set_botline(wp: WinHandle, val: c_int);
    fn nvim_win_set_empty_rows(wp: WinHandle, val: c_int);

    // Current window accessor
    fn nvim_get_curwin() -> WinHandle;
}

// =============================================================================
// View State
// =============================================================================

/// Saved view state of a window.
///
/// This captures the scroll position and cursor column information
/// needed to restore a window's view after temporary modifications.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(C)]
pub struct ViewState {
    /// Desired cursor column (virtual column)
    pub curswant: i32,
    /// Leftmost column displayed (for horizontal scroll)
    pub leftcol: i32,
    /// Number of columns to skip for 'wrap'
    pub skipcol: i32,
    /// Top line number displayed
    pub topline: i32,
    /// Number of filler lines above topline
    pub topfill: i32,
    /// Bottom line number (approximate)
    pub botline: i32,
    /// Number of empty rows at bottom
    pub empty_rows: i32,
}

impl ViewState {
    /// Create a new empty view state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            curswant: 0,
            leftcol: 0,
            skipcol: 0,
            topline: 1,
            topfill: 0,
            botline: 0,
            empty_rows: 0,
        }
    }

    /// Save the current view state from a window.
    ///
    /// # Safety
    ///
    /// `wp` must be a valid window pointer.
    #[must_use]
    pub unsafe fn from_window(wp: WinHandle) -> Self {
        if wp.is_null() {
            return Self::new();
        }
        Self {
            curswant: nvim_win_get_curswant(wp),
            leftcol: nvim_win_get_leftcol(wp),
            skipcol: nvim_win_get_skipcol(wp),
            topline: nvim_win_get_topline(wp),
            topfill: nvim_win_get_topfill(wp),
            botline: nvim_win_get_botline(wp),
            empty_rows: nvim_win_get_empty_rows(wp),
        }
    }

    /// Restore this view state to a window.
    ///
    /// # Safety
    ///
    /// `wp` must be a valid window pointer.
    pub unsafe fn restore_to_window(&self, wp: WinHandle) {
        if wp.is_null() {
            return;
        }
        nvim_win_set_curswant(wp, self.curswant);
        nvim_win_set_leftcol(wp, self.leftcol);
        nvim_win_set_skipcol(wp, self.skipcol);
        nvim_win_set_topline(wp, self.topline);
        nvim_win_set_topfill(wp, self.topfill);
        nvim_win_set_botline(wp, self.botline);
        nvim_win_set_empty_rows(wp, self.empty_rows);
    }

    /// Save the current view state from the current window.
    ///
    /// # Safety
    ///
    /// Accesses the global `curwin` variable.
    #[must_use]
    pub unsafe fn from_curwin() -> Self {
        Self::from_window(nvim_get_curwin())
    }

    /// Restore this view state to the current window.
    ///
    /// # Safety
    ///
    /// Accesses the global `curwin` variable.
    pub unsafe fn restore_to_curwin(&self) {
        self.restore_to_window(nvim_get_curwin());
    }

    /// Check if this view state differs from another.
    #[must_use]
    pub const fn differs_from(&self, other: &Self) -> bool {
        self.curswant != other.curswant
            || self.leftcol != other.leftcol
            || self.skipcol != other.skipcol
            || self.topline != other.topline
            || self.topfill != other.topfill
    }

    /// Check if scroll position differs.
    #[must_use]
    pub const fn scroll_differs(&self, other: &Self) -> bool {
        self.leftcol != other.leftcol
            || self.skipcol != other.skipcol
            || self.topline != other.topline
            || self.topfill != other.topfill
    }
}

// =============================================================================
// Extended Incsearch State
// =============================================================================

/// Position type (line, column).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(C)]
pub struct Position {
    /// Line number (1-based)
    pub lnum: i32,
    /// Column number (0-based byte offset)
    pub col: i32,
}

impl Position {
    /// Create a new position.
    #[must_use]
    pub const fn new(lnum: i32, col: i32) -> Self {
        Self { lnum, col }
    }

    /// Create an invalid/empty position.
    #[must_use]
    pub const fn invalid() -> Self {
        Self { lnum: 0, col: 0 }
    }

    /// Check if position is valid.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.lnum > 0
    }

    /// Clear the position (make invalid).
    pub fn clear(&mut self) {
        self.lnum = 0;
        self.col = 0;
    }
}

/// Extended incremental search state including view state.
///
/// This extends the basic `IncsearchState` from `search.rs` with
/// view state tracking needed for proper cursor and scroll restoration.
#[derive(Debug, Clone, Copy, Default)]
pub struct IncsearchViewState {
    /// Window handle where this state is valid
    pub winid: i64,
    /// Initial view state (before incsearch started)
    pub init_viewstate: ViewState,
    /// Previous view state (for detecting changes)
    pub old_viewstate: ViewState,
    /// Position where search started
    pub search_start: Position,
    /// Saved cursor position
    pub save_cursor: Position,
    /// Start of current match
    pub match_start: Position,
    /// End of current match
    pub match_end: Position,
    /// Whether incsearch has been done
    pub did_incsearch: bool,
    /// Whether incsearch is postponed
    pub incsearch_postponed: bool,
}

impl IncsearchViewState {
    /// Create a new incsearch view state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            winid: 0,
            init_viewstate: ViewState::new(),
            old_viewstate: ViewState::new(),
            search_start: Position::invalid(),
            save_cursor: Position::invalid(),
            match_start: Position::invalid(),
            match_end: Position::invalid(),
            did_incsearch: false,
            incsearch_postponed: false,
        }
    }

    /// Reset the state.
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Check if a match has been found.
    #[must_use]
    pub const fn has_match(&self) -> bool {
        self.match_start.is_valid()
    }

    /// Clear the match positions.
    pub fn clear_match(&mut self) {
        self.match_start.clear();
        self.match_end.clear();
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Save view state from current window to a ViewState struct.
///
/// # Safety
///
/// `vs` must be a valid pointer to a ViewState.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_save_viewstate(vs: *mut ViewState) {
    if vs.is_null() {
        return;
    }
    *vs = ViewState::from_curwin();
}

/// Restore view state to current window from a ViewState struct.
///
/// # Safety
///
/// `vs` must be a valid pointer to a ViewState.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_restore_viewstate(vs: *const ViewState) {
    if vs.is_null() {
        return;
    }
    (*vs).restore_to_curwin();
}

/// Save view state from a specific window.
///
/// # Safety
///
/// `wp` must be a valid window pointer, `vs` must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_save_viewstate_win(wp: WinHandle, vs: *mut ViewState) {
    if vs.is_null() {
        return;
    }
    *vs = ViewState::from_window(wp);
}

/// Restore view state to a specific window.
///
/// # Safety
///
/// `wp` must be a valid window pointer, `vs` must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_restore_viewstate_win(wp: WinHandle, vs: *const ViewState) {
    if vs.is_null() {
        return;
    }
    (*vs).restore_to_window(wp);
}

/// Check if two view states differ.
///
/// # Safety
///
/// Both pointers must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_viewstate_differs(
    vs1: *const ViewState,
    vs2: *const ViewState,
) -> c_int {
    if vs1.is_null() || vs2.is_null() {
        return 1;
    }
    c_int::from((*vs1).differs_from(&*vs2))
}

/// Check if scroll position differs between two view states.
///
/// # Safety
///
/// Both pointers must be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_viewstate_scroll_differs(
    vs1: *const ViewState,
    vs2: *const ViewState,
) -> c_int {
    if vs1.is_null() || vs2.is_null() {
        return 1;
    }
    c_int::from((*vs1).scroll_differs(&*vs2))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewstate_new() {
        let vs = ViewState::new();
        assert_eq!(vs.curswant, 0);
        assert_eq!(vs.topline, 1);
        assert_eq!(vs.leftcol, 0);
    }

    #[test]
    fn test_viewstate_differs() {
        let vs1 = ViewState::new();
        let mut vs2 = ViewState::new();

        assert!(!vs1.differs_from(&vs2));

        vs2.topline = 10;
        assert!(vs1.differs_from(&vs2));
    }

    #[test]
    fn test_viewstate_scroll_differs() {
        let vs1 = ViewState::new();
        let mut vs2 = ViewState::new();

        assert!(!vs1.scroll_differs(&vs2));

        vs2.leftcol = 5;
        assert!(vs1.scroll_differs(&vs2));

        vs2.leftcol = 0;
        vs2.curswant = 10;
        assert!(!vs1.scroll_differs(&vs2)); // curswant doesn't affect scroll
    }

    #[test]
    fn test_position() {
        let pos = Position::new(1, 0);
        assert!(pos.is_valid());

        let invalid = Position::invalid();
        assert!(!invalid.is_valid());

        let mut pos2 = Position::new(5, 3);
        pos2.clear();
        assert!(!pos2.is_valid());
    }

    #[test]
    fn test_incsearch_view_state() {
        let mut state = IncsearchViewState::new();
        assert!(!state.has_match());

        state.match_start = Position::new(1, 0);
        assert!(state.has_match());

        state.clear_match();
        assert!(!state.has_match());
    }
}
