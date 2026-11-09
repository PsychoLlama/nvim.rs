//! The popup menu's own grid, and what the editor wants it to show next.
//!
//! `PUM_WANT` is a request rather than a state: completion writes the item
//! it would like selected and whether to insert it, and the menu applies
//! that on its next redraw.
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
use crate::grid::defs::ScreenGrid;
use core::ffi::c_int;

#[derive(Copy, Clone, Debug)]
pub(crate) struct PumWant {
    pub active: bool,
    pub item: c_int,
    pub insert: bool,
    pub finish: bool,
}
pub(crate) static MUST_REDRAW_PUM: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static PUM_GRID: GlobalCell<ScreenGrid> = GlobalCell::new(ScreenGrid::empty());
pub(crate) static PUM_WANT: GlobalCell<PumWant> = GlobalCell::new(PumWant {
    active: false,
    item: 0,
    insert: false,
    finish: false,
});

/// One row of the popup menu, owning its text.
///
/// Upstream's rows point into the strings of whoever built them -- the
/// completion's matches, the command line's expansion, a menu's entries --
/// and the menu has to come down before any of those is freed. Here a row
/// owns copies, so the menu's array (an `Rc<[PumItem]>` shared with the
/// producer) stays good whatever the producer does next. `pum_text` is the
/// one column every row has; the other three are absent rather than empty.
#[derive(Default, Clone, Debug)]
pub struct PumItem {
    pub pum_text: crate::memory::XString,
    pub pum_kind: Option<crate::memory::XString>,
    pub pum_extra: Option<crate::memory::XString>,
    pub pum_info: Option<crate::memory::XString>,
    pub pum_cpt_source_idx: ::core::ffi::c_int,
    pub pum_user_abbr_hlattr: ::core::ffi::c_int,
    pub pum_user_kind_hlattr: ::core::ffi::c_int,
}
