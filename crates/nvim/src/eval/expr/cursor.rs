//! The text an expression is parsed from, and how far the parser has got.
//!
//! A [`Cursor`] borrows the expression's bytes and carries an offset into
//! them. Every level of the descent takes it `&mut`, reads the byte under it
//! and moves it on past what it consumed; a nested evaluation — a function
//! call, an autocommand, Lua — builds a cursor of its own, so nothing up the
//! stack is aliased while it runs.
//!
//! The slice does **not** include the C string's terminator, and reading at
//! or past its end answers `NUL`. That keeps every `== NUL` test the parser
//! inherited meaning "the expression is over", and it is also what lets a
//! cursor run over a piece of a line (`text[..end]`) that has no terminator
//! of its own, where the pointer form had to write one into the line.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr::byte_at;

/// A position in an expression's text.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Cursor<'a> {
    text: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    /// A cursor at the start of `text`.
    pub(crate) const fn new(text: &'a [u8]) -> Self {
        Self { text, offset: 0 }
    }

    /// The whole text, from its start — not from the cursor.
    pub(crate) const fn text(&self) -> &'a [u8] {
        self.text
    }

    /// How many bytes of the text are behind the cursor.
    pub(crate) const fn offset(&self) -> usize {
        self.offset
    }

    /// The byte under the cursor; `NUL` at the end.
    pub(crate) fn byte(&self) -> u8 {
        self.at(0)
    }

    /// The byte `i` past the cursor; `NUL` at or past the end.
    pub(crate) fn at(&self, i: usize) -> u8 {
        byte_at(self.text, self.offset.saturating_add(i))
    }

    /// What is left of the text from the cursor on: the "rest of the line"
    /// an error message quotes. Empty at or past the end.
    pub(crate) fn rest(&self) -> &'a [u8] {
        self.text.get(self.offset..).unwrap_or_default()
    }

    /// Step `n` bytes on.
    pub(crate) fn bump(&mut self, n: usize) {
        self.offset += n;
    }

    /// Step past any spaces and tabs.
    pub(crate) fn skip_white(&mut self) {
        while matches!(self.byte(), b' ' | b'\t') {
            self.offset += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_nul_at_and_past_the_end() {
        let mut cursor = Cursor::new(b"ab");
        assert_eq!((cursor.byte(), cursor.at(1), cursor.at(2)), (b'a', b'b', 0));
        assert_eq!(cursor.at(usize::MAX), 0);
        cursor.bump(2);
        assert_eq!(cursor.byte(), 0);
        assert_eq!(cursor.rest(), b"");
        cursor.bump(5);
        assert_eq!((cursor.byte(), cursor.rest()), (0, &b""[..]));
    }

    #[test]
    fn an_empty_text_is_at_its_end() {
        let cursor = Cursor::new(b"");
        assert_eq!((cursor.byte(), cursor.rest()), (0, &b""[..]));
    }

    #[test]
    fn skip_white_steps_over_spaces_and_tabs_only() {
        let mut cursor = Cursor::new(b"+ \t x\n");
        cursor.bump(1);
        cursor.skip_white();
        assert_eq!((cursor.offset(), cursor.byte()), (4, b'x'));
        cursor.bump(1);
        cursor.skip_white();
        assert_eq!(cursor.byte(), b'\n');
        let mut blank = Cursor::new(b"   ");
        blank.skip_white();
        assert_eq!((blank.offset(), blank.rest()), (3, &b""[..]));
    }

    #[test]
    fn rest_is_the_text_from_the_cursor_on() {
        let mut cursor = Cursor::new(b"abc def");
        assert_eq!(cursor.rest(), b"abc def");
        cursor.bump(3);
        assert_eq!(cursor.rest(), b" def");
        assert_eq!(cursor.text(), b"abc def");
    }

    #[test]
    fn an_interior_nul_is_an_ordinary_byte() {
        let mut cursor = Cursor::new(b"a\0b");
        cursor.bump(1);
        assert_eq!(
            (cursor.byte(), cursor.at(1), cursor.rest()),
            (0, b'b', &b"\0b"[..])
        );
    }
}
