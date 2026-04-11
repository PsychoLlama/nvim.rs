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
// Phase 3: Prepare and Restore Orchestration
// =============================================================================

extern "C" {
    // Window iteration (FOR_ALL_WINDOWS_IN_TAB equivalent)
    fn nvim_get_curtab() -> *mut c_void;
    fn nvim_tabpage_first_win(tp: *mut c_void) -> WinHandle;
    fn nvim_win_next(wp: WinHandle) -> WinHandle;

    // Buffer from window
    fn nvim_win_get_w_buffer_raw(wp: WinHandle) -> BufHandle;

    // b_changed
    fn nvim_buf_get_b_changed(buf: BufHandle) -> bool;
    fn nvim_buf_set_b_changed(buf: BufHandle, val: bool);

    // b_p_ma (getter only; setter already declared above)
    fn nvim_buf_get_b_p_ma(buf: BufHandle) -> c_int;

    // b_p_ul (getter only; setter already declared above)
    fn nvim_buf_get_b_p_ul(buf: BufHandle) -> i64;

    // b_op_start / b_op_end
    fn nvim_buf_get_b_op_start_lnum(buf: BufHandle) -> c_int;
    fn nvim_buf_get_b_op_start_col(buf: BufHandle) -> c_int;
    fn nvim_buf_get_b_op_start_coladd(buf: BufHandle) -> c_int;
    fn nvim_buf_get_b_op_end_lnum(buf: BufHandle) -> c_int;
    fn nvim_buf_get_b_op_end_col(buf: BufHandle) -> c_int;
    fn nvim_buf_get_b_op_end_coladd(buf: BufHandle) -> c_int;
    fn nvim_buf_set_b_op_start(buf: BufHandle, lnum: c_int, col: c_int, coladd: c_int);
    fn nvim_buf_set_b_op_end(buf: BufHandle, lnum: c_int, col: c_int, coladd: c_int);

    // changedtick (use _direct variant to avoid conflict with nvim API function)
    fn nvim_buf_get_changedtick_direct(buf: BufHandle) -> i64;
    fn buf_set_changedtick(buf: BufHandle, changedtick: i64);

    // Undo field accessors (from undo.c)
    fn nvim_buf_get_b_u_oldhead(buf: BufHandle) -> *mut c_void;
    fn nvim_buf_set_b_u_oldhead(buf: BufHandle, val: *mut c_void);
    fn nvim_buf_get_b_u_newhead(buf: BufHandle) -> *mut c_void;
    fn nvim_buf_set_b_u_newhead(buf: BufHandle, val: *mut c_void);
    fn nvim_buf_get_b_u_curhead(buf: BufHandle) -> *mut c_void;
    fn nvim_buf_set_b_u_curhead(buf: BufHandle, val: *mut c_void);
    fn nvim_buf_get_b_u_numhead(buf: BufHandle) -> c_int;
    fn nvim_buf_set_b_u_numhead(buf: BufHandle, val: c_int);
    fn nvim_buf_get_b_u_synced(buf: BufHandle) -> bool;
    fn nvim_buf_set_b_u_synced(buf: BufHandle, val: bool);
    fn nvim_buf_get_b_u_seq_last(buf: BufHandle) -> c_int;
    fn nvim_buf_set_b_u_seq_last(buf: BufHandle, val: c_int);
    fn nvim_buf_get_b_u_save_nr_last(buf: BufHandle) -> c_int;
    fn nvim_buf_set_b_u_save_nr_last(buf: BufHandle, val: c_int);
    fn nvim_buf_get_b_u_seq_cur(buf: BufHandle) -> c_int;
    fn nvim_buf_set_b_u_seq_cur(buf: BufHandle, val: c_int);
    fn nvim_buf_get_b_u_time_cur(buf: BufHandle) -> i64;
    fn nvim_buf_set_b_u_time_cur(buf: BufHandle, val: i64);
    fn nvim_buf_get_b_u_save_nr_cur(buf: BufHandle) -> c_int;
    fn nvim_buf_set_b_u_save_nr_cur(buf: BufHandle, val: c_int);
    fn nvim_buf_get_b_u_line_ptr(buf: BufHandle) -> *mut std::ffi::c_char;
    fn nvim_buf_set_b_u_line_ptr(buf: BufHandle, val: *mut std::ffi::c_char);
    fn nvim_buf_get_b_u_line_lnum(buf: BufHandle) -> c_int;
    fn nvim_buf_set_b_u_line_lnum(buf: BufHandle, val: c_int);
    fn nvim_buf_get_b_u_line_colnr(buf: BufHandle) -> c_int;
    fn nvim_buf_set_b_u_line_colnr(buf: BufHandle, val: c_int);

    // undo operations
    fn u_clearall(buf: BufHandle);
    fn u_blockfree(buf: BufHandle);
    fn u_sync(force: bool);
    fn u_undo_and_forget(count: c_int, do_buf_event: bool) -> bool;

    // Undo step counting (from cmdpreview.c helpers)
    fn nvim_buf_count_undo_steps(buf: BufHandle) -> c_int;

    // extmarks
    fn extmark_clear(
        buf: BufHandle,
        ns_id: u32,
        l_row: c_int,
        l_col: c_int,
        u_row: c_int,
        u_col: c_int,
    ) -> bool;

    // window cursor
    fn nvim_win_get_cursor_lnum(wp: WinHandle) -> c_int;
    fn nvim_win_set_cursor_lnum(wp: WinHandle, val: c_int);
    fn nvim_win_get_cursor_col(wp: WinHandle) -> c_int;
    fn nvim_win_set_cursor_col(wp: WinHandle, val: c_int);
    fn nvim_win_get_cursor_coladd(wp: WinHandle) -> c_int;
    fn nvim_win_set_cursor_coladd(wp: WinHandle, val: c_int);

    // window cursor/line options
    fn nvim_win_get_w_p_cul(wp: WinHandle) -> c_int;
    fn nvim_win_set_w_p_cul(wp: WinHandle, val: bool);
    fn nvim_win_get_w_p_cuc(wp: WinHandle) -> c_int;
    fn nvim_win_set_w_p_cuc(wp: WinHandle, val: bool);

    // update_topline
    fn update_topline(wp: WinHandle);

    // search patterns
    fn save_search_patterns();
    fn restore_search_patterns();

    // p_hls (declared in C as int, but our getter returns bool-like int)
    fn nvim_get_p_hls() -> c_int;
    fn nvim_set_p_hls(val: bool);

    // cmdmod save/restore via C helpers
    fn nvim_cmdpreview_save_cmdmod() -> *mut c_void;
    fn nvim_cmdpreview_restore_cmdmod(saved: *mut c_void);

    // win_size save/restore
    fn rs_win_size_save(gap: *mut c_void);
    fn rs_win_size_restore(gap: *mut c_void);

    // garray heap management
    fn nvim_ga_alloc_int() -> *mut c_void;
    fn nvim_ga_clear_free(gap: *mut c_void);

    // cmdpreview cmdmod setup
    fn nvim_cmdpreview_setup_cmdmod();
}

/// Saved undo state for a buffer during command preview.
struct CpUndoSaved {
    oldhead: *mut c_void,
    newhead: *mut c_void,
    curhead: *mut c_void,
    numhead: c_int,
    synced: bool,
    seq_last: c_int,
    save_nr_last: c_int,
    seq_cur: c_int,
    time_cur: i64,
    save_nr_cur: c_int,
    line_ptr: *mut std::ffi::c_char,
    line_lnum: c_int,
    line_colnr: c_int,
}

/// Saved state for one buffer during command preview.
struct CpBufSaved {
    buf: BufHandle,
    save_b_p_ul: i64,
    save_b_p_ma: c_int,
    save_b_changed: bool,
    save_b_op_start: (c_int, c_int, c_int), // (lnum, col, coladd)
    save_b_op_end: (c_int, c_int, c_int),
    save_changedtick: i64,
    undo: CpUndoSaved,
}

/// Saved state for one window during command preview.
struct CpWinSaved {
    win: WinHandle,
    save_cursor_lnum: c_int,
    save_cursor_col: c_int,
    save_cursor_coladd: c_int,
    save_viewstate: crate::viewstate::ViewState,
    save_w_p_cul: c_int,
    save_w_p_cuc: c_int,
}

/// All saved state for command preview (owned by Rust).
struct CpState {
    buf_info: Vec<CpBufSaved>,
    win_info: Vec<CpWinSaved>,
    save_hls: c_int,
    save_cmdmod: *mut c_void,
    save_view: *mut c_void,
}

// Safety: CpState contains raw pointers but is only used on the main Neovim
// thread. The pointers are all valid C objects during the prepare/restore cycle.
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Send for CpState {}
#[allow(clippy::non_send_fields_in_send_ty)]
unsafe impl Sync for CpState {}

/// Save undo fields from a buffer handle into `CpUndoSaved`.
///
/// # Safety
///
/// `buf` must be a valid buffer pointer.
unsafe fn save_undo(buf: BufHandle) -> CpUndoSaved {
    CpUndoSaved {
        oldhead: nvim_buf_get_b_u_oldhead(buf),
        newhead: nvim_buf_get_b_u_newhead(buf),
        curhead: nvim_buf_get_b_u_curhead(buf),
        numhead: nvim_buf_get_b_u_numhead(buf),
        synced: nvim_buf_get_b_u_synced(buf),
        seq_last: nvim_buf_get_b_u_seq_last(buf),
        save_nr_last: nvim_buf_get_b_u_save_nr_last(buf),
        seq_cur: nvim_buf_get_b_u_seq_cur(buf),
        time_cur: nvim_buf_get_b_u_time_cur(buf),
        save_nr_cur: nvim_buf_get_b_u_save_nr_cur(buf),
        line_ptr: nvim_buf_get_b_u_line_ptr(buf),
        line_lnum: nvim_buf_get_b_u_line_lnum(buf),
        line_colnr: nvim_buf_get_b_u_line_colnr(buf),
    }
}

/// Restore undo fields from `CpUndoSaved` into a buffer handle.
///
/// # Safety
///
/// `buf` must be a valid buffer pointer. All pointer fields in `saved`
/// must remain valid (they were saved from the same buffer before preview).
unsafe fn restore_undo(buf: BufHandle, saved: &CpUndoSaved) {
    nvim_buf_set_b_u_oldhead(buf, saved.oldhead);
    nvim_buf_set_b_u_newhead(buf, saved.newhead);
    nvim_buf_set_b_u_curhead(buf, saved.curhead);
    nvim_buf_set_b_u_numhead(buf, saved.numhead);
    nvim_buf_set_b_u_seq_last(buf, saved.seq_last);
    nvim_buf_set_b_u_save_nr_last(buf, saved.save_nr_last);
    nvim_buf_set_b_u_seq_cur(buf, saved.seq_cur);
    nvim_buf_set_b_u_time_cur(buf, saved.time_cur);
    nvim_buf_set_b_u_save_nr_cur(buf, saved.save_nr_cur);
    nvim_buf_set_b_u_line_ptr(buf, saved.line_ptr);
    nvim_buf_set_b_u_line_lnum(buf, saved.line_lnum);
    nvim_buf_set_b_u_line_colnr(buf, saved.line_colnr);
    if saved.curhead.is_null() {
        nvim_buf_set_b_u_synced(buf, saved.synced);
    }
}

/// Save state and prepare all buffers/windows for command preview.
///
/// Equivalent to C `cmdpreview_prepare`. Returns an opaque Box as raw pointer.
///
/// # Safety
///
/// Must be called on the main Neovim thread with valid global state.
unsafe fn cmdpreview_prepare_impl() -> *mut c_void {
    let mut buf_info: Vec<CpBufSaved> = Vec::new();
    let mut win_info: Vec<CpWinSaved> = Vec::new();
    // Track which buffers have already been saved (replace Set(ptr_t)).
    let mut saved_bufs: std::collections::HashSet<usize> = std::collections::HashSet::new();

    let curtab = nvim_get_curtab();
    let mut wp = nvim_tabpage_first_win(curtab);

    while !wp.is_null() {
        let buf = nvim_win_get_w_buffer_raw(wp);
        let buf_handle = nvim_buf_get_handle(buf);
        let preview_bufnr = nvim_get_cmdpreview_bufnr();

        // Don't save state of command preview buffer or preview window.
        if buf_handle == preview_bufnr {
            wp = nvim_win_next(wp);
            continue;
        }

        let buf_key = buf as usize;
        if saved_bufs.insert(buf_key) {
            let undo = save_undo(buf);
            let saved = CpBufSaved {
                buf,
                save_b_p_ul: nvim_buf_get_b_p_ul(buf),
                save_b_p_ma: nvim_buf_get_b_p_ma(buf),
                save_b_changed: nvim_buf_get_b_changed(buf),
                save_b_op_start: (
                    nvim_buf_get_b_op_start_lnum(buf),
                    nvim_buf_get_b_op_start_col(buf),
                    nvim_buf_get_b_op_start_coladd(buf),
                ),
                save_b_op_end: (
                    nvim_buf_get_b_op_end_lnum(buf),
                    nvim_buf_get_b_op_end_col(buf),
                    nvim_buf_get_b_op_end_coladd(buf),
                ),
                save_changedtick: nvim_buf_get_changedtick_direct(buf),
                undo,
            };
            buf_info.push(saved);

            u_clearall(buf);
            // Make sure we can undo all changes (b_p_ul = INT_MAX)
            nvim_buf_set_b_p_ul(buf, i32::MAX as i64);
        }

        let win_saved = CpWinSaved {
            win: wp,
            save_cursor_lnum: nvim_win_get_cursor_lnum(wp),
            save_cursor_col: nvim_win_get_cursor_col(wp),
            save_cursor_coladd: nvim_win_get_cursor_coladd(wp),
            save_viewstate: crate::viewstate::ViewState::from_window(wp),
            save_w_p_cul: nvim_win_get_w_p_cul(wp),
            save_w_p_cuc: nvim_win_get_w_p_cuc(wp),
        };
        win_info.push(win_saved);

        // Disable 'cursorline'/'cursorcolumn' so they don't mess up highlights
        nvim_win_set_w_p_cul(wp, false);
        nvim_win_set_w_p_cuc(wp, false);

        wp = nvim_win_next(wp);
    }

    let save_hls = nvim_get_p_hls();
    let save_cmdmod = nvim_cmdpreview_save_cmdmod();
    let save_view = nvim_ga_alloc_int();
    rs_win_size_save(save_view);
    save_search_patterns();

    // Disable search highlighting, tab/split modifiers, enable noswapfile
    nvim_set_p_hls(false);
    nvim_cmdpreview_setup_cmdmod();

    u_sync(true);

    let state = Box::new(CpState {
        buf_info,
        win_info,
        save_hls,
        save_cmdmod,
        save_view,
    });
    Box::into_raw(state).cast::<c_void>()
}

/// Restore all buffer/window state after command preview.
///
/// Equivalent to C `cmdpreview_restore_state`. Frees the state box.
///
/// # Safety
///
/// `state` must be a pointer returned by `rs_cmdpreview_prepare`.
/// Must be called on the main Neovim thread.
unsafe fn cmdpreview_restore_state_impl(state: *mut c_void) {
    let state = Box::from_raw(state.cast::<CpState>());

    let preview_ns = nvim_get_cmdpreview_ns();

    for cp in &state.buf_info {
        let buf = cp.buf;

        nvim_buf_set_b_changed(buf, cp.save_b_changed);

        // Clear preview highlights.
        extmark_clear(buf, preview_ns as u32, 0, 0, c_int::MAX, c_int::MAX);

        // Check if undo restoration is needed.
        let cur_seq = nvim_buf_get_b_u_seq_cur(buf);
        if cur_seq != cp.undo.seq_cur {
            let count = nvim_buf_count_undo_steps(buf);
            let aco = nvim_buf_aucmd_prepbuf_alloc(buf);
            // Ensure all entries will be undone.
            if !nvim_buf_get_b_u_synced(buf) {
                u_sync(true);
            }
            // Undo invisibly. This also moves the cursor!
            if !u_undo_and_forget(count, false) {
                // This should not happen — abort as C did.
                std::process::abort();
            }
            nvim_buf_aucmd_restbuf_free(aco);
        }

        u_blockfree(buf);
        restore_undo(buf, &cp.undo);

        nvim_buf_set_b_op_start(
            buf,
            cp.save_b_op_start.0,
            cp.save_b_op_start.1,
            cp.save_b_op_start.2,
        );
        nvim_buf_set_b_op_end(
            buf,
            cp.save_b_op_end.0,
            cp.save_b_op_end.1,
            cp.save_b_op_end.2,
        );

        let cur_tick = nvim_buf_get_changedtick_direct(buf);
        if cp.save_changedtick != cur_tick {
            buf_set_changedtick(buf, cp.save_changedtick);
        }

        nvim_buf_set_b_p_ul(buf, cp.save_b_p_ul); // Restore 'undolevels'
        nvim_buf_set_b_p_ma(buf, cp.save_b_p_ma); // Restore 'modifiable'
    }

    for cw in &state.win_info {
        let win = cw.win;

        // Restore window cursor position.
        nvim_win_set_cursor_lnum(win, cw.save_cursor_lnum);
        nvim_win_set_cursor_col(win, cw.save_cursor_col);
        nvim_win_set_cursor_coladd(win, cw.save_cursor_coladd);

        // Restore viewstate.
        cw.save_viewstate.restore_to_window(win);

        // Restore 'cursorline' and 'cursorcolumn'.
        nvim_win_set_w_p_cul(win, cw.save_w_p_cul != 0);
        nvim_win_set_w_p_cuc(win, cw.save_w_p_cuc != 0);

        update_topline(win);
    }

    nvim_cmdpreview_restore_cmdmod(state.save_cmdmod); // Restore cmdmod + free
    nvim_set_p_hls(state.save_hls != 0); // Restore 'hlsearch'
    restore_search_patterns(); // Restore search patterns
    rs_win_size_restore(state.save_view); // Restore window sizes
    nvim_ga_clear_free(state.save_view); // Free the garray
                                         // state is dropped here (Box freed)
}

/// Prepare Rust-managed cmdpreview state (FFI, called from C).
///
/// Returns an opaque `*mut c_void` that must be passed to `rs_cmdpreview_restore_state`.
///
/// # Safety
///
/// Must be called on the main Neovim thread with valid global state.
#[no_mangle]
pub unsafe extern "C" fn rs_cmdpreview_prepare() -> *mut c_void {
    cmdpreview_prepare_impl()
}

/// Restore cmdpreview state and free it (FFI, called from C).
///
/// # Safety
///
/// `state` must be a pointer returned by `rs_cmdpreview_prepare`.
#[no_mangle]
pub unsafe extern "C" fn rs_cmdpreview_restore_state(state: *mut c_void) {
    cmdpreview_restore_state_impl(state);
}

// =============================================================================
// Phase 4: cmdpreview_may_show Implementation
// =============================================================================

extern "C" {
    // ccline cmdbuff
    fn nvim_get_ccline_cmdbuff() -> *mut std::ffi::c_char;

    // xstrdup / xfree (C memory)
    fn xstrdup(s: *const std::ffi::c_char) -> *mut std::ffi::c_char;
    fn xfree(ptr: *mut std::ffi::c_char);

    // emsg_off
    fn rs_emsg_off_enter();
    fn rs_emsg_off_leave();

    // emsg_silent
    fn rs_emsg_silent_enter();
    fn rs_emsg_silent_leave();

    // msg_silent
    fn rs_msg_silent_enter();
    fn rs_msg_silent_leave();

    // autocmds
    fn block_autocmds();
    fn unblock_autocmds();

    // screen
    fn cursorcmd();
    fn rs_cmdline_ui_flush();
    fn update_screen();
    fn redrawcmdline();
    fn nvim_get_RedrawingDisabled() -> c_int;
    fn nvim_set_RedrawingDisabled(val: c_int);

    // parse context (heap exarg_T + CmdParseInfo)
    fn nvim_cmdpreview_alloc_parse_ctx() -> *mut c_void;
    fn nvim_cmdpreview_free_parse_ctx(ctx: *mut c_void);
    fn nvim_cmdpreview_do_parse(ctx: *mut c_void, cmdline: *mut *mut std::ffi::c_char) -> bool;
    fn nvim_cmdpreview_ctx_has_preview(ctx: *mut c_void) -> bool;
    fn nvim_cmdpreview_ctx_has_range(ctx: *mut c_void) -> bool;
    fn nvim_cmdpreview_ctx_get_line1(ctx: *mut c_void) -> c_int;
    fn nvim_cmdpreview_ctx_get_line2(ctx: *mut c_void) -> c_int;
    fn nvim_cmdpreview_ctx_set_line1(ctx: *mut c_void, val: c_int);
    fn nvim_cmdpreview_ctx_set_line2(ctx: *mut c_void, val: c_int);
    fn nvim_cmdpreview_ctx_undo_cmdmod(ctx: *mut c_void);
    fn nvim_cmdpreview_try_execute(ctx: *mut c_void) -> c_int;

    // cmdpreview global and options
    fn nvim_set_cmdpreview_global(val: bool);
    fn nvim_get_p_icm_is_split() -> bool;
    fn nvim_set_option_icm_nosplit();
    fn nvim_cmdpreview_ensure_ns() -> c_int;
}

/// Implementation of cmdpreview_may_show.
///
/// # Safety
///
/// Must be called on the main Neovim thread.
unsafe fn cmdpreview_may_show_impl() -> bool {
    // Copy the command line so we can modify it.
    let cmdbuff = nvim_get_ccline_cmdbuff();
    let mut cmdline: *mut std::ffi::c_char = xstrdup(cmdbuff);

    // Allocate parse context (heap exarg_T + CmdParseInfo).
    let ctx = nvim_cmdpreview_alloc_parse_ctx();

    // Block errors when parsing; don't update v:errmsg.
    rs_emsg_off_enter();
    let parse_ok = nvim_cmdpreview_do_parse(ctx, std::ptr::addr_of_mut!(cmdline));
    rs_emsg_off_leave();

    if !parse_ok {
        nvim_cmdpreview_free_parse_ctx(ctx);
        xfree(cmdline);
        return false;
    }

    // Check if command is previewable; if not, bail.
    if !nvim_cmdpreview_ctx_has_preview(ctx) {
        nvim_cmdpreview_ctx_undo_cmdmod(ctx);
        nvim_cmdpreview_free_parse_ctx(ctx);
        xfree(cmdline);
        return false;
    }

    // Cursor may be at the end of the message grid rather than at cmdspos.
    // Place it there in case preview callback flushes it. #30696
    cursorcmd();
    // Flush now: external cmdline may itself wish to update the screen.
    rs_cmdline_ui_flush();

    // Swap invalid command range if needed.
    if nvim_cmdpreview_ctx_has_range(ctx) {
        let line1 = nvim_cmdpreview_ctx_get_line1(ctx);
        let line2 = nvim_cmdpreview_ctx_get_line2(ctx);
        if line1 > line2 {
            nvim_cmdpreview_ctx_set_line1(ctx, line2);
            nvim_cmdpreview_ctx_set_line2(ctx, line1);
        }
    }

    let mut icm_split = nvim_get_p_icm_is_split();
    let mut cmdpreview_buf: BufHandle = std::ptr::null_mut();
    let mut cmdpreview_win: WinHandle = std::ptr::null_mut();

    // Block error reporting, messages, and events.
    rs_emsg_silent_enter();
    rs_msg_silent_enter();
    block_autocmds();

    // Save current state and prepare for command preview.
    let cpstate = rs_cmdpreview_prepare();

    // Open preview buffer if inccommand=split.
    if icm_split {
        cmdpreview_buf = rs_cmdpreview_open_buf();
        if cmdpreview_buf.is_null() {
            nvim_set_option_icm_nosplit();
            icm_split = false;
        }
    }

    // Ensure cmdpreview namespace is set.
    nvim_cmdpreview_ensure_ns();

    // Set cmdpreview flag.
    nvim_set_cmdpreview_global(true);

    // Execute the preview callback (via TRY_WRAP in C helper).
    let mut cmdpreview_type = nvim_cmdpreview_try_execute(ctx);

    // If inccommand=split and preview callback returns 2, open preview window.
    if icm_split && cmdpreview_type == 2 {
        cmdpreview_win = rs_cmdpreview_open_win(cmdpreview_buf);
        if cmdpreview_win.is_null() {
            cmdpreview_type = 1;
        }
    }

    // If preview callback return value is nonzero, update screen now.
    if cmdpreview_type != 0 {
        let save_rd = nvim_get_RedrawingDisabled();
        nvim_set_RedrawingDisabled(0);
        update_screen();
        nvim_set_RedrawingDisabled(save_rd);
    }

    // Close preview window if it's open.
    if icm_split && cmdpreview_type == 2 && !cmdpreview_win.is_null() {
        rs_cmdpreview_close_win();
    }

    // Restore state.
    rs_cmdpreview_restore_state(cpstate);

    unblock_autocmds();
    rs_msg_silent_leave();
    rs_emsg_silent_leave();
    redrawcmdline();

    nvim_cmdpreview_free_parse_ctx(ctx);
    xfree(cmdline);
    cmdpreview_type != 0
}

/// Show 'inccommand' preview if command is previewable (FFI, replaces C symbol).
///
/// # Safety
///
/// Must be called on the main Neovim thread.
#[allow(clippy::must_use_candidate)]
#[export_name = "rs_cmdpreview_may_show"]
pub unsafe extern "C" fn rs_cmdpreview_may_show_export() -> bool {
    cmdpreview_may_show_impl()
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
