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

use crate::global_cell::GlobalCell;
use crate::regexp::state::OPTION_MAGIC_NOT_SET;
use crate::types::{ColNr, LineNr, OptMagic};
use core::ffi::c_int;

pub(crate) static highlight_match: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static search_match_lines: GlobalCell<LineNr> = GlobalCell::new(0);
pub(crate) static search_match_endcol: GlobalCell<ColNr> = GlobalCell::new(0);
pub(crate) static search_first_line: GlobalCell<LineNr> = GlobalCell::new(0 as LineNr);
/// `MAXLNUM`, spelled as the type's own maximum: the two are the same
/// number, and saying it this way needs no cast.
pub(crate) static search_last_line: GlobalCell<LineNr> = GlobalCell::new(LineNr::MAX);
pub(crate) static no_smartcase: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static searchcmdlen: GlobalCell<c_int> = GlobalCell::new(0);
pub(crate) static no_hlsearch: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static magic_overruled: GlobalCell<OptMagic> = GlobalCell::new(OPTION_MAGIC_NOT_SET);
