//! What the last search left behind.
//!
//! `HIGHLIGHT_MATCH` and the `search_match_*` pair are the range `:s` and
//! incsearch ask the drawing code to light up; `SEARCH_FIRST_LINE` and
//! `SEARCH_LAST_LINE` bound the incsearch preview. `NO_HLSEARCH` is
//! `'hlsearch'` switched off for this search only (what `:nohlsearch` sets),
//! `NO_SMARTCASE` the same for `'smartcase'`, and `MAGIC_OVERRULED` records
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

use crate::global_cell::GlobalCell;
use crate::os::libc::int64_t;
use crate::regexp::{OPTION_MAGIC_NOT_SET, OptMagic};
use crate::types::ProfTime;
use crate::types::{ColNr, LineNr};
use core::ffi::c_int;

pub(crate) static HIGHLIGHT_MATCH: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static SEARCH_MATCH_LINES: GlobalCell<i32> = GlobalCell::new(0);
pub(crate) static SEARCH_MATCH_ENDCOL: GlobalCell<ColNr> = GlobalCell::new(ColNr::ZERO);
pub(crate) static SEARCH_FIRST_LINE: GlobalCell<LineNr> = GlobalCell::new(LineNr::ZERO);
/// `MAXLNUM`, spelled as the type's own maximum: the two are the same
/// number, and saying it this way needs no cast.
pub(crate) static SEARCH_LAST_LINE: GlobalCell<LineNr> = GlobalCell::new(LineNr::MAX);
pub(crate) static NO_SMARTCASE: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static SEARCHCMDLEN: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static NO_HLSEARCH: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static MAGIC_OVERRULED: GlobalCell<OptMagic> = GlobalCell::new(OPTION_MAGIC_NOT_SET);

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
