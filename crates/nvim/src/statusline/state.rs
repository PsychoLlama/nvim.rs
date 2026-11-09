//! What a click on the drawn status or tab line means.
//!
//! Building the line records, per screen cell, what was under it
//! (`TAB_PAGE_CLICK_DEFS`), so a later mouse report can be turned back into
//! the tab or the `%@Func@` region it landed on. `STL_SYNTAX` says the line
//! being built is being parsed rather than drawn.
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
use crate::option::defs::StlSyntax;
use crate::statusline::defs::ClickLine;

pub(crate) static STL_SYNTAX: GlobalCell<StlSyntax> = GlobalCell::new(StlSyntax::NONE);
/// The tab page line's click definitions, one per screen column; sized by
/// the screen resize.
pub(crate) static TAB_PAGE_CLICK_DEFS: GlobalCell<ClickLine> = GlobalCell::new(ClickLine {
    cells: Vec::new(),
    funcs: Vec::new(),
});
