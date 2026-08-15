#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

/// Which of the two charsize functions a line needs. `init_charsize_arg`
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
#[derive(Copy, Clone, Default)]
pub struct CharSize {
    pub width: ::core::ffi::c_int,
    pub head: ::core::ffi::c_int,
}
/// The per-line state the charsize functions walk with.
///
/// Zero is not a meaningful value for any of these: every field is set by
/// `init_charsize_arg` before the first character is measured. `Default`
/// exists only so callers can declare one without spelling out the marktree
/// iterator, which is what c2rust made them do.
#[derive(Copy, Clone, Default)]
pub struct CharsizeArg {
    pub win: *mut win_T,
    pub line: *mut ::core::ffi::c_char,
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
    pub iter: [MarkTreeIter; 1],
}
