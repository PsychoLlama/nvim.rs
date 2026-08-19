//! Building the byte sequences the terminal emulator writes back to its host.
//!
//! Upstream formatted every reply with a truncating `snprintf` into the
//! terminal's scratch buffer and pushed the result. [`EscapeSeq`] does the
//! same job on the stack: it fills to a fixed capacity, remembers whether
//! anything was dropped, and refuses to hand out a sequence that did not fit,
//! because half an escape sequence is worse on the wire than none.
//!
//! Ported from libvterm, Copyright (c) 2008 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libvterm-LICENSE.txt.

#![forbid(unsafe_code)]

use core::fmt;

/// One outbound escape sequence, assembled in place.
pub struct EscapeSeq {
    buf: [u8; EscapeSeq::CAPACITY],
    len: usize,
    overflowed: bool,
}

impl EscapeSeq {
    /// Comfortably above the longest reply this builder is used for, which is
    /// the DECRQSS report of the pen: a DCS introducer, up to twenty-odd SGR
    /// parameters, a final byte and a string terminator. Upstream's own
    /// ceiling was the terminal's 4 KiB scratch buffer.
    const CAPACITY: usize = 256;

    /// An empty sequence, for replies that carry no C1 introducer.
    pub fn new() -> Self {
        EscapeSeq {
            buf: [0; Self::CAPACITY],
            len: 0,
            overflowed: false,
        }
    }

    /// A sequence introduced by the C1 control `ctrl`: the 8-bit control
    /// itself when the host has asked for 8-bit controls, and its two-byte
    /// `ESC`-prefixed form otherwise.
    fn c1(ctrl: u8, ctrl8bit: bool) -> Self {
        let mut seq = Self::new();
        if ctrl >= 0x80 && !ctrl8bit {
            seq.push(0x1b);
            seq.push(ctrl - 0x40);
        } else {
            seq.push(ctrl);
        }
        seq
    }

    /// A control sequence: `CSI` (0x9b) or `ESC [`.
    pub fn csi(ctrl8bit: bool) -> Self {
        Self::c1(0x9b, ctrl8bit)
    }

    /// A single-shift-3 sequence: `SS3` (0x8f) or `ESC O`.
    pub fn ss3(ctrl8bit: bool) -> Self {
        Self::c1(0x8f, ctrl8bit)
    }

    /// A device-control string: `DCS` (0x90) or `ESC P`.
    pub fn dcs(ctrl8bit: bool) -> Self {
        Self::c1(0x90, ctrl8bit)
    }

    /// Close a control string with `ST` (0x9c), or its `ESC \` form.
    pub fn terminate(&mut self, ctrl8bit: bool) {
        if ctrl8bit {
            self.push(0x9c);
        } else {
            self.extend(b"\x1b\\");
        }
    }

    pub fn push(&mut self, byte: u8) {
        match self.buf.get_mut(self.len) {
            Some(slot) => {
                *slot = byte;
                self.len += 1;
            }
            None => self.overflowed = true,
        }
    }

    pub fn extend(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.push(byte);
        }
    }

    /// Append `codepoint` the way the editor's `utf_char2bytes` encodes it:
    /// the historical six-byte UTF-8, with no surrogate or U+10FFFF ceiling,
    /// and a negative value truncated into a single byte.
    pub fn push_utf8(&mut self, codepoint: i32) {
        if codepoint < 0x80 {
            self.push(codepoint as u8);
            return;
        }
        let (lead, trailing) = match codepoint {
            ..0x800 => (0xc0, 1),
            0x800..0x10000 => (0xe0, 2),
            0x1_0000..0x20_0000 => (0xf0, 3),
            0x20_0000..0x400_0000 => (0xf8, 4),
            _ => (0xfc, 5),
        };
        let cp = codepoint as u32;
        self.push(lead | (cp >> (6 * trailing)) as u8);
        for shift in (0..trailing).rev() {
            self.push(0x80 | (cp >> (6 * shift)) as u8 & 0x3f);
        }
    }

    /// The finished sequence, or `None` if it outgrew the buffer.
    pub fn finish(&self) -> Option<&[u8]> {
        (!self.overflowed).then(|| &self.buf[..self.len])
    }
}

impl Default for EscapeSeq {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Write for EscapeSeq {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.extend(s.as_bytes());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::fmt::Write;

    #[test]
    fn c1_introducers_follow_the_hosts_control_width() {
        assert_eq!(EscapeSeq::csi(false).finish(), Some(&b"\x1b["[..]));
        assert_eq!(EscapeSeq::csi(true).finish(), Some(&b"\x9b"[..]));
        assert_eq!(EscapeSeq::ss3(false).finish(), Some(&b"\x1bO"[..]));
        assert_eq!(EscapeSeq::ss3(true).finish(), Some(&b"\x8f"[..]));
        assert_eq!(EscapeSeq::dcs(false).finish(), Some(&b"\x1bP"[..]));
        assert_eq!(EscapeSeq::dcs(true).finish(), Some(&b"\x90"[..]));
    }

    #[test]
    fn a_control_string_is_closed_by_a_matching_terminator() {
        let mut seq = EscapeSeq::dcs(false);
        seq.extend(b"0$r");
        seq.terminate(false);
        assert_eq!(seq.finish(), Some(&b"\x1bP0$r\x1b\\"[..]));

        let mut seq = EscapeSeq::dcs(true);
        seq.extend(b"0$r");
        seq.terminate(true);
        assert_eq!(seq.finish(), Some(&b"\x900$r\x9c"[..]));
    }

    #[test]
    fn formatting_appends_to_the_introducer() {
        let mut seq = EscapeSeq::csi(false);
        write!(seq, "{};{}u", 97, 5).unwrap();
        assert_eq!(seq.finish(), Some(&b"\x1b[97;5u"[..]));
    }

    #[test]
    fn an_overlong_sequence_is_dropped_whole() {
        let mut seq = EscapeSeq::new();
        seq.extend(&[b'x'; EscapeSeq::CAPACITY]);
        assert_eq!(seq.finish(), Some(&[b'x'; EscapeSeq::CAPACITY][..]));
        seq.push(b'y');
        assert_eq!(seq.finish(), None);
    }

    #[test]
    fn utf8_matches_the_editors_six_byte_encoding() {
        let encode = |cp| {
            let mut seq = EscapeSeq::new();
            seq.push_utf8(cp);
            seq.finish().unwrap().to_vec()
        };
        assert_eq!(encode(0x41), b"A");
        assert_eq!(encode(0), b"\0");
        assert_eq!(encode(0x7f), b"\x7f");
        assert_eq!(encode(0xe9), "é".as_bytes());
        assert_eq!(encode(0x20ac), "€".as_bytes());
        assert_eq!(encode(0x1f600), "😀".as_bytes());
        // Beyond what `char` can hold, where the historical encoding keeps going.
        assert_eq!(encode(0x20_0000), b"\xf8\x88\x80\x80\x80");
        assert_eq!(encode(0x400_0000), b"\xfc\x84\x80\x80\x80\x80");
        // Surrogates are encoded, not rejected.
        assert_eq!(encode(0xd800), b"\xed\xa0\x80");
        // A negative codepoint truncates into one byte, as the C did.
        assert_eq!(encode(-1), b"\xff");
    }
}
