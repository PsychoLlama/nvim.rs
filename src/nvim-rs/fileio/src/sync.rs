//! Buffer-file synchronization utilities.
//!
//! This module provides:
//! - File change detection (timestamp, size, mode)
//! - Change reason classification
//! - Reload decision logic

use std::ffi::c_int;

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
