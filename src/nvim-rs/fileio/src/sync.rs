//! Buffer-file synchronization utilities.
//!
//! This module provides:
//! - File change detection (timestamp, size, mode)
//! - Change reason classification
//! - Reload decision logic

use crate::{bref_void, buf_mut_void};
use std::ffi::{c_char, c_int, c_void};
use std::sync::atomic::{AtomicBool, Ordering};

// =============================================================================
// Change Reasons
// =============================================================================

/// Reason why a file has changed.
///
/// Used for FileChangedShell autocmd v:fcs_reason variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChangeReason {
    /// File was deleted
    Deleted,
    /// File changed and buffer was also modified (conflict)
    Conflict,
    /// File contents changed
    Changed,
    /// Only file mode/permissions changed
    Mode,
    /// Only timestamp changed (no content change)
    Time,
}

impl ChangeReason {
    /// Returns the string representation for v:fcs_reason.
    pub fn as_str(&self) -> &'static str {
        match self {
            ChangeReason::Deleted => "deleted",
            ChangeReason::Conflict => "conflict",
            ChangeReason::Changed => "changed",
            ChangeReason::Mode => "mode",
            ChangeReason::Time => "time",
        }
    }

    /// Parse from the reason string.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "deleted" => Some(ChangeReason::Deleted),
            "conflict" => Some(ChangeReason::Conflict),
            "changed" => Some(ChangeReason::Changed),
            "mode" => Some(ChangeReason::Mode),
            "time" => Some(ChangeReason::Time),
            _ => None,
        }
    }
}

// =============================================================================
// File Change Detection
// =============================================================================

/// Result of checking file status against stored buffer info.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileStatus {
    /// File is unchanged
    Unchanged,
    /// File no longer exists
    Deleted,
    /// File timestamp changed
    TimeChanged,
    /// File mode/permissions changed
    ModeChanged,
    /// File size changed
    SizeChanged,
    /// File contents changed (detected via checksum or size)
    ContentsChanged,
}

impl FileStatus {
    /// Check if this status indicates any change.
    pub fn is_changed(&self) -> bool {
        !matches!(self, FileStatus::Unchanged)
    }
}

/// Information about a file for change detection.
#[derive(Debug, Clone, Copy, Default)]
pub struct FileInfo {
    /// File modification time (seconds since epoch)
    pub mtime: i64,
    /// File modification time (nanoseconds)
    pub mtime_ns: i64,
    /// File size in bytes
    pub size: u64,
    /// File mode/permissions
    pub mode: i32,
    /// Whether the file exists
    pub exists: bool,
}

impl FileInfo {
    /// Create a new FileInfo.
    pub fn new(mtime: i64, mtime_ns: i64, size: u64, mode: i32, exists: bool) -> Self {
        Self {
            mtime,
            mtime_ns,
            size,
            mode,
            exists,
        }
    }

    /// Check if time differs from stored values (with FAT tolerance).
    pub fn time_differs(
        &self,
        stored_mtime: i64,
        stored_mtime_ns: i64,
        fat_tolerance: bool,
    ) -> bool {
        crate::time_differs(
            self.mtime,
            self.mtime_ns,
            stored_mtime,
            stored_mtime_ns,
            fat_tolerance,
        )
    }
}

/// Stored buffer file information for change detection.
#[derive(Debug, Clone, Copy, Default)]
pub struct BufferFileInfo {
    /// Last known modification time (seconds)
    pub mtime: i64,
    /// Last known modification time (nanoseconds)
    pub mtime_ns: i64,
    /// Last known file size
    pub orig_size: u64,
    /// Last known file mode
    pub orig_mode: i32,
}

impl BufferFileInfo {
    /// Create new buffer file info.
    pub fn new(mtime: i64, mtime_ns: i64, orig_size: u64, orig_mode: i32) -> Self {
        Self {
            mtime,
            mtime_ns,
            orig_size,
            orig_mode,
        }
    }

    /// Update from current file info.
    pub fn update_from(&mut self, file_info: &FileInfo) {
        self.mtime = file_info.mtime;
        self.mtime_ns = file_info.mtime_ns;
        self.orig_size = file_info.size;
        self.orig_mode = file_info.mode;
    }
}

// =============================================================================
// Change Detection Logic
// =============================================================================

/// Determine the change reason based on file status and buffer state.
///
/// # Arguments
/// * `file_exists` - Whether the file currently exists
/// * `buffer_modified` - Whether the buffer has unsaved changes
/// * `size_changed` - Whether the file size differs from stored
/// * `mode_changed` - Whether the file mode differs from stored
/// * `contents_changed` - Whether contents actually differ (from checksum)
///
/// # Returns
/// The appropriate change reason
pub fn determine_change_reason(
    file_exists: bool,
    buffer_modified: bool,
    size_changed: bool,
    mode_changed: bool,
    contents_changed: bool,
) -> ChangeReason {
    if !file_exists {
        return ChangeReason::Deleted;
    }

    if buffer_modified {
        return ChangeReason::Conflict;
    }

    if size_changed || contents_changed {
        return ChangeReason::Changed;
    }

    if mode_changed {
        return ChangeReason::Mode;
    }

    // Only timestamp changed
    ChangeReason::Time
}

/// FFI wrapper for determine_change_reason.
///
/// # Returns
/// Pointer to static string for the reason.
///
/// # Safety
/// All parameters are plain integers.
#[no_mangle]
pub extern "C" fn rs_determine_change_reason(
    file_exists: c_int,
    buffer_modified: c_int,
    size_changed: c_int,
    mode_changed: c_int,
    contents_changed: c_int,
) -> *const u8 {
    let reason = determine_change_reason(
        file_exists != 0,
        buffer_modified != 0,
        size_changed != 0,
        mode_changed != 0,
        contents_changed != 0,
    );

    // Return pointer to static string
    reason.as_str().as_ptr()
}

// =============================================================================
// Reload Decision
// =============================================================================

/// User's choice for handling changed files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileChangedChoice {
    /// Keep buffer contents, ignore file changes
    Ok,
    /// Reload file contents, keep buffer options
    Reload,
    /// Reload file with re-detection of options (encoding, etc.)
    Edit,
    /// Ask the user what to do
    Ask,
}

impl FileChangedChoice {
    /// Parse from v:fcs_choice string.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "" | "ok" => Some(FileChangedChoice::Ok),
            "reload" => Some(FileChangedChoice::Reload),
            "edit" => Some(FileChangedChoice::Edit),
            "ask" => Some(FileChangedChoice::Ask),
            _ => None,
        }
    }

    /// Convert to string for v:fcs_choice.
    pub fn as_str(&self) -> &'static str {
        match self {
            FileChangedChoice::Ok => "",
            FileChangedChoice::Reload => "reload",
            FileChangedChoice::Edit => "edit",
            FileChangedChoice::Ask => "ask",
        }
    }
}

/// Determine if a buffer should be auto-reloaded.
///
/// # Arguments
/// * `autoread_enabled` - Whether 'autoread' is set (buffer-local or global)
/// * `buffer_modified` - Whether the buffer has unsaved changes
/// * `file_exists` - Whether the file still exists
/// * `reason` - The change reason
///
/// # Returns
/// Whether to auto-reload the buffer
pub fn should_auto_reload(
    autoread_enabled: bool,
    buffer_modified: bool,
    file_exists: bool,
    reason: ChangeReason,
) -> bool {
    // Never auto-reload if file was deleted
    if reason == ChangeReason::Deleted || !file_exists {
        return false;
    }

    // Never auto-reload if buffer has unsaved changes
    if buffer_modified {
        return false;
    }

    // Only auto-reload if 'autoread' is enabled
    autoread_enabled
}

/// FFI wrapper for should_auto_reload.
///
/// # Safety
/// - `reason_ptr` must be a valid pointer to `reason_len` bytes if not null
#[no_mangle]
pub unsafe extern "C" fn rs_should_auto_reload(
    autoread_enabled: c_int,
    buffer_modified: c_int,
    file_exists: c_int,
    reason_ptr: *const u8,
    reason_len: usize,
) -> c_int {
    let reason_str = if reason_ptr.is_null() || reason_len == 0 {
        ""
    } else {
        let slice = std::slice::from_raw_parts(reason_ptr, reason_len);
        std::str::from_utf8(slice).unwrap_or("")
    };

    let reason = ChangeReason::parse(reason_str).unwrap_or(ChangeReason::Time);

    c_int::from(should_auto_reload(
        autoread_enabled != 0,
        buffer_modified != 0,
        file_exists != 0,
        reason,
    ))
}

// =============================================================================
// Timestamp Check State
// =============================================================================

/// State for managing timestamp checking.
///
/// This tracks whether we've already checked timestamps to avoid
/// repeated checks (e.g., during focus events) and whether a check
/// is needed later.
#[derive(Debug, Clone, Copy, Default)]
pub struct TimestampCheckState {
    /// Whether timestamps have been checked since last typing
    pub did_check: bool,
    /// Whether a check is needed but was postponed
    pub need_check: bool,
    /// Counter for suppressing checks (e.g., during system() calls)
    pub no_check_count: i32,
}

impl TimestampCheckState {
    /// Create a new timestamp check state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if timestamp checking is currently suppressed.
    pub fn is_suppressed(&self) -> bool {
        self.no_check_count > 0
    }

    /// Increment the suppression counter.
    pub fn suppress(&mut self) {
        self.no_check_count += 1;
    }

    /// Decrement the suppression counter.
    pub fn unsuppress(&mut self) {
        if self.no_check_count > 0 {
            self.no_check_count -= 1;
        }
    }

    /// Mark that timestamps should be checked later.
    pub fn schedule_check(&mut self) {
        self.need_check = true;
    }

    /// Mark that a check was performed.
    pub fn mark_checked(&mut self) {
        self.did_check = true;
        self.need_check = false;
    }

    /// Reset state after user input.
    pub fn reset_after_input(&mut self) {
        self.did_check = false;
    }

    /// Check if a focus event should trigger a check.
    ///
    /// Returns false if we already checked and should just schedule for later.
    pub fn should_check_on_focus(&mut self) -> bool {
        if self.did_check {
            self.schedule_check();
            false
        } else {
            true
        }
    }
}

// =============================================================================
// Buffer Flags for Sync
// =============================================================================

/// Buffer flags related to file synchronization.
#[derive(Debug, Clone, Copy, Default)]
pub struct BufferSyncFlags {
    /// Buffer is being saved
    pub saving: bool,
    /// Buffer has never been edited (new file)
    pub not_edited: bool,
    /// "New file" warning has been displayed
    pub new_warning_shown: bool,
}

impl BufferSyncFlags {
    /// Create new buffer sync flags.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if the buffer should be checked for timestamp changes.
    pub fn should_check_timestamp(&self) -> bool {
        // Don't check if currently saving
        !self.saving
    }
}

// =============================================================================
// FFI: buf_store_file_info
// =============================================================================

extern "C" {
    /// Get file modification time (seconds) from a FileInfo struct.
    fn nvim_fileinfo_get_mtime(fi: *const c_void) -> i64;
    /// Get file modification time (nanoseconds) from a FileInfo struct.
    fn nvim_fileinfo_get_mtime_ns(fi: *const c_void) -> i64;
    /// Get file size in bytes from a FileInfo struct.
    #[link_name = "os_fileinfo_size"]
    fn nvim_fileinfo_get_size(fi: *const c_void) -> u64;
    /// Get file mode from a FileInfo struct.
    fn nvim_fileinfo_get_mode(fi: *const c_void) -> i32;
}

/// Store file metadata from a FileInfo struct into a buffer.
///
/// Replaces the C `buf_store_file_info` function.
///
/// # Safety
/// - `buf` must be a valid non-null pointer to a buf_T.
/// - `file_info` must be a valid non-null pointer to a FileInfo.
#[export_name = "buf_store_file_info"]
pub unsafe extern "C" fn rs_buf_store_file_info(buf: *mut c_void, file_info: *const c_void) {
    let mtime = unsafe { nvim_fileinfo_get_mtime(file_info) };
    let mtime_ns = unsafe { nvim_fileinfo_get_mtime_ns(file_info) };
    let size = unsafe { nvim_fileinfo_get_size(file_info) };
    let mode = unsafe { nvim_fileinfo_get_mode(file_info) };

    let b = unsafe { buf_mut_void(buf) };
    b.b_mtime = mtime;
    b.b_mtime_ns = mtime_ns;
    b.b_orig_size = size;
    b.b_orig_mode = mode;
}

// =============================================================================
// FFI: check_timestamps
// =============================================================================

/// Tracks whether we have already shown a "file changed" warning during the
/// current batch of `check_timestamps` / `buf_check_timestamp` calls.
///
/// Replaces the `static bool already_warned` local to fileio.c.
pub static ALREADY_WARNED: AtomicBool = AtomicBool::new(false);

extern "C" {
    /// Returns non-zero if the global `no_check_timestamps` counter is > 0.
    fn nvim_get_no_check_timestamps() -> c_int;
    /// Returns the `did_check_timestamps` flag.
    fn nvim_get_did_check_timestamps() -> bool;
    /// Sets the `did_check_timestamps` flag.
    static mut did_check_timestamps: bool;
    /// Sets `need_check_timestamps`.
    fn nvim_set_need_check_timestamps(val: c_int);
    /// Returns whether the stuff buffer is empty (direct linkage).
    #[link_name = "stuff_empty"]
    fn stuff_empty_sync() -> bool;
    /// Returns the `global_busy` flag.
    fn nvim_get_global_busy() -> bool;
    /// Returns non-zero if the typebuf has been typed.
    #[link_name = "typebuf_typed"]
    fn nvim_typebuf_typed() -> c_int;
    /// Returns the `autocmd_busy` flag.
    fn nvim_get_autocmd_busy() -> bool;
    /// Returns `curbuf->b_ro_locked`.
    fn nvim_get_curbuf_b_ro_locked() -> c_int;
    /// Returns `allbuf_lock`.
    static mut allbuf_lock: c_int;
    /// `no_wait_return` global.
    static mut no_wait_return: c_int;
    /// Returns `need_wait_return`.
    static mut need_wait_return: bool;
    /// Returns the first buffer (firstbuf).
    fn nvim_get_firstbuf() -> *mut c_void;
    /// Initializes a bufref_T (opaque) to point to buf.
    #[link_name = "set_bufref"]
    fn nvim_set_bufref(br: *mut c_void, buf: *mut c_void);
    /// Returns non-zero if the bufref still points to a valid buffer.
    fn nvim_bufref_valid(br: *mut c_void) -> c_int;
    /// Returns sizeof(bufref_T) – used to sanity-check our stack allocation.
    fn nvim_bufref_size() -> c_int;
    // Note: buf_check_timestamp is now called as rs_buf_check_timestamp (defined in this file).
    /// Calls `msg_puts("\n")`.
    fn msg_puts(s: *const c_char);
    /// Flushes the UI.
    fn ui_flush();
}

/// Check all open buffers for external file changes.
///
/// Replaces the C `check_timestamps` function in `fileio.c`.
///
/// # Safety
/// Calls into C. The C globals are accessed only on the main thread.
#[export_name = "check_timestamps"]
pub unsafe extern "C" fn rs_check_timestamps(focus: c_int) -> c_int {
    // Don't check while system() or another low-level function may cause
    // us to lose and gain focus.
    if nvim_get_no_check_timestamps() > 0 {
        return 0;
    }

    // Avoid doing a check twice.  The OK/Reload dialog can cause a focus
    // event and we would keep on checking if the file is steadily growing.
    // Do check again after typing something.
    if focus != 0 && nvim_get_did_check_timestamps() {
        nvim_set_need_check_timestamps(1);
        return 0;
    }

    let mut didit: c_int = 0;

    if !stuff_empty_sync()
        || nvim_get_global_busy()
        || nvim_typebuf_typed() == 0
        || nvim_get_autocmd_busy()
        || nvim_get_curbuf_b_ro_locked() > 0
        || allbuf_lock > 0
    {
        // Check later when conditions are safe.
        nvim_set_need_check_timestamps(1);
    } else {
        no_wait_return += 1;
        did_check_timestamps = true;
        ALREADY_WARNED.store(false, Ordering::Relaxed);

        // bufref_T is { buf_T*, int, int } = 16 bytes. We use [u64; 2]
        // which is 16 bytes and pointer-aligned.
        debug_assert_eq!(nvim_bufref_size() as usize, 16);
        let mut bufref: [u64; 2] = [0; 2];
        let bufref_ptr = bufref.as_mut_ptr() as *mut c_void;

        let mut buf = nvim_get_firstbuf();
        while !buf.is_null() {
            // Only check buffers in a window.
            if unsafe { bref_void(buf as *const c_void).b_nwindows } > 0 {
                nvim_set_bufref(bufref_ptr, buf);
                let n = rs_buf_check_timestamp(buf);
                if n > didit {
                    didit = n;
                }
                if n > 0 && nvim_bufref_valid(bufref_ptr) == 0 {
                    // Autocommands have removed the buffer, start at the first one again.
                    buf = nvim_get_firstbuf();
                    continue;
                }
            }
            buf = unsafe { bref_void(buf as *const c_void).b_next };
        }

        no_wait_return -= 1;
        nvim_set_need_check_timestamps(0);
        if need_wait_return && didit == 2 {
            // make sure msg isn't overwritten
            msg_puts(c"\n".as_ptr());
            ui_flush();
        }
    }
    didit
}

/// Get the current value of `already_warned` (used by `buf_check_timestamp`).
///
/// # Safety
/// Must be called from the main thread only.
#[no_mangle]
pub extern "C" fn rs_get_already_warned() -> c_int {
    c_int::from(ALREADY_WARNED.load(Ordering::Relaxed))
}

/// Set the `already_warned` flag (used by `buf_check_timestamp`).
///
/// # Safety
/// Must be called from the main thread only.
#[no_mangle]
pub extern "C" fn rs_set_already_warned(val: c_int) {
    ALREADY_WARNED.store(val != 0, Ordering::Relaxed);
}

// =============================================================================
// FFI: buf_check_timestamp
// =============================================================================

/// Constants for reload decision.
const RELOAD_NONE: c_int = 0;
const RELOAD_NORMAL: c_int = 1;
const RELOAD_DETECT: c_int = 2;

/// Buffer flags related to new-file state.
const BF_NEW: c_int = 0x04;
const BF_NEW_W: c_int = 0x20;
const BF_NOTEDITED: c_int = 0x08;

/// UNDO_HASH_SIZE (from undo_defs.h)
const UNDO_HASH_SIZE: usize = 32;

extern "C" {
    /// Returns true if buf has type 'normal'.
    #[link_name = "rs_bt_normal"]
    fn nvim_bt_normal(buf: *mut c_void) -> bool;
    /// Global p_ar (autoread option).
    static p_ar: i64;
    // Note: rs_buf_store_file_info is defined in this file, not an extern.
    /// os_fileinfo wrapper: fills metadata, returns 1 on success.
    fn nvim_os_fileinfo(
        fname: *const c_char,
        mtime_sec: *mut i64,
        mtime_ns: *mut i64,
        size: *mut u64,
        mode: *mut i32,
    ) -> c_int;
    /// os_isdir: returns true if name is a directory.
    #[link_name = "os_isdir"]
    fn nvim_os_isdir2(name: *const c_char) -> bool;
    /// bufIsChanged wrapper.
    fn nvim_buf_is_changed(buf: *mut c_void) -> c_int;
    /// buf_contents_changed: returns true if buffer contents changed on disk.
    #[link_name = "buf_contents_changed"]
    fn nvim_buf_contents_changed(buf: *mut c_void) -> bool;
    /// set_vim_var_string(idx, val, len): set a v: variable string.
    fn set_vim_var_string(idx: c_int, val: *const c_char, len: isize);
    /// get_vim_var_str(idx): get a v: variable string.
    fn get_vim_var_str(idx: c_int) -> *const c_char;
    // Note: nvim_get_autocmd_busy, nvim_set_allbuf_lock, nvim_get_allbuf_lock
    // are declared in the check_timestamps extern block above.
    /// Emit error message.
    fn emsg(msg: *const c_char);
    /// Displays a warning dialog and returns the button pressed.
    fn nvim_do_dialog_file_changed(tbuf: *const c_char) -> c_int;
    /// msg_start().
    fn msg_start();
    /// msg_puts_hl(s, hl_id, hist)
    #[link_name = "msg_puts_hl"]
    fn nvim_msg_puts_hl(s: *const c_char, hl_id: c_int, hist: bool);
    /// msg_clr_eos().
    fn msg_clr_eos();
    /// msg_end().
    #[link_name = "msg_end"]
    fn nvim_msg_end_wrap();
    /// Returns emsg_silent global.
    static emsg_silent: c_int;
    /// Returns in_assert_fails global.
    static in_assert_fails: bool;
    /// ui_has(extension) -> bool
    #[link_name = "ui_has"]
    fn nvim_ui_has(extension: c_int) -> bool;
    /// os_delay(ms, ignoreinput)
    #[link_name = "os_delay"]
    fn nvim_os_delay(ms: u64, ignoreinput: bool);
    /// Sets redraw_cmdline. (defined in window/src/globals.rs, takes bool)
    fn nvim_set_redraw_cmdline(val: bool);
    /// State global.
    static mut State: c_int;
    /// home_replace_save: returns newly allocated string (caller must xfree).
    #[link_name = "home_replace_save"]
    fn nvim_home_replace_save(buf: *const c_void, fname: *const c_char) -> *mut c_char;
    /// xmalloc.
    fn xmalloc(size: usize) -> *mut c_char;
    /// xfree.
    fn xfree(ptr: *mut c_void);
    /// xstrlcat.
    fn xstrlcat(dst: *mut c_char, src: *const c_char, dstlen: usize) -> usize;
    /// snprintf.
    fn snprintf(buf: *mut c_char, n: usize, fmt: *const c_char, ...) -> c_int;
    /// buf_reload (implemented by Rust rs_buf_reload).
    #[link_name = "buf_reload"]
    fn nvim_buf_reload(buf: *mut c_void, orig_mode: c_int, reload_options: c_int);
    /// u_compute_hash wrapper.
    #[link_name = "u_compute_hash"]
    fn nvim_u_compute_hash(buf: *mut c_void, hash: *mut u8);
    /// u_write_undo: write undo file.
    #[link_name = "u_write_undo"]
    fn nvim_u_write_undo(name: *const c_char, forceit: c_int, buf: *mut c_void, hash: *mut u8);
    /// os_path_exists (from undo.c).
    fn nvim_os_path_exists(path: *const c_char) -> bool;
    /// rs_time_differs: compare file time vs stored.
    fn rs_time_differs(
        file_sec: i64,
        file_nsec: i64,
        mtime: i64,
        mtime_ns: i64,
        fat_tolerance: c_int,
    ) -> c_int;
    /// apply_autocmds(event, fname, fname_io, force, buf) -> bool.
    fn apply_autocmds(
        event: c_int,
        fname: *const c_char,
        fname_io: *const c_char,
        force: bool,
        buf: *mut c_void,
    ) -> bool;
    // Note: nvim_bufref_valid and nvim_set_bufref are declared in
    // the check_timestamps extern block above.
}

// On Linux, FAT tolerance is enabled (1); on other platforms, 0.
#[cfg(target_os = "linux")]
const FAT_TOLERANCE: c_int = 1;
#[cfg(not(target_os = "linux"))]
const FAT_TOLERANCE: c_int = 0;

// EVENT_FILECHANGEDSHELL = 20, EVENT_FILECHANGEDSHELLPOST = 21
// (from auevents_enum.generated.h in the build directory)
const EVENT_FILECHANGEDSHELL: c_int = 20;
const EVENT_FILECHANGEDSHELLPOST: c_int = 21;

// v: variable indices (from eval_defs.h VimVarIndex enum)
const VV_WARNINGMSG: c_int = 4;
const VV_FCS_REASON: c_int = 38;
const VV_FCS_CHOICE: c_int = 39;

// Mode constants from state_defs.h
const MODE_NORMAL_BUSY: c_int = 0x1001;
const MODE_CMDLINE: c_int = 0x08;

/// Re-entrancy guard for `rs_buf_check_timestamp`.
static BUF_CHECK_BUSY: AtomicBool = AtomicBool::new(false);

/// Check if buffer "buf" has been changed externally.
///
/// Returns 1 if a changed buffer was found, 2 if a message has been
/// displayed, or 0 otherwise.
///
/// Replaces the C `buf_check_timestamp` function in `fileio.c`.
///
/// # Safety
/// Calls into C. Must be called from the main thread only.
#[export_name = "buf_check_timestamp"]
pub unsafe extern "C" fn rs_buf_check_timestamp(buf: *mut c_void) -> c_int {
    // Re-entrancy guard: set busy = true while we run
    if BUF_CHECK_BUSY.swap(true, Ordering::Relaxed) {
        return 0;
    }

    let result = buf_check_timestamp_inner(buf);

    BUF_CHECK_BUSY.store(false, Ordering::Relaxed);
    result
}

unsafe fn buf_check_timestamp_inner(buf: *mut c_void) -> c_int {
    // If terminal, no filename, not loaded, not normal buftype, saving, or re-entrant: skip.
    let br = bref_void(buf as *const c_void);
    if !br.terminal.is_null()
        || br.b_ffname.is_null()
        || br.ml_mfp.is_null()
        || !nvim_bt_normal(buf)
        || br.b_saving != 0
    {
        return 0;
    }

    let mut retval: c_int = 0;
    let mut mesg: *const c_char = std::ptr::null();
    let mut mesg2: *const c_char = c"".as_ptr();
    let mut helpmesg = false;
    let mut can_reload = false;
    let mut reload: c_int = RELOAD_NONE;

    let br = bref_void(buf as *const c_void);
    let orig_size = br.b_orig_size;
    let orig_mode = br.b_orig_mode;
    let b_flags = br.b_flags;

    // Set up a bufref to detect if buf gets deleted by autocmds
    let mut bufref: [u64; 2] = [0; 2];
    let bufref_ptr = bufref.as_mut_ptr() as *mut c_void;
    nvim_set_bufref(bufref_ptr, buf);

    let ffname = br.b_ffname;

    let b_mtime = br.b_mtime;
    let b_mtime_ns = br.b_mtime_ns;

    // Check if timestamp changed (file modified since we read it)
    if (b_flags & BF_NOTEDITED) == 0 && b_mtime != 0 {
        let mut fi_mtime_sec: i64 = 0;
        let mut fi_mtime_ns: i64 = 0;
        let mut fi_size: u64 = 0;
        let mut fi_mode: i32 = 0;
        let file_info_ok = nvim_os_fileinfo(
            ffname,
            &mut fi_mtime_sec,
            &mut fi_mtime_ns,
            &mut fi_size,
            &mut fi_mode,
        ) != 0;

        let time_differs = if file_info_ok {
            rs_time_differs(
                fi_mtime_sec,
                fi_mtime_ns,
                b_mtime,
                b_mtime_ns,
                FAT_TOLERANCE,
            ) != 0
        } else {
            false
        };
        let mode_differs = file_info_ok && (fi_mode as c_int) != orig_mode as c_int;

        if !file_info_ok || time_differs || mode_differs {
            let prev_b_mtime = b_mtime;

            retval = 1;

            // Update stored metadata immediately to stop further warnings
            // (e.g., when firing FileChangedShell autocmd)
            if !file_info_ok {
                let bm = buf_mut_void(buf);
                bm.b_mtime = -1;
                bm.b_orig_size = 0;
                bm.b_orig_mode = 0;
            } else {
                let bm = buf_mut_void(buf);
                bm.b_mtime = fi_mtime_sec;
                bm.b_mtime_ns = fi_mtime_ns;
                bm.b_orig_size = fi_size;
                bm.b_orig_mode = fi_mode;
            }

            let b_fname = bref_void(buf as *const c_void).b_fname;

            if nvim_os_isdir2(b_fname) {
                // Don't do anything for a directory.
            } else if (bref_void(buf as *const c_void).b_p_ar >= 0
                && bref_void(buf as *const c_void).b_p_ar != 0
                || bref_void(buf as *const c_void).b_p_ar < 0 && p_ar != 0)
                && nvim_buf_is_changed(buf) == 0
                && file_info_ok
            {
                // autoread: buffer not modified, file still exists
                reload = RELOAD_NORMAL;
            } else {
                // Determine reason for the change
                let reason: *const c_char = if !file_info_ok {
                    c"deleted".as_ptr()
                } else if nvim_buf_is_changed(buf) != 0 {
                    c"conflict".as_ptr()
                } else if orig_size != bref_void(buf as *const c_void).b_orig_size
                    || nvim_buf_contents_changed(buf)
                {
                    c"changed".as_ptr()
                } else if orig_mode != bref_void(buf as *const c_void).b_orig_mode {
                    c"mode".as_ptr()
                } else {
                    c"time".as_ptr()
                };

                // Only give warning if no FileChangedShell autocmds.
                // Avoid re-entrancy by temporarily setting busy on the Rust side.
                // (BUF_CHECK_BUSY is already set at the outer level; the inner guard
                //  in rs_buf_check_timestamp handles recursion. We just need to set
                //  allbuf_lock here to match C behavior.)
                set_vim_var_string(VV_FCS_REASON, reason, -1);
                set_vim_var_string(VV_FCS_CHOICE, c"".as_ptr(), -1);
                let old_allbuf_lock = allbuf_lock;
                allbuf_lock = old_allbuf_lock + 1;
                let n = apply_autocmds(EVENT_FILECHANGEDSHELL, b_fname, b_fname, false, buf);
                allbuf_lock = old_allbuf_lock;
                // show_mesg tracks whether to show the user a warning message.
                // It mirrors the C idiom: `bool n = apply_autocmds(); if (n) { ...; if (ask) n=false; else return 2; } if (!n) { show message; }`
                let mut show_mesg = !n;
                if n {
                    if nvim_bufref_valid(bufref_ptr) == 0 {
                        emsg(c"E246: FileChangedShell autocommand deleted buffer".as_ptr());
                    }
                    let s = get_vim_var_str(VV_FCS_CHOICE);
                    let choice = if s.is_null() {
                        ""
                    } else {
                        std::ffi::CStr::from_ptr(s).to_str().unwrap_or("")
                    };
                    // reason[0] = 'd' means deleted
                    let first_char = *reason as u8;
                    if choice == "reload" && first_char != b'd' {
                        reload = RELOAD_NORMAL;
                    } else if choice == "edit" {
                        reload = RELOAD_DETECT;
                    } else if choice == "ask" {
                        show_mesg = true; // fall through to show dialog
                    } else {
                        return 2;
                    }
                }
                if show_mesg {
                    let first_char = *reason as u8;
                    if first_char == b'd' {
                        // Only give the message once.
                        if prev_b_mtime != -1 {
                            mesg = c"E211: File \"%s\" no longer available".as_ptr();
                        }
                    } else {
                        helpmesg = true;
                        can_reload = true;

                        // Check 3rd char: 'n' = "conflict", 'h' = "changed"
                        let reason_bytes = std::ffi::CStr::from_ptr(reason).to_bytes();
                        if reason_bytes.get(2) == Some(&b'n') {
                            mesg = c"W12: Warning: File \"%s\" has changed and the buffer was changed in Vim as well".as_ptr();
                            mesg2 = c"See \":help W12\" for more info.".as_ptr();
                        } else if reason_bytes.get(1) == Some(&b'h') {
                            mesg = c"W11: Warning: File \"%s\" has changed since editing started"
                                .as_ptr();
                            mesg2 = c"See \":help W11\" for more info.".as_ptr();
                        } else if first_char == b'm' {
                            mesg = c"W16: Warning: Mode of file \"%s\" has changed since editing started".as_ptr();
                            mesg2 = c"See \":help W16\" for more info.".as_ptr();
                        } else {
                            // Only timestamp changed: store to avoid check_mtime() warnings.
                            {
                                let bm = buf_mut_void(buf);
                                bm.b_mtime_read = bm.b_mtime;
                                bm.b_mtime_read_ns = bm.b_mtime_ns;
                            }
                        }
                    }
                }
            }
        }
    } else if (b_flags & BF_NEW) != 0 && (b_flags & BF_NEW_W) == 0 && nvim_os_path_exists(ffname) {
        retval = 1;
        mesg = c"W13: Warning: File \"%s\" has been created after editing started".as_ptr();
        buf_mut_void(buf).b_flags = b_flags | BF_NEW_W;
        can_reload = true;
    }

    if !mesg.is_null() {
        let b_fname = bref_void(buf as *const c_void).b_fname;
        let path = nvim_home_replace_save(buf, b_fname);

        if !helpmesg {
            mesg2 = c"".as_ptr();
        }

        let mesg_len = libc::strlen(mesg);
        let path_len = libc::strlen(path);
        let mesg2_len = libc::strlen(mesg2);
        let tbuf_len = path_len + mesg_len + mesg2_len + 2;
        let tbuf = xmalloc(tbuf_len);
        snprintf(tbuf, tbuf_len, mesg, path);

        // Set VV_WARNINGMSG before appending mesg2
        set_vim_var_string(VV_WARNINGMSG, tbuf, -1);

        if can_reload {
            if *mesg2 != 0 {
                xstrlcat(tbuf, c"\n".as_ptr(), tbuf_len - 1);
                xstrlcat(tbuf, mesg2, tbuf_len - 1);
            }
            let choice = nvim_do_dialog_file_changed(tbuf);
            match choice {
                2 => reload = RELOAD_NORMAL,
                3 => reload = RELOAD_DETECT,
                _ => {}
            }
        } else {
            let state = State;
            if state > MODE_NORMAL_BUSY
                || (state & MODE_CMDLINE) != 0
                || ALREADY_WARNED.load(Ordering::Relaxed)
            {
                if *mesg2 != 0 {
                    xstrlcat(tbuf, c"; ".as_ptr(), tbuf_len - 1);
                    xstrlcat(tbuf, mesg2, tbuf_len - 1);
                }
                emsg(tbuf);
                retval = 2;
            } else if !nvim_get_autocmd_busy() {
                msg_start();
                nvim_msg_puts_hl(tbuf, 6, true); // HLF_E
                if *mesg2 != 0 {
                    nvim_msg_puts_hl(mesg2, 25, true); // HLF_W
                }
                msg_clr_eos();
                nvim_msg_end_wrap();
                if emsg_silent == 0 && !in_assert_fails && !nvim_ui_has(4) {
                    ui_flush();
                    nvim_os_delay(1004, true);
                    nvim_set_redraw_cmdline(false);
                }
                ALREADY_WARNED.store(true, Ordering::Relaxed);
            }
        }

        xfree(path as *mut c_void);
        xfree(tbuf as *mut c_void);
    }

    if reload != RELOAD_NONE {
        let reload_options = if reload == RELOAD_DETECT { 1 } else { 0 };
        nvim_buf_reload(buf, orig_mode, reload_options);
        let bsync = bref_void(buf as *const c_void);
        if bsync.b_p_udf != 0 && !bsync.b_ffname.is_null() {
            let mut hash: [u8; UNDO_HASH_SIZE] = [0; UNDO_HASH_SIZE];
            nvim_u_compute_hash(buf, hash.as_mut_ptr());
            nvim_u_write_undo(std::ptr::null(), 0, buf, hash.as_mut_ptr());
        }
    }

    // Trigger FileChangedShellPost when the file was changed in any way.
    if nvim_bufref_valid(bufref_ptr) != 0 && retval != 0 {
        let b_fname = bref_void(buf as *const c_void).b_fname;
        apply_autocmds(EVENT_FILECHANGEDSHELLPOST, b_fname, b_fname, false, buf);
    }

    retval
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_reason_strings() {
        assert_eq!(ChangeReason::Deleted.as_str(), "deleted");
        assert_eq!(ChangeReason::Conflict.as_str(), "conflict");
        assert_eq!(ChangeReason::Changed.as_str(), "changed");
        assert_eq!(ChangeReason::Mode.as_str(), "mode");
        assert_eq!(ChangeReason::Time.as_str(), "time");

        assert_eq!(ChangeReason::parse("deleted"), Some(ChangeReason::Deleted));
        assert_eq!(
            ChangeReason::parse("conflict"),
            Some(ChangeReason::Conflict)
        );
        assert_eq!(ChangeReason::parse("changed"), Some(ChangeReason::Changed));
        assert_eq!(ChangeReason::parse("mode"), Some(ChangeReason::Mode));
        assert_eq!(ChangeReason::parse("time"), Some(ChangeReason::Time));
        assert_eq!(ChangeReason::parse("unknown"), None);
    }

    #[test]
    fn test_file_status_is_changed() {
        assert!(!FileStatus::Unchanged.is_changed());
        assert!(FileStatus::Deleted.is_changed());
        assert!(FileStatus::TimeChanged.is_changed());
        assert!(FileStatus::ModeChanged.is_changed());
        assert!(FileStatus::SizeChanged.is_changed());
        assert!(FileStatus::ContentsChanged.is_changed());
    }

    #[test]
    fn test_determine_change_reason() {
        // Deleted file
        assert_eq!(
            determine_change_reason(false, false, false, false, false),
            ChangeReason::Deleted
        );

        // Buffer modified = conflict
        assert_eq!(
            determine_change_reason(true, true, true, false, false),
            ChangeReason::Conflict
        );

        // Size changed = contents changed
        assert_eq!(
            determine_change_reason(true, false, true, false, false),
            ChangeReason::Changed
        );

        // Contents changed by checksum
        assert_eq!(
            determine_change_reason(true, false, false, false, true),
            ChangeReason::Changed
        );

        // Mode changed only
        assert_eq!(
            determine_change_reason(true, false, false, true, false),
            ChangeReason::Mode
        );

        // Only timestamp changed
        assert_eq!(
            determine_change_reason(true, false, false, false, false),
            ChangeReason::Time
        );
    }

    #[test]
    fn test_file_changed_choice() {
        assert_eq!(FileChangedChoice::parse(""), Some(FileChangedChoice::Ok));
        assert_eq!(FileChangedChoice::parse("ok"), Some(FileChangedChoice::Ok));
        assert_eq!(
            FileChangedChoice::parse("reload"),
            Some(FileChangedChoice::Reload)
        );
        assert_eq!(
            FileChangedChoice::parse("edit"),
            Some(FileChangedChoice::Edit)
        );
        assert_eq!(
            FileChangedChoice::parse("ask"),
            Some(FileChangedChoice::Ask)
        );
        assert_eq!(FileChangedChoice::parse("invalid"), None);

        assert_eq!(FileChangedChoice::Ok.as_str(), "");
        assert_eq!(FileChangedChoice::Reload.as_str(), "reload");
        assert_eq!(FileChangedChoice::Edit.as_str(), "edit");
        assert_eq!(FileChangedChoice::Ask.as_str(), "ask");
    }

    #[test]
    fn test_should_auto_reload() {
        // Should reload: autoread enabled, no modifications, file exists
        assert!(should_auto_reload(true, false, true, ChangeReason::Changed));
        assert!(should_auto_reload(true, false, true, ChangeReason::Time));
        assert!(should_auto_reload(true, false, true, ChangeReason::Mode));

        // Should NOT reload: file deleted
        assert!(!should_auto_reload(
            true,
            false,
            false,
            ChangeReason::Deleted
        ));

        // Should NOT reload: buffer modified (conflict)
        assert!(!should_auto_reload(true, true, true, ChangeReason::Changed));

        // Should NOT reload: autoread disabled
        assert!(!should_auto_reload(
            false,
            false,
            true,
            ChangeReason::Changed
        ));
    }

    #[test]
    fn test_timestamp_check_state() {
        let mut state = TimestampCheckState::new();

        // Initial state
        assert!(!state.is_suppressed());
        assert!(!state.did_check);
        assert!(!state.need_check);

        // Suppress/unsuppress
        state.suppress();
        assert!(state.is_suppressed());
        state.suppress();
        assert_eq!(state.no_check_count, 2);
        state.unsuppress();
        assert!(state.is_suppressed());
        state.unsuppress();
        assert!(!state.is_suppressed());

        // Check scheduling
        state.schedule_check();
        assert!(state.need_check);
        state.mark_checked();
        assert!(!state.need_check);
        assert!(state.did_check);

        // Reset after input
        state.reset_after_input();
        assert!(!state.did_check);
    }

    #[test]
    fn test_timestamp_check_state_focus() {
        let mut state = TimestampCheckState::new();

        // First focus event should trigger check
        assert!(state.should_check_on_focus());

        // After checking, focus should just schedule
        state.mark_checked();
        assert!(!state.should_check_on_focus());
        assert!(state.need_check);

        // After user input, should check again
        state.reset_after_input();
        state.need_check = false;
        assert!(state.should_check_on_focus());
    }

    #[test]
    fn test_file_info_time_differs() {
        let file_info = FileInfo::new(1000, 500, 1024, 0o644, true);

        // Exact match
        assert!(!file_info.time_differs(1000, 500, false));
        assert!(!file_info.time_differs(1000, 500, true));

        // Nanosec differs
        assert!(file_info.time_differs(1000, 501, false));
        assert!(file_info.time_differs(1000, 501, true));

        // Sec differs by 1, FAT tolerance
        assert!(file_info.time_differs(999, 500, false));
        assert!(!file_info.time_differs(999, 500, true));
    }

    #[test]
    fn test_buffer_file_info_update() {
        let mut buf_info = BufferFileInfo::default();
        let file_info = FileInfo::new(1234, 5678, 2048, 0o755, true);

        buf_info.update_from(&file_info);

        assert_eq!(buf_info.mtime, 1234);
        assert_eq!(buf_info.mtime_ns, 5678);
        assert_eq!(buf_info.orig_size, 2048);
        assert_eq!(buf_info.orig_mode, 0o755);
    }

    #[test]
    fn test_buffer_sync_flags() {
        let mut flags = BufferSyncFlags::new();

        // Default state should allow checking
        assert!(flags.should_check_timestamp());

        // Saving should prevent checking
        flags.saving = true;
        assert!(!flags.should_check_timestamp());
    }
}
