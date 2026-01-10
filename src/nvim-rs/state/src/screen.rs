//! Screen state management
//!
//! This module provides screen state infrastructure for Neovim,
//! including dimensions, scrolling, and display state.

use std::ffi::c_int;

// =============================================================================
// Screen Dimensions
// =============================================================================

/// Screen dimensions.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ScreenDimensions {
    /// Number of rows
    pub rows: c_int,
    /// Number of columns
    pub cols: c_int,
    /// Top padding (tabline, etc.)
    pub top_offset: c_int,
    /// Bottom padding (cmdline, etc.)
    pub bottom_offset: c_int,
    /// Left padding
    pub left_offset: c_int,
    /// Right padding
    pub right_offset: c_int,
}

impl ScreenDimensions {
    /// Create new screen dimensions.
    #[must_use]
    pub const fn new(rows: c_int, cols: c_int) -> Self {
        Self {
            rows,
            cols,
            top_offset: 0,
            bottom_offset: 0,
            left_offset: 0,
            right_offset: 0,
        }
    }

    /// Create dimensions with offsets.
    #[must_use]
    pub const fn with_offsets(
        rows: c_int,
        cols: c_int,
        top: c_int,
        bottom: c_int,
        left: c_int,
        right: c_int,
    ) -> Self {
        Self {
            rows,
            cols,
            top_offset: top,
            bottom_offset: bottom,
            left_offset: left,
            right_offset: right,
        }
    }

    /// Get usable rows.
    #[must_use]
    pub const fn usable_rows(&self) -> c_int {
        self.rows - self.top_offset - self.bottom_offset
    }

    /// Get usable columns.
    #[must_use]
    pub const fn usable_cols(&self) -> c_int {
        self.cols - self.left_offset - self.right_offset
    }

    /// Check if dimensions are valid.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.rows > 0 && self.cols > 0 && self.usable_rows() > 0 && self.usable_cols() > 0
    }
}

/// FFI: Create screen dimensions.
#[no_mangle]
pub extern "C" fn rs_screen_dimensions_new(rows: c_int, cols: c_int) -> ScreenDimensions {
    ScreenDimensions::new(rows, cols)
}

/// FFI: Get usable rows.
///
/// # Safety
/// `dims` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_screen_usable_rows(dims: *const ScreenDimensions) -> c_int {
    if dims.is_null() {
        return 0;
    }
    (*dims).usable_rows()
}

/// FFI: Get usable cols.
///
/// # Safety
/// `dims` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_screen_usable_cols(dims: *const ScreenDimensions) -> c_int {
    if dims.is_null() {
        return 0;
    }
    (*dims).usable_cols()
}

// =============================================================================
// Scroll State
// =============================================================================

/// Direction for scrolling.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScrollDirection {
    /// No scroll
    #[default]
    None = 0,
    /// Scroll up (content moves down)
    Up = 1,
    /// Scroll down (content moves up)
    Down = 2,
    /// Scroll left
    Left = 3,
    /// Scroll right
    Right = 4,
}

impl ScrollDirection {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Up,
            2 => Self::Down,
            3 => Self::Left,
            4 => Self::Right,
            _ => Self::None,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if vertical scroll.
    #[must_use]
    pub const fn is_vertical(self) -> bool {
        matches!(self, Self::Up | Self::Down)
    }

    /// Check if horizontal scroll.
    #[must_use]
    pub const fn is_horizontal(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }
}

/// Scroll state.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ScrollState {
    /// Top visible line
    pub topline: i64,
    /// Left column offset
    pub leftcol: c_int,
    /// Scroll binding group
    pub scroll_bind_group: c_int,
    /// Cursor follows scroll
    pub cursor_follows: bool,
    /// Smooth scrolling active
    pub smooth_scroll: bool,
    /// Current scroll offset (for smooth scroll)
    pub scroll_offset: c_int,
}

impl ScrollState {
    /// Create new scroll state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            topline: 1,
            leftcol: 0,
            scroll_bind_group: 0,
            cursor_follows: true,
            smooth_scroll: false,
            scroll_offset: 0,
        }
    }

    /// Scroll vertically.
    pub fn scroll_vertical(&mut self, lines: i64) {
        self.topline = (self.topline + lines).max(1);
    }

    /// Scroll horizontally.
    pub fn scroll_horizontal(&mut self, cols: c_int) {
        self.leftcol = (self.leftcol + cols).max(0);
    }

    /// Reset scroll position.
    pub fn reset(&mut self) {
        self.topline = 1;
        self.leftcol = 0;
        self.scroll_offset = 0;
    }
}

/// FFI: Create scroll state.
#[no_mangle]
pub extern "C" fn rs_scroll_state_new() -> ScrollState {
    ScrollState::new()
}

/// FFI: Scroll vertically.
///
/// # Safety
/// `state` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_scroll_vertical(state: *mut ScrollState, lines: i64) {
    if !state.is_null() {
        (*state).scroll_vertical(lines);
    }
}

/// FFI: Scroll horizontally.
///
/// # Safety
/// `state` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_scroll_horizontal(state: *mut ScrollState, cols: c_int) {
    if !state.is_null() {
        (*state).scroll_horizontal(cols);
    }
}

/// FFI: Reset scroll.
///
/// # Safety
/// `state` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_scroll_reset(state: *mut ScrollState) {
    if !state.is_null() {
        (*state).reset();
    }
}

// =============================================================================
// Redraw State
// =============================================================================

/// Type of redraw needed.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RedrawType {
    /// No redraw needed
    #[default]
    None = 0,
    /// Redraw cursor only
    Cursor = 1,
    /// Redraw current line
    Line = 2,
    /// Redraw visible region
    Region = 3,
    /// Redraw current window
    Window = 4,
    /// Redraw all windows
    AllWindows = 5,
    /// Full screen redraw
    Screen = 6,
    /// Clear and redraw
    Clear = 7,
}

impl RedrawType {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Cursor,
            2 => Self::Line,
            3 => Self::Region,
            4 => Self::Window,
            5 => Self::AllWindows,
            6 => Self::Screen,
            7 => Self::Clear,
            _ => Self::None,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Get the more severe redraw type.
    #[must_use]
    pub const fn max(self, other: Self) -> Self {
        if (self as c_int) > (other as c_int) {
            self
        } else {
            other
        }
    }
}

/// Redraw flags.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct RedrawFlags {
    /// Redraw type needed
    pub redraw_type: c_int,
    /// Status line needs redraw
    pub statusline: bool,
    /// Tab line needs redraw
    pub tabline: bool,
    /// Command line needs redraw
    pub cmdline: bool,
    /// Ruler needs redraw
    pub ruler: bool,
    /// Title needs redraw
    pub title: bool,
    /// Wildmenu needs redraw
    pub wildmenu: bool,
    /// Messages need redraw
    pub messages: bool,
}

impl RedrawFlags {
    /// Create new redraw flags.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            redraw_type: RedrawType::None as c_int,
            statusline: false,
            tabline: false,
            cmdline: false,
            ruler: false,
            title: false,
            wildmenu: false,
            messages: false,
        }
    }

    /// Request redraw of given type.
    pub fn request(&mut self, redraw_type: RedrawType) {
        let current = RedrawType::from_c_int(self.redraw_type);
        self.redraw_type = current.max(redraw_type) as c_int;
    }

    /// Check if any redraw is needed.
    #[must_use]
    pub const fn needs_redraw(&self) -> bool {
        self.redraw_type > 0
            || self.statusline
            || self.tabline
            || self.cmdline
            || self.ruler
            || self.title
            || self.wildmenu
            || self.messages
    }

    /// Clear all redraw flags.
    pub fn clear(&mut self) {
        self.redraw_type = RedrawType::None as c_int;
        self.statusline = false;
        self.tabline = false;
        self.cmdline = false;
        self.ruler = false;
        self.title = false;
        self.wildmenu = false;
        self.messages = false;
    }
}

/// FFI: Create redraw flags.
#[no_mangle]
pub extern "C" fn rs_redraw_flags_new() -> RedrawFlags {
    RedrawFlags::new()
}

/// FFI: Request redraw.
///
/// # Safety
/// `flags` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_redraw_request(flags: *mut RedrawFlags, redraw_type: c_int) {
    if !flags.is_null() {
        (*flags).request(RedrawType::from_c_int(redraw_type));
    }
}

/// FFI: Check if redraw needed.
///
/// # Safety
/// `flags` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_redraw_needs_redraw(flags: *const RedrawFlags) -> c_int {
    if flags.is_null() {
        return 0;
    }
    c_int::from((*flags).needs_redraw())
}

/// FFI: Clear redraw flags.
///
/// # Safety
/// `flags` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_redraw_clear(flags: *mut RedrawFlags) {
    if !flags.is_null() {
        (*flags).clear();
    }
}

// =============================================================================
// Screen State
// =============================================================================

/// Complete screen state.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ScreenState {
    /// Screen dimensions
    pub dimensions: ScreenDimensions,
    /// Redraw flags
    pub redraw: RedrawFlags,
    /// Screen is valid
    pub valid: bool,
    /// Screen is busy (no updates)
    pub busy: bool,
    /// Screen is suspended
    pub suspended: bool,
    /// Using external UI
    pub external_ui: bool,
    /// Grid UI active
    pub grid_ui: bool,
    /// Number of attached UIs
    pub attached_uis: c_int,
}

impl ScreenState {
    /// Create new screen state.
    #[must_use]
    pub const fn new(rows: c_int, cols: c_int) -> Self {
        Self {
            dimensions: ScreenDimensions::new(rows, cols),
            redraw: RedrawFlags::new(),
            valid: false,
            busy: false,
            suspended: false,
            external_ui: false,
            grid_ui: false,
            attached_uis: 0,
        }
    }

    /// Resize screen.
    pub fn resize(&mut self, rows: c_int, cols: c_int) {
        if rows != self.dimensions.rows || cols != self.dimensions.cols {
            self.dimensions.rows = rows;
            self.dimensions.cols = cols;
            self.redraw.request(RedrawType::Screen);
            self.valid = false;
        }
    }

    /// Validate screen.
    pub fn validate(&mut self) {
        self.valid = true;
        self.redraw.clear();
    }

    /// Invalidate screen.
    pub fn invalidate(&mut self) {
        self.valid = false;
        self.redraw.request(RedrawType::Screen);
    }
}

/// FFI: Create screen state.
#[no_mangle]
pub extern "C" fn rs_screen_state_new(rows: c_int, cols: c_int) -> ScreenState {
    ScreenState::new(rows, cols)
}

/// FFI: Resize screen.
///
/// # Safety
/// `state` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_screen_resize(state: *mut ScreenState, rows: c_int, cols: c_int) {
    if !state.is_null() {
        (*state).resize(rows, cols);
    }
}

/// FFI: Validate screen.
///
/// # Safety
/// `state` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_screen_validate(state: *mut ScreenState) {
    if !state.is_null() {
        (*state).validate();
    }
}

/// FFI: Invalidate screen.
///
/// # Safety
/// `state` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_screen_invalidate(state: *mut ScreenState) {
    if !state.is_null() {
        (*state).invalidate();
    }
}

/// FFI: Check if screen valid.
///
/// # Safety
/// `state` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_screen_is_valid(state: *const ScreenState) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from((*state).valid)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_dimensions() {
        let dims = ScreenDimensions::new(24, 80);
        assert!(dims.is_valid());
        assert_eq!(dims.usable_rows(), 24);
        assert_eq!(dims.usable_cols(), 80);

        let dims_offset = ScreenDimensions::with_offsets(24, 80, 1, 1, 0, 0);
        assert_eq!(dims_offset.usable_rows(), 22);
        assert_eq!(dims_offset.usable_cols(), 80);
    }

    #[test]
    fn test_scroll_direction() {
        assert!(ScrollDirection::Up.is_vertical());
        assert!(ScrollDirection::Down.is_vertical());
        assert!(!ScrollDirection::Up.is_horizontal());
        assert!(ScrollDirection::Left.is_horizontal());
        assert!(ScrollDirection::Right.is_horizontal());
    }

    #[test]
    fn test_scroll_state() {
        let mut state = ScrollState::new();
        assert_eq!(state.topline, 1);
        assert_eq!(state.leftcol, 0);

        state.scroll_vertical(10);
        assert_eq!(state.topline, 11);

        state.scroll_horizontal(5);
        assert_eq!(state.leftcol, 5);

        // Can't scroll past start
        state.scroll_vertical(-100);
        assert_eq!(state.topline, 1);

        state.scroll_horizontal(-100);
        assert_eq!(state.leftcol, 0);
    }

    #[test]
    fn test_redraw_type() {
        let none = RedrawType::None;
        let line = RedrawType::Line;
        let screen = RedrawType::Screen;

        assert_eq!(none.max(line), line);
        assert_eq!(line.max(screen), screen);
        assert_eq!(screen.max(none), screen);
    }

    #[test]
    fn test_redraw_flags() {
        let mut flags = RedrawFlags::new();
        assert!(!flags.needs_redraw());

        flags.request(RedrawType::Cursor);
        assert!(flags.needs_redraw());
        assert_eq!(flags.redraw_type, RedrawType::Cursor as c_int);

        flags.request(RedrawType::Screen);
        assert_eq!(flags.redraw_type, RedrawType::Screen as c_int);

        // Should not downgrade
        flags.request(RedrawType::Cursor);
        assert_eq!(flags.redraw_type, RedrawType::Screen as c_int);

        flags.clear();
        assert!(!flags.needs_redraw());
    }

    #[test]
    fn test_screen_state() {
        let mut state = ScreenState::new(24, 80);
        assert!(!state.valid);

        state.validate();
        assert!(state.valid);

        state.resize(30, 100);
        assert!(!state.valid);
        assert_eq!(state.dimensions.rows, 30);
        assert_eq!(state.dimensions.cols, 100);

        state.invalidate();
        assert!(!state.valid);
    }
}
