//! Rendering an unprintable character for display.
//!
//! The forms are `^X` for a C0 control, `<xx>` / `<xxxx>` / `<xxxxxx>` for
//! anything else, and a `~@` prefix for the negative codes the key
//! translation layer uses.

#![forbid(unsafe_code)]

use core::ffi::c_int;

// Nested so that ffigen, which collects a file's top-level constants into
// one flat C namespace, does not publish a name this generic.
mod limit {
    /// The longest rendering: `~@` plus `<xxxxxx>`.
    pub const MAX_LEN: usize = 11;
}
pub use limit::MAX_LEN;

/// A rendered character: a fixed buffer plus the number of bytes used. The
/// buffer is NUL-terminated, because the callers hand it to C string code.
pub struct Rendered {
    pub bytes: [u8; MAX_LEN],
    pub len: usize,
}

impl Rendered {
    #[inline(always)]
    fn new() -> Self {
        Rendered {
            bytes: [0; MAX_LEN],
            len: 0,
        }
    }

    #[inline(always)]
    fn push(&mut self, byte: u8) {
        self.bytes[self.len] = byte;
        self.len += 1;
    }

    /// A character that displays as itself.
    #[inline(always)]
    pub fn literal(byte: u8) -> Rendered {
        let mut out = Rendered::new();
        out.push(byte);
        out
    }

    /// This rendering behind the two-byte escape prefix.
    #[inline(always)]
    pub fn behind(&self, prefix: &[u8; 2]) -> Rendered {
        let mut out = Rendered::new();
        out.push(prefix[0]);
        out.push(prefix[1]);
        for &byte in &self.bytes[..self.len] {
            out.push(byte);
        }
        out
    }
}

/// The low nibble of `n` as a lowercase hex digit.
#[inline(always)]
fn hex_digit(n: c_int) -> u8 {
    let n = (n & 0xf) as u8;
    if n <= 9 { b'0' + n } else { b'a' + n - 10 }
}

/// `<xx>`, widening to four or six digits for larger codepoints. The number
/// of bytes written excludes the terminating NUL, matching what the C
/// returned.
#[inline(always)]
pub fn hex_form(c: c_int) -> Rendered {
    let mut out = Rendered::new();
    out.push(b'<');
    if c > 0xff {
        if c > 0xffff {
            out.push(hex_digit(c >> 20));
            out.push(hex_digit(c >> 16));
        }
        out.push(hex_digit(c >> 12));
        out.push(hex_digit(c >> 8));
    }
    out.push(hex_digit(c >> 4));
    out.push(hex_digit(c));
    out.push(b'>');
    out
}

/// `^X` for a C0 control character, or DEL as `^?`.
#[inline(always)]
pub fn control_form(c: c_int) -> Rendered {
    let mut out = Rendered::new();
    out.push(b'^');
    out.push((c ^ 0x40) as u8);
    out
}

/// The two-byte prefix `transchar` puts in front of a negative code, and the
/// byte it renders instead.
///
/// Upstream also maps `K_SPECIAL` to `KS_SPECIAL` and NUL to `KS_ZERO`
/// here, but both arms are dead: every caller reaches this only for `c < 0`,
/// and neither 0x80 nor 0 is negative. Kept, and pinned, as it stands.
#[inline(always)]
pub fn negative_form(c: c_int) -> (&'static [u8; 2], c_int) {
    const K_SPECIAL: c_int = 0x80;
    const KS_SPECIAL: c_int = 254;
    const KS_ZERO: c_int = 255;
    let byte = if c == K_SPECIAL {
        KS_SPECIAL
    } else if c == 0 {
        KS_ZERO
    } else {
        -c & 0xff
    };
    (b"~@", byte)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(r: Rendered) -> String {
        String::from_utf8(r.bytes[..r.len].to_vec()).unwrap()
    }

    #[test]
    fn hex_widens_with_the_codepoint() {
        assert_eq!(text(hex_form(0x07)), "<07>");
        assert_eq!(text(hex_form(0xff)), "<ff>");
        assert_eq!(text(hex_form(0x100)), "<0100>");
        assert_eq!(text(hex_form(0xffff)), "<ffff>");
        assert_eq!(text(hex_form(0x10000)), "<010000>");
        assert_eq!(text(hex_form(0x10fffe)), "<10fffe>");
    }

    #[test]
    fn hex_form_reports_its_own_length_and_stays_terminated() {
        let r = hex_form(0x1b);
        assert_eq!(r.len, 4);
        assert_eq!(r.bytes[r.len], 0);
        assert_eq!(hex_form(0x10000).len, MAX_LEN - 3);
    }

    #[test]
    fn controls_render_as_caret_forms() {
        assert_eq!(text(control_form(0x00)), "^@");
        assert_eq!(text(control_form(0x09)), "^I");
        assert_eq!(text(control_form(0x1b)), "^[");
        assert_eq!(text(control_form(0x7f)), "^?");
    }

    #[test]
    fn a_prefix_carries_the_rendering_along() {
        assert_eq!(text(control_form(0x1b).behind(b"~@")), "~@^[");
        assert_eq!(text(hex_form(0xff).behind(b"~@")), "~@<ff>");
        assert_eq!(text(Rendered::literal(b'x')), "x");
        // The longest prefixed form still terminates inside the buffer.
        let longest = hex_form(0xff).behind(b"~@");
        assert_eq!(longest.bytes[longest.len], 0);
    }

    #[test]
    fn negative_codes_get_the_escape_prefix() {
        assert_eq!(negative_form(-0x41), (b"~@", 0x41));
        assert_eq!(negative_form(-0xff), (b"~@", 0xff));
        // Not KS_SPECIAL: the `c == K_SPECIAL` arm cannot see a negative c.
        assert_eq!(negative_form(-0x80), (b"~@", 0x80));
    }
}
