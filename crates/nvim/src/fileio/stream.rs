//! The byte-level readers and writers binary formats and line files share,
//! over any `Read`/`BufRead`/`Write`.
//!
//! [`Fgets`] is C's `fgets` -- one line, or as much of it as fits -- with
//! [`Fgets::line_capped`] the editor's `vim_fgets`, which throws the rest of
//! an over-long line away. [`read_be`]/[`read_counted`] read the big-endian
//! integers and counted strings the spell and undo files are made of, and
//! [`write_be`]/[`write_time`] write them. The `FILE *` forms of the same
//! helpers (`legacy_stream`) are thin shims over these.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use std::fs::File;
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};

use crate::types::time_t;

/// C's `fgets` over a reader: the same chunking, so that a long line is cut
/// where upstream cuts it.
pub(crate) struct Fgets<R = BufReader<File>> {
    reader: R,
    /// A read failed with something other than an interrupt: `ferror()`.
    error: bool,
}

impl Fgets {
    pub(crate) fn new(file: File) -> Fgets {
        Fgets::over(BufReader::new(file))
    }
}

impl<R: BufRead> Fgets<R> {
    /// `fgets` over `reader`.
    pub(crate) fn over(reader: R) -> Self {
        Fgets {
            reader,
            error: false,
        }
    }

    /// Read at most `size - 1` bytes, stopping after a newline, into `out`
    /// (which is cleared first). `false` at the end of the file -- or after
    /// an error -- with nothing read, as `fgets` answers NULL.
    pub(crate) fn fgets(&mut self, out: &mut Vec<u8>, size: usize) -> bool {
        out.clear();
        let room = size.saturating_sub(1);
        while out.len() < room {
            let available = match self.reader.fill_buf() {
                Ok(available) => available,
                Err(err) if err.kind() == ErrorKind::Interrupted => continue,
                Err(_) => {
                    self.error = true;
                    break;
                }
            };
            if available.is_empty() {
                break;
            }
            let want = (room - out.len()).min(available.len());
            let (take, newline) = match crate::cstr::find_byte(&available[..want], b'\n') {
                Some(at) => (at + 1, true),
                None => (want, false),
            };
            out.extend_from_slice(&available[..take]);
            self.reader.consume(take);
            if newline {
                break;
            }
        }
        !out.is_empty()
    }

    /// `vim_fgets`: one line of at most `size - 1` bytes, the rest of a
    /// longer one read and thrown away. `false` at the end of the file.
    pub(crate) fn line_capped(&mut self, line: &mut Vec<u8>, size: usize) -> bool {
        if !self.fgets(line, size) {
            return false;
        }
        // The last-but-one byte tells whether the line fitted: `fgets`
        // leaves it alone when the line was shorter than the buffer.
        let filled = |line: &[u8], at: usize| line.get(at).is_some_and(|&c| c != 0 && c != b'\n');
        if filled(line, size - 2) {
            // Throw away the rest of the line.
            let mut rest = Vec::new();
            while self.fgets(&mut rest, 200) && filled(&rest, 200 - 2) {}
        }
        true
    }

    /// Whether reading failed part-way: `ferror()`.
    pub(crate) fn had_error(&self) -> bool {
        self.error
    }
}

/// Read `len` bytes and turn them into an integer, most significant byte
/// first; `None` at the end of the file.
pub(crate) fn read_be(reader: &mut impl Read, len: usize) -> Option<u64> {
    let mut bytes = [0u8; 8];
    reader.read_exact(&mut bytes[..len]).ok()?;
    Some(bytes[..len].iter().fold(0, |n, &b| (n << 8) | u64::from(b)))
}

/// Read a string of `len` bytes; `None` when the file ends first.
pub(crate) fn read_counted(reader: &mut impl Read, len: usize) -> Option<Vec<u8>> {
    let mut bytes = vec![0; len];
    reader.read_exact(&mut bytes).ok()?;
    Some(bytes)
}

/// Write `number` in `len` bytes, most significant byte first: whether it
/// all went out.
pub(crate) fn write_be(writer: &mut impl Write, number: u64, len: usize) -> bool {
    debug_assert!((1..=8).contains(&len));
    writer.write_all(&number.to_be_bytes()[8 - len..]).is_ok()
}

/// Write a `time_t` in 8 bytes, most significant first: whether it went out.
pub(crate) fn write_time(writer: &mut impl Write, time: time_t) -> bool {
    writer.write_all(&time.to_be_bytes()).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integers_are_big_endian_and_stop_at_the_end() {
        let mut input: &[u8] = &[0x12, 0x34, 0x56, 0x78, 0x9a];
        assert_eq!(read_be(&mut input, 2), Some(0x1234));
        assert_eq!(read_be(&mut input, 3), Some(0x56789a));
        assert_eq!(read_be(&mut input, 1), None);

        let mut out = Vec::new();
        assert!(write_be(&mut out, 0x0102_0304, 3));
        assert!(write_time(&mut out, -2));
        assert_eq!(
            out,
            [2, 3, 4, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe]
        );
        let mut back: &[u8] = &out[3..];
        assert_eq!(read_be(&mut back, 8).map(u64::cast_signed), Some(-2));
    }

    #[test]
    fn counted_strings_need_every_byte() {
        let mut input: &[u8] = b"abcde";
        assert_eq!(read_counted(&mut input, 3).as_deref(), Some(&b"abc"[..]));
        assert_eq!(read_counted(&mut input, 3), None);
    }

    #[test]
    fn a_capped_line_throws_the_rest_away() {
        let long = [b'x'; 300];
        let mut text = long.to_vec();
        text.extend_from_slice(b"\nnext\n");
        let mut lines = Fgets::over(&text[..]);
        let mut line = Vec::new();
        assert!(lines.line_capped(&mut line, 10));
        assert_eq!(line, b"xxxxxxxxx");
        assert!(lines.line_capped(&mut line, 10));
        assert_eq!(line, b"next\n");
        assert!(!lines.line_capped(&mut line, 10));
        assert!(!lines.had_error());
    }
}
