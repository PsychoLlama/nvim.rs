//! Buffer write operations for Neovim
//!
//! This crate provides Rust implementations for buffer writing utilities,
//! including line ending conversion, file format handling, and BOM generation.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_const_for_fn)]

pub mod convert;
pub mod encoding;
pub mod error;
pub mod ffi;
pub mod fileinfo;
pub mod format;
pub mod validation;

use std::ffi::c_int;

// =============================================================================
// Constants
// =============================================================================

/// Size of emergency write buffer
pub const SMALLBUFSIZE: usize = 256;

/// Maximum bytes that may remain unconverted between write calls
pub const CONV_RESTLEN: usize = 30;

// =============================================================================
// File Format
// =============================================================================

/// File format types
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FileFormat {
    /// Unix line endings (LF)
    #[default]
    Unix = 0,
    /// DOS/Windows line endings (CRLF)
    Dos = 1,
    /// Classic Mac line endings (CR)
    Mac = 2,
}

impl FileFormat {
    /// Create from raw C integer.
    #[must_use]
    pub const fn from_raw(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Unix),
            1 => Some(Self::Dos),
            2 => Some(Self::Mac),
            _ => None,
        }
    }

    /// Convert to raw C integer.
    #[must_use]
    pub const fn to_raw(self) -> c_int {
        self as c_int
    }

    /// Get the line ending string for this format.
    #[must_use]
    pub const fn line_ending(self) -> &'static [u8] {
        match self {
            Self::Unix => b"\n",
            Self::Dos => b"\r\n",
            Self::Mac => b"\r",
        }
    }

    /// Get the line ending length in bytes.
    #[must_use]
    pub const fn line_ending_len(self) -> usize {
        match self {
            Self::Unix | Self::Mac => 1,
            Self::Dos => 2,
        }
    }
}

// =============================================================================
// FIO Flags (File I/O)
// =============================================================================

/// FIO flag: Latin1 encoding
pub const FIO_LATIN1: u32 = 0x01;
/// FIO flag: UTF-8 encoding
pub const FIO_UTF8: u32 = 0x02;
/// FIO flag: UCS-2 encoding
pub const FIO_UCS2: u32 = 0x04;
/// FIO flag: UCS-4 encoding
pub const FIO_UCS4: u32 = 0x08;
/// FIO flag: UTF-16 encoding
pub const FIO_UTF16: u32 = 0x10;
/// FIO flag: Little endian byte order
pub const FIO_ENDIAN_L: u32 = 0x80;
/// FIO flag: Include BOM (Byte Order Mark)
pub const FIO_BOM: u32 = 0x100;
/// FIO flag: Don't convert NUL to NL
pub const FIO_NOSTRINGS: u32 = 0x200;
/// FIO flag: Skip encoding conversion
pub const FIO_NOCONVERT: u32 = 0x2000;
/// FIO flag: Check for BOM at start of file
pub const FIO_UCSBOM: u32 = 0x4000;
/// FIO flag: Allow all formats
pub const FIO_ALL: i32 = -1;

/// Size of the normal write buffer
pub const WRITEBUFSIZE: usize = 8192;

/// Multiplier for iconv() buffer allocation
pub const ICONV_MULT: usize = 8;

// =============================================================================
// BOM (Byte Order Mark)
// =============================================================================

/// UTF-8 BOM
pub const BOM_UTF8: &[u8] = &[0xEF, 0xBB, 0xBF];
/// UTF-16 LE BOM
pub const BOM_UTF16_LE: &[u8] = &[0xFF, 0xFE];
/// UTF-16 BE BOM
pub const BOM_UTF16_BE: &[u8] = &[0xFE, 0xFF];
/// UTF-32 LE BOM
pub const BOM_UTF32_LE: &[u8] = &[0xFF, 0xFE, 0x00, 0x00];
/// UTF-32 BE BOM
pub const BOM_UTF32_BE: &[u8] = &[0x00, 0x00, 0xFE, 0xFF];

// =============================================================================
// FFI Exports
// =============================================================================

/// Get line ending for file format.
///
/// Returns pointer to static string and sets length.
#[unsafe(no_mangle)]
pub extern "C" fn rs_bufwrite_line_ending_len(format: c_int) -> c_int {
    FileFormat::from_raw(format).map_or(1, FileFormat::line_ending_len) as c_int
}

/// Check if format is Unix.
#[unsafe(no_mangle)]
pub extern "C" fn rs_bufwrite_is_unix(format: c_int) -> c_int {
    c_int::from(FileFormat::from_raw(format) == Some(FileFormat::Unix))
}

/// Check if format is DOS.
#[unsafe(no_mangle)]
pub extern "C" fn rs_bufwrite_is_dos(format: c_int) -> c_int {
    c_int::from(FileFormat::from_raw(format) == Some(FileFormat::Dos))
}

/// Check if format is Mac.
#[unsafe(no_mangle)]
pub extern "C" fn rs_bufwrite_is_mac(format: c_int) -> c_int {
    c_int::from(FileFormat::from_raw(format) == Some(FileFormat::Mac))
}

/// Get BOM length for given FIO flags.
#[unsafe(no_mangle)]
pub extern "C" fn rs_bufwrite_bom_len(flags: u32) -> c_int {
    if flags & FIO_BOM == 0 {
        return 0;
    }

    if flags & FIO_UTF8 != 0 {
        BOM_UTF8.len() as c_int
    } else if flags & (FIO_UCS2 | FIO_UTF16) != 0 {
        BOM_UTF16_LE.len() as c_int
    } else if flags & FIO_UCS4 != 0 {
        BOM_UTF32_LE.len() as c_int
    } else {
        0
    }
}

/// Check if flags indicate little endian.
#[unsafe(no_mangle)]
pub extern "C" fn rs_bufwrite_is_little_endian(flags: u32) -> c_int {
    c_int::from(flags & FIO_ENDIAN_L != 0)
}

/// Get the number of bytes per character for the encoding.
#[unsafe(no_mangle)]
pub extern "C" fn rs_bufwrite_bytes_per_char(flags: u32) -> c_int {
    if flags & FIO_UCS4 != 0 {
        4
    } else if flags & (FIO_UCS2 | FIO_UTF16) != 0 {
        2
    } else {
        1
    }
}

/// Check if flags indicate a Unicode encoding.
#[unsafe(no_mangle)]
pub extern "C" fn rs_bufwrite_is_unicode(flags: u32) -> c_int {
    c_int::from(flags & (FIO_UCS2 | FIO_UCS4 | FIO_UTF8 | FIO_UTF16) != 0)
}

/// Check if flags indicate multi-byte encoding.
#[unsafe(no_mangle)]
pub extern "C" fn rs_bufwrite_is_multibyte(flags: u32) -> c_int {
    c_int::from(flags & (FIO_UCS2 | FIO_UCS4 | FIO_UTF16) != 0)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_format() {
        assert_eq!(FileFormat::from_raw(0), Some(FileFormat::Unix));
        assert_eq!(FileFormat::from_raw(1), Some(FileFormat::Dos));
        assert_eq!(FileFormat::from_raw(2), Some(FileFormat::Mac));
        assert_eq!(FileFormat::from_raw(3), None);

        assert_eq!(FileFormat::Unix.to_raw(), 0);
        assert_eq!(FileFormat::Dos.to_raw(), 1);
        assert_eq!(FileFormat::Mac.to_raw(), 2);
    }

    #[test]
    fn test_line_endings() {
        assert_eq!(FileFormat::Unix.line_ending(), b"\n");
        assert_eq!(FileFormat::Dos.line_ending(), b"\r\n");
        assert_eq!(FileFormat::Mac.line_ending(), b"\r");

        assert_eq!(FileFormat::Unix.line_ending_len(), 1);
        assert_eq!(FileFormat::Dos.line_ending_len(), 2);
        assert_eq!(FileFormat::Mac.line_ending_len(), 1);
    }

    #[test]
    fn test_bom_constants() {
        assert_eq!(BOM_UTF8, &[0xEF, 0xBB, 0xBF]);
        assert_eq!(BOM_UTF16_LE, &[0xFF, 0xFE]);
        assert_eq!(BOM_UTF16_BE, &[0xFE, 0xFF]);
    }

    #[test]
    fn test_fio_flags() {
        // Test individual flags
        assert_eq!(FIO_LATIN1, 0x01);
        assert_eq!(FIO_UTF8, 0x02);
        assert_eq!(FIO_UCS2, 0x04);
        assert_eq!(FIO_UCS4, 0x08);
        assert_eq!(FIO_UTF16, 0x10);
        assert_eq!(FIO_ENDIAN_L, 0x80);
        assert_eq!(FIO_BOM, 0x100);
    }

    #[test]
    fn test_bytes_per_char() {
        assert_eq!(rs_bufwrite_bytes_per_char(FIO_LATIN1), 1);
        assert_eq!(rs_bufwrite_bytes_per_char(FIO_UTF8), 1);
        assert_eq!(rs_bufwrite_bytes_per_char(FIO_UCS2), 2);
        assert_eq!(rs_bufwrite_bytes_per_char(FIO_UTF16), 2);
        assert_eq!(rs_bufwrite_bytes_per_char(FIO_UCS4), 4);
    }

    #[test]
    fn test_is_unicode() {
        assert_eq!(rs_bufwrite_is_unicode(FIO_LATIN1), 0);
        assert_eq!(rs_bufwrite_is_unicode(FIO_UTF8), 1);
        assert_eq!(rs_bufwrite_is_unicode(FIO_UCS2), 1);
        assert_eq!(rs_bufwrite_is_unicode(FIO_UTF16), 1);
        assert_eq!(rs_bufwrite_is_unicode(FIO_UCS4), 1);
    }

    #[test]
    fn test_bom_len() {
        // No BOM flag
        assert_eq!(rs_bufwrite_bom_len(FIO_UTF8), 0);

        // With BOM flag
        assert_eq!(rs_bufwrite_bom_len(FIO_BOM | FIO_UTF8), 3);
        assert_eq!(rs_bufwrite_bom_len(FIO_BOM | FIO_UCS2), 2);
        assert_eq!(rs_bufwrite_bom_len(FIO_BOM | FIO_UTF16), 2);
        assert_eq!(rs_bufwrite_bom_len(FIO_BOM | FIO_UCS4), 4);
        assert_eq!(rs_bufwrite_bom_len(FIO_BOM | FIO_LATIN1), 0);
    }
}
