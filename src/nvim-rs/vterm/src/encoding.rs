//! Character encoding for `VTerm`
//!
//! This module provides character encoding and decoding for terminal emulation,
//! including UTF-8, US-ASCII, and DEC special graphics character sets.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::c_int;

/// Unicode replacement character for invalid sequences
pub const UNICODE_INVALID: u32 = 0xFFFD;

// =============================================================================
// Encoding Types
// =============================================================================

/// Character encoding type
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum EncodingType {
    /// UTF-8 encoding
    #[default]
    Utf8 = 0,
    /// Single 94-character set (like ASCII or DEC graphics)
    Single94 = 1,
}

// =============================================================================
// UTF-8 Decoder
// =============================================================================

/// UTF-8 decoder state
#[derive(Clone, Copy, Debug, Default)]
pub struct Utf8Decoder {
    /// Number of bytes remaining in current codepoint
    bytes_remaining: u8,
    /// Total number of bytes in current codepoint (for overlong detection)
    bytes_total: u8,
    /// Accumulated codepoint value
    this_cp: u32,
}

impl Utf8Decoder {
    /// Create a new UTF-8 decoder
    pub const fn new() -> Self {
        Self {
            bytes_remaining: 0,
            bytes_total: 0,
            this_cp: 0,
        }
    }

    /// Reset the decoder state
    pub fn reset(&mut self) {
        self.bytes_remaining = 0;
        self.bytes_total = 0;
        self.this_cp = 0;
    }

    /// Decode UTF-8 bytes into codepoints
    ///
    /// # Arguments
    /// * `bytes` - Input byte slice
    /// * `pos` - Current position in bytes (updated as bytes are consumed)
    /// * `output` - Output buffer for codepoints
    /// * `out_idx` - Current index in output (updated as codepoints are written)
    ///
    /// Returns when a control character (< 0x20 or 0x7F) is encountered,
    /// or when the output buffer is full.
    pub fn decode(
        &mut self,
        bytes: &[u8],
        pos: &mut usize,
        output: &mut [u32],
        out_idx: &mut usize,
    ) {
        while *pos < bytes.len() && *out_idx < output.len() {
            let c = bytes[*pos];

            if c < 0x20 {
                // C0 control character - stop
                return;
            } else if c < 0x7f {
                // ASCII printable
                if self.bytes_remaining > 0 {
                    output[*out_idx] = UNICODE_INVALID;
                    *out_idx += 1;
                    if *out_idx >= output.len() {
                        return;
                    }
                }
                output[*out_idx] = u32::from(c);
                *out_idx += 1;
                self.bytes_remaining = 0;
                *pos += 1;
            } else if c == 0x7f {
                // DEL - stop
                return;
            } else if c < 0xc0 {
                // Continuation byte (10xxxxxx)
                if self.bytes_remaining == 0 {
                    output[*out_idx] = UNICODE_INVALID;
                    *out_idx += 1;
                    *pos += 1;
                    continue;
                }

                self.this_cp <<= 6;
                self.this_cp |= u32::from(c & 0x3f);
                self.bytes_remaining -= 1;
                *pos += 1;

                if self.bytes_remaining == 0 {
                    // Complete codepoint - check for overlongs
                    let cp = self.validate_codepoint();
                    output[*out_idx] = cp;
                    *out_idx += 1;
                }
            } else if c < 0xe0 {
                // 2-byte sequence start (110xxxxx)
                if self.bytes_remaining > 0 {
                    output[*out_idx] = UNICODE_INVALID;
                    *out_idx += 1;
                    if *out_idx >= output.len() {
                        return;
                    }
                }
                self.this_cp = u32::from(c & 0x1f);
                self.bytes_total = 2;
                self.bytes_remaining = 1;
                *pos += 1;
            } else if c < 0xf0 {
                // 3-byte sequence start (1110xxxx)
                if self.bytes_remaining > 0 {
                    output[*out_idx] = UNICODE_INVALID;
                    *out_idx += 1;
                    if *out_idx >= output.len() {
                        return;
                    }
                }
                self.this_cp = u32::from(c & 0x0f);
                self.bytes_total = 3;
                self.bytes_remaining = 2;
                *pos += 1;
            } else if c < 0xf8 {
                // 4-byte sequence start (11110xxx)
                if self.bytes_remaining > 0 {
                    output[*out_idx] = UNICODE_INVALID;
                    *out_idx += 1;
                    if *out_idx >= output.len() {
                        return;
                    }
                }
                self.this_cp = u32::from(c & 0x07);
                self.bytes_total = 4;
                self.bytes_remaining = 3;
                *pos += 1;
            } else if c < 0xfc {
                // 5-byte sequence start (111110xx) - technically invalid UTF-8
                if self.bytes_remaining > 0 {
                    output[*out_idx] = UNICODE_INVALID;
                    *out_idx += 1;
                    if *out_idx >= output.len() {
                        return;
                    }
                }
                self.this_cp = u32::from(c & 0x03);
                self.bytes_total = 5;
                self.bytes_remaining = 4;
                *pos += 1;
            } else if c < 0xfe {
                // 6-byte sequence start (1111110x) - technically invalid UTF-8
                if self.bytes_remaining > 0 {
                    output[*out_idx] = UNICODE_INVALID;
                    *out_idx += 1;
                    if *out_idx >= output.len() {
                        return;
                    }
                }
                self.this_cp = u32::from(c & 0x01);
                self.bytes_total = 6;
                self.bytes_remaining = 5;
                *pos += 1;
            } else {
                // 0xFE or 0xFF - invalid
                output[*out_idx] = UNICODE_INVALID;
                *out_idx += 1;
                *pos += 1;
            }
        }
    }

    /// Validate a complete codepoint, checking for overlong sequences and invalid values
    fn validate_codepoint(self) -> u32 {
        let cp = self.this_cp;

        // Check for overlong sequences
        let is_overlong = match self.bytes_total {
            2 => cp < 0x0080,
            3 => cp < 0x0800,
            4 => cp < 0x1_0000,
            5 => cp < 0x20_0000,
            6 => cp < 0x400_0000,
            _ => false,
        };

        if is_overlong {
            return UNICODE_INVALID;
        }

        // Check for surrogate pairs and invalid codepoints
        if (0xD800..=0xDFFF).contains(&cp) || cp == 0xFFFE || cp == 0xFFFF {
            return UNICODE_INVALID;
        }

        cp
    }
}

// =============================================================================
// US-ASCII Decoder
// =============================================================================

/// Decode US-ASCII bytes
///
/// Handles both GL (0x20-0x7E) and GR (0xA0-0xFE) ranges.
/// Stops at control characters.
pub fn decode_usascii(bytes: &[u8], pos: &mut usize, output: &mut [u32], out_idx: &mut usize) {
    if *pos >= bytes.len() {
        return;
    }

    // Determine if we're in GR range (high bit set)
    let is_gr = bytes[*pos] & 0x80;

    while *pos < bytes.len() && *out_idx < output.len() {
        let c = bytes[*pos] ^ is_gr;

        if c < 0x20 || c == 0x7f || c >= 0x80 {
            return;
        }

        output[*out_idx] = u32::from(c);
        *out_idx += 1;
        *pos += 1;
    }
}

// =============================================================================
// DEC Special Graphics
// =============================================================================

/// DEC Special Graphics character set translation table
///
/// Maps ASCII characters 0x60-0x7E to special graphics Unicode codepoints
const DEC_DRAWING_TABLE: [(u8, u32); 31] = [
    (0x60, 0x25C6), // BLACK DIAMOND
    (0x61, 0x2592), // MEDIUM SHADE (checkerboard)
    (0x62, 0x2409), // SYMBOL FOR HORIZONTAL TAB
    (0x63, 0x240C), // SYMBOL FOR FORM FEED
    (0x64, 0x240D), // SYMBOL FOR CARRIAGE RETURN
    (0x65, 0x240A), // SYMBOL FOR LINE FEED
    (0x66, 0x00B0), // DEGREE SIGN
    (0x67, 0x00B1), // PLUS-MINUS SIGN
    (0x68, 0x2424), // SYMBOL FOR NEW LINE
    (0x69, 0x240B), // SYMBOL FOR VERTICAL TAB
    (0x6a, 0x2518), // BOX DRAWINGS LIGHT UP AND LEFT
    (0x6b, 0x2510), // BOX DRAWINGS LIGHT DOWN AND LEFT
    (0x6c, 0x250C), // BOX DRAWINGS LIGHT DOWN AND RIGHT
    (0x6d, 0x2514), // BOX DRAWINGS LIGHT UP AND RIGHT
    (0x6e, 0x253C), // BOX DRAWINGS LIGHT VERTICAL AND HORIZONTAL
    (0x6f, 0x23BA), // HORIZONTAL SCAN LINE-1
    (0x70, 0x23BB), // HORIZONTAL SCAN LINE-3
    (0x71, 0x2500), // BOX DRAWINGS LIGHT HORIZONTAL
    (0x72, 0x23BC), // HORIZONTAL SCAN LINE-7
    (0x73, 0x23BD), // HORIZONTAL SCAN LINE-9
    (0x74, 0x251C), // BOX DRAWINGS LIGHT VERTICAL AND RIGHT
    (0x75, 0x2524), // BOX DRAWINGS LIGHT VERTICAL AND LEFT
    (0x76, 0x2534), // BOX DRAWINGS LIGHT UP AND HORIZONTAL
    (0x77, 0x252C), // BOX DRAWINGS LIGHT DOWN AND HORIZONTAL
    (0x78, 0x2502), // BOX DRAWINGS LIGHT VERTICAL
    (0x79, 0x2A7D), // LESS-THAN OR SLANTED EQUAL-TO
    (0x7a, 0x2A7E), // GREATER-THAN OR SLANTED EQUAL-TO
    (0x7b, 0x03C0), // GREEK SMALL LETTER PI
    (0x7c, 0x2260), // NOT EQUAL TO
    (0x7d, 0x00A3), // POUND SIGN
    (0x7e, 0x00B7), // MIDDLE DOT
];

/// Look up DEC drawing character
fn dec_drawing_lookup(c: u8) -> Option<u32> {
    for &(byte, cp) in &DEC_DRAWING_TABLE {
        if byte == c {
            return Some(cp);
        }
    }
    None
}

/// Decode DEC Special Graphics characters
///
/// Translates characters in the 0x60-0x7E range to box drawing characters.
pub fn decode_dec_drawing(bytes: &[u8], pos: &mut usize, output: &mut [u32], out_idx: &mut usize) {
    if *pos >= bytes.len() {
        return;
    }

    // Determine if we're in GR range
    let is_gr = bytes[*pos] & 0x80;

    while *pos < bytes.len() && *out_idx < output.len() {
        let c = bytes[*pos] ^ is_gr;

        if c < 0x20 || c == 0x7f || c >= 0x80 {
            return;
        }

        // Look up in the DEC drawing table
        if let Some(cp) = dec_drawing_lookup(c) {
            output[*out_idx] = cp;
        } else {
            output[*out_idx] = u32::from(c);
        }
        *out_idx += 1;
        *pos += 1;
    }
}

// =============================================================================
// Encoding Lookup
// =============================================================================

/// Encoding designation characters
pub const ENCODING_UTF8: char = 'u';
pub const ENCODING_DEC_DRAWING: char = '0';
pub const ENCODING_USASCII: char = 'B';

/// Character encoding
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Encoding {
    /// UTF-8 encoding
    Utf8,
    /// US-ASCII (7-bit)
    UsAscii,
    /// DEC Special Graphics
    DecDrawing,
}

impl Encoding {
    /// Look up encoding by type and designation character
    pub fn lookup(enc_type: EncodingType, designation: char) -> Option<Self> {
        match (enc_type, designation) {
            (EncodingType::Utf8, 'u') => Some(Self::Utf8),
            (EncodingType::Single94, '0') => Some(Self::DecDrawing),
            (EncodingType::Single94, 'B') => Some(Self::UsAscii),
            _ => None,
        }
    }

    /// Get the encoding type
    pub const fn encoding_type(&self) -> EncodingType {
        match self {
            Self::Utf8 => EncodingType::Utf8,
            Self::UsAscii | Self::DecDrawing => EncodingType::Single94,
        }
    }

    /// Get the designation character
    pub const fn designation(&self) -> char {
        match self {
            Self::Utf8 => 'u',
            Self::UsAscii => 'B',
            Self::DecDrawing => '0',
        }
    }
}

// =============================================================================
// FFI Functions
// =============================================================================

/// Look up an encoding by type and designation
#[no_mangle]
pub extern "C" fn rs_vterm_lookup_encoding(enc_type: c_int, designation: c_int) -> c_int {
    let enc_type = match enc_type {
        0 => EncodingType::Utf8,
        1 => EncodingType::Single94,
        _ => return -1,
    };

    let Some(designation) = char::from_u32(designation as u32) else {
        return -1;
    };

    match Encoding::lookup(enc_type, designation) {
        Some(Encoding::Utf8) => 0,
        Some(Encoding::UsAscii) => 1,
        Some(Encoding::DecDrawing) => 2,
        None => -1,
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utf8_decoder_ascii() {
        let mut decoder = Utf8Decoder::new();
        let bytes = b"Hello";
        let mut output = [0u32; 10];
        let mut pos = 0;
        let mut out_idx = 0;

        decoder.decode(bytes, &mut pos, &mut output, &mut out_idx);

        assert_eq!(pos, 5);
        assert_eq!(out_idx, 5);
        assert_eq!(
            &output[..5],
            &[
                b'H' as u32,
                b'e' as u32,
                b'l' as u32,
                b'l' as u32,
                b'o' as u32
            ]
        );
    }

    #[test]
    fn test_utf8_decoder_2byte() {
        let mut decoder = Utf8Decoder::new();
        let bytes = b"\xC3\xA9"; // é (U+00E9)
        let mut output = [0u32; 10];
        let mut pos = 0;
        let mut out_idx = 0;

        decoder.decode(bytes, &mut pos, &mut output, &mut out_idx);

        assert_eq!(pos, 2);
        assert_eq!(out_idx, 1);
        assert_eq!(output[0], 0xE9);
    }

    #[test]
    fn test_utf8_decoder_3byte() {
        let mut decoder = Utf8Decoder::new();
        let bytes = b"\xE4\xB8\xAD"; // 中 (U+4E2D)
        let mut output = [0u32; 10];
        let mut pos = 0;
        let mut out_idx = 0;

        decoder.decode(bytes, &mut pos, &mut output, &mut out_idx);

        assert_eq!(pos, 3);
        assert_eq!(out_idx, 1);
        assert_eq!(output[0], 0x4E2D);
    }

    #[test]
    fn test_utf8_decoder_4byte() {
        let mut decoder = Utf8Decoder::new();
        let bytes = b"\xF0\x9F\x98\x80"; // 😀 (U+1F600)
        let mut output = [0u32; 10];
        let mut pos = 0;
        let mut out_idx = 0;

        decoder.decode(bytes, &mut pos, &mut output, &mut out_idx);

        assert_eq!(pos, 4);
        assert_eq!(out_idx, 1);
        assert_eq!(output[0], 0x1F600);
    }

    #[test]
    fn test_utf8_decoder_control_stop() {
        let mut decoder = Utf8Decoder::new();
        let bytes = b"Hello\x1bWorld"; // ESC in the middle
        let mut output = [0u32; 10];
        let mut pos = 0;
        let mut out_idx = 0;

        decoder.decode(bytes, &mut pos, &mut output, &mut out_idx);

        assert_eq!(pos, 5); // Stopped at ESC
        assert_eq!(out_idx, 5);
    }

    #[test]
    fn test_utf8_decoder_invalid_continuation() {
        let mut decoder = Utf8Decoder::new();
        let bytes = b"\x80"; // Bare continuation byte
        let mut output = [0u32; 10];
        let mut pos = 0;
        let mut out_idx = 0;

        decoder.decode(bytes, &mut pos, &mut output, &mut out_idx);

        assert_eq!(out_idx, 1);
        assert_eq!(output[0], UNICODE_INVALID);
    }

    #[test]
    fn test_utf8_decoder_overlong() {
        let mut decoder = Utf8Decoder::new();
        // Overlong encoding of '/' (0x2F) as 2-byte sequence
        let bytes = b"\xC0\xAF";
        let mut output = [0u32; 10];
        let mut pos = 0;
        let mut out_idx = 0;

        decoder.decode(bytes, &mut pos, &mut output, &mut out_idx);

        assert_eq!(out_idx, 1);
        assert_eq!(output[0], UNICODE_INVALID);
    }

    #[test]
    fn test_utf8_decoder_surrogate() {
        let mut decoder = Utf8Decoder::new();
        // Encoded surrogate U+D800
        let bytes = b"\xED\xA0\x80";
        let mut output = [0u32; 10];
        let mut pos = 0;
        let mut out_idx = 0;

        decoder.decode(bytes, &mut pos, &mut output, &mut out_idx);

        assert_eq!(out_idx, 1);
        assert_eq!(output[0], UNICODE_INVALID);
    }

    #[test]
    fn test_decode_usascii() {
        let bytes = b"Hello";
        let mut output = [0u32; 10];
        let mut pos = 0;
        let mut out_idx = 0;

        decode_usascii(bytes, &mut pos, &mut output, &mut out_idx);

        assert_eq!(pos, 5);
        assert_eq!(out_idx, 5);
    }

    #[test]
    fn test_decode_usascii_control_stop() {
        let bytes = b"AB\x00CD";
        let mut output = [0u32; 10];
        let mut pos = 0;
        let mut out_idx = 0;

        decode_usascii(bytes, &mut pos, &mut output, &mut out_idx);

        assert_eq!(pos, 2); // Stopped at NUL
        assert_eq!(out_idx, 2);
    }

    #[test]
    fn test_decode_dec_drawing() {
        let bytes = b"qlx"; // horizontal line, corner, vertical line
        let mut output = [0u32; 10];
        let mut pos = 0;
        let mut out_idx = 0;

        decode_dec_drawing(bytes, &mut pos, &mut output, &mut out_idx);

        assert_eq!(pos, 3);
        assert_eq!(out_idx, 3);
        assert_eq!(output[0], 0x2500); // BOX DRAWINGS LIGHT HORIZONTAL
        assert_eq!(output[1], 0x250C); // BOX DRAWINGS LIGHT DOWN AND RIGHT
        assert_eq!(output[2], 0x2502); // BOX DRAWINGS LIGHT VERTICAL
    }

    #[test]
    fn test_dec_drawing_passthrough() {
        // Characters not in the table should pass through
        let bytes = b"ABC";
        let mut output = [0u32; 10];
        let mut pos = 0;
        let mut out_idx = 0;

        decode_dec_drawing(bytes, &mut pos, &mut output, &mut out_idx);

        assert_eq!(pos, 3);
        assert_eq!(out_idx, 3);
        assert_eq!(output[0], b'A' as u32);
        assert_eq!(output[1], b'B' as u32);
        assert_eq!(output[2], b'C' as u32);
    }

    #[test]
    fn test_encoding_lookup() {
        assert_eq!(
            Encoding::lookup(EncodingType::Utf8, 'u'),
            Some(Encoding::Utf8)
        );
        assert_eq!(
            Encoding::lookup(EncodingType::Single94, 'B'),
            Some(Encoding::UsAscii)
        );
        assert_eq!(
            Encoding::lookup(EncodingType::Single94, '0'),
            Some(Encoding::DecDrawing)
        );
        assert_eq!(Encoding::lookup(EncodingType::Utf8, 'x'), None);
    }

    #[test]
    fn test_encoding_properties() {
        assert_eq!(Encoding::Utf8.encoding_type(), EncodingType::Utf8);
        assert_eq!(Encoding::Utf8.designation(), 'u');
        assert_eq!(Encoding::UsAscii.encoding_type(), EncodingType::Single94);
        assert_eq!(Encoding::UsAscii.designation(), 'B');
    }

    #[test]
    fn test_ffi_lookup_encoding() {
        assert_eq!(rs_vterm_lookup_encoding(0, b'u' as c_int), 0); // UTF-8
        assert_eq!(rs_vterm_lookup_encoding(1, b'B' as c_int), 1); // US-ASCII
        assert_eq!(rs_vterm_lookup_encoding(1, b'0' as c_int), 2); // DEC Drawing
        assert_eq!(rs_vterm_lookup_encoding(1, b'x' as c_int), -1); // Unknown
        assert_eq!(rs_vterm_lookup_encoding(99, b'u' as c_int), -1); // Invalid type
    }
}
