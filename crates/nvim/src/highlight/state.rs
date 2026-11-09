//! The attribute numbers the drawing code paints with.
//!
//! `HIGHLIGHT_ATTR` is the resolved attribute for each `HLF_*` slot, which
//! every `win_line` decision indexes; `HIGHLIGHT_USER` and
//! `HIGHLIGHT_STLNC` are the same for the nine `User1`..`User9` groups and
//! their not-current-window variants. The `normal_*`/`cterm_normal_*` pair
//! is what `Normal` resolved to, which the UI needs separately because it is
//! the screen's background rather than a run's attribute.
//!
//! The `ns_hl_*` cells say which highlight namespace is in force —
//! globally, for the window being drawn, and for the fast path a decoration
//! provider runs on — and `NEED_HIGHLIGHT_CHANGED` is the "recompute all of
//! the above" flag `:highlight` and `:colorscheme` raise.
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
use crate::highlight::RgbValue;
use crate::types::Handle;
use crate::types::NS;
use core::ffi::c_int;

pub(crate) static INCLUDE_NONE: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static INCLUDE_DEFAULT: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static INCLUDE_LINK: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static NEED_HIGHLIGHT_CHANGED: GlobalCell<bool> = GlobalCell::new(true);
pub(crate) static HIGHLIGHT_ATTR: GlobalCell<[c_int; 76]> = GlobalCell::new([0; 76]);
pub(crate) static HIGHLIGHT_ATTR_LAST: GlobalCell<[c_int; 76]> = GlobalCell::new([0; 76]);
pub(crate) static HIGHLIGHT_USER: GlobalCell<[c_int; 9]> = GlobalCell::new([0; 9]);
pub(crate) static HIGHLIGHT_STLNC: GlobalCell<[c_int; 9]> = GlobalCell::new([0; 9]);
pub(crate) static CTERM_NORMAL_FG_COLOR: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static CTERM_NORMAL_BG_COLOR: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static NORMAL_FG: GlobalCell<RgbValue> = GlobalCell::new(-1 as RgbValue);
pub(crate) static NORMAL_BG: GlobalCell<RgbValue> = GlobalCell::new(-1 as RgbValue);
pub(crate) static NORMAL_SP: GlobalCell<RgbValue> = GlobalCell::new(-1 as RgbValue);
pub(crate) static NS_HL_GLOBAL: GlobalCell<NS> = GlobalCell::new(Handle::NONE);
pub(crate) static NS_HL_WIN: GlobalCell<NS> = GlobalCell::new(Handle::new(-1));
pub(crate) static NS_HL_FAST: GlobalCell<NS> = GlobalCell::new(Handle::new(-1));
pub(crate) static NS_HL_ACTIVE: GlobalCell<NS> = GlobalCell::new(Handle::NONE);
/// One `HLF_*` attribute table: the built-in [`HIGHLIGHT_ATTR`] or a
/// namespace's. None is ever freed, which is what lets them be `'static`.
pub(crate) type HlAttrTable = GlobalCell<[c_int; 76]>;
/// The table in force, what `HL_ATTR` reads: [`HIGHLIGHT_ATTR`] or the
/// active namespace's. A reference rather than a namespace id resolved per
/// read, because the drawing code reads it per cell: the resolving branch
/// cost scrbench 0.34 %.
pub(crate) static HL_ATTR_ACTIVE: GlobalCell<&'static HlAttrTable> =
    GlobalCell::new(&HIGHLIGHT_ATTR);
