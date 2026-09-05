//! The popup menu's own grid, and what the editor wants it to show next.
//!
//! `pum_want` is a request rather than a state: completion writes the item
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
#![allow(non_upper_case_globals)]

use crate::global_cell::GlobalCell;
use crate::types::ScreenGrid;
use core::ffi::c_int;

#[derive(Copy, Clone)]
pub(crate) struct PumWant {
    pub active: bool,
    pub item: c_int,
    pub insert: bool,
    pub finish: bool,
}
pub(crate) static must_redraw_pum: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static pum_grid: GlobalCell<ScreenGrid> = GlobalCell::new(ScreenGrid::empty());
pub(crate) static pum_want: GlobalCell<PumWant> = GlobalCell::new(PumWant {
    active: false,
    item: 0,
    insert: false,
    finish: false,
});
