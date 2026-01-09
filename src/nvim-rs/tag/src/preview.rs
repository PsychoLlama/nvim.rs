//! Tag preview functionality
//!
//! This module provides Rust implementations for tag preview operations,
//! including preview window management and preview tag state.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::derivable_impls)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::use_self)]
#![allow(clippy::manual_clamp)]

use std::ffi::{c_int, c_void};

// =============================================================================
// Constants
// =============================================================================

/// Default preview window height
pub const PREVIEW_HEIGHT_DEFAULT: c_int = 12;

/// Minimum preview window height
pub const PREVIEW_HEIGHT_MIN: c_int = 1;

/// Maximum preview window height
pub const PREVIEW_HEIGHT_MAX: c_int = 50;

// =============================================================================
// Opaque Handles
// =============================================================================

/// Line number type
type LinenrT = i32;

/// Opaque handle to window
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WinHandle(*mut c_void);

impl WinHandle {
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

/// Opaque handle to buffer
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BufHandle(*mut c_void);

impl BufHandle {
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
// Preview Tag Entry
// =============================================================================

/// Preview tag entry state
///
/// This mirrors the `ptag_entry` static in tag.c which stores
/// preview tag information separately from the main tag stack.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PreviewTagEntry {
    /// Current match index
    pub cur_match: c_int,
    /// File number where tag was invoked
    pub cur_fnum: c_int,
    /// Whether preview tag is set
    pub is_set: bool,
}

impl Default for PreviewTagEntry {
    fn default() -> Self {
        Self {
            cur_match: 0,
            cur_fnum: 0,
            is_set: false,
        }
    }
}

impl PreviewTagEntry {
    /// Create a new preview entry
    pub const fn new(cur_match: c_int, cur_fnum: c_int) -> Self {
        Self {
            cur_match,
            cur_fnum,
            is_set: true,
        }
    }

    /// Clear the preview entry
    pub fn clear(&mut self) {
        self.cur_match = 0;
        self.cur_fnum = 0;
        self.is_set = false;
    }

    /// Update match info
    pub fn update(&mut self, cur_match: c_int, cur_fnum: c_int) {
        self.cur_match = cur_match;
        self.cur_fnum = cur_fnum;
        self.is_set = true;
    }
}

// =============================================================================
// Preview Window State
// =============================================================================

/// State of preview window
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewWindowState {
    /// No preview window
    None = 0,
    /// Preview window exists but hidden
    Hidden = 1,
    /// Preview window is visible
    Visible = 2,
    /// Preview window is focused
    Focused = 3,
}

impl Default for PreviewWindowState {
    fn default() -> Self {
        Self::None
    }
}

/// Preview window info
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PreviewWindowInfo {
    /// Window handle (null if none)
    pub window: WinHandle,
    /// Buffer handle (null if none)
    pub buffer: BufHandle,
    /// Window height
    pub height: c_int,
    /// Current line in preview
    pub line: LinenrT,
    /// State of the preview window
    pub state: PreviewWindowState,
}

impl Default for PreviewWindowInfo {
    fn default() -> Self {
        Self {
            window: WinHandle::null(),
            buffer: BufHandle::null(),
            height: PREVIEW_HEIGHT_DEFAULT,
            line: 1,
            state: PreviewWindowState::None,
        }
    }
}

impl PreviewWindowInfo {
    /// Check if preview window exists
    pub const fn exists(&self) -> bool {
        !self.window.is_null()
    }

    /// Check if preview is visible
    pub const fn is_visible(&self) -> bool {
        matches!(
            self.state,
            PreviewWindowState::Visible | PreviewWindowState::Focused
        )
    }

    /// Check if preview is focused
    pub const fn is_focused(&self) -> bool {
        matches!(self.state, PreviewWindowState::Focused)
    }
}

// =============================================================================
// Preview Request
// =============================================================================

/// Request to show tag preview
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PreviewRequest {
    /// Line number to show
    pub line: LinenrT,
    /// Column to position cursor
    pub col: c_int,
    /// Height for preview window (0 for default)
    pub height: c_int,
    /// Whether to center the line in view
    pub center: bool,
    /// Whether to highlight the line
    pub highlight: bool,
}

impl Default for PreviewRequest {
    fn default() -> Self {
        Self {
            line: 1,
            col: 0,
            height: 0,
            center: true,
            highlight: true,
        }
    }
}

impl PreviewRequest {
    /// Create a preview request for a specific line
    pub const fn at_line(line: LinenrT) -> Self {
        Self {
            line,
            col: 0,
            height: 0,
            center: true,
            highlight: true,
        }
    }

    /// Create a preview request with specific position
    pub const fn at_position(line: LinenrT, col: c_int) -> Self {
        Self {
            line,
            col,
            height: 0,
            center: true,
            highlight: true,
        }
    }

    /// Get effective height (default if 0)
    pub const fn effective_height(&self) -> c_int {
        if self.height > 0 {
            self.height
        } else {
            PREVIEW_HEIGHT_DEFAULT
        }
    }
}

// =============================================================================
// Preview Result
// =============================================================================

/// Result of a preview operation
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewResult {
    /// Preview shown successfully
    Success = 0,
    /// Failed to open preview window
    WindowFailed = 1,
    /// File not found
    FileNotFound = 2,
    /// Invalid line number
    InvalidLine = 3,
    /// Preview is disabled
    Disabled = 4,
    /// Buffer error
    BufferError = 5,
}

impl PreviewResult {
    /// Check if operation was successful
    pub const fn is_success(&self) -> bool {
        matches!(self, PreviewResult::Success)
    }
}

// =============================================================================
// Preview Mode
// =============================================================================

/// Preview mode flags
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PreviewMode {
    flags: c_int,
}

pub const PREVIEW_MODE_SPLIT: c_int = 0x01;
pub const PREVIEW_MODE_POPUP: c_int = 0x02;
pub const PREVIEW_MODE_FLOAT: c_int = 0x04;

impl PreviewMode {
    /// Create default mode (split)
    pub const fn default_mode() -> Self {
        Self {
            flags: PREVIEW_MODE_SPLIT,
        }
    }

    /// Create from raw flags
    pub const fn from_raw(flags: c_int) -> Self {
        Self { flags }
    }

    /// Get raw flags
    pub const fn as_raw(self) -> c_int {
        self.flags
    }

    /// Check if using split mode
    pub const fn is_split(self) -> bool {
        (self.flags & PREVIEW_MODE_SPLIT) != 0
    }

    /// Check if using popup mode
    pub const fn is_popup(self) -> bool {
        (self.flags & PREVIEW_MODE_POPUP) != 0
    }

    /// Check if using floating window mode
    pub const fn is_float(self) -> bool {
        (self.flags & PREVIEW_MODE_FLOAT) != 0
    }
}

// =============================================================================
// Global Preview State
// =============================================================================

/// Global preview state (for g_do_tagpreview)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GlobalPreviewState {
    /// Height for preview (0 = no preview, >0 = preview height)
    pub height: c_int,
    /// Preview tag entry
    pub entry: PreviewTagEntry,
}

impl Default for GlobalPreviewState {
    fn default() -> Self {
        Self {
            height: 0,
            entry: PreviewTagEntry::default(),
        }
    }
}

impl GlobalPreviewState {
    /// Check if preview is active
    pub const fn is_active(&self) -> bool {
        self.height != 0
    }

    /// Enable preview with height
    pub fn enable(&mut self, height: c_int) {
        self.height = height.max(PREVIEW_HEIGHT_MIN).min(PREVIEW_HEIGHT_MAX);
    }

    /// Disable preview
    pub fn disable(&mut self) {
        self.height = 0;
    }

    /// Check if entry matches given tag name hash
    pub const fn entry_matches(&self, cur_match: c_int, cur_fnum: c_int) -> bool {
        self.entry.is_set && self.entry.cur_match == cur_match && self.entry.cur_fnum == cur_fnum
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Create preview tag entry
#[no_mangle]
pub extern "C" fn rs_tag_preview_entry_new(cur_match: c_int, cur_fnum: c_int) -> PreviewTagEntry {
    PreviewTagEntry::new(cur_match, cur_fnum)
}

/// FFI export: Check if preview entry is set
#[no_mangle]
pub extern "C" fn rs_tag_preview_entry_is_set(entry: *const PreviewTagEntry) -> c_int {
    if entry.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*entry).is_set })
}

/// FFI export: Create preview request
#[no_mangle]
pub extern "C" fn rs_tag_preview_request_at_line(line: LinenrT) -> PreviewRequest {
    PreviewRequest::at_line(line)
}

/// FFI export: Get effective preview height
#[no_mangle]
pub extern "C" fn rs_tag_preview_effective_height(request: *const PreviewRequest) -> c_int {
    if request.is_null() {
        return PREVIEW_HEIGHT_DEFAULT;
    }
    unsafe { (*request).effective_height() }
}

/// FFI export: Check if preview window exists
#[no_mangle]
pub extern "C" fn rs_tag_preview_window_exists(info: *const PreviewWindowInfo) -> c_int {
    if info.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*info).exists() })
}

/// FFI export: Check if preview is visible
#[no_mangle]
pub extern "C" fn rs_tag_preview_is_visible(info: *const PreviewWindowInfo) -> c_int {
    if info.is_null() {
        return 0;
    }
    c_int::from(unsafe { (*info).is_visible() })
}

/// FFI export: Check if global preview is active
#[no_mangle]
pub extern "C" fn rs_tag_preview_is_active(height: c_int) -> c_int {
    c_int::from(height != 0)
}

/// FFI export: Clamp preview height
#[no_mangle]
pub extern "C" fn rs_tag_preview_clamp_height(height: c_int) -> c_int {
    height.max(PREVIEW_HEIGHT_MIN).min(PREVIEW_HEIGHT_MAX)
}

/// FFI export: Check preview mode
#[no_mangle]
pub extern "C" fn rs_tag_preview_mode_is_split(flags: c_int) -> c_int {
    c_int::from(PreviewMode::from_raw(flags).is_split())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preview_tag_entry() {
        let mut entry = PreviewTagEntry::default();
        assert!(!entry.is_set);

        entry.update(5, 10);
        assert!(entry.is_set);
        assert_eq!(entry.cur_match, 5);
        assert_eq!(entry.cur_fnum, 10);

        entry.clear();
        assert!(!entry.is_set);
    }

    #[test]
    fn test_preview_window_state() {
        let info = PreviewWindowInfo::default();
        assert!(!info.exists());
        assert!(!info.is_visible());
        assert!(!info.is_focused());
        assert_eq!(info.state, PreviewWindowState::None);
    }

    #[test]
    fn test_preview_request() {
        let req = PreviewRequest::at_line(100);
        assert_eq!(req.line, 100);
        assert!(req.center);
        assert_eq!(req.effective_height(), PREVIEW_HEIGHT_DEFAULT);

        let req = PreviewRequest {
            height: 20,
            ..Default::default()
        };
        assert_eq!(req.effective_height(), 20);
    }

    #[test]
    fn test_preview_result() {
        assert!(PreviewResult::Success.is_success());
        assert!(!PreviewResult::WindowFailed.is_success());
        assert!(!PreviewResult::FileNotFound.is_success());
    }

    #[test]
    fn test_preview_mode() {
        let mode = PreviewMode::default_mode();
        assert!(mode.is_split());
        assert!(!mode.is_popup());
        assert!(!mode.is_float());

        let mode = PreviewMode::from_raw(PREVIEW_MODE_POPUP);
        assert!(!mode.is_split());
        assert!(mode.is_popup());
    }

    #[test]
    fn test_global_preview_state() {
        let mut state = GlobalPreviewState::default();
        assert!(!state.is_active());

        state.enable(15);
        assert!(state.is_active());
        assert_eq!(state.height, 15);

        // Test clamping
        state.enable(100);
        assert_eq!(state.height, PREVIEW_HEIGHT_MAX);

        state.enable(-5);
        assert_eq!(state.height, PREVIEW_HEIGHT_MIN);

        state.disable();
        assert!(!state.is_active());
    }

    #[test]
    fn test_entry_matches() {
        let mut state = GlobalPreviewState::default();
        state.entry.update(5, 10);

        assert!(state.entry_matches(5, 10));
        assert!(!state.entry_matches(5, 11));
        assert!(!state.entry_matches(6, 10));
    }

    #[test]
    fn test_ffi_functions() {
        let entry = rs_tag_preview_entry_new(3, 7);
        assert!(entry.is_set);
        assert_eq!(entry.cur_match, 3);

        assert_eq!(rs_tag_preview_is_active(0), 0);
        assert_eq!(rs_tag_preview_is_active(10), 1);

        assert_eq!(rs_tag_preview_clamp_height(100), PREVIEW_HEIGHT_MAX);
        assert_eq!(rs_tag_preview_clamp_height(0), PREVIEW_HEIGHT_MIN);
        assert_eq!(rs_tag_preview_clamp_height(15), 15);
    }
}
