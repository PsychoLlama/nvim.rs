//! The byte-level readers and writers binary formats and line files share,
//! over any `Read`/`BufRead`/`Write`.
//!
//! [`Fgets`] is C's `fgets` -- one line, or as much of it as fits -- with
//! [`Fgets::line_capped`] the editor's `vim_fgets`, which throws the rest of
//! an over-long line away.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use std::fs::File;
use std::io::{BufRead, BufReader, ErrorKind};

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

#[cfg(test)]
mod tests {
    use super::*;

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
