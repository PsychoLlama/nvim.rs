//! String escape and unescape helpers.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_possible_truncation)]

// =============================================================================
// JSON String Escaping
// =============================================================================

/// Check if character needs JSON escaping.
pub const fn needs_json_escape(c: u8) -> bool {
    c < 0x20 || c == b'"' || c == b'\\'
}

/// Get JSON escape sequence for a character.
///
/// Returns None if character doesn't need escaping.
pub const fn json_escape_char(c: u8) -> Option<&'static [u8]> {
    match c {
        b'"' => Some(b"\\\""),
        b'\\' => Some(b"\\\\"),
        b'\x08' => Some(b"\\b"),
        b'\x0C' => Some(b"\\f"),
        b'\n' => Some(b"\\n"),
        b'\r' => Some(b"\\r"),
        b'\t' => Some(b"\\t"),
        _ if c < 0x20 => None, // Need \uXXXX format, handled separately
        _ => None,
    }
}

/// Escape a string for JSON.
///
/// Returns the escaped string as bytes.
pub fn escape_json_string(input: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(input.len() + 16);

    for &c in input {
        if let Some(escaped) = json_escape_char(c) {
            result.extend_from_slice(escaped);
        } else if c < 0x20 {
            // Control character: use \uXXXX format
            result.extend_from_slice(b"\\u00");
            result.push(hex_digit(c >> 4));
            result.push(hex_digit(c & 0x0F));
        } else {
            result.push(c);
        }
    }

    result
}

/// Get hex digit character for value 0-15.
const fn hex_digit(val: u8) -> u8 {
    match val {
        0..=9 => b'0' + val,
        10..=15 => b'a' + (val - 10),
        _ => b'?',
    }
}

// =============================================================================
// JSON String Unescaping
// =============================================================================

/// Unescape result.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct UnescapeResult {
    /// Whether unescaping succeeded
    pub ok: bool,
    /// The unescaped character (if single char)
    pub char_val: u8,
    /// Bytes consumed from input
    pub consumed: i32,
}

/// Unescape a JSON escape sequence.
///
/// Input should start after the backslash.
/// Returns (unescaped_char, bytes_consumed).
pub fn unescape_json_char(input: &[u8]) -> Option<(u8, usize)> {
    if input.is_empty() {
        return None;
    }

    match input[0] {
        b'"' => Some((b'"', 1)),
        b'\\' => Some((b'\\', 1)),
        b'/' => Some((b'/', 1)),
        b'b' => Some((b'\x08', 1)),
        b'f' => Some((b'\x0C', 1)),
        b'n' => Some((b'\n', 1)),
        b'r' => Some((b'\r', 1)),
        b't' => Some((b'\t', 1)),
        b'u' => {
            // Unicode escape: \uXXXX
            if input.len() < 5 {
                return None;
            }
            let hex_str = std::str::from_utf8(&input[1..5]).ok()?;
            let code = u16::from_str_radix(hex_str, 16).ok()?;

            // Only handle ASCII range for now
            if code <= 0x7F {
                Some((code as u8, 5))
            } else {
                // Non-ASCII Unicode - would need multi-byte handling
                None
            }
        }
        _ => None,
    }
}

// =============================================================================
// VimL String Escaping
// =============================================================================

/// Escape a string for VimL double-quoted string.
pub fn escape_viml_string(input: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(input.len() + 16);

    for &c in input {
        match c {
            b'"' => result.extend_from_slice(b"\\\""),
            b'\\' => result.extend_from_slice(b"\\\\"),
            b'\n' => result.extend_from_slice(b"\\n"),
            b'\r' => result.extend_from_slice(b"\\r"),
            b'\t' => result.extend_from_slice(b"\\t"),
            b'\x1B' => result.extend_from_slice(b"\\e"),
            b'\x08' => result.extend_from_slice(b"\\b"),
            _ if c < 0x20 => {
                // Control character: use \xXX format
                result.extend_from_slice(b"\\x");
                result.push(hex_digit(c >> 4));
                result.push(hex_digit(c & 0x0F));
            }
            _ => result.push(c),
        }
    }

    result
}

/// Check if character needs VimL escaping in double-quoted string.
pub const fn needs_viml_escape(c: u8) -> bool {
    c < 0x20 || c == b'"' || c == b'\\'
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_needs_json_escape() {
        assert!(needs_json_escape(b'"'));
        assert!(needs_json_escape(b'\\'));
        assert!(needs_json_escape(b'\n'));
        assert!(needs_json_escape(0x01));
        assert!(!needs_json_escape(b'a'));
        assert!(!needs_json_escape(b' '));
    }

    #[test]
    fn test_escape_json_string() {
        assert_eq!(escape_json_string(b"hello"), b"hello");
        assert_eq!(escape_json_string(b"a\"b"), b"a\\\"b");
        assert_eq!(escape_json_string(b"a\\b"), b"a\\\\b");
        assert_eq!(escape_json_string(b"a\nb"), b"a\\nb");
    }

    #[test]
    fn test_unescape_json_char() {
        assert_eq!(unescape_json_char(b"\""), Some((b'"', 1)));
        assert_eq!(unescape_json_char(b"\\"), Some((b'\\', 1)));
        assert_eq!(unescape_json_char(b"n"), Some((b'\n', 1)));
        assert_eq!(unescape_json_char(b"u0041"), Some((b'A', 5)));
    }

    #[test]
    fn test_escape_viml_string() {
        assert_eq!(escape_viml_string(b"hello"), b"hello");
        assert_eq!(escape_viml_string(b"a\"b"), b"a\\\"b");
        assert_eq!(escape_viml_string(b"a\nb"), b"a\\nb");
    }
}
