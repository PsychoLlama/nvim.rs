//! File writing infrastructure for Neovim.
//!
//! This module provides:
//! - EINTR-safe write operations
//! - Line ending conversion
//! - Encoding conversion for output
//! - Atomic write support

use std::ffi::c_int;
use std::io::{self, Write};

use crate::{FileFormat, FIO_ENDIAN_L, FIO_UCS2, FIO_UCS4, FIO_UTF16};

// =============================================================================
// Constants
// =============================================================================

/// Default buffer size for file writing operations.
pub const WRITE_BUFFER_SIZE: usize = 8192;

/// Size of emergency write buffer for small writes.
pub const SMALL_BUFFER_SIZE: usize = 256;

/// Buffer size for unconverted bytes between write operations.
pub const CONV_RESTLEN: usize = 30;

// =============================================================================
// EINTR-Safe I/O Operations
// =============================================================================

/// Write to a file descriptor, retrying on EINTR and handling partial writes.
///
/// EINTR can occur when a signal (like SIGWINCH on terminal resize)
/// interrupts a write system call. This function automatically retries
/// and handles partial writes.
///
/// # Arguments
/// * `fd` - File descriptor to write to
/// * `buf` - Buffer containing data to write
///
/// # Returns
/// Number of bytes written, or an error
#[cfg(unix)]
pub fn write_eintr(fd: c_int, buf: &[u8]) -> io::Result<usize> {
    use std::os::unix::io::FromRawFd;

    // Create a File from the raw fd without taking ownership
    // SAFETY: We don't close the fd, the caller manages its lifetime
    let mut file = std::mem::ManuallyDrop::new(unsafe { std::fs::File::from_raw_fd(fd) });

    let mut total_written = 0;

    while total_written < buf.len() {
        match file.write(&buf[total_written..]) {
            Ok(0) => {
                // Shouldn't happen, but handle it
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "write returned 0 bytes",
                ));
            }
            Ok(n) => {
                total_written += n;
            }
            Err(e) if e.kind() == io::ErrorKind::Interrupted => {
                // EINTR, retry
                continue;
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    Ok(total_written)
}

/// FFI wrapper for write_eintr -- directly replaces the C `write_eintr` symbol.
///
/// # Safety
/// - `fd` must be a valid, open file descriptor
/// - `buf` must point to a valid buffer of at least `bufsize` bytes
#[cfg(unix)]
#[export_name = "write_eintr"]
pub unsafe extern "C" fn rs_write_eintr(
    fd: c_int,
    buf: *mut std::ffi::c_void,
    bufsize: usize,
) -> c_int {
    if buf.is_null() || bufsize == 0 {
        return 0; // Nothing to write
    }

    let slice = std::slice::from_raw_parts(buf as *const u8, bufsize);
    match write_eintr(fd, slice) {
        Ok(n) => n as c_int,
        Err(_) => -1,
    }
}

// =============================================================================
// Line Ending Conversion
// =============================================================================

/// Convert a line ending to the target file format.
///
/// # Arguments
/// * `format` - Target file format
///
/// # Returns
/// The byte sequence for the line ending
#[inline]
pub fn line_ending_bytes(format: FileFormat) -> &'static [u8] {
    format.line_ending()
}

/// Estimate the output size for a buffer with line ending conversion.
///
/// # Arguments
/// * `input_len` - Length of input data
/// * `line_count` - Number of lines
/// * `src_format` - Source file format
/// * `dst_format` - Destination file format
///
/// # Returns
/// Estimated output buffer size
pub fn estimate_converted_size(
    input_len: usize,
    line_count: usize,
    src_format: FileFormat,
    dst_format: FileFormat,
) -> usize {
    let src_eol_len = src_format.line_ending().len();
    let dst_eol_len = dst_format.line_ending().len();

    if dst_eol_len > src_eol_len {
        // May expand (e.g., Unix -> DOS)
        input_len + (line_count * (dst_eol_len - src_eol_len))
    } else {
        // Same or smaller (e.g., DOS -> Unix)
        input_len
    }
}

// =============================================================================
// Unicode Character Encoding
// =============================================================================

/// Result of encoding a Unicode character.
#[derive(Debug, Clone, Copy)]
pub struct EncodedChar {
    /// Number of bytes written
    pub len: usize,
    /// Whether an error occurred (character out of range)
    pub error: bool,
}

/// Encode a Unicode character to bytes in the specified format.
///
/// # Arguments
/// * `codepoint` - Unicode code point to encode
/// * `output` - Buffer to write encoded bytes
/// * `fio_flags` - FIO_* flags specifying the encoding
///
/// # Returns
/// Encoding result with length and error status
pub fn encode_ucs_char(codepoint: u32, output: &mut [u8], fio_flags: c_int) -> EncodedChar {
    let mut error = false;
    let mut pos = 0;

    if fio_flags & FIO_UCS4 != 0 {
        // UCS-4: 4 bytes per character
        if output.len() < 4 {
            return EncodedChar {
                len: 0,
                error: true,
            };
        }

        if fio_flags & FIO_ENDIAN_L != 0 {
            // Little endian
            output[pos] = codepoint as u8;
            output[pos + 1] = (codepoint >> 8) as u8;
            output[pos + 2] = (codepoint >> 16) as u8;
            output[pos + 3] = (codepoint >> 24) as u8;
        } else {
            // Big endian
            output[pos] = (codepoint >> 24) as u8;
            output[pos + 1] = (codepoint >> 16) as u8;
            output[pos + 2] = (codepoint >> 8) as u8;
            output[pos + 3] = codepoint as u8;
        }
        pos = 4;
    } else if fio_flags & (FIO_UCS2 | FIO_UTF16) != 0 {
        // UCS-2 or UTF-16: 2 or 4 bytes per character
        if codepoint >= 0x10000 {
            if fio_flags & FIO_UTF16 != 0 {
                // UTF-16: encode as surrogate pair
                if codepoint >= 0x110000 {
                    error = true;
                }

                let adjusted = codepoint - 0x10000;
                let high = ((adjusted >> 10) & 0x3FF) + 0xD800;
                let low = (adjusted & 0x3FF) + 0xDC00;

                if output.len() < 4 {
                    return EncodedChar {
                        len: 0,
                        error: true,
                    };
                }

                if fio_flags & FIO_ENDIAN_L != 0 {
                    output[pos] = high as u8;
                    output[pos + 1] = (high >> 8) as u8;
                    output[pos + 2] = low as u8;
                    output[pos + 3] = (low >> 8) as u8;
                } else {
                    output[pos] = (high >> 8) as u8;
                    output[pos + 1] = high as u8;
                    output[pos + 2] = (low >> 8) as u8;
                    output[pos + 3] = low as u8;
                }
                pos = 4;
            } else {
                // UCS-2: can't represent code points >= 0x10000
                error = true;
                // Still write something
                if output.len() < 2 {
                    return EncodedChar {
                        len: 0,
                        error: true,
                    };
                }
                let replacement = 0xFFFD_u32; // Replacement character
                if fio_flags & FIO_ENDIAN_L != 0 {
                    output[pos] = replacement as u8;
                    output[pos + 1] = (replacement >> 8) as u8;
                } else {
                    output[pos] = (replacement >> 8) as u8;
                    output[pos + 1] = replacement as u8;
                }
                pos = 2;
            }
        } else {
            // BMP character: 2 bytes
            if output.len() < 2 {
                return EncodedChar {
                    len: 0,
                    error: true,
                };
            }

            if fio_flags & FIO_ENDIAN_L != 0 {
                output[pos] = codepoint as u8;
                output[pos + 1] = (codepoint >> 8) as u8;
            } else {
                output[pos] = (codepoint >> 8) as u8;
                output[pos + 1] = codepoint as u8;
            }
            pos = 2;
        }
    } else {
        // Latin-1 or similar: 1 byte per character
        if output.is_empty() {
            return EncodedChar {
                len: 0,
                error: true,
            };
        }

        if codepoint > 0xFF {
            error = true;
            output[pos] = b'?'; // Replacement
        } else {
            output[pos] = codepoint as u8;
        }
        pos = 1;
    }

    EncodedChar { len: pos, error }
}

/// FFI wrapper for encode_ucs_char.
///
/// # Safety
/// `output` must point to a valid buffer of at least `output_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_encode_ucs_char(
    codepoint: u32,
    output: *mut u8,
    output_len: usize,
    fio_flags: c_int,
    bytes_written: *mut usize,
) -> c_int {
    if output.is_null() || bytes_written.is_null() {
        return -1;
    }

    let slice = std::slice::from_raw_parts_mut(output, output_len);
    let result = encode_ucs_char(codepoint, slice, fio_flags);

    *bytes_written = result.len;
    if result.error {
        1
    } else {
        0
    }
}

// =============================================================================
// BOM (Byte Order Mark) Writing
// =============================================================================

/// Get the BOM bytes for a given encoding.
///
/// # Arguments
/// * `fio_flags` - FIO_* flags specifying the encoding
///
/// # Returns
/// BOM bytes, or empty slice if no BOM is needed
pub fn get_bom_bytes(fio_flags: c_int) -> &'static [u8] {
    // UTF-8 BOM
    const UTF8_BOM: &[u8] = &[0xEF, 0xBB, 0xBF];
    // UTF-16 LE BOM
    const UTF16_LE_BOM: &[u8] = &[0xFF, 0xFE];
    // UTF-16 BE BOM
    const UTF16_BE_BOM: &[u8] = &[0xFE, 0xFF];
    // UCS-4 LE BOM
    const UCS4_LE_BOM: &[u8] = &[0xFF, 0xFE, 0x00, 0x00];
    // UCS-4 BE BOM
    const UCS4_BE_BOM: &[u8] = &[0x00, 0x00, 0xFE, 0xFF];

    if fio_flags & FIO_UCS4 != 0 {
        if fio_flags & FIO_ENDIAN_L != 0 {
            UCS4_LE_BOM
        } else {
            UCS4_BE_BOM
        }
    } else if fio_flags & (FIO_UCS2 | FIO_UTF16) != 0 {
        if fio_flags & FIO_ENDIAN_L != 0 {
            UTF16_LE_BOM
        } else {
            UTF16_BE_BOM
        }
    } else {
        // UTF-8 or other: use UTF-8 BOM
        UTF8_BOM
    }
}

/// FFI wrapper to get BOM bytes.
///
/// # Safety
/// `len_out` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_get_bom_bytes(fio_flags: c_int, len_out: *mut usize) -> *const u8 {
    let bom = get_bom_bytes(fio_flags);
    if !len_out.is_null() {
        *len_out = bom.len();
    }
    bom.as_ptr()
}

// =============================================================================
// Write Information Tracking
// =============================================================================

/// Information about a file write operation.
#[derive(Debug, Clone, Default)]
pub struct WriteInfo {
    /// Total bytes written
    pub bytes_written: u64,
    /// Number of lines written
    pub lines_written: u64,
    /// Whether a BOM was written
    pub bom_written: bool,
    /// Number of conversion errors
    pub conv_errors: u64,
    /// First line with a conversion error (0 if none)
    pub first_error_line: u64,
}

impl WriteInfo {
    /// Create a new WriteInfo with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add bytes to the total.
    pub fn add_bytes(&mut self, count: u64) {
        self.bytes_written = self.bytes_written.saturating_add(count);
    }

    /// Increment the line count.
    pub fn add_line(&mut self) {
        self.lines_written = self.lines_written.saturating_add(1);
    }

    /// Record a conversion error.
    pub fn add_conv_error(&mut self, line_num: u64) {
        self.conv_errors = self.conv_errors.saturating_add(1);
        if self.first_error_line == 0 {
            self.first_error_line = line_num;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_ending_bytes() {
        assert_eq!(line_ending_bytes(FileFormat::Unix), b"\n");
        assert_eq!(line_ending_bytes(FileFormat::Dos), b"\r\n");
        assert_eq!(line_ending_bytes(FileFormat::Mac), b"\r");
    }

    #[test]
    fn test_estimate_converted_size() {
        // Same format
        assert_eq!(
            estimate_converted_size(100, 10, FileFormat::Unix, FileFormat::Unix),
            100
        );

        // Unix to DOS (expand)
        assert_eq!(
            estimate_converted_size(100, 10, FileFormat::Unix, FileFormat::Dos),
            110 // 100 + 10 * (2-1)
        );

        // DOS to Unix (same or shrink)
        assert_eq!(
            estimate_converted_size(110, 10, FileFormat::Dos, FileFormat::Unix),
            110 // No expansion
        );
    }

    #[test]
    fn test_encode_ucs_char_ucs4() {
        let mut buf = [0u8; 8];

        // UCS-4 Big Endian
        let result = encode_ucs_char(0x1234, &mut buf, FIO_UCS4);
        assert_eq!(result.len, 4);
        assert!(!result.error);
        assert_eq!(buf[0..4], [0x00, 0x00, 0x12, 0x34]);

        // UCS-4 Little Endian
        let result = encode_ucs_char(0x1234, &mut buf, FIO_UCS4 | FIO_ENDIAN_L);
        assert_eq!(result.len, 4);
        assert!(!result.error);
        assert_eq!(buf[0..4], [0x34, 0x12, 0x00, 0x00]);
    }

    #[test]
    fn test_encode_ucs_char_ucs2() {
        let mut buf = [0u8; 4];

        // UCS-2 Big Endian
        let result = encode_ucs_char(0x1234, &mut buf, FIO_UCS2);
        assert_eq!(result.len, 2);
        assert!(!result.error);
        assert_eq!(buf[0..2], [0x12, 0x34]);

        // UCS-2 Little Endian
        let result = encode_ucs_char(0x1234, &mut buf, FIO_UCS2 | FIO_ENDIAN_L);
        assert_eq!(result.len, 2);
        assert!(!result.error);
        assert_eq!(buf[0..2], [0x34, 0x12]);

        // UCS-2 can't encode > 0xFFFF
        let result = encode_ucs_char(0x10000, &mut buf, FIO_UCS2);
        assert!(result.error);
    }

    #[test]
    fn test_encode_ucs_char_utf16_surrogate() {
        let mut buf = [0u8; 8];

        // UTF-16 Big Endian with surrogate pair
        let result = encode_ucs_char(0x1F600, &mut buf, FIO_UTF16); // 😀
        assert_eq!(result.len, 4);
        assert!(!result.error);
        // Should produce surrogate pair: D83D DE00
        assert_eq!(buf[0..4], [0xD8, 0x3D, 0xDE, 0x00]);

        // UTF-16 Little Endian
        let result = encode_ucs_char(0x1F600, &mut buf, FIO_UTF16 | FIO_ENDIAN_L);
        assert_eq!(result.len, 4);
        assert!(!result.error);
        assert_eq!(buf[0..4], [0x3D, 0xD8, 0x00, 0xDE]);
    }

    #[test]
    fn test_encode_ucs_char_latin1() {
        let mut buf = [0u8; 4];

        // ASCII character
        let result = encode_ucs_char(0x41, &mut buf, 0);
        assert_eq!(result.len, 1);
        assert!(!result.error);
        assert_eq!(buf[0], 0x41);

        // Extended Latin-1
        let result = encode_ucs_char(0xE9, &mut buf, 0);
        assert_eq!(result.len, 1);
        assert!(!result.error);
        assert_eq!(buf[0], 0xE9);

        // Out of range for Latin-1
        let result = encode_ucs_char(0x100, &mut buf, 0);
        assert!(result.error);
        assert_eq!(buf[0], b'?');
    }

    #[test]
    fn test_get_bom_bytes() {
        // UTF-16 LE
        let bom = get_bom_bytes(FIO_UTF16 | FIO_ENDIAN_L);
        assert_eq!(bom, &[0xFF, 0xFE]);

        // UTF-16 BE
        let bom = get_bom_bytes(FIO_UTF16);
        assert_eq!(bom, &[0xFE, 0xFF]);

        // UCS-4 LE
        let bom = get_bom_bytes(FIO_UCS4 | FIO_ENDIAN_L);
        assert_eq!(bom, &[0xFF, 0xFE, 0x00, 0x00]);

        // UCS-4 BE
        let bom = get_bom_bytes(FIO_UCS4);
        assert_eq!(bom, &[0x00, 0x00, 0xFE, 0xFF]);

        // Default (UTF-8)
        let bom = get_bom_bytes(0);
        assert_eq!(bom, &[0xEF, 0xBB, 0xBF]);
    }

    #[test]
    fn test_write_info() {
        let mut info = WriteInfo::new();
        assert_eq!(info.bytes_written, 0);
        assert_eq!(info.lines_written, 0);

        info.add_bytes(100);
        info.add_line();
        info.add_conv_error(5);

        assert_eq!(info.bytes_written, 100);
        assert_eq!(info.lines_written, 1);
        assert_eq!(info.conv_errors, 1);
        assert_eq!(info.first_error_line, 5);

        // Second error shouldn't change first_error_line
        info.add_conv_error(10);
        assert_eq!(info.conv_errors, 2);
        assert_eq!(info.first_error_line, 5);
    }
}
