//! File reading infrastructure for Neovim.
//!
//! This module provides:
//! - EINTR-safe read operations
//! - Buffer allocation and management for file reading
//! - Line extraction from read buffers
//! - File metadata collection

use std::ffi::c_int;
use std::io::{self, Read};

use crate::{FIO_LATIN1, FIO_UCS2, FIO_UCS4, FIO_UCSBOM, FIO_UTF16};

// =============================================================================
// Constants
// =============================================================================

/// Default buffer size for file reading operations.
pub const READ_BUFFER_SIZE: usize = 0x10000; // 64KB

/// Maximum factor for iconv conversion buffer expansion.
pub const ICONV_MULT: usize = 8;

/// Number of bytes that may remain unconverted between read operations.
pub const CONV_RESTLEN: usize = 30;

// =============================================================================
// EINTR-Safe I/O Operations
// =============================================================================

/// Read from a file descriptor, retrying on EINTR.
///
/// EINTR can occur when a signal (like SIGWINCH on terminal resize)
/// interrupts a read system call. This function automatically retries
/// the read in such cases.
///
/// # Arguments
/// * `fd` - File descriptor to read from
/// * `buf` - Buffer to read into
///
/// # Returns
/// Number of bytes read, or an error
#[cfg(unix)]
pub fn read_eintr(fd: c_int, buf: &mut [u8]) -> io::Result<usize> {
    use std::os::unix::io::FromRawFd;

    // Create a File from the raw fd without taking ownership
    // SAFETY: We don't close the fd, the caller manages its lifetime
    let mut file = std::mem::ManuallyDrop::new(unsafe { std::fs::File::from_raw_fd(fd) });

    loop {
        match file.read(buf) {
            Ok(n) => return Ok(n),
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        }
    }
}

/// FFI wrapper for read_eintr.
///
/// # Safety
/// - `fd` must be a valid, open file descriptor
/// - `buf` must point to a valid buffer of at least `bufsize` bytes
#[cfg(unix)]
#[no_mangle]
pub unsafe extern "C" fn rs_read_eintr(fd: c_int, buf: *mut u8, bufsize: usize) -> c_int {
    if buf.is_null() || bufsize == 0 {
        return -1;
    }

    let slice = std::slice::from_raw_parts_mut(buf, bufsize);
    match read_eintr(fd, slice) {
        Ok(n) => n as c_int,
        Err(_) => -1,
    }
}

// =============================================================================
// Buffer Size Calculation
// =============================================================================

/// Calculate the effective read size based on encoding flags.
///
/// Different encodings require different buffer sizes because the
/// conversion from the file encoding to UTF-8 can expand the data.
///
/// # Arguments
/// * `buffer_size` - Available buffer size
/// * `fio_flags` - FIO_* encoding flags
/// * `using_iconv` - Whether iconv is being used for conversion
///
/// # Returns
/// The effective number of bytes that can be read
#[inline]
pub fn calculate_read_size(buffer_size: usize, fio_flags: c_int, using_iconv: bool) -> usize {
    if using_iconv {
        // For iconv(), we don't really know the required space
        buffer_size / ICONV_MULT
    } else if fio_flags & FIO_LATIN1 != 0 {
        // Latin1 to UTF-8: 1 byte becomes up to 2 bytes
        buffer_size / 2
    } else if fio_flags & (FIO_UCS2 | FIO_UTF16) != 0 {
        // UCS-2/UTF-16 to UTF-8: 2 bytes become up to 3 bytes
        // Size must be multiple of 2
        (buffer_size * 2 / 3) & !1
    } else if fio_flags & FIO_UCS4 != 0 {
        // UCS-4 to UTF-8: 4 bytes become up to 6 bytes
        // Size must be multiple of 4
        (buffer_size * 2 / 3) & !3
    } else if fio_flags == FIO_UCSBOM {
        // BOM detection, worst case
        buffer_size / ICONV_MULT
    } else {
        buffer_size
    }
}

/// FFI wrapper for calculate_read_size.
#[no_mangle]
pub extern "C" fn rs_calculate_read_size(
    buffer_size: usize,
    fio_flags: c_int,
    using_iconv: c_int,
) -> usize {
    calculate_read_size(buffer_size, fio_flags, using_iconv != 0)
}

// =============================================================================
// Line Extraction
// =============================================================================

/// Result of extracting a line from a buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineExtractResult {
    /// Length of the line (excluding terminator)
    pub line_len: usize,
    /// Length of the line terminator (0, 1, or 2)
    pub terminator_len: usize,
    /// Whether a complete line was found
    pub complete: bool,
}

/// Extract a line from a buffer.
///
/// Scans the buffer for line terminators (LF, CR, or CRLF) and returns
/// information about the first line found.
///
/// # Arguments
/// * `data` - Buffer to scan
/// * `fileformat` - Expected file format (-1=unknown, 0=unix, 1=dos, 2=mac)
///
/// # Returns
/// Information about the extracted line
pub fn extract_line(data: &[u8], fileformat: c_int) -> LineExtractResult {
    if data.is_empty() {
        return LineExtractResult {
            line_len: 0,
            terminator_len: 0,
            complete: false,
        };
    }

    for (i, &byte) in data.iter().enumerate() {
        match byte {
            b'\n' => {
                // Unix line ending (LF)
                return LineExtractResult {
                    line_len: i,
                    terminator_len: 1,
                    complete: true,
                };
            }
            b'\r' => {
                // Check for DOS line ending (CRLF)
                if i + 1 < data.len() && data[i + 1] == b'\n' {
                    return LineExtractResult {
                        line_len: i,
                        terminator_len: 2,
                        complete: true,
                    };
                }
                // Mac line ending (CR only) - only if format is mac or unknown
                if fileformat == 2 || fileformat == -1 {
                    return LineExtractResult {
                        line_len: i,
                        terminator_len: 1,
                        complete: true,
                    };
                }
                // Otherwise, CR is part of the line content
            }
            b'\0' => {
                // NUL byte - in Vim, this often represents a newline that was converted
                // This can happen when reading from stdin
                return LineExtractResult {
                    line_len: i,
                    terminator_len: 1,
                    complete: true,
                };
            }
            _ => {}
        }
    }

    // No line terminator found
    LineExtractResult {
        line_len: data.len(),
        terminator_len: 0,
        complete: false,
    }
}

/// FFI wrapper for extract_line.
///
/// # Safety
/// - `data` must point to a valid buffer of at least `size` bytes
/// - `line_len`, `terminator_len`, and `complete` must be valid pointers
#[no_mangle]
pub unsafe extern "C" fn rs_extract_line(
    data: *const u8,
    size: usize,
    fileformat: c_int,
    line_len: *mut usize,
    terminator_len: *mut usize,
    complete: *mut c_int,
) -> c_int {
    if data.is_null() || line_len.is_null() || terminator_len.is_null() || complete.is_null() {
        return -1;
    }

    let slice = std::slice::from_raw_parts(data, size);
    let result = extract_line(slice, fileformat);

    *line_len = result.line_len;
    *terminator_len = result.terminator_len;
    *complete = if result.complete { 1 } else { 0 };

    0
}

// =============================================================================
// File Format Detection
// =============================================================================

/// Detect the file format (line ending style) from a buffer.
///
/// Examines the buffer content to determine whether the file uses
/// Unix (LF), DOS (CRLF), or Mac (CR) line endings.
///
/// # Arguments
/// * `data` - Buffer to examine
///
/// # Returns
/// File format: 0=unix, 1=dos, 2=mac, or -1 if unknown
pub fn detect_fileformat(data: &[u8]) -> c_int {
    let mut found_cr = false;

    for (i, &byte) in data.iter().enumerate() {
        match byte {
            b'\n' => {
                if found_cr {
                    return 1; // DOS (CRLF)
                }
                return 0; // Unix (LF)
            }
            b'\r' => {
                if i + 1 < data.len() {
                    if data[i + 1] == b'\n' {
                        return 1; // DOS (CRLF)
                    }
                    return 2; // Mac (CR)
                }
                found_cr = true;
            }
            _ => {
                if found_cr {
                    return 2; // Mac (CR followed by non-LF)
                }
            }
        }
    }

    if found_cr {
        return 2; // Mac (file ends with CR)
    }

    -1 // Unknown (no line endings found)
}

/// FFI wrapper for detect_fileformat.
///
/// # Safety
/// `data` must point to a valid buffer of at least `size` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_detect_fileformat(data: *const u8, size: usize) -> c_int {
    if data.is_null() {
        return -1;
    }

    let slice = std::slice::from_raw_parts(data, size);
    detect_fileformat(slice)
}

// =============================================================================
// Read Buffer Management
// =============================================================================

/// Information about a file read operation.
#[derive(Debug, Clone, Default)]
pub struct ReadInfo {
    /// Total bytes read
    pub bytes_read: u64,
    /// Number of lines read
    pub lines_read: u64,
    /// Whether the last line had no EOL
    pub no_eol_last: bool,
    /// Number of conversion errors
    pub conv_errors: u64,
    /// Number of illegal bytes encountered
    pub illegal_bytes: u64,
}

impl ReadInfo {
    /// Create a new ReadInfo with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add bytes to the total.
    pub fn add_bytes(&mut self, count: u64) {
        self.bytes_read = self.bytes_read.saturating_add(count);
    }

    /// Increment the line count.
    pub fn add_line(&mut self) {
        self.lines_read = self.lines_read.saturating_add(1);
    }

    /// Record a conversion error.
    pub fn add_conv_error(&mut self) {
        self.conv_errors = self.conv_errors.saturating_add(1);
    }

    /// Record an illegal byte.
    pub fn add_illegal_byte(&mut self) {
        self.illegal_bytes = self.illegal_bytes.saturating_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_read_size() {
        let buf_size = 1000;

        // No conversion
        assert_eq!(calculate_read_size(buf_size, 0, false), 1000);

        // iconv
        assert_eq!(calculate_read_size(buf_size, 0, true), 1000 / ICONV_MULT);

        // Latin1
        assert_eq!(calculate_read_size(buf_size, FIO_LATIN1, false), 500);

        // UCS-2 (must be multiple of 2)
        let ucs2_size = calculate_read_size(buf_size, FIO_UCS2, false);
        assert!(ucs2_size < buf_size);
        assert_eq!(ucs2_size % 2, 0);

        // UCS-4 (must be multiple of 4)
        let ucs4_size = calculate_read_size(buf_size, FIO_UCS4, false);
        assert!(ucs4_size < buf_size);
        assert_eq!(ucs4_size % 4, 0);

        // UTF-16 (must be multiple of 2)
        let utf16_size = calculate_read_size(buf_size, FIO_UTF16, false);
        assert!(utf16_size < buf_size);
        assert_eq!(utf16_size % 2, 0);

        // BOM detection
        assert_eq!(
            calculate_read_size(buf_size, FIO_UCSBOM, false),
            buf_size / ICONV_MULT
        );
    }

    #[test]
    fn test_extract_line_unix() {
        // Unix line ending
        let data = b"hello\nworld";
        let result = extract_line(data, 0);
        assert_eq!(result.line_len, 5);
        assert_eq!(result.terminator_len, 1);
        assert!(result.complete);
    }

    #[test]
    fn test_extract_line_dos() {
        // DOS line ending
        let data = b"hello\r\nworld";
        let result = extract_line(data, 1);
        assert_eq!(result.line_len, 5);
        assert_eq!(result.terminator_len, 2);
        assert!(result.complete);
    }

    #[test]
    fn test_extract_line_mac() {
        // Mac line ending
        let data = b"hello\rworld";
        let result = extract_line(data, 2);
        assert_eq!(result.line_len, 5);
        assert_eq!(result.terminator_len, 1);
        assert!(result.complete);
    }

    #[test]
    fn test_extract_line_no_terminator() {
        // No line terminator
        let data = b"hello world";
        let result = extract_line(data, 0);
        assert_eq!(result.line_len, 11);
        assert_eq!(result.terminator_len, 0);
        assert!(!result.complete);
    }

    #[test]
    fn test_extract_line_empty() {
        let result = extract_line(&[], 0);
        assert_eq!(result.line_len, 0);
        assert_eq!(result.terminator_len, 0);
        assert!(!result.complete);
    }

    #[test]
    fn test_extract_line_nul() {
        // NUL byte acts as line terminator
        let data = b"hello\0world";
        let result = extract_line(data, 0);
        assert_eq!(result.line_len, 5);
        assert_eq!(result.terminator_len, 1);
        assert!(result.complete);
    }

    #[test]
    fn test_detect_fileformat() {
        // Unix (LF)
        assert_eq!(detect_fileformat(b"hello\nworld\n"), 0);

        // DOS (CRLF)
        assert_eq!(detect_fileformat(b"hello\r\nworld\r\n"), 1);

        // Mac (CR)
        assert_eq!(detect_fileformat(b"hello\rworld\r"), 2);

        // Unknown (no line endings)
        assert_eq!(detect_fileformat(b"hello world"), -1);

        // Empty
        assert_eq!(detect_fileformat(&[]), -1);

        // CR at end without following character
        assert_eq!(detect_fileformat(b"hello\r"), 2);

        // Mixed (first ending wins)
        assert_eq!(detect_fileformat(b"a\nb\r\nc"), 0); // LF first
        assert_eq!(detect_fileformat(b"a\r\nb\nc"), 1); // CRLF first
    }

    #[test]
    fn test_read_info() {
        let mut info = ReadInfo::new();
        assert_eq!(info.bytes_read, 0);
        assert_eq!(info.lines_read, 0);

        info.add_bytes(100);
        info.add_line();
        info.add_conv_error();

        assert_eq!(info.bytes_read, 100);
        assert_eq!(info.lines_read, 1);
        assert_eq!(info.conv_errors, 1);
    }

    #[test]
    fn test_read_info_saturation() {
        let mut info = ReadInfo::new();
        info.bytes_read = u64::MAX;
        info.add_bytes(1);
        assert_eq!(info.bytes_read, u64::MAX); // Saturated, didn't overflow
    }
}
