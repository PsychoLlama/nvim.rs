//! `:write` command implementation.
//!
//! The `:write` command writes buffer content to a file.
//!
//! ## Usage
//! - `:w[rite]` - Write current buffer
//! - `:w[rite] {file}` - Write to specified file
//! - `:w[rite]!` - Force write (overwrite readonly)
//! - `:{range}w[rite] [file]` - Write specified lines
//! - `:w[rite] >> {file}` - Append to file
//! - `:w[rite] !{cmd}` - Write to shell command (filter)
//! - `:up[date]` - Write only if modified
//! - `:sav[eas] {file}` - Write to file and change buffer name
//!
//! ## Implementation Notes
//!
//! The actual file writing is performed by Neovim's `buf_write()` function.
//! This module provides:
//! - Type definitions for write operations
//! - Validation utilities
//! - Helper functions for the C implementation

use std::ffi::{c_char, c_int};

use crate::range::{LineNr, LineRange};
use crate::{BufHandle, ExArgHandle};

/// Result of a write operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum WriteResult {
    /// Write succeeded
    Ok = 0,
    /// File is readonly
    Readonly = 1,
    /// Directory does not exist
    NoDirectory = 2,
    /// Permission denied
    PermissionDenied = 3,
    /// File already exists (and no ! given)
    FileExists = 4,
    /// Write was interrupted
    Interrupted = 5,
    /// Disk full or quota exceeded
    NoSpace = 6,
    /// Buffer not modified (for :update)
    NotModified = 7,
    /// Other error
    Error = 99,
}

impl WriteResult {
    /// Check if the write was successful.
    #[inline]
    #[must_use]
    pub const fn is_ok(self) -> bool {
        matches!(self, WriteResult::Ok)
    }

    /// Check if this result indicates the write was skipped (not an error).
    #[inline]
    #[must_use]
    pub const fn is_skipped(self) -> bool {
        matches!(self, WriteResult::NotModified)
    }

    /// Convert from C integer return value (0 = success, non-zero = error).
    #[inline]
    #[must_use]
    pub fn from_c_ok_fail(value: c_int) -> Self {
        if value == 0 {
            WriteResult::Ok
        } else {
            WriteResult::Error
        }
    }

    /// Convert to C integer for return.
    #[inline]
    #[must_use]
    pub fn to_c(self) -> c_int {
        self as c_int
    }
}

/// Options for the `:write` command.
#[derive(Debug, Clone, Default)]
pub struct WriteOptions {
    /// Range of lines to write.
    pub range: LineRange,
    /// Force write (ignore readonly, overwrite existing).
    pub force: bool,
    /// Append to file (>>).
    pub append: bool,
    /// Write to filter command.
    pub filter: bool,
    /// Force binary mode.
    pub force_binary: bool,
    /// Force text mode.
    pub force_text: bool,
    /// Specific encoding to use.
    pub encoding: Option<String>,
    /// Create parent directories if needed (++p).
    pub mkdir_p: bool,
}

impl WriteOptions {
    /// Create options for writing the whole buffer.
    #[must_use]
    pub fn whole_buffer(line_count: LineNr) -> Self {
        Self {
            range: LineRange::whole_buffer(line_count),
            ..Default::default()
        }
    }

    /// Create options for writing a specific range.
    #[must_use]
    pub fn with_range(range: LineRange) -> Self {
        Self {
            range,
            ..Default::default()
        }
    }

    /// Create options for appending to a file.
    #[must_use]
    pub fn append_to(range: LineRange) -> Self {
        Self {
            range,
            append: true,
            ..Default::default()
        }
    }

    /// Create options for force-writing.
    #[must_use]
    pub fn force(mut self) -> Self {
        self.force = true;
        self
    }

    /// Set the range.
    #[must_use]
    pub fn range(mut self, range: LineRange) -> Self {
        self.range = range;
        self
    }
}

/// Mode for the write operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WriteMode {
    /// Normal write
    #[default]
    Normal,
    /// Only write if buffer is modified (:update)
    Update,
    /// Write and change buffer name (:saveas)
    SaveAs,
}

/// Error type for write operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WriteError {
    /// Invalid line range.
    InvalidRange,
    /// File path is empty.
    EmptyPath,
    /// Buffer is readonly.
    Readonly,
    /// File already exists.
    FileExists(String),
    /// Permission denied.
    PermissionDenied(String),
    /// Parent directory does not exist.
    NoDirectory(String),
}

impl std::fmt::Display for WriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WriteError::InvalidRange => write!(f, "invalid range"),
            WriteError::EmptyPath => write!(f, "empty file path"),
            WriteError::Readonly => write!(f, "buffer is readonly"),
            WriteError::FileExists(path) => write!(f, "file already exists: {path}"),
            WriteError::PermissionDenied(path) => write!(f, "permission denied: {path}"),
            WriteError::NoDirectory(path) => write!(f, "directory does not exist: {path}"),
        }
    }
}

impl std::error::Error for WriteError {}

/// Validate a write range against buffer bounds.
///
/// # Arguments
/// * `range` - The line range to validate
/// * `line_count` - Total lines in the buffer
///
/// # Returns
/// A clamped valid range, or an error if the range is completely invalid.
pub fn validate_write_range(range: LineRange, line_count: LineNr) -> Result<LineRange, WriteError> {
    if line_count == 0 {
        // Empty buffer - any write is technically valid (writes nothing)
        return Ok(LineRange::empty());
    }

    let clamped = range.clamp(line_count);
    if clamped.is_empty() && !range.is_empty() {
        // The range was non-empty but clamped to empty - that's invalid
        return Err(WriteError::InvalidRange);
    }

    Ok(clamped)
}

/// Check if a write should proceed for :update command.
///
/// # Arguments
/// * `is_modified` - Whether the buffer has been modified
/// * `mode` - The write mode
///
/// # Returns
/// `true` if the write should proceed, `false` if it should be skipped.
#[inline]
#[must_use]
pub fn should_write(is_modified: bool, mode: WriteMode) -> bool {
    match mode {
        WriteMode::Update => is_modified,
        WriteMode::Normal | WriteMode::SaveAs => true,
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Validate write range and return validity.
///
/// Returns 1 if valid, 0 if invalid.
pub extern "C" fn rs_validate_write_range(start: c_int, end: c_int, line_count: c_int) -> c_int {
    let range = LineRange::new(start, end);
    c_int::from(validate_write_range(range, line_count).is_ok())
}

/// Check if write should proceed for update command.
///
/// Returns 1 if should write, 0 if should skip.
pub extern "C" fn rs_should_write_update(is_modified: c_int) -> c_int {
    c_int::from(should_write(is_modified != 0, WriteMode::Update))
}

// =============================================================================
// Phase 1: Write Validation Helpers FFI declarations
// =============================================================================

extern "C" {
    fn nvim_excmds_get_p_write() -> c_int;
    fn nvim_excmds_os_nodetype(fname: *const c_char) -> c_int;
    fn nvim_excmds_node_other_val() -> c_int;
    fn nvim_excmds_eap_get_mkdir_p(eap: *const ExArgHandle) -> c_int;
    fn nvim_excmds_os_file_mkdir(fname: *const c_char) -> c_int;
    fn nvim_excmds_buf_get_b_p_ro(buf: *const BufHandle) -> c_int;
    fn nvim_excmds_buf_get_b_fname(buf: *const BufHandle) -> *const c_char;
    fn nvim_excmds_buf_ffname_path_exists(buf: *const BufHandle) -> c_int;
    fn nvim_excmds_buf_ffname_is_writable(buf: *const BufHandle) -> c_int;
    fn nvim_excmds_p_confirm_or_cmod_confirm() -> c_int;
    fn nvim_excmds_vim_dialog_yesno_question(msg: *const c_char) -> c_int;
    fn nvim_excmds_dialog_msg_readonly(fmt_id: c_int, arg: *const c_char) -> *mut c_char;
    fn nvim_excmds_emsg_readonly() -> c_int;
    fn nvim_excmds_semsg_e503(fname: *const c_char);
    fn nvim_excmds_semsg_e505(fname: *const c_char);
    fn nvim_excmds_emsg_e142();
    fn nvim_excmds_set_forceit(eap: *mut ExArgHandle, val: c_int);
    fn nvim_excmds_eap_get_forceit(eap: *const ExArgHandle) -> c_int;
    fn xfree(ptr: *mut std::ffi::c_void);
}

// =============================================================================
// Phase 1: Write Validation Helpers (Rust implementations)
// =============================================================================

/// Check 'write' option. Returns true (1) if writing is disabled (error printed).
///
/// # Safety
/// No pointers involved.
#[no_mangle]
pub unsafe extern "C" fn rs_not_writing() -> c_int {
    if nvim_excmds_get_p_write() != 0 {
        return 0; // writing is enabled, no error
    }
    nvim_excmds_emsg_e142();
    1 // writing is disabled
}

/// Check if fname is a writable device (Unix only). Returns FAIL (0) or OK (1).
///
/// # Safety
/// `fname` must be a valid C string pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_check_writable(fname: *const c_char) -> c_int {
    #[cfg(unix)]
    {
        if nvim_excmds_os_nodetype(fname) == nvim_excmds_node_other_val() {
            nvim_excmds_semsg_e503(fname);
            return 0; // FAIL
        }
    }
    #[cfg(not(unix))]
    {
        let _ = fname;
    }
    1 // OK
}

/// Handle ++p (mkdir -p) argument for write command.
/// Returns OK (1) on success, FAIL (0) if mkdir failed.
///
/// # Safety
/// `eap` and `fname` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_handle_mkdir_p_arg(
    eap: *mut ExArgHandle,
    fname: *const c_char,
) -> c_int {
    if nvim_excmds_eap_get_mkdir_p(eap) != 0 && nvim_excmds_os_file_mkdir(fname) < 0 {
        return 0; // FAIL
    }
    1 // OK
}

/// Check if buffer is readonly, possibly prompting with a dialog.
/// Returns true (1) if readonly (writing not allowed), false (0) if writing is allowed.
/// May set eap->forceit to true if the user confirms override.
///
/// # Safety
/// `eap` and `buf` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_check_readonly(eap: *mut ExArgHandle, buf: *mut BufHandle) -> c_int {
    let forceit = nvim_excmds_eap_get_forceit(eap);
    if forceit != 0 {
        return 0; // not readonly when forced
    }

    let b_p_ro = nvim_excmds_buf_get_b_p_ro(buf);
    let ffname_exists = nvim_excmds_buf_ffname_path_exists(buf);
    let ffname_writable = nvim_excmds_buf_ffname_is_writable(buf);

    // Check: buffer has 'readonly' set OR (file exists AND is not writable)
    let is_readonly = b_p_ro != 0 || (ffname_exists != 0 && ffname_writable == 0);

    if !is_readonly {
        return 0; // not readonly
    }

    let b_fname = nvim_excmds_buf_get_b_fname(buf);
    if nvim_excmds_p_confirm_or_cmod_confirm() != 0 && !b_fname.is_null() {
        // Show a dialog
        let fmt_id = if b_p_ro != 0 { 0 } else { 1 };
        let buff = nvim_excmds_dialog_msg_readonly(fmt_id, b_fname);
        let yes = nvim_excmds_vim_dialog_yesno_question(buff);
        xfree(buff.cast());
        if yes != 0 {
            // User confirmed: set forceit and allow write
            nvim_excmds_set_forceit(eap, 1);
            return 0; // not readonly (user overrode)
        }
        return 1; // readonly (user declined)
    }

    // No dialog: emit error
    if b_p_ro != 0 {
        nvim_excmds_emsg_readonly();
    } else {
        nvim_excmds_semsg_e505(b_fname);
    }
    1 // readonly
}

// =============================================================================
// ex_update, ex_write, ex_wnext (Phase 3 migration)
// =============================================================================

extern "C" {
    fn nvim_excmds_curbufIsChanged() -> c_int;
    fn nvim_excmds_bt_nofilename_curbuf() -> c_int;
    fn nvim_excmds_curbuf_ffname_not_null() -> c_int;
    fn nvim_excmds_os_path_exists_curbuf_ffname() -> c_int;
    fn nvim_excmds_do_write(eap: *mut ExArgHandle) -> c_int;
    fn nvim_excmds_do_bang_write_filter(eap: *mut ExArgHandle);
    fn nvim_exarg_cmdidx_is_saveas(eap: *const ExArgHandle) -> c_int;
    fn nvim_exarg_get_usefilter(eap: *const ExArgHandle) -> c_int;
    fn nvim_exarg_set_line1(eap: *mut ExArgHandle, line1: c_int);
    fn nvim_exarg_set_line2(eap: *mut ExArgHandle, line2: c_int);
    fn nvim_curbuf_get_b_ml_ml_line_count() -> c_int;
    fn nvim_excmds_curwin_get_w_arg_idx() -> c_int;
    fn nvim_exarg_get_cmd_byte1(eap: *const ExArgHandle) -> c_int;
    fn nvim_exarg_get_line2(eap: *const ExArgHandle) -> c_int;
    fn nvim_excmds_do_argfile(eap: *mut ExArgHandle, i: c_int);
}

/// Implement `:update` command. Replaces C `ex_update`.
///
/// Writes the buffer only if it has been changed or if the file does not exist.
///
/// # Safety
/// `eap` must be a valid exarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_update(eap: *mut ExArgHandle) {
    let is_changed = nvim_excmds_curbufIsChanged() != 0;
    let no_filename = nvim_excmds_bt_nofilename_curbuf() != 0;
    let has_ffname = nvim_excmds_curbuf_ffname_not_null() != 0;
    let path_exists = nvim_excmds_os_path_exists_curbuf_ffname() != 0;

    if is_changed || (!no_filename && has_ffname && !path_exists) {
        nvim_excmds_do_write(eap);
    }
}

/// Implement `:write` and `:saveas` commands. Replaces C `ex_write`.
///
/// # Safety
/// `eap` must be a valid exarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_write(eap: *mut ExArgHandle) {
    if nvim_exarg_cmdidx_is_saveas(eap) != 0 {
        // :saveas does not take a range, uses all lines.
        nvim_exarg_set_line1(eap, 1);
        let line_count = nvim_curbuf_get_b_ml_ml_line_count();
        nvim_exarg_set_line2(eap, line_count);
    }

    if nvim_exarg_get_usefilter(eap) != 0 {
        // input lines to shell command
        nvim_excmds_do_bang_write_filter(eap);
    } else {
        nvim_excmds_do_write(eap);
    }
}

/// Implement `:wnext`, `:wNext`, `:wprevious` commands. Replaces C `ex_wnext`.
///
/// # Safety
/// `eap` must be a valid exarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_wnext(eap: *mut ExArgHandle) {
    let cmd_byte1 = nvim_exarg_get_cmd_byte1(eap);
    let line2_count = nvim_exarg_get_line2(eap);
    let w_arg_idx = nvim_excmds_curwin_get_w_arg_idx();

    let i = if cmd_byte1 == b'n' as c_int {
        w_arg_idx + line2_count
    } else {
        w_arg_idx - line2_count
    };

    nvim_exarg_set_line1(eap, 1);
    let line_count = nvim_curbuf_get_b_ml_ml_line_count();
    nvim_exarg_set_line2(eap, line_count);

    if nvim_excmds_do_write(eap) != 0 {
        nvim_excmds_do_argfile(eap, i);
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_result() {
        assert!(WriteResult::Ok.is_ok());
        assert!(!WriteResult::Readonly.is_ok());
        assert!(!WriteResult::Error.is_ok());

        assert!(WriteResult::NotModified.is_skipped());
        assert!(!WriteResult::Ok.is_skipped());
    }

    #[test]
    fn test_write_result_from_c() {
        assert_eq!(WriteResult::from_c_ok_fail(0), WriteResult::Ok);
        assert_eq!(WriteResult::from_c_ok_fail(1), WriteResult::Error);
    }

    #[test]
    fn test_validate_write_range() {
        // Normal range
        let range = LineRange::new(5, 10);
        let result = validate_write_range(range, 100);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), range);

        // Range extending beyond buffer - gets clamped
        let range = LineRange::new(5, 150);
        let result = validate_write_range(range, 100);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), LineRange::new(5, 100));

        // Empty buffer
        let range = LineRange::new(1, 10);
        let result = validate_write_range(range, 0);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_should_write() {
        // Normal mode always writes
        assert!(should_write(true, WriteMode::Normal));
        assert!(should_write(false, WriteMode::Normal));

        // Update mode only writes if modified
        assert!(should_write(true, WriteMode::Update));
        assert!(!should_write(false, WriteMode::Update));

        // SaveAs always writes
        assert!(should_write(true, WriteMode::SaveAs));
        assert!(should_write(false, WriteMode::SaveAs));
    }

    #[test]
    fn test_write_options_whole_buffer() {
        let opts = WriteOptions::whole_buffer(100);
        assert_eq!(opts.range, LineRange::whole_buffer(100));
        assert!(!opts.force);
        assert!(!opts.append);
    }

    #[test]
    fn test_write_options_with_range() {
        let range = LineRange::new(5, 20);
        let opts = WriteOptions::with_range(range);
        assert_eq!(opts.range, range);
    }

    #[test]
    fn test_write_options_append() {
        let range = LineRange::new(5, 20);
        let opts = WriteOptions::append_to(range);
        assert!(opts.append);
        assert_eq!(opts.range, range);
    }

    #[test]
    fn test_write_options_builder() {
        let opts = WriteOptions::with_range(LineRange::new(1, 10)).force();
        assert!(opts.force);
        assert_eq!(opts.range.start, 1);
        assert_eq!(opts.range.end, 10);
    }

    #[test]
    fn test_write_error_display() {
        let err = WriteError::InvalidRange;
        assert_eq!(format!("{err}"), "invalid range");

        let err = WriteError::EmptyPath;
        assert_eq!(format!("{err}"), "empty file path");

        let err = WriteError::Readonly;
        assert_eq!(format!("{err}"), "buffer is readonly");

        let err = WriteError::FileExists("test.txt".to_string());
        assert_eq!(format!("{err}"), "file already exists: test.txt");
    }

    #[test]
    fn test_rs_validate_write_range() {
        assert_eq!(rs_validate_write_range(1, 10, 100), 1);
        assert_eq!(rs_validate_write_range(5, 150, 100), 1); // Gets clamped
    }

    #[test]
    fn test_rs_should_write_update() {
        assert_eq!(rs_should_write_update(1), 1);
        assert_eq!(rs_should_write_update(0), 0);
    }
}
