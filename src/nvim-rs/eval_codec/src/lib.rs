//! Encode/decode support for VimL values.
//!
//! This module implements encoding and decoding from `src/nvim/eval/decode.c`
//! and `src/nvim/eval/encode.c`:
//! - JSON encoding/decoding
//! - MessagePack encoding/decoding for RPC
//! - String escaping and unescaping
//! - Binary blob handling
//!
//! ## Module Structure
//!
//! - `json`: JSON encoding and decoding
//! - `escape`: String escape/unescape helpers
//! - `blob`: Binary blob encoding

#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::doc_markdown)]

pub mod blob;
pub mod decode;
pub mod encode;
pub mod escape;
pub mod json;

pub use blob::*;
pub use escape::*;
pub use json::*;

use std::ffi::{c_char, c_int, c_uchar, c_void};

// =============================================================================
// Encoding Options
// =============================================================================

/// Options for JSON encoding.
#[derive(Debug, Clone, Copy, Default)]
pub struct JsonEncodeOptions {
    /// Pretty-print with indentation
    pub pretty: bool,
    /// Indent width (when pretty is true)
    pub indent: i32,
    /// Allow NaN and Infinity (non-standard JSON)
    pub allow_nan: bool,
    /// Allow special values (v:null, v:true, v:false)
    pub allow_special: bool,
}

impl JsonEncodeOptions {
    /// Create default options (compact JSON).
    pub const fn compact() -> Self {
        Self {
            pretty: false,
            indent: 0,
            allow_nan: false,
            allow_special: true,
        }
    }

    /// Create pretty-print options.
    pub const fn pretty(indent: i32) -> Self {
        Self {
            pretty: true,
            indent,
            allow_nan: false,
            allow_special: true,
        }
    }

    /// Create from flag bits.
    pub const fn from_bits(bits: u32) -> Self {
        Self {
            pretty: bits & 0x01 != 0,
            indent: ((bits >> 8) & 0xFF) as i32,
            allow_nan: bits & 0x02 != 0,
            allow_special: bits & 0x04 != 0,
        }
    }

    /// Convert to flag bits.
    pub const fn to_bits(&self) -> u32 {
        let mut bits = 0u32;
        if self.pretty {
            bits |= 0x01;
        }
        if self.allow_nan {
            bits |= 0x02;
        }
        if self.allow_special {
            bits |= 0x04;
        }
        bits |= (self.indent as u32 & 0xFF) << 8;
        bits
    }
}

// rs_json_opts_compact and rs_json_opts_pretty were dead FFI exports (no C
// callers). Removed.

// =============================================================================
// Decode Result Types
// =============================================================================

/// Result of a decode operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum DecodeResult {
    /// Successfully decoded
    Ok = 0,
    /// Syntax error in input
    SyntaxError = 1,
    /// Unexpected end of input
    UnexpectedEof = 2,
    /// Invalid escape sequence
    InvalidEscape = 3,
    /// Number out of range
    NumberOutOfRange = 4,
    /// Nesting too deep
    NestingTooDeep = 5,
    /// Invalid UTF-8
    InvalidUtf8 = 6,
    /// Unknown error
    Unknown = -1,
}

impl DecodeResult {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Ok,
            1 => Self::SyntaxError,
            2 => Self::UnexpectedEof,
            3 => Self::InvalidEscape,
            4 => Self::NumberOutOfRange,
            5 => Self::NestingTooDeep,
            6 => Self::InvalidUtf8,
            _ => Self::Unknown,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if result is success.
    pub const fn is_ok(&self) -> bool {
        matches!(self, Self::Ok)
    }
}

// =============================================================================
// Encode Result Types
// =============================================================================

/// Result of an encode operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum EncodeResult {
    /// Successfully encoded
    Ok = 0,
    /// Value contains circular reference
    CircularRef = 1,
    /// Value contains non-encodable type
    InvalidType = 2,
    /// Output buffer too small
    BufferTooSmall = 3,
    /// String contains invalid UTF-8
    InvalidUtf8 = 4,
    /// Unknown error
    Unknown = -1,
}

impl EncodeResult {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Ok,
            1 => Self::CircularRef,
            2 => Self::InvalidType,
            3 => Self::BufferTooSmall,
            4 => Self::InvalidUtf8,
            _ => Self::Unknown,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if result is success.
    pub const fn is_ok(&self) -> bool {
        matches!(self, Self::Ok)
    }
}

// =============================================================================
// C type layout definitions
// =============================================================================

/// Growing array structure matching C `garray_T`.
#[repr(C)]
#[allow(clippy::struct_field_names)]
pub struct GarrayT {
    pub ga_len: c_int,
    pub ga_maxlen: c_int,
    pub ga_itemsize: c_int,
    pub ga_growsize: c_int,
    pub ga_data: *mut c_void,
}

// =============================================================================
// C function declarations
// =============================================================================

extern "C" {
    fn ga_concat_len(gap: *mut GarrayT, data: *const c_char, len: usize);
    fn ga_concat(gap: *mut GarrayT, data: *const c_char);
    fn ga_append(gap: *mut GarrayT, c: c_uchar);
    fn ga_grow(gap: *mut GarrayT, n: c_int);
    fn utf_ptr2char(p: *const c_char) -> c_int;
    fn utf_ptr2len(p: *const c_char) -> c_int;
    fn utf_char2len(c: c_int) -> c_int;
    fn utf_printable(c: c_int) -> bool;
    fn semsg(msg: *const c_char, ...);
}

// =============================================================================
// encode_blob_write
// =============================================================================

/// Msgpack callback for writing to a Blob.
///
/// The `data` argument is a `*mut blob_T` cast to `*mut c_void`. Since `bv_ga`
/// is the first field of `blobvar_S`, the pointer can be used directly as a
/// `*mut GarrayT`.
///
/// # Safety
/// - `data` must be a valid non-null pointer to a `blob_T`.
/// - `buf` must be a valid pointer to `len` bytes (or null if len == 0).
#[unsafe(export_name = "encode_blob_write")]
pub unsafe extern "C" fn rs_encode_blob_write(
    data: *mut c_void,
    buf: *const c_char,
    len: usize,
) -> c_int {
    // bv_ga is the first field of blob_T, so data == &blob.bv_ga
    let gap = data.cast::<GarrayT>();
    ga_concat_len(gap, buf, len);
    len as c_int
}

// =============================================================================
// convert_to_json_string (rs_convert_to_json_string)
// =============================================================================

// Surrogate pair constants matching encode.h
const SURROGATE_HI_START: c_int = 0xD800;
const SURROGATE_HI_END: c_int = 0xDBFF;
const SURROGATE_LO_START: c_int = 0xDC00;
const SURROGATE_LO_END: c_int = 0xDFFF;
const SURROGATE_FIRST_CHAR: c_int = 0x10000;

/// JSON escape sequences for control characters.
/// Index is the byte value; entry is a 2-byte escape sequence.
/// Matches the C `escapes` table in encode.c.
static ESCAPES: [[u8; 2]; 128] = {
    let mut table = [[0u8; 2]; 128];
    // BS = 0x08
    table[0x08] = [b'\\', b'b'];
    // TAB = 0x09
    table[0x09] = [b'\\', b't'];
    // NL = 0x0A
    table[0x0A] = [b'\\', b'n'];
    // FF = 0x0C
    table[0x0C] = [b'\\', b'f'];
    // CAR = 0x0D
    table[0x0D] = [b'\\', b'r'];
    // '"' = 0x22
    table[0x22] = [b'\\', b'"'];
    // '\\' = 0x5C
    table[0x5C] = [b'\\', b'\\'];
    table
};

static XDIGITS: &[u8] = b"0123456789ABCDEF";

/// Check if a codepoint should be encoded raw (not escaped) in JSON.
/// Returns true if ch >= 0x20 and utf_printable(ch).
///
/// # Safety
/// Calls `utf_printable` which is always safe to call.
#[inline]
unsafe fn encode_raw(ch: c_int) -> bool {
    ch >= 0x20 && utf_printable(ch)
}

/// Has a non-zero escape entry for `ch` (0x00..0x7F).
#[inline]
fn has_escape(ch: c_int) -> bool {
    let u = ch as usize;
    u < ESCAPES.len() && ESCAPES[u][0] != 0
}

/// Convert a string to a JSON double-quoted string, appending to `gap`.
///
/// Mirrors the C `convert_to_json_string` in encode.c.
///
/// # Safety
/// - `gap` must be a valid pointer to a `garray_T`.
/// - `buf` must be a valid pointer to `len` bytes, or null if len is 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_convert_to_json_string(
    gap: *mut GarrayT,
    buf: *const c_char,
    len: usize,
) -> c_int {
    const OK: c_int = 0;
    const FAIL: c_int = -1;

    let null_terminated_null_msg =
        b"E474: String \"%.*s\" contains byte that does not start any UTF-8 character\0";
    let surrogate_msg =
        b"E474: UTF-8 string contains code point which belongs to a surrogate pair: %.*s\0";

    if buf.is_null() {
        // ga_concat(gap, "\"\"")
        let empty = b"\"\"\0";
        ga_concat(gap, empty.as_ptr().cast());
        return OK;
    }

    let utf_buf = buf;
    let utf_len = len;

    // First pass: compute the length needed.
    let mut str_len: usize = 0;
    let mut i: usize = 0;
    while i < utf_len {
        let ch = utf_ptr2char(utf_buf.add(i));
        let raw_shift = if ch == 0 {
            1usize
        } else {
            utf_ptr2len(utf_buf.add(i)) as usize
        };
        debug_assert!(raw_shift > 0);
        i += raw_shift;

        if has_escape(ch) {
            str_len += 2;
        } else if ch > 0x7F && raw_shift == 1 {
            // Byte that doesn't start a valid UTF-8 sequence.
            semsg(
                null_terminated_null_msg.as_ptr().cast(),
                (utf_len - (i - raw_shift)) as c_int,
                utf_buf.add(i - raw_shift),
            );
            return FAIL;
        } else if (SURROGATE_HI_START..=SURROGATE_HI_END).contains(&ch)
            || (SURROGATE_LO_START..=SURROGATE_LO_END).contains(&ch)
        {
            semsg(
                surrogate_msg.as_ptr().cast(),
                (utf_len - (i - raw_shift)) as c_int,
                utf_buf.add(i - raw_shift),
            );
            return FAIL;
        } else if encode_raw(ch) {
            str_len += raw_shift;
        } else {
            // \uNNNN, possibly a surrogate pair (two \uNNNN)
            let escape_unit = b"\\u1234".len(); // 6
            str_len += escape_unit * (1 + usize::from(ch >= SURROGATE_FIRST_CHAR));
        }
    }

    // Opening quote
    ga_append(gap, b'"');
    ga_grow(gap, str_len as c_int);

    // Second pass: write the characters.
    let mut i: usize = 0;
    while i < utf_len {
        let ch = utf_ptr2char(utf_buf.add(i));
        let shift = if ch == 0 {
            1usize
        } else {
            utf_char2len(ch) as usize
        };
        debug_assert!(shift > 0);

        if has_escape(ch) {
            let esc = &ESCAPES[ch as usize];
            ga_concat_len(gap, esc.as_ptr().cast(), 2);
        } else if encode_raw(ch) {
            ga_concat_len(gap, utf_buf.add(i), shift);
        } else if ch < SURROGATE_FIRST_CHAR {
            let bytes: [u8; 6] = [
                b'\\',
                b'u',
                XDIGITS[((ch >> 12) & 0xF) as usize],
                XDIGITS[((ch >> 8) & 0xF) as usize],
                XDIGITS[((ch >> 4) & 0xF) as usize],
                XDIGITS[(ch & 0xF) as usize],
            ];
            ga_concat_len(gap, bytes.as_ptr().cast(), 6);
        } else {
            let tmp = ch - SURROGATE_FIRST_CHAR;
            let hi = SURROGATE_HI_START + ((tmp >> 10) & ((1 << 10) - 1));
            let lo = SURROGATE_LO_END + (tmp & ((1 << 10) - 1));
            let bytes: [u8; 12] = [
                b'\\',
                b'u',
                XDIGITS[((hi >> 12) & 0xF) as usize],
                XDIGITS[((hi >> 8) & 0xF) as usize],
                XDIGITS[((hi >> 4) & 0xF) as usize],
                XDIGITS[(hi & 0xF) as usize],
                b'\\',
                b'u',
                XDIGITS[((lo >> 12) & 0xF) as usize],
                XDIGITS[((lo >> 8) & 0xF) as usize],
                XDIGITS[((lo >> 4) & 0xF) as usize],
                XDIGITS[(lo & 0xF) as usize],
            ];
            ga_concat_len(gap, bytes.as_ptr().cast(), 12);
        }
        i += shift;
    }

    // Closing quote
    ga_append(gap, b'"');
    OK
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_encode_options() {
        let compact = JsonEncodeOptions::compact();
        assert!(!compact.pretty);

        let pretty = JsonEncodeOptions::pretty(2);
        assert!(pretty.pretty);
        assert_eq!(pretty.indent, 2);

        let bits = pretty.to_bits();
        let restored = JsonEncodeOptions::from_bits(bits);
        assert!(restored.pretty);
        assert_eq!(restored.indent, 2);
    }

    #[test]
    fn test_decode_result() {
        assert!(DecodeResult::Ok.is_ok());
        assert!(!DecodeResult::SyntaxError.is_ok());
        assert_eq!(DecodeResult::from_c_int(0), DecodeResult::Ok);
        assert_eq!(DecodeResult::from_c_int(1), DecodeResult::SyntaxError);
    }

    #[test]
    fn test_encode_result() {
        assert!(EncodeResult::Ok.is_ok());
        assert!(!EncodeResult::CircularRef.is_ok());
        assert_eq!(EncodeResult::from_c_int(0), EncodeResult::Ok);
        assert_eq!(EncodeResult::from_c_int(1), EncodeResult::CircularRef);
    }

    #[test]
    fn test_escapes_table() {
        // BS
        assert_eq!(ESCAPES[0x08], [b'\\', b'b']);
        // TAB
        assert_eq!(ESCAPES[0x09], [b'\\', b't']);
        // NL
        assert_eq!(ESCAPES[0x0A], [b'\\', b'n']);
        // FF
        assert_eq!(ESCAPES[0x0C], [b'\\', b'f']);
        // CAR
        assert_eq!(ESCAPES[0x0D], [b'\\', b'r']);
        // quote
        assert_eq!(ESCAPES[0x22], [b'\\', b'"']);
        // backslash
        assert_eq!(ESCAPES[0x5C], [b'\\', b'\\']);
    }

    #[test]
    fn test_has_escape() {
        assert!(has_escape(0x08)); // BS
        assert!(has_escape(0x09)); // TAB
        assert!(has_escape(0x0A)); // NL
        assert!(has_escape(0x22)); // '"'
        assert!(has_escape(0x5C)); // '\\'
        assert!(!has_escape(c_int::from(b'a')));
        assert!(!has_escape(c_int::from(b'z')));
    }
}
