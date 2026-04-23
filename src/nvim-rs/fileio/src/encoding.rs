//! File encoding detection and conversion utilities.
//!
//! This module handles:
//! - Encoding name normalization
//! - FIO flag determination from encoding names
//! - Encoding detection heuristics
//! - BOM (Byte Order Mark) handling extensions

use std::ffi::{c_char, c_int, c_void, CStr};

use crate::{FioFlags, FIO_ENDIAN_L, FIO_LATIN1, FIO_UCS2, FIO_UCS4, FIO_UTF16, FIO_UTF8};

// =============================================================================
// Encoding Property Constants (from mbyte_defs.h)
// =============================================================================

/// 8-bit encoding (single-byte)
pub const ENC_8BIT: c_int = 0x01;
/// Double-byte character set
pub const ENC_DBCS: c_int = 0x02;
/// Unicode encoding
pub const ENC_UNICODE: c_int = 0x04;
/// Unicode: Big endian
pub const ENC_ENDIAN_B: c_int = 0x10;
/// Unicode: Little endian
pub const ENC_ENDIAN_L: c_int = 0x20;
/// Unicode: UCS-2
pub const ENC_2BYTE: c_int = 0x40;
/// Unicode: UCS-4
pub const ENC_4BYTE: c_int = 0x80;
/// Unicode: UTF-16
pub const ENC_2WORD: c_int = 0x100;
/// Latin1 encoding
pub const ENC_LATIN1: c_int = 0x200;
/// Latin9 encoding
pub const ENC_LATIN9: c_int = 0x400;
/// Mac Roman encoding
pub const ENC_MACROMAN: c_int = 0x800;

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    /// Get encoding properties from the mbyte crate.
    fn enc_canon_props(name: *const c_char) -> c_int;
    /// Get the current 'encoding' option value.
    static mut p_enc: *mut c_char;
    /// Canonicalize an encoding name. Returns xmalloc'd string.
    fn enc_canonize(enc: *const c_char) -> *mut c_char;
    /// Duplicate n bytes from s as NUL-terminated string.
    fn xmemdupz(src: *const std::ffi::c_void, len: usize) -> *mut c_char;
    /// Free memory.
    fn xfree(ptr: *mut std::ffi::c_void);
    /// Search for character in string.
    fn vim_strchr(s: *const c_char, c: c_int) -> *mut c_char;
    /// Create a temporary filename.
    fn vim_tempname() -> *mut c_char;
    /// Evaluate the charconvert expression.
    fn eval_charconvert(
        enc_from: *const c_char,
        enc_to: *const c_char,
        fname_from: *const c_char,
        fname_to: *const c_char,
    ) -> c_int;
    /// Open a file.
    fn os_open(path: *const c_char, flags: c_int, mode: u32) -> c_int;
    /// Remove a file.
    fn os_remove(path: *const c_char) -> c_int;
    /// Display a message.
    fn msg(m: *const c_char, hl_id: c_int) -> bool;
}

// =============================================================================
// FIO Flags from Encoding
// =============================================================================

/// Determine the FIO_* flags needed for internal conversion based on encoding name.
///
/// Returns the appropriate FIO_* flags for Unicode or Latin-1 encodings,
/// or 0 if the encoding requires iconv() for conversion.
///
/// # Arguments
/// * `name` - Encoding name (e.g., "utf-8", "ucs-2le", "latin1").
///   If empty, uses the current 'encoding' option value.
///
/// # Returns
/// * `FIO_UTF8` for UTF-8
/// * `FIO_UCS2` (optionally with `FIO_ENDIAN_L`) for UCS-2
/// * `FIO_UCS4` (optionally with `FIO_ENDIAN_L`) for UCS-4
/// * `FIO_UTF16` (optionally with `FIO_ENDIAN_L`) for UTF-16
/// * `FIO_LATIN1` for Latin-1
/// * `0` for encodings that require iconv()
#[inline]
pub fn get_fio_flags_from_props(prop: c_int) -> c_int {
    if prop & ENC_UNICODE != 0 {
        // Unicode encoding
        if prop & ENC_2BYTE != 0 {
            // UCS-2
            if prop & ENC_ENDIAN_L != 0 {
                return FIO_UCS2 | FIO_ENDIAN_L;
            }
            return FIO_UCS2;
        }
        if prop & ENC_4BYTE != 0 {
            // UCS-4
            if prop & ENC_ENDIAN_L != 0 {
                return FIO_UCS4 | FIO_ENDIAN_L;
            }
            return FIO_UCS4;
        }
        if prop & ENC_2WORD != 0 {
            // UTF-16
            if prop & ENC_ENDIAN_L != 0 {
                return FIO_UTF16 | FIO_ENDIAN_L;
            }
            return FIO_UTF16;
        }
        // Default Unicode is UTF-8
        return FIO_UTF8;
    }
    if prop & ENC_LATIN1 != 0 {
        return FIO_LATIN1;
    }
    // Must be ENC_DBCS or other, requires iconv()
    0
}

/// Determine the FIO_* flags from an encoding name.
///
/// # Safety
/// `name` must be a valid C string pointer or null.
///
/// # Arguments
/// * `name` - Encoding name as a C string. If null or empty, uses 'encoding' (p_enc).
///
/// # Returns
/// FIO_* flags for the encoding, or 0 if iconv() is required.
#[no_mangle]
pub unsafe extern "C" fn rs_get_fio_flags(name: *const c_char) -> c_int {
    if name.is_null() {
        return 0;
    }

    // If name is empty, fall back to the current 'encoding' option (p_enc),
    // matching the behavior of the C get_fio_flags() function.
    let effective_name = if unsafe { *name == 0 } {
        let enc = unsafe { p_enc.cast_const() };
        if enc.is_null() {
            return 0;
        }
        enc
    } else {
        name
    };

    let prop = unsafe { enc_canon_props(effective_name) };
    get_fio_flags_from_props(prop)
}

/// Wrapper that also checks for empty string.
///
/// # Safety
/// `name` must be a valid C string pointer or null.
#[no_mangle]
pub unsafe extern "C" fn rs_get_fio_flags_checked(name: *const c_char) -> c_int {
    if name.is_null() {
        return 0;
    }

    let c_str = CStr::from_ptr(name);
    if c_str.to_bytes().is_empty() {
        // Empty string means use 'encoding' - caller must handle this
        return 0;
    }

    rs_get_fio_flags(name)
}

// =============================================================================
// Encoding Conversion Check
// =============================================================================

/// Check if file encoding requires conversion.
///
/// Returns true if the file encoding differs from 'encoding' and conversion
/// is needed, false if they match or if the file is UTF-8 and 'encoding' is
/// any Unicode encoding.
///
/// # Arguments
/// * `fenc_flags` - FIO_* flags for the file encoding
/// * `enc_flags` - FIO_* flags for the 'encoding' option
/// * `same_name` - true if the encoding names are identical
///
/// # Returns
/// true if conversion is required, false otherwise
#[inline]
pub fn need_conversion_flags(fenc_flags: c_int, enc_flags: c_int, same_name: bool) -> bool {
    if same_name {
        // Same encoding name, no conversion needed
        return false;
    }

    // Check if they resolve to the same FIO flags
    if enc_flags != 0 && fenc_flags == enc_flags {
        return false;
    }

    // Encodings differ. However, conversion is not needed when 'enc' is any
    // Unicode encoding and the file is UTF-8.
    fenc_flags != FIO_UTF8
}

/// FFI wrapper for need_conversion_flags.
///
/// # Safety
/// All parameters are plain integers, so this is safe.
#[no_mangle]
pub extern "C" fn rs_need_conversion_flags(
    fenc_flags: c_int,
    enc_flags: c_int,
    same_name: c_int,
) -> c_int {
    c_int::from(need_conversion_flags(fenc_flags, enc_flags, same_name != 0))
}

/// Check if file encoding `fenc` requires conversion from or to 'encoding'.
///
/// Replaces the C `need_conversion()` function. Reads `p_enc` via the C
/// accessor and computes flags using `rs_get_fio_flags`.
///
/// # Safety
/// `fenc` must be a valid C string pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_need_conversion(fenc: *const c_char) -> c_int {
    if fenc.is_null() {
        return 0;
    }

    let fenc_bytes = unsafe { std::ffi::CStr::from_ptr(fenc).to_bytes() };

    // Empty fenc: original C returns false (no conversion needed)
    if fenc_bytes.is_empty() {
        return 0;
    }

    // Get the current 'encoding' option
    let p_enc_ptr = unsafe { p_enc.cast_const() };

    let same_encoding;
    let fenc_flags;

    if !p_enc_ptr.is_null() {
        let p_enc_bytes = unsafe { std::ffi::CStr::from_ptr(p_enc_ptr).to_bytes() };
        if p_enc_bytes == fenc_bytes {
            // Same name => same_encoding = true, fenc_flags = 0
            return 0;
        }
        // Ignore difference between aliases by comparing FIO flags
        let enc_flags = unsafe { rs_get_fio_flags(p_enc_ptr) };
        fenc_flags = unsafe { rs_get_fio_flags(fenc) };
        same_encoding = enc_flags != 0 && fenc_flags == enc_flags;
    } else {
        fenc_flags = unsafe { rs_get_fio_flags(fenc) };
        same_encoding = false;
    }

    if same_encoding {
        return 0;
    }

    // Encodings differ. Conversion is not needed when 'enc' is any Unicode
    // encoding and the file is UTF-8.
    // need_conversion_flags(fenc_flags, enc_flags, same_name=false) =>
    //   returns fenc_flags != FIO_UTF8
    c_int::from(fenc_flags != crate::FIO_UTF8)
}

// =============================================================================
// Encoding Name Utilities
// =============================================================================

/// Common encoding name aliases and their canonical forms.
pub const ENCODING_ALIASES: &[(&str, &str)] = &[
    ("ansi", "latin1"),
    ("iso-8859-1", "latin1"),
    ("iso_8859-1", "latin1"),
    ("latin-1", "latin1"),
    ("iso-8859-15", "latin9"),
    ("iso_8859-15", "latin9"),
    ("latin-9", "latin9"),
    ("utf8", "utf-8"),
    ("utf16", "utf-16"),
    ("utf16le", "utf-16le"),
    ("utf16be", "utf-16"),
    ("ucs2", "ucs-2"),
    ("ucs2le", "ucs-2le"),
    ("ucs2be", "ucs-2"),
    ("ucs4", "ucs-4"),
    ("ucs4le", "ucs-4le"),
    ("ucs4be", "ucs-4"),
];

/// Get FioFlags from an encoding name string (Rust-native version).
///
/// This is a pure Rust implementation that doesn't rely on the C encoding table.
/// It handles common encoding names and returns the appropriate flags.
pub fn fio_flags_from_name(name: &str) -> Option<FioFlags> {
    let name_lower = name.to_ascii_lowercase();
    let name_ref = name_lower.as_str();

    match name_ref {
        "utf-8" | "utf8" => Some(FioFlags::UTF8),
        "latin1" | "latin-1" | "iso-8859-1" | "iso_8859-1" | "ansi" => Some(FioFlags::LATIN1),
        "ucs-2" | "ucs2" | "ucs-2be" | "ucs2be" => Some(FioFlags::UCS2),
        "ucs-2le" | "ucs2le" => Some(FioFlags::UCS2_LE),
        "ucs-4" | "ucs4" | "ucs-4be" | "ucs4be" => Some(FioFlags::UCS4),
        "ucs-4le" | "ucs4le" => Some(FioFlags::UCS4_LE),
        "utf-16" | "utf16" | "utf-16be" | "utf16be" => Some(FioFlags::UTF16),
        "utf-16le" | "utf16le" => Some(FioFlags::UTF16_LE),
        _ => None,
    }
}

/// Detect if a byte sequence appears to be UTF-8.
///
/// Performs a heuristic check for valid UTF-8 sequences.
/// This is useful when encoding is unknown and we need to guess.
///
/// # Arguments
/// * `data` - Byte slice to check
///
/// # Returns
/// true if the data appears to be valid UTF-8, false otherwise
pub fn looks_like_utf8(data: &[u8]) -> bool {
    std::str::from_utf8(data).is_ok()
}

/// Detect if a byte sequence appears to be UTF-16.
///
/// Checks for patterns typical of UTF-16 encoding:
/// - Every other byte is often 0x00 for ASCII text
/// - Presence of surrogate pairs
///
/// # Arguments
/// * `data` - Byte slice to check
/// * `little_endian` - true to check for UTF-16LE, false for UTF-16BE
///
/// # Returns
/// A score from 0.0 to 1.0 indicating likelihood of UTF-16
pub fn utf16_likelihood(data: &[u8], little_endian: bool) -> f32 {
    if data.len() < 2 {
        return 0.0;
    }

    let mut null_count = 0;
    let mut total_pairs = 0;

    // Check if alternating bytes are null (common for ASCII in UTF-16)
    let check_index = if little_endian { 1 } else { 0 };

    for chunk in data.chunks(2) {
        if chunk.len() == 2 {
            total_pairs += 1;
            if chunk[check_index] == 0 {
                null_count += 1;
            }
        }
    }

    if total_pairs == 0 {
        return 0.0;
    }

    null_count as f32 / total_pairs as f32
}

/// Detect if a byte sequence contains only Latin-1 characters.
///
/// Latin-1 is a superset of ASCII that uses all 256 byte values.
/// This checks if the data could be valid Latin-1 (which it always is,
/// since any byte sequence is valid Latin-1), but we can check for
/// common control characters that suggest it's NOT plain text.
///
/// # Arguments
/// * `data` - Byte slice to check
///
/// # Returns
/// true if the data appears to be valid Latin-1 text
pub fn looks_like_latin1(data: &[u8]) -> bool {
    // Check for problematic control characters (0x00-0x08, 0x0E-0x1F except common ones)
    for &byte in data {
        match byte {
            // Allow common control characters
            0x09 | 0x0A | 0x0D => continue, // tab, LF, CR
            // Reject other control characters in 0x00-0x1F range
            0x00..=0x08 | 0x0B..=0x0C | 0x0E..=0x1F => return false,
            // Allow everything else (0x20-0xFF)
            _ => continue,
        }
    }
    true
}

// =============================================================================
// next_fenc and readfile_charconvert
// =============================================================================

/// Get the next fileencoding from the fileencodings option list.
///
/// Directly replaces the static C `next_fenc` symbol.
///
/// # Safety
/// `pp` and `alloced` must be valid non-null pointers.
#[export_name = "next_fenc"]
pub unsafe extern "C" fn rs_next_fenc(pp: *mut *mut c_char, alloced: *mut bool) -> *mut c_char {
    unsafe { *alloced = false };
    let p_ref = unsafe { *pp };
    if p_ref.is_null() || unsafe { *p_ref } == 0 {
        unsafe { *pp = std::ptr::null_mut() };
        return c"".as_ptr() as *mut c_char;
    }
    let comma = unsafe { vim_strchr(p_ref as *const c_char, b',' as c_int) };
    let r: *mut c_char;
    if comma.is_null() {
        r = unsafe { enc_canonize(p_ref as *const c_char) };
        let len = unsafe { libc::strlen(p_ref as *const libc::c_char) };
        unsafe { *pp = p_ref.add(len) };
    } else {
        let seg_len = unsafe { comma.offset_from(p_ref) } as usize;
        let dup = unsafe { xmemdupz(p_ref as *const std::ffi::c_void, seg_len) };
        let canonical = unsafe { enc_canonize(dup as *const c_char) };
        unsafe { xfree(dup as *mut std::ffi::c_void) };
        r = canonical;
        unsafe { *pp = comma.add(1) };
    }
    unsafe { *alloced = true };
    r
}

/// FAIL constant (matches C define)
const FAIL: c_int = 0;

/// Convert a file with the charconvert expression.
///
/// Directly replaces the static C `readfile_charconvert` symbol.
///
/// # Safety
/// `fname`, `fenc`, `fdp` must be valid non-null pointers.
#[export_name = "readfile_charconvert"]
pub unsafe extern "C" fn rs_readfile_charconvert(
    fname: *mut c_char,
    fenc: *mut c_char,
    fdp: *mut c_int,
) -> *mut c_char {
    let mut errmsg: *const c_char = std::ptr::null();
    let tmpname = unsafe { vim_tempname() };
    if tmpname.is_null() {
        errmsg = c"Can't find temp file for conversion".as_ptr();
    } else {
        unsafe { libc::close(*fdp) };
        unsafe { *fdp = -1 };
        if unsafe { eval_charconvert(fenc, c"utf-8".as_ptr(), fname, tmpname) } == FAIL {
            errmsg = c"Conversion with 'charconvert' failed".as_ptr();
        }
        if errmsg.is_null() {
            let new_fd = unsafe { os_open(tmpname, libc::O_RDONLY, 0) };
            if new_fd < 0 {
                errmsg = c"can't read output of 'charconvert'".as_ptr();
            } else {
                unsafe { *fdp = new_fd };
            }
        }
    }
    if !errmsg.is_null() {
        unsafe { msg(errmsg, 0) };
        if !tmpname.is_null() {
            unsafe { os_remove(tmpname) };
            unsafe { xfree(tmpname as *mut std::ffi::c_void) };
            return std::ptr::null_mut();
        }
    }
    if unsafe { *fdp } < 0 {
        unsafe { *fdp = os_open(fname, libc::O_RDONLY, 0) };
    }
    tmpname
}

// =============================================================================
// set_forced_fenc: set the forced fileencoding from exarg
// =============================================================================

extern "C" {
    /// Set the local fileencoding option from a string. Wraps set_option_direct.
    fn nvim_set_fileencoding_local(fenc: *const c_char);
}

/// Set forced 'fileencoding' from exarg.
///
/// Replaces the C `set_forced_fenc` function.
///
/// # Safety
/// - `eap` must be a valid non-null pointer to an exarg_T.
#[export_name = "set_forced_fenc"]
pub unsafe extern "C" fn rs_set_forced_fenc(eap: *const c_void) {
    let ea = &*(eap as *const nvim_ex_cmds_types::ExArg);
    let force_enc = ea.force_enc;
    if force_enc == 0 {
        return;
    }
    let cmd = ea.cmd;
    if cmd.is_null() {
        return;
    }
    let enc_start = cmd.add(force_enc as usize);
    let fenc = enc_canonize(enc_start);
    if !fenc.is_null() {
        nvim_set_fileencoding_local(fenc);
        xfree(fenc as *mut std::ffi::c_void);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_fio_flags_from_props() {
        // UTF-8 (Unicode without specific byte width)
        assert_eq!(get_fio_flags_from_props(ENC_UNICODE), FIO_UTF8);

        // UCS-2 Big Endian
        assert_eq!(get_fio_flags_from_props(ENC_UNICODE | ENC_2BYTE), FIO_UCS2);

        // UCS-2 Little Endian
        assert_eq!(
            get_fio_flags_from_props(ENC_UNICODE | ENC_2BYTE | ENC_ENDIAN_L),
            FIO_UCS2 | FIO_ENDIAN_L
        );

        // UCS-4 Big Endian
        assert_eq!(get_fio_flags_from_props(ENC_UNICODE | ENC_4BYTE), FIO_UCS4);

        // UCS-4 Little Endian
        assert_eq!(
            get_fio_flags_from_props(ENC_UNICODE | ENC_4BYTE | ENC_ENDIAN_L),
            FIO_UCS4 | FIO_ENDIAN_L
        );

        // UTF-16 Big Endian
        assert_eq!(get_fio_flags_from_props(ENC_UNICODE | ENC_2WORD), FIO_UTF16);

        // UTF-16 Little Endian
        assert_eq!(
            get_fio_flags_from_props(ENC_UNICODE | ENC_2WORD | ENC_ENDIAN_L),
            FIO_UTF16 | FIO_ENDIAN_L
        );

        // Latin-1
        assert_eq!(get_fio_flags_from_props(ENC_LATIN1), FIO_LATIN1);

        // DBCS (requires iconv)
        assert_eq!(get_fio_flags_from_props(ENC_DBCS), 0);

        // 8-bit (requires iconv)
        assert_eq!(get_fio_flags_from_props(ENC_8BIT), 0);
    }

    #[test]
    fn test_need_conversion_flags() {
        // Same encoding name
        assert!(!need_conversion_flags(FIO_UTF8, FIO_UTF8, true));

        // Same flags, different names (e.g., "ansi" vs "latin1")
        assert!(!need_conversion_flags(FIO_LATIN1, FIO_LATIN1, false));

        // Different encodings
        assert!(need_conversion_flags(FIO_LATIN1, FIO_UTF8, false));

        // File is UTF-8, encoding is Unicode - no conversion needed
        assert!(!need_conversion_flags(FIO_UTF8, FIO_UCS2, false));
        assert!(!need_conversion_flags(FIO_UTF8, FIO_UCS4, false));
        assert!(!need_conversion_flags(FIO_UTF8, FIO_UTF16, false));
    }

    #[test]
    fn test_fio_flags_from_name() {
        assert_eq!(fio_flags_from_name("utf-8"), Some(FioFlags::UTF8));
        assert_eq!(fio_flags_from_name("UTF-8"), Some(FioFlags::UTF8));
        assert_eq!(fio_flags_from_name("utf8"), Some(FioFlags::UTF8));

        assert_eq!(fio_flags_from_name("latin1"), Some(FioFlags::LATIN1));
        assert_eq!(fio_flags_from_name("iso-8859-1"), Some(FioFlags::LATIN1));
        assert_eq!(fio_flags_from_name("ansi"), Some(FioFlags::LATIN1));

        assert_eq!(fio_flags_from_name("ucs-2"), Some(FioFlags::UCS2));
        assert_eq!(fio_flags_from_name("ucs-2le"), Some(FioFlags::UCS2_LE));

        assert_eq!(fio_flags_from_name("utf-16"), Some(FioFlags::UTF16));
        assert_eq!(fio_flags_from_name("utf-16le"), Some(FioFlags::UTF16_LE));

        assert_eq!(fio_flags_from_name("unknown"), None);
    }

    #[test]
    fn test_looks_like_utf8() {
        // Valid UTF-8
        assert!(looks_like_utf8(b"Hello, world!"));
        assert!(looks_like_utf8("Héllo, wörld!".as_bytes()));
        assert!(looks_like_utf8("日本語".as_bytes()));

        // Invalid UTF-8 (truncated sequence)
        assert!(!looks_like_utf8(&[0xC3])); // incomplete 2-byte sequence
        assert!(!looks_like_utf8(&[0xE2, 0x80])); // incomplete 3-byte sequence

        // Invalid UTF-8 (overlong encoding)
        assert!(!looks_like_utf8(&[0xC0, 0x80])); // overlong NUL
    }

    #[test]
    fn test_utf16_likelihood() {
        // ASCII text as UTF-16LE: "Hi" = 'H' 0x00 'i' 0x00
        let utf16le_ascii = [0x48, 0x00, 0x69, 0x00];
        assert!(utf16_likelihood(&utf16le_ascii, true) > 0.5);

        // ASCII text as UTF-16BE: "Hi" = 0x00 'H' 0x00 'i'
        let utf16be_ascii = [0x00, 0x48, 0x00, 0x69];
        assert!(utf16_likelihood(&utf16be_ascii, false) > 0.5);

        // Regular UTF-8 text should have low UTF-16 likelihood
        let utf8_text = b"Hello, world!";
        assert!(utf16_likelihood(utf8_text, true) < 0.5);
        assert!(utf16_likelihood(utf8_text, false) < 0.5);
    }

    #[test]
    fn test_looks_like_latin1() {
        // Valid Latin-1 text
        assert!(looks_like_latin1(b"Hello, world!"));
        assert!(looks_like_latin1(&[0xC9, 0x63, 0x6F, 0x6C, 0x65])); // "École" in Latin-1

        // Contains NUL
        assert!(!looks_like_latin1(&[0x00]));

        // Contains problematic control characters
        assert!(!looks_like_latin1(&[0x01])); // SOH
        assert!(!looks_like_latin1(&[0x1B])); // ESC

        // Tab, LF, CR are allowed
        assert!(looks_like_latin1(&[0x09, 0x0A, 0x0D]));
    }
}
