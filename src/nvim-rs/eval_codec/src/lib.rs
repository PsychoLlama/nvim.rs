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
pub mod escape;
pub mod json;

pub use blob::*;
pub use escape::*;
pub use json::*;

use std::ffi::c_int;

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

/// FFI export: create compact JSON options.
#[no_mangle]
pub extern "C" fn rs_json_opts_compact() -> u32 {
    JsonEncodeOptions::compact().to_bits()
}

/// FFI export: create pretty JSON options.
#[no_mangle]
pub extern "C" fn rs_json_opts_pretty(indent: c_int) -> u32 {
    JsonEncodeOptions::pretty(indent).to_bits()
}

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

/// FFI export: check if decode result is ok.
#[no_mangle]
pub extern "C" fn rs_decode_is_ok(result: c_int) -> bool {
    DecodeResult::from_c_int(result).is_ok()
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

/// FFI export: check if encode result is ok.
#[no_mangle]
pub extern "C" fn rs_encode_is_ok(result: c_int) -> bool {
    EncodeResult::from_c_int(result).is_ok()
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
}
