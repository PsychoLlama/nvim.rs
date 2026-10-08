//! Scaffolding shared by the VimL parsers: the reader that feeds them lines,
//! the cursor over those lines, and the highlight log they append to.
//!
//! Upstream reads its input through a getter callback and a cookie, keeps
//! every line it has read in a `kvec` (tokens and nodes point into them), and
//! can convert each one between encodings. Every caller in the tree hands
//! over a ready-made array of lines and asks for no conversion, so the input
//! here is that array, borrowed for the parse: a line the reader has read is
//! a line of the caller's, and the tree can borrow it too.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::CStr;

use crate::types::{ParserHighlightChunk, ParserPosition, ParserState};

impl<'a> ParserState<'a> {
    /// A parser over `input`, which has read nothing yet. With `highlight`
    /// it logs how every token is highlighted.
    pub fn new(input: &'a [&'a [u8]], highlight: bool) -> Self {
        ParserState {
            input,
            lines_read: 0,
            pos: ParserPosition { line: 0, col: 0 },
            colors: highlight.then(Vec::new),
        }
    }

    /// How many lines the reader has handed out, counting the absent one
    /// past the end once the parse has asked for it.
    pub fn lines_read(&self) -> usize {
        self.lines_read
    }

    /// The `index`th line the reader has read; `None` for the absent line
    /// past the end of the input.
    pub fn line(&self, index: usize) -> Option<&'a [u8]> {
        debug_assert!(
            index < self.lines_read,
            "the parser has not read that many lines"
        );
        self.input.get(index).copied()
    }

    /// The rest of the current line, from the cursor on, reading one more
    /// line from the input if the cursor just walked off the end of the last
    /// one. `None` at end of input.
    pub fn remaining_line(&mut self) -> Option<&'a [u8]> {
        if self.pos.line == self.lines_read {
            self.lines_read += 1;
        }
        debug_assert!(self.pos.line == self.lines_read - 1);
        let line = self.line(self.lines_read - 1)?;
        // The cursor is never past the line's end: `advance` wraps to the
        // next line first.
        Some(&line[self.pos.col..])
    }

    /// Advance the cursor by `len` bytes, at most to the start of the next
    /// line.
    pub fn advance(&mut self, len: usize) {
        debug_assert!(self.pos.line == self.lines_read - 1);
        let size = self.line(self.lines_read - 1).map_or(0, <[u8]>::len);
        if self.pos.col.wrapping_add(len) >= size {
            self.pos.line += 1;
            self.pos.col = 0;
        } else {
            self.pos.col = self.pos.col.wrapping_add(len);
        }
    }

    /// Record the highlighting of `len` bytes at `start`. A no-op when the
    /// caller asked for no highlighting. Chunks must arrive in order and must
    /// not overlap.
    pub fn highlight(&mut self, start: ParserPosition, len: usize, group: &'static CStr) {
        let Some(colors) = &mut self.colors else {
            return;
        };
        if len == 0 {
            return;
        }
        debug_assert!(
            colors
                .last()
                .is_none_or(|last| last.start.line < start.line || last.end_col <= start.col),
            "highlight chunks must be recorded in order"
        );
        colors.push(ParserHighlightChunk {
            start,
            end_col: start.col.wrapping_add(len),
            group,
        });
    }

    /// How many highlight chunks have been recorded so far; `None` when the
    /// caller asked for no highlighting.
    pub fn highlight_count(&self) -> Option<usize> {
        self.colors.as_ref().map(Vec::len)
    }

    /// Rewrite the group of a chunk already recorded. A no-op without
    /// highlighting.
    pub fn recolour(&mut self, index: usize, group: &'static CStr) {
        if let Some(colors) = &mut self.colors {
            colors[index].group = group;
        }
    }

    /// The highlight log, taken out of the parser: empty when the caller
    /// asked for none.
    pub fn take_highlight(&mut self) -> Vec<ParserHighlightChunk> {
        self.colors.take().unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The cursor stops at the start of the next line rather than running off
    /// the end of this one, and a `len` that lands exactly on the end still
    /// wraps.
    #[test]
    fn advance_wraps_to_the_next_line() {
        let lines: [&[u8]; 1] = [b"abcd"];
        let mut pstate = ParserState::new(&lines, false);
        assert_eq!(pstate.remaining_line(), Some(&b"abcd"[..]));

        pstate.advance(2);
        assert_eq!((pstate.pos.line, pstate.pos.col), (0, 2));
        assert_eq!(pstate.remaining_line(), Some(&b"cd"[..]));
        pstate.advance(2);
        assert_eq!((pstate.pos.line, pstate.pos.col), (1, 0));
        // Past the last line there is nothing more, however often it asks.
        assert_eq!(pstate.remaining_line(), None);
        assert_eq!(pstate.remaining_line(), None);
    }
}
