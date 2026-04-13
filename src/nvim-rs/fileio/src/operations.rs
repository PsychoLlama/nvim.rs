//! High-level file operation utilities.
//!
//! This module provides supporting types and utilities for the main
//! file I/O operations (readfile, buf_write) including:
//! - Read/write result tracking
//! - File message formatting
//! - Operation state management
//! - Statistics collection

use std::ffi::c_int;

use crate::{bref_void, buf_mut_void, FileFormat, ReadFlags, WriteFlags};

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
    fn vim_snprintf(buf: *mut c_char, buflen: usize, fmt: *const c_char, ...) -> c_int;
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

    // --- filemess globals ---
    static msg_col: c_int;
    static mut msg_silent: c_int;
    static mut msg_scroll: c_int;
    static mut msg_scrolled_ign: bool;
    static mut p_verbose: i64;
    static mut msg_listdo_overwrite: c_int;
    static mut exiting: bool;

    // --- filemess functions ---
    fn msg_check_for_delay(check_msg_scroll: bool);
    fn msg_start();
    fn msg_putchar(c: c_int);
    fn msg_may_trunc(force: bool, s: *mut c_char) -> *mut c_char;
    fn msg_outtrans(str_: *const c_char, hl_id: c_int, hist: bool) -> c_int;
    fn msg_clr_eos();
    fn ui_flush();
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

// SHM_OVERALL = 'O'
const SHM_OVERALL: c_int = b'O' as c_int;

/// Display a message for a file operation.
///
/// Directly replaces the C `filemess` symbol.
///
/// # Safety
/// Accesses many C globals via FFI.
#[export_name = "filemess"]
pub unsafe extern "C" fn rs_filemess(buf: *const c_void, name: *const c_char, s: *const c_char) {
    let prev_msg_col = unsafe { msg_col };

    if unsafe { msg_silent } != 0 {
        return;
    }

    // Fill IObuff with quoted filename.
    let iobuf_ptr = std::ptr::addr_of_mut!(IObuff) as *mut c_char;
    unsafe { rs_add_quoted_fname(iobuf_ptr, 1025 - 100, buf, name) };

    // Append the status string.
    unsafe { xstrlcat(iobuf_ptr, s, 1025) };

    // For the first message may have to start a new line.
    // For further ones overwrite the previous one, reset msg_scroll before calling filemess().
    let msg_scroll_save = unsafe { msg_scroll };
    if unsafe { shortmess(SHM_OVERALL) }
        && unsafe { msg_listdo_overwrite } == 0
        && !unsafe { exiting }
        && unsafe { p_verbose } == 0
    {
        unsafe { msg_scroll = 0 };
    }
    if unsafe { msg_scroll } == 0 {
        // wait a bit when overwriting an error msg
        unsafe { msg_check_for_delay(false) };
    }
    unsafe { msg_start() };
    if prev_msg_col != 0 && unsafe { msg_col } == 0 {
        unsafe { msg_putchar(b'\r' as c_int) }; // overwrite any previous message.
    }
    unsafe { msg_scroll = msg_scroll_save };
    unsafe { msg_scrolled_ign = true };
    // may truncate the message to avoid a hit-return prompt
    let trunc = unsafe { msg_may_trunc(false, iobuf_ptr) };
    unsafe { msg_outtrans(trunc, 0, false) };
    unsafe { msg_clr_eos() };
    unsafe { ui_flush() };
    unsafe { msg_scrolled_ign = false };
}

// =============================================================================
// prep_exarg: prepare exarg_T for buffer reload
// =============================================================================

extern "C" {
    /// Set eap->cmd (takes ownership of the pointer).
    fn nvim_exarg_set_cmd(eap: *mut c_void, cmd: *mut c_char);
    /// Set eap->force_enc.
    fn nvim_exarg_set_force_enc(eap: *mut c_void, val: c_int);
    /// Set eap->bad_char.
    fn nvim_exarg_set_bad_char(eap: *mut c_void, val: c_int);
    /// Set eap->force_ff.
    fn nvim_exarg_set_force_ff(eap: *mut c_void, val: c_int);
    /// Set eap->force_bin.
    fn nvim_exarg_set_force_bin(eap: *mut c_void, val: c_int);
    /// Set eap->read_edit.
    fn nvim_exarg_set_read_edit(eap: *mut c_void, val: c_int);
    /// Set eap->forceit.
    fn nvim_exarg_set_forceit(eap: *mut c_void, val: c_int);
    /// Allocate memory (like malloc but abort on OOM).
    fn xmalloc(size: usize) -> *mut c_char;
    /// snprintf: format into buf.
    fn snprintf(buf: *mut c_char, n: usize, fmt: *const c_char, ...) -> c_int;
}

// FORCE_BIN and FORCE_NOBIN from ex_cmds_defs.h
const FORCE_BIN: c_int = 1;
const FORCE_NOBIN: c_int = 2;

/// Prepare exarg_T for a buffer reload operation.
///
/// Replaces the C `prep_exarg` function.
///
/// # Safety
/// - `eap` must be a valid non-null pointer to an exarg_T.
/// - `buf` must be a valid non-null pointer to a buf_T.
#[export_name = "prep_exarg"]
pub unsafe extern "C" fn rs_prep_exarg(eap: *mut c_void, buf: *const c_void) {
    let fenc = unsafe { bref_void(buf).b_p_fenc };
    // cmd_len = 15 + strlen(buf->b_p_fenc)
    let fenc_len = if fenc.is_null() {
        0
    } else {
        unsafe { libc::strlen(fenc as *const libc::c_char) }
    };
    let cmd_len = 15 + fenc_len;
    let cmd = unsafe { xmalloc(cmd_len) };
    unsafe { snprintf(cmd, cmd_len, c"e ++enc=%s".as_ptr(), fenc) };

    unsafe { nvim_exarg_set_cmd(eap, cmd) };
    unsafe { nvim_exarg_set_force_enc(eap, 8) };
    unsafe { nvim_exarg_set_bad_char(eap, bref_void(buf).b_bad_char) };
    unsafe { nvim_exarg_set_force_ff(eap, c_int::from(bref_void(buf).fileformat_char0())) };
    let bin = unsafe { bref_void(buf).b_p_bin != 0 };
    let force_bin = if bin { FORCE_BIN } else { FORCE_NOBIN };
    unsafe { nvim_exarg_set_force_bin(eap, force_bin) };
    unsafe { nvim_exarg_set_read_edit(eap, 0) };
    unsafe { nvim_exarg_set_forceit(eap, 0) };
}

// =============================================================================
// set_file_options and set_rw_fname
// =============================================================================

// Return values
const OK: c_int = 1;
const FAIL: c_int = 0;

// Buffer flags
const BF_NOTEDITED: c_int = 0x08;

// Autocmd event codes (from auevents_enum.generated.h)
const EVENT_BUFADD: c_int = 0;
const EVENT_BUFDELETE: c_int = 2;
const EVENT_BUFNEW: c_int = 9;
const EVENT_BUFWIPEOUT: c_int = 18;

extern "C" {
    /// Get p_ffs (fileformats option) -- returns pointer to the string.
    fn nvim_get_p_ffs() -> *const c_char;
    /// set_fileformat(ff, opt) -- wraps C set_fileformat.
    fn nvim_set_fileformat_local(ff: c_int);
    /// get_fileformat_force(buf, eap) -- returns file format integer.
    fn nvim_get_fileformat_force(buf: *mut c_void, eap: *mut c_void) -> c_int;
    /// set_options_bin(oldval, newval, opt) -- update binary option.
    fn nvim_set_options_bin(oldval: c_int, newval: c_int, opt: c_int);
    /// Get eap->force_bin.
    fn nvim_exarg_get_force_bin(eap: *const c_void) -> c_int;
    /// Get eap->force_ff.
    fn nvim_exarg_get_force_ff(eap: *const c_void) -> c_int;
    /// apply_autocmds(event, fname, fname_io, force, buf).
    fn apply_autocmds(
        event: c_int,
        fname: *const c_char,
        fname_io: *const c_char,
        force: bool,
        buf: *mut c_void,
    ) -> bool;
    /// Check if script should abort.
    fn aborting() -> bool;
    /// setfname(buf, fname, sfname, noswap).
    fn setfname(
        buf: *mut c_void,
        fname: *const c_char,
        sfname: *const c_char,
        noswap: bool,
    ) -> c_int;
    /// Display error message.
    fn emsg(msg: *const c_char);
    /// augroup_exists(name) -> int.
    fn augroup_exists(name: *const c_char) -> c_int;
    /// do_doautocmd(autocmds, check_after_done, ret_did_aucmd).
    fn do_doautocmd(
        autocmds: *const c_char,
        check_after_done: bool,
        ret_did_aucmd: *mut c_int,
    ) -> c_int;
    /// do_modelines(flags).
    fn do_modelines(flags: c_int);
    /// Get default fileformat (from path/rs_default_fileformat).
    fn rs_default_fileformat() -> c_int;
}

/// OPT_LOCAL flag for set_fileformat and set_options_bin
const OPT_LOCAL: c_int = 0x02;

/// Set default or forced 'fileformat' and 'binary'.
///
/// Replaces the C `set_file_options` function.
///
/// # Safety
/// Accesses global curbuf and eap fields via FFI.
#[export_name = "set_file_options"]
pub unsafe extern "C" fn rs_set_file_options(set_options: c_int, eap: *mut c_void) {
    if set_options != 0 {
        let force_ff = if eap.is_null() {
            0
        } else {
            unsafe { nvim_exarg_get_force_ff(eap) }
        };
        if force_ff != 0 {
            let buf = unsafe { nvim_get_curbuf() };
            let ff = unsafe { nvim_get_fileformat_force(buf, eap) };
            unsafe { nvim_set_fileformat_local(ff) };
        } else {
            let p_ffs = unsafe { nvim_get_p_ffs() };
            if !p_ffs.is_null() && unsafe { *p_ffs } != 0 {
                let ff = unsafe { rs_default_fileformat() };
                unsafe { nvim_set_fileformat_local(ff) };
            }
        }
    }

    let force_bin = if eap.is_null() {
        0
    } else {
        unsafe { nvim_exarg_get_force_bin(eap) }
    };
    if force_bin != 0 {
        let buf = unsafe { nvim_get_curbuf() };
        let oldval = unsafe { bref_void(buf as *const c_void).b_p_bin };
        let newval = if force_bin == FORCE_BIN { 1 } else { 0 };
        unsafe { buf_mut_void(buf).b_p_bin = newval };
        unsafe { nvim_set_options_bin(oldval, newval, OPT_LOCAL) };
    }
}

/// Set the name of the current buffer.
///
/// Used when the buffer doesn't have a name and a ":r" or ":w" command
/// with a file name is used. Replaces the C `set_rw_fname` function.
///
/// # Safety
/// Accesses global curbuf and fires autocmds via FFI.
#[export_name = "set_rw_fname"]
pub unsafe extern "C" fn rs_set_rw_fname(fname: *mut c_char, sfname: *mut c_char) -> c_int {
    let buf = unsafe { nvim_get_curbuf() };

    // Fire BufDelete/BufWipeout like the unnamed buffer is being deleted.
    if unsafe { bref_void(buf as *const c_void).b_p_bl } != 0 {
        unsafe {
            apply_autocmds(
                EVENT_BUFDELETE,
                std::ptr::null(),
                std::ptr::null(),
                false,
                buf,
            )
        };
    }
    unsafe {
        apply_autocmds(
            EVENT_BUFWIPEOUT,
            std::ptr::null(),
            std::ptr::null(),
            false,
            buf,
        )
    };

    if unsafe { aborting() } {
        return FAIL;
    }
    if unsafe { nvim_get_curbuf() } != buf {
        // We are in another buffer now, don't do the renaming.
        unsafe { emsg(c"E812: Autocommands changed buffer or buffer name".as_ptr()) };
        return FAIL;
    }

    let r = unsafe { setfname(buf, fname, sfname, false) };
    if r == OK {
        let flags = unsafe { bref_void(buf as *const c_void).b_flags };
        unsafe { buf_mut_void(buf).b_flags = flags | BF_NOTEDITED };
    }

    // Fire BufNew/BufAdd like a new named buffer is being created.
    unsafe { apply_autocmds(EVENT_BUFNEW, std::ptr::null(), std::ptr::null(), false, buf) };
    let cur = unsafe { nvim_get_curbuf() };
    if unsafe { bref_void(cur as *const c_void).b_p_bl } != 0 {
        unsafe { apply_autocmds(EVENT_BUFADD, std::ptr::null(), std::ptr::null(), false, cur) };
    }
    if unsafe { aborting() } {
        return FAIL;
    }

    // Do filetype detection if 'filetype' is empty.
    let cur = unsafe { nvim_get_curbuf() };
    let ft = unsafe { bref_void(cur as *const c_void).b_p_ft };
    if ft.is_null() || unsafe { *ft } == 0 {
        if unsafe { augroup_exists(c"filetypedetect".as_ptr()) } != 0 {
            unsafe {
                do_doautocmd(
                    c"filetypedetect BufRead".as_ptr(),
                    false,
                    std::ptr::null_mut(),
                )
            };
        }
        unsafe { do_modelines(0) };
    }

    OK
}

// =============================================================================
// shorten_buf_fname and shorten_fnames
// =============================================================================

const MAXPATHL: usize = 4096;

extern "C" {
    /// Check if buftype is "nofile" or similar.
    fn nvim_bt_nofilename(buf: *mut c_void) -> c_int;
    /// Returns non-zero if the given path looks like a URL.
    fn path_with_url(fname: *const c_char) -> c_int;
    /// Returns non-zero if the path is absolute.
    fn path_is_absolute(fname: *const c_char) -> bool;
    /// Returns a pointer into ffname shortened relative to dirname, or NULL.
    fn path_shorten_fname(ffname: *const c_char, dirname: *const c_char) -> *mut c_char;
    /// Duplicate a C string.
    fn xstrdup(s: *const c_char) -> *mut c_char;
    /// Free memory.
    fn xfree(ptr: *mut c_void);
    /// Call mf_fullname(buf->b_ml.ml_mfp).
    fn nvim_buf_mf_fullname(buf: *mut c_void);
    /// Get dirname from current working directory.
    fn os_dirname(buf: *mut c_char, len: usize) -> c_int;
    /// Trigger redraw of all status lines.
    fn status_redraw_all();
    /// Set redraw_tabline global.
    fn nvim_set_redraw_tabline(val: c_int);
    /// Get firstbuf pointer.
    fn nvim_get_firstbuf() -> *mut c_void;
}

/// Shorten a single buffer's filename relative to the current directory.
///
/// Replaces the C `shorten_buf_fname` function.
///
/// # Safety
/// All pointer arguments must be valid for their documented lifetimes.
#[export_name = "shorten_buf_fname"]
pub unsafe extern "C" fn rs_shorten_buf_fname(
    buf: *mut c_void,
    dirname: *const c_char,
    force: c_int,
) {
    let b_fname = unsafe { bref_void(buf as *const c_void).b_fname };
    if b_fname.is_null() {
        return;
    }
    if unsafe { nvim_bt_nofilename(buf) } != 0 {
        return;
    }
    if unsafe { path_with_url(b_fname) } != 0 {
        return;
    }
    let b_sfname = unsafe { bref_void(buf as *const c_void).b_sfname };
    let b_ffname = unsafe { bref_void(buf as *const c_void).b_ffname };
    let should_shorten = force != 0 || b_sfname.is_null() || unsafe { path_is_absolute(b_sfname) };
    if !should_shorten {
        return;
    }

    // Free b_sfname if it's not aliased to b_ffname.
    if b_sfname != b_ffname {
        unsafe { xfree(b_sfname as *mut c_void) };
        unsafe { buf_mut_void(buf).b_sfname = std::ptr::null() };
    }

    let p = unsafe { path_shorten_fname(b_ffname, dirname) };
    if !p.is_null() {
        let new_sfname = unsafe { xstrdup(p) };
        unsafe { buf_mut_void(buf).b_sfname = new_sfname };
        unsafe { buf_mut_void(buf).b_fname = new_sfname };
    } else {
        unsafe { buf_mut_void(buf).b_fname = b_ffname };
    }
}

/// Shorten filenames for all buffers.
///
/// Replaces the C `shorten_fnames` function.
///
/// # Safety
/// Accesses global buffer list and global state.
#[export_name = "shorten_fnames"]
pub unsafe extern "C" fn rs_shorten_fnames(force: c_int) {
    let mut dirname = vec![0u8; MAXPATHL];
    unsafe { os_dirname(dirname.as_mut_ptr() as *mut c_char, MAXPATHL) };

    let mut buf = unsafe { nvim_get_firstbuf() };
    while !buf.is_null() {
        unsafe { rs_shorten_buf_fname(buf, dirname.as_ptr() as *const c_char, force) };
        unsafe { nvim_buf_mf_fullname(buf) };
        buf = unsafe { bref_void(buf as *const c_void).b_next };
    }

    unsafe { status_redraw_all() };
    unsafe { nvim_set_redraw_tabline(1) };
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
