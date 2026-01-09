//! UI core types and functions for Neovim
//!
//! This crate provides Rust wrappers for Neovim's UI infrastructure,
//! including UI extensions, RemoteUI state, and cursor management.
//!
//! # UI Extensions
//!
//! Neovim supports various UI extensions that clients can request:
//! - Cmdline: External cmdline rendering
//! - Popupmenu: External popup menu
//! - Tabline: External tabline
//! - Wildmenu: External wildmenu
//! - Messages: External messages
//! - Linegrid: Per-line grid updates
//! - Multigrid: Multiple grid support
//! - HlState: Highlight state tracking
//! - TermColors: Terminal color support
//!
//! # Design
//!
//! This crate provides:
//! - UIExtension enum for UI capabilities
//! - RemoteUI opaque handle for UI client state
//! - Cursor position tracking
//! - UI state query functions

#![allow(unsafe_code)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::doc_markdown)]

use std::ffi::{c_int, c_void};

// =============================================================================
// Constants
// =============================================================================

/// Maximum number of attached UIs
pub const MAX_UI_COUNT: usize = 16;

/// Buffer size for pending msgpack data in UI
pub const UI_BUF_SIZE: usize = 4096; // ARENA_BLOCK_SIZE

/// Guaranteed size for each new event
pub const EVENT_BUF_SIZE: usize = 256;

// =============================================================================
// UI Extensions
// =============================================================================

/// UI extension capabilities
///
/// These correspond to the `UIExtension` enum in C's `ui_defs.h`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UIExtension {
    /// External command-line rendering
    Cmdline = 0,
    /// External popup menu
    Popupmenu = 1,
    /// External tabline
    Tabline = 2,
    /// External wildmenu
    Wildmenu = 3,
    /// External messages
    Messages = 4,
    /// Per-line grid updates (boundary for global count)
    Linegrid = 5,
    /// Multiple grid support
    Multigrid = 6,
    /// Highlight state tracking
    HlState = 7,
    /// Terminal color support
    TermColors = 8,
    /// Float debug mode
    FloatDebug = 9,
}

impl UIExtension {
    /// Total number of UI extensions
    pub const COUNT: usize = 10;

    /// Number of "global" extensions (before Linegrid)
    pub const GLOBAL_COUNT: usize = 5;

    /// Convert from C int
    #[must_use]
    pub fn from_c_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::Cmdline),
            1 => Some(Self::Popupmenu),
            2 => Some(Self::Tabline),
            3 => Some(Self::Wildmenu),
            4 => Some(Self::Messages),
            5 => Some(Self::Linegrid),
            6 => Some(Self::Multigrid),
            7 => Some(Self::HlState),
            8 => Some(Self::TermColors),
            9 => Some(Self::FloatDebug),
            _ => None,
        }
    }

    /// Check if this is a global extension (affects all UIs)
    #[must_use]
    pub const fn is_global(self) -> bool {
        (self as usize) < Self::GLOBAL_COUNT
    }

    /// Get the extension name
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Cmdline => "ext_cmdline",
            Self::Popupmenu => "ext_popupmenu",
            Self::Tabline => "ext_tabline",
            Self::Wildmenu => "ext_wildmenu",
            Self::Messages => "ext_messages",
            Self::Linegrid => "ext_linegrid",
            Self::Multigrid => "ext_multigrid",
            Self::HlState => "ext_hlstate",
            Self::TermColors => "ext_termcolors",
            Self::FloatDebug => "_debug_float",
        }
    }
}

/// Line flags for grid_line events
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineFlags(pub c_int);

impl LineFlags {
    /// Line wraps to next line
    pub const WRAP: Self = Self(1);
    /// Line content is invalid (needs redraw)
    pub const INVALID: Self = Self(2);

    /// Check if wrap flag is set
    #[must_use]
    pub const fn is_wrap(self) -> bool {
        (self.0 & Self::WRAP.0) != 0
    }

    /// Check if invalid flag is set
    #[must_use]
    pub const fn is_invalid(self) -> bool {
        (self.0 & Self::INVALID.0) != 0
    }
}

// =============================================================================
// Opaque Handle Types
// =============================================================================

/// Opaque handle to RemoteUI
///
/// RemoteUI represents a connected UI client's state.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteUIHandle(*mut c_void);

impl RemoteUIHandle {
    /// Create a null handle
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Check if handle is null
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Get the raw pointer
    #[must_use]
    pub const fn as_ptr(self) -> *mut c_void {
        self.0
    }
}

/// Opaque handle to UIClientHandler
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UIClientHandlerHandle(*mut c_void);

impl UIClientHandlerHandle {
    /// Create a null handle
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Check if handle is null
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// =============================================================================
// C Accessor Functions
// =============================================================================

extern "C" {
    // UI state accessors
    fn nvim_get_ui_ext(ext: c_int) -> c_int;
    fn nvim_ui_active() -> usize;

    // RemoteUI accessors
    fn nvim_remote_ui_get_width(ui: RemoteUIHandle) -> c_int;
    fn nvim_remote_ui_get_height(ui: RemoteUIHandle) -> c_int;
    fn nvim_remote_ui_is_rgb(ui: RemoteUIHandle) -> c_int;
    fn nvim_remote_ui_is_override(ui: RemoteUIHandle) -> c_int;
    fn nvim_remote_ui_is_composed(ui: RemoteUIHandle) -> c_int;
    fn nvim_remote_ui_get_ext(ui: RemoteUIHandle, ext: c_int) -> c_int;
    fn nvim_remote_ui_get_channel_id(ui: RemoteUIHandle) -> u64;
    fn nvim_remote_ui_is_stdin_tty(ui: RemoteUIHandle) -> c_int;
    fn nvim_remote_ui_is_stdout_tty(ui: RemoteUIHandle) -> c_int;
    fn nvim_remote_ui_get_term_colors(ui: RemoteUIHandle) -> c_int;
    fn nvim_remote_ui_get_pum_nlines(ui: RemoteUIHandle) -> c_int;
    fn nvim_remote_ui_is_pum_pos(ui: RemoteUIHandle) -> c_int;
    fn nvim_remote_ui_get_cursor_row(ui: RemoteUIHandle) -> i64;
    fn nvim_remote_ui_get_cursor_col(ui: RemoteUIHandle) -> i64;
    fn nvim_remote_ui_get_hl_id(ui: RemoteUIHandle) -> c_int;
    fn nvim_remote_ui_has_incomplete_event(ui: RemoteUIHandle) -> c_int;
}

// =============================================================================
// UI State Query Functions
// =============================================================================

// Note: rs_ui_has, rs_ui_current_row, rs_ui_current_col are defined in nvim-grid crate
// to avoid duplicate symbol errors.

/// Get the number of active UIs
///
/// # Safety
///
/// This function accesses global UI state.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_active() -> usize {
    nvim_ui_active()
}

/// Check if any UI extension is active
///
/// # Safety
///
/// This function accesses global UI state.
pub unsafe fn ui_has_ext(ext: UIExtension) -> bool {
    nvim_get_ui_ext(ext as c_int) != 0
}

// =============================================================================
// RemoteUI Wrapper Functions
// =============================================================================

/// Get RemoteUI width
///
/// # Safety
///
/// `ui` must be a valid RemoteUI handle
#[no_mangle]
pub unsafe extern "C" fn rs_remote_ui_get_width(ui: RemoteUIHandle) -> c_int {
    if ui.is_null() {
        return 0;
    }
    nvim_remote_ui_get_width(ui)
}

/// Get RemoteUI height
///
/// # Safety
///
/// `ui` must be a valid RemoteUI handle
#[no_mangle]
pub unsafe extern "C" fn rs_remote_ui_get_height(ui: RemoteUIHandle) -> c_int {
    if ui.is_null() {
        return 0;
    }
    nvim_remote_ui_get_height(ui)
}

/// Check if RemoteUI uses RGB colors
///
/// # Safety
///
/// `ui` must be a valid RemoteUI handle
#[no_mangle]
pub unsafe extern "C" fn rs_remote_ui_is_rgb(ui: RemoteUIHandle) -> c_int {
    if ui.is_null() {
        return 0;
    }
    nvim_remote_ui_is_rgb(ui)
}

/// Check if RemoteUI has override mode
///
/// # Safety
///
/// `ui` must be a valid RemoteUI handle
#[no_mangle]
pub unsafe extern "C" fn rs_remote_ui_is_override(ui: RemoteUIHandle) -> c_int {
    if ui.is_null() {
        return 0;
    }
    nvim_remote_ui_is_override(ui)
}

/// Check if RemoteUI is composed
///
/// # Safety
///
/// `ui` must be a valid RemoteUI handle
#[no_mangle]
pub unsafe extern "C" fn rs_remote_ui_is_composed(ui: RemoteUIHandle) -> c_int {
    if ui.is_null() {
        return 0;
    }
    nvim_remote_ui_is_composed(ui)
}

/// Get RemoteUI extension support
///
/// # Safety
///
/// `ui` must be a valid RemoteUI handle
#[no_mangle]
pub unsafe extern "C" fn rs_remote_ui_get_ext(ui: RemoteUIHandle, ext: c_int) -> c_int {
    if ui.is_null() {
        return 0;
    }
    nvim_remote_ui_get_ext(ui, ext)
}

/// Get RemoteUI channel ID
///
/// # Safety
///
/// `ui` must be a valid RemoteUI handle
#[no_mangle]
pub unsafe extern "C" fn rs_remote_ui_get_channel_id(ui: RemoteUIHandle) -> u64 {
    if ui.is_null() {
        return 0;
    }
    nvim_remote_ui_get_channel_id(ui)
}

/// Check if RemoteUI has stdin TTY
///
/// # Safety
///
/// `ui` must be a valid RemoteUI handle
#[no_mangle]
pub unsafe extern "C" fn rs_remote_ui_is_stdin_tty(ui: RemoteUIHandle) -> c_int {
    if ui.is_null() {
        return 0;
    }
    nvim_remote_ui_is_stdin_tty(ui)
}

/// Check if RemoteUI has stdout TTY
///
/// # Safety
///
/// `ui` must be a valid RemoteUI handle
#[no_mangle]
pub unsafe extern "C" fn rs_remote_ui_is_stdout_tty(ui: RemoteUIHandle) -> c_int {
    if ui.is_null() {
        return 0;
    }
    nvim_remote_ui_is_stdout_tty(ui)
}

/// Get terminal colors count
///
/// # Safety
///
/// `ui` must be a valid RemoteUI handle
#[no_mangle]
pub unsafe extern "C" fn rs_remote_ui_get_term_colors(ui: RemoteUIHandle) -> c_int {
    if ui.is_null() {
        return 0;
    }
    nvim_remote_ui_get_term_colors(ui)
}

/// Get popup menu lines count
///
/// # Safety
///
/// `ui` must be a valid RemoteUI handle
#[no_mangle]
pub unsafe extern "C" fn rs_remote_ui_get_pum_nlines(ui: RemoteUIHandle) -> c_int {
    if ui.is_null() {
        return 0;
    }
    nvim_remote_ui_get_pum_nlines(ui)
}

/// Check if RemoteUI reports pum position
///
/// # Safety
///
/// `ui` must be a valid RemoteUI handle
#[no_mangle]
pub unsafe extern "C" fn rs_remote_ui_is_pum_pos(ui: RemoteUIHandle) -> c_int {
    if ui.is_null() {
        return 0;
    }
    nvim_remote_ui_is_pum_pos(ui)
}

/// Get RemoteUI cursor row
///
/// # Safety
///
/// `ui` must be a valid RemoteUI handle
#[no_mangle]
pub unsafe extern "C" fn rs_remote_ui_get_cursor_row(ui: RemoteUIHandle) -> i64 {
    if ui.is_null() {
        return 0;
    }
    nvim_remote_ui_get_cursor_row(ui)
}

/// Get RemoteUI cursor column
///
/// # Safety
///
/// `ui` must be a valid RemoteUI handle
#[no_mangle]
pub unsafe extern "C" fn rs_remote_ui_get_cursor_col(ui: RemoteUIHandle) -> i64 {
    if ui.is_null() {
        return 0;
    }
    nvim_remote_ui_get_cursor_col(ui)
}

/// Get RemoteUI highlight ID
///
/// # Safety
///
/// `ui` must be a valid RemoteUI handle
#[no_mangle]
pub unsafe extern "C" fn rs_remote_ui_get_hl_id(ui: RemoteUIHandle) -> c_int {
    if ui.is_null() {
        return 0;
    }
    nvim_remote_ui_get_hl_id(ui)
}

/// Check if RemoteUI has incomplete event
///
/// # Safety
///
/// `ui` must be a valid RemoteUI handle
#[no_mangle]
pub unsafe extern "C" fn rs_remote_ui_has_incomplete_event(ui: RemoteUIHandle) -> c_int {
    if ui.is_null() {
        return 0;
    }
    nvim_remote_ui_has_incomplete_event(ui)
}

/// Check if UI is a TUI (terminal UI)
///
/// A UI is considered a TUI if it has stdin or stdout TTY.
///
/// # Safety
///
/// `ui` must be a valid RemoteUI handle
#[no_mangle]
pub unsafe extern "C" fn rs_remote_ui_is_tui(ui: RemoteUIHandle) -> c_int {
    if ui.is_null() {
        return 0;
    }
    c_int::from(nvim_remote_ui_is_stdin_tty(ui) != 0 || nvim_remote_ui_is_stdout_tty(ui) != 0)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_extension_from_c_int() {
        assert_eq!(UIExtension::from_c_int(0), Some(UIExtension::Cmdline));
        assert_eq!(UIExtension::from_c_int(1), Some(UIExtension::Popupmenu));
        assert_eq!(UIExtension::from_c_int(6), Some(UIExtension::Multigrid));
        assert_eq!(UIExtension::from_c_int(99), None);
    }

    #[test]
    fn test_ui_extension_is_global() {
        assert!(UIExtension::Cmdline.is_global());
        assert!(UIExtension::Messages.is_global());
        assert!(!UIExtension::Linegrid.is_global());
        assert!(!UIExtension::Multigrid.is_global());
    }

    #[test]
    fn test_ui_extension_name() {
        assert_eq!(UIExtension::Cmdline.name(), "ext_cmdline");
        assert_eq!(UIExtension::Multigrid.name(), "ext_multigrid");
        assert_eq!(UIExtension::FloatDebug.name(), "_debug_float");
    }

    #[test]
    fn test_line_flags() {
        let wrap = LineFlags::WRAP;
        assert!(wrap.is_wrap());
        assert!(!wrap.is_invalid());

        let invalid = LineFlags::INVALID;
        assert!(!invalid.is_wrap());
        assert!(invalid.is_invalid());

        let both = LineFlags(LineFlags::WRAP.0 | LineFlags::INVALID.0);
        assert!(both.is_wrap());
        assert!(both.is_invalid());
    }

    #[test]
    fn test_remote_ui_handle_null() {
        let handle = RemoteUIHandle::null();
        assert!(handle.is_null());
    }

    #[test]
    fn test_ui_client_handler_handle_null() {
        let handle = UIClientHandlerHandle::null();
        assert!(handle.is_null());
    }

    #[test]
    fn test_constants() {
        assert_eq!(MAX_UI_COUNT, 16);
        assert_eq!(EVENT_BUF_SIZE, 256);
        assert_eq!(UIExtension::COUNT, 10);
        assert_eq!(UIExtension::GLOBAL_COUNT, 5);
    }
}
