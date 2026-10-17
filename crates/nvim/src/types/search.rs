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

#[derive(Copy, Clone)]
pub struct SearchOffset {
    pub dir: ::core::ffi::c_char,
    pub line: bool,
    pub end: bool,
    pub off: int64_t,
}
/// What a caller of `searchit` may add: a line to stop after, a deadline,
/// and the two answers -- whether the deadline was passed, and whether the
/// search wrapped around the end of the buffer.
#[derive(Default)]
pub struct SearchItArg<'a> {
    /// Stop after this line, when it is not zero.
    pub sa_stop_lnum: LineNr,
    /// Give up at this time.
    pub sa_tm: Option<&'a ProfTime>,
    /// Set when `sa_tm` was passed.
    pub sa_timed_out: bool,
    /// Set when the search wrapped around.
    pub sa_wrapped: bool,
}
