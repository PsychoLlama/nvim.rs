//! What a click on the drawn status or tab line means.
//!
//! Building the line records, per screen cell, what was under it
//! (`tab_page_click_defs`), so a later mouse report can be turned back into
//! the tab or the `%@Func@` region it landed on. `stl_syntax` says the line
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
#![allow(non_upper_case_globals)]

use crate::global_cell::GlobalCell;
use crate::types::{StlClickDefinition, StlSyntax};

pub(crate) static stl_syntax: GlobalCell<StlSyntax> = GlobalCell::new(StlSyntax::NONE);
/// The tab page line's click definitions, one per screen column; sized by
/// the screen resize. Each `func` string is owned by the run of cells that
/// holds it (see `ClickArena`).
pub(crate) static tab_page_click_defs: GlobalCell<Vec<StlClickDefinition>> =
    GlobalCell::new(Vec::new());
