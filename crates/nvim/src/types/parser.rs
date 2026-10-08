#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

/// One highlighted stretch of a line: `[start, end_col)` on `start.line`, in
/// the group the parser named.
#[derive(Copy, Clone, Debug)]
pub struct ParserHighlightChunk {
    pub start: ParserPosition,
    pub end_col: size_t,
    pub group: &'static ::core::ffi::CStr,
}

/// A line and column inside a parsed string.
///
/// `Copy`: a position is a value.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ParserPosition {
    pub line: size_t,
    pub col: size_t,
}

/// What a VimL parser reads and where it has got to: the input lines, the
/// cursor, and the highlight log when the caller asked for one.
///
/// The lines are the caller's and are borrowed for the whole parse, which is
/// what lets the tree and the error point into them; a parse ends at the
/// first line past the last one, as upstream's reader ended at a null line.
pub struct ParserState<'a> {
    /// Every line of the input, in order.
    pub(crate) input: &'a [&'a [u8]],
    /// How many lines the reader has handed out, counting the absent one
    /// past the end once the parse has asked for it.
    pub(crate) lines_read: usize,
    pub pos: ParserPosition,
    /// The highlight log; `None` when the caller wanted none.
    pub(crate) colors: Option<Vec<ParserHighlightChunk>>,
}
