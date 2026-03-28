//! Buffer-file synchronization utilities.
//!
//! This module provides:
//! - File change detection (timestamp, size, mode)
//! - Change reason classification
//! - Reload decision logic

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
    fn nvim_fileinfo_get_size(fi: *const c_void) -> u64;
    /// Get file mode from a FileInfo struct.
    fn nvim_fileinfo_get_mode(fi: *const c_void) -> i32;
    /// Set buf->b_mtime (in memline_shim.c).
    fn nvim_buf_set_b_mtime(buf: *mut c_void, val: i64);
    /// Set buf->b_mtime_ns (in memline_shim.c).
    fn nvim_buf_set_b_mtime_ns(buf: *mut c_void, val: i64);
    /// Set buf->b_orig_size (in memline_shim.c, takes int64_t).
    fn nvim_buf_set_b_orig_size(buf: *mut c_void, val: i64);
    /// Set buf->b_orig_mode (in buffer_shim.c).
    fn nvim_buf_set_b_orig_mode(buf: *mut c_void, val: i32);
}

/// Store file metadata from a FileInfo struct into a buffer.
///
/// Replaces the C `buf_store_file_info` function.
///
/// # Safety
/// - `buf` must be a valid non-null pointer to a buf_T.
/// - `file_info` must be a valid non-null pointer to a FileInfo.
#[no_mangle]
pub unsafe extern "C" fn rs_buf_store_file_info(buf: *mut c_void, file_info: *const c_void) {
    let mtime = unsafe { nvim_fileinfo_get_mtime(file_info) };
    let mtime_ns = unsafe { nvim_fileinfo_get_mtime_ns(file_info) };
    let size = unsafe { nvim_fileinfo_get_size(file_info) };
    let mode = unsafe { nvim_fileinfo_get_mode(file_info) };

    unsafe { nvim_buf_set_b_mtime(buf, mtime) };
    unsafe { nvim_buf_set_b_mtime_ns(buf, mtime_ns) };
    unsafe { nvim_buf_set_b_orig_size(buf, size as i64) };
    unsafe { nvim_buf_set_b_orig_mode(buf, mode) };
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
    fn nvim_set_did_check_timestamps(val: bool);
    /// Sets `need_check_timestamps`.
    fn nvim_set_need_check_timestamps(val: c_int);
    /// Returns non-zero if the stuff buffer is empty.
    fn nvim_stuff_empty() -> c_int;
    /// Returns the `global_busy` flag.
    fn nvim_get_global_busy() -> bool;
    /// Returns non-zero if the typebuf has been typed.
    fn nvim_typebuf_typed() -> c_int;
    /// Returns the `autocmd_busy` flag.
    fn nvim_get_autocmd_busy() -> bool;
    /// Returns `curbuf->b_ro_locked`.
    fn nvim_get_curbuf_b_ro_locked() -> c_int;
    /// Returns `allbuf_lock`.
    fn nvim_get_allbuf_lock() -> c_int;
    /// Returns `no_wait_return`.
    fn nvim_get_no_wait_return() -> c_int;
    /// Sets `no_wait_return`.
    fn nvim_set_no_wait_return(val: c_int);
    /// Returns `need_wait_return`.
    fn nvim_get_need_wait_return() -> bool;
    /// Returns the first buffer (firstbuf).
    fn nvim_get_firstbuf() -> *mut c_void;
    /// Returns `buf->b_next`.
    fn nvim_buf_get_b_next(buf: *mut c_void) -> *mut c_void;
    /// Returns `buf->b_nwindows`.
    fn nvim_buf_get_nwindows(buf: *mut c_void) -> c_int;
    /// Initializes a bufref_T (opaque) to point to buf.
    fn nvim_set_bufref(br: *mut c_void, buf: *mut c_void);
    /// Returns non-zero if the bufref still points to a valid buffer.
    fn nvim_bufref_valid(br: *mut c_void) -> c_int;
    /// Returns sizeof(bufref_T) – used to sanity-check our stack allocation.
    fn nvim_bufref_size() -> c_int;
    /// Calls `buf_check_timestamp` on a single buffer.
    fn buf_check_timestamp(buf: *mut c_void) -> c_int;
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
#[no_mangle]
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

    if nvim_stuff_empty() == 0
        || nvim_get_global_busy()
        || nvim_typebuf_typed() == 0
        || nvim_get_autocmd_busy()
        || nvim_get_curbuf_b_ro_locked() > 0
        || nvim_get_allbuf_lock() > 0
    {
        // Check later when conditions are safe.
        nvim_set_need_check_timestamps(1);
    } else {
        nvim_set_no_wait_return(nvim_get_no_wait_return() + 1);
        nvim_set_did_check_timestamps(true);
        ALREADY_WARNED.store(false, Ordering::Relaxed);

        // bufref_T is { buf_T*, int, int } = 16 bytes. We use [u64; 2]
        // which is 16 bytes and pointer-aligned.
        debug_assert_eq!(nvim_bufref_size() as usize, 16);
        let mut bufref: [u64; 2] = [0; 2];
        let bufref_ptr = bufref.as_mut_ptr() as *mut c_void;

        let mut buf = nvim_get_firstbuf();
        while !buf.is_null() {
            // Only check buffers in a window.
            if nvim_buf_get_nwindows(buf) > 0 {
                nvim_set_bufref(bufref_ptr, buf);
                let n = buf_check_timestamp(buf);
                if n > didit {
                    didit = n;
                }
                if n > 0 && nvim_bufref_valid(bufref_ptr) == 0 {
                    // Autocommands have removed the buffer, start at the first one again.
                    buf = nvim_get_firstbuf();
                    continue;
                }
            }
            buf = nvim_buf_get_b_next(buf);
        }

        nvim_set_no_wait_return(nvim_get_no_wait_return() - 1);
        nvim_set_need_check_timestamps(0);
        if nvim_get_need_wait_return() && didit == 2 {
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
