//! High-level file operation utilities.
//!
//! This module provides supporting types and utilities for the main
//! file I/O operations (readfile, buf_write) including:
//! - Read/write result tracking
//! - File message formatting
//! - Operation state management
//! - Statistics collection

use std::ffi::c_int;

use crate::{FileFormat, ReadFlags, WriteFlags};

// =============================================================================
// Read Operation Result
// =============================================================================

/// Result of a file read operation.
#[derive(Debug, Clone, Default)]
pub struct ReadResult {
    /// Number of lines read
    pub lines_read: usize,
    /// Number of bytes read
    pub bytes_read: u64,
    /// Detected file format
    pub file_format: FileFormat,
    /// Whether the file was converted (encoding changed)
    pub converted: bool,
    /// Whether errors occurred during reading
    pub had_errors: bool,
    /// Number of lines with errors
    pub error_lines: usize,
    /// Whether the file had no final newline
    pub no_final_newline: bool,
    /// Detected encoding name (if different from buffer)
    pub detected_encoding: Option<String>,
}

impl ReadResult {
    /// Create a new read result.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if the read was successful.
    pub fn is_success(&self) -> bool {
        !self.had_errors
    }
}

// =============================================================================
// Write Operation Result
// =============================================================================

/// Result of a file write operation.
#[derive(Debug, Clone, Default)]
pub struct WriteResult {
    /// Number of lines written
    pub lines_written: usize,
    /// Number of bytes written
    pub bytes_written: u64,
    /// Whether the write was successful
    pub success: bool,
    /// Whether a backup was created
    pub backup_created: bool,
    /// Path to backup file (if created)
    pub backup_path: Option<String>,
    /// Whether the file was converted (encoding changed)
    pub converted: bool,
}

impl WriteResult {
    /// Create a new write result.
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark as successful.
    pub fn mark_success(&mut self) {
        self.success = true;
    }
}

// =============================================================================
// File Message Types
// =============================================================================

/// Types of messages displayed during file operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileMessageType {
    /// Reading file message
    Reading,
    /// Writing file message
    Writing,
    /// File info (after read/write)
    Info,
    /// Warning message
    Warning,
    /// Error message
    Error,
}

/// Information for formatting file messages.
#[derive(Debug, Clone, Default)]
pub struct FileMessageInfo {
    /// File name (may be shortened for display)
    pub filename: String,
    /// Number of lines
    pub lines: usize,
    /// Number of bytes/characters
    pub bytes: u64,
    /// File format indicator
    pub format: Option<FileFormat>,
    /// Whether file is new
    pub is_new: bool,
    /// Whether file is read-only
    pub is_readonly: bool,
    /// Whether file was converted
    pub converted: bool,
    /// Additional info flags
    pub flags: Vec<String>,
}

impl FileMessageInfo {
    /// Create new message info.
    pub fn new(filename: impl Into<String>) -> Self {
        Self {
            filename: filename.into(),
            ..Default::default()
        }
    }

    /// Set the line count.
    pub fn with_lines(mut self, lines: usize) -> Self {
        self.lines = lines;
        self
    }

    /// Set the byte count.
    pub fn with_bytes(mut self, bytes: u64) -> Self {
        self.bytes = bytes;
        self
    }

    /// Set the file format.
    pub fn with_format(mut self, format: FileFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// Mark as new file.
    pub fn as_new(mut self) -> Self {
        self.is_new = true;
        self
    }

    /// Mark as read-only.
    pub fn as_readonly(mut self) -> Self {
        self.is_readonly = true;
        self
    }

    /// Add a flag.
    pub fn with_flag(mut self, flag: impl Into<String>) -> Self {
        self.flags.push(flag.into());
        self
    }

    /// Format the message for display.
    ///
    /// Returns a string like: `"filename" 123L, 4567B`
    pub fn format_message(&self) -> String {
        let mut parts = Vec::new();

        // Line count
        if self.lines > 0 {
            parts.push(format!("{}L", self.lines));
        }

        // Byte count
        if self.bytes > 0 {
            parts.push(format!("{}B", self.bytes));
        }

        // Format indicator
        if let Some(format) = self.format {
            match format {
                FileFormat::Dos => parts.push("[dos]".to_string()),
                FileFormat::Mac => parts.push("[mac]".to_string()),
                FileFormat::Unix | FileFormat::Unknown => {}
            }
        }

        // New file
        if self.is_new {
            parts.push("[New]".to_string());
        }

        // Read-only
        if self.is_readonly {
            parts.push("[RO]".to_string());
        }

        // Converted
        if self.converted {
            parts.push("[converted]".to_string());
        }

        // Additional flags
        for flag in &self.flags {
            parts.push(format!("[{}]", flag));
        }

        if parts.is_empty() {
            format!("\"{}\"", self.filename)
        } else {
            format!("\"{}\" {}", self.filename, parts.join(" "))
        }
    }
}

// =============================================================================
// Read Operation State
// =============================================================================

/// State tracking for a read operation.
#[derive(Debug, Clone)]
pub struct ReadState {
    /// Read flags
    pub flags: ReadFlags,
    /// Starting line number
    pub from_line: i64,
    /// Lines to skip
    pub lines_to_skip: i64,
    /// Lines to read (MAXLNUM for all)
    pub lines_to_read: i64,
    /// Current line number being read
    pub current_line: i64,
    /// Whether we're reading from stdin
    pub from_stdin: bool,
    /// Whether we're reading from a buffer
    pub from_buffer: bool,
    /// Whether this is a new file
    pub is_new_file: bool,
    /// Whether we're in filter mode
    pub is_filter: bool,
    /// Whether to keep undo info
    pub keep_undo: bool,
}

impl ReadState {
    /// Create a new read state from flags.
    pub fn from_flags(flags: ReadFlags, from_line: i64) -> Self {
        Self {
            flags,
            from_line,
            lines_to_skip: 0,
            lines_to_read: i64::MAX,
            current_line: from_line,
            from_stdin: flags.contains(ReadFlags::STDIN),
            from_buffer: flags.contains(ReadFlags::BUFFER),
            is_new_file: flags.contains(ReadFlags::NEW),
            is_filter: flags.contains(ReadFlags::FILTER),
            keep_undo: flags.contains(ReadFlags::KEEP_UNDO),
        }
    }

    /// Check if we should trigger autocmds.
    pub fn should_trigger_autocmds(&self) -> bool {
        !self.flags.contains(ReadFlags::DUMMY)
    }

    /// Check if this is a dummy read (for checking file changes).
    pub fn is_dummy(&self) -> bool {
        self.flags.contains(ReadFlags::DUMMY)
    }
}

// =============================================================================
// Write Operation State
// =============================================================================

/// State tracking for a write operation.
#[derive(Debug, Clone)]
pub struct WriteState {
    /// Write flags
    pub flags: WriteFlags,
    /// Starting line number
    pub start_line: i64,
    /// Ending line number
    pub end_line: i64,
    /// Whether this is an append operation
    pub is_append: bool,
    /// Whether this is a forced write
    pub is_forced: bool,
    /// Whether writing the whole file
    pub is_whole_file: bool,
    /// Whether this is a "save as" operation
    pub is_save_as: bool,
}

impl WriteState {
    /// Create a new write state from flags.
    pub fn from_flags(flags: WriteFlags, start_line: i64, end_line: i64) -> Self {
        Self {
            flags,
            start_line,
            end_line,
            is_append: flags.contains(WriteFlags::APPEND),
            is_forced: flags.contains(WriteFlags::FORCE),
            is_whole_file: flags.contains(WriteFlags::WHOLE),
            is_save_as: flags.contains(WriteFlags::SAVEAS),
        }
    }

    /// Check if we should create a backup.
    pub fn should_backup(&self) -> bool {
        // Don't backup for appending
        !self.is_append
    }

    /// Get the line range being written.
    pub fn line_range(&self) -> (i64, i64) {
        (self.start_line, self.end_line)
    }
}

// =============================================================================
// Operation Statistics
// =============================================================================

/// Statistics collected during file operations.
#[derive(Debug, Clone, Default)]
pub struct FileOpStats {
    /// Total bytes processed
    pub bytes_total: u64,
    /// Total lines processed
    pub lines_total: usize,
    /// Lines with encoding errors
    pub encoding_errors: usize,
    /// Lines with format issues (mixed line endings)
    pub format_issues: usize,
    /// Number of null bytes encountered
    pub null_bytes: usize,
    /// Number of long lines (exceeding limit)
    pub long_lines: usize,
}

impl FileOpStats {
    /// Create new stats.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add bytes to the total.
    pub fn add_bytes(&mut self, bytes: u64) {
        self.bytes_total += bytes;
    }

    /// Add lines to the total.
    pub fn add_lines(&mut self, lines: usize) {
        self.lines_total += lines;
    }

    /// Record an encoding error.
    pub fn record_encoding_error(&mut self) {
        self.encoding_errors += 1;
    }

    /// Record a format issue.
    pub fn record_format_issue(&mut self) {
        self.format_issues += 1;
    }

    /// Check if there were any issues.
    pub fn has_issues(&self) -> bool {
        self.encoding_errors > 0 || self.format_issues > 0 || self.null_bytes > 0
    }
}

// =============================================================================
// Shortmess Flags
// =============================================================================

/// Flags from 'shortmess' option relevant to file I/O.
#[derive(Debug, Clone, Copy, Default)]
pub struct ShortmessFlags {
    /// 'f' - use "(3 of 5)" instead of "(file 3 of 5)"
    pub file_count_short: bool,
    /// 'i' - use "[noeol]" instead of "[Incomplete last line]"
    pub noeol_short: bool,
    /// 'l' - use "999L, 888B" instead of "999 lines, 888 bytes"
    pub line_count_short: bool,
    /// 'n' - use "[New]" instead of "[New File]"
    pub new_short: bool,
    /// 'o' - overwrite message for writing a file with subsequent message
    pub overwrite: bool,
    /// 'O' - message for reading a file overwrites previous
    pub overwrite_read: bool,
    /// 'r' - use "[RO]" instead of "[readonly]"
    pub readonly_short: bool,
    /// 'w' - use "[w]" instead of "written"
    pub written_short: bool,
    /// 'x' - use "[dos]" instead of "[dos format]"
    pub format_short: bool,
}

impl ShortmessFlags {
    /// Create from a 'shortmess' string.
    pub fn from_shortmess(s: &str) -> Self {
        Self {
            file_count_short: s.contains('f'),
            noeol_short: s.contains('i'),
            line_count_short: s.contains('l'),
            new_short: s.contains('n'),
            overwrite: s.contains('o'),
            overwrite_read: s.contains('O'),
            readonly_short: s.contains('r'),
            written_short: s.contains('w'),
            format_short: s.contains('x'),
        }
    }

    /// Format line count based on shortmess.
    pub fn format_lines(&self, count: usize) -> String {
        if self.line_count_short {
            format!("{}L", count)
        } else if count == 1 {
            "1 line".to_string()
        } else {
            format!("{} lines", count)
        }
    }

    /// Format byte count based on shortmess.
    pub fn format_bytes(&self, count: u64) -> String {
        if self.line_count_short {
            format!("{}B", count)
        } else if count == 1 {
            "1 byte".to_string()
        } else {
            format!("{} bytes", count)
        }
    }

    /// Format "new file" indicator.
    pub fn format_new(&self) -> &'static str {
        if self.new_short {
            "[New]"
        } else {
            "[New File]"
        }
    }

    /// Format "readonly" indicator.
    pub fn format_readonly(&self) -> &'static str {
        if self.readonly_short {
            "[RO]"
        } else {
            "[readonly]"
        }
    }

    /// Format "written" indicator.
    pub fn format_written(&self) -> &'static str {
        if self.written_short {
            " [w]"
        } else {
            " written"
        }
    }

    /// Format file format indicator.
    pub fn format_fileformat(&self, format: FileFormat) -> &'static str {
        match format {
            FileFormat::Dos => {
                if self.format_short {
                    "[dos]"
                } else {
                    "[dos format]"
                }
            }
            FileFormat::Mac => {
                if self.format_short {
                    "[mac]"
                } else {
                    "[mac format]"
                }
            }
            FileFormat::Unix | FileFormat::Unknown => "",
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI wrapper for formatting a file message.
///
/// # Safety
/// - `filename` must be a valid pointer to `filename_len` bytes
/// - `output` must be a valid buffer of at least `output_len` bytes
#[no_mangle]
pub unsafe extern "C" fn rs_format_file_message(
    filename: *const u8,
    filename_len: usize,
    lines: usize,
    bytes: u64,
    format: c_int,
    is_new: c_int,
    is_readonly: c_int,
    output: *mut u8,
    output_len: usize,
) -> usize {
    if filename.is_null() || output.is_null() || output_len == 0 {
        return 0;
    }

    let filename_str = {
        let slice = std::slice::from_raw_parts(filename, filename_len);
        std::str::from_utf8(slice).unwrap_or("")
    };

    let mut info = FileMessageInfo::new(filename_str)
        .with_lines(lines)
        .with_bytes(bytes);

    if format >= 0 {
        info = info.with_format(FileFormat::from_c(format));
    }

    if is_new != 0 {
        info = info.as_new();
    }

    if is_readonly != 0 {
        info = info.as_readonly();
    }

    let message = info.format_message();
    let message_bytes = message.as_bytes();

    if message_bytes.len() >= output_len {
        return 0;
    }

    let out_slice = std::slice::from_raw_parts_mut(output, output_len);
    out_slice[..message_bytes.len()].copy_from_slice(message_bytes);
    out_slice[message_bytes.len()] = 0;

    message_bytes.len()
}

// =============================================================================
// Direct C replacements (export_name = original C symbol)
// =============================================================================

use std::ffi::{c_char, c_void};

extern "C" {
    /// Access the global IObuff character array.
    static mut IObuff: [c_char; 1025];
    /// Returns true if shortmess flag `x` is set.
    fn shortmess(x: c_int) -> bool;
    /// Safe string concat: xstrlcat(dst, src, dstlen).
    fn xstrlcat(dst: *mut c_char, src: *const c_char, dstlen: usize) -> usize;
    /// Sprintf into a buffer.
    fn vim_snprintf(buf: *mut c_char, buflen: usize, fmt: *const c_char, ...);
    /// Replace $HOME with ~ in path, writing into dst.
    fn home_replace(
        buf: *const c_void,
        src: *const c_char,
        dst: *mut c_char,
        dstlen: usize,
        one: bool,
    ) -> usize;
    /// Get curbuf pointer.
    fn nvim_get_curbuf() -> *mut c_void;
    /// Get buf->b_no_eol_lnum.
    fn nvim_bw_buf_get_no_eol_lnum(buf: *mut c_void) -> i32;
    /// Set buf->b_no_eol_lnum.
    fn nvim_bw_buf_set_no_eol_lnum(buf: *mut c_void, lnum: i32);
    /// gettext translation.
    fn gettext(msgid: *const c_char) -> *const c_char;
}

// SHM_LINES = 'l'
const SHM_LINES: c_int = b'l' as c_int;

/// Adjust the line number for a buffer with missing eol.
///
/// Directly replaces the C `write_lnum_adjust` symbol.
///
/// # Safety
/// Accesses the C global `curbuf` via FFI.
#[export_name = "write_lnum_adjust"]
pub unsafe extern "C" fn rs_write_lnum_adjust(offset: i32) {
    let buf = unsafe { nvim_get_curbuf() };
    if !buf.is_null() {
        let lnum = unsafe { nvim_bw_buf_get_no_eol_lnum(buf) };
        if lnum != 0 {
            unsafe { nvim_bw_buf_set_no_eol_lnum(buf, lnum + offset) };
        }
    }
}

/// Append file format string to IObuff.
///
/// Returns true if something was appended.
/// Directly replaces the C `msg_add_fileformat` symbol.
///
/// # Safety
/// Accesses global IObuff via FFI.
#[export_name = "msg_add_fileformat"]
pub unsafe extern "C" fn rs_msg_add_fileformat(eol_type: c_int) -> bool {
    // EOL_DOS=1, EOL_MAC=2, EOL_UNIX=0
    // On non-Windows: [dos] and [mac] are non-default
    if eol_type == 1 {
        // EOL_DOS
        let s = unsafe { gettext(c"[dos]".as_ptr()) };
        unsafe { xstrlcat(std::ptr::addr_of_mut!(IObuff) as *mut c_char, s, 1025) };
        return true;
    }
    if eol_type == 2 {
        // EOL_MAC
        let s = unsafe { gettext(c"[mac]".as_ptr()) };
        unsafe { xstrlcat(std::ptr::addr_of_mut!(IObuff) as *mut c_char, s, 1025) };
        return true;
    }
    false
}

/// Append line and character count to IObuff.
///
/// Directly replaces the C `msg_add_lines` symbol.
///
/// # Safety
/// Accesses global IObuff via FFI.
#[export_name = "msg_add_lines"]
pub unsafe extern "C" fn rs_msg_add_lines(insert_space: c_int, lnum: i32, nchars: i64) {
    // Find end of current IObuff content
    let iobuf_ptr = std::ptr::addr_of_mut!(IObuff) as *mut c_char;
    let iobuf_len: usize = 1025;
    let current_len = unsafe { libc::strlen(iobuf_ptr as *const libc::c_char) };
    let mut p = unsafe { iobuf_ptr.add(current_len) };
    let mut remaining = iobuf_len - current_len;

    if insert_space != 0 && remaining > 0 {
        unsafe { *p = b' ' as c_char };
        p = unsafe { p.add(1) };
        remaining -= 1;
    }

    if unsafe { shortmess(SHM_LINES) } {
        unsafe {
            vim_snprintf(
                p,
                remaining,
                c"%ldL, %ldB".as_ptr(),
                lnum as libc::c_long,
                nchars as libc::c_long,
            )
        };
    } else {
        let fmt_lines = if lnum == 1 {
            c"%ld line, ".as_ptr()
        } else {
            c"%ld lines, ".as_ptr()
        };
        unsafe { vim_snprintf(p, remaining, fmt_lines, lnum as libc::c_long) };
        let new_len = unsafe { libc::strlen(p as *const libc::c_char) };
        p = unsafe { p.add(new_len) };
        remaining -= new_len;
        let fmt_bytes = if nchars == 1 {
            c"%ld byte".as_ptr()
        } else {
            c"%ld bytes".as_ptr()
        };
        unsafe { vim_snprintf(p, remaining, fmt_bytes, nchars as libc::c_long) };
    }
}

/// Put a quoted filename into a buffer.
///
/// Directly replaces the C `add_quoted_fname` symbol.
///
/// # Safety
/// - `ret_buf` must be a valid buffer of at least `buf_len` bytes
/// - `buf` may be null
/// - `fname` may be null (uses "-stdin-" in that case)
#[export_name = "add_quoted_fname"]
pub unsafe extern "C" fn rs_add_quoted_fname(
    ret_buf: *mut c_char,
    buf_len: usize,
    buf: *const c_void,
    fname: *const c_char,
) {
    let fname_ptr = if fname.is_null() {
        c"-stdin-".as_ptr()
    } else {
        fname
    };
    if ret_buf.is_null() || buf_len < 4 {
        return;
    }
    unsafe { *ret_buf = b'"' as c_char };
    unsafe { home_replace(buf, fname_ptr, ret_buf.add(1), buf_len - 4, true) };
    unsafe { xstrlcat(ret_buf, c"\" ".as_ptr(), buf_len) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_result() {
        let mut result = ReadResult::new();
        assert!(result.is_success());

        result.had_errors = true;
        assert!(!result.is_success());
    }

    #[test]
    fn test_write_result() {
        let mut result = WriteResult::new();
        assert!(!result.success);

        result.mark_success();
        assert!(result.success);
    }

    #[test]
    fn test_file_message_info_basic() {
        let info = FileMessageInfo::new("test.txt")
            .with_lines(100)
            .with_bytes(2048);

        let msg = info.format_message();
        assert!(msg.contains("test.txt"));
        assert!(msg.contains("100L"));
        assert!(msg.contains("2048B"));
    }

    #[test]
    fn test_file_message_info_with_flags() {
        let info = FileMessageInfo::new("test.txt")
            .with_lines(10)
            .with_bytes(256)
            .with_format(FileFormat::Dos)
            .as_new()
            .as_readonly();

        let msg = info.format_message();
        assert!(msg.contains("[dos]"));
        assert!(msg.contains("[New]"));
        assert!(msg.contains("[RO]"));
    }

    #[test]
    fn test_file_message_info_empty() {
        let info = FileMessageInfo::new("empty.txt");
        let msg = info.format_message();
        assert_eq!(msg, "\"empty.txt\"");
    }

    #[test]
    fn test_read_state() {
        let flags = ReadFlags::NEW | ReadFlags::STDIN;
        let state = ReadState::from_flags(flags, 0);

        assert!(state.is_new_file);
        assert!(state.from_stdin);
        assert!(!state.is_dummy());
        assert!(state.should_trigger_autocmds());
    }

    #[test]
    fn test_read_state_dummy() {
        let flags = ReadFlags::DUMMY;
        let state = ReadState::from_flags(flags, 0);

        assert!(state.is_dummy());
        assert!(!state.should_trigger_autocmds());
    }

    #[test]
    fn test_write_state() {
        let flags = WriteFlags::WHOLE | WriteFlags::FORCE;
        let state = WriteState::from_flags(flags, 1, 100);

        assert!(state.is_whole_file);
        assert!(state.is_forced);
        assert!(!state.is_append);
        assert!(state.should_backup());
        assert_eq!(state.line_range(), (1, 100));
    }

    #[test]
    fn test_write_state_append() {
        let flags = WriteFlags::APPEND;
        let state = WriteState::from_flags(flags, 1, 50);

        assert!(state.is_append);
        assert!(!state.should_backup());
    }

    #[test]
    fn test_file_op_stats() {
        let mut stats = FileOpStats::new();
        assert!(!stats.has_issues());

        stats.add_bytes(1024);
        stats.add_lines(10);
        assert_eq!(stats.bytes_total, 1024);
        assert_eq!(stats.lines_total, 10);
        assert!(!stats.has_issues());

        stats.record_encoding_error();
        assert!(stats.has_issues());
    }

    #[test]
    fn test_shortmess_flags() {
        let flags = ShortmessFlags::from_shortmess("filnrwx");

        assert!(flags.file_count_short);
        assert!(flags.noeol_short);
        assert!(flags.line_count_short);
        assert!(flags.new_short);
        assert!(flags.readonly_short);
        assert!(flags.written_short);
        assert!(flags.format_short);

        assert_eq!(flags.format_lines(100), "100L");
        assert_eq!(flags.format_bytes(1024), "1024B");
        assert_eq!(flags.format_new(), "[New]");
        assert_eq!(flags.format_readonly(), "[RO]");
        assert_eq!(flags.format_written(), " [w]");
        assert_eq!(flags.format_fileformat(FileFormat::Dos), "[dos]");
    }

    #[test]
    fn test_shortmess_flags_long() {
        let flags = ShortmessFlags::from_shortmess("");

        assert_eq!(flags.format_lines(1), "1 line");
        assert_eq!(flags.format_lines(100), "100 lines");
        assert_eq!(flags.format_bytes(1), "1 byte");
        assert_eq!(flags.format_bytes(1024), "1024 bytes");
        assert_eq!(flags.format_new(), "[New File]");
        assert_eq!(flags.format_readonly(), "[readonly]");
        assert_eq!(flags.format_written(), " written");
        assert_eq!(flags.format_fileformat(FileFormat::Dos), "[dos format]");
        assert_eq!(flags.format_fileformat(FileFormat::Mac), "[mac format]");
        assert_eq!(flags.format_fileformat(FileFormat::Unix), "");
    }
}
