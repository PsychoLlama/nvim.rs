//! What the last search left behind.
//!
//! `highlight_match` and the `search_match_*` pair are the range `:s` and
//! incsearch ask the drawing code to light up; `search_first_line` and
//! `search_last_line` bound the incsearch preview. `no_hlsearch` is
//! `'hlsearch'` switched off for this search only (what `:nohlsearch` sets),
//! `no_smartcase` the same for `'smartcase'`, and `magic_overruled` records
//! that the pattern itself said `\v` or `\V`.
#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::global_cell::GlobalCell;
use crate::os::libc::int64_t;
use crate::regexp::OptMagic;
use crate::regexp::state::OPTION_MAGIC_NOT_SET;
use crate::types::ProfTime;
use crate::types::{ColNr, LineNr};
use core::ffi::c_int;

pub(crate) static highlight_match: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static search_match_lines: GlobalCell<i32> = GlobalCell::new(0);
pub(crate) static search_match_endcol: GlobalCell<ColNr> = GlobalCell::new(ColNr::ZERO);
pub(crate) static search_first_line: GlobalCell<LineNr> = GlobalCell::new(LineNr::ZERO);
/// `MAXLNUM`, spelled as the type's own maximum: the two are the same
/// number, and saying it this way needs no cast.
pub(crate) static search_last_line: GlobalCell<LineNr> = GlobalCell::new(LineNr::MAX);
pub(crate) static no_smartcase: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static searchcmdlen: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static no_hlsearch: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static magic_overruled: GlobalCell<OptMagic> = GlobalCell::new(OPTION_MAGIC_NOT_SET);

#[derive(Copy, Clone, Debug)]
pub(crate) struct SearchOffset {
    pub dir: ::core::ffi::c_char,
    pub line: bool,
    pub end: bool,
    pub off: int64_t,
}
/// What a caller of `searchit` may add: a line to stop after, a deadline,
/// and the two answers -- whether the deadline was passed, and whether the
/// search wrapped around the end of the buffer.
#[derive(Default, Debug)]
pub(crate) struct SearchItArg<'a> {
    /// Stop after this line, when it is not zero.
    pub sa_stop_lnum: LineNr,
    /// Give up at this time.
    pub sa_tm: Option<&'a ProfTime>,
    /// Set when `sa_tm` was passed.
    pub sa_timed_out: bool,
    /// Set when the search wrapped around.
    pub sa_wrapped: bool,
}
