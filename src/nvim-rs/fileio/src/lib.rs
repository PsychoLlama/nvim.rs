//! File I/O utilities for Neovim
//!
//! Provides utility functions for file operations including:
//! - File time comparison with FAT filesystem tolerance
//! - Device file detection (/dev/fd/N)
//! - BOM (Byte Order Mark) detection for Unicode files
//! - File encoding flags and conversion helpers
//! - File format detection and conversion
//! - Read/write flag management
//! - Encoding detection and conversion

#![allow(unsafe_code)]

use std::ffi::{c_char, c_int, CStr};

pub mod encoding;
pub mod read;

// =============================================================================
// Opaque Handles
// =============================================================================
// These types represent pointers to C structs that Rust doesn't need to know
// the internals of. They're used for type safety when passing handles between
// Rust and C.

/// Opaque handle to a C `FileInfo` struct.
/// Used for file metadata (size, mtime, permissions, etc.)
#[repr(C)]
pub struct FileInfoHandle {
    _opaque: [u8; 0],
}

/// Opaque handle to a C `buf_T` struct.
/// Represents a Neovim buffer.
#[repr(C)]
pub struct BufferHandle {
    _opaque: [u8; 0],
}

/// Opaque handle to a C `context_sha256_T` struct.
/// Used for SHA-256 checksum computation.
#[repr(C)]
pub struct Sha256ContextHandle {
    _opaque: [u8; 0],
}

/// Opaque handle to a C `exarg_T` struct.
/// Represents Ex command arguments.
#[repr(C)]
pub struct ExArgHandle {
    _opaque: [u8; 0],
}

// =============================================================================
// File Format (End-of-Line Style)
// =============================================================================

/// End-of-line format for files.
///
/// Corresponds to Neovim's `EOL_*` constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i32)]
pub enum FileFormat {
    /// Unknown format (not yet determined)
    #[default]
    Unknown = -1,
    /// Unix format: LF only (`\n`)
    Unix = 0,
    /// DOS/Windows format: CR LF (`\r\n`)
    Dos = 1,
    /// Classic Mac format: CR only (`\r`)
    Mac = 2,
}

impl FileFormat {
    /// Convert from C integer value.
    #[inline]
    pub fn from_c(value: c_int) -> Self {
        match value {
            0 => FileFormat::Unix,
            1 => FileFormat::Dos,
            2 => FileFormat::Mac,
            _ => FileFormat::Unknown,
        }
    }

    /// Convert to C integer value.
    #[inline]
    pub fn to_c(self) -> c_int {
        self as c_int
    }

    /// Returns the line ending string for this format.
    #[inline]
    pub fn line_ending(self) -> &'static [u8] {
        match self {
            FileFormat::Unix | FileFormat::Unknown => b"\n",
            FileFormat::Dos => b"\r\n",
            FileFormat::Mac => b"\r",
        }
    }
}

// =============================================================================
// Read Flags
// =============================================================================

bitflags::bitflags! {
    /// Flags for `readfile()` operations.
    ///
    /// These control how files are read into buffers.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct ReadFlags: u32 {
        /// Reading a file into a new buffer
        const NEW = 0x01;
        /// Reading filter output
        const FILTER = 0x02;
        /// Read from stdin instead of a file
        const STDIN = 0x04;
        /// Read from curbuf (converting after reading stdin)
        const BUFFER = 0x08;
        /// Reading into a dummy buffer (to check if file contents changed)
        const DUMMY = 0x10;
        /// Don't clear undo info or read it from a file
        const KEEP_UNDO = 0x20;
        /// Read from fifo/socket instead of a file
        const FIFO = 0x40;
        /// Do not trigger BufWinEnter
        const NOWINENTER = 0x80;
        /// Do not read a file, only trigger BufReadCmd
        const NOFILE = 0x100;
    }
}

impl ReadFlags {
    /// Convert from C integer flags.
    #[inline]
    pub fn from_c(flags: c_int) -> Self {
        Self::from_bits_truncate(flags as u32)
    }

    /// Convert to C integer flags.
    #[inline]
    pub fn to_c(self) -> c_int {
        self.bits() as c_int
    }
}

// =============================================================================
// Write Flags (for buf_write)
// =============================================================================

bitflags::bitflags! {
    /// Flags for `buf_write()` operations.
    ///
    /// These control how buffers are written to files.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct WriteFlags: u32 {
        /// Writing the whole file
        const WHOLE = 0x01;
        /// Appending to the file
        const APPEND = 0x02;
        /// Writing part of the file
        const PART = 0x04;
        /// ":w!" forced write
        const FORCE = 0x08;
        /// Writing for ":saveas" or similar
        const SAVEAS = 0x10;
        /// Writing to a new file
        const NEW = 0x20;
    }
}

impl WriteFlags {
    /// Convert from C integer flags.
    #[inline]
    pub fn from_c(flags: c_int) -> Self {
        Self::from_bits_truncate(flags as u32)
    }

    /// Convert to C integer flags.
    #[inline]
    pub fn to_c(self) -> c_int {
        self.bits() as c_int
    }
}

// =============================================================================
// Conversion Type
// =============================================================================

/// Type of character encoding conversion.
///
/// Corresponds to Neovim's `CONV_*` constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(i32)]
pub enum ConvType {
    /// No conversion needed
    #[default]
    None = 0,
    /// Convert to UTF-8
    ToUtf8 = 1,
    /// Convert Latin-9 to UTF-8
    Latin9ToUtf8 = 2,
    /// Convert to Latin-1
    ToLatin1 = 3,
    /// Convert to Latin-9
    ToLatin9 = 4,
    /// Use iconv for conversion
    Iconv = 5,
}

impl ConvType {
    /// Convert from C integer value.
    #[inline]
    pub fn from_c(value: c_int) -> Self {
        match value {
            0 => ConvType::None,
            1 => ConvType::ToUtf8,
            2 => ConvType::Latin9ToUtf8,
            3 => ConvType::ToLatin1,
            4 => ConvType::ToLatin9,
            5 => ConvType::Iconv,
            _ => ConvType::None,
        }
    }

    /// Convert to C integer value.
    #[inline]
    pub fn to_c(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Bad Character Handling
// =============================================================================

/// How to handle invalid/unconvertible characters during encoding conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BadCharBehavior {
    /// Replace invalid characters with '?' (default)
    Replace,
    /// Keep invalid characters as-is
    Keep,
    /// Drop/erase invalid characters
    Drop,
    /// Replace with a specific character
    ReplaceWith(u8),
}

impl BadCharBehavior {
    /// The default replacement character.
    pub const DEFAULT_REPLACEMENT: u8 = b'?';

    /// Convert from C integer value.
    #[inline]
    pub fn from_c(value: c_int) -> Self {
        match value {
            -1 => BadCharBehavior::Keep,
            -2 => BadCharBehavior::Drop,
            c if c == Self::DEFAULT_REPLACEMENT as c_int => BadCharBehavior::Replace,
            c if (0..=255).contains(&c) => BadCharBehavior::ReplaceWith(c as u8),
            _ => BadCharBehavior::Replace,
        }
    }

    /// Convert to C integer value.
    #[inline]
    pub fn to_c(self) -> c_int {
        match self {
            BadCharBehavior::Replace => Self::DEFAULT_REPLACEMENT as c_int,
            BadCharBehavior::Keep => -1,
            BadCharBehavior::Drop => -2,
            BadCharBehavior::ReplaceWith(c) => c as c_int,
        }
    }
}

// =============================================================================
// File I/O Encoding Flags
// =============================================================================

// Legacy constants for C compatibility (still exported for FFI)
/// Convert Latin1 encoding
pub const FIO_LATIN1: c_int = 0x01;
/// Convert UTF-8 encoding
pub const FIO_UTF8: c_int = 0x02;
/// Convert UCS-2 encoding
pub const FIO_UCS2: c_int = 0x04;
/// Convert UCS-4 encoding
pub const FIO_UCS4: c_int = 0x08;
/// Convert UTF-16 encoding
pub const FIO_UTF16: c_int = 0x10;
/// Little endian byte order
pub const FIO_ENDIAN_L: c_int = 0x80;
/// Skip encoding conversion
pub const FIO_NOCONVERT: c_int = 0x2000;
/// Check for BOM at start of file
pub const FIO_UCSBOM: c_int = 0x4000;
/// Allow all formats (for BOM detection)
pub const FIO_ALL: c_int = -1;

bitflags::bitflags! {
    /// File I/O encoding flags.
    ///
    /// These flags control encoding detection and conversion during file I/O.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
    pub struct FioFlags: u32 {
        /// Convert Latin-1 encoding
        const LATIN1 = 0x01;
        /// Convert UTF-8 encoding
        const UTF8 = 0x02;
        /// Convert UCS-2 encoding
        const UCS2 = 0x04;
        /// Convert UCS-4 encoding
        const UCS4 = 0x08;
        /// Convert UTF-16 encoding
        const UTF16 = 0x10;
        /// Little endian byte order
        const ENDIAN_L = 0x80;
        /// Skip encoding conversion
        const NOCONVERT = 0x2000;
        /// Check for BOM at start of file
        const UCSBOM = 0x4000;

        // Common combinations
        /// UCS-2 Little Endian
        const UCS2_LE = Self::UCS2.bits() | Self::ENDIAN_L.bits();
        /// UCS-4 Little Endian
        const UCS4_LE = Self::UCS4.bits() | Self::ENDIAN_L.bits();
        /// UTF-16 Little Endian
        const UTF16_LE = Self::UTF16.bits() | Self::ENDIAN_L.bits();
    }
}

impl FioFlags {
    /// Special value representing all formats (for BOM detection).
    /// This is -1 as a signed integer, meaning all bits set.
    pub const ALL: c_int = -1;

    /// Convert from C integer flags.
    ///
    /// Handles the special case of -1 (FIO_ALL).
    #[inline]
    pub fn from_c(flags: c_int) -> Option<Self> {
        if flags == -1 {
            None // Represents FIO_ALL
        } else {
            Some(Self::from_bits_truncate(flags as u32))
        }
    }

    /// Convert to C integer flags.
    #[inline]
    pub fn to_c(self) -> c_int {
        self.bits() as c_int
    }

    /// Check if this represents "all formats" mode (FIO_ALL = -1).
    #[inline]
    pub fn is_fio_all(flags: c_int) -> bool {
        flags == -1
    }

    /// Returns the encoding name for common flag combinations.
    pub fn encoding_name(self) -> Option<&'static str> {
        match self {
            f if f == Self::LATIN1 => Some("latin1"),
            f if f == Self::UTF8 => Some("utf-8"),
            f if f == Self::UCS2 => Some("ucs-2"),
            f if f == Self::UCS2_LE => Some("ucs-2le"),
            f if f == Self::UCS4 => Some("ucs-4"),
            f if f == Self::UCS4_LE => Some("ucs-4le"),
            f if f == Self::UTF16 => Some("utf-16"),
            f if f == Self::UTF16_LE => Some("utf-16le"),
            _ => None,
        }
    }
}

/// Check if file times differ.
///
/// On Linux/Windows, there's a FAT filesystem tolerance: the seconds portion
/// can differ by up to 1 second due to FAT's 5-bit second storage limitation.
///
/// # Arguments
/// * `file_sec` - File modification time (seconds)
/// * `file_nsec` - File modification time (nanoseconds)
/// * `mtime` - Expected modification time (seconds)
/// * `mtime_ns` - Expected modification time (nanoseconds)
/// * `fat_tolerance` - Whether to apply FAT filesystem tolerance (Linux/Windows)
///
/// # Returns
/// `true` if the times differ, `false` if they match
#[inline]
pub fn time_differs(
    file_sec: i64,
    file_nsec: i64,
    mtime: i64,
    mtime_ns: i64,
    fat_tolerance: bool,
) -> bool {
    if file_nsec != mtime_ns {
        return true;
    }

    if fat_tolerance {
        // On FAT filesystem, there are only 5 bits to store the seconds.
        // The time may change unexpectedly by one second during inode flush.
        let diff = file_sec - mtime;
        !(-1..=1).contains(&diff)
    } else {
        file_sec != mtime
    }
}

/// FFI wrapper for `time_differs`.
///
/// # Safety
/// All parameters are plain integers, so this is safe.
#[no_mangle]
pub extern "C" fn rs_time_differs(
    file_sec: i64,
    file_nsec: i64,
    mtime: i64,
    mtime_ns: i64,
    fat_tolerance: c_int,
) -> c_int {
    c_int::from(time_differs(
        file_sec,
        file_nsec,
        mtime,
        mtime_ns,
        fat_tolerance != 0,
    ))
}

/// Check if a filename is a /dev/fd/ special file.
///
/// The /dev/fd/ mechanism is provided by some shells on some operating systems,
/// e.g., bash on SunOS. Do not accept "/dev/fd/[012]" since opening these may
/// hang Vim (stdin/stdout/stderr).
///
/// Pattern must match:
/// - Starts with "/dev/fd/"
/// - Followed by one or more digits
/// - Nothing after the digits
/// - Not just "/dev/fd/0", "/dev/fd/1", or "/dev/fd/2" (single digit 0, 1, or 2)
#[inline]
fn is_dev_fd_file_impl(fname: &[u8]) -> bool {
    // Must start with "/dev/fd/"
    if !fname.starts_with(b"/dev/fd/") {
        return false;
    }

    let after_prefix = &fname[8..];

    // Must have at least one digit
    if after_prefix.is_empty() || !after_prefix[0].is_ascii_digit() {
        return false;
    }

    // Find end of digits
    let mut digit_end = 0;
    for (i, &c) in after_prefix.iter().enumerate() {
        if c.is_ascii_digit() {
            digit_end = i + 1;
        } else {
            break;
        }
    }

    // Must be NUL-terminated (no trailing chars) - for C strings, the byte after digits
    // is either NUL (end of slice from CStr) or we check if there's anything after
    if digit_end < after_prefix.len() && after_prefix[digit_end] != 0 {
        return false;
    }

    // Now check: if it's a single digit 0, 1, or 2, reject it
    // Accept if: more than one digit, OR single digit that's not 0/1/2
    if after_prefix.len() == 1 || (digit_end == 1 && after_prefix.len() > 1) {
        // Single digit case - reject 0, 1, 2
        let single = after_prefix[0];
        if single == b'0' || single == b'1' || single == b'2' {
            return false;
        }
    }

    true
}

/// FFI wrapper for `is_dev_fd_file`.
///
/// Check if fname is a /dev/fd/N path (excluding 0, 1, 2).
///
/// # Safety
/// `fname` must be a valid C string pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_is_dev_fd_file(fname: *const c_char) -> bool {
    if fname.is_null() {
        return false;
    }

    let c_str = unsafe { CStr::from_ptr(fname) };
    is_dev_fd_file_impl(c_str.to_bytes())
}

// =============================================================================
// BOM (Byte Order Mark) Detection
// =============================================================================

/// Result of BOM detection, containing the encoding name and BOM length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BomResult {
    /// The encoding name (e.g., "utf-8", "utf-16le")
    pub encoding: &'static str,
    /// Length of the BOM in bytes
    pub len: usize,
}

/// Check for a Unicode BOM (Byte Order Mark) at the start of a byte buffer.
///
/// Detects the following BOMs:
/// - UTF-8: EF BB BF (3 bytes)
/// - UTF-16 LE: FF FE (2 bytes)
/// - UTF-16 BE: FE FF (2 bytes)
/// - UCS-4 LE: FF FE 00 00 (4 bytes)
/// - UCS-4 BE: 00 00 FE FF (4 bytes)
///
/// # Arguments
/// * `data` - Byte slice to check (must be at least 2 bytes)
/// * `flags` - FIO_* flags indicating which encodings to check for
///
/// # Returns
/// `Some(BomResult)` with encoding name and BOM length, or `None` if no BOM found.
pub fn check_for_bom(data: &[u8], flags: c_int) -> Option<BomResult> {
    if data.len() < 2 {
        return None;
    }

    // UTF-8 BOM: EF BB BF
    if data.len() >= 3
        && data[0] == 0xef
        && data[1] == 0xbb
        && data[2] == 0xbf
        && (flags == FIO_ALL || flags == FIO_UTF8 || flags == 0)
    {
        return Some(BomResult {
            encoding: "utf-8",
            len: 3,
        });
    }

    // Check FF FE prefix (UTF-16 LE or UCS-4 LE)
    if data[0] == 0xff && data[1] == 0xfe {
        // UCS-4 LE: FF FE 00 00
        if data.len() >= 4
            && data[2] == 0
            && data[3] == 0
            && (flags == FIO_ALL || flags == (FIO_UCS4 | FIO_ENDIAN_L))
        {
            return Some(BomResult {
                encoding: "ucs-4le",
                len: 4,
            });
        }

        // UCS-2 LE: FF FE
        if flags == (FIO_UCS2 | FIO_ENDIAN_L) {
            return Some(BomResult {
                encoding: "ucs-2le",
                len: 2,
            });
        }

        // UTF-16 LE (preferred, also works for UCS-2 LE): FF FE
        if flags == FIO_ALL || flags == (FIO_UTF16 | FIO_ENDIAN_L) {
            return Some(BomResult {
                encoding: "utf-16le",
                len: 2,
            });
        }
    }

    // UTF-16 BE or UCS-2 BE: FE FF
    if data[0] == 0xfe
        && data[1] == 0xff
        && (flags == FIO_ALL || flags == FIO_UCS2 || flags == FIO_UTF16)
    {
        // Default to utf-16, it works also for ucs-2 text
        if flags == FIO_UCS2 {
            return Some(BomResult {
                encoding: "ucs-2",
                len: 2,
            });
        }
        return Some(BomResult {
            encoding: "utf-16",
            len: 2,
        });
    }

    // UCS-4 BE: 00 00 FE FF
    if data.len() >= 4
        && data[0] == 0
        && data[1] == 0
        && data[2] == 0xfe
        && data[3] == 0xff
        && (flags == FIO_ALL || flags == FIO_UCS4)
    {
        return Some(BomResult {
            encoding: "ucs-4",
            len: 4,
        });
    }

    None
}

/// FFI wrapper for `check_for_bom`.
///
/// Checks for a Unicode BOM at the start of the given byte buffer.
///
/// # Arguments
/// * `data` - Pointer to the byte buffer
/// * `size` - Size of the buffer in bytes (must be >= 2)
/// * `lenp` - Output pointer for BOM length
/// * `flags` - FIO_* flags indicating which encodings to check
///
/// # Returns
/// Pointer to a static encoding name string, or NULL if no BOM found.
///
/// # Safety
/// - `data` must point to a valid buffer of at least `size` bytes
/// - `lenp` must be a valid pointer for writing
#[no_mangle]
pub unsafe extern "C" fn rs_check_for_bom(
    data: *const u8,
    size: c_int,
    lenp: *mut c_int,
    flags: c_int,
) -> *const c_char {
    if data.is_null() || size < 2 || lenp.is_null() {
        if !lenp.is_null() {
            *lenp = 2; // Default length as in C code
        }
        return std::ptr::null();
    }

    let slice = std::slice::from_raw_parts(data, size as usize);

    match check_for_bom(slice, flags) {
        Some(result) => {
            *lenp = result.len as c_int;
            // Return pointer to static string
            match result.encoding {
                "utf-8" => c"utf-8".as_ptr(),
                "utf-16" => c"utf-16".as_ptr(),
                "utf-16le" => c"utf-16le".as_ptr(),
                "ucs-2" => c"ucs-2".as_ptr(),
                "ucs-2le" => c"ucs-2le".as_ptr(),
                "ucs-4" => c"ucs-4".as_ptr(),
                "ucs-4le" => c"ucs-4le".as_ptr(),
                _ => std::ptr::null(),
            }
        }
        None => {
            *lenp = 2; // Default length as in C code
            std::ptr::null()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_differs_exact_match() {
        // Exact match, no tolerance
        assert!(!time_differs(1000, 500, 1000, 500, false));
        assert!(!time_differs(1000, 500, 1000, 500, true));
    }

    #[test]
    fn test_time_differs_nanosec_mismatch() {
        // Nanoseconds differ - always different
        assert!(time_differs(1000, 500, 1000, 501, false));
        assert!(time_differs(1000, 500, 1000, 501, true));
    }

    #[test]
    fn test_time_differs_sec_mismatch_no_tolerance() {
        // Seconds differ by 1, no FAT tolerance
        assert!(time_differs(1001, 500, 1000, 500, false));
        assert!(time_differs(999, 500, 1000, 500, false));
    }

    #[test]
    fn test_time_differs_fat_tolerance() {
        // Seconds differ by exactly 1 - FAT tolerance allows this
        assert!(!time_differs(1001, 500, 1000, 500, true));
        assert!(!time_differs(999, 500, 1000, 500, true));

        // Seconds differ by more than 1 - FAT tolerance rejects this
        assert!(time_differs(1002, 500, 1000, 500, true));
        assert!(time_differs(998, 500, 1000, 500, true));
    }

    #[test]
    fn test_ffi_time_differs() {
        // Exact match
        assert_eq!(rs_time_differs(1000, 500, 1000, 500, 0), 0);
        assert_eq!(rs_time_differs(1000, 500, 1000, 500, 1), 0);

        // Nanosec differ
        assert_eq!(rs_time_differs(1000, 500, 1000, 501, 0), 1);

        // Sec differ by 1, FAT tolerance
        assert_eq!(rs_time_differs(1001, 500, 1000, 500, 1), 0);

        // Sec differ by 2, FAT tolerance
        assert_eq!(rs_time_differs(1002, 500, 1000, 500, 1), 1);
    }

    #[test]
    fn test_is_dev_fd_file() {
        // Valid /dev/fd/N paths (N >= 3 or multiple digits)
        assert!(is_dev_fd_file_impl(b"/dev/fd/3"));
        assert!(is_dev_fd_file_impl(b"/dev/fd/4"));
        assert!(is_dev_fd_file_impl(b"/dev/fd/5"));
        assert!(is_dev_fd_file_impl(b"/dev/fd/9"));
        assert!(is_dev_fd_file_impl(b"/dev/fd/10"));
        assert!(is_dev_fd_file_impl(b"/dev/fd/123"));
        assert!(is_dev_fd_file_impl(b"/dev/fd/63")); // max on most systems

        // Invalid: /dev/fd/0, /dev/fd/1, /dev/fd/2 (stdin/stdout/stderr)
        assert!(!is_dev_fd_file_impl(b"/dev/fd/0"));
        assert!(!is_dev_fd_file_impl(b"/dev/fd/1"));
        assert!(!is_dev_fd_file_impl(b"/dev/fd/2"));

        // Invalid: wrong prefix or no prefix
        assert!(!is_dev_fd_file_impl(b"/dev/fd"));
        assert!(!is_dev_fd_file_impl(b"/dev/fd/"));
        assert!(!is_dev_fd_file_impl(b"/dev/fd/abc"));
        assert!(!is_dev_fd_file_impl(b"dev/fd/5"));
        assert!(!is_dev_fd_file_impl(b"/dev/null"));
        assert!(!is_dev_fd_file_impl(b"/tmp/file"));
        assert!(!is_dev_fd_file_impl(b""));

        // Invalid: trailing characters after digits
        assert!(!is_dev_fd_file_impl(b"/dev/fd/5abc"));
        assert!(!is_dev_fd_file_impl(b"/dev/fd/10.txt"));
    }

    #[test]
    fn test_time_differs_zero_values() {
        // Zero times should match
        assert!(!time_differs(0, 0, 0, 0, false));
        assert!(!time_differs(0, 0, 0, 0, true));
    }

    #[test]
    fn test_time_differs_large_values() {
        // Large timestamps should work correctly
        let large = 1_700_000_000i64;
        assert!(!time_differs(large, 123_456, large, 123_456, false));
        assert!(time_differs(large, 123_456, large, 123_457, false));
    }

    #[test]
    fn test_time_differs_negative_diff() {
        // Negative difference within FAT tolerance
        assert!(!time_differs(999, 0, 1000, 0, true));
        // Negative difference outside FAT tolerance
        assert!(time_differs(997, 0, 1000, 0, true));
    }

    // =========================================================================
    // BOM Detection Tests
    // =========================================================================

    #[test]
    fn test_check_for_bom_utf8() {
        // UTF-8 BOM: EF BB BF
        let data = [0xef, 0xbb, 0xbf, 0x68, 0x65, 0x6c, 0x6c, 0x6f];
        let result = check_for_bom(&data, FIO_ALL);
        assert_eq!(
            result,
            Some(BomResult {
                encoding: "utf-8",
                len: 3
            })
        );

        // Also works with FIO_UTF8 flag
        let result = check_for_bom(&data, FIO_UTF8);
        assert_eq!(
            result,
            Some(BomResult {
                encoding: "utf-8",
                len: 3
            })
        );

        // Also works with flags == 0
        let result = check_for_bom(&data, 0);
        assert_eq!(
            result,
            Some(BomResult {
                encoding: "utf-8",
                len: 3
            })
        );
    }

    #[test]
    fn test_check_for_bom_utf16le() {
        // UTF-16 LE BOM: FF FE
        let data = [0xff, 0xfe, 0x68, 0x00];
        let result = check_for_bom(&data, FIO_ALL);
        assert_eq!(
            result,
            Some(BomResult {
                encoding: "utf-16le",
                len: 2
            })
        );

        let result = check_for_bom(&data, FIO_UTF16 | FIO_ENDIAN_L);
        assert_eq!(
            result,
            Some(BomResult {
                encoding: "utf-16le",
                len: 2
            })
        );
    }

    #[test]
    fn test_check_for_bom_utf16be() {
        // UTF-16 BE BOM: FE FF
        let data = [0xfe, 0xff, 0x00, 0x68];
        let result = check_for_bom(&data, FIO_ALL);
        assert_eq!(
            result,
            Some(BomResult {
                encoding: "utf-16",
                len: 2
            })
        );

        let result = check_for_bom(&data, FIO_UTF16);
        assert_eq!(
            result,
            Some(BomResult {
                encoding: "utf-16",
                len: 2
            })
        );
    }

    #[test]
    fn test_check_for_bom_ucs2() {
        // UCS-2 LE: FF FE (with UCS2 flag)
        let data = [0xff, 0xfe, 0x68, 0x00];
        let result = check_for_bom(&data, FIO_UCS2 | FIO_ENDIAN_L);
        assert_eq!(
            result,
            Some(BomResult {
                encoding: "ucs-2le",
                len: 2
            })
        );

        // UCS-2 BE: FE FF (with UCS2 flag)
        let data = [0xfe, 0xff, 0x00, 0x68];
        let result = check_for_bom(&data, FIO_UCS2);
        assert_eq!(
            result,
            Some(BomResult {
                encoding: "ucs-2",
                len: 2
            })
        );
    }

    #[test]
    fn test_check_for_bom_ucs4le() {
        // UCS-4 LE BOM: FF FE 00 00
        let data = [0xff, 0xfe, 0x00, 0x00, 0x68, 0x00, 0x00, 0x00];
        let result = check_for_bom(&data, FIO_ALL);
        assert_eq!(
            result,
            Some(BomResult {
                encoding: "ucs-4le",
                len: 4
            })
        );

        let result = check_for_bom(&data, FIO_UCS4 | FIO_ENDIAN_L);
        assert_eq!(
            result,
            Some(BomResult {
                encoding: "ucs-4le",
                len: 4
            })
        );
    }

    #[test]
    fn test_check_for_bom_ucs4be() {
        // UCS-4 BE BOM: 00 00 FE FF
        let data = [0x00, 0x00, 0xfe, 0xff, 0x00, 0x00, 0x00, 0x68];
        let result = check_for_bom(&data, FIO_ALL);
        assert_eq!(
            result,
            Some(BomResult {
                encoding: "ucs-4",
                len: 4
            })
        );

        let result = check_for_bom(&data, FIO_UCS4);
        assert_eq!(
            result,
            Some(BomResult {
                encoding: "ucs-4",
                len: 4
            })
        );
    }

    #[test]
    fn test_check_for_bom_no_bom() {
        // Regular ASCII text, no BOM
        let data = b"Hello, world!";
        let result = check_for_bom(data, FIO_ALL);
        assert_eq!(result, None);

        // Empty slice
        let result = check_for_bom(&[], FIO_ALL);
        assert_eq!(result, None);

        // Single byte (too short)
        let result = check_for_bom(&[0xef], FIO_ALL);
        assert_eq!(result, None);
    }

    #[test]
    fn test_check_for_bom_wrong_flags() {
        // UTF-8 BOM with wrong flags should not match
        let data = [0xef, 0xbb, 0xbf, 0x68];
        let result = check_for_bom(&data, FIO_UTF16);
        assert_eq!(result, None);

        // UTF-16 LE BOM with UTF-8 flag should not match
        let data = [0xff, 0xfe, 0x68, 0x00];
        let result = check_for_bom(&data, FIO_UTF8);
        assert_eq!(result, None);
    }

    #[test]
    fn test_check_for_bom_ffi() {
        // Test FFI wrapper with UTF-8 BOM
        let data = [0xef_u8, 0xbb, 0xbf, 0x68];
        let mut len: c_int = 0;
        let result = unsafe { rs_check_for_bom(data.as_ptr(), 4, &mut len, FIO_ALL) };
        assert!(!result.is_null());
        assert_eq!(len, 3);

        // Test with no BOM
        let data = b"Hello";
        let mut len: c_int = 0;
        let result = unsafe { rs_check_for_bom(data.as_ptr(), 5, &mut len, FIO_ALL) };
        assert!(result.is_null());
        assert_eq!(len, 2); // Default length

        // Test with null pointer
        let mut len: c_int = 0;
        let result = unsafe { rs_check_for_bom(std::ptr::null(), 5, &mut len, FIO_ALL) };
        assert!(result.is_null());
        assert_eq!(len, 2);
    }

    // =========================================================================
    // Phase 1: Core Types & Constants Tests
    // =========================================================================

    #[test]
    fn test_file_format_conversions() {
        // C to Rust conversions
        assert_eq!(FileFormat::from_c(-1), FileFormat::Unknown);
        assert_eq!(FileFormat::from_c(0), FileFormat::Unix);
        assert_eq!(FileFormat::from_c(1), FileFormat::Dos);
        assert_eq!(FileFormat::from_c(2), FileFormat::Mac);
        assert_eq!(FileFormat::from_c(999), FileFormat::Unknown);

        // Rust to C conversions
        assert_eq!(FileFormat::Unknown.to_c(), -1);
        assert_eq!(FileFormat::Unix.to_c(), 0);
        assert_eq!(FileFormat::Dos.to_c(), 1);
        assert_eq!(FileFormat::Mac.to_c(), 2);
    }

    #[test]
    fn test_file_format_line_endings() {
        assert_eq!(FileFormat::Unix.line_ending(), b"\n");
        assert_eq!(FileFormat::Dos.line_ending(), b"\r\n");
        assert_eq!(FileFormat::Mac.line_ending(), b"\r");
        assert_eq!(FileFormat::Unknown.line_ending(), b"\n"); // defaults to Unix
    }

    #[test]
    fn test_read_flags() {
        // Test individual flags
        let flags = ReadFlags::NEW | ReadFlags::FILTER;
        assert!(flags.contains(ReadFlags::NEW));
        assert!(flags.contains(ReadFlags::FILTER));
        assert!(!flags.contains(ReadFlags::STDIN));

        // Test C conversion round-trip
        let flags = ReadFlags::from_c(0x45); // NEW | STDIN | FIFO
        assert!(flags.contains(ReadFlags::NEW));
        assert!(flags.contains(ReadFlags::STDIN));
        assert!(flags.contains(ReadFlags::FIFO));
        assert_eq!(flags.to_c(), 0x45);

        // Test all flags
        assert_eq!(ReadFlags::NEW.to_c(), 0x01);
        assert_eq!(ReadFlags::FILTER.to_c(), 0x02);
        assert_eq!(ReadFlags::STDIN.to_c(), 0x04);
        assert_eq!(ReadFlags::BUFFER.to_c(), 0x08);
        assert_eq!(ReadFlags::DUMMY.to_c(), 0x10);
        assert_eq!(ReadFlags::KEEP_UNDO.to_c(), 0x20);
        assert_eq!(ReadFlags::FIFO.to_c(), 0x40);
        assert_eq!(ReadFlags::NOWINENTER.to_c(), 0x80);
        assert_eq!(ReadFlags::NOFILE.to_c(), 0x100);
    }

    #[test]
    fn test_write_flags() {
        let flags = WriteFlags::WHOLE | WriteFlags::FORCE;
        assert!(flags.contains(WriteFlags::WHOLE));
        assert!(flags.contains(WriteFlags::FORCE));
        assert!(!flags.contains(WriteFlags::APPEND));

        // Test values match expected C values
        assert_eq!(WriteFlags::WHOLE.to_c(), 0x01);
        assert_eq!(WriteFlags::APPEND.to_c(), 0x02);
        assert_eq!(WriteFlags::PART.to_c(), 0x04);
        assert_eq!(WriteFlags::FORCE.to_c(), 0x08);
    }

    #[test]
    fn test_conv_type_conversions() {
        // C to Rust
        assert_eq!(ConvType::from_c(0), ConvType::None);
        assert_eq!(ConvType::from_c(1), ConvType::ToUtf8);
        assert_eq!(ConvType::from_c(2), ConvType::Latin9ToUtf8);
        assert_eq!(ConvType::from_c(3), ConvType::ToLatin1);
        assert_eq!(ConvType::from_c(4), ConvType::ToLatin9);
        assert_eq!(ConvType::from_c(5), ConvType::Iconv);
        assert_eq!(ConvType::from_c(99), ConvType::None);

        // Rust to C
        assert_eq!(ConvType::None.to_c(), 0);
        assert_eq!(ConvType::ToUtf8.to_c(), 1);
        assert_eq!(ConvType::Latin9ToUtf8.to_c(), 2);
        assert_eq!(ConvType::ToLatin1.to_c(), 3);
        assert_eq!(ConvType::ToLatin9.to_c(), 4);
        assert_eq!(ConvType::Iconv.to_c(), 5);
    }

    #[test]
    fn test_bad_char_behavior() {
        // C to Rust
        assert_eq!(
            BadCharBehavior::from_c(b'?' as c_int),
            BadCharBehavior::Replace
        );
        assert_eq!(BadCharBehavior::from_c(-1), BadCharBehavior::Keep);
        assert_eq!(BadCharBehavior::from_c(-2), BadCharBehavior::Drop);
        assert_eq!(
            BadCharBehavior::from_c(b'X' as c_int),
            BadCharBehavior::ReplaceWith(b'X')
        );

        // Rust to C
        assert_eq!(BadCharBehavior::Replace.to_c(), b'?' as c_int);
        assert_eq!(BadCharBehavior::Keep.to_c(), -1);
        assert_eq!(BadCharBehavior::Drop.to_c(), -2);
        assert_eq!(BadCharBehavior::ReplaceWith(b'X').to_c(), b'X' as c_int);
    }

    #[test]
    fn test_fio_flags() {
        // Test flag values match legacy constants
        assert_eq!(FioFlags::LATIN1.bits() as c_int, FIO_LATIN1);
        assert_eq!(FioFlags::UTF8.bits() as c_int, FIO_UTF8);
        assert_eq!(FioFlags::UCS2.bits() as c_int, FIO_UCS2);
        assert_eq!(FioFlags::UCS4.bits() as c_int, FIO_UCS4);
        assert_eq!(FioFlags::UTF16.bits() as c_int, FIO_UTF16);
        assert_eq!(FioFlags::ENDIAN_L.bits() as c_int, FIO_ENDIAN_L);
        assert_eq!(FioFlags::NOCONVERT.bits() as c_int, FIO_NOCONVERT);
        assert_eq!(FioFlags::UCSBOM.bits() as c_int, FIO_UCSBOM);

        // Test combinations
        assert_eq!(FioFlags::UCS2_LE.bits() as c_int, FIO_UCS2 | FIO_ENDIAN_L);
        assert_eq!(FioFlags::UCS4_LE.bits() as c_int, FIO_UCS4 | FIO_ENDIAN_L);
        assert_eq!(FioFlags::UTF16_LE.bits() as c_int, FIO_UTF16 | FIO_ENDIAN_L);

        // Test from_c
        assert_eq!(FioFlags::from_c(-1), None); // FIO_ALL
        assert_eq!(FioFlags::from_c(FIO_UTF8), Some(FioFlags::UTF8));

        // Test encoding names
        assert_eq!(FioFlags::LATIN1.encoding_name(), Some("latin1"));
        assert_eq!(FioFlags::UTF8.encoding_name(), Some("utf-8"));
        assert_eq!(FioFlags::UCS2.encoding_name(), Some("ucs-2"));
        assert_eq!(FioFlags::UCS2_LE.encoding_name(), Some("ucs-2le"));
        assert_eq!(FioFlags::UTF16.encoding_name(), Some("utf-16"));
        assert_eq!(FioFlags::UTF16_LE.encoding_name(), Some("utf-16le"));
    }

    #[test]
    fn test_file_format_default() {
        assert_eq!(FileFormat::default(), FileFormat::Unknown);
    }

    #[test]
    fn test_flags_default() {
        assert_eq!(ReadFlags::default(), ReadFlags::empty());
        assert_eq!(WriteFlags::default(), WriteFlags::empty());
        assert_eq!(FioFlags::default(), FioFlags::empty());
    }
}
