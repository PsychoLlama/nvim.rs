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
#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_double, c_int, c_void};

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
// Module-level state (moved from C statics)
// =============================================================================

/// handle_T is c_int in C (types_defs.h)
type HandleT = c_int;

/// DEFAULT_GRID_HANDLE is 1
const DEFAULT_GRID_HANDLE: HandleT = 1;

/// SHAPE_IDX_N is 0 (cursor_shape.h)
const SHAPE_IDX_N: c_int = 0;

static mut CURSOR_ROW: c_int = 0;
static mut CURSOR_COL: c_int = 0;
static mut CURSOR_GRID_HANDLE: HandleT = DEFAULT_GRID_HANDLE;
static mut PENDING_CURSOR_UPDATE: bool = false;
static mut UI_MODE_IDX: c_int = SHAPE_IDX_N;
static mut BUSY: c_int = 0;
static mut PENDING_MODE_INFO_UPDATE: bool = false;
static mut PENDING_MODE_UPDATE: bool = false;
static mut HAS_MOUSE: bool = false;
static mut PENDING_HAS_MOUSE: c_int = -1;
static mut PENDING_DEFAULT_COLORS: bool = false;

// =============================================================================
// FFI declarations: C accessors and ui_call_* functions
// =============================================================================

extern "C" {
    /// Get number of active UIs
    fn nvim_ui_active() -> usize;
    /// Get termguicolors option value
    fn nvim_get_p_tgc() -> c_int;
    /// Get uis[i] as void*
    fn nvim_ui_get_uis_ptr(i: usize) -> *mut c_void;
    /// Get rgb field of RemoteUI
    fn nvim_remoteui_get_rgb(ui: *mut c_void) -> bool;
    /// Get stdin_tty field of RemoteUI
    fn nvim_remoteui_get_stdin_tty(ui: *mut c_void) -> bool;
    /// Get stdout_tty field of RemoteUI
    fn nvim_remoteui_get_stdout_tty(ui: *mut c_void) -> bool;
    /// Get override field of RemoteUI
    fn nvim_remoteui_get_override(ui: *mut c_void) -> bool;
    /// Get pum_nlines field of RemoteUI
    fn nvim_remoteui_get_pum_nlines(ui: *mut c_void) -> c_int;
    /// Get pum_pos field of RemoteUI
    fn nvim_remoteui_get_pum_pos(ui: *mut c_void) -> bool;
    /// Get pum_width field of RemoteUI
    fn nvim_remoteui_get_pum_width(ui: *mut c_void) -> c_double;
    /// Get pum_height field of RemoteUI
    fn nvim_remoteui_get_pum_height(ui: *mut c_void) -> c_double;
    /// Get pum_row field of RemoteUI
    fn nvim_remoteui_get_pum_row(ui: *mut c_void) -> c_double;
    /// Get pum_col field of RemoteUI
    fn nvim_remoteui_get_pum_col(ui: *mut c_void) -> c_double;
    /// Get full_screen global
    fn nvim_get_full_screen() -> bool;
    /// Get starting global
    fn nvim_get_starting() -> c_int;
    /// Get cursor mode index
    fn cursor_get_mode_idx() -> c_int;
    /// Conceal check cursor line
    fn conceal_check_cursor_line();
    /// UI call: busy_start
    fn ui_call_busy_start();
    /// UI call: busy_stop
    fn ui_call_busy_stop();
    /// UI call: default_colors_set (Integer = i64)
    fn ui_call_default_colors_set(
        rgb_fg: i64,
        rgb_bg: i64,
        rgb_sp: i64,
        cterm_fg: i64,
        cterm_bg: i64,
    );
    /// Get normal_fg global (RgbValue = i32)
    fn nvim_get_normal_fg() -> i32;
    /// Get normal_bg global
    fn nvim_get_normal_bg() -> i32;
    /// Get normal_sp global
    fn nvim_get_normal_sp() -> i32;
    /// Get cterm_normal_fg_color global
    fn nvim_get_cterm_normal_fg_color() -> c_int;
    /// Get cterm_normal_bg_color global
    fn nvim_get_cterm_normal_bg_color() -> c_int;
}

// =============================================================================
// Rust-side accessors for state that C still reads/writes
// =============================================================================

/// Get cursor row (for C consumers)
#[no_mangle]
pub unsafe extern "C" fn rs_ui_get_cursor_row() -> c_int {
    CURSOR_ROW
}

/// Get cursor col (for C consumers)
#[no_mangle]
pub unsafe extern "C" fn rs_ui_get_cursor_col() -> c_int {
    CURSOR_COL
}

/// Get cursor grid handle (for C consumers)
#[no_mangle]
pub unsafe extern "C" fn rs_ui_get_cursor_grid_handle() -> HandleT {
    CURSOR_GRID_HANDLE
}

/// Get pending_cursor_update (for C consumers)
#[no_mangle]
pub unsafe extern "C" fn rs_ui_get_pending_cursor_update() -> bool {
    PENDING_CURSOR_UPDATE
}

/// Set pending_cursor_update (for C consumers like ui_refresh and ui_line)
#[no_mangle]
pub unsafe extern "C" fn rs_ui_set_pending_cursor_update(val: bool) {
    PENDING_CURSOR_UPDATE = val;
}

/// Set cursor position (for C consumers like ui_refresh)
#[no_mangle]
pub unsafe extern "C" fn rs_ui_set_cursor_pos(row: c_int, col: c_int) {
    CURSOR_ROW = row;
    CURSOR_COL = col;
}

/// Get pending_mode_info_update (for C consumers)
#[no_mangle]
pub unsafe extern "C" fn rs_ui_get_pending_mode_info_update() -> bool {
    PENDING_MODE_INFO_UPDATE
}

/// Set pending_mode_info_update (for C consumers)
#[no_mangle]
pub unsafe extern "C" fn rs_ui_set_pending_mode_info_update(val: bool) {
    PENDING_MODE_INFO_UPDATE = val;
}

/// Get pending_mode_update (for C consumers)
#[no_mangle]
pub unsafe extern "C" fn rs_ui_get_pending_mode_update() -> bool {
    PENDING_MODE_UPDATE
}

/// Set pending_mode_update (for C consumers like ui_refresh)
#[no_mangle]
pub unsafe extern "C" fn rs_ui_set_pending_mode_update(val: bool) {
    PENDING_MODE_UPDATE = val;
}

/// Get ui_mode_idx (for C consumers)
#[no_mangle]
pub unsafe extern "C" fn rs_ui_get_mode_idx() -> c_int {
    UI_MODE_IDX
}

/// Get busy counter (for C consumers)
#[no_mangle]
pub unsafe extern "C" fn rs_ui_get_busy() -> c_int {
    BUSY
}

/// Get has_mouse (for C consumers)
#[no_mangle]
pub unsafe extern "C" fn rs_ui_get_has_mouse() -> bool {
    HAS_MOUSE
}

/// Get pending_has_mouse (for C consumers)
#[no_mangle]
pub unsafe extern "C" fn rs_ui_get_pending_has_mouse() -> c_int {
    PENDING_HAS_MOUSE
}

/// Set pending_has_mouse (for C consumers like ui_refresh)
#[no_mangle]
pub unsafe extern "C" fn rs_ui_set_pending_has_mouse(val: c_int) {
    PENDING_HAS_MOUSE = val;
}

/// Get pending_default_colors (for C consumers)
#[no_mangle]
pub unsafe extern "C" fn rs_ui_get_pending_default_colors() -> bool {
    PENDING_DEFAULT_COLORS
}

/// Set pending_default_colors (for C consumers)
#[no_mangle]
pub unsafe extern "C" fn rs_ui_set_pending_default_colors(val: bool) {
    PENDING_DEFAULT_COLORS = val;
}

/// Set has_mouse (for C consumers like ui_check_mouse, still in C)
#[no_mangle]
pub unsafe extern "C" fn rs_ui_set_has_mouse(val: bool) {
    HAS_MOUSE = val;
}

// =============================================================================
// Migrated functions: Phase 1
// =============================================================================

/// Returns true if any `rgb=true` UI is attached.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_rgb_attached() -> bool {
    if nvim_get_p_tgc() != 0 {
        return true;
    }
    let count = nvim_ui_active();
    for i in 0..count {
        let ui = nvim_ui_get_uis_ptr(i);
        let tui = nvim_remoteui_get_stdin_tty(ui) || nvim_remoteui_get_stdout_tty(ui);
        if !tui && nvim_remoteui_get_rgb(ui) {
            return true;
        }
    }
    false
}

/// Returns true if a GUI is attached.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_gui_attached() -> bool {
    let count = nvim_ui_active();
    for i in 0..count {
        let ui = nvim_ui_get_uis_ptr(i);
        let tui = nvim_remoteui_get_stdin_tty(ui) || nvim_remoteui_get_stdout_tty(ui);
        if !tui {
            return true;
        }
    }
    false
}

/// Returns true if any UI requested `override=true`.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_override() -> bool {
    let count = nvim_ui_active();
    for i in 0..count {
        let ui = nvim_ui_get_uis_ptr(i);
        if nvim_remoteui_get_override(ui) {
            return true;
        }
    }
    false
}

/// Gets the number of UIs connected to this server.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_active() -> usize {
    nvim_ui_active()
}

/// Gets the popup menu height.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_pum_get_height() -> c_int {
    let count = nvim_ui_active();
    let mut pum_height: c_int = 0;
    for i in 0..count {
        let ui = nvim_ui_get_uis_ptr(i);
        let ui_pum_height = nvim_remoteui_get_pum_nlines(ui);
        if ui_pum_height != 0 {
            pum_height = if pum_height != 0 {
                pum_height.min(ui_pum_height)
            } else {
                ui_pum_height
            };
        }
    }
    pum_height
}

/// Gets the popup menu position. Returns true if position was found.
///
/// # Safety
/// Calls C accessor functions and writes to out parameters.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_pum_get_pos(
    pwidth: *mut c_double,
    pheight: *mut c_double,
    prow: *mut c_double,
    pcol: *mut c_double,
) -> bool {
    let count = nvim_ui_active();
    for i in 0..count {
        let ui = nvim_ui_get_uis_ptr(i);
        if !nvim_remoteui_get_pum_pos(ui) {
            continue;
        }
        *pwidth = nvim_remoteui_get_pum_width(ui);
        *pheight = nvim_remoteui_get_pum_height(ui);
        *prow = nvim_remoteui_get_pum_row(ui);
        *pcol = nvim_remoteui_get_pum_col(ui);
        return true;
    }
    false
}

/// Move the cursor to the given position on the default grid.
///
/// # Safety
/// Modifies Rust-owned cursor state.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_cursor_goto(new_row: c_int, new_col: c_int) {
    rs_ui_grid_cursor_goto(DEFAULT_GRID_HANDLE, new_row, new_col);
}

/// Move the cursor to the given position on the specified grid.
///
/// # Safety
/// Modifies Rust-owned cursor state.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_grid_cursor_goto(
    grid_handle: HandleT,
    new_row: c_int,
    new_col: c_int,
) {
    if new_row == CURSOR_ROW && new_col == CURSOR_COL && grid_handle == CURSOR_GRID_HANDLE {
        return;
    }
    CURSOR_ROW = new_row;
    CURSOR_COL = new_col;
    CURSOR_GRID_HANDLE = grid_handle;
    PENDING_CURSOR_UPDATE = true;
}

/// Mark cursor as needing update if it lives on the given grid.
///
/// # Safety
/// Modifies Rust-owned cursor state.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_check_cursor_grid(grid_handle: HandleT) {
    if CURSOR_GRID_HANDLE == grid_handle {
        PENDING_CURSOR_UPDATE = true;
    }
}

/// Set pending_mode_info_update flag.
///
/// # Safety
/// Modifies Rust-owned state.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_mode_info_set() {
    PENDING_MODE_INFO_UPDATE = true;
}

/// Check if current mode has changed; update cursor shape state.
///
/// # Safety
/// Calls C accessor for mode index, modifies Rust-owned state.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_cursor_shape_no_check_conceal() {
    if !nvim_get_full_screen() {
        return;
    }
    let new_mode_idx = cursor_get_mode_idx();
    if new_mode_idx != UI_MODE_IDX {
        UI_MODE_IDX = new_mode_idx;
        PENDING_MODE_UPDATE = true;
    }
}

/// Check if current mode has changed; also check conceal.
///
/// # Safety
/// Calls C functions.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_cursor_shape() {
    rs_ui_cursor_shape_no_check_conceal();
    conceal_check_cursor_line();
}

/// Set pending_default_colors flag; flush immediately if not starting.
///
/// # Safety
/// Calls C accessor and modifies Rust-owned state.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_default_colors_set() {
    PENDING_DEFAULT_COLORS = true;
    if nvim_get_starting() == 0 {
        rs_ui_may_set_default_colors();
    }
}

/// Flush default colors to UI if pending.
///
/// # Safety
/// Calls C ui_call function.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_may_set_default_colors() {
    if PENDING_DEFAULT_COLORS {
        PENDING_DEFAULT_COLORS = false;
        ui_call_default_colors_set(
            i64::from(nvim_get_normal_fg()),
            i64::from(nvim_get_normal_bg()),
            i64::from(nvim_get_normal_sp()),
            i64::from(nvim_get_cterm_normal_fg_color()),
            i64::from(nvim_get_cterm_normal_bg_color()),
        );
    }
}

/// Increment busy counter; call ui_call_busy_start on first increment.
///
/// # Safety
/// Calls C ui_call function.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_busy_start() {
    if BUSY == 0 {
        ui_call_busy_start();
    }
    BUSY += 1;
}

/// Decrement busy counter; call ui_call_busy_stop when it reaches zero.
///
/// # Safety
/// Calls C ui_call function.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_busy_stop() {
    BUSY -= 1;
    if BUSY == 0 {
        ui_call_busy_stop();
    }
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
