//! Command preview (inccommand) functionality
//!
//! This module provides types and utilities for the 'inccommand' feature,
//! which shows live previews of substitute and other commands.

#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::{c_int, c_void};

// =============================================================================
// C Function Declarations
// =============================================================================

/// Opaque handle to a buffer (buf_T*)
type BufHandle = *mut c_void;
/// Opaque handle to a window (win_T*)
type WinHandle = *mut c_void;

extern "C" {
    /// Get the cmdpreview_bufnr static variable from C.
    fn nvim_get_cmdpreview_bufnr() -> c_int;

    // Phase 2: buffer/window manipulation

    /// Get the handle (b_fnum) of a buffer.
    fn nvim_buf_get_handle(buf: BufHandle) -> c_int;
    /// Find buffer by number.
    fn buflist_findnr(nr: c_int) -> BufHandle;
    /// Get pointer to curbuf.
    fn nvim_get_curbuf_ptr() -> BufHandle;
    /// Get pointer to curwin.
    fn nvim_get_curwin() -> WinHandle;
    /// Heap-allocate aco_save_T and call aucmd_prepbuf.
    fn nvim_buf_aucmd_prepbuf_alloc(buf: BufHandle) -> *mut c_void;
    /// Restore buf from aco and free the aco.
    fn nvim_buf_aucmd_restbuf_free(aco: *mut c_void);
    /// rename_buffer("[Preview]") -- returns FAIL or OK.
    fn rename_buffer(new_fname: *const u8) -> c_int;
    /// buf_clear() -- clear current buffer content.
    fn buf_clear();
    /// Set curbuf->b_p_ma.
    fn nvim_buf_set_b_p_ma(buf: BufHandle, val: c_int);
    /// Set buf->b_p_ul.
    fn nvim_buf_set_b_p_ul(buf: BufHandle, val: i64);
    /// Set buf->b_p_tw.
    fn nvim_buf_set_b_p_tw(buf: BufHandle, val: i64);
    /// Set cmdpreview_bufnr static.
    fn nvim_set_cmdpreview_bufnr(val: c_int);
    /// win_split(size, flags): split a window. Returns OK or FAIL.
    fn win_split(size: c_int, flags: c_int) -> c_int;
    /// TRY_WRAP + do_buffer_ext(GOTO, FIRST, FORWARD, buf_handle, 0).
    fn nvim_cmdpreview_try_do_buffer(buf_handle: c_int) -> c_int;
    /// Set preview window options (cul, cuc, spell, fen = false).
    fn nvim_win_set_preview_options(win: WinHandle);
    /// win_enter(win, false).
    fn nvim_win_enter(wp: WinHandle, undo_sync: c_int);
    /// close_windows(buf, false).
    fn close_windows(buf: BufHandle, keep_curwin: c_int);
    /// Get p_cwh option (cmdwinheight).
    fn nvim_get_p_cwh() -> c_int;
}

// WSP_BOT flag for win_split
const WSP_BOT: c_int = 0x10;
// FAIL return value
const FAIL: c_int = 0;

// =============================================================================
// Preview Mode
// =============================================================================

/// Preview mode for 'inccommand' option.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PreviewMode {
    /// No preview (inccommand=)
    #[default]
    None = 0,
    /// Preview without split (inccommand=nosplit)
    NoSplit = 1,
    /// Preview with split window (inccommand=split)
    Split = 2,
}

impl PreviewMode {
    /// Check if preview is enabled.
    #[must_use]
    pub const fn is_enabled(self) -> bool {
        !matches!(self, Self::None)
    }

    /// Check if split window should be used.
    #[must_use]
    pub const fn uses_split(self) -> bool {
        matches!(self, Self::Split)
    }

    /// Parse from string representation.
    #[must_use]
    pub fn parse(s: &str) -> Self {
        match s {
            "nosplit" => Self::NoSplit,
            "split" => Self::Split,
            _ => Self::None,
        }
    }

    /// Get string representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "",
            Self::NoSplit => "nosplit",
            Self::Split => "split",
        }
    }
}

// =============================================================================
// Preview Type
// =============================================================================

/// Type of command being previewed.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PreviewType {
    /// No preview
    #[default]
    None = 0,
    /// Substitute command (:s)
    Substitute = 1,
    /// Global command (:g)
    Global = 2,
    /// vglobal command (:v)
    VGlobal = 3,
}

impl PreviewType {
    /// Check if a type is previewable.
    #[must_use]
    pub const fn is_previewable(self) -> bool {
        !matches!(self, Self::None)
    }
}

// =============================================================================
// Preview State
// =============================================================================

/// State for command preview.
#[derive(Debug, Clone, Copy, Default)]
pub struct PreviewState {
    /// Whether preview is currently active.
    pub active: bool,
    /// The type of preview being shown.
    pub preview_type: PreviewType,
    /// The preview mode from options.
    pub mode: PreviewMode,
    /// Handle to the preview buffer (if split mode).
    pub bufnr: i64,
    /// Namespace for preview extmarks.
    pub namespace: i32,
}

impl PreviewState {
    /// Create a new preview state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            active: false,
            preview_type: PreviewType::None,
            mode: PreviewMode::None,
            bufnr: 0,
            namespace: 0,
        }
    }

    /// Check if preview should be shown.
    #[must_use]
    pub const fn should_show(&self) -> bool {
        self.mode.is_enabled() && self.preview_type.is_previewable()
    }

    /// Check if split window is needed.
    #[must_use]
    pub const fn needs_split(&self) -> bool {
        self.mode.uses_split() && self.preview_type.is_previewable()
    }

    /// Reset the state.
    pub fn reset(&mut self) {
        self.active = false;
        self.preview_type = PreviewType::None;
    }

    /// Set up for a new preview.
    pub fn init(&mut self, mode: PreviewMode, preview_type: PreviewType) {
        self.mode = mode;
        self.preview_type = preview_type;
        self.active = self.should_show();
    }
}

// =============================================================================
// Preview Buffer Info
// =============================================================================

/// Information about a buffer participating in preview.
#[derive(Debug, Clone, Copy, Default)]
pub struct PreviewBufInfo {
    /// Buffer handle.
    pub handle: i64,
    /// Whether this buffer was modified during preview.
    pub was_modified: bool,
    /// Whether this buffer needs undo restoration.
    pub needs_undo_restore: bool,
}

// =============================================================================
// Undo Information
// =============================================================================

/// Undo state saved before preview.
#[derive(Debug, Clone, Copy, Default)]
pub struct PreviewUndoInfo {
    /// Whether there's saved undo state.
    pub saved: bool,
    /// Undo sequence number.
    pub undo_seq: i64,
    /// Whether buffer was changed before preview.
    pub was_changed: bool,
}

impl PreviewUndoInfo {
    /// Create new undo info.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            saved: false,
            undo_seq: 0,
            was_changed: false,
        }
    }

    /// Save current undo state.
    pub fn save(&mut self, seq: i64, changed: bool) {
        self.saved = true;
        self.undo_seq = seq;
        self.was_changed = changed;
    }

    /// Clear saved state.
    pub fn clear(&mut self) {
        *self = Self::new();
    }
}

// =============================================================================
// Command Classification
// =============================================================================

/// Check if a command name is previewable.
#[must_use]
pub fn is_previewable_command(cmd: &[u8]) -> PreviewType {
    if cmd.is_empty() {
        return PreviewType::None;
    }

    // Substitute commands
    if cmd.starts_with(b"s") && is_substitute_prefix(cmd) {
        return PreviewType::Substitute;
    }

    // Global command
    if cmd == b"g" || cmd == b"global" {
        return PreviewType::Global;
    }

    // vglobal command
    if cmd == b"v" || cmd == b"vglobal" {
        return PreviewType::VGlobal;
    }

    PreviewType::None
}

/// Check if a command prefix is a substitute variant.
fn is_substitute_prefix(cmd: &[u8]) -> bool {
    matches!(
        cmd,
        b"s" | b"su"
            | b"sub"
            | b"subs"
            | b"subst"
            | b"substi"
            | b"substit"
            | b"substitu"
            | b"substitut"
            | b"substitute"
            | b"sm"
            | b"sma"
            | b"smag"
            | b"smagi"
            | b"smagic"
            | b"sno"
            | b"snom"
            | b"snoma"
            | b"snomag"
            | b"snomagi"
            | b"snomagic"
    )
}

// =============================================================================
// Preview Window Configuration
// =============================================================================

/// Window options that should be disabled for preview windows.
///
/// These options are disabled to avoid messing up the preview display.
#[derive(Debug, Clone, Copy, Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct PreviewWindowOptions {
    /// Disable 'cursorline'.
    pub cursorline: bool,
    /// Disable 'cursorcolumn'.
    pub cursorcolumn: bool,
    /// Disable 'spell'.
    pub spell: bool,
    /// Disable folding.
    pub foldenable: bool,
}

impl PreviewWindowOptions {
    /// Create options for preview window (all disabled).
    #[must_use]
    pub const fn for_preview() -> Self {
        Self {
            cursorline: false,
            cursorcolumn: false,
            spell: false,
            foldenable: false,
        }
    }
}

/// Check if preview buffer should be skipped during state save.
#[must_use]
pub const fn should_skip_buffer_for_preview(buf_handle: i64, preview_bufnr: i64) -> bool {
    buf_handle == preview_bufnr
}

/// Get the namespace ID for command preview extmarks.
///
/// This is lazily initialized and cached.
static CMDPREVIEW_NS: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(-1);

/// Get or initialize the preview namespace ID.
#[must_use]
pub fn get_preview_namespace() -> i32 {
    let ns = CMDPREVIEW_NS.load(std::sync::atomic::Ordering::Relaxed);
    if ns >= 0 {
        return ns;
    }
    // Return cached value from C side
    unsafe { nvim_get_cmdpreview_ns() }
}

// C function to get the preview namespace
extern "C" {
    fn nvim_get_cmdpreview_ns() -> c_int;
}

/// Set the preview namespace ID (called from C during initialization).
pub fn set_preview_namespace(ns: i32) {
    CMDPREVIEW_NS.store(ns, std::sync::atomic::Ordering::Relaxed);
}

// =============================================================================
// Preview Buffer State
// =============================================================================

/// Check if undo restoration is needed for a buffer.
#[must_use]
pub const fn needs_undo_restore(current_seq: i64, saved_seq: i64) -> bool {
    current_seq != saved_seq
}

/// Preview cmdmod flags that should be set.
pub mod cmdmod_flags {
    /// Disable swap file for preview.
    pub const CMOD_NOSWAPFILE: i32 = 0x1000;
}

// =============================================================================
// Phase 2: Buffer/Window Open/Close Implementations
// =============================================================================

/// Open or create the command preview buffer.
///
/// Returns the buffer pointer on success, null on failure.
///
/// # Safety
///
/// Calls multiple C functions that manipulate global state.
unsafe fn cmdpreview_open_buf_impl() -> BufHandle {
    // Find existing preview buffer, or NULL if not yet created.
    let preview_bufnr = nvim_get_cmdpreview_bufnr();
    let mut cmdpreview_buf: BufHandle = if preview_bufnr != 0 {
        buflist_findnr(preview_bufnr)
    } else {
        std::ptr::null_mut()
    };

    // If preview buffer doesn't exist, create one.
    if cmdpreview_buf.is_null() {
        let bufnr = nvim_cmdpreview_create_buf();
        if bufnr < 0 {
            return std::ptr::null_mut();
        }
        cmdpreview_buf = buflist_findnr(bufnr);
    }

    // Preview buffer cannot preview itself!
    if cmdpreview_buf == nvim_get_curbuf_ptr() {
        return std::ptr::null_mut();
    }

    // Rename preview buffer.
    let name = b"[Preview]\0";
    let aco = nvim_buf_aucmd_prepbuf_alloc(cmdpreview_buf);
    let retv = rename_buffer(name.as_ptr());
    nvim_buf_aucmd_restbuf_free(aco);

    if retv == FAIL {
        return std::ptr::null_mut();
    }

    // Temporarily switch to preview buffer to set it up for previewing.
    let aco = nvim_buf_aucmd_prepbuf_alloc(cmdpreview_buf);
    buf_clear();
    let curbuf = nvim_get_curbuf_ptr();
    nvim_buf_set_b_p_ma(curbuf, 1); // true
    nvim_buf_set_b_p_ul(curbuf, -1);
    nvim_buf_set_b_p_tw(curbuf, 0); // Reset 'textwidth'
    nvim_buf_aucmd_restbuf_free(aco);
    nvim_set_cmdpreview_bufnr(nvim_buf_get_handle(cmdpreview_buf));

    cmdpreview_buf
}

/// Open the command preview window.
///
/// Returns the window pointer on success, null on failure.
/// The caller must have obtained `cmdpreview_buf` from `cmdpreview_open_buf`.
///
/// # Safety
///
/// Calls C functions that manipulate global window state.
unsafe fn cmdpreview_open_win_impl(cmdpreview_buf: BufHandle) -> WinHandle {
    let save_curwin = nvim_get_curwin();

    // Open preview window at the bottom.
    if win_split(nvim_get_p_cwh(), WSP_BOT) == FAIL {
        return std::ptr::null_mut();
    }

    let preview_win = nvim_get_curwin();

    // Switch to preview buffer using TRY_WRAP wrapper.
    let buf_handle = nvim_buf_get_handle(cmdpreview_buf);
    if nvim_cmdpreview_try_do_buffer(buf_handle) == FAIL {
        return std::ptr::null_mut();
    }

    // Disable distracting options in preview window.
    nvim_win_set_preview_options(nvim_get_curwin());

    // Return to original window.
    nvim_win_enter(save_curwin, 0);
    preview_win
}

/// Close any open command preview windows.
///
/// # Safety
///
/// Calls C functions that manipulate global window state.
unsafe fn cmdpreview_close_win_impl() {
    let preview_bufnr = nvim_get_cmdpreview_bufnr();
    if preview_bufnr != 0 {
        let buf = buflist_findnr(preview_bufnr);
        if !buf.is_null() {
            close_windows(buf, 0); // keep_curwin = false
        }
    }
}

extern "C" {
    /// Create an unlisted scratch buffer. Returns handle or -1 on error.
    fn nvim_cmdpreview_create_buf() -> c_int;
}

// =============================================================================
// FFI Exports
// =============================================================================

// Note: rs_cmdpreview_get_ns is defined in lib.rs to avoid duplication

/// Get the cmdpreview buffer number (direct C symbol replacement).
///
/// # Safety
///
/// Calls external C function to access static variable.
#[must_use]
#[export_name = "cmdpreview_get_bufnr"]
pub unsafe extern "C" fn cmdpreview_get_bufnr_rs() -> c_int {
    nvim_get_cmdpreview_bufnr()
}

/// Open or create the command preview buffer (FFI, called from C).
///
/// Returns a non-null buf_T* on success, null on failure.
///
/// # Safety
///
/// Calls C functions manipulating global state.
#[no_mangle]
pub unsafe extern "C" fn rs_cmdpreview_open_buf() -> BufHandle {
    cmdpreview_open_buf_impl()
}

/// Open the command preview window (FFI, called from C).
///
/// Returns a non-null win_T* on success, null on failure.
///
/// # Safety
///
/// Calls C functions manipulating global state.
#[no_mangle]
pub unsafe extern "C" fn rs_cmdpreview_open_win(cmdpreview_buf: BufHandle) -> WinHandle {
    cmdpreview_open_win_impl(cmdpreview_buf)
}

/// Close command preview windows (FFI, called from C).
///
/// # Safety
///
/// Calls C functions manipulating global state.
#[no_mangle]
pub unsafe extern "C" fn rs_cmdpreview_close_win() {
    cmdpreview_close_win_impl();
}

/// Set preview namespace (FFI).
#[no_mangle]
pub extern "C" fn rs_cmdpreview_set_ns(ns: c_int) {
    set_preview_namespace(ns);
}

/// Check if buffer should be skipped for preview (FFI).
#[no_mangle]
pub extern "C" fn rs_cmdpreview_should_skip_buffer(buf_handle: i64, preview_bufnr: i64) -> c_int {
    c_int::from(should_skip_buffer_for_preview(buf_handle, preview_bufnr))
}

/// Check if undo restoration is needed (FFI).
#[no_mangle]
pub extern "C" fn rs_cmdpreview_needs_undo_restore(current_seq: i64, saved_seq: i64) -> c_int {
    c_int::from(needs_undo_restore(current_seq, saved_seq))
}

/// Check if preview mode is enabled (FFI).
#[no_mangle]
pub extern "C" fn rs_preview_mode_is_enabled(mode: c_int) -> c_int {
    let preview_mode = match mode {
        1 => PreviewMode::NoSplit,
        2 => PreviewMode::Split,
        _ => PreviewMode::None,
    };
    c_int::from(preview_mode.is_enabled())
}

/// Check if preview mode uses split (FFI).
#[no_mangle]
pub extern "C" fn rs_preview_mode_uses_split(mode: c_int) -> c_int {
    let preview_mode = match mode {
        1 => PreviewMode::NoSplit,
        2 => PreviewMode::Split,
        _ => PreviewMode::None,
    };
    c_int::from(preview_mode.uses_split())
}

/// Check if a preview type is previewable (FFI).
#[no_mangle]
pub extern "C" fn rs_preview_type_is_previewable(preview_type: c_int) -> c_int {
    let pt = match preview_type {
        1 => PreviewType::Substitute,
        2 => PreviewType::Global,
        3 => PreviewType::VGlobal,
        _ => PreviewType::None,
    };
    c_int::from(pt.is_previewable())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preview_mode() {
        assert!(!PreviewMode::None.is_enabled());
        assert!(PreviewMode::NoSplit.is_enabled());
        assert!(PreviewMode::Split.is_enabled());

        assert!(!PreviewMode::None.uses_split());
        assert!(!PreviewMode::NoSplit.uses_split());
        assert!(PreviewMode::Split.uses_split());

        assert_eq!(PreviewMode::parse("nosplit"), PreviewMode::NoSplit);
        assert_eq!(PreviewMode::parse("split"), PreviewMode::Split);
        assert_eq!(PreviewMode::parse(""), PreviewMode::None);
        assert_eq!(PreviewMode::parse("invalid"), PreviewMode::None);

        assert_eq!(PreviewMode::None.as_str(), "");
        assert_eq!(PreviewMode::NoSplit.as_str(), "nosplit");
        assert_eq!(PreviewMode::Split.as_str(), "split");
    }

    #[test]
    fn test_preview_type() {
        assert!(!PreviewType::None.is_previewable());
        assert!(PreviewType::Substitute.is_previewable());
        assert!(PreviewType::Global.is_previewable());
        assert!(PreviewType::VGlobal.is_previewable());
    }

    #[test]
    fn test_preview_state() {
        let mut state = PreviewState::new();
        assert!(!state.active);
        assert!(!state.should_show());
        assert!(!state.needs_split());

        state.init(PreviewMode::NoSplit, PreviewType::Substitute);
        assert!(state.should_show());
        assert!(!state.needs_split());

        state.init(PreviewMode::Split, PreviewType::Substitute);
        assert!(state.should_show());
        assert!(state.needs_split());

        state.reset();
        assert!(!state.active);
    }

    #[test]
    fn test_preview_undo_info() {
        let mut undo = PreviewUndoInfo::new();
        assert!(!undo.saved);

        undo.save(42, true);
        assert!(undo.saved);
        assert_eq!(undo.undo_seq, 42);
        assert!(undo.was_changed);

        undo.clear();
        assert!(!undo.saved);
    }

    #[test]
    fn test_is_previewable_command() {
        assert_eq!(is_previewable_command(b"s"), PreviewType::Substitute);
        assert_eq!(
            is_previewable_command(b"substitute"),
            PreviewType::Substitute
        );
        assert_eq!(is_previewable_command(b"smagic"), PreviewType::Substitute);
        assert_eq!(is_previewable_command(b"snomagic"), PreviewType::Substitute);

        assert_eq!(is_previewable_command(b"g"), PreviewType::Global);
        assert_eq!(is_previewable_command(b"global"), PreviewType::Global);

        assert_eq!(is_previewable_command(b"v"), PreviewType::VGlobal);
        assert_eq!(is_previewable_command(b"vglobal"), PreviewType::VGlobal);

        assert_eq!(is_previewable_command(b"edit"), PreviewType::None);
        assert_eq!(is_previewable_command(b""), PreviewType::None);
    }

    #[test]
    fn test_preview_window_options() {
        let opts = PreviewWindowOptions::for_preview();
        assert!(!opts.cursorline);
        assert!(!opts.cursorcolumn);
        assert!(!opts.spell);
        assert!(!opts.foldenable);
    }

    #[test]
    fn test_should_skip_buffer_for_preview() {
        // Same buffer handle - should skip
        assert!(should_skip_buffer_for_preview(42, 42));

        // Different buffer handle - should not skip
        assert!(!should_skip_buffer_for_preview(42, 100));

        // Zero preview bufnr (not set) - should not skip
        assert!(!should_skip_buffer_for_preview(42, 0));
    }

    #[test]
    fn test_needs_undo_restore() {
        // Same sequence - no restore needed
        assert!(!needs_undo_restore(10, 10));

        // Different sequence - restore needed
        assert!(needs_undo_restore(15, 10));
        assert!(needs_undo_restore(10, 15));
    }
}
