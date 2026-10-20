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
use crate::winlayer::Win;

/// Which of the two charsize functions a line needs. `CharsizeArg::new`
/// decides once per line and every walk over that line uses the answer.
///
/// `Fast` is the common case: nothing on the line can change a character's
/// width beyond the tabstop, so `charsize_fast` -- which is inlined into its
/// callers -- suffices. `Regular` means inline virtual text, 'linebreak',
/// 'breakindent' or 'showbreak' is in play and every character has to go
/// through `charsize_regular`.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum CharsizeKind {
    Regular = 0,
    Fast = 1,
}
#[derive(Default)]
pub struct CharSize {
    pub width: ::core::ffi::c_int,
    pub head: ::core::ffi::c_int,
}
/// The per-line state the charsize functions walk with, seated on the line
/// it measures.
///
/// `line` is the text without its NUL: a walk reads a NUL past its end, and
/// stops at a NUL inside it, exactly as the pointer walk it replaces
/// stopped at the terminator. Positions in it are byte offsets.
pub struct CharsizeArg<'a> {
    /// The window being measured in, taken live by `CharsizeArg::new`; the
    /// walk is one call that nothing in it can close a window during.
    pub win: Win,
    pub line: &'a [u8],
    pub use_tabstop: bool,
    /// Width of 'showbreak' plus 'breakindent', memoised across the line;
    /// `c_int::MIN` until the first character needs it.
    pub indent_width: ::core::ffi::c_int,
    /// Row the inline-virtual-text iterator is positioned on, or -1 when the
    /// line has no inline virtual text.
    pub virt_row: ::core::ffi::c_int,
    pub cur_text_width_left: ::core::ffi::c_int,
    pub cur_text_width_right: ::core::ffi::c_int,
    pub max_head_vcol: ::core::ffi::c_int,
    /// The inline-virtual-text walk, positioned on `virt_row`; `None` when
    /// the line has no inline virtual text. Boxed because it is the rare
    /// case and a `MarkTreeIter` is ~200 bytes: inline, every walk state
    /// paid two copies of it on the way out of its constructor.
    pub iter: Option<Box<MarkTreeIter>>,
}
